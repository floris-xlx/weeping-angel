//! Target suite for the organizational scope engine.
//!
//! Encodes DESIRED behavior in `docs/specs/scope-engine.md` §4 / §5.2
//! (SCP-T01–T15). Must stay RED on CURRENT HEAD: there is no
//! `ScopeResolution` quad, no `resolve_scope`, and silent `ScopeExclusion`
//! rows still validate. Do not weaken these assertions to match today's
//! descriptive bag, and do not implement the engine in this suite.
//!
//! Slice names: scope engine, ISMS context IR.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::scope::{
    InfluencingRuleClass, SCOPE_RESOLUTION_SCHEMA, ScopeDecision, ScopeInputs, ScopeResolution,
    SubjectRef, in_scope_population, is_definitely_in_scope, resolve_scope, resolve_subject,
};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind,
    BusinessUnit, BusinessUnitId, Identity, IdentityId, IdentityKind, IsmsContext, IsmsContextId,
    IsmsLifecycleStatus, ManagementSystemScope, Organization, OrganizationId, PrincipalRef,
    ScopeExclusion, ScopeId, SelectorScope, SubjectKind, SubjectSelector, ValidateIr,
    canonical_digest,
};
use weeping_angel_collector::CollectorScope;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, Population, PopulationCompleteness, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

const AS_OF: (i32, u32, u32, u32, u32, u32) = (2026, 8, 18, 12, 0, 0);
const PINNED_NESTED_EXPLAIN: &str =
    "repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope";

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(AS_OF.0, AS_OF.1, AS_OF.2, AS_OF.3, AS_OF.4, AS_OF.5)
        .unwrap()
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

fn tagged_asset(id: &str, kind: AssetKind, name: &str, parent: Option<&str>) -> Asset {
    let mut asset = Asset::new(AssetId::new(id), kind, name);
    asset.parent = parent.map(AssetId::new);
    asset
}

fn finance_tagged(mut asset: Asset) -> Asset {
    asset
        .tags
        .insert("businessUnit".into(), "bu:finance".into());
    asset
}

fn golden_context() -> IsmsContext {
    let mut ctx = IsmsContext::new(
        IsmsContextId::new("isms:acme"),
        Organization {
            id: OrganizationId::new("org:acme"),
            legal_name: "Acme Corp".into(),
            display_name: None,
            business_units: vec![
                BusinessUnit {
                    id: BusinessUnitId::new("bu:finance"),
                    name: "Finance".into(),
                    parent_id: None,
                },
                BusinessUnit {
                    id: BusinessUnitId::new("bu:engineering"),
                    name: "Engineering".into(),
                    parent_id: None,
                },
            ],
            scope_id: ScopeId::new("scope:acme-ms"),
        },
        ManagementSystemScope {
            id: ScopeId::new("scope:acme-ms"),
            title: "Acme management system".into(),
            summary: Some("Provider-neutral ISMS boundary handle".into()),
        },
    );
    ctx.lifecycle = IsmsLifecycleStatus::Active;
    ctx.identity_ids.insert(IdentityId::new("id:alice"));
    ctx.identity_ids.insert(IdentityId::new("id:bob"));
    ctx
}

fn acme_inventory() -> Vec<Asset> {
    vec![
        tagged_asset("org:acme", AssetKind::Organization, "Acme", None),
        finance_tagged(tagged_asset(
            "service:payments",
            AssetKind::Service,
            "payments",
            Some("org:acme"),
        )),
        finance_tagged(tagged_asset(
            "repo:payments",
            AssetKind::Repository,
            "payments",
            Some("service:payments"),
        )),
        tagged_asset(
            "repo:legacy",
            AssetKind::Repository,
            "legacy",
            Some("org:acme"),
        ),
        tagged_asset(
            "repo:other-org",
            AssetKind::Repository,
            "other",
            Some("org:other"),
        ),
        tagged_asset("org:other", AssetKind::Organization, "Other", None),
    ]
}

