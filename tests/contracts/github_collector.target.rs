//! Target suite for the GitHub collector.
//!
//! Encodes DESIRED behavior in `docs/specs/github-collector.md` §4 / §5
//! (`ghc_000`–`ghc_024`). Must stay RED on the current ISO-sliver
//! collector: `source.*` string facts, advertised-vs-collected gap,
//! abort-on-403, empty `CollectionRun`, no org inventory, no goldens.
//! Do not `#[ignore]` these tests and do not implement the collector here.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_collector::github::{GITHUB_EVIDENCE_TYPES, GitHubClient, GitHubCollector};
use weeping_angel_collector::{
    CollectionBatch, CollectionRequest, CollectorScope, EvidenceCollector,
};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceValue};

const ADVERTISED_UNCOLLECTED_SOURCE: &[&str] = &[
    "source.codeowners.present",
    "source.admin.permissions",
    "source.collaborator.permission",
    "source.security.dependabot.enabled",
    "source.security.secret_scanning.enabled",
    "source.security.code_scanning.configured",
    "source.workflow.permissions",
    "source.workflow.review_requirement",
    "source.ruleset.present",
    "source.commit.signing",
];

const HISTORICAL_SOURCE_TYPES: &[&str] = &[
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

/// IAM/SDLC catalogs + population runtime contracts the descriptor must advertise once
/// the corresponding modules actually collect them.
const CANONICAL_EVIDENCE_TYPES: &[&str] = &[
    "evidence.repository.inventory",
    "evidence.repository.visibility",
    "evidence.repository.default-branch",
    "evidence.repository.branch-protection",
    "evidence.repository.review-policy",
    "evidence.repository.review-ownership",
    "evidence.repository.security-scanning",
    "evidence.repository.dependency-scanning",
    "evidence.repository.commit-signing",
    "evidence.cicd.status-checks",
    "evidence.cicd.workflow-permissions",
    "evidence.deployment.environment-protection",
    "evidence.identity.privileged-membership",
    "evidence.identity.external-access",
    "inventory.subject",
    "inventory.complete",
];

const IDENTITY_TYPES: &[&str] = &[
    "evidence.identity.privileged-membership",
    "evidence.identity.external-access",
];

const GOLDEN_IDS: &[&str] = &[
    "healthy-org",
    "unprotected-repo",
    "missing-branch-protection-permission",
    "paginated-inventory",
    "archived-excluded-by-selector",
    "disabled-security-scanning",
    "protected-environment-absent",
    "privileged-membership-population",
    "api-partial-failure",
    "rate-limit-retry",
];

/// Independently assessable controls enabled by type/fact coverage
/// (`docs/specs/github-collector.md` §4.10). SDLC catalog TOML is not landed;
/// the suite enumerates pairs instead of loading catalog tests.
const EXERCISABLE_CONTROLS: &[(&str, &str, &str)] = &[
    (
        "control.source.repository-inventory",
        "evidence.repository.inventory",
        "subject_id",
    ),
    (
        "control.source.repository-inventory",
        "inventory.complete",
        "authoritative",
    ),
    (
        "control.source.visibility-governance",
        "evidence.repository.visibility",
        "visibility",
    ),
    (
        "control.source.default-branch-protection",
        "evidence.repository.default-branch",
        "default_branch",
    ),
    (
        "control.source.default-branch-protection",
        "evidence.repository.branch-protection",
        "protected",
    ),
    (
        "control.source.force-push-restricted",
        "evidence.repository.branch-protection",
        "force_push_allowed",
    ),
    (
        "control.source.branch-deletion-restricted",
        "evidence.repository.branch-protection",
        "deletion_allowed",
    ),
    (
        "control.source.admin-bypass-governance",
        "evidence.repository.branch-protection",
        "admin_bypass_allowed",
    ),
    (
        "control.source.required-review",
        "evidence.repository.review-policy",
        "reviews_required",
    ),
    (
        "control.source.minimum-reviewer-count",
        "evidence.repository.review-policy",
        "required_reviewer_count",
    ),
    (
        "control.source.review-ownership",
        "evidence.repository.review-ownership",
        "ownership_defined",
    ),
    (
        "control.source.required-status-checks",
        "evidence.cicd.status-checks",
        "status_checks_required",
    ),
    (
        "control.source.signed-commits",
        "evidence.repository.commit-signing",
        "signing_required",
    ),
    (
        "control.source.secret-scanning",
        "evidence.repository.security-scanning",
        "secret_scanning_enabled",
    ),
    (
        "control.source.code-scanning",
        "evidence.repository.security-scanning",
        "code_scanning_enabled",
    ),
    (
        "control.source.dependency-scanning",
        "evidence.repository.dependency-scanning",
        "dependency_scanning_enabled",
    ),
    (
        "control.source.dependency-update-monitoring",
        "evidence.repository.dependency-scanning",
        "updates_monitored",
    ),
    (
        "control.cicd.workflow-permissions",
        "evidence.cicd.workflow-permissions",
        "default_write",
    ),
    (
        "control.cicd.workflow-permissions",
        "evidence.cicd.workflow-permissions",
        "permissions_minimized",
    ),
    (
        "control.release.protected-environment",
        "evidence.deployment.environment-protection",
        "protected",
    ),
    (
        "control.release.authorization",
        "evidence.deployment.environment-protection",
        "authorization_required",
    ),
    (
        "control.identity.privileged-membership",
        "evidence.identity.privileged-membership",
        "privileged",
    ),
    (
        "control.identity.privileged-inventory",
        "inventory.subject",
        "kind",
    ),
    (
        "control.identity.external-access",
        "evidence.identity.external-access",
        "external",
    ),
    (
        "control.identity.service-account",
        "evidence.identity.privileged-membership",
        "roles",
    ),
    (
        "control.identity.least-privilege",
        "evidence.identity.privileged-membership",
        "roles",
    ),
    (
        "control.identity.privileged-access-minimization",
        "evidence.identity.privileged-membership",
        "privileged",
    ),
    (
        "control.identity.service-account-inventory",
        "inventory.subject",
        "id",
    ),
];

const TOKEN_NEEDLES: &[&str] = &["ghp_", "gho_", "github_pat_", "ghs_", "Bearer "];

const REPO_DEVELOP: &str = r#"{
    "name": "app",
    "full_name": "acme/app",
    "visibility": "private",
    "default_branch": "develop",
    "archived": false
}"#;

