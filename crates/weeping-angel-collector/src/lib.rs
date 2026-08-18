//! Collectors emit observations of declared evidence types. They cannot declare compliance.

use std::collections::BTreeSet;

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    looks_like_compliance_claim, EvidenceEnvelope, EvidenceError, EvidenceObservation,
    EvidenceProvenance, EvidenceType,
};

/// Fixed collection instant so fixture normalize is deterministic.
const FIXTURE_COLLECTED_AT: (i32, u32, u32, u32, u32, u32) = (2026, 8, 18, 12, 0, 0);

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("observation is a compliance claim: {narrative}")]
    ComplianceClaim { narrative: String },
    #[error("collector attempted to emit a framework result: {detail}")]
    FrameworkResult { detail: String },
    #[error("undeclared evidence type: {evidence_type}")]
    UndeclaredEvidenceType { evidence_type: String },
    #[error("asset out of scope: {asset}")]
    OutOfScope { asset: String },
    #[error("evidence seal failed: {0}")]
    Seal(#[from] EvidenceError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectorDescriptor {
    pub id: String,
    pub version: String,
    pub evidence_types: BTreeSet<EvidenceType>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectorScope {
    allowed: BTreeSet<AssetId>,
}

impl CollectorScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_asset(mut self, asset: AssetId) -> Self {
        self.allowed.insert(asset);
        self
    }

    pub fn allows(&self, asset: &AssetId) -> bool {
        self.allowed.contains(asset)
    }

    pub fn as_label(&self) -> String {
        self.allowed
            .iter()
            .map(|a| a.as_str().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub trait EvidenceCollector {
    fn descriptor(&self) -> CollectorDescriptor;
    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>;
}

#[derive(Debug, Clone)]
pub struct FixtureCollector {
    id: String,
    version: String,
    evidence_types: BTreeSet<EvidenceType>,
    planned: Vec<(AssetId, EvidenceObservation)>,
}

impl FixtureCollector {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            evidence_types: BTreeSet::new(),
            planned: Vec::new(),
        }
    }

    pub fn with_evidence_types(mut self, types: impl IntoIterator<Item = EvidenceType>) -> Self {
        self.evidence_types.extend(types);
        self
    }

    pub fn with_planned(mut self, asset: AssetId, observation: EvidenceObservation) -> Self {
        self.planned.push((asset, observation));
        self
    }
}

impl EvidenceCollector for FixtureCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: self.id.clone(),
            version: self.version.clone(),
            evidence_types: self.evidence_types.clone(),
        }
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        let collected_at = Utc
            .with_ymd_and_hms(
                FIXTURE_COLLECTED_AT.0,
                FIXTURE_COLLECTED_AT.1,
                FIXTURE_COLLECTED_AT.2,
                FIXTURE_COLLECTED_AT.3,
                FIXTURE_COLLECTED_AT.4,
                FIXTURE_COLLECTED_AT.5,
            )
            .unwrap();
        let mut out = Vec::new();
        for (asset, observation) in &self.planned {
            if !scope.allows(asset) {
                return Err(CollectorError::OutOfScope {
                    asset: asset.to_string(),
                });
            }
            if observation.evidence_type().as_str() == "control_test_result" {
                return Err(CollectorError::FrameworkResult {
                    detail: observation.narrative().to_string(),
                });
            }
            if !self.evidence_types.contains(observation.evidence_type()) {
                return Err(CollectorError::UndeclaredEvidenceType {
                    evidence_type: observation.evidence_type().to_string(),
                });
            }
            if looks_like_compliance_claim(observation.narrative()) {
                return Err(CollectorError::ComplianceClaim {
                    narrative: observation.narrative().to_string(),
                });
            }
            let provenance = EvidenceProvenance {
                collector_id: self.id.clone(),
                collected_at,
                scope: scope.as_label(),
                asset: asset.clone(),
            };
            out.push(EvidenceEnvelope::seal(observation.clone(), provenance)?);
        }
        Ok(out)
    }
}
