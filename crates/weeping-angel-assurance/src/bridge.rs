//! One-way scanner → observation adapter. Does not rewrite `to_semantic_finding`.

use weeping_angel::contract::SemanticFinding;
use weeping_angel::engines::EngineHit;
use weeping_angel_evidence::{EvidenceObservation, EvidenceType};

pub fn from_engine_hit(hit: &EngineHit) -> EvidenceObservation {
    EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", &hit.rule_id)
        .with_fact("path", &hit.path)
        .with_fact("category", &hit.category)
        .with_narrative(&hit.title)
}

pub fn from_semantic_finding(finding: &SemanticFinding) -> EvidenceObservation {
    EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", &finding.rule_id)
        .with_fact("finding_id", &finding.finding_id)
        .with_narrative(&finding.title)
}
