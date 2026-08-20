//! Target suite for Operational ISMS v1 incident governance (Prompt 19).
//!
//! Encodes DESIRED behavior in `docs/specs/incident-governance.md` §4 / §6
//! (IG-001–IG-012). Must stay RED on characterization HEAD: no `Incident` IR,
//! no `AssessmentDefinition.incidents` inventory, no declare/promote path.
//! Do not `#[ignore]` these tests and do not implement the engine here.
//!
//! Compiles against current IR (`AssessmentDefinition`, `Finding`, `ValidateIr`)
//! and asserts additive incident records, explicit promotion, timeline order,
//! PIR / recovery rules, exercise vs real, graph integrity, and append-only
//! history. Baseline (`sdd_incident_governance_baseline`) is skip-superseded.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, Control,
    ControlId, ControlImplementation, ControlImplementationId, Identity, IdentityId, IdentityKind,
    PlannedControlTest, ProcessingActivity, ProcessingActivityId, Risk, RiskId, ValidateIr,
    canonical_digest,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::Effectiveness;

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

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(present.is_empty(), "{label}: forbidden surface {present:?}");
}

fn require_incident_engine(label: &str) {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        label,
        &ir,
        &[
            "pub struct Incident",
            "enum IncidentStatus",
            "enum IncidentKind",
            "struct PostIncidentReview",
            "struct IncidentTimelineEvent",
            "struct IncidentEvent",
            "struct ControlFailureRef",
            "struct ExternalIncidentRef",
            "enum DetectionSource",
            "typed_id!(IncidentId)",
            "fn declare",
            "incidents: Vec<Incident>",
        ],
    );
    assert!(
        crate_src("weeping-angel-assurance-ir")
            .join("incident.rs")
            .is_file(),
        "{label}: expected crates/weeping-angel-assurance-ir/src/incident.rs"
    );
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.incident-governance.target"))
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn graph_assessment() -> AssessmentDefinition {
    let mut assessment = empty_assessment();
    assessment.assets.push(Asset::new(
        AssetId::new("asset:prod-api"),
        AssetKind::Service,
        "prod-api",
    ));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:alice"),
        IdentityKind::User,
    ));
    assessment
        .processing_activities
        .push(ProcessingActivity::new(
            ProcessingActivityId::new("ropa:customer-data"),
            "customer data",
        ));
    assessment.controls.push(sample_control());
    assessment.tests.push(PlannedControlTest::new(
        weeping_angel_assurance_ir::ControlTestId::new("test.access.mfa"),
        ControlId::new("control.access.mfa"),
    ));
    assessment.risks.push(Risk::new(
        RiskId::new("risk:unprotected-branch"),
        "unprotected default branch",
        "source of record can be rewritten",
    ));
    assessment
}

fn declared_at() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn declared_incident_payload() -> Value {
    json!({
        "id": "inc.declared.finding-1",
        "kind": "real",
        "title": "Unprotected default branch declared as incident",
        "summary": "Promoted from scanner finding after review",
        "classification": "integrity",
        "severity": "high",
        "status": "declared",
        "detection": { "finding": "finding:unprotected-branch" },
        "externalRefs": [],
        "declaredAt": "2026-08-19T12:00:00Z",
        "declaredBy": { "identity": "identity:alice" },
        "responseOwner": { "role": "incident-commander" },
        "assetIds": ["asset:prod-api"],
        "processingActivityIds": ["ropa:customer-data"],
        "population": [{
            "kind": "identity",
            "ids": ["identity:alice"]
        }],
        "timeline": [
            {
                "at": "2026-08-19T11:00:00Z",
                "kind": "detected",
                "detail": "scanner alert"
            },
            {
                "at": "2026-08-19T12:00:00Z",
                "kind": "declared",
                "principal": { "identity": "identity:alice" }
            }
        ],
        "recoveryRefs": [],
        "communications": [],
        "evidenceRefs": [],
        "controlFailureRefs": [],
        "riskIds": ["risk:unprotected-branch"],
        "correctiveActionIds": [],
        "version": 1,
        "history": [{
            "version": 1,
            "at": "2026-08-19T12:00:00Z",
            "principal": { "identity": "identity:alice" },
            "kind": "declared"
        }],
        "tags": ["isms", "integrity"]
    })
}

