//! Baseline suite for subject population runtime (Prompt 03).
//!
//! SUPERSEDED by `sdd_population_runtime_target`. Placeholder CoverageAtLeast,
//! Count→NotTested, missing population arms, and no-metrics results are retired.
//! Characterization tests are `#[ignore]` so CI does not require the old
//! placeholder. Registration remains required-green.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssetId, ControlId, ControlTestId, Exception, ExceptionId, SubjectKind,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, CountPredicate, Effectiveness,
    EvidenceSelector, EvidenceSet, EvidenceValue, SubjectSelector, TestExpr, ValueExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

const PLACEHOLDER_RATIONALE: &str = "subject coverage remains partial unless the threshold is met";

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
    fs::read_to_string(crate_src(name).join(rel)).unwrap()
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn envelope(evidence_type: &str, asset: &str, field: &str, value: &str) -> EvidenceEnvelope {
    let obs = EvidenceObservation::new(EvidenceType::new(evidence_type)).with_fact(field, value);
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.population-baseline".into(),
            collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 11, 0, 0).unwrap(),
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn compiled(expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.source.branch-protection.coverage"))
        .control_id(ControlId::new("source.branch-protection"))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn coverage_expr(percentage: &str) -> TestExpr {
    TestExpr::CoverageAtLeast {
        selector: SubjectSelector {
            kind: Some("repository".into()),
            id: None,
        },
        evidence: EvidenceSelector {
            evidence_type: EvidenceType::new("source.branch.protection"),
            subject_selector: SubjectSelector {
                kind: Some("repository".into()),
                id: None,
            },
            field: Some("protected".into()),
            freshness: None,
        },
        percentage: percentage.into(),
    }
}

fn json_keys(result: &weeping_angel_control_test::ControlTestResult) -> Vec<String> {
    serde_json::to_value(result)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

#[test]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_population_runtime_baseline")
            && toml.contains("sdd_population_runtime_target"),
        "dual-suite must be listed in root Cargo.toml"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn coverage_at_least_is_placeholder_partially_effective() {
    let result = evaluate(
        &compiled(coverage_expr("100")),
        &EvidenceSet::new(),
        &fresh_context(),
    );
    assert_eq!(result.effectiveness, Effectiveness::PartiallyEffective);
    assert_eq!(result.rationale, PLACEHOLDER_RATIONALE);
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn coverage_at_least_ignores_selector_evidence_and_percentage() {
    let mut set = EvidenceSet::new();
    for i in 1..=50 {
        set.insert(envelope(
            "source.branch.protection",
            &format!("repo:{i:02}"),
            "protected",
            "true",
        ));
    }
    let full = evaluate(&compiled(coverage_expr("100")), &set, &fresh_context());
    let low = evaluate(&compiled(coverage_expr("0")), &set, &fresh_context());
    assert_eq!(full.effectiveness, Effectiveness::PartiallyEffective);
    assert_eq!(low.effectiveness, Effectiveness::PartiallyEffective);
    assert_eq!(full.rationale, PLACEHOLDER_RATIONALE);
    assert_eq!(low.rationale, PLACEHOLDER_RATIONALE);
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn count_exists_in_ast_but_evaluates_to_not_tested() {
    let expr_src = read_crate_file("weeping-angel-control-test", "expr.rs");
    assert!(
        expr_src.contains("Count {"),
        "Count is present in the AST today"
    );
    let mut set = EvidenceSet::new();
    set.insert(envelope(
        "source.branch.protection",
        "repo:01",
        "protected",
        "true",
    ));
    let result = evaluate(
        &compiled(TestExpr::Count {
            selector: EvidenceSelector::of_type(EvidenceType::new("source.branch.protection")),
            predicate: CountPredicate::Gte(1),
        }),
        &set,
        &fresh_context(),
    );
    assert_eq!(result.effectiveness, Effectiveness::NotTested);
    assert!(
        result.rationale.contains("unsupported expression arm"),
        "Count falls through: {}",
        result.rationale
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn population_test_expr_arms_do_not_exist() {
    let src = read_crate_file("weeping-angel-control-test", "expr.rs");
    for arm in [
        "CountWhere",
        "AllSubjects",
        "AnySubject",
        "NoneSubjects",
        "CoverageExactly",
        "MissingSubjects",
    ] {
        assert!(
            !src.contains(arm),
            "baseline: {arm} must not exist on current TestExpr"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn control_test_result_has_no_population_metrics() {
    let result = evaluate(
        &compiled(coverage_expr("95")),
        &EvidenceSet::new(),
        &fresh_context(),
    );
    let keys = json_keys(&result);
    for forbidden in [
        "population",
        "evaluated",
        "passing",
        "failing",
        "missing",
        "coverage",
        "failingSubjects",
        "missingSubjects",
        "populationEvaluation",
        "coverageBreakdown",
        "staleSubjects",
    ] {
        assert!(
            !keys.iter().any(|k| k == forbidden),
            "baseline ControlTestResult unexpectedly has `{forbidden}` (keys={keys:?})"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn evidence_set_is_digest_map_and_first_selector_is_linear() {
    let lib = read_crate_file("weeping-angel-control-test", "lib.rs");
    assert!(
        lib.contains("envelopes: BTreeMap<String, EvidenceEnvelope>"),
        "EvidenceSet is a digest-keyed BTreeMap"
    );
    assert!(lib.contains("fn first_selector"), "first_selector exists");
    assert!(
        lib.contains("envelopes.iter().copied().find"),
        "first_selector is a linear scan"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn evaluate_compiled_never_attaches_test_expr() {
    let src = crate_sources_joined("weeping-angel-assurance");
    assert!(
        src.contains("fn evaluate_compiled"),
        "facade evaluate_compiled must exist"
    );
    let start = src.find("fn evaluate_compiled").unwrap();
    let body = &src[start..start + 1200.min(src.len() - start)];
    assert!(
        !body.contains(".expr("),
        "evaluate_compiled currently does not attach TestExpr"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn compiled_test_has_no_expression_field() {
    let src = crate_sources_joined("weeping-angel-framework");
    let start = src.find("pub struct CompiledTest").expect("CompiledTest");
    let body = &src[start..start + 400.min(src.len() - start)];
    assert!(
        body.contains("pub required:") && body.contains("pub break_on:"),
        "CompiledTest is required/break_on only"
    );
    assert!(
        !body.contains("pub expr"),
        "CompiledTest currently has no expr field"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn two_subject_selector_types_exist() {
    let thin = SubjectSelector {
        kind: Some("repository".into()),
        id: Some("repo:01".into()),
    };
    let ir = weeping_angel_assurance_ir::SubjectSelector {
        kind: SubjectKind::Repository,
        ids: ["repo:01".into()].into_iter().collect(),
        tags: Default::default(),
        scope: Default::default(),
    };
    assert_eq!(thin.id.as_deref(), Some("repo:01"));
    assert!(ir.ids.contains("repo:01"));
    let ct_src = read_crate_file("weeping-angel-control-test", "expr.rs");
    assert!(
        ct_src.contains("pub struct SubjectSelector") && ct_src.contains("pub id: Option<String>"),
        "control-test still owns a thin {{kind,id}} SubjectSelector"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn subject_kind_is_the_pre_extension_set() {
    let src = read_crate_file("weeping-angel-assurance-ir", "subject.rs");
    for required in [
        "Organization",
        "Asset",
        "Repository",
        "Service",
        "Identity",
        "User",
        "PrivilegedIdentity",
        "Device",
        "Vendor",
        "Dataset",
        "ProcessingActivity",
    ] {
        assert!(src.contains(required), "SubjectKind missing {required}");
    }
    for absent in [
        "Branch",
        "Application",
        "Database",
        "CloudAccount",
        "CloudResource",
        "ServiceAccount",
        "Endpoint",
        "DataStore",
        "Network",
        "Deployment",
    ] {
        assert!(
            !src.contains(absent),
            "baseline SubjectKind must not yet list {absent}"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn exception_has_no_subject_binding() {
    let src = read_crate_file("weeping-angel-assurance-ir", "exception.rs");
    assert!(
        !src.contains("subjects") && !src.contains("applies_to"),
        "Exception currently has no subject binding"
    );
    let ex = Exception::new(ExceptionId::new("exc:baseline"), "timeboxed waiver");
    let json = serde_json::to_value(&ex).unwrap();
    assert!(
        json.get("subjects").is_none() && json.get("appliesTo").is_none(),
        "serialized Exception has no subjects: {json}"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn facade_assessment_scope_is_asset_allow_set() {
    let src = crate_sources_joined("weeping-angel-assurance");
    assert!(
        src.contains("allowed: std::collections::BTreeSet<AssetId>"),
        "facade AssessmentScope is an AssetId allow-set"
    );
    let _ =
        weeping_angel_assurance::AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope"));
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn planned_control_test_evaluation_is_id_ref() {
    let src = read_crate_file("weeping-angel-assurance-ir", "test.rs");
    assert!(
        src.contains("pub struct TestEvaluationRef") && src.contains("pub evaluation:"),
        "PlannedControlTest.evaluation is an id ref"
    );
    assert!(
        !src.contains("pub expr:"),
        "PlannedControlTest does not embed TestExpr"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn catalog_tree_has_not_landed() {
    assert!(
        !manifest_dir().join("catalog").is_dir(),
        "Prompt 01 catalog/ tree is not present on this SHA"
    );
}

#[test]
#[ignore = "superseded by sdd_population_runtime_target"]
fn contains_in_evaluate_as_typed_evidence_not_population() {
    // Prompt 02 landed Contains/In before this slice. They evaluate (missing
    // evidence → InsufficientEvidence) and must not be confused with the still-
    // unimplemented population arms / CoverageAtLeast placeholder.
    let sel = EvidenceSelector::of_type(EvidenceType::new("source.branch.protection"));
    let cases = [
        TestExpr::Contains(ValueExpr::Field(sel.clone()), EvidenceValue::string("x")),
        TestExpr::In(
            ValueExpr::Field(sel.clone()),
            vec![EvidenceValue::string("x")],
        ),
    ];
    for expr in cases {
        let result = evaluate(&compiled(expr), &EvidenceSet::new(), &fresh_context());
        assert_ne!(
            result.effectiveness,
            Effectiveness::NotTested,
            "Prompt 02 comparison arms evaluate; got {:?}",
            result.effectiveness
        );
        assert_eq!(result.effectiveness, Effectiveness::InsufficientEvidence);
    }
    let coverage = evaluate(
        &compiled(coverage_expr("100")),
        &EvidenceSet::new(),
        &fresh_context(),
    );
    assert_eq!(coverage.effectiveness, Effectiveness::PartiallyEffective);
    assert_eq!(coverage.rationale, PLACEHOLDER_RATIONALE);
}
