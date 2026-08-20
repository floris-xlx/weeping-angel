//! Target suite for subject population runtime.
//!
//! Encodes DESIRED semantics in `docs/specs/population-runtime.md` §4–§7.
//! Must stay RED on current placeholder / linear / no-population code.
//! Do not `#[ignore]` these tests and do not weaken them to match today's evaluator.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssetId, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus, IdentityKind,
    SubjectKind,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, CountPredicate, Effectiveness,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

const PLACEHOLDER_RATIONALE: &str = "subject coverage remains partial unless the threshold is met";

const POPULATION_ARMS: &[&str] = &[
    "CountWhere",
    "AllSubjects",
    "AnySubject",
    "NoneSubjects",
    "CoverageExactly",
    "MissingSubjects",
];

const REQUIRED_SUBJECT_KINDS: &[&str] = &[
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
];

const CONCEPTUAL_KINDS: &[&str] = &[
    "organization",
    "repository",
    "branch",
    "application",
    "service",
    "database",
    "cloud account",
    "cloud resource",
    "identity",
    "privileged identity",
    "service account",
    "endpoint",
    "vendor",
    "data store",
    "processing activity",
    "network",
    "deployment",
];

const HANDOFF_SENTENCES: &[&str] = &[
    "all privileged identities have MFA",
    "100% of non-archived repositories protect default branch",
    "no critical vulnerability exceeds SLA",
    "at least 95% of endpoints report encryption enabled",
];

const PROVIDER_TOKENS: &[&str] = &[
    "GithubRepository",
    "AwsAccountSelector",
    "AzureSubscription",
    "GcpProjectSelector",
    "OktaUser",
];

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
}

fn read_crate_file(name: &str, rel: &str) -> String {
    fs::read_to_string(crate_src(name).join(rel)).unwrap()
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let found: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(n))
        .collect();
    assert!(
        found.is_empty(),
        "{label}: provider/ISO/org-graph surface must not appear ({found:?})"
    );
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn collected(hours_ago: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap() - chrono::Duration::hours(hours_ago)
}

fn seal(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, &str)],
    at: DateTime<Utc>,
) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.population-target".into(),
            collected_at: at,
            scope: "target".into(),
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

fn thin_selector(kind: &str) -> SubjectSelector {
    SubjectSelector {
        kind: Some(kind.into()),
        id: None,
    }
}

fn evidence_sel(evidence_type: &str, kind: &str, field: &str) -> EvidenceSelector {
    EvidenceSelector {
        evidence_type: EvidenceType::new(evidence_type),
        subject_selector: thin_selector(kind),
        field: Some(field.into()),
        freshness: None,
    }
}

fn coverage_at_least(kind: &str, evidence_type: &str, field: &str, percentage: &str) -> TestExpr {
    TestExpr::CoverageAtLeast {
        selector: thin_selector(kind),
        evidence: evidence_sel(evidence_type, kind, field),
        percentage: percentage.into(),
    }
}

fn coverage_expr(percentage: &str) -> TestExpr {
    coverage_at_least(
        "repository",
        "source.branch.protection",
        "protected",
        percentage,
    )
}

fn authoritative_inventory(set: &mut EvidenceSet, n: usize) {
    set.insert(seal(
        "inventory.complete",
        "org:fixture",
        &[("kind", "repository"), ("authoritative", "true")],
        collected(1),
    ));
    for i in 1..=n {
        let id = format!("repo:{i:02}");
        set.insert(seal(
            "inventory.subject",
            &id,
            &[("kind", "repository"), ("id", &id)],
            collected(1),
        ));
    }
}

fn protection(set: &mut EvidenceSet, i: usize, protected: bool, hours_ago: i64) {
    let id = format!("repo:{i:02}");
    set.insert(seal(
        "source.branch.protection",
        &id,
        &[("protected", if protected { "true" } else { "false" })],
        collected(hours_ago),
    ));
}

fn scale_population(subjects: usize) -> EvidenceSet {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, subjects);
    for i in 1..=subjects {
        protection(&mut set, i, true, 1);
    }
    set
}

fn u64_metric(root: &Value, key: &str) -> Option<u64> {
    let nested = [
        root.get(key),
        root.get("populationEvaluation").and_then(|p| p.get(key)),
        root.get("coverageBreakdown").and_then(|p| p.get(key)),
        root.get("population").and_then(|p| p.get(key)),
    ];
    for cell in nested.into_iter().flatten() {
        if let Some(n) = cell.as_u64() {
            return Some(n);
        }
        if let Some(n) = cell.as_i64() {
            return Some(n as u64);
        }
        if key == "population" {
            if let Some(n) = cell.get("size").and_then(|s| s.as_u64()) {
                return Some(n);
            }
            if let Some(n) = cell.get("population").and_then(|s| s.as_u64()) {
                return Some(n);
            }
            if let Some(ids) = cell.get("subjectIds").and_then(|s| s.as_array()) {
                return Some(ids.len() as u64);
            }
        }
    }
    None
}

