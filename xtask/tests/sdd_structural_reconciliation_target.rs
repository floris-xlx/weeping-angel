//! Target suite for Structural Reconciliation Phase 0+1.
//!
//! Encodes SR-T01–T15 desired acceptance. Must FAIL (RED) until SSOT honesty
//! gaps close (ADR Accepted in header, §5 checkboxes, post-implement current
//! plane) and Phase 0+1 contract remains enforced. Do not change product code
//! in this suite.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;
use xtask::{
    CheckStatus, InventoryReport, main_with_args, repo_root_from_xtask_manifest, run_guard,
};

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn xtask_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xtask"));
    cmd.current_dir(live_root());
    cmd
}

/// SR-T01: inventory module exists; inventory CLI succeeds for json/markdown/check.
#[test]
fn sr_t01_inventory_module_and_cli_succeed() {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/inventory.rs");
    assert!(module.is_file(), "xtask/src/inventory.rs must exist");
    assert_eq!(main_with_args(["inventory"]), 0);
    assert_eq!(main_with_args(["inventory", "--json"]), 0);
    assert_eq!(main_with_args(["inventory", "--markdown"]), 0);
    assert_eq!(main_with_args(["inventory", "--check"]), 0);
}

/// SR-T02: --json includes schema + required counts + exclusions.
#[test]
fn sr_t02_json_schema_counts_exclusions() {
    let output = xtask_bin()
        .args(["inventory", "--json"])
        .output()
        .expect("spawn inventory --json");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("weeping-angel/inventory/v1"));
    for key in [
        "root_test_binaries",
        "tests_rs_autodiscovered",
        "tests_contracts_rs",
        "ignored_test_attrs",
        "unwrap_calls",
        "expect_calls",
        "unwrap_plus_expect",
        "require_needles_fns",
        "require_needles_calls",
        "adr_markdown_files",
        "catalog_test_toml",
        "framework_packs",
        "schema_json_files",
        "exclusions",
        "absences",
        "inventory_module",
        "debt_current_md",
        "structural_reconciliation_spec",
    ] {
        assert!(
            stdout.contains(key),
            "inventory --json must include {key}: {stdout}"
        );
    }
    assert!(stdout.contains("target/") && stdout.contains("node_modules/"));
    let report = InventoryReport::collect(&live_root());
    assert_eq!(report.counts.framework_packs, 2);
    assert!(!report.absences.inventory_module);
    assert!(!report.absences.debt_current_md);
    assert!(!report.absences.structural_reconciliation_spec);
}

/// SR-T03: --markdown matches committed docs/debt/current.md stable sections.
#[test]
fn sr_t03_markdown_matches_current_md() {
    let report = InventoryReport::collect(&live_root());
    report
        .check_current_md(&live_root())
        .expect("current.md must match inventory --markdown");
    let md = read_live("docs/debt/current.md");
    assert!(md.contains("current") && md.contains("Historical"));
    assert!(md.contains("baseline-2026-08.md"));
}

