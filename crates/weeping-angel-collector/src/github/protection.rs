use chrono::{DateTime, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

use crate::{CollectorError, CollectorScope};

pub fn from_protection_json(
    json: &Value,
    asset: &AssetId,
    collected_at: DateTime<Utc>,
    scope: &CollectorScope,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let mut out = Vec::new();
    let prov = || EvidenceProvenance {
        collector_id: "collector.github".into(),
        collected_at,
        scope: scope.as_label(),
        asset: asset.clone(),
    };
    out.push(fact(
        "source.branch.protection",
        "enabled",
        "true",
        "branch protection is configured",
        prov(),
    )?);
    let reviews = json
        .pointer("/required_pull_request_reviews/required_approving_review_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    out.push(fact(
        "source.branch.required_reviews",
        "count",
        &reviews.to_string(),
        "required approving review count",
        prov(),
    )?);
    let force = json
        .pointer("/allow_force_pushes/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    out.push(fact(
        "source.branch.force_push_protection",
        "enabled",
        if force { "false" } else { "true" },
        "force-push protection",
        prov(),
    )?);
    let deletion = json
        .pointer("/allow_deletions/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    out.push(fact(
        "source.branch.deletion_protection",
        "enabled",
        if deletion { "false" } else { "true" },
        "deletion protection",
        prov(),
    )?);
    let checks = json.pointer("/required_status_checks").is_some();
    out.push(fact(
        "source.branch.required_status_checks",
        "configured",
        if checks { "true" } else { "false" },
        "required status checks",
        prov(),
    )?);
    Ok(out)
}

fn fact(
    ty: &str,
    key: &str,
    value: &str,
    narrative: &str,
    prov: EvidenceProvenance,
) -> Result<EvidenceEnvelope, CollectorError> {
    let obs = EvidenceObservation::new(EvidenceType::new(ty))
        .with_fact(key, value)
        .with_narrative(narrative);
    Ok(EvidenceEnvelope::seal(obs, prov)?)
}
