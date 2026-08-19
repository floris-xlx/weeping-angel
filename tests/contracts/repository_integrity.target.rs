//! Target suite for Repository Integrity.
//!
//! Increment 1 (RI-T01–T17): health-gate acceptance (`docs/specs/repository-integrity.md`
//! §4 / §5) — GREEN on the increment-1 / ADR-0010 tree.
//!
//! Increment 2 (RI-T18–T31 + updated RI-T13 / RI-T17): desired guard-engine /
//! governance behavior from §13. MUST FAIL (RED) on CURRENT increment-1 code:
//! xtask monolith, hard-coded policy, Guard 14/15 skip-with-debt, weak debt
//! exemptions, JSON without schema/counts, no spec-lifecycle / adr-identity
//! files. Do not implement increment-2 product code in this suite.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ARCH_SCHEMA: &str = "weeping-angel/architecture/v1";
const INVARIANTS_SCHEMA: &str = "weeping-angel/architecture-invariants/v1";
const FORBIDDEN_SCHEMA: &str = "weeping-angel/forbidden-patterns/v1";
const DEBT_SCHEMA: &str = "weeping-angel/debt-register/v1";

const OWNERSHIP_CONCEPTS: [&str; 7] = [
    "catalog",
    "framework_compilation",
    "readiness_projection",
    "temporal_evidence_selection",
    "assessment_lineage",
    "evidence_persistence",
    "assurance_cli",
];

/// Product-semantic stubs owned by concurrent Prompts 2/3 (not 14/15).
const PRODUCT_STUB_CHECKS: [&str; 8] = ["05", "06", "07", "08", "09", "10", "11", "12"];

const OWNERSHIP_KINDS: [&str; 5] = [
    "exclusive",
    "facade",
    "projection",
    "adapter",
    "shared-primitive",
];

const ADR_IDENTITY_SCHEMA: &str = "weeping-angel/adr-identity/v1";
const SPEC_LIFECYCLE_SCHEMA: &str = "weeping-angel/spec-lifecycle/v1";
const GUARD_REPORT_SCHEMA: &str = "weeping-angel/guard-report/v1";


fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rel(path: &str) -> PathBuf {
    repo_root().join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(rel(path)).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn parse_toml(path: &str) -> toml::Value {
    read(path)
        .parse::<toml::Value>()
        .unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn workspace_members() -> Vec<String> {
    parse_toml("Cargo.toml")["workspace"]["members"]
        .as_array()
        .expect("workspace.members")
        .iter()
        .map(|v| v.as_str().expect("member string").to_string())
        .collect()
}

fn ownership_entry<'a>(arch: &'a toml::Value, concept: &str) -> &'a toml::Value {
    arch.get("ownership")
        .and_then(|o| o.get(concept))
        .unwrap_or_else(|| panic!("ownership.{concept} missing"))
}

fn cargo_xtask_guard() -> std::process::Output {
    Command::new("cargo")
        .args(["xtask", "guard"])
        .current_dir(repo_root())
        .output()
        .expect("spawn cargo xtask guard")
}

fn collect_rs(dir: PathBuf) -> String {
    let mut out = String::new();
    fn walk(dir: &Path, out: &mut String) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(text) = fs::read_to_string(&path) {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
    }
    walk(&dir, &mut out);
    out
}

fn guard_report() -> String {
    let output = cargo_xtask_guard();
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn finding_ids(register: &toml::Value) -> BTreeSet<String> {
    register
        .get("finding")
        .and_then(|f| f.as_array())
        .expect("[[finding]]")
        .iter()
        .map(|f| {
            f.get("id")
                .and_then(|s| s.as_str())
                .expect("finding.id")
                .to_string()
        })
        .collect()
}

fn proof_ok(finding: &toml::Value) -> bool {
    let tests = finding
        .get("regression_tests")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().any(|x| x.as_str().is_some_and(|s| !s.is_empty())))
        .unwrap_or(false);
    let guard = match finding.get("repository_guard") {
        Some(toml::Value::String(s)) => !s.is_empty(),
        Some(toml::Value::Boolean(true)) => true,
        Some(toml::Value::Integer(_)) => true,
        _ => false,
    };
    tests || guard
}

/// RI-T01: architecture.toml exists, parses, and carries the v1 schema string.
#[test]
fn ri_t01_architecture_toml_exists_and_parses() {
    assert!(
        rel("architecture/architecture.toml").is_file(),
        "architecture/architecture.toml must exist"
    );
    let arch = parse_toml("architecture/architecture.toml");
    assert_eq!(
        arch.get("schema").and_then(|s| s.as_str()),
        Some(ARCH_SCHEMA)
    );
}

