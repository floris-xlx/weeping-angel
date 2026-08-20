//! Operational Statement of Applicability projection. Not a certification document.
//!
//! Applicability, implementation, and effectiveness are independent dimensions.
//! Missing implementation and insufficient evidence MUST NOT become not applicable.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Exception, ExceptionStatus, ImplementationStatus, Mapping,
    MappingCompleteness, MappingRelation, PrincipalRef, typed_canonical_digest,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_framework::load_framework_pack;

use crate::applicability::{ApplicabilityDecision, ApplicabilitySnapshot};
use crate::lineage::{LINEAGE_SNAPSHOT_SCHEMA, StatementOfApplicabilitySnapshot};
use crate::snapshot::{SnapshotDiff, SoaDiffCause};

pub const OPERATIONAL_SOA_INPUT_SCHEMA: &str = "weeping-angel/operational-soa-input/v1";
pub const RISK_TREATMENT_REF_SCHEMA: &str = "weeping-angel/risk-treatment-ref/v1";
pub const RISK_REGISTER_REF_SCHEMA: &str = "weeping-angel/risk-register-ref/v1";

const READINESS_DISCLAIMER: &str =
    "This Statement of Applicability projection is a readiness aid and is not certification.";

/// Generic three-state applicability consumed by SoA projection.
/// `Unresolved` is the projection alias of Kleene `ManualDeterminationRequired`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Applicability {
    Applicable,
    NotApplicable,
    Unresolved,
}

