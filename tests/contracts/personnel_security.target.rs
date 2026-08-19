//! Target suite for Operational ISMS v1 Prompt 17 (personnel security).
//!
//! Encodes DESIRED behavior in `docs/specs/personnel-security.md` §4 / §5
//! (PER-001…016). Must stay RED on the current tree: no additive
//! `personnel.toml` lifecycle slice, no eight personnel fixtures, and no
//! population-honest joiner / mover / leaver tests. Do not `#[ignore]` these
//! tests and do not implement catalog content or fixtures here.
//!
//! Scan **catalog TOML and product crates** only (I4a). Never grep this file
//! for a token that also appears in an assertion string.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlImplementation, ControlImplementationId, ControlTestId,
    Exception, ExceptionId, ExceptionStatus, Identity, IdentityId, IdentityKind, SelectorScope,
    SubjectKind,
};
use weeping_angel_canonical_catalog::{CATALOG_SCHEMA, CanonicalCatalog, DIGEST_PREFIX};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, CountPredicate,
    Effectiveness, EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
    looks_like_compliance_claim,
};

const GOVERNANCE_PERSONNEL_CONTROLS: &[&str] = &[
    "control.personnel.security-awareness",
    "control.personnel.role-specific-training",
    "control.personnel.onboarding-offboarding",
    "control.personnel.confidentiality-commitment",
    "control.personnel.policy-acknowledgement",
];

const REQUIRED_LIFECYCLE_CONTROLS: &[&str] = &[
    "control.personnel.screening",
    "control.personnel.joiner-grace",
    "control.personnel.role-change",
    "control.personnel.leaver-access",
    "control.personnel.asset-return",
];

const OPTIONAL_PROVISIONING_CONTROL: &str = "control.personnel.access-provisioning";

const EXISTING_PERSONNEL_EVIDENCE: &[&str] = &[
    "evidence.personnel.training",
    "evidence.personnel.acknowledgement",
];

const NEW_PERSONNEL_EVIDENCE: &[&str] = &[
    "evidence.personnel.screening",
    "evidence.personnel.joiner-grace",
    "evidence.personnel.population-membership",
    "evidence.personnel.asset-return",
];

const EXISTING_PERSONNEL_TESTS: &[&str] = &[
    "test.personnel.awareness-current-all",
    "test.personnel.training-current-all",
    "test.personnel.jml-process-attested",
    "test.personnel.confidentiality-acknowledged-all",
    "test.personnel.policy-acknowledged-all",
];

const REQUIRED_LIFECYCLE_TESTS: &[&str] = &[
    "test.personnel.screening-recorded",
    "test.personnel.joiner-grace-honored",
    "test.personnel.mover-privileges-reduced",
    "test.personnel.no-leaver-active-access",
    "test.personnel.asset-return-recorded",
];

const OPTIONAL_PROVISIONING_TEST: &str = "test.personnel.joiner-access-provisioned";

const PERSONNEL_FIXTURE_NAMES: &[&str] = &[
    "complete-training-population",
    "one-overdue-user",
    "new-joiner-grace",
    "leaver-with-active-access",
    "mover-retaining-excessive-privileges",
    "expired-exception",
    "missing-personnel-source",
    "manual-screening-evidence",
];

const GOVERNANCE_FAMILY_PREFIXES: &[&str] = &[
    "control.governance.",
    "control.risk.",
    "control.personnel.",
    "control.vendor.",
    "control.incident.",
    "control.resilience.",
];

const IAM_JML_CONTROLS: &[&str] = &[
    "control.identity.joiner-mover-leaver",
    "control.identity.terminated-user-removal",
    "control.identity.access-revocation-timeliness",
];

const PERSONNEL_TOML: &[&str] = &[
    "catalog/canonical/v1/controls/personnel.toml",
    "catalog/canonical/v1/evidence/personnel.toml",
    "catalog/canonical/v1/tests/personnel.toml",
];

const POPULATION_OPS: &[&str] = &[
    "all-subjects",
    "all_subjects",
    "AllSubjects",
    "none-subjects",
    "none_subjects",
    "NoneSubjects",
    "coverage-at-least",
    "coverage_at_least",
    "CoverageAtLeast",
    "count-where",
    "fresh-within",
    "manual-review",
    "manual_review",
    "ManualReview",
];

const FORBIDDEN_PROVIDER_TOKENS: &[&str] = &[
    "workday", "bamboohr", "okta", "entra", "knowbe4", "intune", "jamf", "rippling",
];

const FORBIDDEN_FRAMEWORK_TOKENS: &[&str] = &[
    "iso27001",
    "iso-27001",
    "soc2",
    "soc-2",
    "nis2",
    "nis-2",
    "dora",
    "gdpr",
];

