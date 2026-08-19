//! Baseline suite for Operational ISMS v1 Prompt 17 (personnel security).
//!
//! Characterization of CURRENT tree (`docs/specs/personnel-security.md` §3)
//! on SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: five
//! `control.personnel.*` rows in `governance.toml`, two personnel evidence
//! types, IAM technical JML (manual-review / none-subjects on string
//! `status`), thin `Identity`, GitHub+local collectors, and no personnel
//! lifecycle fixtures or additive `personnel.toml`.
//!
//! Skip-superseded by `sdd_personnel_security_target`
//! (`#[ignore = "superseded by sdd_personnel_security_target"]`).
//! Does **not** implement personnel lifecycle.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlImplementation, ControlImplementationId, ControlTestId,
    Exception, ExceptionId, Identity, IdentityId, IdentityKind, SubjectKind,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
    looks_like_compliance_claim,
};

const GOVERNANCE_PERSONNEL_CONTROLS: &[(&str, &str, &str, &str)] = &[
    (
        "control.personnel.security-awareness",
        "hybrid",
        "evidence.personnel.training",
        "test.personnel.awareness-current-all",
    ),
    (
        "control.personnel.role-specific-training",
        "hybrid",
        "evidence.personnel.training",
        "test.personnel.training-current-all",
    ),
    (
        "control.personnel.onboarding-offboarding",
        "manual",
        "evidence.manual.attestation",
        "test.personnel.jml-process-attested",
    ),
    (
        "control.personnel.confidentiality-commitment",
        "hybrid",
        "evidence.personnel.acknowledgement",
        "test.personnel.confidentiality-acknowledged-all",
    ),
    (
        "control.personnel.policy-acknowledgement",
        "hybrid",
        "evidence.personnel.acknowledgement",
        "test.personnel.policy-acknowledged-all",
    ),
];

const EXISTING_PERSONNEL_EVIDENCE: &[&str] = &[
    "evidence.personnel.training",
    "evidence.personnel.acknowledgement",
];

const ABSENT_LIFECYCLE_CONTROLS: &[&str] = &[
    "control.personnel.screening",
    "control.personnel.joiner-grace",
    "control.personnel.access-provisioning",
    "control.personnel.role-change",
    "control.personnel.leaver-access",
    "control.personnel.asset-return",
];

const ABSENT_LIFECYCLE_EVIDENCE: &[&str] = &[
    "evidence.personnel.screening",
    "evidence.personnel.asset-return",
    "evidence.personnel.joiner-grace",
    "evidence.personnel.population-membership",
];

const ABSENT_LIFECYCLE_TESTS: &[&str] = &[
    "test.personnel.screening-recorded",
    "test.personnel.joiner-grace-honored",
    "test.personnel.joiner-access-provisioned",
    "test.personnel.mover-privileges-reduced",
    "test.personnel.no-leaver-active-access",
    "test.personnel.asset-return-recorded",
];

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

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
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
            collector_id: "fixture.personnel-baseline".into(),
            collected_at: at,
            scope: "baseline".into(),
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

