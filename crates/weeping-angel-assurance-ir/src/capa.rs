//! Canonical nonconformity and CAPA records. Created only by explicit open/propose.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::remediation::{VerificationMode, VerificationState};
use crate::validation::IrValidationError;
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssetId, AuditFindingId, AuditId, ControlId,
    CorrectiveActionId, EventRef, IncidentId, NonconformityId, PrincipalRef, ProcessingActivityId,
    RemediationRef, RequirementId, SubjectSelector,
};

fn capa_version_default() -> u32 {
    1
}

fn default_capa_min_effective_results() -> u32 {
    2
}

fn default_sustained_window_secs() -> u64 {
    14 * 24 * 3600
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CapaError {
    #[error("illegal CAPA status transition {from:?} → {to:?}")]
    InvalidTransition {
        from: NonconformityStatus,
        to: NonconformityStatus,
    },
    #[error("root cause analysis is required before leaving Contained")]
    MissingRootCause,
    #[error("classification decision is required before planning corrective action")]
    Unclassified,
    #[error("closed, cancelled, or superseded CAPA records are immutable")]
    ImmutableClosure,
    #[error("effectiveness review is not satisfied; cannot close")]
    EffectivenessNotSatisfied,
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NonconformitySourceKind {
    AuditFinding,
    Incident,
    ControlRegression,
    #[default]
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NonconformitySource {
    pub kind: NonconformitySourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_finding_id: Option<AuditFindingId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<AuditId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident_id: Option<IncidentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_ref: Option<EventRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NonconformityClassification {
    Major,
    Minor,
    Opportunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NonconformitySeverity {
    Informational,
    Notable,
    Material,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NonconformityStatus {
    #[default]
    Open,
    Contained,
    RootCauseIdentified,
    CorrectiveActionPlanned,
    Implemented,
    EffectivenessReview,
    Closed,
    Cancelled,
    Superseded,
}

impl NonconformityStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Superseded)
    }

    pub fn is_immutable(self) -> bool {
        matches!(self, Self::Closed | Self::Cancelled | Self::Superseded)
    }

    pub fn can_transition(from: Self, to: Self) -> bool {
        use NonconformityStatus::*;
        matches!(
            (from, to),
            (Open, Contained)
                | (Open, Cancelled)
                | (Open, Superseded)
                | (Contained, RootCauseIdentified)
                | (Contained, Cancelled)
                | (Contained, Superseded)
                | (RootCauseIdentified, CorrectiveActionPlanned)
                | (RootCauseIdentified, Cancelled)
                | (RootCauseIdentified, Superseded)
                | (CorrectiveActionPlanned, Implemented)
                | (CorrectiveActionPlanned, Cancelled)
                | (CorrectiveActionPlanned, Superseded)
                | (Implemented, EffectivenessReview)
                | (Implemented, CorrectiveActionPlanned)
                | (Implemented, Cancelled)
                | (Implemented, Superseded)
                | (EffectivenessReview, Closed)
                | (EffectivenessReview, Implemented)
                | (EffectivenessReview, CorrectiveActionPlanned)
                | (EffectivenessReview, Cancelled)
                | (EffectivenessReview, Superseded)
                | (Closed, Open)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NonconformityScope {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<RequirementId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_ids: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processing_activity_ids: Vec<ProcessingActivityId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub population: Vec<SubjectSelector>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainmentAction {
    pub id: String,
    pub description: String,
    pub performed_at: DateTime<Utc>,
    pub performed_by: PrincipalRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootCauseAnalysis {
    pub method: String,
    pub statement: String,
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: PrincipalRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CorrectiveActionKind {
    Corrective,
    Preventive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CorrectiveActionStatus {
    #[default]
    Planned,
    InProgress,
    Implemented,
    EffectivenessReview,
    Verified,
    FailedReview,
    Cancelled,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivenessCriteria {
    pub mode: VerificationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
    #[serde(default = "default_capa_min_effective_results")]
    pub min_effective_results: u32,
    #[serde(default)]
    pub independent_verifier: bool,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
}

impl Default for EffectivenessCriteria {
    fn default() -> Self {
        Self {
            mode: VerificationMode::SustainedWindow,
            window: Some(default_sustained_window_secs()),
            min_effective_results: default_capa_min_effective_results(),
            independent_verifier: false,
            statement: String::new(),
            control_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EffectivenessReviewStatus {
    #[default]
    NotStarted,
    InWindow,
    Satisfied,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivenessReview {
    pub period: ReviewPeriod,
    pub reviewer: PrincipalRef,
    pub status: EffectivenessReviewStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub result_digests: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClosureOutcome {
    ClosedEffective,
    Cancelled,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosureDecision {
    pub closed_by: PrincipalRef,
    pub closed_at: DateTime<Utc>,
    pub rationale: String,
    pub outcome: ClosureOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NonconformityEventKind {
    Opened,
    Classified,
    Contained,
    RootCauseRecorded,
    ActionPlanned,
    Implemented,
    ReviewStarted,
    ReviewFailed,
    Closed,
    Cancelled,
    Superseded,
    Reopened,
    FieldsRevised,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonconformityEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: NonconformityEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CorrectiveActionEventKind {
    Planned,
    Implemented,
    ReviewFailed,
    Cancelled,
    Superseded,
    FieldsRevised,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectiveActionEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: CorrectiveActionEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nonconformity {
    pub id: NonconformityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    pub title: String,
    pub description: String,
    pub source: NonconformitySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<NonconformityClassification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classified_by: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classified_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<NonconformitySeverity>,
    pub status: NonconformityStatus,
    pub owner: PrincipalRef,
    pub detected_at: DateTime<Utc>,
    pub opened_at: DateTime<Utc>,
    pub opened_by: PrincipalRef,
    #[serde(default)]
    pub affected: NonconformityScope,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containment: Vec<ContainmentAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<RootCauseAnalysis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corrective_action_ids: Vec<CorrectiveActionId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<RemediationRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectiveness: Option<EffectivenessReview>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure: Option<ClosureDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<NonconformityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<NonconformityId>,
    #[serde(default = "capa_version_default")]
    pub version: u32,
    #[serde(default)]
    pub history: Vec<NonconformityEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectiveAction {
    pub id: CorrectiveActionId,
    pub nonconformity_id: NonconformityId,
    pub kind: CorrectiveActionKind,
    pub title: String,
    pub description: String,
    pub owner: PrincipalRef,
    pub target_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implemented_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implementation_evidence: Vec<String>,
    pub effectiveness_criteria: EffectivenessCriteria,
    pub review_period: ReviewPeriod,
    pub reviewer: PrincipalRef,
    pub status: CorrectiveActionStatus,
    #[serde(default)]
    pub verification_state: VerificationState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<RemediationRef>,
    #[serde(default)]
    pub history: Vec<CorrectiveActionEvent>,
}

impl Nonconformity {
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        id: NonconformityId,
        title: impl Into<String>,
        description: impl Into<String>,
        source: NonconformitySource,
        owner: PrincipalRef,
        detected_at: DateTime<Utc>,
        opened_at: DateTime<Utc>,
        opened_by: PrincipalRef,
    ) -> Self {
        let affected = NonconformityScope {
            control_ids: source.control_ids.clone(),
            ..NonconformityScope::default()
        };
        Self {
            id,
            schema_version: Some(ASSURANCE_IR_SCHEMA.into()),
            title: title.into(),
            description: description.into(),
            source,
            classification: None,
            classification_rationale: None,
            classified_by: None,
            classified_at: None,
            severity: None,
            status: NonconformityStatus::Open,
            owner,
            detected_at,
            opened_at,
            opened_by: opened_by.clone(),
            affected,
            containment: Vec::new(),
            root_cause: None,
            corrective_action_ids: Vec::new(),
            remediation_refs: Vec::new(),
            effectiveness: None,
            closure: None,
            superseded_by: None,
            supersedes: None,
            version: 1,
            history: vec![NonconformityEvent {
                version: 1,
                at: opened_at,
                principal: Some(opened_by),
                kind: NonconformityEventKind::Opened,
            }],
        }
    }

    fn reject_immutable(&self) -> Result<(), CapaError> {
        if self.status.is_immutable() {
            Err(CapaError::ImmutableClosure)
        } else {
            Ok(())
        }
    }

    fn append(&mut self, kind: NonconformityEventKind, principal: PrincipalRef, at: DateTime<Utc>) {
        self.version = self.version.saturating_add(1);
        self.history.push(NonconformityEvent {
            version: self.version,
            at,
            principal: Some(principal),
            kind,
        });
    }

    fn set_status(
        &mut self,
        to: NonconformityStatus,
        kind: NonconformityEventKind,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        if !NonconformityStatus::can_transition(self.status, to) {
            return Err(CapaError::InvalidTransition {
                from: self.status,
                to,
            });
        }
        self.status = to;
        self.append(kind, principal, at);
        Ok(())
    }

    pub fn contain(
        &mut self,
        action: ContainmentAction,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        if action.description.trim().is_empty() {
            return Err(CapaError::Message(
                "containment description is required".into(),
            ));
        }
        if !principal_present(&action.performed_by) {
            return Err(CapaError::Message(
                "containment performedBy is required".into(),
            ));
        }
        if action.evidence_refs.iter().all(|r| r.trim().is_empty())
            && action.description.trim().is_empty()
        {
            return Err(CapaError::Message(
                "containment requires evidence refs or a statement".into(),
            ));
        }
        self.containment.push(action);
        if self.status == NonconformityStatus::Open {
            self.set_status(
                NonconformityStatus::Contained,
                NonconformityEventKind::Contained,
                principal,
                at,
            )?;
        } else {
            self.append(NonconformityEventKind::Contained, principal, at);
        }
        Ok(())
    }

    pub fn record_root_cause(
        &mut self,
        rca: RootCauseAnalysis,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        if rca.statement.trim().is_empty() {
            return Err(CapaError::MissingRootCause);
        }
        if !principal_present(&rca.recorded_by) {
            return Err(CapaError::MissingRootCause);
        }
        if self.status != NonconformityStatus::Contained {
            return Err(CapaError::InvalidTransition {
                from: self.status,
                to: NonconformityStatus::RootCauseIdentified,
            });
        }
        self.root_cause = Some(rca);
        self.set_status(
            NonconformityStatus::RootCauseIdentified,
            NonconformityEventKind::RootCauseRecorded,
            principal,
            at,
        )
    }

    pub fn classify(
        &mut self,
        classification: NonconformityClassification,
        rationale: impl Into<String>,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        let rationale = rationale.into();
        if rationale.trim().is_empty() {
            return Err(CapaError::Message(
                "classification rationale is required".into(),
            ));
        }
        if !principal_present(&principal) {
            return Err(CapaError::Message(
                "classification principal is required".into(),
            ));
        }
        self.classification = Some(classification);
        self.classification_rationale = Some(rationale);
        self.classified_by = Some(principal.clone());
        self.classified_at = Some(at);
        self.append(NonconformityEventKind::Classified, principal, at);
        Ok(())
    }

    pub fn plan_corrective_action(
        &mut self,
        action_id: CorrectiveActionId,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        if self.classification.is_none() {
            return Err(CapaError::Unclassified);
        }
        if self
            .root_cause
            .as_ref()
            .is_none_or(|r| r.statement.trim().is_empty())
        {
            return Err(CapaError::MissingRootCause);
        }
        if self.status != NonconformityStatus::RootCauseIdentified
            && self.status != NonconformityStatus::CorrectiveActionPlanned
        {
            return Err(CapaError::InvalidTransition {
                from: self.status,
                to: NonconformityStatus::CorrectiveActionPlanned,
            });
        }
        if !self.corrective_action_ids.iter().any(|id| id == &action_id) {
            self.corrective_action_ids.push(action_id);
        }
        if self.status == NonconformityStatus::RootCauseIdentified {
            self.set_status(
                NonconformityStatus::CorrectiveActionPlanned,
                NonconformityEventKind::ActionPlanned,
                principal,
                at,
            )?;
        } else {
            self.append(NonconformityEventKind::ActionPlanned, principal, at);
        }
        Ok(())
    }

    pub fn mark_implemented(
        &mut self,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        self.set_status(
            NonconformityStatus::Implemented,
            NonconformityEventKind::Implemented,
            principal,
            at,
        )
    }

    pub fn start_effectiveness_review(
        &mut self,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        let period = self
            .effectiveness
            .as_ref()
            .map(|review| review.period.clone())
            .unwrap_or(ReviewPeriod {
                start: at,
                end: at + chrono::Duration::days(21),
            });
        self.effectiveness = Some(EffectivenessReview {
            period,
            reviewer: principal.clone(),
            status: EffectivenessReviewStatus::InWindow,
            result_digests: Vec::new(),
            note: None,
        });
        self.set_status(
            NonconformityStatus::EffectivenessReview,
            NonconformityEventKind::ReviewStarted,
            principal,
            at,
        )
    }

    pub fn close(&mut self, decision: ClosureDecision) -> Result<(), CapaError> {
        if self.status.is_terminal() || self.status == NonconformityStatus::Closed {
            return Err(CapaError::ImmutableClosure);
        }
        if self.status != NonconformityStatus::EffectivenessReview {
            return Err(CapaError::InvalidTransition {
                from: self.status,
                to: NonconformityStatus::Closed,
            });
        }
        if decision.rationale.trim().is_empty() {
            return Err(CapaError::Message("closure rationale is required".into()));
        }
        let satisfied = self
            .effectiveness
            .as_ref()
            .is_some_and(|r| r.status == EffectivenessReviewStatus::Satisfied);
        if decision.outcome == ClosureOutcome::ClosedEffective && !satisfied {
            return Err(CapaError::EffectivenessNotSatisfied);
        }
        let principal = decision.closed_by.clone();
        let at = decision.closed_at;
        self.closure = Some(decision);
        self.set_status(
            NonconformityStatus::Closed,
            NonconformityEventKind::Closed,
            principal,
            at,
        )
    }

    pub fn cancel(
        &mut self,
        rationale: impl Into<String>,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        let rationale = rationale.into();
        if rationale.trim().is_empty() {
            return Err(CapaError::Message(
                "cancellation rationale is required".into(),
            ));
        }
        self.closure = Some(ClosureDecision {
            closed_by: principal.clone(),
            closed_at: at,
            rationale,
            outcome: ClosureOutcome::Cancelled,
        });
        self.set_status(
            NonconformityStatus::Cancelled,
            NonconformityEventKind::Cancelled,
            principal,
            at,
        )
    }

    pub fn supersede(
        &mut self,
        successor: NonconformityId,
        rationale: impl Into<String>,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        self.reject_immutable()?;
        if successor == self.id {
            return Err(CapaError::Message(
                "supersession successor must not be self".into(),
            ));
        }
        let rationale = rationale.into();
        if rationale.trim().is_empty() {
            return Err(CapaError::Message(
                "supersession rationale is required".into(),
            ));
        }
        self.superseded_by = Some(successor);
        self.closure = Some(ClosureDecision {
            closed_by: principal.clone(),
            closed_at: at,
            rationale,
            outcome: ClosureOutcome::Superseded,
        });
        self.set_status(
            NonconformityStatus::Superseded,
            NonconformityEventKind::Superseded,
            principal,
            at,
        )
    }

    pub fn reopen(
        &mut self,
        principal: PrincipalRef,
        rationale: impl Into<String>,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        if self.status != NonconformityStatus::Closed {
            return Err(CapaError::InvalidTransition {
                from: self.status,
                to: NonconformityStatus::Open,
            });
        }
        if rationale.into().trim().is_empty() {
            return Err(CapaError::Message("reopen rationale is required".into()));
        }
        if !principal_present(&principal) {
            return Err(CapaError::Message("reopen principal is required".into()));
        }
        self.closure = None;
        self.effectiveness = None;
        self.set_status(
            NonconformityStatus::Open,
            NonconformityEventKind::Reopened,
            principal,
            at,
        )
    }

    pub fn transition(
        &mut self,
        to: NonconformityStatus,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<(), CapaError> {
        if self.status.is_terminal() {
            return Err(CapaError::ImmutableClosure);
        }
        if self.status == NonconformityStatus::Closed && to != NonconformityStatus::Open {
            return Err(CapaError::ImmutableClosure);
        }
        match to {
            NonconformityStatus::Contained => {
                if self.containment.is_empty() {
                    return Err(CapaError::Message(
                        "containment actions are required".into(),
                    ));
                }
                self.set_status(to, NonconformityEventKind::Contained, principal, at)
            }
            NonconformityStatus::RootCauseIdentified => {
                if !has_rca(self) {
                    return Err(CapaError::MissingRootCause);
                }
                self.set_status(to, NonconformityEventKind::RootCauseRecorded, principal, at)
            }
            NonconformityStatus::CorrectiveActionPlanned => {
                if !has_rca(self) {
                    return Err(CapaError::MissingRootCause);
                }
                if self.classification.is_none() {
                    return Err(CapaError::Unclassified);
                }
                if self.corrective_action_ids.is_empty()
                    && self.status != NonconformityStatus::Implemented
                    && self.status != NonconformityStatus::EffectivenessReview
                {
                    return Err(CapaError::Message(
                        "at least one corrective action is required".into(),
                    ));
                }
                let kind = if self.status == NonconformityStatus::EffectivenessReview {
                    NonconformityEventKind::ReviewFailed
                } else {
                    NonconformityEventKind::ActionPlanned
                };
                self.set_status(to, kind, principal, at)
            }
            NonconformityStatus::Implemented => {
                let kind = if self.status == NonconformityStatus::EffectivenessReview {
                    NonconformityEventKind::ReviewFailed
                } else {
                    NonconformityEventKind::Implemented
                };
                self.set_status(to, kind, principal, at)
            }
            NonconformityStatus::EffectivenessReview => {
                self.set_status(to, NonconformityEventKind::ReviewStarted, principal, at)
            }
            NonconformityStatus::Closed => Err(CapaError::Message(
                "close requires an explicit ClosureDecision".into(),
            )),
            NonconformityStatus::Cancelled => Err(CapaError::Message(
                "cancel requires accountable rationale".into(),
            )),
            NonconformityStatus::Superseded => Err(CapaError::Message(
                "supersede requires rationale and a successor id".into(),
            )),
            NonconformityStatus::Open => Err(CapaError::Message(
                "reopen requires principal and rationale".into(),
            )),
        }
    }
}

impl CorrectiveAction {
    #[allow(clippy::too_many_arguments)]
    pub fn plan(
        id: CorrectiveActionId,
        nonconformity_id: NonconformityId,
        kind: CorrectiveActionKind,
        title: impl Into<String>,
        description: impl Into<String>,
        owner: PrincipalRef,
        target_date: DateTime<Utc>,
        effectiveness_criteria: EffectivenessCriteria,
        review_period: ReviewPeriod,
        reviewer: PrincipalRef,
    ) -> Self {
        Self {
            id,
            nonconformity_id,
            kind,
            title: title.into(),
            description: description.into(),
            owner,
            target_date,
            implemented_at: None,
            implementation_evidence: Vec::new(),
            effectiveness_criteria,
            review_period,
            reviewer,
            status: CorrectiveActionStatus::Planned,
            verification_state: VerificationState::default(),
            remediation_refs: Vec::new(),
            history: vec![CorrectiveActionEvent {
                version: 1,
                at: target_date,
                principal: None,
                kind: CorrectiveActionEventKind::Planned,
            }],
        }
    }

    pub fn mark_implemented(
        &mut self,
        implemented_at: DateTime<Utc>,
        evidence: Vec<String>,
    ) -> Result<(), CapaError> {
        if matches!(
            self.status,
            CorrectiveActionStatus::Cancelled | CorrectiveActionStatus::Superseded
        ) {
            return Err(CapaError::ImmutableClosure);
        }
        if evidence.iter().all(|r| r.trim().is_empty()) {
            return Err(CapaError::Message(
                "implementation evidence is required".into(),
            ));
        }
        self.implemented_at = Some(implemented_at);
        self.implementation_evidence = evidence;
        self.status = CorrectiveActionStatus::Implemented;
        self.history.push(CorrectiveActionEvent {
            version: self.history.len() as u32 + 1,
            at: implemented_at,
            principal: None,
            kind: CorrectiveActionEventKind::Implemented,
        });
        Ok(())
    }

    pub fn is_overdue(&self, as_of: DateTime<Utc>, parent: &Nonconformity) -> bool {
        if matches!(
            parent.status,
            NonconformityStatus::Closed
                | NonconformityStatus::Cancelled
                | NonconformityStatus::Superseded
        ) {
            return false;
        }
        if matches!(
            self.status,
            CorrectiveActionStatus::Implemented
                | CorrectiveActionStatus::Verified
                | CorrectiveActionStatus::Cancelled
                | CorrectiveActionStatus::Superseded
        ) {
            return false;
        }
        self.target_date <= as_of
    }
}

fn has_rca(nc: &Nonconformity) -> bool {
    nc.root_cause
        .as_ref()
        .is_some_and(|r| !r.statement.trim().is_empty())
}

pub(crate) fn principal_present(principal: &PrincipalRef) -> bool {
    match principal {
        PrincipalRef::Identity(id) => !id.as_str().trim().is_empty(),
        PrincipalRef::Team(name) | PrincipalRef::Role(name) => !name.trim().is_empty(),
    }
}

fn later_than_contained(status: NonconformityStatus) -> bool {
    matches!(
        status,
        NonconformityStatus::RootCauseIdentified
            | NonconformityStatus::CorrectiveActionPlanned
            | NonconformityStatus::Implemented
            | NonconformityStatus::EffectivenessReview
            | NonconformityStatus::Closed
    )
}

fn later_than_rca(status: NonconformityStatus) -> bool {
    matches!(
        status,
        NonconformityStatus::CorrectiveActionPlanned
            | NonconformityStatus::Implemented
            | NonconformityStatus::EffectivenessReview
            | NonconformityStatus::Closed
    )
}

pub fn validate_capa_inventory(assessment: &AssessmentDefinition) -> Result<(), IrValidationError> {
    if assessment.nonconformities.is_empty() && assessment.corrective_actions.is_empty() {
        return Ok(());
    }

    let mut nc_ids = BTreeSet::new();
    for nc in &assessment.nonconformities {
        if !nc_ids.insert(nc.id.as_str().to_string()) {
            return Err(msg(format!("duplicate nonconformity id {}", nc.id)));
        }
        validate_nonconformity(nc, assessment)?;
    }

    let mut action_ids = BTreeSet::new();
    let mut actions_by_nc: BTreeMap<String, Vec<&CorrectiveAction>> = BTreeMap::new();
    for action in &assessment.corrective_actions {
        if !action_ids.insert(action.id.as_str().to_string()) {
            return Err(msg(format!("duplicate corrective action id {}", action.id)));
        }
        if !nc_ids.contains(action.nonconformity_id.as_str()) {
            return Err(msg(format!(
                "dangling corrective action {} nonconformity {}",
                action.id, action.nonconformity_id
            )));
        }
        validate_corrective_action(action, assessment)?;
        actions_by_nc
            .entry(action.nonconformity_id.as_str().to_string())
            .or_default()
            .push(action);
    }

    for nc in &assessment.nonconformities {
        for action_id in &nc.corrective_action_ids {
            let Some(action) = assessment
                .corrective_actions
                .iter()
                .find(|a| a.id == *action_id)
            else {
                return Err(msg(format!(
                    "dangling nonconformity {} corrective action {}",
                    nc.id, action_id
                )));
            };
            if action.nonconformity_id != nc.id {
                return Err(msg(format!(
                    "corrective action {} does not point back to {}",
                    action.id, nc.id
                )));
            }
        }
        if later_than_rca(nc.status)
            && nc.corrective_action_ids.is_empty()
            && !matches!(
                nc.status,
                NonconformityStatus::Cancelled | NonconformityStatus::Superseded
            )
        {
            return Err(msg(format!(
                "nonconformity {} requires at least one corrective action",
                nc.id
            )));
        }
        let _ = actions_by_nc.get(nc.id.as_str());
    }

    resolve_audit_nonconformity_refs(assessment, &nc_ids)?;
    Ok(())
}

pub fn validate_capa_at(
    assessment: &AssessmentDefinition,
    _as_of: DateTime<Utc>,
) -> Result<(), IrValidationError> {
    validate_capa_inventory(assessment)?;
    for action in &assessment.corrective_actions {
        if action.review_period.end <= action.review_period.start {
            return Err(msg(format!(
                "corrective action {} review period end must be after start",
                action.id
            )));
        }
    }
    for nc in &assessment.nonconformities {
        if nc.status == NonconformityStatus::Closed {
            let satisfied = nc
                .effectiveness
                .as_ref()
                .is_some_and(|r| r.status == EffectivenessReviewStatus::Satisfied);
            let closed_effective = nc
                .closure
                .as_ref()
                .is_some_and(|c| c.outcome == ClosureOutcome::ClosedEffective);
            if closed_effective && !satisfied {
                return Err(msg(format!(
                    "nonconformity {} is Closed without a Satisfied effectiveness review",
                    nc.id
                )));
            }
        }
    }
    Ok(())
}

fn validate_nonconformity(
    nc: &Nonconformity,
    assessment: &AssessmentDefinition,
) -> Result<(), IrValidationError> {
    if let Some(schema) = &nc.schema_version
        && schema != ASSURANCE_IR_SCHEMA
    {
        return Err(msg(format!(
            "nonconformity {} schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {schema}",
            nc.id
        )));
    }
    if nc.title.trim().is_empty() || nc.description.trim().is_empty() {
        return Err(msg(format!(
            "nonconformity {} title and description must be non-empty",
            nc.id
        )));
    }
    if !principal_present(&nc.owner) || !principal_present(&nc.opened_by) {
        return Err(msg(format!(
            "nonconformity {} owner and opener are required",
            nc.id
        )));
    }
    if later_than_contained(nc.status) && !has_rca(nc) {
        return Err(msg(format!(
            "nonconformity {} is missing root cause analysis",
            nc.id
        )));
    }
    if later_than_rca(nc.status) && nc.classification.is_none() {
        return Err(msg(format!("nonconformity {} is unclassified", nc.id)));
    }
    if nc.status == NonconformityStatus::Closed {
        let Some(closure) = &nc.closure else {
            return Err(msg(format!(
                "nonconformity {} is Closed without a decision",
                nc.id
            )));
        };
        if closure.rationale.trim().is_empty() {
            return Err(msg(format!(
                "nonconformity {} closure rationale is required",
                nc.id
            )));
        }
        if closure.outcome == ClosureOutcome::ClosedEffective
            && !nc
                .effectiveness
                .as_ref()
                .is_some_and(|r| r.status == EffectivenessReviewStatus::Satisfied)
        {
            return Err(msg(format!(
                "nonconformity {} ClosedEffective requires Satisfied effectiveness",
                nc.id
            )));
        }
    }
    if matches!(
        nc.status,
        NonconformityStatus::Cancelled | NonconformityStatus::Superseded
    ) {
        let Some(closure) = &nc.closure else {
            return Err(msg(format!(
                "nonconformity {} {} requires rationale",
                nc.id,
                format!("{:?}", nc.status).to_ascii_lowercase()
            )));
        };
        if closure.rationale.trim().is_empty() {
            return Err(msg(format!(
                "nonconformity {} terminal rationale is required",
                nc.id
            )));
        }
        if nc.status == NonconformityStatus::Superseded && nc.superseded_by.is_none() {
            return Err(msg(format!(
                "nonconformity {} supersession requires a successor id",
                nc.id
            )));
        }
    }
    if !assessment.audit_findings.is_empty()
        && let Some(id) = &nc.source.audit_finding_id
        && !assessment.audit_findings.iter().any(|f| f.id == *id)
    {
        return Err(msg(format!(
            "nonconformity {} dangling audit finding {}",
            nc.id, id
        )));
    }
    if !assessment.audits.is_empty()
        && let Some(id) = &nc.source.audit_id
        && !assessment.audits.iter().any(|a| a.id == *id)
    {
        return Err(msg(format!(
            "nonconformity {} dangling audit {}",
            nc.id, id
        )));
    }
    if !assessment.incidents.is_empty()
        && let Some(id) = &nc.source.incident_id
        && !assessment.incidents.iter().any(|i| i.id == *id)
    {
        return Err(msg(format!(
            "nonconformity {} dangling incident {}",
            nc.id, id
        )));
    }
    if !assessment.remediations.is_empty() {
        for rem in &nc.remediation_refs {
            if !assessment
                .remediations
                .iter()
                .any(|r| r.id.as_str() == rem.as_str())
            {
                return Err(msg(format!(
                    "nonconformity {} dangling remediation {}",
                    nc.id, rem
                )));
            }
        }
    }
    resolve_scope(nc, assessment)?;
    Ok(())
}

fn validate_corrective_action(
    action: &CorrectiveAction,
    assessment: &AssessmentDefinition,
) -> Result<(), IrValidationError> {
    if action.title.trim().is_empty() || action.description.trim().is_empty() {
        return Err(msg(format!(
            "corrective action {} title and description must be non-empty",
            action.id
        )));
    }
    if !principal_present(&action.owner) || !principal_present(&action.reviewer) {
        return Err(msg(format!(
            "corrective action {} owner and reviewer are required",
            action.id
        )));
    }
    if action.effectiveness_criteria.statement.trim().is_empty() {
        return Err(msg(format!(
            "corrective action {} effectiveness criteria statement is required",
            action.id
        )));
    }
    if action.review_period.end <= action.review_period.start {
        return Err(msg(format!(
            "corrective action {} review period end must be after start",
            action.id
        )));
    }
    if !assessment.remediations.is_empty() {
        for rem in &action.remediation_refs {
            if !assessment
                .remediations
                .iter()
                .any(|r| r.id.as_str() == rem.as_str())
            {
                return Err(msg(format!(
                    "corrective action {} dangling remediation {}",
                    action.id, rem
                )));
            }
        }
    }
    Ok(())
}

fn resolve_scope(
    nc: &Nonconformity,
    assessment: &AssessmentDefinition,
) -> Result<(), IrValidationError> {
    if !assessment.controls.is_empty() {
        for id in nc
            .affected
            .control_ids
            .iter()
            .chain(nc.source.control_ids.iter())
        {
            if !assessment.controls.iter().any(|c| c.id() == id) {
                return Err(msg(format!(
                    "nonconformity {} dangling control {}",
                    nc.id, id
                )));
            }
        }
    }
    if !assessment.requirements.is_empty() {
        for id in &nc.affected.requirement_ids {
            if !assessment.requirements.iter().any(|r| r.id() == id) {
                return Err(msg(format!(
                    "nonconformity {} dangling requirement {}",
                    nc.id, id
                )));
            }
        }
    }
    if !assessment.assets.is_empty() {
        for id in &nc.affected.asset_ids {
            if !assessment.assets.iter().any(|a| a.id == *id) {
                return Err(msg(format!(
                    "nonconformity {} dangling asset {}",
                    nc.id, id
                )));
            }
        }
    }
    if !assessment.processing_activities.is_empty() {
        for id in &nc.affected.processing_activity_ids {
            if !assessment.processing_activities.iter().any(|p| p.id == *id) {
                return Err(msg(format!(
                    "nonconformity {} dangling processing activity {}",
                    nc.id, id
                )));
            }
        }
    }
    Ok(())
}

fn resolve_audit_nonconformity_refs(
    assessment: &AssessmentDefinition,
    nc_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    if nc_ids.is_empty() {
        return Ok(());
    }
    for finding in &assessment.audit_findings {
        if let Some(id) = &finding.nonconformity_id
            && !nc_ids.contains(id.as_str())
        {
            return Err(msg(format!(
                "dangling nonconformityId {id} on audit finding {}",
                finding.id
            )));
        }
    }
    for audit in &assessment.audits {
        for id in &audit.nonconformity_refs {
            if !nc_ids.contains(id.as_str()) {
                return Err(msg(format!(
                    "dangling nonconformityRef {id} on audit {}",
                    audit.id
                )));
            }
        }
    }
    Ok(())
}

fn msg(text: String) -> IrValidationError {
    IrValidationError::Message(text)
}

pub fn overdue_action_ids(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Vec<CorrectiveActionId> {
    let ncs: BTreeMap<_, _> = assessment
        .nonconformities
        .iter()
        .map(|nc| (nc.id.as_str(), nc))
        .collect();
    assessment
        .corrective_actions
        .iter()
        .filter(|action| {
            ncs.get(action.nonconformity_id.as_str())
                .is_some_and(|nc| action.is_overdue(as_of, nc))
        })
        .map(|action| action.id.clone())
        .collect()
}
