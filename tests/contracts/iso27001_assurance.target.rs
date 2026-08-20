//! Target suite for the ISO 27001 automated-assurance MVP.
//!
//! Encodes the *desired* vertical in `docs/specs/iso-27001-automated-assurance-mvp.md`
//! (ISO-001…010, EVD-001…010, CTL-001…012, GH-001…012, Phase 54 acceptance).
//! These assertions describe the landed product, not the stub spine. They MUST
//! stay RED until pack / ledger / DSL / GitHub / local / manual / readiness /
//! SoA / CLI exist. Do not `#[ignore]` them and do not weaken them to match
//! today's catalogs.
//!
//! IR types are consumed via the published contracts (`assurance-ir/v1`).
//! This suite does not fork `Control`, `Requirement`, or `Mapping`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use clap::Parser;
use serde_json::{Value, json};
use weeping_angel::cli::Cli;
use weeping_angel::engines::EngineHit;
use weeping_angel_assurance::bridge;
use weeping_angel_assurance::{AssessmentReport, AssessmentScope, AssuranceEngine};
use weeping_angel_assurance_ir::crosswalk::ComplianceGraph;
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, Control, ControlId, EvidenceRequirement, EvidenceRequirementId,
    FrameworkId, FrameworkVersion, Mapping, MappingCompleteness, MappingDirection, Requirement,
    RequirementId,
};
use weeping_angel_collector::{CollectorDescriptor, EvidenceCollector, FixtureCollector};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSet, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::{
    Assessment, AssessmentRequests, FrameworkCapabilities, FrameworkCompileError, FrameworkContext,
    FrameworkProfile, FrameworkTarget, compile_framework, stub_catalog,
};

const FORBIDDEN_CERTIFICATION_PHRASES: &[&str] = &[
    "iso 27001 certified",
    "iso 27001 compliant",
    "certification guaranteed",
    "audit passed",
];

// superseded by sdd_iso27001_remap_target — prefixes/ids now catalog control.*
const CANONICAL_CONTROL_PREFIXES: &[&str] = &["control."];

const EXPECTED_CANONICAL_CONTROLS: &[&str] = &[
    "control.identity.privileged-mfa",
    "control.identity.mfa",
    "control.identity.strong-authentication-policy",
    "control.identity.privileged-access-minimization",
    "control.identity.least-privilege",
    "control.identity.periodic-access-review",
    "control.identity.access-approval",
    "control.identity.unique-user-identities",
    "control.identity.joiner-mover-leaver",
    "control.identity.terminated-user-removal",
    "control.identity.access-revocation-timeliness",
    "control.source.protected-branch",
];

const GITHUB_EVIDENCE_TYPES: &[&str] = &[
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

const FORBIDDEN_NETWORK_PACKAGES: &[&str] = &[
    "reqwest",
    "hyper",
    "h2",
    "octocrab",
    "octorust",
    "cloudflare",
    "aws-sdk-s3",
    "aws-sdk-iam",
    "tokio-tungstenite",
];

const TEST_EXPR_NEEDLES: &[&str] = &[
    "enum TestExpr",
    "FreshWithin",
    "CoverageAtLeast",
    "ManualReview",
    "EvidenceSelector",
];

const RICH_EFFECTIVENESS: &[&str] = &[
    "PartiallyEffective",
    "NotApplicable",
    "NotTested",
    "StaleEvidence",
    "ManualReviewRequired",
    "ExceptionApproved",
];

const LEDGER_NEEDLES: &[&str] = &[
    "EvidenceLedger",
    "CollectionRun",
    "EvidenceArtifactRef",
    "fn append",
    "fn supersede",
];

fn iso_pack_dir() -> PathBuf {
    manifest_dir().join("frameworks/iso-27001/2022")
}

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
}

fn walk_text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_text_files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("toml" | "md" | "json" | "csv" | "txt" | "yaml" | "yml")
        ) {
            out.push(path);
        }
    }
}

fn pack_texts() -> String {
    let mut files = Vec::new();
    walk_text_files(&iso_pack_dir(), &mut files);
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

fn object_keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("json object")
        .keys()
        .cloned()
        .collect()
}

fn cargo_metadata() -> Value {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            &manifest_dir().join("Cargo.toml").to_string_lossy(),
        ])
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata json")
}

