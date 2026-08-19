//! Repository health gate: `cargo xtask guard`.
//!
//! Implemented checks this increment: 01, 02, 03, 13.
//! Checks 04–12 and 14–15 skip only with a registered `DEBT-GUARD-NN` finding
//! (fail closed with `not-yet-implemented: check NN` otherwise). Silent pass
//! is forbidden.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const ARCH_SCHEMA: &str = "weeping-angel/architecture/v1";
pub const INVARIANTS_SCHEMA: &str = "weeping-angel/architecture-invariants/v1";
pub const FORBIDDEN_SCHEMA: &str = "weeping-angel/forbidden-patterns/v1";
pub const DEBT_SCHEMA: &str = "weeping-angel/debt-register/v1";

const FORBIDDEN_PACKAGES: [&str; 2] = ["weeping-angel-catalog", "weeping-angel-assurance-cli"];

const ALLOWED_STATUS: [&str; 6] = [
    "open",
    "confirmed",
    "in-progress",
    "resolved",
    "rejected",
    "superseded",
];

/// Concept → (package name, required path needles).
pub const REQUIRED_OWNERSHIP: [(&str, &str, &[&str]); 7] = [
    (
        "catalog",
        "weeping-angel-canonical-catalog",
        &["crates/weeping-angel-canonical-catalog"],
    ),
    (
        "framework_compilation",
        "weeping-angel-framework",
        &["crates/weeping-angel-framework"],
    ),
    (
        "readiness_projection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/readiness.rs"],
    ),
    (
        "temporal_evidence_selection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/temporal.rs"],
    ),
    (
        "assessment_lineage",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/lineage.rs"],
    ),
    (
        "evidence_persistence",
        "weeping-angel-evidence",
        &["crates/weeping-angel-evidence"],
    ),
    (
        "assurance_cli",
        "weeping-angel",
        &["src/main.rs", "src/cli.rs"],
    ),
];

const STUB_CHECKS: [(&str, &str); 11] = [
    ("04", "architecture-invariants"),
    ("05", "catalog-ssot"),
    ("06", "framework-pack-parse"),
    ("07", "framework-digest"),
    ("08", "readiness-ssot"),
    ("09", "temporal-evidence-selection"),
    ("10", "assessment-lineage-rebuild"),
    ("11", "evidence-latest-vs-current"),
    ("12", "soa-invariants"),
    ("14", "adr-graph"),
    ("15", "spec-lifecycle"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Skip { debt_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: CheckStatus,
}

impl CheckResult {
    fn pass(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Pass,
        }
    }

    fn fail(id: &str, name: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Fail(message.into()),
        }
    }

    fn skip(id: &str, name: &str, debt_id: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Skip {
                debt_id: debt_id.into(),
            },
        }
    }

    pub fn report_line(&self) -> String {
        match &self.status {
            CheckStatus::Pass => format!("{}  {}  pass", self.id, self.name),
            CheckStatus::Fail(msg) => format!("{}  {}  fail  {msg}", self.id, self.name),
            CheckStatus::Skip { debt_id } => {
                format!("{}  {}  skip({debt_id})", self.id, self.name)
            }
        }
    }

    pub fn is_fail(&self) -> bool {
        matches!(self.status, CheckStatus::Fail(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardReport {
    pub checks: Vec<CheckResult>,
}

impl GuardReport {
    pub fn render(&self) -> String {
        let mut out = String::from("cargo xtask guard\n");
        for check in &self.checks {
            out.push_str(&check.report_line());
            out.push('\n');
        }
        out
    }

    pub fn failed(&self) -> bool {
        self.checks.iter().any(CheckResult::is_fail)
    }
}

/// Workspace root: parent of the `xtask` crate.
pub fn repo_root_from_xtask_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask crate lives under the workspace root")
        .to_path_buf()
}

pub fn main_with_args<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    match args.first().map(String::as_str) {
        Some("guard") => {
            let report = run_guard(&repo_root_from_xtask_manifest());
            print!("{}", report.render());
            if report.failed() { 1 } else { 0 }
        }
        _ => {
            eprintln!("usage: cargo xtask guard");
            2
        }
    }
}

pub fn run_guard(root: &Path) -> GuardReport {
    let mut checks = Vec::new();
    checks.push(check_01_architecture_manifest(root));
    checks.push(check_02_ownership(root));
    checks.push(check_03_forbidden_patterns(root));

    let (debt_result, finding_ids) = match load_and_validate_debt_register(root) {
        Ok(ids) => (CheckResult::pass("13", "debt-register"), ids),
        Err(err) => (
            CheckResult::fail("13", "debt-register", err),
            BTreeSet::new(),
        ),
    };
    checks.push(debt_result);

    for (id, name) in STUB_CHECKS {
        checks.push(stub_check(id, name, &finding_ids));
    }

    GuardReport { checks }
}

fn stub_check(id: &str, name: &str, finding_ids: &BTreeSet<String>) -> CheckResult {
    let debt_id = format!("DEBT-GUARD-{id}");
    if finding_ids.contains(&debt_id) {
        CheckResult::skip(id, name, debt_id)
    } else {
        CheckResult::fail(
            id,
            name,
            format!("not-yet-implemented: check {id} (no registered {debt_id} finding)"),
        )
    }
}

fn read_toml(path: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("parse {}: {e}", path.display()))
}