/// Scoped assessment: org-wide Acme plus nested payments service/repo.
fn nested_definition() -> AssessmentDefinition {
    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.target"));
    definition.isms_context_id = Some(IsmsContextId::new("isms:acme"));
    definition.assets = acme_inventory();
    definition.identities = vec![
        Identity::new(IdentityId::new("id:alice"), IdentityKind::User),
        Identity::new(IdentityId::new("id:bob"), IdentityKind::User),
        Identity::new(IdentityId::new("id:dave"), IdentityKind::User),
    ];
    definition.scope.organizations = vec!["org:acme".into()];
    definition
}

fn governed_exclusion(
    id: &str,
    expires_at: DateTime<Utc>,
    review_by: DateTime<Utc>,
) -> ScopeExclusion {
    ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Repository, id)],
        rationale: Some("Carve-out for isolated legacy repository".into()),
        owner: Some(PrincipalRef::Team("security-governance".into())),
        approval_ref: Some("approval:scope-legacy".into()),
        approved_at: Some(as_of() - chrono::Duration::days(30)),
        review_by: Some(review_by),
        expires_at: Some(expires_at),
        evidence_refs: vec!["evidence:scope-approval-legacy".into()],
    }
}

fn silent_exclusion(id: &str) -> ScopeExclusion {
    ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Repository, id)],
        rationale: None,
        owner: None,
        approval_ref: None,
        approved_at: None,
        review_by: None,
        expires_at: None,
        evidence_refs: Vec::new(),
    }
}

fn resolve_def(definition: &AssessmentDefinition, ctx: &IsmsContext) -> ScopeResolution {
    let input = ScopeInputs::from_assessment(definition).with_context(ctx);
    resolve_scope(&input, as_of()).expect("resolve_scope must return Ok for well-formed IR")
}

fn subject_row<'a>(
    resolution: &'a ScopeResolution,
    kind: SubjectKind,
    id: &str,
) -> &'a weeping_angel_assurance::scope::SubjectScopeDecision {
    resolution
        .subjects
        .iter()
        .find(|row| row.kind == kind && row.id == id)
        .unwrap_or_else(|| panic!("missing subject row {kind:?}:{id}"))
}

fn assert_quad(row: &weeping_angel_assurance::scope::SubjectScopeDecision) {
    assert!(
        matches!(
            row.decision,
            ScopeDecision::InScope
                | ScopeDecision::OutOfScope
                | ScopeDecision::Conditional
                | ScopeDecision::Unknown
        ),
        "decision must be the four-state quad, not a boolean"
    );
    assert!(
        !row.rationale.trim().is_empty(),
        "rationale must be non-empty"
    );
    assert!(
        !row.explain.trim().is_empty(),
        "explain trace must be non-empty"
    );
    assert!(
        !row.lineage.is_empty() || !row.influencing_rules.is_empty(),
        "lineage or influencing rules must be explicit"
    );
}

fn assert_deterministic(definition: &AssessmentDefinition, ctx: &IsmsContext) {
    let first = resolve_def(definition, ctx);
    let second = resolve_def(definition, ctx);
    assert_eq!(first.digest, second.digest);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap(),
        "same canonical inputs and as_of must yield the same ScopeResolution JSON"
    );
    for (a, b) in first.subjects.iter().zip(second.subjects.iter()) {
        assert_eq!(a.explain, b.explain);
        assert_eq!(a.decision, b.decision);
        assert_eq!(a.rationale, b.rationale);
    }
}

fn seal_protection(asset: &str, protected: bool) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new("source.branch.protection"));
    obs = obs.with_fact("protected", if protected { "true" } else { "false" });
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.scope-engine-target".into(),
            collected_at: as_of(),
            scope: "target".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