fn decode_assessment_with_incidents(incidents: Vec<Value>) -> AssessmentDefinition {
    let mut encoded = serde_json::to_value(graph_assessment()).unwrap();
    let obj = encoded.as_object_mut().expect("assessment JSON object");
    obj.insert("incidents".into(), Value::Array(incidents));
    serde_json::from_value(encoded).expect("assessment with incidents must decode")
}

fn retained_incidents(assessment: &AssessmentDefinition) -> Vec<Value> {
    serde_json::to_value(assessment)
        .unwrap()
        .get("incidents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load")
}

/// IG-001: constructing a scanner Finding (or citing an alert) is not an incident
/// until `declare`; no `From<Finding> for Incident`; collectors do not insert.
#[test]
fn ig_001_alert_or_finding_is_not_an_incident_until_declare() {
    require_incident_engine("IG-001");

    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an ISMS incident")
        .build();
    let encoded = serde_json::to_value(&finding).unwrap();
    let back: Finding = serde_json::from_value(encoded).unwrap();
    assert_eq!(back.id, "unprotected-branch");

    let assessment = graph_assessment();
    assert!(
        retained_incidents(&assessment).is_empty(),
        "IG-001: AssessmentDefinition starts with zero incidents"
    );
    let _ = finding;
    assert!(
        retained_incidents(&assessment).is_empty(),
        "IG-001: constructing/deserializing Finding must not insert incidents"
    );
    assessment
        .validate()
        .expect("graph assessment without declare remains valid");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-001 no Finding document in IR",
        &ir,
        &["pub struct Finding ", "pub struct Alert ", "struct Alert {"],
    );
    forbid_needles(
        "IG-001 no From<Finding> for Incident",
        &ir,
        &[
            "impl From<Finding> for Incident",
            "From<Finding> for Incident",
            "impl From<weeping_angel::finding::Finding>",
        ],
    );
    let finding_src = read_repo_file("apps/cli/src/finding.rs");
    assert!(
        !finding_src.contains("Incident") && !finding_src.contains("declare"),
        "src/finding.rs must not promote into incident IR"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    forbid_needles(
        "IG-001 collector must not auto-insert incidents",
        &collector,
        &[
            "incidents.push",
            "assessment.incidents",
            "From<Finding> for Incident",
        ],
    );

    require_needles(
        "IG-001 declare is the only constructor",
        &ir,
        &["fn declare", "DetectionSource", "FindingRef", "AlertRef"],
    );
}

/// IG-002: declare yields stable IncidentId, Declared status, retained source,
/// seeded history, camelCase serde, stable digest under BTree ordering.
#[test]
fn ig_002_declare_yields_stable_id_declared_status_and_camelcase_round_trip() {
    require_incident_engine("IG-002");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-002 declare API",
        &ir,
        &[
            "IncidentId",
            "IncidentStatus::Declared",
            "declared_at",
            "declared_by",
            "fn declare",
        ],
    );

    let assessment = decode_assessment_with_incidents(vec![declared_incident_payload()]);
    let incidents = retained_incidents(&assessment);
    assert_eq!(
        incidents.len(),
        1,
        "IG-002: AssessmentDefinition.incidents must persist declared records"
    );
    let row = &incidents[0];
    assert_eq!(row["id"], "inc.declared.finding-1");
    assert_eq!(row["status"], "declared");
    assert_eq!(row["kind"], "real");
    assert_eq!(row["declaredAt"], "2026-08-19T12:00:00Z");
    assert_eq!(row["detection"]["finding"], "finding:unprotected-branch");
    assert_eq!(row["version"], 1);
    assert_eq!(row["history"][0]["kind"], "declared");
    assert_eq!(row["history"][0]["version"], 1);

    for key in [
        "declaredAt",
        "declaredBy",
        "responseOwner",
        "assetIds",
        "processingActivityIds",
        "controlFailureRefs",
        "riskIds",
        "correctiveActionIds",
        "externalRefs",
        "recoveryRefs",
        "postIncidentReview",
    ] {
        assert!(
            row.get(key).is_some()
                || matches!(key, "postIncidentReview" | "externalRefs" | "recoveryRefs"),
            "IG-002: camelCase key `{key}` must survive serde"
        );
    }

    assessment
        .validate()
        .expect("declared incident on a complete graph must validate");

    let mut a = declared_incident_payload();
    a["tags"] = json!(["integrity", "isms"]);
    let mut b = declared_incident_payload();
    b["tags"] = json!(["isms", "integrity"]);
    let left = decode_assessment_with_incidents(vec![a]);
    let right = decode_assessment_with_incidents(vec![b]);
    assert_eq!(
        canonical_digest(&left).expect("digest"),
        canonical_digest(&right).expect("digest"),
        "IG-002: canonical digest must be stable under BTree tag ordering"
    );
    assert_eq!(declared_at().to_rfc3339(), "2026-08-19T12:00:00+00:00");
}

