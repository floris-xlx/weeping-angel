//! Durable ISMS context IR. Definition, not a point-in-time assessment.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::id::{
    AssetId, BusinessUnitId, IdentityId, InterestedPartyId, IsmsContextId, IssueId, ObjectiveId,
    ObligationId, OrganizationId, RiskMethodologyId, ScopeId, VendorId,
};
use crate::implementation::PrincipalRef;
use crate::validation::IrValidationError;
use crate::{ASSURANCE_IR_SCHEMA, AssessmentDefinition, ValidateIr};

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

/// Operational lifecycle of a durable [`IsmsContext`] definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum IsmsLifecycleStatus {
    #[default]
    Draft,
    Active,
    UnderReview,
    Retired,
    Superseded,
}

/// Internal vs external context input. Not a finding and not a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IssueKind {
    Internal,
    External,
}

/// Who an obligation answers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InterestedPartyKind {
    Customer,
    Regulator,
    Employee,
    Supplier,
    Insurer,
    Internal,
    Other,
}

/// Calendar unit for governance cadence intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CadenceUnit {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Whole-count interval. `count == 0` is impossible and fails validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CadenceInterval {
    pub count: u32,
    pub unit: CadenceUnit,
}

/// How often management review, internal audit, and risk assessment repeat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceCadence {
    pub management_review: CadenceInterval,
    pub internal_audit: CadenceInterval,
    pub risk_assessment: CadenceInterval,
}

/// Named organizational unit under a single ISMS organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessUnit {
    pub id: BusinessUnitId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BusinessUnitId>,
}

/// Legal / management-system entity. Not an IAM [`crate::Identity`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: OrganizationId,
    pub legal_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub business_units: Vec<BusinessUnit>,
    pub scope_id: ScopeId,
}

/// Named management-system boundary handle. Not a scope-resolution result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementSystemScope {
    pub id: ScopeId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Context input issue (internal or external).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextIssue {
    pub id: IssueId,
    pub kind: IssueKind,
    pub title: String,
    pub description: String,
}

/// Party the management system answers to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestedParty {
    pub id: InterestedPartyId,
    pub name: String,
    pub kind: InterestedPartyKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligation_ids: Vec<ObligationId>,
}

/// Graph record for a duty owed to an interested party. Not a mapping engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Obligation {
    pub id: ObligationId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub interested_party_id: InterestedPartyId,
}

/// Declared security objective. Not a point-in-time status projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityObjective {
    pub id: ObjectiveId,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
}

/// Durable management-system definition. Root object for operational ISMS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsmsContext {
    pub schema_version: String,
    pub id: IsmsContextId,
    pub organization: Organization,
    pub scope: ManagementSystemScope,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<ContextIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interested_parties: Vec<InterestedParty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<Obligation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objectives: Vec<SecurityObjective>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_methodology_id: Option<RiskMethodologyId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub asset_ids: BTreeSet<AssetId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub vendor_ids: BTreeSet<VendorId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub identity_ids: BTreeSet<IdentityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cadence: Option<GovernanceCadence>,
    pub lifecycle: IsmsLifecycleStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<IsmsContextId>,
}

impl IsmsContext {
    pub fn new(
        id: IsmsContextId,
        organization: Organization,
        scope: ManagementSystemScope,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            organization,
            scope,
            issues: Vec::new(),
            interested_parties: Vec::new(),
            obligations: Vec::new(),
            objectives: Vec::new(),
            risk_methodology_id: None,
            asset_ids: BTreeSet::new(),
            vendor_ids: BTreeSet::new(),
            identity_ids: BTreeSet::new(),
            cadence: None,
            lifecycle: IsmsLifecycleStatus::Draft,
            superseded_by: None,
        }
    }
}

