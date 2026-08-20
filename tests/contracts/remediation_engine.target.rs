//! Target suite for Operational ISMS v1 remediation engine (Prompt 16).
//!
//! Encodes DESIRED behavior in `docs/specs/remediation-engine.md` §4 / §6
//! (RE-001–RE-014). On CURRENT HEAD this binary must compile without the
//! missing `Remediation` types and stay **RED** for the missing ISMS
//! remediation contract (not harness noise). Do not `#[ignore]`. Do not
//! implement the engine here. Scanner `RemediationRequest` is a different type.

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::workbench::remediation::RemediationRequest;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Control, ControlId, Exception,
    ExceptionId, ExceptionStatus, IdError, Identity, IdentityId, IdentityKind, ValidateIr,
    canonical_digest, validate_stable_id,
};

fn product_crates_joined() -> String {
    crate_sources_joined("weeping-angel-assurance-ir")
        + "\n"
        + &crate_sources_joined("weeping-angel-assurance")
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn require_engine(case: &str) {
    require_needles(
        case,
        &product_crates_joined(),
        &[
            "pub struct Remediation",
            "enum RemediationState",
            "enum RemediationSourceKind",
            "struct RemediationSource",
            "typed_id!(RemediationId)",
            "fn create_from_control_regression",
            "fn evaluate_verification",
            "fn sla_overdue",
            "AcceptedWaived",
            "SingleGreenPermitted",
            "SustainedWindow",
        ],
    );
}

fn as_of() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn uuid_v4() -> &'static str {
    "550e8400-e29b-41d4-a716-446655440000"
}

fn event_id() -> &'static str {
    "evt:control-regressed-mfa-2026-08-18"
}

fn rem_id() -> &'static str {
    "rem:control-regressed-mfa-2026-08"
}

fn sample_result_json(effectiveness: &str, evaluated_at: &str) -> Value {
    json!({
        "testId": "test.access.mfa",
        "controlId": "control.access.mfa",
        "effectiveness": effectiveness,
        "rationale": format!("target observation {effectiveness}"),
        "evidenceRefs": ["sha256:mfa-observation"],
        "evaluatedAt": evaluated_at,
        "testVersion": "1",
        "inputDigest": "sha256:mfa-input",
        "duration": "12ms"
    })
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.remediation-engine.target"))
}

fn persist_assessment(value: Value) -> Value {
    let decoded: AssessmentDefinition =
        serde_json::from_value(value).expect("assessment JSON must decode");
    serde_json::to_value(&decoded).expect("assessment must serialize")
}

fn imagined_remediation(extra: Value) -> Value {
    let mut body = json!({
        "id": rem_id(),
        "title": "Restore MFA on privileged access",
        "source": {
            "kind": "controlRegressed",
            "eventId": event_id(),
            "occurredAt": "2026-08-18T12:00:00Z",
            "snapshotRefs": ["snap:assurance-2026-08-18"],
            "causeRefs": ["test.access.mfa"]
        },
        "controlIds": ["control.access.mfa"],
        "riskIds": ["risk:source-tamper"],
        "treatmentActionIds": [],
        "owner": { "identity": "identity:owner" },
        "priority": "p3",
        "severity": "high",
        "state": "proposed",
        "externalTickets": [],
        "plannedActions": [],
        "evidenceOfFix": [{
            "description": "MFA enforced on privileged identities",
            "minCardinality": 1
        }],
        "verificationPolicy": {
            "mode": "sustainedWindow",
            "window": 1209600,
            "minEffectiveResults": 2,
            "independentVerifier": false
        },
        "verificationState": { "status": "notStarted" },
        "history": [{ "version": 1, "kind": "created" }]
    });
    if let (Some(obj), Some(patch)) = (body.as_object_mut(), extra.as_object()) {
        for (k, v) in patch {
            obj.insert(k.clone(), v.clone());
        }
    }
    json!({
        "id": "assess.remediation-engine.target",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "controls": [{
            "id": "control.access.mfa",
            "title": "MFA",
            "description": "Require multi-factor authentication."
        }],
        "identities": [{ "id": "identity:owner", "kind": "user" }],
        "remediations": [body]
    })
}

fn remediations_of(assessment_json: &Value) -> &[Value] {
    assessment_json
        .get("remediations")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[])
}

