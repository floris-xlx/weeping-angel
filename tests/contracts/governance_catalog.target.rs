//! Target suite for governance catalog (governance / risk / personnel / vendor /
//! incident / continuity-governance catalog).
//!
//! Encodes DESIRED behavior in
//! `docs/specs/governance-canonical-assurance-catalog.md` §4 / §5
//! (GOV-001…016). Must stay RED on the current tree: no
//! `control.governance.*` family, no first-class `evidence.manual.attestation`,
//! and no `fixtures/assurance/canonical/v1/governance/*`. Do not `#[ignore]`
//! these tests and do not implement catalog content here.
//!
//! Consumes `CanonicalCatalog::{load,validate,digest}`, `EvidenceValue::with_value`,
//! population runtime population evaluation, and IR `Exception` / `Risk`. Does not fork
//! a second loader, value enum, personnel/vendor resolver, or exception engine.
//!
//! Scan **catalog TOML and product crates** only (xylex-sdd AC-2 / I4a). Never
//! read this file and assert it lacks a substring that also appears in the
//! assertion.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssetId, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus, SelectorScope,
    SubjectKind,
};
use weeping_angel_canonical_catalog::{CATALOG_SCHEMA, CanonicalCatalog, DIGEST_PREFIX};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceError, EvidenceObservation, EvidenceProvenance, EvidenceType,
    EvidenceValue,
};
use weeping_angel_framework::load_framework_pack;

const PINNED_FIXTURE_CONTROL: &str = "control.source.protected-branch";

const PROMPT08_CONTROLS: &[&str] = &[
    "control.governance.information-security-policy",
    "control.governance.policy-review",
    "control.governance.roles-and-responsibilities",
    "control.governance.security-objectives",
    "control.governance.documented-scope",
    "control.governance.internal-audit",
    "control.governance.management-review",
    "control.governance.corrective-action",
    "control.governance.continual-improvement",
    "control.governance.data-classification-policy",
    "control.governance.acceptable-use-policy",
    "control.governance.asset-ownership",
    "control.governance.document-control",
    "control.governance.evidence-retention",
    "control.governance.audit-program",
    "control.risk.assessment",
    "control.risk.treatment",
    "control.risk.ownership",
    "control.risk.acceptance",
    "control.incident.response-plan",
    "control.incident.exercise",
    "control.incident.postmortem",
    "control.personnel.security-awareness",
    "control.personnel.role-specific-training",
    "control.personnel.onboarding-offboarding",
    "control.personnel.confidentiality-commitment",
    "control.personnel.policy-acknowledgement",
    "control.vendor.inventory",
    "control.vendor.risk-review",
    "control.vendor.security-requirements",
    "control.vendor.reassessment",
    "control.vendor.cloud-governance",
    "control.resilience.business-continuity-plan",
    "control.resilience.disaster-recovery-governance",
];

const PROMPT08_EVIDENCE: &[&str] = &[
    "evidence.manual.attestation",
    "evidence.governance.policy",
    "evidence.governance.policy-review",
    "evidence.governance.management-review",
    "evidence.governance.internal-audit",
    "evidence.risk.assessment",
    "evidence.risk.treatment",
    "evidence.personnel.training",
    "evidence.personnel.acknowledgement",
    "evidence.vendor.inventory",
    "evidence.vendor.risk-review",
    "evidence.incident.exercise",
    "evidence.resilience.continuity-plan",
];

const REQUIRED_PROMPT08_TESTS: &[&str] = &[
    "test.governance.policy-current",
    "test.governance.management-review-current",
    "test.governance.internal-audit-current",
    "test.personnel.training-current-all",
    "test.vendor.critical-risk-review-current",
    "test.incident.exercise-current",
];

const PROMPT08_TESTS: &[&str] = &[
    "test.governance.policy-current",
    "test.governance.policy-review-current",
    "test.governance.roles-attested",
    "test.governance.objectives-attested",
    "test.governance.scope-documented",
    "test.governance.internal-audit-current",
    "test.governance.management-review-current",
    "test.governance.corrective-action-recorded",
    "test.governance.improvement-attested",
    "test.governance.classification-policy-current",
    "test.governance.acceptable-use-current",
    "test.governance.asset-ownership-attested",
    "test.governance.document-control-attested",
    "test.governance.retention-attested",
    "test.governance.audit-program-attested",
    "test.risk.assessment-current",
    "test.risk.treatment-current",
    "test.risk.owners-assigned",
    "test.risk.acceptance-attested",
    "test.incident.plan-current",
    "test.incident.exercise-current",
    "test.incident.postmortem-recorded",
    "test.personnel.awareness-current-all",
    "test.personnel.training-current-all",
    "test.personnel.jml-process-attested",
    "test.personnel.confidentiality-acknowledged-all",
    "test.personnel.policy-acknowledged-all",
    "test.vendor.inventory-authoritative",
    "test.vendor.critical-risk-review-current",
    "test.vendor.requirements-attested",
    "test.vendor.reassessment-current",
    "test.vendor.cloud-governance-attested",
    "test.resilience.continuity-plan-current",
    "test.resilience.dr-governance-attested",
];

const GOVERNANCE_FIXTURES: &[&str] = &[
    "current-documents",
    "stale-documents",
    "missing-documents",
    "incomplete-training-population",
    "vendor-review-gaps",
    "approved-exception",
    "expired-exception",
    "manual-review-despite-evidence",
];

