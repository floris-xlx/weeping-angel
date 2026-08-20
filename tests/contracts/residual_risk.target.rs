//! Target suite for Operational ISMS v1 residual risk (Prompt 09).
//!
//! Encodes DESIRED behavior in `docs/specs/residual-risk.md` §4 / §4.11
//! (P09-T01–T20). Must stay RED on CURRENT HEAD: residual projection types and
//! `project_residual_risk` do not exist. Do not implement the projector here
//! and do not `#[ignore]`.
//!
//! These tests import the intended public contract. On characterization HEAD
//! they fail because those types/APIs are missing (not because of a passing
//! spec-id lock).

use chrono::{DateTime, TimeZone, Utc};
use weeping_angel_assurance::residual::{
    ResidualRiskRequest, ResidualRiskStore, project_residual_risk, query_residual_risk,
};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlTestId, ControlTestSnapshotRef, Exception, ExceptionId,
    ExceptionStatus, IdentityId, InherentRiskRef, InherentRiskSnapshot, ManualResidualAssessment,
    MethodologyRef, PrincipalRef, ResidualRiskError, ResidualRiskId, ResidualRiskMode,
    ResidualRiskProjection, Risk, RiskId, TreatmentCompleteness, TreatmentPlanRef,
    TreatmentPlanSnapshot,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

const NO_REDUCTION: &str = "residual-methodology:no-reduction";
const CONTROL_EFFECTIVENESS: &str = "residual-methodology:control-effectiveness";
const METHODOLOGY_V1: &str = "v1";

const INHERENT_HIGH_ORDINAL: u32 = 4;
const INHERENT_HIGH_RATING: &str = "high";
const MIN_RESIDUAL_FLOOR: u32 = 1;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));

fn projected_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn assessed_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 11, 30, 0).unwrap()
}

fn risk_id() -> RiskId {
    RiskId::new("risk:source-tamper")
}

fn mfa() -> ControlId {
    ControlId::new("control.access.mfa")
}

fn logging() -> ControlId {
    ControlId::new("control.access.logging")
}

fn inherent_high() -> InherentRiskSnapshot {
    InherentRiskSnapshot {
        pin: InherentRiskRef {
            risk_id: risk_id(),
            version: "inherent.v1".into(),
            digest: Some("sha256:inherent-v1".into()),
        },
        rating_id: INHERENT_HIGH_RATING.into(),
        ordinal: INHERENT_HIGH_ORDINAL,
    }
}

fn treatment(
    completeness: TreatmentCompleteness,
    controls: Vec<ControlId>,
) -> TreatmentPlanSnapshot {
    TreatmentPlanSnapshot {
        pin: TreatmentPlanRef {
            plan_id: "treat:source-tamper".into(),
            version: "treatment.v1".into(),
            digest: Some("sha256:treatment-v1".into()),
        },
        relevant_control_ids: controls,
        completeness,
    }
}

fn methodology(id: &str) -> MethodologyRef {
    MethodologyRef {
        methodology_id: id.into(),
        version: METHODOLOGY_V1.into(),
    }
}

