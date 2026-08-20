//! Target suite: Compliance IR deepen (IR-001…025).
//!
//! Desired behavior for weeping-angel-assurance-ir. Must stay GREEN with
//! ACT-001…015. Do not treat findings or collectors as compliance results.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::json;
use weeping_angel_assurance_ir::crosswalk::ComplianceGraph;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ApplicabilityRule, AssessmentDefinition, AssessmentScope, Asset, AssetId,
    AssetKind, CanonicalizationVersion, Control, ControlDomain, ControlId, ControlImplementation,
    ControlImplementationId, ControlTestId, EvidenceCardinality, EvidenceCollectionKind,
    EvidenceCriticality, EvidenceRequirement, EvidenceRequirementId, EvidenceType, Exception,
    ExceptionId, ExtensionMap, FrameworkId, FrameworkRef, FrameworkVersion, IdError, Identity,
    IdentityId, IdentityKind, ImplementationStatus, Mapping, MappingCompleteness,
    MappingConfidence, MappingDirection, MappingId, MappingProvenance, MappingRelation,
    MappingSource, MappingVersionConstraint, PlannedControlTest, PlannedTestKind, PrincipalRef,
    ProcessingActivity, ProcessingActivityId, Requirement, RequirementId, RequirementKind, Risk,
    RiskId, SubjectKind, SubjectSelector, ValidateIr, Vendor, VendorId, canonical_digest,
    typed_canonical_digest,
};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn ir_src() -> PathBuf {
    manifest_dir().join("crates/weeping-angel-assurance-ir")
}

fn ir_sources() -> String {
    let mut files = Vec::new();
    walk_rs(&ir_src(), &mut files);
    files
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn walk_rs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn sample_requirement() -> Requirement {
    Requirement::new(
        RequirementId::new("req.iso27001.2022.a-5-15"),
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        "Access control",
        "Limit access to information.",
    )
}

#[test]
fn ir_001_stable_ids_reject_empty_values() {
    assert!(
        matches!(ControlId::try_new(""), Err(IdError::Empty)),
        "IR-001: empty ControlId must be rejected"
    );
    assert!(
        matches!(ControlId::try_new("     "), Err(IdError::Empty)),
        "IR-001: whitespace-only ControlId must be rejected"
    );
    assert!(
        RequirementId::try_new("req.iso27001.2022.a-5-15").is_ok(),
        "IR-001: valid IDs must still construct"
    );
}

#[test]
fn ir_002_title_changes_do_not_mutate_identity() {
    let id = ControlId::new("control.access.mfa");
    let a = Control::new(id.clone(), "MFA", "one");
    let b = Control::new(id.clone(), "Multi-factor authentication", "two");
    assert_eq!(a.id(), b.id(), "IR-002: title is not identity");
    assert_eq!(a.id().as_str(), "control.access.mfa");
}

#[test]
fn ir_003_controls_contain_no_framework_specific_fields() {
    let control = sample_control()
        .with_objective("Prevent unauthorized access")
        .with_domain(ControlDomain::Authentication);
    let value = serde_json::to_value(&control).unwrap();
    let mut keys = BTreeSet::new();
    collect_keys(&value, &mut keys);
    for key in &keys {
        let folded = key.to_ascii_lowercase().replace('-', "_");
        assert!(
            !folded.contains("annex")
                && !folded.contains("soa")
                && !folded.contains("clause")
                && !folded.contains("iso27001")
                && !folded.contains("gdpr")
                && !folded.contains("soc2"),
            "IR-003: Control must not carry framework-specific field `{key}`"
        );
    }
    assert!(value.get("owner").is_none());
    assert!(value.get("implemented").is_none());
}

#[test]
fn ir_004_requirements_preserve_framework_identity() {
    let req = sample_requirement();
    assert_eq!(req.framework().id().as_str(), "iso-27001");
    assert_eq!(req.framework().version().as_str(), "2022");
    assert_eq!(req.kind(), RequirementKind::Requirement);
}

#[test]
fn ir_005_mappings_reject_dangling_nodes() {
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-005",
    ));
    assessment.controls.push(sample_control());
    assessment.mappings.push(Mapping::new(
        RequirementId::new("req.missing"),
        ControlId::new("control.access.mfa"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    ));
    let err = assessment
        .validate()
        .expect_err("IR-005: dangling mapping must fail");
    assert!(
        err.to_string().contains("dangling") || err.to_string().contains("mapping"),
        "IR-005: got {err}"
    );
}

