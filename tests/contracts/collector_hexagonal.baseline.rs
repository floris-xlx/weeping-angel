//! Baseline suite for collector hexagonal increment 1.
//!
//! Encodes CURRENT monolith behavior on characterization SHA
//! `0015f6395e7ead042e3cfd3066fefde3d39aa36b` as specified in
//! `docs/specs/collector-hexagonal.md` §3. Must stay GREEN until the
//! target suite is GREEN and this file is superseded. Does not implement
//! hexagonal modules, CollectionEngine, or observation-only adapters.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use weeping_angel_assurance_ir::AssetId;
use weeping_angel_collector::github::{GITHUB_EVIDENCE_TYPES, GitHubClient, GitHubCollector};
use weeping_angel_collector::{
    CollectionRequest, CollectorCapabilities, CollectorDescriptor, CollectorScope,
    EvidenceCollector, FixtureCollector, LocalCollector,
};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceObservation, EvidenceType};

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

fn crate_root() -> PathBuf {
    manifest_dir()
        .join("crates")
        .join("weeping-angel-collector")
}

fn crate_src() -> PathBuf {
    let path = crate_root().join("src");
    assert!(
        path.is_dir(),
        "expected collector sources at {}",
        path.display()
    );
    path
}

fn github_src() -> PathBuf {
    crate_src().join("github")
}