fn load_named_fixture(family: &str, name: &str) -> EvidenceSet {
    let dir = manifest_dir()
        .join("fixtures/assurance/canonical/v1")
        .join(family)
        .join(name);
    let raw = fs::read_to_string(dir.join("evidence.json"))
        .unwrap_or_else(|e| panic!("read {family}/{name}/evidence.json: {e}"));
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
    set
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
        .unwrap_or("organization");
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
    match op {
        "all-subjects" | "all_subjects" | "AllSubjects" => TestExpr::AllSubjects {
            selector,
            evidence: evidence_sel,
        },
        "none-subjects" | "none_subjects" | "NoneSubjects" => TestExpr::NoneSubjects {
            selector,
            evidence: evidence_sel,
        },
        other => panic!("{test_id} uses unsupported baseline op `{other}`"),
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

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn expression_op_and_field(test_id: &str) -> (String, Option<String>, Option<String>) {
    let catalog = load_catalog();
    let test = catalog
        .tests()
        .get(test_id)
        .unwrap_or_else(|| panic!("missing {test_id}"));
    let op = test
        .expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let evidence = test
        .expression
        .get("evidence")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let field = test
        .expression
        .get("field")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    (op, evidence, field)
}

/// PER-B001: governance.toml ships exactly the five `control.personnel.*` rows.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b001_five_governance_personnel_controls() {
    let catalog = load_catalog();
    let personnel: BTreeSet<&str> = catalog
        .controls()
        .keys()
        .filter(|id| id.starts_with("control.personnel."))
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = GOVERNANCE_PERSONNEL_CONTROLS
        .iter()
        .map(|(id, _, _, _)| *id)
        .collect();
    assert_eq!(
        personnel, expected,
        "PER-B001: loaded catalog must have exactly the five governance personnel controls"
    );

    for (id, automation, evidence, test_id) in GOVERNANCE_PERSONNEL_CONTROLS {
        let control = catalog.control(id).expect(id);
        assert_eq!(control.automation, *automation, "{id} automation");
        assert_eq!(control.evidence, vec![*evidence], "{id} evidence");
        assert_eq!(control.tests, vec![*test_id], "{id} test");
        assert!(
            control
                .domains
                .iter()
                .any(|d| d == "personnelSecurity" || d == "governance"),
            "{id} must stay in personnelSecurity/governance"
        );
        catalog
            .tests()
            .get(*test_id)
            .unwrap_or_else(|| panic!("missing test {test_id}"));
    }

    let family: Vec<_> = catalog
        .controls()
        .keys()
        .filter(|id| GOVERNANCE_FAMILY_PREFIXES.iter().any(|p| id.starts_with(p)))
        .collect();
    assert!(
        (30..=45).contains(&family.len()),
        "PER-B001: governance-family slice stays 30–45, found {}",
        family.len()
    );
}

/// PER-B002: no additive lifecycle personnel controls/tests exist yet.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b002_no_lifecycle_personnel_controls_or_tests() {
    let catalog = load_catalog();
    for id in ABSENT_LIFECYCLE_CONTROLS {
        assert!(
            catalog.control(id).is_err(),
            "PER-B002: `{id}` must be absent on characterization HEAD"
        );
    }
    for id in ABSENT_LIFECYCLE_TESTS {
        assert!(
            !catalog.tests().contains_key(*id),
            "PER-B002: `{id}` must be absent on characterization HEAD"
        );
    }
}

/// PER-B003: only `evidence.personnel.{{training,acknowledgement}}` are declared.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b003_only_two_personnel_evidence_types() {
    let catalog = load_catalog();
    let personnel: BTreeSet<&str> = catalog
        .evidence()
        .keys()
        .filter(|id| id.starts_with("evidence.personnel."))
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = EXISTING_PERSONNEL_EVIDENCE.iter().copied().collect();
    assert_eq!(personnel, expected);

    for id in EXISTING_PERSONNEL_EVIDENCE {
        let ev = catalog.evidence().get(*id).expect(id);
        assert_eq!(ev.collection, "hybrid");
        assert_eq!(ev.criticality, "required");
        assert!(
            ev.evidence_type.starts_with("personnel."),
            "{id} evidence_type stays personnel.*"
        );
    }
    for id in ABSENT_LIFECYCLE_EVIDENCE {
        assert!(
            !catalog.evidence().contains_key(*id),
            "PER-B003: `{id}` must be absent"
        );
    }
}

/// PER-B004: personnel tests are all-subjects/`current` or manual-review; not existence-only.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b004_personnel_tests_are_current_flags_or_manual() {
    let (op, evidence, field) = expression_op_and_field("test.personnel.training-current-all");
    assert_eq!(op, "all-subjects");
    assert_eq!(evidence.as_deref(), Some("evidence.personnel.training"));
    assert_eq!(field.as_deref(), Some("current"));

    let (op, evidence, field) = expression_op_and_field("test.personnel.awareness-current-all");
    assert_eq!(op, "all-subjects");
    assert_eq!(evidence.as_deref(), Some("evidence.personnel.training"));
    assert_eq!(field.as_deref(), Some("current"));

    let (op, evidence, field) =
        expression_op_and_field("test.personnel.confidentiality-acknowledged-all");
    assert_eq!(op, "all-subjects");
    assert_eq!(
        evidence.as_deref(),
        Some("evidence.personnel.acknowledgement")
    );
    assert_eq!(field.as_deref(), Some("current"));

    let (op, evidence, field) = expression_op_and_field("test.personnel.policy-acknowledged-all");
    assert_eq!(op, "all-subjects");
    assert_eq!(
        evidence.as_deref(),
        Some("evidence.personnel.acknowledgement")
    );
    assert_eq!(field.as_deref(), Some("current"));

    let (op, evidence, field) = expression_op_and_field("test.personnel.jml-process-attested");
    assert_eq!(op, "manual-review");
    assert!(evidence.is_none());
    assert!(field.is_none());
}

