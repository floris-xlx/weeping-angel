//! Target suite for the organization-context / applicability engine.
//!
//! Encodes DESIRED Kleene / snapshot behavior in
//! `docs/specs/applicability-engine.md` §4 / §6.2 (`P10-T01`–`P10-T16`).
//! Must stay RED on the current static-only / no-evaluator HEAD. Do not
//! `#[ignore]` these tests and do not implement the engine in this suite.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use weeping_angel_assurance::applicability::{
    APPLICABILITY_SNAPSHOT_SCHEMA, ApplicabilityContext, ApplicabilityDecision,
    ApplicabilityOutcome, ApplicabilitySnapshot, ContextExtras, FactKey, FactValue,
    InventoryCompleteness, InventoryFamily, build_applicability_context, evaluate_applicability,
    evaluate_assessment_applicability,
};
use weeping_angel_assurance_ir::{
    ApplicabilityPredicate, ApplicabilityRule, AssessmentDefinition, AssessmentId, AssessmentScope,
    Asset, AssetId, AssetKind, Control, ControlId, FrameworkId, FrameworkVersion, Requirement,
    RequirementId, ScopeExclusion, SelectorScope, SubjectKind, SubjectSelector, Vendor, VendorId,
};
use weeping_angel_control_test::{EvidenceSet, Population, PopulationCompleteness};

const SNAPSHOT_SCHEMA: &str = "weeping-angel/applicability-snapshot/v1";

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
    manifest_dir().join("crates").join(name).join("src")
}

#[allow(dead_code)]
fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn applicability_sources() -> String {
    let root = crate_src("weeping-angel-assurance");
    let mut files = Vec::new();
    let dir = root.join("applicability");
    if dir.is_dir() {
        walk_rs_files(&dir, &mut files);
    }
    let single = root.join("applicability.rs");
    if single.is_file() {
        files.push(single);
    }
    assert!(
        !files.is_empty(),
        "weeping-angel-assurance must own applicability/{{context,evaluator,snapshot}} (or applicability.rs)"
    );
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn requirement_with(id: &str, rule: ApplicabilityRule) -> Requirement {
    let req = Requirement::new(
        RequirementId::new(id),
        FrameworkId::new("canonical"),
        FrameworkVersion::new("1"),
        "target requirement",
        "desired Kleene evaluator",
    );
    let mut value = serde_json::to_value(&req).unwrap();
    value["applicability"] = serde_json::to_value(&rule).unwrap();
    serde_json::from_value(value).unwrap()
}

fn control_with(id: &str, rule: ApplicabilityRule, subjects: Vec<SubjectSelector>) -> Control {
    let control = Control::new(
        ControlId::new(id),
        "target control",
        "desired Kleene evaluator",
    );
    let mut value = serde_json::to_value(&control).unwrap();
    value["applicability"] = serde_json::to_value(&rule).unwrap();
    value["subjects"] = serde_json::to_value(&subjects).unwrap();
    serde_json::from_value(value).unwrap()
}

fn tagged_org(id: &str, name: &str, tags: &[(&str, &str)]) -> Asset {
    let mut asset = Asset::new(AssetId::new(id), AssetKind::Organization, name);
    for (k, v) in tags {
        asset.tags.insert((*k).into(), (*v).into());
    }
    asset
}

fn asset(id: &str, kind: AssetKind, name: &str) -> Asset {
    Asset::new(AssetId::new(id), kind, name)
}

fn id_selector(kind: SubjectKind, id: &str) -> SubjectSelector {
    let mut ids = BTreeSet::new();
    ids.insert(id.to_string());
    SubjectSelector {
        kind,
        ids,
        tags: BTreeMap::new(),
        scope: SelectorScope::AnyOf,
    }
}

fn empty_definition(id: &str) -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new(id))
}

fn context_from(def: &AssessmentDefinition, extras: ContextExtras) -> ApplicabilityContext {
    build_applicability_context(def, extras)
}

fn eval_rule(rule: &ApplicabilityRule, ctx: &ApplicabilityContext) -> ApplicabilityOutcome {
    evaluate_applicability(rule, ctx)
}

fn assert_sorted_strings(label: &str, values: &[String]) {
    let mut expected = values.to_vec();
    expected.sort();
    expected.dedup();
    assert_eq!(
        values, expected,
        "{label} must be lexicographically unique and sorted"
    );
}

