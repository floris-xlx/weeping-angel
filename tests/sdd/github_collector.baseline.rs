//! Baseline suite for the GitHub collector (Prompt 09).
//!
//! Encodes CURRENT ISO-sliver behavior on characterization SHA
//! `e430980c0d27a8138a153d49b62ddf3c57827891` as specified in
//! `docs/sdd/github-collector.md` §3. Must stay GREEN until the target
//! suite is GREEN and this file is superseded. Does not implement the
//! reference-grade collector.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use weeping_angel_assurance_ir::AssetId;
use weeping_angel_collector::github::{GITHUB_EVIDENCE_TYPES, GitHubClient, GitHubCollector};
use weeping_angel_collector::{
    CollectionRequest, CollectorError, CollectorScope, EvidenceCollector,
};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceValue, redact};

const ADVERTISED_UNCOLLECTED: &[&str] = &[
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

const STUB_MODULES: &[(&str, &str)] = &[
    ("branches.rs", "branches"),
    ("collaborators.rs", "collaborators"),
    ("repositories.rs", "repositories"),
    ("rulesets.rs", "rulesets"),
    ("security.rs", "security"),
    ("workflows.rs", "workflows"),
];

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

fn github_src() -> PathBuf {
    crate_src("weeping-angel-collector").join("github")
}

fn github_file(rel: &str) -> String {
    fs::read_to_string(github_src().join(rel)).unwrap_or_else(|e| {
        panic!("read github/{rel}: {e}");
    })
}

fn github_sources_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&github_src(), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collector_sources_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src("weeping-angel-collector"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
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

fn client_with(fixtures: &[(&str, u16, &str)]) -> GitHubClient {
    let mut client = GitHubClient::new(Some("test-token".into()));
    for (path, status, body) in fixtures {
        client = client.with_fixture(path, *status, body, None);
    }
    client
}

fn collect_ok(fixtures: &[(&str, u16, &str)], scope: &CollectorScope) -> Vec<EvidenceEnvelope> {
    GitHubCollector::with_client(client_with(fixtures))
        .collect(scope)
        .expect("collect should succeed on this fixture set")
}

fn types_of(envelopes: &[EvidenceEnvelope]) -> BTreeSet<String> {
    envelopes
        .iter()
        .map(|e| e.observation().evidence_type().as_str().to_string())
        .collect()
}

fn fact_of<'a>(envelopes: &'a [EvidenceEnvelope], ty: &str, key: &str) -> &'a str {
    envelopes
        .iter()
        .find(|e| e.observation().evidence_type().as_str() == ty)
        .and_then(|e| e.observation().fact(key))
        .unwrap_or_else(|| panic!("missing {ty}.{key}"))
}

fn all_facts_are_strings(envelopes: &[EvidenceEnvelope]) {
    for env in envelopes {
        for (key, value) in env.observation().facts() {
            assert!(
                matches!(value, EvidenceValue::String(_)),
                "{} fact `{key}` is {value:?}, not a string (current with_fact path)",
                env.observation().evidence_type()
            );
        }
    }
}

const REPO_PRIVATE_DEVELOP: &str = r#"{
    "name": "app",
    "visibility": "private",
    "default_branch": "develop",
    "archived": false
}"#;

const REPO_ARCHIVED: &str = r#"{
    "name": "legacy",
    "visibility": "internal",
    "default_branch": "main",
    "archived": true
}"#;

const PROTECTION_HEALTHY: &str = r#"{
    "required_pull_request_reviews": { "required_approving_review_count": 2 },
    "allow_force_pushes": { "enabled": false },
    "allow_deletions": { "enabled": true },
    "required_status_checks": { "strict": true }
}"#;

