//! Parse and validate `docs/debt/register.toml` (expiry, exemptions, orphans).

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEBT_SCHEMA: &str = "weeping-angel/debt-register/v1";

const ALLOWED_STATUS: [&str; 6] = [
    "open",
    "confirmed",
    "in-progress",
    "resolved",
    "rejected",
    "superseded",
];

const ALLOWED_SEVERITY: [&str; 4] = ["low", "medium", "high", "critical"];

/// Product-semantic stub checks that still skip-with-debt.
pub const STUB_EXEMPTION_CHECKS: [&str; 0] = [];

pub const KNOWN_CHECK_IDS: [&str; 15] = [
    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
];

const IMPLEMENTED_CHECKS: [&str; 7] = ["01", "02", "03", "04", "13", "14", "15"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsoDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Validate a debt-register TOML string. Rejects duplicate ids and
/// `status = "resolved"` without `regression_tests` or `repository_guard`.
pub fn validate_debt_register_str(text: &str) -> Result<BTreeSet<String>, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("debt register is not parseable TOML: {e}"))?;
    validate_debt_register_value(&value, evaluation_date()?)
}

pub fn validate_debt_register_file(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    validate_debt_register_str(&text)
}

pub fn load_and_validate_debt_register(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("docs/debt/register.toml");
    if !path.is_file() {
        return Err("docs/debt/register.toml is not a file".into());
    }
    validate_debt_register_file(&path)
}

pub fn evaluation_date() -> Result<IsoDate, String> {
    if let Ok(raw) = env::var("WEEPING_ANGEL_GUARD_AS_OF") {
        return parse_iso_date(&raw)
            .map_err(|e| format!("WEEPING_ANGEL_GUARD_AS_OF is malformed: {e}"));
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock before unix epoch: {e}"))?
        .as_secs();
    Ok(unix_days_to_date((secs / 86_400) as i64))
}

pub fn parse_iso_date(s: &str) -> Result<IsoDate, String> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        return Err(format!("date {s:?} is not YYYY-MM-DD"));
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| format!("date {s:?} has malformed year"))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| format!("date {s:?} has malformed month"))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| format!("date {s:?} has malformed day"))?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || year < 1970 {
        return Err(format!("date {s:?} is not a valid ISO calendar date"));
    }
    Ok(IsoDate { year, month, day })
}

fn unix_days_to_date(days: i64) -> IsoDate {
    // Howard Hinnant civil_from_days (days since 1970-01-01).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    IsoDate {
        year: y as i32,
        month: m as u32,
        day: d as u32,
    }
}

fn validate_debt_register_value(
    value: &toml::Value,
    as_of: IsoDate,
) -> Result<BTreeSet<String>, String> {
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
        if status == "resolved" {
            validate_resolved_guard_closure(finding, &id)?;
        }
        validate_check_refs(finding, &id)?;
        if is_live_exemption(finding, &id) {
            validate_live_exemption(finding, &id, as_of)?;
        }
    }
    Ok(ids)
}

fn validate_resolved_guard_closure(finding: &toml::Value, id: &str) -> Result<(), String> {
    let Some(nn) = id.strip_prefix("DEBT-GUARD-") else {
        return Ok(());
    };
    if STUB_EXEMPTION_CHECKS.contains(&nn) {
        return Err(format!(
            "resolved debt {id} still needs a live guard or named regression tests (check {nn} is still a stub)"
        ));
    }
    if matches!(nn, "14" | "15") {
        if !has_named_regression_tests(finding) {
            return Err(format!(
                "resolved debt {id} still needs named regression tests in addition to repository_guard"
            ));
        }
        if guard_id(finding) != Some(nn) {
            return Err(format!(
                "resolved debt {id} still needs a live guard (repository_guard = \"{nn}\")"
            ));
        }
        if !IMPLEMENTED_CHECKS.contains(&nn) {
            return Err(format!(
                "resolved debt {id} still needs a live guard (check {nn} is not implemented)"
            ));
        }
    }
    Ok(())
}

fn validate_check_refs(finding: &toml::Value, id: &str) -> Result<(), String> {
    if let Some(guard) = guard_id(finding)
        && !KNOWN_CHECK_IDS.contains(&guard)
    {
        return Err(format!(
            "orphaned debt {id}: repository_guard names unknown check {guard}"
        ));
    }
    if let Some(skip) = skip_check_id(finding)
        && !KNOWN_CHECK_IDS.contains(&skip)
    {
        return Err(format!(
            "orphaned debt {id}: skip_check names unknown check {skip}"
        ));
    }
    Ok(())
}

fn is_live_exemption(finding: &toml::Value, id: &str) -> bool {
    let status = finding.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if status == "resolved" {
        return false;
    }
    if id == "DEBT-DUP-ADR" {
        return true;
    }
    if skip_check_id(finding).is_some() {
        return true;
    }
    if let Some(nn) = id.strip_prefix("DEBT-GUARD-") {
        return STUB_EXEMPTION_CHECKS.contains(&nn);
    }
    false
}

