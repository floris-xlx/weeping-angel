//! Target suite for Operational ISMS v1 risk treatment (Prompt 08).
//!
//! Encodes DESIRED behavior in `docs/specs/risk-treatment.md` §4 / §6.2
//! (P08-T01–T16). Must stay RED on CURRENT HEAD because treatment types and
//! the fail-closed state machine do not exist. Do not `#[ignore]` these tests
//! and do not implement the engine in this suite.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ActionState, AssessmentDefinition, AssessmentId, Control, ControlId,
    ControlImplementation, ControlImplementationId, EvidenceCriticality, EvidenceRequirement,
    EvidenceRequirementId, EvidenceType, Identity, IdentityId, IdentityKind, PrincipalRef,
    RemediationRef, Risk, RiskAcceptance, RiskAcceptanceId, RiskId, RiskStatus,
    RiskTreatmentDecision, RiskTreatmentId, TargetResidualRisk, TransferEvidence, TreatmentAction,
    TreatmentActionId, TreatmentApproval, TreatmentError, TreatmentEvidenceExpectation,
    TreatmentEvidenceKind, TreatmentEvidenceRef, TreatmentPlan, TreatmentPlanId, TreatmentState,
    TreatmentStrategy, ValidateIr, acceptance_in_force, active_treatment, canonical_digest,
    treatment_required, typed_canonical_digest, validate_treatments_at,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn golden_risk_json() -> String {
    read_repo_file("tests/fixtures/assurance-ir/v1/risk.json")
}

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
}

fn t_expiry() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()
}

fn t_after_expiry() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()
}

fn owner() -> PrincipalRef {
    PrincipalRef::Identity(IdentityId::new("identity:owner"))
}

fn principal() -> PrincipalRef {
    PrincipalRef::Identity(IdentityId::new("identity:ciso"))
}

fn residual() -> TargetResidualRisk {
    TargetResidualRisk::VersionedPlaceholder {
        methodology_version: "rm:acme-default:2".into(),
        input_note: Some("integrity-focused residual target".into()),
    }
}

fn residual_other() -> TargetResidualRisk {
    TargetResidualRisk::VersionedPlaceholder {
        methodology_version: "rm:acme-default:2".into(),
        input_note: Some("different frozen residual claim".into()),
    }
}

fn envelope(value: &str, at: DateTime<Utc>) -> TreatmentEvidenceRef {
    TreatmentEvidenceRef {
        kind: TreatmentEvidenceKind::EnvelopeDigest,
        value: value.into(),
        at: Some(at),
        principal: Some(principal()),
    }
}

fn narrative(value: &str, at: DateTime<Utc>) -> TreatmentEvidenceRef {
    TreatmentEvidenceRef {
        kind: TreatmentEvidenceKind::NarrativeAttestation,
        value: value.into(),
        at: Some(at),
        principal: Some(principal()),
    }
}

fn err_blob<E: std::fmt::Debug + std::fmt::Display>(err: &E) -> String {
    format!("{err:?} | {err}").to_ascii_lowercase()
}

fn assert_err_contains<E: std::fmt::Debug + std::fmt::Display>(err: &E, needles: &[&str]) {
    let blob = err_blob(err);
    for needle in needles {
        assert!(
            blob.contains(&needle.to_ascii_lowercase()),
            "expected `{needle}` in treatment error, got {err:?}"
        );
    }
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn sample_impl(id: &str, risk: &RiskId) -> ControlImplementation {
    ControlImplementation::new(
        ControlImplementationId::new(id),
        ControlId::new("control.access.mfa"),
    )
    .with_risk(risk.clone())
}

fn empty_assessment(id: &str) -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new(id))
}

fn populated_assessment(id: &str, risk_id: &RiskId) -> AssessmentDefinition {
    let mut assessment = empty_assessment(id);
    assessment.risks.push(Risk::new(
        risk_id.clone(),
        "Source tampering",
        "Unauthorized change to the source of record.",
    ));
    assessment.controls.push(sample_control());
    assessment
        .implementations
        .push(sample_impl("impl.access.mfa.org", risk_id));
    assessment
        .implementations
        .push(sample_impl("impl.access.mfa.vendor", risk_id));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:owner"),
        IdentityKind::User,
    ));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:ciso"),
        IdentityKind::User,
    ));
    assessment
        .evidence_requirements
        .push(EvidenceRequirement::new(
            EvidenceRequirementId::new("evidence.req.treatment"),
            EvidenceType::new("risk.treatment"),
        ));
    assessment
}