fn f64_metric(root: &Value, key: &str) -> Option<f64> {
    let nested = [
        root.get(key),
        root.get("populationEvaluation").and_then(|p| p.get(key)),
        root.get("coverageBreakdown").and_then(|p| p.get(key)),
        root.get("population").and_then(|p| p.get(key)),
    ];
    for cell in nested.into_iter().flatten() {
        if let Some(n) = cell.as_f64() {
            return Some(n);
        }
        if let Some(s) = cell.as_str()
            && let Ok(n) = s.parse::<f64>()
        {
            return Some(n);
        }
    }
    None
}

fn string_list(root: &Value, key: &str) -> Vec<String> {
    let nested = [
        root.get(key),
        root.get("populationEvaluation").and_then(|p| p.get(key)),
        root.get("coverageBreakdown").and_then(|p| p.get(key)),
        root.get("population").and_then(|p| p.get(key)),
    ];
    for cell in nested.into_iter().flatten() {
        if let Some(arr) = cell.as_array() {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect();
        }
    }
    Vec::new()
}

fn result_json(
    expr: TestExpr,
    set: &EvidenceSet,
) -> (weeping_angel_control_test::ControlTestResult, Value) {
    let result = evaluate(&compiled(expr), set, &fresh_context());
    let json = serde_json::to_value(&result).unwrap();
    (result, json)
}

fn assert_population_metrics(
    json: &Value,
    population: u64,
    evaluated: u64,
    passing: u64,
    failing: u64,
    missing: u64,
) {
    assert_eq!(u64_metric(json, "population"), Some(population), "{json}");
    assert_eq!(u64_metric(json, "evaluated"), Some(evaluated), "{json}");
    assert_eq!(u64_metric(json, "passing"), Some(passing), "{json}");
    assert_eq!(u64_metric(json, "failing"), Some(failing), "{json}");
    assert_eq!(u64_metric(json, "missing"), Some(missing), "{json}");
}

fn parse_expr(json: &str) -> TestExpr {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("TestExpr must deserialize: {e}; {json}"))
}

fn population_arm_json(arm: &str, percentage: Option<&str>) -> String {
    let selector = r#"{"kind":"repository"}"#;
    let evidence = r#"{
        "evidenceType":"source.branch.protection",
        "subjectSelector":{"kind":"repository"},
        "field":"protected"
    }"#;
    match arm {
        "CountWhere" => format!(
            r#"{{"CountWhere":{{"selector":{selector},"evidence":{evidence},"predicate":{{"gte":1}}}}}}"#
        ),
        "AllSubjects" | "AnySubject" | "NoneSubjects" | "MissingSubjects" => {
            format!(r#"{{"{arm}":{{"selector":{selector},"evidence":{evidence}}}}}"#)
        }
        "CoverageExactly" => format!(
            r#"{{"CoverageExactly":{{"selector":{selector},"evidence":{evidence},"percentage":"{}"}}}}"#,
            percentage.unwrap_or("100")
        ),
        other => panic!("unknown arm {other}"),
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[test]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !toml.contains("sdd_population_runtime_baseline")
            && harness_src().contains("population_runtime.target.rs")
            && !toml.contains("tests/contracts/population_runtime.baseline.rs")
            && harness_src().contains("population_runtime.target.rs")
    );
}

// ---------------------------------------------------------------------------
// CoverageAtLeast is real (no placeholder)
// ---------------------------------------------------------------------------

#[test]
fn coverage_at_least_is_no_longer_a_placeholder() {
    let (result, _) = result_json(coverage_expr("100"), &EvidenceSet::new());
    assert_ne!(
        result.rationale, PLACEHOLDER_RATIONALE,
        "CoverageAtLeast must not return the placeholder rationale"
    );
    assert_ne!(
        result.effectiveness,
        Effectiveness::PartiallyEffective,
        "empty/zero population must not keep the placeholder PartiallyEffective"
    );
}