fn unknown_keys(outcome: &ApplicabilityOutcome) -> Vec<FactKey> {
    outcome
        .unknown_facts
        .iter()
        .map(|f| f.key.clone())
        .collect()
}

fn predicate_values(outcome: &ApplicabilityOutcome) -> Vec<(ApplicabilityPredicate, FactValue)> {
    outcome
        .predicates
        .iter()
        .map(|t| (t.predicate.clone(), t.value))
        .collect()
}

#[test]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_applicability_engine_baseline")
            && toml.contains("sdd_applicability_engine_target")
            && !toml.contains("tests/contracts/applicability_engine.baseline.rs")
            && toml.contains("tests/contracts/applicability_engine.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
}

/// P10: static Always/Never
#[test]
fn p10_t01_static_always_never() {
    let def = empty_definition("assess.p10.t01");
    let ctx = context_from(&def, ContextExtras::new());
    let always = eval_rule(&ApplicabilityRule::Always, &ctx);
    assert_eq!(always.decision, ApplicabilityDecision::Applicable);
    assert!(
        always.predicates.is_empty() && always.unknown_facts.is_empty(),
        "Always consults no facts: predicates={:?} unknown={:?}",
        always.predicates,
        always.unknown_facts
    );

    let never = eval_rule(&ApplicabilityRule::Never, &ctx);
    assert_eq!(never.decision, ApplicabilityDecision::NotApplicable);
    assert!(
        never.predicates.is_empty() && never.unknown_facts.is_empty(),
        "Never consults no facts"
    );
}

/// P10: known true/false predicates
#[test]
fn p10_t02_known_true_false_predicates() {
    let mut def = empty_definition("assess.p10.t02");
    def.scope.organizations = vec!["org:acme".into()];
    def.assets.push(tagged_org(
        "asset:org-acme",
        "acme",
        &[("jurisdiction", "EU")],
    ));
    def.vendors
        .push(Vendor::new(VendorId::new("vendor:payroll"), "Payroll Co"));
    let extras = ContextExtras::new()
        .with_completeness(
            InventoryFamily::Assets,
            InventoryCompleteness::Authoritative,
        )
        .with_completeness(
            InventoryFamily::Vendors,
            InventoryCompleteness::Authoritative,
        )
        .with_completeness(
            InventoryFamily::Jurisdictions,
            InventoryCompleteness::Authoritative,
        );
    let ctx = context_from(&def, extras);

    let eu = eval_rule(&ApplicabilityRule::jurisdiction("EU"), &ctx);
    assert_eq!(eu.decision, ApplicabilityDecision::Applicable);
    assert!(eu.unknown_facts.is_empty());

    let us = eval_rule(&ApplicabilityRule::jurisdiction("US"), &ctx);
    assert_eq!(us.decision, ApplicabilityDecision::NotApplicable);
    assert!(us.unknown_facts.is_empty());

    let vendor = eval_rule(
        &ApplicabilityRule::Predicate(ApplicabilityPredicate::HasVendor(true)),
        &ctx,
    );
    assert_eq!(vendor.decision, ApplicabilityDecision::Applicable);

    let no_vendor = eval_rule(
        &ApplicabilityRule::Predicate(ApplicabilityPredicate::HasVendor(false)),
        &ctx,
    );
    assert_eq!(no_vendor.decision, ApplicabilityDecision::NotApplicable);
}

/// P10: unknown predicates
#[test]
fn p10_t03_unknown_predicates() {
    let def = empty_definition("assess.p10.t03");
    let ctx = context_from(&def, ContextExtras::new());
    let pd = eval_rule(&ApplicabilityRule::processes_personal_data(true), &ctx);
    assert_eq!(
        pd.decision,
        ApplicabilityDecision::ManualDeterminationRequired,
        "ProcessesPersonalData(true) with no personal-data fact is unknown, never NotApplicable"
    );
    assert_ne!(pd.decision, ApplicabilityDecision::NotApplicable);
    assert!(
        unknown_keys(&pd).contains(&FactKey::PersonalData),
        "unknown facts must name personal data: {:?}",
        pd.unknown_facts
    );
    assert!(
        predicate_values(&pd).iter().any(|(p, v)| {
            matches!(p, ApplicabilityPredicate::ProcessesPersonalData(true))
                && *v == FactValue::Unknown
        }),
        "predicate trace must record Unknown, not False: {:?}",
        pd.predicates
    );
}

