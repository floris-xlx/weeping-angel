//! Target suite for the IAM Canonical Assurance Catalog (IAM catalog).
//!
//! Encodes DESIRED behavior in `docs/specs/iam-canonical-assurance-catalog.md`
//! §4 / §5 (IAM-001…016). Must stay RED on the current tree: no
//! `control.identity.*` family, no `evidence.identity.*` contracts, no
//! population fixtures, and `CoverageAtLeast` still a stub. Do not
//! `#[ignore]` these tests and do not implement catalog content here.
//!
//! Consumes the catalog infrastructure catalog tree, typed evidence evidence envelopes, and
//! population runtime population evaluator. Does not fork a second loader,
//! `EvidenceValue`, or `AllSubjects` implementation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssetId, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus,
};
use weeping_angel_collector::github::GITHUB_EVIDENCE_TYPES;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceError, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::load_framework_pack;

const PLACEHOLDER_RATIONALE: &str = "subject coverage remains partial unless the threshold is met";

const CANONICAL_IDENTITY_CONTROLS: &[&str] = &[
    "control.identity.unique-user-identities",
    "control.identity.mfa",
    "control.identity.privileged-mfa",
    "control.identity.strong-authentication-policy",
    "control.identity.privileged-inventory",
    "control.identity.least-privilege",
    "control.identity.privileged-access-minimization",
    "control.identity.access-approval",
    "control.identity.periodic-access-review",
    "control.identity.inactive-account-lifecycle",
    "control.identity.terminated-user-removal",
    "control.identity.joiner-mover-leaver",
    "control.identity.service-account-inventory",
    "control.identity.service-account-ownership",
    "control.identity.service-account-credential-governance",
    "control.identity.break-glass-access",
    "control.identity.shared-account-restriction",
    "control.identity.credential-management",
    "control.identity.privileged-role-change-monitoring",
    "control.identity.external-guest-access",
    "control.identity.stale-privileged-membership",
    "control.identity.access-revocation-timeliness",
    "control.identity.segregation-of-duties",
];

const CANONICAL_IDENTITY_EVIDENCE: &[&str] = &[
    "evidence.identity.inventory",
    "evidence.identity.authentication-state",
    "evidence.identity.mfa-status",
    "evidence.identity.privileged-membership",
    "evidence.identity.role-membership",
    "evidence.identity.last-active",
    "evidence.identity.account-status",
    "evidence.identity.account-owner",
    "evidence.identity.access-review",
    "evidence.identity.lifecycle-event",
    "evidence.identity.service-account",
    "evidence.identity.external-access",
];

const REQUIRED_IDENTITY_TESTS: &[&str] = &[
    "test.identity.mfa-enabled",
    "test.identity.privileged-mfa-enabled",
    "test.identity.no-inactive-privileged-accounts",
    "test.identity.no-terminated-active-accounts",
    "test.identity.all-service-accounts-have-owner",
    "test.identity.access-review-current",
    "test.identity.no-unapproved-guest-access",
];

const EXTRA_IDENTITY_TESTS: &[&str] = &[
    "test.identity.unique-user-identities",
    "test.identity.strong-authentication-policy",
    "test.identity.privileged-inventory-complete",
    "test.identity.least-privilege",
    "test.identity.privileged-access-minimized",
    "test.identity.access-approval-recorded",
    "test.identity.jml-events-recorded",
    "test.identity.service-accounts-inventoried",
    "test.identity.service-account-credentials-governed",
    "test.identity.break-glass-governed",
    "test.identity.no-ungoverned-shared-accounts",
    "test.identity.credentials-managed",
    "test.identity.privileged-role-changes-monitored",
    "test.identity.no-stale-privileged-membership",
    "test.identity.revocation-timely",
    "test.identity.sod-review",
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

const HYBRID_OR_MANUAL_CONTROLS: &[&str] = &[
    "control.identity.access-approval",
    "control.identity.periodic-access-review",
    "control.identity.segregation-of-duties",
];

const FORBIDDEN_PROVIDER_TOKENS: &[&str] = &[
    "okta",
    "entra",
    "azure-ad",
    "google-workspace",
    "workspace",
    "github",
    "aws",
    "cognito",
];

const FORBIDDEN_FRAMEWORK_TOKENS: &[&str] = &[
    "iso27001",
    "iso-27001",
    "soc2",
    "soc-2",
    "nis2",
    "dora",
    "gdpr",
];

const ISO_IAM_CONTROLS: &[&str] = &[
    "access.mfa.privileged",
    "access.least-privilege",
    "access.periodic-review",
    "personnel.access-termination",
];

const ISO_IAM_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.8.5", "access.mfa.privileged"),
    ("iso27001:a.8.2", "access.mfa.privileged"),
    ("iso27001:a.8.2", "access.least-privilege"),
    ("iso27001:a.8.3", "access.least-privilege"),
    ("iso27001:a.5.15", "access.least-privilege"),
    ("iso27001:a.5.18", "access.periodic-review"),
    ("iso27001:a.5.16", "personnel.access-termination"),
    ("iso27001:a.6.5", "personnel.access-termination"),
];

