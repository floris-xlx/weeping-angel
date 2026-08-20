//! Target suite for architectural-cleanup PROGRAM increment 1.
//!
//! Encodes ACP-T01–T17 (desired Phase 0 freeze + Phase 1 architecture-as-law).
//! Must FAIL on CURRENT stub/skip/presence-only `cargo xtask guard` (RED).
//! Do not implement Guard 04 / ownership kinds / forbidden-kind execution /
//! structured CLI in this file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;
use xtask::{
    CheckStatus, REQUIRED_OWNERSHIP, check_02_ownership, check_03_forbidden_patterns,
    main_with_args, repo_root_from_xtask_manifest, run_guard,
};

const REMAINING_STUBS: [&str; 8] = ["05", "06", "07", "08", "09", "10", "11", "12"];

const OWNERSHIP_KINDS: [&str; 5] = [
    "exclusive",
    "facade",
    "projection",
    "adapter",
    "shared-primitive",
];

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn xtask_lib_src() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = String::new();
    fn walk(dir: &Path, out: &mut String) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push_str(&fs::read_to_string(&path).unwrap());
                out.push('\n');
            }
        }
    }
    walk(&root, &mut out);
    out
}

fn check_named<'a>(report: &'a xtask::GuardReport, id: &str) -> &'a xtask::CheckResult {
    report
        .checks
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("missing check {id} in {:?}", report.checks))
}

fn architecture_toml_with_kinds() -> &'static str {
    r#"schema = "weeping-angel/architecture/v1"

[policy]
ownership_kinds = ["exclusive", "facade", "projection", "adapter", "shared-primitive"]
required_concepts = [
  "catalog",
  "framework_compilation",
  "readiness_projection",
  "temporal_evidence_selection",
  "assessment_lineage",
  "evidence_persistence",
  "assurance_cli",
]

[ownership.catalog]
crate = "weeping-angel-canonical-catalog"
kind = "exclusive"
paths = ["crates/weeping-angel-canonical-catalog"]

[ownership.framework_compilation]
crate = "weeping-angel-framework"
kind = "exclusive"
paths = ["crates/weeping-angel-framework"]

[ownership.readiness_projection]
crate = "weeping-angel-assurance"
kind = "projection"
paths = ["crates/weeping-angel-assurance/src/readiness.rs"]

[ownership.temporal_evidence_selection]
crate = "weeping-angel-assurance"
kind = "exclusive"
paths = ["crates/weeping-angel-assurance/src/temporal.rs"]

[ownership.assessment_lineage]
crate = "weeping-angel-assurance"
kind = "exclusive"
paths = ["crates/weeping-angel-assurance/src/lineage.rs"]

[ownership.evidence_persistence]
crate = "weeping-angel-evidence"
kind = "exclusive"
paths = ["crates/weeping-angel-evidence"]

[ownership.assurance_cli]
crate = "weeping-angel"
kind = "facade"
paths = ["src/main.rs", "src/cli.rs"]

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
"#
}

fn evaluated_invariants_toml() -> &'static str {
    r#"schema = "weeping-angel/architecture-invariants/v1"

[[invariant]]
id = "INV-OWNERSHIP-LIVE-CRATES"
summary = "Ownership table names live workspace packages and existing paths only"
guard_check = "02"

[[invariant]]
id = "INV-NO-HYPOTHETICAL-PACKAGES"
summary = "Packages weeping-angel-catalog and weeping-angel-assurance-cli must not exist"
guard_check = "02"

[[invariant]]
id = "INV-DEBT-RESOLVED-HAS-PROOF"
summary = "status=resolved requires regression_tests or repository_guard"
guard_check = "13"

[[invariant]]
id = "INV-INVARIANTS-EVALUATED"
summary = "Every [[invariant]] is evaluated against RepositoryModel; skip is illegal without a live debt id"
guard_check = "04"
"#
}

fn forbidden_patterns_toml() -> &'static str {
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
"#
}

