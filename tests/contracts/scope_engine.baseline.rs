//! Baseline suite for the organizational scope engine.
//!
//! Characterization of CURRENT behavior on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (`docs/specs/scope-engine.md` §3).
//! IR `AssessmentScope` is a descriptive bag; silent `ScopeExclusion` rows drop
//! inventory members and never expire. There is no `ScopeResolution` quad or
//! explain traces. Facade `AssessmentScope` and `CollectorScope` remain
//! `AssetId` allow-sets. `src/engine/scope.rs` is crawl URL membership.
//! `SubjectKind` lacks generic business-unit / location / data-domain /
//! personnel-population names. `resolve_population` does not consult IR
//! `AssessmentScope`. ISMS context IR is not in product crates.
//!
//! Skip-superseded by `sdd_scope_engine_target`
//! (`#[ignore = "superseded by target suite"]`). Does not implement the scope
//! engine.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use url::Url;
use weeping_angel::authz::Authorization;
use weeping_angel::engine::scope;
use weeping_angel_assurance::applicability::{ContextExtras, build_applicability_context};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentScope, Asset, AssetId,
    AssetKind, Identity, IdentityId, IdentityKind, ScopeExclusion, SelectorScope, SubjectKind,
    SubjectSelector, ValidateIr,
};
use weeping_angel_framework::{
    Assessment, FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
    compile_framework,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
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

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let entries = fs::read_dir(&crates_dir).unwrap_or_else(|e| {
        panic!("read {}: {e}", crates_dir.display());
    });
    let mut chunks = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
}

fn any_of(kind: SubjectKind, id: &str) -> SubjectSelector {
    let mut ids = BTreeSet::new();
    ids.insert(id.to_string());
    SubjectSelector {
        kind,
        ids,
        tags: BTreeMap::new(),
        scope: SelectorScope::AnyOf,
    }
}

fn repo_asset(id: &str, name: &str) -> Asset {
    Asset::new(AssetId::new(id), AssetKind::Repository, name)
}

fn silent_exclusion(id: &str) -> ScopeExclusion {
    ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Repository, id)],
        rationale: None,
        ..Default::default()
    }
}

/// SCP-B01: IR `AssessmentScope` fields are organizations, subjects, exclusions only.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b01_ir_assessment_scope_is_descriptive_bag() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    let start = src
        .find("pub struct AssessmentScope")
        .expect("IR AssessmentScope must exist");
    let body = src[start..]
        .split("pub struct AssessmentDefinition")
        .next()
        .unwrap();
    assert!(body.contains("pub organizations: Vec<String>"));
    assert!(body.contains("pub subjects: Vec<SubjectSelector>"));
    assert!(body.contains("pub exclusions: Vec<ScopeExclusion>"));
    for forbidden in [
        "inclusion_rules",
        "inclusionRules",
        "scope_id",
        "scopeId",
        "as_of",
        "asOf",
    ] {
        assert!(
            !body.contains(forbidden),
            "IR AssessmentScope is still the three-field bag; found `{forbidden}`"
        );
    }

    let scope = AssessmentScope {
        organizations: vec!["org:acme".into()],
        subjects: vec![any_of(SubjectKind::Repository, "repo:payments")],
        exclusions: vec![silent_exclusion("repo:legacy")],
    };
    let json = serde_json::to_value(&scope).unwrap();
    let obj = json
        .as_object()
        .expect("AssessmentScope serializes as object");
    let keys: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        BTreeSet::from(["organizations", "subjects", "exclusions"]),
        "found-case: IR AssessmentScope JSON keys are the descriptive bag only"
    );

    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.baseline"));
    assert!(assessment.scope.organizations.is_empty());
    assert!(assessment.scope.subjects.is_empty());
    assert!(assessment.scope.exclusions.is_empty());
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