impl ValidateIr for IsmsContext {
    fn validate(&self) -> Result<(), IrValidationError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(IrValidationError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {}",
                self.schema_version
            )));
        }

        if is_blank(&self.organization.legal_name) {
            return Err(IrValidationError::Message(
                "empty organization legalName".into(),
            ));
        }
        if is_blank(&self.scope.title) {
            return Err(IrValidationError::Message("empty scope title".into()));
        }
        if self.organization.scope_id != self.scope.id {
            return Err(IrValidationError::Message(format!(
                "dangling organization scopeId {} (expected {})",
                self.organization.scope_id, self.scope.id
            )));
        }

        let mut unit_ids = BTreeSet::new();
        for unit in &self.organization.business_units {
            if !unit_ids.insert(unit.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate business unit id {}",
                    unit.id
                )));
            }
            if is_blank(&unit.name) {
                return Err(IrValidationError::Message(format!(
                    "empty business unit name {}",
                    unit.id
                )));
            }
        }
        for unit in &self.organization.business_units {
            if let Some(parent) = &unit.parent_id
                && (parent == &unit.id || !unit_ids.contains(parent.as_str()))
            {
                return Err(IrValidationError::Message(format!(
                    "dangling business unit parentId {} on {}",
                    parent, unit.id
                )));
            }
        }

        let mut issue_ids = BTreeSet::new();
        for issue in &self.issues {
            if !issue_ids.insert(issue.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate issue id {}",
                    issue.id
                )));
            }
            if is_blank(&issue.title) {
                return Err(IrValidationError::Message(format!(
                    "empty issue title {}",
                    issue.id
                )));
            }
        }

        let mut party_ids = BTreeSet::new();
        for party in &self.interested_parties {
            if !party_ids.insert(party.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate interested party id {}",
                    party.id
                )));
            }
            if is_blank(&party.name) {
                return Err(IrValidationError::Message(format!(
                    "empty interested party name {}",
                    party.id
                )));
            }
        }

        let mut obligation_ids = BTreeSet::new();
        for obligation in &self.obligations {
            if !obligation_ids.insert(obligation.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate obligation id {}",
                    obligation.id
                )));
            }
            if is_blank(&obligation.title) {
                return Err(IrValidationError::Message(format!(
                    "empty obligation title {}",
                    obligation.id
                )));
            }
        }

        for party in &self.interested_parties {
            for obligation_id in &party.obligation_ids {
                if !obligation_ids.contains(obligation_id.as_str()) {
                    return Err(IrValidationError::Message(format!(
                        "dangling obligation id {obligation_id} on interested party {}",
                        party.id
                    )));
                }
            }
        }

        for obligation in &self.obligations {
            if !party_ids.contains(obligation.interested_party_id.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling interestedPartyId {} on obligation {}",
                    obligation.interested_party_id, obligation.id
                )));
            }
            let listed = self.interested_parties.iter().any(|party| {
                party.id == obligation.interested_party_id
                    && party.obligation_ids.iter().any(|id| id == &obligation.id)
            });
            if !listed {
                return Err(IrValidationError::Message(format!(
                    "dangling obligation {} is not listed on interested party {}",
                    obligation.id, obligation.interested_party_id
                )));
            }
        }

        let mut objective_ids = BTreeSet::new();
        for objective in &self.objectives {
            if !objective_ids.insert(objective.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate objective id {}",
                    objective.id
                )));
            }
            if is_blank(&objective.title) {
                return Err(IrValidationError::Message(format!(
                    "empty objective title {}",
                    objective.id
                )));
            }
        }

        if let Some(cadence) = &self.cadence {
            for (label, interval) in [
                ("managementReview", cadence.management_review),
                ("internalAudit", cadence.internal_audit),
                ("riskAssessment", cadence.risk_assessment),
            ] {
                if interval.count == 0 {
                    return Err(IrValidationError::Message(format!(
                        "impossible cadence count 0 on {label}"
                    )));
                }
            }
        }

        match self.lifecycle {
            IsmsLifecycleStatus::Draft => {
                if self.superseded_by.is_some() {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: supersededBy is present on draft".into(),
                    ));
                }
            }
            IsmsLifecycleStatus::Active | IsmsLifecycleStatus::UnderReview => {
                if self.superseded_by.is_some() {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: supersededBy is present when status is not superseded"
                            .into(),
                    ));
                }
                if self.risk_methodology_id.is_none() {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: active/underReview requires riskMethodologyId"
                            .into(),
                    ));
                }
                if self.cadence.is_none() {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: active/underReview requires cadence".into(),
                    ));
                }
            }
            IsmsLifecycleStatus::Retired => {
                if self.superseded_by.is_some() {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: supersededBy is present on retired".into(),
                    ));
                }
            }
            IsmsLifecycleStatus::Superseded => match &self.superseded_by {
                None => {
                    return Err(IrValidationError::Message(
                        "impossible lifecycle: superseded without supersededBy".into(),
                    ));
                }
                Some(successor) if successor == &self.id => {
                    return Err(IrValidationError::Message(
                        "duplicate lifecycle successor: supersededBy equals context id".into(),
                    ));
                }
                Some(_) => {}
            },
        }

        Ok(())
    }
}