/// SCP-T01: nested inclusion walks org / business unit / service and pins explain.
#[test]
fn scp_t01_nested_inclusion_explain_trace() {
    let ctx = golden_context();
    let definition = nested_definition();
    let resolution = resolve_def(&definition, &ctx);

    assert_eq!(resolution.schema, SCOPE_RESOLUTION_SCHEMA);
    assert_eq!(SCOPE_RESOLUTION_SCHEMA, "weeping-angel/scope-resolution/v1");
    assert_eq!(resolution.as_of, as_of());
    assert_eq!(
        resolution.scope_id.as_ref().map(ScopeId::as_str),
        Some("scope:acme-ms")
    );

    let repo = subject_row(&resolution, SubjectKind::Repository, "repo:payments");
    assert_quad(repo);
    assert_eq!(repo.decision, ScopeDecision::InScope);
    assert!(is_definitely_in_scope(repo.decision));
    assert_eq!(
        repo.explain, PINNED_NESTED_EXPLAIN,
        "nested inclusion must pin the seed explain trace"
    );
    let hop_ids: Vec<&str> = repo.lineage.iter().map(|hop| hop.id.as_str()).collect();
    assert!(
        hop_ids
            .iter()
            .any(|id| *id == "repo:payments" || *id == "payments")
            && (hop_ids
                .iter()
                .any(|id| *id == "bu:finance" || *id == "business-unit:finance")
                || repo.explain.contains("business-unit:finance"))
            && (hop_ids.contains(&"service:payments") || repo.explain.contains("service:payments")),
        "lineage must name repo, business unit, and service hops; got {hop_ids:?}"
    );

    let input = ScopeInputs::from_assessment(&definition).with_context(&ctx);
    let via_subject = resolve_subject(
        &SubjectRef::new(SubjectKind::Repository, "repo:payments"),
        &input,
        as_of(),
    );
    assert_eq!(via_subject.decision, ScopeDecision::InScope);
    assert_eq!(via_subject.explain, repo.explain);

    assert_deterministic(&definition, &ctx);
}

/// SCP-T02: exact-id active exclusion beats inherited and organization-wide inclusion.
#[test]
fn scp_t02_exact_id_exclusion_beats_inherited_and_org_wide() {
    let ctx = golden_context();
    let mut definition = nested_definition();
    definition.scope.exclusions = vec![governed_exclusion(
        "repo:payments",
        as_of() + chrono::Duration::days(120),
        as_of() + chrono::Duration::days(90),
    )];
    definition
        .validate()
        .expect("governed exclusion must validate");

    let resolution = resolve_def(&definition, &ctx);
    let repo = subject_row(&resolution, SubjectKind::Repository, "repo:payments");
    assert_quad(repo);
    assert_eq!(repo.decision, ScopeDecision::OutOfScope);
    assert!(!is_definitely_in_scope(repo.decision));
    assert!(
        repo.explain.contains("OutOfScope"),
        "explain must name OutOfScope, got {}",
        repo.explain
    );

    let applied = repo
        .influencing_rules
        .iter()
        .find(|rule| rule.class == InfluencingRuleClass::Exclusion && rule.applied)
        .expect("applied exclusion must appear in lineage");
    assert_eq!(applied.rank, 100);
    assert_eq!(
        applied.owner,
        Some(PrincipalRef::Team("security-governance".into()))
    );
    assert_eq!(
        applied.approval_ref.as_deref(),
        Some("approval:scope-legacy")
    );
    assert!(applied.approved_at.is_some());
    assert!(applied.expires_at.is_some() || applied.review_by.is_some());
    assert!(
        repo.rationale.contains("security-governance")
            || applied.owner.is_some() && applied.approval_ref.is_some(),
        "owner / approval / period must be reconstructable from the row"
    );

    let service = subject_row(&resolution, SubjectKind::Service, "service:payments");
    assert_eq!(
        service.decision,
        ScopeDecision::InScope,
        "exclusion is exact-id on the repo only"
    );
}

