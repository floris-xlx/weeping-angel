//! Target suite for the Infrastructure Canonical Assurance Catalog (infrastructure catalog).
//!
//! Encodes DESIRED behavior in
//! `docs/specs/infrastructure-canonical-assurance-catalog.md` §4 / §5
//! (INFRA-001…016). Must stay RED on the current tree: no
//! `control.network.*` family, no required `evidence.database.*` contracts,
//! and no population fixtures. Do not `#[ignore]` these tests and do not
//! implement catalog content here.
//!
//! Consumes catalog infrastructure `CanonicalCatalog::{load,validate,digest}`, typed evidence
//! `EvidenceValue` / `with_value`, and population runtime `AllSubjects` /
//! `NoneSubjects` / `CoverageAtLeast`. Does not fork a second loader,
//! `EvidenceValue`, or `resolve_database_inventory`.
//!
//! Scan **product** trees only (xylex-sdd AC-2 / I4a). Never read this file
//! and assert it lacks a substring that also appears in the assertion.

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
use weeping_angel_collector::github::GITHUB_EVIDENCE_TYPES;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceError, EvidenceObservation, EvidenceProvenance, EvidenceType,
    EvidenceValue,
};
use weeping_angel_framework::load_framework_pack;

const CANONICAL_INFRA_CONTROLS: &[&str] = &[
    "control.network.admin-interface-restriction",
    "control.network.public-exposure-governance",
    "control.network.segmentation",
    "control.network.firewall-policy-current",
    "control.network.no-unnecessary-public-databases",
    "control.network.management-access-protection",
    "control.network.tls-sensitive-traffic",
    "control.network.insecure-protocol-restriction",
    "control.crypto.encryption-at-rest",
    "control.crypto.encryption-in-transit",
    "control.crypto.key-lifecycle",
    "control.secret.storage",
    "control.secret.credential-storage",
    "control.crypto.key-rotation",
    "control.crypto.certificate-validity",
    "control.crypto.backup-encryption",
    "control.data.production-inventory",
    "control.data.access-restriction",
    "control.data.retention-policy",
    "control.data.sensitive-classification",
    "control.database.inventory",
    "control.database.access-restriction",
    "control.database.encryption",
    "control.database.backup-enabled",
    "control.database.auditing",
    "control.logging.audit-enabled",
    "control.logging.admin-events",
    "control.logging.auth-security-events",
    "control.logging.retention-meets-policy",
    "control.logging.time-synchronization",
    "control.logging.security-alerting",
    "control.logging.privileged-actions-observable",
    "control.logging.integrity-protected-storage",
    "control.logging.monitoring-coverage",
    "control.backup.enabled",
    "control.backup.population-coverage",
    "control.backup.retention",
    "control.backup.restore-testing",
    "control.resilience.recovery-procedure",
    "control.resilience.disaster-recovery-exercise",
    "control.resilience.redundancy",
    "control.resilience.recovery-objectives",
    "control.resilience.recovery-evidence-freshness",
];

const REQUIRED_EVIDENCE: &[&str] = &[
    "evidence.network.exposure",
    "evidence.network.firewall-policy",
    "evidence.network.tls-configuration",
    "evidence.data.encryption-at-rest",
    "evidence.data.encryption-in-transit",
    "evidence.crypto.key-state",
    "evidence.secret.storage-configuration",
    "evidence.database.inventory",
    "evidence.database.access-configuration",
    "evidence.logging.configuration",
    "evidence.logging.retention",
    "evidence.logging.alerting",
    "evidence.backup.configuration",
    "evidence.backup.run",
    "evidence.backup.restore-test",
    "evidence.resilience.recovery-plan",
];

const REQUIRED_POPULATION_TESTS: &[&str] = &[
    "test.database.critical-encrypt-at-rest",
    "test.network.public-endpoints-acceptable-tls",
    "test.logging.critical-assets-audit-current",
    "test.logging.retention-meets-threshold",
    "test.backup.required-stores-current",
    "test.backup.restore-test-fresh",
    "test.network.no-prohibited-public-databases",
    "test.secret.approved-storage",
];

const ALL_INFRA_TESTS: &[&str] = &[
    "test.network.admin-interfaces-restricted",
    "test.network.public-exposure-governed",
    "test.network.segmentation-rationale",
    "test.network.firewall-policy-current",
    "test.network.no-prohibited-public-databases",
    "test.network.management-access-protected",
    "test.network.public-endpoints-acceptable-tls",
    "test.network.insecure-protocols-restricted",
    "test.crypto.encryption-at-rest-enabled",
    "test.crypto.encryption-in-transit-enabled",
    "test.crypto.key-lifecycle-managed",
    "test.secret.approved-storage",
    "test.secret.credentials-approved-storage",
    "test.crypto.keys-rotated",
    "test.crypto.certificates-valid",
    "test.crypto.backups-encrypted",
    "test.data.production-stores-inventoried",
    "test.data.access-restricted",
    "test.data.retention-policy-represented",
    "test.data.sensitive-classification-present",
    "test.database.inventoried",
    "test.database.access-restricted",
    "test.database.critical-encrypt-at-rest",
    "test.database.backup-enabled",
    "test.database.auditing-enabled",
    "test.logging.critical-assets-audit-current",
    "test.logging.admin-events-recorded",
    "test.logging.auth-security-events-recorded",
    "test.logging.retention-meets-threshold",
    "test.logging.time-synchronized",
    "test.logging.alerting-configured",
    "test.logging.privileged-actions-observable",
    "test.logging.integrity-protected",
    "test.logging.monitoring-coverage",
    "test.backup.enabled",
    "test.backup.required-stores-current",
    "test.backup.retention-meets-threshold",
    "test.backup.restore-test-fresh",
    "test.resilience.recovery-procedure-present",
    "test.resilience.dr-exercise-recorded",
    "test.resilience.redundancy-where-required",
    "test.resilience.recovery-objectives-documented",
    "test.resilience.recovery-evidence-fresh",
];