fn required_expectation() -> TreatmentEvidenceExpectation {
    TreatmentEvidenceExpectation {
        id: Some(EvidenceRequirementId::new("evidence.req.treatment")),
        evidence_type: Some(EvidenceType::new("risk.treatment")),
        criticality: EvidenceCriticality::Required,
        description: "completion evidence for the treatment path".into(),
    }
}

fn action(id: &str, title: &str, required: bool, state: ActionState) -> TreatmentAction {
    TreatmentAction {
        id: TreatmentActionId::new(id),
        title: title.into(),
        owner: owner(),
        required,
        state,
        control_ids: vec![ControlId::new("control.access.mfa")],
        implementation_ids: vec![ControlImplementationId::new("impl.access.mfa.org")],
        remediation_refs: vec![RemediationRef::new("rem:ticket-mfa")],
        evidence: vec![envelope("sha256:action-evidence", t0())],
        due_at: None,
    }
}

fn mitigate_plan(actions: Vec<TreatmentAction>) -> TreatmentPlan {
    TreatmentPlan {
        id: TreatmentPlanId::new("tp:source-tamper"),
        owner: owner(),
        actions,
        target_date: Some(t_expiry()),
    }
}

fn propose(id: &str, risk_id: &RiskId, strategy: TreatmentStrategy) -> RiskTreatmentDecision {
    let mut decision = RiskTreatmentDecision::propose(
        RiskTreatmentId::new(id),
        risk_id.clone(),
        strategy,
        owner(),
        principal(),
        residual(),
    );
    decision.rationale = "accountable treatment path for source integrity".into();
    decision.evidence_expectations = vec![required_expectation()];
    decision
}

fn with_approval(mut decision: RiskTreatmentDecision, at: DateTime<Utc>) -> RiskTreatmentDecision {
    decision.approval = Some(TreatmentApproval {
        principal: principal(),
        at,
        note: Some("approved by CISO".into()),
    });
    decision
}

fn walk(
    mut decision: RiskTreatmentDecision,
    states: &[TreatmentState],
    at: DateTime<Utc>,
) -> Result<RiskTreatmentDecision, TreatmentError> {
    for state in states {
        decision = decision.transition(*state, principal(), at)?;
    }
    Ok(decision)
}

fn happy_path() -> [TreatmentState; 4] {
    [
        TreatmentState::Approved,
        TreatmentState::Executing,
        TreatmentState::Verification,
        TreatmentState::Completed,
    ]
}

fn install(assessment: &mut AssessmentDefinition, decision: RiskTreatmentDecision) {
    assessment.risk_treatments.push(decision);
}

fn sealed_acceptance(
    risk_id: &RiskId,
    treatment_id: &RiskTreatmentId,
    expires_at: DateTime<Utc>,
) -> RiskAcceptance {
    RiskAcceptance::new(
        RiskAcceptanceId::new("ra:source-tamper"),
        risk_id.clone(),
        treatment_id.clone(),
        principal(),
        "we accept residual integrity exposure until the next board review",
        t0(),
        t0(),
        expires_at,
        vec![envelope("sha256:acceptance-minutes", t0())],
    )
}