/// Pair validator. Standalone assessments without a pointer remain valid.
pub fn validate_assessment_against_context(
    assessment: &AssessmentDefinition,
    context: &IsmsContext,
) -> Result<(), IrValidationError> {
    let Some(pointer) = &assessment.isms_context_id else {
        return Ok(());
    };
    if pointer != &context.id {
        return Err(IrValidationError::Message(format!(
            "dangling isms_context_id {pointer} (expected {})",
            context.id
        )));
    }

    let asset_ids: BTreeSet<_> = assessment
        .assets
        .iter()
        .map(|asset| asset.id.as_str().to_string())
        .collect();
    for id in &context.asset_ids {
        if !asset_ids.contains(id.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling context asset id {id} is absent from assessment inventory"
            )));
        }
    }

    let vendor_ids: BTreeSet<_> = assessment
        .vendors
        .iter()
        .map(|vendor| vendor.id.as_str().to_string())
        .collect();
    for id in &context.vendor_ids {
        if !vendor_ids.contains(id.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling context vendor id {id} is absent from assessment inventory"
            )));
        }
    }

    let identity_ids: BTreeSet<_> = assessment
        .identities
        .iter()
        .map(|identity| identity.id.as_str().to_string())
        .collect();
    for id in &context.identity_ids {
        if !identity_ids.contains(id.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling context identity id {id} is absent from assessment inventory"
            )));
        }
    }

    Ok(())
}

/// Deterministic definition explain. Not an assessment-result projection.
pub fn explain_isms_context(ctx: &IsmsContext) -> String {
    let bus = ctx
        .organization
        .business_units
        .iter()
        .map(|unit| format!("{} ({})", unit.id, unit.name))
        .collect::<Vec<_>>()
        .join(", ");
    let issues = ctx
        .issues
        .iter()
        .map(|issue| {
            let kind = match issue.kind {
                IssueKind::Internal => "internal",
                IssueKind::External => "external",
            };
            format!("issue:{kind}:{}", issue.id)
        })
        .collect::<Vec<_>>()
        .join(" ; ");
    let parties = ctx
        .interested_parties
        .iter()
        .map(|party| {
            let obligations = party
                .obligation_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            format!("{} ({}) -> {obligations}", party.id, party.name)
        })
        .collect::<Vec<_>>()
        .join(" ; ");
    let objectives = ctx
        .objectives
        .iter()
        .map(|objective| format!("{} ({})", objective.id, objective.title))
        .collect::<Vec<_>>()
        .join(", ");
    let methodology = ctx
        .risk_methodology_id
        .as_ref()
        .map(|id| id.as_str())
        .unwrap_or("none");
    let lifecycle = match ctx.lifecycle {
        IsmsLifecycleStatus::Draft => "draft",
        IsmsLifecycleStatus::Active => "active",
        IsmsLifecycleStatus::UnderReview => "underReview",
        IsmsLifecycleStatus::Retired => "retired",
        IsmsLifecycleStatus::Superseded => "superseded",
    };

    format!(
        "{} -> {} ({}) -> {}\n  {bus}\n  {issues}\n  {parties}\n  {objectives}\n  methodology:{methodology}\n  lifecycle:{lifecycle}",
        ctx.id, ctx.organization.id, ctx.organization.legal_name, ctx.scope.id
    )
}