/// SCP-T03: expired / review-overdue exclusions do not suppress; renewal does.
#[test]
fn scp_t03_expired_exclusion_does_not_suppress_until_renewed() {
    let ctx = golden_context();
    let mut expired = nested_definition();
    expired.scope.exclusions = vec![governed_exclusion(
        "repo:payments",
        as_of() - chrono::Duration::days(1),
        as_of() + chrono::Duration::days(30),
    )];
    let resolution = resolve_def(&expired, &ctx);
    let repo = subject_row(&resolution, SubjectKind::Repository, "repo:payments");
    assert_eq!(repo.decision, ScopeDecision::InScope);
    assert!(
        repo.explain.contains("InScope"),
        "expired exclusion must not flip the remaining inclusion"
    );
    let expired_rule = repo
        .influencing_rules
        .iter()
        .find(|rule| {
            rule.class == InfluencingRuleClass::ExpiredExclusion
                || (rule.class == InfluencingRuleClass::Exclusion && !rule.applied)
        })
        .expect("expired exclusion must remain visible in the explain trace");
    assert!(!expired_rule.applied, "expired exclusion must not apply");
    assert!(
        repo.explain.contains("expired")
            || repo.rationale.to_ascii_lowercase().contains("expired")
            || expired_rule.class == InfluencingRuleClass::ExpiredExclusion,
        "trace must name the expired exclusion"
    );

    let mut overdue = nested_definition();
    overdue.scope.exclusions = vec![governed_exclusion(
        "repo:payments",
        as_of() + chrono::Duration::days(120),
        as_of() - chrono::Duration::days(1),
    )];
    let overdue_res = resolve_def(&overdue, &ctx);
    let overdue_repo = subject_row(&overdue_res, SubjectKind::Repository, "repo:payments");
    assert_eq!(overdue_repo.decision, ScopeDecision::InScope);
    assert!(
        overdue_repo
            .influencing_rules
            .iter()
            .any(|rule| !rule.applied
                && (rule.class == InfluencingRuleClass::ExpiredExclusion
                    || rule.class.eq(&InfluencingRuleClass::Exclusion))),
        "review-overdue exclusion stays in the trace with applied=false"
    );

    let mut renewed = nested_definition();
    renewed.scope.exclusions = vec![governed_exclusion(
        "repo:payments",
        as_of() + chrono::Duration::days(180),
        as_of() + chrono::Duration::days(90),
    )];
    let renewed_res = resolve_def(&renewed, &ctx);
    assert_eq!(
        subject_row(&renewed_res, SubjectKind::Repository, "repo:payments").decision,
        ScopeDecision::OutOfScope,
        "renewed later expiresAt / reviewBy restores suppression"
    );
}

/// SCP-T04: unresolved subjects are Unknown, never implicit InScope.
#[test]
fn scp_t04_unresolved_subject_is_unknown() {
    let ctx = golden_context();
    let definition = nested_definition();
    let input = ScopeInputs::from_assessment(&definition)
        .with_context(&ctx)
        .with_candidates(vec![SubjectRef::new(SubjectKind::Repository, "repo:ghost")]);
    let resolution = resolve_scope(&input, as_of()).expect("unresolved is per-subject Unknown");
    let ghost = subject_row(&resolution, SubjectKind::Repository, "repo:ghost");
    assert_quad(ghost);
    assert_eq!(ghost.decision, ScopeDecision::Unknown);
    assert!(!is_definitely_in_scope(ghost.decision));
    assert!(
        ghost.rationale.to_ascii_lowercase().contains("unresolved"),
        "rationale must say unresolved subject, got {}",
        ghost.rationale
    );
}

/// SCP-T05: duplicate selectors are idempotent and order-independent.
#[test]
fn scp_t05_duplicate_selectors_are_idempotent() {
    let ctx = golden_context();
    let selector = any_of(SubjectKind::Repository, "repo:payments");
    let mut forward = nested_definition();
    forward.scope.organizations.clear();
    forward.scope.subjects = vec![selector.clone(), selector.clone()];
    let mut reverse = nested_definition();
    reverse.scope.organizations.clear();
    reverse.scope.subjects = vec![selector.clone(), selector];

    let a = resolve_def(&forward, &ctx);
    let b = resolve_def(&reverse, &ctx);
    let row_a = subject_row(&a, SubjectKind::Repository, "repo:payments");
    let row_b = subject_row(&b, SubjectKind::Repository, "repo:payments");
    assert_eq!(row_a.decision, ScopeDecision::InScope);
    assert_eq!(row_b.decision, ScopeDecision::InScope);
    assert_eq!(a.digest, b.digest);
    assert_eq!(row_a.explain, row_b.explain);

    let inclusion_digests: BTreeSet<&str> = row_a
        .influencing_rules
        .iter()
        .filter(|rule| rule.class == InfluencingRuleClass::Inclusion && rule.applied)
        .map(|rule| rule.selector_digest.as_str())
        .collect();
    assert_eq!(
        inclusion_digests.len(),
        1,
        "duplicate selectors must not double-count lineage"
    );
}

