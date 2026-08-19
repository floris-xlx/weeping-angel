//! Baseline suite for Operational ISMS v1 remediation engine (Prompt 16).
//!
//! Characterization of CURRENT tree (`docs/specs/remediation-engine.md` §3) on
//! SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: `weeping-angel-assurance-ir`
//! has no `Remediation` / `RemediationId`; `AssessmentDefinition` has no
//! remediations inventory; Prompt 15 `IsmsEvent` / `EventId` / `ControlRegressed`
//! / `detect_isms_drift` are specified but absent from product crates; `Risk` is
//! still `{id,title,description,status}`; `Exception` is a control exception;
//! `ControlTestResult` is an immutable observation with no work-item side
//! effect; `src/workbench/remediation.rs` `RemediationRequest` is a scanner
//! patch type; there is no Jira/Linear/GitHub Issues client.
//!
//! Must stay GREEN until `sdd_remediation_engine_target` is GREEN and this file
//! is skip-superseded. Does **not** implement the remediation engine.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::workbench::remediation::{RemediationRequest, RemediationResult};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Control, ControlId,
    ControlImplementation, ControlImplementationId, Exception, ExceptionId, ExceptionStatus,
    IdError, Risk, RiskId, RiskStatus, ValidateIr, canonical_digest, validate_stable_id,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSet, evaluate,
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

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let entries = fs::read_dir(&crates_dir).unwrap_or_else(|e| {
        panic!("read {}: {e}", crates_dir.display());
    });
    let mut chunks = Vec::new();
    for entry in entries {
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

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: remediation-engine IR must be absent on characterization HEAD; found {present:?}"
    );
}

fn remediation_engine_needles() -> &'static [&'static str] {
    &[
        "pub struct Remediation",
        "struct Remediation ",
        "enum RemediationState",
        "enum RemediationSourceKind",
        "struct RemediationSource",
        "struct VerificationPolicy",
        "struct VerificationState",
        "struct ExternalTicketRef",
        "struct WaiverBinding",
        "struct RemediationEvent",
        "struct RemediationAction",
        "struct EvidenceOfFixRequirement",
        "enum RemediationPriority",
        "fn create_from_control_regression",
        "fn create_from_source",
        "fn evaluate_verification",
        "fn sla_overdue",
        "fn waiver_in_force",
        "fn reopen_expired_waiver",
        "typed_id!(RemediationId)",
        "typed_id!(RemediationActionId)",
        "typed_id!(SlaPolicyId)",
        "pub struct RemediationId",
        "AcceptedWaived",
        "SingleGreenPermitted",
        "SustainedWindow",
    ]
}

fn prompt15_event_needles() -> &'static [&'static str] {
    &[
        "struct IsmsEvent",
        "enum IsmsEventKind",
        "pub struct EventId",
        "typed_id!(EventId)",
        "struct ControlRegressed",
        "ControlRegressed",
        "fn detect_isms_drift",
        "ISMS_EVENT_SCHEMA",
        "isms-event/v1",
    ]
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.remediation-engine.baseline"))
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn sample_result(effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: weeping_angel_assurance_ir::ControlTestId::new("test.access.mfa"),
        control_id: ControlId::new("control.access.mfa"),
        effectiveness,
        rationale: "found-case observation; no remediation side effect".into(),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        test_version: "1".into(),
        input_digest: "sha256:unused-by-remediation".into(),
        duration: None,
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn assessment_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        max_age: Duration::from_secs(86_400),
    }
}

/// RE-B01: weeping-angel-assurance-ir has no Remediation / RemediationId / RemediationState.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b01_no_isms_remediation_type() {
    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("remediation.rs").is_file(),
        "RE-B01: remediation.rs must not exist on characterization HEAD"
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles("RE-B01", &ir, remediation_engine_needles());

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("pub mod risk;"),
        "IR crate still has a risk module"
    );
    assert!(
        !lib.contains("mod remediation") && !lib.contains("Remediation"),
        "RE-B01: lib.rs must not declare or re-export Remediation"
    );

    let listed = [
        "applicability.rs",
        "assessment.rs",
        "asset.rs",
        "control.rs",
        "crosswalk.rs",
        "digest.rs",
        "evidence.rs",
        "exception.rs",
        "extension.rs",
        "framework.rs",
        "id.rs",
        "identity.rs",
        "implementation.rs",
        "lib.rs",
        "mapping.rs",
        "privacy.rs",
        "requirement.rs",
        "risk.rs",
        "subject.rs",
        "test.rs",
        "validation.rs",
        "vendor.rs",
    ];
    for name in listed {
        assert!(
            ir_src.join(name).is_file(),
            "expected IR module file {name}"
        );
    }
    for unexpected in ["remediation.rs", "event.rs", "risk_treatment.rs"] {
        assert!(
            !ir_src.join(unexpected).is_file(),
            "unexpected IR module {unexpected} on characterization HEAD"
        );
    }

    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !assurance_lib.contains("mod remediation")
            && !assurance_lib.contains("create_from_control_regression"),
        "weeping-angel-assurance must not export a remediation engine module"
    );
}

