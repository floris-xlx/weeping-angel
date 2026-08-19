//! Canonical assessment input. Compile targets stay in the framework crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::{Audit, AuditFinding, AuditProgram};
use crate::capa::{CorrectiveAction, Nonconformity};
use crate::id::validate_stable_id;
use crate::remediation::Remediation;
use crate::risk_treatment::RiskTreatmentDecision;
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentId, Asset, ContinuityResilienceProfile, Control,
    ControlImplementation, EvidenceRequirement, Exception, Identity, Incident, IsmsContextId,
    PlannedControlTest, PrincipalRef, ProcessingActivity, Requirement, Risk, SubjectSelector,
    Vendor,
};

use super::mapping::Mapping;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentRequests {
    pub statement_of_applicability: bool,
    pub control_applicability: bool,
    pub privacy_processing: bool,
    pub risk_treatment: bool,
    pub manual_attestation: bool,
    pub sampling: bool,
    pub audit_program: bool,
    pub nonconformities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScopeExclusion {
    #[serde(default)]
    pub subjects: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_by: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

impl ScopeExclusion {
    /// Complete governance record that may suppress, before the `as_of` clock.
    pub fn governance_is_complete(&self) -> bool {
        self.governance_error().is_none()
    }

    pub fn governance_error(&self) -> Option<&'static str> {
        if self.subjects.is_empty() {
            return Some("exclusion subjects must be non-empty");
        }
        if self
            .rationale
            .as_deref()
            .map(str::trim)
            .is_none_or(|r| r.is_empty())
        {
            return Some("silent exclusion: rationale is required");
        }
        match &self.owner {
            Some(PrincipalRef::Identity(id)) if !id.as_str().trim().is_empty() => {}
            Some(PrincipalRef::Team(name) | PrincipalRef::Role(name))
                if !name.trim().is_empty() => {}
            _ => return Some("exclusion owner/principal is required"),
        }
        let approval = self.approval_ref.as_deref().map(str::trim).unwrap_or("");
        if approval.is_empty() || validate_stable_id(approval).is_err() {
            return Some("exclusion approvalRef is required");
        }
        let Some(approved_at) = self.approved_at else {
            return Some("exclusion approvedAt is required");
        };
        if self.review_by.is_none() && self.expires_at.is_none() {
            return Some("exclusion reviewBy and/or expiresAt is required");
        }
        if self.review_by.is_some_and(|t| t < approved_at) {
            return Some("exclusion reviewBy precedes approvedAt");
        }
        if self.expires_at.is_some_and(|t| t < approved_at) {
            return Some("exclusion expiresAt precedes approvedAt");
        }
        if self.evidence_refs.is_empty() {
            return Some("exclusion evidenceRefs are required");
        }
        if self
            .evidence_refs
            .iter()
            .any(|r| r.trim().is_empty() || validate_stable_id(r.trim()).is_err())
        {
            return Some("exclusion evidenceRefs must be stable ids");
        }
        None
    }

    pub fn is_expired_at(&self, as_of: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|t| t <= as_of)
    }

    pub fn is_review_overdue_at(&self, as_of: DateTime<Utc>) -> bool {
        self.review_by.is_some_and(|t| t <= as_of)
    }

    /// Only complete, unexpired, non-overdue exclusions may suppress.
    pub fn is_active_at(&self, as_of: DateTime<Utc>) -> bool {
        self.governance_is_complete()
            && !self.is_expired_at(as_of)
            && !self.is_review_overdue_at(as_of)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentScope {
    #[serde(default)]
    pub organizations: Vec<String>,
    #[serde(default)]
    pub subjects: Vec<SubjectSelector>,
    #[serde(default)]
    pub exclusions: Vec<ScopeExclusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentDefinition {
    pub id: AssessmentId,
    pub schema_version: String,
    #[serde(default)]
    pub requirements: Vec<Requirement>,
    #[serde(default)]
    pub controls: Vec<Control>,
    #[serde(default)]
    pub mappings: Vec<Mapping>,
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirement>,
    #[serde(default)]
    pub tests: Vec<PlannedControlTest>,
    #[serde(default)]
    pub requests: AssessmentRequests,
    #[serde(default)]
    pub implementations: Vec<ControlImplementation>,
    #[serde(default)]
    pub scope: AssessmentScope,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub identities: Vec<Identity>,
    #[serde(default)]
    pub vendors: Vec<Vendor>,
    #[serde(default)]
    pub risks: Vec<Risk>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    #[serde(default)]
    pub processing_activities: Vec<ProcessingActivity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidents: Vec<Incident>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_treatments: Vec<RiskTreatmentDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isms_context_id: Option<IsmsContextId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediations: Vec<Remediation>,
    #[serde(
        default,
        rename = "continuityProfiles",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub continuity_profiles: Vec<ContinuityResilienceProfile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_programs: Vec<AuditProgram>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audits: Vec<Audit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_findings: Vec<AuditFinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nonconformities: Vec<Nonconformity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corrective_actions: Vec<CorrectiveAction>,
}

impl AssessmentDefinition {
    pub fn new(id: AssessmentId) -> Self {
        Self {
            id,
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            requirements: Vec::new(),
            controls: Vec::new(),
            mappings: Vec::new(),
            evidence_requirements: Vec::new(),
            tests: Vec::new(),
            requests: AssessmentRequests::default(),
            implementations: Vec::new(),
            scope: AssessmentScope::default(),
            assets: Vec::new(),
            identities: Vec::new(),
            vendors: Vec::new(),
            risks: Vec::new(),
            exceptions: Vec::new(),
            processing_activities: Vec::new(),
            incidents: Vec::new(),
            risk_treatments: Vec::new(),
            isms_context_id: None,
            remediations: Vec::new(),
            continuity_profiles: Vec::new(),
            audit_programs: Vec::new(),
            audits: Vec::new(),
            audit_findings: Vec::new(),
            nonconformities: Vec::new(),
            corrective_actions: Vec::new(),
        }
    }
}

/// Compatibility name used by the framework compiler.
pub type Assessment = AssessmentDefinition;