fn test_result(control: ControlId, test: &str, effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: ControlTestId::new(test),
        control_id: control,
        effectiveness,
        rationale: format!("target fixture {effectiveness:?}"),
        evidence_refs: vec!["sha256:evidence-mfa".into()],
        missing_evidence: Vec::new(),
        checked_at: projected_at(),
        test_version: "1".into(),
        input_digest: format!("sha256:{test}"),
        duration: None,
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

fn snapshot_ref(results: &[ControlTestResult]) -> ControlTestSnapshotRef {
    let mut result_ids: Vec<String> = results
        .iter()
        .map(|r| {
            format!(
                "{}:{}:{}",
                r.test_id.as_str(),
                r.control_id.as_str(),
                r.input_digest
            )
        })
        .collect();
    result_ids.sort();
    ControlTestSnapshotRef {
        digest: format!("sha256:tests:{}", result_ids.join("|")),
        result_ids,
    }
}

fn calculated_request(
    completeness: TreatmentCompleteness,
    results: Vec<ControlTestResult>,
    methodology_id: &str,
) -> ResidualRiskRequest {
    let controls: Vec<ControlId> = results.iter().map(|r| r.control_id.clone()).collect();
    ResidualRiskRequest {
        mode: ResidualRiskMode::Calculated,
        inherent: inherent_high(),
        treatment: treatment(completeness, controls),
        methodology: methodology(methodology_id),
        control_tests: snapshot_ref(&results),
        control_test_results: results,
        exceptions: Vec::new(),
        manual: None,
        projected_at: projected_at(),
    }
}

fn approved_exception() -> Exception {
    let mut exception = Exception::new(ExceptionId::new("exc.mfa.break-glass"), "break-glass MFA");
    exception.control_id = Some(mfa());
    exception.status = ExceptionStatus::Approved;
    exception.approved_by = Some(PrincipalRef::Identity(IdentityId::new("identity:ciso")));
    exception
}

fn complete_manual(ordinal: u32, rating: &str) -> ManualResidualAssessment {
    ManualResidualAssessment {
        principal: Some(PrincipalRef::Identity(IdentityId::new("identity:ciso"))),
        rationale: "management accepts remaining source-tamper exposure".into(),
        assessed_at: Some(assessed_at()),
        approved_by: Some(PrincipalRef::Identity(IdentityId::new("identity:ciso"))),
        residual_ordinal: ordinal,
        residual_rating_id: rating.into(),
    }
}

fn project(req: ResidualRiskRequest) -> Result<ResidualRiskProjection, ResidualRiskError> {
    let mut store = ResidualRiskStore::new();
    project_residual_risk(&mut store, req)
}

fn err_text(err: ResidualRiskError) -> String {
    err.to_string().to_ascii_lowercase()
}

fn assert_fail_closed(result: Result<ResidualRiskProjection, ResidualRiskError>, needle: &str) {
    let err = match result {
        Ok(projection) => panic!(
            "expected fail-closed containing `{needle}`, got residual ordinal {}",
            projection.residual_ordinal
        ),
        Err(err) => err,
    };
    let msg = err_text(err);
    assert!(
        msg.contains(&needle.to_ascii_lowercase()),
        "fail-closed error must contain `{needle}`, got `{msg}`"
    );
}

fn assert_lineage(projection: &ResidualRiskProjection, req: &ResidualRiskRequest) {
    assert_eq!(projection.risk_id, req.inherent.pin.risk_id);
    assert_eq!(projection.mode, req.mode);
    assert_eq!(
        projection.inherent.version, req.inherent.pin.version,
        "every projection must pin the inherent-risk version"
    );
    assert!(
        !projection.inherent.version.is_empty(),
        "inherent-risk version must not be empty"
    );
    assert_eq!(
        projection.treatment.version, req.treatment.pin.version,
        "every projection must pin the treatment-plan version"
    );
    assert_eq!(
        projection.methodology.methodology_id,
        req.methodology.methodology_id
    );
    assert_eq!(projection.methodology.version, req.methodology.version);
    assert_eq!(
        projection.relevant_control_ids, req.treatment.relevant_control_ids,
        "relevant controls from the treatment pin must appear on the projection"
    );
    assert_eq!(projection.control_tests.digest, req.control_tests.digest);
    assert!(
        !projection.control_tests.digest.is_empty(),
        "control-test snapshot digest is required lineage"
    );
    assert_eq!(projection.projected_at, req.projected_at);
    if req.mode == ResidualRiskMode::Assessed || req.mode == ResidualRiskMode::Hybrid {
        assert!(
            projection.manual.is_some(),
            "Assessed/Hybrid projections must record the manual assessment"
        );
    }
}

fn assert_not_zero_residual(projection: &ResidualRiskProjection) {
    assert_ne!(
        projection.residual_ordinal, 0,
        "Effectiveness::Effective must never map directly to zero residual"
    );
    assert!(
        projection.residual_ordinal >= MIN_RESIDUAL_FLOOR,
        "calculated methodologies have a mandatory non-zero floor, got {}",
        projection.residual_ordinal
    );
    let rating = projection.residual_rating_id.to_ascii_lowercase();
    assert_ne!(rating, "none");
    assert_ne!(rating, "zero");
    assert_ne!(rating, "absent");
}

fn trace_blob(projection: &ResidualRiskProjection) -> String {
    format!("{:?}", projection.reduction_trace).to_ascii_lowercase()
}

/// P09-T01: `P09: effective control does not map to zero residual`
#[test]
fn p09_t01_effective_control_does_not_map_to_zero_residual() {
    let req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    let projection = project(req.clone()).expect("Effective is a valid calculated signal");
    assert_lineage(&projection, &req);
    assert_not_zero_residual(&projection);
    assert!(
        projection.residual_ordinal < INHERENT_HIGH_ORDINAL,
        "Effective may reduce residual, but not to zero; got {}",
        projection.residual_ordinal
    );
    let blob = trace_blob(&projection);
    assert!(
        blob.contains("control.access.mfa") && blob.contains("effective"),
        "reduction trace must explain the Effective control: {blob}"
    );
}

/// P09-T02: `P09: ineffective control grants no reduction`
#[test]
fn p09_t02_ineffective_control_grants_no_reduction() {
    let req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Ineffective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    let projection = project(req.clone()).expect("Ineffective still yields a projection");
    assert_lineage(&projection, &req);
    assert_eq!(
        projection.residual_ordinal, INHERENT_HIGH_ORDINAL,
        "Ineffective grants no reduction from inherent"
    );
    assert_eq!(
        projection.residual_rating_id.to_ascii_lowercase(),
        INHERENT_HIGH_RATING
    );
    let blob = trace_blob(&projection);
    assert!(
        blob.contains("ineffective") || blob.contains("no reduction") || blob.contains("step: 0"),
        "trace must record a zero step for Ineffective: {blob}"
    );
}

/// P09-T03: `P09: missing required control fails closed`
#[test]
fn p09_t03_missing_required_control_fails_closed() {
    let mut req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    req.treatment
        .relevant_control_ids
        .push(ControlId::new("control.access.missing"));
    assert_fail_closed(project(req), "dangling control");
}

/// P09-T04: `P09: partial treatment is not full credit`
#[test]
fn p09_t04_partial_treatment_is_not_full_credit() {
    let full = project(calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    ))
    .expect("complete + Effective");
    let partial_completeness = project(calculated_request(
        TreatmentCompleteness::Partial,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    ))
    .expect("partial completeness");
    let partial_effectiveness = project(calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::PartiallyEffective,
        )],
        CONTROL_EFFECTIVENESS,
    ))
    .expect("PartiallyEffective");

    assert!(
        partial_completeness.residual_ordinal > full.residual_ordinal,
        "treatment completeness partial must not receive full Effective credit ({} vs {})",
        partial_completeness.residual_ordinal,
        full.residual_ordinal
    );
    assert!(
        partial_effectiveness.residual_ordinal > full.residual_ordinal,
        "PartiallyEffective must be a strictly smaller step than Effective ({} vs {})",
        partial_effectiveness.residual_ordinal,
        full.residual_ordinal
    );
    assert!(
        partial_completeness.residual_ordinal < INHERENT_HIGH_ORDINAL
            || partial_effectiveness.residual_ordinal < INHERENT_HIGH_ORDINAL,
        "partial treatment is first-class: residual must still move, but not to the full mitigated band"
    );
    assert_not_zero_residual(&full);
    assert_not_zero_residual(&partial_completeness);
    assert_not_zero_residual(&partial_effectiveness);
}

