//! Target suite for Repository Integrity increment 1 (health gate).
//!
//! Encodes DESIRED behavior from `docs/specs/repository-integrity.md` §4 / §5
//! (RI-T01–T16 + remaining acceptance criteria). Must stay RED on CURRENT
//! HEAD: no `architecture/` manifests, no `docs/debt/` register, no `xtask`
//! member, CI does not run `cargo xtask guard`. Do not implement the gate
//! in this suite; do not weaken assertions to match characterization absences.

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

const STUB_CHECKS: [&str; 10] = ["05", "06", "07", "08", "09", "10", "11", "12", "14", "15"];

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
    for id in STUB_CHECKS {
        let finding = format!("DEBT-GUARD-{id}");
        assert!(
            ids.contains(finding.as_str()),
            "register must contain {finding} so stub skips are attributable"
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

/// RI-T13: check 04 is pass/evaluated; stubs 05–12 and 14–15 never silently pass.
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
    for id in STUB_CHECKS {
        let skip = combined.contains(&format!("DEBT-GUARD-{id}"))
            || combined.contains(&format!("skip(DEBT-GUARD-{id})"))
            || combined.contains(&format!("skip (DEBT-GUARD-{id})"));
        let nyi = combined.contains(&format!("not-yet-implemented: check {id}"))
            || combined.contains(&format!("not-yet-implemented: check {id}"));
        assert!(
            skip || nyi,
            "stub check {id} must skip-with-debt or fail closed; output={combined}"
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
    assert!(cargo.contains("name = \"sdd_repository_integrity_baseline\""));
    assert!(cargo.contains("name = \"sdd_repository_integrity_target\""));
    assert!(cargo.contains("path = \"tests/contracts/repository_integrity.baseline.rs\""));
    assert!(cargo.contains("path = \"tests/contracts/repository_integrity.target.rs\""));
}

/// After target GREEN, baseline absence tests are skip-superseded.
#[test]
fn ri_t15b_baseline_is_ignore_superseded() {
    let baseline = read("tests/contracts/repository_integrity.baseline.rs");
    assert!(
        baseline.contains("#[ignore = \"superseded by sdd_repository_integrity_target\"]")
            || baseline.contains("#[ignore = \"superseded by sdd_repository_integrity_target\"]"),
        "baseline suite must be ignore-superseded after target GREEN"
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
    // Guard checks 05–12 / 14–15 stay stubs (fail-closed or skip-with-debt), not real P0 remediations. Check 04 is ADR 0010.
    let xtask_src = collect_rs(rel("xtask"));
    assert!(
        !xtask_src.is_empty(),
        "xtask sources must exist so remaining_backlog stubs can be inspected"
    );
    for id in STUB_CHECKS {
        let implemented_as_real = xtask_src.contains(&format!("check {id} implemented"))
            && !xtask_src.contains("not-yet-implemented")
            && !xtask_src.contains("DEBT-GUARD-");
        assert!(
            !implemented_as_real,
            "check {id} must remain a stub this slice"
        );
    }
}