const POPULATION_OPS: &[&str] = &[
    "all-subjects",
    "all_subjects",
    "coverage-at-least",
    "coverage_at_least",
    "CoverageAtLeast",
    "AllSubjects",
    "none-subjects",
    "none_subjects",
];

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn walk_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_files(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_files(dir, "rs", out);
}

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
}

fn product_rs_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    walk_rs_files(&manifest_dir().join("apps/cli/src"), &mut files);
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
        "IAM-001: catalog infrastructure catalog tree catalog/canonical/v1 must exist so the IAM family can load"
    );
    dir
}

fn catalog_toml_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(&catalog_v1_dir(), "toml", &mut files);
    assert!(
        !files.is_empty(),
        "IAM-001: catalog/canonical/v1 must contain TOML documents"
    );
    files
}

fn identity_catalog_text() -> String {
    let mut chunks = Vec::new();
    for path in catalog_toml_files() {
        let text = fs::read_to_string(&path).unwrap();
        let rel = path.to_string_lossy();
        if rel.contains("identity")
            || text.contains("control.identity.")
            || text.contains("evidence.identity.")
            || text.contains("test.identity.")
        {
            chunks.push(text);
        }
    }
    assert!(
        !chunks.is_empty(),
        "IAM family documents (control|evidence|test.identity.*) must be present under catalog/canonical/v1"
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

fn fixture_dir(name: &str) -> PathBuf {
    manifest_dir()
        .join("fixtures/assurance/canonical/v1/identity")
        .join(name)
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
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
            collector_id: "fixture.iam-target".into(),
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

fn coverage_100(kind: &str, evidence_type: &str, field: &str) -> TestExpr {
    TestExpr::CoverageAtLeast {
        selector: SubjectSelector {
            kind: Some(kind.into()),
            id: None,
        },
        evidence: EvidenceSelector {
            evidence_type: EvidenceType::new(evidence_type),
            subject_selector: SubjectSelector {
                kind: Some(kind.into()),
                id: None,
            },
            field: Some(field.into()),
            freshness: None,
        },
        percentage: "100".into(),
    }
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

fn inventory_user(set: &mut EvidenceSet, id: &str, kind: &str) {
    set.insert(seal(
        "evidence.identity.inventory",
        id,
        &[
            ("subject_id", id),
            ("account_kind", kind),
            ("unique_key", id),
            ("authoritative", "true"),
        ],
        collected(1),
    ));
}

fn mfa(set: &mut EvidenceSet, id: &str, enabled: bool) {
    set.insert(seal(
        "evidence.identity.mfa-status",
        id,
        &[
            ("subject_id", id),
            ("mfa_enabled", if enabled { "true" } else { "false" }),
        ],
        collected(1),
    ));
}

fn privileged(set: &mut EvidenceSet, id: &str, privileged: bool) {
    set.insert(seal(
        "evidence.identity.privileged-membership",
        id,
        &[
            ("subject_id", id),
            ("privileged", if privileged { "true" } else { "false" }),
            ("roles", if privileged { "admin" } else { "user" }),
            ("membership_observed_at", "2026-08-18T12:00:00Z"),
        ],
        collected(1),
    ));
}

fn account_status(set: &mut EvidenceSet, id: &str, status: &str) {
    set.insert(seal(
        "evidence.identity.account-status",
        id,
        &[("subject_id", id), ("status", status)],
        collected(1),
    ));
}

fn last_active(set: &mut EvidenceSet, id: &str, inactive: bool, hours_ago: i64) {
    set.insert(seal(
        "evidence.identity.last-active",
        id,
        &[
            ("subject_id", id),
            ("inactive", if inactive { "true" } else { "false" }),
            (
                "last_active_at",
                if hours_ago > 24 * 90 {
                    "2025-01-01T00:00:00Z"
                } else {
                    "2026-08-18T11:00:00Z"
                },
            ),
        ],
        collected(hours_ago),
    ));
}

fn service_account(set: &mut EvidenceSet, id: &str, owner: Option<&str>) {
    set.insert(seal(
        "evidence.identity.service-account",
        id,
        &[("subject_id", id), ("is_service_account", "true")],
        collected(1),
    ));
    let (assigned, owner_id) = match owner {
        Some(o) => ("true", o),
        None => ("false", ""),
    };
    set.insert(seal(
        "evidence.identity.account-owner",
        id,
        &[
            ("subject_id", id),
            ("owner_assigned", assigned),
            ("owner_subject_id", owner_id),
        ],
        collected(1),
    ));
}

fn access_review(set: &mut EvidenceSet, id: &str, hours_ago: i64, result: &str) {
    let reviewed = if hours_ago > 24 {
        "2026-07-01T00:00:00Z"
    } else {
        "2026-08-18T11:00:00Z"
    };
    set.insert(seal(
        "evidence.identity.access-review",
        id,
        &[
            ("subject_id", id),
            ("reviewed_at", reviewed),
            ("result", result),
        ],
        collected(hours_ago),
    ));
}

fn lifecycle(set: &mut EvidenceSet, id: &str, event: &str) {
    set.insert(seal(
        "evidence.identity.lifecycle-event",
        id,
        &[
            ("subject_id", id),
            ("event", event),
            ("occurred_at", "2026-08-01T00:00:00Z"),
            ("approved", "true"),
        ],
        collected(1),
    ));
}

fn guest(set: &mut EvidenceSet, id: &str, approved: bool) {
    set.insert(seal(
        "evidence.identity.external-access",
        id,
        &[
            ("subject_id", id),
            ("external", "true"),
            ("approved", if approved { "true" } else { "false" }),
        ],
        collected(1),
    ));
}

fn healthy_population() -> EvidenceSet {
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.identity.inventory",
        "org:healthy",
        &[
            ("population_id", "org:healthy"),
            ("authoritative", "true"),
            ("account_kind", "organization"),
        ],
        collected(1),
    ));
    for id in ["user:alice", "user:bob"] {
        inventory_user(&mut set, id, "user");
        mfa(&mut set, id, true);
        privileged(&mut set, id, false);
        account_status(&mut set, id, "active");
        last_active(&mut set, id, false, 1);
        access_review(&mut set, id, 1, "approved");
    }
    inventory_user(&mut set, "user:admin", "user");
    mfa(&mut set, "user:admin", true);
    privileged(&mut set, "user:admin", true);
    account_status(&mut set, "user:admin", "active");
    last_active(&mut set, "user:admin", false, 1);
    access_review(&mut set, "user:admin", 1, "approved");
    inventory_user(&mut set, "sa:deploy", "service");
    service_account(&mut set, "sa:deploy", Some("user:alice"));
    set
}

fn assert_population_not_placeholder(
    result: &weeping_angel_control_test::ControlTestResult,
    label: &str,
) {
    assert_ne!(
        result.rationale, PLACEHOLDER_RATIONALE,
        "{label}: CoverageAtLeast must not stay a population-runtime stub"
    );
    if result.effectiveness == Effectiveness::PartiallyEffective {
        panic!("{label}: stub PartiallyEffective is not a population result");
    }
}

fn control_record_automation(text: &str, control_id: &str) -> String {
    let marker = format!("id = \"{control_id}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing control record {control_id}"));
    let window = &text[start..start + 800.min(text.len() - start)];
    for key in ["automation", "class", "kind"] {
        let needle = format!("{key} = \"");
        if let Some(idx) = window.find(&needle) {
            let rest = &window[idx + needle.len()..];
            if let Some(end) = rest.find('"') {
                return rest[..end].to_ascii_lowercase();
            }
        }
    }
    panic!("{control_id} must declare automation/class/kind (Automated|Hybrid|Manual)");
}

fn test_expression_window(text: &str, test_id: &str) -> String {
    let marker = format!("id = \"{test_id}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing test record {test_id}"));
    text[start..start + 1200.min(text.len() - start)].to_string()
}

