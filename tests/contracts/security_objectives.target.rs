//! Target suite for the security objectives engine.
//!
//! Encodes DESIRED behavior in `docs/specs/security-objectives.md` §4 / §6.2
//! (SO-T01–T20). Must stay RED on CURRENT HEAD: no measurable
//! `SecurityObjective` / `ObjectiveMetric` / `ObjectiveTarget` /
//! `ObjectiveMeasurement` records, no `evaluate_objective`, and no
//! `weeping-angel/objective-evaluation/v1` snapshot. A concurrent ISMS
//! context declaration named `SecurityObjective` (id/title/description/owner)
//! is not this engine. Do not weaken these assertions to match prose-only
//! `Control.objective`, and do not implement the evaluator in this suite.
//!
//! Imports the intended public contract. On characterization HEAD the binary
//! fails because those types/APIs are missing (not because of a missing
//! harness). After implement, the same fixtures must go GREEN.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::lineage::{EvidenceSnapshot, seal_evidence_snapshot};
use weeping_angel_assurance::objectives::{
    ComparisonOp, MetricKind, ObjectiveError, ObjectiveEvaluation, ObjectiveEvaluationSnapshot,
    ObjectiveLifecycle, ObjectiveMeasurement, ObjectiveMeasurementSource, ObjectiveMetric,
    ObjectiveStatus, ObjectiveTarget, PopulationCompleteness, SecurityObjective,
    evaluate_objective,
};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentScope, Control, ControlId, EvidenceCollectionKind,
    EvidenceRequirementId, EvidenceType, FreshnessRequirement, ObjectiveMeasurementId,
    ObjectiveMetricId, ObjectiveTargetId, PrincipalRef, ScopeExclusion, SecurityObjectiveId,
    SelectorScope, SubjectKind, SubjectSelector, ValidateIr, canonical_digest,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_evidence::{EvidenceValue, looks_like_compliance_claim};

const EVAL_SCHEMA: &str = "weeping-angel/objective-evaluation/v1";
const SLA_OBJECTIVE: &str = "objective.vuln.critical-sla";
const SLA_METRIC: &str = "metric.vuln.critical-sla";
const SLA_TARGET: &str = "target.vuln.critical-sla.gte-98";
const SLA_FIELD: &str = "remediated_within_sla_percent";
const VULN_EVIDENCE: &str = "evidence.vulnerability.remediation";
const MANUAL_EVIDENCE: &str = "evidence.manual.attestation";
const IN_SCOPE_REPO: &str = "repo:payments-api";
const OUT_OF_SCOPE_REPO: &str = "repo:sandbox-lab";
const ENVELOPE: &str = "ev:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ATTESTATION: &str =
    "ev:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CADENCE_30D: u64 = 2_592_000;
const FRESHNESS_7D: u64 = 7 * 24 * 3600;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let mut chunks = Vec::new();
    for entry in fs::read_dir(&crates_dir).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
}

fn start_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
}

fn deadline_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 30, 0, 0, 0).unwrap()
}

fn as_of_before_deadline() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap()
}

fn as_of_after_deadline() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 1, 12, 0, 0).unwrap()
}

fn observed_fresh() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 28, 12, 0, 0).unwrap()
}

fn repo_selector(ids: &[&str]) -> SubjectSelector {
    SubjectSelector {
        kind: SubjectKind::Repository,
        ids: ids.iter().map(|id| (*id).to_string()).collect(),
        tags: BTreeMap::new(),
        scope: SelectorScope::AnyOf,
    }
}

fn isms_scope(ids: &[&str]) -> AssessmentScope {
    AssessmentScope {
        organizations: vec!["org:acme".into()],
        subjects: vec![repo_selector(ids)],
        exclusions: Vec::new(),
    }
}

fn pinned_evidence(digests: &[&str]) -> EvidenceSnapshot {
    seal_evidence_snapshot(
        digests.iter().map(|d| (*d).to_string()),
        ["run.objectives.target".to_string()],
    )
}

fn sla_metric() -> ObjectiveMetric {
    ObjectiveMetric {
        id: ObjectiveMetricId::new(SLA_METRIC),
        kind: MetricKind::Percentage,
        unit: Some("percent".into()),
        domain_min: Some(EvidenceValue::integer(0)),
        domain_max: Some(EvidenceValue::integer(100)),
        evidence_type: EvidenceType::new(VULN_EVIDENCE),
        value_field: SLA_FIELD.into(),
        freshness: Some(FreshnessRequirement {
            max_age_seconds: FRESHNESS_7D,
        }),
    }
}

fn sla_target(value: EvidenceValue) -> ObjectiveTarget {
    ObjectiveTarget {
        id: ObjectiveTargetId::new(SLA_TARGET),
        comparison: ComparisonOp::Gte,
        value,
    }
}

fn sla_objective(deadline: Option<DateTime<Utc>>) -> SecurityObjective {
    SecurityObjective {
        id: SecurityObjectiveId::new(SLA_OBJECTIVE),
        schema_version: ASSURANCE_IR_SCHEMA.into(),
        title: "Critical vulnerabilities remediated within seven days".into(),
        description: "Percentage of in-scope critical vulnerabilities remediated within 7 days."
            .into(),
        owner: Some(PrincipalRef::Team("security-governance".into())),
        scope: isms_scope(&[IN_SCOPE_REPO]),
        metric_id: ObjectiveMetricId::new(SLA_METRIC),
        target_id: ObjectiveTargetId::new(SLA_TARGET),
        baseline: None,
        measurement_source: ObjectiveMeasurementSource {
            evidence_type: EvidenceType::new(VULN_EVIDENCE),
            collection: EvidenceCollectionKind::Automated,
            evidence_requirement_id: Some(EvidenceRequirementId::new(VULN_EVIDENCE)),
        },
        cadence_seconds: Some(CADENCE_30D),
        start_at: Some(start_at()),
        deadline_at: deadline,
        review_at: None,
        lifecycle: ObjectiveLifecycle::Active,
        logical_id: SLA_OBJECTIVE.into(),
        revision: 1,
        supersedes: None,
    }
}

