//! Operational supplier-security lifecycle node for the compliance graph.
//!
//! Expands the inventory stub `{ id, name }` with risk-tiered review, access,
//! approval, and organizational risk linkage. Evidence presence is not
//! acceptance. `Vendor::new` still serializes as two camelCase keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AssetId, ControlId, ExceptionId, IdentityId, PrincipalRef, ProcessingActivityId, RiskId,
    SupplierIssueId, SupplierRequirementId, SupplierReviewId, VendorId,
};

fn default_version() -> u32 {
    1
}

fn version_is_one(value: &u32) -> bool {
    *value == 1
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SupplierClassification {
    #[default]
    Unspecified,
    Supplier,
    Processor,
    HostedService,
    CloudProvider,
    ProfessionalServices,
    Other,
}

impl SupplierClassification {
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }

    pub fn is_processor(self) -> bool {
        matches!(self, Self::Processor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SupplierCriticality {
    #[default]
    Unspecified,
    Low,
    Medium,
    High,
    Critical,
}

impl SupplierCriticality {
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SupplierLifecycleStatus {
    #[default]
    Unspecified,
    Candidate,
    UnderReview,
    Approved,
    Active,
    Restricted,
    Suspended,
    Terminating,
    Terminated,
}

impl SupplierLifecycleStatus {
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Terminated)
    }

    pub fn in_contract_window(self) -> bool {
        matches!(
            self,
            Self::Approved | Self::Active | Self::Restricted | Self::Suspended | Self::Terminating
        )
    }

    pub fn in_offboarding(self) -> bool {
        matches!(self, Self::Terminating | Self::Terminated)
    }

    pub fn can_transition(from: Self, to: Self) -> bool {
        use SupplierLifecycleStatus::*;
        matches!(
            (from, to),
            (Unspecified, Candidate | UnderReview)
                | (Candidate, UnderReview | Terminated)
                | (UnderReview, Candidate | Approved | Terminated)
                | (Approved, Active | UnderReview | Terminated)
                | (Active, Restricted | Suspended | Terminating | UnderReview)
                | (Restricted, Active | Suspended | Terminating)
                | (Suspended, Active | Restricted | Terminating)
                | (Terminating, Terminated | Active)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SupplierMonitoringStatus {
    #[default]
    Unspecified,
    NotMonitored,
    Healthy,
    Degraded,
    Incident,
}

impl SupplierMonitoringStatus {
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SupplierAccessGrantStatus {
    #[default]
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierAccessGrant {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<AssetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_id: Option<IdentityId>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub privileged: bool,
    #[serde(default)]
    pub status: SupplierAccessGrantStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SupplierAccess {
    #[serde(default, skip_serializing_if = "is_false")]
    pub privileged: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub data_access: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<SupplierAccessGrant>,
}

impl SupplierAccess {
    pub fn is_unspecified(&self) -> bool {
        !self.privileged && !self.data_access && self.grants.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplierReviewKind {
    Onboarding,
    Periodic,
    AdHoc,
    Offboarding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplierReviewSource {
    Questionnaire,
    ManualReview,
    AutomatedPosture,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierReview {
    pub id: SupplierReviewId,
    pub kind: SupplierReviewKind,
    pub performed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    pub source: SupplierReviewSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierRiskAssessment {
    pub performed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_risk_ids: Vec<RiskId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplierRequirementSource {
    Contract,
    Policy,
    Obligation,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierSecurityRequirement {
    pub id: SupplierRequirementId,
    pub title: String,
    pub source: SupplierRequirementSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obligation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplierApprovalDecision {
    Approved,
    Rejected,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierApproval {
    pub principal: PrincipalRef,
    pub at: DateTime<Utc>,
    pub decision: SupplierApprovalDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierReassessmentCadence {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplierIssueStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierIssue {
    pub id: SupplierIssueId,
    pub title: String,
    pub status: SupplierIssueStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VendorEventKind {
    Created,
    FieldsRevised,
    StatusTransition {
        from: SupplierLifecycleStatus,
        to: SupplierLifecycleStatus,
    },
    ReviewRecorded,
    AssessmentExpired {
        #[serde(rename = "asOf")]
        as_of: DateTime<Utc>,
    },
    ApprovalRecorded,
    AccessRevoked,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: VendorEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VendorTransitionError {
    #[error("illegal lifecycle transition from {from:?} to {to:?}")]
    Illegal {
        from: SupplierLifecycleStatus,
        to: SupplierLifecycleStatus,
    },
    #[error("supplier approval is required before Approved; evidence does not imply approval")]
    ApprovalRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vendor {
    pub id: VendorId,
    pub name: String,
    #[serde(
        default,
        skip_serializing_if = "SupplierClassification::is_unspecified"
    )]
    pub classification: SupplierClassification,
    #[serde(default, skip_serializing_if = "SupplierCriticality::is_unspecified")]
    pub criticality: SupplierCriticality,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supplied_service_ids: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processing_activity_ids: Vec<ProcessingActivityId>,
    #[serde(default, skip_serializing_if = "SupplierAccess::is_unspecified")]
    pub access: SupplierAccess,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    #[serde(
        default,
        skip_serializing_if = "SupplierLifecycleStatus::is_unspecified"
    )]
    pub status: SupplierLifecycleStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onboarding_review: Option<SupplierReview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviews: Vec<SupplierReview>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_requirements: Vec<SupplierSecurityRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_assessment: Option<SupplierRiskAssessment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<SupplierApproval>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contract_document_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligation_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reassessment_cadence: Option<SupplierReassessmentCadence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_review: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "SupplierMonitoringStatus::is_unspecified"
    )]
    pub monitoring_status: SupplierMonitoringStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<SupplierIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exception_ids: Vec<ExceptionId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_ids: Vec<RiskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default = "default_version", skip_serializing_if = "version_is_one")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<VendorEvent>,
}

impl Vendor {
    pub fn new(id: VendorId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            classification: SupplierClassification::Unspecified,
            criticality: SupplierCriticality::Unspecified,
            supplied_service_ids: Vec::new(),
            processing_activity_ids: Vec::new(),
            access: SupplierAccess::default(),
            owner: None,
            status: SupplierLifecycleStatus::Unspecified,
            onboarding_review: None,
            reviews: Vec::new(),
            security_requirements: Vec::new(),
            risk_assessment: None,
            approval: None,
            contract_document_refs: Vec::new(),
            obligation_ids: Vec::new(),
            reassessment_cadence: None,
            next_review: None,
            monitoring_status: SupplierMonitoringStatus::Unspecified,
            issues: Vec::new(),
            exception_ids: Vec::new(),
            risk_ids: Vec::new(),
            control_ids: Vec::new(),
            evidence_refs: Vec::new(),
            version: 1,
            history: Vec::new(),
        }
    }

    pub fn with_criticality(mut self, criticality: SupplierCriticality) -> Self {
        self.criticality = criticality;
        self
    }

    pub fn with_owner(mut self, owner: PrincipalRef) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_services(mut self, supplied_service_ids: Vec<AssetId>) -> Self {
        self.supplied_service_ids = supplied_service_ids;
        self
    }

    pub fn with_access(mut self, access: SupplierAccess) -> Self {
        self.access = access;
        self
    }

    pub fn with_review(mut self, review: SupplierReview) -> Self {
        self.reviews.push(review);
        self
    }

    pub fn with_requirement(mut self, requirement: SupplierSecurityRequirement) -> Self {
        self.security_requirements.push(requirement);
        self
    }

    pub fn with_risk(mut self, risk_id: RiskId) -> Self {
        self.risk_ids.push(risk_id);
        self
    }

    pub fn with_exception(mut self, exception_id: ExceptionId) -> Self {
        self.exception_ids.push(exception_id);
        self
    }

    pub fn has_privileged_access(&self) -> bool {
        self.access.privileged || self.access.grants.iter().any(|grant| grant.privileged)
    }

    pub fn has_lingering_access(&self) -> bool {
        if !self.status.in_offboarding() {
            return false;
        }
        if self
            .access
            .grants
            .iter()
            .any(|grant| grant.status == SupplierAccessGrantStatus::Active)
        {
            return true;
        }
        self.access.privileged || self.access.data_access
    }

    pub fn review_current(&self, as_of: DateTime<Utc>) -> bool {
        if self.next_review.is_some_and(|next| next >= as_of) {
            return true;
        }
        self.iter_reviews()
            .any(|review| review.valid_until.is_some_and(|until| until >= as_of))
    }

    pub fn requires_current_security_review(&self) -> bool {
        if !self.status.in_contract_window() {
            return false;
        }
        matches!(
            self.effective_review_tier(),
            SupplierCriticality::Medium | SupplierCriticality::High | SupplierCriticality::Critical
        )
    }

    pub fn requires_contract_security_requirement(&self) -> bool {
        if !self.status.in_contract_window() {
            return false;
        }
        if self.has_privileged_access() || self.classification.is_processor() {
            return true;
        }
        matches!(
            self.criticality,
            SupplierCriticality::High | SupplierCriticality::Critical
        )
    }

    pub fn has_contract_security_requirement(&self) -> bool {
        self.security_requirements.iter().any(|req| {
            req.source == SupplierRequirementSource::Contract
                && (req
                    .document_ref
                    .as_ref()
                    .is_some_and(|r| !r.trim().is_empty())
                    || req
                        .obligation_id
                        .as_ref()
                        .is_some_and(|r| !r.trim().is_empty())
                    || self
                        .contract_document_refs
                        .iter()
                        .any(|r| !r.trim().is_empty()))
        })
    }

    pub fn effective_review_tier(&self) -> SupplierCriticality {
        if self.has_privileged_access() {
            match self.criticality {
                SupplierCriticality::Unspecified => SupplierCriticality::Unspecified,
                SupplierCriticality::Low | SupplierCriticality::Medium => SupplierCriticality::High,
                other => other,
            }
        } else {
            self.criticality
        }
    }

    pub fn transition(
        &mut self,
        to: SupplierLifecycleStatus,
    ) -> Result<&mut Self, VendorTransitionError> {
        if to == SupplierLifecycleStatus::Approved && !self.has_approved_decision() {
            return Err(VendorTransitionError::ApprovalRequired);
        }
        if !SupplierLifecycleStatus::can_transition(self.status, to) {
            return Err(VendorTransitionError::Illegal {
                from: self.status,
                to,
            });
        }
        let from = self.status;
        self.status = to;
        self.bump_version();
        let kind = if to == SupplierLifecycleStatus::Terminated {
            VendorEventKind::Terminated
        } else {
            VendorEventKind::StatusTransition { from, to }
        };
        self.history.push(VendorEvent {
            version: self.version,
            at: Utc::now(),
            principal: None,
            kind,
        });
        Ok(self)
    }

    pub fn record_review(&mut self, review: SupplierReview) -> &mut Self {
        if matches!(review.kind, SupplierReviewKind::Onboarding) {
            self.onboarding_review = Some(review.clone());
        }
        self.reviews.push(review);
        self.bump_version();
        self.history.push(VendorEvent {
            version: self.version,
            at: Utc::now(),
            principal: None,
            kind: VendorEventKind::ReviewRecorded,
        });
        self
    }

    pub fn approve(&mut self, approval: SupplierApproval) -> &mut Self {
        self.approval = Some(approval);
        self.bump_version();
        self.history.push(VendorEvent {
            version: self.version,
            at: Utc::now(),
            principal: None,
            kind: VendorEventKind::ApprovalRecorded,
        });
        self
    }

    pub fn attach_evidence(&mut self, evidence_ref: impl Into<String>) -> &mut Self {
        self.evidence_refs.push(evidence_ref.into());
        self
    }

    pub fn record_assessment_expired(&mut self, as_of: DateTime<Utc>) -> &mut Self {
        self.bump_version();
        self.history.push(VendorEvent {
            version: self.version,
            at: as_of,
            principal: None,
            kind: VendorEventKind::AssessmentExpired { as_of },
        });
        self
    }

    pub fn revise(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self.bump_version();
        self.history.push(VendorEvent {
            version: self.version,
            at: Utc::now(),
            principal: None,
            kind: VendorEventKind::FieldsRevised,
        });
        self
    }

    fn has_approved_decision(&self) -> bool {
        matches!(
            self.approval.as_ref().map(|a| a.decision),
            Some(SupplierApprovalDecision::Approved)
        )
    }

    fn iter_reviews(&self) -> impl Iterator<Item = &SupplierReview> {
        self.onboarding_review.iter().chain(self.reviews.iter())
    }

    fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1).max(2);
    }
}
