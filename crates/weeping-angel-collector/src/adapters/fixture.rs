use std::collections::BTreeSet;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceObservation, EvidenceType};

use crate::domain::{
    CollectionCoverage, CollectionRequest, CollectorCapabilities, CollectorDescriptor,
    CollectorInstance, CollectorScope, CredentialRef, ObservationBatch, ObservationCandidate,
};
use crate::ports::CollectorAdapter;
use crate::{CollectorError, EvidenceCollector};

/// Fixed collection instant so fixture normalize is deterministic.
const FIXTURE_COLLECTED_AT: (i32, u32, u32, u32, u32, u32) = (2026, 8, 18, 12, 0, 0);

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

    fn instance(&self) -> CollectorInstance {
        CollectorInstance::new(
            format!("fixture:{}", self.id),
            self.id.clone(),
            CredentialRef::new(format!("fixture:{}", self.id)),
        )
    }

    fn fixture_instant() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(
            FIXTURE_COLLECTED_AT.0,
            FIXTURE_COLLECTED_AT.1,
            FIXTURE_COLLECTED_AT.2,
            FIXTURE_COLLECTED_AT.3,
            FIXTURE_COLLECTED_AT.4,
            FIXTURE_COLLECTED_AT.5,
        )
        .unwrap()
    }
}

impl CollectorAdapter for FixtureCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: self.id.clone(),
            version: self.version.clone(),
            evidence_types: self.evidence_types.clone(),
            provider_family: "fixture".into(),
            subject_types: BTreeSet::from(["repository".into()]),
            capabilities: CollectorCapabilities {
                offline: true,
                worker_safe: true,
                ..CollectorCapabilities::default()
            },
            required_permissions: Vec::new(),
        }
    }

    fn collect_observations(
        &self,
        _instance: &CollectorInstance,
        request: &CollectionRequest,
    ) -> Result<ObservationBatch, CollectorError> {
        let collected_at = Self::fixture_instant();
        let mut candidates = Vec::new();
        for (asset, observation) in &self.planned {
            if !request.scope.allows(asset) {
                return Err(CollectorError::OutOfScope {
                    asset: asset.to_string(),
                });
            }
            candidates.push(ObservationCandidate {
                asset: asset.clone(),
                evidence_type: observation.evidence_type().clone(),
                facts: observation.facts().clone(),
                narrative: observation.narrative().to_string(),
                observed_at: Some(collected_at),
                valid_from: None,
                valid_until: None,
                source_revision: None,
            });
        }
        Ok(ObservationBatch {
            candidates,
            diagnostics: Vec::new(),
            coverage: CollectionCoverage {
                hole: false,
                strict_scope: true,
            },
            collected_at: Some(collected_at),
        })
    }
}

impl EvidenceCollector for FixtureCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorAdapter::descriptor(self)
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        crate::application::engine::collect_envelopes(self, &self.instance(), scope)
    }
}
