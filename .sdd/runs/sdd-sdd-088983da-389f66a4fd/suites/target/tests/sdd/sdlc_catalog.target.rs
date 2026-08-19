//! Target suite for the SDLC Canonical Assurance Catalog (Prompt 05).
//!
//! Encodes DESIRED behavior in `docs/sdd/sdd-sdd-088983da-389f66a4fd/spec.md`
//! §4 / §5 (AC-1…AC-16 / SDLC-001…016). Must stay RED on the current tree:
//! no `control.source.default-branch-protection` family, no
//! `evidence.repository|cicd|deployment|release|supply-chain.*` contracts,
//! no seven multi-repo fixtures, and no `sdd_sdlc_catalog_*` harness rows.
//! Do not `#[ignore]` these tests and do not implement catalog content here.
//!
//! Consumes the Prompt 01 catalog tree, Prompt 02 evidence envelopes, and
//! Prompt 03 population evaluator. Does not fork a second loader,
//! `EvidenceValue`, or `resolve_repository_inventory`.

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
use weeping_angel_canonical_catalog::{CATALOG_SCHEMA, CanonicalCatalog};
use weeping_angel_collector::github::GITHUB_EVIDENCE_TYPES;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::load_framework_pack;

const PINNED_FIXTURE_CONTROL: &str = "control.source.protected-branch";
const PINNED_FIXTURE_EVIDENCE: &str = "evidence.source.protected-branch";
const PINNED_FIXTURE_TEST: &str = "test.source.protected-branch";

/// Spec §4.3 — 26 independently assessable SDLC controls.
const CANONICAL_SDLC_CONTROLS: &[&str] = &[
    "control.source.repository-inventory",
    "control.source.visibility-governance",
    "control.source.default-branch-protection",
    "control.source.force-push-restricted",
    "control.source.branch-deletion-restricted",
    "control.source.required-review",
    "control.source.minimum-reviewer-count",
    "control.source.review-ownership",
    "control.source.required-status-checks",
    "control.source.admin-bypass-governance",
    "control.source.signed-commits",
    "control.source.secret-scanning",
    "control.source.code-scanning",
    "control.source.dependency-scanning",
    "control.source.dependency-update-monitoring",
    "control.supply-chain.dependency-integrity",
    "control.cicd.workflow-permissions",
    "control.release.protected-environment",
    "control.release.authorization",
    "control.release.authority-separation",
    "control.supply-chain.build-provenance",
    "control.supply-chain.artifact-integrity",
    "control.source.change-traceability",
    "control.source.security-review",
    "control.source.secure-development-policy",
    "control.supply-chain.unsupported-components",
];

/// Spec §4.4 — facts, not conclusions.
const CANONICAL_SDLC_EVIDENCE: &[&str] = &[
    "evidence.repository.inventory",
    "evidence.repository.visibility",
    "evidence.repository.default-branch",
    "evidence.repository.branch-protection",
    "evidence.repository.review-policy",
    "evidence.repository.review-ownership",
    "evidence.repository.security-scanning",
    "evidence.repository.dependency-scanning",
    "evidence.repository.commit-signing",
    "evidence.repository.change-trace",
    "evidence.repository.security-review",
    "evidence.repository.secure-development-policy",
    "evidence.cicd.workflow-permissions",
    "evidence.cicd.status-checks",
    "evidence.deployment.environment-protection",
    "evidence.release.authorization",
    "evidence.supply-chain.build-provenance",
    "evidence.supply-chain.artifact-integrity",
    "evidence.supply-chain.lockfile-state",
    "evidence.supply-chain.component-support",
];

/// Prompt 05 required example test ids (spec §4.5).
const PROMPT05_EXAMPLE_TESTS: &[&str] = &[
    "test.source.default-branches-protected",
    "test.source.force-push-restricted",
    "test.source.reviews-required",
    "test.source.minimum-reviewer-count",
    "test.source.secret-scanning-enabled",
    "test.cicd.workflow-permissions-minimized",
    "test.release.environments-protected",
    "test.source.dependency-scanning-current",
    "test.supply-chain.artifacts-have-integrity",
];

const CANONICAL_SDLC_TESTS: &[&str] = &[
    "test.source.repository-inventory-complete",
    "test.source.visibility-governed",
    "test.source.default-branches-protected",
    "test.source.force-push-restricted",
    "test.source.branch-deletion-restricted",
    "test.source.reviews-required",
    "test.source.minimum-reviewer-count",
    "test.source.review-ownership-present",
    "test.source.required-status-checks",
    "test.source.admin-bypass-governed",
    "test.source.signed-commits-required",
    "test.source.secret-scanning-enabled",
    "test.source.code-scanning-enabled",
    "test.source.dependency-scanning-current",
    "test.source.dependency-updates-monitored",
    "test.supply-chain.lockfile-integrity",
    "test.cicd.workflow-permissions-minimized",
    "test.release.environments-protected",
    "test.release.authorization-recorded",
    "test.release.authority-separated",
    "test.supply-chain.provenance-present",
    "test.supply-chain.artifacts-have-integrity",
    "test.source.changes-traceable",
    "test.source.security-review-recorded",
    "test.source.secure-development-policy-attested",
    "test.supply-chain.unsupported-components-handled",
];

const SDLC_FIXTURES: &[&str] = &[
    "healthy-org",
    "degraded-org",
    "partial-coverage",
    "unprotected-default-branch",
    "missing-scan-evidence",
    "stale-dependency-scan",
    "approved-exception",
];

const HYBRID_OR_MANUAL_CONTROLS: &[&str] = &[
    "control.release.authorization",
    "control.release.authority-separation",
    "control.source.security-review",
    "control.source.secure-development-policy",
];

const IAM_FAMILY_PINS: &[&str] = &[
    "control.identity.mfa",
    "control.identity.privileged-mfa",
    "evidence.identity.mfa-status",
    "test.identity.privileged-mfa-enabled",
];

const ISO_SOURCE_CONTROLS: &[&str] = &[
    "source.branch-protection",
    "source.required-review",
    "source.code-ownership",
    "source.security-scanning",
    "source.commit-signing",
];

