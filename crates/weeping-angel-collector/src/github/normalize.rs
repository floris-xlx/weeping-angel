use chrono::{DateTime, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
};

use crate::{CollectorError, CollectorScope};

pub struct EmitCtx<'a> {
    pub collected_at: DateTime<Utc>,
    pub scope: &'a CollectorScope,
    pub asset: AssetId,
}

pub fn emit(
    ty: &str,
    facts: Vec<(&str, EvidenceValue)>,
    narrative: &str,
    ctx: &EmitCtx<'_>,
) -> Result<EvidenceEnvelope, CollectorError> {
    let mut obs = EvidenceObservation::new(EvidenceType::new(ty)).with_narrative(narrative);
    for (k, v) in facts {
        obs = obs.with_value(k, v);
    }
    let prov = EvidenceProvenance {
        collector_id: "collector.github".into(),
        collected_at: ctx.collected_at,
        scope: ctx.scope.as_label(),
        asset: ctx.asset.clone(),
    };
    Ok(EvidenceEnvelope::seal(obs, prov)?)
}

pub fn repo_subject_id(owner: &str, name: &str) -> String {
    format!("repo:{owner}/{name}")
}

pub fn visibility_of(repo: &Value) -> String {
    repo.get("visibility")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            repo.get("private")
                .and_then(Value::as_bool)
                .map(|p| if p { "private" } else { "public" }.to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

pub fn default_branch_of(repo: &Value) -> Option<&str> {
    repo.get("default_branch").and_then(Value::as_str)
}

pub fn archived_of(repo: &Value) -> bool {
    repo.get("archived")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn owner_of(repo: &Value, fallback: &str) -> String {
    repo.get("owner")
        .and_then(|o| o.get("login"))
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

pub fn repository_envelopes(
    repo: &Value,
    owner: &str,
    name: &str,
    in_scope: bool,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let mut out = Vec::new();
    let subject = repo_subject_id(owner, name);
    let archived = archived_of(repo);
    out.push(emit(
        "evidence.repository.inventory",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("archived", EvidenceValue::from_bool(archived)),
            ("in_scope", EvidenceValue::from_bool(in_scope)),
            ("owner_id", EvidenceValue::string(owner_of(repo, owner))),
        ],
        "repository inventory observation",
        ctx,
    )?);
    out.push(emit(
        "evidence.repository.visibility",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("visibility", EvidenceValue::string(visibility_of(repo))),
        ],
        "repository visibility observation",
        ctx,
    )?);
    if let Some(branch) = default_branch_of(repo) {
        out.push(emit(
            "evidence.repository.default-branch",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("default_branch", EvidenceValue::string(branch)),
            ],
            "default branch observation",
            ctx,
        )?);
    }
    out.push(emit(
        "inventory.subject",
        vec![
            ("kind", EvidenceValue::string("repository")),
            ("id", EvidenceValue::string(&subject)),
        ],
        "in-scope repository subject",
        ctx,
    )?);
    Ok(out)
}