impl Applicability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applicable => "applicable",
            Self::NotApplicable => "notApplicable",
            Self::Unresolved => "unresolved",
        }
    }

    fn from_pack(raw: &str, fallback_bool: Option<bool>) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "applicable" | "true" => Self::Applicable,
            "notapplicable" | "not-applicable" | "not_applicable" | "false" => Self::NotApplicable,
            "unresolved" | "manual" | "manualdeterminationrequired" => Self::Unresolved,
            "" => match fallback_bool {
                Some(true) => Self::Applicable,
                Some(false) => Self::NotApplicable,
                None => Self::Unresolved,
            },
            _ => Self::Unresolved,
        }
    }

    fn from_kleene(decision: ApplicabilityDecision) -> Self {
        match decision {
            ApplicabilityDecision::Applicable => Self::Applicable,
            ApplicabilityDecision::NotApplicable => Self::NotApplicable,
            ApplicabilityDecision::ManualDeterminationRequired => Self::Unresolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementOfApplicability {
    pub framework: String,
    pub framework_version: String,
    pub entries: Vec<SoaEntry>,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SoaApproval {
    #[serde(default)]
    pub principal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub review_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoaEntry {
    pub reference: String,
    pub applicability: Applicability,
    pub applicable: bool,
    pub applicability_rationale: String,
    pub implementation_state: String,
    pub automated_effectiveness: Option<Effectiveness>,
    pub manual_review_state: String,
    pub evidence: Vec<String>,
    pub exceptions: Vec<String>,
    pub mapped_controls: Vec<String>,
    pub notes: String,
    #[serde(default)]
    pub linked_risks: Vec<String>,
    #[serde(default)]
    pub treatment_rationale: String,
    #[serde(default)]
    pub treatment_refs: Vec<String>,
    #[serde(default)]
    pub canonical_controls: Vec<String>,
    #[serde(default)]
    pub implementation_refs: Vec<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub implementation_status: String,
    #[serde(default)]
    pub effectiveness_status: Option<Effectiveness>,
    #[serde(default)]
    pub evidence_lineage: Vec<String>,
    #[serde(default)]
    pub readiness_gaps: Vec<String>,
    #[serde(default)]
    pub review_state: String,
    #[serde(default)]
    pub approval: SoaApproval,
    #[serde(default)]
    pub inclusion_reasons: Vec<String>,
    #[serde(default)]
    pub exclusion_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskTreatmentRef {
    #[serde(default = "default_treatment_schema")]
    pub schema: String,
    pub id: String,
    pub risk_id: String,
    pub strategy: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement_id: Option<String>,
}

fn default_treatment_schema() -> String {
    RISK_TREATMENT_REF_SCHEMA.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRegisterRef {
    #[serde(default = "default_register_schema")]
    pub schema: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_id: Option<String>,
}

fn default_register_schema() -> String {
    RISK_REGISTER_REF_SCHEMA.into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalSoaInput {
    #[serde(default = "default_input_schema")]
    pub schema: String,
    pub framework: String,
    pub version: String,
    pub assessment: AssessmentDefinition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kleene: Option<ApplicabilitySnapshot>,
    #[serde(default)]
    pub results: Vec<ControlTestResult>,
    #[serde(default)]
    pub treatments: Vec<RiskTreatmentRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_register: Option<RiskRegisterRef>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub require_kleene: bool,
    #[serde(default)]
    pub treatment_driven_requirement_ids: Vec<String>,
}

fn default_input_schema() -> String {
    OPERATIONAL_SOA_INPUT_SCHEMA.into()
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OperationalSoaError {
    #[error("missing risk treatment ref {id}")]
    MissingRiskTreatment { id: String },
    #[error("missing risk-register digest")]
    MissingRiskRegister,
    #[error("missing input digest for {kind}")]
    MissingInputDigest { kind: String },
    #[error("missing applicability snapshot")]
    MissingApplicabilitySnapshot,
}

pub fn project_soa_from_snapshot(
    snapshot: &StatementOfApplicabilitySnapshot,
) -> StatementOfApplicability {
    snapshot.soa.clone()
}

/// Seal a SoA document. Digest is a function of the pinned payload + pack digest,
/// not of later live pack-file bytes.
pub fn pin_soa_snapshot(
    soa: StatementOfApplicability,
    framework_pack_digest: impl Into<String>,
) -> StatementOfApplicabilitySnapshot {
    let framework_pack_digest = framework_pack_digest.into();
    let digest = snapshot_digest(&soa, &framework_pack_digest);
    StatementOfApplicabilitySnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        digest,
        framework_pack_digest,
        soa,
    }
}

fn snapshot_digest(soa: &StatementOfApplicability, framework_pack_digest: &str) -> String {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body<'a> {
        schema: &'static str,
        framework_pack_digest: &'a str,
        soa: &'a StatementOfApplicability,
    }
    typed_canonical_digest(
        "soa-snapshot",
        &Body {
            schema: LINEAGE_SNAPSHOT_SCHEMA,
            framework_pack_digest,
            soa,
        },
    )
    .unwrap_or_default()
}

pub fn project_soa(framework: &str, version: &str) -> StatementOfApplicability {
    // Pinned StatementOfApplicabilitySnapshot + project_soa_from_snapshot is
    // the historical reconstruction path. Live project_soa is a convenience
    // over pack default/structural flags + an empty operational graph (notImplemented).
    // Digest identity lives on the snapshot, not live disk.
    let input = OperationalSoaInput {
        schema: OPERATIONAL_SOA_INPUT_SCHEMA.into(),
        framework: framework.into(),
        version: version.into(),
        assessment: AssessmentDefinition::new(AssessmentId::new(format!(
            "soa-live:{framework}:{version}"
        ))),
        kleene: None,
        results: Vec::new(),
        treatments: Vec::new(),
        risk_register: None,
        as_of: Utc::now(),
        require_kleene: false,
        treatment_driven_requirement_ids: Vec::new(),
    };
    project_operational_soa(&input).unwrap_or_else(|_| StatementOfApplicability {
        framework: framework.into(),
        framework_version: version.into(),
        entries: Vec::new(),
        disclaimer: READINESS_DISCLAIMER.into(),
    })
}

pub fn project_operational_soa(
    input: &OperationalSoaInput,
) -> Result<StatementOfApplicability, OperationalSoaError> {
    validate_operational_input(input)?;

    let pack = load_framework_pack(&input.framework, &input.version).ok();
    let pack_rows = load_pack_soa_rows(&input.framework, &input.version);
    let mut mappings = pack
        .as_ref()
        .map(|p| p.mappings.clone())
        .unwrap_or_default();
    for mapping in &input.assessment.mappings {
        let exists = mappings.iter().any(|m| {
            m.from_requirement() == mapping.from_requirement()
                && m.to_control() == mapping.to_control()
        });
        if !exists {
            mappings.push(mapping.clone());
        }
    }

    let mut references: Vec<(String, String, Applicability, String)> = Vec::new();
    for row in &pack_rows {
        references.push((
            row.reference.clone(),
            row.requirement.clone(),
            row.applicability,
            row.rationale.clone(),
        ));
    }
    for req in &input.assessment.requirements {
        let id = req.id().as_str().to_string();
        if !references
            .iter()
            .any(|(_, requirement, _, _)| requirement == &id)
        {
            references.push((id.clone(), id, Applicability::Unresolved, String::new()));
        }
    }

    let mut entries = Vec::new();
    for (reference, requirement, pack_default, pack_rationale) in references {
        entries.push(project_row(
            input,
            &reference,
            &requirement,
            pack_default,
            &pack_rationale,
            &mappings,
        ));
    }

    Ok(StatementOfApplicability {
        framework: input.framework.clone(),
        framework_version: input.version.clone(),
        entries,
        disclaimer: READINESS_DISCLAIMER.into(),
    })
}

fn validate_operational_input(input: &OperationalSoaInput) -> Result<(), OperationalSoaError> {
    if input.require_kleene && input.kleene.is_none() {
        return Err(OperationalSoaError::MissingApplicabilitySnapshot);
    }
    for treatment in &input.treatments {
        if treatment.digest.trim().is_empty() {
            return Err(OperationalSoaError::MissingInputDigest {
                kind: "risk-treatment".into(),
            });
        }
    }
    if let Some(register) = &input.risk_register
        && register.digest.trim().is_empty()
    {
        return Err(OperationalSoaError::MissingInputDigest {
            kind: "risk-register".into(),
        });
    }
    if !input.treatments.is_empty()
        && input
            .risk_register
            .as_ref()
            .is_none_or(|r| r.digest.trim().is_empty())
    {
        return Err(OperationalSoaError::MissingRiskRegister);
    }
    if !input.treatment_driven_requirement_ids.is_empty() && input.treatments.is_empty() {
        return Err(OperationalSoaError::MissingRiskTreatment {
            id: input
                .treatment_driven_requirement_ids
                .first()
                .cloned()
                .unwrap_or_default(),
        });
    }
    for requirement_id in &input.treatment_driven_requirement_ids {
        let cited = input.treatments.iter().any(|t| {
            t.requirement_id.as_deref() == Some(requirement_id.as_str()) || t.id == *requirement_id
        });
        if !cited && !input.treatments.is_empty() {
            let missing = input
                .treatments
                .iter()
                .find(|t| t.requirement_id.as_deref() == Some(requirement_id.as_str()));
            if missing.is_none() {
                return Err(OperationalSoaError::MissingRiskTreatment {
                    id: requirement_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn project_row(
    input: &OperationalSoaInput,
    reference: &str,
    requirement: &str,
    pack_default: Applicability,
    pack_rationale: &str,
    mappings: &[Mapping],
) -> SoaEntry {
    let kleene = kleene_for(input.kleene.as_ref(), requirement, reference);
    let (applicability, mut rationale, mut inclusion, exclusion) = match kleene {
        Some((decision, kleene_rationale)) => {
            let applicability = Applicability::from_kleene(decision);
            let rationale = if kleene_rationale.is_empty() {
                pack_rationale.to_string()
            } else {
                kleene_rationale
            };
            let (inc, exc) = match applicability {
                Applicability::Applicable => (vec!["kleene applicable".into()], Vec::new()),
                Applicability::NotApplicable => (Vec::new(), vec!["kleene not applicable".into()]),
                Applicability::Unresolved => (
                    vec!["kleene manual determination required".into()],
                    Vec::new(),
                ),
            };
            (applicability, rationale, inc, exc)
        }
        None => {
            let (inc, exc) = match pack_default {
                Applicability::Applicable => (
                    vec!["pack structural default applicable".into()],
                    Vec::new(),
                ),
                Applicability::NotApplicable => (
                    Vec::new(),
                    vec!["pack structural default not-applicable".into()],
                ),
                Applicability::Unresolved => (
                    vec!["pack structural default unresolved".into()],
                    Vec::new(),
                ),
            };
            (pack_default, pack_rationale.to_string(), inc, exc)
        }
    };

    let row_mappings: Vec<&Mapping> = mappings
        .iter()
        .filter(|m| {
            let from = m.from_requirement().as_str();
            from == requirement || from.eq_ignore_ascii_case(reference)
        })
        .collect();
    let mapped_controls: Vec<String> = row_mappings
        .iter()
        .map(|m| m.to_control().as_str().to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut readiness_gaps = Vec::new();
    let partial = !row_mappings.is_empty() && row_mappings.iter().all(|m| mapping_is_partial(m));
    if partial {
        readiness_gaps.push("partialCanonicalMapping".into());
    }

    let impls: Vec<_> = input
        .assessment
        .implementations
        .iter()
        .filter(|imp| {
            mapped_controls
                .iter()
                .any(|c| c == imp.control_id().as_str())
        })
        .collect();

    // Missing implementation is first-class notImplemented. It MUST NOT become
    // not applicable. IR ImplementationStatus::NotApplicable does not set SoA
    // applicability — applicability stays Kleene/pack.
    let (implementation_status, implementation_refs) = if impls.is_empty() {
        (
            status_label(ImplementationStatus::NotImplemented),
            Vec::new(),
        )
    } else {
        let status = impls
            .iter()
            .map(|imp| imp.status())
            .find(|s| *s != ImplementationStatus::NotApplicable)
            .unwrap_or(ImplementationStatus::NotImplemented);
        let refs = impls
            .iter()
            .map(|imp| imp.id().as_str().to_string())
            .collect();
        (status_label(status), refs)
    };

    let owner = None;

    let mut linked_risks: BTreeSet<String> = BTreeSet::new();
    for imp in &impls {
        for risk in imp.risk_ids() {
            linked_risks.insert(risk.as_str().to_string());
        }
    }
    for risk in &input.assessment.risks {
        if linked_risks.contains(risk.id.as_str()) {
            linked_risks.insert(format!("{}:{}", risk.id.as_str(), risk.title));
        }
    }

    let mut exception_ids: BTreeSet<String> = BTreeSet::new();
    for imp in &impls {
        for exception in imp.exception_ids() {
            exception_ids.insert(exception.as_str().to_string());
        }
    }
    for exception in &input.assessment.exceptions {
        if exception
            .control_id
            .as_ref()
            .is_some_and(|c| mapped_controls.iter().any(|m| m == c.as_str()))
        {
            exception_ids.insert(exception.id.as_str().to_string());
        }
    }
    let row_exceptions: Vec<&Exception> = input
        .assessment
        .exceptions
        .iter()
        .filter(|ex| exception_ids.contains(ex.id.as_str()))
        .collect();

    let (approval, na_gaps, review_state) =
        na_governance(applicability, &rationale, &row_exceptions, input.as_of);
    readiness_gaps.extend(na_gaps);

    let matching_results: Vec<&ControlTestResult> = input
        .results
        .iter()
        .filter(|r| mapped_controls.iter().any(|c| c == r.control_id.as_str()))
        .collect();
    let effectiveness_status = combine_effectiveness(&matching_results);
    // InsufficientEvidence is first-class Effectiveness. It MUST NOT become
    // not applicable and is never a NotApplicable justification.
    if matches!(
        effectiveness_status,
        Some(Effectiveness::InsufficientEvidence) | Some(Effectiveness::StaleEvidence)
    ) {
        readiness_gaps.push("insufficientEvidence".into());
    }
    if matches!(effectiveness_status, Some(Effectiveness::Effective))
        && impls
            .iter()
            .any(|imp| imp.status() == ImplementationStatus::Implemented)
    {
        // Implemented + Effectiveness::Effective is representable; Implemented
        // does not imply Effective by itself.
    }

    let mut evidence_lineage = Vec::new();
    for result in &matching_results {
        evidence_lineage.extend(result.evidence_refs.iter().cloned());
        for missing in &result.missing_evidence {
            evidence_lineage.push(format!("missing:{missing}"));
            if !readiness_gaps.iter().any(|g| g == "insufficientEvidence") {
                readiness_gaps.push("insufficientEvidence".into());
            }
        }
    }

    let mut treatment_refs = Vec::new();
    let mut treatment_rationale = String::new();
    let treatment_driven = input
        .treatment_driven_requirement_ids
        .iter()
        .any(|id| id == requirement || id == reference);
    if treatment_driven {
        for treatment in &input.treatments {
            if treatment.requirement_id.as_deref() == Some(requirement)
                || treatment.requirement_id.as_deref() == Some(reference)
                || treatment.id == requirement
            {
                treatment_refs.push(treatment.id.clone());
                if treatment_rationale.is_empty() {
                    treatment_rationale =
                        format!("treatment {} strategy {}", treatment.id, treatment.strategy);
                }
            }
        }
        inclusion.push(format!("treatment-driven:{requirement}"));
    }

    let notes = if partial {
        "partialCanonicalMapping".into()
    } else {
        String::new()
    };

    let review_state = if review_state.is_empty() {
        match applicability {
            Applicability::Unresolved => "manual determination required".into(),
            Applicability::NotApplicable => "not applicable".into(),
            Applicability::Applicable => "pending".into(),
        }
    } else {
        review_state
    };

    if applicability == Applicability::Applicable && rationale.is_empty() {
        rationale = pack_rationale.to_string();
    }

    SoaEntry {
        reference: reference.into(),
        applicability,
        applicable: matches!(applicability, Applicability::Applicable),
        applicability_rationale: rationale,
        implementation_state: implementation_status.clone(),
        automated_effectiveness: effectiveness_status,
        manual_review_state: review_state.clone(),
        evidence: matching_results
            .iter()
            .flat_map(|r| r.evidence_refs.iter().cloned())
            .collect(),
        exceptions: exception_ids.into_iter().collect(),
        mapped_controls: mapped_controls.clone(),
        notes,
        linked_risks: linked_risks.into_iter().collect(),
        treatment_rationale,
        treatment_refs,
        canonical_controls: mapped_controls,
        implementation_refs,
        owner,
        implementation_status,
        effectiveness_status,
        evidence_lineage,
        readiness_gaps,
        review_state,
        approval,
        inclusion_reasons: inclusion,
        exclusion_reasons: exclusion,
    }
}

fn kleene_for(
    snapshot: Option<&ApplicabilitySnapshot>,
    requirement: &str,
    reference: &str,
) -> Option<(ApplicabilityDecision, String)> {
    let snapshot = snapshot?;
    snapshot
        .requirement_decisions
        .iter()
        .chain(snapshot.control_decisions.iter())
        .find(|item| item.id == requirement || item.id.eq_ignore_ascii_case(reference))
        .map(|item| {
            let rationale = item
                .rationale
                .iter()
                .map(|r| r.message.clone())
                .collect::<Vec<_>>()
                .join("; ");
            (item.decision, rationale)
        })
}

fn mapping_is_partial(mapping: &Mapping) -> bool {
    matches!(
        mapping.relation(),
        MappingRelation::PartiallySatisfies
            | MappingRelation::Supports
            | MappingRelation::Related
            | MappingRelation::EvidenceFor
            | MappingRelation::SubsetOf
    ) || matches!(
        mapping.completeness(),
        MappingCompleteness::Partial | MappingCompleteness::Related
    )
}

fn status_label(status: ImplementationStatus) -> String {
    match status {
        ImplementationStatus::NotImplemented => "notImplemented".into(),
        ImplementationStatus::Planned => "planned".into(),
        ImplementationStatus::PartiallyImplemented => "partiallyImplemented".into(),
        ImplementationStatus::Implemented => "implemented".into(),
        ImplementationStatus::NotApplicable => "notApplicable".into(),
        ImplementationStatus::Retired => "retired".into(),
        ImplementationStatus::Ineffective => "ineffective".into(),
        ImplementationStatus::Unknown => "unknown".into(),
    }
    // IR ImplementationStatus::NotApplicable does not set SoA applicability.
}

fn combine_effectiveness(results: &[&ControlTestResult]) -> Option<Effectiveness> {
    if results.is_empty() {
        return None;
    }
    let states: Vec<Effectiveness> = results.iter().map(|r| r.effectiveness).collect();
    if states.contains(&Effectiveness::Ineffective) {
        return Some(Effectiveness::Ineffective);
    }
    if states.contains(&Effectiveness::StaleEvidence) {
        return Some(Effectiveness::StaleEvidence);
    }
    if states.contains(&Effectiveness::InsufficientEvidence) {
        return Some(Effectiveness::InsufficientEvidence);
    }
    if states.contains(&Effectiveness::PartiallyEffective) {
        return Some(Effectiveness::PartiallyEffective);
    }
    if states.contains(&Effectiveness::Effective) {
        return Some(Effectiveness::Effective);
    }
    states.first().copied()
}

fn na_governance(
    applicability: Applicability,
    rationale: &str,
    exceptions: &[&Exception],
    as_of: DateTime<Utc>,
) -> (SoaApproval, Vec<String>, String) {
    if applicability != Applicability::NotApplicable {
        return (SoaApproval::default(), Vec::new(), String::new());
    }

    let mut gaps = Vec::new();
    let approved = exceptions.iter().find(|ex| {
        matches!(ex.status, ExceptionStatus::Approved)
            && ex.approved_by.is_some()
            && !exception_expired(ex, as_of)
    });
    let expired = exceptions
        .iter()
        .any(|ex| matches!(ex.status, ExceptionStatus::Expired) || exception_expired(ex, as_of));

    if rationale.trim().is_empty() {
        gaps.push("missingNaRationale".into());
    }

    if let Some(exception) = approved {
        let principal = exception.approved_by.as_ref().map(principal_label);
        let approval = SoaApproval {
            principal,
            approved_at: None,
            expires_at: exception.expires_at,
            review_state: "approved".into(),
        };
        return (approval, gaps, "approved".into());
    }

    if expired {
        gaps.push("expiredNaApproval".into());
        let principal = exceptions
            .iter()
            .find_map(|ex| ex.approved_by.as_ref().map(principal_label));
        let expires_at = exceptions.iter().find_map(|ex| ex.expires_at);
        return (
            SoaApproval {
                principal,
                approved_at: None,
                expires_at,
                review_state: "expired".into(),
            },
            gaps,
            "expired".into(),
        );
    }

    gaps.push("missingNaApproval".into());
    (
        SoaApproval {
            principal: None,
            approved_at: None,
            expires_at: None,
            review_state: "readiness gap".into(),
        },
        gaps,
        "readiness gap".into(),
    )
}

fn exception_expired(exception: &Exception, as_of: DateTime<Utc>) -> bool {
    matches!(exception.status, ExceptionStatus::Expired)
        || exception.expires_at.is_some_and(|expires| expires <= as_of)
}

fn principal_label(principal: &PrincipalRef) -> String {
    match principal {
        PrincipalRef::Identity(id) => id.as_str().to_string(),
        PrincipalRef::Team(name) | PrincipalRef::Role(name) => name.clone(),
    }
}

struct PackSoaRow {
    reference: String,
    requirement: String,
    applicability: Applicability,
    rationale: String,
}

fn load_pack_soa_rows(framework: &str, version: &str) -> Vec<PackSoaRow> {
    // Prefer the framework pack loader (fail-closed PackError path) over a
    // second TOML parse. Missing/unknown packs still yield an empty SoA row set.
    let Ok(pack) = weeping_angel_framework::load_framework_pack(framework, version) else {
        return Vec::new();
    };
    pack.applicability
        .iter()
        .filter(|row| row.soa.unwrap_or(true))
        .map(|row| PackSoaRow {
            reference: row.reference.clone(),
            requirement: row.requirement.clone(),
            applicability: Applicability::from_pack(&row.applicability, row.applicable),
            rationale: row.applicability_rationale.clone(),
        })
        .collect()
}

/// Classify material SoA snapshot changes with the six-cause taxonomy.
pub fn diff_soa_snapshots(
    previous: &StatementOfApplicabilitySnapshot,
    next: &StatementOfApplicabilitySnapshot,
) -> SnapshotDiff {
    let mut diff = SnapshotDiff::default();
    if previous.framework_pack_digest != next.framework_pack_digest {
        diff.framework_pack_digest_changed = true;
    }

    let prev: BTreeMap<&str, &SoaEntry> = previous
        .soa
        .entries
        .iter()
        .map(|e| (e.reference.as_str(), e))
        .collect();
    let nxt: BTreeMap<&str, &SoaEntry> = next
        .soa
        .entries
        .iter()
        .map(|e| (e.reference.as_str(), e))
        .collect();

    let mut keys: BTreeSet<&str> = BTreeSet::new();
    keys.extend(prev.keys().copied());
    keys.extend(nxt.keys().copied());

    for key in keys {
        let before = prev.get(key).copied();
        let after = nxt.get(key).copied();
        match (before, after) {
            (None, Some(next_row)) => {
                if next_row.applicability == Applicability::Applicable {
                    diff.requirement_became_applicable.push(key.into());
                    push_cause(&mut diff, SoaDiffCause::ApplicabilityChange);
                }
            }
            (Some(prev_row), None) => {
                if prev_row.applicability == Applicability::Applicable {
                    diff.requirement_became_not_applicable.push(key.into());
                    push_cause(&mut diff, SoaDiffCause::ApplicabilityChange);
                }
            }
            (Some(prev_row), Some(next_row)) => {
                if prev_row.applicability != next_row.applicability {
                    push_cause(&mut diff, SoaDiffCause::ApplicabilityChange);
                    match next_row.applicability {
                        Applicability::Applicable => {
                            diff.requirement_became_applicable.push(key.into());
                        }
                        Applicability::NotApplicable => {
                            diff.requirement_became_not_applicable.push(key.into());
                        }
                        Applicability::Unresolved => {}
                    }
                }
                if prev_row.implementation_status != next_row.implementation_status
                    || prev_row.implementation_state != next_row.implementation_state
                    || prev_row.implementation_refs != next_row.implementation_refs
                {
                    push_cause(&mut diff, SoaDiffCause::ImplementationChange);
                }
                if is_effectiveness_regression(
                    prev_row.effectiveness_status,
                    next_row.effectiveness_status,
                ) {
                    push_cause(&mut diff, SoaDiffCause::EffectivenessRegression);
                    diff.control_became_ineffective
                        .push(format!("{key} effectiveness regression"));
                }
                if prev_row.mapped_controls != next_row.mapped_controls
                    || prev_row.canonical_controls != next_row.canonical_controls
                {
                    push_cause(&mut diff, SoaDiffCause::MappingChange);
                }
                if prev_row.treatment_refs != next_row.treatment_refs
                    || prev_row.treatment_rationale != next_row.treatment_rationale
                {
                    push_cause(&mut diff, SoaDiffCause::TreatmentChange);
                }
                let expired = next_row
                    .readiness_gaps
                    .iter()
                    .any(|g| g == "expiredNaApproval")
                    && !prev_row
                        .readiness_gaps
                        .iter()
                        .any(|g| g == "expiredNaApproval")
                    || next_row.exceptions != prev_row.exceptions
                        && next_row
                            .readiness_gaps
                            .iter()
                            .any(|g| g == "expiredNaApproval")
                    || (next_row.approval.expires_at.is_some()
                        && next_row.approval.review_state == "expired"
                        && prev_row.approval.review_state != "expired");
                if expired {
                    push_cause(&mut diff, SoaDiffCause::ExceptionExpiry);
                    diff.expired_exceptions.push(key.into());
                }
            }
            (None, None) => {}
        }
    }
    diff
}

fn push_cause(diff: &mut SnapshotDiff, cause: SoaDiffCause) {
    if !diff.soa_causes.contains(&cause) {
        diff.soa_causes.push(cause);
    }
}

fn is_effectiveness_regression(
    previous: Option<Effectiveness>,
    next: Option<Effectiveness>,
) -> bool {
    matches!(previous, Some(Effectiveness::Effective))
        && matches!(
            next,
            Some(Effectiveness::Ineffective)
                | Some(Effectiveness::PartiallyEffective)
                | Some(Effectiveness::InsufficientEvidence)
                | Some(Effectiveness::StaleEvidence)
        )
}