fn measurement(
    id: &str,
    observed_at: DateTime<Utc>,
    value: EvidenceValue,
    scope: AssessmentScope,
    completeness: PopulationCompleteness,
    evidence_refs: &[&str],
    attestation_ref: Option<&str>,
) -> ObjectiveMeasurement {
    ObjectiveMeasurement {
        id: ObjectiveMeasurementId::new(id),
        objective_id: SecurityObjectiveId::new(SLA_OBJECTIVE),
        observed_at,
        value,
        scope,
        evidence_refs: evidence_refs.iter().map(|d| (*d).to_string()).collect(),
        attestation_ref: attestation_ref.map(str::to_string),
        population_completeness: completeness,
    }
}

fn sla_measurement(id: &str, value: EvidenceValue) -> ObjectiveMeasurement {
    measurement(
        id,
        observed_fresh(),
        value,
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    )
}

fn project(
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    target: &ObjectiveTarget,
    measurements: &[ObjectiveMeasurement],
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
) -> Result<ObjectiveEvaluation, ObjectiveError> {
    evaluate_objective(objective, metric, target, measurements, as_of, evidence)
}

fn project_ok(
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    target: &ObjectiveTarget,
    measurements: &[ObjectiveMeasurement],
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
) -> ObjectiveEvaluation {
    project(objective, metric, target, measurements, as_of, evidence)
        .unwrap_or_else(|err| panic!("expected evaluation, got {err}"))
}

fn is_success(status: ObjectiveStatus) -> bool {
    matches!(status, ObjectiveStatus::OnTrack | ObjectiveStatus::Achieved)
}

fn reason_blob(eval: &ObjectiveEvaluation) -> String {
    let mut parts: Vec<String> = eval.reason_codes.iter().map(|c| c.to_string()).collect();
    parts.extend(eval.snapshot.reason_codes.iter().map(|c| c.to_string()));
    parts.join(" ")
}

fn assert_reason(eval: &ObjectiveEvaluation, code: &str) {
    let blob = reason_blob(eval);
    assert!(
        blob.contains(code),
        "expected reason `{code}` in `{blob}` (status {:?})",
        eval.status
    );
}

fn assert_never_success(eval: &ObjectiveEvaluation, why: &str) {
    assert!(
        !is_success(eval.status),
        "{why}: missing/stale/partial/unscoped evidence must not yield {:?}; got {:?}",
        ObjectiveStatus::OnTrack,
        eval.status
    );
    assert_eq!(
        eval.status,
        ObjectiveStatus::InsufficientEvidence,
        "{why}: degradation must be InsufficientEvidence, got {:?}",
        eval.status
    );
}

fn assert_lineage(
    eval: &ObjectiveEvaluation,
    objective: &SecurityObjective,
    metric: &ObjectiveMetric,
    target: &ObjectiveTarget,
    as_of: DateTime<Utc>,
    evidence: &EvidenceSnapshot,
) {
    let snap: &ObjectiveEvaluationSnapshot = &eval.snapshot;
    assert_eq!(snap.schema, EVAL_SCHEMA);
    assert_eq!(snap.objective_id, objective.id);
    assert_eq!(snap.metric_id, metric.id);
    assert_eq!(snap.target_id, target.id);
    assert_eq!(snap.as_of, as_of);
    assert_eq!(snap.status, eval.status);
    assert_eq!(snap.evidence_snapshot_digest, evidence.digest);
    assert!(
        !snap.objective_digest.is_empty(),
        "management review needs an objective document digest"
    );
    assert!(
        !snap.metric_digest.is_empty(),
        "management review needs a metric digest"
    );
    assert!(
        !snap.target_digest.is_empty(),
        "management review needs a target encoding digest"
    );
    assert!(
        !snap.scope_digest.is_empty(),
        "management review needs a scope digest"
    );
    assert_eq!(
        snap.objective_digest,
        canonical_digest(objective).expect("objective digest")
    );
    assert!(
        !snap.evidence_snapshot_digest.is_empty(),
        "replay pins EvidenceSnapshot.digest"
    );
}

/// SO: types exist — SecurityObjective, ObjectiveMetric, ObjectiveTarget, ObjectiveMeasurement, ObjectiveStatus, evaluate_objective
#[test]
fn so_t01_types_and_evaluate_objective_exist() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let measured = sla_measurement("meas.sla.t01", EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    objective
        .validate()
        .expect("active SLA objective with owner, scope, and start must validate");
    let eval = project_ok(
        &objective,
        &metric,
        &target,
        &[measured],
        as_of_before_deadline(),
        &evidence,
    );
    assert!(
        matches!(
            eval.status,
            ObjectiveStatus::OnTrack
                | ObjectiveStatus::AtRisk
                | ObjectiveStatus::Missed
                | ObjectiveStatus::Achieved
                | ObjectiveStatus::InsufficientEvidence
        ),
        "ObjectiveStatus must be the five-variant projection, got {:?}",
        eval.status
    );
    assert_lineage(
        &eval,
        &objective,
        &metric,
        &target,
        as_of_before_deadline(),
        &evidence,
    );
}

