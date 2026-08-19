//! Canonical ISMS remediation work record.
//!
//! Lifecycle lives on this type; verification evaluation lives in
//! `weeping-angel-assurance`. External tickets are adapter references only.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::event::{IsmsEvent, IsmsEventKind};
use crate::validation::IrValidationError;
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, ControlId, ControlImplementationId,
    EvidenceRequirementId, EvidenceType, ExceptionId, ExceptionStatus, PrincipalRef,
    RemediationActionId, RemediationId, RiskAcceptanceId, RiskId, SlaPolicyId, SubjectSelector,
    TestFailureSeverity, TreatmentActionId, canonical_digest, validate_stable_id,
};

fn default_schema_version() -> String {
    ASSURANCE_IR_SCHEMA.into()
}

fn schema_is_default(value: &str) -> bool {
    value == ASSURANCE_IR_SCHEMA
}

fn default_version() -> u32 {
    1
}

fn version_is_one(value: &u32) -> bool {
    *value == 1
}

fn default_event_at() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH
}

fn default_min_cardinality() -> u32 {
    1
}

fn default_min_effective_results() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RemediationError {
    #[error("invalid remediation transition {from:?} → {to:?}")]
    InvalidTransition {
        from: RemediationState,
        to: RemediationState,
    },
    #[error("immutable closure: closed remediation {0} cannot be mutated")]
    ImmutableClosure(RemediationId),
    #[error("source must be ControlRegressed")]
    NotControlRegression,
    #[error("duplicate external ticket {system:?} {key}")]
    DuplicateTicket { system: TicketSystem, key: String },
    #[error("verification is not satisfied")]
    VerificationNotSatisfied,
    #[error("verification failed; Verified/Closed are forbidden")]
    VerificationFailed,
    #[error("waiver is not in force")]
    WaiverNotInForce,
    #[error("required evidence of fix is missing")]
    MissingEvidenceOfFix,
    #[error("{0}")]
    Message(String),
}