const ISO_SOURCE_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.8.25", "source.branch-protection"),
    ("iso27001:a.8.25", "source.required-review"),
    ("iso27001:a.8.25", "source.code-ownership"),
    ("iso27001:a.8.26", "source.security-scanning"),
];

const PINNED_GITHUB_EVIDENCE_TYPES: &[&str] = &[
    "source.repository.exists",
    "source.repository.visibility",
    "source.default_branch",
    "source.branch.protection",
    "source.branch.required_reviews",
    "source.branch.required_status_checks",
    "source.branch.force_push_protection",
    "source.branch.deletion_protection",
    "source.codeowners.present",
    "source.admin.permissions",
    "source.collaborator.permission",
    "source.security.dependabot.enabled",
    "source.security.secret_scanning.enabled",
    "source.security.code_scanning.configured",
    "source.workflow.permissions",
    "source.workflow.review_requirement",
    "source.ruleset.present",
    "source.repository.archived",
    "source.commit.signing",
];

const FORBIDDEN_PROVIDER_TOKENS: &[&str] = &[
    "github",
    "gitlab",
    "bitbucket",
    "azure-devops",
    "azuredevops",
    "gitea",
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

const FORBIDDEN_NATIVE_ID_TOKENS: &[&str] = &["codeowners", "rulesets", "dependabot"];

const POPULATION_OPS: &[&str] = &[
    "all-subjects",
    "all_subjects",
    "AllSubjects",
    "coverage-at-least",
    "coverage_at_least",
    "CoverageAtLeast",
    "none-subjects",
    "none_subjects",
    "NoneSubjects",
    "manual-review",
    "manual_review",
    "ManualReview",
];

const SDLC_CONTROL_PREFIXES: &[&str] = &[
    "control.source.",
    "control.cicd.",
    "control.release.",
    "control.supply-chain.",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_files(dir, "rs", out);
}

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
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

fn catalog_v1_dir() -> PathBuf {
    let dir = manifest_dir().join("catalog/canonical/v1");
    assert!(
        dir.is_dir(),
        "AC-11/AC-16: Prompt 01 catalog tree catalog/canonical/v1 must exist"
    );
    dir
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1_dir()).unwrap_or_else(|e| {
        panic!("AC-16: CanonicalCatalog::load/validate must accept the SDLC slice: {e}")
    })
}

fn catalog_toml_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(&catalog_v1_dir(), "toml", &mut files);
    assert!(
        !files.is_empty(),
        "AC-16: catalog/canonical/v1 must contain TOML documents"
    );
    files
}

fn is_sdlc_family_id(id: &str) -> bool {
    if id == PINNED_FIXTURE_CONTROL || id == PINNED_FIXTURE_TEST || id == PINNED_FIXTURE_EVIDENCE {
        return false;
    }
    id.starts_with("control.source.")
        || id.starts_with("control.cicd.")
        || id.starts_with("control.release.")
        || id.starts_with("control.supply-chain.")
        || id.starts_with("evidence.repository.")
        || id.starts_with("evidence.cicd.")
        || id.starts_with("evidence.deployment.")
        || id.starts_with("evidence.release.")
        || id.starts_with("evidence.supply-chain.")
        || id.starts_with("test.source.")
        || id.starts_with("test.cicd.")
        || id.starts_with("test.release.")
        || id.starts_with("test.supply-chain.")
}