#[test]
fn coverage_at_least_honors_percentage_and_evidence() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=50 {
        protection(&mut set, i, true, 1);
    }
    let full = evaluate(&compiled(coverage_expr("100")), &set, &fresh_context());
    let zero = evaluate(&compiled(coverage_expr("0")), &set, &fresh_context());
    assert_eq!(full.effectiveness, Effectiveness::Effective);
    assert_ne!(full.rationale, PLACEHOLDER_RATIONALE);
    assert_ne!(
        zero.effectiveness,
        Effectiveness::PartiallyEffective,
        "percentage 0 must not keep the placeholder"
    );
}

// ---------------------------------------------------------------------------
// Ten golden tests
// ---------------------------------------------------------------------------

#[test]
fn golden_50_of_50_passing() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=50 {
        protection(&mut set, i, true, 1);
    }
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_population_metrics(&json, 50, 50, 50, 0, 0);
    let coverage = f64_metric(&json, "coverage").expect("coverage ratio");
    assert!((coverage - 1.0).abs() < 1e-9, "coverage={coverage}");
}

#[test]
fn golden_47_of_50_with_three_failures() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=47 {
        protection(&mut set, i, true, 1);
    }
    for i in 48..=50 {
        protection(&mut set, i, false, 1);
    }
    let (full, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(full.effectiveness, Effectiveness::Ineffective);
    assert_population_metrics(&json, 50, 50, 47, 3, 0);
    assert_eq!(
        string_list(&json, "failingSubjects"),
        vec![
            "repo:48".to_string(),
            "repo:49".to_string(),
            "repo:50".to_string()
        ]
    );

    let (threshold, threshold_json) = result_json(coverage_expr("90"), &set);
    assert!(
        matches!(
            threshold.effectiveness,
            Effectiveness::Effective | Effectiveness::PartiallyEffective
        ),
        "CoverageAtLeast(90) must pass a 47/50 explicit-fail population, got {:?}",
        threshold.effectiveness
    );
    assert_eq!(
        string_list(&threshold_json, "failingSubjects"),
        vec![
            "repo:48".to_string(),
            "repo:49".to_string(),
            "repo:50".to_string()
        ],
        "threshold pass must still list failing subjects"
    );
}

#[test]
fn golden_47_pass_2_fail_1_missing() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=47 {
        protection(&mut set, i, true, 1);
    }
    protection(&mut set, 48, false, 1);
    protection(&mut set, 49, false, 1);
    let (result, json) = result_json(coverage_expr("95"), &set);
    assert_population_metrics(&json, 50, 49, 47, 2, 1);
    let coverage = f64_metric(&json, "coverage").expect("coverage");
    assert!(
        (coverage - 0.98).abs() < 1e-9,
        "coverage must be 49/50=0.98, not a hidden 47/50 pass rate; got {coverage}"
    );
    assert_eq!(string_list(&json, "missingSubjects"), vec!["repo:50"]);
    assert_eq!(
        string_list(&json, "failingSubjects"),
        vec!["repo:48".to_string(), "repo:49".to_string()]
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "pessimistic 47/50 < 95% <= optimistic 48/50 must be InsufficientEvidence, got {:?}",
        result.effectiveness
    );
}

#[test]
fn golden_unknown_incomplete_population() {
    let mut set = EvidenceSet::new();
    for i in 1..=47 {
        protection(&mut set, i, true, 1);
    }
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Inconclusive,
        "observations without an authoritative population must not yield a strong conclusion"
    );
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert!(
        f64_metric(&json, "coverage").is_none() || u64_metric(&json, "population").is_none(),
        "unknown population must not emit a fake coverage ratio over the observed set"
    );
}

#[test]
fn golden_partial_inventory_is_not_authoritative() {
    let mut set = EvidenceSet::new();
    for i in 1..=10 {
        let id = format!("repo:{i:02}");
        set.insert(seal(
            "inventory.subject",
            &id,
            &[("kind", "repository"), ("id", &id)],
            collected(1),
        ));
        protection(&mut set, i, true, 1);
    }
    let (result, _) = result_json(coverage_expr("100"), &set);
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "inventory.subject without inventory.complete is Partial — not a strong all-subject Effective"
    );
    assert!(
        matches!(
            result.effectiveness,
            Effectiveness::Inconclusive | Effectiveness::InsufficientEvidence
        ),
        "partial completeness cannot yield strong all-subject Effective, got {:?}",
        result.effectiveness
    );
}

