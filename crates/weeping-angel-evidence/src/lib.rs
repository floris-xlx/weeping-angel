//! Immutable evidence envelopes. Observations are facts, never compliance claims.

pub mod ledger;
pub mod value;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{AssetId, canonical_digest};

pub use ledger::{EvidenceLedger, LedgerError};
pub use value::{
    DecimalText, EVIDENCE_VALUE_SCHEMA, EVIDENCE_VALUE_TAG, EvidenceValue, EvidenceValueError,
};
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

pub const EVIDENCE_SCHEMA: &str = "evidence/v1";

#[derive(Debug, Error)]
pub enum EvidenceError {
    #[error("observation is a compliance claim: {narrative}")]
    ComplianceClaim { narrative: String },
    #[error("credential material in evidence payload: {key}")]
    CredentialInPayload { key: String },
    #[error("reserved object key in evidence payload: {key}")]
    ReservedObjectKey { key: String },
    #[error("digest failed: {0}")]
    Digest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceObservation {
    evidence_type: EvidenceType,
    facts: BTreeMap<String, EvidenceValue>,
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
        self.facts
            .insert(key.into(), EvidenceValue::String(value.into()));
        self
    }

    pub fn with_value(mut self, key: impl Into<String>, value: EvidenceValue) -> Self {
        // Handoff: EvidenceValue::Bool(true), EvidenceValue::Integer(365),
        // EvidenceValue::StringList(vec!["owner".into(), "admin".into()]).
        self.facts.insert(key.into(), value);
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
        self.facts.get(key).and_then(EvidenceValue::as_str)
    }

    pub fn fact_value(&self, key: &str) -> Option<&EvidenceValue> {
        self.facts.get(key)
    }

    pub fn facts(&self) -> &BTreeMap<String, EvidenceValue> {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceArtifactRef {
    pub artifact_id: String,
    pub digest: String,
    pub media_type: String,
    pub size: u64,
    pub storage_locator: String, // storageLocator
    pub redaction_state: String, // redactionState
}

impl Default for EvidenceArtifactRef {
    fn default() -> Self {
        Self {
            artifact_id: String::new(),
            digest: String::new(),
            media_type: "application/octet-stream".into(),
            size: 0,
            storage_locator: String::new(),
            redaction_state: "none".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRun {
    pub run_id: String,
    pub collector_id: String,
    pub collector_version: String, // collectorVersion
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub scope: String,
    pub status: String,
    pub evidence_count: u32,
    pub error_count: u32,
    pub configuration_digest: String, // configurationDigest
}

impl CollectionRun {
    pub fn new(collector_id: impl Into<String>, collector_version: impl Into<String>) -> Self {
        let collector_id = collector_id.into();
        let started_at = Utc::now();
        let run_id = format!(
            "run:{}",
            &canonical_digest(&(collector_id.as_str(), started_at.to_rfc3339()))
                .unwrap_or_else(|_| "0".repeat(16))[..16]
        );
        Self {
            run_id,
            collector_id,
            collector_version: collector_version.into(),
            started_at,
            completed_at: None,
            scope: String::new(),
            status: "started".into(),
            evidence_count: 0,
            error_count: 0,
            configuration_digest: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEnvelope {
    evidence_id: String,
    schema_version: String,
    artifact_ref: Option<EvidenceArtifactRef>,
    collection_run_id: String,
    content_digest: String,
    sensitivity: String,
    scope: String,
    supersedes: Option<String>,
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
        let collection_run_id = deterministic_run_id(&provenance);
        let scope = provenance.scope.clone();
        let body = DigestBody {
            observation: &observation,
            provenance: &provenance,
        };
        let digest = canonical_digest(&body).map_err(|e| EvidenceError::Digest(e.to_string()))?;
        let content_digest = digest.clone();
        Ok(Self {
            evidence_id: format!("ev:sha256:{digest}"),
            schema_version: EVIDENCE_SCHEMA.into(),
            artifact_ref: None,
            collection_run_id,
            content_digest,
            sensitivity: "normal".into(),
            scope,
            supersedes: None,
            observation,
            provenance,
            digest,
        })
    }

    pub fn with_collection_run(mut self, run_id: impl Into<String>) -> Self {
        self.collection_run_id = run_id.into();
        self
    }

    pub fn with_artifact_ref(mut self, artifact_ref: EvidenceArtifactRef) -> Self {
        self.artifact_ref = Some(artifact_ref);
        self
    }

    pub fn with_supersedes(mut self, previous: impl Into<String>) -> Self {
        self.supersedes = Some(previous.into());
        self
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

    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    pub fn collection_run_id(&self) -> &str {
        &self.collection_run_id
    }

    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }

    pub fn supersedes(&self) -> Option<&str> {
        self.supersedes.as_deref()
    }
}

fn deterministic_run_id(provenance: &EvidenceProvenance) -> String {
    let body = (
        provenance.collector_id.as_str(),
        provenance.collected_at.to_rfc3339(),
        provenance.scope.as_str(),
    );
    let digest = canonical_digest(&body).unwrap_or_else(|_| "0".repeat(16));
    format!("run:{}", &digest[..16.min(digest.len())])
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
        || lower.contains("iso 27001 certified")
        || lower.contains("iso27001 certified")
        || lower.contains("certification guaranteed")
        || lower.contains("audit passed")
        || lower.contains("gdpr compliant")
        || lower.contains("soc 2 compliant")
        || lower.contains("soc2 compliant")
        || lower.contains("controltestresult")
        || lower.contains("control test result")
        || lower.contains("nis2 compliant")
        || lower.contains("dora compliant")
}

/// Redact credential-shaped tokens from diagnostics. Never persist tokens.
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for needle in ["Bearer ", "token=", "ghp_", "gho_", "github_pat_"] {
        if let Some(idx) = out.find(needle) {
            let rest = &out[idx + needle.len()..];
            let cut = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .unwrap_or(rest.len());
            out.replace_range(idx + needle.len()..idx + needle.len() + cut, "[redacted]");
        }
    }
    out
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
    for (key, value) in &obs.facts {
        reject_value_keys(key, value)?;
    }
    Ok(())
}

fn reject_value_keys(key: &str, value: &EvidenceValue) -> Result<(), EvidenceError> {
    if key == EVIDENCE_VALUE_TAG {
        return Err(EvidenceError::ReservedObjectKey {
            key: key.to_string(),
        });
    }
    let folded = key.to_ascii_lowercase().replace('-', "_");
    if CREDENTIAL_KEYS.contains(&folded.as_str()) {
        return Err(EvidenceError::CredentialInPayload {
            key: key.to_string(),
        });
    }
    if let EvidenceValue::Object(nested) = value {
        for (child_key, child) in nested {
            reject_value_keys(child_key, child)?;
        }
    }
    Ok(())
}