fn read_rel(rel: &str) -> String {
    fs::read_to_string(crate_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn collector_sources_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
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

fn workspace_toml() -> String {
    fs::read_to_string(manifest_dir().join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("read workspace Cargo.toml: {e}"))
}

fn repo_scope(owner: &str, name: &str) -> CollectorScope {
    CollectorScope::new().allow_asset(AssetId::new(format!("repo:{owner}/{name}")))
}

fn client_with(fixtures: &[(&str, u16, &str)]) -> GitHubClient {
    let mut client = GitHubClient::new(Some("test-token".into()));
    for (path, status, body) in fixtures {
        client = client.with_fixture(path, *status, body, None);
    }
    client
}

fn contains_ident(src: &str, ident: &str) -> bool {
    src.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|tok| tok == ident)
}

/// chx_b001 — still one Cargo package; no collector subcrates.
#[test]
fn chx_b001_single_cargo_package() {
    let pkg = read_rel("Cargo.toml");
    assert!(
        pkg.contains("name = \"weeping-angel-collector\""),
        "collector crate package name must remain weeping-angel-collector"
    );
    assert!(
        pkg.contains("publish = false"),
        "collector crate stays unpublished"
    );
    let workspace = workspace_toml();
    assert!(
        workspace.contains("\"crates/weeping-angel-collector\""),
        "workspace must list the single collector package"
    );
    assert!(
        !workspace.contains("weeping-angel-collector-"),
        "current tree has no collector subcrate members"
    );
}

/// chx_b002 — domain types are defined in src/lib.rs, not extracted.
#[test]
fn chx_b002_lib_rs_defines_monolith_types() {
    let lib = read_rel("src/lib.rs");
    for needle in [
        "pub struct CollectorCapabilities",
        "pub struct CollectorDescriptor",
        "pub struct CollectorScope",
        "pub struct CollectionRequest",
        "pub struct CollectionBatch",
        "pub trait EvidenceCollector",
        "pub struct FixtureCollector",
        "pub enum CollectorError",
    ] {
        assert!(
            lib.contains(needle),
            "current monolith defines `{needle}` inline in src/lib.rs"
        );
    }
    assert!(
        !lib.contains("pub mod domain"),
        "lib.rs is not a hexagonal facade today (no `pub mod domain`)"
    );
    assert!(
        !lib.contains("mod application"),
        "lib.rs does not declare an application layer today"
    );
}

/// chx_b003 — CollectorCapabilities has the eight bool fields (found case).
#[test]
fn chx_b003_capabilities_eight_bool_fields() {
    let caps = CollectorCapabilities {
        incremental: false,
        pagination: false,
        historical: false,
        point_in_time: false,
        event_driven: false,
        sensitive_artifacts: false,
        offline: true,
        worker_safe: true,
    };
    assert!(!caps.incremental);
    assert!(!caps.pagination);
    assert!(!caps.historical);
    assert!(!caps.point_in_time);
    assert!(!caps.event_driven);
    assert!(!caps.sensitive_artifacts);
    assert!(caps.offline);
    assert!(caps.worker_safe);

    let lib = read_rel("src/lib.rs");
    for field in [
        "pub incremental: bool",
        "pub pagination: bool",
        "pub historical: bool",
        "pub point_in_time: bool",
        "pub event_driven: bool",
        "pub sensitive_artifacts: bool",
        "pub offline: bool",
        "pub worker_safe: bool",
    ] {
        assert!(
            lib.contains(field),
            "CollectorCapabilities field `{field}` lives in src/lib.rs today"
        );
    }
}

/// chx_b004 — descriptor / scope / request / batch public shapes (found case).
#[test]
fn chx_b004_descriptor_scope_request_batch_shapes() {
    let desc = CollectorDescriptor {
        id: "collector.github".into(),
        version: "0".into(),
        evidence_types: BTreeSet::new(),
        provider_family: "github".into(),
        subject_types: BTreeSet::from(["repository".into()]),
        capabilities: CollectorCapabilities::default(),
        required_permissions: Vec::new(),
    };
    assert_eq!(desc.id, "collector.github");
    assert_eq!(desc.provider_family, "github");
    assert!(desc.subject_types.contains("repository"));

    let scope = CollectorScope::new().allow_asset(AssetId::new("repo:acme/app"));
    assert!(scope.allows(&AssetId::new("repo:acme/app")));
    assert_eq!(scope.as_label(), "repo:acme/app");

    let request = CollectionRequest {
        scope: scope.clone(),
    };
    assert_eq!(request.scope.as_label(), "repo:acme/app");

    let lib = read_rel("src/lib.rs");
    assert!(
        lib.contains("pub errors: Vec<String>"),
        "CollectionBatch.errors is Vec<String> today"
    );
    assert!(
        lib.contains("pub envelopes: Vec<EvidenceEnvelope>"),
        "CollectionBatch carries sealed envelopes today"
    );
}

/// chx_b005 — hexagonal directories do not exist on the current tree.
#[test]
fn chx_b005_no_hexagonal_directories() {
    let src = crate_src();
    for name in ["domain", "application", "ports", "adapters"] {
        let path = src.join(name);
        assert!(
            !path.exists(),
            "current monolith has no {} (found {})",
            path.display(),
            path.display()
        );
    }
    for file in [
        "domain/capabilities.rs",
        "domain/descriptor.rs",
        "domain/scope.rs",
        "domain/collector.rs",
        "domain/observation.rs",
        "domain/coverage.rs",
        "domain/diagnostic.rs",
        "domain/batch.rs",
        "domain/cursor.rs",
        "domain/instance.rs",
        "application/engine.rs",
        "application/registry.rs",
        "application/gate.rs",
        "application/envelope.rs",
        "ports/adapter.rs",
    ] {
        assert!(
            !src.join(file).exists(),
            "extracted hexagonal file must not exist yet: {file}"
        );
    }
}

/// chx_b006 — neighbor github_src() path stays on disk.
#[test]
fn chx_b006_github_src_on_disk() {
    assert!(
        github_src().is_dir(),
        "sdd_github_collector_target github_src() is src/github; keep on disk"
    );
    for rel in [
        "mod.rs",
        "normalize.rs",
        "client.rs",
        "descriptor.rs",
        "protection.rs",
    ] {
        assert!(
            github_src().join(rel).is_file(),
            "expected crates/weeping-angel-collector/src/github/{rel}"
        );
    }
    assert!(
        crate_src().join("local").join("mod.rs").is_file(),
        "local adapter lives at src/local/mod.rs"
    );
}

/// chx_b007 — adapters invent provenance and seal envelopes today.
#[test]
fn chx_b007_adapters_construct_provenance_and_seal() {
    let normalize = read_rel("src/github/normalize.rs");
    assert!(
        normalize.contains("EvidenceProvenance"),
        "github/normalize.rs constructs EvidenceProvenance today"
    );
    assert!(
        normalize.contains("EvidenceEnvelope::seal"),
        "github/normalize.rs calls EvidenceEnvelope::seal today"
    );
    assert!(
        normalize.contains("collector_id: \"collector.github\""),
        "GitHub emit hard-codes type id collector.github on provenance"
    );
    assert!(
        normalize.contains("pub fn emit") && normalize.contains("-> Result<EvidenceEnvelope"),
        "normalize::emit returns EvidenceEnvelope today, not ObservationCandidate"
    );

    let local = read_rel("src/local/mod.rs");
    assert!(
        local.contains("EvidenceProvenance") && local.contains("EvidenceEnvelope::seal"),
        "LocalCollector / ManualEvidence seal envelopes today"
    );
    assert!(
        local.contains("collector_id: \"collector.local\"")
            && local.contains("collector_id: \"collector.manual\""),
        "local/manual invent provenance collector_id today"
    );

    let lib = read_rel("src/lib.rs");
    assert!(
        lib.contains("let provenance = EvidenceProvenance")
            && lib.contains("EvidenceEnvelope::seal(observation.clone(), provenance"),
        "FixtureCollector constructs EvidenceProvenance and seals today"
    );
}

/// chx_b008 — type == instance; secrets sit on GitHubCollector / GitHubClient.
#[test]
fn chx_b008_token_on_collector_no_instance_or_credential_ref() {
    let _ = GitHubCollector::new(Some("characterization-token".into()));
    let _ = GitHubCollector::new(None);

    let src = collector_sources_joined();
    assert!(
        !contains_ident(&src, "CollectorInstance"),
        "CollectorInstance is absent on the current tree"
    );
    assert!(
        !contains_ident(&src, "CredentialRef"),
        "CredentialRef is absent on the current tree"
    );

    let github_mod = read_rel("src/github/mod.rs");
    assert!(
        github_mod.contains("pub fn new(token: Option<String>)"),
        "GitHubCollector::new takes a token today"
    );
    let client = read_rel("src/github/client.rs");
    assert!(
        client.contains("token: Option<String>"),
        "GitHubClient holds token material today"
    );
}

/// chx_b009 — application-layer types do not exist.
#[test]
fn chx_b009_application_layer_symbols_absent() {
    let src = collector_sources_joined();
    for ident in [
        "CollectionEngine",
        "CollectorRegistry",
        "ObservationGate",
        "EnvelopeFactory",
        "CollectorAdapter",
        "ObservationCandidate",
        "ObservationBatch",
    ] {
        assert!(
            !contains_ident(&src, ident),
            "`{ident}` must be absent on the current monolith"
        );
    }
}

/// chx_b010 — public facade names compile for scheduler/assurance consumers.
#[test]
fn chx_b010_public_facade_compiles() {
    fn assert_collector<C: EvidenceCollector>(c: &C) -> CollectorDescriptor {
        c.descriptor()
    }

    let github = GitHubCollector::new(None);
    let local = LocalCollector::new(".");
    let fixture = FixtureCollector::new("fixture.chx", "0");

    let g = assert_collector(&github);
    let l = assert_collector(&local);
    let f = assert_collector(&fixture);
    assert_eq!(g.id, "collector.github");
    assert_eq!(l.id, "collector.local");
    assert_eq!(f.id, "fixture.chx");
}

/// chx_b011 — 403 ≠ false (current GitHub behavior; do not weaken ghc_*).
#[test]
fn chx_b011_403_is_not_false() {
    let github = github_sources_joined();
    assert!(
        github.contains("401 | 403"),
        "GitHub fetch maps 401/403 to Denied today"
    );
    assert!(
        GITHUB_EVIDENCE_TYPES.contains(&"source.branch.protection"),
        "GITHUB_EVIDENCE_TYPES historical list is unchanged"
    );

    let repo = r#"{
        "name": "app",
        "full_name": "acme/app",
        "visibility": "private",
        "default_branch": "develop",
        "archived": false
    }"#;
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
        ("/repos/acme/app", 200, repo),
    ]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .expect("protection 403 is a diagnostic, not collect_batch Err");

    for env in &batch.envelopes {
        if env.observation().evidence_type().as_str() == "evidence.repository.branch-protection" {
            panic!(
                "403 must not emit branch-protection facts (got protected={:?})",
                env.observation().fact("protected")
            );
        }
    }
    assert!(
        batch.errors.iter().any(|e| e.contains("403")
            || e.to_ascii_lowercase().contains("permission")
            || e.to_ascii_lowercase().contains("insufficient")),
        "403 must surface as PermissionDenied / insufficient diagnostic, got {:?}",
        batch.errors
    );
}