fn expression_is_existence_only(window: &str) -> bool {
    let lower = window.to_ascii_lowercase();
    let has_exists = lower.contains("op = \"exists\"") || lower.contains("exists(");
    let has_population = POPULATION_OPS
        .iter()
        .any(|op| window.contains(op) || lower.contains(&op.to_ascii_lowercase()));
    has_exists && !has_population
}

#[test]
fn iam_000_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !toml.contains("sdd_iam_catalog_baseline")
            && !toml.contains("tests/contracts/iam_catalog.baseline.rs")
            && harness_src().contains("iam_catalog.target.rs")
            && harness_src().contains("iam_catalog.target.rs"),
        "dual-suite sdd_iam_catalog_baseline + sdd_iam_catalog_target must be wired as a harness module"
    );
}

#[test]
fn iam_001_catalog_loader_loads_iam_family_offline() {
    let dir = catalog_v1_dir();
    assert!(
        dir.join("manifest.toml").is_file(),
        "IAM-001: catalog/canonical/v1/manifest.toml is required"
    );
    let crate_dir = manifest_dir().join("crates/weeping-angel-canonical-catalog");
    assert!(
        crate_dir.is_dir(),
        "IAM-001: consume catalog infrastructure crate weeping-angel-canonical-catalog; do not invent a second loader"
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
            "IAM-001: CanonicalCatalog API missing `{needle}`"
        );
    }
    let members = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        members.contains("weeping-angel-canonical-catalog"),
        "IAM-001: catalog crate must be a workspace member"
    );
    let text = identity_catalog_text();
    assert!(
        text.contains("control.identity.mfa"),
        "IAM-001: loaded catalog must include the identity family"
    );
}