/// P09-T05: `P09: assessed residual requires principal rationale and time`
#[test]
fn p09_t05_assessed_residual_requires_principal_rationale_and_time() {
    let results = vec![test_result(
        mfa(),
        "test.access.mfa",
        Effectiveness::Effective,
    )];
    let mut ok = ResidualRiskRequest {
        mode: ResidualRiskMode::Assessed,
        inherent: inherent_high(),
        treatment: treatment(TreatmentCompleteness::Complete, vec![mfa()]),
        methodology: methodology(CONTROL_EFFECTIVENESS),
        control_tests: snapshot_ref(&results),
        control_test_results: results.clone(),
        exceptions: Vec::new(),
        manual: Some(complete_manual(2, "medium")),
        projected_at: projected_at(),
    };
    let assessed = project(ok.clone()).expect("complete Assessed must succeed");
    assert_lineage(&assessed, &ok);
    assert_eq!(assessed.mode, ResidualRiskMode::Assessed);
    assert_eq!(assessed.residual_ordinal, 2);
    let manual = assessed
        .manual
        .as_ref()
        .expect("manual assessment recorded");
    assert!(manual.principal.is_some());
    assert!(!manual.rationale.trim().is_empty());
    assert!(manual.assessed_at.is_some());

    ok.manual = None;
    assert_fail_closed(project(ok.clone()), "missing manual assessment");

    let mut missing_principal = ResidualRiskRequest {
        manual: Some(ManualResidualAssessment {
            principal: None,
            rationale: "has rationale".into(),
            assessed_at: Some(assessed_at()),
            approved_by: None,
            residual_ordinal: 2,
            residual_rating_id: "medium".into(),
        }),
        ..ok.clone()
    };
    missing_principal.mode = ResidualRiskMode::Assessed;
    assert_fail_closed(project(missing_principal), "missing manual assessment");

    let missing_rationale = ResidualRiskRequest {
        mode: ResidualRiskMode::Assessed,
        manual: Some(ManualResidualAssessment {
            principal: Some(PrincipalRef::Identity(IdentityId::new("identity:ciso"))),
            rationale: "   ".into(),
            assessed_at: Some(assessed_at()),
            approved_by: None,
            residual_ordinal: 2,
            residual_rating_id: "medium".into(),
        }),
        ..ok.clone()
    };
    assert_fail_closed(project(missing_rationale), "missing manual assessment");

    let missing_time = ResidualRiskRequest {
        mode: ResidualRiskMode::Assessed,
        manual: Some(ManualResidualAssessment {
            principal: Some(PrincipalRef::Identity(IdentityId::new("identity:ciso"))),
            rationale: "accountable but untimed".into(),
            assessed_at: None,
            approved_by: None,
            residual_ordinal: 2,
            residual_rating_id: "medium".into(),
        }),
        ..ok
    };
    assert_fail_closed(project(missing_time), "missing manual assessment");
}