#[test]
fn ir_006_partial_mapping_never_becomes_equivalence() {
    let mut graph = ComplianceGraph::new();
    let a = RequirementId::new("fw-a:r1");
    let b = RequirementId::new("fw-b:r1");
    let c = RequirementId::new("fw-c:r1");
    graph.link(
        a.clone(),
        b.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    graph.link(
        b,
        c.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert!(
        !graph.equivalent(&a, &c),
        "IR-006: no transitive equivalence"
    );
}

#[test]
fn ir_007_generated_mapping_does_not_gain_curated_authority() {
    let generated = MappingProvenance {
        source: MappingSource::Generated,
        author: None,
        reference: None,
        reviewed_at: None,
    };
    let curated = MappingProvenance {
        source: MappingSource::AuditorApproved,
        author: Some("auditor".into()),
        reference: None,
        reviewed_at: None,
    };
    assert!(
        !generated.has_curated_authority(),
        "IR-007: generated mappings are not curated"
    );
    assert!(curated.has_curated_authority());
}

#[test]
fn ir_008_control_implementation_is_not_control_definition() {
    assert_ne!(
        std::any::TypeId::of::<Control>(),
        std::any::TypeId::of::<ControlImplementation>()
    );
    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("control.access.mfa"),
    )
    .with_status(ImplementationStatus::Implemented);
    assert_eq!(impln.control_id().as_str(), "control.access.mfa");
    assert_eq!(impln.status(), ImplementationStatus::Implemented);
    let _ = (
        Asset::new(
            AssetId::new("asset:org:root"),
            AssetKind::Organization,
            "Org",
        ),
        Identity::new(IdentityId::new("identity:alice"), IdentityKind::User),
        Vendor::new(VendorId::new("vendor:acme"), "Acme"),
        ProcessingActivity::new(ProcessingActivityId::new("ropa:payroll"), "Payroll"),
        MappingId::new("map.req.control"),
        MappingConfidence::High,
        MappingRelation::Satisfies,
        PrincipalRef::Role("ciso".into()),
    );
}

#[test]
fn ir_009_implementation_status_is_not_effectiveness() {
    let _status = ImplementationStatus::Implemented;
    assert_ne!(
        std::any::TypeId::of::<ImplementationStatus>(),
        std::any::TypeId::of::<weeping_angel_control_test::Effectiveness>()
    );
}

#[test]
fn ir_010_applicability_round_trips_deterministically() {
    let rule = ApplicabilityRule::All(vec![
        ApplicabilityRule::jurisdiction("EU"),
        ApplicabilityRule::processes_personal_data(true),
    ]);
    let bytes = serde_json::to_vec(&rule).unwrap();
    let again: ApplicabilityRule = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rule, again);
    assert_eq!(
        canonical_digest(&rule).unwrap(),
        canonical_digest(&again).unwrap()
    );
}

#[test]
fn ir_011_subject_selectors_are_provider_neutral() {
    let selector = SubjectSelector {
        kind: SubjectKind::Repository,
        ids: BTreeSet::from(["asset:github:repo:xylex-group/athena".into()]),
        tags: Default::default(),
        scope: Default::default(),
    };
    let json = serde_json::to_value(&selector).unwrap();
    assert!(json.get("kind").is_some());
    let src = ir_sources();
    for forbidden in [
        "GithubRepositorySelector",
        "AwsIamRoleSelector",
        "CloudflareZoneSelector",
    ] {
        assert!(
            !src.contains(forbidden),
            "IR-011: IR must not define {forbidden}"
        );
    }
}

#[test]
fn ir_012_evidence_requirements_contain_no_collector_identity() {
    let ev = EvidenceRequirement::new(
        EvidenceRequirementId::new("ev.branch_protection"),
        EvidenceType::new("branch_protection"),
    )
    .with_cardinality(EvidenceCardinality::One)
    .with_collection(EvidenceCollectionKind::Automated)
    .with_criticality(EvidenceCriticality::Required);
    let json = serde_json::to_value(&ev).unwrap();
    assert!(json.get("collectorId").is_none());
    assert!(json.get("provider").is_none());
}

