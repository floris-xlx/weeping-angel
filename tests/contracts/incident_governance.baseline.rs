//! SUPERSEDED by `sdd_incident_governance_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/incident-governance.md` §3): no `Incident` / `IncidentId` IR,
//! `AssessmentDefinition` inventories omit incidents, `Risk` is still the
//! four-field stub, `validate_assessment_ir` never walks incidents, scanner
//! `Finding` in `src/finding.rs` is not IR and is not auto-promoted, Prompt 15
//! events/drift and Prompt 16 remediation are prompt-only seams, and
//! `control.incident.{response-plan,exercise,postmortem}` plus
//! `evidence.incident.exercise` remain governance-catalog capability tests.
//!
//! Target suite is the SSOT. Characterization tests are `#[ignore]` so CI does
//! not require the retired absences. Dual-suite registration remains.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentRequests, Control,
    ControlDomain, ControlId, ControlImplementation, ControlImplementationId, Risk, RiskId,
    RiskStatus, ValidateIr,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::Effectiveness;

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
        "{label}: incident-governance IR must be absent on characterization HEAD; found {present:?}"
    );
}

fn incident_engine_needles() -> &'static [&'static str] {
    &[
        "struct Incident ",
        "struct Incident{",
        "pub struct Incident",
        "enum IncidentStatus",
        "enum IncidentKind",
        "struct PostIncidentReview",
        "struct IncidentTimelineEvent",
        "struct IncidentEvent",
        "struct ControlFailureRef",
        "struct ExternalIncidentRef",
        "struct NotificationRecord",
        "enum DetectionSource",
        "typed_id!(IncidentId)",
        "pub struct IncidentId",
        "pub mod incident",
        "declare_incident",
        "fn declare_incident",
        "incident_postmortem_missing",
        "closed_incidents_with_open_corrective_actions",
    ]
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.incident-governance.baseline"))
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load")
}

/// IG-001 found case: constructing/deserializing scanner `Finding` does not create an incident.
#[test]
#[ignore = "superseded by target suite"]
fn ig_001_alert_or_finding_is_not_promoted() {
    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an ISMS incident")
        .build();
    assert_eq!(finding.id, "unprotected-branch");
    assert!(!finding.title.is_empty());

    let encoded = serde_json::to_value(&finding).unwrap();
    let back: Finding = serde_json::from_value(encoded).unwrap();
    assert_eq!(back.id, "unprotected-branch");

    let assessment = empty_assessment();
    let assessment_json = serde_json::to_value(&assessment).unwrap();
    assert!(
        assessment_json.get("incidents").is_none(),
        "IG-001: AssessmentDefinition JSON has no incidents inventory today"
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-001",
        &ir,
        &[
            "pub struct Finding",
            "struct FindingRef",
            "struct Alert",
            "struct AlertRef",
            "From<Finding>",
            "impl From<weeping_angel",
        ],
    );

    let finding_src = read_repo_file("src/finding.rs");
    assert!(
        finding_src.contains("pub struct Finding"),
        "scanner Finding remains in src/finding.rs"
    );
    assert!(
        !finding_src.contains("Incident") && !finding_src.contains("declare"),
        "src/finding.rs must not promote into incident IR"
    );
}

/// IG-002 found case: there is no declare API and no Incident / IncidentId type.
#[test]
#[ignore = "superseded by target suite"]
fn ig_002_no_declared_incident_record() {
    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("incident.rs").is_file(),
        "IG-002: incident.rs must not exist on characterization HEAD"
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles("IG-002", &ir, incident_engine_needles());

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !lib.contains("pub mod incident")
            && !lib.contains("IncidentId")
            && !lib.contains("Incident"),
        "IG-002: lib.rs must not re-export incident types"
    );

    let ids = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        !ids.contains("typed_id!(IncidentId)") && !ids.contains("IncidentId"),
        "IG-002: id.rs typed_id! list has no IncidentId"
    );
}

/// IG-003 found case: validation never walks a timeline or incidents inventory.
#[test]
#[ignore = "superseded by target suite"]
fn ig_003_validate_does_not_order_or_inspect_incident_timelines() {
    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    for needle in [
        "incident",
        "timeline",
        "declared_at",
        "postmortem",
        "post_incident",
        "recovery_refs",
    ] {
        assert!(
            !validation.to_ascii_lowercase().contains(needle),
            "IG-003: validation.rs must not mention `{needle}` today"
        );
    }

    empty_assessment()
        .validate()
        .expect("empty assessment still validates with no incident walk");
}

