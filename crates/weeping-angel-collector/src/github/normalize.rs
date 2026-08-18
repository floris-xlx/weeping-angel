use chrono::{DateTime, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

use crate::{CollectorError, CollectorScope};

pub fn repository_facts(
    repo: &Value,
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
    out.push(seal(
        "source.repository.exists",
        &[("exists", "true")],
        "repository exists",
        prov(),
    )?);
    let visibility = repo
        .get("visibility")
        .and_then(Value::as_str)
        .or_else(|| {
            repo.get("private")
                .and_then(Value::as_bool)
                .map(|p| if p { "private" } else { "public" })
        })
        .unwrap_or("unknown");
    out.push(seal(
        "source.repository.visibility",
        &[("visibility", visibility)],
        "repository visibility",
        prov(),
    )?);
    if let Some(branch) = repo.get("default_branch").and_then(Value::as_str) {
        out.push(seal(
            "source.default_branch",
            &[("name", branch)],
            "default branch",
            prov(),
        )?);
    }
    let archived = repo
        .get("archived")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    out.push(seal(
        "source.repository.archived",
        &[("archived", if archived { "true" } else { "false" })],
        "repository archived flag",
        prov(),
    )?);
    Ok(out)
}

fn seal(
    ty: &str,
    facts: &[(&str, &str)],
    narrative: &str,
    prov: EvidenceProvenance,
) -> Result<EvidenceEnvelope, CollectorError> {
    let mut obs = EvidenceObservation::new(EvidenceType::new(ty)).with_narrative(narrative);
    for (k, v) in facts {
        obs = obs.with_fact(*k, *v);
    }
    Ok(EvidenceEnvelope::seal(obs, prov)?)
}
