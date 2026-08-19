use serde_json::Value;
use weeping_angel_evidence::EvidenceValue;

use crate::domain::ObservationCandidate;

use super::normalize::{EmitCtx, emit, repo_subject_id};
use crate::CollectorError;

pub const MODULE: &str = "workflows";

pub fn from_workflow_permissions(
    json: &Value,
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<ObservationCandidate>, CollectorError> {
    let subject = repo_subject_id(owner, name);
    let default = json
        .get("default_workflow_permissions")
        .and_then(Value::as_str)
        .unwrap_or("write");
    let default_write = default == "write";
    let permissions_minimized = default == "read";
    Ok(vec![emit(
        "evidence.cicd.workflow-permissions",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("default_write", EvidenceValue::from_bool(default_write)),
            (
                "permissions_minimized",
                EvidenceValue::from_bool(permissions_minimized),
            ),
        ],
        "workflow default permissions observation",
        ctx,
    )?])
}

pub fn from_environments(
    json: &Value,
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<ObservationCandidate>, CollectorError> {
    let envs = json
        .get("environments")
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| json.as_array().cloned())
        .unwrap_or_default();
    if envs.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for env in &envs {
        let env_name = env.get("name").and_then(Value::as_str).unwrap_or("unknown");
        let rules = env
            .get("protection_rules")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let reviewers = env
            .get("reviewers")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        let protected = rules || reviewers;
        let production = env_name.eq_ignore_ascii_case("production")
            || env_name.eq_ignore_ascii_case("prod")
            || env
                .get("production")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let subject = format!("repo:{owner}/{name}/environment:{env_name}");
        out.push(emit(
            "evidence.deployment.environment-protection",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("production", EvidenceValue::from_bool(production)),
                (
                    "authorization_required",
                    EvidenceValue::from_bool(reviewers || protected),
                ),
                ("protected", EvidenceValue::from_bool(protected)),
            ],
            "deployment environment protection observation",
            ctx,
        )?);
    }
    Ok(out)
}