const FORBIDDEN_GRC_TOKENS: &[&str] = &["vanta", "drata"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1_dir() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1_dir()).expect("canonical catalog must load")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn walk_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            walk_files(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
}

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_files(&crate_src(name), "rs", &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn crate_toml(name: &str) -> String {
    read_repo_file(&format!("crates/{name}/Cargo.toml"))
}

fn personnel_toml_text() -> String {
    let mut chunks = Vec::new();
    for rel in PERSONNEL_TOML {
        let path = manifest_dir().join(rel);
        assert!(
            path.is_file(),
            "PER-001: additive catalog file `{rel}` must exist"
        );
        chunks.push(fs::read_to_string(&path).unwrap());
    }
    chunks.join("\n")
}

fn require_personnel_slice() -> CanonicalCatalog {
    let manifest = read_repo_file("catalog/canonical/v1/manifest.toml");
    assert!(
        manifest.contains("controls/personnel.toml")
            && manifest.contains("evidence/personnel.toml")
            && manifest.contains("tests/personnel.toml"),
        "PER-001: manifest.toml [files] must list catalog/canonical/v1/{{controls,evidence,tests}}/personnel.toml"
    );
    let catalog = load_catalog();
    catalog
        .validate()
        .expect("PER-001: CanonicalCatalog::validate must accept the personnel slice");
    for id in REQUIRED_LIFECYCLE_CONTROLS {
        catalog.control(id).unwrap_or_else(|e| {
            panic!("PER-001: loaded catalog missing lifecycle control `{id}`: {e}")
        });
    }
    catalog
}

fn fixture_root() -> PathBuf {
    manifest_dir().join("fixtures/assurance/canonical/v1/personnel")
}

fn fixture_dir(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn require_eight_fixtures() {
    assert!(
        fixture_root().is_dir(),
        "PER-009: fixtures/assurance/canonical/v1/personnel must exist"
    );
    for name in PERSONNEL_FIXTURE_NAMES {
        let evidence = fixture_dir(name).join("evidence.json");
        assert!(
            evidence.is_file(),
            "PER-009: fixture `{name}` must ship evidence.json at {}",
            evidence.display()
        );
        let blob = fs::read_to_string(&evidence).unwrap();
        let lower = blob.to_ascii_lowercase();
        assert!(
            !looks_like_compliance_claim(&blob),
            "PER-016: fixture `{name}` must not emit compliance-claim narratives"
        );
        for token in FORBIDDEN_GRC_TOKENS
            .iter()
            .chain(FORBIDDEN_PROVIDER_TOKENS.iter())
        {
            assert!(
                !lower.contains(token),
                "PER-009: fixture `{name}` must not name a provider/GRC product"
            );
        }
        assert!(
            !blob.contains("source.") && !blob.contains("evidence.github."),
            "PER-009: fixture `{name}` must emit canonical personnel/identity facts only"
        );
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        max_age: Duration::from_secs(365 * 24 * 3600),
    }
}

fn collected(hours_ago: i64) -> DateTime<Utc> {
    fresh_context().now - chrono::Duration::hours(hours_ago)
}

fn seal(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, EvidenceValue)],
    at: DateTime<Utc>,
) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_value(*k, v.clone());
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.personnel-target".into(),
            collected_at: at,
            scope: "target".into(),
            asset: weeping_angel_assurance_ir::AssetId::new(asset),
        },
    )
    .unwrap()
}

fn compiled(test_id: &str, control_id: &str, expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(test_id))
        .control_id(ControlId::new(control_id))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn json_fact(value: &Value) -> EvidenceValue {
    match value {
        Value::Bool(flag) => EvidenceValue::Bool(*flag),
        Value::Number(n) if n.is_i64() => EvidenceValue::Integer(n.as_i64().unwrap()),
        Value::String(text) => {
            if text == "true" {
                EvidenceValue::Bool(true)
            } else if text == "false" {
                EvidenceValue::Bool(false)
            } else if let Ok(n) = text.parse::<i64>() {
                EvidenceValue::Integer(n)
            } else if let Ok(ts) = DateTime::parse_from_rfc3339(text) {
                EvidenceValue::Timestamp(ts.with_timezone(&Utc))
            } else {
                EvidenceValue::String(text.clone())
            }
        }
        other => EvidenceValue::String(other.to_string()),
    }
}

fn load_personnel_fixture(name: &str) -> EvidenceSet {
    let dir = fixture_dir(name);
    let evidence_path = dir.join("evidence.json");
    assert!(
        evidence_path.is_file(),
        "PER-009: fixture `{name}` missing evidence.json"
    );
    let raw = fs::read_to_string(&evidence_path).unwrap();
    let doc: Value = serde_json::from_str(&raw).unwrap();
    let collected_at = doc
        .get("collectedAt")
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|ts| ts.with_timezone(&Utc))
        .unwrap_or_else(|| collected(0));
    let mut set = EvidenceSet::new();
    let rows = doc
        .get("evidence")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for row in rows {
        let ty = row
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{name}: evidence row missing type"));
        let subject = row
            .get("subject_id")
            .or_else(|| row.get("subjectId"))
            .and_then(Value::as_str)
            .unwrap_or("org:personnel");
        let mut owned = Vec::new();
        if let Some(map) = row.get("facts").and_then(Value::as_object) {
            for (k, v) in map {
                owned.push((k.clone(), json_fact(v)));
            }
        }
        let refs: Vec<(&str, EvidenceValue)> =
            owned.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        set.insert(seal(ty, subject, &refs, collected_at));
    }
    let exceptions_path = dir.join("exceptions.json");
    if exceptions_path.is_file() {
        let blob = fs::read_to_string(exceptions_path).unwrap();
        let listed: Vec<Exception> = serde_json::from_str(&blob).unwrap();
        for exception in listed {
            set.insert_exception(exception);
        }
    }
    if let Some(arr) = doc.get("exceptions").and_then(Value::as_array) {
        for row in arr {
            let parsed: Exception = serde_json::from_value(row.clone()).unwrap();
            set.insert_exception(parsed);
        }
    }
    set
}