#[test]
fn ir_013_control_tests_contain_no_provider_identity() {
    let test = PlannedControlTest::new(
        ControlTestId::new("test.branch-protection"),
        ControlId::new("control.source.branch-protection"),
    )
    .with_kind(PlannedTestKind::Automated);
    let json = serde_json::to_value(&test).unwrap();
    assert!(json.get("collectorId").is_none());
    assert!(json.get("provider").is_none());
    let src = ir_sources();
    for forbidden in ["GitHubClient", "octocrab", "aws_sdk", "reqwest::"] {
        assert!(!src.contains(forbidden), "IR-013: {forbidden}");
    }
}

#[test]
fn ir_014_unknown_extensions_survive_round_trip() {
    let mut extensions = ExtensionMap::new();
    extensions.insert("user.note".into(), json!("keep-me"));
    let control = sample_control().with_extensions(extensions);
    let bytes = serde_json::to_vec(&control).unwrap();
    let back: Control = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        back.extensions().get("user.note"),
        Some(&json!("keep-me")),
        "IR-014"
    );
}

#[test]
fn ir_015_canonical_digest_is_deterministic() {
    let control = sample_control();
    let a = canonical_digest(&control).unwrap();
    let b = canonical_digest(&control).unwrap();
    assert_eq!(a, b, "IR-015");
}

#[test]
fn ir_016_digest_domain_separation_works() {
    #[derive(serde::Serialize)]
    struct Body {
        v: u8,
    }
    let body = Body { v: 1 };
    let control = typed_canonical_digest("control", &body).unwrap();
    let req = typed_canonical_digest("requirement", &body).unwrap();
    assert_ne!(control, req, "IR-016: domain prefix must change the digest");
    assert_eq!(CanonicalizationVersion::CURRENT.as_str(), "canon/v1");
}

#[test]
fn ir_017_duplicate_canonical_ids_fail_validation() {
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-017",
    ));
    assessment.controls.push(sample_control());
    assessment.controls.push(sample_control());
    assessment
        .validate()
        .expect_err("IR-017: duplicate control ids");
}

#[test]
fn ir_018_assessment_scope_is_deterministic() {
    let mut a = AssessmentScope::default();
    a.subjects.push(SubjectSelector {
        kind: SubjectKind::Organization,
        ids: BTreeSet::from(["org:xylex".into()]),
        tags: Default::default(),
        scope: Default::default(),
    });
    let mut b = a.clone();
    assert_eq!(canonical_digest(&a).unwrap(), canonical_digest(&b).unwrap());
    b.subjects[0].ids.insert("org:other".into());
    assert_ne!(canonical_digest(&a).unwrap(), canonical_digest(&b).unwrap());
}

#[test]
fn ir_019_risk_references_must_resolve() {
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-019",
    ));
    assessment.controls.push(sample_control());
    assessment.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    assessment.validate().expect_err("IR-019: dangling risk");
}

#[test]
fn ir_020_exception_references_must_resolve() {
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-020",
    ));
    assessment.controls.push(sample_control());
    assessment.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_exception(ExceptionId::new("exc:missing")),
    );
    assessment
        .validate()
        .expect_err("IR-020: dangling exception");
}

#[test]
fn ir_021_schema_version_mismatch_fails_closed() {
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-021",
    ));
    assessment.schema_version = "assurance-ir/v0".into();
    let err = assessment.validate().expect_err("IR-021");
    assert!(
        err.to_string().contains("schema") || err.to_string().contains("version"),
        "{err}"
    );
}

#[test]
fn ir_022_no_random_uuid_identities_appear_in_persisted_ir() {
    assert!(
        ControlId::try_new("550e8400-e29b-41d4-a716-446655440000").is_err(),
        "IR-022: UUIDv4 must not be a persisted IR id"
    );
}

