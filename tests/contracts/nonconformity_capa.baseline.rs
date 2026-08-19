//! Baseline suite for Operational ISMS v1 nonconformity / CAPA (Prompt 22).
//!
//! Characterization of CURRENT tree (`docs/specs/nonconformity-capa.md` §3) on
//! SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` + landed Prompts 15/16/19/21:
//! no `Nonconformity` / `CorrectiveAction` product type; `AssessmentDefinition`
//! has no CAPA inventories; `AssessmentRequests.nonconformities` and
//! `FrameworkCapabilities.supports_nonconformities` are fail-closed compile
//! flags that construct no CAPA objects; `AuditFinding.nonconformity_id` is
//! opaque `NonconformityRef = String` and `kind = nonconformity` does not start
//! CAPA; incident `corrective_action_ids` / PIR proposed ids are
//! `RemediationRef`; drift snapshot bags are empty `GovernanceRecord` no-ops;
//! catalog `control.governance.corrective-action` remains an attestation;
//! validation never walks CAPA; `ComplianceNodeRef` has no CAPA node;
//! `ASSURANCE_IR_SCHEMA` is still `assurance-ir/v1`.
//!
//! Must stay GREEN until `sdd_nonconformity_capa_target` is GREEN and this file
//! is skip-superseded. Does **not** implement the CAPA engine.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance::audit::record_finding;
use weeping_angel_assurance::drift::{GovernanceRecord, IsmsSnapshot, detect_isms_drift};
use weeping_angel_assurance::closed_incidents_with_open_corrective_actions;
use weeping_angel_assurance_ir::crosswalk::ComplianceNodeRef;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentRequests, Audit,
    AuditFinding, AuditFindingKind, AuditFindingSeverity, Control, ControlId, DetectionSource,
    Incident, IncidentId, IncidentKind, IncidentStatus, IsmsEventKind, NonconformityRef,
    PostIncidentReview, PrincipalRef, RemediationRef, RemediationSourceKind, ValidateIr,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkCompileError, FrameworkContext, FrameworkProfile,
    FrameworkTarget, compile_framework,
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
        "{label}: CAPA product types must be absent on characterization HEAD; found {present:?}"
    );
}

fn capa_engine_needles() -> &'static [&'static str] {
    &[
        "pub struct Nonconformity {",
        "pub struct Nonconformity{",
        "struct Nonconformity {",
        "typed_id!(NonconformityId)",
        "enum NonconformityStatus",
        "enum NonconformityClassification",
        "enum NonconformitySeverity",
        "struct RootCauseAnalysis",
        "struct ContainmentAction",
        "struct EffectivenessReview",
        "struct ClosureDecision",
        "pub struct CorrectiveAction {",
        "pub struct CorrectiveAction{",
        "struct CorrectiveAction {",
        "typed_id!(CorrectiveActionId)",
        "enum CorrectiveActionStatus",
        "fn contain_nonconformity",
        "fn record_root_cause",
        "fn evaluate_capa_effectiveness",
        "fn close_nonconformity",
        "fn propose_from_audit_finding",
        "fn propose_from_incident",
        "fn propose_from_control_regression",
        "fn overdue_corrective_actions",
        "fn open_nonconformities",
        "pub mod capa",
    ]
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.nonconformity-capa.baseline"))
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn assessment_definition_struct_src() -> String {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    let start = src
        .find("pub struct AssessmentDefinition")
        .expect("AssessmentDefinition struct");
    let rest = &src[start..];
    let end = rest
        .find("impl AssessmentDefinition")
        .expect("AssessmentDefinition impl");
    rest[..end].to_string()
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load")
}

fn fail_closed_iso27001() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn clock() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap()
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
        "selectedControls": ["control.access.mfa"],
        "selectedRequirements": ["iso27001:9.2"],
        "procedures": [],
        "observations": [],
        "findings": [],
        "nonconformityRefs": [],
        "status": "inProgress",
        "history": []
    })
}

