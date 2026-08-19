//! Target suite for Canonical Assurance Catalog v1 infrastructure (Prompt 01).
//!
//! Encodes DESIRED behavior in `docs/sdd/canonical-assurance-catalog-v1.md`
//! §4 / §5 (CAT-001…016). Must stay RED on the current tree (no catalog
//! crate, no `catalog/canonical/v1`, no `assurance catalog` CLI). Do not
//! weaken these assertions to match today's absence, and do not implement
//! the feature in this suite.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::Parser;
use weeping_angel::cli::Cli;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, Control, ControlId, EvidenceRequirement, Mapping, PlannedControlTest,
    Requirement,
};
use weeping_angel_framework::pack::FRAMEWORK_PACK_SCHEMA;

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

// ── CAT-016 / registration ─────────────────────────────────────────────────

#[test]
fn cat_016_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_canonical_assurance_catalog_baseline")
            && toml.contains("sdd_canonical_assurance_catalog_target")
            && toml.contains("tests/sdd/canonical_assurance_catalog.baseline.rs")
            && toml.contains("tests/sdd/canonical_assurance_catalog.target.rs"),
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
        "Prompt 01 fixture.example.toml must remain under catalog/canonical/v1/controls"
    );
    assert!(
        !control_files.is_empty(),
        "catalog/canonical/v1/controls must ship at least the Prompt 01 fixture"
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
        names.iter().any(|n| *n == "catalog"),
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