fn sdlc_catalog_text() -> String {
    let mut chunks = Vec::new();
    for path in catalog_toml_files() {
        let text = fs::read_to_string(&path).unwrap();
        if text.lines().any(|line| {
            CANONICAL_SDLC_CONTROLS.iter().any(|id| line.contains(id))
                || line.contains("evidence.repository.")
                || line.contains("evidence.cicd.")
                || line.contains("evidence.deployment.")
                || line.contains("evidence.release.")
                || line.contains("evidence.supply-chain.")
                || line.contains("control.cicd.")
                || line.contains("control.release.")
                || line.contains("control.supply-chain.")
        }) {
            chunks.push(text);
        }
    }
    assert!(
        !chunks.is_empty(),
        "AC-3: SDLC family documents (control.source|cicd|release|supply-chain.*) must exist under catalog/canonical/v1"
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

fn fixture_root() -> PathBuf {
    manifest_dir().join("fixtures/assurance/canonical/v1/sdlc")
}

fn fixture_dir(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn collected(hours_ago: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap() - chrono::Duration::hours(hours_ago)
}

fn seal_named(
    collector_id: &str,
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
            collector_id: collector_id.into(),
            collected_at: at,
            scope: "target".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn seal(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, &str)],
    at: DateTime<Utc>,
) -> EvidenceEnvelope {
    seal_named("fixture.sdlc-target", evidence_type, asset, facts, at)
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

fn inventory_repo(set: &mut EvidenceSet, id: &str, archived: bool) {
    set.insert(seal(
        "inventory.subject",
        id,
        &[
            ("id", id),
            ("kind", "repository"),
            ("archived", if archived { "true" } else { "false" }),
        ],
        collected(1),
    ));
    set.insert(seal(
        "evidence.repository.inventory",
        id,
        &[
            ("subject_id", id),
            ("in_scope", "true"),
            ("archived", if archived { "true" } else { "false" }),
            ("criticality", "production"),
        ],
        collected(1),
    ));
}

fn inventory_complete(set: &mut EvidenceSet) {
    set.insert(seal(
        "inventory.complete",
        "org:acme",
        &[("kind", "repository"), ("authoritative", "true")],
        collected(1),
    ));
}

fn branch_protection(set: &mut EvidenceSet, id: &str, protected: bool, hours_ago: i64) {
    set.insert(seal(
        "evidence.repository.branch-protection",
        id,
        &[
            ("subject_id", id),
            ("protected", if protected { "true" } else { "false" }),
            ("force_push_allowed", "false"),
            ("deletion_allowed", "false"),
            ("admin_bypass_allowed", "false"),
        ],
        collected(hours_ago),
    ));
}

fn review_policy(set: &mut EvidenceSet, id: &str, required: bool) {
    set.insert(seal(
        "evidence.repository.review-policy",
        id,
        &[
            ("subject_id", id),
            (
                "reviews_required",
                if required { "true" } else { "false" },
            ),
            ("required_reviewer_count", "2"),
            (
                "meets_review_threshold",
                if required { "true" } else { "false" },
            ),
        ],
        collected(1),
    ));
}

fn security_scan(set: &mut EvidenceSet, id: &str, enabled: bool, hours_ago: i64) {
    set.insert(seal(
        "evidence.repository.security-scanning",
        id,
        &[
            ("subject_id", id),
            (
                "secret_scanning_enabled",
                if enabled { "true" } else { "false" },
            ),
            (
                "code_scanning_enabled",
                if enabled { "true" } else { "false" },
            ),
            ("applicable", "true"),
        ],
        collected(hours_ago),
    ));
}

fn dependency_scan(set: &mut EvidenceSet, id: &str, enabled: bool, scanned_at: &str, hours_ago: i64) {
    set.insert(seal(
        "evidence.repository.dependency-scanning",
        id,
        &[
            ("subject_id", id),
            (
                "dependency_scanning_enabled",
                if enabled { "true" } else { "false" },
            ),
            ("scanned_at", scanned_at),
            ("updates_monitored", if enabled { "true" } else { "false" }),
            ("critical", "true"),
        ],
        collected(hours_ago),
    ));
}

fn workflow_permissions(set: &mut EvidenceSet, id: &str, minimized: bool) {
    set.insert(seal(
        "evidence.cicd.workflow-permissions",
        id,
        &[
            ("subject_id", id),
            (
                "permissions_minimized",
                if minimized { "true" } else { "false" },
            ),
            ("default_write", if minimized { "false" } else { "true" }),
        ],
        collected(1),
    ));
}

fn env_protection(set: &mut EvidenceSet, id: &str, required: bool) {
    set.insert(seal(
        "evidence.deployment.environment-protection",
        id,
        &[
            ("subject_id", id),
            ("production", "true"),
            (
                "authorization_required",
                if required { "true" } else { "false" },
            ),
            ("protected", if required { "true" } else { "false" }),
        ],
        collected(1),
    ));
}

fn artifact_integrity(set: &mut EvidenceSet, id: &str, present: bool) {
    set.insert(seal(
        "evidence.supply-chain.artifact-integrity",
        id,
        &[
            ("subject_id", id),
            (
                "integrity_evidence_present",
                if present { "true" } else { "false" },
            ),
        ],
        collected(1),
    ));
}

fn healthy_population() -> EvidenceSet {
    let mut set = EvidenceSet::new();
    inventory_complete(&mut set);
    for id in ["repo:app", "repo:lib", "repo:api"] {
        inventory_repo(&mut set, id, false);
        branch_protection(&mut set, id, true, 1);
        review_policy(&mut set, id, true);
        security_scan(&mut set, id, true, 1);
        dependency_scan(&mut set, id, true, "2026-08-19T11:00:00Z", 1);
        workflow_permissions(&mut set, id, true);
        env_protection(&mut set, id, true);
        artifact_integrity(&mut set, id, true);
    }
    set
}

fn control_record_window<'a>(text: &'a str, control_id: &str) -> &'a str {
    let marker = format!("id = \"{control_id}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("catalog missing control record {control_id}"));
    &text[start..start + 900.min(text.len() - start)]
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

fn bind_repo_exception(
    status: ExceptionStatus,
    repo: &str,
    expires: Option<DateTime<Utc>>,
) -> Exception {
    let mut ex = Exception::new(
        ExceptionId::new("exc:sdlc-legacy-repo"),
        "timeboxed unprotected legacy repository",
    );
    ex.status = status;
    ex.control_id = Some(ControlId::new("control.source.default-branch-protection"));
    ex.expires_at = expires;
    let mut ids = BTreeSet::new();
    ids.insert(repo.to_string());
    ex.subjects
        .push(weeping_angel_assurance_ir::SubjectSelector {
            kind: SubjectKind::Repository,
            ids,
            tags: BTreeMap::new(),
            scope: SelectorScope::AnyOf,
        });
    ex
}

fn sibling_suite_paths() -> [(&'static str, &'static str); 6] {
    [
        (
            "sdd_iso27001_assurance_target",
            "tests/sdd/iso27001_assurance.target.rs",
        ),
        ("sdd_iam_catalog_target", "tests/sdd/iam_catalog.target.rs"),
        (
            "sdd_canonical_assurance_catalog_target",
            "tests/sdd/canonical_assurance_catalog.target.rs",
        ),
        (
            "sdd_iso27001_assurance_baseline",
            "tests/sdd/iso27001_assurance.baseline.rs",
        ),
        (
            "sdd_iam_catalog_baseline",
            "tests/sdd/iam_catalog.baseline.rs",
        ),
        (
            "sdd_canonical_assurance_catalog_baseline",
            "tests/sdd/canonical_assurance_catalog.baseline.rs",
        ),
    ]
}

// ── AC-1 ───────────────────────────────────────────────────────────────────

#[test]
fn ac1_dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_sdlc_catalog_baseline")
            && toml.contains("tests/sdd/sdlc_catalog.baseline.rs")
            && toml.contains("sdd_sdlc_catalog_target")
            && toml.contains("tests/sdd/sdlc_catalog.target.rs"),
        "AC-1: dual-suite sdd_sdlc_catalog_baseline + sdd_sdlc_catalog_target must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/sdd/sdlc_catalog.baseline.rs")
            .is_file(),
        "AC-1: tests/sdd/sdlc_catalog.baseline.rs must exist (harness/implement registers it)"
    );
    assert!(
        manifest_dir()
            .join("tests/sdd/sdlc_catalog.target.rs")
            .is_file(),
        "AC-1: tests/sdd/sdlc_catalog.target.rs must exist"
    );
}

// ── AC-2 ───────────────────────────────────────────────────────────────────

