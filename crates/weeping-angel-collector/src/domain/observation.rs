use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{EvidenceType, EvidenceValue};

/// Adapter output: a canonical observation candidate. Not an envelope.
#[derive(Debug, Clone)]
pub struct ObservationCandidate {
    pub asset: AssetId,
    pub evidence_type: EvidenceType,
    pub facts: BTreeMap<String, EvidenceValue>,
    pub narrative: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub source_revision: Option<String>,
}

impl ObservationCandidate {
    pub fn fact(&self, key: &str) -> Option<&str> {
        self.facts.get(key).and_then(EvidenceValue::as_str)
    }
}
