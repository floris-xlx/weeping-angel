//! Target suite for Operational ISMS v1 nonconformity / CAPA (Prompt 22).
//!
//! Encodes DESIRED behavior in `docs/specs/nonconformity-capa.md` §4 / §6
//! (NC-001…NC-012). On current HEAD the product APIs do not exist — this
//! binary must stay **RED** for the missing lifecycle (not harness noise).
//! Do not `#[ignore]`. Do not implement the engine here.
//!
//! Baseline (`sdd_nonconformity_capa_baseline`) stays GREEN until this suite
//! is GREEN and the absence tests are skip-superseded.

use std::path::PathBuf;

use chrono::{Duration, TimeZone, Utc};
use serde_json::json;
use weeping_angel_assurance::audit::record_finding;
use weeping_angel_assurance::capa::{
    close_nonconformity, evaluate_capa_effectiveness, open_nonconformities,
    overdue_corrective_actions, propose_from_audit_finding, propose_from_control_regression,
    propose_from_incident,
};
use weeping_angel_assurance::closed_incidents_with_open_corrective_actions;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Audit, AuditFinding, AuditFindingKind,
    AuditFindingSeverity, CapaError, ClosureDecision, ClosureOutcome, ContainmentAction, ControlId,
    ControlTestId, CorrectiveAction, CorrectiveActionId, CorrectiveActionKind,
    CorrectiveActionStatus, DetectionSource, EffectivenessCriteria, EffectivenessReviewStatus,
    EventRef, Incident, IncidentId, IncidentKind, IncidentSeverity, IncidentStatus, IsmsEvent,
    IsmsEventKind, Nonconformity, NonconformityClassification, NonconformityEventKind,
    NonconformityId, NonconformitySource, NonconformitySourceKind, NonconformityStatus,
    PostIncidentReview, PrincipalRef, RemediationRef, ReviewPeriod, RootCauseAnalysis, ValidateIr,
    VerificationMode, canonical_digest, validate_assessment_ir,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_framework::FrameworkCapabilities;

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn product_crates_joined() -> String {
    crate_sources_joined("weeping-angel-assurance-ir")
        + "\n"
        + &crate_sources_joined("weeping-angel-assurance")
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));

fn require_capa_engine(label: &str) {
    require_needles(
        label,
        &product_crates_joined(),
        &[
            "pub struct Nonconformity",
            "pub struct CorrectiveAction",
            "enum NonconformityStatus",
            "enum NonconformityClassification",
            "typed_id!(NonconformityId)",
            "typed_id!(CorrectiveActionId)",
            "fn propose_from_audit_finding",
            "fn propose_from_incident",
            "fn evaluate_capa_effectiveness",
            "fn overdue_corrective_actions",
            "fn close_nonconformity",
            "nonconformities: Vec<Nonconformity>",
            "corrective_actions: Vec<CorrectiveAction>",
        ],
    );
    assert!(
        crate_src("weeping-angel-assurance-ir")
            .join("capa.rs")
            .is_file()
            || crate_src("weeping-angel-assurance-ir")
                .join("nonconformity.rs")
                .is_file(),
        "{label}: expected capa/nonconformity IR module"
    );
    assert!(
        crate_src("weeping-angel-assurance")
            .join("capa.rs")
            .is_file(),
        "{label}: expected weeping-angel-assurance/src/capa.rs"
    );
}

fn clock() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap()
}

fn owner() -> PrincipalRef {
    PrincipalRef::Identity(weeping_angel_assurance_ir::IdentityId::new(
        "identity:capa-owner",
    ))
}

fn reviewer() -> PrincipalRef {
    PrincipalRef::Identity(weeping_angel_assurance_ir::IdentityId::new(
        "identity:capa-reviewer",
    ))
}

fn opener() -> PrincipalRef {
    PrincipalRef::Identity(weeping_angel_assurance_ir::IdentityId::new(
        "identity:capa-opener",
    ))
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.nonconformity-capa.target"))
}

fn manual_source() -> NonconformitySource {
    NonconformitySource {
        kind: NonconformitySourceKind::Manual,
        audit_finding_id: None,
        audit_id: None,
        incident_id: None,
        event_ref: None,
        control_ids: vec![ControlId::new("control.access.mfa")],
    }
}

fn default_criteria() -> EffectivenessCriteria {
    EffectivenessCriteria {
        mode: VerificationMode::SustainedWindow,
        window: Some(14 * 24 * 3600),
        min_effective_results: 2,
        independent_verifier: false,
        statement: "MFA must remain Effective for 14 days with at least two greens and no intervening fail.".into(),
        control_ids: vec![ControlId::new("control.access.mfa")],
    }
}

