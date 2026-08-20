//! Structural IR validation. Compile receives already-valid documents.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::audit::{AuditInventory, validate_audit_inventories};
use crate::incident::IncidentGraph;
use crate::registry::overlap_report;
use crate::risk::{
    Risk, RiskEventKind, RiskStatus, evidence_ref_is_requirement,
    evidence_ref_is_well_formed_digest,
};
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssetKind, ControlImplementation,
    EvidenceCriticality, Exception, ExceptionStatus, ImplementationStatus, PrincipalRef,
    SubjectKind, SupplierCriticality, SupplierLifecycleStatus, SupplierRequirementSource, Vendor,
    VendorEventKind,
    extension::{extension_key_is_well_formed, extensions_override_canonical},
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IrValidationError {
    #[error("{0}")]
    Message(String),
}

pub trait ValidateIr {
    fn validate(&self) -> Result<(), IrValidationError>;
}

pub fn validate_assessment_ir(assessment: &AssessmentDefinition) -> Result<(), IrValidationError> {
    assessment.validate()
}

impl ValidateIr for AssessmentDefinition {
    fn validate(&self) -> Result<(), IrValidationError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(IrValidationError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {}",
                self.schema_version
            )));
        }

        let mut requirement_ids = BTreeSet::new();
        for req in &self.requirements {
            if !requirement_ids.insert(req.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate requirement id {}",
                    req.id()
                )));
            }
            validate_extensions(req.framework().id().as_str())?;
        }

        let mut control_ids = BTreeSet::new();
        for control in &self.controls {
            if !control_ids.insert(control.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate control id {}",
                    control.id()
                )));
            }
            for key in control.extensions().keys() {
                if !extension_key_is_well_formed(key) || extensions_override_canonical(key) {
                    return Err(IrValidationError::Message(format!(
                        "invalid extension namespace {key}"
                    )));
                }
            }
        }

        let mut evidence_ids = BTreeSet::new();
        for ev in &self.evidence_requirements {
            if !evidence_ids.insert(ev.id().as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate evidence requirement id {}",
                    ev.id()
                )));
            }
        }

        let asset_ids: BTreeSet<_> = self
            .assets
            .iter()
            .map(|a| a.id.as_str().to_string())
            .collect();
        let processing_ids: BTreeSet<_> = self
            .processing_activities
            .iter()
            .map(|p| p.id.as_str().to_string())
            .collect();
        let mut vendor_ids = BTreeSet::new();
        for vendor in &self.vendors {
            if !vendor_ids.insert(vendor.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate vendor id {}",
                    vendor.id
                )));
            }
        }
        let identity_ids: BTreeSet<_> = self
            .identities
            .iter()
            .map(|i| i.id.as_str().to_string())
            .collect();

        let mut risk_ids = BTreeSet::new();
        for risk in &self.risks {
            if !risk_ids.insert(risk.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate risk id {}",
                    risk.id
                )));
            }
        }
        for risk in &self.risks {
            validate_risk_record(
                risk,
                &asset_ids,
                &processing_ids,
                &vendor_ids,
                &control_ids,
                &evidence_ids,
                &identity_ids,
                &risk_ids,
            )?;
        }

        let exception_ids: BTreeSet<_> = self
            .exceptions
            .iter()
            .map(|e| e.id.as_str().to_string())
            .collect();

        for mapping in &self.mappings {
            if mapping.from_requirement().as_str() == mapping.to_control().as_str() {
                return Err(IrValidationError::Message(
                    "self mapping is not allowed".into(),
                ));
            }
            if !requirement_ids.contains(mapping.from_requirement().as_str())
                || !control_ids.contains(mapping.to_control().as_str())
            {
                return Err(IrValidationError::Message(format!(
                    "dangling mapping {} → {}",
                    mapping.from_requirement(),
                    mapping.to_control()
                )));
            }
            if let Some(req) = self
                .requirements
                .iter()
                .find(|r| r.id() == mapping.from_requirement())
                && !mapping.valid_for().contains(req.framework_version())
            {
                return Err(IrValidationError::Message(format!(
                    "mapping version constraint does not include {}",
                    req.framework_version()
                )));
            }
        }

        for test in &self.tests {
            if !control_ids.contains(test.control_id.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling test {} control {}",
                    test.id, test.control_id
                )));
            }
        }

        validate_control_implementations(
            self,
            &control_ids,
            &risk_ids,
            &exception_ids,
            &evidence_ids,
            &asset_ids,
            &identity_ids,
            &vendor_ids,
        )?;

        validate_incidents(self, &control_ids, &risk_ids)?;

        crate::risk_treatment::validate_treatment_inventory(self)
            .map_err(|err| IrValidationError::Message(err.to_string()))?;

        crate::remediation::validate_remediation_inventory(self, None)
            .map_err(|err| IrValidationError::Message(err.to_string()))?;

        validate_audit_inventories(AuditInventory {
            programs: &self.audit_programs,
            audits: &self.audits,
            findings: &self.audit_findings,
            control_ids: &control_ids,
            requirement_ids: &requirement_ids,
        })?;

        crate::capa::validate_capa_inventory(self)?;

        for activity in &self.processing_activities {
            for processor in &activity.processors {
                if !vendor_ids.contains(processor.as_str()) {
                    return Err(IrValidationError::Message(format!(
                        "dangling processor vendor {processor} on processing activity {}",
                        activity.id
                    )));
                }
            }
        }

        for vendor in &self.vendors {
            validate_vendor_record(
                vendor,
                &asset_ids,
                &processing_ids,
                &identity_ids,
                &risk_ids,
                &exception_ids,
                &control_ids,
            )?;
        }

        crate::continuity::validate_continuity_profiles(self)
            .map_err(IrValidationError::Message)?;

        validate_scope_exclusions(&self.scope)?;

        Ok(())
    }
}