const GOVERNANCE_FAMILY_FILES: &[&str] = &[
    "governance.toml",
    "risk.toml",
    "personnel.toml",
    "vendor.toml",
    "incident.toml",
    "continuity.toml",
];

const HYBRID_OR_MANUAL_CONTROLS: &[&str] = &[
    "control.governance.roles-and-responsibilities",
    "control.governance.security-objectives",
    "control.governance.continual-improvement",
    "control.governance.corrective-action",
    "control.governance.document-control",
    "control.governance.evidence-retention",
    "control.governance.audit-program",
    "control.risk.acceptance",
    "control.incident.response-plan",
    "control.incident.postmortem",
    "control.personnel.onboarding-offboarding",
    "control.vendor.security-requirements",
    "control.vendor.cloud-governance",
    "control.resilience.disaster-recovery-governance",
];

const HONEST_MANUAL_TESTS: &[&str] = &[
    "test.governance.roles-attested",
    "test.governance.objectives-attested",
    "test.governance.improvement-attested",
    "test.personnel.jml-process-attested",
    "test.vendor.requirements-attested",
    "test.vendor.cloud-governance-attested",
    "test.resilience.dr-governance-attested",
];

const POPULATION_TESTS: &[&str] = &[
    "test.personnel.training-current-all",
    "test.personnel.awareness-current-all",
    "test.personnel.confidentiality-acknowledged-all",
    "test.personnel.policy-acknowledged-all",
    "test.vendor.critical-risk-review-current",
];

const FRESHNESS_TESTS: &[&str] = &[
    "test.governance.policy-current",
    "test.governance.management-review-current",
    "test.governance.internal-audit-current",
    "test.incident.exercise-current",
];

const CANONICAL_IDENTITY_PINS: &[&str] = &[
    "control.identity.mfa",
    "control.identity.privileged-mfa",
    "evidence.identity.mfa-status",
    "test.identity.privileged-mfa-enabled",
];

const ISO_ORG_CONTROLS: &[&str] = &[
    "incident.response-process",
    "supplier.security-assessment",
    "personnel.access-termination",
    "access.periodic-review",
];

const ISO_ORG_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.5.19", "supplier.security-assessment"),
    ("iso27001:a.5.24", "incident.response-process"),
    ("iso27001:5.2", "incident.response-process"),
    ("iso27001:a.5.1", "incident.response-process"),
    ("iso27001:a.5.16", "personnel.access-termination"),
    ("iso27001:a.6.5", "personnel.access-termination"),
    ("iso27001:a.5.18", "access.periodic-review"),
];

const FORBIDDEN_GRC_TOKENS: &[&str] = &["vanta", "drata", "servicenow", "jira"];

const FORBIDDEN_PROVIDER_TOKENS: &[&str] = &[
    "okta",
    "entra",
    "azure-ad",
    "github",
    "aws",
    "gcp",
    "google-workspace",
    "cognito",
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

const FORBIDDEN_SIBLING_IDS: &[&str] = &[
    "evidence.secret.exposure",
    "evidence.vulnerability.exception",
    "evidence.resilience.recovery-plan",
    "control.resilience.recovery-procedure",
    "control.resilience.disaster-recovery-exercise",
    "control.resilience.redundancy",
    "control.resilience.recovery-objectives",
    "control.resilience.recovery-evidence-freshness",
    "control.source.secure-development-policy",
];

const SHARED_ATTESTATION_FACTS: &[&str] = &[
    "subject_id",
    "attested_by",
    "attested_at",
    "kind",
    "artifact_ref",
    "review_state",
    "valid_until",
];

const POPULATION_OPS: &[&str] = &[
    "all-subjects",
    "all_subjects",
    "AllSubjects",
    "coverage-at-least",
    "coverage_at_least",
    "CoverageAtLeast",
    "none-subjects",
    "none_subjects",
    "fresh-within",
    "manual-review",
    "manual_review",
    "ManualReview",
];

const CONCLUSION_PHRASES: &[&str] = &[
    "isms certified",
    "control effective",
    "management review passed",
    "audit passed",
    "supplier approved",
    "ir capability proven",
];

const IAM_FIXTURES: &[&str] = &[
    "healthy-org",
    "privileged-without-mfa",
    "inactive-admin-active",
    "terminated-employee-active",
    "service-account-without-owner",
    "partial-inventory",
    "stale-access-review",
    "break-glass-approved-exception",
];

fn manifest_dir() -> PathBuf {
    option_env!("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"))
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

fn catalog_v1_dir() -> PathBuf {
    let dir = manifest_dir().join("catalog/canonical/v1");
    assert!(
        dir.is_dir(),
        "GOV-001: catalog infrastructure catalog tree catalog/canonical/v1 must exist"
    );
    dir
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1_dir()).unwrap_or_else(|e| {
        panic!("GOV-001: CanonicalCatalog::load/validate must accept the governance slice: {e}")
    })
}

fn require_governance_family() -> CanonicalCatalog {
    let catalog = load_catalog();
    catalog
        .control("control.governance.information-security-policy")
        .unwrap_or_else(|e| {
            panic!(
                "GOV family missing: `control.governance.information-security-policy` is not loaded ({e}). \
                 Current tree still has only fixture.example + identity."
            )
        });
    assert!(
        catalog
            .evidence()
            .contains_key("evidence.manual.attestation"),
        "GOV family missing: `evidence.manual.attestation` is not declared"
    );
    catalog
}