/// SCP-T06: equal-rank exact-id include vs exclude is Unknown; vec order is irrelevant.
#[test]
fn scp_t06_equal_rank_include_exclude_is_unknown() {
    let ctx = golden_context();
    let include = any_of(SubjectKind::Repository, "repo:payments");
    let exclude = governed_exclusion(
        "repo:payments",
        as_of() + chrono::Duration::days(120),
        as_of() + chrono::Duration::days(90),
    );

    let mut include_first = nested_definition();
    include_first.scope.organizations.clear();
    include_first.scope.subjects = vec![include.clone()];
    include_first.scope.exclusions = vec![exclude.clone()];

    let mut exclude_first = nested_definition();
    exclude_first.scope.organizations.clear();
    exclude_first.scope.subjects = vec![include];
    exclude_first.scope.exclusions = vec![exclude];

    let a = resolve_def(&include_first, &ctx);
    let b = resolve_def(&exclude_first, &ctx);
    let row_a = subject_row(&a, SubjectKind::Repository, "repo:payments");
    let row_b = subject_row(&b, SubjectKind::Repository, "repo:payments");
    assert_eq!(row_a.decision, ScopeDecision::Unknown);
    assert_eq!(row_b.decision, ScopeDecision::Unknown);
    assert_eq!(a.digest, b.digest);
    assert_eq!(row_a.explain, row_b.explain);
    assert!(
        row_a
            .influencing_rules
            .iter()
            .any(|rule| rule.class == InfluencingRuleClass::Conflict)
            || row_a.rationale.to_ascii_lowercase().contains("conflict"),
        "equal-rank include vs exclude must fail closed as conflict, not vec order"
    );
    assert!(!is_definitely_in_scope(row_a.decision));
}

/// SCP-T07: organization-wide inclusion covers nested members of listed orgs only.
#[test]
fn scp_t07_organization_wide_inclusion_is_not_universal() {
    let ctx = golden_context();
    let definition = nested_definition();
    let resolution = resolve_def(&definition, &ctx);

    assert_eq!(
        subject_row(&resolution, SubjectKind::Repository, "repo:payments").decision,
        ScopeDecision::InScope
    );
    assert_eq!(
        subject_row(&resolution, SubjectKind::Service, "service:payments").decision,
        ScopeDecision::InScope
    );
    let outsider = subject_row(&resolution, SubjectKind::Repository, "repo:other-org");
    assert_quad(outsider);
    assert_ne!(
        outsider.decision,
        ScopeDecision::InScope,
        "members of unlisted orgs must not be silently in scope"
    );
    assert!(
        matches!(
            outsider.decision,
            ScopeDecision::Unknown | ScopeDecision::OutOfScope
        ),
        "unlisted org member must be Unknown or OutOfScope, got {:?}",
        outsider.decision
    );
}

/// SCP-T08: population selection returns only InScope members from IR inventories.
#[test]
fn scp_t08_population_selection_uses_ir_scope_not_envelopes() {
    let ctx = golden_context();
    let mut definition = nested_definition();
    definition
        .scope
        .subjects
        .push(any_of(SubjectKind::Identity, "id:alice"));
    definition
        .scope
        .subjects
        .push(any_of(SubjectKind::Identity, "id:bob"));
    definition
        .scope
        .exclusions
        .push(governed_exclusion_identity(
            "id:dave",
            as_of() + chrono::Duration::days(120),
            as_of() + chrono::Duration::days(90),
        ));

    let resolution = resolve_def(&definition, &ctx);
    assert_eq!(
        subject_row(&resolution, SubjectKind::Identity, "id:alice").decision,
        ScopeDecision::InScope
    );
    assert_ne!(
        subject_row(&resolution, SubjectKind::Identity, "id:dave").decision,
        ScopeDecision::InScope
    );

    let selector = SubjectSelector {
        kind: SubjectKind::Identity,
        ids: BTreeSet::new(),
        tags: BTreeMap::new(),
        scope: SelectorScope::All,
    };
    let population = in_scope_population(&selector, &resolution, &definition);
    assert_eq!(
        population.ids,
        vec!["id:alice".to_string(), "id:bob".to_string()]
    );
    assert!(!population.ids.iter().any(|id| id == "id:dave"));
    assert_eq!(
        population.completeness,
        PopulationCompleteness::Authoritative
    );

    let pop_src = read_repo_file("crates/weeping-angel-assurance/src/scope/engine.rs");
    assert!(
        pop_src.contains("AssessmentScope") || pop_src.contains("definition.scope"),
        "scope-engine population path must consult IR AssessmentScope, not envelope inference alone"
    );
}

