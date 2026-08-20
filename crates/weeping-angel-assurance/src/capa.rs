//! Nonconformity / CAPA engine. Network-free; no ticket clients.

use chrono::{DateTime, Utc};
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AuditFinding, CapaError, ClosureDecision, ControlId, CorrectiveAction,
    CorrectiveActionId, EffectivenessCriteria, EffectivenessReview, EffectivenessReviewStatus,
    EventRef, Incident, IsmsEvent, IsmsEventKind, Nonconformity, NonconformityId,
    NonconformitySource, NonconformitySourceKind, NonconformityStatus, PrincipalRef, ReviewPeriod,
    VerificationMode,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

pub fn propose_from_audit_finding(
    finding: &AuditFinding,
    id: NonconformityId,
    owner: PrincipalRef,
    at: DateTime<Utc>,
    opened_by: PrincipalRef,
) -> Result<Nonconformity, CapaError> {
    let source = NonconformitySource {
        kind: NonconformitySourceKind::AuditFinding,
        audit_finding_id: Some(finding.id.clone()),
        audit_id: Some(finding.audit_id.clone()),
        incident_id: None,
        event_ref: None,
        control_ids: finding.control_ids.clone(),
    };
    let mut nc = Nonconformity::open(
        id,
        finding.title.clone(),
        finding.description.clone(),
        source,
        owner,
        finding.created_at,
        at,
        opened_by,
    );
    nc.affected.requirement_ids = finding.requirement_ids.clone();
    nc.affected.control_ids = finding.control_ids.clone();
    Ok(nc)
}

pub fn propose_from_incident(
    incident: &Incident,
    id: NonconformityId,
    owner: PrincipalRef,
    at: DateTime<Utc>,
    opened_by: PrincipalRef,
) -> Result<Nonconformity, CapaError> {
    let control_ids: Vec<ControlId> = incident
        .control_failure_refs
        .iter()
        .map(|r| r.control_id.clone())
        .collect();
    let source = NonconformitySource {
        kind: NonconformitySourceKind::Incident,
        audit_finding_id: None,
        audit_id: None,
        incident_id: Some(incident.id.clone()),
        event_ref: None,
        control_ids: control_ids.clone(),
    };
    let description = if incident.summary.trim().is_empty() {
        incident.title.clone()
    } else {
        incident.summary.clone()
    };
    let mut nc = Nonconformity::open(
        id,
        incident.title.clone(),
        description,
        source,
        owner,
        incident.declared_at,
        at,
        opened_by,
    );
    nc.affected.asset_ids = incident.asset_ids.clone();
    nc.affected.processing_activity_ids = incident.processing_activity_ids.clone();
    nc.affected.population = incident.population.clone();
    nc.affected.control_ids = control_ids;
    nc.remediation_refs = incident.corrective_action_ids.clone();
    Ok(nc)
}

pub fn propose_from_control_regression(
    event: &IsmsEvent,
    id: NonconformityId,
    owner: PrincipalRef,
    at: DateTime<Utc>,
    opened_by: PrincipalRef,
) -> Result<Nonconformity, CapaError> {
    if event.kind != IsmsEventKind::ControlRegressed {
        return Err(CapaError::Message(
            "propose_from_control_regression requires IsmsEventKind::ControlRegressed".into(),
        ));
    }
    let control_ids = event
        .payload
        .get("controlId")
        .and_then(|v| v.as_str())
        .map(|raw| vec![ControlId::new(raw)])
        .unwrap_or_default();
    let source = NonconformitySource {
        kind: NonconformitySourceKind::ControlRegression,
        audit_finding_id: None,
        audit_id: None,
        incident_id: None,
        event_ref: Some(EventRef::new(event.event_id.as_str())),
        control_ids: control_ids.clone(),
    };
    let mut nc = Nonconformity::open(
        id,
        "Control regression",
        "A control test regressed; this is a proposed nonconformity, not a classification.",
        source,
        owner,
        at,
        at,
        opened_by,
    );
    nc.affected.control_ids = control_ids;
    Ok(nc)
}

