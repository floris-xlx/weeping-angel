use serde_json::Value;
use weeping_angel_evidence::EvidenceValue;

use weeping_angel_evidence::EvidenceEnvelope;

use super::normalize::{EmitCtx, emit, repo_subject_id};
use crate::CollectorError;

pub const MODULE: &str = "rulesets";

pub fn from_rulesets_json(
    json: &Value,
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let rules: Vec<&Value> = if let Some(arr) = json.as_array() {
        arr.iter().collect()
    } else {
        Vec::new()
    };
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let subject = repo_subject_id(owner, name);
    let signing = rules.iter().any(|r| {
        r.get("rules")
            .and_then(Value::as_array)
            .map(|rs| {
                rs.iter()
                    .any(|x| x.get("type").and_then(Value::as_str) == Some("required_signatures"))
            })
            .unwrap_or(false)
            || r.get("type").and_then(Value::as_str) == Some("required_signatures")
    });
    let mut out = Vec::new();
    if signing {
        out.push(emit(
            "evidence.repository.commit-signing",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("signing_required", EvidenceValue::from_bool(true)),
            ],
            "ruleset commit signing observation",
            ctx,
        )?);
    }
    Ok(out)
}
