//! Target suite for the SDLC Canonical Assurance Catalog (SDLC catalog).
//!
//! Encodes DESIRED behavior in `docs/specs/sdlc-canonical-assurance-catalog.md`
//! §4 / §5 (SDLC-001…016). Must stay RED on the current tree: no SDLC
//! family documents, no seven multi-repo fixtures. Do not implement
//! catalog content here.
//!
//! Consumes CanonicalCatalog::{load,validate,digest}, EvidenceValue, and
//! population runtime AllSubjects / CoverageAtLeast / ExceptionApproved /
//! InsufficientEvidence. Does not fork a second loader or a
//! repository-inventory resolver.
//!
//! Assert loaded catalog IDs and fixture evaluation. Do not scan this
//! suite's source text for a substring that the assertion itself quotes.

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
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::load_framework_pack;

const PINNED_FIXTURE_CONTROL: &str = "control.source.protected-branch";
const PINNED_FIXTURE_EVIDENCE: &str = "evidence.source.protected-branch";
const PINNED_FIXTURE_TEST: &str = "test.source.protected-branch";
const POPULATION_DEFAULT_BRANCH: &str = "control.source.default-branch-protection";

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

const CONCLUSION_PHRASES: &[&str] = &[
    "compliant",
    "control passed",
    "branch protection effective",
    "visibility control passed",
    "review control passed",
    "release authorized conclusion",
    "security review effective",
];

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn harness_relpath(kind: &str) -> String {
    format!("tests/contracts/sdlc_catalog.{kind}.rs")
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

fn catalog_v1_dir() -> PathBuf {
    let dir = manifest_dir().join("catalog/canonical/v1");
    assert!(
        dir.is_dir(),
        "SDLC-001: catalog infrastructure catalog tree catalog/canonical/v1 must exist"
    );
    dir
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1_dir()).unwrap_or_else(|e| {
        panic!("SDLC-001: CanonicalCatalog::load/validate must accept the SDLC slice: {e}")
    })
}

/// Product-state gate: every acceptance test requires the SDLC population
/// family to be loaded. Current tree has only the exists-only CAT fixture.
fn require_sdlc_family() -> CanonicalCatalog {
    let catalog = load_catalog();
    catalog
        .control(POPULATION_DEFAULT_BRANCH)
        .unwrap_or_else(|e| {
            panic!(
                "SDLC family missing: `{POPULATION_DEFAULT_BRANCH}` is not loaded ({e}). \
             Current tree still has only `{PINNED_FIXTURE_CONTROL}`."
            )
        });
    catalog
}

