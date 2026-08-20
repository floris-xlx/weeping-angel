//! As-of selection, period projection, and the Prompt 13 freshness/clock seam.

use std::collections::{BTreeSet, HashSet};
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_evidence::validity::select_valid_leaf_as_of;
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType, project_validity};

use crate::expr::{EvidenceSelector, TestExpr, ValueExpr};
use crate::{AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSet};

/// Half-open evaluation range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn contains(self, t: DateTime<Utc>) -> bool {
        self.start <= t && t < self.end
    }
}

/// Scheduler handoff. Cadence/retry/daemon live in Prompt 13, not here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreshnessPolicy {
    pub max_age: Duration,
    pub as_of: DateTime<Utc>,
    pub period: Option<TimeRange>,
}

impl FreshnessPolicy {
    pub fn at(as_of: DateTime<Utc>, max_age: Duration) -> Self {
        Self {
            max_age,
            as_of,
            period: None,
        }
    }

    pub fn into_context(self) -> AssessmentContext {
        AssessmentContext {
            now: self.as_of,
            max_age: self.max_age,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemporalSemantics {
    Instant,
    Interval,
    ContinuousUntilSuperseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PeriodEffectiveness {
    ContinuouslyEffective,
    IntermittentRegression,
    InsufficientObservationCoverage,
    Ineffective,
    ManualReviewRequired,
}

#[derive(Debug, Clone, Default)]
pub struct TemporalQuery {
    pub evidence_type: Option<EvidenceType>,
    pub subject: Option<String>,
    pub as_of: Option<DateTime<Utc>>,
    pub range: Option<TimeRange>,
    pub include_revoked: bool,
}

fn implicit_period(context: &AssessmentContext) -> TimeRange {
    let delta = TimeDelta::from_std(context.max_age).unwrap_or_else(|_| TimeDelta::zero());
    TimeRange {
        start: context.now - delta,
        end: context.now,
    }
}

pub fn select_latest_as_of<'a>(
    group: &[&'a EvidenceEnvelope],
    as_of: DateTime<Utc>,
    set: &EvidenceSet,
) -> Option<&'a EvidenceEnvelope> {
    select_valid_leaf_as_of(group.iter().copied(), as_of, set.validity_events())
}

pub fn select_evidence<'a>(
    set: &'a EvidenceSet,
    query: &TemporalQuery,
) -> Vec<&'a EvidenceEnvelope> {
    let events = set.validity_events();
    let mut out: Vec<&'a EvidenceEnvelope> = set
        .iter()
        .filter(|env| {
            if let Some(ty) = &query.evidence_type
                && env.observation().evidence_type() != ty
            {
                return false;
            }
            if let Some(subject) = &query.subject
                && env.provenance().asset().as_str() != subject
            {
                return false;
            }
            if let Some(t) = query.as_of {
                if query.include_revoked {
                    return env.provenance().collected_at <= t && env.observed_at() <= t;
                }
                return project_validity(env, events, t).is_some();
            }
            if let Some(range) = query.range {
                if query.include_revoked {
                    return env.provenance().collected_at < range.end;
                }
                return project_validity(env, events, range.start).is_some()
                    || (env.valid_from() < range.end
                        && env.valid_until().is_none_or(|u| range.start < u)
                        && env.provenance().collected_at < range.end);
            }
            true
        })
        .collect();
    out.sort_by(|a, b| {
        a.observed_at()
            .cmp(&b.observed_at())
            .then_with(|| {
                a.provenance()
                    .collected_at
                    .cmp(&b.provenance().collected_at)
            })
            .then_with(|| a.digest().cmp(b.digest()))
    });
    out
}

pub fn project_period_effectiveness(
    test: &CompiledControlTest,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> PeriodEffectiveness {
    if test.kind == ControlTestKind::Manual || matches!(test.expr, Some(TestExpr::ManualReview)) {
        return PeriodEffectiveness::ManualReviewRequired;
    }
    let period = context.period().unwrap_or_else(|| implicit_period(context));
    if period.start >= period.end {
        return match point_effectiveness(test, evidence, context.as_of(), context.max_age) {
            Effectiveness::Effective => PeriodEffectiveness::InsufficientObservationCoverage,
            Effectiveness::Ineffective => PeriodEffectiveness::Ineffective,
            Effectiveness::ManualReviewRequired => PeriodEffectiveness::ManualReviewRequired,
            _ => PeriodEffectiveness::InsufficientObservationCoverage,
        };
    }

    let semantics = TemporalSemantics::Instant;
    let samples = sample_instants(evidence, period);
    let mut saw_pass = false;
    let mut saw_fail = false;
    let mut saw_gap = false;
    for t in samples {
        match point_effectiveness(test, evidence, t, context.max_age) {
            Effectiveness::Effective | Effectiveness::ExceptionApproved => saw_pass = true,
            Effectiveness::Ineffective => saw_fail = true,
            Effectiveness::ManualReviewRequired => {
                return PeriodEffectiveness::ManualReviewRequired;
            }
            Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence => saw_gap = true,
            _ => {}
        }
    }

    if saw_pass && saw_fail {
        return PeriodEffectiveness::IntermittentRegression;
    }
    if saw_fail && !saw_pass {
        return PeriodEffectiveness::Ineffective;
    }
    if matches!(semantics, TemporalSemantics::Instant) && period.start < period.end {
        return PeriodEffectiveness::InsufficientObservationCoverage;
    }
    if saw_gap || !saw_pass {
        return PeriodEffectiveness::InsufficientObservationCoverage;
    }
    PeriodEffectiveness::ContinuouslyEffective
}

fn sample_instants(evidence: &EvidenceSet, period: TimeRange) -> Vec<DateTime<Utc>> {
    let mut set = BTreeSet::new();
    set.insert(period.start);
    for env in evidence.iter() {
        for t in [
            env.observed_at(),
            env.valid_from(),
            env.provenance().collected_at,
        ] {
            if period.contains(t) {
                set.insert(t);
            }
        }
        if let Some(until) = env.valid_until()
            && period.contains(until)
        {
            set.insert(until);
        }
    }
    for event in evidence.validity_events() {
        if period.contains(event.at) {
            set.insert(event.at);
        }
    }
    if let Some(last) = period.end.checked_sub_signed(TimeDelta::nanoseconds(1))
        && last >= period.start
    {
        set.insert(last);
    }
    set.into_iter().collect()
}

fn point_effectiveness(
    test: &CompiledControlTest,
    evidence: &EvidenceSet,
    at: DateTime<Utc>,
    max_age: Duration,
) -> Effectiveness {
    let ctx = AssessmentContext { now: at, max_age };
    let envelopes: Vec<&EvidenceEnvelope> = evidence.iter().collect();
    match &test.expr {
        Some(TestExpr::Exists(sel)) => match select_for_selector(&envelopes, sel, evidence, &ctx) {
            None => Effectiveness::InsufficientEvidence,
            Some(env) if crate::is_stale_candidate(env, &ctx) => Effectiveness::StaleEvidence,
            Some(_) => Effectiveness::Effective,
        },
        Some(TestExpr::Eq(ValueExpr::Field(sel), expected)) => {
            match select_for_selector(&envelopes, sel, evidence, &ctx) {
                None => Effectiveness::InsufficientEvidence,
                Some(env) if crate::is_stale_candidate(env, &ctx) => Effectiveness::StaleEvidence,
                Some(env) => {
                    let field = sel.field.as_deref().unwrap_or("value");
                    match env.observation().fact_value(field) {
                        Some(have) => match have.typed_eq(expected) {
                            Ok(true) => Effectiveness::Effective,
                            _ => Effectiveness::Ineffective,
                        },
                        None => Effectiveness::InsufficientEvidence,
                    }
                }
            }
        }
        Some(TestExpr::ManualReview) => Effectiveness::ManualReviewRequired,
        _ => {
            let mut seen = HashSet::new();
            let mut any = false;
            for env in envelopes {
                let ty = env.observation().evidence_type().as_str();
                let sub = env.provenance().asset().as_str();
                if !seen.insert((ty, sub)) {
                    continue;
                }
                if select_latest_as_of(&[env], at, evidence).is_some() {
                    any = true;
                }
            }
            if any {
                Effectiveness::Effective
            } else {
                Effectiveness::InsufficientEvidence
            }
        }
    }
}

pub(crate) fn select_for_selector<'a>(
    envelopes: &[&'a EvidenceEnvelope],
    sel: &EvidenceSelector,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> Option<&'a EvidenceEnvelope> {
    let matching: Vec<&'a EvidenceEnvelope> = envelopes
        .iter()
        .copied()
        .filter(|env| {
            env.observation().evidence_type() == &sel.evidence_type
                && sel
                    .subject_selector
                    .id
                    .as_ref()
                    .is_none_or(|id| env.provenance().asset().as_str() == id)
        })
        .collect();
    select_latest_as_of(&matching, context.as_of(), evidence)
}

/// Distinct from stale (`max_age`): expired means outside validity (`valid_until`).
/// Stale is policy freshness of a still-valid candidate; expired is outside validity.
pub fn period_for_evaluate(
    context: &AssessmentContext,
    _evidence: &EvidenceSet,
    effectiveness: Effectiveness,
    _semantics: TemporalSemantics,
    _flag: bool,
) -> PeriodEffectiveness {
    match effectiveness {
        Effectiveness::ManualReviewRequired => PeriodEffectiveness::ManualReviewRequired,
        Effectiveness::Ineffective => PeriodEffectiveness::Ineffective,
        _ => {
            let _period = context.period().unwrap_or_else(|| implicit_period(context));
            PeriodEffectiveness::InsufficientObservationCoverage
        }
    }
}

pub fn selector_as_of<'a>(
    evidence: &'a EvidenceSet,
    sel: &EvidenceSelector,
    as_of: DateTime<Utc>,
) -> Option<&'a EvidenceEnvelope> {
    let envelopes: Vec<_> = evidence.iter().collect();
    let context = AssessmentContext {
        now: as_of,
        max_age: Duration::MAX,
    };
    select_for_selector(&envelopes, sel, evidence, &context)
}