fn parse_duration(raw: &str) -> Duration {
    let trimmed = raw.trim();
    if let Some(days) = trimmed.strip_suffix('d') {
        let n: u64 = days.parse().unwrap_or(365);
        return Duration::from_secs(n * 24 * 3600);
    }
    Duration::from_secs(trimmed.parse().unwrap_or(365 * 24 * 3600))
}

fn expr_from_map(
    expression: &BTreeMap<String, toml::Value>,
    test_id: &str,
    subjects: &[BTreeMap<String, toml::Value>],
) -> TestExpr {
    let op = expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("{test_id} must declare [test.expression].op"));
    if op == "manual-review" || op == "manual_review" {
        return TestExpr::ManualReview;
    }
    if op == "all" || op == "any" {
        let children = expression
            .get("of")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("{test_id} compound `{op}` needs `of`"));
        let parsed: Vec<TestExpr> = children
            .iter()
            .map(|child| {
                let map: BTreeMap<String, toml::Value> = child
                    .as_table()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                expr_from_map(&map, test_id, subjects)
            })
            .collect();
        return if op == "all" {
            TestExpr::All(parsed)
        } else {
            TestExpr::Any(parsed)
        };
    }
    let evidence = expression
        .get("evidence")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("{test_id} must declare expression.evidence"));
    let field = expression
        .get("field")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let kind = subjects
        .first()
        .and_then(|row| row.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    let selector = SubjectSelector {
        kind: Some(kind.into()),
        id: None,
    };
    let freshness = expression
        .get("duration")
        .or_else(|| expression.get("freshness"))
        .and_then(|v| v.as_str())
        .map(parse_duration);
    let evidence_sel = EvidenceSelector {
        evidence_type: EvidenceType::new(evidence),
        subject_selector: selector.clone(),
        field,
        freshness,
    };
    match op {
        "exists" => TestExpr::Exists(evidence_sel),
        "fresh-within" => TestExpr::FreshWithin {
            selector: evidence_sel,
            duration: freshness.unwrap_or(Duration::from_secs(365 * 24 * 3600)),
        },
        "all-subjects" | "all_subjects" | "AllSubjects" => TestExpr::AllSubjects {
            selector,
            evidence: evidence_sel,
        },
        "none-subjects" | "none_subjects" | "NoneSubjects" => TestExpr::NoneSubjects {
            selector,
            evidence: evidence_sel,
        },
        "coverage-at-least" | "coverage_at_least" | "CoverageAtLeast" => {
            TestExpr::CoverageAtLeast {
                selector,
                evidence: evidence_sel,
                percentage: expression
                    .get("percentage")
                    .and_then(|v| {
                        v.as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| v.as_integer().map(|n| n.to_string()))
                    })
                    .unwrap_or_else(|| "100".into()),
            }
        }
        "count-where" => TestExpr::CountWhere {
            selector,
            evidence: evidence_sel,
            predicate: CountPredicate::Eq(0),
        },
        other => panic!("{test_id} uses unsupported op `{other}`"),
    }
}

fn catalog_test_expr(catalog: &CanonicalCatalog, test_id: &str) -> (String, TestExpr) {
    let test = catalog
        .tests()
        .get(test_id)
        .unwrap_or_else(|| panic!("catalog missing test `{test_id}`"));
    (
        test.control.clone(),
        expr_from_map(&test.expression, test_id, &test.subjects),
    )
}

fn evaluate_catalog_test(
    catalog: &CanonicalCatalog,
    test_id: &str,
    set: &EvidenceSet,
) -> ControlTestResult {
    let (control, expr) = catalog_test_expr(catalog, test_id);
    evaluate(&compiled(test_id, &control, expr), set, &fresh_context())
}

fn test_window(catalog_text: &str, test_id: &str) -> String {
    let marker = format!("id = \"{test_id}\"");
    let start = catalog_text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing test record {test_id}"));
    catalog_text[start..start + 1400.min(catalog_text.len() - start)].to_string()
}

fn expression_is_existence_only(window: &str) -> bool {
    let lower = window.to_ascii_lowercase();
    let has_exists = lower.contains("op = \"exists\"") || lower.contains("exists(");
    let has_population_or_fresh = POPULATION_OPS
        .iter()
        .any(|op| window.contains(op) || lower.contains(&op.to_ascii_lowercase()));
    has_exists && !has_population_or_fresh
}

