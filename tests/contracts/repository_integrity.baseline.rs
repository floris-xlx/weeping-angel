//! Baseline suite for Repository Integrity increment 1 (health gate).
//!
//! Characterization of CURRENT tree (`docs/specs/repository-integrity.md` §3)
//! on SHA `f560196c57e77df2573cfb9a4b384d3cf1c21e8a`: no `architecture/`
//! manifests, no `docs/debt/` register, no `xtask` workspace member or
//! `.cargo/config.toml` alias, CI does not run `cargo xtask guard`, and
//! hypothetical packages `weeping-angel-catalog` /
//! `weeping-angel-assurance-cli` do not exist.
//!
//! Encodes the found case (RI-B01–B10), not the desired gate. Does **not**
//! implement `cargo xtask guard`, architecture manifests, or debt files.
//! SUPERSEDED by `sdd_repository_integrity_target`. Tests are
//! `#[ignore = "superseded by sdd_repository_integrity_target"]`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

fn package_name(cargo_toml: &str) -> String {
    parse_toml(cargo_toml)["package"]["name"]
        .as_str()
        .unwrap_or_else(|| panic!("{cargo_toml} missing package.name"))
        .to_string()
}

fn workspace_members() -> Vec<String> {
    parse_toml("Cargo.toml")["workspace"]["members"]
        .as_array()
        .expect("workspace.members")
        .iter()
        .map(|v| v.as_str().expect("member string").to_string())
        .collect()
}

fn cargo_alias_xtask_defined() -> bool {
    let config = rel(".cargo/config.toml");
    if !config.is_file() {
        return false;
    }
    let text = fs::read_to_string(&config).unwrap_or_default();
    let Ok(value) = text.parse::<toml::Value>() else {
        return text.contains("xtask");
    };
    value
        .get("alias")
        .and_then(|a| a.get("xtask"))
        .and_then(|v| v.as_str())
        .is_some()
}

fn crate_dirs() -> Vec<String> {
    let crates = rel("crates");
    let entries =
        fs::read_dir(&crates).unwrap_or_else(|e| panic!("read {}: {e}", crates.display()));
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}

/// RI-B01: architecture ownership manifest is not a file.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b01_architecture_toml_is_absent() {
    assert!(
        !rel("architecture/architecture.toml").is_file(),
        "architecture/architecture.toml must be absent on characterization HEAD"
    );
}

/// RI-B02: architecture invariants file is not a file.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b02_invariants_toml_is_absent() {
    assert!(
        !rel("architecture/invariants.toml").is_file(),
        "architecture/invariants.toml must be absent on characterization HEAD"
    );
}

/// RI-B03: forbidden-patterns file is not a file.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b03_forbidden_patterns_toml_is_absent() {
    assert!(
        !rel("architecture/forbidden-patterns.toml").is_file(),
        "architecture/forbidden-patterns.toml must be absent on characterization HEAD"
    );
}

/// RI-B04: debt register is not a file. Snapshot/README are also absent.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b04_debt_register_is_absent() {
    assert!(
        !rel("docs/debt/register.toml").is_file(),
        "docs/debt/register.toml must be absent on characterization HEAD"
    );
    assert!(
        !rel("docs/debt/README.md").is_file(),
        "docs/debt/README.md must be absent on characterization HEAD"
    );
    assert!(
        !rel("docs/debt/baseline-2026-08.md").is_file(),
        "docs/debt/baseline-2026-08.md must be absent on characterization HEAD"
    );
    assert!(
        !rel("docs/debt").is_dir(),
        "docs/debt/ must be absent on characterization HEAD"
    );
}

/// RI-B05: workspace members are the seven product crates; `xtask` is not a member.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b05_workspace_members_do_not_include_xtask() {
    let members = workspace_members();
    let expected = [
        "crates/weeping-angel-assurance-ir",
        "crates/weeping-angel-framework",
        "crates/weeping-angel-evidence",
        "crates/weeping-angel-collector",
        "crates/weeping-angel-control-test",
        "crates/weeping-angel-assurance",
        "crates/weeping-angel-canonical-catalog",
    ];
    assert_eq!(
        members, expected,
        "characterization HEAD workspace members must be the seven product crates only"
    );
    assert!(
        !members
            .iter()
            .any(|m| m == "xtask" || m.ends_with("/xtask")),
        "workspace.members must not contain xtask; found {members:?}"
    );
    assert!(
        !rel("xtask").is_dir(),
        "xtask/ directory must be absent on characterization HEAD"
    );
    assert!(
        !rel("xtask/Cargo.toml").is_file(),
        "xtask/Cargo.toml must be absent on characterization HEAD"
    );
}

/// RI-B06: cargo alias `xtask` is undefined (`.cargo/config.toml` missing or no alias).
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b06_cargo_xtask_alias_is_absent() {
    assert!(
        !cargo_alias_xtask_defined(),
        ".cargo/config.toml must not define alias xtask on characterization HEAD"
    );
    assert!(
        !rel(".cargo/config.toml").is_file(),
        ".cargo/config.toml must be absent on characterization HEAD"
    );
}

