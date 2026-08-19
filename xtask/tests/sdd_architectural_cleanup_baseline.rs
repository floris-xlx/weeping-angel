//! Baseline suite for architectural-cleanup PROGRAM increment 1.
//!
//! Characterization of CURRENT `cargo xtask guard` (spec
//! `docs/specs/architectural-cleanup-program.md` §3): checks 01/02/03/13 are
//! real; 04–12 and 14–15 are `stub_check` → `skip(DEBT-GUARD-NN)`; check 03 is
//! schema presence only; ownership is `crate` + `paths`; `GuardReport` is
//! `{ checks }` + `render()`; CLI is `guard` only; no `RepositoryModel` /
//! `ArchitectureCheck`. Encodes ACP-B01–B06 (found/current case). Does **not**
//! implement Guard 04 evaluation, ownership kinds, forbidden-kind execution,
//! or structured CLI/report.
//!
//! Dedicated file so later implement can `#[ignore = "superseded by …"]`.

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use xtask::{
    CheckStatus, GuardReport, INVARIANTS_SCHEMA, REQUIRED_OWNERSHIP, check_02_ownership,
    check_03_forbidden_patterns, main_with_args, repo_root_from_xtask_manifest, run_guard,
};

const STUB_IDS: [&str; 11] = [
    "04", "05", "06", "07", "08", "09", "10", "11", "12", "14", "15",
];

fn write_minimal_repo(root: &Path, register: &str) {
    fs::create_dir_all(root.join("architecture")).unwrap();
    fs::create_dir_all(root.join("docs/debt")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-canonical-catalog")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-framework")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-assurance/src")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-evidence")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("architecture/architecture.toml"),
        r#"schema = "weeping-angel/architecture/v1"

[ownership.catalog]
crate = "weeping-angel-canonical-catalog"
paths = ["crates/weeping-angel-canonical-catalog"]

[ownership.framework_compilation]
crate = "weeping-angel-framework"
paths = ["crates/weeping-angel-framework"]

[ownership.readiness_projection]
crate = "weeping-angel-assurance"
paths = ["crates/weeping-angel-assurance/src/readiness.rs"]

[ownership.temporal_evidence_selection]
crate = "weeping-angel-assurance"
paths = ["crates/weeping-angel-assurance/src/temporal.rs"]

[ownership.assessment_lineage]
crate = "weeping-angel-assurance"
paths = ["crates/weeping-angel-assurance/src/lineage.rs"]

[ownership.evidence_persistence]
crate = "weeping-angel-evidence"
paths = ["crates/weeping-angel-evidence"]

[ownership.assurance_cli]
crate = "weeping-angel"
paths = ["src/main.rs", "src/cli.rs"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("architecture/forbidden-patterns.toml"),
        r#"schema = "weeping-angel/forbidden-patterns/v1"
"#,
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/readiness.rs"),
        "",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/temporal.rs"),
        "",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/lineage.rs"),
        "",
    )
    .unwrap();
    fs::write(root.join("src/main.rs"), "").unwrap();
    fs::write(root.join("src/cli.rs"), "").unwrap();
    fs::write(root.join("docs/debt/register.toml"), register).unwrap();
}

fn seed_findings() -> String {
    let mut body = String::from("schema = \"weeping-angel/debt-register/v1\"\n");
    for id in STUB_IDS {
        body.push_str(&format!(
            r#"
[[finding]]
id = "DEBT-GUARD-{id}"
title = "stub"
status = "open"
summary = "stub skip"
"#
        ));
    }
    body
}

fn xtask_lib_src() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("read xtask/src/lib.rs")
}

fn xtask_cargo_toml() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read xtask/Cargo.toml")
}

fn live_root() -> std::path::PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn check_named<'a>(report: &'a xtask::GuardReport, id: &str) -> &'a xtask::CheckResult {
    report
        .checks
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("missing check {id} in {:?}", report.checks))
}