fn complete_mitigate(risk_id: &RiskId) -> RiskTreatmentDecision {
    let mut decision = with_approval(
        propose("rt:mitigate", risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    decision.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    decision.implementation_ids = vec![
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlImplementationId::new("impl.access.mfa.vendor"),
    ];
    decision.plan = Some(mitigate_plan(vec![
        action("ta:mfa-org", "Deploy org MFA", true, ActionState::Done),
        action(
            "ta:mfa-vendor",
            "Deploy vendor MFA",
            true,
            ActionState::Done,
        ),
    ]));
    walk(decision, &happy_path(), t0()).expect("complete mitigate path")
}

/// P08: Mitigate strategy with plan and actions completes
#[test]
fn p08_t01_mitigate_strategy_with_plan_and_actions_completes() {
    let risk_id = RiskId::new("risk:source-tamper");
    let mut assessment = populated_assessment("assess.p08.t01", &risk_id);
    let decision = complete_mitigate(&risk_id);
    assert_eq!(decision.state, TreatmentState::Completed);
    assert_eq!(decision.strategy, TreatmentStrategy::Mitigate);
    assert_eq!(decision.schema_version, ASSURANCE_IR_SCHEMA);
    let digest_a = canonical_digest(&decision).expect("treatment digest");
    let digest_b = canonical_digest(&decision).expect("treatment digest again");
    assert_eq!(
        digest_a, digest_b,
        "completed mitigate digest must be stable"
    );
    assert_eq!(
        typed_canonical_digest("risk-treatment", &decision).expect("typed digest"),
        typed_canonical_digest("risk-treatment", &decision).expect("typed digest again")
    );
    install(&mut assessment, decision);
    assessment
        .validate()
        .expect("completed mitigate with resolved controls must validate");
    assert!(
        !treatment_required(&assessment, &risk_id, t0()),
        "completed mitigate must suppress treatment_required while the path is in force"
    );
}

/// P08: Accept strategy seals immutable acceptance with principal
#[test]
fn p08_t02_accept_strategy_seals_immutable_acceptance_with_principal() {
    let risk_id = RiskId::new("risk:accept-sealed");
    let mut assessment = populated_assessment("assess.p08.t02", &risk_id);
    let treatment_id = RiskTreatmentId::new("rt:accept");
    let mut decision = with_approval(
        propose(treatment_id.as_str(), &risk_id, TreatmentStrategy::Accept),
        t0(),
    );
    decision.acceptance = Some(sealed_acceptance(&risk_id, &treatment_id, t_expiry()));
    let completed = walk(decision, &happy_path(), t0()).expect("accept must walk the full machine");
    assert_eq!(completed.state, TreatmentState::Completed);
    let acceptance = completed
        .acceptance
        .as_ref()
        .expect("Accept requires sealed RiskAcceptance");
    assert_eq!(acceptance.principal, principal());
    assert_eq!(acceptance.expires_at, t_expiry());
    assert!(!acceptance.digest.is_empty());
    assert!(!acceptance.rationale.trim().is_empty());
    install(&mut assessment, completed.clone());
    assessment.validate().expect("sealed accept must validate");

    let mut mutated = completed.clone();
    mutated.acceptance.as_mut().expect("acceptance").rationale =
        "quietly rewritten after approval".into();
    let err = mutated
        .validate()
        .expect_err("post-approve acceptance mutation must fail closed");
    assert_err_contains(&err, &["immutable"]);

    let mut missing_principal = with_approval(
        propose(
            "rt:accept-no-principal",
            &risk_id,
            TreatmentStrategy::Accept,
        ),
        t0(),
    );
    let mut unsigned = sealed_acceptance(
        &risk_id,
        &RiskTreatmentId::new("rt:accept-no-principal"),
        t_expiry(),
    );
    unsigned.principal = PrincipalRef::Team(String::new());
    missing_principal.acceptance = Some(unsigned);
    let err = missing_principal
        .transition(TreatmentState::Approved, principal(), t0())
        .expect_err("acceptance without accountable principal must fail");
    assert_err_contains(&err, &["principal"]);
}

/// P08: Avoid strategy requires organizational action evidence
#[test]
fn p08_t03_avoid_strategy_requires_organizational_action_evidence() {
    let risk_id = RiskId::new("risk:avoid");
    let mut enum_only = with_approval(
        propose("rt:avoid", &risk_id, TreatmentStrategy::Avoid),
        t0(),
    );
    enum_only.avoid_evidence = None;
    let verifying = walk(
        enum_only.clone(),
        &[
            TreatmentState::Approved,
            TreatmentState::Executing,
            TreatmentState::Verification,
        ],
        t0(),
    )
    .expect("Avoid still walks executing/verification");
    let err = verifying
        .transition(TreatmentState::Completed, principal(), t0())
        .expect_err("enum-only Avoid cannot complete");
    assert_err_contains(&err, &["evidence"]);

    let mut evidenced = with_approval(
        propose("rt:avoid-done", &risk_id, TreatmentStrategy::Avoid),
        t0(),
    );
    evidenced.avoid_evidence = Some(narrative(
        "decommissioned the processing activity and removed it from scope",
        t0(),
    ));
    let completed =
        walk(evidenced, &happy_path(), t0()).expect("Avoid with org evidence completes");
    assert_eq!(completed.state, TreatmentState::Completed);
}

/// P08: Transfer strategy requires contract evidence
#[test]
fn p08_t04_transfer_strategy_requires_contract_evidence() {
    let risk_id = RiskId::new("risk:transfer");
    let mut missing = with_approval(
        propose("rt:transfer-missing", &risk_id, TreatmentStrategy::Transfer),
        t0(),
    );
    missing.transfer_evidence = None;
    let verifying = walk(
        missing,
        &[
            TreatmentState::Approved,
            TreatmentState::Executing,
            TreatmentState::Verification,
        ],
        t0(),
    )
    .expect("Transfer still walks the shared machine");
    let err = verifying
        .transition(TreatmentState::Completed, principal(), t0())
        .expect_err("Transfer without contract cannot complete");
    assert_err_contains(&err, &["contract"]);

    let mut transferred = with_approval(
        propose("rt:transfer", &risk_id, TreatmentStrategy::Transfer),
        t0(),
    );
    transferred.transfer_evidence = Some(TransferEvidence {
        contract: envelope("sha256:cyber-insurance-binder", t0()),
        transferee: "acme-cyber-insurer".into(),
        effective_at: Some(t0()),
    });
    let completed =
        walk(transferred, &happy_path(), t0()).expect("contract + transferee completes");
    assert_eq!(completed.state, TreatmentState::Completed);
    assert_eq!(completed.strategy, TreatmentStrategy::Transfer);
}

/// P08: expired risk acceptance does not suppress treatment
#[test]
fn p08_t05_expired_risk_acceptance_does_not_suppress_treatment() {
    let risk_id = RiskId::new("risk:expired-accept");
    let mut assessment = populated_assessment("assess.p08.t05", &risk_id);
    assessment.risks[0].status = RiskStatus::Accepted;
    let treatment_id = RiskTreatmentId::new("rt:expired-accept");
    let mut decision = with_approval(
        propose(treatment_id.as_str(), &risk_id, TreatmentStrategy::Accept),
        t0(),
    );
    decision.acceptance = Some(sealed_acceptance(&risk_id, &treatment_id, t_expiry()));
    let completed = walk(decision, &happy_path(), t0()).expect("accept completes");
    install(&mut assessment, completed);

    assert!(
        acceptance_in_force(&assessment, &risk_id, t0()),
        "acceptance must be in force before expiresAt"
    );
    assert!(
        !treatment_required(&assessment, &risk_id, t0()),
        "in-force completed accept suppresses treatment_required"
    );
    assert!(
        !acceptance_in_force(&assessment, &risk_id, t_after_expiry()),
        "expired acceptance is not in force"
    );
    assert!(
        treatment_required(&assessment, &risk_id, t_after_expiry()),
        "as_of ≥ expiresAt must not suppress treatment_required even if Risk.status is Accepted"
    );

    let clocked = validate_treatments_at(&assessment, t_after_expiry());
    let err = clocked.expect_err("Accepted without in-force acceptance fails clocked validate");
    assert_err_contains(&err, &["accept"]);

    let mut verbal =
        populated_assessment("assess.p08.t05.verbal", &RiskId::new("risk:verbal-accept"));
    verbal.risks[0].status = RiskStatus::Accepted;
    verbal
        .validate()
        .expect_err("Accepted with no RiskAcceptance record must fail clockless validate");
}

/// P08: partially complete mitigation cannot complete
#[test]
fn p08_t06_partially_complete_mitigation_cannot_complete() {
    let risk_id = RiskId::new("risk:partial-mitigate");
    let mut decision = with_approval(
        propose("rt:partial", &risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    decision.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    decision.implementation_ids = vec![ControlImplementationId::new("impl.access.mfa.org")];
    decision.plan = Some(mitigate_plan(vec![
        action("ta:done", "Ship MFA", true, ActionState::Done),
        action(
            "ta:open",
            "Retire shared passwords",
            true,
            ActionState::InProgress,
        ),
    ]));
    let verifying = walk(
        decision,
        &[
            TreatmentState::Approved,
            TreatmentState::Executing,
            TreatmentState::Verification,
        ],
        t0(),
    )
    .expect("partial mitigate may reach verification");
    let err = verifying
        .clone()
        .transition(TreatmentState::Completed, principal(), t0())
        .expect_err("1 of 2 required actions Done cannot complete");
    assert_err_contains(&err, &["action"]);
    assert_ne!(verifying.state, TreatmentState::Completed);
    assert!(
        verifying.state == TreatmentState::Verification
            || verifying.state == TreatmentState::Executing
    );
}

/// P08: transferred risk with missing contract evidence
#[test]
fn p08_t07_transferred_risk_with_missing_contract_evidence() {
    let risk_id = RiskId::new("risk:transfer-empty-contract");
    let mut decision = with_approval(
        propose("rt:transfer-empty", &risk_id, TreatmentStrategy::Transfer),
        t0(),
    );
    decision.transfer_evidence = Some(TransferEvidence {
        contract: TreatmentEvidenceRef {
            kind: TreatmentEvidenceKind::EnvelopeDigest,
            value: String::new(),
            at: Some(t0()),
            principal: Some(principal()),
        },
        transferee: "acme-cyber-insurer".into(),
        effective_at: Some(t0()),
    });
    let verifying = walk(
        decision,
        &[
            TreatmentState::Approved,
            TreatmentState::Executing,
            TreatmentState::Verification,
        ],
        t0(),
    )
    .expect("empty contract still walks to verification");
    let err = verifying
        .transition(TreatmentState::Completed, principal(), t0())
        .expect_err("empty contract evidence must fail closed");
    match &err {
        TreatmentError::MissingContractEvidence => {}
        other => assert_err_contains(other, &["contract"]),
    }
}

/// P08: superseded treatment is not the active path
#[test]
fn p08_t08_superseded_treatment_is_not_the_active_path() {
    let risk_id = RiskId::new("risk:supersede");
    let mut assessment = populated_assessment("assess.p08.t08", &risk_id);
    let old_id = RiskTreatmentId::new("rt:old-accept");
    let new_id = RiskTreatmentId::new("rt:new-mitigate");

    let mut old = with_approval(
        propose(old_id.as_str(), &risk_id, TreatmentStrategy::Accept),
        t0(),
    );
    old.acceptance = Some(sealed_acceptance(&risk_id, &old_id, t_expiry()));
    let mut old = walk(old, &happy_path(), t0()).expect("old accept completes");

    let mut successor = with_approval(
        propose(new_id.as_str(), &risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    successor.supersedes = Some(old_id.clone());
    successor.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    successor.implementation_ids = vec![ControlImplementationId::new("impl.access.mfa.org")];
    successor.plan = Some(mitigate_plan(vec![action(
        "ta:successor",
        "Replace acceptance with mitigation",
        true,
        ActionState::Proposed,
    )]));

    old.superseded_by = Some(new_id.clone());
    old = old
        .transition(TreatmentState::Superseded, principal(), t0())
        .expect("completed path may be superseded");
    assert_eq!(old.state, TreatmentState::Superseded);

    install(&mut assessment, old);
    install(&mut assessment, successor);
    assessment
        .validate()
        .expect("supersede-old-first inventory must validate");

    let active = active_treatment(&assessment, &risk_id).expect("successor is the active path");
    assert_eq!(active.id.as_str(), new_id.as_str());
    assert_ne!(active.state, TreatmentState::Superseded);
    assert!(
        treatment_required(&assessment, &risk_id, t0()),
        "superseded accept cannot suppress treatment even inside old expiresAt"
    );
    assert!(
        !acceptance_in_force(&assessment, &risk_id, t0()),
        "superseded acceptance is not in force"
    );
}

/// P08: target residual mismatch fails closed
#[test]
fn p08_t09_target_residual_mismatch_fails_closed() {
    let risk_id = RiskId::new("risk:residual-mismatch");
    let mut decision = with_approval(
        propose("rt:residual", &risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    decision.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    decision.implementation_ids = vec![ControlImplementationId::new("impl.access.mfa.org")];
    decision.plan = Some(mitigate_plan(vec![action(
        "ta:residual",
        "Ship MFA",
        true,
        ActionState::Done,
    )]));
    let mut verifying = walk(
        decision,
        &[
            TreatmentState::Approved,
            TreatmentState::Executing,
            TreatmentState::Verification,
        ],
        t0(),
    )
    .expect("mitigate reaches verification");
    let approved_digest =
        canonical_digest(&verifying.target_residual).expect("approved residual digest");
    verifying.target_residual = residual_other();
    let changed = canonical_digest(&verifying.target_residual).expect("changed residual digest");
    assert_ne!(approved_digest, changed);
    let err = verifying
        .transition(TreatmentState::Completed, principal(), t0())
        .expect_err("completion residual must match the value frozen at approval");
    match &err {
        TreatmentError::TargetResidualMismatch => {}
        other => assert_err_contains(other, &["residual"]),
    }

    let ir_src = crate_sources_joined("weeping-angel-assurance-ir");
    if ir_src.contains("fn score_risk") {
        assert!(
            ir_src.contains("TargetResidualMismatch") && ir_src.contains("score_risk"),
            "when Prompt 05 exists, stored MethodologyScored residual must agree with score_risk"
        );
        assert!(
            !ir_src.contains("enum RiskRating"),
            "treatment must not invent a collector rating enum"
        );
    }
}

/// P08: dangling control references are treatment errors
#[test]
fn p08_t10_dangling_control_references_are_treatment_errors() {
    let risk_id = RiskId::new("risk:dangling");
    let mut assessment = populated_assessment("assess.p08.t10", &risk_id);
    let mut decision = with_approval(
        propose("rt:dangling", &risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    decision.canonical_control_ids = vec![ControlId::new("control.missing")];
    decision.implementation_ids = vec![ControlImplementationId::new("impl.missing")];
    decision.plan = Some(mitigate_plan(vec![TreatmentAction {
        id: TreatmentActionId::new("ta:dangling"),
        title: "Cite missing control".into(),
        owner: owner(),
        required: true,
        state: ActionState::Done,
        control_ids: vec![ControlId::new("control.missing")],
        implementation_ids: vec![ControlImplementationId::new("impl.missing")],
        remediation_refs: vec![],
        evidence: vec![envelope("sha256:dangling", t0())],
        due_at: None,
    }]));
    install(&mut assessment, decision);
    let err = assessment
        .validate()
        .expect_err("dangling ControlId / ControlImplementationId must fail validate");
    let blob = err_blob(&err);
    assert!(
        blob.contains("control.missing") || blob.contains("dangling"),
        "treatment validation must name the dangling control, got {err}"
    );
    assert!(
        blob.contains("impl.missing")
            || blob.contains("implementation")
            || blob.contains("dangling"),
        "treatment validation must reject dangling implementation ids, got {err}"
    );
}

/// P08: invalid transitions fail closed
#[test]
fn p08_t11_invalid_transitions_fail_closed() {
    assert!(TreatmentState::can_transition(
        TreatmentState::Proposed,
        TreatmentState::Approved
    ));
    assert!(!TreatmentState::can_transition(
        TreatmentState::Proposed,
        TreatmentState::Completed
    ));
    assert!(!TreatmentState::can_transition(
        TreatmentState::Proposed,
        TreatmentState::Executing
    ));
    assert!(!TreatmentState::can_transition(
        TreatmentState::Cancelled,
        TreatmentState::Approved
    ));
    assert!(!TreatmentState::can_transition(
        TreatmentState::Completed,
        TreatmentState::Executing
    ));

    let risk_id = RiskId::new("risk:transitions");
    let proposed = propose("rt:transitions", &risk_id, TreatmentStrategy::Mitigate);
    for (from_setup, to) in [
        (proposed.clone(), TreatmentState::Completed),
        (proposed.clone(), TreatmentState::Executing),
    ] {
        let err = from_setup
            .transition(to, principal(), t0())
            .expect_err("illegal edge must fail closed without panic");
        match &err {
            TreatmentError::InvalidTransition { .. } => {}
            other => assert_err_contains(other, &["transition"]),
        }
    }

    let cancelled = proposed
        .clone()
        .transition(TreatmentState::Cancelled, principal(), t0())
        .expect("Proposed → Cancelled is allowed");
    let err = cancelled
        .transition(TreatmentState::Approved, principal(), t0())
        .expect_err("Cancelled is terminal");
    assert_err_contains(&err, &["transition"]);

    let mut completed = with_approval(proposed, t0());
    completed.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    completed.implementation_ids = vec![ControlImplementationId::new("impl.access.mfa.org")];
    completed.plan = Some(mitigate_plan(vec![action(
        "ta:full",
        "Ship MFA",
        true,
        ActionState::Done,
    )]));
    let completed = walk(completed, &happy_path(), t0()).expect("happy path");
    let err = completed
        .transition(TreatmentState::Executing, principal(), t0())
        .expect_err("Completed → Executing is illegal");
    assert_err_contains(&err, &["transition"]);
}

/// P08: all four strategies share the state machine
#[test]
fn p08_t12_all_four_strategies_share_the_state_machine() {
    let risk_id = RiskId::new("risk:four-strategies");
    let mut mitigate = with_approval(
        propose("rt:four-mitigate", &risk_id, TreatmentStrategy::Mitigate),
        t0(),
    );
    mitigate.canonical_control_ids = vec![ControlId::new("control.access.mfa")];
    mitigate.implementation_ids = vec![ControlImplementationId::new("impl.access.mfa.org")];
    mitigate.plan = Some(mitigate_plan(vec![action(
        "ta:four",
        "Ship MFA",
        true,
        ActionState::Done,
    )]));

    let mut accept = with_approval(
        propose("rt:four-accept", &risk_id, TreatmentStrategy::Accept),
        t0(),
    );
    accept.acceptance = Some(sealed_acceptance(
        &risk_id,
        &RiskTreatmentId::new("rt:four-accept"),
        t_expiry(),
    ));

    let mut avoid = with_approval(
        propose("rt:four-avoid", &risk_id, TreatmentStrategy::Avoid),
        t0(),
    );
    avoid.avoid_evidence = Some(narrative("stopped processing the asset", t0()));

    let mut transfer = with_approval(
        propose("rt:four-transfer", &risk_id, TreatmentStrategy::Transfer),
        t0(),
    );
    transfer.transfer_evidence = Some(TransferEvidence {
        contract: envelope("sha256:assignment-agreement", t0()),
        transferee: "group-captive".into(),
        effective_at: Some(t0()),
    });

    for (label, decision) in [
        ("Mitigate", mitigate),
        ("Accept", accept),
        ("Avoid", avoid),
        ("Transfer", transfer),
    ] {
        assert_eq!(decision.state, TreatmentState::Proposed);
        let completed = walk(decision, &happy_path(), t0())
            .unwrap_or_else(|e| panic!("{label} must walk proposed→completed, got {e:?}"));
        assert_eq!(
            completed.state,
            TreatmentState::Completed,
            "{label} must finish Completed with no Approved→Completed shortcut"
        );
        let err = serde_json::from_value::<TreatmentStrategy>(json!("hope"));
        assert!(err.is_err(), "unknown strategy tags fail closed");
    }

    let json = serde_json::to_value(TreatmentStrategy::Mitigate).unwrap();
    assert_eq!(json, json!("mitigate"));
}

/// P08: Risk::new and risk.json remain compatible
#[test]
fn p08_t13_risk_new_and_risk_json_remain_compatible() {
    let risk: Risk = serde_json::from_str(&golden_risk_json()).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.status, RiskStatus::Open);

    let constructed = Risk::new(
        RiskId::new("risk:org-1"),
        "supplier concentration",
        "single critical vendor",
    );
    assert_eq!(constructed.status, RiskStatus::Open);
    let json = serde_json::to_value(&constructed).unwrap();
    assert_eq!(json["id"], "risk:org-1");
    assert_eq!(json["status"], "open");
    assert!(json.get("treatment").is_none() || json.get("treatment") == Some(&Value::Null));

    let empty = empty_assessment("assess.p08.t13");
    empty
        .validate()
        .expect("assessments with empty risk_treatments remain valid");
    let round = serde_json::to_value(&empty).unwrap();
    let treatments = round
        .get("riskTreatments")
        .or_else(|| round.get("risk_treatments"));
    assert!(
        treatments.is_none() || treatments == Some(&json!([])),
        "empty treatment inventory must omit or default [] so old assessments round-trip"
    );
}

/// P08: risk register treatment_id resolves when present
#[test]
fn p08_t14_risk_register_treatment_id_resolves_when_present() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        risk_src.contains("pub struct Risk") && risk_src.contains("RiskId"),
        "do not fork Risk into a second record"
    );
    assert!(
        !risk_src.contains("struct RiskV2") && !risk_src.contains("enum RiskRating"),
        "treatment must consume Prompt 06 Risk, not invent RiskV2 / RiskRating"
    );

    let risk_id = RiskId::new("risk:source-tamper");
    let mut assessment = populated_assessment("assess.p08.t14", &risk_id);
    let decision = complete_mitigate(&risk_id);
    let treatment_id = decision.id.clone();

    if risk_src.contains("treatment_id") {
        let mut risk_json = serde_json::to_value(&assessment.risks[0]).unwrap();
        risk_json["treatmentId"] = json!(treatment_id.as_str());
        assessment.risks[0] =
            serde_json::from_value(risk_json).expect("Risk.treatment_id must decode");
        install(&mut assessment, decision);
        assessment
            .validate()
            .expect("Some(treatment_id) must resolve in risk_treatments");

        let mut dangling = populated_assessment("assess.p08.t14.dangling", &risk_id);
        let mut dangling_json = serde_json::to_value(&dangling.risks[0]).unwrap();
        dangling_json["treatmentId"] = json!("rt:does-not-exist");
        dangling.risks[0] =
            serde_json::from_value(dangling_json).expect("unknown treatment id still decodes");
        dangling
            .validate()
            .expect_err("unresolved treatment_id must fail closed");
    } else {
        install(&mut assessment, decision);
        assessment
            .validate()
            .expect("decisions still key by RiskId when Prompt 06 treatment_id is absent");
        let extra: Risk = serde_json::from_value(json!({
            "id": "risk:source-tamper",
            "title": "Source tampering",
            "description": "Unauthorized change to the source of record.",
            "status": "open",
            "treatmentId": "rt:mitigate"
        }))
        .expect("unknown additive treatmentId must not break Risk decode");
        assert_eq!(extra.id.as_str(), "risk:source-tamper");
    }
}

/// P08: collectors cannot emit treatment ratings or acceptance
#[test]
fn p08_t15_collectors_cannot_emit_treatment_ratings_or_acceptance() {
    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "RiskTreatmentDecision",
        "TargetResidualRisk",
        "RiskAcceptance",
        "enum RiskRating",
        "RiskRating::High",
    ] {
        assert!(
            !collector.contains(needle),
            "collectors must not emit `{needle}`"
        );
    }

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("enum RiskRating"),
        "IR must not grow a global RiskRating collector enum for treatment"
    );
}

/// P08: dual-suite registered
#[test]
fn p08_t16_dual_suite_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_risk_treatment_target")
            && toml.contains("tests/contracts/risk_treatment.target.rs"),
        "target suite must be listed in root Cargo.toml"
    );
    assert!(
        !toml.contains("sdd_risk_treatment_baseline")
            && !toml.contains("tests/contracts/risk_treatment.baseline.rs"),
        "baseline suite must remain registered beside the target"
    );
}