#[test]
fn ac2_workspace_gates_keep_sibling_suites() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    for (suite, path) in sibling_suite_paths() {
        assert!(
            toml.contains(suite) && toml.contains(path),
            "AC-2: Cargo.toml must keep `{suite}` at `{path}` so cargo test --workspace --features demo stays green"
        );
        assert!(
            manifest_dir().join(path).is_file(),
            "AC-2: sibling suite file `{path}` must remain"
        );
    }
    for suite in ["sdd_sdlc_catalog_baseline", "sdd_sdlc_catalog_target"] {
        assert!(
            toml.contains(suite),
            "AC-2: after implement, `{suite}` is registered so workspace verification can execute it"
        );
    }
}

// ── AC-3 ───────────────────────────────────────────────────────────────────

#[test]
fn ac3_twenty_six_sdlc_controls_with_domains_evidence_tests() {
    assert_eq!(
        CANONICAL_SDLC_CONTROLS.len(),
        26,
        "pinned independently assessable family size is 26"
    );
    let catalog = load_catalog();
    let text = sdlc_catalog_text();
    let ids = quoted_ids(&text, "control.");
    for id in CANONICAL_SDLC_CONTROLS {
        assert!(
            ids.contains(*id) || catalog.control(id).is_ok(),
            "AC-3: missing control `{id}`"
        );
        let control = catalog
            .control(id)
            .unwrap_or_else(|e| panic!("AC-3: CanonicalCatalog must expose `{id}`: {e}"));
        assert!(
            !control.domains.is_empty() || control_record_window(&text, id).contains("domain"),
            "AC-3: {id} must declare domain(s)"
        );
        assert!(
            !control.evidence.is_empty(),
            "AC-3: {id} must declare evidence requirements"
        );
        assert!(
            !control.tests.is_empty(),
            "AC-3: {id} must declare test refs"
        );
        let class = control.automation.to_ascii_lowercase();
        assert!(
            matches!(class.as_str(), "automated" | "hybrid" | "manual"),
            "AC-3: {id} automation class must be Automated|Hybrid|Manual, got {class}"
        );
        assert_eq!(
            *id,
            id.to_ascii_lowercase(),
            "AC-3: ids are lowercase ({id})"
        );
        assert!(
            !id.contains('_'),
            "AC-3: catalog ids use hyphen segments ({id})"
        );
        assert!(
            (!control.title.is_empty() && !control.objective.is_empty())
                || control_record_window(&text, id).contains("objective")
                || control_record_window(&text, id).contains("description"),
            "AC-3: {id} must declare title/objective"
        );
    }

    let assessable: Vec<_> = catalog
        .controls()
        .keys()
        .filter(|id| {
            SDLC_CONTROL_PREFIXES.iter().any(|p| id.starts_with(p)) && *id != PINNED_FIXTURE_CONTROL
        })
        .collect();
    assert!(
        (20..=30).contains(&assessable.len()),
        "AC-3: expected 20–30 independently assessable SDLC controls, found {} ({assessable:?})",
        assessable.len()
    );
    assert!(
        catalog.control(PINNED_FIXTURE_CONTROL).is_ok(),
        "AC-3: control.source.protected-branch remains as the exists-only fixture"
    );
}

// ── AC-4 ───────────────────────────────────────────────────────────────────

#[test]
fn ac4_evidence_types_are_facts_not_conclusions() {
    let catalog = load_catalog();
    let text = sdlc_catalog_text();
    let ids = quoted_ids(&text, "evidence.");
    for id in CANONICAL_SDLC_EVIDENCE {
        assert!(
            ids.contains(*id) || catalog.evidence().contains_key(*id),
            "AC-4: missing evidence contract `{id}`"
        );
        assert!(
            id.starts_with("evidence.repository.")
                || id.starts_with("evidence.cicd.")
                || id.starts_with("evidence.deployment.")
                || id.starts_with("evidence.release.")
                || id.starts_with("evidence.supply-chain."),
            "AC-4: evidence ids are evidence.repository|cicd|deployment|release|supply-chain.*, not `{id}`"
        );
        assert!(
            !id.contains("github") && !id.starts_with("evidence.github."),
            "AC-4: `{id}` must not be evidence.github.*"
        );
    }
    let lower = text.to_ascii_lowercase();
    for phrase in [
        "compliant",
        "control passed",
        "branch protection effective",
        "visibility control passed",
        "review control passed",
        "release authorized conclusion",
        "security review effective",
        "iso 27001",
    ] {
        assert!(
            !lower.contains(phrase),
            "AC-4: evidence contracts are facts, not conclusions (`{phrase}`)"
        );
    }
    for ev in catalog.evidence().values() {
        if !is_sdlc_family_id(&ev.id) {
            continue;
        }
        let used = catalog
            .controls()
            .values()
            .any(|c| c.evidence.iter().any(|e| e == &ev.id));
        assert!(used, "AC-4: evidence `{}` must not be orphaned", ev.id);
    }
}

// ── AC-5 ───────────────────────────────────────────────────────────────────

#[test]
fn ac5_nine_prompt05_tests_evaluate_populations() {
    let catalog = load_catalog();
    let text = sdlc_catalog_text();
    let ids = quoted_ids(&text, "test.");
    assert_eq!(PROMPT05_EXAMPLE_TESTS.len(), 9);
    for id in PROMPT05_EXAMPLE_TESTS {
        assert!(
            ids.contains(*id) || catalog.tests().contains_key(*id),
            "AC-5: missing Prompt-05 example test `{id}`"
        );
        let window = test_expression_window(&text, id);
        assert!(
            !expression_is_existence_only(&window),
            "AC-5: {id} must evaluate a population, not Exists(one envelope)"
        );
        assert!(
            POPULATION_OPS.iter().any(|op| window.contains(op)
                || window
                    .to_ascii_lowercase()
                    .contains(&op.to_ascii_lowercase())),
            "AC-5: {id} must declare a population operator"
        );
    }
    for id in CANONICAL_SDLC_TESTS {
        assert!(
            ids.contains(*id) || catalog.tests().contains_key(*id),
            "AC-5: missing SDLC test `{id}` so its control is not untested"
        );
    }

    let mut lone = EvidenceSet::new();
    branch_protection(&mut lone, "repo:random", true, 1);
    let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.repository.branch-protection",
    )));
    let (exists_ok, _) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        exists,
        &lone,
    );
    assert_eq!(
        exists_ok.effectiveness,
        Effectiveness::Effective,
        "sanity: a lone protection envelope satisfies Exists"
    );

    let (pop, json) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &lone,
    );
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "AC-5: a single protection envelope must not pass all in-scope default branches protected; json={json}"
    );
}

