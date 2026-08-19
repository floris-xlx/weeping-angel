//! SUPERSEDED by `sdd_security_objectives_target`.
//!
//! Historical characterization of prose-only objectives on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (`docs/specs/security-objectives.md` §3).
//! Product crates have no `SecurityObjective` / `ObjectiveMetric` /
//! `ObjectiveTarget` / `ObjectiveMeasurement` (or ids / evaluator).
//! `Control.objective` is a prose `String`; the catalog copies the same
//! field. `control.governance.security-objectives` is manual-review over
//! `evidence.manual.attestation`. `EvidenceValue`, `EvidenceSnapshot`, IR
//! `AssessmentScope` / `SubjectSelector`, and `PrincipalRef` exist but are
//! unused for objectives. `IsmsContext` and `ScopeResolution` are absent
//! unless a concurrent slice lands first. Collectors emit facts only;
//! `Effectiveness` is a control outcome, not objective status.
//!
//! Tests are `#[ignore]` so CI does not require the old absence
//! characterization. Dual-suite registration remains.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel::contract::types::ThreatModelSummary;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentScope as IrAssessmentScope,
    Control, ControlId, EvidenceCollectionKind, EvidenceRequirement, EvidenceRequirementId,
    EvidenceType, FreshnessRequirement, PrincipalRef, ValidateIr,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::Effectiveness;
use weeping_angel_evidence::{EvidenceValue, looks_like_compliance_claim};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
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

fn product_declares_ident(src: &str, name: &str) -> bool {
    src.contains(&format!("struct {name}"))
        || src.contains(&format!("enum {name}"))
        || src.contains(&format!("type {name} "))
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load")
}

/// SO: product crate sources have no SecurityObjective / ObjectiveMetric / ObjectiveTarget / ObjectiveMeasurement
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b01_product_crates_have_no_objective_record_types() {
    let product = product_crate_sources_joined();
    for needle in [
        "struct SecurityObjective",
        "struct ObjectiveMetric",
        "struct ObjectiveTarget",
        "struct ObjectiveMeasurement",
        "enum ObjectiveLifecycle",
        "enum MetricKind",
        "enum ComparisonOp",
        "ObjectiveEvaluationSnapshot",
    ] {
        assert!(
            !product.contains(needle),
            "found-case: product crates must not contain `{needle}`"
        );
    }
}

/// SO: id.rs / IR lib.rs do not define SecurityObjectiveId or an objectives module
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b02_ir_has_no_objective_ids_or_module() {
    let id_src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    for needle in [
        "SecurityObjectiveId",
        "ObjectiveMetricId",
        "ObjectiveTargetId",
        "ObjectiveMeasurementId",
    ] {
        assert!(
            !id_src.contains(needle),
            "found-case: id.rs currently has no `{needle}`"
        );
    }

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !lib.contains("pub mod objectives") && !lib.contains("SecurityObjective"),
        "lib.rs currently does not re-export a security-objectives module"
    );
    assert!(
        !manifest_dir()
            .join("crates/weeping-angel-assurance-ir/src/objectives.rs")
            .exists(),
        "found-case: IR objectives.rs module file is absent"
    );
    assert!(
        !manifest_dir()
            .join("crates/weeping-angel-assurance/src/objectives.rs")
            .exists(),
        "found-case: assurance objectives.rs evaluator module is absent"
    );
}

/// SO: Control.objective is still String; with_objective exists; empty default
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b03_control_objective_is_prose_string() {
    let control = Control::new(
        ControlId::new("control.governance.security-objectives"),
        "Security objectives",
        "Security objectives are recorded as an attestation, not inferred from a score.",
    );
    assert_eq!(control.schema_version(), ASSURANCE_IR_SCHEMA);
    assert_eq!(control.objective(), "");
    let empty_json = serde_json::to_value(&control).unwrap();
    assert!(
        empty_json.get("objective").is_none(),
        "empty Control.objective is skipped on serialize"
    );

    let with_prose = control.with_objective("Require a fresh scan of in-scope repositories.");
    assert_eq!(
        with_prose.objective(),
        "Require a fresh scan of in-scope repositories."
    );
    let json = serde_json::to_value(&with_prose).unwrap();
    assert_eq!(
        json.get("objective").and_then(Value::as_str),
        Some("Require a fresh scan of in-scope repositories.")
    );
    assert!(
        json.get("metricId").is_none()
            && json.get("targetId").is_none()
            && json.get("lifecycle").is_none(),
        "Control JSON is prose, not a SecurityObjective record"
    );

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/control.rs");
    assert!(
        src.contains("objective: String"),
        "Control.objective remains a String field"
    );
    assert!(
        src.contains("pub fn with_objective"),
        "Control::with_objective remains the prose setter"
    );
}