/// IG-003: timeline is non-decreasing; declared_at matches Declared event;
/// out-of-order fails validate().
#[test]
fn ig_003_timeline_must_be_non_decreasing() {
    require_incident_engine("IG-003");

    let ordered = decode_assessment_with_incidents(vec![declared_incident_payload()]);
    let rows = retained_incidents(&ordered);
    assert_eq!(rows.len(), 1, "IG-003: ordered incident must be retained");
    assert_eq!(rows[0]["declaredAt"], rows[0]["timeline"][1]["at"]);
    assert_eq!(rows[0]["timeline"][1]["kind"], "declared");
    ordered
        .validate()
        .expect("detected ≤ declared timeline must validate");

    let mut out_of_order = declared_incident_payload();
    out_of_order["timeline"] = json!([
        {
            "at": "2026-08-19T14:00:00Z",
            "kind": "declared",
            "principal": { "identity": "identity:alice" }
        },
        {
            "at": "2026-08-19T11:00:00Z",
            "kind": "detected"
        }
    ]);
    let bad = decode_assessment_with_incidents(vec![out_of_order]);
    assert_eq!(
        retained_incidents(&bad).len(),
        1,
        "IG-003: out-of-order record must still be stored so validate can reject it"
    );
    let err = bad
        .validate()
        .expect_err("IG-003: out-of-order timeline must fail validate()");
    let msg = err.to_string();
    assert!(
        msg.contains("inc.declared.finding-1") || msg.to_ascii_lowercase().contains("timeline"),
        "IG-003: error must name the incident or timeline: {msg}"
    );
}

/// IG-004: ControlFailureRef resolves ControlId; dangling fails; linking does
/// not set Effectiveness; Prompt 15 event_ref is opaque.
#[test]
fn ig_004_control_failure_ref_resolves_without_setting_effectiveness() {
    require_incident_engine("IG-004");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-004 ControlFailureRef",
        &ir,
        &["struct ControlFailureRef", "event_ref", "control_id"],
    );
    forbid_needles(
        "IG-004 must not require Prompt 15 event types",
        &ir,
        &["enum AssuranceEvent", "struct ControlRegressed"],
    );

    let before = serde_json::to_value(sample_control()).unwrap();
    assert!(before.get("effectiveness").is_none());
    let _ = Effectiveness::Effective;
    let _ = Effectiveness::Ineffective;

    let mut linked = declared_incident_payload();
    linked["controlFailureRefs"] = json!([{
        "controlId": "control.access.mfa",
        "testId": "test.access.mfa",
        "eventRef": "event:control-regressed-1",
        "snapshotDigest": "sha256:opaque"
    }]);
    let ok = decode_assessment_with_incidents(vec![linked]);
    assert_eq!(retained_incidents(&ok).len(), 1);
    ok.validate()
        .expect("known ControlId + opaque event_ref must validate");
    let after = serde_json::to_value(&ok.controls[0]).unwrap();
    assert_eq!(
        after, before,
        "IG-004: linking a control failure must not rewrite Control or Effectiveness"
    );
    assert!(after.get("effectiveness").is_none());

    let mut dangling = declared_incident_payload();
    dangling["controlFailureRefs"] = json!([{
        "controlId": "control.missing.regression",
        "eventRef": "event:control-regressed-1"
    }]);
    let bad = decode_assessment_with_incidents(vec![dangling]);
    bad.validate()
        .expect_err("IG-004: dangling ControlId must fail closed");
}

