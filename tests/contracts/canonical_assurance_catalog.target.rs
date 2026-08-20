//! Target suite for Canonical Assurance Catalog v1 infrastructure (CAT-001…016)
//! plus Prompt 2 trust-boundary law
//! (`docs/specs/catalog-framework-readiness-trust-boundary.md` §4 / §5 / §7.2).
//!
//! CAT-001…016 stay the landed catalog-infrastructure gate. Increment-2
//! `cat_ssot_t*` / `frw_*` / `pin_t*` / `rdy_t*` encode DESIRED behavior that
//! must stay RED on CURRENT code (second catalog parser, dropped expressions,
//! best-effort pack parse, non-semantic digest, live pin reload, forked
//! readiness). Do not implement the feature in this suite and do not weaken
//! these assertions to match today's leaky seams.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::Parser;
use serde_json::json;
use weeping_angel::cli::Cli;
use weeping_angel_assurance::readiness::{FrameworkReadinessSnapshot, project_readiness};
use weeping_angel_assurance::snapshot::{AssessmentRun, catalog_digest};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentId, Control, ControlId, EvidenceRequirement, EvidenceType,
    Mapping, MappingCompleteness, MappingDirection, MappingRelation, PlannedControlTest,
    Requirement,
};
use weeping_angel_canonical_catalog::{CanonicalCatalog, CatalogError};
use weeping_angel_control_test::{ControlTestResult, EvidenceSelector, TestExpr};
use weeping_angel_framework::pack::{FRAMEWORK_PACK_SCHEMA, PackError};
use weeping_angel_framework::{
    CatalogProjection, FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
    assessment_from_pack, compile_framework, load_framework_pack, load_framework_pack_from,
    load_framework_pack_from_with,
};

const CATALOG_SCHEMA: &str = "weeping-angel/canonical-catalog/v1";
const DIGEST_PREFIX: &str = "wa:canonical-catalog:weeping-angel/canonical-catalog/v1:";
const IR_DIGEST_PREFIX: &str = "wa:assurance-ir:";

const PINNED_CONTROL: &str = "control.source.protected-branch";
const PINNED_EVIDENCE: &str = "evidence.source.protected-branch";
const PINNED_TEST: &str = "test.source.protected-branch";

const PROVIDER_SEGMENTS: &[&str] = &[
    "github",
    "gitlab",
    "bitbucket",
    "aws",
    "azure",
    "gcp",
    "google",
    "cloudflare",
    "vercel",
    "okta",
    "entra",
    "auth0",
    "workspace",
];
const FRAMEWORK_SEGMENTS: &[&str] = &[
    "iso27001",
    "iso27701",
    "iso27007",
    "soc2",
    "nis2",
    "dora",
    "gdpr",
    "iso-27001",
    "iso-27701",
    "iso-27007",
    "soc-2",
    "nis-2",
];

const FORBIDDEN_CATALOG_DEPS: &[&str] = &[
    "weeping-angel-framework",
    "weeping-angel-collector",
    "weeping-angel-control-test",
    "weeping-angel-evidence",
    "reqwest",
    "hyper",
    "octocrab",
    "octorust",
    "aws-sdk-",
    "cloudflare",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn catalog_crate_dir() -> PathBuf {
    manifest_dir().join("crates/weeping-angel-canonical-catalog")
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_files(dir, out);
    out.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"));
}

fn crate_sources_joined(name: &str) -> String {
    let src = manifest_dir().join("crates").join(name).join("src");
    assert!(
        src.is_dir(),
        "expected crate sources at {} (dedicated catalog crate is required)",
        src.display()
    );
    let mut files = Vec::new();
    walk_rs_files(&src, &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_crate_toml(name: &str) -> String {
    fs::read_to_string(manifest_dir().join("crates").join(name).join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("read {name} Cargo.toml: {e}"))
}

fn require_shipped_catalog() -> PathBuf {
    let root = catalog_v1();
    assert!(
        root.join("manifest.toml").is_file(),
        "CAT-001: catalog/canonical/v1/manifest.toml must exist"
    );
    for section in ["controls", "evidence", "tests"] {
        assert!(
            root.join(section).is_dir(),
            "CAT-001: catalog/canonical/v1/{section}/ must exist"
        );
    }
    root
}

fn first_toml(dir: &Path) -> PathBuf {
    let fixture = dir.join("fixture.example.toml");
    if fixture.is_file() {
        return fixture;
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    files.sort();
    files
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("expected at least one .toml under {}", dir.display()))
}

fn toml_containing(dir: &Path, needle: &str) -> PathBuf {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    files.sort();
    for path in &files {
        if fs::read_to_string(path).unwrap().contains(needle) {
            return path.clone();
        }
    }
    panic!(
        "expected a .toml under {} containing `{needle}`",
        dir.display()
    )
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), &to).unwrap();
        }
    }
}

fn copy_shipped_catalog() -> (tempfile::TempDir, PathBuf) {
    let src = require_shipped_catalog();
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("v1");
    copy_dir(&src, &dest);
    (tmp, dest)
}

fn output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{stdout}{stderr}")
}

fn run_catalog_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_weeping-angel"))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn weeping-angel {args:?}: {e}"))
}

fn validate_catalog(path: &Path) -> Output {
    let path = path.to_string_lossy();
    run_catalog_cli(&["assurance", "catalog", "validate", path.as_ref()])
}

fn stats_catalog(path: &Path) -> Output {
    let path = path.to_string_lossy();
    run_catalog_cli(&["assurance", "catalog", "stats", path.as_ref()])
}

fn inspect_control(control_id: &str, path: Option<&Path>) -> Output {
    match path {
        Some(path) => {
            let path = path.to_string_lossy();
            run_catalog_cli(&["assurance", "catalog", "inspect", control_id, path.as_ref()])
        }
        None => run_catalog_cli(&["assurance", "catalog", "inspect", control_id]),
    }
}

fn assert_cli_failure_mentions(output: &Output, needles: &[&str], context: &str) {
    assert!(
        !output.status.success(),
        "{context}: validator must fail closed (exit non-zero); stdout+stderr:\n{}",
        output_text(output)
    );
    let text = output_text(output).to_ascii_lowercase();
    assert!(
        !text.contains("unrecognized subcommand"),
        "{context}: failure must come from the catalog validator, not a missing CLI surface; got:\n{}",
        output_text(output)
    );
    let hit = needles
        .iter()
        .any(|n| text.contains(&n.to_ascii_lowercase()));
    assert!(
        hit,
        "{context}: expected one of {needles:?} in validator output, got:\n{}",
        output_text(output)
    );
}