#[test]
fn golden_stale_evidence_on_subset() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=49 {
        protection(&mut set, i, true, 1);
    }
    protection(&mut set, 50, true, 48);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::StaleEvidence,
        "stale subset that decides 100% coverage must be StaleEvidence, not missing/fail"
    );
    assert_eq!(u64_metric(&json, "missing").unwrap_or(0), 0);
    assert_eq!(u64_metric(&json, "failing").unwrap_or(0), 0);
    let stale = string_list(&json, "staleSubjects");
    assert!(
        stale.is_empty() || stale == ["repo:50"],
        "stale subject must be listed when exposed; got {stale:?}"
    );
}

#[test]
fn golden_exceptions_on_subset() {
    let src = read_crate_file("weeping-angel-assurance-ir", "exception.rs");
    require_needles(
        "P?: exception subject binding",
        &src,
        &["subjects", "ExceptionStatus"],
    );
    let mut ex = Exception::new(ExceptionId::new("exc:repo-50"), "approved waiver");
    ex.status = ExceptionStatus::Approved;
    let json = serde_json::to_value(&ex).unwrap();
    assert!(
        json.get("subjects").is_some() || json.get("appliesTo").is_some(),
        "Exception must serialize a subject binding so selected subjects can be excepted"
    );
    let ctx_src = crate_sources_joined("weeping-angel-control-test");
    require_needles(
        "P?: evaluator consumes bound exceptions",
        &ctx_src,
        &["exceptions", "excepted"],
    );
    assert!(
        ctx_src.contains("exceptedSubjects") || ctx_src.contains("excepted"),
        "excepted partition must be first-class, not folded into passing"
    );
}

#[test]
fn unbound_exception_does_not_silently_pass_population() {
    let src = read_crate_file("weeping-angel-assurance-ir", "exception.rs");
    require_needles("P?: Exception.subjects field", &src, &["pub subjects"]);
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 2);
    protection(&mut set, 1, false, 1);
    protection(&mut set, 2, false, 1);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "two explicit failures with no bound exception must stay Ineffective, got {:?}",
        result.effectiveness
    );
    assert_eq!(u64_metric(&json, "failing"), Some(2));
    assert_ne!(result.effectiveness, Effectiveness::ExceptionApproved);
    assert_ne!(result.effectiveness, Effectiveness::Effective);
}

#[test]
fn golden_zero_population_is_not_effective() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 0);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert_ne!(
        result.effectiveness,
        Effectiveness::PartiallyEffective,
        "zero population must not keep the placeholder partial"
    );
    assert!(
        matches!(
            result.effectiveness,
            Effectiveness::NotApplicable | Effectiveness::InsufficientEvidence
        ),
        "zero authoritative population → NotApplicable or InsufficientEvidence, got {:?}",
        result.effectiveness
    );
    assert_eq!(u64_metric(&json, "population").unwrap_or(0), 0);
}

#[test]
fn golden_duplicated_envelopes_do_not_inflate_counts() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 2);
    protection(&mut set, 1, true, 1);
    protection(&mut set, 2, true, 1);
    let dup = seal(
        "source.branch.protection",
        "repo:01",
        &[("protected", "true")],
        collected(1),
    );
    set.insert(dup);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "population"), Some(2));
    assert_eq!(u64_metric(&json, "passing"), Some(2));
    assert_eq!(u64_metric(&json, "evaluated"), Some(2));
}

#[test]
fn golden_latest_superseding_evidence_wins() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 1);
    protection(&mut set, 1, false, 3);
    let older_digest = set
        .iter()
        .find(|e| e.observation().evidence_type().as_str() == "source.branch.protection")
        .unwrap()
        .digest()
        .to_string();
    let newer = seal(
        "source.branch.protection",
        "repo:01",
        &[("protected", "true")],
        collected(1),
    )
    .with_supersedes(older_digest);
    set.insert(newer);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "passing"), Some(1));
    assert_eq!(u64_metric(&json, "failing").unwrap_or(0), 0);
    assert_eq!(u64_metric(&json, "evaluated"), Some(1));
}

#[test]
fn golden_latest_collected_at_wins_without_supersedes() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 1);
    protection(&mut set, 1, false, 3);
    set.insert(seal(
        "source.branch.protection",
        "repo:01",
        &[("protected", "true")],
        collected(1),
    ));
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "passing"), Some(1));
    assert_eq!(u64_metric(&json, "failing").unwrap_or(0), 0);
}

#[test]
fn golden_deterministic_subject_ordering() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 3);
    protection(&mut set, 3, false, 1);
    protection(&mut set, 1, false, 1);
    let (first, json) = result_json(coverage_expr("100"), &set);
    let again = evaluate(&compiled(coverage_expr("100")), &set, &fresh_context());
    assert_eq!(first.effectiveness, again.effectiveness);
    assert_eq!(first.rationale, again.rationale);
    assert_eq!(
        string_list(&json, "failingSubjects"),
        vec!["repo:01".to_string(), "repo:03".to_string()]
    );
    assert_eq!(string_list(&json, "missingSubjects"), vec!["repo:02"]);
}