fn catalog_toml_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(&catalog_v1_dir(), "toml", &mut files);
    assert!(
        !files.is_empty(),
        "SDLC-001: catalog/canonical/v1 must contain TOML documents"
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

fn line_has(line: &str, needle: &str) -> bool {
    line.find(needle).is_some()
}

fn text_lacks(haystack: &str, needle: &str) -> bool {
    haystack.find(needle).is_none()
}

fn sdlc_catalog_text() -> String {
    let mut chunks = Vec::new();
    for path in catalog_toml_files() {
        let text = fs::read_to_string(&path).unwrap();
        if text.lines().any(|line| {
            CANONICAL_SDLC_CONTROLS.iter().any(|id| line_has(line, id))
                || line_has(line, "evidence.repository.")
                || line_has(line, "evidence.cicd.")
                || line_has(line, "evidence.deployment.")
                || line_has(line, "evidence.release.")
                || line_has(line, "evidence.supply-chain.")
                || line_has(line, "control.cicd.")
                || line_has(line, "control.release.")
                || line_has(line, "control.supply-chain.")
        }) {
            chunks.push(text);
        }
    }
    assert!(
        !chunks.is_empty(),
        "SDLC-003: SDLC family documents (control.source|cicd|release|supply-chain.*) must exist under catalog/canonical/v1"
    );
    chunks.join("\n")
}

fn hyphenated_catalog_id(id: &str) -> bool {
    id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

fn id_has_token(id: &str, token: &str) -> bool {
    id.split('.')
        .any(|seg| seg == token || seg.split('-').any(|part| part == token))
}

fn fixture_root() -> PathBuf {
    manifest_dir().join("fixtures/assurance/canonical/v1/sdlc")
}

fn fixture_dir(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn require_seven_fixtures() {
    assert!(
        fixture_root().is_dir(),
        "SDLC-010: fixtures/assurance/canonical/v1/sdlc must exist"
    );
    for name in SDLC_FIXTURES {
        let dir = fixture_dir(name);
        assert!(
            dir.is_dir(),
            "SDLC-010: fixture `{name}` is not shipped at {}",
            dir.display()
        );
        let evidence = dir.join("evidence.json");
        assert!(
            evidence.is_file(),
            "SDLC-010: fixture `{name}` must ship evidence.json at {}",
            evidence.display()
        );
        let blob = fs::read_to_string(&evidence).unwrap();
        assert!(
            text_has(&blob, "evidence.repository.")
                || text_has(&blob, "evidence.cicd.")
                || text_has(&blob, "evidence.deployment.")
                || text_has(&blob, "evidence.release.")
                || text_has(&blob, "evidence.supply-chain.")
                || text_has(&blob, "exception"),
            "SDLC-010: fixture `{name}` must emit canonical SDLC facts"
        );
        assert!(
            text_lacks(&blob, "source.branch.protection") && text_lacks(&blob, "evidence.github."),
            "SDLC-010: fixture `{name}` must not use GitHub-shaped evidence types"
        );
    }
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

fn all_subjects(kind: &str, evidence_type: &str, field: &str) -> TestExpr {
    TestExpr::AllSubjects {
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
    }
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
    set.insert(seal(
        "inventory.complete",
        "org:acme:deployment",
        &[("kind", "deployment"), ("authoritative", "true")],
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
            (
                "force_push_restricted",
                if protected { "true" } else { "false" },
            ),
            (
                "force_push_allowed",
                if protected { "false" } else { "true" },
            ),
            (
                "deletion_restricted",
                if protected { "true" } else { "false" },
            ),
            ("deletion_allowed", if protected { "false" } else { "true" }),
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
            ("reviews_required", if required { "true" } else { "false" }),
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

fn dependency_scan(
    set: &mut EvidenceSet,
    id: &str,
    enabled: bool,
    scanned_at: &str,
    hours_ago: i64,
) {
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
        "inventory.subject",
        &format!("deploy:{id}"),
        &[("id", id), ("kind", "deployment")],
        collected(1),
    ));
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
    let has_exists = text_has(&lower, "op = \"exists\"") || text_has(&lower, "exists(");
    let has_population = POPULATION_OPS
        .iter()
        .any(|op| text_has(window, op) || text_has(&lower, &op.to_ascii_lowercase()));
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
    ex.control_id = Some(ControlId::new(POPULATION_DEFAULT_BRANCH));
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

fn default_branch_expr() -> TestExpr {
    all_subjects(
        "repository",
        "evidence.repository.branch-protection",
        "protected",
    )
}

fn assert_population_op(text: &str, test_id: &str) {
    let window = test_expression_window(text, test_id);
    assert!(
        !expression_is_existence_only(&window),
        "SDLC-009: {test_id} must evaluate a population, not Exists(one envelope)"
    );
    assert!(
        POPULATION_OPS.iter().any(|op| {
            text_has(&window, op)
                || text_has(&window.to_ascii_lowercase(), &op.to_ascii_lowercase())
        }),
        "SDLC-009: {test_id} must declare a population operator"
    );
}

#[test]
fn sdlc_001_catalog_tree_lists_and_loads_sdlc_files() {
    let catalog = require_sdlc_family();
    let manifest = fs::read_to_string(catalog_v1_dir().join("manifest.toml")).unwrap();
    for listed in [
        "controls/sdlc.toml",
        "evidence/sdlc.toml",
        "tests/sdlc.toml",
        "controls/fixture.example.toml",
        "controls/identity.toml",
    ] {
        assert!(
            text_has(&manifest, listed),
            "SDLC-001: manifest.toml must list `{listed}` (prefer sdlc.toml so ghc_b028 stays green)"
        );
    }
    assert!(
        catalog_v1_dir().join("controls/sdlc.toml").is_file()
            && catalog_v1_dir().join("evidence/sdlc.toml").is_file()
            && catalog_v1_dir().join("tests/sdlc.toml").is_file(),
        "SDLC-001: catalog/canonical/v1/{{controls,evidence,tests}}/sdlc.toml must exist"
    );
    catalog
        .validate()
        .expect("SDLC-001: CanonicalCatalog::validate must accept the SDLC slice");
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    let baseline = harness_relpath("baseline");
    let target = harness_relpath("target");
    assert!(
        !text_has(&cargo, "sdd_sdlc_catalog_baseline")
            && !text_has(&cargo, &baseline)
            && sdd_suite_wired("sdd_sdlc_catalog_target")
            && text_has(&harness_src(), &target),
        "SDLC-001: target suite listed; superseded baseline deleted"
    );
}

#[test]
fn sdlc_002_digest_is_deterministic_after_sdlc_files() {
    let catalog = require_sdlc_family();
    assert_eq!(
        CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1",
        "SDLC-002: consume the catalog infrastructure schema constant"
    );
    let digest = catalog
        .digest()
        .expect("SDLC-002: digest remains available");
    let again = load_catalog()
        .digest()
        .expect("SDLC-002: digest is deterministic");
    assert_eq!(
        digest.to_string(),
        again.to_string(),
        "SDLC-002: CanonicalCatalog::digest must stay deterministic after adding SDLC files"
    );
    assert!(
        digest.to_string().starts_with("wa:canonical-catalog:"),
        "SDLC-002: consume CanonicalCatalog::digest, do not invent a second digest"
    );
}

#[test]
fn sdlc_003_twenty_six_controls_are_loaded_with_domains_evidence_tests() {
    let catalog = require_sdlc_family();
    let text = sdlc_catalog_text();
    assert_eq!(
        CANONICAL_SDLC_CONTROLS.len(),
        26,
        "pinned independently assessable family size is 26"
    );
    catalog
        .control(PINNED_FIXTURE_CONTROL)
        .expect("SDLC-003: fixture control.source.protected-branch must survive");
    for id in CANONICAL_SDLC_CONTROLS {
        let control = catalog
            .control(id)
            .unwrap_or_else(|e| panic!("SDLC-003: missing control `{id}`: {e}"));
        assert!(
            !control.domains.is_empty() || text_has(control_record_window(&text, id), "domain"),
            "SDLC-003: {id} must declare domain(s)"
        );
        assert!(
            !control.evidence.is_empty(),
            "SDLC-003: {id} must declare evidence requirements"
        );
        assert!(
            !control.tests.is_empty(),
            "SDLC-003: {id} must declare test refs"
        );
        let class = control.automation.to_ascii_lowercase();
        assert!(
            matches!(class.as_str(), "automated" | "hybrid" | "manual"),
            "SDLC-003: {id} automation class must be Automated|Hybrid|Manual, got {class}"
        );
        assert_eq!(
            *id,
            id.to_ascii_lowercase(),
            "SDLC-003: ids are lowercase ({id})"
        );
        assert!(
            hyphenated_catalog_id(id),
            "SDLC-003: catalog ids use hyphen segments ({id})"
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
        "SDLC-003: expected 20–30 independently assessable SDLC controls, found {} ({assessable:?})",
        assessable.len()
    );
}

#[test]
fn sdlc_004_twenty_evidence_types_are_facts_not_conclusions() {
    let catalog = require_sdlc_family();
    let text = sdlc_catalog_text();
    assert_eq!(CANONICAL_SDLC_EVIDENCE.len(), 20);
    for id in CANONICAL_SDLC_EVIDENCE {
        assert!(
            catalog.evidence().contains_key(*id),
            "SDLC-004: missing evidence contract `{id}`"
        );
        assert!(
            id.starts_with("evidence.repository.")
                || id.starts_with("evidence.cicd.")
                || id.starts_with("evidence.deployment.")
                || id.starts_with("evidence.release.")
                || id.starts_with("evidence.supply-chain."),
            "SDLC-004: evidence ids are evidence.repository|cicd|deployment|release|supply-chain.*, not `{id}`"
        );
        let family = id.split('.').nth(1);
        assert!(
            family != Some("github") && family != Some("gitlab") && family != Some("bitbucket"),
            "SDLC-004: `{id}` must not be a provider-shaped evidence type"
        );
    }
    let lower = text.to_ascii_lowercase();
    for phrase in CONCLUSION_PHRASES {
        assert!(
            text_lacks(&lower, phrase),
            "SDLC-004: evidence contracts are facts, not conclusions (`{phrase}`)"
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
        assert!(used, "SDLC-004: evidence `{}` must not be orphaned", ev.id);
    }
}

#[test]
fn sdlc_005_twenty_six_population_tests_are_declared() {
    let catalog = require_sdlc_family();
    let text = sdlc_catalog_text();
    assert_eq!(CANONICAL_SDLC_TESTS.len(), 26);
    assert_eq!(PROMPT05_EXAMPLE_TESTS.len(), 9);
    for id in CANONICAL_SDLC_TESTS {
        assert!(
            catalog.tests().contains_key(*id),
            "SDLC-005: missing test `{id}`"
        );
        let test = catalog.tests().get(*id).unwrap();
        assert!(
            test.control.starts_with("control.source.")
                || test.control.starts_with("control.cicd.")
                || test.control.starts_with("control.release.")
                || test.control.starts_with("control.supply-chain."),
            "SDLC-005: {id} must reference an SDLC control"
        );
    }
    for id in PROMPT05_EXAMPLE_TESTS {
        assert_population_op(&text, id);
    }
}

#[test]
fn sdlc_006_loaded_sdlc_ids_have_no_provider_tokens() {
    let catalog = require_sdlc_family();
    for id in catalog
        .controls()
        .keys()
        .chain(catalog.evidence().keys())
        .chain(catalog.tests().keys())
    {
        if !is_sdlc_family_id(id) {
            continue;
        }
        for token in FORBIDDEN_PROVIDER_TOKENS {
            assert!(
                !id_has_token(id, token),
                "SDLC-006: provider token `{token}` leaked into id `{id}`"
            );
        }
        for token in FORBIDDEN_NATIVE_ID_TOKENS {
            assert!(
                !id_has_token(&id.to_ascii_lowercase(), token),
                "SDLC-006: GitHub-native object name `{token}` leaked into id `{id}`"
            );
        }
    }
}

#[test]
fn sdlc_007_sdlc_catalog_text_has_no_framework_tokens() {
    let _catalog = require_sdlc_family();
    let lower = sdlc_catalog_text().to_ascii_lowercase();
    for token in FORBIDDEN_FRAMEWORK_TOKENS {
        assert!(
            text_lacks(&lower, token),
            "SDLC-007: canonical SDLC content must not mention `{token}`"
        );
    }
}

#[test]
fn sdlc_008_iso_pack_source_sliver_unchanged_and_population_id_is_canonical() {
    let catalog = require_sdlc_family();
    catalog
        .control(POPULATION_DEFAULT_BRANCH)
        .expect("SDLC-008: population default-branch id lives in the canonical catalog");
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    for id in ISO_SOURCE_CONTROLS {
        assert!(
            text_has(&metadata, id)
                || pack.controls.iter().any(|c| c.id().as_str() == *id)
                || catalog.control(id).is_err(),
            "SDLC-008: this slice must not invent a competing pack-local `{id}` library"
        );
    }
    assert!(
        text_lacks(&metadata, "id = \"control.cicd.")
            && text_lacks(&metadata, "id = \"control.release.")
            && text_lacks(&metadata, "id = \"control.supply-chain."),
        "SDLC-008: do not grow ISO metadata with cicd/release/supply-chain control rows"
    );
    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, _to) in ISO_SOURCE_MAPPINGS {
        assert!(
            text_has(&mappings, &format!("from = \"{from}\"")),
            "SDLC-008: mapping source {from} must stay in the ISO pack"
        );
    }
    assert!(
        text_lacks(&mappings, "control.cicd.") && text_lacks(&mappings, "control.supply-chain."),
        "SDLC-008: this slice must not retarget ISO mappings onto cicd/supply-chain ids"
    );
    let _ = pack;
}

#[test]
fn sdlc_009_default_branch_and_required_examples_are_population_not_exists() {
    let catalog = require_sdlc_family();
    let text = sdlc_catalog_text();
    require_seven_fixtures();
    assert_population_op(&text, "test.source.default-branches-protected");

    let mut lone = EvidenceSet::new();
    branch_protection(&mut lone, "repo:random", true, 1);
    let exists = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        "evidence.repository.branch-protection",
    )));
    let (exists_ok, _) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
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
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &lone,
    );
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "SDLC-009: a single protection envelope must not pass all in-scope default branches protected; json={json}"
    );

    let mut unprotected = healthy_population();
    branch_protection(&mut unprotected, "repo:app", false, 0);
    let (failed, fail_json) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &unprotected,
    );
    assert_eq!(
        failed.effectiveness,
        Effectiveness::Ineffective,
        "SDLC-009: unprotected-default-branch is Ineffective, got {:?}",
        failed.effectiveness
    );
    let failing = string_list(&fail_json, "failingSubjects");
    assert!(
        failing.iter().any(|s| text_has(s, "repo:app")),
        "SDLC-009: failing subject must name the unprotected repository; got {failing:?}"
    );

    let healthy = healthy_population();
    for (test_id, control_id, expr) in [
        (
            "test.source.default-branches-protected",
            POPULATION_DEFAULT_BRANCH,
            default_branch_expr(),
        ),
        (
            "test.source.force-push-restricted",
            "control.source.force-push-restricted",
            coverage_100(
                "repository",
                "evidence.repository.branch-protection",
                "force_push_restricted",
            ),
        ),
        (
            "test.source.reviews-required",
            "control.source.required-review",
            coverage_100(
                "repository",
                "evidence.repository.review-policy",
                "reviews_required",
            ),
        ),
        (
            "test.source.minimum-reviewer-count",
            "control.source.minimum-reviewer-count",
            coverage_100(
                "repository",
                "evidence.repository.review-policy",
                "meets_review_threshold",
            ),
        ),
        (
            "test.source.secret-scanning-enabled",
            "control.source.secret-scanning",
            coverage_100(
                "repository",
                "evidence.repository.security-scanning",
                "secret_scanning_enabled",
            ),
        ),
        (
            "test.cicd.workflow-permissions-minimized",
            "control.cicd.workflow-permissions",
            coverage_100(
                "repository",
                "evidence.cicd.workflow-permissions",
                "permissions_minimized",
            ),
        ),
        (
            "test.release.environments-protected",
            "control.release.protected-environment",
            coverage_100(
                "deployment",
                "evidence.deployment.environment-protection",
                "authorization_required",
            ),
        ),
        (
            "test.supply-chain.artifacts-have-integrity",
            "control.supply-chain.artifact-integrity",
            coverage_100(
                "repository",
                "evidence.supply-chain.artifact-integrity",
                "integrity_evidence_present",
            ),
        ),
    ] {
        let (ok, _) = result_json(test_id, control_id, expr, &healthy);
        assert_eq!(
            ok.effectiveness,
            Effectiveness::Effective,
            "SDLC-009: healthy-org {test_id} must be Effective, got {:?}",
            ok.effectiveness
        );
        let _ = catalog.tests().get(test_id);
    }
}

#[test]
fn sdlc_010_seven_fixtures_distinguish_missing_stale_fail_manual_exception() {
    let _catalog = require_sdlc_family();
    require_seven_fixtures();
    assert_eq!(SDLC_FIXTURES.len(), 7);

    let healthy = healthy_population();
    let (ok, _) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &healthy,
    );
    assert_eq!(ok.effectiveness, Effectiveness::Effective);

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
        "SDLC-010: missing scan evidence is InsufficientEvidence, not technical failure; got {:?} {miss_json}",
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
        "SDLC-010 stale-dependency-scan: StaleEvidence, not Ineffective-as-missing"
    );

    let (manual, _) = result_json(
        "test.source.security-review-recorded",
        "control.source.security-review",
        TestExpr::ManualReview,
        &healthy,
    );
    assert_eq!(
        manual.effectiveness,
        Effectiveness::ManualReviewRequired,
        "SDLC-010: Hybrid/Manual security-review without attestation is ManualReviewRequired"
    );

    let mut write = healthy_population();
    workflow_permissions(&mut write, "repo:app", false);
    let (overbroad, _) = result_json(
        "test.cicd.workflow-permissions-minimized",
        "control.cicd.workflow-permissions",
        coverage_100(
            "repository",
            "evidence.cicd.workflow-permissions",
            "permissions_minimized",
        ),
        &write,
    );
    assert_eq!(
        overbroad.effectiveness,
        Effectiveness::Ineffective,
        "SDLC-010 degraded-org: overbroad CI write permissions are Ineffective"
    );
}