const INFRA_FAMILY_FILES: &[&str] = &[
    "network.toml",
    "crypto.toml",
    "data.toml",
    "database.toml",
    "logging.toml",
    "backup.toml",
    "resilience.toml",
];

const HYBRID_OR_MANUAL_CONTROLS: &[&str] = &[
    "control.network.segmentation",
    "control.network.public-exposure-governance",
    "control.crypto.key-lifecycle",
    "control.data.sensitive-classification",
    "control.data.retention-policy",
    "control.logging.time-synchronization",
    "control.logging.integrity-protected-storage",
    "control.resilience.disaster-recovery-exercise",
    "control.resilience.recovery-objectives",
    "control.resilience.recovery-procedure",
    "control.resilience.redundancy",
];

const HONEST_MANUAL_TESTS: &[&str] = &[
    "test.network.segmentation-rationale",
    "test.resilience.dr-exercise-recorded",
    "test.resilience.recovery-objectives-documented",
];

const INFRA_FIXTURES: &[(&str, &str)] = &[
    ("network", "healthy"),
    ("network", "public-db-exposed"),
    ("network", "insecure-tls"),
    ("network", "partial-inventory"),
    ("network", "stale-firewall-policy"),
    ("network", "exception-approved-exposure"),
    ("crypto", "healthy"),
    ("crypto", "unapproved-secret-storage"),
    ("crypto", "stale-certificate"),
    ("data", "healthy"),
    ("data", "partial-classification"),
    ("database", "healthy"),
    ("database", "unencrypted-critical-db"),
    ("database", "partial-inventory"),
    ("database", "missing-encryption"),
    ("logging", "healthy"),
    ("logging", "retention-below-threshold"),
    ("logging", "stale-audit-log"),
    ("logging", "missing-alerting"),
    ("logging", "partial-coverage"),
    ("backup", "healthy"),
    ("backup", "missing-backup"),
    ("backup", "stale-restore-test"),
    ("backup", "failing-restore"),
    ("resilience", "healthy"),
    ("resilience", "stale-recovery-plan"),
    ("resilience", "missing-dr-exercise"),
    ("resilience", "exception-approved-rto"),
];

const FORBIDDEN_PROVIDER_TOKENS: &[&str] =
    &["aws", "azure", "gcp", "google", "cloudflare", "vercel"];

const FORBIDDEN_FRAMEWORK_TOKENS: &[&str] = &[
    "iso27001",
    "iso-27001",
    "soc2",
    "soc-2",
    "nis2",
    "dora",
    "gdpr",
    "pci",
    "pci-dss",
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
];

const CLASSIFIABLE_FIELDS: &[&str] = &[
    "encrypted",
    "meets_policy",
    "meets_threshold",
    "restricted",
    "approved_storage",
    "audit_enabled",
    "backup_enabled",
    "success",
    "tested_at",
    "ran_at",
    "reviewed_at",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1() -> PathBuf {
    let dir = manifest_dir().join("catalog/canonical/v1");
    assert!(
        dir.is_dir(),
        "INFRA-001: catalog infrastructure catalog tree catalog/canonical/v1 must exist"
    );
    dir
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1()).expect("canonical catalog v1 loads")
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_files(dir, out);
    out.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"));
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

fn product_rs_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    walk_rs_files(&manifest_dir().join("src"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn require_family_files() -> Vec<PathBuf> {
    let root = catalog_v1();
    let mut found = Vec::new();
    for dir in ["controls", "evidence", "tests"] {
        for family in INFRA_FAMILY_FILES {
            let path = root.join(dir).join(family);
            assert!(
                path.is_file(),
                "INFRA-001: missing catalog family file {}",
                path.display()
            );
            found.push(path);
        }
        assert!(
            !root.join(dir).join("secret.toml").exists(),
            "control.secret.* / evidence.secret.storage-configuration live in crypto.toml; do not create {dir}/secret.toml"
        );
        assert!(
            !root.join(dir).join("infrastructure.toml").exists(),
            "use per-family files, not {dir}/infrastructure.toml"
        );
    }
    found
}

fn infrastructure_catalog_text() -> String {
    require_family_files()
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
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

fn fixture_dir(family: &str, name: &str) -> PathBuf {
    manifest_dir()
        .join("fixtures/assurance/canonical/v1")
        .join(family)
        .join(name)
}

fn require_fixture(family: &str, name: &str) -> PathBuf {
    let dir = fixture_dir(family, name);
    assert!(
        dir.join("evidence.json").is_file(),
        "INFRA-010: fixture `{family}/{name}` is not shipped at {}",
        dir.display()
    );
    dir
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
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
            collector_id: "fixture.infrastructure-target".into(),
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
) -> (weeping_angel_control_test::ControlTestResult, Value) {
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

fn load_fixture_set(family: &str, name: &str) -> EvidenceSet {
    let dir = require_fixture(family, name);
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
            .unwrap_or_else(|| panic!("{family}/{name}: evidence row missing type"));
        let subject = row
            .get("subject_id")
            .or_else(|| row.get("subjectId"))
            .and_then(Value::as_str)
            .unwrap_or("org:infra");
        let mut facts = Vec::new();
        if let Some(map) = row.get("facts").and_then(Value::as_object) {
            for (k, v) in map {
                facts.push((k.as_str(), json_fact(v)));
            }
        }
        let owned: Vec<(String, EvidenceValue)> =
            facts.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
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
    set
}

fn catalog_test_expr(catalog: &CanonicalCatalog, test_id: &str) -> (String, TestExpr) {
    let test = catalog
        .tests()
        .get(test_id)
        .unwrap_or_else(|| panic!("INFRA-005: catalog missing test `{test_id}`"));
    let op = test
        .expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("INFRA-005: {test_id} must declare [test.expression].op"));
    if op == "manual-review" || op == "manual_review" {
        return (test.control.clone(), TestExpr::ManualReview);
    }
    let evidence = test
        .expression
        .get("evidence")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("INFRA-005: {test_id} must declare expression.evidence"));
    let field = test
        .expression
        .get("field")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let kind = test
        .subjects
        .first()
        .and_then(|row| row.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("database");
    let selector = SubjectSelector {
        kind: Some(kind.into()),
        id: None,
    };
    let evidence_sel = EvidenceSelector {
        evidence_type: EvidenceType::new(evidence),
        subject_selector: selector.clone(),
        field,
        freshness: None,
    };
    let expr = match op {
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
                percentage: test
                    .expression
                    .get("percentage")
                    .and_then(|v| {
                        v.as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| v.as_integer().map(|n| n.to_string()))
                    })
                    .unwrap_or_else(|| "100".into()),
            }
        }
        other => panic!("INFRA-005: {test_id} uses unsupported op `{other}`"),
    };
    (test.control.clone(), expr)
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
    let has_population = POPULATION_OPS
        .iter()
        .any(|op| window.contains(op) || lower.contains(&op.to_ascii_lowercase()));
    has_exists && !has_population
}

fn bound_exception(control: &str, kind: SubjectKind, subject: &str) -> Exception {
    let mut ex = Exception::new(
        ExceptionId::new(format!("exc:{subject}")),
        "approved unexpired infrastructure exception",
    );
    ex.status = ExceptionStatus::Approved;
    ex.control_id = Some(ControlId::new(control));
    ex.expires_at = Some(fresh_context().now + chrono::Duration::hours(24));
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
fn infra_000_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_infrastructure_catalog_baseline")
            && toml.contains("tests/contracts/infrastructure_catalog.baseline.rs")
            && toml.contains("sdd_infrastructure_catalog_target")
            && toml.contains("tests/contracts/infrastructure_catalog.target.rs"),
        "dual-suite sdd_infrastructure_catalog_baseline + sdd_infrastructure_catalog_target must be listed in root Cargo.toml"
    );
}