pub fn evaluate_capa_effectiveness(
    nc: &Nonconformity,
    actions: &[CorrectiveAction],
    results: &[ControlTestResult],
    as_of: DateTime<Utc>,
    reviewer: PrincipalRef,
) -> Result<EffectivenessReview, CapaError> {
    let criteria = actions
        .first()
        .map(|a| a.effectiveness_criteria.clone())
        .unwrap_or_else(|| EffectivenessCriteria {
            mode: VerificationMode::SustainedWindow,
            window: Some(14 * 24 * 3600),
            min_effective_results: 2,
            independent_verifier: false,
            statement: "default SustainedWindow".into(),
            control_ids: nc.source.control_ids.clone(),
        });
    let period = actions
        .first()
        .map(|a| a.review_period.clone())
        .or_else(|| nc.effectiveness.as_ref().map(|r| r.period.clone()))
        .unwrap_or(ReviewPeriod {
            start: as_of,
            end: as_of + chrono::Duration::days(21),
        });

    if criteria.independent_verifier || criteria.mode == VerificationMode::IndependentReviewRequired
    {
        if reviewer == nc.owner {
            return Err(CapaError::Message(
                "independent reviewer must differ from nonconformity owner".into(),
            ));
        }
        if actions.iter().any(|a| a.owner == reviewer) {
            return Err(CapaError::Message(
                "independent reviewer must differ from action owner".into(),
            ));
        }
    }

    let watched: Vec<&ControlId> = if criteria.control_ids.is_empty() {
        nc.affected
            .control_ids
            .iter()
            .chain(nc.source.control_ids.iter())
            .collect()
    } else {
        criteria.control_ids.iter().collect()
    };

    let mut relevant: Vec<&ControlTestResult> = results
        .iter()
        .filter(|result| {
            (watched.is_empty() || watched.contains(&&result.control_id))
                && result.checked_at >= period.start
                && result.checked_at < period.end
        })
        .collect();
    relevant.sort_by_key(|result| result.checked_at);

    let result_digests: Vec<String> = relevant
        .iter()
        .map(|result| result.input_digest.clone())
        .collect();

    let fail_closed = relevant
        .iter()
        .any(|result| is_fail_closed(result.effectiveness));
    if fail_closed {
        return Ok(EffectivenessReview {
            period,
            reviewer,
            status: EffectivenessReviewStatus::Failed,
            result_digests,
            note: Some("fail-closed Effectiveness during the review window".into()),
        });
    }

    let mut greens: Vec<&ControlTestResult> = relevant
        .iter()
        .copied()
        .filter(|result| result.effectiveness == Effectiveness::Effective)
        .collect();
    greens.sort_by_key(|result| result.checked_at);

    let status = match criteria.mode {
        VerificationMode::SingleGreenPermitted => {
            if greens.len() as u32 >= criteria.min_effective_results.max(1) {
                EffectivenessReviewStatus::Satisfied
            } else if greens.is_empty() {
                EffectivenessReviewStatus::NotStarted
            } else {
                EffectivenessReviewStatus::InWindow
            }
        }
        VerificationMode::IndependentReviewRequired => {
            if greens.len() as u32 >= criteria.min_effective_results.max(1) {
                EffectivenessReviewStatus::Satisfied
            } else if greens.is_empty() {
                EffectivenessReviewStatus::NotStarted
            } else {
                EffectivenessReviewStatus::InWindow
            }
        }
        VerificationMode::SustainedWindow => {
            let needed = criteria.min_effective_results.max(2);
            let window_secs = criteria.window.unwrap_or(14 * 24 * 3600);
            sustained_window_status(&relevant, &greens, needed, window_secs)
        }
    };

    Ok(EffectivenessReview {
        period,
        reviewer,
        status,
        result_digests,
        note: None,
    })
}

pub fn close_nonconformity(
    nc: &mut Nonconformity,
    decision: ClosureDecision,
) -> Result<(), CapaError> {
    nc.close(decision)
}

pub fn overdue_corrective_actions(
    assessment: &AssessmentDefinition,
    as_of: DateTime<Utc>,
) -> Vec<CorrectiveActionId> {
    weeping_angel_assurance_ir::overdue_action_ids(assessment, as_of)
}

pub fn open_nonconformities(assessment: &AssessmentDefinition) -> Vec<&Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| {
            !matches!(
                nc.status,
                NonconformityStatus::Closed
                    | NonconformityStatus::Cancelled
                    | NonconformityStatus::Superseded
            )
        })
        .collect()
}

pub fn failed_effectiveness_reviews(assessment: &AssessmentDefinition) -> Vec<&Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| {
            nc.effectiveness
                .as_ref()
                .is_some_and(|r| r.status == EffectivenessReviewStatus::Failed)
        })
        .collect()
}

pub fn nonconformities_for_audit<'a>(
    assessment: &'a AssessmentDefinition,
    audit_id: &weeping_angel_assurance_ir::AuditId,
) -> Vec<&'a Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| nc.source.audit_id.as_ref() == Some(audit_id))
        .collect()
}

pub fn nonconformities_for_incident<'a>(
    assessment: &'a AssessmentDefinition,
    incident_id: &weeping_angel_assurance_ir::IncidentId,
) -> Vec<&'a Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| nc.source.incident_id.as_ref() == Some(incident_id))
        .collect()
}

pub fn reopened_nonconformities(assessment: &AssessmentDefinition) -> Vec<&Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| {
            nc.history.iter().any(|event| {
                event.kind == weeping_angel_assurance_ir::NonconformityEventKind::Reopened
            })
        })
        .collect()
}

pub fn closed_nonconformities(assessment: &AssessmentDefinition) -> Vec<&Nonconformity> {
    assessment
        .nonconformities
        .iter()
        .filter(|nc| nc.status == NonconformityStatus::Closed)
        .collect()
}

fn is_fail_closed(effectiveness: Effectiveness) -> bool {
    matches!(
        effectiveness,
        Effectiveness::Ineffective
            | Effectiveness::InsufficientEvidence
            | Effectiveness::StaleEvidence
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

fn sustained_window_status(
    relevant: &[&ControlTestResult],
    greens: &[&ControlTestResult],
    needed: u32,
    window_secs: u64,
) -> EffectivenessReviewStatus {
    if greens.is_empty() {
        return EffectivenessReviewStatus::NotStarted;
    }
    let mut count = 0u32;
    let mut window_start = greens[0].checked_at;
    let mut satisfied = false;
    for green in greens {
        let intervening_fail = relevant.iter().any(|result| {
            result.checked_at > window_start
                && result.checked_at < green.checked_at
                && is_window_break(result.effectiveness)
        });
        if intervening_fail {
            count = 1;
            window_start = green.checked_at;
            continue;
        }
        count += 1;
        let span = green
            .checked_at
            .signed_duration_since(window_start)
            .num_seconds();
        if count >= needed && span >= window_secs as i64 {
            satisfied = true;
            break;
        }
    }
    if satisfied {
        EffectivenessReviewStatus::Satisfied
    } else {
        EffectivenessReviewStatus::InWindow
    }
}