/// PER-B005: IAM technical JML is hybrid/manual-review or none-subjects on string `status`.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b005_iam_jml_is_not_population_honest_lifecycle() {
    let catalog = load_catalog();
    for id in [
        "control.identity.joiner-mover-leaver",
        "control.identity.terminated-user-removal",
        "control.identity.access-revocation-timeliness",
    ] {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("missing {id}"));
    }
    catalog
        .evidence()
        .get("evidence.identity.lifecycle-event")
        .expect("IAM lifecycle-event type exists");

    let jml = catalog
        .tests()
        .get("test.identity.jml-events-recorded")
        .expect("jml-events-recorded");
    assert_eq!(jml.kind, "hybrid");
    assert_eq!(
        jml.expression.get("op").and_then(|v| v.as_str()),
        Some("manual-review")
    );

    let revoke = catalog
        .tests()
        .get("test.identity.revocation-timely")
        .expect("revocation-timely");
    assert_eq!(revoke.kind, "hybrid");
    assert_eq!(
        revoke.expression.get("op").and_then(|v| v.as_str()),
        Some("manual-review")
    );

    let removal = catalog
        .tests()
        .get("test.identity.no-terminated-active-accounts")
        .expect("no-terminated-active-accounts");
    assert_eq!(removal.kind, "automated");
    assert_eq!(
        removal.expression.get("op").and_then(|v| v.as_str()),
        Some("none-subjects")
    );
    assert_eq!(
        removal.expression.get("evidence").and_then(|v| v.as_str()),
        Some("evidence.identity.account-status")
    );
    assert_eq!(
        removal.expression.get("field").and_then(|v| v.as_str()),
        Some("status"),
        "PER-B005: terminated-user-removal still predicates the string `status`, not a boolean `active`"
    );

    let result = evaluate_catalog_test(
        &catalog,
        "test.identity.no-terminated-active-accounts",
        &load_named_fixture("identity", "terminated-employee-active"),
    );
    let pop = result
        .population
        .as_ref()
        .expect("none-subjects yields population metadata");
    assert!(
        pop.technical_subjects.iter().any(|id| id == "user:lea"),
        "PER-B005: string status=active is Technical, not a boolean fail; got {:?}",
        pop.technical_subjects
    );
    assert!(
        pop.failing_subjects.is_empty(),
        "PER-B005: status string must not classify as boolean Failing; failing={:?}",
        pop.failing_subjects
    );
}

/// PER-B006: Identity / SubjectKind stay thin; no Employee/Contractor kinds.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b006_identity_ir_has_no_employment_class() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");

    let identity = Identity::new(IdentityId::new("user:ada"), IdentityKind::User);
    assert!(identity.display_name.is_none());
    let json = serde_json::to_value(&identity).unwrap();
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(keys, vec!["id".to_string(), "kind".to_string()]);
    assert_eq!(json["kind"], "user");

    let kinds = [
        (IdentityKind::User, "user"),
        (IdentityKind::Service, "service"),
        (IdentityKind::ServiceAccount, "serviceAccount"),
        (IdentityKind::Team, "team"),
        (IdentityKind::Role, "role"),
        (IdentityKind::Other, "other"),
    ];
    for (kind, expected) in kinds {
        let encoded = serde_json::to_value(kind).unwrap();
        assert_eq!(encoded, expected);
    }

    assert_eq!(SubjectKind::parse_name("user"), Some(SubjectKind::User));
    assert_eq!(
        SubjectKind::parse_name("identity"),
        Some(SubjectKind::Identity)
    );
    assert_eq!(
        SubjectKind::parse_name("privilegedIdentity"),
        Some(SubjectKind::PrivilegedIdentity)
    );
    assert_eq!(SubjectKind::parse_name("employee"), None);
    assert_eq!(SubjectKind::parse_name("contractor"), None);

    let identity_src = read_repo_file("crates/weeping-angel-assurance-ir/src/identity.rs");
    let subject_src = read_repo_file("crates/weeping-angel-assurance-ir/src/subject.rs");
    for src in [&identity_src, &subject_src] {
        assert!(
            !src.contains("Employee") && !src.contains("Contractor"),
            "PER-B006: IR must not grow HR employment kinds"
        );
    }

    let product = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !product.contains("fn resolve_personnel_inventory"),
        "PER-B006: no second personnel population resolver"
    );
}

