//! Fixture-register tests for check 13 (tempdir).

use std::fs;
use std::path::Path;

use tempfile::tempdir;
use xtask::{run_guard, validate_debt_register_file, validate_debt_register_str};

fn write_minimal_repo(root: &Path, register: &str) {
    fs::create_dir_all(root.join("architecture")).unwrap();
    fs::create_dir_all(root.join("docs/debt")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-canonical-catalog")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-framework")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-assurance/src")).unwrap();
    fs::create_dir_all(root.join("crates/weeping-angel-evidence")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = [
    "crates/weeping-angel-canonical-catalog",
    "crates/weeping-angel-framework",
    "crates/weeping-angel-assurance",
    "crates/weeping-angel-evidence",
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
    ] {
        fs::write(
            root.join(format!("crates/{pkg}/Cargo.toml")),
            format!("[package]\nname = \"{pkg}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
        )
        .unwrap();
    }
    let live_domain =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../architecture/domain-ownership.toml");
    let domain_text = fs::read_to_string(&live_domain).unwrap_or_else(|e| {
        panic!("read live architecture/domain-ownership.toml for fixture: {e}")
    });
    fs::write(root.join("architecture/domain-ownership.toml"), domain_text).unwrap();
    fs::write(
        root.join("architecture/architecture.toml"),
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
paths = ["apps/cli/src/main.rs", "apps/cli/src/cli.rs"]

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
        root.join("architecture/invariants.toml"),
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
"#,
    )
    .unwrap();
    fs::write(
        root.join("architecture/forbidden-patterns.toml"),
        r#"schema = "weeping-angel/forbidden-patterns/v1"
"#,
    )
    .unwrap();
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
    fs::create_dir_all(root.join("apps/cli/src")).unwrap();
    fs::write(root.join("apps/cli/src/main.rs"), "").unwrap();
    fs::write(root.join("apps/cli/src/cli.rs"), "").unwrap();
    fs::create_dir_all(root.join("docs/adr")).unwrap();
    fs::create_dir_all(root.join("docs/specs")).unwrap();
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
    fs::write(root.join("docs/debt/register.toml"), register).unwrap();
}

fn seed_findings() -> String {
    let mut body = String::from("schema = \"weeping-angel/debt-register/v1\"\n");
    for id in [
        "DEBT-GUARD-05",
        "DEBT-GUARD-06",
        "DEBT-GUARD-07",
        "DEBT-GUARD-08",
        "DEBT-GUARD-09",
        "DEBT-GUARD-10",
        "DEBT-GUARD-11",
        "DEBT-GUARD-12",
    ] {
        let nn = id.trim_start_matches("DEBT-GUARD-");
        body.push_str(&format!(
            r#"
[[finding]]
id = "{id}"
title = "implemented"
status = "resolved"
summary = "product-law check is real"
owner = "fixture"
introduced = "2026-08-19"
severity = "medium"
remediation = "Guard {nn} evaluates product surfaces"
repository_guard = "{nn}"
regression_tests = ["sdd_architectural_cleanup_target"]
"#
        ));
    }
    body.push_str(
        r#"
[[finding]]
id = "DEBT-DUP-ADR"
title = "duplicate adr prefixes"
status = "confirmed"
summary = "historical prefix collisions"
owner = "fixture"
introduced = "2026-08-19"
severity = "medium"
remediation = "pin historical files"
repository_guard = "14"
review_by = "2027-12-31"
"#,
    );
    body
}

#[test]
fn tempfile_rejects_resolved_without_regression_tests_or_repository_guard() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("register.toml");
    fs::write(
        &path,
        r#"
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-BAD"
title = "closed too soon"
status = "resolved"
summary = "no proof arrays"
"#,
    )
    .unwrap();
    let err = validate_debt_register_file(&path).expect_err("must reject");
    assert!(err.contains("resolved"), "{err}");
    assert!(
        err.contains("regression_tests") || err.contains("repository_guard"),
        "{err}"
    );
}

#[test]
fn tempfile_rejects_duplicate_ids() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("register.toml");
    fs::write(
        &path,
        r#"
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-DUP"
title = "a"
status = "open"
summary = "a"

[[finding]]
id = "DEBT-DUP"
title = "b"
status = "confirmed"
summary = "b"
"#,
    )
    .unwrap();
    let err = validate_debt_register_file(&path).expect_err("must reject duplicate");
    assert!(err.contains("duplicate") || err.contains("unique"), "{err}");
}

#[test]
fn tempfile_accepts_resolved_with_proof() {
    let ok = r#"
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-OK"
title = "ok"
status = "resolved"
summary = "guarded"
repository_guard = "13"
"#;
    validate_debt_register_str(ok).expect("repository_guard is proof");
}

#[test]
fn guard_on_fixture_repo_runs_implemented_checks_and_skips_stubs() {
    let dir = tempdir().unwrap();
    write_minimal_repo(dir.path(), &seed_findings());
    let report = run_guard(dir.path());
    let rendered = report.render();
    assert!(!report.failed(), "{rendered}");
    for id in ["01", "02", "03", "04", "13"] {
        assert!(
            rendered.contains(&format!("{id}  ")) && rendered.contains("pass"),
            "expected pass for check {id}: {rendered}"
        );
    }
    assert!(
        rendered.contains("04  architecture-invariants  pass"),
        "Guard 04 must evaluate invariants: {rendered}"
    );
    for id in ["05", "06", "07", "08", "09", "10", "11", "12"] {
        assert!(
            rendered.contains(&format!("{id}  ")) && rendered.contains("pass"),
            "product-law check {id} must pass: {rendered}"
        );
    }
    for id in ["14", "15"] {
        assert!(
            rendered.contains(&format!("{id}  ")) && rendered.contains("pass"),
            "check {id} must be a real pass: {rendered}"
        );
    }
}

#[test]
fn product_law_checks_pass_without_skip_debt() {
    let dir = tempdir().unwrap();
    write_minimal_repo(
        dir.path(),
        r#"
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-ONLY"
title = "unrelated"
status = "open"
summary = "does not cover product-law checks"
owner = "fixture"
introduced = "2026-08-19"
severity = "medium"
remediation = "unrelated"
review_by = "2027-12-31"
"#,
    );
    let report = run_guard(dir.path());
    let rendered = report.render();
    assert!(
        !report.failed(),
        "05–12 must pass without skip-debt: {rendered}"
    );
    for id in ["05", "06", "07", "08", "09", "10", "11", "12"] {
        assert!(
            rendered.contains(&format!("{id}  ")) && rendered.contains("pass"),
            "{id} must pass: {rendered}"
        );
        assert!(
            !rendered.contains(&format!("skip(DEBT-GUARD-{id})")),
            "{id} must not skip: {rendered}"
        );
    }
}
