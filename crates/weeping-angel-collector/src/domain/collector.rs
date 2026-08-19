use thiserror::Error;
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceError};

use super::{CollectorDescriptor, CollectorScope};

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
    #[error("permission denied: {detail}")]
    PermissionDenied { detail: String },
    #[error("insufficient evidence: {detail}")]
    InsufficientEvidence { detail: String },
    #[error("evidence seal failed: {0}")]
    Seal(#[from] EvidenceError),
}

pub trait EvidenceCollector {
    fn descriptor(&self) -> CollectorDescriptor;
    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>;
}