#[test]
fn sdlc_011_partial_coverage_cannot_be_effective() {
    let _catalog = require_sdlc_family();
    require_seven_fixtures();
    let mut partial = EvidenceSet::new();
    inventory_repo(&mut partial, "repo:app", false);
    branch_protection(&mut partial, "repo:app", true, 1);
    let (part, part_json) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &partial,
    );
    assert_ne!(
        part.effectiveness,
        Effectiveness::Effective,
        "SDLC-011: partial/unknown population must not yield Effective; json={part_json}"
    );
    assert!(
        matches!(
            part.effectiveness,
            Effectiveness::InsufficientEvidence | Effectiveness::Inconclusive
        ),
        "SDLC-011: partial/unknown population → InsufficientEvidence or Inconclusive, got {:?}",
        part.effectiveness
    );
}

#[test]
fn sdlc_012_approved_unexpired_exception_is_excepted_not_fail() {
    let _catalog = require_sdlc_family();
    require_seven_fixtures();

    let mut approved = EvidenceSet::new();
    inventory_complete(&mut approved);
    inventory_repo(&mut approved, "repo:legacy", false);
    inventory_repo(&mut approved, "repo:app", false);
    branch_protection(&mut approved, "repo:legacy", false, 1);
    branch_protection(&mut approved, "repo:app", true, 1);
    approved.insert_exception(bind_repo_exception(
        ExceptionStatus::Approved,
        "repo:legacy",
        Some(Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap()),
    ));
    let (ok, json) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &approved,
    );
    let excepted = string_list(&json, "exceptedSubjects");
    assert!(
        ok.effectiveness == Effectiveness::ExceptionApproved
            || excepted.iter().any(|s| text_has(s, "repo:legacy")),
        "SDLC-012: approved unexpired Exception IR → ExceptionApproved or excepted partition, got {:?} {json}",
        ok.effectiveness
    );
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
            POPULATION_DEFAULT_BRANCH,
            default_branch_expr(),
            &bad,
        );
        assert_ne!(
            result.effectiveness,
            Effectiveness::Effective,
            "SDLC-012: {status:?} exception must not pass"
        );
        assert_ne!(
            result.effectiveness,
            Effectiveness::ExceptionApproved,
            "SDLC-012: {status:?} exception must not be ExceptionApproved"
        );
        assert_eq!(
            result.effectiveness,
            Effectiveness::Ineffective,
            "SDLC-012: expired/revoked leave the unprotected repo as Ineffective, got {:?}",
            result.effectiveness
        );
    }
}