// ── AC-6 ───────────────────────────────────────────────────────────────────

#[test]
fn ac6_seven_fixtures_distinguish_evaluator_outcomes() {
    let root = fixture_root();
    assert!(
        root.is_dir(),
        "AC-6: fixtures/assurance/canonical/v1/sdlc must exist"
    );
    assert_eq!(SDLC_FIXTURES.len(), 7, "Prompt 05 ships seven fixtures");
    for name in SDLC_FIXTURES {
        let dir = fixture_dir(name);
        assert!(
            dir.is_dir(),
            "AC-6: fixture `{name}` is not shipped at {}",
            dir.display()
        );
        let mut files = Vec::new();
        walk_files(&dir, "json", &mut files);
        walk_files(&dir, "toml", &mut files);
        walk_files(&dir, "jsonl", &mut files);
        assert!(
            !files.is_empty(),
            "AC-6: fixture `{name}` must contain evidence documents"
        );
        let blob: String = files
            .iter()
            .map(|p| fs::read_to_string(p).unwrap())
            .collect();
        assert!(
            blob.contains("evidence.repository.")
                || blob.contains("evidence.cicd.")
                || blob.contains("evidence.deployment.")
                || blob.contains("evidence.release.")
                || blob.contains("evidence.supply-chain.")
                || blob.contains("exception")
                || blob.contains("Exception"),
            "AC-6: fixture `{name}` must emit canonical SDLC facts (not source.branch.protection)"
        );
        assert!(
            !blob.contains("source.branch.protection") && !blob.contains("evidence.github."),
            "AC-6: fixture `{name}` must not use GitHub-shaped evidence types"
        );
    }

    let healthy = healthy_population();
    let (ok, _) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &healthy,
    );
    assert_eq!(
        ok.effectiveness,
        Effectiveness::Effective,
        "AC-6 healthy-org: all default branches protected"
    );

    let mut fail = healthy_population();
    branch_protection(&mut fail, "repo:app", false, 1);
    let (failed, fail_json) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &fail,
    );
    assert_eq!(
        failed.effectiveness,
        Effectiveness::Ineffective,
        "AC-6 unprotected-default-branch / degraded-org: fail, got {:?}",
        failed.effectiveness
    );
    let failing = string_list(&fail_json, "failingSubjects");
    assert!(
        failing.iter().any(|s| s.contains("repo:app")),
        "AC-6: failing subject must name the unprotected repository; got {failing:?}"
    );

    let mut missing = EvidenceSet::new();
    inventory_complete(&mut missing);
    for id in ["repo:app", "repo:lib"] {
        inventory_repo(&mut missing, id, false);
        branch_protection(&mut missing, id, true, 1);
    }
    security_scan(&mut missing, "repo:app", true, 1);
    let (miss, miss_json) = result_json(
        "test.source.secret-scanning-enabled",
        "control.source.secret-scanning",
        coverage_100(
            "repository",
            "evidence.repository.security-scanning",
            "secret_scanning_enabled",
        ),
        &missing,
    );
    assert_eq!(
        miss.effectiveness,
        Effectiveness::InsufficientEvidence,
        "AC-6: missing scan evidence is not a technical failure, got {:?} {miss_json}",
        miss.effectiveness
    );
    assert_ne!(miss.effectiveness, Effectiveness::Ineffective);

    let mut stale = EvidenceSet::new();
    inventory_complete(&mut stale);
    inventory_repo(&mut stale, "repo:app", false);
    inventory_repo(&mut stale, "repo:lib", false);
    dependency_scan(&mut stale, "repo:app", true, "2026-07-01T00:00:00Z", 48);
    dependency_scan(&mut stale, "repo:lib", true, "2026-07-01T00:00:00Z", 48);
    let (stale_r, _) = result_json(
        "test.source.dependency-scanning-current",
        "control.source.dependency-scanning",
        coverage_100(
            "repository",
            "evidence.repository.dependency-scanning",
            "scanned_at",
        ),
        &stale,
    );
    assert_eq!(
        stale_r.effectiveness,
        Effectiveness::StaleEvidence,
        "AC-6 stale-dependency-scan: StaleEvidence, not Ineffective-as-missing"
    );

    let (manual, _) = result_json(
        "test.source.security-review-recorded",
        "control.source.security-review",
        TestExpr::ManualReview,
        &healthy_population(),
    );
    assert_eq!(
        manual.effectiveness,
        Effectiveness::ManualReviewRequired,
        "AC-6: Hybrid/Manual security-review without attestation is ManualReviewRequired"
    );

    let mut partial = EvidenceSet::new();
    inventory_repo(&mut partial, "repo:app", false);
    branch_protection(&mut partial, "repo:app", true, 1);
    let (part, part_json) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &partial,
    );
    assert_ne!(
        part.effectiveness,
        Effectiveness::Effective,
        "AC-6 partial-coverage must not yield Effective; json={part_json}"
    );
    assert!(
        matches!(
            part.effectiveness,
            Effectiveness::InsufficientEvidence | Effectiveness::Inconclusive
        ),
        "AC-6: partial/unknown population → InsufficientEvidence or Inconclusive, got {:?}",
        part.effectiveness
    );

    let mut excepted = EvidenceSet::new();
    inventory_complete(&mut excepted);
    inventory_repo(&mut excepted, "repo:legacy", false);
    branch_protection(&mut excepted, "repo:legacy", false, 1);
    excepted.insert_exception(bind_repo_exception(
        ExceptionStatus::Approved,
        "repo:legacy",
        Some(Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap()),
    ));
    let (exc, exc_json) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &excepted,
    );
    let excepted_subjects = string_list(&exc_json, "exceptedSubjects");
    assert!(
        excepted_subjects
            .iter()
            .any(|s| s.contains("repo:legacy"))
            || exc.effectiveness == Effectiveness::ExceptionApproved,
        "AC-6 approved-exception: bound subject is excepted or ExceptionApproved; got {:?} {exc_json}",
        exc.effectiveness
    );
    assert_ne!(
        exc.effectiveness,
        Effectiveness::Ineffective,
        "AC-6: approved exception must not treat the bound subject as a technical fail"
    );
    assert_ne!(
        exc.effectiveness,
        Effectiveness::Effective,
        "AC-6: approved exception is not silent Effective"
    );
}

