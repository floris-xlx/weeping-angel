//! Target suite: Phases 0–8 assurance runtime (ACT-001..015, COL-001..006).
//!
//! Encodes the *desired* Athena-shaped spine in `docs/sdd/assurance-runtime-spine.md`.
//! On the current scanner-only tree these crates/APIs do not exist — the suite
//! MUST be RED. After the spine lands it MUST go GREEN. Do not weaken assertions
//! to match today's product. Do not add `iso_27001` / `gdpr` / `soc2` onto findings.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::contract::{
    ArtifactRecord, Candidate, CoverageDocument, SemanticFinding, normalize_raw_candidate,
};
use weeping_angel::engines::EngineHit;

use weeping_angel_assurance::bridge;
use weeping_angel_assurance::{AssessmentReport, AssessmentScope, AssuranceEngine};
use weeping_angel_assurance_ir::crosswalk::ComplianceGraph;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentId, AssetId, AuditProgramId, Control, ControlId,
    ControlImplementationId, ControlTestId, EvidenceRequirement, EvidenceRequirementId,
    ExceptionId, FrameworkId, FrameworkVersion, IdentityId, Mapping, MappingCompleteness,
    MappingDirection, ProcessingActivityId, Requirement, RequirementId, RiskId, VendorId,
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
    FrameworkProfile, FrameworkTarget, compile_framework,
};

const FORBIDDEN_COMPLIANCE_KEYS: &[&str] = &[
    "iso27001",
    "iso_27001",
    "iso27701",
    "iso_27701",
    "iso27007",
    "iso_27007",
    "gdpr",
    "soc2",
    "soc_2",
    "nis2",
    "dora",
    "controlresult",
    "control_result",
    "controltestresult",
];

const FORBIDDEN_FRAMEWORK_PACKAGES: &[&str] = &[
    "reqwest",
    "hyper",
    "h2",
    "octocrab",
    "octorust",
    "cloudflare",
    "reqwest-middleware",
    "aws-config",
    "aws-sdk-s3",
    "aws-sdk-sts",
    "aws-sdk-iam",
    "aws-sdk-ec2",
    "aws-smithy-runtime",
    "tokio-tungstenite",
];

const PIPELINE_STAGES: &[&str] = &[
    "normalize",
    "resolve_applicability",
    "validate_capabilities",
    "resolve_control_mappings",
    "resolve_evidence_requirements",
    "construct_test_plan",
    "construct_framework_projection",
    "integrity_validation",
];

const ASSURANCE_PACKAGES: &[&str] = &[
    "weeping-angel-assurance-ir",
    "weeping-angel-framework",
    "weeping-angel-evidence",
    "weeping-angel-collector",
    "weeping-angel-control-test",
    "weeping-angel-assurance",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_object_keys(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                out.insert(key.clone());
                collect_object_keys(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_object_keys(child, out);
            }
        }
        _ => {}
    }
}

fn assert_no_forbidden_keys(label: &str, value: &Value) {
    let mut keys = BTreeSet::new();
    collect_object_keys(value, &mut keys);
    for key in &keys {
        let folded = key.to_ascii_lowercase().replace('-', "_");
        assert!(
            !FORBIDDEN_COMPLIANCE_KEYS.contains(&folded.as_str()),
            "{label} serialized a forbidden compliance key `{key}` (keys: {keys:?})"
        );
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

fn looks_like_compliance_claim(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("iso 27001 compliant")
        || lower.contains("iso27001 compliant")
        || lower.contains("gdpr compliant")
        || lower.contains("soc 2 compliant")
        || lower.contains("soc2 compliant")
        || lower.contains("controltestresult")
        || lower.contains("control test result")
}

fn stage_key(raw: &str) -> String {
    let n = raw.to_ascii_lowercase().replace([' ', '-'], "_");
    match n.as_str() {
        "applicability" | "resolve_applicability" => "resolve_applicability".into(),
        "capabilities" | "validate_capabilities" | "capability_validation" => {
            "validate_capabilities".into()
        }
        "mappings" | "resolve_control_mappings" | "resolve_mappings" => {
            "resolve_control_mappings".into()
        }
        "evidence_requirements" | "resolve_evidence_requirements" => {
            "resolve_evidence_requirements".into()
        }
        "test_plan" | "construct_test_plan" => "construct_test_plan".into(),
        "projection" | "construct_framework_projection" | "framework_projection" => {
            "construct_framework_projection".into()
        }
        "integrity" | "integrity_validation" | "digest" => "integrity_validation".into(),
        other => other.to_string(),
    }
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
    serde_json::from_slice(&output.stdout).expect("metadata json")
}

fn package_map(meta: &Value) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    for pkg in meta["packages"].as_array().expect("packages") {
        if let Some(name) = pkg["name"].as_str() {
            out.insert(name.to_string(), pkg.clone());
        }
    }
    out
}

fn package_id_refers_to(id: &str, name: &str) -> bool {
    id.split_whitespace().next() == Some(name)
        || id.starts_with(&format!("{name} "))
        || id.contains(&format!("/{name}#"))
        || id.contains(&format!("#{name}@"))
}

fn package_name_from_id(id: &str) -> Option<String> {
    if let Some(first) = id.split_whitespace().next() {
        if !first.contains('/') && !first.contains(':') && !first.contains('+') {
            return Some(first.to_string());
        }
    }
    if let Some((_, after_hash)) = id.rsplit_once('#') {
        if let Some((name, _)) = after_hash.split_once('@') {
            return Some(name.to_string());
        }
        let path = id.rsplit_once('#')?.0;
        return path.rsplit('/').next().map(str::to_string);
    }
    None
}

fn resolve_pkg_id<'a>(resolve: &'a Value, name: &str) -> Option<&'a str> {
    resolve["nodes"].as_array()?.iter().find_map(|node| {
        let id = node["id"].as_str()?;
        if package_id_refers_to(id, name) {
            Some(id)
        } else {
            None
        }
    })
}