fn catalog_toml_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(&catalog_v1_dir(), "toml", &mut files);
    assert!(
        !files.is_empty(),
        "GOV-001: catalog/canonical/v1 must contain TOML documents"
    );
    files
}

fn is_governance_family_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    GOVERNANCE_FAMILY_FILES.contains(&name)
}

fn is_governance_family_id(id: &str) -> bool {
    id.starts_with("control.governance.")
        || id.starts_with("control.risk.")
        || id.starts_with("control.personnel.")
        || id.starts_with("control.vendor.")
        || id.starts_with("control.incident.")
        || id == "control.resilience.business-continuity-plan"
        || id == "control.resilience.disaster-recovery-governance"
        || id.starts_with("evidence.governance.")
        || id.starts_with("evidence.risk.")
        || id.starts_with("evidence.personnel.")
        || id.starts_with("evidence.vendor.")
        || id.starts_with("evidence.incident.")
        || id == "evidence.resilience.continuity-plan"
        || id == "evidence.manual.attestation"
        || id.starts_with("test.governance.")
        || id.starts_with("test.risk.")
        || id.starts_with("test.personnel.")
        || id.starts_with("test.vendor.")
        || id.starts_with("test.incident.")
        || id.starts_with("test.resilience.")
}

fn governance_catalog_text() -> String {
    let mut chunks = Vec::new();
    for path in catalog_toml_files() {
        let text = fs::read_to_string(&path).unwrap();
        if is_governance_family_path(&path)
            || text.contains("control.governance.")
            || text.contains("evidence.manual.attestation")
            || text.contains("test.personnel.training-current-all")
        {
            chunks.push(text);
        }
    }
    assert!(
        !chunks.is_empty(),
        "GOV-003: governance family documents (control.governance|risk|personnel|vendor|incident.*) must exist under catalog/canonical/v1"
    );
    chunks.join("\n")
}

fn quoted_ids(text: &str, prefix: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find(prefix) {
        let slice = &rest[start..];
        let end = slice
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
            .unwrap_or(slice.len());
        let id = &slice[..end];
        if id.matches('.').count() >= 2 {
            ids.insert(id.to_string());
        }
        rest = &slice[prefix.len()..];
    }
    ids
}

fn slice_control_ids(catalog: &CanonicalCatalog) -> Vec<String> {
    catalog
        .controls()
        .keys()
        .filter(|id| is_governance_family_id(id))
        .cloned()
        .collect()
}

fn fixture_root() -> PathBuf {
    manifest_dir().join("fixtures/assurance/canonical/v1/governance")
}

fn fixture_dir(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn require_eight_fixtures() {
    assert!(
        fixture_root().is_dir(),
        "GOV-010: fixtures/assurance/canonical/v1/governance must exist"
    );
    for name in GOVERNANCE_FIXTURES {
        let dir = fixture_dir(name);
        assert!(
            dir.is_dir(),
            "GOV-010: fixture `{name}` is not shipped at {}",
            dir.display()
        );
        let evidence = dir.join("evidence.json");
        assert!(
            evidence.is_file(),
            "GOV-010: fixture `{name}` must ship evidence.json at {}",
            evidence.display()
        );
        let blob = fs::read_to_string(&evidence).unwrap();
        assert!(
            blob.contains("evidence.governance.")
                || blob.contains("evidence.personnel.")
                || blob.contains("evidence.vendor.")
                || blob.contains("evidence.incident.")
                || blob.contains("evidence.manual.attestation")
                || blob.contains("evidence.resilience.continuity-plan")
                || blob.contains("exception"),
            "GOV-010: fixture `{name}` must emit canonical governance-family facts"
        );
        assert!(
            !blob.contains("policy.security.reviewed")
                && !blob.contains("policy.supplier.assessed")
                && !blob.contains("evidence.github."),
            "GOV-010: fixture `{name}` must not emit ISO-pack or GRC-product evidence types"
        );
        for token in FORBIDDEN_GRC_TOKENS {
            assert!(
                !blob.to_ascii_lowercase().contains(token),
                "GOV-010: fixture `{name}` must not name a GRC product"
            );
        }
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap(),
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
            collector_id: "fixture.governance-target".into(),
            collected_at: at,
            scope: "target".into(),
            asset: AssetId::new(asset),
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

fn result_json(
    test_id: &str,
    control_id: &str,
    expr: TestExpr,
    set: &EvidenceSet,
) -> (ControlTestResult, Value) {
    let result = evaluate(&compiled(test_id, control_id, expr), set, &fresh_context());
    let json = serde_json::to_value(&result).unwrap();
    (result, json)
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

fn load_fixture_set(name: &str) -> EvidenceSet {
    let dir = fixture_dir(name);
    assert!(
        dir.join("evidence.json").is_file(),
        "GOV-010: fixture `{name}` missing evidence.json"
    );
    let raw = fs::read_to_string(dir.join("evidence.json")).unwrap();
    let doc: Value = serde_json::from_str(&raw).unwrap();
    let collected_at = doc
        .get("collectedAt")
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|ts| ts.with_timezone(&Utc))
        .unwrap_or_else(|| collected(1));
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
            .unwrap_or("org:governance");
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
    if let Some(rest) = trimmed.strip_prefix('P') {
        if let Some(days) = rest.strip_suffix('D') {
            let n: u64 = days.parse().unwrap_or(365);
            return Duration::from_secs(n * 24 * 3600);
        }
    }
    Duration::from_secs(trimmed.parse().unwrap_or(365 * 24 * 3600))
}

fn catalog_test_expr(catalog: &CanonicalCatalog, test_id: &str) -> (String, TestExpr) {
    let test = catalog.tests().get(test_id).unwrap_or_else(|| {
        panic!("GOV-005: catalog missing test `{test_id}` (governance family not landed)")
    });
    (
        test.control.clone(),
        expr_from_map(&test.expression, test_id, &test.subjects),
    )
}

fn expr_from_map(
    expression: &BTreeMap<String, toml::Value>,
    test_id: &str,
    subjects: &[BTreeMap<String, toml::Value>],
) -> TestExpr {
    let op = expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("GOV-005: {test_id} must declare [test.expression].op"));
    if op == "manual-review" || op == "manual_review" {
        return TestExpr::ManualReview;
    }
    if op == "all" || op == "any" {
        let children = expression
            .get("of")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("GOV-005: {test_id} compound `{op}` needs `of`"));
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
        .unwrap_or_else(|| panic!("GOV-005: {test_id} must declare expression.evidence"));
    let field = expression
        .get("field")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let kind = subjects
        .first()
        .and_then(|row| row.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("organization");
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
        other => panic!("GOV-005: {test_id} uses unsupported op `{other}`"),
    }
}

fn evaluate_catalog_test(
    catalog: &CanonicalCatalog,
    test_id: &str,
    set: &EvidenceSet,
) -> (ControlTestResult, Value) {
    let (control, expr) = catalog_test_expr(catalog, test_id);
    result_json(test_id, &control, expr, set)
}

fn control_record_window(text: &str, control_id: &str) -> String {
    let marker = format!("id = \"{control_id}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing control record {control_id}"));
    text[start..start + 900.min(text.len() - start)].to_string()
}

fn control_record_automation(text: &str, control_id: &str) -> String {
    let window = control_record_window(text, control_id);
    for key in ["automation", "class", "kind"] {
        let needle = format!("{key} = \"");
        if let Some(idx) = window.find(&needle) {
            let rest = &window[idx + needle.len()..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_ascii_lowercase();
            }
        }
    }
    panic!("{control_id} must declare automation/class/kind (automated|hybrid|manual)");
}

fn test_expression_window(text: &str, test_id: &str) -> String {
    let marker = format!("id = \"{test_id}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing test record {test_id}"));
    text[start..start + 1400.min(text.len() - start)].to_string()
}