#[test]
fn infra_001_catalog_loader_loads_infrastructure_family_offline() {
    require_family_files();
    let crate_dir = manifest_dir().join("crates/weeping-angel-canonical-catalog");
    assert!(
        crate_dir.is_dir(),
        "INFRA-001: consume catalog infrastructure crate weeping-angel-canonical-catalog; do not invent a second loader"
    );
    let rust = product_rs_joined();
    for needle in [
        "struct CanonicalCatalog",
        "fn load",
        "fn validate",
        "fn digest",
        "weeping-angel/canonical-catalog/v1",
    ] {
        assert!(
            rust.contains(needle),
            "INFRA-001: CanonicalCatalog API missing `{needle}`"
        );
    }
    assert_eq!(CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1");

    let catalog = load_catalog();
    catalog
        .validate()
        .expect("INFRA-001: CanonicalCatalog::validate must accept the infrastructure slice");
    catalog
        .control("control.network.admin-interface-restriction")
        .expect("INFRA-001: loaded catalog must include control.network.*");

    let manifest = fs::read_to_string(catalog_v1().join("manifest.toml")).unwrap();
    for family in INFRA_FAMILY_FILES {
        assert!(
            manifest.contains(&format!("controls/{family}"))
                && manifest.contains(&format!("evidence/{family}"))
                && manifest.contains(&format!("tests/{family}")),
            "INFRA-001: manifest.toml [files] must list `{family}` under controls/evidence/tests"
        );
    }
    for listed in [
        "controls/fixture.example.toml",
        "controls/identity.toml",
        "evidence/fixture.example.toml",
        "evidence/identity.toml",
        "tests/fixture.example.toml",
        "tests/identity.toml",
    ] {
        assert!(
            manifest.contains(listed),
            "INFRA-001: keep listing `{listed}` (do not delete fixture.example or IAM)"
        );
    }
}

#[test]
fn infra_002_infrastructure_slice_digest_is_deterministic() {
    require_family_files();
    let catalog = load_catalog();
    catalog.validate().expect("validate");
    let first = catalog.digest().expect("digest");
    assert!(
        first.to_string().starts_with(DIGEST_PREFIX),
        "INFRA-002: digest must use {DIGEST_PREFIX}, got {first}"
    );
    let again = load_catalog().digest().expect("digest");
    assert_eq!(
        first.to_string(),
        again.to_string(),
        "INFRA-002: CanonicalCatalog::digest is deterministic for the same on-disk tree"
    );
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    assert_eq!(
        rust.matches("struct CanonicalCatalog").count(),
        1,
        "INFRA-002: one CanonicalCatalog type; do not invent a second loader"
    );
}

#[test]
fn infra_003_forty_three_infrastructure_controls_are_stable() {
    let catalog = load_catalog();
    let text = infrastructure_catalog_text();
    let ids = quoted_ids(&text, "control.");
    for id in CANONICAL_INFRA_CONTROLS {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("INFRA-003: missing control `{id}`"));
        assert!(
            ids.contains(*id),
            "INFRA-003: missing control `{id}` (have {ids:?})"
        );
    }
    assert_eq!(
        CANONICAL_INFRA_CONTROLS.len(),
        43,
        "pinned family size is 43"
    );
    let family: Vec<_> = ids
        .iter()
        .filter(|id| {
            id.starts_with("control.network.")
                || id.starts_with("control.crypto.")
                || id.starts_with("control.secret.")
                || id.starts_with("control.data.")
                || id.starts_with("control.database.")
                || id.starts_with("control.logging.")
                || id.starts_with("control.backup.")
                || id.starts_with("control.resilience.")
        })
        .collect();
    assert!(
        (35..=50).contains(&family.len()),
        "INFRA-003: expected 35–50 independently assessable infrastructure controls, found {} ({family:?})",
        family.len()
    );
    assert_eq!(family.len(), 43, "INFRA-003: exactly 43 controls");
    for id in &family {
        assert_eq!(
            *id,
            &id.to_ascii_lowercase(),
            "INFRA-003: ids are lowercase ({id})"
        );
        assert!(
            !id.contains('_'),
            "INFRA-003: catalog ids use hyphen segments, not underscores ({id})"
        );
        for token in FORBIDDEN_PROVIDER_TOKENS
            .iter()
            .chain(FORBIDDEN_FRAMEWORK_TOKENS.iter())
        {
            let segment = format!(".{token}.");
            assert!(
                !id.contains(&segment) && !id.ends_with(&format!(".{token}")),
                "INFRA-003: reserved token `{token}` leaked into id `{id}`"
            );
        }
    }
}