/// GREEN-ready increment-1 fixture (no DEBT-GUARD-04; remaining stubs live).
fn write_increment1_repo(root: &Path) {
    fs::create_dir_all(root.join("architecture")).unwrap();
    fs::create_dir_all(root.join("docs/debt")).unwrap();
    fs::create_dir_all(root.join("docs/adr")).unwrap();
    fs::create_dir_all(root.join("docs/specs")).unwrap();
    fs::create_dir_all(root.join("frameworks/iso-27001/2022")).unwrap();
    fs::create_dir_all(root.join("catalog/canonical/v1")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-canonical-catalog")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-framework")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-assurance/src")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-evidence")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-collector")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-control-test")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-assurance-ir")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = [
    "crates/weeping-angel-assurance-ir",
    "crates/weeping-angel-framework",
    "crates/weeping-angel-evidence",
    "crates/weeping-angel-collector",
    "crates/weeping-angel-control-test",
    "crates/weeping-angel-assurance",
    "crates/weeping-angel-canonical-catalog",
]

[package]
name = "weeping-angel"
version = "0.0.0"
edition = "2024"
"#,
    )
    .unwrap();
    for pkg in [
        "weeping-angel-canonical-catalog",
        "weeping-angel-framework",
        "weeping-angel-assurance",
        "weeping-angel-evidence",
        "weeping-angel-collector",
        "weeping-angel-control-test",
        "weeping-angel-assurance-ir",
    ] {
        fs::write(
            root.join(format!("crates/{pkg}/Cargo.toml")),
            format!("[package]\nname = \"{pkg}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
        )
        .unwrap();
    }
    let live_domain =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../architecture/domain-ownership.toml");
    fs::write(
        root.join("architecture/domain-ownership.toml"),
        fs::read_to_string(&live_domain).unwrap_or_else(|e| {
            panic!("read live architecture/domain-ownership.toml for ACP fixture: {e}")
        }),
    )
    .unwrap();
    fs::write(
        root.join("architecture/architecture.toml"),
        architecture_toml_with_kinds(),
    )
    .unwrap();
    fs::write(
        root.join("architecture/invariants.toml"),
        evaluated_invariants_toml(),
    )
    .unwrap();
    fs::write(
        root.join("architecture/forbidden-patterns.toml"),
        forbidden_patterns_toml(),
    )
    .unwrap();
    seed_product_law_sources(root);
    fs::write(root.join("src/main.rs"), "").unwrap();
    fs::write(root.join("src/cli.rs"), "").unwrap();
    fs::write(
        root.join("docs/adr/0010-architecture-as-law.md"),
        r#"# ADR 0010

<!-- weeping-angel-adr-meta
id = "0010"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->
"#,
    )
    .unwrap();
    fs::write(
        root.join("docs/specs/architectural-cleanup-program.md"),
        "# spec\n",
    )
    .unwrap();
    fs::write(
        root.join("architecture/adr-identity.toml"),
        r#"schema = "weeping-angel/adr-identity/v1"
grandfathered_debt = ""
grandfathered_prefixes = []
grandfathered_files = []
"#,
    )
    .unwrap();
    fs::write(
        root.join("architecture/spec-lifecycle.toml"),
        r#"schema = "weeping-angel/spec-lifecycle/v1"

[[spec]]
path = "docs/specs/architectural-cleanup-program.md"
state = "active"
ownership = ["catalog"]
depends_on = []
supersedes = []
successor = ""
"#,
    )
    .unwrap();
    fs::write(root.join("frameworks/iso-27001/2022/pack.toml"), "").unwrap();
    fs::write(root.join("catalog/canonical/v1/catalog.toml"), "").unwrap();
    fs::write(
        root.join("docs/debt/register.toml"),
        resolved_product_debt(),
    )
    .unwrap();
}

