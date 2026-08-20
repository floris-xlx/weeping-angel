//! Target suite for the control-implementation registry (CIR-001–015).
//!
//! Encodes DESIRED behavior in `docs/specs/control-implementation-registry.md` §4 / §5.
//! Must stay RED on CURRENT `ControlImplementation` (thin organizational row,
//! six statuses, no overlap / subject / evidence-expectation integrity).
//! Do not `#[ignore]` these tests and do not implement the registry here.

use std::any::TypeId;

use serde_json::{Value, json};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, Control,
    ControlId, ControlImplementation, ControlImplementationId, EvidenceCriticality,
    EvidenceRequirement, EvidenceRequirementId, EvidenceType, Exception, ExceptionId, Identity,
    IdentityId, IdentityKind, ImplementationStatus, Risk, RiskId, ValidateIr, Vendor, VendorId,
    validate_assessment_ir,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

fn implementation_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/implementation.rs")
}

fn validation_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs")
}

fn lib_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs")
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn impl_from_json(value: Value) -> ControlImplementation {
    serde_json::from_value(value)
        .unwrap_or_else(|e| panic!("deserialize ControlImplementation: {e}"))
}

fn round_trip(value: Value) -> Value {
    serde_json::to_value(impl_from_json(value)).unwrap()
}

fn control_with_required_evidence() -> Control {
    serde_json::from_value(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "control.access.mfa",
        "title": "MFA",
        "description": "Require multi-factor authentication.",
        "evidenceRequirements": ["evidence.req.mfa"]
    }))
    .expect("Control must deserialize evidenceRequirements")
}

fn required_evidence() -> EvidenceRequirement {
    EvidenceRequirement::new(
        EvidenceRequirementId::new("evidence.req.mfa"),
        EvidenceType::new("mfa-enrollment"),
    )
    .with_criticality(EvidenceCriticality::Required)
}

fn supporting_evidence() -> EvidenceRequirement {
    EvidenceRequirement::new(
        EvidenceRequirementId::new("evidence.req.mfa.supporting"),
        EvidenceType::new("mfa-attestation"),
    )
    .with_criticality(EvidenceCriticality::Supporting)
}

fn inventory_assessment() -> AssessmentDefinition {
    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.cir.target"));
    assessment.controls.push(control_with_required_evidence());
    assessment.evidence_requirements.push(required_evidence());
    assessment.evidence_requirements.push(supporting_evidence());
    assessment.identities.extend([
        Identity::new(IdentityId::new("identity:alice"), IdentityKind::User),
        Identity::new(IdentityId::new("identity:bob"), IdentityKind::User),
        Identity::new(IdentityId::new("identity:carol"), IdentityKind::User),
    ]);
    assessment.assets.extend([
        Asset::new(AssetId::new("asset:idp:okta"), AssetKind::Service, "Okta"),
        Asset::new(
            AssetId::new("asset:idp:azure-ad"),
            AssetKind::Service,
            "Entra ID",
        ),
        Asset::new(AssetId::new("asset:vpn:edge"), AssetKind::Network, "VPN"),
    ]);
    assessment.risks.push(Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    ));
    assessment.exceptions.push(Exception::new(
        ExceptionId::new("exc:1"),
        "timeboxed contractor waiver",
    ));
    assessment
        .vendors
        .push(Vendor::new(VendorId::new("vendor:okta"), "Okta"));
    assessment
}

fn employee_selector() -> Value {
    json!([{
        "kind": "identity",
        "ids": ["identity:alice", "identity:bob"],
        "tags": { "workforce": "employee" },
        "scope": "anyOf"
    }])
}

fn contractor_selector() -> Value {
    json!([{
        "kind": "identity",
        "ids": ["identity:carol"],
        "tags": { "workforce": "contractor" },
        "scope": "anyOf"
    }])
}

fn operational_row(id: &str, status: &str, applies_to: Value, asset_ids: Value) -> Value {
    json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": id,
        "controlId": "control.access.mfa",
        "status": status,
        "owner": { "identity": "identity:alice" },
        "description": "Okta MFA enrollment for the named population",
        "implementedAt": "2026-01-15T00:00:00Z",
        "effectiveFrom": "2026-01-15T00:00:00Z",
        "appliesTo": applies_to,
        "assetIds": asset_ids,
        "reviewCadence": { "intervalDays": 90 },
        "nextReview": "2026-04-15T00:00:00Z",
        "evidenceExpectations": ["evidence.req.mfa"],
        "documentRefs": [{
            "id": "pol:access-control",
            "title": "Access Control Policy",
            "kind": "policy"
        }],
        "riskIds": ["risk:source-tamper"],
        "treatmentIds": ["treat:mfa-rollout"],
        "exceptionIds": [],
        "automation": "hybrid",
        "compensatingControls": []
    })
}

