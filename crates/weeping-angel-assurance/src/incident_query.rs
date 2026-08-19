//! Audit and management-review *preparation* queries over incident IR.
//!
//! Does not generate audit conclusions or management-review minutes.

use chrono::{DateTime, Utc};
use weeping_angel_assurance_ir::{AssessmentDefinition, Incident, IncidentKind, IncidentStatus};

/// Incidents whose `declared_at` falls in `[from, to]` (inclusive).
pub fn incidents_in_period(
    assessment: &AssessmentDefinition,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Vec<&Incident> {
    assessment
        .incidents
        .iter()
        .filter(|incident| incident.declared_at >= from && incident.declared_at <= to)
        .collect()
}

/// Real incidents that are closed without a post-incident review.
pub fn incident_postmortem_missing(assessment: &AssessmentDefinition) -> Vec<&Incident> {
    assessment
        .incidents
        .iter()
        .filter(|incident| {
            incident.kind == IncidentKind::Real
                && incident.status == IncidentStatus::Closed
                && incident.post_incident_review.is_none()
        })
        .collect()
}

/// Closed incidents that still cite corrective-action ids.
///
/// Until a Prompt 16 remediation inventory exists, every linked id is treated
/// as unresolved. Incident close does not close the action.
pub fn closed_incidents_with_open_corrective_actions(
    assessment: &AssessmentDefinition,
) -> Vec<&Incident> {
    assessment
        .incidents
        .iter()
        .filter(|incident| {
            incident.status == IncidentStatus::Closed && !incident.corrective_action_ids.is_empty()
        })
        .collect()
}

pub fn real_incidents(assessment: &AssessmentDefinition) -> Vec<&Incident> {
    assessment
        .incidents
        .iter()
        .filter(|incident| incident.kind == IncidentKind::Real)
        .collect()
}

pub fn exercise_incidents(assessment: &AssessmentDefinition) -> Vec<&Incident> {
    assessment
        .incidents
        .iter()
        .filter(|incident| incident.kind == IncidentKind::Exercise)
        .collect()
}
