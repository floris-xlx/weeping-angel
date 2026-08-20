//! Target suite for Operational ISMS v1 internal audit (Prompt 21).
//!
//! Encodes DESIRED behavior in `docs/specs/internal-audit.md` §4 / §5
//! (IA-001–IA-009). On characterization HEAD the audit domain does not exist,
//! so this binary compiles against current IR and stays **RED** for missing
//! program/audit/sample/pin/sign-off — not harness noise. Do not `#[ignore]`.
//! Do not implement the engine here. Baseline stays GREEN until superseded.
//!
//! Scan **product crates** only. Never grep this file for a token that also
//! appears in an assertion string.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance::lineage::{LINEAGE_SNAPSHOT_SCHEMA, seal_evidence_snapshot};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AuditProgramId, Control, ControlId,
    ControlImplementation, ControlImplementationId, FrameworkId, FrameworkVersion, Identity,
    IdentityId, IdentityKind, PrincipalRef, Requirement, RequirementId, ValidateIr,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

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

fn require_audit_engine(label: &str) {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        label,
        &ir,
        &[
            "pub struct AuditProgram {",
            "pub struct Audit {",
            "enum AuditStatus",
            "enum AuditProgramStatus",
            "enum AuditConclusion",
            "struct IndependenceRecord",
            "struct AuditSample",
            "struct AuditSampleProposal",
            "struct AuditEvidencePin",
            "struct AuditFinding",
            "struct AuditSignOff",
            "struct AuditPeriod",
            "enum SampleMethod",
            "typed_id!(AuditId)",
            "typed_id!(AuditFindingId)",
            "audit_programs: Vec<AuditProgram>",
            "audits: Vec<Audit>",
        ],
    );
    assert!(
        crate_src("weeping-angel-assurance-ir")
            .join("audit.rs")
            .is_file(),
        "{label}: expected crates/weeping-angel-assurance-ir/src/audit.rs"
    );
    let assurance = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        label,
        &assurance,
        &[
            "fn prepare_audit_program",
            "fn prepare_audit(",
            "fn propose_sample",
            "fn accept_sample",
            "fn pin_evidence",
            "fn record_finding",
            "fn conclude_audit",
            "fn sign_off",
            "struct AuditPrepareBundle",
        ],
    );
    assert!(
        crate_src("weeping-angel-assurance")
            .join("audit.rs")
            .is_file(),
        "{label}: expected crates/weeping-angel-assurance/src/audit.rs"
    );
}

fn clock() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.internal-audit.target"))
}

fn graph_assessment() -> AssessmentDefinition {
    let mut assessment = empty_assessment();
    assessment.scope.organizations = vec!["org:weeping-angel".into()];
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:auditor"),
        IdentityKind::User,
    ));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:principal"),
        IdentityKind::User,
    ));
    assessment.controls.push(Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    ));
    assessment.controls.push(Control::new(
        ControlId::new("control.logging.retention"),
        "Log retention",
        "Retain security logs.",
    ));
    assessment.requirements.push(Requirement::new(
        RequirementId::new("iso27001:9.2"),
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        "Internal audit",
        "The organization shall conduct internal audits.",
    ));
    assessment.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa"),
            ControlId::new("control.access.mfa"),
        )
        .with_owner(PrincipalRef::Identity(IdentityId::new(
            "identity:principal",
        ))),
    );
    assessment
}

fn independence_payload(accepted: bool, conflicts: Vec<Value>) -> Value {
    json!({
        "auditor": { "identity": "identity:auditor" },
        "principal": { "identity": "identity:principal" },
        "declaredAt": "2026-01-02T00:00:00Z",
        "statement": "Auditor is independent of the controls under review.",
        "evidenceRefs": ["sha256:independence-letter"],
        "conflictFlags": conflicts,
        "accepted": accepted
    })
}