const ADDITIVE_JSON_KEYS: &[&str] = &[
    "assetIds",
    "effectiveFrom",
    "reviewCadence",
    "nextReview",
    "evidenceExpectations",
    "documentRefs",
    "treatmentIds",
    "automation",
    "supersedes",
    "supersededBy",
    "supersededAt",
];

fn json_slot(value: &Value, key: &str) -> Value {
    match value.get(key) {
        Some(v) => v.clone(),
        None => json!([]),
    }
}

fn persist_operational_fields(out: &Value, payload: &Value) {
    for key in [
        "owner",
        "description",
        "effectiveFrom",
        "reviewCadence",
        "nextReview",
        "automation",
        "implementedAt",
    ] {
        assert!(
            out.get(key).is_some(),
            "CIR operational key `{key}` must survive serde, got {out}"
        );
        assert_eq!(out[key], payload[key], "round-trip mismatch for `{key}`");
    }
    for key in [
        "appliesTo",
        "assetIds",
        "evidenceExpectations",
        "documentRefs",
        "treatmentIds",
    ] {
        assert_eq!(
            json_slot(out, key),
            json_slot(payload, key),
            "round-trip mismatch for `{key}` (empty vec may omit); out={out}"
        );
        if payload
            .get(key)
            .is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()))
        {
            assert!(
                out.get(key).is_some(),
                "CIR operational key `{key}` must survive serde, got {out}"
            );
        }
    }
    assert!(
        out.get("effectiveness").is_none(),
        "ControlImplementation must not serialize effectiveness, got {out}"
    );
}

fn err_mentions(err: &impl std::fmt::Display, needles: &[&str]) {
    let text = err.to_string();
    for needle in needles {
        assert!(
            text.contains(needle),
            "error `{text}` must mention `{needle}`"
        );
    }
}

fn registry_query_src() -> String {
    format!(
        "{}\n{}\n{}",
        crate_sources_joined("weeping-angel-assurance-ir"),
        crate_sources_joined("weeping-angel-assurance"),
        lib_src()
    )
}

#[test]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_control_implementation_registry_baseline")
            && harness_src().contains("control_implementation_registry.target.rs")
            && !toml.contains("tests/contracts/control_implementation_registry.baseline.rs")
            && harness_src().contains("control_implementation_registry.target.rs"),
        "dual-suite must be wired as a harness module"
    );
}

/// CIR-001: Split-population implementations of one control (disjoint selectors)
#[test]
fn cir_001_split_population_implementations() {
    require_needles(
        "CIR-001 registry surfaces",
        &implementation_src(),
        &[
            "asset_ids",
            "effective_from",
            "review_cadence",
            "next_review",
            "evidence_expectations",
            "document_refs",
            "treatment_ids",
            "automation",
            "pub fn applies_to(",
            "pub fn owner(",
            "pub fn with_applies_to(",
            "ReviewCadence",
            "DocumentRef",
            "ImplementationAutomation",
        ],
    );
    require_needles(
        "CIR-001 query surface",
        &registry_query_src(),
        &[
            "fn implementations_for",
            "fn current_implementations_for",
            "fn implementation_by_id",
        ],
    );

    let employees = operational_row(
        "impl.access.mfa.employees",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    let contractors = operational_row(
        "impl.access.mfa.contractors",
        "implemented",
        contractor_selector(),
        json!(["asset:idp:okta"]),
    );
    persist_operational_fields(&round_trip(employees.clone()), &employees);
    persist_operational_fields(&round_trip(contractors.clone()), &contractors);

    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(employees));
    assessment.implementations.push(impl_from_json(contractors));
    validate_assessment_ir(&assessment)
        .expect("CIR-001: disjoint population implementations of one control must validate");

    let ids: Vec<_> = assessment
        .implementations
        .iter()
        .map(|row| row.id().as_str().to_string())
        .collect();
    assert_eq!(
        ids,
        vec![
            "impl.access.mfa.employees".to_string(),
            "impl.access.mfa.contractors".to_string()
        ]
    );
    assert!(
        assessment
            .implementations
            .iter()
            .all(|row| row.control_id().as_str() == "control.access.mfa")
    );
}