fn resolved_dep_names(resolve: &Value, pkg_name: &str) -> BTreeSet<String> {
    let Some(nodes) = resolve["nodes"].as_array() else {
        return BTreeSet::new();
    };
    let Some(id) = resolve_pkg_id(resolve, pkg_name) else {
        return BTreeSet::new();
    };
    let Some(node) = nodes.iter().find(|n| n["id"].as_str() == Some(id)) else {
        return BTreeSet::new();
    };
    node["deps"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| {
            d["pkg"]
                .as_str()
                .and_then(package_name_from_id)
                .or_else(|| d["name"].as_str().map(|n| n.replace('_', "-")))
        })
        .collect()
}

fn is_forbidden_network_dep(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    FORBIDDEN_FRAMEWORK_PACKAGES
        .iter()
        .any(|f| n == *f || n.starts_with("aws-sdk-") || n.starts_with("aws-smithy-"))
        || n.contains("octokit")
        || n.contains("octocrab")
        || n.contains("cloudflare")
}

fn fresh_provenance(asset: &str) -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.github-like".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new(asset),
    }
}

fn branch_protection_obs(enabled: bool) -> EvidenceObservation {
    EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_fact("enabled", if enabled { "true" } else { "false" })
        .with_narrative("repository in-scope has branch_protection enabled")
}

fn capabilities_for_soa() -> FrameworkCapabilities {
    FrameworkCapabilities {
        supports_statement_of_applicability: true,
        supports_control_applicability: true,
        supports_manual_attestation: true,
        ..FrameworkCapabilities::default()
    }
}

fn stub_assessment(request_soa: bool) -> Assessment {
    let requirement = Requirement::new(
        RequirementId::new("iso27001:2022:stub-1"),
        FrameworkId::new("iso-27001"),
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
    let mut assessment = Assessment::new(AssessmentId::new("assess-stub-1"));
    assessment.requirements = vec![requirement];
    assessment.controls = vec![control];
    assessment.mappings = vec![mapping];
    assessment.evidence_requirements = vec![evidence_req];
    assessment.requests = AssessmentRequests {
        statement_of_applicability: request_soa,
        ..AssessmentRequests::default()
    };
    assessment
}

fn compiled_branch_test() -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.branch-protection"))
        .control_id(ControlId::new("canonical.source-control"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("branch_protection"))
        .break_on(EvidenceType::new("exposed_without_auth"))
        .build()
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn assert_digest_stable(digest: &str) {
    assert!(!digest.is_empty(), "integrity digest must be non-empty");
    assert!(
        digest.len() >= 32,
        "digest should be a stable hash, got {digest:?}"
    );
}

// ── ACT-001 ────────────────────────────────────────────────────────────────

#[test]
fn act_001_finding_is_not_a_compliance_result() {
    let finding = sample_hit().to_semantic_finding();
    let value = serde_json::to_value(&finding).unwrap();
    assert_no_forbidden_keys("SemanticFinding", &value);
    assert!(
        value.get("effectiveness").is_none(),
        "SemanticFinding must not carry control effectiveness"
    );
    assert!(
        value.get("controlId").is_none() && value.get("control_id").is_none(),
        "SemanticFinding must not grow a control id"
    );

    let parsed: Result<ControlTestResult, _> = serde_json::from_value(value.clone());
    assert!(
        parsed.is_err(),
        "ACT-001: a SemanticFinding must not deserialize as ControlTestResult"
    );

    // Type/API reject: findings cannot be evaluated as control results.
    let mut empty = EvidenceSet::new();
    let _ = &mut empty;
    let ctx = fresh_context();
    let test = compiled_branch_test();
    let result = evaluate(&test, &EvidenceSet::new(), &ctx);
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "ACT-001: an empty evidence set (a finding was never a result) cannot be Effective"
    );
}

// ── ACT-002 ────────────────────────────────────────────────────────────────

#[test]
fn act_002_collector_cannot_declare_compliance() {
    let prov = fresh_provenance("repo:in-scope");
    let claim = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_narrative("ISO 27001 compliant");
    let sealed = EvidenceEnvelope::seal(claim, prov.clone());
    assert!(
        matches!(sealed, Err(EvidenceError::ComplianceClaim { .. })),
        "ACT-002: sealing a compliance sentence must fail, got {sealed:?}"
    );

    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:in-scope"),
            EvidenceObservation::new(EvidenceType::new("branch_protection"))
                .with_narrative("this organization is SOC 2 compliant"),
        );
    let err = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect_err("ACT-002: collector returning a compliance sentence must fail");
    assert!(
        matches!(
            err,
            CollectorError::ComplianceClaim { .. } | CollectorError::FrameworkResult { .. }
        ),
        "ACT-002: expected compliance/framework reject, got {err:?}"
    );
}