fn package_dep_names(meta: &Value, package: &str) -> BTreeSet<String> {
    let packages = meta["packages"].as_array().expect("packages");
    let pkg = packages
        .iter()
        .find(|p| p["name"] == package)
        .unwrap_or_else(|| panic!("package {package} missing from cargo metadata"));
    pkg["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .filter_map(|d| d["name"].as_str().map(str::to_string))
        .collect()
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities {
            supports_control_applicability: true,
            supports_statement_of_applicability: true,
            supports_risk_treatment: true,
            supports_manual_attestation: true,
            ..FrameworkCapabilities::default()
        },
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn fresh_provenance(asset: &str) -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.github-like".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new(asset),
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn sample_hit() -> EngineHit {
    EngineHit {
        rule_id: "path-traversal.archive-extraction".into(),
        anchor: "archive-entry-write-without-containment".into(),
        instance: None,
        title: "Unsafe archive extraction".into(),
        summary: "An attacker-controlled path reaches a filesystem write.".into(),
        evidence: "open(join(dest, name))".into(),
        severity: "high",
        confidence: "high",
        confidence_rationale: "static pattern".into(),
        category: "path-traversal".into(),
        cwe: vec!["CWE-22".into()],
        remediation: "Contain extraction paths.".into(),
        path: "src/extract.py".into(),
        start_line: 41,
        end_line: Some(44),
        role: "sink",
        snippet: "extract(archive, dest)".into(),
        validation_json: None,
        attack_path_json: None,
    }
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn looks_like_protected_iso_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("information security management system")
        && lower.contains("the organization shall")
        || lower.contains("annex a") && lower.contains("control objective")
        || lower.contains("normative text reproduced from iso/iec 27001")
}

fn report_contains_forbidden_claim(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    FORBIDDEN_CERTIFICATION_PHRASES
        .iter()
        .any(|phrase| lower.contains(phrase))
}

// ── Dual-suite registration ────────────────────────────────────────────────

#[test]
fn dual_suite_is_registered_in_root_cargo_toml() {
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !cargo.contains("sdd_iso27001_assurance_baseline")
            && !cargo.contains("tests/contracts/iso27001_assurance.baseline.rs")
            && harness_src().contains("iso27001_assurance.target.rs")
            && harness_src().contains("iso27001_assurance.target.rs"),
        "wired as a harness module the dual-suite (tests/contracts is not auto-discovered)"
    );
}

// ── ISO-001…010 ────────────────────────────────────────────────────────────

#[test]
fn iso_001_framework_pack_compiles_deterministically() {
    let manifest = iso_pack_dir().join("manifest.toml");
    assert!(
        manifest.is_file(),
        "ISO-001: expected versioned pack at {}",
        manifest.display()
    );
    let framework_src = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "ISO-001 pack loader",
        &framework_src,
        &[
            "FrameworkPackDigest",
            "load_framework_pack",
            "weeping-angel/framework-pack/v1",
        ],
    );
    let catalog = stub_catalog(FrameworkProfile::Iso27001);
    assert!(
        !catalog.is_empty(),
        "ISO-001: Iso27001 catalog must compile from the pack, not remain an empty stub"
    );
}

#[test]
fn iso_002_public_pack_has_no_protected_normative_text() {
    let dir = iso_pack_dir();
    assert!(
        dir.is_dir(),
        "ISO-002: public pack directory must exist at {}",
        dir.display()
    );
    let text = pack_texts();
    assert!(
        !text.is_empty(),
        "ISO-002: pack must contain structural files"
    );
    assert!(
        !looks_like_protected_iso_text(&text),
        "ISO-002: public pack must not reproduce protected ISO/IEC 27001 normative wording"
    );
}

#[test]
fn iso_003_catalog_has_no_provider_types() {
    let dir = iso_pack_dir();
    assert!(dir.is_dir(), "ISO-003: pack must exist");
    let pack = pack_texts();
    let framework_src = crate_sources_joined("weeping-angel-framework");
    for hay in [&pack, &framework_src] {
        for forbidden in ["octocrab", "octorust", "Octokit", "reqwest::", "github.com"] {
            assert!(
                !hay.contains(forbidden),
                "ISO-003: catalog/compiler must not embed provider type `{forbidden}`"
            );
        }
    }
}