/// IG-005: Real Recovered/Closed requires recovery evidence; Exercise may close
/// without it.
#[test]
fn ig_005_real_recovered_or_closed_requires_recovery_evidence() {
    require_incident_engine("IG-005");

    let mut real_closed = declared_incident_payload();
    real_closed["status"] = json!("closed");
    real_closed["recoveryRefs"] = json!([]);
    real_closed["postIncidentReview"] = json!({
        "recordedAt": "2026-08-19T16:00:00Z",
        "recordedBy": { "identity": "identity:alice" },
        "lessonsLearned": "rotate credentials",
        "proposedRiskIds": [],
        "proposedControlIds": [],
        "proposedCorrectiveActionIds": [],
        "evidenceRefs": []
    });
    real_closed["timeline"] = json!([
        { "at": "2026-08-19T11:00:00Z", "kind": "detected" },
        { "at": "2026-08-19T12:00:00Z", "kind": "declared", "principal": { "identity": "identity:alice" } },
        { "at": "2026-08-19T13:00:00Z", "kind": "statusTransition" },
        { "at": "2026-08-19T16:00:00Z", "kind": "reviewRecorded" }
    ]);
    let missing = decode_assessment_with_incidents(vec![real_closed]);
    assert_eq!(retained_incidents(&missing).len(), 1);
    missing
        .validate()
        .expect_err("IG-005: Real Closed without recovery evidence must fail");

    let mut real_recovered = declared_incident_payload();
    real_recovered["status"] = json!("recovered");
    real_recovered["recoveryRefs"] = json!(["evidence.digest:restore-1"]);
    real_recovered["timeline"] = json!([
        { "at": "2026-08-19T11:00:00Z", "kind": "detected" },
        { "at": "2026-08-19T12:00:00Z", "kind": "declared", "principal": { "identity": "identity:alice" } },
        { "at": "2026-08-19T15:00:00Z", "kind": "recovered" }
    ]);
    decode_assessment_with_incidents(vec![real_recovered])
        .validate()
        .expect("IG-005: Real Recovered with recoveryRefs must pass");

    let mut exercise_closed = declared_incident_payload();
    exercise_closed["id"] = json!("inc.exercise.tabletop-1");
    exercise_closed["kind"] = json!("exercise");
    exercise_closed["status"] = json!("closed");
    exercise_closed["recoveryRefs"] = json!([]);
    exercise_closed["postIncidentReview"] = Value::Null;
    decode_assessment_with_incidents(vec![exercise_closed])
        .validate()
        .expect("IG-005: Exercise may close without recovery evidence");
}