fn expression_is_existence_only(window: &str) -> bool {
    let lower = window.to_ascii_lowercase();
    let has_exists = lower.contains("op = \"exists\"") || lower.contains("exists(");
    let has_population_or_fresh = POPULATION_OPS
        .iter()
        .any(|op| window.contains(op) || lower.contains(&op.to_ascii_lowercase()));
    has_exists && !has_population_or_fresh
}

fn id_has_token(id: &str, token: &str) -> bool {
    id.split('.')
        .any(|seg| seg == token || seg.split('-').any(|part| part == token))
}

fn hyphenated_catalog_id(id: &str) -> bool {
    id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

fn allowed_control_prefix(id: &str) -> bool {
    id.starts_with("control.governance.")
        || id.starts_with("control.risk.")
        || id.starts_with("control.personnel.")
        || id.starts_with("control.vendor.")
        || id.starts_with("control.incident.")
        || id.starts_with("control.resilience.")
}

fn bound_exception(control: &str, kind: SubjectKind, subject: &str) -> Exception {
    let mut ex = Exception::new(
        ExceptionId::new(format!("exc:{subject}")),
        "approved unexpired organizational waiver",
    );
    ex.status = ExceptionStatus::Approved;
    ex.control_id = Some(ControlId::new(control));
    ex.expires_at = Some(fresh_context().now + chrono::Duration::days(30));
    let mut ids = BTreeSet::new();
    ids.insert(subject.into());
    ex.subjects
        .push(weeping_angel_assurance_ir::SubjectSelector {
            kind,
            ids,
            tags: BTreeMap::new(),
            scope: SelectorScope::AnyOf,
        });
    ex
}

#[test]
fn gov_000_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_governance_catalog_baseline")
            && toml.contains("tests/contracts/governance_catalog.baseline.rs")
            && toml.contains("sdd_governance_catalog_target")
            && toml.contains("tests/contracts/governance_catalog.target.rs"),
        "governance catalog dual-suite must be listed in root Cargo.toml (tests/contracts/*.rs is not auto-discovered)"
    );
}