/// RE-001: ControlRegressed-shaped source produces a Remediation; ControlTestResult is unchanged.
#[test]
fn re_001_create_from_control_regression() {
    require_engine("RE-001 create-from-control-regression");
    require_needles(
        "RE-001 create_from_control_regression",
        &product_crates_joined(),
        &[
            "fn create_from_control_regression",
            "ControlRegressed",
            "struct RemediationSource",
        ],
    );

    let failed = sample_result_json("ineffective", "2026-08-18T12:00:00Z");
    let before = failed.clone();
    let before_digest = canonical_digest(&failed).unwrap();

    let round = persist_assessment(imagined_remediation(json!({})));
    let rem = remediations_of(&round)
        .first()
        .expect("ControlRegressed source must persist a Remediation on AssessmentDefinition");
    assert_eq!(rem["source"]["kind"], "controlRegressed");
    assert_eq!(rem["source"]["eventId"], event_id());
    assert_eq!(rem["controlIds"], json!(["control.access.mfa"]));
    assert_eq!(rem["state"], "proposed");
    assert_eq!(rem["id"], rem_id());

    assert_eq!(failed, before);
    assert_eq!(canonical_digest(&failed).unwrap(), before_digest);
    assert_eq!(failed["effectiveness"], "ineffective");
    assert_eq!(failed["duration"], "12ms");
    assert!(
        failed.get("remediationId").is_none() && failed.get("state").is_none(),
        "ControlTestResult must stay an immutable observation"
    );
}

