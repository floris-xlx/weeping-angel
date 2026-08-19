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
    fs::write(
        root.join("architecture/architecture.toml"),
        r#"schema = "weeping-angel/architecture/v1"

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
    for id in [
        "DEBT-GUARD-05",
        "DEBT-GUARD-06",
        "DEBT-GUARD-07",
        "DEBT-GUARD-08",
        "DEBT-GUARD-09",
        "DEBT-GUARD-10",
        "DEBT-GUARD-11",
        "DEBT-GUARD-12",
        "DEBT-GUARD-14",
        "DEBT-GUARD-15",
    ] {
        body.push_str(&format!(
            r#"
[[finding]]
id = "{id}"
title = "stub"
status = "open"
summary = "stub skip"
"#
        ));
    }
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
    for id in ["05", "06", "07", "08", "09", "10", "11", "12", "14", "15"] {
        assert!(
            rendered.contains(&format!("skip(DEBT-GUARD-{id})")),
            "stub {id} must skip-with-debt: {rendered}"
        );
    }
}

#[test]
fn stub_without_debt_finding_fails_closed() {
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
    let rendered = report.render();
    assert!(
        report.failed(),
        "stubs without debt must fail closed: {rendered}"
    );
    assert!(
        rendered.contains("not-yet-implemented: check 05"),
        "{rendered}"
    );
    assert!(
        rendered.contains("04  architecture-invariants  pass"),
        "Guard 04 is real and must evaluate even when remaining stubs fail closed: {rendered}"
    );
}