/// SCP-B02: `ScopeExclusion` is subjects + optional rationale; no governance fields.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b02_scope_exclusion_lacks_governance_fields() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    let start = src
        .find("pub struct ScopeExclusion")
        .expect("ScopeExclusion must exist");
    let body = src[start..]
        .split("pub struct AssessmentScope")
        .next()
        .unwrap();
    assert!(body.contains("pub subjects: Vec<SubjectSelector>"));
    assert!(body.contains("pub rationale: Option<String>"));
    for forbidden in [
        "approvalRef",
        "approval_ref",
        "approvedAt",
        "approved_at",
        "expiresAt",
        "expires_at",
        "reviewBy",
        "review_by",
        "evidenceRefs",
        "evidence_refs",
        "owner",
        "PrincipalRef",
    ] {
        assert!(
            !body.contains(forbidden),
            "found-case: ScopeExclusion source must not name `{forbidden}`"
        );
    }

    let silent = ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Repository, "repo:legacy")],
        rationale: None,
        ..Default::default()
    };
    let json = serde_json::to_value(&silent).unwrap();
    let obj = json.as_object().unwrap();
    assert!(obj.get("rationale").is_none(), "None rationale is omitted");
    assert!(obj.get("owner").is_none());
    assert!(obj.get("approvalRef").is_none());
    assert!(obj.get("approvedAt").is_none());
    assert!(obj.get("expiresAt").is_none());
    assert!(obj.get("reviewBy").is_none());
    assert!(obj.get("evidenceRefs").is_none());

    let decoded: ScopeExclusion = serde_json::from_value(serde_json::json!({
        "subjects": [{
            "kind": "repository",
            "ids": ["repo:legacy"],
            "scope": "anyOf"
        }]
    }))
    .expect("silent exclusion JSON still deserializes");
    assert!(decoded.rationale.is_none());
    assert!(!decoded.subjects.is_empty());
}

/// SCP-B03: applicability synthesizes “excluded by assessment scope[i]” and silent exclusions drop members.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b03_silent_exclusions_are_operational() {
    let ctx_src = read_repo_file("crates/weeping-angel-assurance/src/applicability/context.rs");
    assert!(
        ctx_src.contains("excluded by assessment scope["),
        "found-case: context.rs synthesizes a silent exclusion rationale"
    );
    assert!(
        ctx_src.contains("unwrap_or_else(|| format!(\"excluded by assessment scope[{index}]\"))"),
        "missing rationale is replaced, not rejected"
    );

    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.silent"));
    definition.assets = vec![
        repo_asset("repo:payments", "payments"),
        repo_asset("repo:legacy", "legacy"),
    ];
    definition.scope.exclusions = vec![silent_exclusion("repo:legacy")];

    let ctx = build_applicability_context(&definition, ContextExtras::new());
    let remaining: Vec<&str> = ctx.assets.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(remaining, vec!["repo:payments"]);
    assert_eq!(ctx.excluded_subjects.len(), 1);
    assert_eq!(ctx.excluded_subjects[0].id, "repo:legacy");
    assert_eq!(
        ctx.excluded_subjects[0].reason, "excluded by assessment scope[0]",
        "found-case: silent exclusion still removes the inventory member"
    );
}

/// SCP-B04: product crates have no ISMS ScopeResolution / four-state decision.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b04_no_scope_resolution_or_decision_quad() {
    let product = product_crate_sources_joined();
    for needle in [
        "struct ScopeResolution",
        "SCOPE_RESOLUTION_SCHEMA",
        "enum ScopeDecision",
        "weeping-angel/scope-resolution/v1",
        "fn resolve_scope(",
        "ScopeDecision::InScope",
    ] {
        assert!(
            !product.contains(needle),
            "found-case: product crates must not contain `{needle}`"
        );
    }

    let assurance_src = crate_src("weeping-angel-assurance");
    assert!(
        !assurance_src.join("scope.rs").exists() && !assurance_src.join("scope").exists(),
        "weeping-angel-assurance has no scope engine module yet"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("mod scope") && !lib.contains("pub mod scope"),
        "assurance facade must not declare a scope engine module"
    );
}