fn extract_digest(output: &Output) -> String {
    assert!(
        output.status.success(),
        "stats must succeed to expose the digest; got:\n{}",
        output_text(output)
    );
    let text = output_text(output);
    assert!(
        text.contains(DIGEST_PREFIX),
        "digest must be domain-separated with `{DIGEST_PREFIX}`; got:\n{text}"
    );
    assert!(
        !text.contains(IR_DIGEST_PREFIX),
        "catalog digest must not reuse `{IR_DIGEST_PREFIX}`; got:\n{text}"
    );
    text.lines()
        .flat_map(|line| line.split_whitespace())
        .find(|tok| tok.starts_with(DIGEST_PREFIX))
        .map(|s| {
            s.trim_matches(|c| c == '"' || c == ',' || c == '\'')
                .to_string()
        })
        .unwrap_or_else(|| panic!("stats must print the digest token; got:\n{text}"))
}

fn catalog_text() -> String {
    let root = require_shipped_catalog();
    let mut files = Vec::new();
    walk_files(&root, &mut files);
    files
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn replace_in(path: &Path, from: &str, to: &str) {
    let src = fs::read_to_string(path).unwrap();
    assert!(
        src.contains(from),
        "expected `{from}` in {} (cannot mutate fixture for fail-closed case)",
        path.display()
    );
    fs::write(path, src.replace(from, to)).unwrap();
}

fn crate_src_file(name: &str, rel: &str) -> String {
    fs::read_to_string(
        manifest_dir()
            .join("crates")
            .join(name)
            .join("src")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("read {name}/src/{rel}: {e}"))
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn write_trust_boundary_pack(dir: &Path, mapping_block: &str, metadata: &str) {
    fs::write(
        dir.join("manifest.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"

[framework]
id = "iso-27001"
version = "2022"
content_mode = "StructuralOnly"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("requirements.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"

[[requirement]]
id = "iso27001:a.8.5"
title = "Authentication (structural)"
kind = "annex"
"#,
    )
    .unwrap();
    fs::write(dir.join("metadata.toml"), metadata).unwrap();
    fs::write(
        dir.join("mappings.toml"),
        format!(
            r#"schema = "weeping-angel/framework-pack/v1"

{mapping_block}
"#
        ),
    )
    .unwrap();
}

fn honest_mapping_block() -> &'static str {
    r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "partial"
relation = "PartiallySatisfies"
rationale = "privileged MFA is a slice of authentication"
provenance = { source = "BuiltIn", reference = "catalog/canonical/v1", author = "weeping-angel" }
"#
}

fn workspace_catalog_projection() -> CatalogProjection {
    CanonicalCatalog::load(catalog_v1())
        .expect("workspace catalog")
        .projection()
        .expect("catalog projection")
}

fn load_isolated_pack(dir: &Path) -> Result<weeping_angel_framework::LoadedPack, PackError> {
    load_framework_pack_from_with(dir, Some(&workspace_catalog_projection()))
}

fn compile_iso_pack() -> weeping_angel_framework::CompiledFramework {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack must load for compile");
    let assessment = assessment_from_pack(&pack, &iso_target());
    compile_framework(&assessment, &iso_target()).expect("ISO pack must compile")
}

fn synthetic_effective(control: &str, test: &str) -> ControlTestResult {
    serde_json::from_value(json!({
        "testId": test,
        "controlId": control,
        "effectiveness": "effective",
        "rationale": "synthetic effective result"
    }))
    .expect("ControlTestResult JSON")
}

// ── CAT-016 / registration ─────────────────────────────────────────────────

#[test]
fn cat_016_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !toml.contains("sdd_canonical_assurance_catalog_baseline")
            && toml.contains("sdd_canonical_assurance_catalog_target")
            && !toml.contains("tests/contracts/canonical_assurance_catalog.baseline.rs")
            && toml.contains("tests/contracts/canonical_assurance_catalog.target.rs"),
        "dual-suite binaries must stay registered in root Cargo.toml"
    );
}

// ── CAT-001 / CAT-015 / content bound ──────────────────────────────────────

#[test]
fn cat_001_catalog_canonical_v1_tree_and_schema() {
    let root = require_shipped_catalog();
    let manifest = fs::read_to_string(root.join("manifest.toml")).unwrap();
    assert!(
        manifest.contains(&format!("schema = \"{CATALOG_SCHEMA}\""))
            || manifest.contains(&format!("schema = '{CATALOG_SCHEMA}'")),
        "manifest schema must be {CATALOG_SCHEMA}; got:\n{manifest}"
    );
    assert!(
        !manifest.contains("assurance-ir/v1")
            && !manifest.contains("weeping-angel/framework-pack/v1"),
        "canonical catalog must not reuse IR or framework-pack schema ids"
    );
}

#[test]
fn cat_015_fixture_ids_are_pinned() {
    let text = catalog_text();
    for id in [PINNED_CONTROL, PINNED_EVIDENCE, PINNED_TEST] {
        assert!(
            text.contains(id),
            "shipped fixture must keep public id `{id}` (accidental rename)"
        );
    }
}

#[test]
fn shipped_catalog_is_minimal_and_regime_free() {
    let text = catalog_text().to_ascii_lowercase();
    for needle in [
        "soc2",
        "soc 2",
        "soc-2",
        "nis2",
        "nis 2",
        "nis-2",
        "dora",
        "iso27001",
        "iso 27001",
        "iso-27001",
        "iso27701",
        "gdpr",
    ] {
        assert!(
            !text.contains(needle),
            "catalog/canonical/v1 must not ship {needle} normative content"
        );
    }
    let controls_dir = require_shipped_catalog().join("controls");
    let control_files: Vec<_> = fs::read_dir(&controls_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    assert!(
        control_files
            .iter()
            .any(|e| e.file_name() == "fixture.example.toml"),
        "catalog infrastructure fixture.example.toml must remain under catalog/canonical/v1/controls"
    );
    assert!(
        !control_files.is_empty(),
        "catalog/canonical/v1/controls must ship at least the catalog infrastructure fixture"
    );
}

// ── CAT-002 / CAT-014 / crate API ──────────────────────────────────────────

#[test]
fn cat_002_catalog_loads_offline() {
    let crate_dir = catalog_crate_dir();
    assert!(
        crate_dir.join("src").is_dir(),
        "CAT-002: crates/weeping-angel-canonical-catalog must exist"
    );
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        cargo.contains("weeping-angel-canonical-catalog"),
        "workspace / root package must list weeping-angel-canonical-catalog"
    );

    let src = crate_sources_joined("weeping-angel-canonical-catalog");
    for needle in [
        "CanonicalCatalog",
        "fn load",
        "fn validate",
        "fn digest",
        CATALOG_SCHEMA,
        DIGEST_PREFIX,
    ] {
        assert!(
            src.contains(needle),
            "CanonicalCatalog API / schema `{needle}` must exist in the catalog crate"
        );
    }
    for forbidden in [
        "reqwest",
        "octocrab",
        "std::net::",
        "TcpStream",
        "UdpSocket",
    ] {
        assert!(
            !src.contains(forbidden),
            "load/validate must perform zero network I/O; found `{forbidden}`"
        );
    }

    let root = require_shipped_catalog();
    let output = validate_catalog(&root);
    assert!(
        output.status.success(),
        "CanonicalCatalog::load/validate of the shipped fixture must succeed offline; got:\n{}",
        output_text(&output)
    );
}

#[test]
fn cat_014_catalog_crate_depends_only_on_ir_toml_serde_digest() {
    let toml = read_crate_toml("weeping-angel-canonical-catalog");
    assert!(
        toml.contains("weeping-angel-assurance-ir") && toml.contains("toml"),
        "catalog crate must depend on IR + toml"
    );
    for forbidden in FORBIDDEN_CATALOG_DEPS {
        assert!(
            !toml.contains(forbidden),
            "catalog crate must not depend on `{forbidden}`"
        );
    }
}

#[test]
fn loader_reads_manifest_file_list_not_hardcoded_fixture_names() {
    let src = crate_sources_joined("weeping-angel-canonical-catalog");
    assert!(
        !src.contains("fixture.example.toml"),
        "downstream must add TOML + a manifest entry without editing loader source"
    );
}

// ── CAT-003 digest ─────────────────────────────────────────────────────────

#[test]
fn cat_003_digest_is_deterministic_and_domain_separated() {
    let root = require_shipped_catalog();
    let first = extract_digest(&stats_catalog(&root));
    let second = extract_digest(&stats_catalog(&root));
    assert_eq!(
        first, second,
        "in-process reload must not change the digest"
    );
    assert!(
        first.starts_with(DIGEST_PREFIX),
        "digest must start with {DIGEST_PREFIX}"
    );
    assert!(
        !first.contains(IR_DIGEST_PREFIX),
        "catalog digest is domain-separated from assurance-ir/v1"
    );

    let (_tmp, copy) = copy_shipped_catalog();
    let control = first_toml(&copy.join("controls"));
    let raw = fs::read_to_string(&control).unwrap();
    // Comments / blank lines / key order in the TOML source must not change
    // the digest of parsed documents. Do not swap key *names* (that would
    // also swap parsed values).
    let shuffled = format!("\n# key-order / whitespace noise\n{raw}\n");
    fs::write(&control, shuffled).unwrap();
    let recopied = extract_digest(&stats_catalog(&copy));
    assert_eq!(
        first, recopied,
        "digest must be of parsed documents (TOML key order / whitespace must not change it)"
    );
}

// ── CAT-004…010 fail-closed validator ──────────────────────────────────────

#[test]
fn cat_004_duplicate_ids_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let controls = first_toml(&root.join("controls"));
    let mut body = fs::read_to_string(&controls).unwrap();
    body.push_str(&format!(
        "\n[[control]]\nid = \"{PINNED_CONTROL}\"\ntitle = \"duplicate\"\n"
    ));
    fs::write(&controls, body).unwrap();
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["duplicate"],
        "CAT-004 duplicate control id",
    );
}