#[test]
fn gov_001_catalog_loader_loads_governance_family_offline() {
    let catalog = require_governance_family();
    catalog
        .validate()
        .expect("GOV-001: CanonicalCatalog::validate must accept the governance slice");
    assert_eq!(CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1");
    catalog
        .control(PINNED_FIXTURE_CONTROL)
        .expect("GOV-001: fixture.example control remains after the family lands");

    let manifest = fs::read_to_string(catalog_v1_dir().join("manifest.toml")).unwrap();
    assert!(
        manifest.contains("governance.toml")
            || manifest.contains("personnel.toml")
            || manifest.contains("vendor.toml")
            || manifest.contains("incident.toml"),
        "GOV-001: manifest.toml [files] must list the governance-family documents"
    );
    assert!(
        !catalog_v1_dir().join("controls/resilience.toml").is_file()
            || !manifest.contains("controls/resilience.toml")
            || catalog
                .control("control.resilience.business-continuity-plan")
                .is_ok(),
        "GOV-001: this slice must not invent infrastructure catalog resilience.toml as its home"
    );

    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    assert_eq!(
        rust.matches("struct CanonicalCatalog").count(),
        1,
        "GOV-001: consume the single catalog infrastructure loader; do not invent a second one"
    );
}

#[test]
fn gov_002_governance_slice_digest_is_deterministic() {
    let catalog = require_governance_family();
    let digest = catalog.digest().expect("GOV-002: digest");
    assert!(
        digest.to_string().starts_with(DIGEST_PREFIX),
        "GOV-002: digest must use {DIGEST_PREFIX}, got {digest}"
    );
    let again = load_catalog();
    assert_eq!(
        digest.to_string(),
        again.digest().expect("digest").to_string(),
        "GOV-002: CanonicalCatalog::digest is deterministic with governance files listed"
    );
}

#[test]
fn gov_003_thirty_six_independently_assessable_controls() {
    let catalog = require_governance_family();
    let text = governance_catalog_text();
    for id in PROMPT08_CONTROLS {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("GOV-003: missing control `{id}`"));
        let class = control_record_automation(&text, id);
        assert!(
            matches!(class.as_str(), "automated" | "hybrid" | "manual"),
            "GOV-003: {id} automation class must be Automated|Hybrid|Manual, got {class}"
        );
        let window = control_record_window(&text, id);
        assert!(
            window.contains("domains") || window.contains("domain"),
            "GOV-003: {id} must declare domain(s)"
        );
        assert!(
            window.contains("evidence"),
            "GOV-003: {id} must declare evidence requirements"
        );
        assert!(
            window.contains("tests") || window.contains("test"),
            "GOV-003: {id} must declare test refs"
        );
    }

    let slice = slice_control_ids(&catalog);
    assert!(
        (30..=45).contains(&slice.len()),
        "GOV-003: expected 30–45 independently assessable governance-family controls, found {} ({slice:?})",
        slice.len()
    );
    for id in &slice {
        assert!(
            allowed_control_prefix(id),
            "GOV-003: unexpected control prefix `{id}`"
        );
        assert!(
            hyphenated_catalog_id(id),
            "GOV-003: ids are lowercase hyphenated ({id})"
        );
        assert!(
            !id.contains('_'),
            "GOV-003: catalog ids use hyphen segments, not underscores ({id})"
        );
    }
}

#[test]
fn gov_004_manual_attestation_and_domain_evidence_are_facts() {
    let catalog = require_governance_family();
    for id in PROMPT08_EVIDENCE {
        assert!(
            catalog.evidence().contains_key(*id),
            "GOV-004: missing evidence contract `{id}`"
        );
    }
    let text = governance_catalog_text();
    let lower = text.to_ascii_lowercase();
    for phrase in CONCLUSION_PHRASES {
        assert!(
            !lower.contains(phrase),
            "GOV-004: evidence contracts are facts, not conclusions (`{phrase}`)"
        );
    }
    for fact in SHARED_ATTESTATION_FACTS {
        assert!(
            lower.contains(fact),
            "GOV-004: shared attestation shape must declare `{fact}` (principal/timestamp/subject/artifact/freshness/review state)"
        );
    }
    for id in PROMPT08_EVIDENCE {
        let referenced = catalog
            .controls()
            .values()
            .any(|c| c.evidence.iter().any(|ev| ev == id))
            || catalog
                .tests()
                .values()
                .any(|t| t.required_evidence.iter().any(|ev| ev == id));
        assert!(referenced, "GOV-004: evidence `{id}` must not be orphaned");
    }
}

#[test]
fn gov_005_required_freshness_and_population_tests_are_declared() {
    let catalog = require_governance_family();
    let text = governance_catalog_text();
    for id in REQUIRED_PROMPT08_TESTS.iter().chain(PROMPT08_TESTS.iter()) {
        assert!(
            catalog.tests().contains_key(*id),
            "GOV-005: missing test `{id}`"
        );
        let window = test_expression_window(&text, id);
        assert!(
            window.contains("control.") || window.contains("control ="),
            "GOV-005: {id} must reference a control"
        );
    }
    for id in FRESHNESS_TESTS {
        let window = test_expression_window(&text, id);
        assert!(
            !expression_is_existence_only(&window),
            "GOV-005: {id} must not be Exists(one PDF)"
        );
        assert!(
            window.contains("fresh-within")
                || window.contains("all-subjects")
                || window.contains("reviewed_at")
                || window.contains("exercised_at")
                || window.contains("audited_at"),
            "GOV-005: {id} must be a freshness predicate, not document-present"
        );
    }
    for id in POPULATION_TESTS {
        let window = test_expression_window(&text, id);
        assert!(
            !expression_is_existence_only(&window),
            "GOV-005: {id} must not be Exists(one envelope)"
        );
        assert!(
            window.contains("all-subjects")
                || window.contains("coverage-at-least")
                || window.contains("AllSubjects"),
            "GOV-005: {id} must be a population predicate"
        );
    }
}