/// RE-B02: AssessmentDefinition serialized JSON from ::new has no remediations key.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b02_assessment_has_no_remediations_inventory() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("remediations").is_none());
    assert!(json.get("remediation").is_none());
    assert_eq!(json["schema_version"], ASSURANCE_IR_SCHEMA);

    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "assets".to_string(),
            "controls".to_string(),
            "evidence_requirements".to_string(),
            "exceptions".to_string(),
            "id".to_string(),
            "identities".to_string(),
            "implementations".to_string(),
            "mappings".to_string(),
            "processing_activities".to_string(),
            "requests".to_string(),
            "requirements".to_string(),
            "risks".to_string(),
            "schema_version".to_string(),
            "scope".to_string(),
            "tests".to_string(),
            "vendors".to_string(),
        ]
    );

    let imagined = json!({
        "id": "assess.remediation-engine.baseline",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "remediations": [{
            "id": "rem:control-regressed-mfa",
            "state": "open",
            "source": { "kind": "controlRegressed", "eventId": "evt:mfa-regressed" }
        }]
    });
    let decoded: AssessmentDefinition = serde_json::from_value(imagined).unwrap();
    let round = serde_json::to_value(&decoded).unwrap();
    assert!(
        round.get("remediations").is_none(),
        "unknown remediations inventory is dropped on current AssessmentDefinition"
    );

    let golden: Value = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden.get("remediations").is_none());
    assert_eq!(golden["requests"]["risk_treatment"], false);
    assert_eq!(golden["requests"]["nonconformities"], false);
}

/// RE-B03: id.rs source does not contain typed_id!(RemediationId).
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b03_id_rs_has_no_remediation_ids() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        src.contains("typed_id!(RiskId);"),
        "RiskId must remain a typed_id!"
    );
    assert!(
        src.contains("typed_id!(ExceptionId);"),
        "ExceptionId must remain a typed_id!"
    );
    for absent in [
        "typed_id!(RemediationId);",
        "typed_id!(RemediationActionId);",
        "typed_id!(SlaPolicyId);",
        "typed_id!(EventId);",
        "typed_id!(TreatmentActionId);",
        "typed_id!(RemediationRef);",
        "RemediationId",
        "RemediationActionId",
        "SlaPolicyId",
    ] {
        assert!(
            !src.contains(absent),
            "id.rs must not define `{absent}` on characterization HEAD"
        );
    }

    let err = validate_stable_id("550e8400-e29b-41d4-a716-446655440000")
        .expect_err("uuid-v4 shaped identities remain invalid");
    assert_eq!(err, IdError::InvalidCharacter);
    assert!(
        RiskId::try_new("550e8400-e29b-41d4-a716-446655440000").is_err(),
        "typed_id! still rejects random v4"
    );
}

/// RE-B04: crate sources contain no ControlRegressed type/module (Prompt 15 absence).
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b04_prompt15_control_regressed_absent_from_product() {
    let product = product_crate_sources_joined();
    forbid_needles("RE-B04", &product, prompt15_event_needles());

    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("event.rs").is_file(),
        "Prompt 15 event.rs must be absent; remediation must not invent a parallel bus"
    );

    let spec = read_repo_file("docs/specs/isms-events-drift.md");
    assert!(
        spec.contains("ControlRegressed") && spec.contains("detect_isms_drift"),
        "Prompt 15 human spec remains the event contract; product types are absent"
    );
}

/// RE-B05: Risk public fields remain id, title, description, status.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b05_risk_is_four_field_stub() {
    let risk = Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    );
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let json = serde_json::to_value(&risk).unwrap();
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "description".to_string(),
            "id".to_string(),
            "status".to_string(),
            "title".to_string()
        ]
    );
    for absent in [
        "treatmentActionIds",
        "remediationRefs",
        "remediationId",
        "dueAt",
        "owner",
        "slaPolicyId",
    ] {
        assert!(
            json.get(absent).is_none(),
            "found-case Risk JSON must not contain `{absent}`"
        );
    }

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        src.contains("//! Minimal risk record. Not a risk engine."),
        "risk.rs keeps the found-case module comment"
    );
}