/// ACP-B01: check 04 skips with DEBT-GUARD-04 when that finding exists.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b01_check_04_skips_with_debt_guard_04() {
    let dir = tempdir().unwrap();
    write_minimal_repo(dir.path(), &seed_findings());
    // Check 04 does not require architecture/invariants.toml today.
    assert!(!dir.path().join("architecture/invariants.toml").is_file());

    let report = run_guard(dir.path());
    assert!(!report.failed(), "{}", report.render());

    let c04 = check_named(&report, "04");
    assert_eq!(c04.name, "architecture-invariants");
    assert_eq!(
        c04.status,
        CheckStatus::Skip {
            debt_id: "DEBT-GUARD-04".into()
        }
    );
    assert!(
        report
            .render()
            .contains("04  architecture-invariants  skip(DEBT-GUARD-04)")
    );

    for id in STUB_IDS {
        match &check_named(&report, id).status {
            CheckStatus::Skip { debt_id } => {
                assert_eq!(debt_id, &format!("DEBT-GUARD-{id}"));
            }
            other => panic!("stub {id} must skip-with-debt, got {other:?}"),
        }
    }

    for id in ["01", "02", "03", "13"] {
        assert_eq!(
            check_named(&report, id).status,
            CheckStatus::Pass,
            "implemented check {id}"
        );
    }

    // Report order today: 01, 02, 03, 13, then stubs (04 is not evaluated).
    let ids: Vec<&str> = report.checks.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "01", "02", "03", "13", "04", "05", "06", "07", "08", "09", "10", "11", "12", "14",
            "15"
        ]
    );
}

/// ACP-B01 (live tree): check 04 is still a stub skip on the workspace.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b01_live_repo_check_04_is_stub_skip() {
    let report = run_guard(&live_root());
    assert!(!report.failed(), "{}", report.render());
    assert_eq!(
        check_named(&report, "04").status,
        CheckStatus::Skip {
            debt_id: "DEBT-GUARD-04".into()
        }
    );
    let src = xtask_lib_src();
    assert!(
        src.contains("fn stub_check"),
        "skip path is stub_check, not skip_guard_check"
    );
    assert!(
        !src.contains("fn skip_guard_check"),
        "there is no skip_guard_check helper today"
    );
    assert!(
        !src.contains("fn check_04"),
        "check 04 is not a real function today"
    );
}

/// ACP-B02: check 03 passes on schema presence even when [[pattern]] kinds
/// would fail if executed (existing path + hypothetical package names).
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b02_check_03_ignores_pattern_kinds() {
    let dir = tempdir().unwrap();
    write_minimal_repo(dir.path(), &seed_findings());
    fs::create_dir_all(dir.path().join("tests/sdd")).unwrap();
    fs::write(dir.path().join("tests/sdd/placeholder.rs"), "").unwrap();
    fs::write(
        dir.path().join("architecture/forbidden-patterns.toml"),
        r#"schema = "weeping-angel/forbidden-patterns/v1"

[[pattern]]
id = "FORBID-HYPOTHETICAL-CATALOG"
kind = "package"
value = "weeping-angel-catalog"

[[pattern]]
id = "FORBID-HYPOTHETICAL-ASSURANCE-CLI"
kind = "package"
value = "weeping-angel-assurance-cli"

[[pattern]]
id = "FORBID-TESTS-SDD"
kind = "path"
value = "tests/sdd/"

[[pattern]]
id = "FORBID-EXISTING-FILE"
kind = "path"
value = "src/main.rs"
"#,
    )
    .unwrap();

    let c03 = check_03_forbidden_patterns(dir.path());
    assert_eq!(
        c03.status,
        CheckStatus::Pass,
        "check 03 is presence+schema only; kinds are not executed: {}",
        c03.report_line()
    );

    let report = run_guard(dir.path());
    assert_eq!(check_named(&report, "03").status, CheckStatus::Pass);
    assert!(
        !report.failed(),
        "kinds not executed so existing tests/sdd/ and src/main.rs must not fail 03: {}",
        report.render()
    );
}