/// P10: nested All/Any/Not with unknown values
#[test]
fn p10_t04_nested_all_any_not_with_unknown_values() {
    let def = empty_definition("assess.p10.t04");
    let ctx = context_from(&def, ContextExtras::new());
    let pred = ApplicabilityRule::processes_personal_data(true);

    assert_eq!(
        eval_rule(&ApplicabilityRule::All(vec![]), &ctx).decision,
        ApplicabilityDecision::Applicable,
        "empty All is vacuously true"
    );
    assert_eq!(
        eval_rule(&ApplicabilityRule::Any(vec![]), &ctx).decision,
        ApplicabilityDecision::NotApplicable,
        "empty Any is vacuously false"
    );

    let all_true_unknown = ApplicabilityRule::All(vec![ApplicabilityRule::Always, pred.clone()]);
    assert_eq!(
        eval_rule(&all_true_unknown, &ctx).decision,
        ApplicabilityDecision::ManualDeterminationRequired,
        "All(true, unknown) is unknown"
    );

    let all_false_unknown = ApplicabilityRule::All(vec![ApplicabilityRule::Never, pred.clone()]);
    assert_eq!(
        eval_rule(&all_false_unknown, &ctx).decision,
        ApplicabilityDecision::NotApplicable,
        "All(false, unknown) is false"
    );

    let any_true_unknown = ApplicabilityRule::Any(vec![ApplicabilityRule::Always, pred.clone()]);
    assert_eq!(
        eval_rule(&any_true_unknown, &ctx).decision,
        ApplicabilityDecision::Applicable,
        "Any(true, unknown) is true"
    );

    let any_false_unknown = ApplicabilityRule::Any(vec![ApplicabilityRule::Never, pred.clone()]);
    assert_eq!(
        eval_rule(&any_false_unknown, &ctx).decision,
        ApplicabilityDecision::ManualDeterminationRequired,
        "Any(false, unknown) is unknown"
    );

    let nested = ApplicabilityRule::All(vec![ApplicabilityRule::Any(vec![
        ApplicabilityRule::Never,
        pred,
    ])]);
    assert_eq!(
        eval_rule(&nested, &ctx).decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );
}

/// P10: jurisdiction-specific context
#[test]
fn p10_t05_jurisdiction_specific_context() {
    let mut known = empty_definition("assess.p10.t05.known");
    known.scope.organizations = vec!["org:acme".into()];
    known.assets.push(tagged_org(
        "asset:org-acme",
        "acme",
        &[("jurisdictionCode", "NL")],
    ));
    let known_ctx = context_from(
        &known,
        ContextExtras::new()
            .with_completeness(
                InventoryFamily::Assets,
                InventoryCompleteness::Authoritative,
            )
            .with_completeness(
                InventoryFamily::Jurisdictions,
                InventoryCompleteness::Authoritative,
            ),
    );
    assert_eq!(
        eval_rule(&ApplicabilityRule::jurisdiction("NL"), &known_ctx).decision,
        ApplicabilityDecision::Applicable
    );
    assert_eq!(
        eval_rule(&ApplicabilityRule::jurisdiction("nl"), &known_ctx).decision,
        ApplicabilityDecision::Applicable,
        "jurisdiction match is case-insensitive"
    );
    assert_eq!(
        eval_rule(&ApplicabilityRule::jurisdiction("DE"), &known_ctx).decision,
        ApplicabilityDecision::NotApplicable
    );

    let unknown = empty_definition("assess.p10.t05.unknown");
    let unknown_ctx = context_from(&unknown, ContextExtras::new());
    let outcome = eval_rule(&ApplicabilityRule::jurisdiction("EU"), &unknown_ctx);
    assert_eq!(
        outcome.decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );
    assert!(unknown_keys(&outcome).contains(&FactKey::Jurisdiction("EU".into())));
}