/// P09-T06: `P09: historical projection remains queryable after new projection`
#[test]
fn p09_t06_historical_projection_remains_queryable_after_new_projection() {
    let mut store = ResidualRiskStore::new();
    let first_req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    let first = project_residual_risk(&mut store, first_req.clone()).expect("first projection");
    let first_id: ResidualRiskId = first.id.clone();
    let first_ordinal = first.residual_ordinal;
    let first_json = serde_json::to_value(&first).expect("serialize first");

    let mut regression = first_req.clone();
    regression.control_test_results[0].effectiveness = Effectiveness::Ineffective;
    regression.control_test_results[0].input_digest = "sha256:test.access.mfa.regressed".into();
    regression.control_tests = snapshot_ref(&regression.control_test_results);
    regression.projected_at = Utc.with_ymd_and_hms(2026, 8, 20, 12, 0, 0).unwrap();

    let second = project_residual_risk(&mut store, regression).expect("regression projection");
    assert_ne!(
        first_id, second.id,
        "control regression must write a new projection id"
    );
    assert!(
        second.residual_ordinal > first_ordinal,
        "regression must not mutate the first residual downward in place"
    );

    let loaded =
        query_residual_risk(&store, &first_id).expect("historical projection remains queryable");
    assert_eq!(loaded.id, first_id);
    assert_eq!(loaded.residual_ordinal, first_ordinal);
    assert_eq!(
        serde_json::to_value(&loaded).expect("serialize loaded"),
        first_json,
        "historical projection bytes/semantic fields stay unchanged"
    );
}

/// P09-T07: `P09: stale evidence fails closed`
#[test]
fn p09_t07_stale_evidence_fails_closed() {
    let req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::StaleEvidence,
        )],
        CONTROL_EFFECTIVENESS,
    );
    assert_fail_closed(project(req), "stale evidence");
}