fn seed_product_law_sources(root: &Path) {
    fs::create_dir_all(root.join("crates/weeping-angel-canonical-catalog/src")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-framework/src")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-evidence/src")).unwrap();
    fs::write(
        root.join("crates/weeping-angel-canonical-catalog/src/lib.rs"),
        "pub struct CanonicalCatalog;\nimpl CanonicalCatalog { pub fn load() {} }\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-framework/src/lib.rs"),
        "pub enum PackError {}\npub struct FrameworkPackDigest;\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/readiness.rs"),
        "pub fn project_readiness() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/temporal.rs"),
        "",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/lineage.rs"),
        "pub fn replay_assessment() {}\npub fn project_soa_from_snapshot() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-assurance/src/lib.rs"),
        "pub fn project_readiness() {}\npub fn replay_assessment() {}\npub fn project_soa_from_snapshot() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/weeping-angel-evidence/src/lib.rs"),
        "pub fn current() {}\npub fn as_of() {}\npub fn latest() {}\n",
    )
    .unwrap();
}

fn resolved_product_debt() -> String {
    let mut body = String::from("schema = \"weeping-angel/debt-register/v1\"\n");
    for id in REMAINING_STUBS {
        body.push_str(&format!(
            r#"
[[finding]]
id = "DEBT-GUARD-{id}"
title = "implemented"
status = "resolved"
summary = "product-law check is a real ArchitectureCheck"
owner = "fixture"
introduced = "2026-08-19"
severity = "medium"
remediation = "Guard {id} evaluates product surfaces"
repository_guard = "{id}"
regression_tests = ["sdd_architectural_cleanup_target"]
"#
        ));
    }
    body
}

fn xtask_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xtask"));
    cmd.current_dir(live_root());
    cmd
}

fn assert_json_guard_report(stdout: &str) {
    let trimmed = stdout.trim_start();
    assert!(
        trimmed.starts_with('{'),
        "guard --json must emit a JSON object, got: {stdout}"
    );
    for field in [
        "checks",
        "violations",
        "skipped",
        "debt_exemptions",
        "duration",
    ] {
        assert!(
            stdout.contains(&format!("\"{field}\"")) || stdout.contains(field),
            "JSON GuardReport must include {field}: {stdout}"
        );
    }
}

/// ACP-T01: RepositoryModel is the shared snapshot loaded once per run_guard.
#[test]
fn acp_t01_repository_model_loads_workspace_graph_and_manifests() {
    let src = xtask_lib_src();
    assert!(
        src.contains("struct RepositoryModel"),
        "Phase 1 requires RepositoryModel (workspace, package graph, filesystem, manifests, debt, ADR/spec metadata, framework packs, catalog sources)"
    );
    for needle in [
        "ArchitectureManifest",
        "package graph",
        "framework",
        "catalog",
        "docs/adr",
        "docs/specs",
        "docs/debt/register.toml",
    ] {
        assert!(
            src.contains(needle)
                || src
                    .to_ascii_lowercase()
                    .contains(&needle.to_ascii_lowercase()),
            "RepositoryModel must load {needle}"
        );
    }
    assert!(
        !src.contains("STUB_CHECKS") || !src.contains("(\"04\", \"architecture-invariants\")"),
        "Guard 04 must not remain in STUB_CHECKS"
    );
}

/// ACP-T02: checks implement ArchitectureCheck against one model (not independent greps).
#[test]
fn acp_t02_architecture_check_trait_is_the_evaluation_plane() {
    let src = xtask_lib_src();
    assert!(
        src.contains("trait ArchitectureCheck"),
        "missing trait ArchitectureCheck"
    );
    assert!(
        src.contains("fn check(&self, repo: &RepositoryModel)"),
        "ArchitectureCheck::check must take &RepositoryModel"
    );
    assert!(
        src.contains("struct ArchitectureInvariant"),
        "missing ArchitectureInvariant"
    );
    assert!(
        src.contains("struct InvariantResult"),
        "missing InvariantResult"
    );
    assert!(
        src.contains("fn check_04")
            || src.contains("ArchitectureInvariants")
            || src.contains("architecture-invariants"),
        "Guard 04 must be a real ArchitectureCheck, not stub_check"
    );
    let run_guard_fn = src.split("pub fn run_guard").nth(1).expect("run_guard");
    assert!(
        run_guard_fn.contains("RepositoryModel") || run_guard_fn.contains("load"),
        "run_guard must load one RepositoryModel"
    );
}