/// SO: percentage boundary 98 vs target gte 98 is success path; 97 is not success
#[test]
fn so_t02_percentage_boundary_98_vs_97() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let at = as_of_before_deadline();

    let on_track = project_ok(
        &objective,
        &metric,
        &target,
        &[sla_measurement(
            "meas.sla.t02.98",
            EvidenceValue::integer(98),
        )],
        at,
        &evidence,
    );
    assert_eq!(on_track.status, ObjectiveStatus::OnTrack);

    let at_risk = project_ok(
        &objective,
        &metric,
        &target,
        &[sla_measurement(
            "meas.sla.t02.97",
            EvidenceValue::integer(97),
        )],
        at,
        &evidence,
    );
    assert_eq!(at_risk.status, ObjectiveStatus::AtRisk);
    assert!(
        !is_success(at_risk.status),
        "97% against gte 98 must not be OnTrack/Achieved"
    );

    let not_rounded = project_ok(
        &objective,
        &metric,
        &target,
        &[sla_measurement(
            "meas.sla.t02.97999",
            EvidenceValue::decimal("97.999").expect("canonical 97.999"),
        )],
        at,
        &evidence,
    );
    assert_eq!(
        not_rounded.status,
        ObjectiveStatus::AtRisk,
        "97.999 must not round up to the 98% boundary"
    );
}