#[test]
fn cat_005_dangling_references_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let controls = toml_containing(&root.join("controls"), PINNED_EVIDENCE);
    replace_in(&controls, PINNED_EVIDENCE, "evidence.source.does-not-exist");
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["unknown", "dangling", "does-not-exist", "missing"],
        "CAT-005 dangling evidence reference",
    );
}

#[test]
fn cat_006_orphaned_tests_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let tests = first_toml(&root.join("tests"));
    let mut body = fs::read_to_string(&tests).unwrap();
    body.push_str(
        "\n[[test]]\n\
         id = \"test.source.orphaned-fixture\"\n\
         control = \"control.source.protected-branch\"\n\
         kind = \"automated\"\n\
         required_evidence = [\"evidence.source.protected-branch\"]\n",
    );
    fs::write(&tests, body).unwrap();
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["orphan", "orphaned", "not listed"],
        "CAT-006 orphaned test (not listed on its control)",
    );
}

#[test]
fn cat_007_provider_names_cannot_appear_in_catalog_ids() {
    let src = crate_sources_joined("weeping-angel-canonical-catalog");
    for segment in PROVIDER_SEGMENTS {
        assert!(
            src.contains(segment),
            "reserved provider segment `{segment}` must be encoded in one denylist"
        );
    }

    let (_tmp, root) = copy_shipped_catalog();
    let controls = toml_containing(&root.join("controls"), PINNED_CONTROL);
    replace_in(&controls, PINNED_CONTROL, "control.github.protected-branch");
    let tests = toml_containing(&root.join("tests"), PINNED_CONTROL);
    replace_in(&tests, PINNED_CONTROL, "control.github.protected-branch");
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["github", "provider", "reserved", "namespace"],
        "CAT-007 control.github.*",
    );
}

#[test]
fn cat_008_framework_names_cannot_appear_in_catalog_ids() {
    let src = crate_sources_joined("weeping-angel-canonical-catalog");
    for segment in FRAMEWORK_SEGMENTS {
        assert!(
            src.contains(segment),
            "reserved framework segment `{segment}` must be encoded in one denylist"
        );
    }

    let (_tmp, root) = copy_shipped_catalog();
    let controls = toml_containing(&root.join("controls"), PINNED_CONTROL);
    replace_in(
        &controls,
        PINNED_CONTROL,
        "control.iso27001.protected-branch",
    );
    let tests = toml_containing(&root.join("tests"), PINNED_CONTROL);
    replace_in(&tests, PINNED_CONTROL, "control.iso27001.protected-branch");
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["iso27001", "framework", "reserved", "namespace", "regime"],
        "CAT-008 control.iso27001.*",
    );
}

#[test]
fn cat_009_unsupported_schema_fails_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    replace_in(
        &root.join("manifest.toml"),
        CATALOG_SCHEMA,
        "weeping-angel/canonical-catalog/v0",
    );
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["schema", "unsupported", "v0"],
        "CAT-009 unsupported schema",
    );
}

#[test]
fn cat_010_malformed_selectors_and_expressions_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let tests = first_toml(&root.join("tests"));
    let raw = fs::read_to_string(&tests).unwrap();
    if raw.contains("op = \"exists\"") {
        replace_in(&tests, "op = \"exists\"", "op = \"not-a-real-operator\"");
    } else {
        let mut body = raw;
        body.push_str("\n[test.expression]\nop = \"not-a-real-operator\"\n");
        fs::write(&tests, body).unwrap();
    }
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &[
            "expression",
            "operator",
            "op",
            "selector",
            "malformed",
            "unknown",
        ],
        "CAT-010 malformed expression",
    );
}