/// ACP-B03: ownership rows need only crate + paths (no kind).
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b03_ownership_is_crate_and_paths_without_kind() {
    assert_eq!(REQUIRED_OWNERSHIP.len(), 7);
    for (concept, crate_name, paths) in REQUIRED_OWNERSHIP {
        assert!(!concept.is_empty());
        assert!(!crate_name.is_empty());
        assert!(!paths.is_empty(), "{concept} paths");
    }
    // Tuple is (concept, crate, paths) — no kind slot.
    let _ = REQUIRED_OWNERSHIP[0].2;

    let dir = tempdir().unwrap();
    write_minimal_repo(dir.path(), &seed_findings());
    let arch = fs::read_to_string(dir.path().join("architecture/architecture.toml")).unwrap();
    assert!(
        !arch.contains("kind"),
        "fixture ownership has no kind field"
    );
    let c02 = check_02_ownership(dir.path());
    assert_eq!(
        c02.status,
        CheckStatus::Pass,
        "kind is not required today: {}",
        c02.report_line()
    );

    let live = read_live("architecture/architecture.toml");
    assert!(
        !live.contains("kind ="),
        "live architecture.toml has no ownership kind today"
    );
    assert!(live.contains("[ownership.temporal_evidence_selection]"));
    assert!(live.contains("crate = \"weeping-angel-assurance\""));
}

/// ACP-B04: GuardReport is { checks } + render(); CLI is `guard` only.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b04_guard_report_is_checks_plus_render_cli_is_guard_only() {
    // Struct literal with only `checks` is the current shape (no violations /
    // skipped / debt_exemptions / duration).
    let empty = GuardReport {
        checks: vec![],
        violations: vec![],
        skipped: vec![],
        debt_exemptions: vec![],
        duration: std::time::Duration::ZERO,
    };
    assert!(!empty.failed());
    assert_eq!(empty.render(), "cargo xtask guard\n");

    let src = xtask_lib_src();
    assert!(src.contains("pub struct GuardReport {\n    pub checks: Vec<CheckResult>,\n}"));
    for field in ["violations", "skipped", "debt_exemptions", "duration"] {
        assert!(
            !src.contains(&format!("pub {field}")),
            "GuardReport must not expose {field} today"
        );
    }

    let cargo = xtask_cargo_toml();
    assert!(
        !cargo.contains("serde_json"),
        "xtask has no JSON reporter dependency today"
    );

    assert_eq!(
        main_with_args(["help"]),
        2,
        "non-guard first arg is usage exit 2"
    );
    assert_eq!(
        main_with_args(["--json"]),
        2,
        "--json as first arg is not a command today"
    );
    assert_eq!(
        main_with_args(["--check", "09"]),
        2,
        "--check is not a command today"
    );
    assert_eq!(
        main_with_args(["--explain", "INV-INVARIANTS-EVALUATED"]),
        2,
        "--explain is not a command today"
    );

    // Extra tokens after `guard` are ignored; human render still runs.
    assert_eq!(
        main_with_args(["guard", "--json"]),
        0,
        "guard --json is still the human guard command today"
    );
    assert_eq!(
        main_with_args(["guard", "--check", "09"]),
        0,
        "guard --check 09 does not select a single check today"
    );
}

/// ACP-B05: INV-INVARIANTS-EVALUATED says evaluation is remaining_backlog.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b05_inv_invariants_evaluated_is_remaining_backlog() {
    assert_eq!(
        INVARIANTS_SCHEMA,
        "weeping-angel/architecture-invariants/v1"
    );
    let text = read_live("architecture/invariants.toml");
    assert!(text.contains("schema = \"weeping-angel/architecture-invariants/v1\""));
    assert!(text.contains("id = \"INV-INVARIANTS-EVALUATED\""));
    assert!(text.contains("guard_check = \"04\""));
    assert!(
        text.contains("Evaluating this file against the tree is remaining_backlog"),
        "INV-INVARIANTS-EVALUATED still claims evaluation is remaining_backlog"
    );

    let src = xtask_lib_src();
    assert!(
        !src.contains("architecture/invariants.toml")
            || src
                .lines()
                .filter(|l| l.contains("architecture/invariants.toml"))
                .all(|l| l.trim_start().starts_with("//") || l.trim_start().starts_with("//!")),
        "check 04 does not open architecture/invariants.toml today"
    );
}