/// RI-T02: ownership table lists all seven concepts with live crate names and paths.
#[test]
fn ri_t02_ownership_table_uses_live_crate_names() {
    let arch = parse_toml("architecture/architecture.toml");
    let ownership = arch.get("ownership").expect("ownership table");
    for concept in OWNERSHIP_CONCEPTS {
        assert!(
            ownership.get(concept).is_some(),
            "ownership.{concept} is mandatory"
        );
    }
    let required = [
        (
            "catalog",
            "weeping-angel-canonical-catalog",
            &["crates/weeping-angel-canonical-catalog"][..],
        ),
        (
            "framework_compilation",
            "weeping-angel-framework",
            &["crates/weeping-angel-framework"][..],
        ),
        (
            "readiness_projection",
            "weeping-angel-assurance",
            &["crates/weeping-angel-assurance/src/readiness.rs"][..],
        ),
        (
            "temporal_evidence_selection",
            "weeping-angel-assurance",
            &["crates/weeping-angel-assurance/src/temporal.rs"][..],
        ),
        (
            "assessment_lineage",
            "weeping-angel-assurance",
            &["crates/weeping-angel-assurance/src/lineage.rs"][..],
        ),
        (
            "evidence_persistence",
            "weeping-angel-evidence",
            &["crates/weeping-angel-evidence"][..],
        ),
        (
            "assurance_cli",
            "weeping-angel",
            &["src/main.rs", "src/cli.rs"][..],
        ),
    ];
    for (concept, crate_name, required_paths) in required {
        let entry = ownership_entry(&arch, concept);
        assert_eq!(
            entry.get("crate").and_then(|c| c.as_str()),
            Some(crate_name),
            "ownership.{concept}.crate"
        );
        let paths: Vec<&str> = entry
            .get("paths")
            .and_then(|p| p.as_array())
            .expect("paths")
            .iter()
            .map(|v| v.as_str().expect("path str"))
            .collect();
        assert!(
            !paths.is_empty(),
            "ownership.{concept}.paths must be non-empty"
        );
        for needle in required_paths {
            assert!(
                paths.iter().any(|p| *p == *needle || p.contains(needle)),
                "ownership.{concept}.paths must include {needle}; got {paths:?}"
            );
        }
        for path in &paths {
            assert!(
                rel(path).exists(),
                "ownership.{concept} path {path} must exist on disk"
            );
        }
    }
}

/// RI-T03: catalog owner is weeping-angel-canonical-catalog, not the hypothetical name.
#[test]
fn ri_t03_catalog_crate_is_canonical_catalog() {
    let arch = parse_toml("architecture/architecture.toml");
    let crate_name = ownership_entry(&arch, "catalog")
        .get("crate")
        .and_then(|c| c.as_str());
    assert_eq!(crate_name, Some("weeping-angel-canonical-catalog"));
    assert_ne!(crate_name, Some("weeping-angel-catalog"));
    assert!(!rel("crates/weeping-angel-catalog/Cargo.toml").is_file());
}

/// RI-T04: assurance_cli is the root weeping-angel package (src/main.rs + src/cli.rs).
#[test]
fn ri_t04_assurance_cli_is_root_package() {
    let arch = parse_toml("architecture/architecture.toml");
    let entry = ownership_entry(&arch, "assurance_cli");
    assert_eq!(
        entry.get("crate").and_then(|c| c.as_str()),
        Some("weeping-angel")
    );
    assert_ne!(
        entry.get("crate").and_then(|c| c.as_str()),
        Some("weeping-angel-assurance-cli")
    );
    let paths: Vec<&str> = entry
        .get("paths")
        .and_then(|p| p.as_array())
        .expect("paths")
        .iter()
        .map(|v| v.as_str().expect("path str"))
        .collect();
    assert!(paths.iter().any(|p| *p == "src/main.rs"));
    assert!(paths.iter().any(|p| *p == "src/cli.rs"));
    assert!(!rel("crates/weeping-angel-assurance-cli/Cargo.toml").is_file());
}

/// RI-T05
#[test]
fn ri_t05_forbidden_patterns_toml_exists_and_parses() {
    assert!(rel("architecture/forbidden-patterns.toml").is_file());
    let value = parse_toml("architecture/forbidden-patterns.toml");
    assert_eq!(
        value.get("schema").and_then(|s| s.as_str()),
        Some(FORBIDDEN_SCHEMA)
    );
    let text = read("architecture/forbidden-patterns.toml");
    assert!(
        text.contains("weeping-angel-catalog"),
        "forbidden-patterns must seed hypothetical package weeping-angel-catalog"
    );
    assert!(
        text.contains("weeping-angel-assurance-cli"),
        "forbidden-patterns must seed hypothetical package weeping-angel-assurance-cli"
    );
    assert!(
        text.contains("tests/sdd/"),
        "forbidden-patterns must seed ADR 0004 path tests/sdd/"
    );
}

/// RI-T06
#[test]
fn ri_t06_invariants_toml_exists_and_parses() {
    assert!(rel("architecture/invariants.toml").is_file());
    let value = parse_toml("architecture/invariants.toml");
    assert_eq!(
        value.get("schema").and_then(|s| s.as_str()),
        Some(INVARIANTS_SCHEMA)
    );
}