// ---------------------------------------------------------------------------
// TestExpr arms evaluate for real
// ---------------------------------------------------------------------------

#[test]
fn count_evaluates_instead_of_not_tested() {
    let mut set = EvidenceSet::new();
    protection(&mut set, 1, true, 1);
    protection(&mut set, 2, true, 1);
    let result = evaluate(
        &compiled(TestExpr::Count {
            selector: EvidenceSelector::of_type(EvidenceType::new("source.branch.protection")),
            predicate: CountPredicate::Gte(2),
        }),
        &set,
        &fresh_context(),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "Count must evaluate; got {:?} ({})",
        result.effectiveness,
        result.rationale
    );
    assert!(!result.rationale.contains("unsupported expression arm"));
}

#[test]
fn population_arms_deserialize_and_evaluate() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 2);
    protection(&mut set, 1, true, 1);
    protection(&mut set, 2, true, 1);

    for arm in POPULATION_ARMS {
        let expr = parse_expr(&population_arm_json(arm, Some("100")));
        let result = evaluate(&compiled(expr), &set, &fresh_context());
        assert_ne!(
            result.effectiveness,
            Effectiveness::NotTested,
            "{arm} must not fall through to NotTested ({})",
            result.rationale
        );
        assert!(
            !result.rationale.contains("unsupported expression arm"),
            "{arm} must evaluate: {}",
            result.rationale
        );
    }
}

#[test]
fn coverage_exactly_uses_pessimistic_rate() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 50);
    for i in 1..=47 {
        protection(&mut set, i, true, 1);
    }
    for i in 48..=50 {
        protection(&mut set, i, false, 1);
    }
    let expr = parse_expr(&population_arm_json("CoverageExactly", Some("94")));
    let (result, json) = result_json(expr, &set);
    assert_eq!(u64_metric(&json, "passing"), Some(47));
    assert_eq!(u64_metric(&json, "failing"), Some(3));
    assert_ne!(result.effectiveness, Effectiveness::NotTested);
    assert_ne!(result.rationale, PLACEHOLDER_RATIONALE);
}

#[test]
fn missing_subjects_arm_is_effective_only_when_none_missing() {
    let mut complete = EvidenceSet::new();
    authoritative_inventory(&mut complete, 2);
    protection(&mut complete, 1, true, 1);
    protection(&mut complete, 2, true, 1);
    let expr = parse_expr(&population_arm_json("MissingSubjects", None));
    let (ok, ok_json) = result_json(expr.clone(), &complete);
    assert_eq!(ok.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&ok_json, "missing"), Some(0));

    let mut gap = EvidenceSet::new();
    authoritative_inventory(&mut gap, 2);
    protection(&mut gap, 1, true, 1);
    let (missing, missing_json) = result_json(expr, &gap);
    assert_eq!(missing.effectiveness, Effectiveness::InsufficientEvidence);
    assert_eq!(
        string_list(&missing_json, "missingSubjects"),
        vec!["repo:02"]
    );
}

#[test]
fn all_subjects_requires_authoritative_full_pass() {
    let expr = parse_expr(&population_arm_json("AllSubjects", None));
    let mut unknown = EvidenceSet::new();
    protection(&mut unknown, 1, true, 1);
    let (unknown_result, _) = result_json(expr.clone(), &unknown);
    assert_eq!(
        unknown_result.effectiveness,
        Effectiveness::Inconclusive,
        "AllSubjects on unknown population must not be Effective"
    );

    let mut full = EvidenceSet::new();
    authoritative_inventory(&mut full, 2);
    protection(&mut full, 1, true, 1);
    protection(&mut full, 2, true, 1);
    let (full_result, _) = result_json(expr, &full);
    assert_eq!(full_result.effectiveness, Effectiveness::Effective);
}

#[test]
fn any_subject_may_succeed_on_unknown_population() {
    let expr = parse_expr(&population_arm_json("AnySubject", None));
    let mut set = EvidenceSet::new();
    protection(&mut set, 1, true, 1);
    let (result, _) = result_json(expr, &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "AnySubject may succeed without an authoritative inventory"
    );
}

// ---------------------------------------------------------------------------
// Population model + IR SSOT
// ---------------------------------------------------------------------------