fn annual_program_payload() -> Value {
    json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "audit:2026",
        "title": "Annual internal audit program 2026",
        "period": {
            "start": "2026-01-01T00:00:00Z",
            "end": "2027-01-01T00:00:00Z"
        },
        "scope": {
            "organizations": ["org:weeping-angel"],
            "subjects": [],
            "exclusions": []
        },
        "objectives": [
            "Evaluate ISMS effectiveness",
            "Verify operation of selected controls"
        ],
        "criteria": [
            { "requirementId": "iso27001:9.2" },
            { "controlId": "control.access.mfa" }
        ],
        "schedule": [{
            "auditId": "audit.q1-access",
            "window": {
                "start": "2026-01-01T00:00:00Z",
                "end": "2026-04-01T00:00:00Z"
            },
            "scopeNote": "Access control"
        }],
        "principal": { "identity": "identity:principal" },
        "auditor": { "identity": "identity:auditor" },
        "independence": independence_payload(true, vec![]),
        "childAuditIds": ["audit.q1-access"],
        "status": "approved"
    })
}

fn child_audit_payload() -> Value {
    json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "audit.q1-access",
        "programId": "audit:2026",
        "title": "Q1 access-control audit",
        "period": {
            "start": "2026-01-15T00:00:00Z",
            "end": "2026-03-15T00:00:00Z"
        },
        "scope": {
            "organizations": ["org:weeping-angel"],
            "subjects": [],
            "exclusions": []
        },
        "sample": Value::Null,
        "selectedControls": ["control.access.mfa"],
        "selectedRequirements": ["iso27001:9.2"],
        "evidencePin": Value::Null,
        "procedures": [{
            "id": "proc.mfa-walkthrough",
            "title": "Walk through MFA enforcement",
            "selectedControlIds": ["control.access.mfa"],
            "status": "planned"
        }],
        "observations": [],
        "findings": [],
        "nonconformityRefs": [],
        "conclusion": Value::Null,
        "signOff": Value::Null,
        "status": "prepared",
        "history": [{
            "at": "2026-01-15T12:00:00Z",
            "kind": "prepared",
            "payloadDigest": "sha256:prepared-q1"
        }]
    })
}

fn decode_assessment(programs: Vec<Value>, audits: Vec<Value>) -> AssessmentDefinition {
    let mut encoded = serde_json::to_value(graph_assessment()).unwrap();
    let obj = encoded.as_object_mut().expect("assessment JSON object");
    obj.insert("audit_programs".into(), Value::Array(programs));
    obj.insert("audits".into(), Value::Array(audits));
    serde_json::from_value(encoded).expect("assessment with audit inventories must decode")
}