#[test]
fn iam_002_iam_slice_digest_is_deterministic() {
    assert!(
        manifest_dir()
            .join("crates/weeping-angel-canonical-catalog")
            .is_dir(),
        "IAM-002: catalog infrastructure catalog crate must exist so digest is not a second implementation"
    );
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    assert!(
        rust.contains("fn digest") || rust.contains("CatalogDigest"),
        "IAM-002: catalog infrastructure digest API must exist for the IAM slice"
    );
    let text = identity_catalog_text();
    assert!(
        text.contains("control.identity."),
        "IAM-002: digest inputs include identity controls"
    );
}

#[test]
fn iam_003_twenty_three_identity_controls_are_stable() {
    let text = identity_catalog_text();
    let ids = quoted_ids(&text, "control.identity.");
    for id in CANONICAL_IDENTITY_CONTROLS {
        assert!(
            ids.contains(*id),
            "IAM-003: missing control `{id}` (have {ids:?})"
        );
    }
    let identity_only: Vec<_> = ids
        .iter()
        .filter(|id| id.starts_with("control.identity."))
        .collect();
    assert!(
        (20..=30).contains(&identity_only.len()),
        "IAM-003: expected 20–30 independently assessable identity controls, found {} ({identity_only:?})",
        identity_only.len()
    );
    assert_eq!(
        CANONICAL_IDENTITY_CONTROLS.len(),
        23,
        "pinned family size is 23"
    );
    for id in &ids {
        assert!(
            id.starts_with("control.identity."),
            "IAM-003: identity control id must stay in control.identity.* ({id})"
        );
        assert_eq!(
            id,
            &id.to_ascii_lowercase(),
            "IAM-003: ids are lowercase ({id})"
        );
        assert!(
            !id.contains('_'),
            "IAM-003: catalog ids use hyphen segments, not underscores ({id})"
        );
    }
}

#[test]
fn iam_003_controls_declare_domains_evidence_and_tests() {
    let text = identity_catalog_text();
    for id in CANONICAL_IDENTITY_CONTROLS {
        let marker = format!("id = \"{id}\"");
        let start = text
            .find(&marker)
            .unwrap_or_else(|| panic!("IAM-003: control record {id} missing"));
        let window = &text[start..start + 900.min(text.len() - start)];
        assert!(
            window.contains("domains") || window.contains("domain"),
            "IAM-003: {id} must declare domain(s)"
        );
        assert!(
            window.contains("evidence"),
            "IAM-003: {id} must declare evidence requirements"
        );
        assert!(
            window.contains("tests") || window.contains("test"),
            "IAM-003: {id} must declare test refs"
        );
        let class = control_record_automation(&text, id);
        assert!(
            matches!(class.as_str(), "automated" | "hybrid" | "manual"),
            "IAM-003: {id} automation class must be Automated|Hybrid|Manual, got {class}"
        );
    }
}