#[test]
fn infra_003_controls_declare_domains_evidence_tests_and_automation() {
    let text = infrastructure_catalog_text();
    for id in CANONICAL_INFRA_CONTROLS {
        let window = control_record_window(&text, id);
        assert!(
            window.contains("domains") || window.contains("domain"),
            "INFRA-003: {id} must declare domain(s)"
        );
        assert!(
            window.contains("evidence"),
            "INFRA-003: {id} must declare evidence requirements"
        );
        assert!(
            window.contains("tests") || window.contains("test"),
            "INFRA-003: {id} must declare test refs"
        );
        let class = control_record_automation(&text, id);
        assert!(
            matches!(class.as_str(), "automated" | "hybrid" | "manual"),
            "INFRA-003: {id} automation class must be automated|hybrid|manual, got {class}"
        );
    }
}

#[test]
fn infra_004_sixteen_evidence_contracts_are_facts_not_conclusions() {
    let catalog = load_catalog();
    let text = infrastructure_catalog_text();
    let ids = quoted_ids(&text, "evidence.");
    for id in REQUIRED_EVIDENCE {
        assert!(
            catalog.evidence().contains_key(*id),
            "INFRA-004: missing evidence contract `{id}`"
        );
        assert!(
            ids.contains(*id),
            "INFRA-004: evidence `{id}` must be declared in family TOML"
        );
    }
    assert_eq!(REQUIRED_EVIDENCE.len(), 16);
    let lower = text.to_ascii_lowercase();
    for phrase in [
        "compliant",
        "certified",
        "control passed",
        "firewall effective",
        "least privilege effective",
        "dr effective",
        "logging control passed",
    ] {
        assert!(
            !lower.contains(phrase),
            "INFRA-004: evidence contracts are facts, not conclusions (`{phrase}`)"
        );
    }
    for forbidden in [
        "evidence.aws.",
        "evidence.cloudflare.",
        "evidence.azure.",
        "evidence.gcp.",
        "evidence.secret.exposure",
    ] {
        assert!(
            !ids.iter()
                .any(|id| id.contains(forbidden) || id.starts_with(forbidden)),
            "INFRA-004: do not declare `{forbidden}` in this slice"
        );
        assert!(
            !text.contains(forbidden),
            "INFRA-004: infrastructure family TOML must not mention `{forbidden}`"
        );
    }
}

#[test]
fn infra_005_required_population_tests_are_declared_and_not_exists() {
    let catalog = load_catalog();
    let text = infrastructure_catalog_text();
    let ids = quoted_ids(&text, "test.");
    for id in ALL_INFRA_TESTS {
        assert!(
            catalog.tests().contains_key(*id),
            "INFRA-005: missing test `{id}`"
        );
        assert!(ids.contains(*id), "INFRA-005: missing test `{id}` in TOML");
    }
    for id in REQUIRED_POPULATION_TESTS {
        let window = test_expression_window(&text, id);
        assert!(
            !expression_is_existence_only(&window),
            "INFRA-005: {id} must not be Exists(one envelope)"
        );
        assert!(
            POPULATION_OPS.iter().any(|op| window.contains(op)
                || window
                    .to_ascii_lowercase()
                    .contains(&op.to_ascii_lowercase())),
            "INFRA-005: {id} must use AllSubjects / NoneSubjects / CoverageAtLeast"
        );
        let field = catalog
            .tests()
            .get(*id)
            .and_then(|t| t.expression.get("field"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            CLASSIFIABLE_FIELDS
                .iter()
                .any(|f| field == *f || field.ends_with("_at")),
            "INFRA-005: {id} must bind a population-runtime-classifiable field, not raw retention_days; got `{field}`"
        );
        assert_ne!(
            field, "retention_days",
            "INFRA-005: {id} must not ask AllSubjects to compare raw retention_days"
        );
        assert_ne!(
            field, "min_protocol",
            "INFRA-005: {id} must bind meets_policy, not raw min_protocol"
        );
    }
}

#[test]
fn infra_006_validator_rejects_provider_tokens_in_ids() {
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    for token in ["aws", "azure", "gcp", "cloudflare", "google", "vercel"] {
        assert!(
            rust.contains(&format!("\"{token}\"")),
            "INFRA-006: catalog infrastructure validator must reserve provider token `{token}`"
        );
    }
    let text = infrastructure_catalog_text();
    for id in quoted_ids(&text, "control.")
        .into_iter()
        .chain(quoted_ids(&text, "evidence."))
        .chain(quoted_ids(&text, "test."))
    {
        for token in FORBIDDEN_PROVIDER_TOKENS {
            let segment = format!(".{token}.");
            let suffix = format!(".{token}");
            assert!(
                !id.contains(&segment) && !id.ends_with(&suffix),
                "INFRA-006: provider token `{token}` leaked into id `{id}`"
            );
        }
        assert!(
            !id.starts_with("evidence.aws.") && !id.starts_with("evidence.cloudflare."),
            "INFRA-006: provider-specific contract `{id}` is forbidden"
        );
    }
}

#[test]
fn infra_007_canonical_infrastructure_content_has_no_framework_tokens() {
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    for token in ["iso27001", "soc2", "nis2", "dora", "gdpr"] {
        assert!(
            rust.to_ascii_lowercase().contains(token),
            "INFRA-007: validator must reserve framework token `{token}`"
        );
    }
    let text = infrastructure_catalog_text();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            !lower.contains(token),
            "INFRA-007: canonical infrastructure file text must not mention `{token}` (pci/pci-dss rejected even if absent from FRAMEWORK_SEGMENTS)"
        );
    }
}

