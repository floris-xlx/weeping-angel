use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use weeping_angel_evidence::EvidenceType;

use super::CollectorCapabilities;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectorDescriptor {
    pub id: String,
    pub version: String,
    pub evidence_types: BTreeSet<EvidenceType>,
    pub provider_family: String,
    pub subject_types: BTreeSet<String>,
    pub capabilities: CollectorCapabilities,
    pub required_permissions: Vec<String>,
}
