//! SUPERSEDED by `sdd_control_implementation_registry_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/control-implementation-registry.md` §3): six statuses, thin
//! fields, silent overlap / dangling subjects. The operational registry is now
//! the SSOT in the target suite. Characterization tests are
//! `#[ignore = "superseded by target suite"]` so CI does not require the
//! retired absences. Dual-suite registration remains.

use std::any::TypeId;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Control, ControlId,
    ControlImplementation, ControlImplementationId, Exception, ExceptionId, ImplementationStatus,
    Risk, RiskId, ValidateIr, validate_assessment_ir,
};
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

fn read_crate_file(name: &str, rel: &str) -> String {
    fs::read_to_string(crate_src(name).join(rel))
        .unwrap_or_else(|e| panic!("read crates/{name}/src/{rel}: {e}"))
}

fn implementation_src() -> String {
    read_crate_file("weeping-angel-assurance-ir", "implementation.rs")
}

fn validation_src() -> String {
    read_crate_file("weeping-angel-assurance-ir", "validation.rs")
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn empty_assessment() -> AssessmentDefinition {
    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.cir.baseline"));
    assessment.controls.push(sample_control());
    assessment
}

fn impl_from_json(value: Value) -> ControlImplementation {
    serde_json::from_value(value)
        .unwrap_or_else(|e| panic!("deserialize ControlImplementation: {e}"))
}

fn status_json_name(status: ImplementationStatus) -> &'static str {
    match status {
        ImplementationStatus::NotImplemented => "notImplemented",
        ImplementationStatus::Planned => "planned",
        ImplementationStatus::PartiallyImplemented => "partiallyImplemented",
        ImplementationStatus::Implemented => "implemented",
        ImplementationStatus::NotApplicable => "notApplicable",
        ImplementationStatus::Retired => "retired",
        ImplementationStatus::Ineffective => "ineffective",
        ImplementationStatus::Unknown => "unknown",
    }
}

#[ignore = "superseded by target suite"]
#[test]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_control_implementation_registry_baseline")
            && toml.contains("sdd_control_implementation_registry_target")
            && toml.contains("tests/contracts/control_implementation_registry.baseline.rs")
            && toml.contains("tests/contracts/control_implementation_registry.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
}

/// CIR-B01: `ImplementationStatus` is exactly the six current variants.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b01_implementation_status_variants() {
    let variants = [
        ImplementationStatus::NotImplemented,
        ImplementationStatus::Planned,
        ImplementationStatus::PartiallyImplemented,
        ImplementationStatus::Implemented,
        ImplementationStatus::NotApplicable,
        ImplementationStatus::Retired,
    ];
    assert_eq!(variants.len(), 6);
    assert_eq!(
        ImplementationStatus::default(),
        ImplementationStatus::NotImplemented
    );
    let src = implementation_src();
    assert!(src.contains("enum ImplementationStatus"));
    for name in [
        "NotImplemented",
        "Planned",
        "PartiallyImplemented",
        "Implemented",
        "NotApplicable",
        "Retired",
    ] {
        assert!(src.contains(name), "missing variant {name}");
    }
    for absent in ["Ineffective", "Unknown", "Disabled", "Effective"] {
        assert!(
            !src.contains(&format!("{absent},")) && !src.contains(&format!("{absent}\n")),
            "baseline ImplementationStatus must not list {absent}"
        );
    }
}

/// CIR-B02: six camelCase strings round-trip; `ineffective` / `unknown` fail today.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b02_serde_camelcase_and_unknown_rejected() {
    let pairs = [
        (ImplementationStatus::NotImplemented, "notImplemented"),
        (ImplementationStatus::Planned, "planned"),
        (
            ImplementationStatus::PartiallyImplemented,
            "partiallyImplemented",
        ),
        (ImplementationStatus::Implemented, "implemented"),
        (ImplementationStatus::NotApplicable, "notApplicable"),
        (ImplementationStatus::Retired, "retired"),
    ];
    for (status, name) in pairs {
        assert_eq!(status_json_name(status), name);
        let value = serde_json::to_value(status).unwrap();
        assert_eq!(value, json!(name));
        let back: ImplementationStatus = serde_json::from_value(value).unwrap();
        assert_eq!(back, status);
    }
    for unknown in ["ineffective", "unknown", "disabled", "effective"] {
        let err = serde_json::from_value::<ImplementationStatus>(json!(unknown))
            .expect_err("unknown status string must fail today");
        assert!(
            !err.to_string().is_empty(),
            "serde error for `{unknown}` should be non-empty"
        );
    }
}