/// ACP-T03: Guard 04 parses invariants.toml and evaluates every [[invariant]].
#[test]
fn acp_t03_guard_04_evaluates_every_invariant_and_passes_on_valid_fixture() {
    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    assert!(dir.path().join("architecture/invariants.toml").is_file());

    let report = run_guard(dir.path());
    let rendered = report.render();
    assert!(
        !report.failed(),
        "increment-2 fixture must be green (01–15 pass): {rendered}"
    );
    assert_eq!(
        check_named(&report, "04").status,
        CheckStatus::Pass,
        "Guard 04 must evaluate invariants and pass, not stub-skip: {rendered}"
    );
    assert_eq!(check_named(&report, "04").name, "architecture-invariants");
    for id in ["01", "02", "03", "13", "14", "15"] {
        assert_eq!(
            check_named(&report, id).status,
            CheckStatus::Pass,
            "implemented check {id}: {rendered}"
        );
    }
    for id in REMAINING_STUBS {
        assert_eq!(
            check_named(&report, id).status,
            CheckStatus::Pass,
            "product-law check {id} must pass on the seeded fixture: {rendered}"
        );
    }
}

/// ACP-T04: INV-INVARIANTS-EVALUATED no longer claims remaining_backlog.
#[test]
fn acp_t04_inv_invariants_evaluated_is_not_remaining_backlog() {
    let text = read_live("architecture/invariants.toml");
    assert!(text.contains("id = \"INV-INVARIANTS-EVALUATED\""));
    assert!(
        !text.contains("Evaluating this file against the tree is remaining_backlog"),
        "INV-INVARIANTS-EVALUATED must be rewritten; evaluation is Guard 04 law"
    );
    assert!(
        !text.contains("remaining_backlog"),
        "invariants.toml must not claim evaluation is remaining_backlog"
    );
    let src = xtask_lib_src();
    assert!(
        src.contains("architecture/invariants.toml")
            && src
                .lines()
                .any(|l| l.contains("architecture/invariants.toml")
                    && !l.trim_start().starts_with("//")
                    && !l.trim_start().starts_with("//!")),
        "check 04 must open architecture/invariants.toml"
    );
}

/// ACP-T05: missing / unevaluated invariants fail check 04 (not stub skip / nyi).
#[test]
fn acp_t05_missing_invariant_evaluation_fails_check_04() {
    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    fs::remove_file(dir.path().join("architecture/invariants.toml")).unwrap();

    let report = run_guard(dir.path());
    match &check_named(&report, "04").status {
        CheckStatus::Fail(msg) => {
            assert!(
                !msg.contains("not-yet-implemented"),
                "04 is a real evaluation, not a stub: {msg}"
            );
            assert!(
                msg.contains("invariant") || msg.contains("invariants.toml"),
                "fail must cite invariants evaluation: {msg}"
            );
        }
        other => panic!("missing invariants.toml must fail check 04, got {other:?}"),
    }
}

/// ACP-T06: ownership rows require kind in the closed enum.
#[test]
fn acp_t06_ownership_rows_require_kind() {
    let src = xtask_lib_src();
    for kind in OWNERSHIP_KINDS {
        assert!(
            src.contains(kind),
            "check 02 must validate ownership kind `{kind}`"
        );
    }

    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    let mut without_kind = architecture_toml_with_kinds().to_string();
    for kind in OWNERSHIP_KINDS {
        without_kind = without_kind.replace(&format!("kind = \"{kind}\"\n"), "");
    }
    fs::write(
        dir.path().join("architecture/architecture.toml"),
        without_kind,
    )
    .unwrap();

    let c02 = check_02_ownership(dir.path());
    match &c02.status {
        CheckStatus::Fail(msg) => {
            assert!(
                msg.contains("kind"),
                "missing ownership kind must fail closed: {msg}"
            );
        }
        other => panic!("ownership without kind must fail check 02, got {other:?}"),
    }

    let live = read_live("architecture/architecture.toml");
    assert!(
        live.contains("kind ="),
        "live architecture.toml must declare ownership kinds"
    );
    for (concept, _, _) in REQUIRED_OWNERSHIP {
        let header = format!("[ownership.{concept}]");
        let idx = live
            .find(&header)
            .unwrap_or_else(|| panic!("missing {header}"));
        let rest = &live[idx..];
        let block = rest.split("\n[").next().unwrap_or(rest);
        assert!(
            OWNERSHIP_KINDS
                .iter()
                .any(|k| block.contains(&format!("kind = \"{k}\""))),
            "{header} must set kind ∈ exclusive|facade|projection|adapter|shared-primitive:\n{block}"
        );
    }
}