/// SCP-B05: facade AssessmentScope and CollectorScope are AssetId allow-sets.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b05_facade_and_collector_scope_are_asset_allow_sets() {
    let facade_name = std::any::type_name::<weeping_angel_assurance::AssessmentScope>();
    let ir_name = std::any::type_name::<AssessmentScope>();
    assert_ne!(
        facade_name, ir_name,
        "facade AssessmentScope is a different type from IR AssessmentScope"
    );

    let facade_src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        facade_src.contains("allowed: std::collections::BTreeSet<AssetId>"),
        "facade AssessmentScope is an AssetId allow-set"
    );
    assert!(facade_src.contains("pub fn allow_asset(mut self, asset: AssetId)"));
    assert!(facade_src.contains("fn to_collector_scope(&self) -> CollectorScope"));

    let collector_src = read_repo_file("crates/weeping-angel-collector/src/lib.rs");
    assert!(collector_src.contains("pub struct CollectorScope"));
    assert!(collector_src.contains("allowed: BTreeSet<AssetId>"));
    assert!(collector_src.contains("pub fn allow_asset(mut self, asset: AssetId)"));
    assert!(collector_src.contains("pub fn allows(&self, asset: &AssetId) -> bool"));

    let mut facade = weeping_angel_assurance::AssessmentScope::new();
    facade = facade.allow_asset(AssetId::new("repo:payments"));
    assert!(facade.describe().contains("repo:payments"));

    let collector =
        weeping_angel_collector::CollectorScope::new().allow_asset(AssetId::new("repo:payments"));
    assert!(collector.allows(&AssetId::new("repo:payments")));
    assert!(!collector.allows(&AssetId::new("repo:legacy")));
}

/// SCP-B06: `src/engine/scope.rs` is crawl URL membership, not ISMS.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b06_crawl_scope_is_url_membership() {
    let src = read_repo_file("src/engine/scope.rs");
    assert!(src.contains("pub fn in_scope(authz: &Authorization, url: &Url) -> bool"));
    assert!(src.contains("authz.url_in_scope(url)"));
    assert!(
        !src.contains("AssessmentScope")
            && !src.contains("ScopeResolution")
            && !src.contains("SubjectKind"),
        "crawl scope.rs must not grow ISMS types"
    );

    let authz = Authorization::new(true, ["example.com".into()], false, false);
    let inside = Url::parse("https://example.com/app").unwrap();
    let outside = Url::parse("https://other.example/app").unwrap();
    assert!(scope::in_scope(&authz, &inside));
    assert!(!scope::in_scope(&authz, &outside));
}

/// SCP-B07: generic business-unit / location / data-domain / population kinds do not parse.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b07_generic_scopeable_kinds_do_not_parse() {
    assert_eq!(
        SubjectKind::parse_name("repository"),
        Some(SubjectKind::Repository)
    );
    assert_eq!(
        SubjectKind::parse_name("organization"),
        Some(SubjectKind::Organization)
    );
    assert_eq!(SubjectKind::parse_name("businessunit"), None);
    assert_eq!(SubjectKind::parse_name("business-unit"), None);
    assert_eq!(SubjectKind::parse_name("location"), None);
    assert_eq!(SubjectKind::parse_name("datadomain"), None);
    assert_eq!(SubjectKind::parse_name("data-domain"), None);
    assert_eq!(SubjectKind::parse_name("personnelpopulation"), None);
    assert_eq!(SubjectKind::parse_name("population"), None);

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/subject.rs");
    for forbidden in [
        "BusinessUnit",
        "Location",
        "DataDomain",
        "PersonnelPopulation",
    ] {
        assert!(
            !src.contains(forbidden),
            "SubjectKind currently has no `{forbidden}` variant"
        );
    }
}

/// SCP-B08: `resolve_population` source does not mention AssessmentScope.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b08_resolve_population_does_not_consult_ir_assessment_scope() {
    let pop = read_repo_file("crates/weeping-angel-control-test/src/population.rs");
    assert!(pop.contains("pub fn resolve_population"));
    for needle in [
        "AssessmentScope",
        "AssessmentDefinition",
        "ScopeExclusion",
        "definition.scope",
        "scope.subjects",
        "scope.exclusions",
    ] {
        assert!(
            !pop.contains(needle),
            "found-case: resolve_population must not mention `{needle}`"
        );
    }
}

/// SCP-B09: no `struct IsmsContext` in product crate sources.
#[test]
#[ignore = "superseded by target suite"]
fn scp_b09_isms_context_ir_not_in_product() {
    let product = product_crate_sources_joined();
    assert!(
        !product.contains("struct IsmsContext"),
        "found-case: ISMS context IR is not landed; product crates have no struct IsmsContext"
    );
    let ir_lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !ir_lib.contains("IsmsContext") && !ir_lib.contains("pub mod isms"),
        "assurance-ir lib.rs does not re-export IsmsContext"
    );
}