// ── ACT-003 ────────────────────────────────────────────────────────────────

#[test]
fn act_003_framework_crate_has_no_network_or_sdk_deps() {
    let meta = cargo_metadata();
    let resolve = &meta["resolve"];
    let deps = resolved_dep_names(resolve, "weeping-angel-framework");
    assert!(
        !deps.is_empty() || package_map(&meta).contains_key("weeping-angel-framework"),
        "ACT-003: weeping-angel-framework must be a workspace package"
    );
    let forbidden: Vec<_> = deps
        .iter()
        .filter(|d| is_forbidden_network_dep(d))
        .cloned()
        .collect();
    assert!(
        forbidden.is_empty(),
        "ACT-003 / INV-3: framework must not depend on network/SDK crates: {forbidden:?} (deps={deps:?})"
    );
    assert!(
        deps.iter().any(|d| d == "weeping-angel-assurance-ir"),
        "framework must depend on weeping-angel-assurance-ir"
    );
}

// ── ACT-004 ────────────────────────────────────────────────────────────────

#[test]
fn act_004_control_test_is_provider_blind() {
    let test = compiled_branch_test();
    let env = EvidenceEnvelope::seal(
        branch_protection_obs(true),
        fresh_provenance("repo:in-scope"),
    )
    .expect("valid observation");
    let mut set = EvidenceSet::new();
    set.insert(env);
    let set_json = serde_json::to_value(&set).unwrap();
    let mut keys = BTreeSet::new();
    collect_object_keys(&set_json, &mut keys);
    for key in &keys {
        let folded = key.to_ascii_lowercase();
        assert!(
            !folded.contains("provider")
                && !folded.contains("github")
                && !folded.contains("octokit")
                && folded != "collector_client",
            "ACT-004 / INV-4: EvidenceSet must not carry provider identity (`{key}`)"
        );
    }

    let ctx = fresh_context();
    let result = evaluate(&test, &set, &ctx);
    let result_json = serde_json::to_value(&result).unwrap();
    let mut rkeys = BTreeSet::new();
    collect_object_keys(&result_json, &mut rkeys);
    for key in &rkeys {
        let folded = key.to_ascii_lowercase();
        assert!(
            !folded.contains("provider_id") && !folded.contains("githubclient"),
            "ACT-004: ControlTestResult must not name a provider (`{key}`)"
        );
    }

    let src_root = [
        manifest_dir().join("crates/weeping-angel-control-test"),
        manifest_dir().join("weeping-angel-control-test"),
    ]
    .into_iter()
    .find(|p| p.is_dir())
    .expect("ACT-004: weeping-angel-control-test crate directory");
    let mut rust_files = Vec::new();
    walk_rs_files(&src_root, &mut rust_files);
    let mut hits = Vec::new();
    for path in rust_files {
        let text = std::fs::read_to_string(&path).unwrap();
        for needle in ["GitHubClient", "Octokit", "AwsClient", "provider_id:"] {
            if text.contains(needle) {
                hits.push(format!("{}: {needle}", path.display()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "ACT-004: control-test crate must not mention provider clients: {hits:?}"
    );
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
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

// ── ACT-005 ────────────────────────────────────────────────────────────────

#[test]
fn act_005_crosswalk_preserves_direction_and_refuses_partial_equivalence() {
    let a = RequirementId::new("framework-a:r1");
    let b = RequirementId::new("framework-b:r1");
    let c = RequirementId::new("framework-c:r1");
    let mut graph = ComplianceGraph::new();
    graph.link(
        a.clone(),
        b.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    graph.link(
        b.clone(),
        c.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );

    assert!(
        !graph.equivalent(&a, &c),
        "ACT-005 / INV-5: A —partial→ B —partial→ C must not yield A ≡ C"
    );
    assert!(
        !graph.equivalent(&a, &b),
        "partial mapping is never equivalent"
    );
    assert!(
        !graph.equivalent(&c, &a),
        "direction must be preserved; reverse path is not invented"
    );
    assert_eq!(graph.maps(&a, &b), Some(MappingCompleteness::Partial));
    assert_eq!(
        graph.maps(&b, &a),
        None,
        "ACT-005: forward partial must not imply reverse"
    );

    let x = RequirementId::new("fw-x:r1");
    let y = RequirementId::new("fw-y:r1");
    let mut full = ComplianceGraph::new();
    full.link(
        x.clone(),
        y.clone(),
        MappingDirection::Bidirectional,
        MappingCompleteness::Full,
    );
    assert!(
        full.equivalent(&x, &y),
        "explicit full bidirectional mapping is equivalent"
    );
}

// ── ACT-006 ────────────────────────────────────────────────────────────────

#[test]
fn act_006_ir_ids_control_has_no_iso_fields_requirement_is_not_control() {
    let _ = (
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        RequirementId::new("iso27001:2022:A.8.2"),
        ControlId::new("canonical.source-control"),
        ControlImplementationId::new("impl.source-control.github"),
        ControlTestId::new("test.branch-protection"),
        AssetId::new("repo:in-scope"),
        IdentityId::new("identity:alice"),
        VendorId::new("vendor:acme"),
        ProcessingActivityId::new("ropa:payroll"),
        EvidenceRequirementId::new("ev.branch_protection"),
        RiskId::new("risk:source-tamper"),
        ExceptionId::new("exc:1"),
        AssessmentId::new("assess-1"),
        AuditProgramId::new("audit:2026"),
    );

    let control = Control::new(
        ControlId::new("canonical.source-control"),
        "Source control",
        "Protect the authoritative software source.",
    );
    assert_eq!(control.schema_version(), ASSURANCE_IR_SCHEMA);
    let control_json = serde_json::to_value(&control).unwrap();
    assert_no_forbidden_keys("Control", &control_json);
    let mut keys = BTreeSet::new();
    collect_object_keys(&control_json, &mut keys);
    for key in &keys {
        let folded = key.to_ascii_lowercase().replace('-', "_");
        assert!(
            !folded.contains("annex")
                && !folded.contains("soa")
                && !folded.contains("clause")
                && !folded.contains("iso27001"),
            "ACT-006: Control must not carry ISO-specific field `{key}`"
        );
    }

    let requirement = Requirement::new(
        RequirementId::new("iso27001:2022:stub-1"),
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        "Stub requirement",
        "Protect the authoritative source of software.",
    );
    assert_ne!(
        std::any::TypeId::of::<Requirement>(),
        std::any::TypeId::of::<Control>(),
        "ACT-006: Requirement and Control are distinct types"
    );
    let mapping = Mapping::new(
        requirement.id().clone(),
        control.id().clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert_eq!(mapping.from_requirement(), requirement.id());
    assert_eq!(mapping.to_control(), control.id());
    assert_eq!(mapping.completeness(), MappingCompleteness::Partial);
    assert_ne!(
        mapping.completeness(),
        MappingCompleteness::Full,
        "ACT-006: Mapping must not collapse into identity/full by default"
    );

    let ir_src = [
        manifest_dir().join("crates/weeping-angel-assurance-ir"),
        manifest_dir().join("weeping-angel-assurance-ir"),
    ]
    .into_iter()
    .find(|p| p.is_dir())
    .expect("ACT-006: weeping-angel-assurance-ir crate");
    let mut files = Vec::new();
    walk_rs_files(&ir_src, &mut files);
    let joined = files
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "octocrab",
        "aws_sdk",
        "cloudflare",
        "reqwest::",
        "GitHubClient",
    ] {
        assert!(
            !joined.contains(forbidden),
            "ACT-006: IR must not mention provider/SDK type {forbidden}"
        );
    }
}

// ── ACT-007 ────────────────────────────────────────────────────────────────

#[test]
fn act_007_missing_capability_is_fail_closed_violation() {
    let target = FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    };
    assert!(
        !target.capabilities.supports_statement_of_applicability
            && !target.capabilities.supports_control_applicability
            && !target.capabilities.supports_privacy_processing
            && !target.capabilities.supports_risk_treatment
            && !target.capabilities.supports_manual_attestation
            && !target.capabilities.supports_sampling
            && !target.capabilities.supports_audit_program
            && !target.capabilities.supports_nonconformities,
        "Default capabilities are fail-closed (all false)"
    );

    let err = compile_framework(&stub_assessment(true), &target)
        .expect_err("ACT-007: SoA without supports_statement_of_applicability must fail");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            let folded = capability.to_ascii_lowercase();
            assert!(
                folded.contains("statement_of_applicability")
                    || folded.contains("soa")
                    || folded.contains("supports_statement_of_applicability"),
                "ACT-007: violation must name the missing flag, got {capability}"
            );
        }
        other => panic!("ACT-007: expected CapabilityViolation, got {other:?}"),
    }
}

// ── ACT-008 ────────────────────────────────────────────────────────────────

#[test]
fn act_008_compile_framework_pipeline_and_compiled_shape() {
    let target = FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: capabilities_for_soa(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    };
    let compiled = compile_framework(&stub_assessment(true), &target)
        .expect("ACT-008: compile_framework must succeed when capabilities match");

    let _ = &compiled.applicable_requirements;
    let _ = &compiled.controls;
    let _ = &compiled.tests;
    let _ = &compiled.evidence_requirements;
    let _ = &compiled.validation;
    assert_digest_stable(&compiled.digest);

    let stages: Vec<String> = compiled
        .validation
        .stages
        .iter()
        .map(|s| stage_key(s))
        .collect();
    assert_eq!(
        stages, PIPELINE_STAGES,
        "ACT-008: compile pipeline must be the eight Athena-shaped stages in order"
    );
    assert!(
        compiled.validation.ok,
        "successful compile records validation.ok"
    );

    let again = compile_framework(&stub_assessment(true), &target).unwrap();
    assert_eq!(
        compiled.digest, again.digest,
        "ACT-008: digest is deterministic over canonical serialization"
    );

    let unknown = FrameworkProfile::try_from("pci-dss");
    assert!(
        unknown.is_err(),
        "ACT-008: unknown profile is a typed reject, not a panic"
    );
}

// ── ACT-009 ────────────────────────────────────────────────────────────────

#[test]
fn act_009_evidence_envelope_is_immutable_and_not_a_claim() {
    let obs = branch_protection_obs(true);
    let env = EvidenceEnvelope::seal(obs.clone(), fresh_provenance("repo:in-scope"))
        .expect("ACT-009: observation is not a compliance claim");
    assert!(!looks_like_compliance_claim(env.observation().narrative()));
    let digest = env.digest().to_string();
    assert_digest_stable(&digest);

    let mutated = obs
        .clone()
        .with_narrative("repository in-scope has branch_protection disabled");
    let env2 = EvidenceEnvelope::seal(mutated, fresh_provenance("repo:in-scope")).unwrap();
    assert_ne!(
        digest,
        env2.digest(),
        "ACT-009: digest must change when the payload changes"
    );

    let claim = EvidenceObservation::new(EvidenceType::new("branch_protection"))
        .with_narrative("GDPR compliant");
    assert!(
        EvidenceEnvelope::seal(claim, fresh_provenance("repo:in-scope")).is_err(),
        "ACT-009: observation text must not be a compliance claim"
    );

    let again = EvidenceEnvelope::seal(obs, fresh_provenance("repo:in-scope")).unwrap();
    assert_eq!(
        digest,
        again.digest(),
        "ACT-009: sealing the same payload+provenance is deterministic"
    );
}

// ── ACT-010 ────────────────────────────────────────────────────────────────

#[test]
fn act_010_collector_descriptor_has_evidence_types_not_frameworks() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0").with_evidence_types([
        EvidenceType::new("branch_protection"),
        EvidenceType::new("repository_visibility"),
    ]);
    let desc: CollectorDescriptor = collector.descriptor();
    assert_eq!(desc.id, "fixture.github-like");
    assert_eq!(desc.version, "1.0.0");
    assert!(
        desc.evidence_types
            .contains(&EvidenceType::new("branch_protection"))
    );
    assert!(
        desc.evidence_types
            .contains(&EvidenceType::new("repository_visibility"))
    );

    let value = serde_json::to_value(&desc).unwrap();
    let obj = value.as_object().expect("descriptor object");
    assert!(
        obj.get("frameworks").is_none(),
        "ACT-010: CollectorDescriptor.frameworks is INVALID"
    );
    let keys: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    assert!(
        keys.contains("evidenceTypes") || keys.contains("evidence_types"),
        "ACT-010: descriptor must advertise evidence_types, keys={keys:?}"
    );
}