#[test]
fn iso_004_tests_are_canonical_not_github_or_iso_prefixed() {
    let mappings = iso_pack_dir().join("mappings.toml");
    assert!(
        mappings.is_file(),
        "ISO-004: mappings.toml is required so ISO refs point at canonical controls"
    );
    let catalog = stub_catalog(FrameworkProfile::Iso27001);
    let compiled = compile_framework(&in_memory_iso_assessment(), &iso_target())
        .unwrap_or_else(|e| panic!("ISO-004: pack-backed compile should succeed: {e}"));
    assert!(
        catalog.len() >= 20 || compiled.controls.len() >= 20,
        "ISO-004: at least 20 canonical automated/hybrid controls are required (got catalog={}, controls={})",
        catalog.len(),
        compiled.controls.len()
    );
    for control in &compiled.controls {
        let id = control.id().as_str();
        assert!(
            !id.starts_with("iso27001.") && !id.contains(".github."),
            "ISO-004: control id `{id}` must be canonical, not framework- or provider-prefixed"
        );
        assert!(
            CANONICAL_CONTROL_PREFIXES
                .iter()
                .any(|prefix| id.starts_with(prefix)),
            "ISO-004: `{id}` is not in the canonical control library"
        );
    }
    for test in &compiled.tests {
        let id = test.id.as_str();
        assert!(
            !id.contains("iso27001.") && !id.contains(".github."),
            "ISO-004: test id `{id}` must not be an ISO- or GitHub-specific test"
        );
    }
}

#[test]
fn iso_005_partial_mapping_cannot_fully_satisfy_requirement() {
    let mapping = Mapping::new(
        RequirementId::new("iso27001:a.8.25"),
        ControlId::new("source.branch-protection"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    let json = serde_json::to_value(&mapping).unwrap();
    assert!(
        json.get("relation").is_some() || json.get("rationale").is_some(),
        "ISO-005: mappings must carry explicit relation/rationale so PartiallySatisfies cannot be collapsed"
    );

    let mut graph = ComplianceGraph::new();
    graph.link(
        RequirementId::new("iso27001:a.8.25"),
        RequirementId::new("canonical:source.branch-protection"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert!(
        !graph.equivalent(
            &RequirementId::new("iso27001:a.8.25"),
            &RequirementId::new("canonical:source.branch-protection")
        ),
        "ISO-005 / ACT-005: partial never upgrades to equivalence"
    );

    let assurance_src = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "ISO-005 readiness aggregation",
        &assurance_src,
        &[
            "PartiallySatisfies",
            "partially covered",
            "FrameworkReadinessSnapshot",
        ],
    );
}

#[test]
fn iso_006_unsupported_capability_fails_closed() {
    let mut assessment = in_memory_iso_assessment();
    assessment.requests.statement_of_applicability = true;
    let mut target = iso_target();
    target.capabilities.supports_statement_of_applicability = false;
    let err = compile_framework(&assessment, &target)
        .expect_err("ISO-006: SoA without capability must fail closed");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(capability, "supports_statement_of_applicability");
        }
        other => panic!("ISO-006: expected CapabilityViolation, got {other:?}"),
    }
}

#[test]
fn iso_007_pack_digest_is_stable_and_recorded_on_snapshot() {
    let framework_src = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "ISO-007",
        &framework_src,
        &["FrameworkPackDigest", "load_framework_pack"],
    );
    let assurance_src = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "ISO-007 snapshot",
        &assurance_src,
        &["frameworkPackDigest", "AssessmentRun"],
    );
    assert!(
        iso_pack_dir().join("manifest.toml").is_file(),
        "ISO-007: pack must exist so two loads can compare FrameworkPackDigest"
    );
}

#[test]
fn iso_008_unknown_requirement_or_pack_is_rejected() {
    let framework_src = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "ISO-008",
        &framework_src,
        &["UnknownRequirement", "UnknownPack", "load_framework_pack"],
    );
}

#[test]
fn iso_009_invalid_mapping_is_rejected() {
    let framework_src = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "ISO-009",
        &framework_src,
        &["dangling", "rationale", "unsupported relation"],
    );
    assert!(
        iso_pack_dir().join("mappings.toml").is_file(),
        "ISO-009: mappings.toml must exist so invalid relations can be rejected"
    );
}