fn bound_exception(control: &str, subject: &str) -> Exception {
    let mut ex = Exception::new(
        ExceptionId::new(format!("exc:{subject}")),
        "approved unexpired personnel waiver",
    );
    ex.status = ExceptionStatus::Approved;
    ex.control_id = Some(ControlId::new(control));
    ex.expires_at = Some(fresh_context().now + chrono::Duration::days(30));
    let mut ids = BTreeSet::new();
    ids.insert(subject.into());
    ex.subjects
        .push(weeping_angel_assurance_ir::SubjectSelector {
            kind: SubjectKind::User,
            ids,
            tags: BTreeMap::new(),
            scope: SelectorScope::AnyOf,
        });
    ex
}

fn fixture_types(set: &EvidenceSet) -> BTreeSet<String> {
    set.iter()
        .map(|env| env.observation().evidence_type().as_str().to_string())
        .collect()
}

fn named_subjects(result: &ControlTestResult) -> Vec<String> {
    result
        .population
        .as_ref()
        .map(|pop| pop.failing_subjects.clone())
        .unwrap_or_default()
}

fn never_effective(result: &ControlTestResult, label: &str) {
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "{label}: must never be Effective; {} ({:?})",
        result.rationale,
        result.effectiveness
    );
}

/// PER-000: dual-suite is registered (tests/contracts is not auto-discovered).
#[test]
fn per_000_dual_suite_is_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("sdd_personnel_security_baseline")
            && cargo.contains("tests/contracts/personnel_security.baseline.rs")
            && cargo.contains("sdd_personnel_security_target")
            && cargo.contains("tests/contracts/personnel_security.target.rs"),
        "PER-000: dual-suite [[test]] rows must be listed in root Cargo.toml"
    );
}

/// PER-001: catalog loads additive `personnel.toml` offline.
#[test]
fn per_001_catalog_loads_personnel_toml_offline() {
    let catalog = require_personnel_slice();
    assert_eq!(CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1");
    catalog
        .control("control.personnel.security-awareness")
        .expect("PER-001: existing five governance personnel rows stay loaded");
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    assert_eq!(
        rust.matches("struct CanonicalCatalog").count(),
        1,
        "PER-001: consume the single catalog loader; do not invent a second one"
    );
}

/// PER-002: digest remains deterministic after personnel files are listed.
#[test]
fn per_002_digest_stays_deterministic_with_personnel_files() {
    let catalog = require_personnel_slice();
    let digest = catalog.digest().expect("PER-002: digest");
    assert!(
        digest.to_string().starts_with(DIGEST_PREFIX),
        "PER-002: digest must use {DIGEST_PREFIX}, got {digest}"
    );
    let again = load_catalog();
    assert_eq!(
        digest.to_string(),
        again.digest().expect("digest").to_string(),
        "PER-002: CanonicalCatalog::digest is deterministic with personnel.toml listed"
    );
}

/// PER-003: keep the five rows; add ≤6 lifecycle controls; family stays 30–45.
#[test]
fn per_003_five_rows_kept_additive_lifecycle_controls_in_band() {
    let catalog = require_personnel_slice();
    for id in GOVERNANCE_PERSONNEL_CONTROLS {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("PER-003: existing `{id}` must remain"));
    }
    for id in REQUIRED_LIFECYCLE_CONTROLS {
        let control = catalog.control(id).expect(id);
        assert!(
            matches!(
                control.automation.as_str(),
                "hybrid" | "automated" | "manual"
            ),
            "PER-003: `{id}` automation must be honest, got {}",
            control.automation
        );
        assert!(
            control
                .domains
                .iter()
                .any(|d| d == "personnelSecurity" || d == "accessControl"),
            "PER-003: `{id}` domains must include personnelSecurity (or accessControl)"
        );
        assert!(
            !control.evidence.is_empty() && !control.tests.is_empty(),
            "PER-003: `{id}` must reference evidence and tests"
        );
    }

    let personnel: Vec<_> = catalog
        .controls()
        .keys()
        .filter(|id| id.starts_with("control.personnel."))
        .cloned()
        .collect();
    let added = personnel
        .len()
        .saturating_sub(GOVERNANCE_PERSONNEL_CONTROLS.len());
    assert!(
        (5..=6).contains(&added) || personnel.len() >= GOVERNANCE_PERSONNEL_CONTROLS.len() + 5,
        "PER-003: expected five kept rows plus ≤6 additive lifecycle controls; personnel ids={personnel:?}"
    );
    assert!(
        added <= 6,
        "PER-003: additive control.personnel.* count must be ≤6 (GOV-003); found {added} new among {personnel:?}"
    );
    if catalog.control(OPTIONAL_PROVISIONING_CONTROL).is_err() {
        let joiner = catalog
            .control("control.personnel.joiner-grace")
            .expect("joiner-grace");
        assert!(
            joiner
                .tests
                .iter()
                .any(|t| t == OPTIONAL_PROVISIONING_TEST
                    || t == "test.personnel.joiner-grace-honored"),
            "PER-003: when access-provisioning is merged, joiner-grace must still declare lifecycle tests; {:?}",
            joiner.tests
        );
    }

    let family: Vec<_> = catalog
        .controls()
        .keys()
        .filter(|id| GOVERNANCE_FAMILY_PREFIXES.iter().any(|p| id.starts_with(p)))
        .collect();
    assert!(
        (30..=45).contains(&family.len()),
        "PER-003: governance-family slice stays 30–45, found {}",
        family.len()
    );
}