/// SO: control.governance.security-objectives remains manual + objectives-attested manual-review
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b04_governance_catalog_attests_existence_not_measurement() {
    let catalog = load_catalog();
    let control = catalog
        .control("control.governance.security-objectives")
        .expect("governance security-objectives control exists today");
    assert_eq!(control.title, "Security objectives");
    assert_eq!(
        control.description,
        "Security objectives are recorded as an attestation, not inferred from a score."
    );
    assert_eq!(
        control.objective,
        "Require an attested objective set for the in-scope organization."
    );
    assert_eq!(control.automation, "manual");
    assert_eq!(control.evidence, ["evidence.manual.attestation"]);
    assert_eq!(control.tests, ["test.governance.objectives-attested"]);

    let test = catalog
        .tests()
        .get("test.governance.objectives-attested")
        .expect("objectives-attested catalog test exists today");
    assert_eq!(test.control, "control.governance.security-objectives");
    assert_eq!(test.kind, "manual");
    assert_eq!(test.required_evidence, ["evidence.manual.attestation"]);
    assert_eq!(
        test.expression.get("op").and_then(toml::Value::as_str),
        Some("manual-review")
    );

    let controls_toml = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(
        controls_toml.contains("id = \"control.governance.security-objectives\""),
        "governance TOML still lists the attestation control"
    );
    let tests_toml = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(
        tests_toml.contains("id = \"test.governance.objectives-attested\""),
        "governance tests TOML still lists objectives-attested"
    );
    assert!(
        tests_toml.contains("op = \"manual-review\""),
        "objectives-attested remains manual-review"
    );
}

/// SO: no evaluate_objective / ObjectiveStatus / OnTrack projection in crates
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b05_no_objective_status_projection() {
    let product = product_crate_sources_joined();
    for needle in [
        "fn evaluate_objective",
        "enum ObjectiveStatus",
        "struct ObjectiveEvaluation",
        "ObjectiveEvaluationSnapshot",
        "weeping-angel/objective-evaluation/v1",
    ] {
        assert!(
            !product.contains(needle),
            "found-case: product crates must not contain `{needle}`"
        );
    }
    assert!(
        !product.contains("OnTrack") && !product.contains("onTrack"),
        "found-case: no OnTrack objective-status variant in product crates"
    );

    match Effectiveness::Effective {
        Effectiveness::Effective
        | Effectiveness::Ineffective
        | Effectiveness::PartiallyEffective
        | Effectiveness::NotApplicable
        | Effectiveness::NotTested
        | Effectiveness::InsufficientEvidence
        | Effectiveness::StaleEvidence
        | Effectiveness::ManualReviewRequired
        | Effectiveness::ExceptionApproved
        | Effectiveness::Inconclusive => {}
    }
}

/// SO: collectors have no objective-status types
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b06_collectors_emit_facts_only() {
    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "ObjectiveStatus",
        "evaluate_objective",
        "SecurityObjective",
        "OnTrack",
        "struct ObjectiveMeasurement",
    ] {
        assert!(
            !collector.contains(needle),
            "found-case: collector sources must not contain `{needle}`"
        );
    }
    assert!(
        looks_like_compliance_claim("ISO 27001 compliant"),
        "seal still treats compliance narratives as claims, not objective status"
    );
    assert!(
        !looks_like_compliance_claim("critical vulnerabilities remediated within seven days"),
        "metric prose is not itself a sealed compliance claim"
    );
}

/// SO: IsmsContext and ScopeResolution remain absent unless a concurrent slice landed
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b07_neighbor_context_and_scope_engine_absent_or_orthogonal() {
    let product = product_crate_sources_joined();
    if !product_declares_ident(&product, "IsmsContext") {
        assert!(
            !product.contains("IsmsContext"),
            "found-case: IsmsContext is absent on this HEAD"
        );
    }
    if !product_declares_ident(&product, "ScopeResolution") {
        assert!(
            !product.contains("ScopeResolution"),
            "found-case: ScopeResolution is absent on this HEAD"
        );
    }

    assert!(
        !product_declares_ident(&product, "SecurityObjective"),
        "security objectives remain prose-only even if a neighbor slice landed"
    );

    let ir_lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !ir_lib.contains("pub mod objectives"),
        "IR still has no objectives module"
    );
}

