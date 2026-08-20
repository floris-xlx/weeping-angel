//! Internal-audit engine: prepare candidates, propose samples, pin evidence,
//! record findings, and accept human conclusion / sign-off.
//!
//! Never auto-signs. Never auto-accepts independence. Machine lists are proposals.

use chrono::{DateTime, Utc};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentScope, Audit, AuditConclusion, AuditEvidencePin, AuditFinding,
    AuditHistoryEvent, AuditId, AuditPeriod, AuditProgram, AuditProgramId, AuditProgramStatus,
    AuditSample, AuditSampleProposal, AuditSignOff, AuditStatus, ControlId, IndependenceRecord,
    PrincipalRef, RiskId, RiskStatus, SampleMethod, compute_proposal_digest, compute_sample_digest,
    population_digest,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

use crate::lineage::{EvidenceSnapshot, seal_evidence_snapshot};
use crate::snapshot::AssessmentRun;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuditEngineError {
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopulationRef {
    pub population_id: String,
    pub member_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorFindingRef {
    pub finding_id: String,
    pub audit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemediationRef {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct AuditPrepareBundle {
    pub candidate_scope: AssessmentScope,
    pub stale_or_failed_controls: Vec<ControlId>,
    pub risk_hotspots: Vec<RiskId>,
    pub evidence_bundle: EvidenceSnapshot,
    pub sample_populations: Vec<PopulationRef>,
    pub sample_proposal: Option<AuditSampleProposal>,
    pub prior_findings: Vec<PriorFindingRef>,
    pub remediation_status: Vec<RemediationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReplay {
    pub evidence_snapshot_digest: Option<String>,
    pub envelope_digests: Vec<String>,
    pub sample_digest: Option<String>,
    pub findings: Vec<String>,
    pub conclusion: Option<AuditConclusion>,
    pub sign_off: Option<AuditSignOff>,
}

fn default_auditor(definition: &AssessmentDefinition) -> PrincipalRef {
    definition
        .identities
        .first()
        .map(|identity| PrincipalRef::Identity(identity.id.clone()))
        .unwrap_or_else(|| PrincipalRef::Role("auditor".into()))
}

fn default_principal(definition: &AssessmentDefinition) -> PrincipalRef {
    definition
        .identities
        .get(1)
        .or(definition.identities.first())
        .map(|identity| PrincipalRef::Identity(identity.id.clone()))
        .unwrap_or_else(|| PrincipalRef::Role("principal".into()))
}

fn stale_or_failed(results: &[ControlTestResult]) -> Vec<ControlId> {
    let mut ids = Vec::new();
    for result in results {
        if matches!(
            result.effectiveness,
            Effectiveness::Ineffective
                | Effectiveness::StaleEvidence
                | Effectiveness::InsufficientEvidence
                | Effectiveness::PartiallyEffective
        ) && !ids.iter().any(|id: &ControlId| id == &result.control_id)
        {
            ids.push(result.control_id.clone());
        }
    }
    ids
}

fn risk_hotspots(definition: &AssessmentDefinition) -> Vec<RiskId> {
    definition
        .risks
        .iter()
        .filter(|risk| risk.status == RiskStatus::Open)
        .map(|risk| risk.id.clone())
        .collect()
}

fn prior_findings(definition: &AssessmentDefinition) -> Vec<PriorFindingRef> {
    let signed: std::collections::BTreeSet<_> = definition
        .audits
        .iter()
        .filter(|audit| audit.status.is_signed())
        .map(|audit| audit.id.as_str().to_string())
        .collect();
    definition
        .audit_findings
        .iter()
        .filter(|finding| signed.contains(finding.audit_id.as_str()))
        .map(|finding| PriorFindingRef {
            finding_id: finding.id.as_str().to_string(),
            audit_id: finding.audit_id.as_str().to_string(),
        })
        .collect()
}

fn evidence_from_run(last_run: Option<&AssessmentRun>) -> EvidenceSnapshot {
    match last_run {
        Some(run) if !run.evidence_snapshot_digest.is_empty() => EvidenceSnapshot {
            schema: crate::lineage::LINEAGE_SNAPSHOT_SCHEMA.into(),
            envelope_digests: Vec::new(),
            collection_run_ids: run.collector_runs.clone(),
            digest: run.evidence_snapshot_digest.clone(),
        },
        _ => seal_evidence_snapshot(Vec::new(), Vec::new()),
    }
}

fn empty_independence(
    auditor: PrincipalRef,
    principal: PrincipalRef,
    clock: DateTime<Utc>,
) -> IndependenceRecord {
    IndependenceRecord {
        auditor,
        principal,
        declared_at: clock,
        statement: String::new(),
        evidence_refs: Vec::new(),
        conflict_flags: Vec::new(),
        accepted: false,
    }
}

fn append_history(audit: &mut Audit, clock: DateTime<Utc>, kind: &str, payload: &str) {
    audit.history.push(AuditHistoryEvent {
        at: clock,
        principal: None,
        kind: kind.into(),
        payload_digest: payload.into(),
    });
}

fn sort_unique(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids.dedup();
    ids
}

fn select_seeded(sorted: &[String], seed: &str, size: usize) -> Vec<String> {
    let mut ranked: Vec<(String, &String)> = sorted
        .iter()
        .map(|id| {
            let digest = weeping_angel_assurance_ir::canonical_digest(&format!("{seed}\0{id}"))
                .unwrap_or_default();
            (digest, id)
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    ranked
        .into_iter()
        .take(size)
        .map(|(_, id)| id.clone())
        .collect()
}

fn select_systematic(sorted: &[String], seed: &str, size: usize) -> Vec<String> {
    if sorted.is_empty() || size == 0 {
        return Vec::new();
    }
    let offset = seed.as_bytes().iter().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(usize::from(*b))
    }) % sorted.len();
    let step = (sorted.len() / size).max(1);
    let mut selected = Vec::new();
    let mut idx = offset;
    while selected.len() < size && selected.len() < sorted.len() {
        let id = sorted[idx % sorted.len()].clone();
        if !selected.contains(&id) {
            selected.push(id);
        }
        idx = idx.wrapping_add(step);
        if idx == offset {
            break;
        }
    }
    selected.sort();
    selected
}

/// Prepare a draft annual (or other-period) program. Does not approve or sign.
pub fn prepare_audit_program(
    definition: &AssessmentDefinition,
    period: AuditPeriod,
    last_run: Option<&AssessmentRun>,
    last_results: &[ControlTestResult],
    clock: DateTime<Utc>,
) -> (AuditProgram, AuditPrepareBundle) {
    let auditor = default_auditor(definition);
    let principal = default_principal(definition);
    let bundle = build_prepare_bundle(definition, last_run, last_results, &definition.scope, clock);
    let program = AuditProgram {
        schema_version: weeping_angel_assurance_ir::ASSURANCE_IR_SCHEMA.into(),
        id: AuditProgramId::new(format!("audit:{}", period.start.format("%Y"))),
        title: format!("Internal audit program {}", period.start.format("%Y")),
        period,
        scope: definition.scope.clone(),
        objectives: Vec::new(),
        criteria: Vec::new(),
        schedule: Vec::new(),
        principal: principal.clone(),
        auditor: auditor.clone(),
        independence: empty_independence(auditor, principal, clock),
        child_audit_ids: Vec::new(),
        status: AuditProgramStatus::Draft,
        prepared_from: None,
    };
    (program, bundle)
}

/// Prepare a child audit. Leaves `sample`, `conclusion`, and `signOff` unset.
pub fn prepare_audit(
    program: &AuditProgram,
    scope: AssessmentScope,
    last_run: Option<&AssessmentRun>,
    last_results: &[ControlTestResult],
    definition: &AssessmentDefinition,
    clock: DateTime<Utc>,
) -> (Audit, AuditPrepareBundle) {
    let bundle = build_prepare_bundle(definition, last_run, last_results, &scope, clock);
    let audit = Audit {
        schema_version: weeping_angel_assurance_ir::ASSURANCE_IR_SCHEMA.into(),
        id: AuditId::new(format!(
            "{}.engagement",
            program.id.as_str().replace(':', "-")
        )),
        program_id: program.id.clone(),
        title: format!("{} engagement", program.title),
        period: program.period.clone(),
        scope,
        sample: None,
        selected_controls: Vec::new(),
        selected_requirements: Vec::new(),
        evidence_pin: None,
        procedures: Vec::new(),
        observations: Vec::new(),
        findings: Vec::new(),
        nonconformity_refs: Vec::new(),
        conclusion: None,
        sign_off: None,
        status: AuditStatus::Prepared,
        history: vec![AuditHistoryEvent {
            at: clock,
            principal: None,
            kind: "prepared".into(),
            payload_digest: "sha256:prepared".into(),
        }],
        sample_proposal: bundle.sample_proposal.clone(),
    };
    (audit, bundle)
}

fn build_prepare_bundle(
    definition: &AssessmentDefinition,
    last_run: Option<&AssessmentRun>,
    last_results: &[ControlTestResult],
    scope: &AssessmentScope,
    clock: DateTime<Utc>,
) -> AuditPrepareBundle {
    let stale = stale_or_failed(last_results);
    let population: Vec<String> = if stale.is_empty() {
        definition
            .controls
            .iter()
            .map(|control| control.id().as_str().to_string())
            .collect()
    } else {
        stale.iter().map(|id| id.as_str().to_string()).collect()
    };
    let proposal = if population.is_empty() {
        None
    } else {
        propose_sample(
            "pop:prepare",
            population.clone(),
            SampleMethod::SeededRandom,
            Some("prepare"),
            1,
            "stale/failed hotspot sample",
            clock,
        )
        .ok()
    };
    AuditPrepareBundle {
        candidate_scope: scope.clone(),
        stale_or_failed_controls: stale,
        risk_hotspots: risk_hotspots(definition),
        evidence_bundle: evidence_from_run(last_run),
        sample_populations: vec![PopulationRef {
            population_id: "pop:prepare".into(),
            member_ids: sort_unique(population),
        }],
        sample_proposal: proposal,
        prior_findings: prior_findings(definition),
        remediation_status: Vec::new(),
    }
}

/// Deterministic sample proposal. `Judgmental` is an auditor method and is not
/// emitted here.
pub fn propose_sample(
    population_id: &str,
    population: impl IntoIterator<Item = String>,
    method: SampleMethod,
    seed: Option<&str>,
    size: u32,
    rationale: &str,
    clock: DateTime<Utc>,
) -> Result<AuditSampleProposal, AuditEngineError> {
    if matches!(method, SampleMethod::Judgmental) {
        return Err(AuditEngineError::Message(
            "judgmental samples cannot be machine-proposed".into(),
        ));
    }
    let sorted = sort_unique(population.into_iter().collect());
    if method.requires_seed() && seed.map(str::trim).unwrap_or("").is_empty() {
        return Err(AuditEngineError::Message(
            "systematic and seededRandom require a seed".into(),
        ));
    }
    let selected = match method {
        SampleMethod::Census => sorted.clone(),
        SampleMethod::SeededRandom => {
            let take = (size as usize).min(sorted.len());
            select_seeded(&sorted, seed.unwrap_or(""), take)
        }
        SampleMethod::Systematic => {
            let take = (size as usize).min(sorted.len()).max(1);
            select_systematic(&sorted, seed.unwrap_or(""), take)
        }
        SampleMethod::Judgmental => unreachable!("rejected before propose"),
    };
    let pop_digest =
        population_digest(&sorted).map_err(|err| AuditEngineError::Message(err.to_string()))?;
    let mut proposal = AuditSampleProposal {
        population_id: population_id.into(),
        population_digest: pop_digest,
        method,
        seed: seed.map(str::to_string),
        size: selected.len() as u32,
        suggested_ids: selected,
        rationale: rationale.into(),
        generated_at: clock,
        proposal_digest: String::new(),
        kind: "proposal".into(),
    };
    proposal.proposal_digest = compute_proposal_digest(&proposal)
        .map_err(|err| AuditEngineError::Message(err.to_string()))?;
    Ok(proposal)
}

/// Persist an accepted sample. A proposal alone is not the sample.
#[allow(clippy::too_many_arguments)]
pub fn accept_sample(
    audit: &mut Audit,
    proposal: Option<&AuditSampleProposal>,
    selected_ids: Vec<String>,
    method: SampleMethod,
    seed: Option<String>,
    population_id: String,
    population_digest_value: String,
    accepted_by: PrincipalRef,
    clock: DateTime<Utc>,
) -> Result<AuditSample, AuditEngineError> {
    if audit.status.is_frozen() {
        return Err(AuditEngineError::Message(
            "signed audits cannot change sample".into(),
        ));
    }
    if matches!(method, SampleMethod::Judgmental) && selected_ids.is_empty() {
        return Err(AuditEngineError::Message(
            "judgmental sample requires explicit selectedIds".into(),
        ));
    }
    if method.requires_seed() && seed.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AuditEngineError::Message(
            "systematic and seededRandom require a seed".into(),
        ));
    }
    let selected = sort_unique(selected_ids);
    let sample_digest =
        compute_sample_digest(method, seed.as_deref(), &population_digest_value, &selected)
            .map_err(|err| AuditEngineError::Message(err.to_string()))?;
    let sample = AuditSample {
        population_id,
        population_digest: population_digest_value,
        method,
        seed,
        size: selected.len() as u32,
        selected_ids: selected,
        accepted_by,
        accepted_at: clock,
        proposal_digest: proposal.map(|row| row.proposal_digest.clone()),
        sample_digest: sample_digest.clone(),
    };
    audit.sample = Some(sample.clone());
    append_history(audit, clock, "sampleAccepted", &sample_digest);
    Ok(sample)
}

pub fn pin_evidence(
    audit: &mut Audit,
    snapshot: &EvidenceSnapshot,
    pinned_by: PrincipalRef,
    clock: DateTime<Utc>,
) -> Result<AuditEvidencePin, AuditEngineError> {
    if audit.status.is_frozen() {
        return Err(AuditEngineError::Message(
            "signed audits cannot change the evidence pin".into(),
        ));
    }
    let pin = AuditEvidencePin {
        evidence_snapshot_digest: snapshot.digest.clone(),
        envelope_digests: snapshot.envelope_digests.clone(),
        collection_run_ids: snapshot.collection_run_ids.clone(),
        assessment_run_id: None,
        assessment_definition_digest: None,
        framework_pack_digest: None,
        canonical_catalog_digest: None,
        as_of: None,
        period: Some(audit.period.clone()),
        pinned_at: clock,
        pinned_by,
    };
    audit.evidence_pin = Some(pin.clone());
    append_history(audit, clock, "pinned", &pin.evidence_snapshot_digest);
    Ok(pin)
}

pub fn record_finding(
    audit: &mut Audit,
    findings: &mut Vec<AuditFinding>,
    finding: AuditFinding,
) -> Result<(), AuditEngineError> {
    if audit.status.is_frozen() {
        return Err(AuditEngineError::Message(
            "signed audits cannot record new findings".into(),
        ));
    }
    if finding.audit_id != audit.id {
        return Err(AuditEngineError::Message(
            "finding audit id does not match the target audit".into(),
        ));
    }
    if let Some(pin) = &audit.evidence_pin {
        for digest in &finding.evidence_digests {
            if !pin.envelope_digests.iter().any(|pinned| pinned == digest) {
                return Err(AuditEngineError::Message(format!(
                    "finding evidence {digest} is not in the audit pin"
                )));
            }
        }
    }
    if !audit.findings.iter().any(|id| id == &finding.id) {
        audit.findings.push(finding.id.clone());
    }
    if let Some(nc) = &finding.nonconformity_id
        && !audit.nonconformity_refs.iter().any(|id| id == nc)
    {
        audit.nonconformity_refs.push(nc.clone());
    }
    findings.retain(|row| row.id != finding.id);
    findings.push(finding);
    Ok(())
}

pub fn conclude_audit(audit: &mut Audit, program: &AuditProgram) -> Result<(), AuditEngineError> {
    if !program.independence.is_accepted_declaration() {
        return Err(AuditEngineError::Message(
            "independence must be accepted before conclude".into(),
        ));
    }
    if !matches!(
        program.status,
        AuditProgramStatus::Approved | AuditProgramStatus::InProgress
    ) {
        return Err(AuditEngineError::Message(
            "program must be approved before conclude".into(),
        ));
    }
    if audit.sample.is_none() {
        return Err(AuditEngineError::Message(
            "accept_sample required before conclude".into(),
        ));
    }
    if audit.evidence_pin.is_none() {
        return Err(AuditEngineError::Message(
            "pin_evidence required before conclude".into(),
        ));
    }
    if !audit.procedures_complete() {
        return Err(AuditEngineError::Message(
            "unfinished procedures cannot conclude".into(),
        ));
    }
    if audit.status.is_frozen() {
        return Err(AuditEngineError::Message(
            "frozen audit cannot be re-concluded".into(),
        ));
    }
    audit.status = AuditStatus::Concluded;
    Ok(())
}

/// Human principal + conclusion + statement. Effectiveness never writes sign-off.
pub fn sign_off(
    audit: &mut Audit,
    program: &AuditProgram,
    principal: PrincipalRef,
    conclusion: AuditConclusion,
    statement: String,
    clock: DateTime<Utc>,
) -> Result<(), AuditEngineError> {
    conclude_audit(audit, program)?;
    if statement.trim().is_empty() {
        return Err(AuditEngineError::Message(
            "sign-off requires a non-empty statement".into(),
        ));
    }
    if !conclusion.is_signable() {
        return Err(AuditEngineError::Message(
            "notConcluded is not a successful sign-off value".into(),
        ));
    }
    audit.conclusion = Some(conclusion);
    audit.sign_off = Some(AuditSignOff {
        principal: Some(principal),
        signed_at: clock,
        conclusion,
        statement,
    });
    audit.status = AuditStatus::Signed;
    append_history(audit, clock, "signed", "sha256:signed");
    Ok(())
}

/// Replay uses the signed body + pins, not the live graph.
pub fn replay_audit(audit: &Audit) -> AuditReplay {
    AuditReplay {
        evidence_snapshot_digest: audit
            .evidence_pin
            .as_ref()
            .map(|pin| pin.evidence_snapshot_digest.clone()),
        envelope_digests: audit
            .evidence_pin
            .as_ref()
            .map(|pin| pin.envelope_digests.clone())
            .unwrap_or_default(),
        sample_digest: audit
            .sample
            .as_ref()
            .map(|sample| sample.sample_digest.clone()),
        findings: audit
            .findings
            .iter()
            .map(|id| id.as_str().to_string())
            .collect(),
        conclusion: audit.conclusion,
        sign_off: audit.sign_off.clone(),
    }
}

pub fn reviewed_envelopes(audit: &Audit) -> Vec<String> {
    replay_audit(audit).envelope_digests
}