#[test]
fn extra_unlisted_section_files_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    fs::write(
        root.join("controls/unlisted.extra.toml"),
        format!("schema = \"{CATALOG_SCHEMA}\"\n"),
    )
    .unwrap();
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["unlisted", "extra", "not listed", "undeclared"],
        "extra section TOML not listed in the manifest",
    );
}

#[test]
fn path_escape_in_manifest_fails_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let outside = root.parent().unwrap().join("escaped.toml");
    fs::write(&outside, "pwned = true\n").unwrap();
    let manifest_path = root.join("manifest.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    let mutated = manifest.replace("controls/", "../");
    assert_ne!(
        mutated, manifest,
        "manifest must list controls/ paths so a `../` escape can be tested"
    );
    fs::write(&manifest_path, mutated).unwrap();
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["escape", "path", "..", "outside", "root"],
        "listed paths must not escape the catalog root",
    );
}

#[test]
fn malformed_catalog_ids_fail_closed() {
    let (_tmp, root) = copy_shipped_catalog();
    let controls = toml_containing(&root.join("controls"), PINNED_CONTROL);
    replace_in(&controls, PINNED_CONTROL, "source.branch-protection");
    assert_cli_failure_mentions(
        &validate_catalog(&root),
        &["namespace", "malformed", "control.", "invalid"],
        "catalog ids must use control.*/evidence.*/test.*",
    );
}

// ── CAT-011 CLI ────────────────────────────────────────────────────────────

#[test]
fn cat_011_cli_parses_and_inspect_shows_fixture_control() {
    let cmd = Cli::clap_command();
    let assurance = cmd
        .get_subcommands()
        .find(|c| c.get_name() == "assurance")
        .expect("assurance family exists");
    let names: Vec<&str> = assurance.get_subcommands().map(|c| c.get_name()).collect();
    assert!(
        names.contains(&"catalog"),
        "AssuranceCommand must grow `catalog`; have {names:?}"
    );

    for argv in [
        &["weeping-angel", "assurance", "catalog", "validate"][..],
        &["weeping-angel", "assurance", "catalog", "stats"][..],
        &[
            "weeping-angel",
            "assurance",
            "catalog",
            "inspect",
            PINNED_CONTROL,
        ][..],
    ] {
        let parsed = Cli::try_parse_from(argv);
        assert!(
            parsed.is_ok(),
            "src/cli.rs must parse `{:?}`: {parsed:?}",
            &argv[1..]
        );
    }

    let cli_src = fs::read_to_string(manifest_dir().join("src/cli.rs")).unwrap();
    assert!(
        cli_src.contains("Catalog"),
        "parser surface lives in src/cli.rs"
    );
    assert!(
        !cli_src.contains("CanonicalCatalog::load")
            && !cli_src.contains("CanonicalCatalog::validate")
            && !cli_src.contains("CanonicalCatalog::digest"),
        "execution must not be inlined in the clap enum"
    );

    let main = fs::read_to_string(manifest_dir().join("src/main.rs")).unwrap();
    let exec_module = manifest_dir().join("src/assurance_catalog.rs");
    assert!(
        main.contains("AssuranceCommand::Catalog")
            || main.contains("catalog::")
            || exec_module.is_file(),
        "Commands::Assurance must dispatch Catalog (not the blanket stub)"
    );

    let root = require_shipped_catalog();
    let inspect = inspect_control(PINNED_CONTROL, Some(&root));
    assert!(
        inspect.status.success(),
        "inspect {PINNED_CONTROL} must succeed; got:\n{}",
        output_text(&inspect)
    );
    let text = output_text(&inspect);
    for id in [PINNED_CONTROL, PINNED_EVIDENCE, PINNED_TEST] {
        assert!(
            text.contains(id),
            "inspect must show the named control and linked evidence/tests; missing `{id}` in:\n{text}"
        );
    }

    let stats = stats_catalog(&root);
    assert!(
        stats.status.success(),
        "catalog stats must succeed; got:\n{}",
        output_text(&stats)
    );
}

// ── CAT-012 / CAT-013 crate graph ──────────────────────────────────────────

#[test]
fn cat_012_framework_stays_collector_sdk_and_catalog_free() {
    let toml = read_crate_toml("weeping-angel-framework");
    for forbidden in [
        "weeping-angel-collector",
        "weeping-angel-canonical-catalog",
        "weeping-angel-control-test",
        "reqwest",
        "octocrab",
        "aws-sdk-",
        "cloudflare",
    ] {
        assert!(
            !toml.contains(forbidden),
            "framework must not depend on `{forbidden}`"
        );
    }
    assert!(
        toml.contains("weeping-angel-assurance-ir"),
        "framework still depends on IR"
    );
}

#[test]
fn cat_013_collector_stays_framework_and_catalog_blind() {
    let toml = read_crate_toml("weeping-angel-collector");
    for forbidden in [
        "weeping-angel-framework",
        "weeping-angel-canonical-catalog",
        "weeping-angel-control-test",
        "iso27001",
        "soc2",
        "gdpr",
    ] {
        assert!(
            !toml.contains(forbidden),
            "collector must not mention `{forbidden}`"
        );
    }
}

// ── Compatibility: IR / ISO packs stay put ─────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn ir_schema_and_iso_pack_ids_are_not_remapped() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    assert_eq!(FRAMEWORK_PACK_SCHEMA, "weeping-angel/framework-pack/v1");
    assert!(ControlId::try_new("source.branch-protection").is_ok());
    assert!(ControlId::try_new("control.github.branch").is_ok());

    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        metadata.contains("id = \"source.branch-protection\"")
            && metadata.contains("id = \"test.source.branch-protection\""),
        "ISO pack IDs must not be remapped in this slice"
    );
    assert!(
        !metadata.contains("id = \"control.source.protected-branch\""),
        "ISO metadata is not rewritten onto catalog control.* ids here"
    );

    let _ = std::any::type_name::<Control>();
    let _ = std::any::type_name::<Requirement>();
    let _ = std::any::type_name::<Mapping>();
    let _ = std::any::type_name::<EvidenceRequirement>();
    let _ = std::any::type_name::<PlannedControlTest>();
}

// ── Prompt 2 / increment 2: catalog SSOT, fail-closed pack, pins, readiness ─

