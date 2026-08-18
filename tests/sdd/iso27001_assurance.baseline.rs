//! SUPERSEDED by `sdd_iso27001_assurance_target` after the ISO 27001 MVP landed.
//!
//! Historical characterization of the stub spine in
//! `docs/sdd/iso-27001-automated-assurance-mvp.md` §3 / §7.1 (planning SHA `8c0f36ed…`).
//! Kept for rollback narrative. Do not delete. Tests are ignored because the workspace
//! now contains the pack / ledger / DSL / collectors / readiness / SoA / CLI vertical.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use clap::Parser;
use serde_json::Value;
use weeping_angel::cli::{Cli, Commands};
use weeping_angel::engines::EngineHit;
use weeping_angel_assurance::bridge;
use weeping_angel_assurance::{AssessmentReport, AssessmentScope, AssuranceEngine};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentId, AssetId, Control, ControlId, EvidenceRequirement,
    EvidenceRequirementId, FrameworkId, FrameworkVersion, Mapping, MappingCompleteness,
    MappingDirection, Requirement, RequirementId,
};
use weeping_angel_collector::{
    CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector, FixtureCollector,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSet, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceError, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::{
    Assessment, AssessmentRequests, FrameworkCapabilities, FrameworkCompileError, FrameworkContext,
    FrameworkProfile, FrameworkTarget, compile_framework, stub_catalog,
};

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

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
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

fn fresh_provenance(asset: &str) -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.github-like".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new(asset),
    }
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn in_memory_assessment() -> Assessment {
    let requirement = Requirement::new(
        RequirementId::new("canonical:stub-1"),
        FrameworkId::new("canonical"),
        FrameworkVersion::new("2022"),
        "Stub requirement",
        "Protect the authoritative source of software.",
    );
    let control = Control::new(
        ControlId::new("canonical.source-control"),
        "Source control",
        "Protect the authoritative software source.",
    );
    let mapping = Mapping::new(
        requirement.id().clone(),
        control.id().clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    let evidence_req = EvidenceRequirement::new(
        EvidenceRequirementId::new("ev.branch_protection"),
        EvidenceType::new("branch_protection"),
    );
    let mut assessment = Assessment::new(AssessmentId::new("assess-runtime-1"));
    assessment.requirements = vec![requirement];
    assessment.controls = vec![control];
    assessment.mappings = vec![mapping];
    assessment.evidence_requirements = vec![evidence_req];
    assessment.requests = AssessmentRequests::default();
    assessment
}

fn compiled_presence_test() -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.ev.branch_protection",
        ))
        .control_id(ControlId::new("canonical.source-control"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("branch_protection"))
        .break_on(EvidenceType::new("exposed_without_auth"))
        .build()
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: std::time::Duration::from_secs(24 * 3600),
    }
}