/// RE-B06: Exception type exists; ExceptionStatus includes Approved and Expired.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b06_exception_exists_not_waiver_engine() {
    let mut exception = Exception::new(ExceptionId::new("exc.mfa-waiver"), "temporary waiver");
    exception.status = ExceptionStatus::Approved;
    assert_eq!(exception.status, ExceptionStatus::Approved);
    exception.status = ExceptionStatus::Expired;
    assert_eq!(exception.status, ExceptionStatus::Expired);

    let json = serde_json::to_value(&exception).unwrap();
    assert_eq!(json["status"], "expired");
    assert!(json.get("remediationId").is_none());

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/exception.rs");
    assert!(src.contains("enum ExceptionStatus"));
    for variant in ["Proposed", "Approved", "Expired", "Revoked"] {
        assert!(src.contains(variant), "missing ExceptionStatus::{variant}");
    }
    assert!(
        !src.contains("fn can_transition") && !src.contains("fn transition"),
        "Exception still has no transition function and is not a remediation waiver engine"
    );

    let mut assessment = empty_assessment();
    assessment.exceptions.push(exception);
    assessment
        .validate()
        .expect("expired Exception does not interact with a remediations inventory");
}

/// RE-B07: src/workbench/remediation.rs still defines RemediationRequest with finding_id.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b07_workbench_remediation_request_is_scanner_type() {
    let req = RemediationRequest {
        finding_id: "finding.unprotected-branch".into(),
        rule_id: "unprotected-branch".into(),
        path: "src/lib.rs".into(),
        start_line: 1,
        title: "scanner patch request is not IR".into(),
    };
    assert_eq!(req.finding_id, "finding.unprotected-branch");

    let encoded = serde_json::to_value(&req).unwrap();
    assert_eq!(encoded["finding_id"], "finding.unprotected-branch");
    assert!(
        encoded.get("source").is_none() && encoded.get("controlIds").is_none(),
        "scanner RemediationRequest must not grow ISMS remediation fields"
    );

    let src = read_repo_file("src/workbench/remediation.rs");
    assert!(src.contains("pub struct RemediationRequest"));
    assert!(src.contains("pub finding_id: String"));
    assert!(src.contains("pub struct RemediationResult"));
    assert!(
        src.contains("generate unified diffs") || src.contains("unified diff"),
        "workbench remediation remains a patch generator"
    );

    let _result_shape = RemediationResult {
        finding_id: req.finding_id.clone(),
        rule_id: req.rule_id.clone(),
        strategy: "none".into(),
        state: "failed".into(),
        summary: "documentary scanner state string, not RemediationState".into(),
        patch_path: None,
        patch_preview: None,
        files_touched: Vec::new(),
    };

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("RemediationRequest") && !ir.contains("finding_id"),
        "IR crate must not import the scanner RemediationRequest type"
    );
}

/// RE-B08: no Jira/Linear/GitHub Issues client under weeping-angel-* crates.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b08_no_ticket_system_clients() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "RE-B08",
        &product,
        &[
            "JiraClient",
            "LinearClient",
            "GitHubIssuesClient",
            "GitHubIssues",
            "create_jira_issue",
            "create_linear_issue",
            "fn create_jira",
            "struct ExternalTicketRef",
            "octocrab",
        ],
    );

    for crate_name in [
        "weeping-angel-assurance-ir",
        "weeping-angel-assurance",
        "weeping-angel-control-test",
        "weeping-angel-framework",
        "weeping-angel-evidence",
        "weeping-angel-collector",
        "weeping-angel-canonical-catalog",
    ] {
        let cargo = read_repo_file(&format!("crates/{crate_name}/Cargo.toml"));
        assert!(
            !cargo.contains("jira") && !cargo.contains("linear") && !cargo.contains("octocrab"),
            "{crate_name} must not depend on a ticket-system client"
        );
    }
}