/// RI-B07: `cargo xtask guard` fails (unknown cargo subcommand).
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b07_cargo_xtask_guard_fails() {
    let output = Command::new("cargo")
        .args(["xtask", "guard"])
        .current_dir(repo_root())
        .output()
        .expect("spawn cargo xtask guard");
    assert!(
        !output.status.success(),
        "cargo xtask guard must fail on characterization HEAD; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no such command") && stderr.contains("xtask"),
        "expected unknown-command failure for cargo xtask; stderr={stderr}"
    );
}

/// RI-B08: `cargo run -p xtask -- guard` fails (unknown package).
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b08_cargo_run_package_xtask_fails() {
    let output = Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "guard"])
        .current_dir(repo_root())
        .output()
        .expect("spawn cargo run -p xtask");
    assert!(
        !output.status.success(),
        "cargo run -p xtask must fail on characterization HEAD; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("xtask")
            && (stderr.contains("not found") || stderr.contains("did not match")),
        "expected unknown-package failure for -p xtask; stderr={stderr}"
    );
}

/// RI-B09: CI does not run a repository health gate.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b09_ci_does_not_contain_xtask_guard() {
    let ci = read(".github/workflows/ci.yml");
    assert!(
        !ci.contains("xtask guard"),
        ".github/workflows/ci.yml must not contain 'xtask guard' on characterization HEAD"
    );
    assert!(
        ci.contains("cargo fmt --all -- --check"),
        "CI still runs rustfmt check"
    );
    assert!(
        ci.contains("cargo clippy --all-targets --features demo -- -D warnings"),
        "CI still runs clippy with --features demo, not --workspace"
    );
    assert!(
        ci.contains("cargo test --features demo --all-targets"),
        "CI still runs cargo test --features demo --all-targets"
    );
    assert!(
        !ci.contains("cargo clippy --workspace"),
        "CI clippy is not --workspace on characterization HEAD"
    );
}

/// RI-B10: hypothetical packages do not exist; live crate names are the found map.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b10_hypothetical_packages_do_not_exist() {
    let members = workspace_members();
    let dirs = crate_dirs();
    assert!(
        !dirs.iter().any(|d| d == "weeping-angel-catalog"),
        "crates/weeping-angel-catalog must not exist"
    );
    assert!(
        !dirs.iter().any(|d| d == "weeping-angel-assurance-cli"),
        "crates/weeping-angel-assurance-cli must not exist"
    );
    assert!(
        !members.iter().any(|m| m.contains("weeping-angel-catalog")
            && !m.contains("weeping-angel-canonical-catalog")),
        "workspace must not list weeping-angel-catalog; members={members:?}"
    );
    assert!(
        !members
            .iter()
            .any(|m| m.contains("weeping-angel-assurance-cli")),
        "workspace must not list weeping-angel-assurance-cli; members={members:?}"
    );
    assert!(!rel("crates/weeping-angel-catalog/Cargo.toml").is_file());
    assert!(!rel("crates/weeping-angel-assurance-cli/Cargo.toml").is_file());

    assert_eq!(
        package_name("Cargo.toml"),
        "weeping-angel",
        "root package is weeping-angel (assurance CLI lives here)"
    );
    assert_eq!(
        package_name("crates/weeping-angel-canonical-catalog/Cargo.toml"),
        "weeping-angel-canonical-catalog"
    );
    assert_eq!(
        package_name("crates/weeping-angel-framework/Cargo.toml"),
        "weeping-angel-framework"
    );
    assert_eq!(
        package_name("crates/weeping-angel-assurance/Cargo.toml"),
        "weeping-angel-assurance"
    );
    assert_eq!(
        package_name("crates/weeping-angel-evidence/Cargo.toml"),
        "weeping-angel-evidence"
    );
    assert!(rel("src/main.rs").is_file());
    assert!(rel("src/cli.rs").is_file());
    let cli = read("src/cli.rs");
    assert!(
        cli.contains("enum AssuranceCommand"),
        "assurance CLI is root src/cli.rs AssuranceCommand, not a separate crate"
    );
    assert!(rel("crates/weeping-angel-assurance/src/readiness.rs").is_file());
    assert!(rel("crates/weeping-angel-assurance/src/temporal.rs").is_file());
    assert!(rel("crates/weeping-angel-assurance/src/lineage.rs").is_file());
    assert!(rel("crates/weeping-angel-control-test/src/temporal.rs").is_file());
}

/// Architecture directory itself is absent (not an empty tree).
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn architecture_directory_is_absent() {
    assert!(
        !rel("architecture").exists(),
        "architecture/ must not exist on characterization HEAD"
    );
}

/// Spec-first docs exist; they are not product files.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn spec_first_docs_exist_without_product_files() {
    assert!(rel("docs/specs/repository-integrity.md").is_file());
    assert!(rel("docs/sdd/repository-integrity.md").is_file());
    assert!(rel("docs/adr/0009-repository-health-gate.md").is_file());
    let spec = read("docs/specs/repository-integrity.md");
    assert!(spec.contains("remaining_backlog"));
    assert!(spec.contains("weeping-angel-canonical-catalog"));
    assert!(spec.contains("RI-B01"));
}