/// IG-004 found case: no ControlFailureRef; linking a regression is not an IR surface.
#[test]
#[ignore = "superseded by target suite"]
fn ig_004_no_control_regression_linkage_on_incidents() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "IG-004",
        &product,
        &[
            "struct ControlFailureRef",
            "ControlRegressed",
            "control_failure_refs",
        ],
    );

    let _ = Effectiveness::Effective;
    let _ = Effectiveness::Ineffective;
    let control = sample_control();
    assert!(control.domains().is_empty());
    let json = serde_json::to_value(&control).unwrap();
    assert!(json.get("controlFailureRefs").is_none());
    assert!(json.get("incidentIds").is_none());
}

/// IG-005 found case: recovered/closed recovery evidence is not an IR rule.
#[test]
#[ignore = "superseded by target suite"]
fn ig_005_no_recovery_evidence_rule() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-005",
        &ir,
        &[
            "recovery_refs",
            "recoveryRefs",
            "eradication_refs",
            "eradicationRefs",
            "struct IncidentContainment",
        ],
    );
}

/// IG-006 found case: no PostIncidentReview; catalog postmortem is attestation only.
#[test]
#[ignore = "superseded by target suite"]
fn ig_006_no_post_incident_review_record() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-006",
        &ir,
        &[
            "struct PostIncidentReview",
            "post_incident_review",
            "postIncidentReview",
            "incident_postmortem_missing",
        ],
    );

    let catalog = load_catalog();
    let control = catalog
        .control("control.incident.postmortem")
        .expect("governance catalog already publishes postmortem capability");
    assert!(
        control
            .evidence
            .iter()
            .any(|e| e == "evidence.manual.attestation")
    );
    assert!(
        control
            .tests
            .iter()
            .any(|t| t == "test.incident.postmortem-recorded")
    );
    assert_eq!(control.automation, "hybrid");
}

/// IG-007 found case: no IncidentKind; catalog exercise family is capability evidence.
#[test]
#[ignore = "superseded by target suite"]
fn ig_007_exercise_vs_real_is_catalog_capability_not_a_register() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles("IG-007", &ir, &["enum IncidentKind", "IncidentKind::"]);

    let catalog = load_catalog();
    let exercise = catalog
        .control("control.incident.exercise")
        .expect("governance catalog owns control.incident.exercise");
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
    assert!(exercise.domains.iter().any(|d| d == "incidentResponse"));

    let evidence = catalog
        .evidence()
        .get("evidence.incident.exercise")
        .expect("evidence.incident.exercise remains in the catalog");
    assert_eq!(evidence.evidence_type, "incident.exercise");

    let test = catalog
        .tests()
        .get("test.incident.exercise-current")
        .expect("test.incident.exercise-current remains in the catalog");
    assert_eq!(test.control, "control.incident.exercise");

    let plan = catalog
        .control("control.incident.response-plan")
        .expect("governance catalog owns control.incident.response-plan");
    assert!(plan.tests.iter().any(|t| t == "test.incident.plan-current"));

    let _ = ControlDomain::IncidentResponse;
}

/// IG-008 found case: no incident close / corrective-action pairing; Prompt 16 is absent.
#[test]
#[ignore = "superseded by target suite"]
fn ig_008_no_closed_incident_with_open_corrective_action() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-008",
        &ir,
        &[
            "corrective_action_ids",
            "correctiveActionIds",
            "closed_incidents_with_open_corrective_actions",
            "pub struct Remediation",
            "typed_id!(RemediationId)",
        ],
    );

    let assurance = crate_sources_joined("weeping-angel-assurance");
    forbid_needles(
        "IG-008 assurance",
        &assurance,
        &[
            "closed_incidents_with_open_corrective_actions",
            "pub struct Remediation",
            "incident_query",
        ],
    );

    let workbench = read_repo_file("src/workbench/remediation.rs");
    assert!(
        workbench.contains("pub struct RemediationRequest"),
        "scanner workbench remediations remain a patch helper, not IR"
    );
    assert!(
        !workbench.contains("Incident") && !workbench.contains("IncidentStatus"),
        "workbench remediation must not close or declare incidents"
    );
}

/// IG-009 found case: no append-only incident history or transition table.
#[test]
#[ignore = "superseded by target suite"]
fn ig_009_no_immutable_incident_history() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IG-009",
        &ir,
        &[
            "struct IncidentEvent",
            "enum IncidentStatus",
            "fn can_transition",
            "IncidentStatus::can_transition",
        ],
    );
}

/// IG-010 found case: no ExternalIncidentRef / adapter-free external pointer type.
#[test]
#[ignore = "superseded by target suite"]
fn ig_010_no_external_incident_system_refs() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "IG-010",
        &product,
        &[
            "struct ExternalIncidentRef",
            "external_refs",
            "PagerDutyIncident",
            "ServiceNowIncident",
        ],
    );
}

