//! Timeline and temporal-diff primitives for readiness and audit library exports.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_control_test::{EvidenceSet, TemporalQuery};
use weeping_angel_control_test::{PeriodEffectiveness, TimeRange, select_evidence};
use weeping_angel_evidence::EvidenceValidityKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineInterval {
    pub envelope_digest: String,
    pub event_id: String,
    pub valid_from: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    pub kind: String,
    pub observed_at: DateTime<Utc>,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceTimeline {
    pub subject: String,
    pub evidence_type: String,
    pub intervals: Vec<TimelineInterval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemporalDiff {
    pub observation_gaps: Vec<TimeRange>,
    pub expired_at: Vec<String>,
    pub revoked: Vec<String>,
    pub superseded: Vec<String>,
    pub intermittent_controls: Vec<String>,
    pub coverage_insufficient: Vec<String>,
}

pub fn project_timeline(
    set: &EvidenceSet,
    range: TimeRange,
    evidence_type: Option<&str>,
    subject: Option<&str>,
) -> EvidenceTimeline {
    let query = TemporalQuery {
        evidence_type: evidence_type.map(weeping_angel_evidence::EvidenceType::new),
        subject: subject.map(str::to_string),
        as_of: None,
        range: Some(range),
        include_revoked: true,
    };
    let selected = select_evidence(set, &query);
    let mut intervals = Vec::new();
    for env in selected {
        let events: Vec<_> = set
            .validity_events()
            .iter()
            .filter(|e| e.envelope_digest == env.digest())
            .cloned()
            .collect();
        let event_id = events
            .last()
            .map(|e| e.event_id.clone())
            .unwrap_or_default();
        intervals.push(TimelineInterval {
            envelope_digest: env.digest().to_string(),
            event_id,
            valid_from: env.valid_from(),
            valid_until: env.valid_until(),
            kind: "asserted".into(),
            observed_at: env.observed_at(),
            collected_at: env.provenance().collected_at,
        });
    }
    intervals.sort_by(|a, b| {
        a.valid_from
            .cmp(&b.valid_from)
            .then_with(|| a.envelope_digest.cmp(&b.envelope_digest))
    });
    EvidenceTimeline {
        subject: subject.unwrap_or("").into(),
        evidence_type: evidence_type.unwrap_or("").into(),
        intervals,
    }
}

pub fn compare_temporal(
    range: TimeRange,
    set: &EvidenceSet,
    period_by_control: &[(String, PeriodEffectiveness)],
) -> TemporalDiff {
    let mut diff = TemporalDiff::default();
    let mut covered = false;
    for env in set.iter() {
        if env.valid_from() < range.end && env.valid_until().is_none_or(|u| range.start < u) {
            covered = true;
        }
        if env
            .valid_until()
            .is_some_and(|u| range.contains(u) || u <= range.end)
        {
            diff.expired_at.push(env.digest().to_string());
        }
        if env.supersedes().is_some() {
            diff.superseded.push(env.digest().to_string());
        }
    }
    for event in set.validity_events() {
        if range.contains(event.at) {
            match event.kind {
                EvidenceValidityKind::Revoked | EvidenceValidityKind::Invalidated => {
                    diff.revoked.push(event.event_id.clone());
                }
                EvidenceValidityKind::Superseded => {
                    diff.superseded.push(event.envelope_digest.clone());
                }
                EvidenceValidityKind::Asserted => {}
            }
        }
    }
    if !covered {
        diff.observation_gaps.push(range);
    }
    for (id, period) in period_by_control {
        match period {
            PeriodEffectiveness::IntermittentRegression => {
                diff.intermittent_controls.push(id.clone());
            }
            PeriodEffectiveness::InsufficientObservationCoverage => {
                diff.coverage_insufficient.push(id.clone());
            }
            _ => {}
        }
    }
    diff
}

pub fn diff_period(
    range: TimeRange,
    set: &EvidenceSet,
    period_by_control: &[(String, PeriodEffectiveness)],
) -> TemporalDiff {
    compare_temporal(range, set, period_by_control)
}