#[test]
fn ir_023_mapping_version_ranges_are_respected() {
    let mapping = Mapping::new(
        RequirementId::new("req.iso27001.2022.a-5-15"),
        ControlId::new("control.access.mfa"),
        MappingDirection::Forward,
        MappingCompleteness::Full,
    )
    .with_valid_for(MappingVersionConstraint {
        from: Some(FrameworkVersion::new("2022")),
        to: Some(FrameworkVersion::new("2022")),
    });
    let mut assessment = AssessmentDefinition::new(weeping_angel_assurance_ir::AssessmentId::new(
        "assess.ir-023",
    ));
    assessment.requirements.push(sample_requirement());
    assessment.controls.push(sample_control());
    assessment.mappings.push(mapping);
    assessment.validate().expect("in-range mapping");

    assessment.mappings[0] = Mapping::new(
        RequirementId::new("req.iso27001.2022.a-5-15"),
        ControlId::new("control.access.mfa"),
        MappingDirection::Forward,
        MappingCompleteness::Full,
    )
    .with_valid_for(MappingVersionConstraint {
        from: Some(FrameworkVersion::new("2013")),
        to: Some(FrameworkVersion::new("2013")),
    });
    assessment
        .validate()
        .expect_err("IR-023: out-of-range mapping");
}

#[test]
fn ir_024_requirement_external_ids_are_not_used_as_internal_identity() {
    let req = sample_requirement().with_external_id("A.5.15");
    assert_eq!(req.id().as_str(), "req.iso27001.2022.a-5-15");
    assert_eq!(req.external_id(), Some("A.5.15"));
    assert_ne!(req.id().as_str(), req.external_id().unwrap());
}

#[test]
fn ir_025_framework_catalogs_compile_without_extending_control() {
    let _ = FrameworkRef {
        id: FrameworkId::new("iso-27001"),
        version: FrameworkVersion::new("2022"),
    };
    let control = sample_control();
    let json = serde_json::to_value(&control).unwrap();
    assert!(json.get("iso27001").is_none());
    assert_eq!(control.schema_version(), ASSURANCE_IR_SCHEMA);
}

#[test]
fn ir_golden_fixtures_round_trip() {
    let dir = manifest_dir().join("tests/fixtures/assurance-ir/v1");
    let control: Control =
        serde_json::from_str(&std::fs::read_to_string(dir.join("control.json")).unwrap()).unwrap();
    assert_eq!(control.id().as_str(), "control.access.mfa");
    let requirement: Requirement =
        serde_json::from_str(&std::fs::read_to_string(dir.join("requirement.json")).unwrap())
            .unwrap();
    assert_eq!(requirement.framework().id().as_str(), "iso-27001");
    let mapping: Mapping =
        serde_json::from_str(&std::fs::read_to_string(dir.join("mapping.json")).unwrap()).unwrap();
    assert_eq!(mapping.completeness(), MappingCompleteness::Partial);
    let impln: ControlImplementation = serde_json::from_str(
        &std::fs::read_to_string(dir.join("control-implementation.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(impln.status(), ImplementationStatus::Implemented);
    let risk: Risk =
        serde_json::from_str(&std::fs::read_to_string(dir.join("risk.json")).unwrap()).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    let exception: Exception =
        serde_json::from_str(&std::fs::read_to_string(dir.join("exception.json")).unwrap())
            .unwrap();
    assert_eq!(exception.id.as_str(), "exc:1");
    let activity: ProcessingActivity = serde_json::from_str(
        &std::fs::read_to_string(dir.join("processing-activity.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(activity.id.as_str(), "ropa:payroll");
    let assessment: AssessmentDefinition =
        serde_json::from_str(&std::fs::read_to_string(dir.join("assessment.json")).unwrap())
            .unwrap();
    assessment.validate().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(dir.join("control.json")).unwrap()
        )
        .unwrap(),
        serde_json::to_value(&control).unwrap()
    );
}

#[test]
fn dual_suite_is_registered() {
    assert_suite_in_harness("compliance_ir.target.rs");
}

fn collect_keys(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                out.insert(k.clone());
                collect_keys(v, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_keys(item, out);
            }
        }
        _ => {}
    }
}

// Types referenced so IR-009 names the runtime Effectiveness without importing it into IR.
#[allow(dead_code)]
fn _keep_control_test_dep() {
    let _ = std::any::TypeId::of::<weeping_angel_control_test::Effectiveness>();
}