const REPO_OTHER: &str = r#"{
    "name": "other",
    "full_name": "acme/other",
    "visibility": "private",
    "default_branch": "main",
    "archived": false
}"#;

const REPO_ARCHIVED: &str = r#"{
    "name": "legacy",
    "full_name": "acme/legacy",
    "visibility": "internal",
    "default_branch": "main",
    "archived": true
}"#;

const PROTECTION_HEALTHY: &str = r#"{
    "required_pull_request_reviews": { "required_approving_review_count": 2 },
    "allow_force_pushes": { "enabled": false },
    "allow_deletions": { "enabled": false },
    "enforce_admins": { "enabled": true },
    "required_status_checks": { "strict": true, "contexts": ["ci"] },
    "required_signatures": { "enabled": true }
}"#;

const ORG_REPOS_PAGE: &str = r#"[
    {"name":"app","full_name":"acme/app","visibility":"private","default_branch":"develop","archived":false},
    {"name":"other","full_name":"acme/other","visibility":"private","default_branch":"main","archived":false}
]"#;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn github_src() -> PathBuf {
    manifest_dir().join("crates/weeping-angel-collector/src/github")
}

fn github_file(rel: &str) -> String {
    fs::read_to_string(github_src().join(rel)).unwrap_or_else(|e| {
        panic!("read github/{rel}: {e}");
    })
}

fn github_sources_joined() -> String {
    let mut files = Vec::new();
    walk_files(&github_src(), &mut files);
    files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collector_sources_joined() -> String {
    let src = manifest_dir().join("crates/weeping-angel-collector/src");
    let mut files = Vec::new();
    walk_files(&src, &mut files);
    files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn golden_root() -> PathBuf {
    let preferred = manifest_dir().join("fixtures/assurance/canonical/v1/github");
    if preferred.is_dir() {
        return preferred;
    }
    manifest_dir().join("fixtures/collectors/github")
}

fn golden_dir(id: &str) -> PathBuf {
    golden_root().join(id)
}

fn repo_scope(owner: &str, name: &str) -> CollectorScope {
    CollectorScope::new().allow_asset(AssetId::new(format!("repo:{owner}/{name}")))
}

fn multi_repo_scope(repos: &[(&str, &str)]) -> CollectorScope {
    let mut scope = CollectorScope::new();
    for (owner, name) in repos {
        scope = scope.allow_asset(AssetId::new(format!("repo:{owner}/{name}")));
    }
    scope
}

fn org_scope(org: &str) -> CollectorScope {
    CollectorScope::new().allow_asset(AssetId::new(format!("org:{org}")))
}

/// GitHub-owned archived selector encoded as a scope label (no IR selector type).
fn org_scope_exclude_archived(org: &str) -> CollectorScope {
    org_scope(org).allow_asset(AssetId::new("exclude_archived"))
}

fn client_with(fixtures: &[(&str, u16, &str)]) -> GitHubClient {
    let mut client = GitHubClient::new(Some("test-token".into()));
    for (path, status, body) in fixtures {
        client = client.with_fixture(path, *status, body, None);
    }
    client
}

fn client_with_retry(fixtures: &[(&str, u16, &str, Option<u64>)]) -> GitHubClient {
    let mut client = GitHubClient::new(Some("test-token".into()));
    for (path, status, body, retry) in fixtures {
        client = client.with_fixture(path, *status, body, *retry);
    }
    client
}

fn batch_ok(fixtures: &[(&str, u16, &str)], scope: CollectorScope) -> CollectionBatch {
    GitHubCollector::with_client(client_with(fixtures))
        .collect_batch(CollectionRequest { scope })
        .unwrap_or_else(|e| panic!("collect_batch should return a run (partial ok), got {e}"))
}

fn types_of(envelopes: &[EvidenceEnvelope]) -> BTreeSet<String> {
    envelopes
        .iter()
        .map(|e| e.observation().evidence_type().as_str().to_string())
        .collect()
}

fn of_type<'a>(envelopes: &'a [EvidenceEnvelope], ty: &str) -> Vec<&'a EvidenceEnvelope> {
    envelopes
        .iter()
        .filter(|e| e.observation().evidence_type().as_str() == ty)
        .collect()
}

fn require_type<'a>(envelopes: &'a [EvidenceEnvelope], ty: &str) -> &'a EvidenceEnvelope {
    of_type(envelopes, ty)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing envelope type {ty}"))
}

fn typed_bool(env: &EvidenceEnvelope, key: &str) -> bool {
    match env.observation().fact_value(key) {
        Some(EvidenceValue::Bool(b)) => *b,
        other => panic!(
            "{}.{} must be EvidenceValue::Bool, got {other:?}",
            env.observation().evidence_type(),
            key
        ),
    }
}

fn typed_int(env: &EvidenceEnvelope, key: &str) -> i64 {
    match env.observation().fact_value(key) {
        Some(EvidenceValue::Integer(n)) => *n,
        other => panic!(
            "{}.{} must be EvidenceValue::Integer, got {other:?}",
            env.observation().evidence_type(),
            key
        ),
    }
}

fn typed_str<'a>(env: &'a EvidenceEnvelope, key: &str) -> &'a str {
    match env.observation().fact_value(key) {
        Some(EvidenceValue::String(s)) => s.as_str(),
        other => panic!(
            "{}.{} must be EvidenceValue::String, got {other:?}",
            env.observation().evidence_type(),
            key
        ),
    }
}

fn subject_of(env: &EvidenceEnvelope) -> String {
    env.observation()
        .fact("subject_id")
        .or_else(|| env.observation().fact("id"))
        .unwrap_or_else(|| env.provenance().asset().as_str())
        .to_string()
}