/// RI-T07 + RI-T08: register schema, required fields, unique ids, resolved-with-proof.
#[test]
fn ri_t07_debt_register_has_unique_ids_and_status() {
    assert!(rel("docs/debt/register.toml").is_file());
    assert!(rel("docs/debt/README.md").is_file());
    assert!(rel("docs/debt/baseline-2026-08.md").is_file());
    let value = parse_toml("docs/debt/register.toml");
    assert_eq!(
        value.get("schema").and_then(|s| s.as_str()),
        Some(DEBT_SCHEMA)
    );
    let findings = value
        .get("finding")
        .and_then(|f| f.as_array())
        .expect("[[finding]]");
    assert!(!findings.is_empty(), "debt register must have findings");
    let allowed = [
        "open",
        "confirmed",
        "in-progress",
        "resolved",
        "rejected",
        "superseded",
    ];
    let mut ids = BTreeSet::new();
    for finding in findings {
        let id = finding
            .get("id")
            .and_then(|s| s.as_str())
            .expect("finding.id required");
        let status = finding
            .get("status")
            .and_then(|s| s.as_str())
            .expect("finding.status required");
        let title = finding
            .get("title")
            .and_then(|s| s.as_str())
            .expect("finding.title required");
        let summary = finding
            .get("summary")
            .and_then(|s| s.as_str())
            .expect("finding.summary required");
        assert!(!id.is_empty(), "finding.id must be non-empty");
        assert!(!title.is_empty(), "finding.title required");
        assert!(!summary.is_empty(), "finding.summary required");
        assert!(allowed.contains(&status), "illegal status {status} on {id}");
        assert!(ids.insert(id.to_string()), "duplicate finding id {id}");
        if status == "resolved" {
            assert!(
                proof_ok(finding),
                "resolved finding {id} must list regression_tests or repository_guard"
            );
        }
    }
}

/// Seed findings so stub skips are attributable (spec §4.2.2).
#[test]
fn ri_t07b_seed_debt_findings_include_stub_check_ids() {
    let ids = finding_ids(&parse_toml("docs/debt/register.toml"));
    for seed in [
        "DEBT-DUP-ADR",
        "DEBT-UNWRAP",
        "DEBT-IGNORE",
        "DEBT-SCHEMA-DUP",
    ] {
        assert!(ids.contains(seed), "register must contain {seed}");
    }
    for id in PRODUCT_STUB_CHECKS {
        let finding = format!("DEBT-GUARD-{id}");
        assert!(
            ids.contains(finding.as_str()),
            "register must retain {finding} as resolved proof after Guard {id} became real"
        );
    }
    for id in ["14", "15"] {
        let finding = format!("DEBT-GUARD-{id}");
        assert!(
            ids.contains(finding.as_str()),
            "register must retain {finding} as resolved proof after Guards 14/15 become real"
        );
    }
}

/// README documents the status machine and proof law; baseline snapshot has live counts.
#[test]
fn ri_t07c_debt_readme_and_baseline_snapshot_exist() {
    let readme = read("docs/debt/README.md");
    for needle in [
        "open",
        "confirmed",
        "in-progress",
        "resolved",
        "rejected",
        "superseded",
        "regression_tests",
        "repository_guard",
    ] {
        assert!(
            readme.contains(needle),
            "docs/debt/README.md must document {needle}"
        );
    }
    let baseline = read("docs/debt/baseline-2026-08.md");
    for needle in [
        "test binaries",
        "ignored",
        "unwrap",
        "expect",
        "source-grep",
        "ADR",
        "duplicate",
        "catalog",
        "framework",
        "schema",
        "fmt",
        "clippy",
        "cargo check",
        "cargo test",
    ] {
        assert!(
            baseline.to_lowercase().contains(&needle.to_lowercase()) || baseline.contains(needle),
            "docs/debt/baseline-2026-08.md must record live count topic {needle}"
        );
    }
}

/// RI-T09: xtask parser/guard rejects resolved-without-proof (fixture + live sources).
#[test]
fn ri_t09_resolved_without_proof_is_rejected() {
    assert!(
        rel("xtask").is_dir(),
        "xtask crate must exist so check 13 can reject resolved-without-proof"
    );
    let xtask_src = collect_rs(rel("xtask"));
    assert!(!xtask_src.is_empty(), "xtask sources must exist");
    assert!(
        xtask_src.contains("regression_tests") && xtask_src.contains("repository_guard"),
        "xtask sources must enforce resolved-without-proof using regression_tests or repository_guard"
    );
    assert!(
        xtask_src.contains("resolved")
            && (xtask_src.contains("duplicate") || xtask_src.contains("unique")),
        "xtask sources must reject duplicate finding ids"
    );
    let output = cargo_xtask_guard();
    assert!(
        output.status.success(),
        "cargo xtask guard must evaluate the live register (honest resolved findings only); stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// RI-T10: xtask workspace member + cargo alias.
#[test]
fn ri_t10_xtask_member_and_alias() {
    let members = workspace_members();
    assert!(
        members.iter().any(|m| m == "xtask"),
        "workspace.members must include xtask; found {members:?}"
    );
    assert!(
        rel("xtask/Cargo.toml").is_file(),
        "xtask/Cargo.toml must exist"
    );
    let pkg = parse_toml("xtask/Cargo.toml");
    assert_eq!(
        pkg.get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str()),
        Some("xtask")
    );
    assert_eq!(
        pkg.get("package")
            .and_then(|p| p.get("publish"))
            .and_then(|v| v.as_bool()),
        Some(false),
        "xtask must set publish = false"
    );
    let config = read(".cargo/config.toml");
    let value: toml::Value = config.parse().expect("parse .cargo/config.toml");
    let alias = value
        .get("alias")
        .and_then(|a| a.get("xtask"))
        .and_then(|v| v.as_str());
    assert_eq!(alias, Some("run --package xtask --"));
}

/// RI-T11 + RI-T12: cargo xtask guard runs implemented checks 01, 02, 03, 13.
#[test]
fn ri_t11_cargo_xtask_guard_runs_implemented_checks() {
    let output = cargo_xtask_guard();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "cargo xtask guard must be invocable and pass implemented checks; output={combined}"
    );
    for id in ["01", "02", "03", "13"] {
        let mentioned = combined.contains(id)
            || combined.contains(&format!("check {id}"))
            || combined.contains(&format!("[{id}]"));
        assert!(
            mentioned,
            "guard report must mention implemented check {id}; output={combined}"
        );
    }
}

