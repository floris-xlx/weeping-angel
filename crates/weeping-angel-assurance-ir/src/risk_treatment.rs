//! Risk treatment decisions, plans, actions, and immutable acceptance.
//!
//! Network-free IR engine. Does not calculate residual effectiveness, emit
//! tickets, or interpret framework applicability.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, ControlId, ControlImplementationId,
    EvidenceCriticality, EvidenceRequirementId, EvidenceType, PrincipalRef, RemediationRef,
    RiskAcceptanceId, RiskId, RiskStatus, RiskTreatmentId, TreatmentActionId, TreatmentPlanId,
    canonical_digest, validate_stable_id,
};

fn default_version() -> u32 {
    1
}

fn version_is_one(value: &u32) -> bool {
    *value == 1
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TreatmentError {
    #[error("invalid treatment transition {from:?} → {to:?}")]
    InvalidTransition {
        from: TreatmentState,
        to: TreatmentState,
    },
    #[error("missing contract evidence for transfer")]
    MissingContractEvidence,
    #[error("target residual mismatch vs approval")]
    TargetResidualMismatch,
    #[error("immutable acceptance: sealed RiskAcceptance cannot be edited")]
    ImmutableAcceptance,
    #[error("accountable principal is required")]
    MissingPrincipal,
    #[error("missing strategy evidence")]
    MissingStrategyEvidence,
    #[error("partially complete mitigation: required action is not done")]
    IncompleteActions,
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TreatmentStrategy {
    #[default]
    Mitigate,
    Accept,
    Avoid,
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TreatmentState {
    #[default]
    Proposed,
    Approved,
    Executing,
    Verification,
    Completed,
    Cancelled,
    Superseded,
}

impl TreatmentState {
    pub fn can_transition(from: Self, to: Self) -> bool {
        use TreatmentState::*;
        matches!(
            (from, to),
            (Proposed, Approved | Cancelled)
                | (Approved, Executing | Cancelled | Superseded)
                | (Executing, Verification | Cancelled | Superseded)
                | (Verification, Completed | Executing | Cancelled | Superseded)
                | (Completed, Superseded)
        )
    }

    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Proposed | Self::Approved | Self::Executing | Self::Verification
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Superseded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ActionState {
    #[default]
    Proposed,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TreatmentEvidenceKind {
    EnvelopeDigest,
    EvidenceRequirement,
    NarrativeAttestation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum TargetResidualRisk {
    VersionedPlaceholder {
        methodology_version: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        input_note: Option<String>,
    },
}

impl TargetResidualRisk {
    fn validate(&self) -> Result<(), TreatmentError> {
        match self {
            Self::VersionedPlaceholder {
                methodology_version,
                ..
            } => {
                if methodology_version.trim().is_empty() {
                    return Err(TreatmentError::Message(
                        "target residual methodologyVersion is required".into(),
                    ));
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentApproval {
    pub principal: PrincipalRef,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentEvidenceRef {
    pub kind: TreatmentEvidenceKind,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
}

impl TreatmentEvidenceRef {
    fn is_non_empty(&self) -> bool {
        !self.value.trim().is_empty()
    }

    fn demonstrates_organizational_action(&self) -> bool {
        if !self.is_non_empty() {
            return false;
        }
        match self.kind {
            TreatmentEvidenceKind::NarrativeAttestation => {
                self.principal.is_some() && self.at.is_some()
            }
            TreatmentEvidenceKind::EnvelopeDigest | TreatmentEvidenceKind::EvidenceRequirement => {
                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentEvidenceExpectation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<EvidenceRequirementId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_type: Option<EvidenceType>,
    pub criticality: EvidenceCriticality,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum TreatmentEventKind {
    Created,
    FieldsRevised,
    StateTransition {
        from: TreatmentState,
        to: TreatmentState,
    },
    Superseded {
        successor: RiskTreatmentId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    #[serde(flatten)]
    pub kind: TreatmentEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentAction {
    pub id: TreatmentActionId,
    pub title: String,
    pub owner: PrincipalRef,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub state: ActionState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implementation_ids: Vec<ControlImplementationId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<RemediationRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<TreatmentEvidenceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlan {
    pub id: TreatmentPlanId,
    pub owner: PrincipalRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<TreatmentAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEvidence {
    pub contract: TreatmentEvidenceRef,
    pub transferee: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskAcceptance {
    pub id: RiskAcceptanceId,
    pub risk_id: RiskId,
    pub treatment_id: RiskTreatmentId,
    pub principal: PrincipalRef,
    pub rationale: String,
    pub approved_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<TreatmentEvidenceRef>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub digest: String,
}

impl RiskAcceptance {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RiskAcceptanceId,
        risk_id: RiskId,
        treatment_id: RiskTreatmentId,
        principal: PrincipalRef,
        rationale: impl Into<String>,
        approved_at: DateTime<Utc>,
        valid_from: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        evidence: Vec<TreatmentEvidenceRef>,
    ) -> Self {
        let mut acceptance = Self {
            id,
            risk_id,
            treatment_id,
            principal,
            rationale: rationale.into(),
            approved_at,
            valid_from,
            expires_at,
            review_at: None,
            evidence,
            digest: String::new(),
        };
        acceptance.digest = acceptance.body_digest().unwrap_or_default();
        acceptance
    }

    fn body_digest(&self) -> Result<String, TreatmentError> {
        let mut body = self.clone();
        body.digest.clear();
        digest_of(&body)
    }

    fn in_force_at(&self, as_of: DateTime<Utc>) -> bool {
        self.valid_from <= as_of && as_of < self.expires_at
    }

    fn validate_fields(&self) -> Result<(), TreatmentError> {
        require_principal(&self.principal)?;
        if self.rationale.trim().is_empty() {
            return Err(TreatmentError::Message(
                "acceptance rationale is required".into(),
            ));
        }
        if self.valid_from >= self.expires_at {
            return Err(TreatmentError::Message(
                "acceptance validFrom must be before expiresAt".into(),
            ));
        }
        if let Some(review_at) = self.review_at
            && review_at > self.expires_at
        {
            return Err(TreatmentError::Message(
                "acceptance reviewAt must be on or before expiresAt".into(),
            ));
        }
        if self.evidence.is_empty() || self.evidence.iter().all(|e| !e.is_non_empty()) {
            return Err(TreatmentError::MissingStrategyEvidence);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTreatmentDecision {
    pub schema_version: String,
    pub id: RiskTreatmentId,
    pub risk_id: RiskId,
    pub strategy: TreatmentStrategy,
    #[serde(default)]
    pub state: TreatmentState,
    pub owner: PrincipalRef,
    pub decision_principal: PrincipalRef,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_date: Option<DateTime<Utc>>,
    pub target_residual: TargetResidualRisk,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implementation_ids: Vec<ControlImplementationId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<RemediationRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_expectations: Vec<TreatmentEvidenceExpectation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<TreatmentApproval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<TreatmentPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<RiskAcceptance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avoid_evidence: Option<TreatmentEvidenceRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_evidence: Option<TransferEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RiskTreatmentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<RiskTreatmentId>,
    #[serde(default = "default_version", skip_serializing_if = "version_is_one")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<TreatmentEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_target_residual_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sealed_acceptance_digest: Option<String>,
}

impl RiskTreatmentDecision {
    pub fn propose(
        id: RiskTreatmentId,
        risk_id: RiskId,
        strategy: TreatmentStrategy,
        owner: PrincipalRef,
        decision_principal: PrincipalRef,
        target_residual: TargetResidualRisk,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            risk_id,
            strategy,
            state: TreatmentState::Proposed,
            owner,
            decision_principal,
            rationale: String::new(),
            target_date: None,
            target_residual,
            canonical_control_ids: Vec::new(),
            implementation_ids: Vec::new(),
            remediation_refs: Vec::new(),
            evidence_expectations: Vec::new(),
            approval: None,
            review_at: None,
            expires_at: None,
            plan: None,
            acceptance: None,
            avoid_evidence: None,
            transfer_evidence: None,
            supersedes: None,
            superseded_by: None,
            version: 1,
            history: vec![TreatmentEvent {
                version: 1,
                at: Utc::now(),
                principal: None,
                kind: TreatmentEventKind::Created,
            }],
            approved_target_residual_digest: None,
            sealed_acceptance_digest: None,
        }
    }

    pub fn transition(
        mut self,
        to: TreatmentState,
        principal: PrincipalRef,
        at: DateTime<Utc>,
    ) -> Result<Self, TreatmentError> {
        let from = self.state;
        if !TreatmentState::can_transition(from, to) {
            return Err(TreatmentError::InvalidTransition { from, to });
        }
        match to {
            TreatmentState::Approved => self.guard_approve()?,
            TreatmentState::Completed => self.guard_complete()?,
            TreatmentState::Superseded => self.guard_supersede()?,
            _ => {}
        }
        self.state = to;
        self.version = self.version.saturating_add(1).max(2);
        self.history.push(TreatmentEvent {
            version: self.version,
            at,
            principal: Some(principal),
            kind: TreatmentEventKind::StateTransition { from, to },
        });
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), TreatmentError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(TreatmentError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}"
            )));
        }
        self.target_residual.validate()?;
        require_principal(&self.owner)?;
        require_principal(&self.decision_principal)?;
        self.validate_history()?;
        if !matches!(self.state, TreatmentState::Proposed) {
            if self.rationale.trim().is_empty() {
                return Err(TreatmentError::Message(
                    "rationale is required once a treatment leaves Proposed".into(),
                ));
            }
            if matches!(
                self.state,
                TreatmentState::Approved
                    | TreatmentState::Executing
                    | TreatmentState::Verification
                    | TreatmentState::Completed
            ) {
                self.ensure_approval()?;
                self.ensure_residual_frozen()?;
                self.ensure_acceptance_sealed()?;
            }
        }
        if self.state == TreatmentState::Completed {
            self.strategy_completion()?;
            self.ensure_residual_matches_approval()?;
        }
        if self.state == TreatmentState::Superseded {
            self.guard_supersede()?;
        }
        Ok(())
    }

    fn guard_approve(&mut self) -> Result<(), TreatmentError> {
        self.ensure_approval()?;
        if self.rationale.trim().is_empty() {
            return Err(TreatmentError::Message(
                "rationale is required to approve a treatment".into(),
            ));
        }
        require_principal(&self.decision_principal)?;
        self.target_residual.validate()?;
        self.approved_target_residual_digest = Some(digest_of(&self.target_residual)?);
        if self.strategy == TreatmentStrategy::Accept {
            self.seal_acceptance()?;
        }
        Ok(())
    }

    fn guard_complete(&self) -> Result<(), TreatmentError> {
        self.ensure_residual_matches_approval()?;
        self.strategy_completion()?;
        Ok(())
    }

    fn guard_supersede(&self) -> Result<(), TreatmentError> {
        match &self.superseded_by {
            Some(successor) if successor.as_str() != self.id.as_str() => Ok(()),
            _ => Err(TreatmentError::Message(
                "supersession requires supersededBy pointing at a different treatment id".into(),
            )),
        }
    }

    fn ensure_approval(&self) -> Result<(), TreatmentError> {
        let Some(approval) = &self.approval else {
            return Err(TreatmentError::Message(
                "treatment approval (principal + time) is required".into(),
            ));
        };
        require_principal(&approval.principal)
    }

    fn ensure_residual_frozen(&self) -> Result<(), TreatmentError> {
        if self.approved_target_residual_digest.is_none() {
            return Err(TreatmentError::TargetResidualMismatch);
        }
        Ok(())
    }

    fn ensure_residual_matches_approval(&self) -> Result<(), TreatmentError> {
        let current = digest_of(&self.target_residual)?;
        if self.approved_target_residual_digest.as_ref() != Some(&current) {
            return Err(TreatmentError::TargetResidualMismatch);
        }
        Ok(())
    }

    fn seal_acceptance(&mut self) -> Result<(), TreatmentError> {
        let Some(acceptance) = self.acceptance.as_mut() else {
            return Err(TreatmentError::Message(
                "accept strategy requires RiskAcceptance before approval".into(),
            ));
        };
        acceptance.validate_fields()?;
        if acceptance.digest.is_empty() {
            acceptance.digest = acceptance.body_digest()?;
        } else if acceptance.digest != acceptance.body_digest()? {
            return Err(TreatmentError::ImmutableAcceptance);
        }
        self.sealed_acceptance_digest = Some(digest_of(acceptance)?);
        Ok(())
    }

    fn ensure_acceptance_sealed(&self) -> Result<(), TreatmentError> {
        if self.strategy != TreatmentStrategy::Accept {
            return Ok(());
        }
        let Some(acceptance) = &self.acceptance else {
            return Err(TreatmentError::Message(
                "accept strategy requires RiskAcceptance".into(),
            ));
        };
        let Some(sealed) = &self.sealed_acceptance_digest else {
            return Err(TreatmentError::ImmutableAcceptance);
        };
        if digest_of(acceptance)? != *sealed {
            return Err(TreatmentError::ImmutableAcceptance);
        }
        Ok(())
    }

    fn strategy_completion(&self) -> Result<(), TreatmentError> {
        match self.strategy {
            TreatmentStrategy::Mitigate => self.complete_mitigate()?,
            TreatmentStrategy::Accept => self.complete_accept()?,
            TreatmentStrategy::Avoid => self.complete_avoid()?,
            TreatmentStrategy::Transfer => self.complete_transfer()?,
        }
        self.ensure_required_evidence()
    }

    fn complete_mitigate(&self) -> Result<(), TreatmentError> {
        let Some(plan) = &self.plan else {
            return Err(TreatmentError::IncompleteActions);
        };
        let required: Vec<_> = plan.actions.iter().filter(|a| a.required).collect();
        if required.is_empty() {
            return Err(TreatmentError::IncompleteActions);
        }
        if required.iter().any(|a| a.state != ActionState::Done) {
            return Err(TreatmentError::IncompleteActions);
        }
        if required.iter().any(|a| a.title.trim().is_empty()) {
            return Err(TreatmentError::Message(
                "mitigation action title is required".into(),
            ));
        }
        Ok(())
    }

    fn complete_accept(&self) -> Result<(), TreatmentError> {
        let Some(acceptance) = &self.acceptance else {
            return Err(TreatmentError::Message(
                "accept strategy requires sealed RiskAcceptance".into(),
            ));
        };
        acceptance.validate_fields()?;
        self.ensure_acceptance_sealed()
    }

    fn complete_avoid(&self) -> Result<(), TreatmentError> {
        match &self.avoid_evidence {
            Some(evidence) if evidence.demonstrates_organizational_action() => Ok(()),
            _ => Err(TreatmentError::MissingStrategyEvidence),
        }
    }

    fn complete_transfer(&self) -> Result<(), TreatmentError> {
        let Some(evidence) = &self.transfer_evidence else {
            return Err(TreatmentError::MissingContractEvidence);
        };
        if !evidence.contract.is_non_empty() {
            return Err(TreatmentError::MissingContractEvidence);
        }
        if evidence.transferee.trim().is_empty() {
            return Err(TreatmentError::Message(
                "transfer transferee is required".into(),
            ));
        }
        Ok(())
    }

    fn ensure_required_evidence(&self) -> Result<(), TreatmentError> {
        let required = self
            .evidence_expectations
            .iter()
            .any(|e| e.criticality == EvidenceCriticality::Required);
        if !required {
            return Ok(());
        }
        if self.attached_evidence().iter().any(|e| e.is_non_empty()) {
            return Ok(());
        }
        Err(TreatmentError::MissingStrategyEvidence)
    }

    fn attached_evidence(&self) -> Vec<&TreatmentEvidenceRef> {
        let mut out = Vec::new();
        if let Some(plan) = &self.plan {
            for action in &plan.actions {
                out.extend(action.evidence.iter());
            }
        }
        if let Some(evidence) = &self.avoid_evidence {
            out.push(evidence);
        }
        if let Some(transfer) = &self.transfer_evidence {
            out.push(&transfer.contract);
        }
        if let Some(acceptance) = &self.acceptance {
            out.extend(acceptance.evidence.iter());
        }
        out
    }

    fn validate_history(&self) -> Result<(), TreatmentError> {
        let mut last_state = TreatmentState::Proposed;
        let mut saw_transition = false;
        for event in &self.history {
            if let TreatmentEventKind::StateTransition { from, to } = event.kind {
                if !TreatmentState::can_transition(from, to) {
                    return Err(TreatmentError::InvalidTransition { from, to });
                }
                last_state = to;
                saw_transition = true;
            }
        }
        if saw_transition && last_state != self.state {
            return Err(TreatmentError::Message(
                "treatment history does not match current state".into(),
            ));
        }
        Ok(())
    }

    fn cited_control_ids(&self) -> Vec<&ControlId> {
        let mut ids: Vec<&ControlId> = self.canonical_control_ids.iter().collect();
        if let Some(plan) = &self.plan {
            for action in &plan.actions {
                ids.extend(action.control_ids.iter());
            }
        }
        ids
    }

    fn cited_implementation_ids(&self) -> Vec<&ControlImplementationId> {
        let mut ids: Vec<&ControlImplementationId> = self.implementation_ids.iter().collect();
        if let Some(plan) = &self.plan {
            for action in &plan.actions {
                ids.extend(action.implementation_ids.iter());
            }
        }
        ids
    }
}

fn digest_of<T: Serialize>(value: &T) -> Result<String, TreatmentError> {
    canonical_digest(value).map_err(|err| TreatmentError::Message(err.to_string()))
}

fn require_principal(principal: &PrincipalRef) -> Result<(), TreatmentError> {
    match principal {
        PrincipalRef::Identity(_) => Ok(()),
        PrincipalRef::Team(name) | PrincipalRef::Role(name) => {
            if name.trim().is_empty() {
                Err(TreatmentError::MissingPrincipal)
            } else {
                Ok(())
            }
        }
    }
}

fn principal_identity<'a>(principal: &'a PrincipalRef) -> Option<&'a str> {
    match principal {
        PrincipalRef::Identity(id) => Some(id.as_str()),
        PrincipalRef::Team(_) | PrincipalRef::Role(_) => None,
    }
}

pub fn active_treatment<'a>(
    assessment: &'a AssessmentDefinition,
    risk_id: &RiskId,
) -> Option<&'a RiskTreatmentDecision> {
    assessment
        .risk_treatments
        .iter()
        .find(|decision| decision.risk_id == *risk_id && decision.state.is_active())
}

pub fn acceptance_in_force(
    assessment: &AssessmentDefinition,
    risk_id: &RiskId,
    as_of: DateTime<Utc>,
) -> bool {
    assessment
        .risk_treatments
        .iter()
        .any(|decision| decision.risk_id == *risk_id && acceptance_in_force_for(decision, as_of))
}

fn acceptance_in_force_for(decision: &RiskTreatmentDecision, as_of: DateTime<Utc>) -> bool {
    if decision.strategy != TreatmentStrategy::Accept {
        return false;
    }
    if decision.state.is_terminal() || decision.state != TreatmentState::Completed {
        return false;
    }
    decision
        .acceptance
        .as_ref()
        .is_some_and(|acceptance| acceptance.in_force_at(as_of))
}

pub fn treatment_required(
    assessment: &AssessmentDefinition,
    risk_id: &RiskId,
    as_of: DateTime<Utc>,
) -> bool {
    let Some(risk) = assessment.risks.iter().find(|risk| risk.id == *risk_id) else {
        return true;
    };
    if risk.status.is_terminal() {
        return false;
    }
    !assessment
        .risk_treatments
        .iter()
        .any(|decision| decision.risk_id == *risk_id && is_suppressing(decision, as_of))
}

fn is_suppressing(decision: &RiskTreatmentDecision, as_of: DateTime<Utc>) -> bool {
    if decision.state.is_terminal() || decision.state != TreatmentState::Completed {
        return false;
    }
    match decision.strategy {
        TreatmentStrategy::Accept => acceptance_in_force_for(decision, as_of),
        TreatmentStrategy::Mitigate | TreatmentStrategy::Avoid | TreatmentStrategy::Transfer => {
            match decision.expires_at {
                Some(expires_at) if as_of >= expires_at => false,
                _ => true,
            }
        }
    }
}

pub fn validate_treatments_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Result<(), TreatmentError> {
    validate_treatment_inventory(assessment)?;
    for risk in &assessment.risks {
        if risk.status == RiskStatus::Accepted && !acceptance_in_force(assessment, &risk.id, as_of)
        {
            return Err(TreatmentError::Message(format!(
                "expired or missing in-force acceptance for Accepted risk {}",
                risk.id
            )));
        }
    }
    Ok(())
}

pub fn validate_treatment_inventory(
    assessment: &AssessmentDefinition,
) -> Result<(), TreatmentError> {
    let risk_ids: BTreeSet<_> = assessment
        .risks
        .iter()
        .map(|risk| risk.id.as_str().to_string())
        .collect();
    let control_ids: BTreeSet<_> = assessment
        .controls
        .iter()
        .map(|control| control.id().as_str().to_string())
        .collect();
    let implementation_ids: BTreeSet<_> = assessment
        .implementations
        .iter()
        .map(|impln| impln.id().as_str().to_string())
        .collect();
    let evidence_ids: BTreeSet<_> = assessment
        .evidence_requirements
        .iter()
        .map(|ev| ev.id().as_str().to_string())
        .collect();
    let identity_ids: BTreeSet<_> = assessment
        .identities
        .iter()
        .map(|identity| identity.id.as_str().to_string())
        .collect();
    let treatment_ids: BTreeSet<_> = assessment
        .risk_treatments
        .iter()
        .map(|decision| decision.id.as_str().to_string())
        .collect();

    let mut seen_treatments = BTreeSet::new();
    let mut seen_plans = BTreeSet::new();
    let mut seen_actions = BTreeSet::new();
    let mut seen_acceptances = BTreeSet::new();
    let mut active_by_risk: BTreeMap<String, Vec<&RiskTreatmentDecision>> = BTreeMap::new();

    for decision in &assessment.risk_treatments {
        if !seen_treatments.insert(decision.id.as_str().to_string()) {
            return Err(TreatmentError::Message(format!(
                "duplicate treatment id {}",
                decision.id
            )));
        }
        if !risk_ids.contains(decision.risk_id.as_str()) {
            return Err(TreatmentError::Message(format!(
                "dangling treatment riskId {}",
                decision.risk_id
            )));
        }
        decision.validate()?;
        collect_principal_errors(decision, &identity_ids)?;

        let mut dangling = Vec::new();
        for control in decision.cited_control_ids() {
            if !control_ids.contains(control.as_str()) {
                dangling.push(format!("dangling control reference {control}"));
            }
        }
        for impl_id in decision.cited_implementation_ids() {
            if !implementation_ids.contains(impl_id.as_str()) {
                dangling.push(format!("dangling implementation {impl_id}"));
            }
        }
        if !dangling.is_empty() {
            return Err(TreatmentError::Message(dangling.join("; ")));
        }

        for expectation in &decision.evidence_expectations {
            if let Some(id) = &expectation.id
                && !evidence_ids.contains(id.as_str())
            {
                return Err(TreatmentError::Message(format!(
                    "dangling evidence requirement {id} on treatment {}",
                    decision.id
                )));
            }
        }

        if let Some(plan) = &decision.plan {
            if !seen_plans.insert(plan.id.as_str().to_string()) {
                return Err(TreatmentError::Message(format!(
                    "duplicate treatment plan id {}",
                    plan.id
                )));
            }
            for action in &plan.actions {
                if !seen_actions.insert(action.id.as_str().to_string()) {
                    return Err(TreatmentError::Message(format!(
                        "duplicate treatment action id {}",
                        action.id
                    )));
                }
                for remediation in &action.remediation_refs {
                    validate_stable_id(remediation.as_str()).map_err(|err| {
                        TreatmentError::Message(format!(
                            "malformed remediation ref {}: {err}",
                            remediation
                        ))
                    })?;
                }
            }
        }
        if let Some(acceptance) = &decision.acceptance
            && !seen_acceptances.insert(acceptance.id.as_str().to_string())
        {
            return Err(TreatmentError::Message(format!(
                "duplicate acceptance id {}",
                acceptance.id
            )));
        }

        if let Some(prior) = &decision.supersedes {
            if prior.as_str() == decision.id.as_str() || !treatment_ids.contains(prior.as_str()) {
                return Err(TreatmentError::Message(format!(
                    "invalid supersedes reference {prior} on treatment {}",
                    decision.id
                )));
            }
        }
        if let Some(successor) = &decision.superseded_by {
            if successor.as_str() == decision.id.as_str()
                || !treatment_ids.contains(successor.as_str())
            {
                return Err(TreatmentError::Message(format!(
                    "invalid supersededBy reference {successor} on treatment {}",
                    decision.id
                )));
            }
        }

        if decision.state.is_active() {
            active_by_risk
                .entry(decision.risk_id.as_str().to_string())
                .or_default()
                .push(decision);
        }
    }

    for (risk_id, actives) in &active_by_risk {
        if actives.len() > 1 {
            return Err(TreatmentError::Message(format!(
                "risk {risk_id} has more than one active treatment path"
            )));
        }
    }

    for risk in &assessment.risks {
        if let Some(treatment_id) = &risk.treatment_id {
            if !treatment_ids.contains(treatment_id.as_str()) {
                return Err(TreatmentError::Message(format!(
                    "dangling treatment reference {treatment_id} on risk {}",
                    risk.id
                )));
            }
            if let Some(active) = active_treatment(assessment, &risk.id) {
                if active.id.as_str() != treatment_id.as_str() {
                    return Err(TreatmentError::Message(format!(
                        "risk {} treatment_id must equal the active treatment {}",
                        risk.id, active.id
                    )));
                }
            } else if let Some(completed) = latest_completed(assessment, &risk.id)
                && completed.id.as_str() != treatment_id.as_str()
            {
                return Err(TreatmentError::Message(format!(
                    "risk {} treatment_id must equal completed treatment {}",
                    risk.id, completed.id
                )));
            }
        }
        if risk.status == RiskStatus::Accepted {
            let has_record = assessment
                .risk_treatments
                .iter()
                .any(|decision| decision.risk_id == risk.id && decision.acceptance.is_some());
            if !has_record {
                return Err(TreatmentError::Message(format!(
                    "Accepted risk {} is missing an acceptance record",
                    risk.id
                )));
            }
        }
    }

    Ok(())
}

fn latest_completed<'a>(
    assessment: &'a AssessmentDefinition,
    risk_id: &RiskId,
) -> Option<&'a RiskTreatmentDecision> {
    assessment
        .risk_treatments
        .iter()
        .filter(|decision| {
            decision.risk_id == *risk_id && decision.state == TreatmentState::Completed
        })
        .next_back()
}

fn collect_principal_errors(
    decision: &RiskTreatmentDecision,
    identity_ids: &BTreeSet<String>,
) -> Result<(), TreatmentError> {
    let mut principals = vec![&decision.owner, &decision.decision_principal];
    if let Some(approval) = &decision.approval {
        principals.push(&approval.principal);
    }
    if let Some(plan) = &decision.plan {
        principals.push(&plan.owner);
        for action in &plan.actions {
            principals.push(&action.owner);
        }
    }
    if let Some(acceptance) = &decision.acceptance {
        principals.push(&acceptance.principal);
    }
    for principal in principals {
        require_principal(principal)?;
        if let Some(id) = principal_identity(principal)
            && !identity_ids.contains(id)
        {
            return Err(TreatmentError::Message(format!(
                "dangling identity {id} on treatment {}",
                decision.id
            )));
        }
    }
    Ok(())
}