fn opaque_nonconformity_finding(kind: &str, severity: &str) -> Value {
    json!({
        "id": "finding.audit.mfa-gap",
        "auditId": "audit.q1-access",
        "kind": kind,
        "severity": severity,
        "title": "MFA exception undocumented",
        "description": "Auditor labelled a nonconformity; that is not a CAPA open.",
        "controlIds": ["control.access.mfa"],
        "requirementIds": ["iso27001:9.2"],
        "evidenceDigests": [],
        "createdBy": { "identity": "identity:auditor" },
        "createdAt": "2026-01-16T00:00:00Z",
        "nonconformityId": "nc:opaque-prompt-22"
    })
}

/// NC-B001 found case: no Nonconformity / CorrectiveAction product type.
#[test]
fn nc_b001_no_nonconformity_or_corrective_action_product_type() {
    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("capa.rs").is_file() && !ir_src.join("nonconformity.rs").is_file(),
        "NC-B001: capa/nonconformity module must not exist on characterization HEAD"
    );
    assert!(
        !crate_src("weeping-angel-assurance").join("capa.rs").is_file(),
        "NC-B001: assurance capa engine must not exist on characterization HEAD"
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles("NC-B001 ir", &ir, capa_engine_needles());

    let ids = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        !ids.contains("typed_id!(NonconformityId)") && !ids.contains("typed_id!(CorrectiveActionId)"),
        "NC-B001: id.rs has no NonconformityId / CorrectiveActionId"
    );

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !lib.contains("pub mod capa")
            && !lib.contains("pub mod nonconformity")
            && !lib.contains("NonconformityId")
            && !lib.contains("CorrectiveActionId")
            && !lib.contains("pub use capa"),
        "NC-B001: lib.rs re-exports no CAPA types"
    );

    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !assurance_lib.contains("pub mod capa") && !assurance_lib.contains("close_nonconformity"),
        "NC-B001: assurance facade has no capa module"
    );

    let product = product_crate_sources_joined();
    forbid_needles("NC-B001 product", &product, capa_engine_needles());

    let root_src = manifest_dir().join("src");
    let mut root_files = Vec::new();
    walk_rs_files(&root_src, &mut root_files);
    let root_joined = root_files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    forbid_needles("NC-B001 src/", &root_joined, capa_engine_needles());
}

/// NC-B002 found case: AssessmentDefinition has no CAPA inventories.
#[test]
fn nc_b002_assessment_definition_has_no_capa_inventories() {
    let fields = assessment_definition_struct_src();
    assert!(
        !fields.contains("nonconformities") && !fields.contains("corrective_actions"),
        "NC-B002: AssessmentDefinition struct must not declare CAPA inventories"
    );

    let assessment = empty_assessment();
    assessment
        .validate()
        .expect("empty assessment remains valid");
    let json = serde_json::to_value(&assessment).unwrap();
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert!(
        !keys.iter().any(|k| k == "nonconformities"
            || k == "corrective_actions"
            || k == "correctiveActions"),
        "NC-B002: serialized AssessmentDefinition has no CAPA keys; got {keys:?}"
    );
    assert!(json.get("nonconformities").is_none());
    assert!(json.get("corrective_actions").is_none());
    assert!(json.get("correctiveActions").is_none());
    assert_eq!(json["schema_version"], ASSURANCE_IR_SCHEMA);

    let with_unknown = serde_json::from_value::<AssessmentDefinition>(json!({
        "id": "assess.capa.unknown-key",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "nonconformities": [{ "id": "nc.should-be-dropped" }],
        "correctiveActions": [{ "id": "ca.should-be-dropped" }]
    }))
    .expect("unknown CAPA keys are ignored; there is no inventory field");
    let round = serde_json::to_value(&with_unknown).unwrap();
    assert!(round.get("nonconformities").is_none());
    assert!(round.get("correctiveActions").is_none());
    assert!(round.get("corrective_actions").is_none());

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    let golden_json: Value = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden_json.get("nonconformities").is_none());
    assert!(golden_json.get("correctiveActions").is_none());
    assert_eq!(golden_json["requests"]["nonconformities"], false);
    assert!(golden.validate().is_ok());
}