// ── ACT-011 ────────────────────────────────────────────────────────────────

#[test]
fn act_011_bridge_projects_observation_without_rewriting_to_semantic_finding() {
    let hit = sample_hit();
    let before = serde_json::to_value(&hit.to_semantic_finding()).unwrap();
    let obs = bridge::from_engine_hit(&hit);
    assert_eq!(
        obs.evidence_type(),
        &EvidenceType::new("security_finding"),
        "ACT-011: scanner hits become security_finding observations, not framework results"
    );
    assert!(
        !looks_like_compliance_claim(obs.narrative()),
        "bridge must not emit a compliance sentence"
    );
    assert!(
        obs.narrative().contains("Unsafe archive")
            || obs.fact("rule_id") == Some("path-traversal.archive-extraction"),
        "observation must retain security evidence from the hit"
    );

    let after = serde_json::to_value(&hit.to_semantic_finding()).unwrap();
    assert_eq!(
        before, after,
        "ACT-011: to_semantic_finding stays security-only and is not rewritten by the bridge"
    );
    assert_no_forbidden_keys("to_semantic_finding after bridge", &after);
    let ext = after["extensions"].as_object().expect("extensions");
    let ext_keys: BTreeSet<&str> = ext.keys().map(String::as_str).collect();
    assert_eq!(
        ext_keys,
        BTreeSet::from(["engine", "snippet", "validationMethod"])
    );

    let semantic = hit.to_semantic_finding();
    let from_finding = bridge::from_semantic_finding(&semantic);
    assert_eq!(
        from_finding.evidence_type(),
        &EvidenceType::new("security_finding")
    );
    assert!(!looks_like_compliance_claim(from_finding.narrative()));
}