/// PER-004: new evidence types are facts; no orphans; no conclusion phrases.
#[test]
fn per_004_new_personnel_evidence_types_are_facts() {
    let catalog = require_personnel_slice();
    for id in EXISTING_PERSONNEL_EVIDENCE {
        assert!(catalog.evidence().contains_key(*id), "PER-004: keep `{id}`");
    }
    for id in NEW_PERSONNEL_EVIDENCE {
        let ev = catalog
            .evidence()
            .get(*id)
            .unwrap_or_else(|| panic!("PER-004: `{id}` must be declared as a fact type"));
        assert_eq!(ev.criticality, "required", "{id} criticality");
        assert!(
            ev.evidence_type.starts_with("personnel."),
            "{id} evidence_type stays personnel.*"
        );
        let lower = format!("{} {}", ev.title, ev.evidence_type).to_ascii_lowercase();
        assert!(
            !lower.contains("cleared")
                && !lower.contains("compliant")
                && !lower.contains("certified"),
            "PER-004: `{id}` title must not be a conclusion"
        );
        let referenced_by_control = catalog.controls().values().any(|c| {
            c.evidence.contains(&id.to_string())
                || *id == "evidence.personnel.population-membership"
        });
        let referenced_by_test = catalog.tests().values().any(|t| {
            t.required_evidence.iter().any(|e| e == id)
                || t.expression
                    .get("evidence")
                    .and_then(|v| v.as_str())
                    .is_some_and(|e| e == *id)
                || *id == "evidence.personnel.population-membership"
        });
        assert!(
            referenced_by_control && referenced_by_test,
            "PER-004: `{id}` must not be an orphan (control+test refs, membership may be inventory support)"
        );
    }

    let toml = personnel_toml_text();
    let lower = toml.to_ascii_lowercase();
    for phrase in [
        "cleared",
        "compliant",
        "audit passed",
        "offboarding certified",
        "training control passed",
        "joiner control effective",
    ] {
        assert!(
            !lower.contains(phrase),
            "PER-004: personnel.toml must not encode conclusion phrase `{phrase}`"
        );
    }
}

/// PER-005: required tests are declared and none are existence-only.
#[test]
fn per_005_lifecycle_tests_are_population_predicates() {
    let catalog = require_personnel_slice();
    for id in EXISTING_PERSONNEL_TESTS {
        assert!(catalog.tests().contains_key(*id), "PER-005: keep `{id}`");
    }
    let toml = personnel_toml_text();
    for id in REQUIRED_LIFECYCLE_TESTS {
        let test = catalog
            .tests()
            .get(*id)
            .unwrap_or_else(|| panic!("PER-005: missing `{id}`"));
        let window = test_window(&toml, id);
        assert!(
            !expression_is_existence_only(&window),
            "PER-005: `{id}` must not be existence-only; one envelope never proves coverage"
        );
        let op = test
            .expression
            .get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_ne!(
            op, "exists",
            "PER-005: `{id}` must not be Exists(evidence) as the body of a population test"
        );
        assert!(
            !test.required_evidence.is_empty(),
            "PER-005: `{id}` must list required evidence"
        );
    }
    if catalog.control(OPTIONAL_PROVISIONING_CONTROL).is_ok() {
        assert!(
            catalog.tests().contains_key(OPTIONAL_PROVISIONING_TEST),
            "PER-005: access-provisioning requires `{OPTIONAL_PROVISIONING_TEST}`"
        );
    }
}

/// PER-006: catalog TOML has no provider / HRIS / LMS / MDM / framework tokens.
#[test]
fn per_006_personnel_catalog_toml_is_provider_neutral() {
    let _catalog = require_personnel_slice();
    let toml = personnel_toml_text();
    let lower = toml.to_ascii_lowercase();
    for token in FORBIDDEN_PROVIDER_TOKENS
        .iter()
        .chain(FORBIDDEN_FRAMEWORK_TOKENS.iter())
        .chain(FORBIDDEN_GRC_TOKENS.iter())
    {
        assert!(
            !id_has_token_in_catalog(&toml, token) && !lower.contains(&format!(".{token}.")),
            "PER-006: personnel catalog TOML must not use reserved token `{token}` as an id subject"
        );
        for line in toml.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("id = ") {
                let value = trimmed.to_ascii_lowercase();
                assert!(
                    !value.contains(&format!(".{token}."))
                        && !value.contains(&format!("control.{token}."))
                        && !value.contains(&format!("test.{token}.")),
                    "PER-006: id line must not be provider/framework-keyed: {trimmed}"
                );
            }
        }
    }
    let kinds = toml.to_ascii_lowercase();
    assert!(
        !kinds.contains("kind = \"employee\"")
            && !kinds.contains("kind = \"contractor\"")
            && !kinds.contains("subjectkind::employee")
            && !kinds.contains("identitykind::contractor"),
        "PER-006: catalog TOML must not introduce Employee/Contractor as kinds"
    );
}