#[test]
fn cat_ssot_t01_framework_has_no_second_catalog_toml_parser() {
    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    assert!(
        !pack.contains("fn discover_catalog_index"),
        "CAT-SSOT-T01: weeping-angel-framework must not re-parse catalog/canonical/v1"
    );
    assert!(
        !pack.contains("struct CatalogIndex")
            && !pack.contains("struct IndexedControl")
            && !pack.contains("struct IndexedTest"),
        "CAT-SSOT-T01: CatalogIndex / IndexedControl / IndexedTest are a competing parser"
    );
    assert!(
        !pack.contains("let Ok(text) = fs::read_to_string(&path) else {")
            && !pack.contains("let Ok(parsed) = toml::from_str::<toml::Value>(&text) else {"),
        "CAT-SSOT-T01: pack load must not continue over catalog IO/TOML"
    );
    assert!(
        !pack.contains("catalog/canonical"),
        "CAT-SSOT-T01: pack load must not walk catalog/canonical/v1"
    );

    let tmp = tempfile::tempdir().expect("isolated pack");
    write_trust_boundary_pack(
        tmp.path(),
        honest_mapping_block(),
        "schema = \"weeping-angel/framework-pack/v1\"\n",
    );
    match load_framework_pack_from(tmp.path()) {
        Err(PackError::Dangling { to, .. }) => {
            assert_eq!(to, "control.identity.privileged-mfa");
        }
        other => panic!(
            "CAT-SSOT-T01: without a supplied CanonicalCatalog projection, mapping onto catalog ids must fail closed, got {other:?}"
        ),
    }
}

#[test]
fn cat_ssot_t02_competing_metadata_control_library_fails_pack_load() {
    let tmp = tempfile::tempdir().expect("competing metadata pack");
    write_trust_boundary_pack(
        tmp.path(),
        honest_mapping_block(),
        r#"schema = "weeping-angel/framework-pack/v1"

[[control]]
id = "sliver.mfa.privileged"
title = "competing sliver"
description = "non-control.* row must not be skipped"

[[control]]
id = "control.identity.privileged-mfa"
title = "pack-local privileged MFA"
description = "competing catalog id"

[[test]]
id = "test.pack.privileged-mfa"
control = "control.identity.privileged-mfa"
kind = "automated"
"#,
    );
    match load_framework_pack_from(tmp.path()) {
        Err(err) => {
            let text = err.to_string().to_ascii_lowercase();
            assert!(
                text.contains("compet")
                    || text.contains("library")
                    || text.contains("metadata")
                    || text.contains("control"),
                "CAT-SSOT-T02: competing metadata library must be a typed PackError, got {err}"
            );
        }
        Ok(pack) => panic!(
            "CAT-SSOT-T02: metadata [[control]]/[[test]] must fail pack load, got {} controls / {} tests",
            pack.controls.len(),
            pack.tests.len()
        ),
    }
}

#[test]
fn cat_ssot_t03_catalog_still_fail_closed_and_nested_unknown_ops_error() {
    let (_tmp, root) = copy_shipped_catalog();
    let controls = first_toml(&root.join("controls"));
    let mut body = fs::read_to_string(&controls).unwrap();
    body.push_str(
        "\n[[control]]\nid = \"control.source.protected-branch\"\ntitle = \"duplicate\"\n",
    );
    fs::write(&controls, body).unwrap();
    match CanonicalCatalog::load(&root) {
        Err(CatalogError::Duplicate { .. }) => {}
        other => {
            panic!("CAT-SSOT-T03: CanonicalCatalog must still reject duplicate ids, got {other:?}")
        }
    }

    let (_tmp, nested_root) = copy_shipped_catalog();
    let tests = toml_containing(&nested_root.join("tests"), PINNED_TEST);
    let mut nested = fs::read_to_string(&tests).unwrap();
    nested.push_str(
        r#"

[[test]]
id = "test.source.nested-unknown-op"
control = "control.source.protected-branch"
kind = "automated"
required_evidence = ["evidence.source.protected-branch"]

[test.expression]
op = "all"

[[test.expression.children]]
op = "not-a-real-operator"
"#,
    );
    fs::write(&tests, nested).unwrap();
    let control = toml_containing(&nested_root.join("controls"), PINNED_CONTROL);
    replace_in(
        &control,
        "tests = [\"test.source.protected-branch\"]",
        "tests = [\"test.source.protected-branch\", \"test.source.nested-unknown-op\"]",
    );
    match CanonicalCatalog::load(&nested_root) {
        Err(CatalogError::UnknownOperator { op, .. }) => {
            assert_eq!(op, "not-a-real-operator");
        }
        Err(CatalogError::MalformedExpression { reason, .. }) => {
            assert!(
                reason.to_ascii_lowercase().contains("op")
                    || reason.to_ascii_lowercase().contains("operator"),
                "CAT-SSOT-T03: nested unknown op must be malformed/unknown, got {reason}"
            );
        }
        other => panic!(
            "CAT-SSOT-T03: nested unknown child op must fail closed, got {}",
            match other {
                Ok(_) => "Ok(CanonicalCatalog)".into(),
                Err(e) => format!("Err({e})"),
            }
        ),
    }
}

#[test]
fn frw_expr_t01_catalog_expression_survives_compile_onto_compiled_test() {
    let catalog = CanonicalCatalog::load(catalog_v1()).expect("catalog load");
    let catalog_test = catalog
        .tests()
        .get("test.identity.privileged-mfa-enabled")
        .expect("catalog test.identity.privileged-mfa-enabled");
    assert_eq!(
        catalog_test.expression.get("op").and_then(|v| v.as_str()),
        Some("coverage-at-least"),
        "catalog SSOT stores the coverage expression"
    );

    let lib = crate_src_file("weeping-angel-framework", "lib.rs");
    let plan = lib
        .split("fn construct_test_plan(")
        .nth(1)
        .expect("construct_test_plan");
    assert!(
        !plan.contains("expr: None,"),
        "FRW-EXPR-T01: construct_test_plan must not drop catalog [test.expression]"
    );

    let compiled = compile_iso_pack();
    let test = compiled
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.identity.privileged-mfa-enabled")
        .expect("compiled plan must include catalog test.identity.privileged-mfa-enabled");
    let expr = test
        .expr
        .clone()
        .expect("FRW-EXPR-T01: CompiledTest.expr must be Some");
    let parsed: TestExpr = serde_json::from_value(expr.clone())
        .expect("FRW-EXPR-T01: CompiledTest.expr must round-trip as TestExpr");
    let again = serde_json::to_value(&parsed).expect("re-serialize TestExpr");
    assert_eq!(
        serde_json::from_value::<TestExpr>(again).unwrap(),
        parsed,
        "FRW-EXPR-T01: parse/serialize/reload of the compiled expression must be lossless"
    );
    let text = expr.to_string();
    assert!(
        text.contains("CoverageAtLeast")
            || text.contains("coverage-at-least")
            || text.contains("coverageAtLeast")
            || text.contains("100"),
        "FRW-EXPR-T01: privileged-MFA expr must preserve coverage-at-least 100, got {expr}"
    );

    let manual = compiled
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.identity.strong-authentication-policy")
        .expect("manual-review catalog test must compile");
    let manual_expr = manual
        .expr
        .as_ref()
        .expect("FRW-EXPR-T01: manual-review must not be dropped");
    let manual_text = manual_expr.to_string();
    assert!(
        manual_text.contains("ManualReview") || manual_text.contains("manual-review"),
        "FRW-EXPR-T01: manual-review must survive compile, got {manual_expr}"
    );
}

