//! Organizational implementation state. Not control effectiveness.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ASSURANCE_IR_SCHEMA, AssetId, ControlId, ControlImplementationId, EvidenceRequirementId,
    ExceptionId, IdentityId, RiskId, SubjectSelector,
};

/// Organizational implementation state. Never an effectiveness verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ImplementationStatus {
    #[default]
    NotImplemented,
    Planned,
    PartiallyImplemented,
    Implemented,
    NotApplicable,
    Retired,
    /// Present in the org but switched off / not operating as a **state record**.
    #[serde(alias = "disabled")]
    Ineffective,
    /// State has not been determined.
    Unknown,
}

impl ImplementationStatus {
    /// Coverage-active statuses count toward overlap / population coverage.
    pub fn is_coverage_active(self) -> bool {
        matches!(
            self,
            Self::Planned
                | Self::PartiallyImplemented
                | Self::Implemented
                | Self::Ineffective
                | Self::Unknown
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrincipalRef {
    Identity(IdentityId),
    Team(String),
    Role(String),
}

/// Review interval in whole days. Not an ISO Annex A text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCadence {
    pub interval_days: u32,
}

impl ReviewCadence {
    pub fn new(interval_days: u32) -> Self {
        Self { interval_days }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentKind {
    Policy,
    Standard,
    Procedure,
    Record,
    Plan,
    Runbook,
}

/// Opaque policy/document pointer until the controlled-documents registry lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRef {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<DocumentKind>,
}

impl DocumentRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: None,
            kind: None,
        }
    }
}

/// How the organization operates the implementation. Not a collector id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImplementationAutomation {
    Manual,
    Automated,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlImplementation {
    schema_version: String,
    id: ControlImplementationId,
    control_id: ControlId,
    status: ImplementationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    implemented_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effective_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    applies_to: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    asset_ids: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    review_cadence: Option<ReviewCadence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_review: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence_expectations: Vec<EvidenceRequirementId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    document_refs: Vec<DocumentRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    compensating_controls: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exception_ids: Vec<ExceptionId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    risk_ids: Vec<RiskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    treatment_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    automation: Option<ImplementationAutomation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    supersedes: Option<ControlImplementationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    superseded_by: Option<ControlImplementationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    superseded_at: Option<DateTime<Utc>>,
}

impl ControlImplementation {
    pub fn new(id: ControlImplementationId, control_id: ControlId) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            control_id,
            status: ImplementationStatus::NotImplemented,
            owner: None,
            description: None,
            implemented_at: None,
            effective_from: None,
            applies_to: Vec::new(),
            asset_ids: Vec::new(),
            review_cadence: None,
            next_review: None,
            evidence_expectations: Vec::new(),
            document_refs: Vec::new(),
            compensating_controls: Vec::new(),
            exception_ids: Vec::new(),
            risk_ids: Vec::new(),
            treatment_ids: Vec::new(),
            automation: None,
            supersedes: None,
            superseded_by: None,
            superseded_at: None,
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn id(&self) -> &ControlImplementationId {
        &self.id
    }

    pub fn control_id(&self) -> &ControlId {
        &self.control_id
    }

    pub fn status(&self) -> ImplementationStatus {
        self.status
    }

    pub fn owner(&self) -> Option<&PrincipalRef> {
        self.owner.as_ref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn implemented_at(&self) -> Option<DateTime<Utc>> {
        self.implemented_at
    }

    pub fn effective_from(&self) -> Option<DateTime<Utc>> {
        self.effective_from
    }

    pub fn applies_to(&self) -> &[SubjectSelector] {
        &self.applies_to
    }

    pub fn asset_ids(&self) -> &[AssetId] {
        &self.asset_ids
    }

    pub fn review_cadence(&self) -> Option<ReviewCadence> {
        self.review_cadence
    }

    pub fn next_review(&self) -> Option<DateTime<Utc>> {
        self.next_review
    }

    pub fn evidence_expectations(&self) -> &[EvidenceRequirementId] {
        &self.evidence_expectations
    }

    pub fn document_refs(&self) -> &[DocumentRef] {
        &self.document_refs
    }

    pub fn compensating_controls(&self) -> &[ControlId] {
        &self.compensating_controls
    }

    pub fn exception_ids(&self) -> &[ExceptionId] {
        &self.exception_ids
    }

    pub fn risk_ids(&self) -> &[RiskId] {
        &self.risk_ids
    }

    pub fn treatment_ids(&self) -> &[String] {
        &self.treatment_ids
    }

    pub fn automation(&self) -> Option<ImplementationAutomation> {
        self.automation
    }

    pub fn supersedes(&self) -> Option<&ControlImplementationId> {
        self.supersedes.as_ref()
    }

    pub fn superseded_by(&self) -> Option<&ControlImplementationId> {
        self.superseded_by.as_ref()
    }

    pub fn superseded_at(&self) -> Option<DateTime<Utc>> {
        self.superseded_at
    }

    /// Retired / N/A / not-implemented / superseded snapshots are not coverage-active.
    pub fn is_coverage_active(&self) -> bool {
        self.superseded_by.is_none() && self.status.is_coverage_active()
    }

    pub fn with_status(mut self, status: ImplementationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_owner(mut self, owner: PrincipalRef) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_applies_to(mut self, selectors: impl Into<Vec<SubjectSelector>>) -> Self {
        self.applies_to = selectors.into();
        self
    }

    pub fn with_asset(mut self, asset: AssetId) -> Self {
        self.asset_ids.push(asset);
        self
    }

    pub fn with_implemented_at(mut self, at: DateTime<Utc>) -> Self {
        self.implemented_at = Some(at);
        self
    }

    pub fn with_effective_from(mut self, at: DateTime<Utc>) -> Self {
        self.effective_from = Some(at);
        self
    }

    pub fn with_review(mut self, cadence: ReviewCadence, next_review: DateTime<Utc>) -> Self {
        self.review_cadence = Some(cadence);
        self.next_review = Some(next_review);
        self
    }

    pub fn with_evidence_expectation(mut self, requirement: EvidenceRequirementId) -> Self {
        self.evidence_expectations.push(requirement);
        self
    }

    pub fn with_document(mut self, document: DocumentRef) -> Self {
        self.document_refs.push(document);
        self
    }

    pub fn with_treatment(mut self, treatment_id: impl Into<String>) -> Self {
        self.treatment_ids.push(treatment_id.into());
        self
    }

    pub fn with_automation(mut self, automation: ImplementationAutomation) -> Self {
        self.automation = Some(automation);
        self
    }

    pub fn with_compensating_control(mut self, control: ControlId) -> Self {
        self.compensating_controls.push(control);
        self
    }

    pub fn with_risk(mut self, risk: RiskId) -> Self {
        self.risk_ids.push(risk);
        self
    }

    pub fn with_exception(mut self, exception: ExceptionId) -> Self {
        self.exception_ids.push(exception);
        self
    }

    pub fn superseding(mut self, prior_id: ControlImplementationId) -> Self {
        self.supersedes = Some(prior_id);
        self
    }
}