/// IG-006: Real Closed without PostIncidentReview fails; PIR proposals do not
/// auto-insert risks/controls/remediations.
#[test]
fn ig_006_real_closed_requires_pir_and_proposals_do_not_mutate_graph() {
    require_incident_engine("IG-006");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-006 PIR",
        &ir,
        &[
            "struct PostIncidentReview",
            "proposed_risk_ids",
            "proposed_control_ids",
            "proposed_corrective_action_ids",
        ],
    );

    let mut missing_pir = declared_incident_payload();
    missing_pir["status"] = json!("closed");
    missing_pir["recoveryRefs"] = json!(["evidence.digest:restore-1"]);
    missing_pir["postIncidentReview"] = Value::Null;
    decode_assessment_with_incidents(vec![missing_pir])
        .validate()
        .expect_err("IG-006: Real Closed without PostIncidentReview must fail");

    let mut with_pir = declared_incident_payload();
    with_pir["status"] = json!("closed");
    with_pir["recoveryRefs"] = json!(["evidence.digest:restore-1"]);
    with_pir["postIncidentReview"] = json!({
        "recordedAt": "2026-08-19T16:00:00Z",
        "recordedBy": { "identity": "identity:alice" },
        "rootCause": "missing branch protection",
        "lessonsLearned": "require reviews on default branch",
        "proposedRiskIds": ["risk:proposed-from-pir"],
        "proposedControlIds": ["control.proposed.from-pir"],
        "proposedCorrectiveActionIds": ["capa:proposed-1"],
        "evidenceRefs": ["evidence.digest:pir-notes"]
    });
    let assessed = decode_assessment_with_incidents(vec![with_pir]);
    assert_eq!(retained_incidents(&assessed).len(), 1);
    assessed
        .validate()
        .expect("IG-006: Real Closed with PIR must pass");
    assert!(
        assessed
            .risks
            .iter()
            .all(|r| r.id.as_str() != "risk:proposed-from-pir"),
        "IG-006: PIR proposed_risk_ids must not auto-insert into assessment.risks"
    );
    assert!(
        assessed
            .controls
            .iter()
            .all(|c| c.id().as_str() != "control.proposed.from-pir"),
        "IG-006: PIR proposed_control_ids must not auto-insert into assessment.controls"
    );
    let encoded = serde_json::to_value(&assessed).unwrap();
    assert!(
        encoded.get("remediations").is_none()
            || encoded["remediations"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true),
        "IG-006: PIR proposals must not auto-insert remediations"
    );
}

/// IG-007: Exercise vs Real is a field on the same record; catalog
/// control.incident.exercise stays governance-only.
#[test]
fn ig_007_exercise_vs_real_is_kind_on_same_record() {
    require_incident_engine("IG-007");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-007 kind on one type",
        &ir,
        &["enum IncidentKind", "Real", "Exercise"],
    );
    assert!(
        !ir.contains("struct ExerciseIncident") && !ir.contains("struct RealIncident"),
        "IG-007: kind is a field, not a second record type"
    );

    let catalog = load_catalog();
    let exercise = catalog
        .control("control.incident.exercise")
        .expect("governance catalog keeps control.incident.exercise");
    assert!(
        exercise
            .evidence
            .iter()
            .any(|e| e == "evidence.incident.exercise")
    );
    assert!(
        exercise
            .tests
            .iter()
            .any(|t| t == "test.incident.exercise-current")
    );
    let evidence = catalog
        .evidence()
        .get("evidence.incident.exercise")
        .expect("evidence.incident.exercise remains capability evidence");
    assert_eq!(evidence.evidence_type, "incident.exercise");
    catalog
        .control("control.incident.response-plan")
        .expect("control.incident.response-plan stays governance-only");
    catalog
        .control("control.incident.postmortem")
        .expect("control.incident.postmortem stays governance-only");

    let gov = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(
        !gov.contains("control.incident.operational-register"),
        "IG-007: do not invent a catalog operational-register control"
    );

    let mut exercise_row = declared_incident_payload();
    exercise_row["id"] = json!("inc.exercise.tabletop-1");
    exercise_row["kind"] = json!("exercise");
    exercise_row["status"] = json!("closed");
    exercise_row["recoveryRefs"] = json!([]);
    exercise_row["postIncidentReview"] = Value::Null;
    let exercise_assessment = decode_assessment_with_incidents(vec![exercise_row]);
    let rows = retained_incidents(&exercise_assessment);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["kind"], "exercise");
    exercise_assessment
        .validate()
        .expect("IG-007: Exercise Closed without PIR is valid");

    let mut real_row = declared_incident_payload();
    real_row["kind"] = json!("real");
    real_row["status"] = json!("closed");
    real_row["recoveryRefs"] = json!(["evidence.digest:restore-1"]);
    real_row["postIncidentReview"] = Value::Null;
    decode_assessment_with_incidents(vec![real_row])
        .validate()
        .expect_err("IG-007: Real Closed without PIR is not valid");
}