// ── ACT-012 ────────────────────────────────────────────────────────────────

#[test]
fn act_012_control_test_fail_closed_and_may_be_ineffective() {
    let test = compiled_branch_test();
    let ctx = fresh_context();

    let empty = evaluate(&test, &EvidenceSet::new(), &ctx);
    assert_ne!(
        empty.effectiveness,
        Effectiveness::Effective,
        "ACT-012: missing evidence cannot be Effective"
    );
    assert!(
        matches!(
            empty.effectiveness,
            Effectiveness::InsufficientEvidence | Effectiveness::Inconclusive
        ),
        "empty/missing → InsufficientEvidence or Inconclusive, got {:?}",
        empty.effectiveness
    );

    let absent = EvidenceObservation::new(EvidenceType::new("security_findings_absent"))
        .with_narrative("scan completed with no vulnerabilities");
    let mut no_vuln = EvidenceSet::new();
    no_vuln.insert(EvidenceEnvelope::seal(absent, fresh_provenance("repo:in-scope")).unwrap());
    let no_vuln_result = evaluate(&test, &no_vuln, &ctx);
    assert_ne!(
        no_vuln_result.effectiveness,
        Effectiveness::Effective,
        "ACT-012: absence of a vuln does not prove Effective"
    );

    let stale_prov = EvidenceProvenance {
        collected_at: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        ..fresh_provenance("repo:in-scope")
    };
    let mut stale_set = EvidenceSet::new();
    stale_set.insert(EvidenceEnvelope::seal(branch_protection_obs(true), stale_prov).unwrap());
    let stale = evaluate(&test, &stale_set, &ctx);
    assert_ne!(
        stale.effectiveness,
        Effectiveness::Effective,
        "ACT-012: stale evidence cannot be Effective"
    );

    let manual = CompiledControlTest::builder()
        .id(ControlTestId::new("test.manual-access-review"))
        .control_id(ControlId::new("canonical.access-review"))
        .kind(ControlTestKind::Manual)
        .require(EvidenceType::new("manual_attestation"))
        .build();
    let auto_pass = evaluate(&manual, &EvidenceSet::new(), &ctx);
    assert_ne!(
        auto_pass.effectiveness,
        Effectiveness::Effective,
        "ACT-012: manual controls cannot auto-pass"
    );

    let break_obs = EvidenceObservation::new(EvidenceType::new("exposed_without_auth"))
        .with_narrative("route /admin is exposed_without_auth");
    let mut broken = EvidenceSet::new();
    broken.insert(EvidenceEnvelope::seal(break_obs, fresh_provenance("repo:in-scope")).unwrap());
    let ineffective = evaluate(&test, &broken, &ctx);
    assert_eq!(
        ineffective.effectiveness,
        Effectiveness::Ineffective,
        "ACT-012: a breaking observation may prove Ineffective"
    );

    let mut good = EvidenceSet::new();
    good.insert(
        EvidenceEnvelope::seal(
            branch_protection_obs(true),
            fresh_provenance("repo:in-scope"),
        )
        .unwrap(),
    );
    let effective = evaluate(&test, &good, &ctx);
    assert_eq!(
        effective.effectiveness,
        Effectiveness::Effective,
        "fresh matching observation can be Effective"
    );

    let ct_root = [
        manifest_dir().join("crates/weeping-angel-control-test"),
        manifest_dir().join("weeping-angel-control-test"),
    ]
    .into_iter()
    .find(|p| p.is_dir())
    .expect("control-test crate");
    let cargo = std::fs::read_to_string(ct_root.join("Cargo.toml")).unwrap();
    for needle in ["reqwest", "octocrab", "aws-sdk", "cloudflare"] {
        assert!(
            !cargo.contains(needle),
            "ACT-012: control-test has zero network I/O; found `{needle}` in Cargo.toml"
        );
    }
}