/// NC-B003 found case: request/capability bits fail-closed; enabling both still yields no CAPA objects.
#[test]
fn nc_b003_request_and_capability_flags_fail_closed() {
    let requests = AssessmentRequests::default();
    assert!(
        !requests.nonconformities,
        "NC-B003: AssessmentRequests.nonconformities defaults false"
    );
    let req_json = serde_json::to_value(&requests).unwrap();
    assert_eq!(req_json["nonconformities"], false);

    let caps = FrameworkCapabilities::default();
    assert!(
        !caps.supports_nonconformities,
        "NC-B003: FrameworkCapabilities.supports_nonconformities defaults false"
    );

    let mut requested = empty_assessment();
    requested.requests.nonconformities = true;
    let err = compile_framework(&requested, &fail_closed_iso27001())
        .expect_err("NC-B003: nonconformities without support is CapabilityViolation");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(capability, "supports_nonconformities");
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }

    let enabled = FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities {
            supports_nonconformities: true,
            ..FrameworkCapabilities::default()
        },
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    };
    let compiled = compile_framework(&requested, &enabled)
        .expect("NC-B003: request + support compiles; it still constructs no CAPA objects");
    assert!(compiled.validation.ok);
    assert!(!compiled.digest.is_empty());
    let compiled_json = serde_json::to_value(&compiled).unwrap();
    assert!(compiled_json.get("nonconformities").is_none());
    assert!(compiled_json.get("correctiveActions").is_none());
    assert!(compiled_json.get("corrective_actions").is_none());

    let framework_src = read_repo_file("crates/weeping-angel-framework/src/lib.rs");
    assert!(
        framework_src.contains("supports_nonconformities")
            && framework_src.contains("req.nonconformities"),
        "NC-B003: capability pairing needle remains supports_nonconformities"
    );
}

/// NC-B004 found case: AuditFinding.nonconformity_id is an opaque string; kind=nonconformity does not start CAPA.
#[test]
fn nc_b004_audit_nonconformity_ref_is_opaque_and_does_not_start_capa() {
    let r: NonconformityRef = "nc:opaque-prompt-22".to_string();
    assert_eq!(r, "nc:opaque-prompt-22");
    let as_string: String = r.clone();
    assert_eq!(as_string, "nc:opaque-prompt-22");

    let audit_src = read_repo_file("crates/weeping-angel-assurance-ir/src/audit.rs");
    assert!(
        audit_src.contains("pub type NonconformityRef = String;"),
        "NC-B004: NonconformityRef remains a String alias"
    );
    assert!(
        audit_src.contains("Nonconformity,"),
        "NC-B004: AuditFindingKind::Nonconformity remains an auditor label"
    );

    let mut audit: Audit =
        serde_json::from_value(child_audit_payload()).expect("audit payload must decode");
    let finding: AuditFinding = serde_json::from_value(opaque_nonconformity_finding(
        "nonconformity",
        "major",
    ))
    .expect("finding with opaque nonconformityId must decode");
    assert_eq!(finding.kind, AuditFindingKind::Nonconformity);
    assert_eq!(finding.severity, Some(AuditFindingSeverity::Major));
    assert_eq!(
        finding.nonconformity_id.as_deref(),
        Some("nc:opaque-prompt-22")
    );

    let mut findings = Vec::new();
    record_finding(&mut audit, &mut findings, finding.clone()).expect("record_finding copies the opaque ref");
    assert_eq!(audit.nonconformity_refs, vec!["nc:opaque-prompt-22".to_string()]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, AuditFindingKind::Nonconformity);

    let mut assessment = empty_assessment();
    assessment.audits.push(audit);
    assessment.audit_findings.extend(findings);
    // Incomplete audit graph (no program) is not the point: opaque refs persist
    // and no CAPA inventory is created. Empty IR still validates.
    empty_assessment()
        .validate()
        .expect("empty assessment remains valid");
    let encoded = serde_json::to_value(&assessment).unwrap();
    assert!(encoded.get("nonconformities").is_none());
    assert_eq!(
        encoded["audit_findings"][0]["nonconformityId"],
        "nc:opaque-prompt-22"
    );
    assert_eq!(encoded["audit_findings"][0]["kind"], "nonconformity");
    assert_eq!(encoded["audit_findings"][0]["severity"], "major");

    let engine = crate_sources_joined("weeping-angel-assurance");
    assert!(
        engine.contains("fn record_finding"),
        "NC-B004: record_finding remains the audit seam"
    );
    forbid_needles(
        "NC-B004 record_finding must not mint CAPA",
        &engine,
        &[
            "fn propose_from_audit_finding",
            "Nonconformity::open",
            "From<AuditFinding> for Nonconformity",
        ],
    );
}