#[test]
fn iso_010_soa_preserves_applicability_rationale() {
    let pack_applicability = iso_pack_dir().join("applicability.toml");
    assert!(
        pack_applicability.is_file(),
        "ISO-010: applicability.toml must exist"
    );
    let assurance_src = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "ISO-010",
        &assurance_src,
        &[
            "StatementOfApplicability",
            "applicability rationale",
            "applicable",
        ],
    );
}

// ── Canonical control library / catalog coverage ───────────────────────────

#[test]
fn mvp_ships_at_least_twenty_canonical_controls() {
    let catalog = stub_catalog(FrameworkProfile::Iso27001);
    let ids: BTreeSet<String> = catalog
        .iter()
        .map(|r| r.id().as_str().to_string())
        .collect();
    let compiled_ids = compile_framework(&in_memory_iso_assessment(), &iso_target())
        .map(|c| {
            c.controls
                .iter()
                .map(|ctl| ctl.id().as_str().to_string())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let union: BTreeSet<_> = ids.union(&compiled_ids).cloned().collect();
    let matched: Vec<&str> = EXPECTED_CANONICAL_CONTROLS
        .iter()
        .copied()
        .filter(|id| union.iter().any(|have| have == id || have.ends_with(id)))
        .collect();
    assert!(
        union.len() >= 20 && matched.len() >= EXPECTED_CANONICAL_CONTROLS.len().min(12),
        "need >=20 canonical controls including the Phase 4 library; have {union:?}"
    );
}

// ── EVD-001…010 ────────────────────────────────────────────────────────────

#[test]
fn evd_001_envelopes_are_immutable_and_carry_identity_fields() {
    let obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_fact("enabled", "true")
        .with_narrative("repository in-scope has branch protection enabled");
    let env = EvidenceEnvelope::seal(obs, fresh_provenance("repo:in-scope")).unwrap();
    let json = serde_json::to_value(&env).unwrap();
    for required in [
        "evidenceId",
        "schemaVersion",
        "artifactRef",
        "collectionRunId",
        "contentDigest",
        "sensitivity",
    ] {
        assert!(
            json.get(required).is_some(),
            "EVD-001: EvidenceEnvelope must expose `{required}` (got keys {:?})",
            object_keys(&json)
        );
    }
}

#[test]
fn evd_002_duplicate_evidence_is_deduplicated_by_the_ledger() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles("EVD-002", &src, LEDGER_NEEDLES);
    assert!(
        !src.contains("set_compliant") && !src.contains("set_control_status"),
        "EVD-002: ledger must not own conclusions"
    );
}

#[test]
fn evd_003_supersession_preserves_history() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles("EVD-003", &src, &["fn supersede", "supersedes"]);
}

#[test]
fn evd_004_artifact_digest_is_verified() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "EVD-004",
        &src,
        &["EvidenceArtifactRef", "storageLocator", "redactionState"],
    );
}

#[test]
fn evd_005_collection_run_trace_is_preserved() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "EVD-005",
        &src,
        &["CollectionRun", "collectorVersion", "configurationDigest"],
    );
}

#[test]
fn evd_006_secret_keys_are_rejected_and_tokens_never_persist() {
    let src = crate_sources_joined("weeping-angel-evidence");
    require_needles("EVD-006", &src, &["CredentialInPayload", "redact"]);
    let collector_src = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector_src.to_ascii_lowercase().contains("ghp_"),
        "EVD-006 / GH-009: collector sources must not embed GitHub tokens"
    );
}

#[test]
fn evd_007_framework_claims_are_rejected() {
    let claim = EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
        .with_narrative("ISO 27001 certified");
    let err = EvidenceEnvelope::seal(claim, fresh_provenance("repo:in-scope"));
    assert!(
        err.is_err(),
        "EVD-007: certification/compliance claims must be rejected at seal"
    );
}

#[test]
fn evd_008_stale_evidence_is_an_explicit_state() {
    let parsed: Result<Effectiveness, _> = serde_json::from_value(json!("staleEvidence"));
    assert!(
        parsed.is_ok(),
        "EVD-008 / CTL-005: Effectiveness must include StaleEvidence; never treat stale as Effective"
    );
}