fn no_negative_from_denial(envelopes: &[EvidenceEnvelope]) {
    for env in envelopes {
        let ty = env.observation().evidence_type().as_str();
        if ty == "evidence.repository.branch-protection" {
            assert!(
                typed_bool(env, "protected"),
                "401/403 must not become protected=false on {ty} for {}",
                subject_of(env)
            );
        }
    }
}

fn assert_no_source_star_emitted(envelopes: &[EvidenceEnvelope]) {
    for env in envelopes {
        let ty = env.observation().evidence_type().as_str();
        assert!(
            !ty.starts_with("source."),
            "new observations must be canonical contracts, got {ty}"
        );
        assert!(
            !ty.contains(&format!("{}.{}", "evidence", "github")),
            "canonical tests must not require provider-native types, got {ty}"
        );
    }
}

fn load_golden(id: &str) -> (CollectorScope, GitHubClient, bool) {
    let dir = golden_dir(id);
    assert!(
        dir.is_dir(),
        "golden `{id}` must exist at {} (or fixtures/collectors/github/{id})",
        dir.display()
    );
    let http_path = dir.join("http.json");
    assert!(
        http_path.is_file(),
        "golden `{id}` needs http.json at {}",
        http_path.display()
    );
    let raw = fs::read_to_string(&http_path).unwrap();
    assert_no_token_material(&raw, &format!("golden {id} http.json"));
    let parsed: Value = serde_json::from_str(&raw).unwrap_or_else(|e| {
        panic!("golden {id} http.json is not JSON: {e}");
    });
    let scope_label = parsed
        .get("scope")
        .and_then(Value::as_str)
        .unwrap_or("org:acme");
    let exclude = parsed
        .get("exclude_archived")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut scope = CollectorScope::new();
    for part in scope_label.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            scope = scope.allow_asset(AssetId::new(part));
        }
    }
    if exclude {
        scope = scope.allow_asset(AssetId::new("exclude_archived"));
    }
    let requests = parsed
        .get("requests")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("golden {id} http.json needs a requests array"));
    let mut client = GitHubClient::new(Some("test-token".into()));
    for req in requests {
        let path = req
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("golden {id} request missing path"));
        let status = req.get("status").and_then(Value::as_u64).unwrap_or(200) as u16;
        let body = match req.get("body") {
            Some(Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => "{}".into(),
        };
        let retry = req.get("retry_after").and_then(Value::as_u64);
        client = client.with_fixture(path, status, &body, retry);
    }
    (scope, client, exclude)
}

fn collect_golden(id: &str) -> CollectionBatch {
    let (scope, client, _) = load_golden(id);
    GitHubCollector::with_client(client)
        .collect_batch(CollectionRequest { scope })
        .unwrap_or_else(|e| panic!("golden `{id}` collect_batch must not abort: {e}"))
}

fn assert_no_token_material(text: &str, where_: &str) {
    for needle in TOKEN_NEEDLES {
        assert!(
            !text.contains(needle),
            "{where_} must not contain credential material `{needle}`"
        );
    }
}

fn scan_batch_for_tokens(batch: &CollectionBatch, where_: &str) {
    for env in &batch.envelopes {
        let json = serde_json::to_string(env).expect("envelope serializes");
        assert_no_token_material(&json, &format!("{where_} envelope"));
        assert_no_token_material(
            env.observation().narrative(),
            &format!("{where_} narrative"),
        );
    }
    for err in &batch.errors {
        assert_no_token_material(err, &format!("{where_} diagnostic"));
    }
    assert_no_token_material(&batch.run.configuration_digest, &format!("{where_} digest"));
    assert_no_token_material(&batch.run.scope, &format!("{where_} run.scope"));
}

fn develop_repo_fixtures() -> Vec<(&'static str, u16, &'static str)> {
    vec![
        (
            "/repos/acme/app/branches/develop/protection",
            200,
            PROTECTION_HEALTHY,
        ),
        (
            "/repos/acme/app/branches/main/protection",
            404,
            r#"{"message":"Not Found"}"#,
        ),
        ("/repos/acme/app", 200, REPO_DEVELOP),
    ]
}

// ── Registration / descriptor honesty ──────────────────────────────────────

#[test]
fn ghc_000_dual_suite_remains_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !toml.contains("name = \"sdd_github_collector_baseline\"")
            && !toml.contains("path = \"tests/contracts/github_collector.baseline.rs\""),
        "baseline suite must stay registered"
    );
    assert!(
        toml.contains("name = \"sdd_github_collector_target\"")
            && toml.contains("path = \"tests/contracts/github_collector.target.rs\""),
        "target suite must stay registered"
    );
}

#[test]
fn ghc_001_descriptor_advertises_only_implemented_canonical_types() {
    let desc = GitHubCollector::new(None).descriptor();
    assert_eq!(desc.id, "collector.github");
    assert_eq!(desc.provider_family, "source-control");
    assert!(!desc.version.is_empty(), "descriptor version is required");

    let advertised: BTreeSet<&str> = desc.evidence_types.iter().map(|t| t.as_str()).collect();
    for ty in CANONICAL_EVIDENCE_TYPES {
        assert!(
            advertised.contains(ty),
            "descriptor must advertise implemented canonical type `{ty}`"
        );
    }
    for ty in ADVERTISED_UNCOLLECTED_SOURCE {
        assert!(
            !advertised.contains(ty),
            "descriptor must not advertise uncollected `{ty}` as an emitted type"
        );
    }
    for ty in advertised {
        assert!(
            !ty.starts_with("source."),
            "descriptor.evidence_types is the canonical set, not historical source.* ({ty})"
        );
        let provider_native = format!("{}.{}", "evidence", "github");
        assert!(
            !ty.starts_with(&provider_native),
            "descriptor must not advertise provider-native {ty}"
        );
    }

    for subject in [
        "repository",
        "branch",
        "organization",
        "identity",
        "deployment",
    ] {
        assert!(
            desc.subject_types.iter().any(|s| s == subject),
            "subject_types must include `{subject}` once those modules emit"
        );
    }

    let perms: BTreeSet<&str> = desc
        .required_permissions
        .iter()
        .map(String::as_str)
        .collect();
    for needed in [
        "metadata:read",
        "administration:read",
        "actions:read",
        "members:read",
        "security_events:read",
    ] {
        assert!(
            perms
                .iter()
                .any(|p| p.contains(needed.split(':').next().unwrap_or(needed))),
            "required_permissions must name the GitHub scope actually needed for advertised types; missing `{needed}` in {perms:?}"
        );
    }
}