/// SR-T04: --check exit 0 on synced tree; exit 1 if current.md counts tampered.
#[test]
fn sr_t04_check_detects_tamper() {
    assert_eq!(main_with_args(["inventory", "--check"]), 0);

    let dir = tempdir().unwrap();
    let root = dir.path();
    // Minimal tree with a mismatched current.md.
    fs::create_dir_all(root.join("docs/debt")).unwrap();
    fs::create_dir_all(root.join("docs/adr")).unwrap();
    fs::create_dir_all(root.join("docs/specs")).unwrap();
    fs::create_dir_all(root.join("catalog/canonical/v1/tests")).unwrap();
    fs::create_dir_all(root.join("frameworks/iso-27001/2022")).unwrap();
    fs::create_dir_all(root.join("frameworks/wa-baseline/1")).unwrap();
    fs::create_dir_all(root.join("tests/contracts")).unwrap();
    fs::create_dir_all(root.join("xtask/src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
    fs::write(root.join("xtask/src/inventory.rs"), "").unwrap();
    fs::write(
        root.join("docs/specs/structural-reconciliation.md"),
        "# spec\n",
    )
    .unwrap();
    fs::write(root.join("frameworks/iso-27001/2022/manifest.toml"), "").unwrap();
    fs::write(root.join("frameworks/wa-baseline/1/manifest.toml"), "").unwrap();
    fs::write(
        root.join("docs/debt/current.md"),
        "# Current repository counts (mechanical)\n\nTAMPERED COUNTS TABLE\n",
    )
    .unwrap();
    let report = InventoryReport::collect(root);
    assert!(
        report.check_current_md(root).is_err(),
        "tampered current.md must fail --check"
    );
}

/// SR-T05: baseline-2026-08 Historical; README points at current.md.
#[test]
fn sr_t05_baseline_historical_readme_points_current() {
    let baseline = read_live("docs/debt/baseline-2026-08.md");
    let title = baseline.lines().next().unwrap_or("");
    assert!(
        title.to_ascii_lowercase().contains("historical"),
        "baseline-2026-08.md must carry Historical marker: {title}"
    );
    let readme = read_live("docs/debt/README.md");
    assert!(
        readme.contains("current.md"),
        "debt README must point at current.md"
    );
    assert!(
        readme.to_ascii_lowercase().contains("historical"),
        "debt README must mark baseline as Historical"
    );
}

/// SR-T06: active RI header does not claim 05–12 stub/skip; archaeology Historical.
#[test]
fn sr_t06_active_ri_matches_live_guards() {
    let text = read_live("docs/specs/repository-integrity.md");
    let header: String = text
        .lines()
        .take_while(|l| !l.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !(header.contains("skip-with-debt")
            && (header.contains("05–12") || header.contains("05-12"))),
        "active RI header must not claim 05–12 skip-with-debt:\n{header}"
    );
    assert!(
        !(header.contains("stay stubs") && (header.contains("05–12") || header.contains("05-12"))),
        "active RI header must not claim 05–12 stay stubs:\n{header}"
    );
    assert!(
        header.contains("01–15") || header.contains("01-15") || header.contains("ProductLawCheck"),
        "active RI header must describe live 01–15 / ProductLawCheck plane"
    );
    assert!(
        text.contains("Historical"),
        "RI must retain an explicit Historical fence for archaeology"
    );
}

/// SR-T07: active-spec drift fails on banned active phrases; Historical-fenced ok.
#[test]
fn sr_t07_active_spec_drift_guard() {
    use xtask::{active_spec_drift_in_text, check_active_spec_drift};

    assert!(
        check_active_spec_drift(&live_root()).is_ok(),
        "live active specs must pass drift check"
    );

    let banned = r#"# Drift fixture

| Field | Value |
| --- | --- |
| Collision fence | Guards **05–12** stay stubs |
| Increment-2 current plane | **05–12** skip-with-debt |
"#;
    assert!(
        active_spec_drift_in_text(banned).is_some(),
        "banned active phrases must fail drift: {:?}",
        active_spec_drift_in_text(banned)
    );

    let fenced = r#"# Drift fixture

| Field | Value |
| --- | --- |
| Collision fence | Guards **01–15** pass |

## Historical — old skip plane

Guards **05–12** stay stubs and skip-with-debt.
"#;
    assert!(
        active_spec_drift_in_text(fenced).is_none(),
        "Historical-fenced phrases must not fail drift"
    );

    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("architecture")).unwrap();
    fs::create_dir_all(root.join("docs/specs")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-canonical-catalog")).unwrap();
    fs::write(
        root.join("architecture/architecture.toml"),
        r#"schema = "weeping-angel/architecture/v1"
[policy]
ownership_kinds = ["exclusive"]
required_concepts = ["catalog"]
[ownership.catalog]
crate = "weeping-angel-canonical-catalog"
kind = "exclusive"
paths = ["crates/weeping-angel-canonical-catalog"]

[program.architectural_consolidation]
status = "active"
feature_expansion = "restricted"
allowed_change_classes = [
  "bug-fix",
  "security-fix",
  "consolidation",
  "non-semantic-collector",
  "consolidation-docs",
]
forbidden_change_classes = [
  "new-public-domain-type",
  "new-persistence-representation",
  "new-projection-path",
  "new-root-test-binary",
  "new-duplicated-helper",
  "new-compatibility-alias",
  "second-ssot",
]
"#,
    )
    .unwrap();
    fs::write(
        root.join("architecture/spec-lifecycle.toml"),
        r#"schema = "weeping-angel/spec-lifecycle/v1"

[[spec]]
path = "docs/specs/drift-fixture.md"
state = "active"
ownership = ["catalog"]
depends_on = []
supersedes = []
successor = ""
"#,
    )
    .unwrap();
    fs::write(root.join("docs/specs/drift-fixture.md"), banned).unwrap();
    let err = check_active_spec_drift(root);
    assert!(
        err.is_err(),
        "fixture tree with banned header must fail drift: {err:?}"
    );
}

/// SR-T08: live guard still 01–15 pass.
#[test]
fn sr_t08_live_guard_01_through_15_pass() {
    let report = run_guard(&live_root());
    let rendered = report.render();
    assert!(!report.failed(), "{rendered}");
    for id in [
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    ] {
        assert_eq!(
            report.checks.iter().find(|c| c.id == id).unwrap().status,
            CheckStatus::Pass,
            "{rendered}"
        );
    }
}

/// SR-T09: spec registered in CANONICAL_SPECS + spec-lifecycle.toml.
#[test]
fn sr_t09_spec_registered() {
    let layout = read_live("tests/contracts/documentation_layout.rs");
    assert!(layout.contains("docs/specs/structural-reconciliation.md"));
    let life = read_live("architecture/spec-lifecycle.toml");
    assert!(life.contains("docs/specs/structural-reconciliation.md"));
    assert!(life.contains("repository_guard"));
}

/// SR-T10: no tests/sdd/; target suite under xtask/tests/; baseline deleted.
#[test]
fn sr_t10_suites_under_xtask_tests() {
    let xtask_tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    assert!(
        !xtask_tests
            .join("sdd_structural_reconciliation_baseline.rs")
            .exists(),
        "superseded xtask baseline suite must be deleted"
    );
    assert!(
        xtask_tests
            .join("sdd_structural_reconciliation_target.rs")
            .is_file()
    );
    assert!(
        !live_root().join("tests/sdd").exists(),
        "tests/sdd/ is forbidden"
    );
}

fn ssot_header() -> String {
    let text = read_live("docs/specs/structural-reconciliation.md");
    text.lines()
        .take_while(|l| !l.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// SR-T11: ADR 0048 Accepted in ADR body/meta and SSOT header (not Draft).
#[test]
fn sr_t11_adr_0048_accepted_in_ssot_and_meta() {
    let adr = read_live("docs/adr/0048-structural-reconciliation.md");
    let adr_header: String = adr
        .lines()
        .take_while(|l| !l.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        adr_header.contains("**Accepted**") || adr_header.contains("| Status | Accepted"),
        "ADR 0048 Status must be Accepted:\n{adr_header}"
    );
    assert!(
        adr.contains("status = \"accepted\""),
        "ADR 0048 weeping-angel-adr-meta must set status = \"accepted\""
    );

    let header = ssot_header();
    let adr_row = header
        .lines()
        .find(|l| l.contains("| ADR |") || l.starts_with("| ADR |"))
        .unwrap_or("");
    assert!(
        adr_row.contains("**Accepted**") || adr_row.contains("Accepted"),
        "SSOT ADR field must say Accepted after target GREEN, not Draft:\n{adr_row}"
    );
    assert!(
        !adr_row.contains("**Draft**") && !adr_row.contains("Draft"),
        "SSOT ADR field must not remain Draft once Phase 1 is done:\n{adr_row}"
    );
}

/// SR-T12: §5 acceptance criteria checkboxes are checked after Phase 0+1.
#[test]
fn sr_t12_acceptance_criteria_checkboxes_complete() {
    let text = read_live("docs/specs/structural-reconciliation.md");
    let section = text
        .split("## 5. Acceptance criteria")
        .nth(1)
        .unwrap_or("")
        .split("\n## ")
        .next()
        .unwrap_or("");
    assert!(
        !section.trim().is_empty(),
        "SSOT must contain ## 5. Acceptance criteria"
    );
    let unchecked: Vec<&str> = section
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("- [ ]") || t.starts_with("* [ ]")
        })
        .collect();
    assert!(
        unchecked.is_empty(),
        "Phase 0+1 acceptance checkboxes must be marked done; still open:\n{}",
        unchecked.join("\n")
    );
    assert!(
        section.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("- [x]") || t.starts_with("- [X]") || t.starts_with("* [x]")
        }),
        "§5 must contain at least one completed checkbox"
    );
}

