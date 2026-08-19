//! Immutable ISMS event observations (`isms-event` / `weeping-angel/isms-event/v1`).
//!
//! Events are state-transition observations, not workflow tickets.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::id::EventId;
use crate::typed_canonical_digest;

/// Frozen schema advertised on every persisted event document.
pub const ISMS_EVENT_SCHEMA: &str = "weeping-angel/isms-event/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventSeverity {
    Informational,
    Notable,
    Material,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventSubjectKind {
    Control,
    Asset,
    Risk,
    Exception,
    Evidence,
    Vendor,
    Implementation,
    Test,
    Requirement,
    Objective,
    Policy,
    Finding,
    Nonconformity,
    Other,
}

impl EventSubjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Asset => "asset",
            Self::Risk => "risk",
            Self::Exception => "exception",
            Self::Evidence => "evidence",
            Self::Vendor => "vendor",
            Self::Implementation => "implementation",
            Self::Test => "test",
            Self::Requirement => "requirement",
            Self::Objective => "objective",
            Self::Policy => "policy",
            Self::Finding => "finding",
            Self::Nonconformity => "nonconformity",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventCauseKind {
    Event,
    Control,
    Risk,
    Evidence,
    Exception,
    Snapshot,
    Other,
}

impl EventCauseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Control => "control",
            Self::Risk => "risk",
            Self::Evidence => "evidence",
            Self::Exception => "exception",
            Self::Snapshot => "snapshot",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSubjectRef {
    pub kind: EventSubjectKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventCauseRef {
    pub kind: EventCauseKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsmsEventKind {
    ControlRegressed,
    ControlRecovered,
    EvidenceExpired,
    EvidenceRevoked,
    RiskIncreased,
    RiskDecreased,
    RiskAccepted,
    ExceptionExpired,
    NewAssetDetected,
    AssetRemoved,
    VendorRiskChanged,
    ObjectiveMissed,
    PolicyExpired,
    AuditFindingOpened,
    NonconformityOpened,
    CorrectiveActionOverdue,
    Extensible { name: String },
}

impl IsmsEventKind {
    pub fn as_label(&self) -> String {
        match self {
            Self::ControlRegressed => "ControlRegressed".into(),
            Self::ControlRecovered => "ControlRecovered".into(),
            Self::EvidenceExpired => "EvidenceExpired".into(),
            Self::EvidenceRevoked => "EvidenceRevoked".into(),
            Self::RiskIncreased => "RiskIncreased".into(),
            Self::RiskDecreased => "RiskDecreased".into(),
            Self::RiskAccepted => "RiskAccepted".into(),
            Self::ExceptionExpired => "ExceptionExpired".into(),
            Self::NewAssetDetected => "NewAssetDetected".into(),
            Self::AssetRemoved => "AssetRemoved".into(),
            Self::VendorRiskChanged => "VendorRiskChanged".into(),
            Self::ObjectiveMissed => "ObjectiveMissed".into(),
            Self::PolicyExpired => "PolicyExpired".into(),
            Self::AuditFindingOpened => "AuditFindingOpened".into(),
            Self::NonconformityOpened => "NonconformityOpened".into(),
            Self::CorrectiveActionOverdue => "CorrectiveActionOverdue".into(),
            Self::Extensible { name } => name.clone(),
        }
    }
}

/// Immutable observation of a management-system state transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsmsEvent {
    pub schema_version: String,
    pub event_id: EventId,
    pub kind: IsmsEventKind,
    pub occurred_at: String,
    #[serde(default)]
    pub source_snapshots: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub previous_snapshot_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub next_snapshot_digest: String,
    #[serde(default)]
    pub subjects: Vec<EventSubjectRef>,
    #[serde(default, alias = "causes")]
    pub cause_refs: Vec<EventCauseRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<EventSeverity>,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventIdentityBody<'a> {
    schema_version: &'a str,
    kind: &'a IsmsEventKind,
    occurred_at: &'a str,
    source_snapshots: &'a [String],
    previous_snapshot_digest: &'a str,
    next_snapshot_digest: &'a str,
    subjects: &'a [EventSubjectRef],
    cause_refs: &'a [EventCauseRef],
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<EventSeverity>,
    payload: &'a Value,
}

impl IsmsEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: IsmsEventKind,
        occurred_at: DateTime<Utc>,
        previous_snapshot: impl Into<String>,
        next_snapshot: impl Into<String>,
        mut subjects: Vec<EventSubjectRef>,
        mut cause_refs: Vec<EventCauseRef>,
        severity: Option<EventSeverity>,
        payload: Value,
    ) -> Self {
        subjects.sort_by(|a, b| {
            (a.kind.as_str(), a.id.as_str()).cmp(&(b.kind.as_str(), b.id.as_str()))
        });
        cause_refs.sort_by(|a, b| {
            (a.kind.as_str(), a.id.as_str()).cmp(&(b.kind.as_str(), b.id.as_str()))
        });
        let previous_snapshot = previous_snapshot.into();
        let next_snapshot = next_snapshot.into();
        let occurred_at = rfc3339_z(occurred_at);
        let event = Self {
            schema_version: ISMS_EVENT_SCHEMA.into(),
            event_id: EventId::new("event:pending"),
            kind,
            occurred_at,
            source_snapshots: vec![previous_snapshot.clone(), next_snapshot.clone()],
            previous_snapshot_digest: previous_snapshot,
            next_snapshot_digest: next_snapshot,
            subjects,
            cause_refs,
            severity,
            payload,
        };
        event.sealed()
    }

    pub fn with_causes(mut self, cause_refs: Vec<EventCauseRef>) -> Self {
        self.cause_refs = cause_refs;
        self.cause_refs.sort_by(|a, b| {
            (a.kind.as_str(), a.id.as_str()).cmp(&(b.kind.as_str(), b.id.as_str()))
        });
        self.sealed()
    }

    fn sealed(mut self) -> Self {
        let hex = typed_canonical_digest("isms-event", &self.identity_body())
            .expect("isms-event identity must serialize");
        self.event_id = EventId::new(format!("event:sha256:{hex}"));
        self
    }

    fn identity_body(&self) -> EventIdentityBody<'_> {
        EventIdentityBody {
            schema_version: &self.schema_version,
            kind: &self.kind,
            occurred_at: &self.occurred_at,
            source_snapshots: &self.source_snapshots,
            previous_snapshot_digest: &self.previous_snapshot_digest,
            next_snapshot_digest: &self.next_snapshot_digest,
            subjects: &self.subjects,
            cause_refs: &self.cause_refs,
            severity: self.severity,
            payload: &self.payload,
        }
    }
}

pub fn rfc3339_z(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(SecondsFormat::Secs, true)
}