#[test]
fn population_ast_and_runtime_surfaces_exist() {
    let expr = read_crate_file("weeping-angel-control-test", "expr.rs");
    require_needles("P?: population TestExpr arms", &expr, POPULATION_ARMS);
    let runtime = crate_sources_joined("weeping-angel-control-test");
    require_needles(
        "P?: Population + completeness + evaluation",
        &runtime,
        &[
            "struct Population",
            "authoritative",
            "subject_ids",
            "PopulationEvaluation",
            "PopulationCompleteness",
            "observed_at",
        ],
    );
}

#[test]
fn unknown_completeness_cannot_yield_strong_all_subject_effective() {
    let runtime = crate_sources_joined("weeping-angel-control-test");
    require_needles(
        "P?: completeness is first-class",
        &runtime,
        &["PopulationCompleteness", "Unknown", "Authoritative"],
    );
    let mut set = EvidenceSet::new();
    for i in 1..=50 {
        protection(&mut set, i, true, 1);
    }
    for expr in [
        coverage_expr("100"),
        parse_expr(&population_arm_json("AllSubjects", None)),
    ] {
        let (result, _) = result_json(expr, &set);
        assert_ne!(result.effectiveness, Effectiveness::Effective);
        assert_eq!(result.effectiveness, Effectiveness::Inconclusive);
    }
}

#[test]
fn subject_kinds_cover_required_conceptual_set() {
    let src = read_crate_file("weeping-angel-assurance-ir", "subject.rs");
    require_needles("P?: SubjectKind extensions", &src, REQUIRED_SUBJECT_KINDS);
    let identity = read_crate_file("weeping-angel-assurance-ir", "identity.rs");
    assert!(
        identity.contains("ServiceAccount"),
        "IdentityKind must grow ServiceAccount to map SubjectKind::ServiceAccount"
    );
    let _ = (SubjectKind::Repository, IdentityKind::User);
    let joined = format!("{src}\n{identity}");
    for kind in CONCEPTUAL_KINDS {
        let token = kind
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => format!("{}{}", f.to_uppercase(), c.as_str()),
                    None => String::new(),
                }
            })
            .collect::<String>();
        assert!(
            joined.contains(&token) || joined.to_lowercase().contains(&kind.replace(' ', "")),
            "conceptual subject kind {kind} must be expressible (looked for {token})"
        );
    }
}

#[test]
fn ir_subject_selector_is_ssot() {
    let ct = read_crate_file("weeping-angel-control-test", "expr.rs");
    assert!(
        !ct.contains("pub id: Option<String>")
            || ct.contains("weeping_angel_assurance_ir::SubjectSelector")
            || ct.contains("pub use weeping_angel_assurance_ir::SubjectSelector"),
        "control-test must adapt/alias IR SubjectSelector, not keep a competing long-term type"
    );
    assert!(
        ct.contains("weeping_angel_assurance_ir::SubjectSelector")
            || ct.contains("pub ids:")
            || crate_sources_joined("weeping-angel-control-test")
                .contains("pub use weeping_angel_assurance_ir::SubjectSelector"),
        "IR SubjectSelector (kind, ids, tags, scope) must be the only long-term selector"
    );
}

// ---------------------------------------------------------------------------
// Evaluation output + distinct defect classes
// ---------------------------------------------------------------------------

#[test]
fn results_expose_population_evaluation() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 3);
    protection(&mut set, 1, true, 1);
    protection(&mut set, 2, false, 1);
    let (_, json) = result_json(coverage_expr("100"), &set);
    for key in [
        "population",
        "evaluated",
        "passing",
        "failing",
        "missing",
        "coverage",
        "failingSubjects",
        "missingSubjects",
    ] {
        let present = u64_metric(&json, key).is_some()
            || f64_metric(&json, key).is_some()
            || !string_list(&json, key).is_empty()
            || json.get(key).is_some()
            || json
                .get("populationEvaluation")
                .and_then(|p| p.get(key))
                .is_some();
        assert!(
            present,
            "result must expose `{key}` (nested object allowed): {json}"
        );
    }
    assert_population_metrics(&json, 3, 2, 1, 1, 1);
    assert_eq!(string_list(&json, "failingSubjects"), vec!["repo:02"]);
    assert_eq!(string_list(&json, "missingSubjects"), vec!["repo:03"]);
    let coverage = f64_metric(&json, "coverage").expect("coverage");
    assert!((coverage - (2.0 / 3.0)).abs() < 1e-9, "coverage={coverage}");
}