#[test]
fn iam_004_identity_evidence_types_are_declared_facts() {
    let text = identity_catalog_text();
    let ids = quoted_ids(&text, "evidence.identity.");
    for id in CANONICAL_IDENTITY_EVIDENCE {
        assert!(
            ids.contains(*id),
            "IAM-004: missing evidence contract `{id}`"
        );
    }
    let forbidden_conclusions = [
        "compliant",
        "mfa control passed",
        "least privilege effective",
        "periodic review effective",
        "inactive lifecycle effective",
    ];
    let lower = text.to_ascii_lowercase();
    for phrase in forbidden_conclusions {
        assert!(
            !lower.contains(phrase),
            "IAM-004: evidence contracts are facts, not conclusions (`{phrase}`)"
        );
    }
    for id in CANONICAL_IDENTITY_EVIDENCE {
        let control_hits = CANONICAL_IDENTITY_CONTROLS
            .iter()
            .filter(|_| text.contains(id))
            .count();
        assert!(
            control_hits > 0,
            "IAM-004: evidence `{id}` must not be orphaned"
        );
    }
}

#[test]
fn iam_005_required_and_non_orphan_identity_tests_are_declared() {
    let text = identity_catalog_text();
    let ids = quoted_ids(&text, "test.identity.");
    for id in REQUIRED_IDENTITY_TESTS
        .iter()
        .chain(EXTRA_IDENTITY_TESTS.iter())
    {
        assert!(ids.contains(*id), "IAM-005: missing test `{id}`");
    }
    for id in &ids {
        let window = test_expression_window(&text, id);
        assert!(
            window.contains("control.identity.") || window.contains("control ="),
            "IAM-005: {id} must reference a control"
        );
    }
}

#[test]
fn iam_006_validator_rejects_provider_tokens_in_iam_ids() {
    let crate_dir = manifest_dir().join("crates/weeping-angel-canonical-catalog");
    assert!(
        crate_dir.is_dir(),
        "IAM-006: catalog infrastructure validator crate is required"
    );
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    for token in FORBIDDEN_PROVIDER_TOKENS {
        assert!(
            rust.to_ascii_lowercase().contains(token),
            "IAM-006: validator must reserve provider token `{token}`"
        );
    }
    let text = identity_catalog_text();
    for id in quoted_ids(&text, "control.")
        .into_iter()
        .chain(quoted_ids(&text, "evidence."))
        .chain(quoted_ids(&text, "test."))
    {
        if !id.contains(".identity.") {
            continue;
        }
        for token in FORBIDDEN_PROVIDER_TOKENS {
            let segment = format!(".{token}.");
            let suffix = format!(".{token}");
            assert!(
                !id.contains(&segment) && !id.ends_with(&suffix),
                "IAM-006: provider token `{token}` leaked into id `{id}`"
            );
        }
    }
}

#[test]
fn iam_007_canonical_iam_content_has_no_framework_tokens() {
    let crate_dir = manifest_dir().join("crates/weeping-angel-canonical-catalog");
    assert!(
        crate_dir.is_dir(),
        "IAM-007: catalog infrastructure validator crate is required"
    );
    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            rust.to_ascii_lowercase().contains(&token.replace('-', "")),
            "IAM-007: validator must reserve framework token `{token}`"
        );
    }
    let text = identity_catalog_text();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            !lower.contains(token),
            "IAM-007: canonical IAM content must not mention `{token}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn iam_008_iso_pack_is_unchanged_and_has_no_control_identity() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_IAM_CONTROLS {
        assert!(
            control_ids.contains(id),
            "IAM-008: ISO sliver `{id}` must remain (have {control_ids:?})"
        );
    }
    assert!(
        !control_ids
            .iter()
            .any(|id| id.starts_with("control.identity.")),
        "IAM-008: do not move IAM controls into the ISO pack"
    );

    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_IAM_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "IAM-008/016: mapping {from} → {to} must stay in the ISO pack"
        );
    }
    assert!(
        !mappings.contains("control.identity."),
        "IAM-008: ISO mappings must not retarget control.identity.*"
    );
}