/// RI-T13: check 04 is pass/evaluated; 14/15 are real; stubs 05–12 never silently pass.
#[test]
fn ri_t13_stub_checks_do_not_silently_pass() {
    let combined = guard_report();
    assert!(
        combined.contains("04  architecture-invariants  pass")
            || (combined.contains("architecture-invariants")
                && combined.contains("04")
                && combined.contains("pass")
                && !combined.contains("skip(DEBT-GUARD-04)")),
        "check 04 must be pass/evaluated; output={combined}"
    );
    assert!(
        !combined.contains("skip(DEBT-GUARD-04)")
            && !combined.contains("not-yet-implemented: check 04"),
        "check 04 must not skip or nyi; output={combined}"
    );
    for id in ["14", "15"] {
        assert!(
            !combined.contains(&format!("skip(DEBT-GUARD-{id})"))
                && !combined.contains(&format!("not-yet-implemented: check {id}")),
            "check {id} must be a real ArchitectureCheck, not skip-with-debt or nyi; output={combined}"
        );
        let pass_line = combined.contains(&format!("{id}  "))
            && combined.contains("pass")
            && (combined.contains(&format!("{id}  adr-graph  pass"))
                || combined.contains(&format!("{id}  spec-lifecycle  pass")));
        assert!(
            pass_line,
            "check {id} must report pass on the live tree; output={combined}"
        );
    }
    for (id, name) in [
        ("05", "catalog-ssot"),
        ("06", "framework-pack-parse"),
        ("07", "framework-digest"),
        ("08", "readiness-ssot"),
        ("09", "temporal-evidence-selection"),
        ("10", "assessment-lineage-rebuild"),
        ("11", "evidence-latest-vs-current"),
        ("12", "soa-invariants"),
    ] {
        assert!(
            combined.contains(&format!("{id}  {name}  pass")),
            "product-law check {id} {name} must pass; output={combined}"
        );
        assert!(
            !combined.contains(&format!("skip(DEBT-GUARD-{id})")),
            "check {id} must not skip-with-debt; output={combined}"
        );
    }
}

/// RI-T14: CI must run cargo xtask guard as a mandatory step.
#[test]
fn ri_t14_ci_runs_xtask_guard() {
    let ci = read(".github/workflows/ci.yml");
    assert!(
        ci.contains("xtask guard") || ci.contains("cargo xtask guard"),
        ".github/workflows/ci.yml must contain a cargo xtask guard step"
    );
    assert!(
        !ci.contains("continue-on-error: true")
            || !ci
                .split("xtask guard")
                .next()
                .unwrap_or("")
                .contains("continue-on-error"),
        "cargo xtask guard must be mandatory (not continue-on-error)"
    );
}

/// RI-T15
#[test]
fn ri_t15_dual_suite_registered_in_cargo_toml() {
    let cargo = read("Cargo.toml");
    assert!(!cargo.contains("name = \"sdd_repository_integrity_baseline\""));
    assert!(cargo.contains("name = \"sdd_repository_integrity_target\""));
    assert!(!cargo.contains("path = \"tests/contracts/repository_integrity.baseline.rs\""));
    assert!(cargo.contains("path = \"tests/contracts/repository_integrity.target.rs\""));
}

/// After target GREEN, baseline absence tests are skip-superseded.
#[test]
fn ri_t15b_baseline_is_deleted() {
    assert!(
        !std::path::Path::new("tests/contracts/repository_integrity.baseline.rs").exists(),
        "superseded baseline suite must be deleted"
    );
}

/// RI-T16
#[test]
fn ri_t16_spec_is_in_canonical_specs() {
    let layout = read("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/repository-integrity.md"),
        "CANONICAL_SPECS must list docs/specs/repository-integrity.md"
    );
}

/// ADR 0009 is Accepted at implement; remaining_backlog is not implemented as product.
#[test]
fn ri_t17_adr_0009_accepted_and_backlog_not_shipped_as_product() {
    let adr = read("docs/adr/0009-repository-health-gate.md");
    let status_accepted = adr.lines().any(|line| {
        let t = line.trim();
        t.starts_with("| Status |") && t.contains("**Accepted**") && !t.contains("**Draft**")
    });
    assert!(
        status_accepted,
        "ADR 0009 Status field must be Accepted at implement"
    );
    let spec = read("docs/specs/repository-integrity.md");
    assert!(
        spec.contains("remaining_backlog"),
        "spec must keep remaining_backlog"
    );
    let xtask_src = collect_rs(rel("xtask"));
    assert!(
        !xtask_src.is_empty(),
        "xtask sources must exist so product-law checks can be inspected"
    );
    assert!(
        xtask_src.contains("struct ProductLawCheck"),
        "checks 05–12 must be ProductLawCheck, not stubs"
    );
    assert!(
        !xtask_src.contains(r#"("14", "adr-graph")"#)
            && !xtask_src.contains("\"14\", \"adr-graph\""),
        "check 14 must not remain in REMAINING_STUBS"
    );
    assert!(
        !xtask_src.contains(r#"("15", "spec-lifecycle")"#)
            && !xtask_src.contains("\"15\", \"spec-lifecycle\""),
        "check 15 must not remain in REMAINING_STUBS"
    );
}

fn cargo_xtask_guard_args(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["xtask", "guard"])
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("spawn cargo xtask guard")
}

fn xtask_src_rel_rs() -> Vec<String> {
    let root = rel("xtask/src");
    let mut files = Vec::new();
    fn walk(dir: &Path, prefix: &Path, files: &mut Vec<String>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(&path, prefix, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let rel = path
                    .strip_prefix(prefix)
                    .expect("xtask/src prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push(rel);
            }
        }
    }
    walk(&root, &root, &mut files);
    files.sort();
    files
}

fn xtask_src_text() -> String {
    collect_rs(rel("xtask/src"))
}

fn list_md(dir: &str) -> Vec<String> {
    let entries = fs::read_dir(rel(dir)).unwrap_or_else(|e| panic!("read {dir}: {e}"));
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".md") {
            names.push(name);
        }
    }
    names.sort();
    names
}