fn review_period(start: chrono::DateTime<Utc>) -> ReviewPeriod {
    ReviewPeriod {
        start,
        end: start + Duration::days(21),
    }
}

fn containment(at: chrono::DateTime<Utc>) -> ContainmentAction {
    ContainmentAction {
        id: "contain.mfa-exception-revoke".into(),
        description: "Revoke undocumented MFA exceptions and require re-enrollment.".into(),
        performed_at: at,
        performed_by: owner(),
        evidence_refs: vec!["sha256:containment-ticket".into()],
    }
}

fn rca(at: chrono::DateTime<Utc>) -> RootCauseAnalysis {
    RootCauseAnalysis {
        method: "5-why".into(),
        statement: "Privileged accounts were provisioned without the MFA enrollment gate.".into(),
        recorded_at: at,
        recorded_by: owner(),
        evidence_refs: vec!["sha256:rca-notes".into()],
    }
}

fn planned_action(
    nc_id: &NonconformityId,
    target: chrono::DateTime<Utc>,
    implemented_start: chrono::DateTime<Utc>,
) -> CorrectiveAction {
    CorrectiveAction::plan(
        CorrectiveActionId::new("ca.mfa-gate"),
        nc_id.clone(),
        CorrectiveActionKind::Corrective,
        "Enforce MFA enrollment at provision time",
        "Add a blocking enrollment check on privileged identity create/update.",
        owner(),
        target,
        default_criteria(),
        review_period(implemented_start),
        reviewer(),
    )
}