/// SO: security-objectives spec path exists under docs/specs
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b08_security_objectives_spec_exists() {
    let path = manifest_dir().join("docs/specs/security-objectives.md");
    assert!(
        path.is_file(),
        "found-case: docs/specs/security-objectives.md must exist"
    );
    let body = fs::read_to_string(&path).unwrap();
    assert!(
        body.contains("Security Objectives Engine"),
        "spec names the security objectives slice"
    );
}

/// SO: AssessmentDefinition inventories have no objectives list
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b09_assessment_definition_has_no_objectives_inventory() {
    let assessment =
        AssessmentDefinition::new(AssessmentId::new("assess.security-objectives.baseline"));
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assert!(assessment.requirements.is_empty());
    assert!(assessment.controls.is_empty());
    assert!(assessment.mappings.is_empty());
    assert!(assessment.evidence_requirements.is_empty());
    assert!(assessment.tests.is_empty());
    assert!(assessment.implementations.is_empty());
    assert!(assessment.scope.organizations.is_empty());
    assert!(assessment.scope.subjects.is_empty());
    assert!(assessment.scope.exclusions.is_empty());
    assert!(assessment.assets.is_empty());
    assert!(assessment.identities.is_empty());
    assert!(assessment.vendors.is_empty());
    assert!(assessment.risks.is_empty());
    assert!(assessment.exceptions.is_empty());
    assert!(assessment.processing_activities.is_empty());
    assessment
        .validate()
        .expect("empty AssessmentDefinition::new must validate today");

    let json = serde_json::to_value(&assessment).unwrap();
    let obj = json
        .as_object()
        .expect("AssessmentDefinition serializes as an object");
    assert!(
        obj.get("objectives").is_none() && obj.get("securityObjectives").is_none(),
        "found-case: AssessmentDefinition JSON has no objectives inventory"
    );

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !src.contains("objectives"),
        "AssessmentDefinition currently has no objectives field"
    );
}

/// SO: EvidenceValue comparison exists and is unused as an objective metric type
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b10_evidence_value_thresholds_are_not_objective_status() {
    let measured = EvidenceValue::integer(98);
    let target = EvidenceValue::decimal("98.0").expect("canonical decimal 98.0");
    assert_eq!(
        measured.cmp_numeric(&target).expect("integer↔decimal"),
        Ordering::Equal
    );
    let below = EvidenceValue::decimal("97.999").expect("canonical decimal 97.999");
    assert_eq!(
        measured.cmp_numeric(&below).expect("98 vs 97.999"),
        Ordering::Greater
    );
    assert!(
        EvidenceValue::from_bool(true)
            .cmp_numeric(&EvidenceValue::integer(1))
            .is_err(),
        "type mismatch is an error string, not a silent coerce"
    );
    assert!(
        EvidenceValue::from_bool(true)
            .typed_eq(&EvidenceValue::from_bool(true))
            .expect("bool typed_eq"),
        "boolean equality is typed_eq, not an objective Achieved"
    );

    let product = product_crate_sources_joined();
    assert!(
        !product.contains("fn evaluate_objective"),
        "98% vs 97% is not projected to OnTrack/AtRisk today"
    );
}

/// SO: missing measurements are not success because no evaluator maps absence to OnTrack
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b11_missing_data_is_not_objective_success() {
    let assessment =
        AssessmentDefinition::new(AssessmentId::new("assess.security-objectives.missing"));
    assert!(
        assessment.tests.is_empty() && assessment.controls.is_empty(),
        "no stored measurements or objective evaluations on a new assessment"
    );
    assert!(
        !matches!(
            Effectiveness::InsufficientEvidence,
            Effectiveness::Effective
        ),
        "control InsufficientEvidence is not treated as Effective"
    );
    assert!(
        !matches!(Effectiveness::NotTested, Effectiveness::Effective),
        "untested controls are not success"
    );
}

/// SO: stale evidence is a control Effectiveness variant, not an objective stale rule
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b12_stale_measurement_is_control_effectiveness_only() {
    let freshness = FreshnessRequirement {
        max_age_seconds: 7 * 24 * 3600,
    };
    assert_eq!(freshness.max_age_seconds, 604_800);
    let req = EvidenceRequirement::new(
        EvidenceRequirementId::new("evidence.manual.attestation"),
        EvidenceType::new("evidence.manual.attestation"),
    );
    let _ = req;
    let _kind = EvidenceCollectionKind::Manual;

    match Effectiveness::StaleEvidence {
        Effectiveness::StaleEvidence => {}
        other => panic!("StaleEvidence is a control outcome, got {other:?}"),
    }
    let product = product_crate_sources_joined();
    assert!(
        !product.contains("staleMeasurement") && !product.contains("fn evaluate_objective"),
        "no objective-level staleMeasurement reason code today"
    );
}