/// CIR-002: Partial rollout (`PartiallyImplemented` on a subset)
#[test]
fn cir_002_partial_rollout() {
    let payload = operational_row(
        "impl.access.mfa.employees.partial",
        "partiallyImplemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    let out = round_trip(payload.clone());
    persist_operational_fields(&out, &payload);
    assert_eq!(out["status"], "partiallyImplemented");
    assert!(out.get("effectiveness").is_none());
    assert_ne!(
        TypeId::of::<ImplementationStatus>(),
        TypeId::of::<Effectiveness>(),
        "CIR-002: partial rollout is organizational, not PartiallyEffective"
    );

    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(payload));
    validate_assessment_ir(&assessment)
        .expect("CIR-002: PartiallyImplemented subset rollout must validate");

    let src = implementation_src();
    assert!(
        !src.contains("effectiveness:"),
        "CIR-002: ControlImplementation must not grow an effectiveness field"
    );
    assert!(
        !src.contains("PartiallyEffective"),
        "CIR-002: do not encode test PartiallyEffective on the implementation record"
    );
}

/// CIR-003: Retired implementation sharing selectors with a current row
#[test]
fn cir_003_retired_implementation() {
    require_needles(
        "CIR-003 coverage-active / retired exclusion",
        &registry_query_src(),
        &[
            "fn current_implementations_for",
            "fn overlap_report",
            "Retired",
        ],
    );
    let src = format!("{}\n{}", implementation_src(), validation_src());
    assert!(
        src.contains("coverage_active")
            || src.contains("is_coverage_active")
            || src.contains("coverage-active"),
        "CIR-003: coverage-active helper must exclude Retired from overlap"
    );

    let current = operational_row(
        "impl.access.mfa.employees",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    let mut retired = operational_row(
        "impl.access.mfa.employees.legacy",
        "retired",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    retired["description"] = json!("Former Duo MFA for employees");
    retired["supersededBy"] = json!("impl.access.mfa.employees");
    retired["supersededAt"] = json!("2026-01-15T00:00:00Z");

    let out = round_trip(retired.clone());
    assert_eq!(out["status"], "retired");
    assert_eq!(out["supersededBy"], "impl.access.mfa.employees");
    persist_operational_fields(&round_trip(current.clone()), &current);

    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(retired));
    assessment.implementations.push(impl_from_json(current));
    validate_assessment_ir(&assessment)
        .expect("CIR-003: Retired (or superseded) rows may share selectors with the current row");
}

/// CIR-004: Missing evidence expectations on `Implemented`
#[test]
fn cir_004_missing_evidence_expectations() {
    let mut payload = operational_row(
        "impl.access.mfa.employees",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    payload["evidenceExpectations"] = json!([]);
    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(payload));
    let err = validate_assessment_ir(&assessment)
        .expect_err("CIR-004: Implemented with empty evidence_expectations must fail closed");
    err_mentions(&err, &["impl.access.mfa.employees", "evidence"]);

    let mut partial = operational_row(
        "impl.access.mfa.employees.partial",
        "partiallyImplemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    partial["evidenceExpectations"] = json!([]);
    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(partial));
    let err = validate_assessment_ir(&assessment)
        .expect_err("CIR-004: PartiallyImplemented also requires evidence expectation refs");
    assert!(err.to_string().contains("evidence"), "CIR-004: got {err}");
}

/// CIR-005: Control required `EvidenceRequirement` omitted from implementation
#[test]
fn cir_005_missing_required_evidence_refs() {
    let mut payload = operational_row(
        "impl.access.mfa.employees",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    payload["evidenceExpectations"] = json!(["evidence.req.mfa.supporting"]);
    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(payload));
    let err = validate_assessment_ir(&assessment).expect_err(
        "CIR-005: omitting a Required Control evidence-requirement ref must fail closed",
    );
    err_mentions(&err, &["impl.access.mfa.employees", "evidence.req.mfa"]);
}

/// CIR-006: One control, multiple systems (`asset_ids` disjoint)
#[test]
fn cir_006_one_control_multiple_systems() {
    let okta = operational_row(
        "impl.access.mfa.okta",
        "implemented",
        json!([]),
        json!(["asset:idp:okta"]),
    );
    let entra = operational_row(
        "impl.access.mfa.entra",
        "implemented",
        json!([]),
        json!(["asset:idp:azure-ad"]),
    );
    persist_operational_fields(&round_trip(okta.clone()), &okta);
    persist_operational_fields(&round_trip(entra.clone()), &entra);
    assert_eq!(
        round_trip(okta.clone())["assetIds"],
        json!(["asset:idp:okta"])
    );
    assert_eq!(
        round_trip(entra.clone())["assetIds"],
        json!(["asset:idp:azure-ad"])
    );

    require_needles(
        "CIR-006 asset_ids getter/builder",
        &implementation_src(),
        &["pub fn asset_ids(", "pub fn with_asset("],
    );

    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(okta));
    assessment.implementations.push(impl_from_json(entra));
    validate_assessment_ir(&assessment)
        .expect("CIR-006: disjoint asset_ids for one control must validate");
}