#[test]
fn infra_008_iso_pack_is_not_grown_or_retargeted_by_this_slice() {
    let _ = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    for prefix in [
        "control.network.",
        "control.crypto.",
        "control.secret.",
        "control.data.",
        "control.database.",
        "control.logging.",
        "control.backup.",
        "control.resilience.",
    ] {
        assert!(
            !metadata.contains(&format!("id = \"{prefix}")),
            "INFRA-008: do not add `{prefix}*` to the ISO pack metadata"
        );
    }
    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    assert!(
        !mappings.contains("to = \"control.network.")
            && !mappings.contains("to = \"control.crypto.")
            && !mappings.contains("to = \"control.secret.")
            && !mappings.contains("to = \"control.data.")
            && !mappings.contains("to = \"control.database.")
            && !mappings.contains("to = \"control.logging.")
            && !mappings.contains("to = \"control.backup.")
            && !mappings.contains("to = \"control.resilience."),
        "INFRA-008: this slice must not retarget ISO mappings onto infrastructure catalog ids (ISO remap owns remap)"
    );
}

#[test]
fn infra_009_critical_encrypt_at_rest_is_population_not_existence() {
    let catalog = load_catalog();
    let text = infrastructure_catalog_text();
    let window = test_expression_window(&text, "test.database.critical-encrypt-at-rest");
    assert!(
        !expression_is_existence_only(&window),
        "INFRA-009: test.database.critical-encrypt-at-rest must not be Exists(some encryption envelope)"
    );

    let (control, expr) = catalog_test_expr(&catalog, "test.database.critical-encrypt-at-rest");
    assert_eq!(control, "control.database.encryption");

    let mut lone = EvidenceSet::new();
    lone.insert(seal(
        "evidence.data.encryption-at-rest",
        "db:only",
        &[("encrypted", EvidenceValue::Bool(true))],
        collected(1),
    ));
    let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.data.encryption-at-rest",
    )));
    let (exists_ok, _) = result_json(
        "test.database.critical-encrypt-at-rest",
        "control.database.encryption",
        exists,
        &lone,
    );
    assert_eq!(
        exists_ok.effectiveness,
        Effectiveness::Effective,
        "sanity: a lone encryption envelope satisfies Exists"
    );

    let (pop, json) = result_json(
        "test.database.critical-encrypt-at-rest",
        &control,
        expr.clone(),
        &lone,
    );
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "INFRA-009: a single encryption envelope must not pass all critical databases encrypt at rest; json={json}"
    );

    let fail_set = load_fixture_set("database", "unencrypted-critical-db");
    let (failed, fail_json) = result_json(
        "test.database.critical-encrypt-at-rest",
        &control,
        expr,
        &fail_set,
    );
    assert_eq!(
        failed.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-009: unencrypted-critical-db is Ineffective, got {:?}",
        failed.effectiveness
    );
    let failing = string_list(&fail_json, "failingSubjects");
    assert!(
        !failing.is_empty(),
        "INFRA-009: failing subject must name the unencrypted database; got {failing:?}"
    );
}

#[test]
fn infra_010_fixtures_distinguish_missing_stale_fail_manual_exception() {
    let root = manifest_dir().join("fixtures/assurance/canonical/v1");
    assert!(
        root.is_dir(),
        "INFRA-010: fixtures/assurance/canonical/v1 must exist"
    );
    for (family, name) in INFRA_FIXTURES {
        let dir = require_fixture(family, name);
        let blob = fs::read_to_string(dir.join("evidence.json")).unwrap();
        assert!(
            blob.contains("evidence.") || blob.contains("inventory.subject"),
            "INFRA-010: fixture `{family}/{name}` must emit canonical evidence types"
        );
        assert!(
            !blob.contains("encryption.at-rest.configured"),
            "INFRA-010: fixture `{family}/{name}` must not emit pack-local encryption.at-rest.configured"
        );
        let lower = blob.to_ascii_lowercase();
        for key in [
            "password",
            "private_key",
            "connection_string",
            "secret_value",
        ] {
            assert!(
                !lower.contains(key),
                "INFRA-010: fixture `{family}/{name}` must not carry secret material (`{key}`)"
            );
        }
        assert!(
            !lower.contains("compliant") && !lower.contains("certified"),
            "INFRA-010: fixture `{family}/{name}` must not carry compliance narratives"
        );
    }

    let catalog = load_catalog();

    let healthy = load_fixture_set("database", "healthy");
    let (ok_ctrl, ok_expr) = catalog_test_expr(&catalog, "test.database.critical-encrypt-at-rest");
    let (ok, _) = result_json(
        "test.database.critical-encrypt-at-rest",
        &ok_ctrl,
        ok_expr,
        &healthy,
    );
    assert_eq!(
        ok.effectiveness,
        Effectiveness::Effective,
        "INFRA-010: database/healthy → Effective"
    );

    let missing = load_fixture_set("database", "missing-encryption");
    let (miss_ctrl, miss_expr) =
        catalog_test_expr(&catalog, "test.database.critical-encrypt-at-rest");
    let (miss, _) = result_json(
        "test.database.critical-encrypt-at-rest",
        &miss_ctrl,
        miss_expr,
        &missing,
    );
    assert_eq!(
        miss.effectiveness,
        Effectiveness::InsufficientEvidence,
        "INFRA-010: missing-encryption → InsufficientEvidence (missing ≠ fail), got {:?}",
        miss.effectiveness
    );

    let stale = load_fixture_set("logging", "stale-audit-log");
    let (stale_ctrl, stale_expr) =
        catalog_test_expr(&catalog, "test.logging.critical-assets-audit-current");
    let (stale_r, _) = result_json(
        "test.logging.critical-assets-audit-current",
        &stale_ctrl,
        stale_expr,
        &stale,
    );
    assert_eq!(
        stale_r.effectiveness,
        Effectiveness::StaleEvidence,
        "INFRA-010: stale-audit-log → StaleEvidence, got {:?}",
        stale_r.effectiveness
    );

    let fail = load_fixture_set("logging", "retention-below-threshold");
    let (ret_ctrl, ret_expr) =
        catalog_test_expr(&catalog, "test.logging.retention-meets-threshold");
    let (fail_r, _) = result_json(
        "test.logging.retention-meets-threshold",
        &ret_ctrl,
        ret_expr,
        &fail,
    );
    assert_eq!(
        fail_r.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-010: retention-below-threshold → Ineffective, got {:?}",
        fail_r.effectiveness
    );

    let restore = load_fixture_set("backup", "failing-restore");
    let (rst_ctrl, rst_expr) = catalog_test_expr(&catalog, "test.backup.restore-test-fresh");
    let (rst, _) = result_json(
        "test.backup.restore-test-fresh",
        &rst_ctrl,
        rst_expr,
        &restore,
    );
    assert_eq!(
        rst.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-010: failing-restore → Ineffective, got {:?}",
        rst.effectiveness
    );

    let stale_restore = load_fixture_set("backup", "stale-restore-test");
    let (sr_ctrl, sr_expr) = catalog_test_expr(&catalog, "test.backup.restore-test-fresh");
    let (sr, _) = result_json(
        "test.backup.restore-test-fresh",
        &sr_ctrl,
        sr_expr,
        &stale_restore,
    );
    assert_eq!(
        sr.effectiveness,
        Effectiveness::StaleEvidence,
        "INFRA-010: stale-restore-test → StaleEvidence, got {:?}",
        sr.effectiveness
    );

    let dr = load_fixture_set("resilience", "missing-dr-exercise");
    let (dr_ctrl, dr_expr) = catalog_test_expr(&catalog, "test.resilience.dr-exercise-recorded");
    let (dr_r, _) = result_json(
        "test.resilience.dr-exercise-recorded",
        &dr_ctrl,
        dr_expr,
        &dr,
    );
    assert!(
        matches!(
            dr_r.effectiveness,
            Effectiveness::ManualReviewRequired | Effectiveness::InsufficientEvidence
        ),
        "INFRA-010: missing-dr-exercise → ManualReviewRequired or InsufficientEvidence, never Effective; got {:?}",
        dr_r.effectiveness
    );
    assert_ne!(dr_r.effectiveness, Effectiveness::Effective);
}