/// SO: Integer↔Decimal 98 vs 98.0 compares equal via EvidenceValue::cmp_numeric; no f64
#[test]
fn so_t03_integer_decimal_98_cmp_numeric_no_f64() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::decimal("98.0").expect("canonical 98.0"));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let eval = project_ok(
        &objective,
        &metric,
        &target,
        &[sla_measurement("meas.sla.t03", EvidenceValue::integer(98))],
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(eval.status, ObjectiveStatus::OnTrack);
    assert_eq!(
        EvidenceValue::integer(98)
            .cmp_numeric(&EvidenceValue::decimal("98.0").unwrap())
            .expect("integer↔decimal"),
        std::cmp::Ordering::Equal
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    let assurance = crate_sources_joined("weeping-angel-assurance");
    for (label, src) in [("ir", ir.as_str()), ("assurance", assurance.as_str())] {
        if let Some(module) = src
            .split("pub mod objectives")
            .nth(1)
            .or_else(|| src.contains("fn evaluate_objective").then_some(src))
        {
            assert!(
                !module.contains("f64") && !module.contains("f32"),
                "{label} objective comparison must not use f64/f32"
            );
        }
    }
    let objectives_rs = crate_src("weeping-angel-assurance").join("objectives.rs");
    let body = fs::read_to_string(&objectives_rs)
        .unwrap_or_else(|e| panic!("assurance objectives.rs must exist for typed comparison: {e}"));
    assert!(
        body.contains("cmp_numeric") && body.contains("typed_eq"),
        "evaluator must compare through EvidenceValue::typed_eq / cmp_numeric"
    );
    assert!(
        !body.contains("f64") && !body.contains("f32"),
        "evaluator must not introduce a floating metric type"
    );
}

fn kind_metric(kind: MetricKind, unit: &str, field: &str) -> ObjectiveMetric {
    let (min, max) = match kind {
        MetricKind::Percentage => (
            Some(EvidenceValue::integer(0)),
            Some(EvidenceValue::integer(100)),
        ),
        MetricKind::BoundedNumeric => (
            Some(EvidenceValue::integer(0)),
            Some(EvidenceValue::integer(10)),
        ),
        _ => (None, None),
    };
    ObjectiveMetric {
        id: ObjectiveMetricId::new(format!("metric.kind.{field}")),
        kind,
        unit: Some(unit.into()),
        domain_min: min,
        domain_max: max,
        evidence_type: EvidenceType::new(VULN_EVIDENCE),
        value_field: field.into(),
        freshness: None,
    }
}

fn kind_objective(id: &str, metric_id: &str, target_id: &str) -> SecurityObjective {
    let mut objective = sla_objective(None);
    objective.id = SecurityObjectiveId::new(id);
    objective.logical_id = id.into();
    objective.metric_id = ObjectiveMetricId::new(metric_id);
    objective.target_id = ObjectiveTargetId::new(target_id);
    objective.cadence_seconds = None;
    objective
}

fn kind_measurement(
    objective_id: &str,
    meas_id: &str,
    value: EvidenceValue,
) -> ObjectiveMeasurement {
    let mut m = sla_measurement(meas_id, value);
    m.objective_id = SecurityObjectiveId::new(objective_id);
    m
}

/// SO: count / duration / boolean / ratio / bounded-numeric each have a boundary; ratio denominator 0 is InsufficientEvidence
#[test]
fn so_t04_count_duration_boolean_ratio_bounded_numeric_boundaries() {
    let evidence = pinned_evidence(&[ENVELOPE]);
    let at = as_of_before_deadline();

    let count_metric = kind_metric(MetricKind::Count, "count", "finding_count");
    let count_obj = kind_objective(
        "objective.kind.count",
        count_metric.id.as_str(),
        "target.kind.count",
    );
    let count_target = ObjectiveTarget {
        id: ObjectiveTargetId::new("target.kind.count"),
        comparison: ComparisonOp::Gte,
        value: EvidenceValue::integer(10),
    };
    let count_ok = project_ok(
        &count_obj,
        &count_metric,
        &count_target,
        &[kind_measurement(
            count_obj.id.as_str(),
            "meas.kind.count.10",
            EvidenceValue::integer(10),
        )],
        at,
        &evidence,
    );
    assert_eq!(count_ok.status, ObjectiveStatus::OnTrack);
    let count_low = project_ok(
        &count_obj,
        &count_metric,
        &count_target,
        &[kind_measurement(
            count_obj.id.as_str(),
            "meas.kind.count.9",
            EvidenceValue::integer(9),
        )],
        at,
        &evidence,
    );
    assert_eq!(count_low.status, ObjectiveStatus::AtRisk);

    let dur_metric = kind_metric(MetricKind::Duration, "seconds", "mttr_seconds");
    let dur_obj = kind_objective(
        "objective.kind.duration",
        dur_metric.id.as_str(),
        "target.kind.duration",
    );
    let dur_target = ObjectiveTarget {
        id: ObjectiveTargetId::new("target.kind.duration"),
        comparison: ComparisonOp::Lte,
        value: EvidenceValue::duration_seconds(FRESHNESS_7D),
    };
    let dur_ok = project_ok(
        &dur_obj,
        &dur_metric,
        &dur_target,
        &[kind_measurement(
            dur_obj.id.as_str(),
            "meas.kind.duration.ok",
            EvidenceValue::duration_seconds(FRESHNESS_7D),
        )],
        at,
        &evidence,
    );
    assert_eq!(dur_ok.status, ObjectiveStatus::OnTrack);
    let dur_high = project_ok(
        &dur_obj,
        &dur_metric,
        &dur_target,
        &[kind_measurement(
            dur_obj.id.as_str(),
            "meas.kind.duration.high",
            EvidenceValue::duration_seconds(FRESHNESS_7D + 1),
        )],
        at,
        &evidence,
    );
    assert_eq!(dur_high.status, ObjectiveStatus::AtRisk);

    let bool_metric = kind_metric(MetricKind::Boolean, "boolean", "attested");
    let bool_obj = kind_objective(
        "objective.kind.boolean",
        bool_metric.id.as_str(),
        "target.kind.boolean",
    );
    let bool_target = ObjectiveTarget {
        id: ObjectiveTargetId::new("target.kind.boolean"),
        comparison: ComparisonOp::Eq,
        value: EvidenceValue::from_bool(true),
    };
    let bool_ok = project_ok(
        &bool_obj,
        &bool_metric,
        &bool_target,
        &[kind_measurement(
            bool_obj.id.as_str(),
            "meas.kind.bool.true",
            EvidenceValue::from_bool(true),
        )],
        at,
        &evidence,
    );
    assert_eq!(bool_ok.status, ObjectiveStatus::OnTrack);
    let bool_false = project_ok(
        &bool_obj,
        &bool_metric,
        &bool_target,
        &[kind_measurement(
            bool_obj.id.as_str(),
            "meas.kind.bool.false",
            EvidenceValue::from_bool(false),
        )],
        at,
        &evidence,
    );
    assert_eq!(bool_false.status, ObjectiveStatus::AtRisk);

    let mut ratio = BTreeMap::new();
    ratio.insert("numerator".into(), EvidenceValue::integer(98));
    ratio.insert("denominator".into(), EvidenceValue::integer(100));
    let ratio_metric = kind_metric(MetricKind::Ratio, "ratio", "remediated_ratio");
    let ratio_obj = kind_objective(
        "objective.kind.ratio",
        ratio_metric.id.as_str(),
        "target.kind.ratio",
    );
    let ratio_target = ObjectiveTarget {
        id: ObjectiveTargetId::new("target.kind.ratio"),
        comparison: ComparisonOp::Gte,
        value: EvidenceValue::decimal("0.98").expect("canonical ratio target"),
    };
    let ratio_ok = project_ok(
        &ratio_obj,
        &ratio_metric,
        &ratio_target,
        &[kind_measurement(
            ratio_obj.id.as_str(),
            "meas.kind.ratio.ok",
            EvidenceValue::object(ratio),
        )],
        at,
        &evidence,
    );
    assert_eq!(ratio_ok.status, ObjectiveStatus::OnTrack);

    let mut zero_den = BTreeMap::new();
    zero_den.insert("numerator".into(), EvidenceValue::integer(1));
    zero_den.insert("denominator".into(), EvidenceValue::integer(0));
    let ratio_zero = project_ok(
        &ratio_obj,
        &ratio_metric,
        &ratio_target,
        &[kind_measurement(
            ratio_obj.id.as_str(),
            "meas.kind.ratio.zero",
            EvidenceValue::object(zero_den),
        )],
        at,
        &evidence,
    );
    assert_never_success(&ratio_zero, "ratio denominator 0");
    assert_reason(&ratio_zero, "outOfDomain");

    let bounded_metric = kind_metric(MetricKind::BoundedNumeric, "score", "control_score");
    let bounded_obj = kind_objective(
        "objective.kind.bounded",
        bounded_metric.id.as_str(),
        "target.kind.bounded",
    );
    let bounded_target = ObjectiveTarget {
        id: ObjectiveTargetId::new("target.kind.bounded"),
        comparison: ComparisonOp::Gte,
        value: EvidenceValue::integer(8),
    };
    let bounded_ok = project_ok(
        &bounded_obj,
        &bounded_metric,
        &bounded_target,
        &[kind_measurement(
            bounded_obj.id.as_str(),
            "meas.kind.bounded.8",
            EvidenceValue::integer(8),
        )],
        at,
        &evidence,
    );
    assert_eq!(bounded_ok.status, ObjectiveStatus::OnTrack);
    let bounded_ood = project_ok(
        &bounded_obj,
        &bounded_metric,
        &bounded_target,
        &[kind_measurement(
            bounded_obj.id.as_str(),
            "meas.kind.bounded.11",
            EvidenceValue::integer(11),
        )],
        at,
        &evidence,
    );
    assert_never_success(&bounded_ood, "bounded numeric 11 outside [0,10]");
    assert_reason(&bounded_ood, "outOfDomain");
}

/// SO: missing measurement yields InsufficientEvidence, never OnTrack/Achieved
#[test]
fn so_t05_missing_measurement_is_insufficient_never_success() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let eval = project_ok(
        &objective,
        &metric,
        &target,
        &[],
        as_of_before_deadline(),
        &evidence,
    );
    assert_never_success(&eval, "missing measurement");
    assert_reason(&eval, "missingMeasurement");
    assert!(
        eval.snapshot.measurement_id.is_none(),
        "lineage must record that no candidate measurement was selected"
    );
}

