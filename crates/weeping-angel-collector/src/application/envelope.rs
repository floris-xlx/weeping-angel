use chrono::Utc;
use weeping_angel_evidence::{
    CollectionRun, EvidenceEnvelope, EvidenceObservation, EvidenceProvenance,
};

use crate::CollectorError;
use crate::domain::{
    CollectionBatch, CollectorInstance, CollectorScope, ObservationBatch, ObservationCandidate,
};
use crate::ports::CollectorAdapter;

/// Only site in this crate that constructs `EvidenceProvenance` and seals envelopes.
pub struct EnvelopeFactory;

impl EnvelopeFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn seal_batch(
        &self,
        instance: &CollectorInstance,
        scope: &CollectorScope,
        adapter: &dyn CollectorAdapter,
        batch: &ObservationBatch,
    ) -> Result<CollectionBatch, CollectorError> {
        let version = adapter.descriptor().version;
        let mut run = CollectionRun::new(&instance.collector_id, version);
        run.scope = scope.as_label();
        run.configuration_digest = adapter.configuration_digest(scope);
        let mut envelopes = Vec::with_capacity(batch.candidates.len());
        for candidate in &batch.candidates {
            envelopes.push(self.seal_candidate(instance, scope, batch, candidate)?);
        }
        run.completed_at = Some(Utc::now());
        run.evidence_count = envelopes.len() as u32;
        run.error_count = batch.diagnostics.len() as u32;
        run.status = if envelopes.is_empty() && !batch.diagnostics.is_empty() {
            "failed".into()
        } else if batch.coverage.hole || !batch.diagnostics.is_empty() {
            "partial".into()
        } else {
            "complete".into()
        };
        Ok(CollectionBatch {
            run,
            envelopes,
            errors: batch.diagnostics.clone(),
        })
    }

    pub fn seal_candidate(
        &self,
        instance: &CollectorInstance,
        scope: &CollectorScope,
        batch: &ObservationBatch,
        candidate: &ObservationCandidate,
    ) -> Result<EvidenceEnvelope, CollectorError> {
        let collected_at = candidate
            .observed_at
            .or(batch.collected_at)
            .unwrap_or_else(Utc::now);
        let mut observation = EvidenceObservation::new(candidate.evidence_type.clone())
            .with_narrative(&candidate.narrative);
        for (key, value) in &candidate.facts {
            observation = observation.with_value(key.clone(), value.clone());
        }
        let provenance = EvidenceProvenance {
            collector_id: instance.collector_id.clone(),
            collected_at,
            scope: scope.as_label(),
            asset: candidate.asset.clone(),
        };
        Ok(EvidenceEnvelope::seal(observation, provenance)?)
    }
}

impl Default for EnvelopeFactory {
    fn default() -> Self {
        Self::new()
    }
}