/// IG-011 found case: AssessmentDefinition has no incidents inventory; Risk is four fields; IR-019 holds.
#[test]
#[ignore = "superseded by target suite"]
fn ig_011_assessment_has_no_incidents_inventory() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
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
    assert!(json.get("incidents").is_none());
    assert_eq!(json["schema_version"], ASSURANCE_IR_SCHEMA);

    let requests = serde_json::to_value(AssessmentRequests::default()).unwrap();
    assert!(requests.get("incidents").is_none());
    assert!(requests.get("risk_treatment").is_some());
    assert!(requests.get("audit_program").is_some());
    assert!(requests.get("nonconformities").is_some());

    let with_unknown = serde_json::from_value::<AssessmentDefinition>(json!({
        "id": "assess.incident-governance.unknown-key",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "incidents": [{
            "id": "inc.should-be-dropped",
            "kind": "real",
            "title": "imported row is not stored today"
        }]
    }))
    .expect("unknown incidents key is ignored; there is no inventory field");
    let round = serde_json::to_value(&with_unknown).unwrap();
    assert!(round.get("incidents").is_none());

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    let golden_json: Value = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden_json.get("incidents").is_none());
    assert!(golden.risks.is_empty());
    golden.validate().unwrap();

    let assessment_src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !assessment_src.contains("incidents") && !assessment_src.contains("Incident"),
        "assessment.rs must not name an incidents inventory"
    );
}

/// IG-011 companion: Risk remains `{id,title,description,status}`; duplicate risk ids collapse.
#[test]
#[ignore = "superseded by target suite"]
fn ig_011_risk_is_four_field_stub_and_ir019_still_holds() {
    let risk = Risk::new(
        RiskId::new("risk:org-1"),
        "supplier concentration",
        "single critical vendor",
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
    assert!(json.get("incidentIds").is_none());
    assert!(json.get("source").is_none());

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        risk_src.contains("Minimal risk record. Not a risk engine."),
        "module docs remain the found-case product statement"
    );

    let mut dangling = empty_assessment();
    dangling.controls.push(sample_control());
    dangling.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling.validate().expect_err("IR-019: dangling risk");
    assert!(
        err.to_string().contains("dangling risk reference"),
        "IR-019 message: {err}"
    );

    let mut dupes = empty_assessment();
    let id = RiskId::new("risk:same");
    dupes
        .risks
        .push(Risk::new(id.clone(), "first", "first copy"));
    dupes
        .risks
        .push(Risk::new(id.clone(), "second", "second copy"));
    dupes.controls.push(sample_control());
    dupes.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(id),
    );
    dupes
        .validate()
        .expect("duplicate RiskIds silently collapse into the IR-019 id bag");
}

/// IG-011 companion: ComplianceNodeRef has no Incident variant.
#[test]
#[ignore = "superseded by target suite"]
fn ig_011_crosswalk_has_no_incident_node() {
    let crosswalk = read_repo_file("crates/weeping-angel-assurance-ir/src/crosswalk.rs");
    assert!(crosswalk.contains("enum ComplianceNodeRef"));
    for variant in [
        "Requirement(RequirementId)",
        "Control(ControlId)",
        "Test(ControlTestId)",
        "EvidenceRequirement(EvidenceRequirementId)",
        "Risk(RiskId)",
        "Exception(ExceptionId)",
    ] {
        assert!(
            crosswalk.contains(variant),
            "ComplianceNodeRef must still list {variant}"
        );
    }
    assert!(
        !crosswalk.contains("Incident(IncidentId)") && !crosswalk.contains("IncidentId"),
        "ComplianceNodeRef must not grow an Incident variant today"
    );
}

/// Prompt 15 found case: no ControlRegressed event bus / drift product.
#[test]
#[ignore = "superseded by target suite"]
fn ig_prompt15_events_drift_is_not_product() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "Prompt 15",
        &product,
        &[
            "ControlRegressed",
            "enum AssuranceEvent",
            "struct EventRef",
            "pub mod events",
            "pub mod drift",
        ],
    );
    assert!(
        !crate_src("weeping-angel-assurance")
            .join("events.rs")
            .is_file()
            && !crate_src("weeping-angel-assurance-ir")
                .join("event.rs")
                .is_file(),
        "Prompt 15 event/drift modules must be absent"
    );
}

/// Schema lock: IR stays assurance-ir/v1.
#[test]
#[ignore = "superseded by target suite"]
fn ig_schema_remains_assurance_ir_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

/// IG-012: dual-suite names are listed in root Cargo.toml.
#[test]
#[ignore = "superseded by target suite"]
fn ig_012_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_incident_governance_baseline")
            && toml.contains("sdd_incident_governance_target")
            && toml.contains("tests/contracts/incident_governance.baseline.rs")
            && toml.contains("tests/contracts/incident_governance.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/incident_governance.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/incident_governance.target.rs")
            .is_file()
    );
}