/// NC-B005 found case: incident corrective actions are Prompt 16 RemediationRef.
#[test]
fn nc_b005_incident_corrective_actions_are_remediation_refs() {
    let at = clock();
    let owner = PrincipalRef::Team("ir-owner".into());
    let mut incident = Incident::declare(
        IncidentId::new("inc.capa.baseline"),
        IncidentKind::Real,
        "Control failure in production",
        DetectionSource::Manual,
        at,
        owner.clone(),
    );
    let rem = RemediationRef::new("rem:prompt-16");
    incident.corrective_action_ids.push(rem.clone());
    incident.recovery_refs.push("sha256:recovery".into());
    incident.post_incident_review = Some(PostIncidentReview {
        recorded_at: at,
        recorded_by: owner.clone(),
        root_cause: Some("human error".into()),
        lessons_learned: "contain faster".into(),
        proposed_risk_ids: Vec::new(),
        proposed_control_ids: Vec::new(),
        proposed_corrective_action_ids: vec![RemediationRef::new("rem:pir-candidate")],
        evidence_refs: Vec::new(),
    });

    incident
        .transition(IncidentStatus::Contained, at, owner.clone())
        .unwrap();
    incident
        .transition(IncidentStatus::Recovered, at, owner.clone())
        .unwrap();
    incident
        .transition(IncidentStatus::Closed, at, owner.clone())
        .unwrap();
    assert_eq!(incident.status, IncidentStatus::Closed);
    assert_eq!(incident.corrective_action_ids, vec![rem.clone()]);

    let mut assessment = empty_assessment();
    assessment.incidents.push(incident);
    assessment
        .validate()
        .expect("closed incident with unresolved RemediationRef is still valid");

    let open = closed_incidents_with_open_corrective_actions(&assessment);
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].id.as_str(), "inc.capa.baseline");
    assert_eq!(
        open[0].corrective_action_ids[0].as_str(),
        "rem:prompt-16",
        "NC-B005: correctiveActionIds stay RemediationRef, not CorrectiveActionId"
    );

    let encoded = serde_json::to_value(&assessment).unwrap();
    assert_eq!(
        encoded["incidents"][0]["correctiveActionIds"],
        json!(["rem:prompt-16"])
    );
    assert!(encoded.get("nonconformities").is_none());
    assert!(encoded.get("correctiveActions").is_none());

    let pir = PostIncidentReview {
        recorded_at: at,
        recorded_by: owner,
        root_cause: Some("human error".into()),
        lessons_learned: "contain faster".into(),
        proposed_risk_ids: Vec::new(),
        proposed_control_ids: Vec::new(),
        proposed_corrective_action_ids: vec![RemediationRef::new("rem:pir-candidate")],
        evidence_refs: Vec::new(),
    };
    let _as_remediation_refs: Vec<RemediationRef> = pir.proposed_corrective_action_ids.clone();
    assert_eq!(
        pir.proposed_corrective_action_ids[0].as_str(),
        "rem:pir-candidate"
    );

    let incident_src = read_repo_file("crates/weeping-angel-assurance-ir/src/incident.rs");
    assert!(
        incident_src.contains("pub corrective_action_ids: Vec<RemediationRef>")
            && incident_src.contains("pub proposed_corrective_action_ids: Vec<RemediationRef>"),
        "NC-B005: incident/PIR corrective action fields remain RemediationRef"
    );
    assert!(
        !incident_src.contains("fn propose_nonconformity")
            && !incident_src.contains("propose_from_incident"),
        "NC-B005: Incident has no propose_nonconformity API"
    );
}