/// SR-T13: post-implement SSOT current plane must not claim inventory / current.md missing.
#[test]
fn sr_t13_ssot_current_plane_matches_live_inventory() {
    let header = ssot_header();
    let plane = header
        .lines()
        .find(|l| l.contains("Current plane"))
        .unwrap_or("");
    assert!(
        !plane.is_empty(),
        "SSOT header must retain a Current plane field"
    );
    // After Phase 1, Status is Implemented — current plane must not advertise found-case absences.
    assert!(
        !plane.contains("inventory.rs") || !plane.to_ascii_lowercase().contains("missing"),
        "Current plane must not claim xtask/src/inventory.rs missing after implement:\n{plane}"
    );
    assert!(
        !(plane.contains("current.md") && plane.to_ascii_lowercase().contains("missing")),
        "Current plane must not claim docs/debt/current.md missing after implement:\n{plane}"
    );
    assert!(
        !(plane.contains("accepts **only** `guard") || plane.contains("accepts **only** guard")),
        "Current plane must acknowledge inventory subcommand exists:\n{plane}"
    );
    assert!(
        plane.contains("inventory")
            && (plane.contains("01–15") || plane.contains("01-15") || plane.contains("pass")),
        "Current plane must describe live inventory + 01–15 pass plane:\n{plane}"
    );
}

