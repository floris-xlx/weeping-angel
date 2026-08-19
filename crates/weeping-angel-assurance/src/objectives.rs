//! Pure security-objective evaluator. Side-effect free; `as_of` is an argument.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentScope, EvidenceCollectionKind, EvidenceType, FreshnessRequirement,
    ObjectiveMeasurementId, ObjectiveMetricId, ObjectiveTargetId, SecurityObjectiveId, SubjectKind,
    ValidateIr, canonical_digest,
};
use weeping_angel_evidence::EvidenceValue;

use crate::lineage::EvidenceSnapshot;
use crate::scope::{ScopeDecision, ScopeResolution};

pub use weeping_angel_assurance_ir::objectives::{
    ObjectiveLifecycle, ObjectiveMeasurementSource, PopulationCompleteness, SecurityObjective,
};
pub use weeping_angel_assurance_ir::{ComparisonOp, MetricKind};

pub const OBJECTIVE_EVALUATION_SCHEMA: &str = "weeping-angel/objective-evaluation/v1";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ObjectiveError {
    #[error("notActive: objective lifecycle is not active")]
    NotActive,
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectiveStatus {
    OnTrack,
    AtRisk,
    Missed,
    Achieved,
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveMetric {
    pub id: ObjectiveMetricId,
    pub kind: MetricKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_min: Option<EvidenceValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_max: Option<EvidenceValue>,
    pub evidence_type: EvidenceType,
    pub value_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness: Option<FreshnessRequirement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveTarget {
    pub id: ObjectiveTargetId,
    pub comparison: ComparisonOp,
    pub value: EvidenceValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveMeasurement {
    pub id: ObjectiveMeasurementId,
    pub objective_id: SecurityObjectiveId,
    pub observed_at: DateTime<Utc>,
    pub value: EvidenceValue,
    pub scope: AssessmentScope,
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation_ref: Option<String>,
    pub population_completeness: PopulationCompleteness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveEvaluationSnapshot {
    pub schema: String,
    pub objective_id: SecurityObjectiveId,
    pub metric_id: ObjectiveMetricId,
    pub target_id: ObjectiveTargetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_id: Option<ObjectiveMeasurementId>,
    pub objective_digest: String,
    pub metric_digest: String,
    pub target_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_digest: Option<String>,
    pub evidence_snapshot_digest: String,
    pub envelope_digests: Vec<String>,
    pub scope_digest: String,
    pub as_of: DateTime<Utc>,
    pub status: ObjectiveStatus,
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<ComparisonOp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_value: Option<EvidenceValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_value: Option<EvidenceValue>,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveEvaluation {
    pub status: ObjectiveStatus,
    pub reason_codes: Vec<String>,
    pub snapshot: ObjectiveEvaluationSnapshot,
}

pub fn evaluate_objective(
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    target: &ObjectiveTarget,
    measurements: &[ObjectiveMeasurement],
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
) -> Result<ObjectiveEvaluation, ObjectiveError> {
    evaluate_objective_with_resolution(
        objective,
        metric,
        target,
        measurements,
        as_of,
        evidence,
        None,
    )
}

/// Same pure projection with a pinned [`ScopeResolution`] when the scope engine is present.
pub fn evaluate_objective_with_resolution(
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    target: &ObjectiveTarget,
    measurements: &[ObjectiveMeasurement],
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
    resolution: Option<&ScopeResolution>,
) -> Result<ObjectiveEvaluation, ObjectiveError> {
    objective
        .validate()
        .map_err(|err| ObjectiveError::Invalid(err.to_string()))?;
    if objective.lifecycle != ObjectiveLifecycle::Active {
        return Err(ObjectiveError::NotActive);
    }

    let mut reasons: Vec<String> = Vec::new();
    let mut status = ObjectiveStatus::InsufficientEvidence;
    let mut candidate: Option<&ObjectiveMeasurement> = None;
    let mut comparison = None;
    let mut measured_value = None;
    let mut target_value = None;

    if objective.start_at.is_some_and(|start| start > as_of) {
        reasons.push("notStarted".into());
    } else {
        candidate = select_candidate(objective, measurements, as_of);
        if candidate.is_none() {
            if cadence_miss(objective, measurements, as_of) {
                reasons.push("staleMeasurement".into());
            } else {
                reasons.push("missingMeasurement".into());
            }
        } else if let Some(meas) = candidate {
            if let Some(code) =
                degradation_reason(objective, metric, meas, as_of, evidence, resolution)
            {
                reasons.push(code.into());
            } else {
                match compare_metric(metric, &meas.value, target) {
                    Ok(holds) => {
                        comparison = Some(target.comparison);
                        measured_value = Some(meas.value.clone());
                        target_value = Some(target.value.clone());
                        status = project_status(holds, objective.deadline_at, as_of);
                    }
                    Err(code) => reasons.push(code.into()),
                }
            }
        }
    }

    if !reasons.is_empty() {
        status = ObjectiveStatus::InsufficientEvidence;
    }

    let snapshot = seal_snapshot(ObjectiveEvaluationSnapshot {
        schema: OBJECTIVE_EVALUATION_SCHEMA.into(),
        objective_id: objective.id.clone(),
        metric_id: metric.id.clone(),
        target_id: target.id.clone(),
        measurement_id: candidate.map(|m| m.id.clone()),
        objective_digest: canonical_digest(objective).unwrap_or_default(),
        metric_digest: canonical_digest(metric).unwrap_or_default(),
        target_digest: canonical_digest(target).unwrap_or_default(),
        measurement_digest: candidate.and_then(|m| canonical_digest(m).ok()),
        evidence_snapshot_digest: evidence.digest.clone(),
        envelope_digests: candidate
            .map(|m| {
                let mut refs = m.evidence_refs.clone();
                refs.sort();
                refs.dedup();
                refs
            })
            .unwrap_or_default(),
        scope_digest: resolution
            .map(|r| r.digest.clone())
            .unwrap_or_else(|| canonical_digest(&objective.scope).unwrap_or_default()),
        as_of,
        status,
        reason_codes: reasons.clone(),
        comparison,
        measured_value,
        target_value,
        digest: String::new(),
    });

    Ok(ObjectiveEvaluation {
        status,
        reason_codes: reasons,
        snapshot,
    })
}

fn seal_snapshot(mut snapshot: ObjectiveEvaluationSnapshot) -> ObjectiveEvaluationSnapshot {
    snapshot.digest = String::new();
    snapshot.digest = canonical_digest(&snapshot).unwrap_or_default();
    snapshot
}

fn select_candidate<'a>(
    objective: &SecurityObjective,
    measurements: &'a [ObjectiveMeasurement],
    as_of: DateTime<Utc>,
) -> Option<&'a ObjectiveMeasurement> {
    let mut best: Option<&ObjectiveMeasurement> = None;
    for meas in measurements {
        if meas.objective_id != objective.id {
            continue;
        }
        if meas.observed_at > as_of {
            continue;
        }
        match best {
            None => best = Some(meas),
            Some(current) => {
                if meas.observed_at > current.observed_at
                    || (meas.observed_at == current.observed_at
                        && meas.id.as_str() > current.id.as_str())
                {
                    best = Some(meas);
                }
            }
        }
    }
    best
}

fn cadence_miss(
    objective: &SecurityObjective,
    measurements: &[ObjectiveMeasurement],
    as_of: DateTime<Utc>,
) -> bool {
    let Some(cadence) = objective.cadence_seconds else {
        return false;
    };
    if select_candidate(objective, measurements, as_of).is_some() {
        return false;
    }
    let has_any = measurements.iter().any(|m| m.objective_id == objective.id);
    has_any && window_start(objective, cadence, as_of).is_some()
}

fn window_start(
    objective: &SecurityObjective,
    cadence: u64,
    as_of: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    let start = objective.start_at?;
    let cadence_start = as_of - Duration::seconds(cadence as i64);
    Some(start.max(cadence_start))
}

fn in_progress(objective: &SecurityObjective, as_of: DateTime<Utc>) -> bool {
    match objective.deadline_at {
        Some(deadline) => as_of <= deadline,
        None => false,
    }
}

fn degradation_reason(
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    meas: &ObjectiveMeasurement,
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
    resolution: Option<&ScopeResolution>,
) -> Option<&'static str> {
    if in_progress(objective, as_of) {
        if let Some(freshness) = &metric.freshness {
            let age = (as_of - meas.observed_at).num_seconds();
            if age > freshness.max_age_seconds as i64 {
                return Some("staleMeasurement");
            }
        }
        if let Some(cadence) = objective.cadence_seconds
            && let Some(start) = window_start(objective, cadence, as_of)
            && meas.observed_at < start
        {
            return Some("staleMeasurement");
        }
    }

    match meas.population_completeness {
        PopulationCompleteness::Partial | PopulationCompleteness::Unknown => {
            return Some("partialEvidence");
        }
        PopulationCompleteness::Authoritative => {}
    }

    if meas.evidence_refs.is_empty() {
        return Some("partialEvidence");
    }

    if !scope_compatible(&objective.scope, &meas.scope, resolution) {
        return Some("scopeMismatch");
    }

    if objective.measurement_source.collection == EvidenceCollectionKind::Manual {
        match &meas.attestation_ref {
            Some(digest) if evidence.envelope_digests.iter().any(|d| d == digest) => {}
            _ => return Some("missingAttestation"),
        }
    }

    for digest in &meas.evidence_refs {
        if !evidence.envelope_digests.iter().any(|d| d == digest) {
            return Some("missingEvidence");
        }
    }
    None
}

fn scope_compatible(
    objective: &AssessmentScope,
    measurement: &AssessmentScope,
    resolution: Option<&ScopeResolution>,
) -> bool {
    let obj = in_scope_subjects(objective);
    let meas = in_scope_subjects(measurement);
    if meas.is_empty() {
        return false;
    }
    if !obj.is_empty() && !meas.is_subset(&obj) {
        return false;
    }
    if !objective.organizations.is_empty()
        && measurement
            .organizations
            .iter()
            .any(|org| !objective.organizations.contains(org))
    {
        return false;
    }
    if let Some(resolution) = resolution {
        for (kind, id) in &meas {
            let decision = resolution
                .subjects
                .iter()
                .find(|row| row.kind == *kind && row.id == *id)
                .map(|row| row.decision);
            if decision != Some(ScopeDecision::InScope) {
                return false;
            }
        }
    }
    true
}

fn in_scope_subjects(scope: &AssessmentScope) -> BTreeSet<(SubjectKind, String)> {
    let mut included = BTreeSet::new();
    for selector in &scope.subjects {
        for id in &selector.ids {
            included.insert((selector.kind, id.clone()));
        }
    }
    for exclusion in &scope.exclusions {
        for selector in &exclusion.subjects {
            for id in &selector.ids {
                included.remove(&(selector.kind, id.clone()));
            }
        }
    }
    included
}

fn project_status(
    holds: bool,
    deadline: Option<DateTime<Utc>>,
    as_of: DateTime<Utc>,
) -> ObjectiveStatus {
    let past_deadline = deadline.is_some_and(|d| as_of > d);
    match (holds, past_deadline) {
        (true, false) => ObjectiveStatus::OnTrack,
        (true, true) => ObjectiveStatus::Achieved,
        (false, false) => ObjectiveStatus::AtRisk,
        (false, true) => ObjectiveStatus::Missed,
    }
}

fn compare_metric(
    metric: &ObjectiveMetric,
    measured: &EvidenceValue,
    target: &ObjectiveTarget,
) -> Result<bool, &'static str> {
    let left = typed_measured(metric, measured)?;
    let right = typed_target(metric, &target.value)?;
    if metric.kind == MetricKind::Boolean {
        if !matches!(target.comparison, ComparisonOp::Eq | ComparisonOp::Neq) {
            return Err("typeMismatch");
        }
        let eq = left.typed_eq(&right).map_err(|_| "typeMismatch")?;
        return Ok(match target.comparison {
            ComparisonOp::Neq => !eq,
            _ => eq,
        });
    }
    let ord = left.cmp_numeric(&right).map_err(|_| "typeMismatch")?;
    Ok(match target.comparison {
        ComparisonOp::Eq => ord == Ordering::Equal,
        ComparisonOp::Neq => ord != Ordering::Equal,
        ComparisonOp::Gt => ord == Ordering::Greater,
        ComparisonOp::Gte => ord != Ordering::Less,
        ComparisonOp::Lt => ord == Ordering::Less,
        ComparisonOp::Lte => ord != Ordering::Greater,
    })
}

fn typed_measured(
    metric: &ObjectiveMetric,
    value: &EvidenceValue,
) -> Result<EvidenceValue, &'static str> {
    let comparable = match metric.kind {
        MetricKind::Percentage => match value {
            EvidenceValue::Integer(_) | EvidenceValue::Decimal(_) => value.clone(),
            _ => return Err("typeMismatch"),
        },
        MetricKind::Count => match value {
            EvidenceValue::Integer(n) if *n >= 0 => value.clone(),
            EvidenceValue::Integer(_) => return Err("outOfDomain"),
            _ => return Err("typeMismatch"),
        },
        MetricKind::Duration => match value {
            EvidenceValue::DurationSeconds(_) => value.clone(),
            _ => return Err("typeMismatch"),
        },
        MetricKind::Boolean => match value {
            EvidenceValue::Bool(_) => value.clone(),
            _ => return Err("typeMismatch"),
        },
        MetricKind::Ratio => ratio_as_decimal(value)?,
        MetricKind::BoundedNumeric => match value {
            EvidenceValue::Integer(_) | EvidenceValue::Decimal(_) => value.clone(),
            _ => return Err("typeMismatch"),
        },
    };
    in_domain(metric, &comparable)?;
    Ok(comparable)
}

fn typed_target(
    metric: &ObjectiveMetric,
    value: &EvidenceValue,
) -> Result<EvidenceValue, &'static str> {
    match metric.kind {
        MetricKind::Ratio => match value {
            EvidenceValue::Object(_) => ratio_as_decimal(value),
            EvidenceValue::Integer(_) | EvidenceValue::Decimal(_) => Ok(value.clone()),
            _ => Err("typeMismatch"),
        },
        MetricKind::Boolean => match value {
            EvidenceValue::Bool(_) => Ok(value.clone()),
            _ => Err("typeMismatch"),
        },
        MetricKind::Duration => match value {
            EvidenceValue::DurationSeconds(_) => Ok(value.clone()),
            _ => Err("typeMismatch"),
        },
        MetricKind::Count => match value {
            EvidenceValue::Integer(_) => Ok(value.clone()),
            _ => Err("typeMismatch"),
        },
        MetricKind::Percentage | MetricKind::BoundedNumeric => match value {
            EvidenceValue::Integer(_) | EvidenceValue::Decimal(_) => Ok(value.clone()),
            _ => Err("typeMismatch"),
        },
    }
}

