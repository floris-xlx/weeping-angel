use weeping_angel_evidence::looks_like_compliance_claim;

use crate::CollectorError;
use crate::domain::{CollectorDescriptor, CollectorScope, ObservationBatch, ObservationCandidate};

const CREDENTIAL_KEYS: &[&str] = &[
    "authorization",
    "token",
    "cookie",
    "password",
    "api_key",
    "apikey",
    "secret",
    "access_token",
    "refresh_token",
    "private_key",
];

/// Defense-in-depth validation of adapter output before `EnvelopeFactory` seal.
pub struct ObservationGate;

impl ObservationGate {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        descriptor: &CollectorDescriptor,
        scope: &CollectorScope,
        batch: &ObservationBatch,
    ) -> Result<(), CollectorError> {
        for candidate in &batch.candidates {
            self.validate_candidate(descriptor, scope, batch, candidate)?;
        }
        Ok(())
    }

    fn validate_candidate(
        &self,
        descriptor: &CollectorDescriptor,
        scope: &CollectorScope,
        batch: &ObservationBatch,
        candidate: &ObservationCandidate,
    ) -> Result<(), CollectorError> {
        let ty = candidate.evidence_type.as_str();
        if ty == "control_test_result" || ty.contains("effectiveness") {
            return Err(CollectorError::FrameworkResult {
                detail: candidate.narrative.clone(),
            });
        }
        if !descriptor.evidence_types.contains(&candidate.evidence_type) {
            return Err(CollectorError::UndeclaredEvidenceType {
                evidence_type: ty.to_string(),
            });
        }
        if looks_like_compliance_claim(&candidate.narrative) {
            return Err(CollectorError::ComplianceClaim {
                narrative: candidate.narrative.clone(),
            });
        }
        if looks_like_compliance_claim(ty) {
            return Err(CollectorError::FrameworkResult {
                detail: ty.to_string(),
            });
        }
        for key in candidate.facts.keys() {
            let folded = key.to_ascii_lowercase().replace('-', "_");
            if CREDENTIAL_KEYS.contains(&folded.as_str()) {
                return Err(CollectorError::InsufficientEvidence {
                    detail: format!("credential-shaped fact `{key}` is not allowed"),
                });
            }
        }
        if batch.coverage.strict_scope && !scope.allows(&candidate.asset) {
            return Err(CollectorError::OutOfScope {
                asset: candidate.asset.to_string(),
            });
        }
        Ok(())
    }
}

impl Default for ObservationGate {
    fn default() -> Self {
        Self::new()
    }
}
