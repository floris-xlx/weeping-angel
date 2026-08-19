use serde_json::Value;
use weeping_angel_evidence::EvidenceValue;

use weeping_angel_evidence::EvidenceEnvelope;

use super::normalize::{EmitCtx, emit, repo_subject_id};
use crate::CollectorError;

pub const MODULE: &str = "security";

fn status_enabled(repo: &Value, pointer: &str) -> Option<bool> {
    repo.pointer(pointer)
        .and_then(Value::as_str)
        .map(|s| s == "enabled")
}

pub fn from_repo_security(
    repo: &Value,
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let analysis = repo.get("security_and_analysis");
    if analysis.is_none() {
        return Ok(Vec::new());
    }
    let subject = repo_subject_id(owner, name);
    let secret =
        status_enabled(repo, "/security_and_analysis/secret_scanning/status").unwrap_or(false);
    let code = status_enabled(repo, "/security_and_analysis/code_scanning/status")
        .or_else(|| status_enabled(repo, "/security_and_analysis/advanced_security/status"))
        .unwrap_or(false);
    let dependabot = status_enabled(
        repo,
        "/security_and_analysis/dependabot_security_updates/status",
    )
    .or_else(|| {
        status_enabled(
            repo,
            "/security_and_analysis/dependabot_vulnerability_alerts/status",
        )
    })
    .unwrap_or(false);

    let mut out = Vec::new();
    out.push(emit(
        "evidence.repository.security-scanning",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("secret_scanning_enabled", EvidenceValue::from_bool(secret)),
            ("code_scanning_enabled", EvidenceValue::from_bool(code)),
            ("applicable", EvidenceValue::from_bool(true)),
        ],
        "security scanning observation",
        ctx,
    )?);
    out.push(emit(
        "evidence.repository.dependency-scanning",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            (
                "dependency_scanning_enabled",
                EvidenceValue::from_bool(dependabot),
            ),
            ("updates_monitored", EvidenceValue::from_bool(dependabot)),
        ],
        "dependency scanning observation",
        ctx,
    )?);
    Ok(out)
}