// ── AC-7 ───────────────────────────────────────────────────────────────────

#[test]
fn ac7_release_and_policy_controls_are_hybrid_or_manual() {
    let catalog = load_catalog();
    let text = sdlc_catalog_text();
    for id in HYBRID_OR_MANUAL_CONTROLS {
        let class = catalog
            .control(id)
            .map(|c| c.automation.to_ascii_lowercase())
            .unwrap_or_else(|_| control_record_automation(&text, id));
        assert!(
            class == "hybrid" || class == "manual",
            "AC-7: {id} must be Hybrid or Manual, got {class}"
        );
        let control = catalog
            .control(id)
            .expect("AC-7: hybrid/manual control present");
        for test_id in &control.tests {
            let window = test_expression_window(&text, test_id);
            assert!(
                !expression_is_existence_only(&window)
                    || window.to_ascii_lowercase().contains("manual"),
                "AC-7: {test_id} must not auto-pass as Exists(one technical envelope)"
            );
        }
    }

    let mut tech_only = healthy_population();
    tech_only.insert(seal(
        "evidence.release.authorization",
        "repo:app",
        &[
            ("subject_id", "repo:app"),
            ("authorized", "true"),
            ("dev_release_separated", "true"),
        ],
        collected(1),
    ));
    let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.release.authorization",
    )));
    let (via_exists, _) = result_json(
        "test.release.authorization-recorded",
        "control.release.authorization",
        exists,
        &tech_only,
    );
    assert_eq!(
        via_exists.effectiveness,
        Effectiveness::Effective,
        "sanity: Exists(release.authorization) would auto-pass"
    );
    let (honest, _) = result_json(
        "test.release.authorization-recorded",
        "control.release.authorization",
        TestExpr::ManualReview,
        &tech_only,
    );
    assert_eq!(
        honest.effectiveness,
        Effectiveness::ManualReviewRequired,
        "AC-7: release authorization cannot auto-pass without attestation"
    );
}

// ── AC-8 / AC-16 ───────────────────────────────────────────────────────────

#[test]
fn ac8_validator_accepts_sdlc_slice_without_provider_or_framework_tokens() {
    let catalog = load_catalog();
    catalog.validate().unwrap_or_else(|e| {
        panic!("AC-8/AC-16: CanonicalCatalog::validate must accept the SDLC slice: {e}")
    });
    assert_eq!(
        CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1",
        "AC-16: consume the Prompt 01 schema constant"
    );

    let mut seen = BTreeSet::new();
    for id in catalog
        .controls()
        .keys()
        .chain(catalog.evidence().keys())
        .chain(catalog.tests().keys())
    {
        if !is_sdlc_family_id(id) {
            continue;
        }
        assert!(seen.insert(id.clone()), "AC-8: duplicate id `{id}`");
        for token in FORBIDDEN_PROVIDER_TOKENS {
            let segment = format!(".{token}.");
            let suffix = format!(".{token}");
            assert!(
                !id.contains(&segment)
                    && !id.ends_with(&suffix)
                    && !id.contains(&format!(".{token}-")),
                "AC-8: provider token `{token}` leaked into id `{id}`"
            );
        }
        for token in FORBIDDEN_FRAMEWORK_TOKENS {
            assert!(
                !id.contains(token),
                "AC-8: framework token `{token}` leaked into id `{id}`"
            );
        }
        for token in FORBIDDEN_NATIVE_ID_TOKENS {
            assert!(
                !id.to_ascii_lowercase().contains(token),
                "AC-8: GitHub-native object name `{token}` leaked into id `{id}`"
            );
        }
    }

    let text = sdlc_catalog_text();
    let lower = text.to_ascii_lowercase();
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            !lower.contains(token),
            "AC-8: canonical SDLC content must not mention `{token}`"
        );
    }
}

#[test]
fn ac16_manifest_lists_sdlc_files_and_digest_is_deterministic() {
    let manifest = fs::read_to_string(catalog_v1_dir().join("manifest.toml")).unwrap();
    let listed = manifest.to_ascii_lowercase();
    let has_sdlc = ["source", "cicd", "release", "supply-chain", "sdlc"]
        .iter()
        .any(|needle| {
            listed.contains(&format!("controls/{needle}"))
                || listed.contains(&format!("evidence/{needle}"))
                || listed.contains(&format!("tests/{needle}"))
                || listed.contains(&format!("{needle}.toml"))
        });
    assert!(
        has_sdlc,
        "AC-16: catalog/canonical/v1/manifest.toml [files] must list SDLC control/evidence/test documents; got:\n{manifest}"
    );
    assert!(
        listed.contains("fixture.example") && listed.contains("identity"),
        "AC-16: manifest must keep fixture.example + identity listings"
    );
    let catalog = load_catalog();
    catalog
        .validate()
        .expect("AC-16: validate after listing SDLC files");
    let digest = catalog.digest().expect("AC-16: digest remains available");
    let again = load_catalog()
        .digest()
        .expect("AC-2/AC-16: digest is deterministic");
    assert_eq!(
        digest.to_string(),
        again.to_string(),
        "AC-16: CanonicalCatalog::digest must stay deterministic after adding SDLC files"
    );
    assert!(
        digest.to_string().starts_with("wa:canonical-catalog:"),
        "AC-16: consume CanonicalCatalog::digest, do not invent a second digest"
    );
}

// ── AC-9 ───────────────────────────────────────────────────────────────────