// ── ACT-013 ────────────────────────────────────────────────────────────────

#[test]
fn act_013_crate_graph_matches_sdd_section_13() {
    let meta = cargo_metadata();
    let packages = package_map(&meta);
    for name in ASSURANCE_PACKAGES {
        assert!(
            packages.contains_key(*name),
            "ACT-013: workspace must include `{name}`"
        );
    }

    let resolve = &meta["resolve"];
    let ir = resolved_dep_names(resolve, "weeping-angel-assurance-ir");
    let framework = resolved_dep_names(resolve, "weeping-angel-framework");
    let evidence = resolved_dep_names(resolve, "weeping-angel-evidence");
    let collector = resolved_dep_names(resolve, "weeping-angel-collector");
    let control_test = resolved_dep_names(resolve, "weeping-angel-control-test");
    let facade = resolved_dep_names(resolve, "weeping-angel-assurance");

    assert!(
        !ir.contains("weeping-angel-framework")
            && !ir.contains("weeping-angel-collector")
            && !ir.contains("weeping-angel-assurance"),
        "assurance-ir must not depend on upper crates: {ir:?}"
    );
    assert!(
        framework.contains("weeping-angel-assurance-ir"),
        "framework → assurance-ir"
    );
    assert!(
        !framework.contains("weeping-angel-collector")
            && !framework.contains("weeping-angel-control-test"),
        "framework must not depend on collector/control-test: {framework:?}"
    );
    assert!(
        evidence.contains("weeping-angel-assurance-ir"),
        "evidence → assurance-ir"
    );
    assert!(
        collector.contains("weeping-angel-evidence"),
        "collector → evidence"
    );
    assert!(
        !collector.contains("weeping-angel-framework"),
        "collector must not depend on framework/ISO catalogs"
    );
    assert!(
        control_test.contains("weeping-angel-assurance-ir")
            && control_test.contains("weeping-angel-evidence"),
        "control-test → ir + evidence"
    );
    assert!(
        !control_test.contains("weeping-angel-collector"),
        "control-test must not depend on collector"
    );
    assert!(
        facade.contains("weeping-angel-framework")
            && facade.contains("weeping-angel-collector")
            && facade.contains("weeping-angel-control-test"),
        "facade → framework + collector + control-test (got {facade:?})"
    );

    for dep in collector.iter() {
        let folded = dep.to_ascii_lowercase();
        assert!(
            !folded.contains("iso27001")
                && !folded.contains("gdpr")
                && !folded.contains("soc2")
                && !folded.contains("nis2")
                && !folded.contains("dora"),
            "ACT-013: collector must not depend on ISO/GDPR/SOC2 types (`{dep}`)"
        );
    }
}