#[test]
fn iam_009_privileged_mfa_is_population_not_existence() {
    let text = identity_catalog_text();
    let window = test_expression_window(&text, "test.identity.privileged-mfa-enabled");
    assert!(
        !expression_is_existence_only(&window),
        "IAM-009: test.identity.privileged-mfa-enabled must not be Exists(some mfa-status)"
    );
    assert!(
        POPULATION_OPS.iter().any(|op| window.contains(op)
            || window
                .to_ascii_lowercase()
                .contains(&op.to_ascii_lowercase()))
            || window.contains("privileged"),
        "IAM-009: privileged-mfa-enabled must describe the privileged population"
    );

    let mut some_mfa = EvidenceSet::new();
    mfa(&mut some_mfa, "user:random", true);
    let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.identity.mfa-status",
    )));
    let (exists_ok, _) = result_json(
        "test.identity.privileged-mfa-enabled",
        "control.identity.privileged-mfa",
        exists,
        &some_mfa,
    );
    assert_eq!(
        exists_ok.effectiveness,
        Effectiveness::Effective,
        "sanity: a lone MFA envelope satisfies Exists"
    );

    let (pop, json) = result_json(
        "test.identity.privileged-mfa-enabled",
        "control.identity.privileged-mfa",
        coverage_100(
            "privilegedIdentity",
            "evidence.identity.mfa-status",
            "mfa_enabled",
        ),
        &some_mfa,
    );
    assert_population_not_placeholder(&pop, "IAM-009 lone envelope");
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "IAM-009: a single MFA envelope must not pass all privileged identities have MFA; json={json}"
    );

    let mut fail_set = healthy_population();
    mfa(&mut fail_set, "user:admin", false);
    privileged(&mut fail_set, "user:admin", true);
    let (failed, fail_json) = result_json(
        "test.identity.privileged-mfa-enabled",
        "control.identity.privileged-mfa",
        coverage_100(
            "privilegedIdentity",
            "evidence.identity.mfa-status",
            "mfa_enabled",
        ),
        &fail_set,
    );
    assert_population_not_placeholder(&failed, "IAM-009 privileged-without-mfa");
    assert_eq!(
        failed.effectiveness,
        Effectiveness::Ineffective,
        "IAM-009: privileged user without MFA is Ineffective, got {:?}",
        failed.effectiveness
    );
    let failing = string_list(&fail_json, "failingSubjects");
    assert!(
        failing.iter().any(|s| s.contains("admin")),
        "IAM-009: failing subject must name the privileged identity; got {failing:?}"
    );
}

#[test]
fn iam_010_eight_fixtures_distinguish_missing_stale_fail_manual_exception() {
    let root = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    assert!(
        root.is_dir(),
        "IAM-010: fixtures/assurance/canonical/v1/identity must exist"
    );
    for name in IAM_FIXTURES {
        let dir = fixture_dir(name);
        assert!(
            dir.is_dir(),
            "IAM-010: fixture `{name}` is not shipped at {}",
            dir.display()
        );
        let mut files = Vec::new();
        walk_files(&dir, "json", &mut files);
        walk_files(&dir, "toml", &mut files);
        walk_files(&dir, "jsonl", &mut files);
        assert!(
            !files.is_empty(),
            "IAM-010: fixture `{name}` must contain evidence documents"
        );
        let blob: String = files
            .iter()
            .map(|p| fs::read_to_string(p).unwrap())
            .collect();
        assert!(
            blob.contains("evidence.identity."),
            "IAM-010: fixture `{name}` must emit canonical evidence.identity.* (not source.admin.permissions)"
        );
        assert!(
            !blob.contains("source.admin.permissions"),
            "IAM-010: fixture `{name}` must not use GitHub-shaped evidence types"
        );
    }

    // In-memory populations encode the same eight intents using CoverageAtLeast
    // (population runtime). Do not treat the stub PartiallyEffective as a result.
    // Negative predicates (inactive/terminated) are declared by the catalog
    // tests on disk; here we only evaluate boolean pass-fields.
    let healthy = healthy_population();
    let (ok, _) = result_json(
        "test.identity.mfa-enabled",
        "control.identity.mfa",
        coverage_100("user", "evidence.identity.mfa-status", "mfa_enabled"),
        &healthy,
    );
    assert_population_not_placeholder(&ok, "healthy-org mfa");
    assert_eq!(ok.effectiveness, Effectiveness::Effective);

    let mut no_mfa = healthy_population();
    mfa(&mut no_mfa, "user:admin", false);
    let (fail, _) = result_json(
        "test.identity.privileged-mfa-enabled",
        "control.identity.privileged-mfa",
        coverage_100(
            "privilegedIdentity",
            "evidence.identity.mfa-status",
            "mfa_enabled",
        ),
        &no_mfa,
    );
    assert_population_not_placeholder(&fail, "privileged-without-mfa");
    assert_eq!(fail.effectiveness, Effectiveness::Ineffective);

    let mut ownerless = healthy_population();
    service_account(&mut ownerless, "sa:orphan", None);
    inventory_user(&mut ownerless, "sa:orphan", "service");
    let (sa, _) = result_json(
        "test.identity.all-service-accounts-have-owner",
        "control.identity.service-account-ownership",
        coverage_100(
            "serviceAccount",
            "evidence.identity.account-owner",
            "owner_assigned",
        ),
        &ownerless,
    );
    assert_population_not_placeholder(&sa, "service-account-without-owner");
    assert_eq!(sa.effectiveness, Effectiveness::Ineffective);

    let mut stale = healthy_population();
    access_review(&mut stale, "user:alice", 48, "approved");
    let (stale_r, _) = result_json(
        "test.identity.access-review-current",
        "control.identity.periodic-access-review",
        coverage_100("user", "evidence.identity.access-review", "reviewed_at"),
        &stale,
    );
    assert_population_not_placeholder(&stale_r, "stale-access-review");
    assert_eq!(
        stale_r.effectiveness,
        Effectiveness::StaleEvidence,
        "IAM-010: stale review must be StaleEvidence, not Ineffective-as-missing"
    );

    let mut guests = healthy_population();
    inventory_user(&mut guests, "user:guest", "guest");
    guest(&mut guests, "user:guest", false);
    let (guest_r, _) = result_json(
        "test.identity.no-unapproved-guest-access",
        "control.identity.external-guest-access",
        coverage_100("user", "evidence.identity.external-access", "approved"),
        &guests,
    );
    assert_population_not_placeholder(&guest_r, "unapproved guest");
    assert_eq!(guest_r.effectiveness, Effectiveness::Ineffective);
}