/// SCP-B10: dual-suite names are registered in root Cargo.toml (not auto-discovered).
#[test]
#[ignore = "superseded by target suite"]
fn scp_b10_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("name = \"sdd_scope_engine_baseline\"")
            && toml.contains("path = \"tests/contracts/scope_engine.baseline.rs\"")
            && toml.contains("name = \"sdd_scope_engine_target\"")
            && toml.contains("path = \"tests/contracts/scope_engine.target.rs\""),
        "scope engine dual-suite must be explicitly listed (tests/contracts is not auto-discovered)"
    );
}

/// Found case: validate_assessment_ir does not walk scope (silent / dangling selectors are valid).
#[test]
#[ignore = "superseded by target suite"]
fn validate_does_not_walk_scope() {
    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        !validation.contains("scope.exclusions")
            && !validation.contains("scope.subjects")
            && !validation.contains("silent"),
        "validate_assessment_ir currently does not inspect AssessmentScope"
    );

    let mut definition =
        AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.validate"));
    definition.scope.exclusions = vec![silent_exclusion("repo:missing")];
    definition.scope.subjects = vec![any_of(SubjectKind::Repository, "repo:unresolved")];
    definition
        .validate()
        .expect("dangling / silent scope still validates on this HEAD");
}

/// Found case: empty subjects keep the full inventory; organizations are copied, not an inclusion expander.
#[test]
#[ignore = "superseded by target suite"]
fn organizations_are_not_an_inclusion_expander() {
    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.orgs"));
    definition.scope.organizations = vec!["org:acme".into()];
    definition.assets = vec![
        repo_asset("repo:payments", "payments"),
        repo_asset("repo:other-org", "other"),
    ];
    let ctx = build_applicability_context(&definition, ContextExtras::new());
    assert_eq!(ctx.organizations, vec!["org:acme".to_string()]);
    assert_eq!(
        ctx.assets.len(),
        2,
        "empty subjects keeps the full inventory"
    );
    assert!(
        ctx.excluded_subjects.is_empty(),
        "organizations do not themselves exclude or include"
    );
}

/// Found case: non-empty subjects retain by direct selector match; Asset.parent is not walked.
#[test]
#[ignore = "superseded by target suite"]
fn nested_parent_is_not_walked_for_membership() {
    let mut child = repo_asset("repo:payments", "payments");
    child.parent = Some(AssetId::new("service:payments"));
    let parent = Asset::new(
        AssetId::new("service:payments"),
        AssetKind::Service,
        "payments",
    );

    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.nested"));
    definition.assets = vec![parent, child];
    definition.scope.subjects = vec![any_of(SubjectKind::Service, "service:payments")];

    let ctx = build_applicability_context(&definition, ContextExtras::new());
    let remaining: Vec<&str> = ctx.assets.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        remaining,
        vec!["service:payments"],
        "found-case: child repo under an included service is not inherited into scope"
    );
}

/// Found case: exclusion apply order is enumeration; a later exclusion drops a previously included subject.
#[test]
#[ignore = "superseded by target suite"]
fn exclusion_enumeration_drops_included_subject() {
    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.order"));
    definition.assets = vec![repo_asset("repo:payments", "payments")];
    definition.scope.subjects = vec![any_of(SubjectKind::Repository, "repo:payments")];
    definition.scope.exclusions = vec![ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Repository, "repo:payments")],
        rationale: Some("carve-out".into()),
        ..Default::default()
    }];
    let ctx = build_applicability_context(&definition, ContextExtras::new());
    assert!(ctx.assets.is_empty());
    assert_eq!(ctx.excluded_subjects[0].reason, "carve-out");
}

/// Found case: Exception expiry exists; ScopeExclusion has no parallel clock.
#[test]
#[ignore = "superseded by target suite"]
fn exception_expiry_exists_exclusions_ignore_it() {
    let exception_src = read_repo_file("crates/weeping-angel-assurance-ir/src/exception.rs");
    assert!(exception_src.contains("pub expires_at: Option<DateTime<Utc>>"));
    let pop = read_repo_file("crates/weeping-angel-control-test/src/population.rs");
    assert!(pop.contains("fn subject_is_excepted"));
    assert!(pop.contains("ex.expires_at.is_some_and(|exp| exp <= now)"));

    let exclusion_src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    let start = exclusion_src.find("pub struct ScopeExclusion").unwrap();
    let body = exclusion_src[start..]
        .split("pub struct AssessmentScope")
        .next()
        .unwrap();
    assert!(
        !body.contains("expires_at") && !body.contains("as_of"),
        "ScopeExclusion has no expiry / as_of clock"
    );
}