#[test]
fn evd_009_scope_is_preserved_on_envelopes() {
    let env = EvidenceEnvelope::seal(
        EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
            .with_narrative("repository in-scope has branch protection enabled"),
        fresh_provenance("repo:in-scope"),
    )
    .unwrap();
    let json = serde_json::to_value(&env).unwrap();
    assert!(
        json.get("scope").is_some()
            || json
                .pointer("/provenance/scope")
                .and_then(Value::as_str)
                .is_some_and(|s| s.contains("repo:in-scope")),
        "EVD-009: sealed envelope must retain collection scope"
    );
    assert!(
        json.get("collectionRunId").is_some(),
        "EVD-009: every envelope must trace to a collection run"
    );
}

#[test]
fn evd_010_failed_collector_does_not_fabricate_evidence() {
    let src = crate_sources_joined("weeping-angel-collector");
    require_needles(
        "EVD-010",
        &src,
        &[
            "PermissionDenied",
            "InsufficientEvidence",
            "CollectionBatch",
        ],
    );
}

// ── CTL-001…012 ────────────────────────────────────────────────────────────

#[test]
fn ctl_001_expression_ast_exists_and_is_deterministic() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-001", &src, TEST_EXPR_NEEDLES);
    let tokens: Vec<String> = src
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect();
    assert!(
        !tokens
            .iter()
            .any(|t| t == "rhai" || t == "lua" || t == "javascript"),
        "CTL-001: TestExpr must be a bounded AST, not a script host"
    );
}

#[test]
fn ctl_002_control_test_crate_is_network_free() {
    let meta = cargo_metadata();
    let deps = package_dep_names(&meta, "weeping-angel-control-test");
    for forbidden in FORBIDDEN_NETWORK_PACKAGES {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "CTL-002: control-test must not depend on `{forbidden}` (deps={deps:?})"
        );
    }
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-002 + DSL", &src, &["enum TestExpr"]);
}

#[test]
fn ctl_003_evaluator_is_provider_blind() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-003", &src, &["EvidenceSelector", "enum TestExpr"]);
    assert!(
        !src.contains("octocrab") && !src.contains("GitHubCollector"),
        "CTL-003: control-test must not import provider collectors"
    );
    assert!(
        !src.contains("collector_id")
            || src.contains("provider-blind")
            || src.contains("provider_blind"),
        "CTL-003: evaluator decision signature must not key on collector id"
    );
}

#[test]
fn ctl_004_missing_evidence_is_not_effective() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-004", &src, &["enum TestExpr", "Missing("]);
    let test = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.source.required-review",
        ))
        .control_id(ControlId::new("source.required-review"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("source.branch.required_reviews"))
        .build();
    let result = evaluate(&test, &EvidenceSet::new(), &fresh_context());
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "CTL-004: missing evidence must not be Effective"
    );
    let json = serde_json::to_value(&result).unwrap();
    assert!(
        json.get("missingEvidence").is_some() && json.get("evidenceRefs").is_some(),
        "CTL-004 / CTL-012: result must list missing evidence and used refs"
    );
}

#[test]
fn ctl_005_stale_evidence_is_not_effective() {
    let parsed: Result<Effectiveness, _> = serde_json::from_value(json!("staleEvidence"));
    assert!(
        parsed.is_ok(),
        "CTL-005: stale evidence must serialize as StaleEvidence, not Inconclusive/Effective"
    );
}

#[test]
fn ctl_006_break_evidence_wins() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-006", &src, &["enum TestExpr", "Ineffective"]);
}

#[test]
fn ctl_007_partial_coverage_remains_partial() {
    let parsed: Result<Effectiveness, _> = serde_json::from_value(json!("partiallyEffective"));
    assert!(
        parsed.is_ok(),
        "CTL-007: Effectiveness must include PartiallyEffective so partial coverage is not a full pass"
    );
}

#[test]
fn ctl_008_manual_review_cannot_auto_pass() {
    let parsed: Result<Effectiveness, _> = serde_json::from_value(json!("manualReviewRequired"));
    assert!(
        parsed.is_ok(),
        "CTL-008: ManualReviewRequired must be a first-class effectiveness state"
    );
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-008", &src, &["ManualReview"]);
}