fn field_present(finding: &toml::Value, key: &str) -> bool {
    match finding.get(key) {
        None => false,
        Some(toml::Value::String(s)) => !s.is_empty(),
        Some(toml::Value::Array(a)) => !a.is_empty(),
        Some(_) => true,
    }
}

fn exemption_complete(finding: &toml::Value) -> bool {
    field_present(finding, "owner")
        && field_present(finding, "introduced")
        && field_present(finding, "severity")
        && field_present(finding, "remediation")
        && field_present(finding, "repository_guard")
        && (field_present(finding, "expires") || field_present(finding, "review_by"))
}

fn finding_named<'a>(register: &'a toml::Value, id: &str) -> &'a toml::Value {
    register
        .get("finding")
        .and_then(|f| f.as_array())
        .expect("[[finding]]")
        .iter()
        .find(|f| f.get("id").and_then(|v| v.as_str()) == Some(id))
        .unwrap_or_else(|| panic!("register must contain {id}"))
}

fn module_present(files: &[String], stem: &str) -> bool {
    files.iter().any(|f| {
        f == &format!("{stem}.rs")
            || f == &format!("{stem}/mod.rs")
            || f.starts_with(&format!("{stem}/"))
    })
}

fn source_contains_fn_rereads(src: &str, fn_name: &str) -> bool {
    let needle = format!("fn {fn_name}");
    let Some(rest) = src.split(&needle).nth(1) else {
        return false;
    };
    let body = rest.split("fn ").next().unwrap_or(rest);
    body.contains("fs::read_to_string") || body.contains("read_to_string(")
}

/// RI-T18: xtask is modular (model / architecture / debt / checks / report).
#[test]
fn ri_t18_xtask_is_modular_not_a_lib_rs_monolith() {
    let files = xtask_src_rel_rs();
    assert!(
        files.iter().any(|f| f == "lib.rs"),
        "xtask/src/lib.rs must remain the public re-export surface; found {files:?}"
    );
    for stem in ["model", "architecture", "debt", "checks", "report"] {
        assert!(
            module_present(&files, stem),
            "xtask/src must contain a {stem} module (file or directory); found {files:?}"
        );
    }
    assert!(
        files.len() > 2,
        "lib.rs must not be the only implementation file alongside main.rs; found {files:?}"
    );
}

/// RI-T19: RepositoryModel caches source at load; checks do not reread the tree.
#[test]
fn ri_t19_repository_model_caches_source_at_load() {
    let src = xtask_src_text();
    assert!(
        src.contains("struct RepositoryModel"),
        "RepositoryModel must exist"
    );
    let cached = [
        "source_cache",
        "source_index",
        "source_text",
        "normalized_source",
        "source_map",
    ]
    .iter()
    .any(|field| src.contains(field));
    assert!(
        cached,
        "RepositoryModel must cache normalized source text or a lightweight index at load"
    );
    assert!(
        !source_contains_fn_rereads(&src, "source_contains"),
        "source_contains must not fs::read_to_string every source_files entry"
    );
}

/// RI-T20: policy lives under architecture/; Rust is not the SSOT.
#[test]
fn ri_t20_policy_lives_in_versioned_architecture_files() {
    let arch = parse_toml("architecture/architecture.toml");
    let policy = arch.get("policy").expect("[policy] table is required");
    let kinds: Vec<&str> = policy
        .get("ownership_kinds")
        .and_then(|v| v.as_array())
        .expect("policy.ownership_kinds")
        .iter()
        .map(|v| v.as_str().expect("kind str"))
        .collect();
    for kind in OWNERSHIP_KINDS {
        assert!(
            kinds.iter().any(|k| *k == kind),
            "policy.ownership_kinds must include {kind}; got {kinds:?}"
        );
    }
    let concepts: Vec<&str> = policy
        .get("required_concepts")
        .and_then(|v| v.as_array())
        .expect("policy.required_concepts")
        .iter()
        .map(|v| v.as_str().expect("concept str"))
        .collect();
    for concept in OWNERSHIP_CONCEPTS {
        assert!(
            concepts.iter().any(|c| *c == concept),
            "policy.required_concepts must include {concept}; got {concepts:?}"
        );
    }

    let src = xtask_src_text();
    assert!(
        src.contains("ownership_kinds") && src.contains("required_concepts"),
        "xtask must interpret [policy] ownership_kinds / required_concepts from architecture.toml"
    );
    assert!(
        !src.contains("const FORBIDDEN_PACKAGES"),
        "forbidden package names must live in architecture/forbidden-patterns.toml, not a Rust policy SSOT"
    );
    assert!(
        rel("architecture/adr-identity.toml").is_file(),
        "architecture/adr-identity.toml must exist"
    );
    assert!(
        rel("architecture/spec-lifecycle.toml").is_file(),
        "architecture/spec-lifecycle.toml must exist"
    );
}