fn validate_scope_exclusions(scope: &crate::AssessmentScope) -> Result<(), IrValidationError> {
    for (index, exclusion) in scope.exclusions.iter().enumerate() {
        if let Some(reason) = exclusion.governance_error() {
            return Err(IrValidationError::Message(format!(
                "silent or incomplete scope exclusion[{index}]: {reason}"
            )));
        }
    }
    Ok(())
}

/// Critical-tier suppliers in the assessment inventory.
pub fn critical_suppliers(assessment: &AssessmentDefinition) -> impl Iterator<Item = &Vendor> {
    assessment
        .vendors
        .iter()
        .filter(|vendor| vendor.criticality == SupplierCriticality::Critical)
}

/// Clocked supplier review check. Clockless [`AssessmentDefinition::validate`]
/// does not fail stale reviews; overdue required reviews fail here.
/// Expired exceptions bound to a vendor do not suppress the gap.
pub fn validate_supplier_reviews_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Result<(), IrValidationError> {
    for vendor in &assessment.vendors {
        if !vendor.requires_current_security_review() {
            continue;
        }
        if vendor.review_current(as_of) {
            continue;
        }
        if supplier_exception_in_force(assessment, vendor, as_of) {
            continue;
        }
        return Err(IrValidationError::Message(format!(
            "expired or missing supplier security review for vendor {}",
            vendor.id
        )));
    }
    Ok(())
}

fn supplier_exception_in_force(
    assessment: &AssessmentDefinition,
    vendor: &Vendor,
    as_of: DateTime<Utc>,
) -> bool {
    assessment.exceptions.iter().any(|exception| {
        exception_binds_vendor(exception, vendor)
            && exception.status == ExceptionStatus::Approved
            && exception.expires_at.is_none_or(|expires| expires >= as_of)
    })
}

fn exception_binds_vendor(exception: &Exception, vendor: &Vendor) -> bool {
    if vendor
        .exception_ids
        .iter()
        .any(|id| id.as_str() == exception.id.as_str())
    {
        return true;
    }
    exception.subjects.iter().any(|selector| {
        selector.kind == SubjectKind::Vendor && selector.ids.contains(vendor.id.as_str())
    })
}