#[test]
fn iam_011_partial_inventory_cannot_be_effective() {
    let mut partial = EvidenceSet::new();
    inventory_user(&mut partial, "user:alice", "user");
    mfa(&mut partial, "user:alice", true);
    // Non-authoritative / incomplete population: known subject without envelopes
    // for the rest of the org, and no authoritative inventory marker.
    let (result, json) = result_json(
        "test.identity.mfa-enabled",
        "control.identity.mfa",
        coverage_100("user", "evidence.identity.mfa-status", "mfa_enabled"),
        &partial,
    );
    assert_population_not_placeholder(&result, "IAM-011 partial-inventory");
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "IAM-011: partial inventory must not yield Effective on all-subjects tests"
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "IAM-011: unknown/partial population → InsufficientEvidence, got {:?} json={json}",
        result.effectiveness
    );
}

#[test]
fn iam_012_approved_break_glass_exception_is_not_silent_effective() {
    let mut set = healthy_population();
    inventory_user(&mut set, "user:break-glass", "break-glass");
    privileged(&mut set, "user:break-glass", true);
    mfa(&mut set, "user:break-glass", false);
    account_status(&mut set, "user:break-glass", "active");

    let mut ex = Exception::new(
        ExceptionId::new("exc:break-glass"),
        "timeboxed emergency access",
    );
    ex.status = ExceptionStatus::Approved;
    ex.control_id = Some(ControlId::new("control.identity.break-glass-access"));
    let json = serde_json::to_value(&ex).unwrap();
    assert!(
        json.get("subjects").is_some() || json.get("appliesTo").is_some(),
        "IAM-012: Exception IR must bind the break-glass subject"
    );

    let (result, out) = result_json(
        "test.identity.privileged-mfa-enabled",
        "control.identity.privileged-mfa",
        coverage_100(
            "privilegedIdentity",
            "evidence.identity.mfa-status",
            "mfa_enabled",
        ),
        &set,
    );
    assert_population_not_placeholder(&result, "IAM-012 break-glass");
    assert_eq!(
        result.effectiveness,
        Effectiveness::ExceptionApproved,
        "IAM-012: approved unexpired break-glass exception → ExceptionApproved, got {:?} {out}",
        result.effectiveness
    );
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert_ne!(result.effectiveness, Effectiveness::Ineffective);

    let mut expired = ex;
    expired.status = ExceptionStatus::Expired;
    assert_ne!(
        expired.status,
        ExceptionStatus::Approved,
        "expired exceptions must not remain Approved"
    );
}

#[test]
fn iam_013_approval_sod_and_review_stay_hybrid_or_manual() {
    let text = identity_catalog_text();
    for id in HYBRID_OR_MANUAL_CONTROLS {
        let class = control_record_automation(&text, id);
        assert!(
            class == "hybrid" || class == "manual",
            "IAM-013: {id} must be Hybrid or Manual, got {class}"
        );
    }

    let mut tech_only = healthy_population();
    lifecycle(&mut tech_only, "user:alice", "access-approved");
    for (test_id, control_id) in [
        (
            "test.identity.access-approval-recorded",
            "control.identity.access-approval",
        ),
        (
            "test.identity.access-review-current",
            "control.identity.periodic-access-review",
        ),
        (
            "test.identity.sod-review",
            "control.identity.segregation-of-duties",
        ),
    ] {
        let window = test_expression_window(&text, test_id);
        assert!(
            !expression_is_existence_only(&window)
                || window.to_ascii_lowercase().contains("manual"),
            "IAM-013: {test_id} must not auto-pass as Exists(one technical envelope)"
        );
        let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
            "evidence.identity.lifecycle-event",
        )));
        let (via_exists, _) = result_json(test_id, control_id, exists, &tech_only);
        assert_eq!(
            via_exists.effectiveness,
            Effectiveness::Effective,
            "sanity: Exists(lifecycle-event) would auto-pass"
        );
        let _ = control_id;
    }
}

