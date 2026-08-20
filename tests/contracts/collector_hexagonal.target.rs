//! Target suite for collector hexagonal increment 1.
//!
//! Encodes DESIRED behavior in `docs/specs/collector-hexagonal.md` §4 / §13
//! (Phases 1–6 + compatibility facade). GREEN after increment 1.
//!
//! Neighbor `sdd_github_collector_target` (`ghc_*`) remains GREEN and is
//! not replaced by this file.

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

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));

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

fn require_file(rel: &str) -> String {
    let path = crate_root().join(rel);
    assert!(
        path.is_file(),
        "hexagonal increment 1 requires {rel} (missing {})",
        path.display()
    );
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
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

fn constructs_provenance_or_seal(src: &str) -> bool {
    src.contains("EvidenceProvenance {") || src.contains("EvidenceEnvelope::seal")
}

/// chx_t001 — still one Cargo package; no collector subcrates.
#[test]
fn chx_t001_single_cargo_package() {
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
        "must not split into collector subcrate members"
    );
}

/// chx_t002 — hexagonal directories exist; lib.rs is a facade.
#[test]
fn chx_t002_hexagonal_layout_and_lib_facade() {
    let src = crate_src();
    for name in ["domain", "application", "ports", "adapters"] {
        let path = src.join(name);
        assert!(
            path.is_dir(),
            "hexagonal increment 1 requires src/{name}/ (missing {})",
            path.display()
        );
    }

    let lib = read_rel("src/lib.rs");
    assert!(
        lib.contains("pub mod domain") || lib.contains("mod domain"),
        "lib.rs must declare the domain module as a hexagonal facade"
    );
    assert!(
        lib.contains("mod application") || lib.contains("pub mod application"),
        "lib.rs must declare the application layer"
    );
    assert!(
        lib.contains("mod ports") || lib.contains("pub mod ports"),
        "lib.rs must declare ports"
    );
    assert!(
        lib.contains("mod adapters") || lib.contains("pub mod adapters"),
        "lib.rs must declare adapters"
    );
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
            !lib.contains(needle),
            "lib.rs is a facade; `{needle}` must live in domain modules, not inline in lib.rs"
        );
    }
}

/// chx_t003 — Phase 1 domain modules exist on disk.
#[test]
fn chx_t003_phase1_domain_modules_exist() {
    for file in [
        "src/domain/mod.rs",
        "src/domain/capabilities.rs",
        "src/domain/descriptor.rs",
        "src/domain/scope.rs",
        "src/domain/collector.rs",
        "src/domain/observation.rs",
        "src/domain/coverage.rs",
        "src/domain/diagnostic.rs",
        "src/domain/batch.rs",
        "src/domain/cursor.rs",
        "src/domain/instance.rs",
        "src/application/mod.rs",
        "src/application/engine.rs",
        "src/application/registry.rs",
        "src/application/gate.rs",
        "src/application/envelope.rs",
        "src/ports/mod.rs",
        "src/ports/adapter.rs",
        "src/adapters/mod.rs",
    ] {
        let _ = require_file(file);
    }
}

/// chx_t004 — capabilities eight fields and descriptor fields preserved; no CollectorId newtype.
#[test]
fn chx_t004_capabilities_and_descriptor_preserved() {
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

    let capabilities = require_file("src/domain/capabilities.rs");
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
            capabilities.contains(field),
            "CollectorCapabilities field `{field}` must live in domain/capabilities.rs"
        );
    }

    let descriptor = require_file("src/domain/descriptor.rs");
    for field in [
        "pub id: String",
        "pub version: String",
        "pub evidence_types:",
        "pub provider_family: String",
        "pub subject_types:",
        "pub capabilities: CollectorCapabilities",
        "pub required_permissions:",
    ] {
        assert!(
            descriptor.contains(field),
            "CollectorDescriptor field `{field}` must be preserved in domain/descriptor.rs"
        );
    }
    assert!(
        !contains_ident(&descriptor, "CollectorId"),
        "do not force CollectorId newtypes before the structural refactor is green"
    );
    assert!(
        !contains_ident(&collector_sources_joined(), "CollectorId"),
        "increment 1 must not introduce a required CollectorId newtype"
    );

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
}

