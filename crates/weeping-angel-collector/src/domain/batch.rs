use chrono::{DateTime, Utc};
use weeping_angel_evidence::{CollectionRun, EvidenceEnvelope};

use super::{CollectionCoverage, CollectorScope, ObservationCandidate};

#[derive(Debug, Clone)]
pub struct CollectionRequest {
    pub scope: CollectorScope,
}

#[derive(Debug, Clone)]
pub struct CollectionBatch {
    pub run: CollectionRun,
    pub envelopes: Vec<EvidenceEnvelope>,
    pub errors: Vec<String>,
}

/// Adapter output batch. Not sealed envelopes.
#[derive(Debug, Clone, Default)]
pub struct ObservationBatch {
    pub candidates: Vec<ObservationCandidate>,
    pub diagnostics: Vec<String>,
    pub coverage: CollectionCoverage,
    pub collected_at: Option<DateTime<Utc>>,
}