fn id_has_token_in_catalog(text: &str, token: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("id = ") {
            return false;
        }
        trimmed
            .to_ascii_lowercase()
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
            .any(|seg| seg == token || seg.split('-').any(|part| part == token))
    })
}

/// PER-007: Identity stays thin; no second population resolver; Prompt 10/12 not landed.
#[test]
fn per_007_identity_stays_thin_no_personnel_resolver() {
    let catalog = require_personnel_slice();
    assert!(
        catalog
            .evidence()
            .contains_key("evidence.personnel.population-membership"),
        "PER-007: populations use population-membership facts, not HR kinds"
    );
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");

    let identity = Identity::new(IdentityId::new("user:ada"), IdentityKind::User);
    let json = serde_json::to_value(&identity).unwrap();
    assert_eq!(json["kind"], "user");
    assert!(json.get("displayName").is_none());
    assert_eq!(SubjectKind::parse_name("employee"), None);
    assert_eq!(SubjectKind::parse_name("contractor"), None);

    let identity_src = read_repo_file("crates/weeping-angel-assurance-ir/src/identity.rs");
    let subject_src = read_repo_file("crates/weeping-angel-assurance-ir/src/subject.rs");
    for src in [&identity_src, &subject_src] {
        assert!(
            !src.contains("Employee") && !src.contains("Contractor"),
            "PER-007: IR must not grow HR employment kinds"
        );
    }
    let product = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !product.contains("fn resolve_personnel_inventory"),
        "PER-007: do not add resolve_personnel_inventory"
    );

    let implementation = ControlImplementation::new(
        ControlImplementationId::new("impl.personnel.target"),
        ControlId::new("control.personnel.security-awareness"),
    );
    let impl_json = serde_json::to_value(&implementation).unwrap();
    for key in ["documentRefs", "document_refs", "reviewCadence"] {
        assert!(
            impl_json.get(key).is_none(),
            "PER-007: Prompt 10 CIR fields `{key}` are not implemented in this slice"
        );
    }
    for src in [&identity_src, &subject_src] {
        assert!(
            !src.contains("pub struct DocumentRef")
                && !src.contains("pub struct ControlledDocument"),
            "PER-007: identity/subject IR must not grow Prompt 12 document types"
        );
    }
}

/// PER-008: IAM JML ids stay; ISO pack is not rewritten.
#[test]
fn per_008_iam_jml_and_iso_pack_are_not_retargeted() {
    let catalog = require_personnel_slice();
    for id in IAM_JML_CONTROLS {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("PER-008: keep IAM `{id}`"));
    }
    let jml = catalog
        .tests()
        .get("test.identity.jml-events-recorded")
        .expect("jml-events-recorded");
    assert_eq!(
        jml.expression.get("op").and_then(|v| v.as_str()),
        Some("manual-review"),
        "PER-008: IAM jml-events-recorded stays manual-review"
    );
    let removal = catalog
        .tests()
        .get("test.identity.no-terminated-active-accounts")
        .expect("no-terminated-active-accounts");
    assert_eq!(
        removal.expression.get("field").and_then(|v| v.as_str()),
        Some("status"),
        "PER-008: IAM terminated-user-removal still predicates string status"
    );
    assert!(
        catalog
            .tests()
            .contains_key("test.personnel.no-leaver-active-access"),
        "PER-008: personnel adds population-honest leaver test without retargeting IAM ids"
    );

    let identity_toml = read_repo_file("catalog/canonical/v1/controls/identity.toml");
    for id in IAM_JML_CONTROLS {
        assert!(
            identity_toml.contains(id),
            "PER-008: do not move IAM `{id}` out of identity.toml"
        );
    }
    let pack = manifest_dir().join("frameworks/iso-27001/2022");
    assert!(pack.is_dir(), "PER-008: ISO pack directory remains");
    let mut pack_files = Vec::new();
    walk_files(&pack, "toml", &mut pack_files);
    let pack_text = pack_files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !pack_text.contains("control.personnel.leaver-access"),
        "PER-008: do not rewrite ISO pack onto personnel lifecycle control ids"
    );
}