/// P09-T07b: `NotTested` and `InsufficientEvidence` also fail closed.
#[test]
fn p09_t07b_not_tested_and_insufficient_evidence_fail_closed() {
    let not_tested = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::NotTested,
        )],
        CONTROL_EFFECTIVENESS,
    );
    assert_fail_closed(project(not_tested), "not tested");

    let insufficient = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::InsufficientEvidence,
        )],
        CONTROL_EFFECTIVENESS,
    );
    assert_fail_closed(project(insufficient), "insufficient evidence");
}

/// P09-T08: `P09: multiple controls compose conservatively and remain explainable`
#[test]
fn p09_t08_multiple_controls_compose_conservatively_and_remain_explainable() {
    let one = project(calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    ))
    .expect("single Effective");
    let two = project(calculated_request(
        TreatmentCompleteness::Complete,
        vec![
            test_result(mfa(), "test.access.mfa", Effectiveness::Effective),
            test_result(logging(), "test.access.logging", Effectiveness::Effective),
        ],
        CONTROL_EFFECTIVENESS,
    ))
    .expect("two Effective");

    assert!(
        two.residual_ordinal <= one.residual_ordinal,
        "a second Effective control may reduce further, never increase residual"
    );
    assert_not_zero_residual(&two);
    assert_eq!(
        two.relevant_control_ids,
        vec![mfa(), logging()],
        "all relevant controls appear on the projection"
    );
    let blob = trace_blob(&two);
    assert!(
        blob.contains("control.access.mfa") && blob.contains("control.access.logging"),
        "each control must have a trace line: {blob}"
    );
}

/// P09-T09: `P09: no-reduction methodology ignores effectiveness`
#[test]
fn p09_t09_no_reduction_methodology_ignores_effectiveness() {
    let req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        NO_REDUCTION,
    );
    let projection = project(req.clone()).expect("no-reduction still projects");
    assert_lineage(&projection, &req);
    assert_eq!(
        projection.residual_ordinal, INHERENT_HIGH_ORDINAL,
        "no-reduction methodology must not lower residual when controls are Effective"
    );
    assert_eq!(
        projection.residual_rating_id.to_ascii_lowercase(),
        INHERENT_HIGH_RATING
    );
    assert_not_zero_residual(&projection);
    let blob = trace_blob(&projection);
    assert!(
        blob.contains("no reduction") || blob.contains("no-reduction"),
        "trace must explain that effectiveness did not lower residual: {blob}"
    );
}

/// P09-T10: `P09: approved exception does not silently mean residual is low`
#[test]
fn p09_t10_approved_exception_does_not_silently_mean_residual_is_low() {
    let results = vec![test_result(
        mfa(),
        "test.access.mfa",
        Effectiveness::ExceptionApproved,
    )];
    let mut req = ResidualRiskRequest {
        mode: ResidualRiskMode::Calculated,
        inherent: inherent_high(),
        treatment: treatment(TreatmentCompleteness::Complete, vec![mfa()]),
        methodology: methodology(CONTROL_EFFECTIVENESS),
        control_tests: snapshot_ref(&results),
        control_test_results: results,
        exceptions: vec![approved_exception()],
        manual: None,
        projected_at: projected_at(),
    };
    let projection = project(req.clone()).expect("exception is governance evidence, not fail-open");
    assert_lineage(&projection, &req);
    assert_eq!(
        projection.residual_ordinal, INHERENT_HIGH_ORDINAL,
        "ExceptionApproved grants zero reduction"
    );
    assert_ne!(
        projection.residual_rating_id.to_ascii_lowercase(),
        "low",
        "approved Exception / ExceptionApproved must not silently yield Low"
    );
    assert!(
        projection.residual_ordinal > MIN_RESIDUAL_FLOOR,
        "exception must not collapse residual to the methodology floor"
    );
    assert!(
        projection
            .exception_ids
            .iter()
            .any(|id| id.as_str() == "exc.mfa.break-glass")
            || trace_blob(&projection).contains("exception"),
        "exception id / ExceptionApproved must appear as governance evidence"
    );

    req.exceptions.clear();
    let by_effectiveness_only =
        project(req).expect("ExceptionApproved without Exception row still projects");
    assert_ne!(
        by_effectiveness_only
            .residual_rating_id
            .to_ascii_lowercase(),
        "low"
    );
    assert_eq!(
        by_effectiveness_only.residual_ordinal,
        INHERENT_HIGH_ORDINAL
    );
}