/// SR-T14: Phase 0 freeze + exit criteria + inventory exit codes documented in SSOT.
#[test]
fn sr_t14_phase0_freeze_and_inventory_exit_codes_documented() {
    let text = read_live("docs/specs/structural-reconciliation.md");
    assert!(
        text.contains("### 4.1 Phase 0") || text.contains("Phase 0 — feature freeze"),
        "SSOT must document Phase 0 feature freeze"
    );
    assert!(
        text.contains("Exit criteria"),
        "SSOT must document Phase 0+1 exit criteria"
    );
    for banned_product in [
        "No new assurance frameworks",
        "collectors",
        "ISMS",
        "product scanners",
    ] {
        assert!(
            text.to_ascii_lowercase()
                .contains(&banned_product.to_ascii_lowercase())
                || text.contains(banned_product),
            "Phase 0 freeze must mention {banned_product}"
        );
    }
    // §4.2 exit codes: success 0, drift 1, usage 2.
    assert!(
        text.contains("exit **0**") || text.contains("exit 0"),
        "inventory success exit 0 must be documented"
    );
    assert!(
        text.contains("exit **1**") || text.contains("exit 1"),
        "inventory drift exit 1 must be documented"
    );
    assert!(
        text.contains("exit **2**") || text.contains("exit 2"),
        "inventory usage exit 2 must be documented"
    );
    assert_eq!(
        main_with_args(["inventory", "--json", "--check"]),
        2,
        "mutually exclusive inventory flags must exit 2"
    );
    assert_eq!(
        main_with_args(["inventory", "--markdown", "--json"]),
        2,
        "mutually exclusive inventory flags must exit 2"
    );
}

/// SR-T15: Phase 0 freeze not violated — still exactly two framework packs; no invented crates.
#[test]
fn sr_t15_phase0_freeze_not_violated() {
    let report = InventoryReport::collect(&live_root());
    assert_eq!(
        report.counts.framework_packs, 2,
        "Phase 0 freeze: no new framework packs (expected iso-27001 + wa-baseline)"
    );
    assert!(
        !live_root().join("crates/weeping-angel-catalog").exists(),
        "must not invent weeping-angel-catalog"
    );
    assert!(
        !live_root()
            .join("crates/weeping-angel-assurance-cli")
            .exists(),
        "must not invent weeping-angel-assurance-cli"
    );
    let exclusions = report.exclusions.join(" ");
    assert!(
        exclusions.contains("target/")
            && exclusions.contains("target-*")
            && exclusions.contains("node_modules/"),
        "exclusions must list target/, target-*, node_modules/: {exclusions}"
    );
}