/// IG-008: Closed incident with an open corrective-action ref is valid;
/// incident close does not close CAPA/remediation.
#[test]
fn ig_008_closed_incident_with_open_corrective_action_is_valid() {
    require_incident_engine("IG-008");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-008 corrective-action seam",
        &ir,
        &["corrective_action_ids", "correctiveActionIds"],
    );
    let assurance = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "IG-008 query helper",
        &assurance,
        &["closed_incidents_with_open_corrective_actions"],
    );
    forbid_needles(
        "IG-008 must not auto-close remediations",
        &(ir + &assurance),
        &[
            "remediation.status = Closed",
            "auto_close_corrective",
            "close_remediation_from_incident",
        ],
    );

    let mut closed = declared_incident_payload();
    closed["status"] = json!("closed");
    closed["recoveryRefs"] = json!(["evidence.digest:restore-1"]);
    closed["correctiveActionIds"] = json!(["capa:branch-protection"]);
    closed["postIncidentReview"] = json!({
        "recordedAt": "2026-08-19T16:00:00Z",
        "recordedBy": { "identity": "identity:alice" },
        "lessonsLearned": "open CAPA remains open",
        "proposedRiskIds": [],
        "proposedControlIds": [],
        "proposedCorrectiveActionIds": [],
        "evidenceRefs": []
    });
    let assessed = decode_assessment_with_incidents(vec![closed]);
    assert_eq!(retained_incidents(&assessed).len(), 1);
    assessed
        .validate()
        .expect("IG-008: Closed + open corrective-action id must validate (Prompt 16 is a seam)");
    assert_eq!(
        retained_incidents(&assessed)[0]["correctiveActionIds"][0],
        "capa:branch-protection"
    );
}

/// IG-009: revise/transition append history and increment version; past
/// timeline bytes are not rewritten; illegal transitions fail.
#[test]
fn ig_009_revise_and_transition_append_history_illegal_transitions_fail() {
    require_incident_engine("IG-009");

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IG-009 history machine",
        &ir,
        &[
            "fn revise",
            "fn transition",
            "fn can_transition",
            "StatusTransition",
            "FieldsRevised",
        ],
    );

    let mut revised = declared_incident_payload();
    revised["title"] = json!("revised title");
    revised["version"] = json!(2);
    revised["history"] = json!([
        {
            "version": 1,
            "at": "2026-08-19T12:00:00Z",
            "principal": { "identity": "identity:alice" },
            "kind": "declared"
        },
        {
            "version": 2,
            "at": "2026-08-19T12:30:00Z",
            "principal": { "identity": "identity:alice" },
            "kind": "fieldsRevised"
        }
    ]);
    let assessed = decode_assessment_with_incidents(vec![revised]);
    let row = &retained_incidents(&assessed)[0];
    assert_eq!(row["version"], 2);
    assert_eq!(row["history"].as_array().map(|h| h.len()), Some(2));
    assert_eq!(row["history"][0]["kind"], "declared");
    assert_eq!(row["timeline"][1]["at"], "2026-08-19T12:00:00Z");
    assessed
        .validate()
        .expect("append-only revise must validate");

    let mut mutated_past = declared_incident_payload();
    mutated_past["timeline"][1]["at"] = json!("2026-08-18T00:00:00Z");
    mutated_past["declaredAt"] = json!("2026-08-19T12:00:00Z");
    decode_assessment_with_incidents(vec![mutated_past])
        .validate()
        .expect_err("IG-009: rewriting a past timeline timestamp must fail validate()");

    let mut illegal = declared_incident_payload();
    illegal["status"] = json!("closed");
    illegal["history"] = json!([
        {
            "version": 1,
            "at": "2026-08-19T12:00:00Z",
            "kind": "declared"
        },
        {
            "version": 2,
            "at": "2026-08-19T12:05:00Z",
            "kind": { "statusTransition": { "from": "declared", "to": "closed" } }
        }
    ]);
    decode_assessment_with_incidents(vec![illegal])
        .validate()
        .expect_err("IG-009: Declared → Closed is an illegal transition");

    assert!(
        ir.contains("Cancelled") && ir.contains("can_transition"),
        "IG-009: Cancelled → Declared must be rejected by the transition table"
    );
}