#[test]
fn gov_006_catalog_toml_has_no_grc_product_tokens() {
    let _catalog = require_governance_family();
    let text = governance_catalog_text();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_GRC_TOKENS
        .iter()
        .chain(FORBIDDEN_PROVIDER_TOKENS.iter())
    {
        assert!(
            !lower.contains(token),
            "GOV-006: governance catalog TOML must not mention `{token}`"
        );
    }
    for id in quoted_ids(&text, "control.")
        .into_iter()
        .chain(quoted_ids(&text, "evidence."))
        .chain(quoted_ids(&text, "test."))
    {
        if !is_governance_family_id(&id) && !id.starts_with("control.resilience.") {
            continue;
        }
        for token in FORBIDDEN_GRC_TOKENS
            .iter()
            .chain(FORBIDDEN_PROVIDER_TOKENS.iter())
        {
            assert!(
                !id_has_token(&id, token),
                "GOV-006: provider/GRC token `{token}` leaked into id `{id}`"
            );
        }
    }
}

#[test]
fn gov_007_catalog_toml_has_no_framework_tokens() {
    let _catalog = require_governance_family();
    let text = governance_catalog_text();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            !lower.contains(token),
            "GOV-007: canonical governance TOML must not mention `{token}`"
        );
    }
    for id in quoted_ids(&text, "control.")
        .into_iter()
        .chain(quoted_ids(&text, "evidence."))
        .chain(quoted_ids(&text, "test."))
    {
        if !is_governance_family_id(&id) {
            continue;
        }
        for token in FORBIDDEN_FRAMEWORK_TOKENS {
            assert!(
                !id_has_token(&id, token),
                "GOV-007: framework token `{token}` leaked into id `{id}`"
            );
        }
    }
}

#[test]
fn gov_008_iso_pack_sliver_unchanged_and_has_no_control_governance() {
    let catalog = require_governance_family();
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    assert!(
        !control_ids
            .iter()
            .any(|id| id.starts_with("control.governance.")
                || id.starts_with("control.risk.")
                || id.starts_with("control.vendor.")),
        "GOV-008: do not move governance controls into the ISO pack"
    );
    for id in ISO_ORG_CONTROLS {
        assert!(
            !control_ids.contains(id),
            "GOV-008: pack-local sliver `{id}` was remapped by ISO remap; do not reintroduce it"
        );
    }

    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    let _ = ISO_ORG_MAPPINGS;
    assert!(
        !mappings.contains("control.governance.")
            && !mappings.contains("to = \"control.incident.")
            && !mappings.contains("to = \"control.vendor."),
        "GOV-008: ISO mappings must not retarget the canonical governance family"
    );
    assert!(
        catalog.control("control.incident.response-plan").is_ok(),
        "GOV-008: catalog owns control.incident.response-plan beside the remapped pack projection"
    );
}

#[test]
fn gov_009_training_current_all_is_population_not_existence() {
    let catalog = require_governance_family();
    require_eight_fixtures();
    let text = governance_catalog_text();
    let window = test_expression_window(&text, "test.personnel.training-current-all");
    assert!(
        !expression_is_existence_only(&window),
        "GOV-009: test.personnel.training-current-all must not be Exists(some training envelope)"
    );

    let incomplete = load_fixture_set("incomplete-training-population");
    let (pop, json) =
        evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &incomplete);
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "GOV-009: a partial training population cannot be Effective; json={json}"
    );
    assert_eq!(
        pop.effectiveness,
        Effectiveness::InsufficientEvidence,
        "GOV-009: missing training for a known subject is InsufficientEvidence, got {:?}",
        pop.effectiveness
    );
}

#[test]
fn gov_010_eight_fixtures_distinguish_missing_stale_fail_manual_exception() {
    let catalog = require_governance_family();
    require_eight_fixtures();

    let current = load_fixture_set("current-documents");
    for test_id in FRESHNESS_TESTS {
        let (ok, json) = evaluate_catalog_test(&catalog, test_id, &current);
        assert_eq!(
            ok.effectiveness,
            Effectiveness::Effective,
            "GOV-010: current-documents must satisfy `{test_id}` when typed current records exist; json={json}"
        );
    }
    let (trained, _) =
        evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &current);
    assert_eq!(
        trained.effectiveness,
        Effectiveness::Effective,
        "GOV-010: current-documents has a complete current training population"
    );
    let (vendors, _) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &current,
    );
    assert_eq!(
        vendors.effectiveness,
        Effectiveness::Effective,
        "GOV-010: current-documents has current risk reviews for every critical vendor"
    );

    let stale = load_fixture_set("stale-documents");
    for test_id in FRESHNESS_TESTS {
        let (stale_r, json) = evaluate_catalog_test(&catalog, test_id, &stale);
        assert_eq!(
            stale_r.effectiveness,
            Effectiveness::StaleEvidence,
            "GOV-010: stale-documents `{test_id}` must be StaleEvidence, not missing/Effective; json={json}"
        );
    }

    let missing = load_fixture_set("missing-documents");
    for test_id in FRESHNESS_TESTS {
        let (miss, json) = evaluate_catalog_test(&catalog, test_id, &missing);
        assert_eq!(
            miss.effectiveness,
            Effectiveness::InsufficientEvidence,
            "GOV-010: missing-documents `{test_id}` is InsufficientEvidence, not Ineffective; json={json}"
        );
    }

    let incomplete = load_fixture_set("incomplete-training-population");
    let (partial, _) =
        evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &incomplete);
    assert_ne!(partial.effectiveness, Effectiveness::Effective);

    let gaps = load_fixture_set("vendor-review-gaps");
    let (gap, gap_json) =
        evaluate_catalog_test(&catalog, "test.vendor.critical-risk-review-current", &gaps);
    assert_ne!(
        gap.effectiveness,
        Effectiveness::Effective,
        "GOV-010: vendor-review-gaps cannot be Effective; json={gap_json}"
    );

    let approved = load_fixture_set("approved-exception");
    let (ex, ex_json) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &approved,
    );
    assert_eq!(
        ex.effectiveness,
        Effectiveness::ExceptionApproved,
        "GOV-010: approved-exception is ExceptionApproved, never silent Effective; json={ex_json}"
    );
    assert_ne!(ex.effectiveness, Effectiveness::Effective);

    let expired = load_fixture_set("expired-exception");
    let (exp, exp_json) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &expired,
    );
    assert_ne!(
        exp.effectiveness,
        Effectiveness::Effective,
        "GOV-010: expired-exception must not suppress the failing result; json={exp_json}"
    );
    assert_ne!(exp.effectiveness, Effectiveness::ExceptionApproved);

    let manual = load_fixture_set("manual-review-despite-evidence");
    let (man, man_json) =
        evaluate_catalog_test(&catalog, "test.governance.roles-attested", &manual);
    assert_eq!(
        man.effectiveness,
        Effectiveness::ManualReviewRequired,
        "GOV-010: manual-review-despite-evidence stays ManualReviewRequired; json={man_json}"
    );
}