#[test]
fn infra_011_partial_inventory_cannot_be_effective() {
    let catalog = load_catalog();
    for (family, test_id) in [
        ("database", "test.database.critical-encrypt-at-rest"),
        ("network", "test.network.public-endpoints-acceptable-tls"),
        ("logging", "test.logging.critical-assets-audit-current"),
    ] {
        let set = load_fixture_set(family, "partial-inventory");
        let (control, expr) = catalog_test_expr(&catalog, test_id);
        let (result, json) = result_json(test_id, &control, expr, &set);
        assert_ne!(
            result.effectiveness,
            Effectiveness::Effective,
            "INFRA-011: {family}/partial-inventory must not yield Effective on {test_id}; json={json}"
        );
        assert_eq!(
            result.effectiveness,
            Effectiveness::InsufficientEvidence,
            "INFRA-011: partial/unknown population → InsufficientEvidence for {test_id}, got {:?}",
            result.effectiveness
        );
    }
}

#[test]
fn infra_012_approved_exceptions_are_not_silent_effective() {
    let catalog = load_catalog();
    let exposure = load_fixture_set("network", "exception-approved-exposure");
    let (ctrl, expr) = catalog_test_expr(&catalog, "test.network.no-prohibited-public-databases");
    let (result, json) = result_json(
        "test.network.no-prohibited-public-databases",
        &ctrl,
        expr,
        &exposure,
    );
    let excepted = string_list(&json, "exceptedSubjects");
    assert!(
        !excepted.is_empty() || result.effectiveness == Effectiveness::ExceptionApproved,
        "INFRA-012: approved unexpired exposure exception must except the bound subject or yield ExceptionApproved; got {:?} {json}",
        result.effectiveness
    );
    assert_ne!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-012: bound approved exception must not stay Ineffective"
    );
    let failing = string_list(&json, "failingSubjects");
    assert!(
        failing.is_empty(),
        "INFRA-012: excepted public DB must not remain in failingSubjects ({failing:?})"
    );

    let rto = load_fixture_set("resilience", "exception-approved-rto");
    let (rto_ctrl, rto_expr) = catalog_test_expr(&catalog, "test.backup.restore-test-fresh");
    let (rto_r, rto_json) =
        result_json("test.backup.restore-test-fresh", &rto_ctrl, rto_expr, &rto);
    assert_ne!(rto_r.effectiveness, Effectiveness::Ineffective);
    let rto_excepted = string_list(&rto_json, "exceptedSubjects");
    assert!(
        !rto_excepted.is_empty() || rto_r.effectiveness == Effectiveness::ExceptionApproved,
        "INFRA-012: exception-approved-rto must except the named store or yield ExceptionApproved; got {:?} {rto_json}",
        rto_r.effectiveness
    );

    let _ = bound_exception(
        "control.network.no-unnecessary-public-databases",
        SubjectKind::Database,
        "db:public",
    );
}

#[test]
fn infra_013_dr_objectives_and_segmentation_stay_hybrid_or_manual() {
    let text = infrastructure_catalog_text();
    for id in HYBRID_OR_MANUAL_CONTROLS {
        let class = control_record_automation(&text, id);
        assert!(
            class == "hybrid" || class == "manual",
            "INFRA-013: {id} must be Hybrid or Manual, got {class}"
        );
    }
    for id in HONEST_MANUAL_TESTS {
        let window = test_expression_window(&text, id);
        let lower = window.to_ascii_lowercase();
        assert!(
            lower.contains("manual-review") || lower.contains("manual_review"),
            "INFRA-013: {id} must use op = \"manual-review\" so a single technical flag cannot auto-pass"
        );
        assert!(
            !expression_is_existence_only(&window),
            "INFRA-013: {id} must not auto-pass as Exists(one technical envelope)"
        );
    }

    let catalog = load_catalog();
    let mut tech_only = EvidenceSet::new();
    tech_only.insert(seal(
        "evidence.resilience.recovery-plan",
        "org:infra",
        &[
            ("procedure_present", EvidenceValue::Bool(true)),
            ("objectives_documented", EvidenceValue::Bool(true)),
        ],
        collected(1),
    ));
    for test_id in HONEST_MANUAL_TESTS {
        let (control, expr) = catalog_test_expr(&catalog, test_id);
        let (via, _) = result_json(test_id, &control, expr, &tech_only);
        assert_eq!(
            via.effectiveness,
            Effectiveness::ManualReviewRequired,
            "INFRA-013: {test_id} cannot auto-pass from one technical flag; got {:?}",
            via.effectiveness
        );
    }
}

