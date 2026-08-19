//! Replaceable inherent-scoring adapter consumed by the operational register.
//!
//! Prompt 05 owns scales, matrices, and [`crate::score_risk`]. This adapter
//! stores register JSON (levelId / cellId snapshots) and derives inherent
//! scores without a hardcoded 5×5 when no methodology document is in scope.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::risk::CiaImpactInputs;

/// Opaque methodology-shaped object (likelihood, impact, score, or rating).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MethodologyValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_id: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)]
    extra: BTreeMap<String, Value>,
}

impl MethodologyValue {
    pub fn is_empty(&self) -> bool {
        self.kind.is_none()
            && self.cell_id.is_none()
            && self.value.is_none()
            && self.methodology_id.is_none()
            && self.revision.is_none()
            && self.rating_id.is_none()
            && self.level_id.is_none()
            && self.extra.is_empty()
    }

    pub fn has_raw_level(&self) -> bool {
        self.level_id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty())
            || self
                .extra
                .get("likelihoodId")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.trim().is_empty())
            || self
                .extra
                .get("impactId")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.trim().is_empty())
    }

    fn raw_level(&self) -> Option<&str> {
        self.level_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .or_else(|| {
                self.extra
                    .get("likelihoodId")
                    .and_then(Value::as_str)
                    .filter(|id| !id.trim().is_empty())
            })
            .or_else(|| {
                self.extra
                    .get("impactId")
                    .and_then(Value::as_str)
                    .filter(|id| !id.trim().is_empty())
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RiskScoringError {
    #[error("methodology version is required to derive inherent score")]
    MissingMethodologyVersion,
    #[error("raw likelihood input is required to derive inherent score")]
    MissingLikelihood,
    #[error("raw impact input is required to derive inherent score")]
    MissingImpact,
}

/// Derive inherent score and rating from raw inputs + methodology version pin.
///
/// Deterministic: equal version + equal raw inputs ⇒ equal outputs. Not a
/// hardcoded 5×5; cell identity is the pair of authored level ids.
pub fn score_inherent(
    methodology_version: &str,
    likelihood: &MethodologyValue,
    impact: &MethodologyValue,
    _cia: Option<&CiaImpactInputs>,
) -> Result<(MethodologyValue, MethodologyValue), RiskScoringError> {
    let version = methodology_version.trim();
    if version.is_empty() {
        return Err(RiskScoringError::MissingMethodologyVersion);
    }
    let likelihood_id = likelihood
        .raw_level()
        .ok_or(RiskScoringError::MissingLikelihood)?;
    let impact_id = impact.raw_level().ok_or(RiskScoringError::MissingImpact)?;
    let cell_id = format!("{likelihood_id}-{impact_id}");
    let (methodology_id, revision) = split_methodology_version(version);
    let score = MethodologyValue {
        kind: Some("qualitative".into()),
        cell_id: Some(cell_id.clone()),
        value: None,
        methodology_id: None,
        revision: None,
        rating_id: None,
        level_id: None,
        extra: BTreeMap::new(),
    };
    let rating = MethodologyValue {
        kind: None,
        cell_id: None,
        value: None,
        methodology_id: Some(methodology_id),
        revision: Some(revision),
        rating_id: Some(format!("cell:{cell_id}")),
        level_id: None,
        extra: BTreeMap::new(),
    };
    Ok((score, rating))
}

fn split_methodology_version(version: &str) -> (String, u32) {
    match version.rsplit_once(':') {
        Some((id, rev)) if !id.is_empty() => {
            let revision = rev.parse::<u32>().unwrap_or(1);
            (id.to_string(), revision.max(1))
        }
        _ => (version.to_string(), 1),
    }
}