fn validate_vendor_record(
    vendor: &Vendor,
    asset_ids: &BTreeSet<String>,
    processing_ids: &BTreeSet<String>,
    identity_ids: &BTreeSet<String>,
    risk_ids: &BTreeSet<String>,
    exception_ids: &BTreeSet<String>,
    control_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    if vendor.version < 1 {
        return Err(IrValidationError::Message(format!(
            "vendor {} version must be >= 1",
            vendor.id
        )));
    }

    if vendor.has_privileged_access() && vendor.criticality == SupplierCriticality::Unspecified {
        return Err(IrValidationError::Message(format!(
            "privileged access cannot remain unspecified criticality on vendor {}",
            vendor.id
        )));
    }

    if vendor.has_lingering_access() {
        return Err(IrValidationError::Message(format!(
            "lingering access after termination on vendor {}",
            vendor.id
        )));
    }

    if vendor.requires_contract_security_requirement()
        && !vendor.has_contract_security_requirement()
    {
        return Err(IrValidationError::Message(format!(
            "missing contract security requirement on vendor {}",
            vendor.id
        )));
    }

    if vendor.status == SupplierLifecycleStatus::Approved
        && !matches!(
            vendor.approval.as_ref().map(|a| a.decision),
            Some(crate::vendor::SupplierApprovalDecision::Approved)
        )
    {
        return Err(IrValidationError::Message(format!(
            "approved vendor {} is missing supplier approval; evidence does not imply approval",
            vendor.id
        )));
    }

    for service in &vendor.supplied_service_ids {
        if !asset_ids.contains(service.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling supplied service asset {service} on vendor {}",
                vendor.id
            )));
        }
    }
    for activity in &vendor.processing_activity_ids {
        if !processing_ids.contains(activity.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling processing activity {activity} on vendor {}",
                vendor.id
            )));
        }
    }
    for grant in &vendor.access.grants {
        if grant.asset_id.is_none() && grant.identity_id.is_none() {
            return Err(IrValidationError::Message(format!(
                "supplier access grant on vendor {} must cite an asset or identity",
                vendor.id
            )));
        }
        if let Some(asset) = &grant.asset_id
            && !asset_ids.contains(asset.as_str())
        {
            return Err(IrValidationError::Message(format!(
                "dangling grant asset {asset} on vendor {}",
                vendor.id
            )));
        }
        if let Some(identity) = &grant.identity_id
            && !identity_ids.contains(identity.as_str())
        {
            return Err(IrValidationError::Message(format!(
                "dangling grant identity {identity} on vendor {}",
                vendor.id
            )));
        }
    }

    validate_vendor_principal("owner", vendor, vendor.owner.as_ref(), identity_ids)?;
    if let Some(approval) = &vendor.approval {
        validate_vendor_principal(
            "approval principal",
            vendor,
            Some(&approval.principal),
            identity_ids,
        )?;
    }

    for risk in &vendor.risk_ids {
        if !risk_ids.contains(risk.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling risk reference {risk} on vendor {}",
                vendor.id
            )));
        }
    }
    if let Some(assessment) = &vendor.risk_assessment {
        for risk in &assessment.linked_risk_ids {
            if !risk_ids.contains(risk.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling risk reference {risk} on vendor {}",
                    vendor.id
                )));
            }
        }
    }
    for exception in &vendor.exception_ids {
        if !exception_ids.contains(exception.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling exception reference {exception} on vendor {}",
                vendor.id
            )));
        }
    }
    for control in &vendor.control_ids {
        if !control_ids.contains(control.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling control reference {control} on vendor {}",
                vendor.id
            )));
        }
    }
    for req in &vendor.security_requirements {
        if req.source == SupplierRequirementSource::Contract
            && req
                .document_ref
                .as_ref()
                .is_some_and(|r| r.trim().is_empty())
        {
            return Err(IrValidationError::Message(format!(
                "empty contract document ref on vendor {}",
                vendor.id
            )));
        }
        for control in &req.control_ids {
            if !control_ids.contains(control.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling control reference {control} on vendor {}",
                    vendor.id
                )));
            }
        }
    }
    for doc in &vendor.contract_document_refs {
        if doc.trim().is_empty() {
            return Err(IrValidationError::Message(format!(
                "empty contract document ref on vendor {}",
                vendor.id
            )));
        }
    }

    validate_vendor_history(vendor)?;
    Ok(())
}