impl From<RemediationError> for IrValidationError {
    fn from(err: RemediationError) -> Self {
        IrValidationError::Message(err.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RemediationState {
    #[default]
    Proposed,
    Open,
    InProgress,
    AwaitingVerification,
    Verified,
    Closed,
    #[serde(alias = "waived")]
    AcceptedWaived,
    Cancelled,
    Superseded,
}

impl RemediationState {
    pub fn can_transition(from: Self, to: Self) -> bool {
        use RemediationState::*;
        matches!(
            (from, to),
            (Proposed, Open | Cancelled)
                | (Open, InProgress | AcceptedWaived | Cancelled | Superseded)
                | (
                    InProgress,
                    AwaitingVerification | Open | AcceptedWaived | Cancelled | Superseded
                )
                | (
                    AwaitingVerification,
                    Verified | InProgress | AcceptedWaived | Cancelled | Superseded
                )
                | (Verified, Closed | InProgress | Superseded)
                | (AcceptedWaived, Open | Cancelled | Superseded)
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::Cancelled | Self::Superseded)
    }

    pub fn stops_sla_clock(self) -> bool {
        matches!(self, Self::Closed | Self::Cancelled | Self::Superseded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RemediationPriority {
    P1,
    P2,
    #[default]
    P3,
    P4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RemediationSourceKind {
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
    RiskTreatmentAction,
    #[default]
    Manual,
}

impl From<&IsmsEventKind> for RemediationSourceKind {
    fn from(kind: &IsmsEventKind) -> Self {
        match kind {
            IsmsEventKind::ControlRegressed => Self::ControlRegressed,
            IsmsEventKind::ControlRecovered => Self::ControlRecovered,
            IsmsEventKind::EvidenceExpired => Self::EvidenceExpired,
            IsmsEventKind::EvidenceRevoked => Self::EvidenceRevoked,
            IsmsEventKind::RiskIncreased => Self::RiskIncreased,
            IsmsEventKind::RiskDecreased => Self::RiskDecreased,
            IsmsEventKind::RiskAccepted => Self::RiskAccepted,
            IsmsEventKind::ExceptionExpired => Self::ExceptionExpired,
            IsmsEventKind::NewAssetDetected => Self::NewAssetDetected,
            IsmsEventKind::AssetRemoved => Self::AssetRemoved,
            IsmsEventKind::VendorRiskChanged => Self::VendorRiskChanged,
            IsmsEventKind::ObjectiveMissed => Self::ObjectiveMissed,
            IsmsEventKind::PolicyExpired => Self::PolicyExpired,
            IsmsEventKind::AuditFindingOpened => Self::AuditFindingOpened,
            IsmsEventKind::NonconformityOpened => Self::NonconformityOpened,
            IsmsEventKind::CorrectiveActionOverdue => Self::CorrectiveActionOverdue,
            IsmsEventKind::Extensible { name } if name == "RiskTreatmentAction" => {
                Self::RiskTreatmentAction
            }
            IsmsEventKind::Extensible { .. } => Self::Manual,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RemediationSource {
    pub kind: RemediationSourceKind,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub snapshot_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cause_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_selectors: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_hint: Option<TestFailureSeverity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_digest: Option<String>,
}

impl RemediationSource {
    pub fn validate(&self) -> Result<(), RemediationError> {
        validate_stable_id(&self.event_id)
            .map_err(|err| RemediationError::Message(format!("invalid eventId: {err}")))?;
        Ok(())
    }
}

impl From<&IsmsEvent> for RemediationSource {
    fn from(event: &IsmsEvent) -> Self {
        let occurred_at = DateTime::parse_from_rfc3339(&event.occurred_at)
            .ok()
            .map(|ts| ts.with_timezone(&Utc));
        let payload_digest = canonical_digest(&event.payload).ok();
        Self {
            kind: RemediationSourceKind::from(&event.kind),
            event_id: event.event_id.as_str().to_string(),
            occurred_at,
            snapshot_refs: event.source_snapshots.clone(),
            cause_refs: event.cause_refs.iter().map(|c| c.id.clone()).collect(),
            subject_selectors: Vec::new(),
            severity_hint: None,
            payload_digest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TicketSystem {
    Jira,
    Linear,
    GitHubIssues,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTicketRef {
    pub system: TicketSystem,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_state: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RemediationActionState {
    #[default]
    Planned,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemediationAction {
    pub id: RemediationActionId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub state: RemediationActionState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceOfFixRequirement {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_requirement_id: Option<EvidenceRequirementId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_type: Option<EvidenceType>,
    pub description: String,
    #[serde(default = "default_min_cardinality")]
    pub min_cardinality: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum VerificationMode {
    SingleGreenPermitted,
    #[default]
    SustainedWindow,
    IndependentReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationPolicy {
    pub mode: VerificationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
    #[serde(default = "default_min_effective_results")]
    pub min_effective_results: u32,
    #[serde(default)]
    pub independent_verifier: bool,
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            mode: VerificationMode::SustainedWindow,
            window: Some(14 * 24 * 3600),
            min_effective_results: 2,
            independent_verifier: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum VerificationStatus {
    #[default]
    NotStarted,
    Failed,
    InWindow,
    Satisfied,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerificationState {
    pub status: VerificationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_result_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_start: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satisfied_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WaiverKind {
    Exception,
    RiskAcceptance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaiverBinding {
    pub kind: WaiverKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception_id: Option<ExceptionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_acceptance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RemediationEventKind {
    Created,
    FieldsRevised,
    StateTransition {
        from: RemediationState,
        to: RemediationState,
    },
    VerificationRecorded {
        status: VerificationStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result_digest: Option<String>,
    },
    ExternalTicketAttached {
        system: TicketSystem,
        key: String,
    },
    WaiverBound {
        kind: WaiverKind,
    },
    Closed,
    Superseded {
        successor: RemediationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemediationEvent {
    pub version: u32,
    #[serde(default = "default_event_at")]
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: RemediationEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Remediation {
    #[serde(
        default = "default_schema_version",
        skip_serializing_if = "schema_is_default"
    )]
    pub schema_version: String,
    pub id: RemediationId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub source: RemediationSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_ids: Vec<RiskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implementation_ids: Vec<ControlImplementationId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_selectors: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_action_ids: Vec<TreatmentActionId>,
    pub owner: PrincipalRef,
    #[serde(default)]
    pub priority: RemediationPriority,
    #[serde(default)]
    pub severity: TestFailureSeverity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sla_policy_id: Option<SlaPolicyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub state: RemediationState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_tickets: Vec<ExternalTicketRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_actions: Vec<RemediationAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_of_fix: Vec<EvidenceOfFixRequirement>,
    #[serde(default)]
    pub verification_policy: VerificationPolicy,
    #[serde(default)]
    pub verification_state: VerificationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver: Option<WaiverBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_by: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RemediationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<RemediationId>,
    #[serde(default = "default_version", skip_serializing_if = "version_is_one")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<RemediationEvent>,
}

impl Remediation {
    pub fn propose(
        id: RemediationId,
        title: impl Into<String>,
        source: RemediationSource,
        owner: PrincipalRef,
    ) -> Result<Self, RemediationError> {
        source.validate()?;
        let title = title.into();
        if title.trim().is_empty() {
            return Err(RemediationError::Message("title is required".into()));
        }
        let rem = Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            title,
            description: None,
            source,
            risk_ids: Vec::new(),
            control_ids: Vec::new(),
            implementation_ids: Vec::new(),
            subject_selectors: Vec::new(),
            treatment_action_ids: Vec::new(),
            owner,
            priority: RemediationPriority::P3,
            severity: TestFailureSeverity::Medium,
            sla_policy_id: None,
            due_at: None,
            state: RemediationState::Proposed,
            external_tickets: Vec::new(),
            planned_actions: Vec::new(),
            evidence_of_fix: Vec::new(),
            verification_policy: VerificationPolicy::default(),
            verification_state: VerificationState::default(),
            waiver: None,
            closed_by: None,
            closed_at: None,
            closure_rationale: None,
            supersedes: None,
            superseded_by: None,
            version: 1,
            history: vec![RemediationEvent {
                version: 1,
                at: Utc::now(),
                principal: None,
                kind: RemediationEventKind::Created,
            }],
        };
        Ok(rem)
    }

    pub fn sla_overdue(&self, as_of: DateTime<Utc>) -> bool {
        match self.due_at {
            None => false,
            Some(due) if due >= as_of => false,
            Some(_) if self.state.stops_sla_clock() => false,
            Some(_) => true,
        }
    }

    pub fn transition(
        mut self,
        to: RemediationState,
        principal: Option<PrincipalRef>,
        at: DateTime<Utc>,
    ) -> Result<Self, RemediationError> {
        if self.state == RemediationState::Closed {
            return Err(RemediationError::ImmutableClosure(self.id.clone()));
        }
        if !RemediationState::can_transition(self.state, to) {
            return Err(RemediationError::InvalidTransition {
                from: self.state,
                to,
            });
        }
        self.guard_transition(to, at)?;
        let from = self.state;
        self.state = to;
        self.version = self.version.saturating_add(1);
        let kind = if to == RemediationState::Closed {
            RemediationEventKind::Closed
        } else if to == RemediationState::Superseded {
            RemediationEventKind::Superseded {
                successor: self
                    .superseded_by
                    .clone()
                    .unwrap_or_else(|| self.id.clone()),
            }
        } else {
            RemediationEventKind::StateTransition { from, to }
        };
        self.history.push(RemediationEvent {
            version: self.version,
            at,
            principal,
            kind,
        });
        Ok(self)
    }

    fn guard_transition(
        &self,
        to: RemediationState,
        at: DateTime<Utc>,
    ) -> Result<(), RemediationError> {
        match (self.state, to) {
            (RemediationState::Proposed, RemediationState::Open) => {
                if self.title.trim().is_empty() || self.source.event_id.trim().is_empty() {
                    return Err(RemediationError::Message(
                        "Proposed → Open requires title, source.eventId, and owner".into(),
                    ));
                }
                self.source.validate()?;
                Ok(())
            }
            (_, RemediationState::InProgress) if matches!(self.state, RemediationState::Open) => {
                Ok(())
            }
            (RemediationState::InProgress, RemediationState::AwaitingVerification) => {
                if !self.evidence_of_fix_satisfied()
                    && self
                        .verification_state
                        .note
                        .as_ref()
                        .is_none_or(|n| n.trim().is_empty())
                {
                    return Err(RemediationError::MissingEvidenceOfFix);
                }
                Ok(())
            }
            (RemediationState::AwaitingVerification, RemediationState::Verified) => {
                if self.verification_state.status == VerificationStatus::Failed {
                    return Err(RemediationError::VerificationFailed);
                }
                if self.verification_state.status != VerificationStatus::Satisfied {
                    return Err(RemediationError::VerificationNotSatisfied);
                }
                Ok(())
            }
            (RemediationState::Verified, RemediationState::Closed) => {
                if self.closed_by.is_none()
                    || self.closed_at.is_none()
                    || self
                        .closure_rationale
                        .as_ref()
                        .is_none_or(|r| r.trim().is_empty())
                {
                    return Err(RemediationError::Message(
                        "Verified → Closed requires closedBy, closedAt, and closureRationale"
                            .into(),
                    ));
                }
                Ok(())
            }
            (_, RemediationState::AcceptedWaived) => {
                if self.waiver.is_none() {
                    return Err(RemediationError::WaiverNotInForce);
                }
                let _ = at;
                Ok(())
            }
            (_, RemediationState::Superseded) => {
                let Some(successor) = &self.superseded_by else {
                    return Err(RemediationError::Message(
                        "Superseded requires supersededBy".into(),
                    ));
                };
                if successor.as_str() == self.id.as_str() {
                    return Err(RemediationError::Message(
                        "supersededBy must be a different RemediationId".into(),
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn evidence_of_fix_satisfied(&self) -> bool {
        if self.evidence_of_fix.is_empty() {
            return true;
        }
        let attached = self
            .planned_actions
            .iter()
            .flat_map(|action| action.evidence_refs.iter())
            .count() as u32;
        self.evidence_of_fix
            .iter()
            .all(|req| req.min_cardinality == 0 || attached >= req.min_cardinality)
    }

    pub fn revise(mut self, title: impl Into<String>) -> Result<Self, RemediationError> {
        if self.state == RemediationState::Closed {
            return Err(RemediationError::ImmutableClosure(self.id.clone()));
        }
        if self.state.is_terminal() {
            return Err(RemediationError::InvalidTransition {
                from: self.state,
                to: self.state,
            });
        }
        self.title = title.into();
        self.version = self.version.saturating_add(1);
        self.history.push(RemediationEvent {
            version: self.version,
            at: Utc::now(),
            principal: None,
            kind: RemediationEventKind::FieldsRevised,
        });
        Ok(self)
    }

    pub fn attach_ticket(
        mut self,
        ticket: ExternalTicketRef,
        principal: Option<PrincipalRef>,
        at: DateTime<Utc>,
    ) -> Result<Self, RemediationError> {
        if self.state == RemediationState::Closed {
            return Err(RemediationError::ImmutableClosure(self.id.clone()));
        }
        if ticket.key.trim().is_empty() {
            return Err(RemediationError::Message(
                "external ticket key is required".into(),
            ));
        }
        if self
            .external_tickets
            .iter()
            .any(|existing| existing.system == ticket.system && existing.key == ticket.key)
        {
            return Err(RemediationError::DuplicateTicket {
                system: ticket.system,
                key: ticket.key,
            });
        }
        self.version = self.version.saturating_add(1);
        self.history.push(RemediationEvent {
            version: self.version,
            at,
            principal,
            kind: RemediationEventKind::ExternalTicketAttached {
                system: ticket.system,
                key: ticket.key.clone(),
            },
        });
        self.external_tickets.push(ticket);
        Ok(self)
    }

    pub fn link_action(mut self, action_id: TreatmentActionId) -> Result<Self, RemediationError> {
        if self.state == RemediationState::Closed {
            return Err(RemediationError::ImmutableClosure(self.id.clone()));
        }
        if !self.treatment_action_ids.iter().any(|id| id == &action_id) {
            self.treatment_action_ids.push(action_id);
            self.version = self.version.saturating_add(1);
            self.history.push(RemediationEvent {
                version: self.version,
                at: Utc::now(),
                principal: None,
                kind: RemediationEventKind::FieldsRevised,
            });
        }
        Ok(self)
    }

    pub fn record_verification(
        mut self,
        state: VerificationState,
        principal: Option<PrincipalRef>,
        at: DateTime<Utc>,
    ) -> Result<Self, RemediationError> {
        if self.state == RemediationState::Closed {
            return Err(RemediationError::ImmutableClosure(self.id.clone()));
        }
        self.verification_state = state.clone();
        self.version = self.version.saturating_add(1);
        self.history.push(RemediationEvent {
            version: self.version,
            at,
            principal,
            kind: RemediationEventKind::VerificationRecorded {
                status: state.status,
                result_digest: state.last_result_digest,
            },
        });
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), RemediationError> {
        if self.title.trim().is_empty() {
            return Err(RemediationError::Message("title is required".into()));
        }
        self.source.validate()?;
        if self.sla_policy_id.is_some() && self.due_at.is_none() {
            return Err(RemediationError::Message(
                "dueAt is required when slaPolicyId is set".into(),
            ));
        }
        if let Some(schema) = Some(self.schema_version.as_str())
            && schema != ASSURANCE_IR_SCHEMA
        {
            return Err(RemediationError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {schema}"
            )));
        }
        let mut tickets = BTreeSet::new();
        for ticket in &self.external_tickets {
            if ticket.key.trim().is_empty() {
                return Err(RemediationError::Message(
                    "external ticket key is required".into(),
                ));
            }
            if !tickets.insert((format!("{:?}", ticket.system), ticket.key.clone())) {
                return Err(RemediationError::DuplicateTicket {
                    system: ticket.system,
                    key: ticket.key.clone(),
                });
            }
        }
        let mut action_ids = BTreeSet::new();
        for action in &self.planned_actions {
            if !action_ids.insert(action.id.as_str().to_string()) {
                return Err(RemediationError::Message(format!(
                    "duplicate remediation action id {}",
                    action.id
                )));
            }
            if action.title.trim().is_empty() {
                return Err(RemediationError::Message(
                    "planned action title is required".into(),
                ));
            }
        }
        if self.state == RemediationState::Closed {
            if self.closed_by.is_none() || self.closed_at.is_none() {
                return Err(RemediationError::Message(
                    "closed remediations require closedBy and closedAt".into(),
                ));
            }
            let last_closed = self.history.last().is_some_and(|ev| {
                matches!(
                    ev.kind,
                    RemediationEventKind::Closed
                        | RemediationEventKind::StateTransition {
                            to: RemediationState::Closed,
                            ..
                        }
                )
            });
            if !last_closed
                && !self
                    .history
                    .iter()
                    .any(|ev| matches!(ev.kind, RemediationEventKind::Closed))
            {
                return Err(RemediationError::Message(
                    "closed remediations require a Closed history event".into(),
                ));
            }
        }
        let mut previous_state = None;
        for event in &self.history {
            if let RemediationEventKind::StateTransition { from, to } = event.kind {
                if !RemediationState::can_transition(from, to) {
                    return Err(RemediationError::InvalidTransition { from, to });
                }
                if let Some(prev) = previous_state
                    && prev != from
                {
                    return Err(RemediationError::Message(
                        "history state transition does not follow prior state".into(),
                    ));
                }
                previous_state = Some(to);
            }
        }
        Ok(())
    }
}

pub fn treatment_action_inventory(assessment: &AssessmentDefinition) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for decision in &assessment.risk_treatments {
        if let Some(plan) = &decision.plan {
            for action in &plan.actions {
                ids.insert(action.id.as_str().to_string());
            }
        }
    }
    ids
}

pub fn waiver_in_force(
    assessment: &AssessmentDefinition,
    remediation: &Remediation,
    as_of: DateTime<Utc>,
) -> bool {
    let Some(waiver) = &remediation.waiver else {
        return false;
    };
    match waiver.kind {
        WaiverKind::Exception => {
            let Some(exception_id) = &waiver.exception_id else {
                return false;
            };
            assessment.exceptions.iter().any(|exception| {
                exception.id.as_str() == exception_id.as_str()
                    && exception.status == ExceptionStatus::Approved
                    && exception.expires_at.is_some_and(|expires| as_of < expires)
            })
        }
        WaiverKind::RiskAcceptance => {
            let Some(acceptance_id) = &waiver.risk_acceptance_id else {
                return false;
            };
            if validate_stable_id(acceptance_id).is_err() {
                return false;
            }
            let Some(expires) = waiver.expires_at else {
                return false;
            };
            if as_of >= expires {
                return false;
            }
            if assessment.risk_treatments.is_empty() {
                return true;
            }
            assessment.risk_treatments.iter().any(|decision| {
                decision.acceptance.as_ref().is_some_and(|acceptance| {
                    acceptance.id.as_str() == acceptance_id
                        && crate::risk_treatment::acceptance_in_force(
                            assessment,
                            &decision.risk_id,
                            as_of,
                        )
                })
            }) || RiskAcceptanceId::try_new(acceptance_id).is_ok()
                && waiver.expires_at.is_some_and(|expires| as_of < expires)
                && assessment.risk_treatments.is_empty()
        }
    }
}

pub fn validate_remediations_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
    injected_treatment_actions: Option<&BTreeSet<TreatmentActionId>>,
) -> Result<(), RemediationError> {
    validate_remediation_inventory(assessment, injected_treatment_actions)?;
    validate_remediation_waivers_at(assessment, as_of)?;
    Ok(())
}

pub fn validate_remediation_inventory(
    assessment: &AssessmentDefinition,
    injected_treatment_actions: Option<&BTreeSet<TreatmentActionId>>,
) -> Result<(), RemediationError> {
    let mut seen = BTreeSet::new();
    let rem_ids: BTreeSet<_> = assessment
        .remediations
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    let risk_ids: BTreeSet<_> = assessment
        .risks
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    let control_ids: BTreeSet<_> = assessment
        .controls
        .iter()
        .map(|c| c.id().as_str().to_string())
        .collect();
    let implementation_ids: BTreeSet<_> = assessment
        .implementations
        .iter()
        .map(|i| i.id().as_str().to_string())
        .collect();
    let exception_ids: BTreeSet<_> = assessment
        .exceptions
        .iter()
        .map(|e| e.id.as_str().to_string())
        .collect();
    let identity_ids: BTreeSet<_> = assessment
        .identities
        .iter()
        .map(|i| i.id.as_str().to_string())
        .collect();
    let evidence_ids: BTreeSet<_> = assessment
        .evidence_requirements
        .iter()
        .map(|e| e.id().as_str().to_string())
        .collect();
    let known_actions = treatment_action_inventory(assessment);

    for rem in &assessment.remediations {
        if !seen.insert(rem.id.as_str().to_string()) {
            return Err(RemediationError::Message(format!(
                "duplicate remediation id {}",
                rem.id
            )));
        }
        rem.validate()?;
        for risk in &rem.risk_ids {
            if !risk_ids.contains(risk.as_str()) {
                return Err(RemediationError::Message(format!(
                    "dangling risk reference {risk} on remediation {}",
                    rem.id
                )));
            }
        }
        for control in &rem.control_ids {
            if !control_ids.contains(control.as_str()) {
                return Err(RemediationError::Message(format!(
                    "dangling control reference {control} on remediation {}",
                    rem.id
                )));
            }
        }
        for impl_id in &rem.implementation_ids {
            if !implementation_ids.contains(impl_id.as_str()) {
                return Err(RemediationError::Message(format!(
                    "dangling implementation {impl_id} on remediation {}",
                    rem.id
                )));
            }
        }
        for action in &rem.treatment_action_ids {
            let known = match injected_treatment_actions {
                Some(set) => set.iter().any(|id| id.as_str() == action.as_str()),
                None => known_actions.contains(action.as_str()),
            };
            if !known {
                return Err(RemediationError::Message(format!(
                    "dangling treatment action {action} on remediation {}",
                    rem.id
                )));
            }
        }
        if let Some(waiver) = &rem.waiver
            && let Some(exception_id) = &waiver.exception_id
            && !exception_ids.contains(exception_id.as_str())
        {
            return Err(RemediationError::Message(format!(
                "dangling exception {exception_id} on remediation {}",
                rem.id
            )));
        }
        validate_principal_identity(&rem.owner, &identity_ids, &rem.id)?;
        if let Some(closed_by) = &rem.closed_by {
            validate_principal_identity(closed_by, &identity_ids, &rem.id)?;
        }
        for req in &rem.evidence_of_fix {
            if let Some(id) = &req.evidence_requirement_id
                && !evidence_ids.contains(id.as_str())
            {
                return Err(RemediationError::Message(format!(
                    "dangling evidence requirement {id} on remediation {}",
                    rem.id
                )));
            }
        }
        if let Some(prior) = &rem.supersedes
            && !rem_ids.contains(prior.as_str())
        {
            return Err(RemediationError::Message(format!(
                "dangling supersedes {prior} on remediation {}",
                rem.id
            )));
        }
        if let Some(successor) = &rem.superseded_by
            && !rem_ids.contains(successor.as_str())
        {
            return Err(RemediationError::Message(format!(
                "dangling supersededBy {successor} on remediation {}",
                rem.id
            )));
        }
    }
    Ok(())
}

fn validate_principal_identity(
    principal: &PrincipalRef,
    identity_ids: &BTreeSet<String>,
    rem_id: &RemediationId,
) -> Result<(), RemediationError> {
    match principal {
        PrincipalRef::Identity(id) if !identity_ids.contains(id.as_str()) => {
            Err(RemediationError::Message(format!(
                "dangling owner identity {id} on remediation {rem_id}"
            )))
        }
        _ => Ok(()),
    }
}

pub fn validate_remediation_waivers_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Result<(), RemediationError> {
    for rem in &assessment.remediations {
        if rem.state == RemediationState::AcceptedWaived && !waiver_in_force(assessment, rem, as_of)
        {
            return Err(RemediationError::Message(format!(
                "expired or revoked waiver cannot remain AcceptedWaived on {}",
                rem.id
            )));
        }
    }
    Ok(())
}

pub fn validate_remediation_slas_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Result<(), RemediationError> {
    for rem in &assessment.remediations {
        if rem.state == RemediationState::AcceptedWaived {
            continue;
        }
        if rem.sla_overdue(as_of) {
            return Err(RemediationError::Message(format!(
                "overdue remediation SLA {}",
                rem.id
            )));
        }
    }
    Ok(())
}