/// ACP-B06: stub without a live DEBT-GUARD-04 finding fails closed.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn acp_b06_stub_without_debt_fails_closed() {
    let dir = tempdir().unwrap();
    write_minimal_repo(
        dir.path(),
        r#"
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-ONLY"
title = "unrelated"
status = "open"
summary = "does not cover stubs"
"#,
    );
    let report = run_guard(dir.path());
    assert!(report.failed(), "{}", report.render());
    match &check_named(&report, "04").status {
        CheckStatus::Fail(msg) => {
            assert!(msg.contains("not-yet-implemented: check 04"), "{msg}");
            assert!(msg.contains("DEBT-GUARD-04"), "{msg}");
        }
        other => panic!("expected fail closed, got {other:?}"),
    }
}

/// No shared evaluation plane: each implemented check reads TOML independently.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn current_tree_has_no_repository_model_or_architecture_check() {
    let src = xtask_lib_src();
    for needle in [
        "struct RepositoryModel",
        "struct ArchitectureManifest",
        "struct ArchitectureInvariant",
        "struct InvariantResult",
        "trait ArchitectureCheck",
        "fn check(&self, repo: &RepositoryModel)",
    ] {
        assert!(
            !src.contains(needle),
            "found future API `{needle}` in xtask/src/lib.rs; baseline is pre-Phase-1"
        );
    }
}

/// Live debt: DEBT-GUARD-04 is open; check 13 would reject resolved-without-proof.
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn live_debt_guard_04_is_open_stub() {
    let register = read_live("docs/debt/register.toml");
    assert!(register.contains("id = \"DEBT-GUARD-04\""));
    assert!(register.contains(
        "Architecture invariants.toml is declared but not evaluated. Check 04 is a stub this increment."
    ));
    let block = register
        .split("[[finding]]")
        .find(|b| b.contains("id = \"DEBT-GUARD-04\""))
        .expect("DEBT-GUARD-04 block");
    assert!(
        block.contains("status = \"open\""),
        "DEBT-GUARD-04 must remain open until Guard 04 evaluates invariants"
    );
    assert!(!block.contains("regression_tests"));
    assert!(!block.contains("repository_guard"));
}

/// Pipeline as-built: EvidenceLedger has latest_as_of, not current().
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn evidence_ledger_has_latest_as_of_not_current() {
    let ledger = read_live("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(ledger.contains("pub fn latest_as_of("));
    assert!(
        !ledger.contains("pub fn current("),
        "EvidenceLedger::current() does not exist today"
    );
    let temporal = read_live("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(temporal.contains("pub fn select_latest_as_of"));
    let assurance_temporal = read_live("crates/weeping-angel-assurance/src/temporal.rs");
    assert!(
        !assurance_temporal.is_empty(),
        "temporal selection is split: assurance facade file exists"
    );
}

/// Dual-suite home for this increment is xtask/tests/*.rs (not tests/sdd/).
#[ignore = "superseded by sdd_architectural_cleanup_target"]
#[test]
fn dual_suite_lives_under_xtask_tests_not_tests_sdd() {
    let tests_sdd = live_root().join("tests/sdd");
    assert!(
        !tests_sdd.exists(),
        "tests/sdd/ must not exist (ADR 0004 / FORBID-TESTS-SDD)"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/sdd_architectural_cleanup_baseline.rs")
            .is_file()
    );
}