#[test]
fn gov_011_partial_training_and_vendor_populations_cannot_be_effective() {
    let catalog = require_governance_family();
    require_eight_fixtures();

    let incomplete = load_fixture_set("incomplete-training-population");
    let (training, t_json) =
        evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &incomplete);
    assert_ne!(
        training.effectiveness,
        Effectiveness::Effective,
        "GOV-011: incomplete training population cannot be Effective; json={t_json}"
    );
    assert_eq!(
        training.effectiveness,
        Effectiveness::InsufficientEvidence,
        "GOV-011: missing evidence is InsufficientEvidence, got {:?}",
        training.effectiveness
    );

    let gaps = load_fixture_set("vendor-review-gaps");
    let (vendor, v_json) =
        evaluate_catalog_test(&catalog, "test.vendor.critical-risk-review-current", &gaps);
    assert_ne!(
        vendor.effectiveness,
        Effectiveness::Effective,
        "GOV-011: vendor-review gap cannot be Effective on all-subjects; json={v_json}"
    );
    let missing_subjects = string_list(&v_json, "missingSubjects");
    assert!(
        !missing_subjects.is_empty()
            || vendor.effectiveness == Effectiveness::InsufficientEvidence
            || vendor.effectiveness == Effectiveness::Ineffective,
        "GOV-011: result must name the gap or stay InsufficientEvidence; json={v_json}"
    );
}

#[test]
fn gov_012_approved_exception_is_never_silent_effective() {
    let catalog = require_governance_family();
    require_eight_fixtures();

    let approved = load_fixture_set("approved-exception");
    assert!(
        !approved.exceptions().is_empty(),
        "GOV-012: approved-exception fixture must carry a bound IR Exception"
    );
    let (result, json) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &approved,
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::ExceptionApproved,
        "GOV-012: approved unexpired IR exception → ExceptionApproved, got {:?} {json}",
        result.effectiveness
    );
    assert_ne!(result.effectiveness, Effectiveness::Effective);

    let expired = load_fixture_set("expired-exception");
    let (expired_r, exp_json) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &expired,
    );
    assert_ne!(
        expired_r.effectiveness,
        Effectiveness::Effective,
        "GOV-012: expired exception must not suppress failing results; {exp_json}"
    );
    assert_ne!(expired_r.effectiveness, Effectiveness::ExceptionApproved);

    let mut empty_subjects = load_fixture_set("vendor-review-gaps");
    let mut blanket = Exception::new(ExceptionId::new("exc:empty"), "no subjects bound");
    blanket.status = ExceptionStatus::Approved;
    blanket.expires_at = Some(fresh_context().now + chrono::Duration::days(30));
    empty_subjects.insert_exception(blanket);
    let (empty_r, _) = evaluate_catalog_test(
        &catalog,
        "test.vendor.critical-risk-review-current",
        &empty_subjects,
    );
    assert_ne!(
        empty_r.effectiveness,
        Effectiveness::Effective,
        "GOV-012: empty Exception.subjects is not the whole inventory"
    );

    let _ = bound_exception(
        "control.vendor.risk-review",
        SubjectKind::Vendor,
        "vendor:excepted",
    );
}