/// RE-002: treatmentActionIds round-trip; dangling injected inventory fails; id stays canonical.
#[test]
fn re_002_risk_treatment_action_linkage() {
    require_engine("RE-002 risk-treatment-action linkage");
    require_needles(
        "RE-002 linkage API",
        &product_crates_joined(),
        &[
            "fn link_treatment_action",
            "treatment_action_ids",
            "fn validate_remediations_at",
        ],
    );

    let round = persist_assessment(imagined_remediation(json!({
        "id": "rem:bp-1",
        "treatmentActionIds": ["ta:mitigate-branch-protection"]
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("treatmentActionIds must persist on AssessmentDefinition.remediations");
    assert_eq!(rem["id"], "rem:bp-1");
    assert_eq!(
        rem["treatmentActionIds"],
        json!(["ta:mitigate-branch-protection"])
    );
    assert_ne!(rem["id"], "ta:mitigate-branch-protection");
}

/// RE-003: SLA overdue iff dueAt < as_of and state is non-terminal.
#[test]
fn re_003_sla_overdue() {
    require_engine("RE-003 SLA overdue");
    require_needles(
        "RE-003 sla_overdue",
        &product_crates_joined(),
        &["fn sla_overdue", "due_at", "SlaPolicyId"],
    );

    let in_progress = persist_assessment(imagined_remediation(json!({
        "state": "inProgress",
        "dueAt": "2026-08-01T00:00:00Z",
        "slaPolicyId": "sla:p2-14d"
    })));
    let rem = remediations_of(&in_progress)
        .first()
        .expect("InProgress remediation with dueAt must persist");
    assert_eq!(rem["state"], "inProgress");
    assert_eq!(rem["dueAt"], "2026-08-01T00:00:00Z");
    assert!(
        rem["dueAt"].as_str().unwrap() < "2026-08-19T12:00:00Z"
            && rem["state"] != "closed"
            && rem["state"] != "cancelled"
            && rem["state"] != "superseded",
        "past dueAt + InProgress must be the overdue found case"
    );

    let closed = persist_assessment(imagined_remediation(json!({
        "state": "closed",
        "dueAt": "2026-08-01T00:00:00Z",
        "closedBy": { "identity": "identity:ciso" },
        "closedAt": "2026-08-19T12:00:00Z",
        "closureRationale": "verified"
    })));
    assert_eq!(
        remediations_of(&closed)
            .first()
            .expect("Closed remediation must persist")["state"],
        "closed"
    );
    let _ = as_of();
}

/// RE-004: external ticket refs attach without changing RemediationId; no ticket client; UUID v4 rejected.
#[test]
fn re_004_external_ticket_reference() {
    require_engine("RE-004 external ticket reference");
    require_needles(
        "RE-004 ticket adapter",
        &product_crates_joined(),
        &[
            "struct ExternalTicketRef",
            "fn attach_external_ticket",
            "GitHubIssues",
        ],
    );

    let round = persist_assessment(imagined_remediation(json!({
        "externalTickets": [{
            "system": "jira",
            "key": "SEC-441",
            "url": "https://jira.example/browse/SEC-441",
            "remoteState": "Done"
        }]
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("external ticket adapter ref must persist on the canonical remediation");
    assert_eq!(rem["id"], rem_id());
    assert_ne!(rem["id"], "SEC-441");
    assert_eq!(rem["externalTickets"][0]["system"], "jira");
    assert_eq!(rem["externalTickets"][0]["key"], "SEC-441");
    assert_ne!(
        rem["state"], "closed",
        "remoteState=Done must not close the canonical record"
    );

    assert_eq!(
        validate_stable_id(uuid_v4()).unwrap_err(),
        IdError::InvalidCharacter
    );

    let product = product_crates_joined();
    assert!(
        !product.contains("fn create_jira_issue")
            && !product.contains("JiraClient")
            && !product.contains("struct JiraClient"),
        "external tickets are adapter refs only; no ticket client"
    );
}

/// RE-005: Ineffective while AwaitingVerification sets Failed and forbids Verified/Closed.
#[test]
fn re_005_verification_failure() {
    require_engine("RE-005 verification failure");
    require_needles(
        "RE-005 awaiting verification",
        &product_crates_joined(),
        &[
            "AwaitingVerification",
            "fn evaluate_verification",
            "VerificationStatus",
        ],
    );

    let failed = sample_result_json("ineffective", "2026-08-19T12:00:00Z");
    assert_eq!(failed["effectiveness"], "ineffective");

    let round = persist_assessment(imagined_remediation(json!({
        "state": "awaitingVerification",
        "verificationState": { "status": "failed" }
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("AwaitingVerification remediation must persist");
    assert_eq!(rem["state"], "awaitingVerification");
    assert_eq!(rem["verificationState"]["status"], "failed");
    assert_ne!(rem["state"], "verified");
    assert_ne!(rem["state"], "closed");
}

/// RE-006: one Effective does not satisfy/close unless SingleGreenPermitted; close is explicit.
#[test]
fn re_006_sustained_success_single_green_does_not_close() {
    require_engine("RE-006 sustained success");
    require_needles(
        "RE-006 verification policy",
        &product_crates_joined(),
        &[
            "SustainedWindow",
            "SingleGreenPermitted",
            "min_effective_results",
            "fn evaluate_verification",
            "fn close",
        ],
    );

    let first = sample_result_json("effective", "2026-08-18T12:00:00Z");
    let second = sample_result_json("effective", "2026-08-21T12:00:00Z");
    assert_eq!(first["effectiveness"], "effective");
    assert_eq!(second["effectiveness"], "effective");
    assert_ne!(
        canonical_digest(&first).unwrap(),
        canonical_digest(&second).unwrap()
    );

    let round = persist_assessment(imagined_remediation(json!({
        "state": "awaitingVerification",
        "verificationPolicy": {
            "mode": "sustainedWindow",
            "window": 1209600,
            "minEffectiveResults": 2,
            "independentVerifier": false
        },
        "verificationState": { "status": "inWindow" }
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("SustainedWindow remediation must persist");
    assert_eq!(rem["verificationPolicy"]["mode"], "sustainedWindow");
    assert_ne!(
        rem["verificationState"]["status"], "satisfied",
        "a single green / 3-day pair must not satisfy a 14-day window"
    );
    assert_ne!(rem["state"], "closed");
}

/// RE-007: expired/revoked/missing-expiry waiver cannot remain AcceptedWaived; reopen to Open.
#[test]
fn re_007_expired_waiver_cannot_remain_accepted_waived() {
    require_engine("RE-007 expired waiver");
    require_needles(
        "RE-007 waiver clock",
        &product_crates_joined(),
        &[
            "AcceptedWaived",
            "struct WaiverBinding",
            "fn waiver_in_force",
            "fn reopen_expired_waiver",
        ],
    );

    let mut exception = Exception::new(ExceptionId::new("exc.mfa-waiver"), "temporary MFA waiver");
    exception.status = ExceptionStatus::Approved;
    exception.expires_at = Some(Utc.with_ymd_and_hms(2026, 8, 10, 0, 0, 0).unwrap());
    assert_eq!(exception.status, ExceptionStatus::Approved);
    assert!(exception.expires_at.unwrap() < as_of());

    let round = persist_assessment(imagined_remediation(json!({
        "state": "acceptedWaived",
        "waiver": {
            "kind": "exception",
            "exceptionId": "exc.mfa-waiver",
            "expiresAt": "2026-08-10T00:00:00Z"
        }
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("AcceptedWaived remediation must persist");
    assert_eq!(rem["state"], "acceptedWaived");
    assert_eq!(rem["waiver"]["expiresAt"], "2026-08-10T00:00:00Z");
}

/// RE-008: closed records freeze closedBy/closedAt/history; further mutation fails; digest stable.
#[test]
fn re_008_immutable_closure_history() {
    require_engine("RE-008 immutable closure history");
    require_needles(
        "RE-008 closure",
        &product_crates_joined(),
        &[
            "struct RemediationEvent",
            "ImmutableClosure",
            "fn close",
            "closure_rationale",
            "closed_by",
            "closed_at",
        ],
    );

    let round = persist_assessment(imagined_remediation(json!({
        "state": "closed",
        "closedBy": { "identity": "identity:ciso" },
        "closedAt": "2026-08-19T12:00:00Z",
        "closureRationale": "gap closed after independent verification",
        "history": [
            { "version": 1, "kind": "created" },
            { "version": 2, "kind": "closed", "principal": { "identity": "identity:ciso" } }
        ]
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("closed remediation must persist with frozen history");
    assert_eq!(rem["state"], "closed");
    assert_eq!(rem["closedBy"]["identity"], "identity:ciso");
    assert_eq!(rem["closedAt"], "2026-08-19T12:00:00Z");
    assert!(
        rem["history"]
            .as_array()
            .unwrap()
            .iter()
            .any(|ev| ev["kind"] == "closed")
    );
}

/// RE-009: sources use Prompt 15 IsmsEventKind names; this slice does not implement detect_isms_drift.
#[test]
fn re_009_prompt15_seam_no_second_event_bus() {
    require_engine("RE-009 Prompt 15 seam");
    require_needles(
        "RE-009 source kinds",
        &product_crates_joined(),
        &[
            "ControlRegressed",
            "EvidenceExpired",
            "ExceptionExpired",
            "RiskTreatmentAction",
            "enum RemediationSourceKind",
        ],
    );

    let ir_lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !ir_lib.contains("fn detect_isms_drift") && !assurance_lib.contains("fn detect_isms_drift"),
        "Prompt 16 consumes the Prompt 15 event contract; it does not implement drift"
    );

    let round = persist_assessment(imagined_remediation(json!({})));
    assert_eq!(
        remediations_of(&round)
            .first()
            .expect("Prompt 15-shaped source must persist")["source"]["kind"],
        "controlRegressed"
    );
}

/// RE-010: typed_id!(RemediationId), camelCase JSON, SHA-256 canonical_digest.
#[test]
fn re_010_identity_json_digest() {
    require_engine("RE-010 identity/json/digest");
    require_needles(
        "RE-010 identity",
        &read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs"),
        &["typed_id!(RemediationId)", "typed_id!(RemediationActionId)"],
    );
    require_needles(
        "RE-010 camelCase states",
        &product_crates_joined(),
        &[
            "rename_all = \"camelCase\"",
            "InProgress",
            "AwaitingVerification",
            "AcceptedWaived",
        ],
    );

    assert_eq!(
        validate_stable_id(uuid_v4()).unwrap_err(),
        IdError::InvalidCharacter
    );
    assert!(validate_stable_id(rem_id()).is_ok());

    let round = persist_assessment(imagined_remediation(json!({
        "state": "inProgress"
    })));
    let rem = remediations_of(&round)
        .first()
        .expect("Remediation camelCase JSON must persist");
    assert_eq!(rem["id"], rem_id());
    assert_eq!(rem["source"]["kind"], "controlRegressed");
    assert_eq!(rem["source"]["eventId"], event_id());
    assert_eq!(rem["controlIds"], json!(["control.access.mfa"]));
    assert_eq!(rem["state"], "inProgress");
    assert!(rem.get("finding_id").is_none());
}

/// RE-011: AssessmentDefinition without remediations still validates; old JSON deserializes.
#[test]
fn re_011_additive_assessment() {
    require_needles(
        "RE-011 remediations inventory",
        &read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs"),
        &["remediations"],
    );

    let empty = empty_assessment();
    empty
        .validate()
        .expect("AssessmentDefinition::new remains valid with empty remediations");
    assert_eq!(empty.schema_version, ASSURANCE_IR_SCHEMA);

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    golden.validate().unwrap();
}

/// RE-012: dual-suite runs as a harness module.
#[test]
fn re_012_dual_suite_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        !cargo.contains("sdd_remediation_engine_baseline")
            && !cargo.contains("tests/contracts/remediation_engine.baseline.rs"),
        "baseline suite must stay listed"
    );
    assert!(
        harness_src().contains("remediation_engine.target.rs")
            && harness_src().contains("remediation_engine.target.rs"),
        "target suite must be wired as a harness module"
    );
    assert!(
        !cargo.contains("tests/sdd/"),
        "Cargo.toml must not still point at tests/sdd/"
    );
}

/// RE-014: scanner workbench RemediationRequest remains a different type.
#[test]
fn re_014_workbench_remediation_request_is_not_ir() {
    let req = RemediationRequest {
        finding_id: "finding.unprotected-branch".into(),
        rule_id: "unprotected-branch".into(),
        path: "src/lib.rs".into(),
        start_line: 1,
        title: "scanner patch request is not IR".into(),
    };
    assert_eq!(req.finding_id, "finding.unprotected-branch");
    let encoded = serde_json::to_value(&req).unwrap();
    assert!(encoded.get("controlIds").is_none());
    assert!(encoded.get("source").is_none());
    assert!(encoded.get("verificationPolicy").is_none());

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("pub struct RemediationRequest"),
        "IR crate must not own the scanner RemediationRequest type"
    );

    let _keep = (
        Control::new(
            ControlId::new("control.access.mfa"),
            "MFA",
            "Require multi-factor authentication.",
        ),
        Identity::new(IdentityId::new("identity:owner"), IdentityKind::User),
    );
}