/// NC-B006 found case: drift names the events; empty bags are no-ops.
#[test]
fn nc_b006_drift_names_events_empty_bags_are_noops() {
    assert_eq!(
        IsmsEventKind::NonconformityOpened.as_label(),
        "NonconformityOpened"
    );
    assert_eq!(
        IsmsEventKind::CorrectiveActionOverdue.as_label(),
        "CorrectiveActionOverdue"
    );
    assert_eq!(
        RemediationSourceKind::from(&IsmsEventKind::NonconformityOpened),
        RemediationSourceKind::NonconformityOpened
    );
    assert_eq!(
        RemediationSourceKind::from(&IsmsEventKind::CorrectiveActionOverdue),
        RemediationSourceKind::CorrectiveActionOverdue
    );

    let prev = IsmsSnapshot {
        snapshot_id: "snap.prev".into(),
        evaluated_at: clock(),
        ..IsmsSnapshot::default()
    };
    let next = IsmsSnapshot {
        snapshot_id: "snap.next".into(),
        evaluated_at: clock(),
        ..IsmsSnapshot::default()
    };
    assert!(prev.nonconformities.is_empty());
    assert!(prev.corrective_actions.is_empty());
    assert!(next.nonconformities.is_empty());
    assert!(next.corrective_actions.is_empty());

    let empty_drift = detect_isms_drift(&prev, &next);
    assert!(
        empty_drift.events.iter().all(|e| {
            !matches!(
                e.kind,
                IsmsEventKind::NonconformityOpened | IsmsEventKind::CorrectiveActionOverdue
            )
        }),
        "NC-B006: empty GovernanceRecord bags emit no CAPA-named events"
    );

    let mut stuffed = next.clone();
    stuffed.nonconformities.push(GovernanceRecord {
        id: "nc.bag-only".into(),
        status: "open".into(),
        due_at: None,
    });
    stuffed.corrective_actions.push(GovernanceRecord {
        id: "ca.bag-only".into(),
        status: "overdue".into(),
        due_at: Some(clock()),
    });
    let stuffed_drift = detect_isms_drift(&prev, &stuffed);
    assert!(
        stuffed_drift
            .events
            .iter()
            .any(|e| matches!(e.kind, IsmsEventKind::NonconformityOpened)),
        "NC-B006: stuffing the bag still emits NonconformityOpened without a CAPA type"
    );
    assert!(
        stuffed_drift
            .events
            .iter()
            .any(|e| matches!(e.kind, IsmsEventKind::CorrectiveActionOverdue)),
        "NC-B006: stuffing the bag still emits CorrectiveActionOverdue without a CAPA type"
    );

    let drift_src = read_repo_file("crates/weeping-angel-assurance/src/drift.rs");
    assert!(
        drift_src.contains("pub nonconformities: Vec<GovernanceRecord>")
            && drift_src.contains("pub corrective_actions: Vec<GovernanceRecord>"),
        "NC-B006: snapshot inventories remain GovernanceRecord adapters"
    );
}

/// NC-B007 found case: catalog corrective-action is attestation, not this engine.
#[test]
fn nc_b007_catalog_corrective_action_is_attestation() {
    let catalog = load_catalog();
    let control = catalog
        .control("control.governance.corrective-action")
        .expect("catalog id must remain");
    assert_eq!(control.automation, "hybrid");
    assert_eq!(control.evidence, vec!["evidence.manual.attestation"]);
    assert_eq!(
        control.tests,
        vec!["test.governance.corrective-action-recorded"]
    );
    assert!(
        control
            .objective
            .to_ascii_lowercase()
            .contains("ticket identifier")
            || control
                .description
                .to_ascii_lowercase()
                .contains("attestation"),
        "NC-B007: catalog row stays an attestation fact: {}",
        control.objective
    );

    let test = catalog
        .tests()
        .get("test.governance.corrective-action-recorded")
        .expect("attestation test id must remain");
    assert_eq!(test.control, "control.governance.corrective-action");
    assert_eq!(test.required_evidence, vec!["evidence.manual.attestation"]);
    let op = test
        .expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(op, "manual-review");

    let toml = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(toml.contains("id = \"control.governance.corrective-action\""));
    assert!(toml.contains("id = \"control.governance.continual-improvement\""));
}