/// PER-009: complete training population → Effective; one envelope never covers N>1.
#[test]
fn per_009_complete_training_population_is_effective() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("complete-training-population");
    let types = fixture_types(&set);
    assert!(
        types.contains("evidence.personnel.training"),
        "PER-009: complete-training-population must emit training facts"
    );
    assert!(
        types.contains("inventory.subject") || types.contains("inventory.complete"),
        "PER-009: complete-training-population must be an authoritative population"
    );

    let training = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &set);
    assert_eq!(
        training.effectiveness,
        Effectiveness::Effective,
        "PER-009: complete training population must be Effective; {}",
        training.rationale
    );
    let awareness = evaluate_catalog_test(&catalog, "test.personnel.awareness-current-all", &set);
    assert_eq!(
        awareness.effectiveness,
        Effectiveness::Effective,
        "PER-009: complete awareness population must be Effective; {}",
        awareness.rationale
    );

    let user_count = set
        .iter()
        .filter(|env| env.observation().evidence_type().as_str() == "inventory.subject")
        .count();
    assert!(
        user_count > 1,
        "PER-009: complete population fixture must enumerate N>1 subjects, found {user_count}"
    );
    let mut single = EvidenceSet::new();
    let mut kept_training = false;
    for env in set.iter() {
        let ty = env.observation().evidence_type().as_str();
        if ty == "evidence.personnel.training" {
            if kept_training {
                continue;
            }
            kept_training = true;
        }
        single.insert(env.clone());
    }
    let one = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &single);
    assert_ne!(
        one.effectiveness,
        Effectiveness::Effective,
        "PER-009: a single training envelope must never prove coverage of N>1; {}",
        one.rationale
    );
}

/// PER-010: one overdue user (`current=false`) → Ineffective naming the subject.
#[test]
fn per_010_one_overdue_user_is_ineffective() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("one-overdue-user");
    let result = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "PER-010: current=false is overdue Ineffective, not missing; {}",
        result.rationale
    );
    let failing = named_subjects(&result);
    assert!(
        !failing.is_empty(),
        "PER-010: overdue subject must be named; pop={:?}",
        result.population
    );
}

/// PER-011: new joiner inside grace is not overdue on joiner-grace-honored.
#[test]
fn per_011_new_joiner_grace_is_not_an_exception() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("new-joiner-grace");
    let types = fixture_types(&set);
    assert!(
        types.contains("evidence.personnel.joiner-grace"),
        "PER-011: grace is a within_grace fact, not an IR Exception"
    );
    assert!(
        set.exceptions().is_empty(),
        "PER-011: joiner grace must not be encoded as Exception records"
    );
    let grace = evaluate_catalog_test(&catalog, "test.personnel.joiner-grace-honored", &set);
    assert_ne!(
        grace.effectiveness,
        Effectiveness::Ineffective,
        "PER-011: in-window joiner must not be Ineffective on joiner-grace-honored; {}",
        grace.rationale
    );
    assert_ne!(
        grace.effectiveness,
        Effectiveness::ExceptionApproved,
        "PER-011: grace is not ExceptionApproved"
    );
    let training = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &set);
    assert_ne!(
        training.effectiveness,
        Effectiveness::Effective,
        "PER-011: strict training-current-all remains honest for a joiner without current training; {}",
        training.rationale
    );
}

/// PER-012: leaver with `active=true` → Ineffective.
#[test]
fn per_012_leaver_with_active_access_is_ineffective() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("leaver-with-active-access");
    let types = fixture_types(&set);
    assert!(
        types.contains("evidence.identity.lifecycle-event")
            && types.contains("evidence.identity.account-status"),
        "PER-012: leaver fixture composes identity lifecycle-event + account-status facts"
    );
    let result = evaluate_catalog_test(&catalog, "test.personnel.no-leaver-active-access", &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "PER-012: leaver + active=true must be Ineffective; {}",
        result.rationale
    );
    assert!(
        !named_subjects(&result).is_empty(),
        "PER-012: the leaver subject must be named; {:?}",
        result.population
    );
}

/// PER-013: mover with `excessive=true` → Ineffective.
#[test]
fn per_013_mover_retaining_excessive_privileges_is_ineffective() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("mover-retaining-excessive-privileges");
    let types = fixture_types(&set);
    assert!(
        types.contains("evidence.identity.lifecycle-event")
            && types.contains("evidence.identity.role-membership"),
        "PER-013: mover fixture composes lifecycle-event + role-membership.excessive"
    );
    let result = evaluate_catalog_test(&catalog, "test.personnel.mover-privileges-reduced", &set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "PER-013: mover + excessive=true must be Ineffective; {}",
        result.rationale
    );
}