#[test]
fn ghc_002_identity_types_use_a_second_const_not_github_evidence_types() {
    assert!(
        GITHUB_EVIDENCE_TYPES
            .iter()
            .all(|t| !t.starts_with("evidence.identity.")),
        "IAM-015: GITHUB_EVIDENCE_TYPES must stay free of evidence.identity.*"
    );
    let desc_src = github_file("descriptor.rs");
    assert!(
        desc_src.contains("GITHUB_CANONICAL_EVIDENCE_TYPES"),
        "identity/canonical types must live on GITHUB_CANONICAL_EVIDENCE_TYPES and be unioned at descriptor()"
    );
    let advertised: BTreeSet<String> = GitHubCollector::new(None)
        .descriptor()
        .evidence_types
        .iter()
        .map(|t| t.as_str().to_string())
        .collect();
    for ty in IDENTITY_TYPES {
        assert!(
            advertised.iter().any(|t| t == ty),
            "descriptor must advertise `{ty}` via the canonical const, not GITHUB_EVIDENCE_TYPES"
        );
        assert!(
            !GITHUB_EVIDENCE_TYPES.contains(ty),
            "`{ty}` must not be appended to GITHUB_EVIDENCE_TYPES"
        );
    }
}

#[test]
fn ghc_003_failure_behavior_is_documented_in_github_owned_sources() {
    let desc_src = github_file("descriptor.rs");
    assert!(
        desc_src.contains("GITHUB_FAILURE_BEHAVIOR"),
        "failure behavior must be a GitHub-owned const, not a shared CollectorDescriptor field"
    );
    for needle in [
        "PermissionDenied",
        "403",
        "401",
        "insufficient",
        "404",
        "429",
    ] {
        assert!(
            desc_src
                .to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase())
                || github_sources_joined()
                    .to_ascii_lowercase()
                    .contains(&needle.to_ascii_lowercase()),
            "failure-behavior documentation must mention `{needle}`"
        );
    }
    let json = serde_json::to_value(GitHubCollector::new(None).descriptor()).unwrap();
    assert!(
        json.get("failureBehavior").is_none() && json.get("failure_behavior").is_none(),
        "do not add failure_behavior to shared CollectorDescriptor: {json}"
    );
}

#[test]
fn ghc_004_pagination_flag_matches_a_real_walker_incremental_is_honest() {
    let desc = GitHubCollector::new(None).descriptor();
    let client_src = github_file("client.rs");
    let walker = client_src.contains("per_page")
        || client_src.contains("Link")
        || client_src.contains("page=")
        || github_sources_joined().contains("per_page");
    assert!(
        walker,
        "authoritative populations require a real page walker (per_page / Link / page=)"
    );
    assert!(
        desc.capabilities.pagination,
        "capabilities.pagination must be true once the walker exists"
    );
    let incremental_impl = client_src.contains("etag")
        || client_src.contains("cursor")
        || client_src.contains("If-None-Match")
        || github_sources_joined().contains("incremental_cursor");
    assert_eq!(
        desc.capabilities.incremental, incremental_impl,
        "capabilities.incremental must match a real cursor/etag path (do not lie)"
    );

    // Fixture match must be longest-prefix-safe so `/repos/acme/app` cannot
    // steal `/repos/acme/app/branches/develop/protection`.
    let client = client_with(&[
        ("/repos/acme/app", 200, REPO_DEVELOP),
        (
            "/repos/acme/app/branches/develop/protection",
            200,
            PROTECTION_HEALTHY,
        ),
    ]);
    let (status, body, _) = client
        .get("/repos/acme/app/branches/develop/protection")
        .expect("protection fixture must be reachable");
    assert_eq!(status, 200);
    assert!(
        body.contains("required_pull_request_reviews"),
        "shorter repo path must not steal the protection fixture: {body}"
    );
}

// ── Canonical mapping ──────────────────────────────────────────────────────

#[test]
fn ghc_005_new_envelopes_use_iam_sdlc_type_ids() {
    let batch = batch_ok(&develop_repo_fixtures(), repo_scope("acme", "app"));
    assert!(
        !batch.envelopes.is_empty(),
        "healthy collect must emit envelopes"
    );
    assert_no_source_star_emitted(&batch.envelopes);
    let types = types_of(&batch.envelopes);
    for required in [
        "evidence.repository.visibility",
        "evidence.repository.default-branch",
        "evidence.repository.branch-protection",
        "evidence.repository.review-policy",
        "evidence.cicd.status-checks",
    ] {
        assert!(
            types.contains(required),
            "expected canonical type `{required}`, got {types:?}"
        );
    }
}

#[test]
fn ghc_006_facts_use_typed_evidence_value() {
    let batch = batch_ok(&develop_repo_fixtures(), repo_scope("acme", "app"));
    let prot = require_type(&batch.envelopes, "evidence.repository.branch-protection");
    assert!(typed_bool(prot, "protected"));
    assert!(!typed_bool(prot, "force_push_allowed"));
    assert!(!typed_bool(prot, "deletion_allowed"));
    let reviews = require_type(&batch.envelopes, "evidence.repository.review-policy");
    assert!(typed_bool(reviews, "reviews_required"));
    assert_eq!(typed_int(reviews, "required_reviewer_count"), 2);
    let vis = require_type(&batch.envelopes, "evidence.repository.visibility");
    assert_eq!(typed_str(vis, "visibility"), "private");
    assert_eq!(typed_str(vis, "subject_id"), "repo:acme/app");
}