fn sample_result(effectiveness: Effectiveness, at: chrono::DateTime<Utc>) -> ControlTestResult {
    ControlTestResult {
        test_id: ControlTestId::new("test.access.mfa"),
        control_id: ControlId::new("control.access.mfa"),
        effectiveness,
        rationale: format!("CAPA observation {effectiveness:?}"),
        evidence_refs: vec!["sha256:mfa-observation".into()],
        missing_evidence: Vec::new(),
        checked_at: at,
        test_version: "1".into(),
        input_digest: "sha256:mfa-input".into(),
        duration: Some("12ms".into()),
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

fn open_nc() -> Nonconformity {
    Nonconformity::open(
        NonconformityId::new("nc.mfa-undocumented-exception"),
        "Undocumented MFA exception",
        "Privileged identities can authenticate without a second factor.",
        manual_source(),
        owner(),
        clock() - Duration::hours(2),
        clock(),
        opener(),
    )
}

fn persist(nc: Nonconformity, actions: Vec<CorrectiveAction>) -> AssessmentDefinition {
    let mut assessment = empty_assessment();
    assessment.nonconformities.push(nc);
    assessment.corrective_actions.extend(actions);
    assessment
}

fn history_kinds(nc: &Nonconformity) -> Vec<NonconformityEventKind> {
    nc.history.iter().map(|event| event.kind).collect()
}

fn err_text(err: &impl std::fmt::Display) -> String {
    err.to_string().to_ascii_lowercase()
}

fn is_missing_rca(err: &CapaError) -> bool {
    matches!(err, CapaError::MissingRootCause)
        || err_text(err).contains("root cause")
        || err_text(err).contains("rca")
}

fn is_unclassified(err: &CapaError) -> bool {
    matches!(err, CapaError::Unclassified) || err_text(err).contains("classif")
}

fn is_immutable(err: &CapaError) -> bool {
    matches!(err, CapaError::ImmutableClosure) || err_text(err).contains("immutable")
}

fn is_not_satisfied(err: &CapaError) -> bool {
    matches!(err, CapaError::EffectivenessNotSatisfied)
        || err_text(err).contains("not satisfied")
        || err_text(err).contains("effectiveness")
}

fn complete_through_implemented() -> (Nonconformity, CorrectiveAction, chrono::DateTime<Utc>) {
    let implemented_at = clock() + Duration::days(3);
    let mut nc = open_nc();
    nc.contain(
        containment(clock() + Duration::hours(1)),
        owner(),
        clock() + Duration::hours(1),
    )
    .expect("contain");
    nc.record_root_cause(
        rca(clock() + Duration::hours(2)),
        owner(),
        clock() + Duration::hours(2),
    )
    .expect("rca");
    nc.classify(
        NonconformityClassification::Major,
        "Privileged access without MFA is a major ISMS nonconformity.",
        opener(),
        clock() + Duration::hours(3),
    )
    .expect("classify");
    let mut action = planned_action(&nc.id, clock() + Duration::days(7), implemented_at);
    nc.plan_corrective_action(action.id.clone(), opener(), clock() + Duration::hours(4))
        .expect("plan");
    action
        .mark_implemented(implemented_at, vec!["sha256:impl-pr".into()])
        .expect("action implemented");
    nc.mark_implemented(opener(), implemented_at)
        .expect("nc implemented");
    (nc, action, implemented_at)
}

fn satisfy_window(
    nc: &mut Nonconformity,
    action: &CorrectiveAction,
    start: chrono::DateTime<Utc>,
) -> EffectivenessReviewStatus {
    nc.start_effectiveness_review(opener(), start)
        .expect("start review");
    let first = sample_result(Effectiveness::Effective, start);
    let second = sample_result(Effectiveness::Effective, start + Duration::days(14));
    let review = evaluate_capa_effectiveness(
        nc,
        std::slice::from_ref(action),
        &[first, second],
        start + Duration::days(14),
        reviewer(),
    )
    .expect("evaluate sustained window");
    nc.effectiveness = Some(review.clone());
    review.status
}

/// NC-001: Complete CAPA — open→contain→RCA→classify→plan→implement→Satisfied→explicit close.
#[test]
fn nc_001_complete_capa() {
    require_capa_engine("NC-001 Complete CAPA");

    let (mut nc, action, implemented_at) = complete_through_implemented();
    assert_eq!(nc.status, NonconformityStatus::Implemented);
    assert_eq!(action.status, CorrectiveActionStatus::Implemented);
    assert!(action.implemented_at.is_some());
    assert!(!action.implementation_evidence.is_empty());

    let status = satisfy_window(&mut nc, &action, implemented_at);
    assert_eq!(status, EffectivenessReviewStatus::Satisfied);
    assert_eq!(nc.status, NonconformityStatus::EffectivenessReview);

    close_nonconformity(
        &mut nc,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at: implemented_at + Duration::days(14),
            rationale: "Sustained window satisfied; MFA enrollment gate is in production.".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .expect("explicit close after Satisfied");
    assert_eq!(nc.status, NonconformityStatus::Closed);
    assert!(nc.closure.is_some());

    let assessment = persist(nc.clone(), vec![action.clone()]);
    assert_eq!(assessment.nonconformities.len(), 1);
    assert_eq!(assessment.corrective_actions.len(), 1);
    assert!(!assessment.requests.nonconformities);
    assessment.validate().expect("complete CAPA must validate");
    validate_assessment_ir(&assessment).expect("validate_assessment_ir Ok");

    let kinds = history_kinds(&nc);
    for required in [
        NonconformityEventKind::Opened,
        NonconformityEventKind::Contained,
        NonconformityEventKind::RootCauseRecorded,
        NonconformityEventKind::Classified,
        NonconformityEventKind::ActionPlanned,
        NonconformityEventKind::Implemented,
        NonconformityEventKind::ReviewStarted,
        NonconformityEventKind::Closed,
    ] {
        assert!(
            kinds.contains(&required),
            "NC-001: history must contain {required:?}; got {kinds:?}"
        );
    }

    let first = canonical_digest(&nc).expect("digest");
    let second = canonical_digest(&nc).expect("digest");
    assert_eq!(first, second, "NC-001: closed digest is stable");
    assert_eq!(open_nonconformities(&assessment).len(), 0);
}

/// NC-002: Missing root cause cannot leave Contained / reach RootCauseIdentified.
#[test]
fn nc_002_missing_root_cause() {
    require_capa_engine("NC-002 Missing root cause");

    let mut nc = open_nc();
    nc.contain(containment(clock()), owner(), clock())
        .expect("contain");
    assert_eq!(nc.status, NonconformityStatus::Contained);
    assert!(nc.root_cause.is_none());

    let err = nc
        .transition(
            NonconformityStatus::RootCauseIdentified,
            owner(),
            clock() + Duration::minutes(1),
        )
        .expect_err("NC-002: Contained → RootCauseIdentified without RCA must fail");
    assert!(is_missing_rca(&err), "NC-002: {err}");
    assert_eq!(nc.status, NonconformityStatus::Contained);

    let empty_statement = RootCauseAnalysis {
        method: "5-why".into(),
        statement: "   ".into(),
        recorded_at: clock(),
        recorded_by: owner(),
        evidence_refs: Vec::new(),
    };
    let err = nc
        .record_root_cause(empty_statement, owner(), clock())
        .expect_err("NC-002: blank RCA statement must fail");
    assert!(is_missing_rca(&err), "NC-002 blank: {err}");
    assert_eq!(nc.status, NonconformityStatus::Contained);

    let err = nc
        .transition(
            NonconformityStatus::CorrectiveActionPlanned,
            owner(),
            clock(),
        )
        .expect_err("NC-002: cannot skip RCA to planned");
    assert!(
        is_missing_rca(&err) || matches!(err, CapaError::InvalidTransition { .. }),
        "NC-002 skip: {err}"
    );
    assert_eq!(nc.status, NonconformityStatus::Contained);
}

/// NC-003: Overdue action is queryable; no auto-transition or reclassify.
#[test]
fn nc_003_overdue_action() {
    require_capa_engine("NC-003 Overdue action");

    let as_of = clock() + Duration::days(10);
    let mut nc = open_nc();
    nc.contain(containment(clock()), owner(), clock()).unwrap();
    nc.record_root_cause(rca(clock()), owner(), clock())
        .unwrap();
    nc.classify(
        NonconformityClassification::Minor,
        "Isolated enrollment gap.",
        opener(),
        clock(),
    )
    .unwrap();
    let action = planned_action(
        &nc.id,
        clock() + Duration::days(2),
        clock() + Duration::days(2),
    );
    nc.plan_corrective_action(action.id.clone(), opener(), clock())
        .unwrap();
    let classification = nc.classification;
    let status = nc.status;

    let assessment = persist(nc, vec![action.clone()]);
    let overdue = overdue_corrective_actions(&assessment, as_of);
    assert!(
        overdue.iter().any(|id| id.as_str() == "ca.mfa-gate"),
        "NC-003: targetDate < as_of and not implemented must be overdue; got {overdue:?}"
    );

    let after = &assessment.nonconformities[0];
    assert_eq!(after.status, status);
    assert_eq!(after.classification, classification);
    assert_eq!(
        assessment.corrective_actions[0].status,
        CorrectiveActionStatus::Planned
    );
    assert_ne!(
        assessment.corrective_actions[0].status,
        CorrectiveActionStatus::Implemented
    );
    assert_ne!(after.status, NonconformityStatus::Closed);

    assessment
        .validate()
        .expect("overdue is a query fact, not a validation mutation");
}

/// NC-004: Failed effectiveness review forbids Closed; return to Implemented/Planned is legal.
#[test]
fn nc_004_failed_effectiveness_review() {
    require_capa_engine("NC-004 Failed effectiveness review");

    let (mut nc, action, implemented_at) = complete_through_implemented();
    nc.start_effectiveness_review(opener(), implemented_at)
        .unwrap();
    let fail = sample_result(
        Effectiveness::Ineffective,
        implemented_at + Duration::days(1),
    );
    let review = evaluate_capa_effectiveness(
        &nc,
        std::slice::from_ref(&action),
        &[fail],
        implemented_at + Duration::days(1),
        reviewer(),
    )
    .expect("evaluate failed review");
    assert_eq!(review.status, EffectivenessReviewStatus::Failed);
    nc.effectiveness = Some(review);

    let err = close_nonconformity(
        &mut nc,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at: implemented_at + Duration::days(1),
            rationale: "one failed retest is not closure".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .expect_err("NC-004: Failed review forbids Closed");
    assert!(is_not_satisfied(&err), "NC-004 close: {err}");
    assert_ne!(nc.status, NonconformityStatus::Closed);

    nc.transition(
        NonconformityStatus::Implemented,
        opener(),
        implemented_at + Duration::days(2),
    )
    .expect("return to Implemented is legal");
    assert_eq!(nc.status, NonconformityStatus::Implemented);
    assert!(
        history_kinds(&nc).contains(&NonconformityEventKind::ReviewFailed)
            || nc
                .effectiveness
                .as_ref()
                .is_some_and(|r| r.status == EffectivenessReviewStatus::Failed)
    );

    nc.transition(
        NonconformityStatus::CorrectiveActionPlanned,
        opener(),
        implemented_at + Duration::days(3),
    )
    .expect("return to CorrectiveActionPlanned is legal");
    assert_eq!(nc.status, NonconformityStatus::CorrectiveActionPlanned);
}

/// NC-005: Closed→Open with principal+rationale; prior Closed remains; fresh Satisfied required.
#[test]
fn nc_005_reopened_nonconformity() {
    require_capa_engine("NC-005 Re-opened nonconformity");

    let (mut nc, action, implemented_at) = complete_through_implemented();
    assert_eq!(
        satisfy_window(&mut nc, &action, implemented_at),
        EffectivenessReviewStatus::Satisfied
    );
    let closed_at = implemented_at + Duration::days(14);
    close_nonconformity(
        &mut nc,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at,
            rationale: "Initial window satisfied.".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .unwrap();
    let version_closed = nc.version;
    let digest_closed = canonical_digest(&nc).unwrap();

    nc.reopen(
        opener(),
        "Regression observed after a privilege-provisioning change.",
        closed_at + Duration::days(1),
    )
    .expect("Closed → Open with principal + rationale");
    assert_eq!(nc.status, NonconformityStatus::Open);
    assert!(nc.version > version_closed);
    assert!(nc.closure.is_none());
    assert!(
        history_kinds(&nc).contains(&NonconformityEventKind::Closed),
        "NC-005: prior Closed remains in history"
    );
    assert!(history_kinds(&nc).contains(&NonconformityEventKind::Reopened));
    assert_ne!(canonical_digest(&nc).unwrap(), digest_closed);

    let err = close_nonconformity(
        &mut nc,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at: closed_at + Duration::days(2),
            rationale: "reuse the old review".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .expect_err("NC-005: new close needs a fresh Satisfied review");
    assert!(is_not_satisfied(&err) || matches!(err, CapaError::InvalidTransition { .. }));
}

/// NC-006: Default SustainedWindow — one green / 3-day pair fail; 14d span + explicit close succeeds.
#[test]
fn nc_006_sustained_verification_window() {
    require_capa_engine("NC-006 Sustained verification window");

    let (mut nc, action, start) = complete_through_implemented();
    nc.start_effectiveness_review(opener(), start).unwrap();
    assert_eq!(
        action.effectiveness_criteria.mode,
        VerificationMode::SustainedWindow
    );
    assert_eq!(action.effectiveness_criteria.min_effective_results, 2);
    assert_eq!(action.effectiveness_criteria.window, Some(14 * 24 * 3600));

    let one = evaluate_capa_effectiveness(
        &nc,
        std::slice::from_ref(&action),
        &[sample_result(Effectiveness::Effective, start)],
        start,
        reviewer(),
    )
    .expect("one green evaluates");
    assert_ne!(one.status, EffectivenessReviewStatus::Satisfied);
    assert_ne!(nc.status, NonconformityStatus::Closed);

    let three_days = evaluate_capa_effectiveness(
        &nc,
        std::slice::from_ref(&action),
        &[
            sample_result(Effectiveness::Effective, start),
            sample_result(Effectiveness::Effective, start + Duration::days(3)),
        ],
        start + Duration::days(3),
        reviewer(),
    )
    .expect("3-day pair evaluates");
    assert_ne!(
        three_days.status,
        EffectivenessReviewStatus::Satisfied,
        "NC-006: two greens 3 days apart on a 14d window are not Satisfied"
    );

    let intervening = evaluate_capa_effectiveness(
        &nc,
        std::slice::from_ref(&action),
        &[
            sample_result(Effectiveness::Effective, start),
            sample_result(Effectiveness::Ineffective, start + Duration::days(7)),
            sample_result(Effectiveness::Effective, start + Duration::days(14)),
        ],
        start + Duration::days(14),
        reviewer(),
    )
    .expect("intervening fail evaluates");
    assert_ne!(intervening.status, EffectivenessReviewStatus::Satisfied);

    let ok = evaluate_capa_effectiveness(
        &nc,
        std::slice::from_ref(&action),
        &[
            sample_result(Effectiveness::Effective, start),
            sample_result(Effectiveness::Effective, start + Duration::days(14)),
        ],
        start + Duration::days(14),
        reviewer(),
    )
    .expect("14d pair evaluates");
    assert_eq!(ok.status, EffectivenessReviewStatus::Satisfied);
    nc.effectiveness = Some(ok);
    close_nonconformity(
        &mut nc,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at: start + Duration::days(14),
            rationale: "Span covers the declared 14-day window with no intervening fail.".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .expect("explicit close after Satisfied window");
    assert_eq!(nc.status, NonconformityStatus::Closed);
}

/// NC-007: Audit linkage — propose binds finding/audit ids; kind=nonconformity does not start CAPA.
#[test]
fn nc_007_audit_linkage() {
    require_capa_engine("NC-007 Audit linkage");

    let finding: AuditFinding = serde_json::from_value(json!({
        "id": "finding.audit.mfa-gap",
        "auditId": "audit.q1-access",
        "kind": "nonconformity",
        "severity": "major",
        "title": "MFA exception undocumented",
        "description": "Auditor labelled a nonconformity; that is not a CAPA open.",
        "controlIds": ["control.access.mfa"],
        "requirementIds": ["iso27001:9.2"],
        "evidenceDigests": [],
        "createdBy": { "identity": "identity:auditor" },
        "createdAt": "2026-01-16T00:00:00Z",
        "nonconformityId": "nc:opaque-prompt-22"
    }))
    .unwrap();
    assert_eq!(finding.kind, AuditFindingKind::Nonconformity);
    assert_eq!(finding.severity, Some(AuditFindingSeverity::Major));

    let mut audit: Audit = serde_json::from_value(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "audit.q1-access",
        "programId": "audit:2026",
        "title": "Q1 access-control audit",
        "period": {
            "start": "2026-01-15T00:00:00Z",
            "end": "2026-03-15T00:00:00Z"
        },
        "scope": { "organizations": ["org:weeping-angel"], "subjects": [], "exclusions": [] },
        "selectedControls": ["control.access.mfa"],
        "selectedRequirements": ["iso27001:9.2"],
        "procedures": [],
        "observations": [],
        "findings": [],
        "nonconformityRefs": [],
        "status": "inProgress",
        "history": []
    }))
    .unwrap();
    let mut findings = Vec::new();
    record_finding(&mut audit, &mut findings, finding.clone()).unwrap();

    let mut labelled = empty_assessment();
    labelled.audits.push(audit);
    labelled.audit_findings.extend(findings);
    assert!(
        labelled.nonconformities.is_empty(),
        "NC-007: kind=nonconformity alone must not start CAPA"
    );

    let proposed = propose_from_audit_finding(
        &finding,
        NonconformityId::new("nc.from-audit"),
        owner(),
        clock(),
        opener(),
    )
    .expect("propose_from_audit_finding");
    assert_eq!(proposed.status, NonconformityStatus::Open);
    assert!(proposed.classification.is_none());
    assert_eq!(proposed.source.kind, NonconformitySourceKind::AuditFinding);
    assert_eq!(
        proposed
            .source
            .audit_finding_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("finding.audit.mfa-gap")
    );
    assert_eq!(
        proposed.source.audit_id.as_ref().map(|id| id.as_str()),
        Some("audit.q1-access")
    );

    let opaque = empty_assessment();
    let mut with_finding = opaque.clone();
    with_finding.audit_findings.push(finding.clone());
    // Empty CAPA inventory: opaque Prompt 21 strings stay valid at the CAPA seam.
    assert!(opaque.nonconformities.is_empty());

    let mut dangling = persist(proposed, Vec::new());
    let mut dangling_finding = finding;
    dangling_finding.nonconformity_id = Some("nc:does-not-exist".into());
    dangling.audit_findings.push(dangling_finding);
    dangling
        .validate()
        .expect_err("NC-007: dangling nonconformityId fails only when CAPA inventory is non-empty");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("impl From<AuditFinding> for Nonconformity"),
        "NC-007: no From<AuditFinding> for Nonconformity"
    );
}

/// NC-008: Incident linkage — bind IncidentId; RemediationRef fields stay; incident close ≠ CAPA close.
#[test]
fn nc_008_incident_linkage() {
    require_capa_engine("NC-008 Incident linkage");

    let at = clock();
    let mut incident = Incident::declare(
        IncidentId::new("inc.capa.target"),
        IncidentKind::Real,
        "Control failure in production",
        DetectionSource::Manual,
        at,
        owner(),
    );
    incident.severity = Some(IncidentSeverity::Critical);
    incident
        .corrective_action_ids
        .push(RemediationRef::new("rem:prompt-16"));
    incident.post_incident_review = Some(PostIncidentReview {
        recorded_at: at,
        recorded_by: owner(),
        root_cause: Some("human error".into()),
        lessons_learned: "contain faster".into(),
        proposed_risk_ids: Vec::new(),
        proposed_control_ids: Vec::new(),
        proposed_corrective_action_ids: vec![RemediationRef::new("rem:pir-candidate")],
        evidence_refs: Vec::new(),
    });
    incident
        .transition(IncidentStatus::Contained, at, owner())
        .unwrap();
    incident
        .transition(IncidentStatus::Recovered, at, owner())
        .unwrap();
    incident
        .transition(IncidentStatus::Closed, at, owner())
        .unwrap();
    assert_eq!(incident.status, IncidentStatus::Closed);

    let proposed = propose_from_incident(
        &incident,
        NonconformityId::new("nc.from-incident"),
        owner(),
        at,
        opener(),
    )
    .expect("propose_from_incident");
    assert_eq!(proposed.source.kind, NonconformitySourceKind::Incident);
    assert_eq!(
        proposed.source.incident_id.as_ref().map(|id| id.as_str()),
        Some("inc.capa.target")
    );
    assert!(
        proposed.classification.is_none(),
        "NC-008: incident severity must not become CAPA classification"
    );
    assert_eq!(proposed.status, NonconformityStatus::Open);

    let mut assessment = persist(proposed, Vec::new());
    assessment.incidents.push(incident.clone());
    let open = closed_incidents_with_open_corrective_actions(&assessment);
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].corrective_action_ids[0].as_str(), "rem:prompt-16");
    assert_eq!(
        assessment.nonconformities[0].status,
        NonconformityStatus::Open
    );

    let pir = incident.post_incident_review.as_ref().unwrap();
    let _as_remediation_refs: &[RemediationRef] = &pir.proposed_corrective_action_ids;
    assert_eq!(
        pir.proposed_corrective_action_ids[0].as_str(),
        "rem:pir-candidate"
    );

    let incident_src = read_repo_file("crates/weeping-angel-assurance-ir/src/incident.rs");
    assert!(
        incident_src.contains("pub corrective_action_ids: Vec<RemediationRef>")
            && incident_src.contains("pub proposed_corrective_action_ids: Vec<RemediationRef>"),
        "NC-008: do not retarget incident corrective-action ids"
    );
}

/// NC-009: Immutable closure — Closed/Cancelled/Superseded reject mutation; rationale + successor.
#[test]
fn nc_009_immutable_closure() {
    require_capa_engine("NC-009 Immutable closure");

    let (mut closed, action, implemented_at) = complete_through_implemented();
    assert_eq!(
        satisfy_window(&mut closed, &action, implemented_at),
        EffectivenessReviewStatus::Satisfied
    );
    close_nonconformity(
        &mut closed,
        ClosureDecision {
            closed_by: reviewer(),
            closed_at: implemented_at + Duration::days(14),
            rationale: "Window satisfied.".into(),
            outcome: ClosureOutcome::ClosedEffective,
        },
    )
    .unwrap();
    let err = closed
        .classify(
            NonconformityClassification::Minor,
            "rewrite after close",
            opener(),
            clock(),
        )
        .expect_err("NC-009: Closed rejects in-place classify");
    assert!(is_immutable(&err), "NC-009 closed classify: {err}");
    let err = closed
        .contain(containment(clock()), owner(), clock())
        .expect_err("NC-009: Closed rejects contain");
    assert!(is_immutable(&err), "NC-009 closed contain: {err}");

    let mut cancel = open_nc();
    cancel
        .cancel("", opener(), clock())
        .expect_err("NC-009: cancel requires rationale");
    cancel
        .cancel("Duplicate of an already-tracked gap.", opener(), clock())
        .expect("cancel with rationale");
    assert_eq!(cancel.status, NonconformityStatus::Cancelled);
    let err = cancel
        .record_root_cause(rca(clock()), owner(), clock())
        .expect_err("NC-009: Cancelled is immutable");
    assert!(is_immutable(&err), "NC-009 cancelled: {err}");

    let mut superseded = open_nc();
    superseded
        .supersede(
            NonconformityId::new("nc.mfa-undocumented-exception"),
            "self is not a successor",
            opener(),
            clock(),
        )
        .expect_err("NC-009: successor must not be self");
    superseded
        .supersede(NonconformityId::new("nc.successor"), "", opener(), clock())
        .expect_err("NC-009: supersede requires rationale");
    superseded
        .supersede(
            NonconformityId::new("nc.successor"),
            "Replaced by a broader access-control CAPA.",
            opener(),
            clock(),
        )
        .expect("supersede with successor + rationale");
    assert_eq!(superseded.status, NonconformityStatus::Superseded);
    assert_eq!(
        superseded.superseded_by.as_ref().map(|id| id.as_str()),
        Some("nc.successor")
    );
    let err = superseded
        .classify(NonconformityClassification::Major, "no", opener(), clock())
        .expect_err("NC-009: Superseded is immutable");
    assert!(is_immutable(&err), "NC-009 superseded: {err}");
}

/// NC-010: No silent classification — propose leaves classification unset; classify is a decision.
#[test]
fn nc_010_no_silent_classification() {
    require_capa_engine("NC-010 No silent classification");

    let finding: AuditFinding = serde_json::from_value(json!({
        "id": "finding.audit.mfa-gap",
        "auditId": "audit.q1-access",
        "kind": "nonconformity",
        "severity": "major",
        "title": "MFA exception undocumented",
        "description": "severity on the finding is not CAPA classification",
        "controlIds": ["control.access.mfa"],
        "requirementIds": [],
        "evidenceDigests": [],
        "createdBy": { "identity": "identity:auditor" },
        "createdAt": "2026-03-01T12:00:00Z"
    }))
    .unwrap();
    let from_finding = propose_from_audit_finding(
        &finding,
        NonconformityId::new("nc.classify.finding"),
        owner(),
        clock(),
        opener(),
    )
    .unwrap();
    assert!(from_finding.classification.is_none());

    let mut incident = Incident::declare(
        IncidentId::new("inc.classify"),
        IncidentKind::Real,
        "Severity must not copy",
        DetectionSource::Manual,
        clock(),
        owner(),
    );
    incident.severity = Some(IncidentSeverity::Critical);
    let from_incident = propose_from_incident(
        &incident,
        NonconformityId::new("nc.classify.incident"),
        owner(),
        clock(),
        opener(),
    )
    .unwrap();
    assert!(from_incident.classification.is_none());

    let event = IsmsEvent::new(
        IsmsEventKind::ControlRegressed,
        clock(),
        "sha256:prev",
        "sha256:next",
        Vec::new(),
        Vec::new(),
        None,
        json!({ "controlId": "control.access.mfa" }),
    );
    let from_event = propose_from_control_regression(
        &event,
        NonconformityId::new("nc.classify.regression"),
        owner(),
        clock(),
        opener(),
    )
    .unwrap();
    assert!(from_event.classification.is_none());
    assert_eq!(
        from_event.source.kind,
        NonconformitySourceKind::ControlRegression
    );
    assert!(from_event.source.event_ref.is_some() || event.event_id.as_str().starts_with("event:"));
    let _event_ref: Option<EventRef> = from_event.source.event_ref.clone();

    let mut nc = open_nc();
    nc.contain(containment(clock()), owner(), clock()).unwrap();
    nc.record_root_cause(rca(clock()), owner(), clock())
        .unwrap();
    nc.classify(NonconformityClassification::Major, "", opener(), clock())
        .expect_err("NC-010: classify requires non-empty rationale");

    let err = nc
        .plan_corrective_action(
            CorrectiveActionId::new("ca.unclassified"),
            opener(),
            clock(),
        )
        .expect_err("NC-010: unclassified cannot reach CorrectiveActionPlanned");
    assert!(is_unclassified(&err) || matches!(err, CapaError::InvalidTransition { .. }));
    assert_eq!(nc.status, NonconformityStatus::RootCauseIdentified);
    assert!(nc.classification.is_none());

    nc.classify(
        NonconformityClassification::Opportunity,
        "Improvement, not a major/minor copy from the finding.",
        opener(),
        clock(),
    )
    .expect("classify with principal + rationale");
    assert_eq!(
        nc.classification,
        Some(NonconformityClassification::Opportunity)
    );
}

/// NC-011: Flags and catalog fence — defaults stay false; inventories do not flip them.
#[test]
fn nc_011_flags_and_catalog_fence() {
    require_capa_engine("NC-011 Flags and catalog fence");

    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    assert!(!weeping_angel_assurance_ir::AssessmentRequests::default().nonconformities);
    assert!(!FrameworkCapabilities::default().supports_nonconformities);

    let (mut nc, action, implemented_at) = complete_through_implemented();
    let _ = satisfy_window(&mut nc, &action, implemented_at);
    let assessment = persist(nc, vec![action]);
    assert!(
        !assessment.nonconformities.is_empty() && !assessment.corrective_actions.is_empty(),
        "NC-011: inventories exist independently of the request flag"
    );
    assert!(
        !assessment.requests.nonconformities,
        "NC-011: presence of CAPA inventories must not flip requests.nonconformities"
    );
    assert!(!FrameworkCapabilities::default().supports_nonconformities);

    let catalog = CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load");
    catalog
        .control("control.governance.corrective-action")
        .expect("catalog attestation control must remain");
    assert!(
        catalog
            .tests()
            .contains_key("test.governance.corrective-action-recorded")
    );
    let toml = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(toml.contains("id = \"control.governance.corrective-action\""));
}

/// NC-012: Dual-suite registration + spec listed in CANONICAL_SPECS at implement.
#[test]
fn nc_012_dual_suite_registration() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        !cargo.contains("name = \"sdd_nonconformity_capa_baseline\"")
            && !cargo.contains("path = \"tests/contracts/nonconformity_capa.baseline.rs\"")
            && cargo.contains("name = \"sdd_nonconformity_capa_target\"")
            && cargo.contains("path = \"tests/contracts/nonconformity_capa.target.rs\"")
    );
    assert!(
        manifest_dir()
            .join("docs/specs/nonconformity-capa.md")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("docs/adr/0028-nonconformity-capa.md")
            .is_file()
    );
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/nonconformity-capa.md"),
        "NC-012: add this spec to CANONICAL_SPECS at implement"
    );
    require_capa_engine("NC-012 Dual-suite");
}