/// NC-B008 found case: validation never walks CAPA.
#[test]
fn nc_b008_validation_never_walks_capa() {
    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        !validation.to_ascii_lowercase().contains("nonconformit")
            && !validation.contains("corrective_action")
            && !validation.contains("CorrectiveAction"),
        "NC-B008: validate_assessment_ir must not walk CAPA inventories"
    );

    empty_assessment()
        .validate()
        .expect("clockless validate_assessment_ir does not require CAPA inventories");
    let finding: AuditFinding =
        serde_json::from_value(opaque_nonconformity_finding("nonconformity", "minor")).unwrap();
    assert_eq!(finding.nonconformity_id.as_deref(), Some("nc:opaque-prompt-22"));
}

/// NC-B009 found case: ComplianceNodeRef has no CAPA variants.
#[test]
fn nc_b009_crosswalk_has_no_capa_node() {
    fn classify(node: &ComplianceNodeRef) -> &'static str {
        match node {
            ComplianceNodeRef::Requirement(_) => "requirement",
            ComplianceNodeRef::Control(_) => "control",
            ComplianceNodeRef::Test(_) => "test",
            ComplianceNodeRef::EvidenceRequirement(_) => "evidence",
            ComplianceNodeRef::Risk(_) => "risk",
            ComplianceNodeRef::Exception(_) => "exception",
            ComplianceNodeRef::Incident(_) => "incident",
        }
    }

    let node = ComplianceNodeRef::Incident(IncidentId::new("inc.capa.baseline"));
    assert_eq!(classify(&node), "incident");

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/crosswalk.rs");
    assert!(
        !src.contains("Nonconformity(") && !src.contains("CorrectiveAction("),
        "NC-B009: ComplianceNodeRef has no CAPA variants"
    );
}

/// NC-B010 found case: schema stays assurance-ir/v1; one green control test mints no CAPA.
#[test]
fn nc_b010_schema_unchanged_and_one_green_is_not_capa() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");

    let result = ControlTestResult {
        test_id: weeping_angel_assurance_ir::ControlTestId::new("test.access.mfa"),
        control_id: ControlId::new("control.access.mfa"),
        effectiveness: Effectiveness::Effective,
        rationale: "single green observation".into(),
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
    };
    assert_eq!(result.effectiveness, Effectiveness::Effective);

    let assessment = empty_assessment();
    let encoded = serde_json::to_value(&assessment).unwrap();
    assert!(encoded.get("nonconformities").is_none());
    assert!(encoded.get("correctiveActions").is_none());

    let _control = Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    );
    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not a CAPA")
        .build();
    assert_eq!(finding.id, "unprotected-branch");
    let finding_src = read_repo_file("src/finding.rs");
    assert!(
        !finding_src.contains("Nonconformity") && !finding_src.contains("CorrectiveAction"),
        "NC-B010: scanner Finding must not promote into CAPA"
    );
}

/// Dual-suite registration so both `--test` binaries exist on this commit.
#[test]
fn nc_b011_dual_suite_is_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("name = \"sdd_nonconformity_capa_baseline\"")
            && cargo.contains("path = \"tests/contracts/nonconformity_capa.baseline.rs\"")
            && cargo.contains("name = \"sdd_nonconformity_capa_target\"")
            && cargo.contains("path = \"tests/contracts/nonconformity_capa.target.rs\""),
        "NC-B011: dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/nonconformity_capa.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/nonconformity_capa.target.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("docs/specs/nonconformity-capa.md")
            .is_file()
    );
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        !layout.contains("docs/specs/nonconformity-capa.md"),
        "NC-B011: CANONICAL_SPECS does not list this spec until implement"
    );
}