/// RE-B09: ControlTestResult exists; evaluating Effective does not write remediations.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b09_effective_control_test_does_not_write_remediation() {
    let compiled = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.access.mfa",
        ))
        .control_id(ControlId::new("control.access.mfa"))
        .kind(ControlTestKind::Automated)
        .build();
    let observed = evaluate(&compiled, &EvidenceSet::new(), &assessment_context());
    assert_eq!(observed.control_id.as_str(), "control.access.mfa");
    assert_eq!(observed.effectiveness, Effectiveness::InsufficientEvidence);

    let green = sample_result(Effectiveness::Effective);
    assert_eq!(green.effectiveness, Effectiveness::Effective);
    let green_json = serde_json::to_value(&green).unwrap();
    for absent in ["remediationId", "workItemId", "closesRemediation", "state"] {
        assert!(
            green_json.get(absent).is_none(),
            "ControlTestResult must not carry a `{absent}` work-item side effect"
        );
    }

    let mut assessment = empty_assessment();
    assessment.controls.push(sample_control());
    let before = serde_json::to_value(&assessment).unwrap();
    let _ = (observed, green);
    let after = serde_json::to_value(&assessment).unwrap();
    assert_eq!(
        before, after,
        "constructing Effective/Ineffective results must not mutate AssessmentDefinition"
    );
    assert!(after.get("remediations").is_none());
}

/// RE-B10: validate_assessment_ir still enforces IR-019; empty assessments validate.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_b10_validate_ir019_and_empty_assessments() {
    empty_assessment()
        .validate()
        .expect("empty assessment still validates with no remediations walk");

    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        validation.contains("dangling risk reference"),
        "IR-019 message remains in validation.rs"
    );
    for needle in [
        "remediation",
        "sla_overdue",
        "AcceptedWaived",
        "verificationState",
        "closedBy",
    ] {
        assert!(
            !validation.contains(needle),
            "validate() must not walk remediations; found `{needle}`"
        );
    }

    let mut dangling = empty_assessment();
    dangling.controls.push(sample_control());
    dangling.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling
        .validate()
        .expect_err("IR-019: dangling implementation→RiskId must still fail");
    let msg = err.to_string();
    assert!(
        msg.contains("dangling risk reference"),
        "IR-019 message: {err}"
    );
    assert!(
        msg.contains("risk:missing"),
        "IR-019 error must name risk:missing, got {msg}"
    );
}

/// Found case for required scenario: creation from control regression has no engine.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_creation_from_control_regression_has_no_engine() {
    let failed = sample_result(Effectiveness::Ineffective);
    assert_eq!(failed.effectiveness, Effectiveness::Ineffective);

    let mut assessment = empty_assessment();
    assessment.controls.push(sample_control());
    assessment
        .validate()
        .expect("a failed control test does not require a remediation record today");
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("remediations").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "creation-from-regression",
        &product,
        &[
            "fn create_from_control_regression",
            "fn create_from_source",
            "ControlRegressed",
            "struct RemediationSource",
        ],
    );
}

/// Found case for required scenario: risk treatment action linkage is absent.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_risk_treatment_action_linkage_absent() {
    let mut assessment = empty_assessment();
    assessment.risks.push(Risk::new(
        RiskId::new("risk:open-untreated"),
        "untreated",
        "no treatment inventory exists",
    ));
    let extra = json!({
        "id": assessment.id.as_str(),
        "schema_version": assessment.schema_version,
        "risks": [{
            "id": "risk:open-untreated",
            "title": "untreated",
            "description": "no treatment inventory exists",
            "status": "open",
            "treatmentActionIds": ["ta:mitigate-branch-protection"]
        }],
        "remediations": [{
            "id": "rem:bp-1",
            "treatmentActionIds": ["ta:mitigate-branch-protection"]
        }]
    });
    let decoded: AssessmentDefinition = serde_json::from_value(extra).unwrap();
    decoded
        .validate()
        .expect("current validate() does not resolve imagined treatmentActionIds");
    let round = serde_json::to_value(&decoded).unwrap();
    assert!(round.get("remediations").is_none());
    assert!(round["risks"][0].get("treatmentActionIds").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "treatment-linkage",
        &product,
        &[
            "struct TreatmentAction",
            "typed_id!(TreatmentActionId)",
            "fn link_treatment_action",
        ],
    );
}

/// Found case for required scenario: SLA overdue is not modeled.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_sla_overdue_absent() {
    let risk = Risk::new(RiskId::new("risk:overdue-gap"), "t", "d");
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("dueAt").is_none());
    assert!(json.get("slaPolicyId").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "sla-overdue",
        &product,
        &[
            "fn sla_overdue",
            "SlaPolicyId",
            "validate_remediation_slas_at",
        ],
    );

    let mut assessment = empty_assessment();
    assessment.risks.push(risk);
    assessment
        .validate()
        .expect("clockless validate() has no SLA overdue walk");
}