#[test]
fn frw_expr_t02_all_any_not_and_threshold_are_not_normalized_together() {
    let compiled = compile_iso_pack();
    let coverage = compiled
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.identity.privileged-mfa-enabled")
        .and_then(|t| t.expr.clone());
    let population = compiled
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.identity.unique-user-identities")
        .and_then(|t| t.expr.clone());
    let manual = compiled
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.identity.strong-authentication-policy")
        .and_then(|t| t.expr.clone());
    assert!(
        coverage.is_some() && population.is_some() && manual.is_some(),
        "FRW-EXPR-T02: coverage / all-subjects / manual-review must each survive compile"
    );
    assert_ne!(
        coverage, population,
        "FRW-EXPR-T02: coverage-at-least must not be normalized into all-subjects"
    );
    assert_ne!(
        coverage, manual,
        "FRW-EXPR-T02: coverage-at-least must not collapse into manual-review"
    );

    let leaf = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.identity.mfa-status",
    )));
    let all = TestExpr::All(vec![leaf.clone()]);
    let any = TestExpr::Any(vec![leaf]);
    let not_all = TestExpr::Not(Box::new(all.clone()));
    assert_ne!(all, any, "FRW-EXPR-T02: all vs any stay distinct");
    assert_ne!(
        all, not_all,
        "FRW-EXPR-T02: dropping not must change the tree"
    );
    assert_ne!(
        serde_json::to_value(&all).unwrap(),
        serde_json::to_value(&not_all).unwrap(),
        "FRW-EXPR-T02: not(all) JSON must not collide with all"
    );
}

#[test]
fn frw_parse_t01_unknown_mapping_tokens_are_typed_pack_errors() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "totally-bogus"
relation = "PartiallySatisfies"
rationale = "unknown completeness must not become Partial"
"#,
            "completeness",
            &["completeness", "unknown", "totally-bogus"],
        ),
        (
            r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "sideways"
completeness = "partial"
relation = "PartiallySatisfies"
rationale = "unknown direction must not become Forward"
"#,
            "direction",
            &["direction", "unknown", "sideways"],
        ),
        (
            r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "partial"
relation = "KindaSatisfies"
rationale = "unknown relation must not parse"
"#,
            "relation",
            &["relation", "unsupported", "kindasatisfies"],
        ),
    ];
    for (mapping, field, needles) in cases {
        let tmp = tempfile::tempdir().expect("malformed mapping pack");
        write_trust_boundary_pack(
            tmp.path(),
            mapping,
            "schema = \"weeping-angel/framework-pack/v1\"\n",
        );
        match load_framework_pack_from(tmp.path()) {
            Err(err) => {
                let text = err.to_string().to_ascii_lowercase();
                assert!(
                    needles.iter().any(|n| text.contains(*n)),
                    "FRW-PARSE-T01: unknown {field} must be a typed PackError mentioning {needles:?}, got {err}"
                );
            }
            Ok(pack) => panic!(
                "FRW-PARSE-T01: unknown {field} must fail closed, got completeness={:?} direction={:?} relation={:?}",
                pack.mappings.first().map(|m| m.completeness()),
                pack.mappings.first().map(|m| m.direction()),
                pack.mappings.first().map(|m| m.relation()),
            ),
        }
    }
}

#[test]
fn frw_parse_t02_dangling_catalog_id_and_malformed_manifest_fail_closed() {
    let dangling = tempfile::tempdir().expect("dangling pack");
    write_trust_boundary_pack(
        dangling.path(),
        r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.does-not-exist"
direction = "forward"
completeness = "partial"
relation = "PartiallySatisfies"
rationale = "unknown catalog id"
"#,
        "schema = \"weeping-angel/framework-pack/v1\"\n",
    );
    match load_framework_pack_from(dangling.path()) {
        Err(PackError::Dangling { to, .. }) => {
            assert!(to.contains("does-not-exist"));
        }
        other => panic!(
            "FRW-PARSE-T02: dangling catalog control id must be PackError::Dangling, got {other:?}"
        ),
    }

    let bad_schema = tempfile::tempdir().expect("bad schema pack");
    write_trust_boundary_pack(
        bad_schema.path(),
        honest_mapping_block(),
        "schema = \"weeping-angel/framework-pack/v1\"\n",
    );
    replace_in(
        &bad_schema.path().join("manifest.toml"),
        "weeping-angel/framework-pack/v1",
        "weeping-angel/framework-pack/v0",
    );
    match load_framework_pack_from(bad_schema.path()) {
        Err(PackError::Schema(message)) => {
            assert!(
                message.contains("v0") || message.to_ascii_lowercase().contains("schema"),
                "FRW-PARSE-T02: malformed/unsupported manifest schema, got {message}"
            );
        }
        other => panic!("FRW-PARSE-T02: unsupported pack schema must fail closed, got {other:?}"),
    }

    let empty_completeness = tempfile::tempdir().expect("empty completeness pack");
    write_trust_boundary_pack(
        empty_completeness.path(),
        r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = ""
relation = "PartiallySatisfies"
rationale = "empty completeness is not silently Partial"
"#,
        "schema = \"weeping-angel/framework-pack/v1\"\n",
    );
    match load_framework_pack_from(empty_completeness.path()) {
        Err(err) => {
            let text = err.to_string().to_ascii_lowercase();
            assert!(
                text.contains("completeness") || text.contains("unknown") || text.contains("empty"),
                "FRW-PARSE-T02: empty completeness must be a typed error, got {err}"
            );
        }
        Ok(pack) => panic!(
            "FRW-PARSE-T02: empty completeness must fail closed, got {:?}",
            pack.mappings.first().map(|m| m.completeness())
        ),
    }
}