/// PER-014: expired exception does not suppress fail; approved bound exception is not silent Effective.
#[test]
fn per_014_expired_exception_does_not_suppress_fail() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let expired = load_personnel_fixture("expired-exception");
    assert!(
        expired.exceptions().iter().any(|ex| {
            ex.status == ExceptionStatus::Expired
                || ex.status == ExceptionStatus::Revoked
                || ex.expires_at.is_some_and(|at| at < fresh_context().now)
        }),
        "PER-014: expired-exception fixture must carry an expired/revoked/past-validity Exception"
    );
    let candidates = [
        "test.personnel.training-current-all",
        "test.personnel.screening-recorded",
        "test.personnel.no-leaver-active-access",
        "test.personnel.mover-privileges-reduced",
        "test.personnel.awareness-current-all",
    ];
    let mut saw_gap = false;
    for test_id in candidates {
        if !catalog.tests().contains_key(test_id) {
            continue;
        }
        let result = evaluate_catalog_test(&catalog, test_id, &expired);
        assert_ne!(
            result.effectiveness,
            Effectiveness::ExceptionApproved,
            "PER-014: expired/revoked exception must not yield ExceptionApproved on {test_id}"
        );
        if matches!(
            result.effectiveness,
            Effectiveness::Ineffective | Effectiveness::InsufficientEvidence
        ) {
            saw_gap = true;
        }
    }
    assert!(
        saw_gap,
        "PER-014: expired exception must not suppress the underlying fail/missing"
    );

    let mut overdue = load_personnel_fixture("one-overdue-user");
    let first = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &overdue);
    assert_eq!(first.effectiveness, Effectiveness::Ineffective);
    let subject = named_subjects(&first)
        .into_iter()
        .next()
        .expect("PER-014: overdue fixture must name a failing subject");
    overdue.insert_exception(bound_exception(
        "control.personnel.role-specific-training",
        &subject,
    ));
    let approved = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &overdue);
    assert_eq!(
        approved.effectiveness,
        Effectiveness::ExceptionApproved,
        "PER-014: approved unexpired bound exception is ExceptionApproved, not silent Effective; {}",
        approved.rationale
    );
}

/// PER-015: missing / non-authoritative personnel source never Effective.
#[test]
fn per_015_missing_personnel_source_never_effective() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("missing-personnel-source");
    for test_id in [
        "test.personnel.training-current-all",
        "test.personnel.awareness-current-all",
        "test.personnel.no-leaver-active-access",
        "test.personnel.screening-recorded",
        "test.personnel.mover-privileges-reduced",
        "test.personnel.asset-return-recorded",
    ] {
        if !catalog.tests().contains_key(test_id) {
            panic!("PER-015: catalog missing `{test_id}`");
        }
        let result = evaluate_catalog_test(&catalog, test_id, &set);
        never_effective(&result, &format!("PER-015 {test_id}"));
        assert!(
            matches!(
                result.effectiveness,
                Effectiveness::InsufficientEvidence | Effectiveness::Inconclusive
            ),
            "PER-015: {test_id} on missing source must be InsufficientEvidence/Inconclusive, got {:?} ({})",
            result.effectiveness,
            result.rationale
        );
    }
}

/// PER-016: screening envelopes are facts; collectors stay isolated.
#[test]
fn per_016_screening_facts_and_collector_isolation() {
    require_eight_fixtures();
    let catalog = require_personnel_slice();
    let set = load_personnel_fixture("manual-screening-evidence");
    let types = fixture_types(&set);
    assert!(
        types.contains("evidence.personnel.screening"),
        "PER-016: manual-screening-evidence must emit screening facts"
    );
    for env in set.iter() {
        if env.observation().evidence_type().as_str() != "evidence.personnel.screening" {
            continue;
        }
        let facts = env.observation().facts();
        assert!(
            facts.contains_key("recorded") || facts.contains_key("screened_at"),
            "PER-016: screening envelopes record recorded/screened_at, not cleared/compliant"
        );
        assert!(
            !facts.contains_key("cleared") && !facts.contains_key("compliant"),
            "PER-016: screening must not carry conclusion keys"
        );
    }
    let result = evaluate_catalog_test(&catalog, "test.personnel.screening-recorded", &set);
    assert!(
        !looks_like_compliance_claim(&result.rationale),
        "PER-016: screening result rationale must stay a fact evaluation, got {}",
        result.rationale
    );
    assert!(
        !matches!(result.effectiveness, Effectiveness::NotTested),
        "PER-016: screening-recorded must actually evaluate the recorded/screened_at facts; {}",
        result.rationale
    );

    let collector_src = crate_src("weeping-angel-collector");
    assert!(collector_src.join("github").is_dir());
    assert!(collector_src.join("local").is_dir());
    for live in ["hris", "idp", "lms", "mdm"] {
        assert!(
            !collector_src.join(live).exists(),
            "PER-016: live `{live}` adapter is out of scope"
        );
    }
    let collector_toml = crate_toml("weeping-angel-collector");
    assert!(!collector_toml.contains("weeping-angel-control-test"));
    assert!(!collector_toml.contains("weeping-angel-framework"));
    let control_test_toml = crate_toml("weeping-angel-control-test");
    let framework_toml = crate_toml("weeping-angel-framework");
    assert!(!control_test_toml.contains("weeping-angel-collector"));
    assert!(!framework_toml.contains("weeping-angel-collector"));
    assert!(looks_like_compliance_claim("iso 27001 compliant"));
    assert!(looks_like_compliance_claim("control test result"));

    let events = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !events.contains("fn resolve_personnel_inventory"),
        "PER-016: control-test stays on resolve_population"
    );
}