// ── Scope ──────────────────────────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b001_collects_only_repo_owner_name_labels() {
    let collector = GitHubCollector::new(None);
    let err = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("org:acme")))
        .expect_err("org selector is out of scope today");
    assert!(
        matches!(err, CollectorError::OutOfScope { ref asset } if asset == "org:acme"),
        "got {err:?}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b002_bare_owner_name_label_rebuilds_repo_prefixed_asset_and_misses_scope() {
    let scope = CollectorScope::new().allow_asset(AssetId::new("acme/app"));
    let err = GitHubCollector::new(None)
        .collect(&scope)
        .expect_err("label acme/app is rewritten to repo:acme/app");
    assert!(
        matches!(err, CollectorError::OutOfScope { ref asset } if asset == "repo:acme/app"),
        "got {err:?}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b003_no_org_inventory_or_archived_selector_in_sources() {
    let src = github_sources_joined();
    for needle in [
        "/orgs/",
        "exclude archived",
        "exclude_archived",
        "inventory.subject",
        "inventory.complete",
    ] {
        assert!(
            !src.contains(needle),
            "current collector has no org inventory / archived selector; found `{needle}`"
        );
    }
}

// ── Emitted source.* facts ─────────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b004_normalize_emits_four_repository_string_facts() {
    let scope = repo_scope("acme", "app");
    // More-specific protection fixture first so prefix-match does not steal it.
    let envelopes = collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                404,
                r#"{"message":"Not Found"}"#,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&envelopes, "source.repository.exists", "exists"),
        "true"
    );
    assert_eq!(
        fact_of(&envelopes, "source.repository.visibility", "visibility"),
        "private"
    );
    assert_eq!(
        fact_of(&envelopes, "source.default_branch", "name"),
        "develop"
    );
    assert_eq!(
        fact_of(&envelopes, "source.repository.archived", "archived"),
        "false"
    );
    all_facts_are_strings(&envelopes);
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b005_visibility_falls_back_to_private_flag_then_unknown() {
    let scope = repo_scope("acme", "app");
    let via_private = collect_ok(
        &[
            ("/repos/acme/app/branches/main/protection", 404, "{}"),
            ("/repos/acme/app", 200, r#"{"private":true}"#),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&via_private, "source.repository.visibility", "visibility"),
        "private"
    );
    assert!(
        via_private
            .iter()
            .all(|e| e.observation().evidence_type().as_str() != "source.default_branch"),
        "missing default_branch must omit source.default_branch"
    );

    let via_public = collect_ok(
        &[
            ("/repos/acme/app/branches/main/protection", 404, "{}"),
            ("/repos/acme/app", 200, r#"{"private":false}"#),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&via_public, "source.repository.visibility", "visibility"),
        "public"
    );

    let unknown = collect_ok(
        &[
            ("/repos/acme/app/branches/main/protection", 404, "{}"),
            ("/repos/acme/app", 200, r#"{}"#),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&unknown, "source.repository.visibility", "visibility"),
        "unknown"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b006_archived_repo_is_collected_like_any_other() {
    let scope = repo_scope("acme", "legacy");
    let envelopes = collect_ok(
        &[
            ("/repos/acme/legacy/branches/main/protection", 404, "{}"),
            ("/repos/acme/legacy", 200, REPO_ARCHIVED),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&envelopes, "source.repository.archived", "archived"),
        "true"
    );
    assert_eq!(
        fact_of(&envelopes, "source.repository.visibility", "visibility"),
        "internal"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b007_protection_hardcodes_main_not_default_branch() {
    let src = github_file("mod.rs");
    assert!(
        src.contains("/repos/{owner}/{name}/branches/{}/protection"),
        "protection path template missing"
    );
    assert!(
        src.contains("\"main\""),
        "protection path must hardcode main today"
    );

    let scope = repo_scope("acme", "app");
    let envelopes = collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            (
                "/repos/acme/app/branches/develop/protection",
                404,
                r#"{"message":"not used"}"#,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&envelopes, "source.branch.protection", "enabled"),
        "true"
    );
    assert_eq!(
        fact_of(&envelopes, "source.branch.required_reviews", "count"),
        "2"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b008_protection_404_is_enabled_false_not_a_diagnostic() {
    let scope = repo_scope("acme", "app");
    let envelopes = collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                404,
                r#"{"message":"Not Found"}"#,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    let prot = envelopes
        .iter()
        .find(|e| e.observation().evidence_type().as_str() == "source.branch.protection")
        .expect("404 protection still emits source.branch.protection");
    assert_eq!(prot.observation().fact("enabled"), Some("false"));
    assert_eq!(
        prot.observation().narrative(),
        "default branch has no protection rule"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b009_protection_200_inverts_force_push_and_deletion() {
    let scope = repo_scope("acme", "app");
    let envelopes = collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&envelopes, "source.branch.force_push_protection", "enabled"),
        "true"
    );
    assert_eq!(
        fact_of(&envelopes, "source.branch.deletion_protection", "enabled"),
        "false"
    );
    assert_eq!(
        fact_of(
            &envelopes,
            "source.branch.required_status_checks",
            "configured"
        ),
        "true"
    );
    all_facts_are_strings(&envelopes);

    let empty_prot = collect_ok(
        &[
            ("/repos/acme/app/branches/main/protection", 200, "{}"),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    assert_eq!(
        fact_of(&empty_prot, "source.branch.required_reviews", "count"),
        "0"
    );
    assert_eq!(
        fact_of(
            &empty_prot,
            "source.branch.force_push_protection",
            "enabled"
        ),
        "true"
    );
    assert_eq!(
        fact_of(&empty_prot, "source.branch.deletion_protection", "enabled"),
        "true"
    );
    assert_eq!(
        fact_of(
            &empty_prot,
            "source.branch.required_status_checks",
            "configured"
        ),
        "false"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b010_emitted_types_are_source_star_only() {
    let scope = repo_scope("acme", "app");
    let envelopes = collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    );
    let types = types_of(&envelopes);
    for ty in &types {
        assert!(
            ty.starts_with("source."),
            "current emit is source.* only, got {ty}"
        );
        assert!(
            !ty.starts_with("evidence."),
            "canonical evidence types are not emitted today: {ty}"
        );
    }
    assert!(types.contains("source.repository.exists"));
    assert!(types.contains("source.branch.protection"));
    assert!(!types.contains("evidence.repository.branch-protection"));
    assert!(!types.contains("evidence.identity.privileged-membership"));
}

// ── Stubs ──────────────────────────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b011_feature_modules_are_module_const_stubs() {
    for (file, module) in STUB_MODULES {
        let text = github_file(file);
        let trimmed = text.trim();
        assert_eq!(
            trimmed,
            format!("pub const MODULE: &str = \"{module}\";"),
            "{file} must remain a MODULE stub today"
        );
    }
    let collect = github_file("mod.rs");
    for (_, module) in STUB_MODULES {
        assert!(
            collect.contains(&format!("{module}::MODULE")),
            "collect_repo must mention {module}::MODULE so the stub is kept alive"
        );
    }
}

// ── Descriptor advertised-vs-collected gap ─────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b012_descriptor_advertises_types_and_pagination_it_does_not_collect() {
    let collector = GitHubCollector::new(None);
    let desc = collector.descriptor();
    assert_eq!(desc.id, "collector.github");
    assert_eq!(desc.provider_family, "source-control");
    assert_eq!(
        desc.subject_types,
        BTreeSet::from(["repository".into(), "branch".into()])
    );
    assert!(
        desc.capabilities.pagination,
        "descriptor lies: pagination=true"
    );
    assert!(desc.capabilities.point_in_time);
    assert!(desc.capabilities.worker_safe);
    assert!(
        !desc.capabilities.incremental,
        "incremental stays at Default::false"
    );
    assert_eq!(
        desc.required_permissions,
        vec![
            "contents:read".to_string(),
            "administration:read".to_string(),
            "metadata:read".to_string(),
        ]
    );

    let advertised: BTreeSet<&str> = GITHUB_EVIDENCE_TYPES.iter().copied().collect();
    assert_eq!(advertised.len(), 19);
    for ty in ADVERTISED_UNCOLLECTED {
        assert!(
            advertised.contains(ty),
            "descriptor still advertises uncollected `{ty}`"
        );
    }
    assert!(
        GITHUB_EVIDENCE_TYPES
            .iter()
            .all(|t| !t.starts_with("evidence.identity.")),
        "IAM-015: GITHUB_EVIDENCE_TYPES must stay free of evidence.identity.*"
    );

    let scope = repo_scope("acme", "app");
    let collected = types_of(&collect_ok(
        &[
            (
                "/repos/acme/app/branches/main/protection",
                200,
                PROTECTION_HEALTHY,
            ),
            ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        ],
        &scope,
    ));
    for ty in ADVERTISED_UNCOLLECTED {
        assert!(
            !collected.contains(*ty),
            "advertised `{ty}` must not appear in a healthy collect today"
        );
    }

    let client_src = github_file("client.rs");
    assert!(
        !client_src.contains("per_page")
            && !client_src.contains("Link")
            && !client_src.contains("page="),
        "client advertises pagination but has no page walker"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b013_descriptor_has_no_structured_failure_behavior() {
    let desc = GitHubCollector::new(None).descriptor();
    let json = serde_json::to_value(&desc).expect("descriptor serializes");
    assert!(
        json.get("failureBehavior").is_none() && json.get("failure_behavior").is_none(),
        "current descriptor has no failure-behavior field: {json}"
    );
}

// ── Permission / error abort ───────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b014_repo_403_aborts_collect_as_permission_denied() {
    let scope = repo_scope("acme", "app");
    let err = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        403,
        r#"{"message":"Resource not accessible"}"#,
    )]))
    .collect(&scope)
    .expect_err("403 on repo aborts");
    match err {
        CollectorError::PermissionDenied { detail } => {
            assert!(detail.contains("403 on repository"), "detail={detail}");
            assert!(
                !detail.to_ascii_lowercase().contains("enabled\": \"false\"")
                    && !detail.contains("protected=false"),
                "403 must not be a false observation: {detail}"
            );
        }
        other => panic!("expected PermissionDenied, got {other:?}"),
    }
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b015_protection_403_aborts_whole_collect() {
    let scope = repo_scope("acme", "app");
    let err = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/main/protection",
            403,
            r#"{"message":"Upgrade required"}"#,
        ),
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
    ]))
    .collect(&scope)
    .expect_err("403 on protection aborts even after repo 200");
    assert!(
        matches!(
            err,
            CollectorError::PermissionDenied { ref detail }
                if detail.contains("403 reading branch protection")
        ),
        "got {err:?}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b016_403_on_second_repo_discards_first_repo_envelopes() {
    let scope = multi_repo_scope(&[("acme", "app"), ("acme", "other")]);
    let err = GitHubCollector::with_client(client_with(&[
        ("/repos/acme/app/branches/main/protection", 404, "{}"),
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        (
            "/repos/acme/other",
            403,
            r#"{"message":"Resource not accessible"}"#,
        ),
    ]))
    .collect(&scope)
    .expect_err("second-repo 403 aborts the batch");
    assert!(matches!(err, CollectorError::PermissionDenied { .. }));
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b017_repo_404_and_other_errors_are_insufficient_evidence_and_abort() {
    let scope = repo_scope("acme", "app");
    let not_found = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        404,
        r#"{"message":"Not Found"}"#,
    )]))
    .collect(&scope)
    .expect_err("repo 404");
    assert!(
        matches!(
            not_found,
            CollectorError::InsufficientEvidence { ref detail }
                if detail.contains("resource not visible")
        ),
        "got {not_found:?}"
    );

    let rate = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        429,
        r#"{"message":"rate limit"}"#,
    )]))
    .collect(&scope)
    .expect_err("repo 429");
    assert!(
        matches!(
            rate,
            CollectorError::InsufficientEvidence { ref detail }
                if detail.contains("rate limited")
        ),
        "got {rate:?}"
    );

    let boom = GitHubCollector::with_client(client_with(&[("/repos/acme/app", 500, "oops")]))
        .collect(&scope)
        .expect_err("repo 500");
    assert!(
        matches!(
            boom,
            CollectorError::InsufficientEvidence { ref detail }
                if detail.contains("unexpected status 500")
        ),
        "got {boom:?}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b018_collect_does_not_retry_429() {
    let src = github_file("mod.rs");
    assert!(
        src.contains("pub fn backoff(") && src.contains("pub fn sleep_retry_after("),
        "unused retry helpers exist on GitHubCollector today"
    );
    assert!(
        !src.contains("Self::backoff")
            && !src.contains("self.backoff")
            && !src.contains("GitHubCollector::backoff")
            && !src.contains("Self::sleep_retry_after")
            && !src.contains("self.sleep_retry_after"),
        "collect must not invoke retry helpers today"
    );
    assert_eq!(GitHubCollector::backoff(0, None), Duration::from_secs(1));
    assert_eq!(GitHubCollector::backoff(1, None), Duration::from_secs(2));
    assert_eq!(GitHubCollector::backoff(5, None), Duration::from_secs(32));
    assert_eq!(GitHubCollector::backoff(9, None), Duration::from_secs(32));
    assert_eq!(
        GitHubCollector::backoff(0, Some(Duration::from_secs(7))),
        Duration::from_secs(7)
    );
    GitHubCollector::sleep_retry_after(Duration::from_millis(0));
}

// ── Client / transport ─────────────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b019_client_is_fixture_only() {
    let bare = GitHubClient::new(None);
    let (status, body, retry) = bare
        .get("/repos/acme/app")
        .expect("401 is Ok, not Transport");
    assert_eq!(status, 401);
    assert!(body.contains("requires Authorization"));
    assert!(retry.is_none());

    let authed = GitHubClient::new(Some("test-token".into()));
    let err = authed
        .get("/repos/acme/app")
        .expect_err("no fixture + token => Transport");
    let msg = err.to_string();
    assert!(
        msg.contains("no fixture and no live transport for /repos/acme/app"),
        "{msg}"
    );

    let toml = fs::read_to_string(manifest_dir().join("crates/weeping-angel-collector/Cargo.toml"))
        .unwrap();
    assert!(
        !toml.contains("reqwest") && !toml.contains("octocrab"),
        "collector crate has no live HTTP dependency today"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b020_fixture_prefix_match_first_wins() {
    let client = client_with(&[
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
        (
            "/repos/acme/app/branches/main/protection",
            404,
            r#"{"message":"never reached"}"#,
        ),
    ]);
    let (status, body, _) = client
        .get("/repos/acme/app/branches/main/protection")
        .unwrap();
    assert_eq!(status, 200);
    assert!(
        body.contains("default_branch"),
        "repo fixture prefixes the protection path; first match wins: {body}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b021_authorization_header_never_returns_the_token() {
    let client = GitHubClient::new(Some("super-secret-token".into()));
    assert_eq!(
        client.authorization_header().as_deref(),
        Some("Bearer [redacted]")
    );
    assert!(GitHubClient::new(None).authorization_header().is_none());
}

// ── CollectionRun ──────────────────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b022_collect_batch_wraps_empty_collection_run() {
    let scope = repo_scope("acme", "app");
    let collector = GitHubCollector::with_client(client_with(&[
        ("/repos/acme/app/branches/main/protection", 404, "{}"),
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
    ]));
    let batch = collector
        .collect_batch(CollectionRequest { scope })
        .expect("batch succeeds when collect succeeds");
    assert!(batch.errors.is_empty());
    assert!(!batch.envelopes.is_empty());
    let run = &batch.run;
    assert_eq!(run.collector_id, "collector.github");
    assert!(!run.collector_version.is_empty());
    assert!(run.completed_at.is_none());
    assert_eq!(run.scope, "");
    assert_eq!(run.status, "started");
    assert_eq!(run.evidence_count, 0);
    assert_eq!(run.error_count, 0);
    assert_eq!(run.configuration_digest, "");

    let env_run = batch.envelopes[0].collection_run_id();
    assert!(
        env_run.starts_with("run:"),
        "envelope run id is a provenance digest, got {env_run}"
    );
    assert_ne!(
        env_run, run.run_id,
        "envelope collection_run_id is not the batch run_id today"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b023_collect_batch_propagates_collect_errors_without_a_run() {
    let err = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        403,
        r#"{"message":"no"}"#,
    )]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .expect_err("batch does not swallow collect errors");
    assert!(matches!(err, CollectorError::PermissionDenied { .. }));
}

// ── Security already present ───────────────────────────────────────────────

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b024_redact_covers_existing_github_token_needles() {
    assert_eq!(
        redact("Authorization: Bearer secret-value more"),
        "Authorization: Bearer [redacted] more"
    );
    assert_eq!(redact("token=abc123"), "token=[redacted]");
    assert_eq!(redact("ghp_liveexample"), "ghp_[redacted]");
    assert_eq!(redact("gho_oauthexample"), "gho_[redacted]");
    assert_eq!(redact("github_pat_finegrained"), "github_pat_[redacted]");
    assert_eq!(
        redact("ghs_installation"),
        "ghs_installation",
        "ghs_ is not in the current redact needle list"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b025_collect_403_uses_fixed_permission_strings_not_response_bodies() {
    let scope = repo_scope("acme", "app");
    let repo_err = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        403,
        r#"{"message":"token=should-not-leak"}"#,
    )]))
    .collect(&scope)
    .expect_err("repo 403");
    match repo_err {
        CollectorError::PermissionDenied { detail } => {
            assert_eq!(detail, "403 on repository; InsufficientEvidence, not false");
            assert!(
                !detail.contains("should-not-leak") && !detail.contains("token="),
                "repo 403 must not copy the response body: {detail}"
            );
        }
        other => panic!("expected PermissionDenied, got {other:?}"),
    }

    let prot_err = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/main/protection",
            403,
            r#"{"message":"token=should-not-leak"}"#,
        ),
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
    ]))
    .collect(&scope)
    .expect_err("protection 403");
    match prot_err {
        CollectorError::PermissionDenied { detail } => {
            assert_eq!(
                detail,
                "403 reading branch protection; InsufficientEvidence"
            );
            assert!(
                !detail.contains("should-not-leak"),
                "protection 403 must not copy the response body: {detail}"
            );
        }
        other => panic!("expected PermissionDenied, got {other:?}"),
    }

    let unauthorized = GitHubCollector::with_client(client_with(&[
        (
            "/repos/acme/app/branches/main/protection",
            401,
            r#"{"message":"token=should-not-leak"}"#,
        ),
        ("/repos/acme/app", 200, REPO_PRIVATE_DEVELOP),
    ]))
    .collect(&scope)
    .expect_err("protection 401 uses handle_status");
    match unauthorized {
        CollectorError::PermissionDenied { detail } => {
            assert_eq!(detail, "unauthorized; check Authorization header");
            assert!(
                !detail.contains("should-not-leak"),
                "401 handle_status uses a static redacted phrase: {detail}"
            );
        }
        other => panic!("expected PermissionDenied, got {other:?}"),
    }
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b026_iso_gh007_gh009_needles_remain() {
    let src = collector_sources_joined();
    assert!(src.contains("PermissionDenied"));
    assert!(src.contains("403"));
    assert!(src.contains("InsufficientEvidence"));
    assert!(
        !src.contains("403 => false") && !src.contains("status == 403 && enabled = false"),
        "GH-007: 403 must not normalize to boolean false"
    );
    assert!(src.contains("redact"));
    assert!(src.contains("Authorization"));
    assert!(
        !src.contains("GITHUB_TOKEN=") && !src.contains("ghp_"),
        "GH-009: collector sources must not contain token literals"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b027_github_module_has_no_framework_ids_or_effectiveness() {
    let src = github_sources_joined().to_ascii_lowercase();
    for needle in [
        "iso27001",
        "iso-27001",
        "soc2",
        "nis2",
        "dora",
        "effective",
        "ineffective",
        "evidence.github.",
    ] {
        assert!(
            !src.contains(needle),
            "github collector must not mention `{needle}` today"
        );
    }
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b028_no_canonical_github_goldens_or_sdlc_catalog_rows() {
    let root = manifest_dir();
    assert!(
        !root.join("fixtures/assurance/canonical/v1/github").is_dir(),
        "github adapter goldens do not exist yet"
    );
    assert!(
        !root.join("fixtures/collectors/github").is_dir(),
        "collector github fixtures do not exist yet"
    );
    assert!(
        !root
            .join("catalog/canonical/v1/evidence/repository.toml")
            .is_file(),
        "Prompt 05 repository evidence catalog is not landed"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b029_no_token_without_fixture_is_permission_denied() {
    let err = GitHubCollector::new(None)
        .collect(&repo_scope("acme", "app"))
        .expect_err("no token and no fixture");
    assert!(
        matches!(
            err,
            CollectorError::PermissionDenied { ref detail }
                if detail.contains("unauthorized")
        ),
        "got {err:?}"
    );
}

#[ignore = "superseded by sdd_github_collector_target"]
#[test]
fn ghc_b030_token_without_fixture_is_insufficient_evidence_transport() {
    let err = GitHubCollector::with_client(GitHubClient::new(Some("test-token".into())))
        .collect(&repo_scope("acme", "app"))
        .expect_err("token + no fixture");
    match err {
        CollectorError::InsufficientEvidence { detail } => {
            assert!(
                detail.contains("no fixture and no live transport"),
                "{detail}"
            );
            assert!(
                !detail.contains("test-token"),
                "token leaked in transport diagnostic: {detail}"
            );
        }
        other => panic!("expected InsufficientEvidence, got {other:?}"),
    }
}