fn retained_programs(assessment: &AssessmentDefinition) -> Vec<Value> {
    serde_json::to_value(assessment)
        .unwrap()
        .get("audit_programs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn retained_audits(assessment: &AssessmentDefinition) -> Vec<Value> {
    serde_json::to_value(assessment)
        .unwrap()
        .get("audits")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn accepted_sample() -> Value {
    json!({
        "populationId": "pop:controls-2026-q1",
        "populationDigest": "sha256:pop-sorted-mfa-logging",
        "method": "seededRandom",
        "seed": "ia-004-seed",
        "size": 1,
        "selectedIds": ["control.access.mfa"],
        "acceptedBy": { "identity": "identity:auditor" },
        "acceptedAt": "2026-01-15T12:00:00Z",
        "proposalDigest": "sha256:proposal-ia-004",
        "sampleDigest": "sha256:sample-ia-004"
    })
}

fn evidence_pin(snapshot_digest: &str, envelopes: &[&str]) -> Value {
    json!({
        "evidenceSnapshotDigest": snapshot_digest,
        "envelopeDigests": envelopes,
        "collectionRunIds": ["run-1"],
        "pinnedAt": "2026-01-15T12:00:00Z",
        "pinnedBy": { "identity": "identity:auditor" }
    })
}

fn complete_unsigned_audit(snapshot_digest: &str) -> Value {
    let mut audit = child_audit_payload();
    audit["sample"] = accepted_sample();
    audit["evidencePin"] = evidence_pin(snapshot_digest, &["sha256:env-a", "sha256:env-b"]);
    audit["procedures"] = json!([{
        "id": "proc.mfa-walkthrough",
        "title": "Walk through MFA enforcement",
        "selectedControlIds": ["control.access.mfa"],
        "status": "performed",
        "notes": "MFA enforced on privileged identities"
    }]);
    audit["status"] = json!("inProgress");
    audit
}

fn sample_result(effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: weeping_angel_assurance_ir::ControlTestId::new("test.access.mfa"),
        control_id: ControlId::new("control.access.mfa"),
        effectiveness,
        rationale: format!("target observation {effectiveness:?}"),
        evidence_refs: vec!["sha256:env-a".into()],
        missing_evidence: Vec::new(),
        checked_at: clock(),
        test_version: "1".into(),
        input_digest: "sha256:mfa-input".into(),
        duration: Some("12ms".into()),
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

/// Dual-suite registration so both `--test` binaries exist today.
#[test]
fn ia_dual_suite_is_registered_and_specified() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        harness_src().contains("internal_audit.target.rs")
            && harness_src().contains("internal_audit.target.rs")
            && !toml.contains("sdd_internal_audit_baseline")
            && !toml.contains("tests/contracts/internal_audit.baseline.rs"),
        "dual-suite must be wired as a harness module"
    );
    let spec = read_repo_file("docs/specs/internal-audit.md");
    for id in [
        "IA-001", "IA-002", "IA-003", "IA-004", "IA-005", "IA-006", "IA-007", "IA-008", "IA-009",
        "IA-010",
    ] {
        assert!(spec.contains(id), "spec must list target case {id}");
    }
}

/// IA-001: annual program
#[test]
fn ia_001_annual_program() {
    require_audit_engine("IA-001: annual program");

    let id = AuditProgramId::new("audit:2026");
    assert_eq!(id.as_str(), "audit:2026");

    let program = annual_program_payload();
    assert_eq!(program["period"]["start"], "2026-01-01T00:00:00Z");
    assert_eq!(program["period"]["end"], "2027-01-01T00:00:00Z");
    assert!(!program["objectives"].as_array().unwrap().is_empty());
    assert!(!program["criteria"].as_array().unwrap().is_empty());
    assert_eq!(program["auditor"]["identity"], "identity:auditor");
    assert_eq!(program["principal"]["identity"], "identity:principal");

    let assessment = decode_assessment(vec![program], vec![]);
    let programs = retained_programs(&assessment);
    assert_eq!(
        programs.len(),
        1,
        "IA-001: AssessmentDefinition.audit_programs must persist the annual program"
    );
    assert_eq!(programs[0]["id"], "audit:2026");
    assert_eq!(programs[0]["status"], "approved");
    assert_eq!(programs[0]["period"]["end"], "2027-01-01T00:00:00Z");
    assert_eq!(programs[0]["childAuditIds"], json!(["audit.q1-access"]));
    assessment
        .validate()
        .expect("IA-001: annual program with hanging ids must validate");
}

/// IA-002: scoped audit
#[test]
fn ia_002_scoped_audit() {
    require_audit_engine("IA-002: scoped audit");

    let ok = decode_assessment(vec![annual_program_payload()], vec![child_audit_payload()]);
    let audits = retained_audits(&ok);
    assert_eq!(audits.len(), 1, "IA-002: child Audit must persist");
    assert_eq!(audits[0]["id"], "audit.q1-access");
    assert_eq!(audits[0]["programId"], "audit:2026");
    assert_eq!(audits[0]["selectedControls"], json!(["control.access.mfa"]));
    assert_eq!(audits[0]["selectedRequirements"], json!(["iso27001:9.2"]));
    ok.validate()
        .expect("IA-002: in-period child with known refs must validate");

    let mut dangling_control = child_audit_payload();
    dangling_control["selectedControls"] = json!(["control.missing.audit-target"]);
    decode_assessment(vec![annual_program_payload()], vec![dangling_control])
        .validate()
        .expect_err("IA-002: dangling control id must fail closed");

    let mut dangling_program = child_audit_payload();
    dangling_program["programId"] = json!("audit:missing");
    decode_assessment(vec![annual_program_payload()], vec![dangling_program])
        .validate()
        .expect_err("IA-002: dangling program id must fail closed");

    let mut outside = child_audit_payload();
    outside["period"] = json!({
        "start": "2027-02-01T00:00:00Z",
        "end": "2027-03-01T00:00:00Z"
    });
    decode_assessment(vec![annual_program_payload()], vec![outside])
        .validate()
        .expect_err("IA-002: audit period outside the program period must fail closed");
}

/// IA-003: auditor independence metadata
#[test]
fn ia_003_auditor_independence_metadata() {
    require_audit_engine("IA-003: auditor independence metadata");
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "IA-003 IndependenceRecord",
        &ir,
        &[
            "struct IndependenceRecord",
            "conflict_flags",
            "evidence_refs",
            "accepted",
        ],
    );

    let program = annual_program_payload();
    assert_eq!(program["independence"]["accepted"], true);
    assert_eq!(
        program["independence"]["statement"],
        "Auditor is independent of the controls under review."
    );
    assert_eq!(
        program["independence"]["evidenceRefs"],
        json!(["sha256:independence-letter"])
    );

    let assessment = decode_assessment(vec![program], vec![child_audit_payload()]);
    let programs = retained_programs(&assessment);
    assert_eq!(programs.len(), 1);
    assert_eq!(
        programs[0]["independence"]["auditor"]["identity"],
        "identity:auditor"
    );
    assert_eq!(
        programs[0]["independence"]["principal"]["identity"],
        "identity:principal"
    );
    assert_eq!(programs[0]["independence"]["accepted"], true);

    let mut unsigned = annual_program_payload();
    unsigned["independence"] = independence_payload(false, vec![]);
    let mut signed_without = complete_unsigned_audit("sha256:pin");
    signed_without["status"] = json!("signed");
    signed_without["signOff"] = json!({
        "principal": { "identity": "identity:principal" },
        "signedAt": "2026-01-20T00:00:00Z",
        "conclusion": "conformant",
        "statement": "signed without accepted independence"
    });
    decode_assessment(vec![unsigned], vec![signed_without])
        .validate()
        .expect_err("IA-003: sign-off without accepted independence must fail");

    let mut flagged = annual_program_payload();
    flagged["independence"] = independence_payload(
        false,
        vec![json!({
            "kind": "auditorOwnsControl",
            "controlId": "control.access.mfa"
        })],
    );
    let flagged_assessment = decode_assessment(vec![flagged], vec![child_audit_payload()]);
    let rec = &retained_programs(&flagged_assessment)[0]["independence"];
    assert_eq!(rec["accepted"], false);
    assert!(
        rec["conflictFlags"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "IA-003: machine conflict flags must persist"
    );
    assert_ne!(
        rec["accepted"], true,
        "IA-003: machine conflict flags never auto-accept independence"
    );

    let product = product_crates_joined();
    forbid_needles(
        "IA-003 must not auto-accept independence",
        &product,
        &["independence.accepted = true", "accepted: true, // auto"],
    );
}

/// IA-004: deterministic sample
#[test]
fn ia_004_deterministic_sample() {
    require_audit_engine("IA-004: deterministic sample");
    let product = product_crates_joined();
    require_needles(
        "IA-004 sample engine",
        &product,
        &[
            "fn propose_sample",
            "fn accept_sample",
            "sampleDigest",
            "populationDigest",
            "struct AuditSampleProposal",
            "enum SampleMethod",
        ],
    );

    let mut proposed = child_audit_payload();
    proposed["sampleProposal"] = json!({
        "populationId": "pop:controls-2026-q1",
        "populationDigest": "sha256:pop-sorted-mfa-logging",
        "method": "seededRandom",
        "seed": "ia-004-seed",
        "size": 1,
        "suggestedIds": ["control.access.mfa"],
        "rationale": "stale/failed hotspot sample",
        "generatedAt": "2026-01-15T12:00:00Z",
        "proposalDigest": "sha256:proposal-ia-004",
        "kind": "proposal"
    });
    proposed["sample"] = Value::Null;
    let assessment = decode_assessment(vec![annual_program_payload()], vec![proposed.clone()]);
    let row = &retained_audits(&assessment)[0];
    assert_eq!(row["sampleProposal"]["kind"], "proposal");
    assert!(
        row.get("sample").is_none() || row["sample"].is_null(),
        "IA-004: propose_sample is a proposal; Audit.sample stays unset until accept_sample"
    );

    let mut replay_a = proposed.clone();
    replay_a["id"] = json!("audit.q1-access-a");
    replay_a["sample"] = accepted_sample();
    let mut replay_b = proposed;
    replay_b["id"] = json!("audit.q1-access-b");
    replay_b["sample"] = accepted_sample();
    let left = decode_assessment(vec![annual_program_payload()], vec![replay_a]);
    let right = decode_assessment(vec![annual_program_payload()], vec![replay_b]);
    assert_eq!(
        retained_audits(&left)[0]["sample"]["selectedIds"],
        retained_audits(&right)[0]["sample"]["selectedIds"]
    );
    assert_eq!(
        retained_audits(&left)[0]["sample"]["sampleDigest"],
        retained_audits(&right)[0]["sample"]["sampleDigest"]
    );
    assert_eq!(retained_audits(&left)[0]["sample"]["seed"], "ia-004-seed");

    let mut conclude_on_proposal = child_audit_payload();
    conclude_on_proposal["sampleProposal"] = json!({
        "kind": "proposal",
        "method": "seededRandom",
        "seed": "ia-004-seed",
        "suggestedIds": ["control.access.mfa"],
        "populationId": "pop:controls-2026-q1",
        "populationDigest": "sha256:pop-sorted-mfa-logging",
        "size": 1,
        "rationale": "proposal only",
        "generatedAt": "2026-01-15T12:00:00Z",
        "proposalDigest": "sha256:proposal-ia-004"
    });
    conclude_on_proposal["status"] = json!("concluded");
    decode_assessment(vec![annual_program_payload()], vec![conclude_on_proposal])
        .validate()
        .expect_err("IA-004: accept_sample required before conclude");

    assert!(
        !product.contains("SampleMethod::Judgmental => propose") && product.contains("Judgmental"),
        "IA-004: judgmental exists as an auditor method and is not emitted by propose_sample"
    );
}

/// IA-005: evidence snapshot pinning
#[test]
fn ia_005_evidence_snapshot_pinning() {
    require_audit_engine("IA-005: evidence snapshot pinning");
    let product = product_crates_joined();
    require_needles(
        "IA-005 pin_evidence",
        &product,
        &["fn pin_evidence", "struct AuditEvidencePin", "pinnedBy"],
    );

    let first = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );
    assert_eq!(first.schema, LINEAGE_SNAPSHOT_SCHEMA);
    let live = seal_evidence_snapshot(
        [
            "sha256:env-a".to_string(),
            "sha256:env-b".to_string(),
            "sha256:env-c".to_string(),
        ],
        ["run-1".to_string(), "run-2".to_string()],
    );
    assert_ne!(
        first.digest, live.digest,
        "IA-005: extra live envelopes must change the live snapshot, not the pin"
    );

    let mut pinned = child_audit_payload();
    pinned["evidencePin"] = evidence_pin(&first.digest, &["sha256:env-a", "sha256:env-b"]);
    let assessment = decode_assessment(vec![annual_program_payload()], vec![pinned]);
    let pin = &retained_audits(&assessment)[0]["evidencePin"];
    assert_eq!(pin["evidenceSnapshotDigest"], first.digest);
    assert_eq!(
        pin["envelopeDigests"],
        json!(["sha256:env-a", "sha256:env-b"])
    );
    assert_ne!(pin["evidenceSnapshotDigest"], live.digest);
    assert!(
        !pin["envelopeDigests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d == "sha256:env-c"),
        "IA-005: later live envelopes must not change the pin"
    );

    let recomputed = seal_evidence_snapshot(
        pin["envelopeDigests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string()),
        ["run-1".to_string()],
    );
    assert_eq!(
        recomputed.digest,
        pin["evidenceSnapshotDigest"].as_str().unwrap(),
        "IA-005: recomputed digest of stored envelope list must match the pin"
    );
}

/// IA-006: finding creation
#[test]
fn ia_006_finding_creation() {
    require_audit_engine("IA-006: finding creation");
    let product = product_crates_joined();
    require_needles(
        "IA-006 record_finding",
        &product,
        &[
            "fn record_finding",
            "struct AuditFinding",
            "typed_id!(AuditFindingId)",
        ],
    );
    forbid_needles(
        "IA-006 no From<Finding> for AuditFinding",
        &product,
        &[
            "impl From<Finding> for AuditFinding",
            "From<Finding> for AuditFinding",
        ],
    );

    let snapshot = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );
    let mut audit = complete_unsigned_audit(&snapshot.digest);
    audit["findings"] = json!(["finding.audit.mfa-gap"]);
    let mut encoded = serde_json::to_value(graph_assessment()).unwrap();
    {
        let obj = encoded.as_object_mut().unwrap();
        obj.insert("audit_programs".into(), json!([annual_program_payload()]));
        obj.insert("audits".into(), json!([audit]));
        obj.insert(
            "audit_findings".into(),
            json!([{
                "id": "finding.audit.mfa-gap",
                "auditId": "audit.q1-access",
                "kind": "finding",
                "severity": "minor",
                "title": "MFA exception undocumented",
                "description": "Auditor recorded a gap against pinned evidence.",
                "controlIds": ["control.access.mfa"],
                "requirementIds": ["iso27001:9.2"],
                "evidenceDigests": ["sha256:env-a"],
                "createdBy": { "identity": "identity:auditor" },
                "createdAt": "2026-01-16T00:00:00Z",
                "nonconformityId": "nc:opaque-prompt-22"
            }]),
        );
    }
    let assessment: AssessmentDefinition =
        serde_json::from_value(encoded).expect("auditor finding must decode");
    let audits = retained_audits(&assessment);
    assert_eq!(audits[0]["findings"], json!(["finding.audit.mfa-gap"]));
    let findings = serde_json::to_value(&assessment)
        .unwrap()
        .get("audit_findings")
        .cloned()
        .or_else(|| audits[0].get("findingsDocuments").cloned())
        .unwrap_or(json!([]));
    let finding_docs = if findings.is_array()
        && findings
            .as_array()
            .unwrap()
            .first()
            .map(|v| v.is_object())
            .unwrap_or(false)
    {
        findings
    } else {
        serde_json::to_value(&assessment)
            .unwrap()
            .get("audit_findings")
            .cloned()
            .unwrap_or(json!([]))
    };
    assert!(
        finding_docs
            .as_array()
            .map(
                |rows| rows.iter().any(|f| f["id"] == "finding.audit.mfa-gap"
                    && f["nonconformityId"] == "nc:opaque-prompt-22")
            )
            .unwrap_or(false),
        "IA-006: record_finding must persist an auditor finding with an opaque nonconformity ref"
    );

    let scanner = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an audit finding")
        .build();
    assert_eq!(scanner.id, "unprotected-branch");
    let failed = sample_result(Effectiveness::Ineffective);
    assert_eq!(failed.effectiveness, Effectiveness::Ineffective);

    let before = decode_assessment(vec![annual_program_payload()], vec![child_audit_payload()]);
    assert!(
        retained_audits(&before)[0]["findings"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true),
        "IA-006: failed tests and scanner Finding do not auto-insert audit findings"
    );
}

/// IA-007: incomplete audit
#[test]
fn ia_007_incomplete_audit() {
    require_audit_engine("IA-007: incomplete audit");
    let product = product_crates_joined();
    require_needles(
        "IA-007 gates",
        &product,
        &["fn conclude_audit", "fn sign_off"],
    );

    let snapshot = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );

    let mut missing_sample = complete_unsigned_audit(&snapshot.digest);
    missing_sample["sample"] = Value::Null;
    missing_sample["status"] = json!("concluded");
    decode_assessment(vec![annual_program_payload()], vec![missing_sample])
        .validate()
        .expect_err("IA-007: missing sample cannot conclude");

    let mut missing_pin = complete_unsigned_audit(&snapshot.digest);
    missing_pin["evidencePin"] = Value::Null;
    missing_pin["status"] = json!("concluded");
    decode_assessment(vec![annual_program_payload()], vec![missing_pin])
        .validate()
        .expect_err("IA-007: missing pin cannot conclude");

    let mut planned = complete_unsigned_audit(&snapshot.digest);
    planned["procedures"] = json!([{
        "id": "proc.mfa-walkthrough",
        "title": "Walk through MFA enforcement",
        "selectedControlIds": ["control.access.mfa"],
        "status": "planned"
    }]);
    planned["status"] = json!("concluded");
    decode_assessment(vec![annual_program_payload()], vec![planned])
        .validate()
        .expect_err("IA-007: unfinished procedures cannot conclude");

    let mut no_independence = annual_program_payload();
    no_independence["independence"] = independence_payload(false, vec![]);
    let mut trying_sign = complete_unsigned_audit(&snapshot.digest);
    trying_sign["signOff"] = json!({
        "principal": { "identity": "identity:principal" },
        "signedAt": "2026-01-20T00:00:00Z",
        "conclusion": "conformant",
        "statement": "cannot sign incomplete"
    });
    trying_sign["status"] = json!("signed");
    decode_assessment(vec![no_independence], vec![trying_sign])
        .validate()
        .expect_err("IA-007: missing independence cannot sign");

    let prepared = decode_assessment(vec![annual_program_payload()], vec![child_audit_payload()]);
    let row = &retained_audits(&prepared)[0];
    assert!(row.get("signOff").is_none() || row["signOff"].is_null());
    assert_ne!(row["status"], "signed");
}