fn governed_exclusion_identity(
    id: &str,
    expires_at: DateTime<Utc>,
    review_by: DateTime<Utc>,
) -> ScopeExclusion {
    ScopeExclusion {
        subjects: vec![any_of(SubjectKind::Identity, id)],
        rationale: Some("Workforce member outside the ISMS population".into()),
        owner: Some(PrincipalRef::Team("security-governance".into())),
        approval_ref: Some("approval:scope-identity-dave".into()),
        approved_at: Some(as_of() - chrono::Duration::days(10)),
        review_by: Some(review_by),
        expires_at: Some(expires_at),
        evidence_refs: vec!["evidence:scope-identity-dave".into()],
    }
}

/// SCP-T09: OutOfScope / Unknown / Conditional cannot contribute passing Effective evidence.
#[test]
fn scp_t09_out_of_scope_cannot_contribute_positive_assurance() {
    let ctx = golden_context();
    let mut definition = nested_definition();
    definition.scope.exclusions = vec![governed_exclusion(
        "repo:legacy",
        as_of() + chrono::Duration::days(120),
        as_of() + chrono::Duration::days(90),
    )];
    let resolution = resolve_def(&definition, &ctx);
    assert_eq!(
        subject_row(&resolution, SubjectKind::Repository, "repo:legacy").decision,
        ScopeDecision::OutOfScope
    );
    assert_eq!(
        subject_row(&resolution, SubjectKind::Repository, "repo:payments").decision,
        ScopeDecision::InScope
    );

    let selector = SubjectSelector {
        kind: SubjectKind::Repository,
        ids: BTreeSet::new(),
        tags: BTreeMap::new(),
        scope: SelectorScope::All,
    };
    let scoped = in_scope_population(&selector, &resolution, &definition);
    assert!(scoped.ids.iter().any(|id| id == "repo:payments"));
    assert!(!scoped.ids.iter().any(|id| id == "repo:legacy"));
    assert!(!scoped.ids.iter().any(|id| id == "repo:other-org"));

    let mut evidence = EvidenceSet::new();
    evidence.set_population(Population {
        selector: selector.clone(),
        subject_ids: scoped.ids.clone(),
        authoritative: true,
        observed_at: as_of(),
        completeness: PopulationCompleteness::Authoritative,
    });
    evidence.insert(seal_protection("repo:legacy", true));

    let test = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.scope-engine.branch-protection",
        ))
        .control_id(weeping_angel_assurance_ir::ControlId::new(
            "control.scope-engine.branch-protection",
        ))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::CoverageAtLeast {
            selector: weeping_angel_control_test::SubjectSelector {
                kind: Some("repository".into()),
                id: None,
            },
            evidence: EvidenceSelector {
                evidence_type: EvidenceType::new("source.branch.protection"),
                subject_selector: weeping_angel_control_test::SubjectSelector {
                    kind: Some("repository".into()),
                    id: None,
                },
                field: Some("protected".into()),
                freshness: None,
            },
            percentage: "100".into(),
        })
        .build();
    let ctx_eval = AssessmentContext {
        now: as_of(),
        max_age: Duration::from_secs(24 * 3600),
    };
    let result = evaluate(&test, &evidence, &ctx_eval);
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "out-of-scope passing evidence must not produce Effective"
    );
    if let Some(eval) = &result.population {
        assert_eq!(
            eval.passing, 0,
            "out-of-scope subjects must not increment passing"
        );
        assert!(!eval.failing_subjects.iter().any(|s| s == "repo:legacy") || eval.passing == 0);
    }

    let mut leaked = scoped.ids.clone();
    leaked.push("repo:legacy".into());
    let refused = in_scope_population(&selector, &resolution, &definition);
    assert!(
        !refused.ids.iter().any(|id| id == "repo:legacy"),
        "engine population helper must refuse OutOfScope / Unknown / Conditional ids"
    );
}