/// SO: mixed manual/automated objectives do not exist; governance row is manual only
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b13_mixed_manual_automated_objectives_absent() {
    let catalog = load_catalog();
    let control = catalog
        .control("control.governance.security-objectives")
        .unwrap();
    assert_eq!(control.automation, "manual");
    let test = catalog
        .tests()
        .get("test.governance.objectives-attested")
        .unwrap();
    assert_eq!(test.kind, "manual");
    assert_eq!(
        test.expression.get("op").and_then(toml::Value::as_str),
        Some("manual-review")
    );
    let product = product_crate_sources_joined();
    assert!(
        !product.contains("missingAttestation"),
        "no manual-objective attestation requirement code today"
    );
}

/// SO: scope is assessment-shaped; no measurement population for an objective
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b14_scope_is_not_an_objective_measurement_binding() {
    let ir_scope = IrAssessmentScope::default();
    assert!(ir_scope.organizations.is_empty());
    assert!(ir_scope.subjects.is_empty());
    assert!(ir_scope.exclusions.is_empty());

    let facade = weeping_angel_assurance::AssessmentScope::new();
    let _ = facade.describe();

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        src.contains("pub struct AssessmentScope"),
        "IR AssessmentScope remains the assessment document scope"
    );
    assert!(
        !src.contains("scopeMismatch"),
        "no objective scopeMismatch reason on IR AssessmentScope"
    );
}

/// SO: historical EvidenceSnapshot exists for control tests, not objective lineage
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b15_historical_lineage_is_control_test_snapshots() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        lineage.contains("pub struct EvidenceSnapshot"),
        "EvidenceSnapshot already exists for control-test lineage"
    );
    assert!(
        lineage.contains("pub struct LineageBundle"),
        "LineageBundle already exists for control-test lineage"
    );
    assert!(
        !lineage.contains("ObjectiveEvaluationSnapshot")
            && !lineage.contains("objective-evaluation"),
        "lineage does not store how an objective status was produced"
    );
}

/// SO: no deterministic objective status transitions
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b16_no_deterministic_objective_status_transitions() {
    let product = product_crate_sources_joined();
    for needle in ["OnTrack", "AtRisk", "Missed"] {
        assert!(
            !product.contains(needle),
            "found-case: no `{needle}` objective transition in product crates"
        );
    }
    assert!(
        !product.contains("fn evaluate_objective"),
        "same inputs cannot yield a replayable ObjectiveEvaluation today"
    );
}

/// SO: scanner ThreatModelSummary.security_objectives is optional prose, not the ISMS engine
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b17_scanner_security_objectives_field_is_prose_strings() {
    let summary = ThreatModelSummary {
        summary: None,
        assets: None,
        trust_boundaries: None,
        attacker_capabilities: None,
        security_objectives: Some(vec![
            "critical vulnerabilities remediated within seven days".into(),
        ]),
        assumptions: None,
    };
    assert_eq!(
        summary.security_objectives.as_ref().map(Vec::as_slice),
        Some(["critical vulnerabilities remediated within seven days".to_string()].as_slice())
    );
    let json = serde_json::to_value(&summary).unwrap();
    assert!(
        json.get("securityObjectives")
            .and_then(Value::as_array)
            .is_some(),
        "scanner contract serializes camelCase securityObjectives strings"
    );
    assert!(
        json.get("status").is_none(),
        "scanner threat-model objectives carry no evaluation status"
    );
}

/// SO: PrincipalRef and catalog objective prose are unused as first-class objective owners
#[test]
#[ignore = "superseded by sdd_security_objectives_target"]
fn so_b18_owner_and_catalog_prose_are_not_objective_records() {
    let _owner = PrincipalRef::Team("security".into());
    let catalog = load_catalog();
    let vuln_with_prose = catalog
        .controls()
        .values()
        .find(|c| !c.objective.is_empty())
        .expect("canonical catalog copies English objective sentences");
    assert!(
        !vuln_with_prose.objective.contains("OnTrack")
            && !vuln_with_prose
                .objective
                .chars()
                .all(|c| c.is_ascii_digit()),
        "catalog objective field is English prose, not a compared metric"
    );
}