#[test]
fn missing_failing_stale_and_technical_failure_stay_distinct() {
    let mut set = EvidenceSet::new();
    authoritative_inventory(&mut set, 4);
    protection(&mut set, 1, true, 1);
    protection(&mut set, 2, false, 1);
    protection(&mut set, 3, true, 48);
    set.insert(seal(
        "source.branch.protection",
        "repo:04",
        &[("protected", "not-a-bool")],
        collected(1),
    ));
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(u64_metric(&json, "passing"), Some(1));
    assert_eq!(u64_metric(&json, "failing"), Some(1));
    assert_eq!(u64_metric(&json, "missing").unwrap_or(0), 0);
    assert_ne!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "type-mismatch technical failure must not be classified as missing evidence"
    );
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    let failing = string_list(&json, "failingSubjects");
    let missing = string_list(&json, "missingSubjects");
    let stale = string_list(&json, "staleSubjects");
    assert!(
        failing.contains(&"repo:02".to_string()),
        "explicit fail listed: {failing:?}"
    );
    assert!(
        !missing.contains(&"repo:04".to_string()),
        "technical failure must not appear in missingSubjects: {missing:?}"
    );
    assert!(
        stale.is_empty() || stale.contains(&"repo:03".to_string()),
        "stale subject listed when exposed: {stale:?}"
    );
}

// ---------------------------------------------------------------------------
// Compiler / facade / index / perf
// ---------------------------------------------------------------------------

#[test]
fn evaluate_compiled_attaches_expressions() {
    let src = crate_sources_joined("weeping-angel-assurance");
    let start = src.find("fn evaluate_compiled").expect("evaluate_compiled");
    let body = &src[start..start + 1600.min(src.len() - start)];
    assert!(
        body.contains(".expr(") || body.contains("expr:"),
        "evaluate_compiled must attach TestExpr / CompiledTest.expr"
    );
    let fw = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "P?: CompiledTest carries expr or evaluation body",
        &fw,
        &["struct CompiledTest"],
    );
    assert!(
        fw.contains("pub expr") || fw.contains("evaluation"),
        "CompiledTest must grow an expression attachment path"
    );
}

#[test]
fn index_avoids_quadratic_scans() {
    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("EvidenceIndex")
            || src.contains("by_subject")
            || src.contains("index_by")
            || src.contains("BTreeMap<(EvidenceType"),
        "population eval must index by evidence type and subject"
    );
    assert!(
        !src.contains("for subject in")
            || src.contains("EvidenceIndex")
            || src.contains("by_subject"),
        "steady-state algorithm must not be O(subjects × all_evidence)"
    );
}

#[test]
fn perf_fixture_100_subjects() {
    let set = scale_population(100);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "population"), Some(100));
    assert_eq!(u64_metric(&json, "passing"), Some(100));
}

#[test]
fn perf_fixture_1000_subjects() {
    let set = scale_population(1_000);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "population"), Some(1_000));
}

#[test]
fn perf_fixture_10000_subjects() {
    let set = scale_population(10_000);
    let (result, json) = result_json(coverage_expr("100"), &set);
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(u64_metric(&json, "population"), Some(10_000));
}