/// CIR-007: Overlapping `SubjectSelector`s (same kind + intersecting ids, or universal vs subset)
#[test]
fn cir_007_overlapping_subject_selectors() {
    require_needles(
        "CIR-007 overlap validation",
        &validation_src(),
        &["overlap"],
    );

    let mut intersecting = inventory_assessment();
    intersecting
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.left",
            "implemented",
            json!([{
                "kind": "identity",
                "ids": ["identity:alice", "identity:bob"],
                "scope": "anyOf"
            }]),
            json!([]),
        )));
    intersecting
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.right",
            "implemented",
            json!([{
                "kind": "identity",
                "ids": ["identity:bob", "identity:carol"],
                "scope": "anyOf"
            }]),
            json!([]),
        )));
    let err = validate_assessment_ir(&intersecting)
        .expect_err("CIR-007: intersecting AnyOf ids must fail closed (no silent double-count)");
    err_mentions(
        &err,
        &[
            "impl.access.mfa.left",
            "impl.access.mfa.right",
            "control.access.mfa",
        ],
    );
    let text = err.to_string();
    assert!(
        text.contains("identity:bob")
            || text.contains("anyOf")
            || text.contains("applies")
            || text.contains("selector"),
        "CIR-007: overlap error must be explainable, got {text}"
    );

    let mut universal = inventory_assessment();
    universal
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.whole",
            "implemented",
            json!([]),
            json!([]),
        )));
    universal
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.subset",
            "implemented",
            employee_selector(),
            json!(["asset:idp:okta"]),
        )));
    let err = validate_assessment_ir(&universal)
        .expect_err("CIR-007: empty applies_to + empty asset_ids is universal and overlaps");
    err_mentions(&err, &["impl.access.mfa.whole", "impl.access.mfa.subset"]);

    let mut asset_overlap = inventory_assessment();
    asset_overlap
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.okta-a",
            "implemented",
            json!([]),
            json!(["asset:idp:okta", "asset:vpn:edge"]),
        )));
    asset_overlap
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.okta-b",
            "implemented",
            json!([]),
            json!(["asset:idp:okta"]),
        )));
    let err = validate_assessment_ir(&asset_overlap)
        .expect_err("CIR-007: intersecting asset_ids must fail closed");
    err_mentions(
        &err,
        &[
            "impl.access.mfa.okta-a",
            "impl.access.mfa.okta-b",
            "asset:idp:okta",
        ],
    );
}