/// ACP-T07: temporal_evidence_selection may be exclusive without moving select_latest_as_of.
#[test]
fn acp_t07_temporal_exclusive_kind_does_not_move_select_latest_as_of() {
    let live = read_live("architecture/architecture.toml");
    let idx = live
        .find("[ownership.temporal_evidence_selection]")
        .expect("temporal ownership row");
    let block = live[idx..].split("\n[").next().unwrap();
    assert!(
        block.contains("kind = \"exclusive\""),
        "temporal_evidence_selection kind must be exclusive (metadata, not a code move):\n{block}"
    );
    assert!(block.contains("crate = \"weeping-angel-assurance\""));

    let temporal = read_live("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(
        temporal.contains("pub fn select_latest_as_of"),
        "increment 1 must not move select_latest_as_of"
    );
}

/// ACP-T08: check 03 executes kind=package and rejects hypothetical members.
#[test]
fn acp_t08_forbidden_package_kind_rejects_hypothetical_crates() {
    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    fs::create_dir_all(dir.path().join("crates/weeping-angel-catalog")).unwrap();
    fs::write(
        dir.path().join("crates/weeping-angel-catalog/Cargo.toml"),
        "[package]\nname = \"weeping-angel-catalog\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    let mut cargo = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    cargo = cargo.replace(
        "\"crates/weeping-angel-canonical-catalog\",",
        "\"crates/weeping-angel-canonical-catalog\",\n    \"crates/weeping-angel-catalog\",",
    );
    fs::write(dir.path().join("Cargo.toml"), cargo).unwrap();

    let c03 = check_03_forbidden_patterns(dir.path());
    match &c03.status {
        CheckStatus::Fail(msg) => {
            assert!(
                msg.contains("weeping-angel-catalog")
                    || msg.contains("FORBID-HYPOTHETICAL-CATALOG"),
                "package kind must reject weeping-angel-catalog: {msg}"
            );
        }
        other => panic!("hypothetical catalog package must fail check 03, got {other:?}"),
    }

    fs::create_dir_all(dir.path().join("crates/weeping-angel-assurance-cli")).unwrap();
    fs::write(
        dir.path()
            .join("crates/weeping-angel-assurance-cli/Cargo.toml"),
        "[package]\nname = \"weeping-angel-assurance-cli\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    let mut cargo = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    cargo = cargo.replace(
        "\"crates/weeping-angel-catalog\",",
        "\"crates/weeping-angel-catalog\",\n    \"crates/weeping-angel-assurance-cli\",",
    );
    fs::write(dir.path().join("Cargo.toml"), cargo).unwrap();
    let c03 = check_03_forbidden_patterns(dir.path());
    match &c03.status {
        CheckStatus::Fail(msg) => {
            assert!(
                msg.contains("weeping-angel-assurance-cli")
                    || msg.contains("FORBID-HYPOTHETICAL-ASSURANCE-CLI")
                    || msg.contains("weeping-angel-catalog"),
                "package kind must reject weeping-angel-assurance-cli: {msg}"
            );
        }
        other => panic!("hypothetical assurance-cli package must fail check 03, got {other:?}"),
    }
}

/// ACP-T09: check 03 executes kind=path and rejects tests/sdd/.
#[test]
fn acp_t09_forbidden_path_kind_rejects_tests_sdd() {
    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    fs::create_dir_all(dir.path().join("tests/sdd")).unwrap();
    fs::write(dir.path().join("tests/sdd/placeholder.rs"), "").unwrap();

    let c03 = check_03_forbidden_patterns(dir.path());
    match &c03.status {
        CheckStatus::Fail(msg) => {
            assert!(
                msg.contains("tests/sdd") || msg.contains("FORBID-TESTS-SDD"),
                "path kind must reject tests/sdd/: {msg}"
            );
        }
        other => panic!("existing tests/sdd/ must fail check 03, got {other:?}"),
    }
}