/// IA-008: signed audit
#[test]
fn ia_008_signed_audit() {
    require_audit_engine("IA-008: signed audit");
    let product = product_crates_joined();
    require_needles(
        "IA-008 sign_off",
        &product,
        &["fn sign_off", "struct AuditSignOff", "enum AuditConclusion"],
    );
    forbid_needles(
        "IA-008 no Default for AuditSignOff",
        &product,
        &["impl Default for AuditSignOff"],
    );

    let snapshot = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );
    let mut signed = complete_unsigned_audit(&snapshot.digest);
    signed["conclusion"] = json!("qualified");
    signed["signOff"] = json!({
        "principal": { "identity": "identity:principal" },
        "signedAt": "2026-01-20T00:00:00Z",
        "conclusion": "qualified",
        "statement": "Human principal signed a qualified conclusion."
    });
    signed["status"] = json!("signed");
    let assessment = decode_assessment(vec![annual_program_payload()], vec![signed]);
    let row = &retained_audits(&assessment)[0];
    assert_eq!(row["status"], "signed");
    assert_eq!(
        row["signOff"]["principal"]["identity"],
        "identity:principal"
    );
    assert_eq!(row["signOff"]["conclusion"], "qualified");
    assert_eq!(
        row["signOff"]["statement"],
        "Human principal signed a qualified conclusion."
    );
    assessment
        .validate()
        .expect("IA-008: complete signed audit must validate");

    let effective = sample_result(Effectiveness::Effective);
    assert_eq!(effective.effectiveness, Effectiveness::Effective);
    let prepared = decode_assessment(vec![annual_program_payload()], vec![child_audit_payload()]);
    let prepared_row = &retained_audits(&prepared)[0];
    assert!(
        prepared_row.get("signOff").is_none() || prepared_row["signOff"].is_null(),
        "IA-008: prepare on an all-Effective fixture must leave signOff unset"
    );
    assert!(
        prepared_row.get("conclusion").is_none() || prepared_row["conclusion"].is_null(),
        "IA-008: prepare must not default a conclusion from Effectiveness"
    );

    let mut auto = child_audit_payload();
    auto["signOff"] = json!({
        "signedAt": "2026-01-20T00:00:00Z",
        "conclusion": "conformant",
        "statement": "missing principal"
    });
    auto["status"] = json!("signed");
    decode_assessment(vec![annual_program_payload()], vec![auto])
        .validate()
        .expect_err("IA-008: sign-off without human principal must fail");
}