#[test]
fn perf_fixture_100000_envelopes_index_contract() {
    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("EvidenceIndex")
            || src.contains("index_envelopes")
            || src.contains("fn build_index")
            || src.contains("by_type_and_subject"),
        "100,000 evidence envelopes require an index constructor, not nested scans"
    );
    let this =
        fs::read_to_string(manifest_dir().join("tests/contracts/population_runtime.target.rs"))
            .unwrap();
    for needle in [
        "100 subjects",
        "1,000 subjects",
        "10,000 subjects",
        "100,000",
    ] {
        assert!(
            this.contains(needle) || src.contains(needle),
            "need a fixture or comment for {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// Handoff sentences + non-goals
// ---------------------------------------------------------------------------

#[test]
fn domain_handoff_sentences_are_expressible_without_provider_types() {
    let expr_src = read_crate_file("weeping-angel-control-test", "expr.rs");
    forbid_needles(
        "P?: TestExpr stays provider-blind",
        &expr_src,
        PROVIDER_TOKENS,
    );
    require_needles(
        "P?: handoff arms",
        &expr_src,
        &["AllSubjects", "NoneSubjects", "CoverageAtLeast"],
    );

    let mut mfa = EvidenceSet::new();
    mfa.insert(seal(
        "inventory.complete",
        "org:fixture",
        &[("kind", "privilegedIdentity"), ("authoritative", "true")],
        collected(1),
    ));
    for id in ["priv:ada", "priv:bob"] {
        mfa.insert(seal(
            "inventory.subject",
            id,
            &[("kind", "privilegedIdentity"), ("id", id)],
            collected(1),
        ));
        mfa.insert(seal(
            "identity.mfa.enabled",
            id,
            &[("enabled", "true")],
            collected(1),
        ));
    }
    let (mfa_result, mfa_json) = result_json(
        coverage_at_least(
            "privilegedIdentity",
            "identity.mfa.enabled",
            "enabled",
            "100",
        ),
        &mfa,
    );
    assert_eq!(
        mfa_result.effectiveness,
        Effectiveness::Effective,
        "all privileged identities have MFA"
    );
    assert_eq!(u64_metric(&mfa_json, "population"), Some(2));

    let mut repos = EvidenceSet::new();
    authoritative_inventory(&mut repos, 2);
    protection(&mut repos, 1, true, 1);
    protection(&mut repos, 2, true, 1);
    let (repo_result, _) = result_json(coverage_expr("100"), &repos);
    assert_eq!(
        repo_result.effectiveness,
        Effectiveness::Effective,
        "100% of non-archived repositories protect default branch"
    );

    let sla = parse_expr(&population_arm_json("NoneSubjects", None));
    let mut vulns = EvidenceSet::new();
    authoritative_inventory(&mut vulns, 2);
    vulns.insert(seal(
        "source.branch.protection",
        "repo:01",
        &[("protected", "true")],
        collected(1),
    ));
    vulns.insert(seal(
        "source.branch.protection",
        "repo:02",
        &[("protected", "true")],
        collected(1),
    ));
    let (sla_result, _) = result_json(sla, &vulns);
    assert_eq!(
        sla_result.effectiveness,
        Effectiveness::Effective,
        "no critical vulnerability exceeds SLA (NoneSubjects on authoritative pop)"
    );

    let mut endpoints = EvidenceSet::new();
    endpoints.insert(seal(
        "inventory.complete",
        "org:fixture",
        &[("kind", "endpoint"), ("authoritative", "true")],
        collected(1),
    ));
    for i in 1..=20 {
        let id = format!("ep:{i:02}");
        endpoints.insert(seal(
            "inventory.subject",
            &id,
            &[("kind", "endpoint"), ("id", &id)],
            collected(1),
        ));
        let enabled = if i <= 19 { "true" } else { "false" };
        endpoints.insert(seal(
            "endpoint.encryption.enabled",
            &id,
            &[("enabled", enabled)],
            collected(1),
        ));
    }
    let (ep_result, ep_json) = result_json(
        coverage_at_least("endpoint", "endpoint.encryption.enabled", "enabled", "95"),
        &endpoints,
    );
    assert!(
        matches!(
            ep_result.effectiveness,
            Effectiveness::Effective | Effectiveness::PartiallyEffective
        ),
        "at least 95% of endpoints report encryption enabled, got {:?}",
        ep_result.effectiveness
    );
    assert_eq!(u64_metric(&ep_json, "population"), Some(20));
    assert_eq!(u64_metric(&ep_json, "passing"), Some(19));
    assert_eq!(u64_metric(&ep_json, "failing"), Some(1));

    let _ = HANDOFF_SENTENCES;
}

#[test]
fn control_test_stays_network_free_and_provider_neutral() {
    let toml = fs::read_to_string(
        manifest_dir()
            .join("crates")
            .join("weeping-angel-control-test")
            .join("Cargo.toml"),
    )
    .unwrap();
    for dep in ["reqwest", "ureq", "hyper", "tokio"] {
        assert!(
            !toml.contains(dep),
            "control-test must stay network-free; found {dep}"
        );
    }
    let src = crate_sources_joined("weeping-angel-control-test");
    forbid_needles(
        "P?: no provider discovery / ISO coverage / org-graph in evaluator",
        &src,
        &[
            "GithubRepositorySelector",
            "iso27001_coverage",
            "OrganizationGraph",
            "discover_provider",
        ],
    );
    require_needles(
        "P?: provider-neutral population runtime still required",
        &src,
        &[
            "struct Population",
            "PopulationCompleteness",
            "PopulationEvaluation",
        ],
    );
}

// Fixture markers the implement phase must honor (names appear in this file on purpose):
// 100 subjects — constructed in perf_fixture_100_subjects.
// 1,000 subjects — constructed in perf_fixture_1000_subjects.
// 10,000 subjects — constructed in perf_fixture_10000_subjects.
// 100,000 evidence envelopes — index contract in perf_fixture_100000_envelopes_index_contract.