#[test]
fn gov_013_hybrid_manual_cannot_auto_pass_from_document_present() {
    let catalog = require_governance_family();
    require_eight_fixtures();
    let text = governance_catalog_text();
    for id in HYBRID_OR_MANUAL_CONTROLS {
        let class = control_record_automation(&text, id);
        assert!(
            class == "hybrid" || class == "manual",
            "GOV-013: {id} must stay Hybrid or Manual, got {class}"
        );
    }
    for id in HONEST_MANUAL_TESTS {
        let window = test_expression_window(&text, id);
        assert!(
            window.contains("manual-review") || window.contains("ManualReview"),
            "GOV-013: {id} must use op = \"manual-review\" so a document-present flag cannot auto-pass"
        );
        let (control, expr) = catalog_test_expr(&catalog, id);
        assert!(
            matches!(expr, TestExpr::ManualReview | TestExpr::All(_)),
            "GOV-013: {id} on {control} must not collapse to Exists"
        );
    }

    let manual = load_fixture_set("manual-review-despite-evidence");
    let (result, json) = evaluate_catalog_test(&catalog, "test.governance.roles-attested", &manual);
    assert_eq!(
        result.effectiveness,
        Effectiveness::ManualReviewRequired,
        "GOV-013: supporting evidence without operational review is ManualReviewRequired; json={json}"
    );
}

#[test]
fn gov_014_credential_shaped_facts_and_secrets_are_rejected() {
    require_governance_family();
    require_eight_fixtures();

    let obs = EvidenceObservation::new(EvidenceType::new("evidence.manual.attestation"))
        .with_value("password", EvidenceValue::String("hunter2".into()))
        .with_value("subject_id", EvidenceValue::String("org:acme".into()));
    let err = EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.governance-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("org:acme"),
        },
    )
    .expect_err("GOV-014: password fact must not seal");
    assert!(matches!(err, EvidenceError::CredentialInPayload { .. }));

    let claim = EvidenceObservation::new(EvidenceType::new("evidence.manual.attestation"))
        .with_narrative("control is ISO 27001 compliant");
    let claim_err = EvidenceEnvelope::seal(
        claim,
        EvidenceProvenance {
            collector_id: "fixture.governance-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("org:acme"),
        },
    )
    .expect_err("GOV-014: compliance narrative must not seal");
    assert!(matches!(claim_err, EvidenceError::ComplianceClaim { .. }));

    for name in GOVERNANCE_FIXTURES {
        let blob = fs::read_to_string(fixture_dir(name).join("evidence.json")).unwrap();
        let lower = blob.to_ascii_lowercase();
        for secret_key in ["password", "secret", "api_key", "token", "private_key"] {
            assert!(
                !lower.contains(&format!("\"{secret_key}\"")),
                "GOV-014: fixture `{name}` must not store secret material under `{secret_key}`"
            );
        }
    }
}

#[test]
fn gov_015_iam_fixture_example_and_siblings_remain() {
    let catalog = require_governance_family();
    for id in CANONICAL_IDENTITY_PINS {
        let present = catalog.controls().contains_key(*id)
            || catalog.evidence().contains_key(*id)
            || catalog.tests().contains_key(*id);
        assert!(present, "GOV-015: IAM pin `{id}` must remain");
    }
    catalog
        .control(PINNED_FIXTURE_CONTROL)
        .expect("GOV-015: fixture.example control remains");

    let identity = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    for name in IAM_FIXTURES {
        assert!(
            identity.join(name).join("evidence.json").is_file(),
            "GOV-015: IAM fixture `{name}` remains"
        );
    }

    let text = governance_catalog_text();
    for id in FORBIDDEN_SIBLING_IDS {
        assert!(
            !text.contains(id),
            "GOV-015: this slice must not declare sibling id `{id}`"
        );
    }
    assert!(
        !catalog_v1_dir()
            .join("controls/vulnerability.toml")
            .is_file()
            || !text.contains("evidence.secret.exposure"),
        "GOV-015: do not create vulnerability.toml / evidence.secret.exposure in this slice"
    );

    let rust_files = {
        let mut files = Vec::new();
        walk_files(&manifest_dir().join("crates"), "rs", &mut files);
        files
    };
    for path in rust_files {
        let src = fs::read_to_string(&path).unwrap();
        assert!(
            !src.contains("struct GovernanceCatalog")
                && !src.contains("fn resolve_personnel_inventory")
                && !src.contains("fn resolve_vendor_inventory")
                && !src.contains("struct GovernanceException"),
            "GOV-015: no second loader / population / exception engine in {}",
            path.display()
        );
    }

    let catalog_ssot =
        fs::read_to_string(manifest_dir().join("docs/specs/canonical-assurance-catalog-v1.md"))
            .unwrap();
    assert!(
        catalog_ssot.starts_with("# SDD: Canonical Assurance Catalog v1 infrastructure"),
        "GOV-015: do not overwrite catalog infrastructure SSOT"
    );
}

#[test]
fn gov_016_iso_and_iam_gates_stay_intact() {
    let _catalog = require_governance_family();
    let pack = load_framework_pack("iso-27001", "2022").unwrap();
    let pack_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    assert!(
        !pack_ids
            .iter()
            .any(|id| id.starts_with("control.governance.") || id.starts_with("control.vendor.")),
        "GOV-016: this slice must not grow the ISO pack with governance ids"
    );
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !metadata.contains("control.governance.")
            && !metadata.contains("evidence.manual.attestation"),
        "GOV-016: ISO pack metadata must stay free of governance catalog catalog ids"
    );

    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        cargo.contains("sdd_iso27001_assurance_target") && cargo.contains("sdd_iam_catalog_target"),
        "GOV-016: ISO and IAM target suites stay registered"
    );

    let collector = crate_src("weeping-angel-collector");
    for name in ["vanta", "drata", "servicenow", "jira"] {
        assert!(
            !collector.join(name).exists(),
            "GOV-016: do not add a {name} GRC collector in this slice"
        );
    }
}