/// CIR-B03: current fields exist; registry surfaces and effectiveness are absent.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b03_fields_and_absent_registry_surfaces() {
    let src = implementation_src();
    for field in [
        "schema_version",
        "id:",
        "control_id",
        "status:",
        "owner:",
        "description:",
        "implemented_at",
        "applies_to",
        "compensating_controls",
        "exception_ids",
        "risk_ids",
    ] {
        assert!(
            src.contains(field),
            "expected field `{field}` in implementation.rs"
        );
    }
    for absent in [
        "asset_ids",
        "review_cadence",
        "next_review",
        "evidence_expectations",
        "document_refs",
        "treatment_ids",
        "automation",
        "supersedes",
        "superseded_by",
        "superseded_at",
        "effective_from",
    ] {
        assert!(
            !src.contains(absent),
            "baseline must not yet store `{absent}`"
        );
    }
    assert!(
        !src.contains("effectiveness:"),
        "ControlImplementation must not have an effectiveness field"
    );

    let populated = impl_from_json(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.org",
        "controlId": "control.access.mfa",
        "status": "implemented",
        "assetIds": ["asset:app-1"],
        "reviewCadence": { "intervalDays": 90 },
        "nextReview": "2026-12-01T00:00:00Z",
        "evidenceExpectations": ["evidence.req.mfa"],
        "documentRefs": [{ "id": "pol:access" }],
        "treatmentIds": ["treat:1"],
        "automation": "automated",
        "supersedes": "impl.access.mfa.prior",
        "supersededBy": "impl.access.mfa.next",
        "effectiveness": "effective"
    }));
    let out = serde_json::to_value(&populated).unwrap();
    for dropped in [
        "assetIds",
        "reviewCadence",
        "nextReview",
        "evidenceExpectations",
        "documentRefs",
        "treatmentIds",
        "automation",
        "supersedes",
        "supersededBy",
        "effectiveness",
        "effectiveFrom",
    ] {
        assert!(
            out.get(dropped).is_none(),
            "today extra field `{dropped}` is ignored, got {out}"
        );
    }
    assert_eq!(out["schemaVersion"], ASSURANCE_IR_SCHEMA);
    assert_eq!(out["id"], "impl.access.mfa.org");
    assert_eq!(out["controlId"], "control.access.mfa");
    assert_eq!(out["status"], "implemented");
}

/// CIR-B04: builders `new` / `with_status` / `with_risk` / `with_exception`; five getters.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b04_builders_and_limited_getters() {
    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("control.access.mfa"),
    )
    .with_status(ImplementationStatus::Implemented)
    .with_risk(RiskId::new("risk:source-tamper"))
    .with_exception(ExceptionId::new("exc:1"));

    assert_eq!(impln.id().as_str(), "impl.access.mfa.org");
    assert_eq!(impln.control_id().as_str(), "control.access.mfa");
    assert_eq!(impln.status(), ImplementationStatus::Implemented);
    assert_eq!(
        impln
            .risk_ids()
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>(),
        vec!["risk:source-tamper"]
    );
    assert_eq!(
        impln
            .exception_ids()
            .iter()
            .map(|e| e.as_str())
            .collect::<Vec<_>>(),
        vec!["exc:1"]
    );

    let fresh = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.fresh"),
        ControlId::new("control.access.mfa"),
    );
    assert_eq!(fresh.status(), ImplementationStatus::NotImplemented);

    let src = implementation_src();
    for present in [
        "pub fn new(",
        "pub fn with_status(",
        "pub fn with_risk(",
        "pub fn with_exception(",
        "pub fn id(",
        "pub fn control_id(",
        "pub fn status(",
        "pub fn risk_ids(",
        "pub fn exception_ids(",
    ] {
        assert!(src.contains(present), "missing `{present}`");
    }
    for absent in [
        "pub fn applies_to(",
        "pub fn owner(",
        "pub fn description(",
        "pub fn implemented_at(",
        "pub fn compensating_controls(",
        "pub fn schema_version(",
        "pub fn with_owner(",
        "pub fn with_applies_to(",
        "pub fn with_asset(",
        "pub fn with_review(",
        "fn superseding(",
    ] {
        assert!(
            !src.contains(absent),
            "baseline must not yet expose `{absent}`"
        );
    }
}

/// CIR-B05: dangling implementation control / risk / exception refs fail closed.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b05_validate_dangling_control_risk_exception() {
    let mut missing_control =
        AssessmentDefinition::new(AssessmentId::new("assess.cir.b05.control"));
    missing_control
        .implementations
        .push(ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        ));
    let err = validate_assessment_ir(&missing_control).expect_err("dangling control_id");
    assert!(
        err.to_string().contains("dangling implementation control"),
        "{err}"
    );

    let mut dangling_risk = empty_assessment();
    dangling_risk.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling_risk.validate().expect_err("IR-019 dangling risk");
    assert!(err.to_string().contains("dangling risk reference"), "{err}");

    let mut dangling_exc = empty_assessment();
    dangling_exc.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_exception(ExceptionId::new("exc:missing")),
    );
    let err = dangling_exc
        .validate()
        .expect_err("IR-020 dangling exception");
    assert!(
        err.to_string().contains("dangling exception reference"),
        "{err}"
    );

    let mut ok = empty_assessment();
    ok.risks.push(Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    ));
    ok.exceptions.push(Exception::new(
        ExceptionId::new("exc:1"),
        "timeboxed waiver",
    ));
    ok.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_status(ImplementationStatus::Implemented)
        .with_risk(RiskId::new("risk:source-tamper"))
        .with_exception(ExceptionId::new("exc:1")),
    );
    ok.validate().expect("resolved control/risk/exception refs");

    let validation = validation_src();
    assert!(validation.contains("dangling implementation control"));
    assert!(validation.contains("dangling risk reference"));
    assert!(validation.contains("dangling exception reference"));
}