#[test]
fn ac9_iso_pack_source_sliver_and_mappings_unchanged() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_SOURCE_CONTROLS {
        assert!(
            control_ids.contains(id),
            "AC-9: ISO sliver `{id}` must remain (have {control_ids:?})"
        );
    }
    assert!(
        !control_ids
            .iter()
            .any(|id| id.starts_with("control.source.")
                || id.starts_with("control.cicd.")
                || id.starts_with("control.release.")
                || id.starts_with("control.supply-chain.")),
        "AC-9: do not move SDLC controls into the ISO pack"
    );

    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_SOURCE_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "AC-9: mapping {from} → {to} must stay in the ISO pack"
        );
    }
    assert!(
        !mappings.contains("control.source.default-branch-protection")
            && !mappings.contains("control.cicd.")
            && !mappings.contains("control.supply-chain."),
        "AC-9: ISO mappings must not retarget the new SDLC family"
    );
}

// ── AC-10 ──────────────────────────────────────────────────────────────────

#[test]
fn ac10_github_collector_and_scanner_stay_untouched() {
    assert_eq!(
        GITHUB_EVIDENCE_TYPES, PINNED_GITHUB_EVIDENCE_TYPES,
        "AC-10: do not expand GITHUB_EVIDENCE_TYPES in this slice"
    );
    assert!(
        !GITHUB_EVIDENCE_TYPES
            .iter()
            .any(|t| t.starts_with("evidence.repository.")
                || t.starts_with("evidence.cicd.")
                || t.starts_with("evidence.supply-chain.")),
        "AC-10: GitHub collector must keep emitting source.* only"
    );
    let collector_src = crate_src("weeping-angel-collector");
    for name in ["gitlab", "bitbucket", "azure_devops"] {
        assert!(
            !collector_src.join(name).exists(),
            "AC-10: do not add a {name} collector in this slice"
        );
    }

    let text = sdlc_catalog_text();
    assert!(
        !text.contains("GITHUB_EVIDENCE_TYPES")
            && !text.contains("source.security.secret_scanning.enabled")
            && !text.contains("depcheck")
            && !text.contains("engines::")
            && !text.contains("src/engines"),
        "AC-10: SDLC catalog tests must not couple to GITHUB_EVIDENCE_TYPES or scanner internals"
    );
}

// ── AC-11 ──────────────────────────────────────────────────────────────────

#[test]
fn ac11_no_second_loader_value_enum_or_repo_inventory_resolver() {
    let rust = product_rs_joined();
    assert!(
        rust.contains("struct CanonicalCatalog")
            && rust.contains("fn load")
            && rust.contains("fn validate"),
        "AC-11: consume CanonicalCatalog::{{load,validate,digest}}"
    );
    let catalog_src = crate_sources_joined("weeping-angel-canonical-catalog");
    let extra_loaders = catalog_src.matches("struct CanonicalCatalog").count();
    assert_eq!(extra_loaders, 1, "AC-11: no second CanonicalCatalog loader");
    assert!(
        !rust.contains("fn resolve_repository_inventory") && !rust.contains("struct SdlcPopulation"),
        "AC-11: do not add resolve_repository_inventory / SdlcPopulation; use inventory.subject + inventory.complete"
    );
    let control_src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        control_src.contains("pub use weeping_angel_evidence::EvidenceValue")
            || control_src.contains("weeping_angel_evidence::EvidenceValue"),
        "AC-11: consume the landed EvidenceValue"
    );
    assert!(
        !control_src.contains("enum EvidenceValue"),
        "AC-11: do not define a second EvidenceValue enum in control-test"
    );
    assert!(
        control_src.contains("inventory.subject") && control_src.contains("inventory.complete"),
        "AC-11: population runtime already resolves inventory.subject + inventory.complete"
    );
}

// ── AC-12 ──────────────────────────────────────────────────────────────────

#[test]
fn ac12_approved_exception_uses_ir_expired_revoked_do_not_pass() {
    let fixture = fixture_dir("approved-exception");
    assert!(
        fixture.is_dir(),
        "AC-12: approved-exception fixture must exist"
    );
    let mut files = Vec::new();
    walk_files(&fixture, "json", &mut files);
    walk_files(&fixture, "toml", &mut files);
    let blob: String = files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    assert!(
        blob.contains("exc:")
            || blob.contains("exception")
            || blob.contains("Exception")
            || blob.contains("approved"),
        "AC-12: fixture must serialize existing Exception IR"
    );

    let mut approved = EvidenceSet::new();
    inventory_complete(&mut approved);
    inventory_repo(&mut approved, "repo:legacy", false);
    branch_protection(&mut approved, "repo:legacy", false, 1);
    approved.insert_exception(bind_repo_exception(
        ExceptionStatus::Approved,
        "repo:legacy",
        Some(Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap()),
    ));
    let (ok, json) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        coverage_100(
            "repository",
            "evidence.repository.branch-protection",
            "protected",
        ),
        &approved,
    );
    let excepted = string_list(&json, "exceptedSubjects");
    assert!(
        ok.effectiveness == Effectiveness::ExceptionApproved
            || excepted.iter().any(|s| s.contains("repo:legacy")),
        "AC-12: approved unexpired Exception IR → ExceptionApproved or excepted partition, got {:?} {json}",
        ok.effectiveness
    );
    assert_ne!(ok.effectiveness, Effectiveness::Effective);
    assert_ne!(ok.effectiveness, Effectiveness::Ineffective);

    for status in [ExceptionStatus::Expired, ExceptionStatus::Revoked] {
        let mut bad = EvidenceSet::new();
        inventory_complete(&mut bad);
        inventory_repo(&mut bad, "repo:legacy", false);
        branch_protection(&mut bad, "repo:legacy", false, 1);
        let expires = if status == ExceptionStatus::Expired {
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
        } else {
            Some(Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap())
        };
        bad.insert_exception(bind_repo_exception(status, "repo:legacy", expires));
        let (result, _) = result_json(
            "test.source.default-branches-protected",
            "control.source.default-branch-protection",
            coverage_100(
                "repository",
                "evidence.repository.branch-protection",
                "protected",
            ),
            &bad,
        );
        assert_ne!(
            result.effectiveness,
            Effectiveness::Effective,
            "AC-12: {status:?} exception must not pass"
        );
        assert_ne!(
            result.effectiveness,
            Effectiveness::ExceptionApproved,
            "AC-12: {status:?} exception must not be ExceptionApproved"
        );
        assert_eq!(
            result.effectiveness,
            Effectiveness::Ineffective,
            "AC-12: expired/revoked leave the unprotected repo as Ineffective, got {:?}",
            result.effectiveness
        );
    }
}