/// P09-T11: `P09: Calculated vs Assessed vs Hybrid`
#[test]
fn p09_t11_calculated_vs_assessed_vs_hybrid() {
    let _ = ResidualRiskMode::Calculated;
    let _ = ResidualRiskMode::Assessed;
    let _ = ResidualRiskMode::Hybrid;

    let results = vec![test_result(
        mfa(),
        "test.access.mfa",
        Effectiveness::Effective,
    )];
    let calculated = ResidualRiskRequest {
        mode: ResidualRiskMode::Calculated,
        inherent: inherent_high(),
        treatment: treatment(TreatmentCompleteness::Complete, vec![mfa()]),
        methodology: methodology(CONTROL_EFFECTIVENESS),
        control_tests: snapshot_ref(&results),
        control_test_results: results.clone(),
        exceptions: Vec::new(),
        manual: None,
        projected_at: projected_at(),
    };
    let assessed = ResidualRiskRequest {
        mode: ResidualRiskMode::Assessed,
        manual: Some(complete_manual(3, "elevated")),
        ..calculated.clone()
    };
    let hybrid = ResidualRiskRequest {
        mode: ResidualRiskMode::Hybrid,
        manual: Some(complete_manual(3, "elevated")),
        ..calculated.clone()
    };

    let calc = project(calculated.clone()).expect("Calculated");
    let assess = project(assessed.clone()).expect("Assessed");
    let hyb = project(hybrid.clone()).expect("Hybrid");

    assert_eq!(calc.mode, ResidualRiskMode::Calculated);
    assert_eq!(assess.mode, ResidualRiskMode::Assessed);
    assert_eq!(hyb.mode, ResidualRiskMode::Hybrid);
    assert_ne!(calc.mode, assess.mode);
    assert_ne!(calc.mode, hyb.mode);
    assert_ne!(assess.mode, hyb.mode);

    assert_lineage(&calc, &calculated);
    assert_lineage(&assess, &assessed);
    assert_lineage(&hyb, &hybrid);

    assert_eq!(
        assess.residual_ordinal, 3,
        "Assessed uses the manual ordinal"
    );
    assert!(
        hyb.manual
            .as_ref()
            .and_then(|m| m.approved_by.as_ref())
            .is_some(),
        "Hybrid must record the approved management assessment"
    );

    let again = project(calculated).expect("Calculated is deterministic");
    assert_eq!(calc.residual_ordinal, again.residual_ordinal);
    assert_eq!(
        format!("{:?}", calc.reduction_trace),
        format!("{:?}", again.reduction_trace),
        "Calculated methodologies are deterministic and versioned"
    );
    assert_eq!(calc.methodology.version, METHODOLOGY_V1);
}

/// P09-T12: `P09: Hybrid fails closed when management assessment is missing`
#[test]
fn p09_t12_hybrid_fails_closed_when_management_assessment_is_missing() {
    let results = vec![test_result(
        mfa(),
        "test.access.mfa",
        Effectiveness::Effective,
    )];
    let mut req = ResidualRiskRequest {
        mode: ResidualRiskMode::Hybrid,
        inherent: inherent_high(),
        treatment: treatment(TreatmentCompleteness::Complete, vec![mfa()]),
        methodology: methodology(CONTROL_EFFECTIVENESS),
        control_tests: snapshot_ref(&results),
        control_test_results: results,
        exceptions: Vec::new(),
        manual: None,
        projected_at: projected_at(),
    };
    assert_fail_closed(project(req.clone()), "missing management assessment");

    req.manual = Some(ManualResidualAssessment {
        principal: Some(PrincipalRef::Identity(IdentityId::new("identity:ciso"))),
        rationale: "signals only; no approval".into(),
        assessed_at: Some(assessed_at()),
        approved_by: None,
        residual_ordinal: 2,
        residual_rating_id: "medium".into(),
    });
    assert_fail_closed(project(req), "missing management assessment");
}