#[test]
fn ctl_009_type_mismatches_fail_closed() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles(
        "CTL-009",
        &src,
        &["enum EvidenceValue", "Integer", "type mismatch"],
    );
}

#[test]
fn ctl_010_threshold_semantics_are_deterministic() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-010", &src, &["Gte", "enum TestExpr", "Integer"]);
}

#[test]
fn ctl_011_subject_coverage_is_computed() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("CTL-011", &src, &["CoverageAtLeast", "SubjectSelector"]);
}

#[test]
fn ctl_012_evidence_trace_is_complete() {
    let parsed: Result<ControlTestResult, _> = serde_json::from_value(json!({
        "testId": "test.source.required-review",
        "controlId": "source.required-review",
        "effectiveness": "ineffective",
        "rationale": "Repository main branch requires 1 approval; policy requires >= 2.",
        "evidenceRefs": ["ev:sha256:abc"],
        "missingEvidence": [],
        "evaluatedAt": "2026-08-18T12:30:00Z",
        "testVersion": "1",
        "inputDigest": "00",
        "status": "ineffective",
        "reason": "Repository main branch requires 1 approval; policy requires >= 2."
    }));
    assert!(
        parsed.is_ok(),
        "CTL-012: ControlTestResult must accept the Phase 24 trace contract, got {parsed:?}"
    );
}

#[test]
fn ctl_richer_effectiveness_states_exist() {
    let src = crate_sources_joined("weeping-angel-control-test");
    require_needles("Phase 23 effectiveness", &src, RICH_EFFECTIVENESS);
}

// ── GH-001…012 ─────────────────────────────────────────────────────────────

#[test]
fn gh_001_012_github_collector_module_and_taxonomy() {
    let github = crate_src("weeping-angel-collector").join("github");
    assert!(
        github.is_dir(),
        "GH-001…012: expected collector module at {}",
        github.display()
    );
    let src = crate_sources_joined("weeping-angel-collector");
    require_needles(
        "GitHub collector",
        &src,
        &[
            "GitHubCollector",
            "source.branch.protection",
            "source.branch.required_reviews",
            "PermissionDenied",
            "Retry-After",
        ],
    );
    for evidence_type in GITHUB_EVIDENCE_TYPES {
        assert!(
            src.contains(evidence_type),
            "GH-012: descriptor taxonomy must advertise `{evidence_type}`"
        );
    }
    assert!(
        !src.contains("iso27001") && !src.contains("soc2_controls"),
        "GH-012: collectors advertise evidence types, never frameworks"
    );
}

#[test]
fn gh_007_403_is_permission_denied_not_false() {
    let src = crate_sources_joined("weeping-angel-collector");
    require_needles(
        "GH-007",
        &src,
        &["PermissionDenied", "403", "InsufficientEvidence"],
    );
    assert!(
        !src.contains("403 => false") && !src.contains("status == 403 && enabled = false"),
        "GH-007: 403 must not be normalized to a boolean false observation"
    );
}

#[test]
fn gh_009_tokens_never_leak() {
    let src = crate_sources_joined("weeping-angel-collector");
    require_needles("GH-009", &src, &["redact", "Authorization"]);
    assert!(
        !src.contains("GITHUB_TOKEN=") && !src.contains("ghp_"),
        "GH-009: collector must not persist or log tokens"
    );
}

// ── Scanner bridge ─────────────────────────────────────────────────────────

#[test]
fn scanner_bridge_is_one_way_and_empty_scan_is_not_effective() {
    let hit = sample_hit();
    let before = serde_json::to_value(hit.to_semantic_finding()).unwrap();
    let obs = bridge::from_engine_hit(&hit);
    let after = serde_json::to_value(hit.to_semantic_finding()).unwrap();
    assert_eq!(before, after, "bridge must not rewrite SemanticFinding");
    assert!(
        obs.fact("iso27001").is_none(),
        "bridge must not attach framework status"
    );

    let bridge_src = fs::read_to_string(crate_src("weeping-angel-assurance").join("bridge.rs"))
        .unwrap_or_default();
    let joined = format!(
        "{bridge_src}\n{}",
        crate_sources_joined("weeping-angel-assurance")
    );
    require_needles(
        "scanner evidence taxonomy",
        &joined,
        &[
            "security.vulnerability.present",
            "security.finding",
            "security.secret.exposure",
        ],
    );
    assert!(
        !joined.contains("security.no_vulnerabilities"),
        "absence of findings must not be positive compliance evidence"
    );
}

