//! Remediation workflow engine. Network-free; no ticket clients.

use chrono::{DateTime, Utc};
use serde::Serialize;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, ControlId, ExternalTicketRef, PrincipalRef, Remediation,
    RemediationError, RemediationId, RemediationSource, RemediationSourceKind, RemediationState,
    TreatmentActionId, VerificationMode, VerificationState, VerificationStatus, canonical_digest,
    waiver_in_force,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultIdentity<'a> {
    test_id: &'a weeping_angel_assurance_ir::ControlTestId,
    control_id: &'a ControlId,
    effectiveness: Effectiveness,
    input_digest: &'a str,
    test_version: &'a str,
    evidence_refs: &'a [String],
}

pub fn create_from_source(
    id: RemediationId,
    title: impl Into<String>,
    source: RemediationSource,
    owner: PrincipalRef,
) -> Result<Remediation, RemediationError> {
    Remediation::propose(id, title, source, owner)
}

pub fn create_from_control_regression(
    id: RemediationId,
    title: impl Into<String>,
    source: RemediationSource,
    control_id: ControlId,
    owner: PrincipalRef,
) -> Result<Remediation, RemediationError> {
    if source.kind != RemediationSourceKind::ControlRegressed {
        return Err(RemediationError::NotControlRegression);
    }
    source.validate()?;
    let subjects = source.subject_selectors.clone();
    let mut rem = create_from_source(id, title, source, owner)?;
    if !rem.control_ids.iter().any(|c| c == &control_id) {
        rem.control_ids.push(control_id);
    }
    rem.subject_selectors = subjects;
    Ok(rem)
}

pub fn link_treatment_action(
    remediation: Remediation,
    action_id: TreatmentActionId,
) -> Result<Remediation, RemediationError> {
    remediation.link_action(action_id)
}

pub fn attach_external_ticket(
    remediation: Remediation,
    ticket: ExternalTicketRef,
) -> Result<Remediation, RemediationError> {
    remediation.attach_ticket(ticket, None, Utc::now())
}

pub fn sla_overdue(remediation: &Remediation, as_of: DateTime<Utc>) -> bool {
    remediation.sla_overdue(as_of)
}

pub fn evaluate_verification(
    remediation: &Remediation,
    results: &[ControlTestResult],
    as_of: DateTime<Utc>,
    verifier: Option<&PrincipalRef>,
) -> Result<VerificationState, RemediationError> {
    let relevant: Vec<&ControlTestResult> = results
        .iter()
        .filter(|result| {
            remediation
                .control_ids
                .iter()
                .any(|id| id == &result.control_id)
        })
        .collect();

    let last = relevant.last();
    let last_result_digest = match last {
        Some(result) => result_identity_digest(result)?,
        None => None,
    };

    if remediation.state == RemediationState::AwaitingVerification
        && relevant
            .iter()
            .any(|result| is_fail_closed(result.effectiveness))
    {
        return Ok(VerificationState {
            status: VerificationStatus::Failed,
            last_result_digest,
            window_start: None,
            satisfied_at: None,
            note: Some("ineffective result while awaiting verification".into()),
        });
    }

    if remediation.verification_policy.independent_verifier
        || remediation.verification_policy.mode == VerificationMode::IndependentReviewRequired
    {
        match verifier {
            None => {
                return Err(RemediationError::Message(
                    "independent verifier is required".into(),
                ));
            }
            Some(principal) if principal == &remediation.owner => {
                return Err(RemediationError::Message(
                    "independent verifier must differ from owner".into(),
                ));
            }
            Some(_) => {}
        }
    }

    let mut greens: Vec<&ControlTestResult> = relevant
        .iter()
        .copied()
        .filter(|result| result.effectiveness == Effectiveness::Effective)
        .collect();
    greens.sort_by_key(|result| result.checked_at);

    let mut state = VerificationState {
        status: VerificationStatus::NotStarted,
        last_result_digest,
        window_start: greens.first().map(|r| r.checked_at),
        satisfied_at: None,
        note: None,
    };

    match remediation.verification_policy.mode {
        VerificationMode::SingleGreenPermitted => {
            if greens.len() as u32 >= remediation.verification_policy.min_effective_results.max(1) {
                state.status = VerificationStatus::Satisfied;
                state.satisfied_at = greens.last().map(|r| r.checked_at).or(Some(as_of));
            } else if !greens.is_empty() {
                state.status = VerificationStatus::InWindow;
            }
        }
        VerificationMode::IndependentReviewRequired => {
            if greens.len() as u32 >= remediation.verification_policy.min_effective_results.max(1) {
                state.status = VerificationStatus::Satisfied;
                state.satisfied_at = Some(as_of);
            } else if !greens.is_empty() {
                state.status = VerificationStatus::InWindow;
            }
        }
        VerificationMode::SustainedWindow => {
            let needed = remediation.verification_policy.min_effective_results.max(2);
            let window_secs = remediation
                .verification_policy
                .window
                .unwrap_or(14 * 24 * 3600);
            if greens.is_empty() {
                state.status = VerificationStatus::NotStarted;
            } else {
                let first = greens[0].checked_at;
                let mut last_ok = first;
                let mut count = 0u32;
                let mut satisfied = false;
                for green in &greens {
                    let intervening_fail = relevant.iter().any(|result| {
                        result.checked_at > first
                            && result.checked_at < green.checked_at
                            && is_window_break(result.effectiveness)
                    });
                    if intervening_fail {
                        count = 1;
                        last_ok = green.checked_at;
                        state.window_start = Some(green.checked_at);
                        continue;
                    }
                    count += 1;
                    last_ok = green.checked_at;
                    let span = last_ok.signed_duration_since(first).num_seconds();
                    if count >= needed && span >= window_secs as i64 {
                        satisfied = true;
                        break;
                    }
                }
                if satisfied {
                    state.status = VerificationStatus::Satisfied;
                    state.satisfied_at = Some(last_ok);
                } else {
                    state.status = VerificationStatus::InWindow;
                }
            }
        }
    }

    Ok(state)
}