/// chx_t005 — CollectorInstance ≠ collector type; credentials are CredentialRef.
#[test]
fn chx_t005_instance_distinct_from_type_credential_ref() {
    let instance = require_file("src/domain/instance.rs");
    assert!(
        contains_ident(&instance, "CollectorInstance"),
        "domain/instance.rs must define CollectorInstance"
    );
    assert!(
        contains_ident(&instance, "CredentialRef"),
        "domain/instance.rs must define CredentialRef"
    );
    for field in ["id", "collector_id", "configuration", "credential_ref"] {
        assert!(
            instance.contains(field),
            "CollectorInstance must expose field `{field}`"
        );
    }

    let type_id = "collector.github";
    let instance_id = "github:xylex-group";
    assert_ne!(
        type_id, instance_id,
        "collector type (collector.github) is distinct from instance (github:xylex-group)"
    );
    assert!(
        instance.contains("pub id") && instance.contains("collector_id"),
        "instance id and collector_id (type) are separate fields"
    );

    let lowered = instance.to_ascii_lowercase();
    for secret in [
        "token: Option<String>",
        "pat:",
        "password:",
        "secret: String",
    ] {
        assert!(
            !lowered.contains(&secret.to_ascii_lowercase()) && !instance.contains("token:"),
            "CollectorInstance must not hold token/PAT/secret material (found `{secret}`)"
        );
    }
    assert!(
        !instance.contains("token: Option<String>"),
        "tokens are not fields of CollectorInstance — CredentialRef only"
    );
}

/// chx_t006 — GitHub adapter/normalizer does not construct envelopes or provenance.
#[test]
fn chx_t006_github_does_not_construct_envelopes() {
    let normalize = require_file("src/github/normalize.rs");
    assert!(
        !constructs_provenance_or_seal(&normalize),
        "github/normalize.rs must not construct EvidenceProvenance or call EvidenceEnvelope::seal"
    );
    assert!(
        !normalize.contains("-> Result<EvidenceEnvelope"),
        "normalize::emit must return ObservationCandidate, not EvidenceEnvelope"
    );
    assert!(
        contains_ident(&normalize, "ObservationCandidate")
            || normalize.contains("ObservationCandidate"),
        "GitHub normalizer must emit ObservationCandidate"
    );

    let github = github_sources_joined();
    assert!(
        !github.contains("EvidenceProvenance {"),
        "GitHub adapter sources must not construct EvidenceProvenance"
    );
    assert!(
        !github.contains("EvidenceEnvelope::seal"),
        "GitHub adapter sources must not seal EvidenceEnvelope"
    );
}

/// chx_t007 — application layer types exist; only EnvelopeFactory seals.
#[test]
fn chx_t007_application_layer_and_exclusive_factory() {
    let engine = require_file("src/application/engine.rs");
    let registry = require_file("src/application/registry.rs");
    let gate = require_file("src/application/gate.rs");
    let envelope = require_file("src/application/envelope.rs");
    let adapter = require_file("src/ports/adapter.rs");

    assert!(
        contains_ident(&engine, "CollectionEngine"),
        "application/engine.rs must define CollectionEngine"
    );
    assert!(
        contains_ident(&registry, "CollectorRegistry"),
        "application/registry.rs must define CollectorRegistry"
    );
    assert!(
        contains_ident(&gate, "ObservationGate"),
        "application/gate.rs must define ObservationGate"
    );
    assert!(
        contains_ident(&envelope, "EnvelopeFactory"),
        "application/envelope.rs must define EnvelopeFactory"
    );
    assert!(
        contains_ident(&adapter, "CollectorAdapter"),
        "ports/adapter.rs must define CollectorAdapter"
    );

    assert!(
        engine.contains("CollectorRegistry")
            && engine.contains("ObservationGate")
            && engine.contains("EnvelopeFactory"),
        "CollectionEngine flow is Registry → Adapter → ObservationGate → EnvelopeFactory"
    );

    assert!(
        envelope.contains("EvidenceProvenance {") && envelope.contains("EvidenceEnvelope::seal"),
        "only EnvelopeFactory constructs EvidenceProvenance and calls EvidenceEnvelope::seal"
    );

    let observation = require_file("src/domain/observation.rs");
    let batch = require_file("src/domain/batch.rs");
    assert!(
        contains_ident(&observation, "ObservationCandidate"),
        "domain/observation.rs must define ObservationCandidate"
    );
    for field in [
        "asset",
        "evidence_type",
        "facts",
        "narrative",
        "observed_at",
        "valid_from",
        "valid_until",
        "source_revision",
    ] {
        assert!(
            observation.contains(field),
            "ObservationCandidate must include field `{field}`"
        );
    }
    assert!(
        contains_ident(&batch, "ObservationBatch")
            || contains_ident(&observation, "ObservationBatch"),
        "ObservationBatch must exist in domain batch/observation modules"
    );
}