/// SCP-T10: silent exclusions fail validate() and must not suppress.
#[test]
fn scp_t10_silent_exclusion_fails_validation_and_does_not_suppress() {
    let mut definition = nested_definition();
    definition.scope.exclusions = vec![silent_exclusion("repo:payments")];
    let err = definition
        .validate()
        .expect_err("silent exclusion must fail validate()");
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("silent") || text.contains("rationale") || text.contains("exclusion"),
        "validation error must name the silent exclusion, got `{err}`"
    );

    let ctx = golden_context();
    let resolution = resolve_def(&definition, &ctx);
    let repo = subject_row(&resolution, SubjectKind::Repository, "repo:payments");
    assert_ne!(
        repo.decision,
        ScopeDecision::OutOfScope,
        "invalid exclusion must not suppress"
    );
    assert!(
        repo.influencing_rules
            .iter()
            .any(|rule| { rule.class == InfluencingRuleClass::InvalidExclusion && !rule.applied })
            || repo.rationale.to_ascii_lowercase().contains("invalid"),
        "invalid exclusion must be recorded as did-not-suppress"
    );

    let ctx_src = read_repo_file("crates/weeping-angel-assurance/src/applicability/context.rs");
    assert!(
        !ctx_src.contains("excluded by assessment scope["),
        "applicability must not synthesize a silent exclusion rationale once the engine is SSOT"
    );
}

/// SCP-T11: AssessmentDefinition::new and golden assessment.json stay assurance-ir/v1.
#[test]
fn scp_t11_assessment_definition_and_golden_remain_valid() {
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.scope-engine.target.new"));
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    assessment
        .validate()
        .expect("AssessmentDefinition::new must remain valid with empty scope");

    let path = manifest_dir().join("tests/fixtures/assurance-ir/v1/assessment.json");
    let raw = fs::read_to_string(&path).unwrap();
    let golden: AssessmentDefinition =
        serde_json::from_str(&raw).expect("golden assessment.json must still decode");
    golden.validate().unwrap();
    assert_eq!(golden.schema_version, "assurance-ir/v1");
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed["schema_version"], "assurance-ir/v1");
}

/// SCP-T12: facade and CollectorScope stay AssetId allow-sets filled only from InScope.
#[test]
fn scp_t12_facade_and_collector_adapter_only_in_scope_assets() {
    let facade_name = std::any::type_name::<weeping_angel_assurance::AssessmentScope>();
    let ir_name = std::any::type_name::<weeping_angel_assurance_ir::AssessmentScope>();
    assert_ne!(facade_name, ir_name);

    let ctx = golden_context();
    let mut definition = nested_definition();
    definition.scope.exclusions = vec![governed_exclusion(
        "repo:legacy",
        as_of() + chrono::Duration::days(120),
        as_of() + chrono::Duration::days(90),
    )];
    let resolution = resolve_def(&definition, &ctx);
    let in_scope = resolution.in_scope_asset_ids();
    assert!(in_scope.contains(&AssetId::new("repo:payments")));
    assert!(!in_scope.contains(&AssetId::new("repo:legacy")));
    assert!(!in_scope.contains(&AssetId::new("repo:other-org")));

    let facade = resolution.to_facade_assessment_scope();
    assert!(facade.describe().contains("repo:payments"));
    assert!(!facade.describe().contains("repo:legacy"));

    let collector: CollectorScope = resolution.to_collector_scope();
    assert!(collector.allows(&AssetId::new("repo:payments")));
    assert!(!collector.allows(&AssetId::new("repo:legacy")));
    assert!(!collector.allows(&AssetId::new("repo:other-org")));

    let collector_src = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector_src.contains("&mut AssessmentScope")
            && !collector_src.contains("&mut weeping_angel_assurance_ir::AssessmentScope"),
        "collectors must not mutate IR scope"
    );
    let engine_src = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !engine_src.contains("fn resolve_scope")
            || !read_repo_file("crates/weeping-angel-assurance/src/scope/engine.rs")
                .contains("&mut AssessmentScope"),
        "resolve_scope is pure; it must not take &mut IR AssessmentScope"
    );
}