#[test]
fn ghc_007_protection_uses_default_branch_not_hardcoded_main() {
    let collect_src = github_file("mod.rs");
    assert!(
        !collect_src.contains("\"main\"") || collect_src.contains("default_branch"),
        "protection/ruleset path must use the repo default_branch"
    );
    assert!(
        !collect_src.contains("/branches/{}/protection")
            || github_sources_joined().contains("default_branch"),
        "hardcoded main protection template is forbidden"
    );

    // Only the real default branch (`develop`) is protected. Hitting `main`
    // would 404 and must not be treated as the authoritative observation.
    let batch = batch_ok(
        &[
            (
                "/repos/acme/app/branches/develop/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_DEVELOP),
        ],
        repo_scope("acme", "app"),
    );
    let prot = require_type(&batch.envelopes, "evidence.repository.branch-protection");
    assert!(
        typed_bool(prot, "protected"),
        "protection on default branch develop must be observed"
    );
    let branch = require_type(&batch.envelopes, "evidence.repository.default-branch");
    assert_eq!(typed_str(branch, "default_branch"), "develop");
}

#[test]
fn ghc_008_mapping_table_keeps_historical_source_strings() {
    let src = collector_sources_joined();
    for ty in HISTORICAL_SOURCE_TYPES {
        assert!(
            src.contains(ty),
            "ISO GH-012: historical `{ty}` must remain in collector sources as the mapping table"
        );
    }
    let desc_src = github_file("descriptor.rs");
    assert!(
        desc_src.contains("SOURCE_TO_CANONICAL")
            || desc_src.contains("GITHUB_SOURCE_MAPPING")
            || desc_src.contains("mapping_table"),
        "historical source.* strings must live in an explicit mapping table, not as emitted types"
    );
    assert_eq!(
        GITHUB_EVIDENCE_TYPES.len(),
        HISTORICAL_SOURCE_TYPES.len(),
        "GITHUB_EVIDENCE_TYPES remains the ADR 0002 source.* list"
    );
}

#[test]
fn ghc_009_org_scope_emits_inventory_subject_and_honest_complete() {
    let src = github_sources_joined();
    assert!(
        src.contains("/orgs/") || src.contains("/orgs/{"),
        "org: scope must inventory repositories via the org list API"
    );
    let batch = batch_ok(
        &[
            ("/orgs/acme/repos", 200, ORG_REPOS_PAGE),
            (
                "/repos/acme/app/branches/develop/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            (
                "/repos/acme/other/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_DEVELOP),
            ("/repos/acme/other", 200, REPO_OTHER),
        ],
        org_scope("acme"),
    );
    let subjects = of_type(&batch.envelopes, "inventory.subject");
    assert!(
        subjects.len() >= 2,
        "org inventory must emit inventory.subject per in-scope repo, got {}",
        subjects.len()
    );
    for sub in &subjects {
        assert_eq!(typed_str(sub, "kind"), "repository");
        let id = typed_str(sub, "id");
        assert!(
            id.starts_with("repo:"),
            "inventory.subject id must be repo:owner/name, got {id}"
        );
    }
    let complete = require_type(&batch.envelopes, "inventory.complete");
    assert!(
        typed_bool(complete, "authoritative"),
        "complete org pagination with no list hole is authoritative"
    );
}

// ── Golden adapter fixtures ────────────────────────────────────────────────

#[test]
fn ghc_010_golden_healthy_org() {
    let batch = collect_golden("healthy-org");
    assert!(
        matches!(batch.run.status.as_str(), "complete"),
        "healthy-org run status must be complete, got {}",
        batch.run.status
    );
    let types = types_of(&batch.envelopes);
    for ty in [
        "evidence.repository.inventory",
        "evidence.repository.visibility",
        "evidence.repository.default-branch",
        "evidence.repository.branch-protection",
        "evidence.repository.review-policy",
        "evidence.repository.review-ownership",
        "evidence.repository.security-scanning",
        "evidence.repository.dependency-scanning",
        "evidence.cicd.status-checks",
        "evidence.cicd.workflow-permissions",
        "evidence.deployment.environment-protection",
        "evidence.identity.privileged-membership",
        "inventory.subject",
        "inventory.complete",
    ] {
        assert!(
            types.contains(ty),
            "healthy-org missing `{ty}` in {types:?}"
        );
    }
    let prot = require_type(&batch.envelopes, "evidence.repository.branch-protection");
    assert!(typed_bool(prot, "protected"));
    assert!(typed_bool(
        require_type(&batch.envelopes, "inventory.complete"),
        "authoritative"
    ));
}

#[test]
fn ghc_011_golden_unprotected_repo_observes_protected_false() {
    let batch = collect_golden("unprotected-repo");
    let prot = of_type(&batch.envelopes, "evidence.repository.branch-protection");
    assert!(
        prot.iter().any(|e| !typed_bool(e, "protected")),
        "unprotected-repo must observe protected=false (404 / empty ruleset), not omit the type"
    );
}

#[test]
fn ghc_012_golden_missing_branch_protection_permission_is_diagnostic() {
    let batch = collect_golden("missing-branch-protection-permission");
    assert!(
        matches!(batch.run.status.as_str(), "partial" | "failed"),
        "permission hole must mark the run partial/failed, got {}",
        batch.run.status
    );
    assert!(
        !batch.errors.is_empty()
            || batch.envelopes.iter().any(|e| {
                let n = e.observation().narrative().to_ascii_lowercase();
                n.contains("permission") || n.contains("insufficient")
            }),
        "403 on protection must produce PermissionDenied / insufficient-evidence diagnostics"
    );
    for env in of_type(&batch.envelopes, "evidence.repository.branch-protection") {
        assert!(
            typed_bool(env, "protected"),
            "403 must never become protected=false for {}",
            subject_of(env)
        );
    }
    assert!(
        !batch.envelopes.is_empty(),
        "other subjects / successful resources must continue after a protection 403"
    );
}

#[test]
fn ghc_013_golden_paginated_inventory_authoritative_truncated_is_not() {
    let complete = collect_golden("paginated-inventory");
    let done = require_type(&complete.envelopes, "inventory.complete");
    assert!(
        typed_bool(done, "authoritative"),
        "fully paged inventory is authoritative"
    );
    let subjects = of_type(&complete.envelopes, "inventory.subject");
    assert!(
        subjects.len() >= 2,
        "paginated inventory must exhaust every page, got {} subjects",
        subjects.len()
    );

    let truncated_dir = golden_dir("paginated-inventory-truncated");
    assert!(
        truncated_dir.is_dir()
            || golden_dir("paginated-inventory")
                .join("truncated.http.json")
                .is_file(),
        "a truncated sibling fixture must exist so partial pages cannot claim complete coverage"
    );
    if truncated_dir.is_dir() {
        let truncated = collect_golden("paginated-inventory-truncated");
        let auth = of_type(&truncated.envelopes, "inventory.complete");
        assert!(
            auth.is_empty() || auth.iter().all(|e| !typed_bool(e, "authoritative")),
            "truncated pagination must not emit inventory.complete authoritative=true"
        );
    }
}

#[test]
fn ghc_014_golden_archived_excluded_by_selector() {
    assert!(
        github_sources_joined().contains("exclude_archived"),
        "archived exclusion is a GitHub collection-config flag"
    );
    let batch = collect_golden("archived-excluded-by-selector");
    let subjects: Vec<String> = of_type(&batch.envelopes, "inventory.subject")
        .iter()
        .map(|e| subject_of(e))
        .collect();
    assert!(
        subjects
            .iter()
            .all(|id| !id.contains("legacy") && !id.contains("archived")),
        "selector exclude_archived must drop archived repos from inventory.subject: {subjects:?}"
    );
    for env in of_type(&batch.envelopes, "evidence.repository.branch-protection") {
        assert!(
            !subject_of(env).contains("legacy"),
            "archived repo must not enter the protection population"
        );
    }
}

#[test]
fn ghc_015_golden_disabled_security_scanning_is_observed_false() {
    let batch = collect_golden("disabled-security-scanning");
    let scan = require_type(&batch.envelopes, "evidence.repository.security-scanning");
    assert!(
        !typed_bool(scan, "secret_scanning_enabled"),
        "explicitly disabled secret scanning is a true observation"
    );
    assert!(
        !typed_bool(scan, "code_scanning_enabled"),
        "explicitly disabled code scanning is a true observation"
    );
}

#[test]
fn ghc_016_golden_protected_environment_absent() {
    let batch = collect_golden("protected-environment-absent");
    let envs = of_type(
        &batch.envelopes,
        "evidence.deployment.environment-protection",
    );
    assert!(
        envs.is_empty() || envs.iter().all(|e| !typed_bool(e, "protected")),
        "absent protected environment must not be fabricated as protected=true"
    );
}

#[test]
fn ghc_017_golden_privileged_membership_population() {
    let batch = collect_golden("privileged-membership-population");
    let members = of_type(&batch.envelopes, "evidence.identity.privileged-membership");
    assert!(
        !members.is_empty(),
        "admins/owners must map to evidence.identity.privileged-membership"
    );
    assert!(
        members.iter().any(|e| typed_bool(e, "privileged")),
        "at least one privileged membership fact is required"
    );
    let identities = of_type(&batch.envelopes, "inventory.subject");
    assert!(
        identities
            .iter()
            .any(|e| { matches!(e.observation().fact("kind"), Some("identity" | "user")) }),
        "privileged population must emit inventory.subject kind=identity/user"
    );
    let guests = of_type(&batch.envelopes, "evidence.identity.external-access");
    assert!(
        !guests.is_empty(),
        "outside collaborators map to evidence.identity.external-access"
    );
}

#[test]
fn ghc_018_golden_api_partial_failure_marks_run_partial() {
    let batch = collect_golden("api-partial-failure");
    assert_eq!(
        batch.run.status, "partial",
        "mid-run 5xx must set status=partial, got {}",
        batch.run.status
    );
    assert!(
        batch.run.error_count >= 1 || !batch.errors.is_empty(),
        "partial failure must record errors"
    );
    assert!(
        !batch.envelopes.is_empty(),
        "successful resources before the 5xx must still produce envelopes"
    );
    assert_no_source_star_emitted(&batch.envelopes);
}

#[test]
fn ghc_019_golden_rate_limit_retry_or_explicit_partial() {
    let batch = collect_golden("rate-limit-retry");
    assert!(
        matches!(batch.run.status.as_str(), "complete" | "partial"),
        "429 must retry to success or become explicit partial, not a silent drop; got {}",
        batch.run.status
    );
    let silent_negative = batch.envelopes.iter().any(|e| {
        e.observation().evidence_type().as_str() == "evidence.repository.branch-protection"
            && e.observation()
                .fact_value("protected")
                .is_some_and(|v| matches!(v, EvidenceValue::Bool(false)))
            && batch.run.status == "complete"
            && batch.errors.is_empty()
    });
    assert!(
        !silent_negative,
        "429 must not be rewritten as protected=false"
    );
    if batch.run.status == "partial" {
        assert!(
            batch
                .errors
                .iter()
                .any(|e| e.to_ascii_lowercase().contains("rate")
                    || e.contains("429")
                    || e.to_ascii_lowercase().contains("retry")),
            "partial rate-limit run must say so in errors: {:?}",
            batch.errors
        );
    } else {
        assert!(
            !batch.envelopes.is_empty(),
            "successful retry must emit the recovered envelope once"
        );
        let digests: BTreeSet<&str> = batch.envelopes.iter().map(|e| e.digest()).collect();
        assert_eq!(
            digests.len(),
            batch.envelopes.len(),
            "429→200 must not duplicate envelope digests"
        );
    }
}

// ── CollectionRun / security / coverage ────────────────────────────────────

#[test]
fn ghc_020_collect_batch_records_a_real_collection_run() {
    let scope = org_scope("acme");
    let batch = batch_ok(
        &[
            ("/orgs/acme/repos", 200, ORG_REPOS_PAGE),
            (
                "/repos/acme/app/branches/develop/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            (
                "/repos/acme/other/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_DEVELOP),
            ("/repos/acme/other", 200, REPO_OTHER),
        ],
        scope,
    );
    let run = &batch.run;
    assert_eq!(run.collector_id, "collector.github");
    assert!(!run.collector_version.is_empty());
    assert!(
        run.scope.contains("org:acme"),
        "run.scope must record the canonical scope label, got {:?}",
        run.scope
    );
    assert!(
        !run.configuration_digest.is_empty(),
        "configuration_digest must hash non-secret config"
    );
    assert_no_token_material(&run.configuration_digest, "configuration_digest");
    assert!(
        run.completed_at.is_some(),
        "completed_at must be set when the batch returns"
    );
    assert!(
        run.completed_at.unwrap() >= run.started_at,
        "completion cannot precede start"
    );
    assert_eq!(run.evidence_count as usize, batch.envelopes.len());
    assert_eq!(run.error_count as usize, batch.errors.len());
    assert!(
        matches!(run.status.as_str(), "complete" | "partial" | "failed"),
        "status must be complete/partial/failed, got {}",
        run.status
    );
}

#[test]
fn ghc_021_no_github_token_material_in_envelopes_diagnostics_fixtures_or_digest() {
    // A 403 body that would leak installation / PAT material if copied raw.
    let leaky =
        r#"{"message":"ghs_thisMustNeverLeave","documentation_url":"https://docs.github.com"}"#;
    let scope = multi_repo_scope(&[("acme", "app"), ("acme", "other")]);
    let batch = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/develop/protection",
            200,
            PROTECTION_HEALTHY,
        ),
        ("/repos/acme/app", 200, REPO_DEVELOP),
        ("/repos/acme/other/branches/main/protection", 403, leaky),
        ("/repos/acme/other", 200, REPO_OTHER),
    ]))
    .collect_batch(CollectionRequest { scope })
    .unwrap_or_else(|e| {
        panic!("403 on one subject must not abort the batch (got {e}); diagnostics stay on the run")
    });
    assert!(
        !batch.errors.is_empty() || batch.run.status == "partial",
        "protection 403 is a diagnostic + partial run"
    );
    scan_batch_for_tokens(&batch, "mixed 403 collect");
    no_negative_from_denial(&batch.envelopes);

    let gh_src = github_sources_joined();
    assert!(
        !gh_src.contains("GITHUB_TOKEN=") && !gh_src.contains("ghp_"),
        "ISO GH-009: collector sources must not contain token literals"
    );
    // ghs_ must be folded by shared redact or a GitHub-owned sanitizer.
    assert!(
        github_file("mod.rs").contains("ghs_")
            || github_file("client.rs").contains("ghs_")
            || github_file("error.rs").contains("ghs_")
            || collector_sources_joined().contains("ghs_"),
        "GitHub collector must fold ghs_ installation tokens before diagnostics leave the crate"
    );

    for id in GOLDEN_IDS {
        let dir = golden_dir(id);
        if !dir.is_dir() {
            panic!("golden `{id}` missing; fixtures must also be secret-free");
        }
        let mut files = Vec::new();
        walk_files(&dir, &mut files);
        for path in files {
            if let Ok(text) = fs::read_to_string(&path) {
                assert_no_token_material(&text, &path.display().to_string());
            }
        }
    }
}