#[test]
fn frw_dig_t01_whitespace_comment_key_order_share_semantic_digest() {
    let mapping = honest_mapping_block();
    let meta = "schema = \"weeping-angel/framework-pack/v1\"\n";
    let a = tempfile::tempdir().expect("pack a");
    let b = tempfile::tempdir().expect("pack b");
    write_trust_boundary_pack(a.path(), mapping, meta);
    write_trust_boundary_pack(
        b.path(),
        &format!("# comment-only / key-order noise\n\n{mapping}\n\n"),
        &format!("\n{meta}\n# trailing comment\n"),
    );
    let mappings_b = fs::read_to_string(b.path().join("mappings.toml")).unwrap();
    let reordered = mappings_b.replace(
        "direction = \"forward\"\ncompleteness = \"partial\"\nrelation = \"PartiallySatisfies\"",
        "relation = \"PartiallySatisfies\"\ncompleteness = \"partial\"\ndirection = \"forward\"",
    );
    fs::write(b.path().join("mappings.toml"), reordered).unwrap();

    let da = load_isolated_pack(a.path());
    let db = load_isolated_pack(b.path());
    let (da, db) = match (da, db) {
        (Ok(a), Ok(b)) => (a, b),
        other => panic!(
            "FRW-DIG-T01: formatting-only packs must load so their semantic digest can match, got {other:?}"
        ),
    };
    assert_eq!(
        da.digest.0, db.digest.0,
        "FRW-DIG-T01: whitespace/comment/key-order must not change FrameworkPackDigest"
    );

    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    let body = pack
        .split("let digest_body = serde_json::json!")
        .nth(1)
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("");
    assert!(
        body.contains("completeness") && body.contains("direction") && body.contains("rationale"),
        "FRW-DIG-T01: semantic digest payload must include completeness/direction/rationale, not id lists only"
    );
}

#[test]
fn frw_dig_t02_completeness_or_relation_change_changes_digest() {
    let mapping = r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "COMPLETENESS"
relation = "RELATION"
rationale = "RATIONALE"
provenance = { source = "BuiltIn", reference = "catalog/canonical/v1", author = "weeping-angel" }
"#;
    let meta = "schema = \"weeping-angel/framework-pack/v1\"\n";
    let partial = tempfile::tempdir().expect("partial");
    let full = tempfile::tempdir().expect("full");
    let supports = tempfile::tempdir().expect("supports");
    write_trust_boundary_pack(
        partial.path(),
        &mapping
            .replace("COMPLETENESS", "partial")
            .replace("RELATION", "PartiallySatisfies")
            .replace("RATIONALE", "slice"),
        meta,
    );
    write_trust_boundary_pack(
        full.path(),
        &mapping
            .replace("COMPLETENESS", "full")
            .replace("RELATION", "PartiallySatisfies")
            .replace("RATIONALE", "slice"),
        meta,
    );
    write_trust_boundary_pack(
        supports.path(),
        &mapping
            .replace("COMPLETENESS", "partial")
            .replace("RELATION", "Supports")
            .replace("RATIONALE", "slice"),
        meta,
    );
    let dp = load_isolated_pack(partial.path()).expect("partial pack");
    let df = load_isolated_pack(full.path()).expect("full pack");
    let ds = load_isolated_pack(supports.path()).expect("supports pack");
    assert_ne!(
        dp.digest.0, df.digest.0,
        "FRW-DIG-T02: completeness partial→full must change FrameworkPackDigest (assessment-affecting)"
    );
    assert_ne!(
        dp.digest.0, ds.digest.0,
        "FRW-DIG-T02: PartiallySatisfies vs Supports must not collide"
    );
    assert_ne!(dp.mappings[0].completeness(), df.mappings[0].completeness());
    assert_eq!(
        dp.mappings[0].relation(),
        MappingRelation::PartiallySatisfies
    );
    assert_eq!(ds.mappings[0].relation(), MappingRelation::Supports);
}

#[test]
fn frw_dig_t03_pack_digest_is_not_catalog_injection() {
    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    let body = pack
        .split("let digest_body = serde_json::json!")
        .nth(1)
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("");
    assert!(
        !body.contains("\"controls\": controls.iter().map(|c| c.id().as_str())")
            && !body.contains("\"tests\": tests.iter().map(|t| t.id.as_str())"),
        "FRW-DIG-T03: pack digest must not hash catalog-injected control/test id lists"
    );

    let iso = load_framework_pack("iso-27001", "2022").expect("ISO pack");
    let catalog = CanonicalCatalog::load(catalog_v1()).expect("catalog");
    let catalog_pin = catalog.digest().expect("catalog digest").to_string();
    assert_ne!(
        iso.digest.0, catalog_pin,
        "FRW-DIG-T03: FrameworkPackDigest must stay a separate identity from CanonicalCatalog::digest"
    );
    assert!(
        catalog_pin.starts_with("wa:canonical-catalog:"),
        "catalog identity remains the catalog pin"
    );
}

#[test]
fn pin_t01_snapshot_serialize_emits_stored_catalog_pin() {
    let readiness = crate_src_file("weeping-angel-assurance", "readiness.rs");
    assert!(
        !readiness.contains("state.serialize_field(\"catalogDigest\", &catalog_digest())"),
        "PIN-T01: FrameworkReadinessSnapshot::serialize must not call catalog_digest()"
    );
    assert!(
        readiness.contains("catalog_digest") || readiness.contains("catalogDigest"),
        "PIN-T01: snapshot must own a stored catalog digest field"
    );

    let snap: FrameworkReadinessSnapshot = serde_json::from_value(json!({
        "assessmentId": "assess-pin-t01",
        "framework": "iso-27001",
        "frameworkVersion": "2022",
        "frameworkPackDigest": "pack-pin-stored",
        "catalogDigest": "stored-catalog-pin-must-win",
        "assessmentDigest": "assessment-pin-stored",
        "evaluatedAt": "2026-01-01T00:00:00Z",
        "requirements": [],
        "controls": [],
        "effective": 0,
        "ineffective": 0,
        "partial": 0,
        "manualReview": 0,
        "insufficientEvidence": 0,
        "notApplicable": 0,
        "automationCoverage": "stored-not-used",
        "evidenceCoverage": "stored-not-used"
    }))
    .expect("deserialize snapshot with stored catalogDigest");
    let json = serde_json::to_value(&snap).expect("serialize snapshot");
    assert_eq!(
        json.get("catalogDigest").and_then(|v| v.as_str()),
        Some("stored-catalog-pin-must-win"),
        "PIN-T01: mutating/reloading live catalog must not replace the stored pin; got {}",
        json.get("catalogDigest").unwrap_or(&json)
    );
    assert_eq!(
        json.get("frameworkPackDigest").and_then(|v| v.as_str()),
        Some("pack-pin-stored")
    );
    let live = catalog_digest();
    assert_ne!(
        json.get("catalogDigest").and_then(|v| v.as_str()),
        Some(live.as_str()),
        "PIN-T01: serialized catalogDigest must not be the live catalog walk"
    );
}