// ── Local + manual evidence ────────────────────────────────────────────────

#[test]
fn local_collector_and_manual_evidence_exist() {
    let src = crate_sources_joined("weeping-angel-collector");
    let assurance_src = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "Phase 18/19",
        &format!("{src}\n{assurance_src}"),
        &[
            "LocalCollector",
            "ManualEvidence",
            "attested-by",
            "CODEOWNERS",
        ],
    );
    assert!(
        crate_src("weeping-angel-collector").join("local").is_dir()
            || src.contains("struct LocalCollector"),
        "local filesystem collector must ship"
    );
}

// ── Readiness, SoA, snapshots ──────────────────────────────────────────────

#[test]
fn readiness_is_a_projection_not_a_single_percentage() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("source.branch.protection")])
        .with_planned(
            AssetId::new("repo:in-scope"),
            EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
                .with_fact("enabled", "true")
                .with_narrative("repository in-scope has branch protection enabled"),
        );
    let report = AssuranceEngine::builder()
        .collector(collector)
        .framework(iso_target())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")));
    let report = report.expect("ISO assess should compile the real pack");
    let json = serde_json::to_value(&report).unwrap();
    for required in [
        "frameworkPackDigest",
        "requirements",
        "controls",
        "insufficientEvidence",
        "manualReview",
        "automationCoverage",
        "evidenceCoverage",
    ] {
        assert!(
            json.get(required).is_some()
                || json.pointer(&format!("/readiness/{required}")).is_some(),
            "readiness snapshot must expose `{required}`, not a single percentage (keys {:?})",
            object_keys(&json)
        );
    }
    assert!(
        json.get("readinessPercent").is_none() && json.get("score").is_none(),
        "readiness must not collapse to one percentage"
    );
    let serialized = serde_json::to_string(&report).unwrap();
    assert!(
        !report_contains_forbidden_claim(&serialized),
        "reports must never emit certified/compliant/audit passed"
    );
}

#[test]
fn assessment_report_carries_not_certification_banner() {
    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-iso-1"),
        profile: "iso-27001".into(),
        digest: "0".repeat(64),
        results: vec![],
        evidence_count: 0,
        ..Default::default()
    };
    let json = serde_json::to_value(&report).unwrap();
    let blob = json.to_string().to_ascii_lowercase();
    assert!(
        blob.contains("not certification")
            || blob.contains("not a certification")
            || json.get("disclaimer").is_some()
            || json.get("banner").is_some(),
        "Phase 38/54: reports must carry an explicit not-certification banner"
    );
}

#[test]
fn snapshots_are_immutable_and_comparable() {
    let src = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "Phase 35/36",
        &src,
        &[
            "AssessmentRun",
            "fn compare",
            "became effective",
            "became ineffective",
            "evidenceSnapshotDigest",
        ],
    );
}

// ── Traceability ───────────────────────────────────────────────────────────

#[test]
fn automated_results_trace_requirement_to_collection_run() {
    let src = format!(
        "{}\n{}\n{}",
        crate_sources_joined("weeping-angel-assurance"),
        crate_sources_joined("weeping-angel-framework"),
        crate_sources_joined("weeping-angel-evidence")
    );
    require_needles(
        "Phase 54 trace",
        &src,
        &[
            "collectionRunId",
            "evidenceRefs",
            "frameworkPackDigest",
            "MapsTo",
            "TestedBy",
            "RequiresEvidence",
        ],
    );
}

// ── CLI family ─────────────────────────────────────────────────────────────

#[test]
fn cli_exposes_assurance_assess_iso27001() {
    let parsed = Cli::try_parse_from([
        "weeping-angel",
        "assurance",
        "assess",
        "--framework",
        "iso-27001",
        "--scope",
        ".",
    ]);
    assert!(
        parsed.is_ok(),
        "CLI must accept `assurance assess --framework iso-27001 --scope .`: {parsed:?}"
    );
}