#[test]
fn infra_014_thresholds_live_in_catalog_or_assessment_context() {
    let product = product_rs_joined();
    assert!(
        !product.contains("ISO_RETENTION_DAYS"),
        "INFRA-014: do not hardcode ISO_RETENTION_DAYS in product crates"
    );
    assert!(
        !product.contains("const MIN_TLS") && !product.contains("PCI_RETENTION"),
        "INFRA-014: do not hardcode MIN_TLS / PCI retention constants in product crates"
    );

    let catalog = load_catalog();
    let retention = catalog
        .tests()
        .get("test.logging.retention-meets-threshold")
        .expect("INFRA-014: retention test must exist");
    assert!(
        retention.expression.contains_key("min_days")
            || retention.expression.contains_key("threshold"),
        "INFRA-014: test.logging.retention-meets-threshold must carry min_days/threshold on [test.expression]"
    );
    let tls = catalog
        .tests()
        .get("test.network.public-endpoints-acceptable-tls")
        .expect("INFRA-014: TLS test must exist");
    assert!(
        tls.expression.contains_key("acceptable_min_protocol")
            || tls.expression.contains_key("min_protocol"),
        "INFRA-014: TLS acceptability must be catalog/test configuration, not a Rust constant"
    );
    let storage = catalog
        .tests()
        .get("test.secret.approved-storage")
        .expect("INFRA-014: approved-storage test must exist");
    assert!(
        storage.expression.contains_key("approved_backends"),
        "INFRA-014: approved-storage backends come from [test.expression].approved_backends"
    );
    let restore = catalog
        .tests()
        .get("test.backup.restore-test-fresh")
        .expect("INFRA-014: restore-test-fresh must exist");
    assert!(
        restore.expression.contains_key("window")
            || restore.expression.contains_key("max_age")
            || restore.expression.contains_key("field"),
        "INFRA-014: restore freshness uses catalog window or AssessmentContext.max_age"
    );
    let _ = fresh_context().max_age;
}

#[test]
fn infra_015_no_cloud_collectors_or_secret_exposure_or_population_fork() {
    let collector_src = crate_src("weeping-angel-collector");
    for name in ["aws", "azure", "gcp", "google", "cloudflare"] {
        assert!(
            !collector_src.join(name).exists(),
            "INFRA-015: do not add a {name} collector in this slice"
        );
    }
    assert!(
        GITHUB_EVIDENCE_TYPES
            .iter()
            .all(|t| t.starts_with("source.")),
        "INFRA-015: GitHub collector must keep emitting source.* only"
    );
    for prefix in [
        "evidence.network.",
        "evidence.database.",
        "evidence.backup.",
        "evidence.logging.",
        "evidence.secret.",
    ] {
        assert!(
            !GITHUB_EVIDENCE_TYPES.iter().any(|t| t.starts_with(prefix)),
            "INFRA-015: GitHub must not advertise `{prefix}`"
        );
    }

    let fw = fs::read_to_string(manifest_dir().join("crates/weeping-angel-framework/Cargo.toml"))
        .unwrap();
    for dep in ["aws-sdk", "azure_mgmt", "google-cloud", "cloudflare"] {
        assert!(
            !fw.contains(dep),
            "INFRA-015: framework crate must not grow a provider SDK ({dep})"
        );
    }

    let product = product_rs_joined();
    assert!(
        !product.contains("resolve_database_inventory")
            && !product.contains("resolve_network_inventory")
            && !product.contains("struct InfraPopulation"),
        "INFRA-015: do not fork population resolution or invent InfraPopulation"
    );
    let value = fs::read_to_string(crate_src("weeping-angel-evidence").join("value.rs")).unwrap();
    assert!(
        value.contains("pub enum EvidenceValue {"),
        "INFRA-015: consume weeping-angel-evidence::EvidenceValue"
    );
    assert!(
        !product.contains("enum InfraEvidenceValue") && !product.contains("enum EvidenceValueFork"),
        "INFRA-015: do not fork EvidenceValue"
    );

    let catalog = load_catalog();
    let infra_text = [
        "crypto.toml",
        "network.toml",
        "database.toml",
        "logging.toml",
    ]
    .iter()
    .map(|name| fs::read_to_string(catalog_v1().join("evidence").join(name)).unwrap_or_default())
    .collect::<Vec<_>>()
    .join("\n");
    assert!(
        !infra_text.contains("evidence.secret.exposure"),
        "INFRA-015: evidence.secret.exposure is vulnerability catalog, not this slice"
    );
    assert!(
        catalog
            .evidence()
            .keys()
            .any(|id| id.starts_with("evidence.")),
        "INFRA-015: catalog still loads"
    );
}