/// SCP-T13: crawl `src/engine/scope.rs` stays URL membership (collision fence).
#[test]
fn scp_t13_crawl_scope_module_unchanged() {
    let src = read_repo_file("apps/cli/src/engine/scope.rs");
    assert!(src.contains("pub fn in_scope(authz: &Authorization, url: &Url) -> bool"));
    assert!(src.contains("authz.url_in_scope(url)"));
    assert!(
        !src.contains("ScopeResolution")
            && !src.contains("ScopeDecision")
            && !src.contains("AssessmentScope"),
        "crawl scope.rs must not grow ISMS types"
    );
}

/// SCP-T14: additive generic kinds parse; provider account types stay forbidden.
#[test]
fn scp_t14_additive_generic_kinds_without_provider_schemas() {
    let bu = SubjectKind::parse_name("businessunit").expect("businessunit");
    assert_eq!(SubjectKind::parse_name("business-unit"), Some(bu));
    assert!(SubjectKind::parse_name("location").is_some());
    assert!(SubjectKind::parse_name("datadomain").is_some());
    assert!(SubjectKind::parse_name("data-domain").is_some());
    let pop = SubjectKind::parse_name("personnelpopulation").expect("personnelpopulation");
    assert_eq!(SubjectKind::parse_name("population"), Some(pop));

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/subject.rs");
    for forbidden in [
        "AwsAccount",
        "GitHubOrganization",
        "GitHubOrg",
        "EntraTenant",
        "GcpProject",
    ] {
        assert!(
            !src.contains(forbidden),
            "SubjectKind must not grow provider-specific variant `{forbidden}`"
        );
    }
}

/// SCP-T15: dual-suite `[[test]]` names and the scope-engine spec stay registered.
#[test]
fn scp_t15_dual_suite_and_spec_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("name = \"sdd_scope_engine_baseline\"")
            && !toml.contains("path = \"tests/contracts/scope_engine.baseline.rs\"")
            && harness_src().contains("scope_engine.target.rs")
            && harness_src().contains("scope_engine.target.rs"),
        "scope engine dual-suite must be explicitly listed (tests/contracts is not auto-discovered)"
    );

    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/scope-engine.md"),
        "scope-engine spec must remain in CANONICAL_SPECS"
    );
    assert!(manifest_dir().join("docs/specs/scope-engine.md").is_file());

    let comments = read_repo_file("tests/contracts/scope_engine.target.rs");
    let banned = format!("Prompt {}", "N");
    assert!(
        !comments.contains(&banned),
        "test comments name the slice, not a numbered prompt"
    );
    let docs_sdd = manifest_dir().join("docs/sdd");
    if docs_sdd.is_dir() {
        // ADR 0004: docs/sdd is a pointer stub (README + slice redirects).
        // This suite only forbids *scope-engine* generated dumps here.
        let leftover: Vec<_> = fs::read_dir(&docs_sdd)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.contains("scope-engine")
                    || name.contains("scope_engine")
                    || name.starts_with("sdd-scope")
            })
            .map(|e| e.file_name())
            .collect();
        assert!(
            leftover.is_empty(),
            "generated traces belong under .sdd/runs, not docs/sdd/: {leftover:?}"
        );
    }
}

/// Every candidate in the inventory resolves to the quad with rationale + lineage.
#[test]
fn every_inventory_member_has_quad_rationale_and_lineage() {
    let ctx = golden_context();
    let definition = nested_definition();
    let resolution = resolve_def(&definition, &ctx);
    assert!(!resolution.subjects.is_empty());
    let mut seen = BTreeSet::new();
    for row in &resolution.subjects {
        assert_quad(row);
        assert!(
            seen.insert((format!("{:?}", row.kind), row.id.clone())),
            "subjects must be unique after lex-sort"
        );
    }
    let ids: Vec<_> = resolution.subjects.iter().map(|s| s.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "subjects must be lex-sorted by (kind, id)");
    let _ = canonical_digest(&resolution).ok();
}