/// Found case: framework compile ignores assessment scope.
#[test]
#[ignore = "superseded by target suite"]
fn compile_ignores_assessment_scope() {
    let framework = crate_sources_joined("weeping-angel-framework");
    let start = framework
        .find("fn resolve_applicability(")
        .expect("resolve_applicability");
    let body = framework[start..]
        .split("\nfn validate_capabilities")
        .next()
        .unwrap();
    assert!(body.contains("req.applicability().statically_applicable() != Some(false)"));
    assert!(body.contains("let _ = target;"));
    for needle in ["scope.subjects", "scope.exclusions", "ScopeResolution"] {
        assert!(
            !body.contains(needle),
            "resolve_applicability does not consult `{needle}`"
        );
    }

    let mut assessment = Assessment::new(AssessmentId::new("assess.scope-engine.compile"));
    assessment.scope.exclusions = vec![silent_exclusion("repo:payments")];
    assessment.requirements = vec![weeping_angel_assurance_ir::Requirement::new(
        weeping_angel_assurance_ir::RequirementId::new("req.keep"),
        weeping_angel_assurance_ir::FrameworkId::new("canonical"),
        weeping_angel_assurance_ir::FrameworkVersion::new("1"),
        "kept",
        "compile ignores IR assessment scope",
    )];
    let compiled = compile_framework(
        &assessment,
        &FrameworkTarget {
            profile: FrameworkProfile::Soc2,
            capabilities: FrameworkCapabilities::default(),
            version: weeping_angel_assurance_ir::FrameworkVersion::new("1"),
            context: FrameworkContext::default(),
        },
    )
    .expect("in-memory assessment compiles");
    assert_eq!(compiled.applicable_requirements.len(), 1);
}

/// Found case: control-test evaluate / population do not filter by IR AssessmentScope,
/// so an excluded inventory subject can still be injected as population/evidence.
#[test]
#[ignore = "superseded by target suite"]
fn out_of_scope_subject_can_contribute_positive_assurance() {
    let mut definition =
        AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.evidence"));
    definition.assets = vec![repo_asset("repo:in", "in"), repo_asset("repo:out", "out")];
    definition.scope.exclusions = vec![silent_exclusion("repo:out")];
    let ctx = build_applicability_context(&definition, ContextExtras::new());
    assert_eq!(ctx.assets.len(), 1);
    assert_eq!(ctx.assets[0].id.as_str(), "repo:in");

    let eval = crate_sources_joined("weeping-angel-control-test");
    for needle in [
        "AssessmentScope",
        "ScopeResolution",
        "ScopeDecision",
        "is_definitely_in_scope",
    ] {
        assert!(
            !eval.contains(needle),
            "found-case: control-test does not consult `{needle}` before counting evidence"
        );
    }
    let set_src = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        set_src.contains("pub fn set_population"),
        "population is still caller-injected; no IR scope gate"
    );
}

/// Found case: identities exist as inventory but are unused by IR AssessmentScope resolution.
#[test]
#[ignore = "superseded by target suite"]
fn identities_exist_but_scope_does_not_select_populations() {
    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.pop"));
    definition.identities = vec![Identity::new(
        IdentityId::new("id:alice"),
        IdentityKind::User,
    )];
    definition.scope.organizations = vec!["org:acme".into()];
    let ctx = build_applicability_context(&definition, ContextExtras::new());
    assert_eq!(ctx.identities.len(), 1);
    assert_eq!(ctx.identities[0].id.as_str(), "id:alice");

    let pop = crate_sources_joined("weeping-angel-control-test");
    assert!(!pop.contains("PersonnelPopulation"));
}

/// Golden assessment.json still decodes with the three-field scope bag.
#[test]
#[ignore = "superseded by target suite"]
fn golden_assessment_json_still_decodes() {
    let path = manifest_dir().join("tests/fixtures/assurance-ir/v1/assessment.json");
    let raw = fs::read_to_string(&path).unwrap();
    let assessment: AssessmentDefinition = serde_json::from_str(&raw).unwrap();
    assessment.validate().unwrap();
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    if let Some(scope) = parsed.get("scope") {
        if let Some(obj) = scope.as_object() {
            for key in obj.keys() {
                assert!(
                    matches!(key.as_str(), "organizations" | "subjects" | "exclusions"),
                    "golden scope must stay the descriptive bag; found `{key}`"
                );
            }
        }
    }
}
