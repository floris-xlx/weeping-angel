//! Append-only evidence-validity/v1 events. Never rewrite a sealed envelope.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::canonical_digest;

use crate::{EvidenceEnvelope, EvidenceError};

pub const EVIDENCE_VALIDITY_SCHEMA: &str = "evidence-validity/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceValidityKind {
    Asserted,
    Superseded,
    Revoked,
    Invalidated,
}

pub type ValidityEventKind = EvidenceValidityKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceValidityEvent {
    pub schema_version: String,
    pub event_id: String,
    pub envelope_digest: String,
    pub kind: EvidenceValidityKind,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidityEventBody<'a> {
    schema_version: &'a str,
    envelope_digest: &'a str,
    kind: EvidenceValidityKind,
    at: DateTime<Utc>,
    observed_at: Option<DateTime<Utc>>,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
    source_revision: Option<&'a str>,
    artifact_digest: Option<&'a str>,
    supersedes_event_id: Option<&'a str>,
    reason: Option<&'a str>,
}

impl EvidenceValidityEvent {
    pub fn new(
        envelope_digest: impl Into<String>,
        kind: EvidenceValidityKind,
        at: DateTime<Utc>,
    ) -> Result<Self, EvidenceError> {
        let envelope_digest = envelope_digest.into();
        let mut event = Self {
            schema_version: EVIDENCE_VALIDITY_SCHEMA.into(),
            event_id: String::new(),
            envelope_digest,
            kind,
            at,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            source_revision: None,
            artifact_digest: None,
            supersedes_event_id: None,
            reason: None,
        };
        event.event_id = event.digest_id()?;
        Ok(event)
    }

    pub fn asserted_for(envelope: &EvidenceEnvelope) -> Result<Self, EvidenceError> {
        Self::asserted_for_at(envelope, envelope.provenance().collected_at)
    }

    pub fn asserted_for_at(
        envelope: &EvidenceEnvelope,
        at: DateTime<Utc>,
    ) -> Result<Self, EvidenceError> {
        let mut event = Self::new(envelope.digest(), EvidenceValidityKind::Asserted, at)?;
        event.observed_at = Some(envelope.observed_at());
        event.valid_from = Some(envelope.valid_from());
        event.valid_until = envelope.valid_until();
        event.source_revision = envelope.source_revision().map(str::to_string);
        event.artifact_digest = envelope.artifact_digest().map(str::to_string);
        event.event_id = event.digest_id()?;
        Ok(event)
    }

    pub fn revoked(
        envelope_digest: impl Into<String>,
        at: DateTime<Utc>,
        reason: Option<String>,
    ) -> Result<Self, EvidenceError> {
        let mut event = Self::new(envelope_digest, EvidenceValidityKind::Revoked, at)?;
        event.reason = reason;
        event.event_id = event.digest_id()?;
        Ok(event)
    }

    pub fn superseded(
        previous_digest: impl Into<String>,
        at: DateTime<Utc>,
    ) -> Result<Self, EvidenceError> {
        Self::new(previous_digest, EvidenceValidityKind::Superseded, at)
    }

    pub fn invalidated(
        envelope_digest: impl Into<String>,
        at: DateTime<Utc>,
        reason: Option<String>,
    ) -> Result<Self, EvidenceError> {
        let mut event = Self::new(envelope_digest, EvidenceValidityKind::Invalidated, at)?;
        event.reason = reason;
        event.event_id = event.digest_id()?;
        Ok(event)
    }

    pub fn with_observed_at(mut self, observed_at: DateTime<Utc>) -> Result<Self, EvidenceError> {
        self.observed_at = Some(observed_at);
        self.event_id = self.digest_id()?;
        Ok(self)
    }