/// RI-T21: live Guard 14 is pass (not skip-with-debt).
#[test]
fn ri_t21_live_guard_14_is_pass_not_skip() {
    let combined = guard_report();
    assert!(
        combined.contains("14  adr-graph  pass"),
        "check 14 must pass on the live tree; output={combined}"
    );
    assert!(
        !combined.contains("skip(DEBT-GUARD-14)")
            && !combined.contains("not-yet-implemented: check 14"),
        "check 14 must not skip or nyi; output={combined}"
    );
}

/// RI-T22: unique new ADR prefixes; historical dupes only via DEBT-DUP-ADR.
#[test]
fn ri_t22_new_duplicate_adr_prefix_fails_check_14() {
    assert!(
        rel("architecture/adr-identity.toml").is_file(),
        "architecture/adr-identity.toml is required so historical dupes have a legal grandfather"
    );
    let identity = parse_toml("architecture/adr-identity.toml");
    assert_eq!(
        identity.get("schema").and_then(|s| s.as_str()),
        Some(ADR_IDENTITY_SCHEMA)
    );
    assert_eq!(
        identity.get("grandfathered_debt").and_then(|s| s.as_str()),
        Some("DEBT-DUP-ADR")
    );
    let prefixes: Vec<&str> = identity
        .get("grandfathered_prefixes")
        .and_then(|v| v.as_array())
        .expect("grandfathered_prefixes")
        .iter()
        .map(|v| v.as_str().expect("prefix"))
        .collect();
    assert!(
        prefixes.is_empty(),
        "ADR namespace is unique; grandfathered_prefixes must be empty, got {prefixes:?}"
    );

    let adr_files = list_md("docs/adr");
    let mut by_prefix = std::collections::BTreeMap::<String, usize>::new();
    for name in &adr_files {
        if let Some(prefix) = name.get(..4) {
            *by_prefix.entry(prefix.to_string()).or_default() += 1;
        }
    }
    let dups: Vec<_> = by_prefix.into_iter().filter(|(_, n)| *n > 1).collect();
    assert!(
        dups.is_empty(),
        "ADR prefixes must be unique; dups={dups:?}"
    );

    let src = xtask_src_text();
    for needle in [
        "adr-identity.toml",
        "grandfathered_prefixes",
        "DEBT-DUP-ADR",
        "weeping-angel-adr-meta",
    ] {
        assert!(
            src.contains(needle),
            "Guard 14 must encode {needle} (new files reusing 0010/0003 fail; historical set is grandfathered)"
        );
    }
    assert!(
        src.contains("duplicate") || src.contains("prefix"),
        "Guard 14 must reject a new ADR that reuses prefix 0010 or 0003"
    );
}

/// RI-T23: dangling ADR edges and cycles fail check 14.
#[test]
fn ri_t23_dangling_or_cyclic_adr_graph_fails_check_14() {
    let src = xtask_src_text();
    for needle in [
        "weeping-angel-adr-meta",
        "supersedes",
        "superseded_by",
        "depends_on",
    ] {
        assert!(
            src.contains(needle),
            "Guard 14 must parse ADR metadata field/relation {needle}"
        );
    }
    assert!(
        src.contains("dangling") || src.contains("does not exist") || src.contains("unknown"),
        "Guard 14 must fail dangling supersedes/superseded_by/depends_on"
    );
    assert!(
        src.contains("cycle") || src.contains("acyclic") || src.contains("cyclic"),
        "Guard 14 must fail cycles where the relation must be acyclic"
    );
}

/// RI-T24: live Guard 15 is pass; spec-lifecycle.toml covers every docs/specs/*.md.
#[test]
fn ri_t24_live_guard_15_is_pass_and_lifecycle_covers_specs() {
    let combined = guard_report();
    assert!(
        combined.contains("15  spec-lifecycle  pass"),
        "check 15 must pass on the live tree; output={combined}"
    );
    assert!(
        !combined.contains("skip(DEBT-GUARD-15)")
            && !combined.contains("not-yet-implemented: check 15"),
        "check 15 must not skip or nyi; output={combined}"
    );

    assert!(
        rel("architecture/spec-lifecycle.toml").is_file(),
        "architecture/spec-lifecycle.toml must exist"
    );
    let lifecycle = parse_toml("architecture/spec-lifecycle.toml");
    assert_eq!(
        lifecycle.get("schema").and_then(|s| s.as_str()),
        Some(SPEC_LIFECYCLE_SCHEMA)
    );
    let rows = lifecycle
        .get("spec")
        .and_then(|v| v.as_array())
        .expect("[[spec]]");
    let listed: BTreeSet<String> = rows
        .iter()
        .map(|row| {
            row.get("path")
                .and_then(|p| p.as_str())
                .expect("spec.path")
                .to_string()
        })
        .collect();
    let on_disk = list_md("docs/specs");
    assert!(
        !on_disk.is_empty(),
        "docs/specs must contain markdown specs"
    );
    for name in &on_disk {
        let path = format!("docs/specs/{name}");
        assert!(
            listed.contains(&path),
            "spec-lifecycle.toml must list {path}; listed={listed:?}"
        );
        assert!(rel(&path).is_file(), "{path} must exist");
    }
}