fn validate_vendor_principal(
    field: &str,
    vendor: &Vendor,
    principal: Option<&PrincipalRef>,
    identity_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    match principal {
        Some(PrincipalRef::Identity(id)) if !identity_ids.contains(id.as_str()) => {
            Err(IrValidationError::Message(format!(
                "dangling identity {field} {id} on vendor {}",
                vendor.id
            )))
        }
        Some(PrincipalRef::Team(name) | PrincipalRef::Role(name)) if name.trim().is_empty() => Err(
            IrValidationError::Message(format!("empty {field} principal on vendor {}", vendor.id)),
        ),
        _ => Ok(()),
    }
}

fn validate_vendor_history(vendor: &Vendor) -> Result<(), IrValidationError> {
    for event in &vendor.history {
        if let VendorEventKind::StatusTransition { from, to } = event.kind
            && !SupplierLifecycleStatus::can_transition(from, to)
        {
            return Err(IrValidationError::Message(format!(
                "illegal lifecycle transition from {from:?} to {to:?} on vendor {}",
                vendor.id
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_control_implementations(
    assessment: &AssessmentDefinition,
    control_ids: &BTreeSet<String>,
    risk_ids: &BTreeSet<String>,
    exception_ids: &BTreeSet<String>,
    evidence_ids: &BTreeSet<String>,
    asset_ids: &BTreeSet<String>,
    identity_ids: &BTreeSet<String>,
    vendor_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    let mut implementation_ids = BTreeSet::new();
    for impln in &assessment.implementations {
        if !implementation_ids.insert(impln.id().as_str().to_string()) {
            return Err(IrValidationError::Message(format!(
                "duplicate implementation id {}",
                impln.id()
            )));
        }
    }

    for impln in &assessment.implementations {
        validate_implementation_row(
            assessment,
            impln,
            control_ids,
            risk_ids,
            exception_ids,
            evidence_ids,
            asset_ids,
            identity_ids,
            vendor_ids,
            &implementation_ids,
        )?;
    }

    detect_implementation_supersession_cycles(assessment)?;

    if let Some(hit) = overlap_report(assessment).into_iter().next() {
        return Err(IrValidationError::Message(format!(
            "implementation overlap {} and {} on control {}: {} ({})",
            hit.left_id, hit.right_id, hit.control_id, hit.reason, hit.selectors_or_assets
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_implementation_row(
    assessment: &AssessmentDefinition,
    impln: &ControlImplementation,
    control_ids: &BTreeSet<String>,
    risk_ids: &BTreeSet<String>,
    exception_ids: &BTreeSet<String>,
    evidence_ids: &BTreeSet<String>,
    asset_ids: &BTreeSet<String>,
    identity_ids: &BTreeSet<String>,
    vendor_ids: &BTreeSet<String>,
    implementation_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    if !control_ids.contains(impln.control_id().as_str()) {
        return Err(IrValidationError::Message(format!(
            "dangling implementation control {}",
            impln.control_id()
        )));
    }
    for control in impln.compensating_controls() {
        if !control_ids.contains(control.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling compensating control {control} on implementation {}",
                impln.id()
            )));
        }
    }
    for risk in impln.risk_ids() {
        if !risk_ids.contains(risk.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling risk reference {}",
                risk
            )));
        }
    }
    for exception in impln.exception_ids() {
        if !exception_ids.contains(exception.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling exception reference {}",
                exception
            )));
        }
    }
    for expectation in impln.evidence_expectations() {
        if !evidence_ids.contains(expectation.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling evidence expectation {} on implementation {}",
                expectation,
                impln.id()
            )));
        }
    }
    for asset in impln.asset_ids() {
        if !asset_ids.contains(asset.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling asset reference {} on implementation {}",
                asset,
                impln.id()
            )));
        }
    }
    if let Some(prior) = impln.supersedes()
        && !implementation_ids.contains(prior.as_str())
    {
        return Err(IrValidationError::Message(format!(
            "dangling supersedes reference {prior} on implementation {}",
            impln.id()
        )));
    }
    if let Some(successor) = impln.superseded_by()
        && !implementation_ids.contains(successor.as_str())
    {
        return Err(IrValidationError::Message(format!(
            "dangling supersededBy reference {successor} on implementation {}",
            impln.id()
        )));
    }
    if let Some(PrincipalRef::Identity(id)) = impln.owner()
        && !identity_ids.contains(id.as_str())
    {
        return Err(IrValidationError::Message(format!(
            "dangling identity owner {id} on implementation {}",
            impln.id()
        )));
    }

    if !assessment.risk_treatments.is_empty() {
        let known: BTreeSet<_> = assessment
            .risk_treatments
            .iter()
            .map(|t| t.id.as_str().to_string())
            .collect();
        for treatment in impln.treatment_ids() {
            if !known.contains(treatment) {
                return Err(IrValidationError::Message(format!(
                    "dangling treatment reference {treatment} on implementation {}",
                    impln.id()
                )));
            }
        }
    }

    for selector in impln.applies_to() {
        validate_implementation_selector(
            assessment,
            impln,
            selector,
            asset_ids,
            identity_ids,
            vendor_ids,
        )?;
    }

    validate_implementation_review(impln)?;
    validate_implementation_evidence(assessment, impln)?;
    Ok(())
}