// ── Framework catalog / compile ────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn stub_catalog_is_empty_for_every_profile_including_iso27001() {
    let profiles = [
        FrameworkProfile::Iso27001,
        FrameworkProfile::Iso27701,
        FrameworkProfile::Gdpr,
        FrameworkProfile::Soc2,
        FrameworkProfile::Nis2,
        FrameworkProfile::Dora,
        FrameworkProfile::Iso27007,
    ];
    for profile in profiles {
        assert!(
            stub_catalog(profile).is_empty(),
            "stub_catalog({profile:?}) currently returns []"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn compile_framework_runs_eight_stage_pipeline_on_in_memory_assessment() {
    let compiled = compile_framework(&in_memory_assessment(), &iso_target())
        .expect("in-memory stub Assessment compiles");
    assert_eq!(
        compiled.validation.stages,
        [
            "normalize",
            "resolve_applicability",
            "validate_capabilities",
            "resolve_control_mappings",
            "resolve_evidence_requirements",
            "construct_test_plan",
            "construct_framework_projection",
            "integrity_validation",
        ]
    );
    assert!(compiled.validation.ok);
    assert_eq!(compiled.applicable_requirements.len(), 1);
    assert_eq!(
        compiled.applicable_requirements[0].id().as_str(),
        "canonical:stub-1"
    );
    assert_eq!(
        compiled.controls[0].id().as_str(),
        "canonical.source-control"
    );
    assert_eq!(
        compiled.tests.len(),
        1,
        "empty tests synthesize a presence test"
    );
    assert_eq!(compiled.tests[0].id.as_str(), "test.ev.branch_protection");
    assert_eq!(
        compiled.tests[0].required,
        vec![EvidenceType::new("branch_protection")]
    );
    assert_eq!(
        compiled.tests[0].break_on,
        vec![EvidenceType::new("exposed_without_auth")]
    );
    assert!(!compiled.digest.is_empty());
    let again = compile_framework(&in_memory_assessment(), &iso_target()).unwrap();
    assert_eq!(compiled.digest, again.digest);
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn applicability_is_identity_over_supplied_requirements() {
    let compiled = compile_framework(&in_memory_assessment(), &iso_target()).unwrap();
    assert_eq!(
        compiled.applicable_requirements.len(),
        in_memory_assessment().requirements.len(),
        "resolve_applicability currently copies every assessment requirement"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn no_on_disk_framework_pack_or_pack_digest() {
    let root = manifest_dir();
    assert!(
        !root.join("frameworks").exists(),
        "current tree has no frameworks/ pack tree"
    );
    assert!(
        !root
            .join("frameworks/iso-27001/2022/manifest.toml")
            .exists(),
        "ISO 27001:2022 pack is not shipped"
    );
    let framework_src = crate_sources_joined("weeping-angel-framework");
    for needle in [
        "FrameworkPackDigest",
        "load_framework_pack",
        "framework-pack/v1",
        "FrameworkContentProvider",
    ] {
        assert!(
            !framework_src.contains(needle),
            "framework crate currently has no pack loader; found `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn soa_request_without_capability_is_capability_violation() {
    let mut assessment = in_memory_assessment();
    assessment.requests.statement_of_applicability = true;
    let err = compile_framework(&assessment, &iso_target())
        .expect_err("SoA without supports_statement_of_applicability fails closed");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(capability, "supports_statement_of_applicability");
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }
}

// ── Facade assess ──────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn facade_assess_compiles_hard_coded_stub_not_a_pack() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:in-scope"),
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_fact("enabled", "true")
                .with_narrative("repository in-scope has branch_protection enabled"),
        );
    let report = AssuranceEngine::builder()
        .collector(collector)
        .framework(iso_target())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect("facade assess on the hard-coded stub");

    assert_eq!(report.assessment_id.as_str(), "assess-runtime-1");
    assert_eq!(report.profile, "iso-27001");
    assert_eq!(report.evidence_count, 1);
    assert_eq!(report.results.len(), 1);
    assert_eq!(
        report.results[0].test_id.as_str(),
        "test.ev.branch_protection"
    );
    assert_eq!(
        report.results[0].control_id.as_str(),
        "canonical.source-control"
    );
    assert_eq!(report.results[0].effectiveness, Effectiveness::Effective);
    assert!(!report.digest.is_empty());

    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(
        object_keys(&json),
        BTreeSet::from([
            "assessmentId".into(),
            "profile".into(),
            "digest".into(),
            "results".into(),
            "evidenceCount".into(),
        ])
    );
    for absent in [
        "readiness",
        "soa",
        "statementOfApplicability",
        "missingEvidence",
        "automationCoverage",
        "requirements",
        "frameworkPackDigest",
    ] {
        assert!(
            json.get(absent).is_none(),
            "AssessmentReport currently has no `{absent}` field"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn assessment_report_type_has_no_readiness_or_soa_fields() {
    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-runtime-1"),
        profile: "iso-27001".into(),
        digest: "0".repeat(64),
        results: vec![],
        evidence_count: 0,
    };
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(
        object_keys(&json),
        BTreeSet::from([
            "assessmentId".into(),
            "profile".into(),
            "digest".into(),
            "results".into(),
            "evidenceCount".into(),
        ])
    );
}

// ── Evidence ───────────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn evidence_envelope_is_observation_provenance_digest_with_string_facts() {
    let obs = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_fact("enabled", "true")
        .with_narrative("repository in-scope has branch_protection enabled");
    let facts = obs.facts();
    assert_eq!(facts.get("enabled").and_then(|v| v.as_str()), Some("true"));
    assert_eq!(obs.fact("enabled"), Some("true"));

    let env = EvidenceEnvelope::seal(obs, fresh_provenance("repo:in-scope")).unwrap();
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(
        object_keys(&json),
        BTreeSet::from(["observation".into(), "provenance".into(), "digest".into(),])
    );
    assert_eq!(
        object_keys(&json["observation"]),
        BTreeSet::from(["evidenceType".into(), "facts".into(), "narrative".into(),])
    );
    assert_eq!(
        object_keys(&json["provenance"]),
        BTreeSet::from([
            "collectorId".into(),
            "collectedAt".into(),
            "scope".into(),
            "asset".into(),
        ])
    );
    for absent in [
        "evidenceId",
        "schemaVersion",
        "artifactRef",
        "validFrom",
        "validUntil",
        "supersedes",
        "sensitivity",
        "collectionRunId",
    ] {
        assert!(
            json.get(absent).is_none(),
            "EvidenceEnvelope currently has no `{absent}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn evidence_set_is_in_memory_digest_map_without_ledger_ops() {
    let obs = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_narrative("repository in-scope has branch_protection enabled");
    let env = EvidenceEnvelope::seal(obs, fresh_provenance("repo:in-scope")).unwrap();
    let mut set = EvidenceSet::new();
    set.insert(env.clone());
    set.insert(env);
    assert_eq!(set.len(), 1, "retry is idempotent by digest only");

    let src = crate_sources_joined("weeping-angel-evidence");
    for needle in [
        "rusqlite",
        "sqlite",
        "fn append",
        "fn supersede",
        "EvidenceLedger",
        "CollectionRun",
        "EvidenceArtifactRef",
        "set_compliant",
        "set_control_status",
    ] {
        assert!(
            !src.contains(needle),
            "evidence crate currently has no ledger/artifact API; found `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn seal_rejects_credential_keys_and_compliance_narratives() {
    let claim = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_narrative("ISO 27001 compliant");
    assert!(matches!(
        EvidenceEnvelope::seal(claim, fresh_provenance("repo:in-scope")),
        Err(EvidenceError::ComplianceClaim { .. })
    ));

    let secret = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_fact("token", "ghp_example")
        .with_narrative("ok");
    assert!(matches!(
        EvidenceEnvelope::seal(secret, fresh_provenance("repo:in-scope")),
        Err(EvidenceError::CredentialInPayload { .. })
    ));
}

// ── Collectors ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn collector_trait_is_sync_and_only_fixture_ships() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:in-scope"),
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_narrative("repository in-scope has branch_protection enabled"),
        );
    let desc: CollectorDescriptor = collector.descriptor();
    assert_eq!(desc.id, "fixture.github-like");
    assert_eq!(desc.version, "1.0.0");
    let desc_json = serde_json::to_value(&desc).unwrap();
    assert_eq!(
        object_keys(&desc_json),
        BTreeSet::from(["id".into(), "version".into(), "evidenceTypes".into()])
    );
    assert!(desc_json.get("frameworks").is_none());
    assert!(desc_json.get("capabilities").is_none());
    assert!(desc_json.get("providerFamily").is_none());

    let envelopes = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .unwrap();
    assert_eq!(envelopes.len(), 1);
    assert_eq!(
        envelopes[0].provenance().collected_at,
        Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap()
    );

    let src_root = crate_src("weeping-angel-collector");
    assert!(src_root.join("lib.rs").is_file());
    assert!(
        !src_root.join("github").exists(),
        "no GitHub collector module today"
    );
    let src = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "GitHubCollector",
        "octocrab",
        "LocalCollector",
        "ManualEvidence",
        "CollectorCapabilities",
        "CollectionBatch",
        "async fn collect",
    ] {
        assert!(
            !src.contains(needle),
            "collector crate currently ships only FixtureCollector; found `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn fixture_collector_rejects_out_of_scope_and_compliance_claims() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:out"),
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_narrative("ISO 27001 compliant"),
        );
    let err = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect_err("out of scope");
    assert!(matches!(err, CollectorError::OutOfScope { .. }));
}

// ── Control-test runtime ───────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn effectiveness_is_four_state_and_evaluator_has_no_expression_ast() {
    let _exhaustive = [
        Effectiveness::Effective,
        Effectiveness::Ineffective,
        Effectiveness::InsufficientEvidence,
        Effectiveness::Inconclusive,
    ];
    match Effectiveness::Effective {
        Effectiveness::Effective
        | Effectiveness::Ineffective
        | Effectiveness::InsufficientEvidence
        | Effectiveness::Inconclusive
        | _ => {}
    }

    let src = crate_sources_joined("weeping-angel-control-test");
    for needle in [
        "enum TestExpr",
        "FreshWithin",
        "CoverageAtLeast",
        "PartiallyEffective",
        "StaleEvidence",
        "ManualReviewRequired",
        "ExceptionApproved",
        "NotApplicable",
        "NotTested",
    ] {
        assert!(
            !src.contains(needle),
            "control-test crate is the four-state presence evaluator; found `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn evaluate_is_presence_break_freshness_and_manual_attestation() {
    let test = compiled_presence_test();
    let ctx = fresh_context();

    let empty = evaluate(&test, &EvidenceSet::new(), &ctx);
    assert_eq!(empty.effectiveness, Effectiveness::InsufficientEvidence);

    let mut good = EvidenceSet::new();
    good.insert(
        EvidenceEnvelope::seal(
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_fact("enabled", "true")
                .with_narrative("repository in-scope has branch_protection enabled"),
            fresh_provenance("repo:in-scope"),
        )
        .unwrap(),
    );
    let effective = evaluate(&test, &good, &ctx);
    assert_eq!(effective.effectiveness, Effectiveness::Effective);

    let mut broken = EvidenceSet::new();
    broken.insert(
        EvidenceEnvelope::seal(
            EvidenceObservation::new(EvidenceType::new("exposed_without_auth"))
                .with_narrative("route /admin is exposed_without_auth"),
            fresh_provenance("repo:in-scope"),
        )
        .unwrap(),
    );
    let ineffective = evaluate(&test, &broken, &ctx);
    assert_eq!(ineffective.effectiveness, Effectiveness::Ineffective);

    let stale_prov = EvidenceProvenance {
        collected_at: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        ..fresh_provenance("repo:in-scope")
    };
    let mut stale_set = EvidenceSet::new();
    stale_set.insert(
        EvidenceEnvelope::seal(
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_narrative("repository in-scope has branch_protection enabled"),
            stale_prov,
        )
        .unwrap(),
    );
    let stale = evaluate(&test, &stale_set, &ctx);
    assert_eq!(stale.effectiveness, Effectiveness::Inconclusive);

    let manual = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.manual",
        ))
        .control_id(ControlId::new("canonical.access-review"))
        .kind(ControlTestKind::Manual)
        .require(EvidenceType::new("manual_attestation"))
        .build();
    let no_attest = evaluate(&manual, &EvidenceSet::new(), &ctx);
    assert_eq!(no_attest.effectiveness, Effectiveness::InsufficientEvidence);
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn control_test_result_is_id_control_effectiveness_rationale() {
    let result = evaluate(
        &compiled_presence_test(),
        &EvidenceSet::new(),
        &fresh_context(),
    );
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(
        object_keys(&json),
        BTreeSet::from([
            "testId".into(),
            "controlId".into(),
            "effectiveness".into(),
            "rationale".into(),
        ])
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn control_test_result_denies_unknown_fields() {
    let parsed: Result<ControlTestResult, _> = serde_json::from_value(serde_json::json!({
        "testId": "t",
        "controlId": "c",
        "effectiveness": "effective",
        "rationale": "x",
        "evidenceRefs": [],
    }));
    assert!(
        parsed.is_err(),
        "ControlTestResult currently denies unknown fields such as evidenceRefs"
    );
}

// ── Mapping IR ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn mapping_is_direction_and_completeness_only() {
    let mapping = Mapping::new(
        RequirementId::new("canonical:stub-1"),
        ControlId::new("canonical.source-control"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert_eq!(mapping.direction(), MappingDirection::Forward);
    assert_eq!(mapping.completeness(), MappingCompleteness::Partial);
    let json = serde_json::to_value(&mapping).unwrap();
    assert_eq!(
        object_keys(&json),
        BTreeSet::from([
            "schemaVersion".into(),
            "fromRequirement".into(),
            "toControl".into(),
            "direction".into(),
            "completeness".into(),
        ])
    );
    for absent in ["relation", "rationale", "provenance", "version"] {
        assert!(
            json.get(absent).is_none(),
            "Mapping IR currently has no `{absent}` field"
        );
    }
}

// ── Scanner bridge ─────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn bridge_projects_security_finding_string_facts_and_is_one_way() {
    let hit = sample_hit();
    let before = serde_json::to_value(&hit.to_semantic_finding()).unwrap();
    let obs = bridge::from_engine_hit(&hit);
    assert_eq!(obs.evidence_type(), &EvidenceType::new("security_finding"));
    assert_eq!(
        obs.fact("rule_id"),
        Some("path-traversal.archive-extraction")
    );
    assert_eq!(obs.fact("path"), Some("src/extract.py"));
    assert_eq!(obs.fact("category"), Some("path-traversal"));
    assert_eq!(obs.narrative(), "Unsafe archive extraction");
    let after = serde_json::to_value(&hit.to_semantic_finding()).unwrap();
    assert_eq!(before, after);

    let from_finding = bridge::from_semantic_finding(&hit.to_semantic_finding());
    assert_eq!(
        from_finding.evidence_type(),
        &EvidenceType::new("security_finding")
    );
    assert_eq!(from_finding.fact("finding_id").is_some(), true);
    assert!(
        from_finding.fact("iso27001").is_none(),
        "bridge does not attach framework status"
    );
}

// ── CLI ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn commands_has_no_assurance_variant() {
    let cmd = Cli::clap_command();
    let names: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
    assert_eq!(
        names,
        vec![
            "scan",
            "finalize",
            "scan-code",
            "scan-diff",
            "workbench",
            "depcheck",
            "version",
            "completions",
        ]
    );
    assert!(
        !names
            .iter()
            .any(|n| n.contains("assurance") || n.contains("assess") || n.contains("soa")),
        "Commands currently has no assurance surface: {names:?}"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn clap_rejects_assurance_subcommand() {
    let err = Cli::try_parse_from(["weeping-angel", "assurance"])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("unrecognized subcommand") || err.contains("unexpected argument"),
        "expected clap to reject `assurance`, got: {err}"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_assurance_target"]
fn commands_match_is_exhaustive_without_assurance() {
    let samples = [
        vec![
            "weeping-angel",
            "scan",
            "example.com",
            "--i-own-this",
            "--allow-host",
            "example.com",
        ],
        vec!["weeping-angel", "finalize", "--scan-dir", "."],
        vec!["weeping-angel", "scan-code", ".", "-o", "out/code"],
        vec![
            "weeping-angel",
            "scan-diff",
            "--repo",
            ".",
            "-o",
            "out/diff",
        ],
        vec!["weeping-angel", "workbench", "list"],
        vec!["weeping-angel", "depcheck", "package.json"],
        vec!["weeping-angel", "version"],
        vec!["weeping-angel", "completions", "powershell"],
    ];
    let mut seen = BTreeSet::new();
    for argv in samples {
        let cli = Cli::try_parse_from(&argv).unwrap_or_else(|e| panic!("parse {argv:?}: {e}"));
        let tag = match cli.command {
            Commands::Scan(_) => "Scan",
            Commands::Finalize(_) => "Finalize",
            Commands::ScanCode(_) => "ScanCode",
            Commands::ScanDiff(_) => "ScanDiff",
            Commands::Workbench(_) => "Workbench",
            Commands::Depcheck(_) => "Depcheck",
            Commands::Version => "Version",
            Commands::Completions { .. } => "Completions",
            Commands::Assurance(_) => "Assurance",
        };
        seen.insert(tag);
    }
    assert_eq!(
        seen.len(),
        8,
        "expected all eight current Commands variants"
    );
}