/// CIR-B06: overlapping `applies_to` on one control validates Ok today.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b06_overlapping_applies_to_validates_ok() {
    let selector = json!([{
        "kind": "identity",
        "ids": ["identity:alice", "identity:bob"],
        "scope": "anyOf"
    }]);
    let mut assessment = empty_assessment();
    assessment.implementations.push(impl_from_json(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.employees",
        "controlId": "control.access.mfa",
        "status": "implemented",
        "appliesTo": selector
    })));
    assessment.implementations.push(impl_from_json(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.contractors",
        "controlId": "control.access.mfa",
        "status": "implemented",
        "appliesTo": selector
    })));
    validate_assessment_ir(&assessment)
        .expect("overlapping implementations of one control validate Ok today");

    let validation = validation_src();
    for needle in ["overlap", "double-count", "double_count", "applies_to"] {
        assert!(
            !validation.contains(needle),
            "validation.rs must not yet mention `{needle}`"
        );
    }
}

/// CIR-B07: `applies_to` / compensating ids not in inventory still validate Ok.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b07_dangling_subject_asset_ids_validate_ok() {
    let mut assessment = empty_assessment();
    assessment.implementations.push(impl_from_json(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.ghost",
        "controlId": "control.access.mfa",
        "status": "implemented",
        "appliesTo": [{
            "kind": "identity",
            "ids": ["identity:does-not-exist"],
            "scope": "anyOf"
        }],
        "compensatingControls": ["control.does-not-exist"]
    })));
    validate_assessment_ir(&assessment)
        .expect("dangling subject / compensating control ids are silent today");

    let mut dup = empty_assessment();
    let row = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("control.access.mfa"),
    );
    dup.implementations.push(row.clone());
    dup.implementations.push(row);
    dup.validate()
        .expect("duplicate ControlImplementation ids are not rejected today");
}

/// CIR-B08: module comment and type split: not control effectiveness.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b08_module_comment_not_effectiveness() {
    let src = implementation_src();
    assert!(
        src.contains("Organizational implementation state. Not control effectiveness."),
        "module docs must keep the effectiveness fence"
    );
    assert_ne!(
        TypeId::of::<ImplementationStatus>(),
        TypeId::of::<Effectiveness>(),
        "IR-009: status is not effectiveness"
    );
    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("control.access.mfa"),
    )
    .with_status(ImplementationStatus::Implemented);
    let json = serde_json::to_value(&impln).unwrap();
    assert!(json.get("effectiveness").is_none());
    assert_eq!(json["status"], "implemented");
}

/// CIR-B09: golden fixture deserializes to `Implemented`.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b09_golden_fixture_implemented() {
    let path = manifest_dir().join("tests/fixtures/assurance-ir/v1/control-implementation.json");
    let raw = fs::read_to_string(&path).unwrap();
    let impln: ControlImplementation = serde_json::from_str(&raw).unwrap();
    assert_eq!(impln.id().as_str(), "impl.access.mfa.org");
    assert_eq!(impln.control_id().as_str(), "control.access.mfa");
    assert_eq!(impln.status(), ImplementationStatus::Implemented);
    assert!(impln.risk_ids().is_empty());
    assert!(impln.exception_ids().is_empty());

    let fixture: Value = serde_json::from_str(&raw).unwrap();
    let mut keys: Vec<_> = fixture.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "controlId".to_string(),
            "id".to_string(),
            "schemaVersion".to_string(),
            "status".to_string()
        ]
    );
}

/// CIR-B10: control-test / evaluator sources do not read `ControlImplementation.applies_to`.
#[ignore = "superseded by target suite"]
#[test]
fn cir_b10_evaluator_does_not_read_applies_to() {
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !control_test.contains("ControlImplementation"),
        "control-test crate must not depend on ControlImplementation today"
    );
    assert!(
        !control_test.contains("applies_to"),
        "control-test evaluator must not mention applies_to"
    );

    let assurance = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !assurance.contains("applies_to"),
        "assurance crate must not read ControlImplementation.applies_to today"
    );
    assert!(
        assurance.contains("imp.control_id().as_str() == control_id"),
        "explain_control first-matches by control_id today"
    );
}