/// SO: stale measurement (freshness or cadence window) yields InsufficientEvidence, never success
#[test]
fn so_t06_stale_measurement_is_insufficient_never_success() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let as_of = as_of_before_deadline();
    let stale_observed = as_of - chrono::Duration::seconds(FRESHNESS_7D as i64 + 3600);
    let stale = measurement(
        "meas.sla.stale-freshness",
        stale_observed,
        EvidenceValue::integer(100),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let freshness = project_ok(&objective, &metric, &target, &[stale], as_of, &evidence);
    assert_never_success(&freshness, "freshness-stale 100%");
    assert_reason(&freshness, "staleMeasurement");

    let mut cadence_metric = sla_metric();
    cadence_metric.freshness = None;
    let outside_window = as_of - chrono::Duration::seconds(CADENCE_30D as i64 + 86_400);
    let cadence_meas = measurement(
        "meas.sla.stale-cadence",
        outside_window,
        EvidenceValue::integer(100),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let cadence = project_ok(
        &objective,
        &cadence_metric,
        &target,
        &[cadence_meas],
        as_of,
        &evidence,
    );
    assert_never_success(&cadence, "cadence-window miss");
    assert_reason(&cadence, "staleMeasurement");
}

/// SO: partial / unknown population completeness is InsufficientEvidence even if the number meets target
#[test]
fn so_t07_partial_or_unknown_population_is_insufficient() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    for (id, completeness) in [
        ("meas.sla.partial", PopulationCompleteness::Partial),
        ("meas.sla.unknown", PopulationCompleteness::Unknown),
    ] {
        let meas = measurement(
            id,
            observed_fresh(),
            EvidenceValue::integer(100),
            isms_scope(&[IN_SCOPE_REPO]),
            completeness,
            &[ENVELOPE],
            None,
        );
        let eval = project_ok(
            &objective,
            &metric,
            &target,
            &[meas],
            as_of_before_deadline(),
            &evidence,
        );
        assert_never_success(&eval, &format!("{completeness:?} completeness at 100%"));
        assert_reason(&eval, "partialEvidence");
    }
}

fn manual_objective() -> SecurityObjective {
    let mut objective = sla_objective(Some(deadline_at()));
    objective.id = SecurityObjectiveId::new("objective.manual.attested-set");
    objective.logical_id = "objective.manual.attested-set".into();
    objective.title = "Attest that an objective set exists".into();
    objective.description = "Manual governance attestation; not inferred from a score.".into();
    objective.metric_id = ObjectiveMetricId::new("metric.manual.attested-set");
    objective.target_id = ObjectiveTargetId::new("target.manual.attested-set");
    objective.measurement_source = ObjectiveMeasurementSource {
        evidence_type: EvidenceType::new(MANUAL_EVIDENCE),
        collection: EvidenceCollectionKind::Manual,
        evidence_requirement_id: Some(EvidenceRequirementId::new(MANUAL_EVIDENCE)),
    };
    objective
}

fn manual_metric() -> ObjectiveMetric {
    ObjectiveMetric {
        id: ObjectiveMetricId::new("metric.manual.attested-set"),
        kind: MetricKind::Boolean,
        unit: Some("boolean".into()),
        domain_min: None,
        domain_max: None,
        evidence_type: EvidenceType::new(MANUAL_EVIDENCE),
        value_field: "attested".into(),
        freshness: Some(FreshnessRequirement {
            max_age_seconds: FRESHNESS_7D,
        }),
    }
}

fn manual_target() -> ObjectiveTarget {
    ObjectiveTarget {
        id: ObjectiveTargetId::new("target.manual.attested-set"),
        comparison: ComparisonOp::Eq,
        value: EvidenceValue::from_bool(true),
    }
}

/// SO: mixed pair — automated OnTrack does not promote a manual objective lacking attestation
#[test]
fn so_t08_automated_on_track_does_not_promote_manual_without_attestation() {
    let auto_obj = sla_objective(Some(deadline_at()));
    let auto_metric = sla_metric();
    let auto_target = sla_target(EvidenceValue::integer(98));
    let auto_evidence = pinned_evidence(&[ENVELOPE]);
    let automated = project_ok(
        &auto_obj,
        &auto_metric,
        &auto_target,
        &[sla_measurement(
            "meas.mixed.auto",
            EvidenceValue::integer(99),
        )],
        as_of_before_deadline(),
        &auto_evidence,
    );
    assert_eq!(automated.status, ObjectiveStatus::OnTrack);

    let man_obj = manual_objective();
    let mut meas = measurement(
        "meas.mixed.manual",
        observed_fresh(),
        EvidenceValue::from_bool(true),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    meas.objective_id = man_obj.id.clone();
    let manual = project_ok(
        &man_obj,
        &manual_metric(),
        &manual_target(),
        &[meas],
        as_of_before_deadline(),
        &auto_evidence,
    );
    assert_never_success(
        &manual,
        "manual boolean without sealed attestation, even when sibling automated objective is OnTrack",
    );
    assert_reason(&manual, "missingAttestation");
    assert_ne!(
        automated.status, manual.status,
        "a mixed bundle must not copy automated OnTrack onto the manual objective"
    );
}

/// SO: manual with sealed attestation meeting target is OnTrack/Achieved; boolean without attestation is InsufficientEvidence
#[test]
fn so_t09_manual_attestation_vs_boolean_without() {
    let man_obj = manual_objective();
    let metric = manual_metric();
    let target = manual_target();
    let evidence = pinned_evidence(&[ATTESTATION]);

    let mut attested = measurement(
        "meas.manual.attested",
        observed_fresh(),
        EvidenceValue::from_bool(true),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ATTESTATION],
        Some(ATTESTATION),
    );
    attested.objective_id = man_obj.id.clone();
    let ok = project_ok(
        &man_obj,
        &metric,
        &target,
        &[attested],
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(ok.status, ObjectiveStatus::OnTrack);

    let mut attested_after = measurement(
        "meas.manual.attested-after",
        observed_fresh(),
        EvidenceValue::from_bool(true),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ATTESTATION],
        Some(ATTESTATION),
    );
    attested_after.objective_id = man_obj.id.clone();
    let after = project_ok(
        &man_obj,
        &metric,
        &target,
        &[attested_after],
        as_of_after_deadline(),
        &evidence,
    );
    assert_eq!(after.status, ObjectiveStatus::Achieved);

    let mut bare = measurement(
        "meas.manual.bare",
        observed_fresh(),
        EvidenceValue::from_bool(true),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ATTESTATION],
        None,
    );
    bare.objective_id = man_obj.id.clone();
    let missing = project_ok(
        &man_obj,
        &metric,
        &target,
        &[bare],
        as_of_before_deadline(),
        &evidence,
    );
    assert_never_success(&missing, "manual boolean without attestationRef");
    assert_reason(&missing, "missingAttestation");
}