/// P10: organization with no cloud assets
#[test]
fn p10_t06_organization_with_no_cloud_assets() {
    let mut def = empty_definition("assess.p10.t06");
    def.scope.organizations = vec!["org:acme".into()];
    def.assets.push(tagged_org("asset:org-acme", "acme", &[]));
    def.assets
        .push(asset("asset:repo-app", AssetKind::Repository, "app"));
    let ctx = context_from(
        &def,
        ContextExtras::new().with_completeness(
            InventoryFamily::Assets,
            InventoryCompleteness::Authoritative,
        ),
    );
    let outcome = eval_rule(
        &ApplicabilityRule::Predicate(ApplicabilityPredicate::UsesCloudProvider(true)),
        &ctx,
    );
    assert_eq!(
        outcome.decision,
        ApplicabilityDecision::NotApplicable,
        "authoritative assets with no CloudAccount/CloudResource make UsesCloudProvider(true) false"
    );
    assert!(outcome.unknown_facts.is_empty());
}

/// P10: cloud state unknown
#[test]
fn p10_t07_cloud_state_unknown() {
    let def = empty_definition("assess.p10.t07");
    let ctx = context_from(&def, ContextExtras::new());
    let outcome = eval_rule(
        &ApplicabilityRule::Predicate(ApplicabilityPredicate::UsesCloudProvider(true)),
        &ctx,
    );
    assert_eq!(
        outcome.decision,
        ApplicabilityDecision::ManualDeterminationRequired,
        "empty unmarked asset inventory is Unknown, not authoritative-empty"
    );
    assert_ne!(outcome.decision, ApplicabilityDecision::NotApplicable);
    assert!(unknown_keys(&outcome).contains(&FactKey::CloudUsage));
}

/// P10: personal-data processing known/unknown
#[test]
fn p10_t08_personal_data_processing_known_unknown() {
    let rule = ApplicabilityRule::processes_personal_data(true);

    let unknown_ctx = context_from(
        &empty_definition("assess.p10.t08.unknown"),
        ContextExtras::new(),
    );
    let unknown = eval_rule(&rule, &unknown_ctx);
    assert_eq!(
        unknown.decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );
    assert_ne!(unknown.decision, ApplicabilityDecision::NotApplicable);

    let true_ctx = context_from(
        &empty_definition("assess.p10.t08.true"),
        ContextExtras::new().with_fact(FactKey::PersonalData, FactValue::True),
    );
    assert_eq!(
        eval_rule(&rule, &true_ctx).decision,
        ApplicabilityDecision::Applicable
    );

    let false_ctx = context_from(
        &empty_definition("assess.p10.t08.false"),
        ContextExtras::new().with_fact(FactKey::PersonalData, FactValue::False),
    );
    assert_eq!(
        eval_rule(&rule, &false_ctx).decision,
        ApplicabilityDecision::NotApplicable
    );

    let mut tagged = empty_definition("assess.p10.t08.tag");
    tagged.assets.push(tagged_org(
        "asset:org-acme",
        "acme",
        &[("personalData", "true")],
    ));
    let tagged_ctx = context_from(&tagged, ContextExtras::new());
    assert_eq!(
        eval_rule(&rule, &tagged_ctx).decision,
        ApplicabilityDecision::Applicable,
        "org tag personalData=true establishes the fact"
    );
}