fn validate_implementation_selector(
    assessment: &AssessmentDefinition,
    impln: &ControlImplementation,
    selector: &crate::SubjectSelector,
    asset_ids: &BTreeSet<String>,
    identity_ids: &BTreeSet<String>,
    vendor_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    for id in &selector.ids {
        let resolved = match selector.kind {
            SubjectKind::Identity
            | SubjectKind::User
            | SubjectKind::PrivilegedIdentity
            | SubjectKind::ServiceAccount => identity_ids.contains(id),
            SubjectKind::Vendor => vendor_ids.contains(id),
            SubjectKind::Organization => {
                assessment.scope.organizations.iter().any(|org| org == id)
                    || assessment.assets.iter().any(|asset| {
                        asset.id.as_str() == id && asset.kind == AssetKind::Organization
                    })
            }
            SubjectKind::Asset
            | SubjectKind::Repository
            | SubjectKind::Service
            | SubjectKind::Device
            | SubjectKind::Application
            | SubjectKind::Database
            | SubjectKind::CloudAccount
            | SubjectKind::CloudResource
            | SubjectKind::Endpoint
            | SubjectKind::DataStore
            | SubjectKind::Network
            | SubjectKind::Deployment => asset_ids.contains(id),
            _ => {
                identity_ids.contains(id)
                    || asset_ids.contains(id)
                    || vendor_ids.contains(id)
                    || assessment.scope.organizations.iter().any(|org| org == id)
            }
        };
        if !resolved {
            return Err(IrValidationError::Message(format!(
                "dangling subject reference {id} on implementation {}",
                impln.id()
            )));
        }
    }
    Ok(())
}

fn validate_implementation_review(impln: &ControlImplementation) -> Result<(), IrValidationError> {
    if let Some(cadence) = impln.review_cadence() {
        if cadence.interval_days < 1 {
            return Err(IrValidationError::Message(format!(
                "review cadence intervalDays must be >= 1 on implementation {}",
                impln.id()
            )));
        }
        if impln.next_review().is_none() {
            return Err(IrValidationError::Message(format!(
                "next_review required when review cadence is set on implementation {}",
                impln.id()
            )));
        }
    }
    if matches!(
        impln.status(),
        ImplementationStatus::Implemented | ImplementationStatus::PartiallyImplemented
    ) && impln.review_cadence().is_none()
        && impln.next_review().is_none()
    {
        return Err(IrValidationError::Message(format!(
            "missing review cadence/next_review on implementation {}",
            impln.id()
        )));
    }
    Ok(())
}