/// CIR-008: `Implemented != Effective`
#[test]
fn cir_008_implemented_is_not_effective() {
    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("control.access.mfa"),
    )
    .with_status(ImplementationStatus::Implemented);
    let impl_json = serde_json::to_value(&impln).unwrap();
    assert_eq!(impl_json["status"], "implemented");
    assert!(
        impl_json.get("effectiveness").is_none(),
        "CIR-008: no effectiveness field on ControlImplementation"
    );
    assert_ne!(
        TypeId::of::<ImplementationStatus>(),
        TypeId::of::<Effectiveness>()
    );
    assert_ne!(
        TypeId::of::<Control>(),
        TypeId::of::<ControlImplementation>()
    );
    assert_ne!(
        TypeId::of::<ControlImplementation>(),
        TypeId::of::<ControlTestResult>()
    );

    let result: ControlTestResult = serde_json::from_value(json!({
        "testId": "test.access.mfa",
        "controlId": "control.access.mfa",
        "effectiveness": "ineffective",
        "rationale": "enrollment below threshold"
    }))
    .unwrap();
    assert_eq!(result.effectiveness, Effectiveness::Ineffective);
    assert_eq!(impln.status(), ImplementationStatus::Implemented);

    let src = implementation_src();
    assert!(src.contains("Organizational implementation state. Not control effectiveness."));
    assert!(!src.contains("effectiveness:"));
    assert!(
        !src.contains("Effective,") && !src.contains("Effective\n"),
        "CIR-008: never add Effective to ImplementationStatus"
    );

    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        lineage.contains("pub implementation: Option<ControlImplementation>"),
        "CIR-008: ControlExplanation.implementation pin must keep compiling"
    );
    assert!(
        lineage.contains("pub effectiveness: Effectiveness"),
        "CIR-008: effectiveness stays on the explanation / test, not the implementation row"
    );
    assert!(
        lineage.contains("imp.control_id().as_str() == control_id"),
        "CIR-008: explain_control first-match-by-control_id stays"
    );
}

/// CIR-009: Supersession
#[test]
fn cir_009_supersession() {
    require_needles(
        "CIR-009 supersession fields",
        &implementation_src(),
        &[
            "supersedes",
            "superseded_by",
            "superseded_at",
            "fn superseding(",
        ],
    );
    require_needles(
        "CIR-009 snapshot query",
        &registry_query_src(),
        &["fn implementation_by_id", "fn current_implementations_for"],
    );

    let mut prior = operational_row(
        "impl.access.mfa.v1",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    prior["description"] = json!("Duo MFA (retired snapshot)");
    prior["supersededBy"] = json!("impl.access.mfa.v2");
    prior["supersededAt"] = json!("2026-03-01T00:00:00Z");

    let mut successor = operational_row(
        "impl.access.mfa.v2",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    successor["description"] = json!("Okta MFA (current)");
    successor["supersedes"] = json!("impl.access.mfa.v1");

    let prior_out = round_trip(prior.clone());
    let next_out = round_trip(successor.clone());
    assert_eq!(prior_out["id"], "impl.access.mfa.v1");
    assert_eq!(prior_out["supersededBy"], "impl.access.mfa.v2");
    assert_eq!(prior_out["supersededAt"], "2026-03-01T00:00:00Z");
    assert_eq!(next_out["supersedes"], "impl.access.mfa.v1");
    persist_operational_fields(&next_out, &successor);

    let mut assessment = inventory_assessment();
    assessment.implementations.push(impl_from_json(prior));
    assessment.implementations.push(impl_from_json(successor));
    validate_assessment_ir(&assessment)
        .expect("CIR-009: successor + prior snapshot must validate; prior is not coverage-active");

    let ids: Vec<_> = assessment
        .implementations
        .iter()
        .map(|row| row.id().as_str().to_string())
        .collect();
    assert!(
        ids.contains(&"impl.access.mfa.v1".to_string()),
        "CIR-009: prior snapshot remains queryable by id"
    );
    assert!(ids.contains(&"impl.access.mfa.v2".to_string()));

    let mut dangling = inventory_assessment();
    let mut orphan = operational_row(
        "impl.access.mfa.orphan",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    orphan["supersedes"] = json!("impl.access.mfa.missing");
    dangling.implementations.push(impl_from_json(orphan));
    let err = validate_assessment_ir(&dangling)
        .expect_err("CIR-009: dangling supersedes must fail closed");
    err_mentions(&err, &["impl.access.mfa.missing"]);
}

/// CIR-010: Dangling subject / asset / risk / control
#[test]
fn cir_010_dangling_refs_fail_closed() {
    let mut missing_control =
        AssessmentDefinition::new(AssessmentId::new("assess.cir.010.control"));
    missing_control
        .implementations
        .push(ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        ));
    let err = validate_assessment_ir(&missing_control)
        .expect_err("CIR-010: dangling implementation control_id");
    assert!(
        err.to_string().contains("dangling implementation control"),
        "{err}"
    );

    let mut dangling_subject = inventory_assessment();
    dangling_subject
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.ghost-id",
            "implemented",
            json!([{
                "kind": "identity",
                "ids": ["identity:does-not-exist"],
                "scope": "anyOf"
            }]),
            json!(["asset:idp:okta"]),
        )));
    let err = validate_assessment_ir(&dangling_subject)
        .expect_err("CIR-010: dangling applies_to identity must fail closed");
    err_mentions(&err, &["identity:does-not-exist"]);

    let mut dangling_asset = inventory_assessment();
    dangling_asset
        .implementations
        .push(impl_from_json(operational_row(
            "impl.access.mfa.ghost-asset",
            "implemented",
            employee_selector(),
            json!(["asset:does-not-exist"]),
        )));
    let err = validate_assessment_ir(&dangling_asset)
        .expect_err("CIR-010: dangling asset_ids must fail closed");
    err_mentions(&err, &["asset:does-not-exist"]);

    let mut dangling_risk = inventory_assessment();
    let mut row = operational_row(
        "impl.access.mfa.ghost-risk",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    row["riskIds"] = json!(["risk:missing"]);
    dangling_risk.implementations.push(impl_from_json(row));
    let err = dangling_risk
        .validate()
        .expect_err("CIR-010: dangling risk_ids (IR-019)");
    assert!(err.to_string().contains("dangling risk reference"), "{err}");

    let mut dangling_expectation = inventory_assessment();
    let mut row = operational_row(
        "impl.access.mfa.ghost-ev",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    );
    row["evidenceExpectations"] = json!(["evidence.req.missing"]);
    dangling_expectation
        .implementations
        .push(impl_from_json(row));
    let err = validate_assessment_ir(&dangling_expectation)
        .expect_err("CIR-010: dangling evidence expectation refs must fail closed");
    err_mentions(&err, &["evidence.req.missing"]);

    let mut dangling_exception = inventory_assessment();
    dangling_exception.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.ghost-exc"),
            ControlId::new("control.access.mfa"),
        )
        .with_exception(ExceptionId::new("exc:missing")),
    );
    let err = dangling_exception
        .validate()
        .expect_err("CIR-010: dangling exception_ids (IR-020)");
    assert!(
        err.to_string().contains("dangling exception reference"),
        "{err}"
    );

    let mut dup = inventory_assessment();
    let row = impl_from_json(operational_row(
        "impl.access.mfa.dup",
        "implemented",
        employee_selector(),
        json!(["asset:idp:okta"]),
    ));
    dup.implementations.push(row.clone());
    dup.implementations.push(row);
    let err = dup
        .validate()
        .expect_err("CIR-010: duplicate ControlImplementation ids fail closed");
    assert!(err.to_string().contains("impl.access.mfa.dup"), "{err}");
}