/// P09-T13: `P09: fail closed missing methodology version`
#[test]
fn p09_t13_fail_closed_missing_methodology_version() {
    let mut req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    req.methodology.version.clear();
    assert_fail_closed(project(req.clone()), "missing methodology version");

    req.methodology.version = "v-unknown".into();
    assert_fail_closed(project(req), "unknown methodology");
}

/// P09-T14: `P09: fail closed missing treatment-plan version`
#[test]
fn p09_t14_fail_closed_missing_treatment_plan_version() {
    let mut req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    req.treatment.pin.version.clear();
    assert_fail_closed(project(req), "missing treatment-plan version");
}

/// P09-T15: `P09: fail closed missing inherent-risk version`
#[test]
fn p09_t15_fail_closed_missing_inherent_risk_version() {
    let mut req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    req.inherent.pin.version.clear();
    assert_fail_closed(project(req), "missing inherent-risk version");
}

/// P09-T16: `P09: fail closed missing control-test snapshot`
#[test]
fn p09_t16_fail_closed_missing_control_test_snapshot() {
    let mut req = calculated_request(
        TreatmentCompleteness::Complete,
        vec![test_result(
            mfa(),
            "test.access.mfa",
            Effectiveness::Effective,
        )],
        CONTROL_EFFECTIVENESS,
    );
    req.control_tests.digest.clear();
    req.control_tests.result_ids.clear();
    req.control_test_results.clear();
    assert_fail_closed(project(req), "missing control-test snapshot");
}

/// P09-T17: Dual-suite binaries registered in root `Cargo.toml`.
#[test]
fn p09_t17_dual_suite_binaries_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_residual_risk_baseline")
            && toml.contains("sdd_residual_risk_target")
            && !toml.contains("tests/contracts/residual_risk.baseline.rs")
            && toml.contains("tests/contracts/residual_risk.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
}

/// P09-T18: IR schema remains `assurance-ir/v1`; `Risk::new` / `risk.json` still decode.
#[test]
fn p09_t18_ir_schema_remains_v1_and_risk_still_decodes() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let risk = Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    );
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    let raw = read_repo_file("tests/fixtures/assurance-ir/v1/risk.json");
    let decoded: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(decoded.id.as_str(), "risk:source-tamper");
}

/// P09-T19: Projection reuses `ControlTestResult` / `Effectiveness` (no parallel enum).
#[test]
fn p09_t19_projection_reuses_canonical_control_test_effectiveness() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    let assurance = crate_sources_joined("weeping-angel-assurance");
    let src = format!("{ir}\n{assurance}");
    assert!(
        src.contains("ControlTestResult") && src.contains("Effectiveness"),
        "residual projection must reuse weeping_angel_control_test::ControlTestResult / Effectiveness"
    );
    assert!(
        !src.contains("enum ResidualEffectiveness") && !src.contains("enum ResidualControlResult"),
        "must not fork a parallel effectiveness enum"
    );
    let _ = Effectiveness::Effective;
    let _ = Effectiveness::ExceptionApproved;
}

/// P09-T20: Collectors / evidence crate sources still have no residual rating types.
#[test]
fn p09_t20_collectors_and_evidence_stay_conclusion_free() {
    let evidence = crate_sources_joined("weeping-angel-evidence");
    let collector = crate_sources_joined("weeping-angel-collector");
    for (label, src) in [
        ("evidence", evidence.as_str()),
        ("collector", collector.as_str()),
    ] {
        for needle in [
            "ResidualRiskProjection",
            "project_residual_risk",
            "ResidualRiskMode",
            "RiskRating",
        ] {
            assert!(
                !src.contains(needle),
                "{label} crate must not contain `{needle}`"
            );
        }
    }
    assert!(
        evidence.contains("Observations are facts, never compliance claims")
            || evidence.contains("never compliance claims"),
        "evidence crate must remain observation-only"
    );
}