pub fn reopen_expired_waiver(
    remediation: Remediation,
    as_of: DateTime<Utc>,
    principal: PrincipalRef,
    assessment: &AssessmentDefinition,
) -> Result<Remediation, RemediationError> {
    if remediation.state != RemediationState::AcceptedWaived {
        return Err(RemediationError::InvalidTransition {
            from: remediation.state,
            to: RemediationState::Open,
        });
    }
    if waiver_in_force(assessment, &remediation, as_of) {
        return Err(RemediationError::Message("waiver is still in force".into()));
    }
    remediation.transition(RemediationState::Open, Some(principal), as_of)
}

pub fn close(
    mut remediation: Remediation,
    principal: PrincipalRef,
    at: DateTime<Utc>,
    rationale: impl Into<String>,
) -> Result<Remediation, RemediationError> {
    if remediation.state == RemediationState::Closed {
        return Err(RemediationError::ImmutableClosure(remediation.id.clone()));
    }
    if remediation.state != RemediationState::Verified {
        return Err(RemediationError::InvalidTransition {
            from: remediation.state,
            to: RemediationState::Closed,
        });
    }
    let rationale = rationale.into();
    if rationale.trim().is_empty() {
        return Err(RemediationError::Message(
            "closureRationale is required".into(),
        ));
    }
    remediation.closed_by = Some(principal.clone());
    remediation.closed_at = Some(at);
    remediation.closure_rationale = Some(rationale);
    remediation.transition(RemediationState::Closed, Some(principal), at)
}

fn is_fail_closed(effectiveness: Effectiveness) -> bool {
    matches!(
        effectiveness,
        Effectiveness::Ineffective
            | Effectiveness::InsufficientEvidence
            | Effectiveness::StaleEvidence
            | Effectiveness::PartiallyEffective
    )
}

fn is_window_break(effectiveness: Effectiveness) -> bool {
    matches!(
        effectiveness,
        Effectiveness::Ineffective
            | Effectiveness::InsufficientEvidence
            | Effectiveness::StaleEvidence
    )
}

fn result_identity_digest(result: &ControlTestResult) -> Result<Option<String>, RemediationError> {
    let body = ResultIdentity {
        test_id: &result.test_id,
        control_id: &result.control_id,
        effectiveness: result.effectiveness,
        input_digest: &result.input_digest,
        test_version: &result.test_version,
        evidence_refs: &result.evidence_refs,
    };
    canonical_digest(&body)
        .map(Some)
        .map_err(|err| RemediationError::Message(err.to_string()))
}