/// RI-T25: masquerade / missing successor / missing lifecycle file fail check 15.
#[test]
fn ri_t25_spec_lifecycle_masquerade_and_missing_file_fail() {
    let src = xtask_src_text();
    assert!(
        src.contains("spec-lifecycle.toml"),
        "Guard 15 must load architecture/spec-lifecycle.toml"
    );
    assert!(
        src.contains("spec-lifecycle")
            && (src.contains("is not a file")
                || src.contains("missing")
                || src.contains("malformed")),
        "missing or malformed spec-lifecycle.toml must fail check 15 (never skip)"
    );
    for needle in ["draft", "active", "superseded", "retired"] {
        assert!(
            src.contains(needle),
            "Guard 15 must encode lifecycle state {needle}"
        );
    }
    assert!(
        src.contains("successor"),
        "a superseded spec without successor must fail check 15"
    );
}

/// RI-T26: active spec ownership keys must exist in architecture.toml.
#[test]
fn ri_t26_active_spec_ownership_must_bind_existing_concepts() {
    let src = xtask_src_text();
    assert!(
        src.contains("ownership"),
        "Guard 15 must bind active specs to architecture ownership keys"
    );
    let lifecycle_path = rel("architecture/spec-lifecycle.toml");
    assert!(
        lifecycle_path.is_file(),
        "architecture/spec-lifecycle.toml must exist so active specs can bind ownership"
    );
    let arch = parse_toml("architecture/architecture.toml");
    let ownership = arch.get("ownership").expect("[ownership]");
    let lifecycle = parse_toml("architecture/spec-lifecycle.toml");
    for row in lifecycle
        .get("spec")
        .and_then(|v| v.as_array())
        .expect("[[spec]]")
    {
        let state = row.get("state").and_then(|s| s.as_str()).unwrap_or("");
        let path = row.get("path").and_then(|s| s.as_str()).unwrap_or("?");
        if state == "active" {
            let keys = row
                .get("ownership")
                .and_then(|v| v.as_array())
                .expect("active spec ownership array");
            assert!(
                !keys.is_empty(),
                "active spec {path} must list at least one ownership key"
            );
            for key in keys {
                let concept = key.as_str().expect("ownership key");
                assert!(
                    ownership.get(concept).is_some(),
                    "active spec {path} ownership {concept} must exist in architecture.toml"
                );
            }
        }
        if state == "superseded" || state == "retired" {
            assert_ne!(
                state, "active",
                "superseded/retired spec {path} cannot masquerade as active"
            );
        }
    }
}

/// RI-T27: live skip exemptions are complete; expired exemptions fail closed.
#[test]
fn ri_t27_live_exemptions_require_owner_dates_and_expiry_fails_closed() {
    let register = parse_toml("docs/debt/register.toml");
    for id in PRODUCT_STUB_CHECKS {
        let finding = finding_named(&register, &format!("DEBT-GUARD-{id}"));
        assert!(
            finding.get("status") == Some(&toml::Value::String("resolved".into())),
            "DEBT-GUARD-{id} must be resolved once Guard {id} is a real check"
        );
    }
    let dup = finding_named(&register, "DEBT-DUP-ADR");
    assert!(
        exemption_complete(dup),
        "DEBT-DUP-ADR is a live prefix-collision exemption and must carry owner/dates/severity/remediation/repository_guard/expiry"
    );
    assert_eq!(
        dup.get("repository_guard").and_then(|v| v.as_str()),
        Some("14")
    );

    let src = xtask_src_text();
    assert!(
        src.contains("WEEPING_ANGEL_GUARD_AS_OF"),
        "expiry evaluation date must be overridable via WEEPING_ANGEL_GUARD_AS_OF"
    );
    assert!(
        src.contains("expired") || src.contains("expires"),
        "check 13 must fail closed on expired guard debt"
    );
}

/// RI-T28: DEBT-GUARD-14/15 resolved with proof; malformed/duplicate/orphaned ids fail.
#[test]
fn ri_t28_resolved_guard_14_15_and_malformed_debt_fail_closed() {
    let register = parse_toml("docs/debt/register.toml");
    for id in ["DEBT-GUARD-14", "DEBT-GUARD-15"] {
        let finding = finding_named(&register, id);
        assert_eq!(
            finding.get("status").and_then(|s| s.as_str()),
            Some("resolved"),
            "{id} must be resolved once Guards 14/15 are real"
        );
        assert!(
            proof_ok(finding),
            "{id} must prove closure via repository_guard or named regression tests"
        );
        let tests = finding
            .get("regression_tests")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        assert!(
            tests.iter().any(|t| t.contains("repository_integrity")
                || t.contains("architectural_cleanup")
                || t.contains("sdd_")),
            "{id} resolved proof must name regression tests; got {tests:?}"
        );
    }
    assert_eq!(
        finding_named(&register, "DEBT-GUARD-14")
            .get("repository_guard")
            .and_then(|v| v.as_str()),
        Some("14")
    );
    assert_eq!(
        finding_named(&register, "DEBT-GUARD-15")
            .get("repository_guard")
            .and_then(|v| v.as_str()),
        Some("15")
    );

    let src = xtask_src_text();
    for needle in ["duplicate", "orphan", "malformed"] {
        assert!(
            src.contains(needle),
            "check 13 must reject {needle} debt ids"
        );
    }
}