fn in_domain(metric: &ObjectiveMetric, value: &EvidenceValue) -> Result<(), &'static str> {
    if metric.kind == MetricKind::Percentage {
        let zero = EvidenceValue::integer(0);
        let hundred = EvidenceValue::integer(100);
        let lo = value.cmp_numeric(&zero).map_err(|_| "typeMismatch")?;
        let hi = value.cmp_numeric(&hundred).map_err(|_| "typeMismatch")?;
        if lo == Ordering::Less || hi == Ordering::Greater {
            return Err("outOfDomain");
        }
    }
    if let Some(min) = &metric.domain_min {
        let ord = value.cmp_numeric(min).map_err(|_| "typeMismatch")?;
        if ord == Ordering::Less {
            return Err("outOfDomain");
        }
    }
    if let Some(max) = &metric.domain_max {
        let ord = value.cmp_numeric(max).map_err(|_| "typeMismatch")?;
        if ord == Ordering::Greater {
            return Err("outOfDomain");
        }
    }
    Ok(())
}

fn ratio_as_decimal(value: &EvidenceValue) -> Result<EvidenceValue, &'static str> {
    let EvidenceValue::Object(map) = value else {
        return Err("typeMismatch");
    };
    let numerator = object_integer(map, "numerator")?;
    let denominator = object_integer(map, "denominator")?;
    if denominator == 0 {
        return Err("outOfDomain");
    }
    if numerator < 0 || denominator < 0 {
        return Err("outOfDomain");
    }
    let text = ratio_decimal_text(numerator, denominator);
    EvidenceValue::decimal(text).map_err(|_| "typeMismatch")
}

fn object_integer(map: &BTreeMap<String, EvidenceValue>, key: &str) -> Result<i64, &'static str> {
    match map.get(key) {
        Some(EvidenceValue::Integer(n)) => Ok(*n),
        Some(_) => Err("typeMismatch"),
        None => Err("typeMismatch"),
    }
}

fn ratio_decimal_text(numerator: i64, denominator: i64) -> String {
    let n = numerator as u128;
    let d = denominator as u128;
    let integer = n / d;
    let mut rem = n % d;
    let mut frac = String::new();
    while rem != 0 && frac.len() < 40 {
        rem *= 10;
        let digit = rem / d;
        frac.push(char::from(b'0' + digit as u8));
        rem %= d;
    }
    if frac.is_empty() {
        integer.to_string()
    } else {
        format!("{integer}.{frac}")
    }
}
