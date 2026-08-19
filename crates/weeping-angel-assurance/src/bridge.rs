//! One-way scanner → observation adapter. Does not rewrite `to_semantic_finding`.
//!
//! Views are traits so this crate does not depend on the root scanner package
//! (that edge is a Cargo cycle with root `[dev-dependencies]`).

use weeping_angel_evidence::{EvidenceObservation, EvidenceType};

/// Field view over a scanner engine hit. Implemented by the CLI crate.
pub trait EngineHitView {
    fn rule_id(&self) -> &str;
    fn path(&self) -> &str;
    fn category(&self) -> &str;
    fn title(&self) -> &str;
}

/// Field view over a semantic finding. Implemented by the CLI crate.
pub trait SemanticFindingView {
    fn rule_id(&self) -> &str;
    fn finding_id(&self) -> &str;
    fn title(&self) -> &str;
}

/// Canonical scanner evidence taxonomy. Absence of findings cannot
/// prove effectiveness and is never emitted as a positive "no vulns" fact.
const SCANNER_EVIDENCE_TYPES: &[&str] = &[
    "security.finding",
    "security.vulnerability.present",
    "security.exposure.present",
    "security.authz.weakness",
    "security.secret.exposure",
    "security.tls.misconfiguration",
    "security.header.misconfiguration",
    "security.dependency_confusion_risk",
];

pub fn from_engine_hit(hit: &impl EngineHitView) -> EvidenceObservation {
    let ty = classify(hit.category(), hit.rule_id());
    EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", hit.rule_id())
        .with_fact("path", hit.path())
        .with_fact("category", hit.category())
        .with_fact("canonical_type", ty)
        .with_narrative(hit.title())
}

pub fn from_semantic_finding(finding: &impl SemanticFindingView) -> EvidenceObservation {
    EvidenceObservation::new(EvidenceType::new("security_finding"))
        .with_fact("rule_id", finding.rule_id())
        .with_fact("finding_id", finding.finding_id())
        .with_narrative(finding.title())
}

fn classify(category: &str, rule_id: &str) -> &'static str {
    let hay = format!("{category} {rule_id}").to_ascii_lowercase();
    if hay.contains("secret") {
        "security.secret.exposure"
    } else if hay.contains("tls") {
        "security.tls.misconfiguration"
    } else if hay.contains("header") {
        "security.header.misconfiguration"
    } else if hay.contains("authz") || hay.contains("authoriz") {
        "security.authz.weakness"
    } else if hay.contains("dependenc") {
        "security.dependency_confusion_risk"
    } else if hay.contains("vuln") || hay.contains("cwe") || hay.contains("traversal") {
        "security.vulnerability.present"
    } else if hay.contains("expos") {
        "security.exposure.present"
    } else {
        let _ = SCANNER_EVIDENCE_TYPES;
        "security.finding"
    }
}