/// SO: measurement scope required; unscoped or out-of-scope mix is InsufficientEvidence (scopeMismatch)
#[test]
fn so_t10_unscoped_or_out_of_scope_is_scope_mismatch() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let at = as_of_before_deadline();

    let unscoped = measurement(
        "meas.sla.unscoped",
        observed_fresh(),
        EvidenceValue::integer(100),
        AssessmentScope::default(),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let empty = project_ok(&objective, &metric, &target, &[unscoped], at, &evidence);
    assert_never_success(&empty, "unscoped measurement");
    assert_reason(&empty, "scopeMismatch");

    let mixed = measurement(
        "meas.sla.mixed-scope",
        observed_fresh(),
        EvidenceValue::integer(100),
        isms_scope(&[IN_SCOPE_REPO, OUT_OF_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let mix = project_ok(&objective, &metric, &target, &[mixed], at, &evidence);
    assert_never_success(&mix, "out-of-scope repo mixed into 100%");
    assert_reason(&mix, "scopeMismatch");

    let mut excluded = isms_scope(&[IN_SCOPE_REPO, OUT_OF_SCOPE_REPO]);
    excluded.exclusions = vec![ScopeExclusion {
        subjects: vec![repo_selector(&[OUT_OF_SCOPE_REPO])],
        rationale: Some("lab is out of ISMS scope".into()),
        ..Default::default()
    }];
    let mut obj_excluded = objective.clone();
    obj_excluded.scope = excluded.clone();
    let still_includes_lab = measurement(
        "meas.sla.excluded-lab",
        observed_fresh(),
        EvidenceValue::integer(100),
        isms_scope(&[IN_SCOPE_REPO, OUT_OF_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let excluded_eval = project_ok(
        &obj_excluded,
        &metric,
        &target,
        &[still_includes_lab],
        at,
        &evidence,
    );
    assert_never_success(&excluded_eval, "measurement includes excluded subject");
    assert_reason(&excluded_eval, "scopeMismatch");
}

/// SO: historical — two as_of clocks on the same pinned snapshots yield two immutable evaluations
#[test]
fn so_t11_historical_as_of_snapshots_are_immutable() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let first_meas = sla_measurement("meas.sla.history.early", EvidenceValue::integer(97));
    let t1 = as_of_before_deadline();
    let first = project_ok(
        &objective,
        &metric,
        &target,
        std::slice::from_ref(&first_meas),
        t1,
        &evidence,
    );
    let first_digest = first.snapshot.digest.clone();
    assert_eq!(first.status, ObjectiveStatus::AtRisk);

    let later_meas = measurement(
        "meas.sla.history.late",
        as_of_after_deadline() - chrono::Duration::hours(12),
        EvidenceValue::integer(99),
        isms_scope(&[IN_SCOPE_REPO]),
        PopulationCompleteness::Authoritative,
        &[ENVELOPE],
        None,
    );
    let t2 = as_of_after_deadline();
    let second = project_ok(
        &objective,
        &metric,
        &target,
        &[first_meas.clone(), later_meas],
        t2,
        &evidence,
    );
    assert_eq!(second.status, ObjectiveStatus::Achieved);
    assert_ne!(
        first_digest, second.snapshot.digest,
        "a later measurement must produce a new snapshot, not rewrite the earlier digest"
    );

    let replay_t1 = project_ok(&objective, &metric, &target, &[first_meas], t1, &evidence);
    assert_eq!(
        replay_t1.snapshot.digest, first_digest,
        "replaying the earlier as_of + measurements must keep the historical digest"
    );
    assert_eq!(replay_t1.status, first.status);
}

/// SO: deterministic transitions — same inputs byte-equal; OnTrack→Achieved and AtRisk→Missed after deadline
#[test]
fn so_t12_deterministic_status_transitions() {
    let standing = sla_objective(None);
    let dated = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let meet = sla_measurement("meas.sla.transition.meet", EvidenceValue::integer(98));
    let miss = sla_measurement("meas.sla.transition.miss", EvidenceValue::integer(97));

    let a = project_ok(
        &dated,
        &metric,
        &target,
        std::slice::from_ref(&meet),
        as_of_before_deadline(),
        &evidence,
    );
    let b = project_ok(
        &dated,
        &metric,
        &target,
        std::slice::from_ref(&meet),
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(a.status, ObjectiveStatus::OnTrack);
    assert_eq!(a.status, b.status);
    assert_eq!(a.snapshot.digest, b.snapshot.digest);
    assert_eq!(
        serde_json::to_value(&a.snapshot).unwrap(),
        serde_json::to_value(&b.snapshot).unwrap(),
        "same inputs must be byte-equal snapshots"
    );

    let achieved = project_ok(
        &dated,
        &metric,
        &target,
        std::slice::from_ref(&meet),
        as_of_after_deadline(),
        &evidence,
    );
    assert_eq!(achieved.status, ObjectiveStatus::Achieved);

    let at_risk = project_ok(
        &dated,
        &metric,
        &target,
        std::slice::from_ref(&miss),
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(at_risk.status, ObjectiveStatus::AtRisk);
    let missed = project_ok(
        &dated,
        &metric,
        &target,
        &[miss],
        as_of_after_deadline(),
        &evidence,
    );
    assert_eq!(missed.status, ObjectiveStatus::Missed);

    let far_future = Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap();
    let ongoing_meet = project_ok(
        &standing,
        &metric,
        &target,
        std::slice::from_ref(&meet),
        far_future,
        &evidence,
    );
    assert_eq!(
        ongoing_meet.status,
        ObjectiveStatus::OnTrack,
        "ongoing objectives without a deadline cannot become Achieved from the clock alone"
    );
    let ongoing_miss = project_ok(
        &standing,
        &metric,
        &target,
        &[sla_measurement(
            "meas.sla.transition.ongoing-miss",
            EvidenceValue::integer(97),
        )],
        far_future,
        &evidence,
    );
    assert_eq!(
        ongoing_miss.status,
        ObjectiveStatus::AtRisk,
        "ongoing objectives without a deadline cannot become Missed from the clock alone"
    );

    let before_start = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap();
    let not_started = project_ok(&dated, &metric, &target, &[meet], before_start, &evidence);
    assert_never_success(&not_started, "as_of before startAt");
    assert_reason(&not_started, "notStarted");
}

/// SO: replay of a pinned EvidenceSnapshot reproduces status without collectors
#[test]
fn so_t13_replay_pinned_evidence_snapshot_without_collectors() {
    let objective = sla_objective(Some(deadline_at()));
    let metric = sla_metric();
    let target = sla_target(EvidenceValue::integer(98));
    let evidence = pinned_evidence(&[ENVELOPE]);
    let measurements = [sla_measurement(
        "meas.sla.replay",
        EvidenceValue::integer(98),
    )];
    let first = project_ok(
        &objective,
        &metric,
        &target,
        &measurements,
        as_of_before_deadline(),
        &evidence,
    );
    assert_lineage(
        &first,
        &objective,
        &metric,
        &target,
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(
        first.snapshot.measurement_id.as_ref().map(|id| id.as_str()),
        Some("meas.sla.replay")
    );
    assert!(
        first
            .snapshot
            .envelope_digests
            .iter()
            .any(|d| d == ENVELOPE),
        "snapshot must pin envelope digests so management review does not re-query collectors"
    );

    let replay = project_ok(
        &objective,
        &metric,
        &target,
        &measurements,
        as_of_before_deadline(),
        &evidence,
    );
    assert_eq!(replay.status, first.status);
    assert_eq!(reason_blob(&replay), reason_blob(&first));
    assert_eq!(replay.snapshot.digest, first.snapshot.digest);
    assert_eq!(
        serde_json::to_value(&replay.snapshot).unwrap(),
        serde_json::to_value(&first.snapshot).unwrap()
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("evaluate_objective"),
        "replay must not require collector-produced ObjectiveStatus"
    );
}

/// SO: collectors / GitHub normalize contain no ObjectiveStatus; seal still rejects compliance narratives
#[test]
fn so_t14_collectors_do_not_emit_objective_status() {
    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "ObjectiveStatus",
        "evaluate_objective",
        "OnTrack",
        "struct ObjectiveMeasurement",
    ] {
        assert!(
            !collector.contains(needle),
            "collectors must not contain `{needle}`"
        );
    }
    assert!(looks_like_compliance_claim("ISO 27001 compliant"));
    assert!(
        !looks_like_compliance_claim("critical vulnerabilities remediated within seven days"),
        "metric prose is not itself a sealed compliance claim"
    );
}

/// SO: IR sources do not define a second EvidenceValue / MetricValue enum; evaluator calls typed_eq / cmp_numeric
#[test]
fn so_t15_no_second_metric_value_enum_uses_evidence_value() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("enum EvidenceValue")
            && !ir.contains("enum MetricValue")
            && !ir.contains("enum ObjectiveValue"),
        "IR must persist evidence-value/v1 JSON and must not fork a metric-value enum"
    );
    let evaluator = fs::read_to_string(crate_src("weeping-angel-assurance").join("objectives.rs"))
        .expect("assurance objectives.rs evaluator module");
    assert!(
        evaluator.contains("typed_eq") && evaluator.contains("cmp_numeric"),
        "evaluator must call EvidenceValue::typed_eq / cmp_numeric"
    );
    assert!(
        evaluator.contains("EvidenceValue"),
        "value-bearing evaluation types live in weeping-angel-assurance and embed EvidenceValue"
    );
}

/// SO: Kleene applicability module is not imported by the objective evaluator
#[test]
fn so_t16_kleene_applicability_not_invoked() {
    let evaluator = fs::read_to_string(crate_src("weeping-angel-assurance").join("objectives.rs"))
        .expect("assurance objectives.rs evaluator module");
    for needle in [
        "evaluate_org_context",
        "crate::applicability",
        "use super::applicability",
        "OrgContext",
        "Kleene",
    ] {
        assert!(
            !evaluator.contains(needle),
            "objective evaluator must not invoke Kleene applicability (`{needle}`)"
        );
    }
}

/// SO: Control.objective string API unchanged; governance catalog TOML for security-objectives unchanged
#[test]
fn so_t17_control_objective_string_and_catalog_toml_unchanged() {
    let control = Control::new(
        ControlId::new("control.governance.security-objectives"),
        "Security objectives",
        "Security objectives are recorded as an attestation, not inferred from a score.",
    );
    assert_eq!(control.objective(), "");
    let with_prose = control.with_objective("Require a fresh scan of in-scope repositories.");
    assert_eq!(
        with_prose.objective(),
        "Require a fresh scan of in-scope repositories."
    );
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/control.rs");
    assert!(src.contains("objective: String"));
    assert!(src.contains("pub fn with_objective"));

    let catalog = CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load");
    let row = catalog
        .control("control.governance.security-objectives")
        .expect("governance security-objectives control remains");
    assert_eq!(row.automation, "manual");
    assert_eq!(row.tests, ["test.governance.objectives-attested"]);
    assert_eq!(
        row.objective,
        "Require an attested objective set for the in-scope organization."
    );
    let test = catalog
        .tests()
        .get("test.governance.objectives-attested")
        .expect("objectives-attested catalog test remains");
    assert_eq!(
        test.expression.get("op").and_then(toml::Value::as_str),
        Some("manual-review")
    );
}

/// SO: Active objective with empty scope fails validate; Draft may omit owner
#[test]
fn so_t18_active_empty_scope_fails_validate_draft_may_omit_owner() {
    let mut draft = sla_objective(None);
    draft.lifecycle = ObjectiveLifecycle::Draft;
    draft.owner = None;
    draft.start_at = None;
    draft.scope = AssessmentScope::default();
    draft
        .validate()
        .expect("Draft may omit owner, start, and populated scope");

    let mut active_empty = sla_objective(Some(deadline_at()));
    active_empty.scope = AssessmentScope::default();
    let err = active_empty
        .validate()
        .expect_err("Active with empty scope must fail closed");
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("scope"),
        "Active empty-scope error must mention scope, got `{err}`"
    );

    let mut no_owner = sla_objective(Some(deadline_at()));
    no_owner.owner = None;
    let owner_err = no_owner
        .validate()
        .expect_err("Active without owner must fail closed");
    assert!(
        owner_err.to_string().to_ascii_lowercase().contains("owner"),
        "Active missing-owner error must mention owner, got `{owner_err}`"
    );

    assert!(
        SecurityObjectiveId::try_new("550e8400-e29b-41d4-a716-446655440000").is_err(),
        "uuid-v4 is not a SecurityObjectiveId"
    );

    let draft_eval = project(
        &draft,
        &sla_metric(),
        &sla_target(EvidenceValue::integer(98)),
        &[sla_measurement("meas.draft", EvidenceValue::integer(100))],
        as_of_before_deadline(),
        &pinned_evidence(&[ENVELOPE]),
    );
    let err = draft_eval.expect_err("evaluating a non-Active objective is a validation error");
    assert!(
        err.to_string().to_ascii_lowercase().contains("notactive")
            || err.to_string().to_ascii_lowercase().contains("not active"),
        "non-Active evaluation must fail with notActive, got `{err}`"
    );
}

/// SO: schema remains assurance-ir/v1 for governance records; evaluation snapshot uses weeping-angel/objective-evaluation/v1
#[test]
fn so_t19_schema_assurance_ir_and_evaluation_snapshot_v1() {
    let objective = sla_objective(Some(deadline_at()));
    assert_eq!(objective.schema_version, ASSURANCE_IR_SCHEMA);
    let json = serde_json::to_value(&objective).unwrap();
    assert_eq!(
        json.get("schemaVersion").and_then(Value::as_str),
        Some(ASSURANCE_IR_SCHEMA)
    );
    assert!(json.get("lifecycle").is_some());
    assert!(json.get("metricId").is_some());

    let eval = project_ok(
        &objective,
        &sla_metric(),
        &sla_target(EvidenceValue::integer(98)),
        &[sla_measurement(
            "meas.sla.schema",
            EvidenceValue::integer(98),
        )],
        as_of_before_deadline(),
        &pinned_evidence(&[ENVELOPE]),
    );
    assert_eq!(eval.snapshot.schema, EVAL_SCHEMA);
    let snap = serde_json::to_value(&eval.snapshot).unwrap();
    assert_eq!(
        snap.get("schema").and_then(Value::as_str),
        Some(EVAL_SCHEMA)
    );
    assert!(snap.get("asOf").is_some());
    assert!(snap.get("reasonCodes").is_some());
    assert!(snap.get("evidenceSnapshotDigest").is_some());
}

/// SO: dual-suite names registered in root Cargo.toml; this spec listed in CANONICAL_SPECS
#[test]
fn so_t20_dual_suite_registered_and_spec_in_canonical_specs() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("name = \"sdd_security_objectives_baseline\"")
            && !toml.contains("path = \"tests/contracts/security_objectives.baseline.rs\"")
            && toml.contains("name = \"sdd_security_objectives_target\"")
            && toml.contains("path = \"tests/contracts/security_objectives.target.rs\""),
        "security objectives dual-suite must be explicitly listed (not auto-discovered)"
    );
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/security-objectives.md"),
        "implement must list the security-objectives spec in CANONICAL_SPECS"
    );
    assert!(
        manifest_dir()
            .join("docs/specs/security-objectives.md")
            .is_file()
    );
    assert!(
        crate_src("weeping-angel-assurance-ir")
            .join("objectives.rs")
            .is_file(),
        "IR governance records live in weeping-angel-assurance-ir/src/objectives.rs"
    );
    assert!(
        crate_src("weeping-angel-assurance")
            .join("objectives.rs")
            .is_file(),
        "pure evaluator lives in weeping-angel-assurance/src/objectives.rs"
    );
}