/// Found case for required scenario: external tickets are not adapter refs on IR.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_external_ticket_reference_absent() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("externalTickets").is_none());

    let imagined = json!({
        "id": "assess.remediation-engine.baseline",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "remediations": [{
            "id": "SEC-441",
            "externalTickets": [{ "system": "jira", "key": "SEC-441" }]
        }]
    });
    let decoded: AssessmentDefinition = serde_json::from_value(imagined).unwrap();
    let round = serde_json::to_value(&decoded).unwrap();
    assert!(round.get("remediations").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "external-ticket",
        &product,
        &[
            "struct ExternalTicketRef",
            "fn attach_external_ticket",
            "GitHubIssues",
        ],
    );
}

/// Found case for required scenario: verification failure has no work-item to return.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_verification_failure_has_no_work_item() {
    let failed = sample_result(Effectiveness::Ineffective);
    assert_eq!(failed.effectiveness, Effectiveness::Ineffective);
    let json = serde_json::to_value(&failed).unwrap();
    assert!(json.get("verificationState").is_none());
    assert!(json.get("state").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "verification-failure",
        &product,
        &[
            "fn evaluate_verification",
            "enum VerificationPolicy",
            "AwaitingVerification",
        ],
    );
}

/// Found case for required scenario: one Effective result closes nothing (nothing exists).
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_sustained_success_does_not_close() {
    let first = sample_result(Effectiveness::Effective);
    let mut second = sample_result(Effectiveness::Effective);
    second.checked_at = Utc.with_ymd_and_hms(2026, 8, 21, 12, 0, 0).unwrap();
    assert_eq!(first.effectiveness, Effectiveness::Effective);
    assert_eq!(second.effectiveness, Effectiveness::Effective);

    let digest_a = canonical_digest(&first).unwrap();
    let digest_b = canonical_digest(&second).unwrap();
    assert_ne!(
        digest_a, digest_b,
        "two Effective observations remain distinct immutable results"
    );

    let mut assessment = empty_assessment();
    assessment.controls.push(sample_control());
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("remediations").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "sustained-success",
        &product,
        &[
            "SustainedWindow",
            "SingleGreenPermitted",
            "minEffectiveResults",
        ],
    );
}

/// Found case for required scenario: expired Exception does not reopen a waived remediation.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_expired_waiver_does_not_reopen_remediation() {
    let mut exception = Exception::new(ExceptionId::new("exc.expired-waiver"), "expired");
    exception.status = ExceptionStatus::Expired;
    exception.expires_at = Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());

    let mut assessment = empty_assessment();
    assessment.exceptions.push(exception);
    assessment
        .risks
        .push(Risk::new(RiskId::new("risk:still-open"), "t", "d"));
    assessment
        .validate()
        .expect("expired Exception does not interact with remediations or reopen work");

    let product = product_crate_sources_joined();
    forbid_needles(
        "expired-waiver",
        &product,
        &[
            "AcceptedWaived",
            "fn waiver_in_force",
            "fn reopen_expired_waiver",
            "struct WaiverBinding",
        ],
    );
}

/// Found case for required scenario: there is no close() or immutable closure history.
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_immutable_closure_history_absent() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("history").is_none());
    assert!(json.get("closedBy").is_none());
    assert!(json.get("closedAt").is_none());

    let product = product_crate_sources_joined();
    forbid_needles(
        "immutable-closure",
        &product,
        &[
            "struct RemediationEvent",
            "ImmutableClosure",
            "fn close(",
            "closureRationale",
        ],
    );

    empty_assessment()
        .validate()
        .expect("absence of closure history is not a validation error");
}

/// Dual-suite baseline registration (target is registered at implement).
#[test]
#[ignore = "superseded by sdd_remediation_engine_target"]
fn re_baseline_suite_is_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("sdd_remediation_engine_baseline")
            && cargo.contains("tests/contracts/remediation_engine.baseline.rs"),
        "baseline suite must be listed in root Cargo.toml"
    );
    assert!(
        !cargo.contains("tests/sdd/"),
        "Cargo.toml must not still point at tests/sdd/"
    );

    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/remediation-engine.md"),
        "remediation-engine spec must remain in CANONICAL_SPECS"
    );
}
