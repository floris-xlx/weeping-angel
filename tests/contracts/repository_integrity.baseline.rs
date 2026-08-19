//! Baseline suite for Repository Integrity.
//!
//! Increment 1 (RI-B01–B10): characterization of SHA `f560196c` absences
//! (`docs/specs/repository-integrity.md` §3). SUPERSEDED by
//! `sdd_repository_integrity_target` — those tests stay
//! `#[ignore = "superseded by sdd_repository_integrity_target"]`.
//!
//! Increment 2 (RI-B11–B18): characterization of CURRENT increment-1 /
//! ADR-0010 behavior (`docs/specs/repository-integrity.md` §12). Encodes the
//! found case (xtask monolith, hard-coded policy, Guard 14/15 skip-with-debt,
//! weak debt exemptions, JSON without schema/counts, no spec-lifecycle /
//! adr-identity files). Does **not** implement increment-2 product code.

use std::fs;
use std::path::{Path, PathBuf};
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
    let mut out = String::new();
    for rel_rs in xtask_src_rel_rs() {
        out.push_str(&read(&format!("xtask/src/{rel_rs}")));
        out.push('\n');
    }
    out
}

fn debt_findings() -> Vec<toml::Value> {
    parse_toml("docs/debt/register.toml")
        .get("finding")
        .and_then(|f| f.as_array())
        .expect("[[finding]]")
        .clone()
}

fn finding_by_id<'a>(findings: &'a [toml::Value], id: &str) -> &'a toml::Value {
    findings
        .iter()
        .find(|f| f.get("id").and_then(|v| v.as_str()) == Some(id))
        .unwrap_or_else(|| panic!("register must contain {id}"))
}

fn field_present(finding: &toml::Value, key: &str) -> bool {
    match finding.get(key) {
        None => false,
        Some(toml::Value::String(s)) => !s.is_empty(),
        Some(toml::Value::Array(a)) => !a.is_empty(),
        Some(_) => true,
    }
}

/// RI-B11: xtask/src Rust sources are only lib.rs and main.rs (monolith).
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b11_xtask_src_is_lib_and_main_only() {
    let files = xtask_src_rel_rs();
    assert_eq!(
        files,
        vec!["lib.rs".to_string(), "main.rs".to_string()],
        "increment-1/ADR-0010 xtask/src must be a two-file monolith; found {files:?}"
    );
    for forbidden in [
        "model.rs",
        "architecture.rs",
        "debt.rs",
        "report.rs",
        "checks.rs",
    ] {
        assert!(
            !files
                .iter()
                .any(|f| f == forbidden || f.ends_with(forbidden)),
            "xtask/src must not contain {forbidden} on CURRENT tree; found {files:?}"
        );
    }
    assert!(
        !files.iter().any(|f| f.starts_with("checks")),
        "xtask/src must not contain checks* modules on CURRENT tree; found {files:?}"
    );
}

/// RI-B12: policy constants live in Rust; remaining stubs include 14 and 15.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b12_policy_is_hard_coded_and_stubs_include_14_15() {
    let src = xtask_src_text();
    for needle in [
        "REQUIRED_OWNERSHIP",
        "OWNERSHIP_KINDS",
        "FORBIDDEN_PACKAGES",
    ] {
        assert!(
            src.contains(needle),
            "xtask sources must contain {needle} as a hard-coded policy constant"
        );
    }
    assert!(
        src.contains(r#"("14", "adr-graph")"#) || src.contains("\"14\", \"adr-graph\""),
        "remaining stubs must include check 14 adr-graph; xtask sources did not"
    );
    assert!(
        src.contains(r#"("15", "spec-lifecycle")"#) || src.contains("\"15\", \"spec-lifecycle\""),
        "remaining stubs must include check 15 spec-lifecycle; xtask sources did not"
    );
}

/// RI-B13: RepositoryModel.source_files is a path list; needles reread disk.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b13_repository_model_source_files_reread_disk() {
    let lib = read("xtask/src/lib.rs");
    assert!(
        lib.contains("pub struct RepositoryModel"),
        "RepositoryModel must be defined in xtask/src/lib.rs"
    );
    assert!(
        lib.contains("pub source_files: Vec<String>"),
        "RepositoryModel must expose source_files as Vec<String>"
    );
    assert!(
        lib.contains("fn source_contains") && lib.contains("fs::read_to_string"),
        "source needle evaluation must use fs::read_to_string (no cached source map)"
    );
    for cache_field in [
        "source_cache",
        "source_index",
        "source_text",
        "normalized_source",
        "source_map",
    ] {
        assert!(
            !lib.contains(&format!("pub {cache_field}"))
                && !lib.contains(&format!("{cache_field}:")),
            "RepositoryModel must not expose cached source field {cache_field} on CURRENT tree"
        );
    }
}