#[test]
fn pin_t02_empty_assessment_run_pin_does_not_reload_catalog() {
    let snapshot = crate_src_file("weeping-angel-assurance", "snapshot.rs");
    let ser = snapshot
        .split("impl Serialize for AssessmentRun")
        .nth(1)
        .unwrap_or("");
    assert!(
        !ser.contains("catalog_digest()") && !ser.contains("CanonicalCatalog::load"),
        "PIN-T02: empty AssessmentRun pin must not invoke live CanonicalCatalog::load"
    );

    let run = AssessmentRun {
        canonical_catalog_pin: String::new(),
        framework_pack_digest: "pack-pin-empty".into(),
        ..AssessmentRun::default()
    };
    let json = serde_json::to_value(&run).expect("serialize run");
    let catalog = json
        .get("catalogDigest")
        .and_then(|v| v.as_str())
        .unwrap_or("missing");
    assert!(
        catalog.is_empty(),
        "PIN-T02: empty pin must stay empty (not live catalog-unavailable / current files); got {catalog}"
    );
    assert_eq!(
        json.get("canonicalCatalogDigest").and_then(|v| v.as_str()),
        Some("")
    );
}

#[test]
fn pin_t03_scheduler_projection_does_not_reload_pack_digest() {
    let scheduler = crate_src_file("weeping-angel-assurance", "scheduler.rs");
    let project = scheduler
        .split("fn run_project(")
        .nth(1)
        .expect("run_project");
    assert!(
        !project.contains("load_framework_pack("),
        "PIN-T03: run_project must not reload load_framework_pack to refresh digest"
    );
    assert!(
        !project.contains("\"unpinned\""),
        "PIN-T03: failed pack reload must not become an unpinned success path"
    );
    let snap = scheduler
        .split("fn run_snapshot(")
        .nth(1)
        .expect("run_snapshot");
    assert!(
        !snap.contains("load_framework_pack("),
        "PIN-T03: run_snapshot must use compiled/assessment identity, not a live pack walk"
    );
}

#[test]
fn rdy_t01_project_readiness_is_the_only_requirement_status_owner() {
    let readiness = crate_src_file("weeping-angel-assurance", "readiness.rs");
    assert!(
        readiness.contains("pub fn project_readiness(") && readiness.contains("partially covered"),
        "RDY-T01: project_readiness owns mapping-honesty status"
    );
    assert!(
        !readiness.contains("fn coverage_metrics(snapshot: &FrameworkReadinessSnapshot)"),
        "RDY-T01: snapshot serialize must not re-derive readiness independently of project_readiness"
    );

    let scheduler = crate_src_file("weeping-angel-assurance", "scheduler.rs");
    assert!(
        !scheduler.contains("\"partially covered\"") && !scheduler.contains("let has_partial"),
        "RDY-T01: scheduler must not reimplement requirement-status strings"
    );
    let snapshot = crate_src_file("weeping-angel-assurance", "snapshot.rs");
    assert!(
        !snapshot.contains("\"partially covered\""),
        "RDY-T01: snapshot compare must not assign requirement status"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack");
    let compiled = compile_iso_pack();
    let results: Vec<ControlTestResult> = compiled
        .controls
        .iter()
        .map(|c| synthetic_effective(c.id().as_str(), &format!("test.{}", c.id().as_str())))
        .collect();
    let first = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-rdy-t01-a"),
    );
    let second = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-rdy-t01-b"),
    );
    let a85 = first
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("A.8.5");
    let a85_b = second
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("A.8.5");
    assert_eq!(
        a85.status, a85_b.status,
        "RDY-T01: duplicated callers must not diverge; both must invoke project_readiness"
    );
    assert_eq!(a85.status, "partially covered");
}

#[test]
fn rdy_t02_no_privileged_mfa_overlay_replacing_catalog_predicate() {
    let scheduler = crate_src_file("weeping-angel-assurance", "scheduler.rs");
    assert!(
        !scheduler.contains("fn overlay_privileged_mfa_presence(")
            && !scheduler.contains("overlay_privileged_mfa_presence("),
        "RDY-T02: privileged-MFA effectiveness is the catalog coverage expression, not a presence overlay"
    );
    assert!(
        !scheduler.contains(".require(EvidenceType::new(\"identity.privileged.mfa\"))"),
        "RDY-T02: overlay must not replace evidence.identity.mfa-status with identity.privileged.mfa"
    );
    assert!(
        !scheduler.contains("automation_coverage: \"0%\".into()"),
        "RDY-T02: empty_readiness must not invent coverage percentages that bypass project_readiness"
    );
}

#[test]
fn rdy_t03_partially_satisfies_supports_and_equivalent_stay_distinct() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack");
    assert!(
        pack.mappings
            .iter()
            .any(|m| m.relation() == MappingRelation::PartiallySatisfies),
        "RDY-T03: pack still carries PartiallySatisfies"
    );
    assert!(
        pack.mappings
            .iter()
            .any(|m| m.relation() == MappingRelation::Supports),
        "RDY-T03: pack still carries Supports"
    );
    assert!(
        pack.mappings
            .iter()
            .all(|m| m.to_control().as_str().starts_with("control.")),
        "RDY-T03: packs project onto control.* only"
    );

    let compiled = compile_iso_pack();
    let results: Vec<ControlTestResult> = compiled
        .controls
        .iter()
        .map(|c| synthetic_effective(c.id().as_str(), &format!("test.{}", c.id().as_str())))
        .collect();
    let snapshot = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-rdy-t03"),
    );
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("A.8.5");
    assert_eq!(
        a85.status, "partially covered",
        "RDY-T03: PartiallySatisfies/Supports with Effective tests stay partially covered, not effective"
    );
    assert_ne!(
        MappingRelation::PartiallySatisfies,
        MappingRelation::Supports
    );
    assert_ne!(
        MappingRelation::PartiallySatisfies,
        MappingRelation::Equivalent
    );
    assert_ne!(MappingCompleteness::Partial, MappingCompleteness::Full);
    assert_ne!(MappingDirection::Forward, MappingDirection::Reverse);

    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("iso27001:")
            && !collector
                .to_ascii_lowercase()
                .contains("iso 27001 compliant"),
        "RDY-T03: collector-facing APIs must not leak ISO-specific status into evidence"
    );
    let catalog_src = crate_sources_joined("weeping-angel-canonical-catalog");
    assert!(
        !catalog_src.contains("iso27001:") && !catalog_src.contains("ISO 27001 compliant"),
        "RDY-T03: catalog-facing APIs must not leak ISO-specific status into evidence"
    );
}