fn validate_live_exemption(finding: &toml::Value, id: &str, as_of: IsoDate) -> Result<(), String> {
    for field in ["owner", "introduced", "severity", "remediation"] {
        required_exemption_field(finding, id, field)?;
    }
    let severity = finding
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !ALLOWED_SEVERITY.contains(&severity) {
        return Err(format!(
            "malformed finding {id}: severity must be low|medium|high|critical, got {severity:?}"
        ));
    }
    let introduced = required_exemption_field(finding, id, "introduced")?;
    parse_iso_date(&introduced).map_err(|e| format!("malformed finding {id}: introduced: {e}"))?;

    if guard_id(finding).is_none() {
        return Err(format!(
            "malformed finding {id}: live exemption requires repository_guard"
        ));
    }

    let expires = optional_date_field(finding, "expires")?;
    let review_by = optional_date_field(finding, "review_by")?;
    match (expires, review_by) {
        (None, None) => {
            return Err(format!(
                "malformed finding {id}: live exemption requires expires or review_by"
            ));
        }
        (Some(d), _) | (None, Some(d)) => {
            if d < as_of {
                return Err(format!("expired debt {id}"));
            }
        }
    }
    if let (Some(e), Some(r)) = (expires, review_by)
        && (e < as_of || r < as_of)
    {
        return Err(format!("expired debt {id}"));
    }
    Ok(())
}

fn optional_date_field(finding: &toml::Value, key: &str) -> Result<Option<IsoDate>, String> {
    match finding.get(key) {
        None => Ok(None),
        Some(toml::Value::String(s)) if s.is_empty() => Ok(None),
        Some(toml::Value::String(s)) => parse_iso_date(s)
            .map(Some)
            .map_err(|e| format!("malformed {key}: {e}")),
        Some(_) => Err(format!("malformed {key}: expected YYYY-MM-DD string")),
    }
}

fn required_exemption_field(
    finding: &toml::Value,
    id: &str,
    field: &str,
) -> Result<String, String> {
    match finding.get(field).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(format!(
            "malformed finding {id}: live exemption missing required {field}"
        )),
    }
}

fn required_nonempty(finding: &toml::Value, field: &str, idx: usize) -> Result<String, String> {
    match finding.get(field).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(format!("finding[{idx}] missing required non-empty {field}")),
    }
}

fn has_named_regression_tests(finding: &toml::Value) -> bool {
    finding
        .get("regression_tests")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().any(|x| x.as_str().is_some_and(|s| !s.is_empty())))
        .unwrap_or(false)
}

fn has_resolution_proof(finding: &toml::Value) -> bool {
    has_named_regression_tests(finding)
        || guard_id(finding).is_some()
        || matches!(
            finding.get("repository_guard"),
            Some(toml::Value::Boolean(true)) | Some(toml::Value::Integer(_))
        )
}

fn guard_id(finding: &toml::Value) -> Option<&str> {
    match finding.get("repository_guard") {
        Some(toml::Value::String(s)) if !s.is_empty() => Some(s.as_str()),
        _ => None,
    }
}

fn skip_check_id(finding: &toml::Value) -> Option<&str> {
    match finding.get("skip_check") {
        Some(toml::Value::String(s)) if !s.is_empty() => Some(s.as_str()),
        _ => None,
    }
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

    #[test]
    fn rejects_orphaned_unknown_check_id() {
        let extra = r#"repository_guard = "99""#;
        let text = format!("{}{}", HEADER, finding("DEBT-X", "open", extra));
        let err = validate_debt_register_str(&text).expect_err("orphan");
        assert!(err.contains("orphan"), "{err}");
    }

    #[test]
    fn rejects_expired_live_exemption() {
        let extra = r#"
owner = "owner"
introduced = "2026-01-01"
severity = "high"
remediation = "fix it"
repository_guard = "05"
skip_check = "05"
expires = "2020-01-01"
"#;
        let text = format!("{}{}", HEADER, finding("DEBT-GUARD-05", "open", extra));
        let value: toml::Value = text.parse().unwrap();
        let as_of = parse_iso_date("2026-08-19").unwrap();
        let err = validate_debt_register_value(&value, as_of).expect_err("expired");
        assert!(err.contains("expired"), "{err}");
    }

    #[test]
    fn rejects_malformed_severity_on_exemption() {
        let extra = r#"
owner = "owner"
introduced = "2026-01-01"
severity = "urgent"
remediation = "fix it"
repository_guard = "05"
skip_check = "05"
expires = "2027-01-01"
"#;
        let text = format!("{}{}", HEADER, finding("DEBT-GUARD-05", "open", extra));
        let err = validate_debt_register_str(&text).expect_err("malformed");
        assert!(err.contains("malformed"), "{err}");
    }
}