#[test]
fn sdlc_013_release_authority_security_review_and_policy_stay_hybrid_or_manual() {
    let catalog = require_sdlc_family();
    let text = sdlc_catalog_text();
    for id in HYBRID_OR_MANUAL_CONTROLS {
        let class = catalog
            .control(id)
            .map(|c| c.automation.to_ascii_lowercase())
            .unwrap_or_else(|_| control_record_automation(&text, id));
        assert!(
            class == "hybrid" || class == "manual",
            "SDLC-013: {id} must be Hybrid or Manual, got {class}"
        );
        let control = catalog
            .control(id)
            .unwrap_or_else(|_| panic!("SDLC-013: missing hybrid/manual control `{id}`"));
        for test_id in &control.tests {
            let window = test_expression_window(&text, test_id);
            assert!(
                !expression_is_existence_only(&window)
                    || text_has(&window.to_ascii_lowercase(), "manual"),
                "SDLC-013: {test_id} must not auto-pass as Exists(one technical envelope)"
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
        "sanity: Exists(release.authorization) would auto-pass from a single flag"
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
        "SDLC-013: release authorization cannot auto-pass from a single technical flag"
    );
}

#[test]
fn sdlc_014_catalog_does_not_couple_to_github_collector_or_scanner() {
    let catalog = require_sdlc_family();
    assert_eq!(
        GITHUB_EVIDENCE_TYPES, PINNED_GITHUB_EVIDENCE_TYPES,
        "SDLC-014: do not expand GITHUB_EVIDENCE_TYPES in this slice"
    );
    assert!(
        GITHUB_EVIDENCE_TYPES.iter().all(|t| {
            !t.starts_with("evidence.repository.")
                && !t.starts_with("evidence.cicd.")
                && !t.starts_with("evidence.supply-chain.")
        }),
        "SDLC-014: GitHub collector must keep emitting source.* only"
    );
    let text = sdlc_catalog_text();
    assert!(
        text_lacks(&text, "GITHUB_EVIDENCE_TYPES")
            && text_lacks(&text, "source.security.secret_scanning.enabled")
            && text_lacks(&text, "depcheck")
            && text_lacks(&text, "engines::")
            && text_lacks(&text, "src/engines"),
        "SDLC-014: SDLC catalog tests must not couple to GITHUB_EVIDENCE_TYPES or scanner internals"
    );
    let _ = catalog;
}

#[test]
fn sdlc_015_no_repository_inventory_resolver_iam_and_cat_fixture_remain() {
    let catalog = require_sdlc_family();
    let pop = fs::read_to_string(
        manifest_dir().join("crates/weeping-angel-control-test/src/population.rs"),
    )
    .unwrap();
    let forbidden_fn = ["resolve", "repository", "inventory"].join("_");
    assert!(
        text_lacks(&pop, &format!("fn {forbidden_fn}")),
        "SDLC-015: do not add a repository-inventory special case"
    );
    assert!(
        text_has(&pop, "inventory.subject") && text_has(&pop, "inventory.complete"),
        "SDLC-015: generic inventory path must remain"
    );
    for id in IAM_FAMILY_PINS {
        let present = catalog.controls().contains_key(*id)
            || catalog.evidence().contains_key(*id)
            || catalog.tests().contains_key(*id);
        assert!(present, "SDLC-015: IAM pin `{id}` must remain");
    }
    for id in [
        PINNED_FIXTURE_CONTROL,
        PINNED_FIXTURE_EVIDENCE,
        PINNED_FIXTURE_TEST,
    ] {
        let present = catalog.controls().contains_key(id)
            || catalog.evidence().contains_key(id)
            || catalog.tests().contains_key(id);
        assert!(present, "SDLC-015: CAT-015 fixture pin `{id}` must remain");
    }
    let fixture_test = catalog
        .tests()
        .get(PINNED_FIXTURE_TEST)
        .expect("SDLC-015: test.source.protected-branch remains");
    let op = fixture_test
        .expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        op, "exists",
        "SDLC-015: CAT fixture test stays exists-only; got {op}"
    );
}

#[test]
fn sdlc_016_no_repository_toml_and_provider_neutral_collectors_share_contracts() {
    let catalog = require_sdlc_family();
    assert!(
        !catalog_v1_dir().join("evidence/repository.toml").is_file(),
        "SDLC-016: prefer evidence/sdlc.toml so ghc_b028 stays green (no evidence/repository.toml)"
    );
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !text_has(&cargo, "sdd_github_collector_baseline")
            && !text_has(&cargo, "tests/contracts/github_collector.baseline.rs"),
        "SDLC-016: superseded github_collector baseline must stay deleted"
    );
    for suite in [
        "sdd_iso27001_assurance_target",
        "sdd_iam_catalog_target",
        "sdd_canonical_assurance_catalog_target",
        "sdd_github_collector_target",
    ] {
        assert!(
            sdd_suite_wired(suite),
            "SDLC-016: sibling suite `{suite}` must remain registered"
        );
    }

    let mut gitlab = EvidenceSet::new();
    inventory_complete(&mut gitlab);
    inventory_repo(&mut gitlab, "repo:app", false);
    branch_protection(&mut gitlab, "repo:app", true, 1);
    gitlab.insert(seal_named(
        "collector.gitlab",
        "evidence.repository.branch-protection",
        "repo:app",
        &[("subject_id", "repo:app"), ("protected", "true")],
        collected(1),
    ));
    let mut bitbucket = EvidenceSet::new();
    inventory_complete(&mut bitbucket);
    inventory_repo(&mut bitbucket, "repo:app", false);
    branch_protection(&mut bitbucket, "repo:app", true, 1);
    bitbucket.insert(seal_named(
        "collector.bitbucket",
        "evidence.repository.branch-protection",
        "repo:app",
        &[("subject_id", "repo:app"), ("protected", "true")],
        collected(1),
    ));
    let (g, _) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &gitlab,
    );
    let (b, _) = result_json(
        "test.source.default-branches-protected",
        POPULATION_DEFAULT_BRANCH,
        default_branch_expr(),
        &bitbucket,
    );
    assert_eq!(
        g.effectiveness, b.effectiveness,
        "SDLC-016: GitLab and Bitbucket collectors populating the same evidence contracts must receive the same control results"
    );
    assert_eq!(g.effectiveness, Effectiveness::Effective);
    let _ = catalog;
}