/// PER-B007: collectors are github+local; control-test/framework stay collector-free.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b007_collectors_are_github_and_local_only() {
    let collector_src = crate_src("weeping-angel-collector");
    assert!(collector_src.join("github").is_dir());
    assert!(collector_src.join("local").is_dir());
    for missing in ["hris", "idp", "lms", "mdm", "personnel"] {
        assert!(
            !collector_src.join(missing).exists(),
            "PER-B007: collector module `{missing}` must be absent"
        );
    }

    let lib = read_repo_file("crates/weeping-angel-collector/src/lib.rs");
    assert!(lib.contains("pub mod github"));
    assert!(lib.contains("pub mod local"));
    assert!(!lib.contains("pub mod hris"));
    assert!(!lib.contains("pub mod idp"));
    assert!(!lib.contains("pub mod lms"));
    assert!(!lib.contains("pub mod mdm"));

    let collector_toml = crate_toml("weeping-angel-collector");
    assert!(!collector_toml.contains("weeping-angel-control-test"));
    assert!(!collector_toml.contains("weeping-angel-framework"));

    let control_test_toml = crate_toml("weeping-angel-control-test");
    let framework_toml = crate_toml("weeping-angel-framework");
    assert!(!control_test_toml.contains("weeping-angel-collector"));
    assert!(!framework_toml.contains("weeping-angel-collector"));

    assert!(looks_like_compliance_claim("iso 27001 compliant"));
    assert!(looks_like_compliance_claim("control test result"));
}

/// PER-B008: Prompt 10 CIR and Prompt 12 document types are specified, not landed.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b008_prompt10_and_prompt12_not_landed_exception_already_binds() {
    let implementation = ControlImplementation::new(
        ControlImplementationId::new("impl.personnel.baseline"),
        ControlId::new("control.personnel.security-awareness"),
    );
    let json = serde_json::to_value(&implementation).unwrap();
    for key in [
        "documentRefs",
        "document_refs",
        "reviewCadence",
        "evidenceExpectations",
    ] {
        assert!(
            json.get(key).is_none(),
            "PER-B008: ControlImplementation must omit `{key}` today"
        );
    }

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(!ir.contains("pub struct DocumentRef"));
    assert!(!ir.contains("pub struct ControlledDocument"));

    let exception = Exception::new(
        ExceptionId::new("exc.personnel.baseline"),
        "characterization subject-bound exception",
    );
    let encoded = serde_json::to_value(&exception).unwrap();
    assert!(encoded.get("subjects").and_then(Value::as_array).is_some());
    assert!(encoded.get("expiresAt").is_none());
    assert_eq!(exception.subjects.len(), 0);
    assert!(exception.expires_at.is_none());
}

/// PER-B009: no `fixtures/assurance/canonical/v1/personnel/` and no personnel.toml slice.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b009_no_personnel_fixtures_or_personnel_toml() {
    let personnel_dir = manifest_dir().join("fixtures/assurance/canonical/v1/personnel");
    assert!(
        !personnel_dir.exists(),
        "PER-B009: personnel fixture tree must be absent"
    );
    for name in PERSONNEL_FIXTURE_NAMES {
        let path = personnel_dir.join(name).join("evidence.json");
        assert!(!path.exists(), "PER-B009: fixture `{name}` must be absent");
    }

    let manifest = read_repo_file("catalog/canonical/v1/manifest.toml");
    assert!(
        !manifest.contains("personnel.toml"),
        "PER-B009: manifest must not list personnel.toml yet"
    );
    for rel in [
        "catalog/canonical/v1/controls/personnel.toml",
        "catalog/canonical/v1/evidence/personnel.toml",
        "catalog/canonical/v1/tests/personnel.toml",
    ] {
        assert!(
            !manifest_dir().join(rel).exists(),
            "PER-B009: `{rel}` must be absent"
        );
    }

    assert!(
        manifest_dir()
            .join("fixtures/assurance/canonical/v1/governance/incomplete-training-population")
            .is_dir()
    );
    assert!(
        manifest_dir()
            .join("fixtures/assurance/canonical/v1/identity/terminated-employee-active")
            .is_dir()
    );
}