/// CIR-011: New states
#[test]
fn cir_011_new_states() {
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
        let value = serde_json::to_value(status).unwrap();
        assert_eq!(value, json!(name), "existing camelCase `{name}` must stay");
        let back: ImplementationStatus = serde_json::from_value(value).unwrap();
        assert_eq!(back, status);
    }

    let ineffective: ImplementationStatus = serde_json::from_value(json!("ineffective"))
        .expect("CIR-011: ineffective must deserialize");
    assert_eq!(
        serde_json::to_value(ineffective).unwrap(),
        json!("ineffective")
    );
    let via_disabled: ImplementationStatus = serde_json::from_value(json!("disabled"))
        .expect("CIR-011: disabled is an accept-alias for Ineffective");
    assert_eq!(
        serde_json::to_value(via_disabled).unwrap(),
        json!("ineffective"),
        "CIR-011: disabled alias must serialize as ineffective"
    );
    let unknown: ImplementationStatus =
        serde_json::from_value(json!("unknown")).expect("CIR-011: unknown must deserialize");
    assert_eq!(serde_json::to_value(unknown).unwrap(), json!("unknown"));

    serde_json::from_value::<ImplementationStatus>(json!("effective"))
        .expect_err("CIR-011: never add Effective / remap implemented");

    let src = implementation_src();
    require_needles(
        "CIR-011 additive variants",
        &src,
        &["Ineffective", "Unknown"],
    );
    assert!(
        !src.contains("Effective,") && !src.contains("    Effective\n"),
        "CIR-011: ImplementationStatus must not grow Effective"
    );
}