/// ACP-T10: unknown forbidden kind fails closed; all five kinds are executable.
#[test]
fn acp_t10_unknown_forbidden_kind_fails_closed() {
    let src = xtask_lib_src();
    for kind in ["package", "path", "dependency", "symbol", "source-pattern"] {
        assert!(
            src.contains(kind),
            "check 03 must execute forbidden kind `{kind}`"
        );
    }

    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    fs::write(
        dir.path().join("architecture/forbidden-patterns.toml"),
        r#"schema = "weeping-angel/forbidden-patterns/v1"

[[pattern]]
id = "FORBID-UNKNOWN"
kind = "grep-framework"
value = "anything"
"#,
    )
    .unwrap();
    let c03 = check_03_forbidden_patterns(dir.path());
    match &c03.status {
        CheckStatus::Fail(msg) => {
            assert!(
                msg.contains("kind") || msg.contains("grep-framework") || msg.contains("unknown"),
                "unknown kind must fail closed: {msg}"
            );
        }
        other => panic!("unknown forbidden kind must fail check 03, got {other:?}"),
    }
}

/// ACP-T11: GuardReport exposes checks, violations, skipped, debt_exemptions, duration.
#[test]
fn acp_t11_guard_report_is_structured() {
    let src = xtask_lib_src();
    assert!(src.contains("pub struct GuardReport"));
    for field in ["violations", "skipped", "debt_exemptions", "duration"] {
        assert!(
            src.contains(&format!("pub {field}")),
            "GuardReport must expose pub {field}"
        );
    }
    assert!(
        !src.contains("pub struct GuardReport {\n    pub checks: Vec<CheckResult>,\n}"),
        "GuardReport must not remain checks-only"
    );

    let dir = tempdir().unwrap();
    write_increment1_repo(dir.path());
    let report = run_guard(dir.path());
    let rendered = report.render();
    for id in REMAINING_STUBS {
        assert_eq!(
            check_named(&report, id).status,
            CheckStatus::Pass,
            "product-law check {id} must pass: {rendered}"
        );
    }
}

/// ACP-T12: `cargo xtask guard --json` emits GuardReport fields.
#[test]
fn acp_t12_cli_json_emits_guard_report_fields() {
    let src = xtask_lib_src();
    assert!(src.contains("--json"), "CLI must parse --json");

    let output = xtask_bin()
        .args(["guard", "--json"])
        .output()
        .expect("spawn xtask guard --json");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "guard --json must exit 0 on live tree after increment 1; stdout={stdout} stderr={stderr}"
    );
    assert_json_guard_report(&stdout);
}

/// ACP-T13: `cargo xtask guard --check 09` runs 09 (skip-with-debt, never silent).
#[test]
fn acp_t13_cli_check_09_runs_named_check() {
    let src = xtask_lib_src();
    assert!(src.contains("--check"), "CLI must parse --check NN");

    let output = xtask_bin()
        .args(["guard", "--check", "09"])
        .output()
        .expect("spawn xtask guard --check 09");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
    assert!(
        combined.contains("09") && combined.contains("temporal-evidence-selection"),
        "--check 09 must run check 09: {combined}"
    );
    assert!(
        output.status.success()
            && (combined.contains("09  temporal-evidence-selection  pass")
                || combined.contains("\"id\":\"09\"")),
        "check 09 must pass as a real ArchitectureCheck: {combined}"
    );
    assert!(
        !combined.contains("01  architecture-manifest  pass")
            || src.contains("selected")
            || combined.contains("\"id\":\"09\""),
        "--check 09 must not silently run the full default suite as unfiltered extra args"
    );
}