fn validate_implementation_evidence(
    assessment: &AssessmentDefinition,
    impln: &ControlImplementation,
) -> Result<(), IrValidationError> {
    let requires_expectations = matches!(
        impln.status(),
        ImplementationStatus::Implemented | ImplementationStatus::PartiallyImplemented
    );
    if requires_expectations && impln.evidence_expectations().is_empty() {
        return Err(IrValidationError::Message(format!(
            "missing evidence expectations on implementation {}",
            impln.id()
        )));
    }
    if !requires_expectations {
        return Ok(());
    }
    let Some(control) = assessment
        .controls
        .iter()
        .find(|control| control.id() == impln.control_id())
    else {
        return Ok(());
    };
    for req_id in control.evidence_requirements() {
        let criticality = assessment
            .evidence_requirements
            .iter()
            .find(|req| req.id() == req_id)
            .map(|req| req.criticality())
            .unwrap_or(EvidenceCriticality::Required);
        if criticality == EvidenceCriticality::Required
            && !impln.evidence_expectations().iter().any(|id| id == req_id)
        {
            return Err(IrValidationError::Message(format!(
                "missing required evidence ref {req_id} on implementation {}",
                impln.id()
            )));
        }
    }
    Ok(())
}

fn detect_implementation_supersession_cycles(
    assessment: &AssessmentDefinition,
) -> Result<(), IrValidationError> {
    for start in &assessment.implementations {
        let mut seen = BTreeSet::new();
        let mut current = Some(start.id().as_str().to_string());
        while let Some(id) = current {
            if !seen.insert(id.clone()) {
                return Err(IrValidationError::Message(format!(
                    "implementation supersession cycle involving {id}"
                )));
            }
            current = assessment
                .implementations
                .iter()
                .find(|row| row.id().as_str() == id)
                .and_then(|row| row.supersedes().map(|prior| prior.as_str().to_string()));
        }
    }
    Ok(())
}

fn validate_incidents(
    assessment: &AssessmentDefinition,
    control_ids: &BTreeSet<String>,
    risk_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    let mut incident_ids = BTreeSet::new();
    for incident in &assessment.incidents {
        if !incident_ids.insert(incident.id.as_str().to_string()) {
            return Err(IrValidationError::Message(format!(
                "duplicate incident id {}",
                incident.id
            )));
        }
    }

    let asset_ids: BTreeSet<_> = assessment
        .assets
        .iter()
        .map(|a| a.id.as_str().to_string())
        .collect();
    let identity_ids: BTreeSet<_> = assessment
        .identities
        .iter()
        .map(|i| i.id.as_str().to_string())
        .collect();
    let processing_activity_ids: BTreeSet<_> = assessment
        .processing_activities
        .iter()
        .map(|p| p.id.as_str().to_string())
        .collect();
    let test_ids: BTreeSet<_> = assessment
        .tests
        .iter()
        .map(|t| t.id.as_str().to_string())
        .collect();
    let remediation_ids: BTreeSet<_> = assessment
        .remediations
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    let graph = IncidentGraph {
        asset_ids: &asset_ids,
        identity_ids: &identity_ids,
        processing_activity_ids: &processing_activity_ids,
        risk_ids,
        control_ids,
        test_ids: &test_ids,
        remediation_ids: if remediation_ids.is_empty() {
            None
        } else {
            Some(&remediation_ids)
        },
    };
    for incident in &assessment.incidents {
        incident.validate_graph(&graph)?;
    }
    Ok(())
}

fn validate_extensions(_unused: &str) -> Result<(), IrValidationError> {
    Ok(())
}