/// RI-B14: live guard report skips Guard 14 and 15 with DEBT-GUARD-NN.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b14_guard_skips_14_and_15_with_debt() {
    let output = cargo_xtask_guard_args(&[]);
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "cargo xtask guard must be invocable on CURRENT tree; output={combined}"
    );
    assert!(
        combined.contains("skip(DEBT-GUARD-14)")
            || combined.contains("14  adr-graph  skip(DEBT-GUARD-14)"),
        "live report must skip Guard 14 with DEBT-GUARD-14; output={combined}"
    );
    assert!(
        combined.contains("skip(DEBT-GUARD-15)")
            || combined.contains("15  spec-lifecycle  skip(DEBT-GUARD-15)"),
        "live report must skip Guard 15 with DEBT-GUARD-15; output={combined}"
    );
}

/// RI-B15: live skip exemptions omit increment-2 exemption metadata.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b15_live_guard_exemptions_omit_expiry_metadata() {
    let findings = debt_findings();
    let ids = [
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
    ];
    for id in ids {
        let finding = finding_by_id(&findings, id);
        let complete = field_present(finding, "owner")
            && field_present(finding, "introduced")
            && field_present(finding, "severity")
            && field_present(finding, "remediation")
            && (field_present(finding, "expires") || field_present(finding, "review_by"));
        assert!(
            !complete,
            "{id} must omit at least one of owner/introduced/severity/remediation/expires|review_by on CURRENT tree; finding={finding}"
        );
    }
}

/// RI-B16: JSON report is checks/violations/skipped/debt_exemptions/duration only.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b16_guard_json_has_duration_and_no_schema_counts() {
    let output = cargo_xtask_guard_args(&["--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo xtask guard --json must succeed; stdout={stdout} stderr={stderr}"
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("guard --json must emit JSON object; parse error {e}; stdout={stdout}")
    });
    let obj = value.as_object().expect("JSON object");
    for key in [
        "checks",
        "violations",
        "skipped",
        "debt_exemptions",
        "duration",
    ] {
        assert!(
            obj.contains_key(key),
            "JSON must include {key}; got {obj:?}"
        );
    }
    let duration = obj.get("duration").and_then(|d| d.as_object());
    let duration = duration.expect("duration object");
    for key in ["secs", "nanos", "as_secs_f64"] {
        assert!(
            duration.contains_key(key),
            "duration must include {key}; got {duration:?}"
        );
    }
    assert!(
        !obj.contains_key("schema"),
        "CURRENT JSON must not include a report schema field"
    );
    assert!(
        !obj.contains_key("version"),
        "CURRENT JSON must not include a report version field"
    );
    assert!(
        !obj.contains_key("counts"),
        "CURRENT JSON must not include an aggregate counts object"
    );
}

/// RI-B17: DEBT-GUARD-14 and DEBT-GUARD-15 remain open stubs.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b17_debt_guard_14_and_15_are_open() {
    let findings = debt_findings();
    for id in ["DEBT-GUARD-14", "DEBT-GUARD-15"] {
        let finding = finding_by_id(&findings, id);
        assert_eq!(
            finding.get("status").and_then(|s| s.as_str()),
            Some("open"),
            "{id} must have status = open on CURRENT tree"
        );
    }
}

/// RI-B18: increment-2 architecture metadata files are absent.
#[ignore = "superseded by sdd_repository_integrity_target"]
#[test]
fn ri_b18_spec_lifecycle_and_adr_identity_files_are_absent() {
    assert!(
        !rel("architecture/spec-lifecycle.toml").is_file(),
        "architecture/spec-lifecycle.toml must not exist on CURRENT increment-1/ADR-0010 tree"
    );
    assert!(
        !rel("architecture/adr-identity.toml").is_file(),
        "architecture/adr-identity.toml must not exist on CURRENT increment-1/ADR-0010 tree"
    );
}