// ── AC-13 ──────────────────────────────────────────────────────────────────

#[test]
fn ac13_iam_family_and_cat_fixture_ids_remain() {
    let catalog = load_catalog();
    for id in IAM_FAMILY_PINS {
        let present = catalog.controls().contains_key(*id)
            || catalog.evidence().contains_key(*id)
            || catalog.tests().contains_key(*id);
        assert!(
            present,
            "AC-13: IAM pin `{id}` must remain in CanonicalCatalog"
        );
    }
    for id in [
        PINNED_FIXTURE_CONTROL,
        PINNED_FIXTURE_EVIDENCE,
        PINNED_FIXTURE_TEST,
    ] {
        let present = catalog.controls().contains_key(id)
            || catalog.evidence().contains_key(id)
            || catalog.tests().contains_key(id);
        assert!(present, "AC-13: CAT-015 fixture pin `{id}` must remain");
    }
    let fixture_test = catalog
        .tests()
        .get(PINNED_FIXTURE_TEST)
        .expect("AC-13: test.source.protected-branch remains");
    assert!(
        fixture_test
            .expression
            .get("op")
            .is_some_and(|v| v.as_str() == Some("exists"))
            || format!("{:?}", fixture_test.expression).contains("exists"),
        "AC-13: CAT fixture test stays exists-only; got {:?}",
        fixture_test.expression
    );
}

// ── AC-14 ──────────────────────────────────────────────────────────────────

#[test]
fn ac14_prompt01_and_iam_ssot_are_not_overwritten() {
    let prompt01 =
        fs::read_to_string(manifest_dir().join("docs/sdd/canonical-assurance-catalog-v1.md"))
            .expect("AC-14: Prompt 01 SSOT must remain");
    assert!(
        prompt01.contains("weeping-angel/canonical-catalog/v1")
            && prompt01.contains("control.source.protected-branch")
            && prompt01.contains("CAT-015"),
        "AC-14: Prompt 01 SSOT still owns catalog infrastructure + CAT fixture"
    );
    assert!(
        !prompt01.contains("sdd_sdlc_catalog_target")
            && !prompt01.contains("control.source.default-branch-protection"),
        "AC-14: do not overwrite Prompt 01 SSOT with this slice's suite ids or SDLC family"
    );

    let iam =
        fs::read_to_string(manifest_dir().join("docs/sdd/iam-canonical-assurance-catalog.md"))
            .expect("AC-14: Prompt 04 IAM SSOT must remain");
    assert!(
        iam.contains("control.identity.mfa"),
        "AC-14: IAM SSOT still owns the identity family"
    );
    assert!(
        !iam.contains("sdd_sdlc_catalog_target")
            && !iam.contains("control.source.default-branch-protection"),
        "AC-14: do not overwrite Prompt 04 IAM SSOT with SDLC content"
    );
}

// ── AC-15 ──────────────────────────────────────────────────────────────────

#[test]
fn ac15_gitlab_or_bitbucket_collector_can_populate_same_contracts() {
    let mut gitlab = EvidenceSet::new();
    gitlab.insert(seal_named(
        "collector.gitlab",
        "inventory.complete",
        "org:acme",
        &[("kind", "repository"), ("authoritative", "true")],
        collected(1),
    ));
    gitlab.insert(seal_named(
        "collector.gitlab",
        "inventory.subject",
        "repo:app",
        &[
            ("id", "repo:app"),
            ("kind", "repository"),
            ("archived", "false"),
        ],
        collected(1),
    ));
    gitlab.insert(seal_named(
        "collector.gitlab",
        "evidence.repository.branch-protection",
        "repo:app",
        &[("subject_id", "repo:app"), ("protected", "true")],
        collected(1),
    ));

    let mut bitbucket = EvidenceSet::new();
    bitbucket.insert(seal_named(
        "collector.bitbucket",
        "inventory.complete",
        "org:acme",
        &[("kind", "repository"), ("authoritative", "true")],
        collected(1),
    ));
    bitbucket.insert(seal_named(
        "collector.bitbucket",
        "inventory.subject",
        "repo:app",
        &[
            ("id", "repo:app"),
            ("kind", "repository"),
            ("archived", "false"),
        ],
        collected(1),
    ));
    bitbucket.insert(seal_named(
        "collector.bitbucket",
        "evidence.repository.branch-protection",
        "repo:app",
        &[("subject_id", "repo:app"), ("protected", "true")],
        collected(1),
    ));

    let expr = coverage_100(
        "repository",
        "evidence.repository.branch-protection",
        "protected",
    );
    let (g, _) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        expr.clone(),
        &gitlab,
    );
    let (b, _) = result_json(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        expr,
        &bitbucket,
    );
    assert_eq!(
        g.effectiveness, b.effectiveness,
        "AC-15: GitLab and Bitbucket collectors must receive the same control result on the same facts"
    );
    assert_eq!(
        g.effectiveness,
        Effectiveness::Effective,
        "AC-15: provider-neutral contracts evaluate without catalog changes; got {:?}",
        g.effectiveness
    );

    let text = sdlc_catalog_text();
    for id in quoted_ids(&text, "control.")
        .into_iter()
        .chain(quoted_ids(&text, "evidence."))
        .chain(quoted_ids(&text, "test."))
    {
        if !is_sdlc_family_id(&id) {
            continue;
        }
        for token in FORBIDDEN_PROVIDER_TOKENS {
            assert!(
                !id.contains(token),
                "AC-15: catalog id `{id}` must stay provider-neutral so a {token} collector needs no catalog change"
            );
        }
    }
}