/// PER-B010: training-current-all can fail a missing trainee; one envelope never proves coverage.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b010_training_population_honesty_already_exists_as_current_flag() {
    let catalog = load_catalog();

    let incomplete = load_named_fixture("governance", "incomplete-training-population");
    let missing =
        evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &incomplete);
    assert_eq!(
        missing.effectiveness,
        Effectiveness::InsufficientEvidence,
        "PER-B010: authoritative 2-user set with one envelope is missing, not Effective; {}",
        missing.rationale
    );

    let current = load_named_fixture("governance", "current-documents");
    let complete = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &current);
    assert_eq!(
        complete.effectiveness,
        Effectiveness::Effective,
        "PER-B010: current-documents still satisfies training-current-all; {}",
        complete.rationale
    );

    let empty = evaluate_catalog_test(
        &catalog,
        "test.personnel.training-current-all",
        &EvidenceSet::new(),
    );
    assert_ne!(
        empty.effectiveness,
        Effectiveness::Effective,
        "PER-B010: missing personnel source must never be Effective"
    );
    assert!(
        matches!(
            empty.effectiveness,
            Effectiveness::Inconclusive | Effectiveness::InsufficientEvidence
        ),
        "PER-B010: empty inventory is Inconclusive/InsufficientEvidence, got {:?}",
        empty.effectiveness
    );
}

/// PER-B011: `current=false` is a boolean overdue fail; joiner/leaver/mover honesty is absent.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b011_overdue_flag_exists_lifecycle_honesty_does_not() {
    let catalog = load_catalog();
    let at = collected(1);
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "inventory.complete",
        "org:acme",
        &[
            ("kind", EvidenceValue::String("user".into())),
            ("authoritative", EvidenceValue::Bool(true)),
        ],
        at,
    ));
    for user in ["user:ada", "user:bea"] {
        set.insert(seal(
            "inventory.subject",
            user,
            &[
                ("kind", EvidenceValue::String("user".into())),
                ("id", EvidenceValue::String(user.into())),
            ],
            at,
        ));
    }
    set.insert(seal(
        "evidence.personnel.training",
        "user:ada",
        &[
            ("subject_id", EvidenceValue::String("user:ada".into())),
            ("current", EvidenceValue::Bool(true)),
            (
                "training_kind",
                EvidenceValue::String("role-specific".into()),
            ),
        ],
        at,
    ));
    set.insert(seal(
        "evidence.personnel.training",
        "user:bea",
        &[
            ("subject_id", EvidenceValue::String("user:bea".into())),
            ("current", EvidenceValue::Bool(false)),
            (
                "training_kind",
                EvidenceValue::String("role-specific".into()),
            ),
        ],
        at,
    ));

    let overdue = evaluate_catalog_test(&catalog, "test.personnel.training-current-all", &set);
    assert_eq!(
        overdue.effectiveness,
        Effectiveness::Ineffective,
        "PER-B011: current=false is a boolean overdue fail; {}",
        overdue.rationale
    );
    let pop = overdue.population.expect("all-subjects population");
    assert!(
        pop.failing_subjects.iter().any(|id| id == "user:bea"),
        "PER-B011: overdue subject must be named; {:?}",
        pop.failing_subjects
    );

    let jml = evaluate_catalog_test(
        &catalog,
        "test.personnel.jml-process-attested",
        &EvidenceSet::new(),
    );
    assert_eq!(jml.effectiveness, Effectiveness::ManualReviewRequired);
}

/// PER-B012: baseline suite is registered; catalog IDs stay camelCase-JSON / SHA-256 / no v4 law.
#[test]
#[ignore = "superseded by sdd_personnel_security_target"]
fn per_b012_dual_suite_baseline_row_and_serde_law() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("sdd_personnel_security_baseline")
            && cargo.contains("tests/contracts/personnel_security.baseline.rs"),
        "PER-B012: baseline [[test]] must be registered (tests/contracts is not auto-discovered)"
    );

    let identity = Identity::new(IdentityId::new("user:ada"), IdentityKind::User);
    let json = serde_json::to_value(&identity).unwrap();
    assert!(json.get("displayName").is_none());
    assert_eq!(json["id"], "user:ada");
}