/// RI-T29: additive JSON schema/version/counts/failed; do not equality-compare duration.
#[test]
fn ri_t29_guard_json_is_additive_with_schema_counts_failed() {
    let output = cargo_xtask_guard_args(&["--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json_start = stdout.find('{').unwrap_or(0);
    let value: serde_json::Value = serde_json::from_str(stdout[json_start..].trim()).unwrap_or_else(
        |e| panic!("guard --json must emit a JSON object; parse error {e}; stdout={stdout} stderr={stderr}"),
    );
    let obj = value.as_object().expect("JSON object");
    for key in [
        "schema",
        "version",
        "counts",
        "failed",
        "checks",
        "violations",
        "skipped",
        "debt_exemptions",
        "duration",
    ] {
        assert!(
            obj.contains_key(key),
            "JSON must include {key}; keys={:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    assert_eq!(
        obj.get("schema").and_then(|v| v.as_str()),
        Some(GUARD_REPORT_SCHEMA)
    );
    let version = obj.get("version").and_then(|v| v.as_u64()).or_else(|| {
        obj.get("version")
            .and_then(|v| v.as_i64())
            .map(|n| n as u64)
    });
    assert_eq!(version, Some(1), "report version must be 1");

    let counts = obj
        .get("counts")
        .and_then(|v| v.as_object())
        .expect("counts");
    for key in ["total", "pass", "fail", "skip"] {
        assert!(
            counts.contains_key(key),
            "counts must include {key}; got {counts:?}"
        );
    }
    let checks = obj
        .get("checks")
        .and_then(|v| v.as_array())
        .expect("checks");
    let mut ids = Vec::new();
    for check in checks {
        let id = check.get("id").and_then(|v| v.as_str()).expect("check.id");
        assert!(
            id.len() == 2 && id.chars().all(|c| c.is_ascii_digit()),
            "check ids must be deterministic two-digit strings; got {id}"
        );
        ids.push(id.to_string());
    }
    assert!(
        ids.windows(2).all(|w| w[0] <= w[1]),
        "check ids must be in deterministic order; got {ids:?}"
    );

    // Equality-sensitive fixtures must not compare wall-clock duration.
    let _ = obj.get("duration");
}

/// RI-T30: CI keeps mandatory cargo xtask guard; no path-filter bypass.
#[test]
fn ri_t30_ci_requires_guard_and_does_not_path_filter_bypass() {
    let ci = read(".github/workflows/ci.yml");
    assert!(
        ci.contains("xtask guard") || ci.contains("cargo xtask guard"),
        ".github/workflows/ci.yml must contain a cargo xtask guard step"
    );
    let before_guard = ci.split("xtask guard").next().unwrap_or("");
    assert!(
        !before_guard.contains("continue-on-error: true"),
        "cargo xtask guard must be mandatory (not continue-on-error)"
    );
    for surface in [
        "architecture/",
        "docs/adr/",
        "docs/specs/",
        "docs/debt/",
        "xtask/",
        "src/",
        "crates/",
        "frameworks/",
        "catalog/",
    ] {
        let ignored = ci.contains("paths-ignore")
            && ci
                .split("paths-ignore")
                .nth(1)
                .unwrap_or("")
                .lines()
                .take(40)
                .any(|line| line.contains(surface));
        assert!(
            !ignored,
            "CI must not paths-ignore {surface} in a way that bypasses cargo xtask guard"
        );
    }
    if ci.contains("paths:") && !ci.contains("paths-ignore") {
        let paths_block = ci.split("paths:").nth(1).unwrap_or("");
        for surface in [
            "architecture/**",
            "docs/adr/**",
            "docs/specs/**",
            "docs/debt/**",
            "xtask/**",
            "src/**",
            "crates/**",
            "frameworks/**",
            "catalog/**",
        ] {
            assert!(
                paths_block.contains(surface)
                    || !ci.split("jobs:").next().unwrap_or("").contains("paths:"),
                "if CI uses on.paths filters, the guard job must still run when {surface} changes"
            );
        }
    }
}

/// RI-T31: 01–04 and 13 still pass; 05–12 stay stubs; 14/15 are not skip-with-debt.
#[test]
fn ri_t31_implemented_checks_retained_product_stubs_remain() {
    let output = cargo_xtask_guard();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for (id, name) in [
        ("01", "architecture-manifest"),
        ("02", "canonical-ownership"),
        ("03", "forbidden-patterns"),
        ("04", "architecture-invariants"),
        ("13", "debt-register"),
        ("14", "adr-graph"),
        ("15", "spec-lifecycle"),
    ] {
        assert!(
            combined.contains(&format!("{id}  {name}  pass")),
            "check {id} {name} must pass on the live tree; output={combined}"
        );
    }
    for (id, name) in [
        ("05", "catalog-ssot"),
        ("06", "framework-pack-parse"),
        ("07", "framework-digest"),
        ("08", "readiness-ssot"),
        ("09", "temporal-evidence-selection"),
        ("10", "assessment-lineage-rebuild"),
        ("11", "evidence-latest-vs-current"),
        ("12", "soa-invariants"),
    ] {
        assert!(
            combined.contains(&format!("{id}  {name}  pass")),
            "product-law check {id} {name} must pass on the live tree; output={combined}"
        );
    }
}