#[test]
fn infra_016_iso_and_iam_siblings_remain_the_gate() {
    let _ = load_framework_pack("iso-27001", "2022").expect("INFRA-016: ISO pack still loads");

    let catalog = load_catalog();
    catalog
        .control("control.identity.mfa")
        .expect("INFRA-016: IAM family must remain");
    catalog
        .control("control.source.protected-branch")
        .expect("INFRA-016: fixture.example control.source.protected-branch must remain");

    for path in [
        "tests/contracts/iso27001_assurance.target.rs",
        "tests/contracts/iam_catalog.target.rs",
        "docs/specs/canonical-assurance-catalog-v1.md",
        "docs/specs/iam-canonical-assurance-catalog.md",
    ] {
        assert!(
            manifest_dir().join(path).is_file(),
            "INFRA-016: sibling path `{path}` must remain"
        );
    }

    let catalog_ssot =
        fs::read_to_string(manifest_dir().join("docs/specs/canonical-assurance-catalog-v1.md"))
            .unwrap();
    assert!(
        catalog_ssot.starts_with("# SDD: Canonical Assurance Catalog v1 infrastructure"),
        "INFRA-016: do not overwrite catalog infrastructure SSOT"
    );
    let iam =
        fs::read_to_string(manifest_dir().join("docs/specs/iam-canonical-assurance-catalog.md"))
            .unwrap();
    assert!(
        iam.starts_with("# SDD: IAM Canonical Assurance Catalog (v1 slice)"),
        "INFRA-016: do not overwrite IAM catalog SSOT"
    );
}

#[test]
fn infra_017_secret_storage_lives_in_crypto_toml() {
    let crypto = catalog_v1().join("controls/crypto.toml");
    assert!(
        crypto.is_file(),
        "INFRA-017: controls/crypto.toml must exist and host control.secret.*"
    );
    let text = fs::read_to_string(crypto).unwrap();
    assert!(
        text.contains("control.secret.storage")
            && text.contains("control.secret.credential-storage"),
        "INFRA-017: control.secret.* must live in crypto.toml"
    );
    let evidence = fs::read_to_string(catalog_v1().join("evidence/crypto.toml")).unwrap();
    assert!(
        evidence.contains("evidence.secret.storage-configuration"),
        "INFRA-017: evidence.secret.storage-configuration lives in evidence/crypto.toml"
    );
    assert!(
        !evidence.contains("evidence.secret.exposure"),
        "INFRA-017: do not declare evidence.secret.exposure here"
    );
    let tests = fs::read_to_string(catalog_v1().join("tests/crypto.toml")).unwrap();
    assert!(
        tests.contains("test.secret.approved-storage"),
        "INFRA-017: test.secret.approved-storage lives in tests/crypto.toml"
    );
}

#[test]
fn infra_018_public_contract_names_the_infrastructure_family() {
    let contract =
        fs::read_to_string(manifest_dir().join("docs/specs/assurance-runtime.md")).unwrap();
    for needle in [
        "control.network.",
        "control.crypto.",
        "evidence.database.",
        "evidence.secret.storage-configuration",
        "fixtures/assurance/canonical/v1/network",
        "fixtures/assurance/canonical/v1/database",
    ] {
        assert!(
            contract.contains(needle),
            "INFRA-018: public contract must name infrastructure `{needle}` so it does not lie"
        );
    }
    assert!(
        contract.contains("control.identity."),
        "INFRA-018: contract must keep documenting the IAM family"
    );
}

#[test]
fn infra_019_healthy_populations_are_non_trivial_and_seal_rejects_secrets() {
    let healthy = load_fixture_set("database", "healthy");
    let subjects: BTreeSet<String> = healthy
        .iter()
        .filter(|env| env.observation().evidence_type().as_str() == "inventory.subject")
        .map(|env| env.provenance().asset().as_str().to_string())
        .collect();
    assert!(
        subjects.len() >= 3,
        "INFRA-019: database/healthy primary population must have n ≥ 3 subjects, got {subjects:?}"
    );

    let obs = EvidenceObservation::new(EvidenceType::new("evidence.secret.storage-configuration"))
        .with_fact("password", "hunter2")
        .with_fact("subject_id", "secret:api");
    let err = EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.infrastructure-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("secret:api"),
        },
    )
    .expect_err("INFRA-019: password fact must not seal");
    assert!(matches!(err, EvidenceError::CredentialInPayload { .. }));

    let claim = EvidenceObservation::new(EvidenceType::new("evidence.network.exposure"))
        .with_narrative("control is PCI-DSS compliant");
    let claim_err = EvidenceEnvelope::seal(
        claim,
        EvidenceProvenance {
            collector_id: "fixture.infrastructure-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("db:orders"),
        },
    )
    .expect_err("INFRA-019: compliance narrative must not seal");
    assert!(matches!(claim_err, EvidenceError::ComplianceClaim { .. }));
}

#[test]
fn infra_020_public_tls_and_secret_storage_population_fixtures() {
    let catalog = load_catalog();

    let tls_fail = load_fixture_set("network", "insecure-tls");
    let (tls_ctrl, tls_expr) =
        catalog_test_expr(&catalog, "test.network.public-endpoints-acceptable-tls");
    let (tls, tls_json) = result_json(
        "test.network.public-endpoints-acceptable-tls",
        &tls_ctrl,
        tls_expr,
        &tls_fail,
    );
    assert_eq!(
        tls.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-020: insecure-tls → Ineffective, got {:?}",
        tls.effectiveness
    );
    assert!(
        !string_list(&tls_json, "failingSubjects").is_empty(),
        "INFRA-020: insecure TLS must name the failing endpoint"
    );

    let exposed = load_fixture_set("network", "public-db-exposed");
    let (db_ctrl, db_expr) =
        catalog_test_expr(&catalog, "test.network.no-prohibited-public-databases");
    let (db, db_json) = result_json(
        "test.network.no-prohibited-public-databases",
        &db_ctrl,
        db_expr,
        &exposed,
    );
    assert_eq!(
        db.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-020: public-db-exposed → Ineffective, got {:?}",
        db.effectiveness
    );
    assert!(
        !string_list(&db_json, "failingSubjects").is_empty(),
        "INFRA-020: prohibited public database must be named"
    );

    let storage = load_fixture_set("crypto", "unapproved-secret-storage");
    let (sec_ctrl, sec_expr) = catalog_test_expr(&catalog, "test.secret.approved-storage");
    let (sec, _) = result_json(
        "test.secret.approved-storage",
        &sec_ctrl,
        sec_expr,
        &storage,
    );
    assert_eq!(
        sec.effectiveness,
        Effectiveness::Ineffective,
        "INFRA-020: unapproved-secret-storage → Ineffective, got {:?}",
        sec.effectiveness
    );
}