/// chx_t008 — public facade and collect_batch stay compile-stable.
#[test]
fn chx_t008_public_facade_and_collect_batch() {
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

    let batch = GitHubCollector::with_client(client_with(&[(
        "/repos/acme/app",
        200,
        r#"{"name":"app","full_name":"acme/app","visibility":"private","default_branch":"main","archived":false}"#,
    )]))
    .collect_batch(CollectionRequest {
        scope: repo_scope("acme", "app"),
    })
    .expect("GitHubCollector::collect_batch stays compile-stable");
    assert_eq!(batch.run.collector_id, "collector.github");
    let _: &Vec<String> = &batch.errors;
    let _: &Vec<EvidenceEnvelope> = &batch.envelopes;
}

/// chx_t009 — GitHub collector remains framework-blind (ghc_024 source law).
#[test]
fn chx_t009_github_has_no_iso_needles() {
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

/// chx_t010 — neighbor github_src() path stays on disk.
#[test]
fn chx_t010_github_src_on_disk() {
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
}

/// chx_t011 — 403 ≠ false (do not weaken ghc_*; no silent negative facts).
#[test]
fn chx_t011_403_is_not_false() {
    let github = github_sources_joined();
    assert!(
        github.contains("401 | 403"),
        "GitHub fetch maps 401/403 to Denied"
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

/// chx_t012 — ObservationGate validates adapter output before seal.
#[test]
fn chx_t012_observation_gate_validates_before_seal() {
    let gate = require_file("src/application/gate.rs");
    assert!(
        gate.contains("fn validate") || gate.contains("validate("),
        "ObservationGate must validate adapter output"
    );
    let lowered = gate.to_ascii_lowercase();
    assert!(
        lowered.contains("compliance")
            || gate.contains("looks_like_compliance_claim")
            || gate.contains("UndeclaredEvidenceType")
            || gate.contains("declared"),
        "ObservationGate must reject undeclared types / compliance claims"
    );

    let engine = require_file("src/application/engine.rs");
    let engine_idx = engine
        .find("ObservationGate")
        .expect("CollectionEngine must invoke ObservationGate");
    let factory_idx = engine
        .find("EnvelopeFactory")
        .or_else(|| engine.find("seal_batch"))
        .expect("CollectionEngine must invoke EnvelopeFactory after the gate");
    assert!(
        engine_idx < factory_idx,
        "ObservationGate must run before EnvelopeFactory::seal_batch"
    );
}

/// chx_t013 — adapters emit ObservationCandidate internally; local/fixture do not seal.
#[test]
fn chx_t013_adapters_emit_candidates_not_envelopes() {
    let local = require_file("src/local/mod.rs");
    assert!(
        !constructs_provenance_or_seal(&local),
        "LocalCollector must not construct EvidenceProvenance or seal envelopes"
    );

    let mut adapter_seal_sites = Vec::new();
    for rel in [
        "src/github/normalize.rs",
        "src/github/mod.rs",
        "src/local/mod.rs",
        "src/lib.rs",
    ] {
        let src = fs::read_to_string(crate_root().join(rel)).unwrap_or_default();
        if constructs_provenance_or_seal(&src) {
            adapter_seal_sites.push(rel);
        }
    }
    assert!(
        adapter_seal_sites.is_empty(),
        "adapters and lib facade must not invent provenance or seal; found {adapter_seal_sites:?}"
    );

    let domain_collector = require_file("src/domain/collector.rs");
    assert!(
        !domain_collector.contains("EvidenceEnvelope::seal"),
        "FixtureCollector / EvidenceCollector must not seal; EnvelopeFactory owns seal"
    );
}

/// chx_t014 — public CollectionBatch shape and GITHUB_EVIDENCE_TYPES preserved.
#[test]
fn chx_t014_collection_batch_and_github_evidence_types_unchanged() {
    let batch_src = require_file("src/domain/batch.rs");
    assert!(
        batch_src.contains("pub errors: Vec<String>"),
        "CollectionBatch.errors stays Vec<String> this increment (typed diagnostics are Phase 7)"
    );
    assert!(
        batch_src.contains("pub envelopes: Vec<EvidenceEnvelope>"),
        "CollectionBatch still carries sealed envelopes for the scheduler facade"
    );

    let historical = [
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
    for ty in historical {
        assert!(
            GITHUB_EVIDENCE_TYPES.contains(&ty),
            "GITHUB_EVIDENCE_TYPES must still list `{ty}`"
        );
    }

    let fixture = FixtureCollector::new("fixture.chx-target", "1");
    let _ = fixture;
    let observation = EvidenceObservation::new(EvidenceType::new("source.codeowners.present"))
        .with_fact("present", "true")
        .with_narrative("CODEOWNERS presence is structural, not effectiveness");
    let _ = observation;
}