#[test]
fn iam_014_credential_shaped_iam_facts_are_rejected() {
    let obs = EvidenceObservation::new(EvidenceType::new("evidence.identity.mfa-status"))
        .with_fact("password", "hunter2")
        .with_fact("subject_id", "user:alice");
    let err = EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.iam-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("user:alice"),
        },
    )
    .expect_err("IAM-014: password fact must not seal");
    assert!(matches!(err, EvidenceError::CredentialInPayload { .. }));

    let claim = EvidenceObservation::new(EvidenceType::new("evidence.identity.mfa-status"))
        .with_narrative("control is ISO 27001 compliant");
    let claim_err = EvidenceEnvelope::seal(
        claim,
        EvidenceProvenance {
            collector_id: "fixture.iam-target".into(),
            collected_at: collected(1),
            scope: "target".into(),
            asset: AssetId::new("user:alice"),
        },
    )
    .expect_err("IAM-014: compliance narrative must not seal");
    assert!(matches!(claim_err, EvidenceError::ComplianceClaim { .. }));
}

#[test]
fn iam_015_no_identity_provider_collectors_or_framework_sdk() {
    let collector_src = crate_src("weeping-angel-collector");
    for name in ["entra", "okta", "google_workspace", "workspace", "cognito"] {
        assert!(
            !collector_src.join(name).exists(),
            "IAM-015: do not add a {name} collector in this slice"
        );
    }
    assert!(
        !GITHUB_EVIDENCE_TYPES
            .iter()
            .any(|t| t.starts_with("evidence.identity.")),
        "IAM-015: GitHub collector must keep emitting source.* only"
    );

    let fw = fs::read_to_string(manifest_dir().join("crates/weeping-angel-framework/Cargo.toml"))
        .unwrap();
    for dep in ["reqwest", "octocrab", "octorust", "azure", "okta"] {
        assert!(
            !fw.contains(dep),
            "IAM-015: framework crate must not grow a provider SDK ({dep})"
        );
    }

    let rust = crate_sources_joined("weeping-angel-control-test");
    assert!(
        rust.contains("enum TestExpr"),
        "IAM-015: consume the existing evaluator"
    );
    assert!(
        !rust.contains("fn iam_all_subjects") && !rust.contains("struct IamPopulation"),
        "IAM-015: do not locally complete CoverageAtLeast / AllSubjects in an IAM fork"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn iam_016_iso_iam_sliver_ids_remain_the_gate_for_iso_suite() {
    let pack = load_framework_pack("iso-27001", "2022").unwrap();
    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    let expected = [
        (
            "test.access.mfa.privileged",
            "access.mfa.privileged",
            "source.admin.permissions",
        ),
        (
            "test.access.least-privilege",
            "access.least-privilege",
            "source.collaborator.permission",
        ),
        (
            "test.access.periodic-review",
            "access.periodic-review",
            "policy.access.reviewed",
        ),
        (
            "test.personnel.access-termination",
            "personnel.access-termination",
            "personnel.access.terminated",
        ),
    ];
    for (test_id, control_id, evidence) in expected {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("IAM-016: ISO pack lost {test_id}"));
        assert_eq!(test.control_id.as_str(), control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(evidence)],
            "{test_id} required evidence must stay GitHub/policy shaped"
        );
    }
}

#[test]
fn iam_sdd_slice_doc_is_not_catalog_ssot() {
    let iam =
        fs::read_to_string(manifest_dir().join("docs/specs/iam-canonical-assurance-catalog.md"))
            .unwrap();
    assert!(
        iam.contains("control.identity.mfa"),
        "IAM slice SDD must remain the SSOT for this family"
    );
    let catalog_ssot = manifest_dir().join("docs/specs/canonical-assurance-catalog-v1.md");
    if catalog_ssot.is_file() {
        let text = fs::read_to_string(&catalog_ssot).unwrap();
        assert!(
            !text.contains("sdd_iam_catalog_target"),
            "do not overwrite catalog infrastructure SSOT with this slice's suite ids"
        );
    }
}