/// IG-010: ExternalIncidentRef round-trips; canonical id remains IncidentId;
/// no PagerDuty/Jira/ServiceNow adapters.
#[test]
fn ig_010_external_incident_ref_round_trips_without_adapters() {
    require_incident_engine("IG-010");

    let product = product_crate_sources_joined();
    require_needles(
        "IG-010 ExternalIncidentRef",
        &product,
        &["struct ExternalIncidentRef", "external_id"],
    );
    forbid_needles(
        "IG-010 no vendor adapters",
        &product,
        &[
            "PagerDutyIncident",
            "ServiceNowIncident",
            "struct JiraIncident",
            "pagerduty.com/api",
            "api.pagerduty.com",
        ],
    );

    let mut row = declared_incident_payload();
    row["externalRefs"] = json!([{
        "system": "pagerduty",
        "externalId": "PD-1234",
        "url": "https://example.pagerduty.com/incidents/PD-1234"
    }]);
    let assessed = decode_assessment_with_incidents(vec![row]);
    let out = &retained_incidents(&assessed)[0];
    assert_eq!(out["id"], "inc.declared.finding-1");
    assert_eq!(out["externalRefs"][0]["system"], "pagerduty");
    assert_eq!(out["externalRefs"][0]["externalId"], "PD-1234");
    assessed
        .validate()
        .expect("external refs are pointers; IncidentId stays canonical");
}

/// IG-011: duplicate IncidentId and dangling asset/risk/identity refs fail;
/// assessments without incidents still decode; IR-019 stays green.
#[test]
fn ig_011_graph_integrity_legacy_decode_and_ir019() {
    require_incident_engine("IG-011");

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(retained_incidents(&golden).is_empty());
    golden
        .validate()
        .expect("assessments without incidents still decode and validate");
    assert_eq!(golden.schema_version, ASSURANCE_IR_SCHEMA);

    let mut dangling_asset = declared_incident_payload();
    dangling_asset["assetIds"] = json!(["asset:missing"]);
    decode_assessment_with_incidents(vec![dangling_asset])
        .validate()
        .expect_err("IG-011: dangling AssetId must fail");

    let mut dangling_risk = declared_incident_payload();
    dangling_risk["riskIds"] = json!(["risk:missing"]);
    decode_assessment_with_incidents(vec![dangling_risk])
        .validate()
        .expect_err("IG-011: dangling RiskId must fail");

    let mut dangling_owner = declared_incident_payload();
    dangling_owner["declaredBy"] = json!({ "identity": "identity:missing" });
    decode_assessment_with_incidents(vec![dangling_owner])
        .validate()
        .expect_err("IG-011: dangling identity owner must fail");

    let dup = declared_incident_payload();
    decode_assessment_with_incidents(vec![dup.clone(), dup])
        .validate()
        .expect_err("IG-011: duplicate IncidentId must fail");

    let mut ir019 = empty_assessment();
    ir019.controls.push(sample_control());
    ir019.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = ir019.validate().expect_err("IR-019: dangling risk");
    assert!(
        err.to_string().contains("dangling risk reference"),
        "IR-019 must stay green: {err}"
    );
}

/// IG-012: dual-suite registered under tests/contracts/; spec listed in
/// CANONICAL_SPECS after implement.
#[test]
fn ig_012_dual_suite_registered_and_canonical_specs() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_incident_governance_baseline")
            && harness_src().contains("incident_governance.target.rs")
            && !toml.contains("tests/contracts/incident_governance.baseline.rs")
            && harness_src().contains("incident_governance.target.rs"),
        "IG-012: target suite listed; superseded baseline deleted"
    );
    assert!(
        !manifest_dir()
            .join("tests/contracts/incident_governance.baseline.rs")
            .exists()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/incident_governance.target.rs")
            .is_file()
    );

    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/incident-governance.md"),
        "IG-012: add docs/specs/incident-governance.md to CANONICAL_SPECS at implement"
    );

    require_incident_engine("IG-012");
}