// ── ACT-014 ────────────────────────────────────────────────────────────────

#[test]
fn act_014_facade_assess_does_not_branch_on_framework_implementations() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(AssetId::new("repo:in-scope"), branch_protection_obs(true));
    let scope = AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope"));

    let mut reports: Vec<AssessmentReport> = Vec::new();
    for profile in [
        FrameworkProfile::Iso27001,
        FrameworkProfile::Iso27701,
        FrameworkProfile::Gdpr,
        FrameworkProfile::Soc2,
        FrameworkProfile::Nis2,
        FrameworkProfile::Dora,
        FrameworkProfile::Iso27007,
    ] {
        let target = FrameworkTarget {
            profile,
            capabilities: capabilities_for_soa(),
            version: FrameworkVersion::new("1"),
            context: FrameworkContext::default(),
        };
        let report = AssuranceEngine::builder()
            .collector(collector.clone())
            .framework(target)
            .definition(stub_assessment(false))
            .assess(scope.clone())
            .expect("ACT-014: generic assess(scope) must run for every profile selector");
        let json = serde_json::to_value(&report).unwrap();
        assert!(
            json.get("compilerTopology").is_none() && json.get("collectorGraph").is_none(),
            "ACT-014: compiler/collector topology is debug-only, not the public report"
        );
        reports.push(report);
    }
    assert_eq!(reports.len(), 7);
}

// ── ACT-015 ────────────────────────────────────────────────────────────────

#[test]
fn act_015_security_domain_types_remain_uncollapsed() {
    let hit = sample_hit();
    let semantic: SemanticFinding = hit.to_semantic_finding();
    let semantic_json = serde_json::to_value(&semantic).unwrap();
    assert_no_forbidden_keys("SemanticFinding", &semantic_json);
    for required in [
        "findingId",
        "occurrenceId",
        "ruleId",
        "identity",
        "fingerprints",
        "title",
        "summary",
        "severity",
        "confidence",
        "taxonomy",
        "locations",
        "remediation",
        "provenance",
    ] {
        assert!(
            semantic_json.get(required).is_some(),
            "ACT-015: SemanticFinding missing {required}"
        );
    }

    let mut scope = BTreeSet::new();
    scope.insert("src/extract.py".into());
    let candidate: Candidate = normalize_raw_candidate(
        &json!({
            "cwe_ids": ["CWE-22"],
            "locations": [{"path": "src/extract.py", "start_line": 41, "role": "sink"}],
            "summary": "path write",
            "evidence": "open(...)",
        }),
        &scope,
    )
    .unwrap();
    assert_eq!(candidate.cwe_ids, vec!["CWE-22".to_string()]);
    assert_no_forbidden_keys("Candidate", &serde_json::to_value(&candidate).unwrap());

    let artifact = ArtifactRecord {
        path: "findings.json".into(),
        sha256: "0".repeat(64),
        media_type: "application/json".into(),
    };
    let artifact_json = serde_json::to_value(&artifact).unwrap();
    let artifact_keys: BTreeSet<&str> = artifact_json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        artifact_keys,
        BTreeSet::from(["path", "sha256", "mediaType"])
    );

    let coverage_path = manifest_dir().join("tests/fixtures/completed-scan/coverage.json");
    let coverage: CoverageDocument =
        serde_json::from_str(&std::fs::read_to_string(coverage_path).unwrap()).unwrap();
    assert_eq!(coverage.document_type, "codex-security.coverage");
    assert_no_forbidden_keys(
        "CoverageDocument",
        &serde_json::to_value(&coverage).unwrap(),
    );

    let rejected = normalize_raw_candidate(
        &json!({
            "cwe_ids": ["CWE-22"],
            "locations": [{"path": "src/extract.py", "start_line": 41}],
            "summary": "path write",
            "evidence": "open(...)",
            "iso_27001": "A.8.2",
        }),
        &scope,
    );
    assert!(
        rejected.is_err(),
        "ACT-015: Candidate must still fail-close on a framework field"
    );
}