/// P10: explicit scope exclusions
#[test]
fn p10_t09_explicit_scope_exclusions() {
    let mut def = empty_definition("assess.p10.t09");
    def.scope.organizations = vec!["org:acme".into()];
    def.assets.push(tagged_org("asset:org-acme", "acme", &[]));
    def.assets
        .push(asset("asset:repo-keep", AssetKind::Repository, "keep"));
    def.assets
        .push(asset("asset:repo-z", AssetKind::Repository, "excluded"));
    def.scope.exclusions.push(ScopeExclusion {
        subjects: vec![id_selector(SubjectKind::Repository, "asset:repo-z")],
        rationale: Some("contractor laptop out of scope".into()),
        ..Default::default()
    });
    def.controls.push(control_with(
        "control.keep-repos",
        ApplicabilityRule::Always,
        vec![SubjectSelector {
            kind: SubjectKind::Repository,
            ids: BTreeSet::new(),
            tags: BTreeMap::new(),
            scope: SelectorScope::All,
        }],
    ));

    let ctx = context_from(
        &def,
        ContextExtras::new().with_completeness(
            InventoryFamily::Assets,
            InventoryCompleteness::Authoritative,
        ),
    );
    let snapshot = evaluate_assessment_applicability(&def, &ctx);
    let decision = snapshot
        .control_decisions
        .iter()
        .find(|d| d.id == "control.keep-repos")
        .expect("control decision present");
    assert_eq!(decision.decision, ApplicabilityDecision::Applicable);
    assert_eq!(
        decision.selected_subjects,
        vec!["asset:repo-keep".to_string()]
    );
    let excluded = decision
        .excluded_subjects
        .iter()
        .find(|row| row.id == "asset:repo-z")
        .expect("exclusion of subject Z must be recorded");
    assert!(
        excluded.reason.contains("contractor laptop out of scope"),
        "exclusion rationale must be preserved: {}",
        excluded.reason
    );

    let mut set = EvidenceSet::new();
    set.set_population(Population {
        selector: SubjectSelector {
            kind: SubjectKind::Repository,
            ids: BTreeSet::new(),
            tags: BTreeMap::new(),
            scope: SelectorScope::All,
        },
        subject_ids: decision.selected_subjects.clone(),
        authoritative: true,
        observed_at: Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap(),
        completeness: PopulationCompleteness::Authoritative,
    });
    assert_eq!(
        set.explicit_population().unwrap().subject_ids,
        decision.selected_subjects,
        "selected scope injects through existing EvidenceSet::set_population"
    );
}

/// P10: vendor-dependent controls
#[test]
fn p10_t10_vendor_dependent_controls() {
    let rule = ApplicabilityRule::Predicate(ApplicabilityPredicate::HasVendor(true));

    let mut present = empty_definition("assess.p10.t10.present");
    present
        .vendors
        .push(Vendor::new(VendorId::new("vendor:one"), "Vendor"));
    let present_ctx = context_from(&present, ContextExtras::new());
    assert_eq!(
        eval_rule(&rule, &present_ctx).decision,
        ApplicabilityDecision::Applicable
    );

    let empty_auth = empty_definition("assess.p10.t10.empty");
    let empty_ctx = context_from(
        &empty_auth,
        ContextExtras::new().with_completeness(
            InventoryFamily::Vendors,
            InventoryCompleteness::Authoritative,
        ),
    );
    assert_eq!(
        eval_rule(&rule, &empty_ctx).decision,
        ApplicabilityDecision::NotApplicable,
        "authoritative empty vendor inventory is false"
    );

    let unknown_ctx = context_from(
        &empty_definition("assess.p10.t10.unknown"),
        ContextExtras::new(),
    );
    let unknown = eval_rule(&rule, &unknown_ctx);
    assert_eq!(
        unknown.decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );
    assert!(unknown_keys(&unknown).contains(&FactKey::VendorPresence));
}

/// P10: deterministic rationale ordering
#[test]
fn p10_t11_deterministic_rationale_ordering() {
    let mut def = empty_definition("assess.p10.t11");
    def.scope.organizations = vec!["org:acme".into()];
    def.assets.push(tagged_org(
        "asset:org-acme",
        "acme",
        &[("jurisdiction", "EU")],
    ));
    def.vendors
        .push(Vendor::new(VendorId::new("vendor:one"), "Vendor"));
    def.assets
        .push(asset("asset:repo-b", AssetKind::Repository, "b"));
    def.assets
        .push(asset("asset:repo-a", AssetKind::Repository, "a"));
    let ctx = context_from(
        &def,
        ContextExtras::new()
            .with_completeness(
                InventoryFamily::Assets,
                InventoryCompleteness::Authoritative,
            )
            .with_completeness(
                InventoryFamily::Vendors,
                InventoryCompleteness::Authoritative,
            )
            .with_fact(FactKey::PersonalData, FactValue::Unknown),
    );
    let rule = ApplicabilityRule::All(vec![
        ApplicabilityRule::jurisdiction("EU"),
        ApplicabilityRule::Predicate(ApplicabilityPredicate::HasVendor(true)),
        ApplicabilityRule::processes_personal_data(true),
    ]);
    let first = eval_rule(&rule, &ctx);
    let second = eval_rule(&rule, &ctx);
    assert_eq!(first.decision, second.decision);
    assert_eq!(first.rationale, second.rationale);
    assert_eq!(first.predicates, second.predicates);
    assert_eq!(first.unknown_facts, second.unknown_facts);
    assert_eq!(first.selected_subjects, second.selected_subjects);

    let pred_order: Vec<_> = first
        .predicates
        .iter()
        .map(|t| t.predicate.clone())
        .collect();
    assert_eq!(
        pred_order,
        vec![
            ApplicabilityPredicate::Jurisdiction("EU".into()),
            ApplicabilityPredicate::HasVendor(true),
            ApplicabilityPredicate::ProcessesPersonalData(true),
        ],
        "predicate traces follow preorder / vec order of All children"
    );
    assert_sorted_strings("selected_subjects", &first.selected_subjects);
    let unknown = unknown_keys(&first);
    let mut sorted_unknown = unknown.clone();
    sorted_unknown.sort();
    assert_eq!(unknown, sorted_unknown, "unknown_facts are lex-sorted");

    def.requirements
        .push(requirement_with("req.p10.t11", rule.clone()));
    let snap_a = evaluate_assessment_applicability(&def, &ctx);
    let snap_b = evaluate_assessment_applicability(&def, &ctx);
    assert_eq!(snap_a.digest, snap_b.digest, "snapshot digest is stable");
    assert!(!snap_a.digest.is_empty());
}