/// Clocked review check. Clockless [`AssessmentDefinition::validate`] does not
/// fail Closed/Retired or unscheduled risks; overdue non-terminal risks fail.
pub fn validate_risk_reviews_at(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Result<(), IrValidationError> {
    for risk in &assessment.risks {
        if matches!(risk.status, RiskStatus::Closed | RiskStatus::Retired) {
            continue;
        }
        if risk.review_overdue(as_of) {
            return Err(IrValidationError::Message(format!(
                "overdue risk review {}",
                risk.id
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_risk_record(
    risk: &Risk,
    asset_ids: &BTreeSet<String>,
    processing_ids: &BTreeSet<String>,
    vendor_ids: &BTreeSet<String>,
    control_ids: &BTreeSet<String>,
    evidence_ids: &BTreeSet<String>,
    identity_ids: &BTreeSet<String>,
    risk_ids: &BTreeSet<String>,
) -> Result<(), IrValidationError> {
    if risk.version.get() < 1 {
        return Err(IrValidationError::Message(format!(
            "risk {} version must be >= 1",
            risk.id
        )));
    }

    for asset in &risk.asset_ids {
        if !asset_ids.contains(asset.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling asset reference {} on risk {}",
                asset, risk.id
            )));
        }
    }
    for process in &risk.processing_activity_ids {
        if !processing_ids.contains(process.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling processing activity reference {} on risk {}",
                process, risk.id
            )));
        }
    }
    for vendor in &risk.vendor_ids {
        if !vendor_ids.contains(vendor.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling vendor reference {} on risk {}",
                vendor, risk.id
            )));
        }
    }
    for control in &risk.control_ids {
        if !control_ids.contains(control.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling control reference {} on risk {}",
                control, risk.id
            )));
        }
    }
    for evidence in &risk.evidence_refs {
        if evidence_ids.contains(evidence.as_str()) {
            continue;
        }
        if evidence_ref_is_requirement(evidence) {
            return Err(IrValidationError::Message(format!(
                "dangling evidence requirement reference {evidence} on risk {}",
                risk.id
            )));
        }
        if !evidence_ref_is_well_formed_digest(evidence) {
            return Err(IrValidationError::Message(format!(
                "malformed evidence digest {evidence} on risk {}",
                risk.id
            )));
        }
    }
    match &risk.owner {
        Some(PrincipalRef::Identity(id)) if !identity_ids.contains(id.as_str()) => {
            return Err(IrValidationError::Message(format!(
                "dangling identity owner {} on risk {}",
                id, risk.id
            )));
        }
        Some(PrincipalRef::Team(name) | PrincipalRef::Role(name)) if name.trim().is_empty() => {
            return Err(IrValidationError::Message(format!(
                "empty owner principal on risk {}",
                risk.id
            )));
        }
        _ => {}
    }
    if let Some(prior) = &risk.supersedes
        && !risk_ids.contains(prior.as_str())
    {
        return Err(IrValidationError::Message(format!(
            "dangling supersedes reference {prior} on risk {}",
            risk.id
        )));
    }
    if let Some(successor) = &risk.superseded_by
        && !risk_ids.contains(successor.as_str())
    {
        return Err(IrValidationError::Message(format!(
            "dangling supersededBy reference {successor} on risk {}",
            risk.id
        )));
    }

    validate_risk_history(risk)?;
    validate_risk_scoring(risk)?;
    Ok(())
}

fn validate_risk_history(risk: &Risk) -> Result<(), IrValidationError> {
    for event in &risk.history {
        if let RiskEventKind::StatusTransition { from, to } = event.kind
            && !RiskStatus::can_transition(from, to)
        {
            return Err(IrValidationError::Message(format!(
                "illegal risk status transition from {from:?} to {to:?} on risk {}",
                risk.id
            )));
        }
    }
    Ok(())
}

fn validate_risk_scoring(risk: &Risk) -> Result<(), IrValidationError> {
    let has_score = risk.inherent_score.is_some();
    let has_rating = risk.inherent_rating.is_some();
    if !has_score && !has_rating {
        return Ok(());
    }
    let version = risk
        .methodology_version
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if version.is_none() {
        return Err(IrValidationError::Message(format!(
            "methodology version required for derived inherent fields on risk {}",
            risk.id
        )));
    }
    let has_raw = risk.likelihood.as_ref().is_some_and(|v| v.has_raw_level())
        && risk.impact.as_ref().is_some_and(|v| v.has_raw_level());
    if !has_raw {
        return Err(IrValidationError::Message(format!(
            "derived rating must not be the only authoring input on risk {}",
            risk.id
        )));
    }
    Ok(())
}