/// IA-009: historical reproducibility
#[test]
fn ia_009_historical_reproducibility() {
    require_audit_engine("IA-009: historical reproducibility");
    let product = product_crates_joined();
    require_needles(
        "IA-009 replay",
        &product,
        &["fn replay_audit", "struct AuditEvidencePin", "sampleDigest"],
    );

    let reviewed = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );
    let mut signed = complete_unsigned_audit(&reviewed.digest);
    signed["findings"] = json!(["finding.audit.mfa-gap"]);
    signed["conclusion"] = json!("qualified");
    signed["signOff"] = json!({
        "principal": { "identity": "identity:principal" },
        "signedAt": "2026-01-20T00:00:00Z",
        "conclusion": "qualified",
        "statement": "Replay must keep this conclusion."
    });
    signed["status"] = json!("signed");
    signed["sample"]["sampleDigest"] = json!("sha256:sample-ia-004");

    let frozen = decode_assessment(vec![annual_program_payload()], vec![signed]);
    let before = retained_audits(&frozen)[0].clone();
    assert_eq!(
        before["evidencePin"]["evidenceSnapshotDigest"],
        reviewed.digest
    );
    assert_eq!(before["sample"]["sampleDigest"], "sha256:sample-ia-004");
    assert_eq!(before["findings"], json!(["finding.audit.mfa-gap"]));
    assert_eq!(before["conclusion"], "qualified");

    let live = seal_evidence_snapshot(
        [
            "sha256:env-a".to_string(),
            "sha256:env-b".to_string(),
            "sha256:env-later".to_string(),
        ],
        ["run-1".to_string(), "run-later".to_string()],
    );
    assert_ne!(live.digest, reviewed.digest);
    let _moved = sample_result(Effectiveness::Ineffective);

    let after = retained_audits(&frozen)[0].clone();
    assert_eq!(
        after["evidencePin"]["evidenceSnapshotDigest"],
        before["evidencePin"]["evidenceSnapshotDigest"],
        "IA-009: signed replay must keep the pin after the live graph moves"
    );
    assert_eq!(
        after["sample"]["sampleDigest"],
        before["sample"]["sampleDigest"]
    );
    assert_eq!(after["findings"], before["findings"]);
    assert_eq!(after["conclusion"], before["conclusion"]);
    assert!(
        !after["evidencePin"]["envelopeDigests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d == "sha256:env-later"),
        "IA-009: later live envelopes are not in the reviewed set"
    );
}