// ── COL-001..006 ───────────────────────────────────────────────────────────

#[test]
fn col_001_emit_only_declared_evidence_types() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:in-scope"),
            EvidenceObservation::new(EvidenceType::new("repository_visibility"))
                .with_narrative("repository is private"),
        );
    let err = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect_err("COL-001: undeclared evidence type must fail");
    assert!(
        matches!(err, CollectorError::UndeclaredEvidenceType { .. }),
        "COL-001: expected UndeclaredEvidenceType, got {err:?}"
    );
}

#[test]
fn col_002_no_framework_results_in_collector_output() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(AssetId::new("repo:in-scope"), branch_protection_obs(true));
    let envelopes = collector
        .collect(&CollectorScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .unwrap();
    assert!(!envelopes.is_empty());
    for env in &envelopes {
        let json = serde_json::to_value(env).unwrap();
        assert_no_forbidden_keys("EvidenceEnvelope", &json);
        assert!(
            !looks_like_compliance_claim(env.observation().narrative()),
            "COL-002: collector output must not be a framework result"
        );
        assert_ne!(
            env.observation().evidence_type(),
            &EvidenceType::new("control_test_result")
        );
    }
}

#[test]
fn col_003_no_credentials_in_payloads() {
    let prov = fresh_provenance("repo:in-scope");
    for (key, value) in [
        ("authorization", "Bearer ghp_exampletoken"),
        ("token", "secret-token"),
        ("cookie", "session=abc"),
        ("password", "hunter2"),
        ("api_key", "sk_live_123"),
    ] {
        let obs = EvidenceObservation::new(EvidenceType::new("branch_protection"))
            .with_fact(key, value)
            .with_narrative("repository has branch_protection enabled");
        let sealed = EvidenceEnvelope::seal(obs, prov.clone());
        assert!(
            matches!(sealed, Err(EvidenceError::CredentialInPayload { .. })),
            "COL-003: fact `{key}` must be rejected, got {sealed:?}"
        );
    }
}

#[test]
fn col_004_normalize_is_deterministic() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(AssetId::new("repo:in-scope"), branch_protection_obs(true));
    let scope = CollectorScope::new().allow_asset(AssetId::new("repo:in-scope"));
    let a = collector.collect(&scope).unwrap();
    let b = collector.collect(&scope).unwrap();
    let da: Vec<_> = a.iter().map(|e| e.digest().to_string()).collect();
    let db: Vec<_> = b.iter().map(|e| e.digest().to_string()).collect();
    assert_eq!(da, db, "COL-004: same fixture twice → same digest");
}

#[test]
fn col_005_retry_does_not_duplicate_immutable_evidence() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(AssetId::new("repo:in-scope"), branch_protection_obs(true));
    let scope = CollectorScope::new().allow_asset(AssetId::new("repo:in-scope"));
    let first = collector.collect(&scope).unwrap();
    let second = collector.collect(&scope).unwrap();
    let mut set = EvidenceSet::new();
    for env in first.iter().chain(second.iter()) {
        set.insert(env.clone());
    }
    assert_eq!(
        set.len(),
        first.len(),
        "COL-005: retry must be idempotent by digest (set semantics)"
    );
}

#[test]
fn col_006_scope_is_fail_closed() {
    let collector = FixtureCollector::new("fixture.github-like", "1.0.0")
        .with_evidence_types([EvidenceType::new("branch_protection")])
        .with_planned(
            AssetId::new("repo:out-of-scope"),
            branch_protection_obs(true),
        );
    let scope = CollectorScope::new().allow_asset(AssetId::new("repo:in-scope"));
    match collector.collect(&scope) {
        Err(CollectorError::OutOfScope { .. }) => {}
        Ok(envelopes) => {
            assert!(
                envelopes
                    .iter()
                    .all(|e| e.provenance().asset() != &AssetId::new("repo:out-of-scope")),
                "COL-006: must never silently collect an out-of-scope asset"
            );
            assert!(
                envelopes.is_empty(),
                "COL-006: omit is allowed only with no collected envelopes (explicit denial)"
            );
        }
        Err(other) => panic!("COL-006: unexpected error {other:?}"),
    }
}