/// ACP-T14: `cargo xtask guard --explain INV-…` explains an invariant.
#[test]
fn acp_t14_cli_explain_prints_invariant_evaluation() {
    let src = xtask_lib_src();
    assert!(src.contains("--explain"), "CLI must parse --explain INV-…");

    let output = xtask_bin()
        .args(["guard", "--explain", "INV-INVARIANTS-EVALUATED"])
        .output()
        .expect("spawn xtask guard --explain");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "--explain must exit 0: {combined}");
    assert!(
        combined.contains("INV-INVARIANTS-EVALUATED"),
        "--explain must print the invariant id: {combined}"
    );
    assert!(
        combined.contains("guard_check") || combined.contains("04"),
        "--explain must print guard_check / evidence: {combined}"
    );
}

/// ACP-T15: live `cargo xtask guard` — 01–15 pass (product-law 05–12 included).
#[test]
fn acp_t15_live_guard_04_passes_remaining_stubs_skip() {
    let report = run_guard(&live_root());
    let rendered = report.render();
    assert!(
        !report.failed(),
        "live guard must exit-equivalent 0: {rendered}"
    );
    assert_eq!(
        check_named(&report, "04").status,
        CheckStatus::Pass,
        "check 04 must be evaluated pass, not skip(DEBT-GUARD-04): {rendered}"
    );
    for id in REMAINING_STUBS {
        assert_eq!(
            check_named(&report, id).status,
            CheckStatus::Pass,
            "live product-law check {id} must pass: {rendered}"
        );
    }
    for id in ["01", "02", "03", "13", "14", "15"] {
        assert_eq!(check_named(&report, id).status, CheckStatus::Pass);
    }

    assert_eq!(
        main_with_args(["guard"]),
        0,
        "cargo xtask guard must exit 0"
    );
}

/// ACP-T16: DEBT-GUARD-04 resolved only with regression_tests or repository_guard=04.
#[test]
fn acp_t16_debt_guard_04_resolved_with_proof() {
    let register = read_live("docs/debt/register.toml");
    let block = register
        .split("[[finding]]")
        .find(|b| b.contains("id = \"DEBT-GUARD-04\""))
        .expect("DEBT-GUARD-04 row");
    assert!(
        block.contains("status = \"resolved\""),
        "close DEBT-GUARD-04 only after Guard 04 tests evaluate invariants; still open:\n{block}"
    );
    let has_tests =
        block.contains("regression_tests") && block.contains("sdd_architectural_cleanup");
    let has_guard = block.contains("repository_guard")
        && (block.contains("\"04\"") || block.contains("repository_guard = 4"));
    assert!(
        has_tests || has_guard,
        "resolved DEBT-GUARD-04 needs regression_tests or repository_guard = \"04\":\n{block}"
    );

    let ri = read_live("tests/contracts/repository_integrity.target.rs");
    assert!(
        !ri.contains("\"04\", \"05\"")
            && (ri.contains("ri_t13") && !ri.contains("for id in STUB_CHECKS")
                || !ri.contains("const STUB_CHECKS: [&str; 11]")),
        "RI-T13 must treat 04 as pass/evaluated (stubs are 05–12 / 14–15 only)"
    );
}

/// ACP-T17: dual-suite under xtask/tests; tests/sdd/ stays forbidden.
#[test]
fn acp_t17_dual_suite_lives_under_xtask_tests() {
    let xtask_tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    assert!(
        !xtask_tests
            .join("sdd_architectural_cleanup_baseline.rs")
            .exists(),
        "superseded xtask baseline suite must be deleted"
    );
    assert!(
        xtask_tests
            .join("sdd_architectural_cleanup_target.rs")
            .is_file()
    );
    assert!(
        !live_root().join("tests/sdd").exists(),
        "tests/sdd/ is forbidden (ADR 0004 / FORBID-TESTS-SDD)"
    );
    let spec = read_live("docs/specs/architectural-cleanup-program.md");
    assert!(spec.contains("Phase 0"));
    assert!(spec.contains("29 phases") || spec.contains("0–28") || spec.contains("0-28"));
    let layout = read_live("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/architectural-cleanup-program.md"),
        "this spec stays in CANONICAL_SPECS"
    );
}