/// chx_b012 — GitHub descriptor id is always the type; no instance id.
#[test]
fn chx_b012_github_id_is_type_not_instance() {
    let desc = GitHubCollector::new(None).descriptor();
    assert_eq!(desc.id, "collector.github");
    assert_ne!(desc.id, "github:xylex-group");
    assert!(
        !desc.id.contains(':'),
        "current descriptor id is the collector type, not an instance slug"
    );

    let batch = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        200,
        r#"{"name":"app","full_name":"acme/app","visibility":"private","default_branch":"main","archived":false}"#,
    )]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .expect("collect_batch returns CollectionBatch today");
    assert_eq!(batch.run.collector_id, "collector.github");
    let _: &Vec<String> = &batch.errors;
    let _: &Vec<EvidenceEnvelope> = &batch.envelopes;
}

/// chx_b013 — FixtureCollector still seals envelopes for COL-004 consumers.
#[test]
fn chx_b013_fixture_collector_seals_envelopes() {
    let asset = AssetId::new("repo:in-scope");
    let ty = EvidenceType::new("source.codeowners.present");
    let observation = EvidenceObservation::new(ty.clone())
        .with_fact("present", "true")
        .with_narrative("CODEOWNERS presence is structural, not effectiveness");
    let collector = FixtureCollector::new("fixture.chx-baseline", "1")
        .with_evidence_types([ty])
        .with_planned(asset.clone(), observation);
    let envelopes = collector
        .collect(&CollectorScope::new().allow_asset(asset.clone()))
        .expect("fixture collect seals today");
    assert_eq!(envelopes.len(), 1);
    assert_eq!(
        envelopes[0].provenance().collector_id,
        "fixture.chx-baseline"
    );
    assert_eq!(envelopes[0].provenance().asset, asset);
}

/// chx_b014 — GitHub adapter is framework-blind (neighbor law, source scan only).
#[test]
fn chx_b014_github_has_no_iso_needles() {
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
    assert!(
        !github_sources_joined().contains("evidence.github"),
        "do not invent evidence.github.*"
    );
}