#[test]
fn cli_exposes_framework_collect_evidence_result_compare_soa() {
    let cmd = Cli::clap_command();
    let names: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
    assert!(
        names.contains(&"assurance"),
        "Commands must grow an `assurance` family without leaking compiler topology; have {names:?}"
    );
    let cases = [
        vec!["weeping-angel", "assurance", "framework", "list"],
        vec![
            "weeping-angel",
            "assurance",
            "framework",
            "validate",
            "frameworks/iso-27001/2022",
        ],
        vec![
            "weeping-angel",
            "assurance",
            "framework",
            "show",
            "iso-27001",
        ],
        vec!["weeping-angel", "assurance", "collect"],
        vec!["weeping-angel", "assurance", "evidence", "list"],
        vec![
            "weeping-angel",
            "assurance",
            "evidence",
            "add",
            "--type",
            "policy.security.reviewed",
            "--subject",
            "organization:default",
            "--attested-by",
            "floris",
        ],
        vec!["weeping-angel", "assurance", "result", "show"],
        vec!["weeping-angel", "assurance", "compare"],
        vec!["weeping-angel", "assurance", "soa"],
    ];
    for argv in cases {
        let parsed = Cli::try_parse_from(&argv);
        assert!(
            parsed.is_ok(),
            "CLI family must accept {argv:?}: {parsed:?}"
        );
    }
}

// ── Ownership / network-free compiler ──────────────────────────────────────

#[test]
fn framework_and_control_test_stay_network_free() {
    let meta = cargo_metadata();
    for package in ["weeping-angel-framework", "weeping-angel-control-test"] {
        let deps = package_dep_names(&meta, package);
        for forbidden in FORBIDDEN_NETWORK_PACKAGES {
            assert!(
                !deps.iter().any(|d| d == forbidden),
                "{package} must stay network-free; found `{forbidden}`"
            );
        }
    }
    let framework_src = crate_sources_joined("weeping-angel-framework");
    require_needles(
        "framework pack (so this invariant is not vacuously green)",
        &framework_src,
        &["load_framework_pack", "FrameworkPackDigest"],
    );
}

#[test]
fn collector_descriptor_has_no_framework_field() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("source.branch.protection")]);
    let desc: CollectorDescriptor = collector.descriptor();
    let json = serde_json::to_value(&desc).unwrap();
    assert!(json.get("frameworks").is_none());
    assert!(json.get("iso_controls").is_none());
    assert!(json.get("soc2_controls").is_none());
    assert!(
        json.get("providerFamily").is_some() && json.get("capabilities").is_some(),
        "Phase 12 descriptor must advertise provider_family + capabilities, not frameworks"
    );
}

#[test]
fn golden_iso_fixture_organization_exists() {
    let fixture = manifest_dir().join("fixtures/assurance/iso27001");
    assert!(
        fixture.is_dir(),
        "Phase 48: expected deterministic fixture org at {}",
        fixture.display()
    );
    for name in ["repo-secure", "repo-insecure"] {
        assert!(
            fixture.join(name).exists()
                || fs::read_to_string(fixture.join("manifest.toml"))
                    .ok()
                    .is_some_and(|t| t.contains(name)),
            "Phase 48 fixture must describe {name}"
        );
    }
}

// ── Shared fixtures used by ISO-004/006 ────────────────────────────────────

fn in_memory_iso_assessment() -> Assessment {
    let requirement = Requirement::new(
        RequirementId::new("iso27001:a.8.25"),
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        "Secure development lifecycle (structural)",
        "Protect the authoritative source of software.",
    );
    let control = Control::new(
        ControlId::new("control.source.protected-branch"),
        "Protected branch",
        "Exists-only protected-branch fixture from the canonical catalog.",
    );
    let mapping = Mapping::new(
        requirement.id().clone(),
        control.id().clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    let evidence_req = EvidenceRequirement::new(
        EvidenceRequirementId::new("ev.source.branch.protection"),
        EvidenceType::new("source.branch.protection"),
    );
    let mut assessment = Assessment::new(AssessmentId::new("assess-iso-target-1"));
    assessment.requirements = vec![requirement];
    assessment.controls = vec![control];
    assessment.mappings = vec![mapping];
    assessment.evidence_requirements = vec![evidence_req];
    assessment.requests = AssessmentRequests::default();
    assessment
}
