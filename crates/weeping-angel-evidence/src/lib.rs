//! Immutable evidence envelopes. Observations are facts, never compliance claims.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{canonical_digest, AssetId};

pub use weeping_angel_assurance_ir::EvidenceType;

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

#[derive(Debug, Error)]
pub enum EvidenceError {
    #[error("observation is a compliance claim: {narrative}")]
    ComplianceClaim { narrative: String },
    #[error("credential material in evidence payload: {key}")]
    CredentialInPayload { key: String },
    #[error("digest failed: {0}")]
    Digest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceObservation {
    evidence_type: EvidenceType,
    facts: BTreeMap<String, String>,
    narrative: String,
}

impl EvidenceObservation {
    pub fn new(evidence_type: EvidenceType) -> Self {
        Self {
            evidence_type,
            facts: BTreeMap::new(),
            narrative: String::new(),
        }
    }

    pub fn with_fact(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.facts.insert(key.into(), value.into());
        self
    }

    pub fn with_narrative(mut self, narrative: impl Into<String>) -> Self {
        self.narrative = narrative.into();
        self
    }

    pub fn evidence_type(&self) -> &EvidenceType {
        &self.evidence_type
    }

    pub fn narrative(&self) -> &str {
        &self.narrative
    }

    pub fn fact(&self, key: &str) -> Option<&str> {
        self.facts.get(key).map(String::as_str)
    }

    pub fn facts(&self) -> &BTreeMap<String, String> {
        &self.facts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceProvenance {
    pub collector_id: String,
    pub collected_at: DateTime<Utc>,
    pub scope: String,
    pub asset: AssetId,
}

impl EvidenceProvenance {
    pub fn asset(&self) -> &AssetId {
        &self.asset
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEnvelope {
    observation: EvidenceObservation,
    provenance: EvidenceProvenance,
    digest: String,
}

impl EvidenceEnvelope {
    pub fn seal(
        observation: EvidenceObservation,
        provenance: EvidenceProvenance,
    ) -> Result<Self, EvidenceError> {
        reject_compliance_claim(observation.narrative())?;
        reject_credentials(&observation)?;
        let body = DigestBody {
            observation: &observation,
            provenance: &provenance,
        };
        let digest = canonical_digest(&body).map_err(|e| EvidenceError::Digest(e.to_string()))?;
        Ok(Self {
            observation,
            provenance,
            digest,
        })
    }

    pub fn observation(&self) -> &EvidenceObservation {
        &self.observation
    }

    pub fn provenance(&self) -> &EvidenceProvenance {
        &self.provenance
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestBody<'a> {
    observation: &'a EvidenceObservation,
    provenance: &'a EvidenceProvenance,
}

pub fn looks_like_compliance_claim(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("iso 27001 compliant")
        || lower.contains("iso27001 compliant")
        || lower.contains("gdpr compliant")
        || lower.contains("soc 2 compliant")
        || lower.contains("soc2 compliant")
        || lower.contains("controltestresult")
        || lower.contains("control test result")
        || lower.contains("nis2 compliant")
        || lower.contains("dora compliant")
}

fn reject_compliance_claim(narrative: &str) -> Result<(), EvidenceError> {
    if looks_like_compliance_claim(narrative) {
        return Err(EvidenceError::ComplianceClaim {
            narrative: narrative.to_string(),
        });
    }
    Ok(())
}

fn reject_credentials(obs: &EvidenceObservation) -> Result<(), EvidenceError> {
    for key in obs.facts.keys() {
        let folded = key.to_ascii_lowercase().replace('-', "_");
        if CREDENTIAL_KEYS.contains(&folded.as_str()) {
            return Err(EvidenceError::CredentialInPayload { key: key.clone() });
        }
    }
    Ok(())
}