fn require_schema(value: &toml::Value, expected: &str, path: &Path) -> Result<(), String> {
    let got = value.get("schema").and_then(|s| s.as_str());
    if got == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "{} schema must be {expected}, got {got:?}",
            path.display()
        ))
    }
}

pub fn check_01_architecture_manifest(root: &Path) -> CheckResult {
    match check_01_inner(root) {
        Ok(()) => CheckResult::pass("01", "architecture-manifest"),
        Err(err) => CheckResult::fail("01", "architecture-manifest", err),
    }
}

fn check_01_inner(root: &Path) -> Result<(), String> {
    let path = root.join("architecture/architecture.toml");
    if !path.is_file() {
        return Err("architecture/architecture.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, ARCH_SCHEMA, &path)
}

pub fn check_02_ownership(root: &Path) -> CheckResult {
    match check_02_inner(root) {
        Ok(()) => CheckResult::pass("02", "canonical-ownership"),
        Err(err) => CheckResult::fail("02", "canonical-ownership", err),
    }
}

fn check_02_inner(root: &Path) -> Result<(), String> {
    let path = root.join("architecture/architecture.toml");
    let value = read_toml(&path)?;
    require_schema(&value, ARCH_SCHEMA, &path)?;
    let ownership = value
        .get("ownership")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "architecture.toml missing [ownership] table".to_string())?;

    for (concept, crate_name, required_paths) in REQUIRED_OWNERSHIP {
        let entry = ownership
            .get(concept)
            .ok_or_else(|| format!("ownership.{concept} is mandatory"))?;
        let got_crate = entry
            .get("crate")
            .and_then(|c| c.as_str())
            .ok_or_else(|| format!("ownership.{concept}.crate is required"))?;
        if got_crate != crate_name {
            return Err(format!(
                "ownership.{concept}.crate must be {crate_name}, got {got_crate}"
            ));
        }
        if FORBIDDEN_PACKAGES.contains(&got_crate) {
            return Err(format!(
                "ownership.{concept}.crate must not be hypothetical package {got_crate}"
            ));
        }
        let paths = entry
            .get("paths")
            .and_then(|p| p.as_array())
            .ok_or_else(|| format!("ownership.{concept}.paths is required"))?;
        if paths.is_empty() {
            return Err(format!("ownership.{concept}.paths must be non-empty"));
        }
        let path_strs: Vec<&str> = paths
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| format!("ownership.{concept}.paths entries must be strings"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for needle in required_paths {
            if !path_strs
                .iter()
                .any(|p| *p == *needle || p.contains(needle))
            {
                return Err(format!("ownership.{concept}.paths must include {needle}"));
            }
        }
        for rel in &path_strs {
            if !root.join(rel).exists() {
                return Err(format!(
                    "ownership.{concept} path {rel} does not exist on disk"
                ));
            }
        }
    }

    for (concept, entry) in ownership {
        let got_crate = entry.get("crate").and_then(|c| c.as_str()).unwrap_or("");
        if FORBIDDEN_PACKAGES.contains(&got_crate) {
            return Err(format!(
                "ownership.{concept} binds hypothetical package {got_crate}"
            ));
        }
        if let Some(paths) = entry.get("paths").and_then(|p| p.as_array()) {
            for p in paths {
                if let Some(rel) = p.as_str() {
                    if !root.join(rel).exists() {
                        return Err(format!(
                            "ownership.{concept} path {rel} does not exist on disk"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn check_03_forbidden_patterns(root: &Path) -> CheckResult {
    match check_03_inner(root) {
        Ok(()) => CheckResult::pass("03", "forbidden-patterns"),
        Err(err) => CheckResult::fail("03", "forbidden-patterns", err),
    }
}

fn check_03_inner(root: &Path) -> Result<(), String> {
    let path = root.join("architecture/forbidden-patterns.toml");
    if !path.is_file() {
        return Err("architecture/forbidden-patterns.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, FORBIDDEN_SCHEMA, &path)
}

/// Validate a debt-register TOML string. Rejects duplicate ids and
/// `status = "resolved"` without `regression_tests` or `repository_guard`.
pub fn validate_debt_register_str(text: &str) -> Result<BTreeSet<String>, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("debt register is not parseable TOML: {e}"))?;
    validate_debt_register_value(&value)
}

pub fn validate_debt_register_file(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    validate_debt_register_str(&text)
}

fn load_and_validate_debt_register(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("docs/debt/register.toml");
    if !path.is_file() {
        return Err("docs/debt/register.toml is not a file".into());
    }
    validate_debt_register_file(&path)
}

fn validate_debt_register_value(value: &toml::Value) -> Result<BTreeSet<String>, String> {
    let schema = value.get("schema").and_then(|s| s.as_str());
    if schema != Some(DEBT_SCHEMA) {
        return Err(format!(
            "debt register schema must be {DEBT_SCHEMA}, got {schema:?}"
        ));
    }
    let findings = value
        .get("finding")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "debt register must contain [[finding]] rows".to_string())?;

    let mut ids = BTreeSet::new();
    for (idx, finding) in findings.iter().enumerate() {
        let id = required_nonempty(finding, "id", idx)?;
        let _title = required_nonempty(finding, "title", idx)?;
        let _summary = required_nonempty(finding, "summary", idx)?;
        let status = required_nonempty(finding, "status", idx)?;
        if !ALLOWED_STATUS.contains(&status.as_str()) {
            return Err(format!("finding {id} has illegal status {status}"));
        }
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate finding id {id} (ids must be unique)"));
        }
        if status == "resolved" && !has_resolution_proof(finding) {
            return Err(format!(
                "resolved finding {id} is rejected without proof: list non-empty regression_tests or repository_guard"
            ));
        }
    }
    Ok(ids)
}

fn required_nonempty(finding: &toml::Value, field: &str, idx: usize) -> Result<String, String> {
    match finding.get(field).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(format!("finding[{idx}] missing required non-empty {field}")),
    }
}

fn has_resolution_proof(finding: &toml::Value) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "schema = \"weeping-angel/debt-register/v1\"\n";

    fn finding(id: &str, status: &str, extra: &str) -> String {
        format!(
            r#"
[[finding]]
id = "{id}"
title = "t"
status = "{status}"
summary = "s"
{extra}
"#
        )
    }

    #[test]
    fn schema_constants_match_spec() {
        assert_eq!(ARCH_SCHEMA, "weeping-angel/architecture/v1");
        assert_eq!(
            INVARIANTS_SCHEMA,
            "weeping-angel/architecture-invariants/v1"
        );
        assert_eq!(FORBIDDEN_SCHEMA, "weeping-angel/forbidden-patterns/v1");
        assert_eq!(DEBT_SCHEMA, "weeping-angel/debt-register/v1");
    }

    #[test]
    fn accepts_open_finding() {
        let text = format!("{}{}", HEADER, finding("DEBT-1", "open", ""));
        let ids = validate_debt_register_str(&text).expect("valid");
        assert!(ids.contains("DEBT-1"));
    }

    #[test]
    fn rejects_duplicate_finding_ids() {
        let text = format!(
            "{}{}{}",
            HEADER,
            finding("DEBT-1", "open", ""),
            finding("DEBT-1", "confirmed", "")
        );
        let err = validate_debt_register_str(&text).expect_err("duplicate");
        assert!(err.contains("duplicate") || err.contains("unique"), "{err}");
    }

    #[test]
    fn rejects_resolved_without_proof() {
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", ""));
        let err = validate_debt_register_str(&text).expect_err("resolved without proof");
        assert!(err.contains("resolved"), "{err}");
        assert!(
            err.contains("regression_tests") && err.contains("repository_guard"),
            "{err}"
        );
    }

    #[test]
    fn accepts_resolved_with_regression_tests() {
        let extra = r#"regression_tests = ["sdd_repository_integrity_target"]"#;
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", extra));
        validate_debt_register_str(&text).expect("proof via regression_tests");
    }

    #[test]
    fn accepts_resolved_with_repository_guard() {
        let extra = r#"repository_guard = "13""#;
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", extra));
        validate_debt_register_str(&text).expect("proof via repository_guard");
    }
}