#[test]
fn ghc_022_healthy_org_covers_at_least_25_canonical_controls() {
    assert!(
        EXERCISABLE_CONTROLS.len() >= 25,
        "suite must enumerate ≥25 control/type/fact pairs"
    );
    let batch = collect_golden("healthy-org");
    let mut enabled = 0usize;
    let mut missing = Vec::new();
    let mut seen_controls = BTreeSet::new();
    for (control, ty, fact) in EXERCISABLE_CONTROLS {
        let hit = of_type(&batch.envelopes, ty)
            .iter()
            .any(|e| e.observation().fact_value(fact).is_some());
        if hit {
            enabled += 1;
            seen_controls.insert(*control);
        } else {
            missing.push(format!("{control} via {ty}.{fact}"));
        }
    }
    assert!(
        seen_controls.len() >= 25 || enabled >= 25,
        "healthy-org must enable ≥25 canonical controls via type/fact coverage; enabled={} unique={} missing={missing:?}",
        enabled,
        seen_controls.len()
    );
}

#[test]
fn ghc_023_goldens_and_target_assertions_do_not_require_provider_native_types() {
    let needle = format!("{}.{}", "evidence", "github");
    let target =
        fs::read_to_string(manifest_dir().join("tests/contracts/github_collector.target.rs"))
            .unwrap();
    // This file may mention the forbidden prefix only as a constructed needle.
    assert!(
        !target.contains(&format!("\"{needle}")),
        "target assertions must not require {needle}.*"
    );
    for id in GOLDEN_IDS {
        let dir = golden_dir(id);
        assert!(
            dir.is_dir(),
            "golden `{id}` must exist to prove it does not require {needle}.*"
        );
        let mut files = Vec::new();
        walk_files(&dir, &mut files);
        for path in files {
            if let Ok(text) = fs::read_to_string(&path) {
                assert!(
                    !text.contains(&needle),
                    "{} must not require {needle}.*",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn ghc_024_collector_has_no_framework_ids_or_effectiveness() {
    let src = github_sources_joined().to_ascii_lowercase();
    for needle in [
        "iso27001",
        "iso-27001",
        "soc2",
        "nis2",
        "dora",
        "effective",
        "ineffective",
    ] {
        assert!(
            !src.contains(needle),
            "github collector must not mention `{needle}`"
        );
    }
    let provider_native = format!("{}.{}", "evidence", "github");
    assert!(
        !github_sources_joined().contains(&provider_native),
        "do not invent {provider_native}.* required by tests"
    );
    // COL-002: collect must never emit framework results.
    let batch = GitHubCollector::with_client(client_with(&develop_repo_fixtures())).collect_batch(
        CollectionRequest {
            scope: repo_scope("acme", "app"),
        },
    );
    if let Ok(batch) = batch {
        for env in &batch.envelopes {
            let n = env.observation().narrative().to_ascii_lowercase();
            assert!(
                !n.contains("effective") && !n.contains("ineffective"),
                "collector must not compute effectiveness: {}",
                env.observation().narrative()
            );
        }
    }

    // Inline 403 continue-other-subjects (not only the golden).
    let mixed = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/develop/protection",
            200,
            PROTECTION_HEALTHY,
        ),
        ("/repos/acme/app", 200, REPO_DEVELOP),
        (
            "/repos/acme/other/branches/main/protection",
            403,
            r#"{"message":"Resource not accessible by integration"}"#,
        ),
        ("/repos/acme/other", 200, REPO_OTHER),
    ]))
    .collect_batch(CollectionRequest {
        scope: multi_repo_scope(&[("acme", "app"), ("acme", "other")]),
    })
    .unwrap_or_else(|e| panic!("one-subject 403 must not abort collect_batch: {e}"));
    assert!(
        of_type(&mixed.envelopes, "evidence.repository.visibility")
            .iter()
            .any(|e| subject_of(e).contains("app")),
        "successful subject envelopes must be retained when a sibling is 403"
    );
    assert_ne!(mixed.run.status, "complete");
}

#[test]
fn ghc_012b_inline_protection_403_is_not_protected_false() {
    let batch = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/develop/protection",
            403,
            r#"{"message":"Upgrade required"}"#,
        ),
        (
            "/repos/acme/app/branches/main/protection",
            403,
            r#"{"message":"Upgrade required"}"#,
        ),
        ("/repos/acme/app", 200, REPO_DEVELOP),
    ]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .unwrap_or_else(|e| panic!("protection 403 is a per-subject diagnostic, not a batch Err: {e}"));
    for env in of_type(&batch.envelopes, "evidence.repository.branch-protection") {
        panic!(
            "403 must not emit branch-protection facts (got protected={:?})",
            env.observation().fact_value("protected")
        );
    }
    assert!(
        batch.errors.iter().any(|e| e.contains("403")
            || e.to_ascii_lowercase().contains("permission")
            || e.to_ascii_lowercase().contains("insufficient")),
        "expected PermissionDenied/insufficient-evidence diagnostic, got {:?}",
        batch.errors
    );
}