/// P10: zero selected subjects is not NotApplicable
#[test]
fn p10_t12_zero_selected_subjects_is_not_not_applicable() {
    let mut def = empty_definition("assess.p10.t12");
    def.controls.push(control_with(
        "control.always.empty",
        ApplicabilityRule::Always,
        vec![SubjectSelector {
            kind: SubjectKind::Repository,
            ids: BTreeSet::new(),
            tags: BTreeMap::new(),
            scope: SelectorScope::All,
        }],
    ));
    let ctx = context_from(
        &def,
        ContextExtras::new().with_completeness(
            InventoryFamily::Assets,
            InventoryCompleteness::Authoritative,
        ),
    );
    let outcome = eval_rule(&ApplicabilityRule::Always, &ctx);
    assert_eq!(outcome.decision, ApplicabilityDecision::Applicable);
    assert!(
        outcome.selected_subjects.is_empty(),
        "Always on an empty inventory stays Applicable with an empty selected set"
    );

    let snapshot = evaluate_assessment_applicability(&def, &ctx);
    let decision = &snapshot.control_decisions[0];
    assert_eq!(decision.decision, ApplicabilityDecision::Applicable);
    assert!(decision.selected_subjects.is_empty());
}

/// P10: Not(Unknown) remains unknown
#[test]
fn p10_t13_not_unknown_remains_unknown() {
    let def = empty_definition("assess.p10.t13");
    let ctx = context_from(&def, ContextExtras::new());
    let rule = ApplicabilityRule::Not(Box::new(ApplicabilityRule::processes_personal_data(true)));
    let outcome = eval_rule(&rule, &ctx);
    assert_eq!(
        outcome.decision,
        ApplicabilityDecision::ManualDeterminationRequired,
        "Not(Unknown) stays unknown; it must not become Applicable"
    );
    assert_ne!(outcome.decision, ApplicabilityDecision::Applicable);
    assert_ne!(outcome.decision, ApplicabilityDecision::NotApplicable);
    assert!(unknown_keys(&outcome).contains(&FactKey::PersonalData));
}

