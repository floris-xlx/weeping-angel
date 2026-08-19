//! SUPERSEDED by `sdd_applicability_engine_target`.
//!
//! Historical static-only characterization on SHA
//! `e430980c0d27a8138a153d49b62ddf3c57827891` (`docs/specs/applicability-engine.md`
//! §3 / §6.1). Kleene evaluator + snapshot are now the SSOT in the target
//! suite. Tests are `#[ignore]` so CI does not require the old absence and
//! static-only characterization. Dual-suite registration remains.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance::project_soa;
use weeping_angel_assurance_ir::{
    ApplicabilityPredicate, ApplicabilityRule, AssessmentDefinition, AssessmentId, AssessmentScope,
    Asset, AssetKind, Control, ControlId, ControlTestId, EvidenceType, FrameworkId,
    FrameworkVersion, Identity, IdentityKind, ProcessingActivity, ProcessingActivityId,
    Requirement, RequirementId, Risk, RiskId, ScopeExclusion, Vendor, VendorId,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, Population, PopulationCompleteness, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_framework::{
    Assessment, FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
    compile_framework,
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

fn product_crates_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn fn_resolve_applicability(src: &str) -> &str {
    let start = src
        .find("fn resolve_applicability(")
        .expect("resolve_applicability must exist");
    let rest = &src[start..];
    let end = rest
        .find("\nfn validate_capabilities")
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_project_soa(src: &str) -> &str {
    let start = src
        .find("pub fn project_soa(")
        .expect("soa.rs must expose project_soa");
    &src[start..]
}

fn requirement_with(id: &str, rule: ApplicabilityRule) -> Requirement {
    let req = Requirement::new(
        RequirementId::new(id),
        FrameworkId::new("canonical"),
        FrameworkVersion::new("1"),
        "baseline requirement",
        "static-only characterization",
    );
    let mut value = serde_json::to_value(&req).unwrap();
    value["applicability"] = serde_json::to_value(&rule).unwrap();
    serde_json::from_value(value).unwrap()
}

fn control_with(id: &str, rule: ApplicabilityRule) -> Control {
    let control = Control::new(
        ControlId::new(id),
        "baseline control",
        "static-only characterization",
    );
    let mut value = serde_json::to_value(&control).unwrap();
    value["applicability"] = serde_json::to_value(&rule).unwrap();
    serde_json::from_value(value).unwrap()
}

fn soc2_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Soc2,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("1"),
        context: FrameworkContext::default(),
    }
}

fn compile_assessment(assessment: &Assessment) -> weeping_angel_framework::CompiledFramework {
    compile_framework(assessment, &soc2_target()).expect("in-memory assessment compiles")
}

fn applicable_ids(compiled: &weeping_angel_framework::CompiledFramework) -> Vec<String> {
    compiled
        .applicable_requirements
        .iter()
        .map(|req| req.id().as_str().to_string())
        .collect()
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_applicability_engine_baseline")
            && toml.contains("sdd_applicability_engine_target")
            && toml.contains("tests/contracts/applicability_engine.baseline.rs")
            && toml.contains("tests/contracts/applicability_engine.target.rs"),
        "dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b01_ir_is_declarative_and_does_not_evaluate_facts() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/applicability.rs");
    assert!(
        src.contains("Declarative applicability. The IR does not evaluate platform facts."),
        "IR module docs still say the crate does not evaluate platform facts"
    );
    assert!(
        src.contains("pub fn statically_applicable(&self) -> Option<bool>"),
        "fn statically_applicable must exist"
    );
    assert!(
        !src.contains("fn evaluate(") && !src.contains("fn evaluate_applicability"),
        "IR must not grow a fact evaluator"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b02_statically_applicable_always_never_predicate() {
    assert_eq!(
        ApplicabilityRule::Always.statically_applicable(),
        Some(true)
    );
    assert_eq!(
        ApplicabilityRule::Never.statically_applicable(),
        Some(false)
    );
    assert_eq!(
        ApplicabilityRule::jurisdiction("EU").statically_applicable(),
        None
    );
    assert_eq!(
        ApplicabilityRule::processes_personal_data(true).statically_applicable(),
        None
    );
    assert_eq!(
        ApplicabilityRule::Predicate(ApplicabilityPredicate::UsesCloudProvider(true))
            .statically_applicable(),
        None
    );
    assert_eq!(
        ApplicabilityRule::Predicate(ApplicabilityPredicate::HasVendor(true))
            .statically_applicable(),
        None
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b03_not_of_predicate_stays_none() {
    let rule = ApplicabilityRule::Not(Box::new(ApplicabilityRule::processes_personal_data(true)));
    assert_eq!(
        rule.statically_applicable(),
        None,
        "Not(None) stays None; unknown is not flipped to true"
    );
    assert_eq!(
        ApplicabilityRule::Not(Box::new(ApplicabilityRule::Always)).statically_applicable(),
        Some(false)
    );
    assert_eq!(
        ApplicabilityRule::Not(Box::new(ApplicabilityRule::Never)).statically_applicable(),
        Some(true)
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b03b_kleene_static_fold_over_combinators() {
    let pred = ApplicabilityRule::jurisdiction("NL");
    assert_eq!(
        ApplicabilityRule::All(vec![]).statically_applicable(),
        Some(true),
        "empty All is vacuously true"
    );
    assert_eq!(
        ApplicabilityRule::Any(vec![]).statically_applicable(),
        Some(false),
        "empty Any is vacuously false"
    );
    assert_eq!(
        ApplicabilityRule::All(vec![ApplicabilityRule::Always, ApplicabilityRule::Always])
            .statically_applicable(),
        Some(true)
    );
    assert_eq!(
        ApplicabilityRule::All(vec![ApplicabilityRule::Always, pred.clone()])
            .statically_applicable(),
        None,
        "All(true, unknown) is unknown"
    );
    assert_eq!(
        ApplicabilityRule::All(vec![ApplicabilityRule::Never, pred.clone()])
            .statically_applicable(),
        Some(false),
        "All(false, unknown) is false"
    );
    assert_eq!(
        ApplicabilityRule::Any(vec![ApplicabilityRule::Always, pred.clone()])
            .statically_applicable(),
        Some(true),
        "Any(true, unknown) is true"
    );
    assert_eq!(
        ApplicabilityRule::Any(vec![ApplicabilityRule::Never, pred.clone()])
            .statically_applicable(),
        None,
        "Any(false, unknown) is unknown"
    );
    assert_eq!(
        ApplicabilityRule::Any(vec![ApplicabilityRule::Never, ApplicabilityRule::Never])
            .statically_applicable(),
        Some(false)
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b04_resolve_applicability_filters_static_false_only() {
    let src = crate_sources_joined("weeping-angel-framework");
    let body = fn_resolve_applicability(&src);
    assert!(
        body.contains("req.applicability().statically_applicable() != Some(false)"),
        "compile keeps a requirement unless statically_applicable is Some(false)"
    );
    assert!(
        body.contains("let _ = target;"),
        "FrameworkTarget is unused today"
    );
    assert!(
        !body.contains("evaluate_applicability"),
        "resolve_applicability must not name evaluate_applicability"
    );

    let mut assessment = Assessment::new(AssessmentId::new("assess.applicability.baseline"));
    assessment.requirements = vec![
        requirement_with("req.always", ApplicabilityRule::Always),
        requirement_with("req.never", ApplicabilityRule::Never),
        requirement_with("req.predicate", ApplicabilityRule::jurisdiction("EU")),
        requirement_with(
            "req.not-pred",
            ApplicabilityRule::Not(Box::new(ApplicabilityRule::processes_personal_data(true))),
        ),
    ];
    let compiled = compile_assessment(&assessment);
    let ids = applicable_ids(&compiled);
    assert!(ids.contains(&"req.always".into()));
    assert!(
        !ids.contains(&"req.never".into()),
        "static Never is dropped"
    );
    assert!(
        ids.contains(&"req.predicate".into()),
        "unknown predicates stay in the compiled set"
    );
    assert!(
        ids.contains(&"req.not-pred".into()),
        "Not(Predicate) is None, so the requirement is kept"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b05_project_soa_reads_pack_booleans() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    let body = fn_project_soa(&src);
    assert!(
        body.contains("applicability.toml"),
        "project_soa rereads pack applicability.toml"
    );
    assert!(
        src.contains("pub applicable: bool"),
        "SoaEntry.applicable is bool, not a three-state"
    );
    assert!(
        !body.contains("evaluate_applicability") && !body.contains("statically_applicable"),
        "SoA is a boolean pack projection, not the IR rule tree"
    );

    let soa = project_soa("iso-27001", "2022");
    assert!(
        !soa.entries.is_empty(),
        "ISO pack applicability.toml must project at least one entry"
    );
    assert!(
        soa.entries.iter().all(|entry| entry.applicable),
        "ISO pack entries are all applicable = true today"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b06_product_crates_lack_evaluator_and_snapshot() {
    let crates = product_crates_joined();
    for needle in [
        "struct ApplicabilitySnapshot",
        "fn evaluate_applicability",
        "fn build_applicability_context",
        "fn evaluate_assessment_applicability",
        "enum ApplicabilityDecision",
        "ManualDeterminationRequired",
        "struct ApplicabilityContext",
        "struct ApplicabilityOutcome",
    ] {
        assert!(
            !crates.contains(needle),
            "product crates currently have no `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b07_no_assurance_applicability_module() {
    let assurance_src = crate_src("weeping-angel-assurance");
    assert!(
        !assurance_src.join("applicability.rs").exists(),
        "weeping-angel-assurance/src/applicability.rs must not exist yet"
    );
    assert!(
        !assurance_src.join("applicability").exists(),
        "weeping-angel-assurance/src/applicability/ must not exist yet"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("mod applicability") && !lib.contains("pub mod applicability"),
        "assurance facade must not declare an applicability module"
    );
    assert!(
        !crate_src("weeping-angel-control-test")
            .join("applicability.rs")
            .exists(),
        "control-test must not host an applicability evaluator module"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b08_compile_keeps_processes_personal_data_predicate() {
    let mut assessment = Assessment::new(AssessmentId::new("assess.applicability.pd"));
    assessment.requirements = vec![requirement_with(
        "req.pd",
        ApplicabilityRule::processes_personal_data(true),
    )];
    let compiled = compile_assessment(&assessment);
    assert_eq!(
        compiled.applicable_requirements.len(),
        1,
        "ProcessesPersonalData(true) is statically unknown and must be kept"
    );
    assert_eq!(compiled.applicable_requirements[0].id().as_str(), "req.pd");
    assert_eq!(
        compiled.applicable_requirements[0]
            .applicability()
            .statically_applicable(),
        None
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b09_control_has_no_public_subjects_getter() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/control.rs");
    assert!(
        src.contains("subjects: Vec<SubjectSelector>"),
        "Control still stores subjects"
    );
    assert!(
        !src.contains("pub fn subjects(") && !src.contains("pub fn subjects "),
        "Control has no public subjects() getter today"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn p10_b10_collision_fence_does_not_import_github_collector() {
    let me = read_repo_file("tests/contracts/applicability_engine.baseline.rs");
    for line in me.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("use ") {
            assert!(
                !trimmed.contains("weeping_angel_collector"),
                "this suite must not import collector types: {trimmed}"
            );
        }
    }
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn compile_does_not_filter_controls_by_applicability() {
    let src = crate_sources_joined("weeping-angel-framework");
    let mappings = src
        .find("fn resolve_control_mappings(")
        .map(|i| &src[i..])
        .expect("resolve_control_mappings");
    let mappings = mappings
        .split("\nfn resolve_evidence_requirements")
        .next()
        .unwrap();
    assert!(
        !mappings.contains("statically_applicable"),
        "controls are not filtered by Control.applicability"
    );

    let mut assessment = Assessment::new(AssessmentId::new("assess.applicability.controls"));
    assessment.requirements = vec![requirement_with("req.keep", ApplicabilityRule::Always)];
    assessment.controls = vec![control_with("control.never", ApplicabilityRule::Never)];
    let compiled = compile_assessment(&assessment);
    assert_eq!(compiled.controls.len(), 1);
    assert_eq!(compiled.controls[0].id().as_str(), "control.never");
    assert_eq!(
        compiled.controls[0].applicability().statically_applicable(),
        Some(false)
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn inventories_exist_but_are_unused_for_applicability() {
    let def = AssessmentDefinition::new(AssessmentId::new("assess.inventories"));
    assert!(def.assets.is_empty());
    assert!(def.identities.is_empty());
    assert!(def.vendors.is_empty());
    assert!(def.processing_activities.is_empty());
    assert!(def.risks.is_empty());
    assert!(def.scope.organizations.is_empty());
    assert!(def.scope.subjects.is_empty());
    assert!(def.scope.exclusions.is_empty());

    let _asset = Asset::new(
        weeping_angel_assurance_ir::AssetId::new("asset:org"),
        AssetKind::Organization,
        "org",
    );
    let _cloud = AssetKind::CloudAccount;
    let _cloud_res = AssetKind::CloudResource;
    let _identity = Identity::new(
        weeping_angel_assurance_ir::IdentityId::new("id:user"),
        IdentityKind::User,
    );
    let _vendor = Vendor::new(VendorId::new("vendor:one"), "Vendor");
    let activity = ProcessingActivity::new(ProcessingActivityId::new("pa:one"), "HR");
    let activity_src = read_repo_file("crates/weeping-angel-assurance-ir/src/privacy.rs");
    assert!(
        !activity_src.contains("category") && !activity_src.contains("personal"),
        "ProcessingActivity has no category / personal-data field"
    );
    let _ = activity;
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("level"), "Risk has no level field");
    let _risk = Risk::new(RiskId::new("risk:one"), "title", "desc");
    let _exclusion = ScopeExclusion {
        subjects: Vec::new(),
        rationale: Some("out of scope".into()),
        ..Default::default()
    };
    let _scope = AssessmentScope {
        organizations: vec!["org:acme".into()],
        subjects: Vec::new(),
        exclusions: Vec::new(),
    };

    let framework_src = crate_sources_joined("weeping-angel-framework");
    let resolve = fn_resolve_applicability(&framework_src);
    for needle in [
        "assets",
        "vendors",
        "identities",
        "processing_activities",
        "scope.exclusions",
    ] {
        assert!(
            !resolve.contains(needle),
            "resolve_applicability does not consult `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn facade_assessment_scope_is_collector_allow_set() {
    let facade = std::any::type_name::<weeping_angel_assurance::AssessmentScope>();
    let ir = std::any::type_name::<AssessmentScope>();
    assert_ne!(
        facade, ir,
        "facade AssessmentScope is a different type from IR AssessmentScope"
    );
    let facade_src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        facade_src.contains("allowed: std::collections::BTreeSet<AssetId>"),
        "facade AssessmentScope is a collector allow-set"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn population_runtime_does_not_walk_ir_inventories() {
    let pop = read_repo_file("crates/weeping-angel-control-test/src/population.rs");
    assert!(pop.contains("pub fn resolve_population"));
    for needle in [
        "AssessmentDefinition",
        "processing_activities",
        "definition.assets",
        "definition.vendors",
    ] {
        assert!(
            !pop.contains(needle),
            "population runtime does not walk IR inventories (`{needle}`)"
        );
    }
    let set_src = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        set_src.contains("pub fn set_population"),
        "callers inject subjects via EvidenceSet::set_population"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn effectiveness_not_applicable_is_unrelated_to_ir_rules() {
    let _ = Effectiveness::NotApplicable;
    let mut set = EvidenceSet::new();
    set.set_population(Population {
        selector: weeping_angel_assurance_ir::SubjectSelector::default(),
        subject_ids: Vec::new(),
        authoritative: true,
        observed_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        completeness: PopulationCompleteness::Authoritative,
    });
    let compiled = CompiledControlTest::builder()
        .id(ControlTestId::new("test.applicability.baseline"))
        .control_id(ControlId::new("canonical.source-control"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::CoverageAtLeast {
            selector: SubjectSelector {
                kind: Some("repository".into()),
                id: None,
            },
            evidence: EvidenceSelector {
                evidence_type: EvidenceType::new("source.branch.protection"),
                subject_selector: SubjectSelector {
                    kind: Some("repository".into()),
                    id: None,
                },
                field: Some("protected".into()),
                freshness: None,
            },
            percentage: "100".into(),
        })
        .build();
    let result = evaluate(&compiled, &set, &fresh_context());
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "authoritative empty population is InsufficientEvidence, not IR NotApplicable"
    );
}

#[test]
#[ignore = "superseded by sdd_applicability_engine_target"]
fn lineage_needles_for_applicability_remain_absent() {
    let crates = product_crates_joined();
    for needle in ["OrgContext", "org_context", "fn evaluate_org_context"] {
        assert!(
            !crates.contains(needle),
            "assessment lineage still characterizes applicability engine as absent; found `{needle}`"
        );
    }
}