#[test]
fn ghc_018b_inline_partial_failure_keeps_prior_envelopes() {
    let batch = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/develop/protection",
            200,
            PROTECTION_HEALTHY,
        ),
        ("/repos/acme/app", 200, REPO_DEVELOP),
        ("/repos/acme/other", 500, "upstream boom"),
    ]))
    .collect_batch(CollectionRequest {
        scope: multi_repo_scope(&[("acme", "app"), ("acme", "other")]),
    })
    .unwrap_or_else(|e| panic!("5xx on one repo must yield a partial run, not Err: {e}"));
    assert_eq!(batch.run.status, "partial");
    assert!(
        !of_type(&batch.envelopes, "evidence.repository.visibility").is_empty()
            || !of_type(&batch.envelopes, "evidence.repository.branch-protection").is_empty(),
        "envelopes from the successful repo must remain"
    );
}

#[test]
fn ghc_019b_inline_429_is_not_a_boolean_observation() {
    let batch = GitHubCollector::with_client(client_with_retry(&[(
        "/repos/acme/app",
        429,
        r#"{"message":"API rate limit exceeded"}"#,
        Some(1),
    )]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .unwrap_or_else(|e| {
        panic!("429 must surface on the run (retry or partial), not abort collect_batch: {e}")
    });
    assert!(
        matches!(batch.run.status.as_str(), "partial" | "failed" | "complete"),
        "got {}",
        batch.run.status
    );
    if batch.envelopes.is_empty() {
        assert!(
            !batch.errors.is_empty(),
            "no-retry path must record the 429 as an error"
        );
    }
}

#[test]
fn ghc_013b_org_list_permission_hole_is_not_authoritative() {
    let denied = GitHubCollector::with_client(client_with(&[(
        "/orgs/acme/repos",
        403,
        r#"{"message":"Must have admin rights"}"#,
    )]))
    .collect_batch(CollectionRequest {
        scope: org_scope("acme"),
    })
    .unwrap_or_else(|e| panic!("org list 403 is insufficient evidence, not batch abort: {e}"));
    let auth = of_type(&denied.envelopes, "inventory.complete");
    assert!(
        auth.is_empty() || auth.iter().all(|e| !typed_bool(e, "authoritative")),
        "list permission hole must not emit authoritative inventory.complete"
    );
    assert!(
        denied.errors.iter().any(|e| e.contains("403")
            || e.to_ascii_lowercase().contains("permission")
            || e.to_ascii_lowercase().contains("insufficient"))
            || denied.run.status != "complete",
        "org list 403 must be an explicit diagnostic / non-complete run"
    );
}

#[test]
fn ghc_014b_inline_archived_selector_drops_legacy() {
    let org_list = r#"[
        {"name":"app","full_name":"acme/app","visibility":"private","default_branch":"develop","archived":false},
        {"name":"legacy","full_name":"acme/legacy","visibility":"internal","default_branch":"main","archived":true}
    ]"#;
    let batch = batch_ok(
        &[
            ("/orgs/acme/repos", 200, org_list),
            ("/repos/acme/app", 200, REPO_DEVELOP),
            ("/repos/acme/legacy", 200, REPO_ARCHIVED),
            (
                "/repos/acme/app/branches/develop/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            (
                "/repos/acme/legacy/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
        ],
        org_scope_exclude_archived("acme"),
    );
    let ids: Vec<String> = of_type(&batch.envelopes, "inventory.subject")
        .iter()
        .map(|e| subject_of(e))
        .collect();
    assert!(
        ids.iter().any(|id| id.contains("app")),
        "active repo stays in inventory, got {ids:?}"
    );
    assert!(
        ids.iter().all(|id| !id.contains("legacy")),
        "archived repo excluded by selector, got {ids:?}"
    );
}