    pub fn with_window(
        mut self,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Result<Self, EvidenceError> {
        self.valid_from = Some(valid_from);
        self.valid_until = valid_until;
        self.event_id = self.digest_id()?;
        Ok(self)
    }

    pub fn with_source_revision(
        mut self,
        revision: impl Into<String>,
    ) -> Result<Self, EvidenceError> {
        self.source_revision = Some(revision.into());
        self.event_id = self.digest_id()?;
        Ok(self)
    }

    pub fn with_artifact_digest(
        mut self,
        digest: impl Into<String>,
    ) -> Result<Self, EvidenceError> {
        self.artifact_digest = Some(digest.into());
        self.event_id = self.digest_id()?;
        Ok(self)
    }

    fn digest_id(&self) -> Result<String, EvidenceError> {
        let body = ValidityEventBody {
            schema_version: &self.schema_version,
            envelope_digest: &self.envelope_digest,
            kind: self.kind,
            at: self.at,
            observed_at: self.observed_at,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            source_revision: self.source_revision.as_deref(),
            artifact_digest: self.artifact_digest.as_deref(),
            supersedes_event_id: self.supersedes_event_id.as_deref(),
            reason: self.reason.as_deref(),
        };
        canonical_digest(&body).map_err(|e| EvidenceError::Digest(e.to_string()))
    }
}

/// Projected window when the envelope is a candidate at `t`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidityView {
    pub observed_at: DateTime<Utc>,
    pub collected_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

pub type ValidityProjection = ValidityView;

pub fn window_contains(
    valid_from: DateTime<Utc>,
    valid_until: Option<DateTime<Utc>>,
    t: DateTime<Utc>,
) -> bool {
    valid_from <= t && valid_until.is_none_or(|until| t < until)
}

pub fn project_validity(
    envelope: &EvidenceEnvelope,
    events: &[EvidenceValidityEvent],
    t: DateTime<Utc>,
) -> Option<ValidityView> {
    let mut observed_at = envelope.observed_at();
    let mut valid_from = envelope.valid_from();
    let mut valid_until = envelope.valid_until();
    let mut withdrawn = false;
    let mut relevant: Vec<&EvidenceValidityEvent> = events
        .iter()
        .filter(|e| e.envelope_digest == envelope.digest() && e.at <= t)
        .collect();
    relevant.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.event_id.cmp(&b.event_id)));
    for event in relevant {
        match event.kind {
            EvidenceValidityKind::Revoked | EvidenceValidityKind::Invalidated => {
                withdrawn = true;
            }
            EvidenceValidityKind::Superseded => {}
            EvidenceValidityKind::Asserted => {
                withdrawn = false;
                if let Some(obs) = event.observed_at {
                    observed_at = obs;
                }
                if let Some(from) = event.valid_from {
                    valid_from = from;
                }
                valid_until = event.valid_until;
            }
        }
    }
    let collected_at = envelope.provenance().collected_at;
    if withdrawn
        || collected_at > t
        || observed_at > t
        || !window_contains(valid_from, valid_until, t)
    {
        return None;
    }
    Some(ValidityView {
        observed_at,
        collected_at,
        valid_from,
        valid_until,
    })
}

pub fn is_candidate_at(
    envelope: &EvidenceEnvelope,
    events: &[EvidenceValidityEvent],
    t: DateTime<Utc>,
) -> bool {
    project_validity(envelope, events, t).is_some()
}

/// Shared validity-leaf algorithm (DUP-007).
///
/// Public clocks stay distinct: control-test `select_latest_as_of` and ledger
/// `as_of`/`current`/`latest` call this helper. Do not name this
/// `select_latest_as_of` (ACP-T07 / TLE-015).
pub fn select_valid_leaf_as_of<'a>(
    candidates: impl IntoIterator<Item = &'a EvidenceEnvelope>,
    as_of: DateTime<Utc>,
    events: &[EvidenceValidityEvent],
) -> Option<&'a EvidenceEnvelope> {
    let usable: Vec<&'a EvidenceEnvelope> = candidates
        .into_iter()
        .filter(|env| project_validity(env, events, as_of).is_some())
        .collect();
    let superseded: BTreeSet<&str> = usable
        .iter()
        .filter_map(|e| e.supersedes())
        .filter(|prev| usable.iter().any(|e| e.digest() == *prev))
        .collect();
    let mut leaves: Vec<&'a EvidenceEnvelope> = usable
        .into_iter()
        .filter(|e| !superseded.contains(e.digest()))
        .collect();
    if leaves.is_empty() {
        return None;
    }
    leaves.sort_by(|a, b| {
        let va = project_validity(a, events, as_of).expect("candidate");
        let vb = project_validity(b, events, as_of).expect("candidate");
        va.observed_at
            .cmp(&vb.observed_at)
            .then_with(|| va.collected_at.cmp(&vb.collected_at))
            .then_with(|| a.digest().cmp(b.digest()))
    });
    leaves.pop()
}
