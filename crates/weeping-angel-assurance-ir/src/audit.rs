//! Internal-audit IR: programs, engagements, samples, pins, findings, sign-off.
//!
//! Machine prepare output is advisory. Humans accept samples, record findings,
//! and sign conclusions. Signed documents are frozen.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::assessment::AssessmentScope;
use crate::digest::canonical_digest;
use crate::validation::IrValidationError;
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentId, AuditFindingId, AuditId, AuditProgramId, ControlId,
    FrameworkId, FrameworkVersion, PrincipalRef, RequirementId, RiskId,
};

fn schema_version_default() -> String {
    ASSURANCE_IR_SCHEMA.into()
}

fn proposal_kind() -> String {
    "proposal".into()
}

/// Half-open review window `[start, end)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl AuditPeriod {
    pub fn contains(&self, inner: &AuditPeriod) -> bool {
        inner.start >= self.start && inner.start < self.end && inner.end <= self.end
    }

    pub fn is_valid(&self) -> bool {
        self.start < self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditProgramStatus {
    Draft,
    Approved,
    InProgress,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditStatus {
    Draft,
    Prepared,
    InProgress,
    Concluded,
    Signed,
    Withdrawn,
}

impl AuditStatus {
    pub fn is_terminal_review(&self) -> bool {
        matches!(self, AuditStatus::Concluded | AuditStatus::Signed)
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, AuditStatus::Signed)
    }

    pub fn is_frozen(&self) -> bool {
        matches!(self, AuditStatus::Signed | AuditStatus::Withdrawn)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditConclusion {
    Conformant,
    Qualified,
    Nonconformant,
    NotConcluded,
}

impl AuditConclusion {
    pub fn is_signable(self) -> bool {
        !matches!(self, AuditConclusion::NotConcluded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SampleMethod {
    Census,
    Systematic,
    SeededRandom,
    Judgmental,
}

impl SampleMethod {
    pub fn requires_seed(self) -> bool {
        matches!(self, SampleMethod::Systematic | SampleMethod::SeededRandom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuditCriterion {
    Requirement {
        #[serde(rename = "requirementId")]
        requirement_id: RequirementId,
    },
    Control {
        #[serde(rename = "controlId")]
        control_id: ControlId,
    },
    Framework {
        #[serde(rename = "frameworkId")]
        framework_id: FrameworkId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        version: Option<FrameworkVersion>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditScheduleEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<AuditId>,
    pub window: AuditPeriod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_note: Option<String>,
}

pub type AuditSchedule = Vec<AuditScheduleEntry>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IndependenceConflict {
    AuditorOwnsControl {
        #[serde(rename = "controlId")]
        control_id: ControlId,
    },
    AuditorIsPrincipal,
    Other {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndependenceRecord {
    pub auditor: PrincipalRef,
    pub principal: PrincipalRef,
    pub declared_at: DateTime<Utc>,
    pub statement: String,
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub conflict_flags: Vec<IndependenceConflict>,
    pub accepted: bool,
}

impl IndependenceRecord {
    pub fn is_accepted_declaration(&self) -> bool {
        self.accepted && !self.statement.trim().is_empty() && !self.evidence_refs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSample {
    pub population_id: String,
    #[serde(rename = "populationDigest")]
    pub population_digest: String,
    pub method: SampleMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    pub size: u32,
    pub selected_ids: Vec<String>,
    pub accepted_by: PrincipalRef,
    pub accepted_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_digest: Option<String>,
    #[serde(rename = "sampleDigest")]
    pub sample_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSampleProposal {
    pub population_id: String,
    #[serde(rename = "populationDigest")]
    pub population_digest: String,
    pub method: SampleMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    pub size: u32,
    pub suggested_ids: Vec<String>,
    pub rationale: String,
    pub generated_at: DateTime<Utc>,
    pub proposal_digest: String,
    #[serde(default = "proposal_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvidencePin {
    pub evidence_snapshot_digest: String,
    pub envelope_digests: Vec<String>,
    #[serde(default)]
    pub collection_run_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_run_id: Option<AssessmentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_definition_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_pack_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_catalog_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "asOf")]
    pub as_of: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<AuditPeriod>,
    pub pinned_at: DateTime<Utc>,
    #[serde(rename = "pinnedBy")]
    pub pinned_by: PrincipalRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditProcedureStatus {
    Planned,
    Performed,
    NotPerformed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditProcedure {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub selected_control_ids: Vec<ControlId>,
    pub status: AuditProcedureStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditObservation {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub procedure_id: Option<String>,
    #[serde(default)]
    pub evidence_digests: Vec<String>,
    pub text: String,
    pub recorded_by: PrincipalRef,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditFindingKind {
    Observation,
    Finding,
    Nonconformity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditFindingSeverity {
    Minor,
    Major,
    Opportunity,
}

/// Opaque Prompt 22 seam. This slice does not own CAPA lifecycle.
pub type NonconformityRef = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditFinding {
    pub id: AuditFindingId,
    pub audit_id: AuditId,
    pub kind: AuditFindingKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<AuditFindingSeverity>,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub control_ids: Vec<ControlId>,
    #[serde(default)]
    pub requirement_ids: Vec<RequirementId>,
    #[serde(default)]
    pub evidence_digests: Vec<String>,
    pub created_by: PrincipalRef,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonconformity_id: Option<NonconformityRef>,
}

/// Human conclusion. No `Default` — nothing auto-signs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSignOff {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub signed_at: DateTime<Utc>,
    pub conclusion: AuditConclusion,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditHistoryEvent {
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: String,
    pub payload_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareDigest {
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditProgram {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: AuditProgramId,
    pub title: String,
    pub period: AuditPeriod,
    #[serde(default)]
    pub scope: AssessmentScope,
    #[serde(default)]
    pub objectives: Vec<String>,
    #[serde(default)]
    pub criteria: Vec<AuditCriterion>,
    #[serde(default)]
    pub schedule: AuditSchedule,
    pub principal: PrincipalRef,
    pub auditor: PrincipalRef,
    pub independence: IndependenceRecord,
    #[serde(default)]
    pub child_audit_ids: Vec<AuditId>,
    pub status: AuditProgramStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_from: Option<PrepareDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: AuditId,
    pub program_id: AuditProgramId,
    pub title: String,
    pub period: AuditPeriod,
    #[serde(default)]
    pub scope: AssessmentScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<AuditSample>,
    #[serde(default)]
    pub selected_controls: Vec<ControlId>,
    #[serde(default)]
    pub selected_requirements: Vec<RequirementId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_pin: Option<AuditEvidencePin>,
    #[serde(default)]
    pub procedures: Vec<AuditProcedure>,
    #[serde(default)]
    pub observations: Vec<AuditObservation>,
    #[serde(default)]
    pub findings: Vec<AuditFindingId>,
    #[serde(default)]
    pub nonconformity_refs: Vec<NonconformityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<AuditConclusion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_off: Option<AuditSignOff>,
    pub status: AuditStatus,
    #[serde(default)]
    pub history: Vec<AuditHistoryEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_proposal: Option<AuditSampleProposal>,
}

impl Audit {
    pub fn procedures_complete(&self) -> bool {
        self.procedures
            .iter()
            .all(|procedure| match procedure.status {
                AuditProcedureStatus::Planned => false,
                AuditProcedureStatus::Performed => true,
                AuditProcedureStatus::NotPerformed => procedure
                    .notes
                    .as_ref()
                    .is_some_and(|notes| !notes.trim().is_empty()),
            })
    }

    pub fn is_complete_for_conclusion(&self) -> bool {
        self.sample.is_some() && self.evidence_pin.is_some() && self.procedures_complete()
    }
}

pub fn population_digest(sorted_ids: &[String]) -> Result<String, IrValidationError> {
    digest_or_err(&sorted_ids)
}

pub fn compute_sample_digest(
    method: SampleMethod,
    seed: Option<&str>,
    population_digest: &str,
    selected_ids: &[String],
) -> Result<String, IrValidationError> {
    // sampleDigest covers method + seed + populationDigest + selectedIds.
    digest_or_err(&(method, seed.unwrap_or(""), population_digest, selected_ids))
}

pub fn compute_proposal_digest(
    proposal: &AuditSampleProposal,
) -> Result<String, IrValidationError> {
    digest_or_err(&(
        &proposal.population_id,
        &proposal.population_digest,
        proposal.method,
        proposal.seed.as_deref().unwrap_or(""),
        proposal.size,
        &proposal.suggested_ids,
        &proposal.rationale,
    ))
}

fn digest_or_err<T: Serialize>(value: &T) -> Result<String, IrValidationError> {
    canonical_digest(value)
        .map(|hex| format!("sha256:{hex}"))
        .map_err(|err| IrValidationError::Message(err.to_string()))
}

pub fn flag_independence_conflicts(
    auditor: &PrincipalRef,
    selected_control_owners: &[(ControlId, PrincipalRef)],
) -> Vec<IndependenceConflict> {
    let mut flags = Vec::new();
    for (control_id, owner) in selected_control_owners {
        if owner == auditor {
            flags.push(IndependenceConflict::AuditorOwnsControl {
                control_id: control_id.clone(),
            });
        }
    }
    flags
}

pub struct AuditInventory<'a> {
    pub programs: &'a [AuditProgram],
    pub audits: &'a [Audit],
    pub findings: &'a [AuditFinding],
    pub control_ids: &'a std::collections::BTreeSet<String>,
    pub requirement_ids: &'a std::collections::BTreeSet<String>,
}

pub fn validate_audit_inventories(inv: AuditInventory<'_>) -> Result<(), IrValidationError> {
    let mut program_ids = std::collections::BTreeSet::new();
    for program in inv.programs {
        if !program_ids.insert(program.id.as_str().to_string()) {
            return Err(IrValidationError::Message(format!(
                "duplicate audit program id {}",
                program.id
            )));
        }
        validate_program(program)?;
    }

    let mut audit_ids = std::collections::BTreeSet::new();
    for audit in inv.audits {
        if !audit_ids.insert(audit.id.as_str().to_string()) {
            return Err(IrValidationError::Message(format!(
                "duplicate audit id {}",
                audit.id
            )));
        }
        let program = inv
            .programs
            .iter()
            .find(|program| program.id == audit.program_id)
            .ok_or_else(|| {
                IrValidationError::Message(format!(
                    "dangling program id {} on audit {}",
                    audit.program_id, audit.id
                ))
            })?;
        validate_audit(audit, program, inv.control_ids, inv.requirement_ids)?;
    }

    let mut finding_ids = std::collections::BTreeSet::new();
    for finding in inv.findings {
        if !finding_ids.insert(finding.id.as_str().to_string()) {
            return Err(IrValidationError::Message(format!(
                "duplicate audit finding id {}",
                finding.id
            )));
        }
        if !audit_ids.contains(finding.audit_id.as_str()) && !inv.audits.is_empty() {
            return Err(IrValidationError::Message(format!(
                "dangling audit id {} on finding {}",
                finding.audit_id, finding.id
            )));
        }
        for control in &finding.control_ids {
            if !inv.control_ids.contains(control.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling finding control id {control}"
                )));
            }
        }
        for requirement in &finding.requirement_ids {
            if !inv.requirement_ids.contains(requirement.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling finding requirement id {requirement}"
                )));
            }
        }
        if let Some(audit) = inv.audits.iter().find(|row| row.id == finding.audit_id)
            && let Some(pin) = &audit.evidence_pin
        {
            for digest in &finding.evidence_digests {
                if !pin.envelope_digests.iter().any(|pinned| pinned == digest) {
                    return Err(IrValidationError::Message(format!(
                        "finding {} evidence {digest} is not in the audit pin",
                        finding.id
                    )));
                }
            }
        }
    }

    Ok(())
}

fn validate_program(program: &AuditProgram) -> Result<(), IrValidationError> {
    if !program.period.is_valid() {
        return Err(IrValidationError::Message(format!(
            "invalid audit program period {}",
            program.id
        )));
    }
    if matches!(
        program.status,
        AuditProgramStatus::Approved | AuditProgramStatus::InProgress | AuditProgramStatus::Closed
    ) && program.objectives.iter().all(|o| o.trim().is_empty())
    {
        return Err(IrValidationError::Message(format!(
            "approved program {} requires objectives",
            program.id
        )));
    }
    for entry in &program.schedule {
        if !program.period.contains(&entry.window) {
            return Err(IrValidationError::Message(format!(
                "schedule window outside program period {}",
                program.id
            )));
        }
    }
    Ok(())
}

fn validate_audit(
    audit: &Audit,
    program: &AuditProgram,
    control_ids: &std::collections::BTreeSet<String>,
    requirement_ids: &std::collections::BTreeSet<String>,
) -> Result<(), IrValidationError> {
    if !audit.period.is_valid() {
        return Err(IrValidationError::Message(format!(
            "invalid audit period {}",
            audit.id
        )));
    }
    if !program.period.contains(&audit.period) {
        return Err(IrValidationError::Message(format!(
            "audit period outside the program period for {}",
            audit.id
        )));
    }
    if !program.scope.organizations.is_empty() {
        for org in &audit.scope.organizations {
            if !program.scope.organizations.iter().any(|owned| owned == org) {
                return Err(IrValidationError::Message(format!(
                    "audit {} scope exceeds program organizations",
                    audit.id
                )));
            }
        }
    }
    for control in &audit.selected_controls {
        if !control_ids.contains(control.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling control id {control} on audit {}",
                audit.id
            )));
        }
    }
    for requirement in &audit.selected_requirements {
        if !requirement_ids.contains(requirement.as_str()) {
            return Err(IrValidationError::Message(format!(
                "dangling requirement id {requirement} on audit {}",
                audit.id
            )));
        }
    }
    for procedure in &audit.procedures {
        for control in &procedure.selected_control_ids {
            if !control_ids.contains(control.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling procedure control id {control} on audit {}",
                    audit.id
                )));
            }
        }
    }

    if audit.status.is_terminal_review() && !audit.is_complete_for_conclusion() {
        return Err(IrValidationError::Message(format!(
            "incomplete audit {} cannot conclude",
            audit.id
        )));
    }

    if audit.status.is_signed() {
        if !program.independence.is_accepted_declaration() {
            return Err(IrValidationError::Message(format!(
                "sign-off without accepted independence on audit {}",
                audit.id
            )));
        }
        let sign_off = audit.sign_off.as_ref().ok_or_else(|| {
            IrValidationError::Message(format!("signed audit {} missing sign-off", audit.id))
        })?;
        if sign_off.principal.is_none() {
            return Err(IrValidationError::Message(format!(
                "sign-off without human principal on audit {}",
                audit.id
            )));
        }
        if sign_off.statement.trim().is_empty() || !sign_off.conclusion.is_signable() {
            return Err(IrValidationError::Message(format!(
                "signed audit {} requires a human conclusion and statement",
                audit.id
            )));
        }
        if let Some(conclusion) = audit.conclusion
            && conclusion != sign_off.conclusion
        {
            return Err(IrValidationError::Message(format!(
                "audit {} conclusion does not match sign-off",
                audit.id
            )));
        }
    }

    Ok(())
}

/// Used by prepare to name open risk ids as hotspots.
pub fn open_risk_hotspots(risks: &[(RiskId, crate::RiskStatus)]) -> Vec<RiskId> {
    risks
        .iter()
        .filter(|(_, status)| *status == crate::RiskStatus::Open)
        .map(|(id, _)| id.clone())
        .collect()
}