/// P10: snapshot fills lineage persist shape
#[test]
fn p10_t14_snapshot_fills_lineage_persist_shape() {
    let mut def = empty_definition("assess.p10.t14");
    def.scope = AssessmentScope {
        organizations: vec!["org:acme".into()],
        subjects: Vec::new(),
        exclusions: Vec::new(),
    };
    def.requirements
        .push(requirement_with("req.z-never", ApplicabilityRule::Never));
    def.requirements
        .push(requirement_with("req.a-always", ApplicabilityRule::Always));
    def.requirements.push(requirement_with(
        "req.m-pd",
        ApplicabilityRule::processes_personal_data(true),
    ));
    def.controls.push(control_with(
        "control.z-never",
        ApplicabilityRule::Never,
        Vec::new(),
    ));
    def.controls.push(control_with(
        "control.a-always",
        ApplicabilityRule::Always,
        Vec::new(),
    ));

    let ctx = context_from(&def, ContextExtras::new());
    let snapshot: ApplicabilitySnapshot = evaluate_assessment_applicability(&def, &ctx);
    assert_eq!(snapshot.schema, SNAPSHOT_SCHEMA);
    assert_eq!(snapshot.schema, APPLICABILITY_SNAPSHOT_SCHEMA);
    assert_eq!(snapshot.assessment_id, def.id);
    assert_eq!(snapshot.scope, def.scope);
    assert!(
        snapshot.pack_entries.is_empty(),
        "pack_entries are artifacts, not Kleene inputs; default empty when none supplied"
    );

    let req_ids: Vec<_> = snapshot
        .requirement_decisions
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(req_ids, vec!["req.a-always", "req.m-pd", "req.z-never"]);
    let ctl_ids: Vec<_> = snapshot
        .control_decisions
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(ctl_ids, vec!["control.a-always", "control.z-never"]);

    let by_id = |id: &str| {
        snapshot
            .requirement_decisions
            .iter()
            .find(|d| d.id == id)
            .unwrap()
    };
    assert_eq!(
        by_id("req.a-always").decision,
        ApplicabilityDecision::Applicable
    );
    assert_eq!(
        by_id("req.z-never").decision,
        ApplicabilityDecision::NotApplicable
    );
    assert_eq!(
        by_id("req.m-pd").decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );

    let kept: Vec<_> = snapshot
        .requirement_decisions
        .iter()
        .filter(|d| d.decision != ApplicabilityDecision::NotApplicable)
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(kept, vec!["req.a-always", "req.m-pd"]);
    assert!(!snapshot.digest.is_empty());
    assert_eq!(
        snapshot.digest,
        evaluate_assessment_applicability(&def, &ctx).digest
    );
}

/// P10: same engine for controls and requirements
#[test]
fn p10_t15_same_engine_for_controls_and_requirements() {
    let rule = ApplicabilityRule::Any(vec![
        ApplicabilityRule::jurisdiction("EU"),
        ApplicabilityRule::processes_personal_data(true),
    ]);
    let mut def = empty_definition("assess.p10.t15");
    def.requirements
        .push(requirement_with("req.shared", rule.clone()));
    def.controls
        .push(control_with("control.shared", rule.clone(), Vec::new()));
    let ctx = context_from(&def, ContextExtras::new());

    let from_rule = eval_rule(&rule, &ctx);
    let snapshot = evaluate_assessment_applicability(&def, &ctx);
    assert_eq!(
        snapshot.requirement_decisions[0].decision,
        from_rule.decision
    );
    assert_eq!(snapshot.control_decisions[0].decision, from_rule.decision);
    assert_eq!(
        snapshot.requirement_decisions[0].decision,
        ApplicabilityDecision::ManualDeterminationRequired
    );
}

/// P10: evaluator has no framework/provider branches
#[test]
fn p10_t16_evaluator_has_no_framework_provider_branches() {
    let src = applicability_sources();
    for needle in [
        "FrameworkProfile",
        "Iso27001",
        "GitHubCollector",
        "GITHUB_EVIDENCE_TYPES",
        "applicability.toml",
        "collector_id",
        "OrgContext",
        "evaluate_org_context",
        "reqwest",
        "ureq",
    ] {
        assert!(
            !src.contains(needle),
            "generic evaluator must not contain `{needle}`"
        );
    }
    for required in [
        "fn evaluate_applicability",
        "fn build_applicability_context",
        "fn evaluate_assessment_applicability",
        "ManualDeterminationRequired",
        "struct ApplicabilitySnapshot",
        "struct ApplicabilityContext",
    ] {
        assert!(
            src.contains(required),
            "engine surface missing `{required}`"
        );
    }

    let ir = read_repo_file("crates/weeping-angel-assurance-ir/src/applicability.rs");
    assert!(
        ir.contains("Declarative applicability. The IR does not evaluate platform facts."),
        "IR stays declarative"
    );
    assert!(
        !ir.contains("fn evaluate_applicability"),
        "IR must not grow a fact evaluator"
    );
    assert_eq!(
        ApplicabilityRule::processes_personal_data(true).statically_applicable(),
        None,
        "statically_applicable meaning is unchanged"
    );

    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        assurance_lib.contains("mod applicability")
            || assurance_lib.contains("pub mod applicability"),
        "assurance facade must declare the applicability module"
    );
}