/// CIR-012: Golden fixture + IR-008/009 still hold
#[test]
fn cir_012_golden_fixture_and_ir_types() {
    let raw = read_repo_file("tests/fixtures/assurance-ir/v1/control-implementation.json");
    let impln: ControlImplementation = serde_json::from_str(&raw).unwrap();
    assert_eq!(impln.id().as_str(), "impl.access.mfa.org");
    assert_eq!(impln.control_id().as_str(), "control.access.mfa");
    assert_eq!(impln.status(), ImplementationStatus::Implemented);
    assert_eq!(
        serde_json::from_str::<Value>(&raw).unwrap()["schemaVersion"],
        ASSURANCE_IR_SCHEMA
    );
    assert_ne!(
        TypeId::of::<Control>(),
        TypeId::of::<ControlImplementation>(),
        "IR-008"
    );
    assert_ne!(
        TypeId::of::<ImplementationStatus>(),
        TypeId::of::<Effectiveness>(),
        "IR-009"
    );
}

/// CIR-013: Additive serde
#[test]
fn cir_013_additive_serde() {
    let fixture = impl_from_json(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.org",
        "controlId": "control.access.mfa",
        "status": "implemented"
    }));
    assert_eq!(fixture.status(), ImplementationStatus::Implemented);
    let out = serde_json::to_value(&fixture).unwrap();
    assert_eq!(out["schemaVersion"], ASSURANCE_IR_SCHEMA);
    assert_eq!(out["status"], "implemented");
    for key in ADDITIVE_JSON_KEYS {
        assert!(
            out.get(*key).is_none() || out[*key] == json!([]) || out[*key] == json!(null),
            "CIR-013: missing additive `{key}` defaults empty, got {:?}",
            out.get(*key)
        );
    }

    assert!(
        lib_src().contains("assurance-ir/v1") && ASSURANCE_IR_SCHEMA == "assurance-ir/v1",
        "CIR-013: schema remains assurance-ir/v1 (no IR fork)"
    );
    let populated = round_trip(json!({
        "schemaVersion": ASSURANCE_IR_SCHEMA,
        "id": "impl.access.mfa.org",
        "controlId": "control.access.mfa",
        "status": "implemented",
        "effectiveFrom": "2026-01-15T00:00:00Z",
        "reviewCadence": { "intervalDays": 90 },
        "nextReview": "2026-04-15T00:00:00Z",
        "assetIds": ["asset:idp:okta"],
        "evidenceExpectations": ["evidence.req.mfa"],
        "documentRefs": [{ "id": "pol:access-control" }],
        "treatmentIds": ["treat:mfa-rollout"],
        "automation": "hybrid"
    }));
    for key in [
        "effectiveFrom",
        "reviewCadence",
        "nextReview",
        "assetIds",
        "evidenceExpectations",
        "documentRefs",
        "treatmentIds",
        "automation",
    ] {
        assert!(
            populated.get(key).is_some(),
            "CIR-013: additive field `{key}` must round-trip once present, got {populated}"
        );
    }
}

/// CIR-014: No competing type
#[test]
fn cir_014_no_competing_type() {
    let src = implementation_src();
    assert!(src.contains("pub struct ControlImplementation"));
    for forbidden in [
        "struct OrgControlImplementation",
        "struct ControlDeployment",
        "struct ImplementationRegistryRecord",
        "struct OrgControlDeployment",
    ] {
        let ir = crate_sources_joined("weeping-angel-assurance-ir");
        assert!(
            !ir.contains(forbidden),
            "CIR-014: do not introduce competing SSOT `{forbidden}`"
        );
    }
    require_needles(
        "CIR-014 ControlImplementation remains SSOT",
        &lib_src(),
        &[
            "ControlImplementation",
            "ImplementationStatus",
            "PrincipalRef",
        ],
    );
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

/// CIR-015: Neighbor targets
#[test]
fn cir_015_neighbor_targets() {
    for name in [
        "sdd_assurance_runtime_target",
        "sdd_iso27001_assurance_target",
        "sdd_assessment_lineage_target",
        "sdd_documentation_layout",
        "sdd_compliance_ir_target",
    ] {
        assert!(
            sdd_suite_wired(name),
            "CIR-015: neighbor suite `{name}` must stay registered"
        );
    }
    let spec = read_repo_file("docs/specs/control-implementation-registry.md");
    assert!(spec.contains("sdd_assurance_runtime_target"));
    assert!(spec.contains("sdd_iso27001_assurance_target"));
    assert!(spec.contains("sdd_assessment_lineage_target"));
    assert!(spec.contains("ControlExplanation.implementation"));
}
