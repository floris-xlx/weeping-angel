//! Target suite for ISMS context IR.
//!
//! Encodes DESIRED behavior in `docs/specs/isms-context.md` §4 / §5.2
//! (CTX-T01–T14). Must stay RED on CURRENT HEAD: `IsmsContext` and the
//! organization / issue / party / obligation / objective / cadence /
//! lifecycle graph do not exist. Do not weaken these assertions to match
//! today's assessment-only IR, and do not implement the feature in this
//! suite.
//!
//! Neighbor IsmsContext-absence found-cases live in
//! `sdd_risk_methodology_baseline` and
//! `sdd_continuous_assurance_scheduler_baseline`; those suites are not this
//! slice's tests.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind,
    BusinessUnit, BusinessUnitId, CadenceInterval, CadenceUnit, ContextIssue, Control,
    EvidenceRequirement, EvidenceType, GovernanceCadence, Identity, IdentityId, IdentityKind,
    InterestedParty, InterestedPartyId, InterestedPartyKind, IrValidationError, IsmsContext,
    IsmsContextId, IsmsLifecycleStatus, IssueId, IssueKind, ManagementSystemScope, Mapping,
    ObjectiveId, Obligation, ObligationId, Organization, OrganizationId, PrincipalRef, Requirement,
    Risk, RiskMethodologyId, ScopeId, SecurityObjective, SubjectSelector, ValidateIr, Vendor,
    VendorId, canonical_digest, explain_isms_context, typed_canonical_digest,
    validate_assessment_against_context,
};

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

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn err_text(err: &IrValidationError) -> String {
    err.to_string().to_ascii_lowercase()
}

fn assert_fails_with(result: Result<(), IrValidationError>, needle: &str) {
    let err = match result {
        Err(err) => err,
        Ok(()) => panic!("expected fail-closed validation containing `{needle}`"),
    };
    let text = err_text(&err);
    assert!(
        text.contains(needle),
        "expected `{needle}` in validation error `{err}`"
    );
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

fn golden_organization() -> Organization {
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
    }
}

fn golden_scope() -> ManagementSystemScope {
    ManagementSystemScope {
        id: ScopeId::new("scope:acme-ms"),
        title: "Acme management system".into(),
        summary: Some("Provider-neutral ISMS boundary handle".into()),
    }
}

fn golden_cadence() -> GovernanceCadence {
    GovernanceCadence {
        management_review: CadenceInterval {
            count: 1,
            unit: CadenceUnit::Quarter,
        },
        internal_audit: CadenceInterval {
            count: 1,
            unit: CadenceUnit::Year,
        },
        risk_assessment: CadenceInterval {
            count: 1,
            unit: CadenceUnit::Year,
        },
    }
}

/// Representative ISMS context: one org, two business units, one internal
/// and one external issue, parties, objectives, methodology reference.
fn golden_context() -> IsmsContext {
    let mut ctx = IsmsContext::new(
        IsmsContextId::new("isms:acme"),
        golden_organization(),
        golden_scope(),
    );
    ctx.issues = vec![
        ContextIssue {
            id: IssueId::new("issue:internal:staffing"),
            kind: IssueKind::Internal,
            title: "Specialist staffing coverage".into(),
            description: "Internal capacity for security operations".into(),
        },
        ContextIssue {
            id: IssueId::new("issue:external:regulation"),
            kind: IssueKind::External,
            title: "External regulatory pressure".into(),
            description: "External obligation climate for customer data".into(),
        },
    ];
    ctx.interested_parties = vec![InterestedParty {
        id: InterestedPartyId::new("party:customers"),
        name: "Customers".into(),
        kind: InterestedPartyKind::Customer,
        obligation_ids: vec![ObligationId::new("obligation:customer-security")],
    }];
    ctx.obligations = vec![Obligation {
        id: ObligationId::new("obligation:customer-security"),
        title: "Customer security commitments".into(),
        description: Some("Protect customer information in contracted services".into()),
        interested_party_id: InterestedPartyId::new("party:customers"),
    }];
    ctx.objectives = vec![SecurityObjective {
        id: ObjectiveId::new("objective:reduce-incidents"),
        title: "Reduce security incidents".into(),
        description: "Declared objective; not a point-in-time score".into(),
        owner: Some(PrincipalRef::Team("security-governance".into())),
    }];
    ctx.risk_methodology_id = Some(RiskMethodologyId::new("risk-method:acme-v1"));
    ctx.cadence = Some(golden_cadence());
    ctx.lifecycle = IsmsLifecycleStatus::Active;
    ctx.superseded_by = None;
    ctx
}

fn assert_representative_graph(ctx: &IsmsContext) {
    assert_eq!(ctx.schema_version, ASSURANCE_IR_SCHEMA);
    assert_eq!(ctx.schema_version, "assurance-ir/v1");
    assert_eq!(ctx.id.as_str(), "isms:acme");
    assert_eq!(ctx.organization.id.as_str(), "org:acme");
    assert_eq!(ctx.organization.legal_name, "Acme Corp");
    assert_eq!(ctx.organization.business_units.len(), 2);
    let bu_ids: BTreeSet<&str> = ctx
        .organization
        .business_units
        .iter()
        .map(|bu| bu.id.as_str())
        .collect();
    assert_eq!(bu_ids, BTreeSet::from(["bu:finance", "bu:engineering"]));
    assert_eq!(ctx.scope.id.as_str(), "scope:acme-ms");
    assert_eq!(ctx.organization.scope_id.as_str(), ctx.scope.id.as_str());
    let mut internal = 0;
    let mut external = 0;
    for issue in &ctx.issues {
        match issue.kind {
            IssueKind::Internal => internal += 1,
            IssueKind::External => external += 1,
        }
    }
    assert_eq!(
        internal, 1,
        "representative fixture needs one internal issue"
    );
    assert_eq!(
        external, 1,
        "representative fixture needs one external issue"
    );
    assert!(
        !ctx.interested_parties.is_empty() && !ctx.obligations.is_empty(),
        "representative fixture needs interested parties and obligations"
    );
    assert!(
        !ctx.objectives.is_empty(),
        "representative fixture needs at least one security objective"
    );
    assert_eq!(
        ctx.risk_methodology_id
            .as_ref()
            .map(RiskMethodologyId::as_str),
        Some("risk-method:acme-v1")
    );
    assert_eq!(ctx.lifecycle, IsmsLifecycleStatus::Active);
    assert!(ctx.cadence.is_some());
}

/// CTX-T01: golden `isms-context.json` constructs, validates, round-trips
/// byte-identically, and has a stable `canonical_digest`.
#[test]
fn ctx_t01_golden_isms_context_round_trips() {
    let constructed = golden_context();
    constructed
        .validate()
        .expect("constructed representative IsmsContext must validate");
    assert_representative_graph(&constructed);

    let path = ir_fixture("isms-context.json");
    assert!(
        path.is_file(),
        "representative fixture must exist: {}",
        path.display()
    );
    let raw = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let loaded: IsmsContext = serde_json::from_slice(&raw)
        .unwrap_or_else(|e| panic!("isms-context.json must decode as IsmsContext: {e}"));
    loaded.validate().expect("isms-context.json must validate");
    assert_representative_graph(&loaded);

    let once = serde_json::to_vec(&loaded).expect("serialize loaded context");
    let round: IsmsContext = serde_json::from_slice(&once).expect("deserialize serialized context");
    let twice = serde_json::to_vec(&round).expect("re-serialize context");
    assert_eq!(
        once, twice,
        "IsmsContext serde round-trip must be byte-identical"
    );

    let digest_a = canonical_digest(&loaded).expect("canonical_digest");
    let digest_b = canonical_digest(&round).expect("canonical_digest after round-trip");
    assert_eq!(digest_a, digest_b);
    assert_eq!(digest_a.len(), 64);

    let loaded_again: IsmsContext =
        serde_json::from_slice(&raw).expect("second load of isms-context.json");
    assert_eq!(
        canonical_digest(&loaded).unwrap(),
        canonical_digest(&loaded_again).unwrap(),
        "canonical_digest must be stable across two fixture loads"
    );

    let typed = typed_canonical_digest("IsmsContext", &loaded).expect("typed digest");
    assert_eq!(typed.len(), 64);
    assert_ne!(typed, digest_a);

    let json = serde_json::to_value(&loaded).unwrap();
    assert_eq!(json["schemaVersion"], "assurance-ir/v1");
    assert!(
        json.get("schema_version").is_none(),
        "IsmsContext is a domain record: schemaVersion, not schema_version"
    );
    assert!(json["organization"].is_object());
    assert!(json["organizations"].is_null() || json.get("organizations").is_none());
}

/// CTX-T02: `AssessmentDefinition::new` and golden `assessment.json` stay
/// valid; missing `isms_context_id` defaults to `None` (snake_case).
#[test]
fn ctx_t02_assessment_definition_stays_compatible() {
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.target"));
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assert!(assessment.isms_context_id.is_none());
    assessment
        .validate()
        .expect("AssessmentDefinition::new must remain valid without an IsmsContext");

    let encoded = serde_json::to_value(&assessment).unwrap();
    let obj = encoded.as_object().expect("assessment JSON object");
    assert!(obj.contains_key("schema_version"));
    assert!(
        obj.get("ismsContextId").is_none(),
        "assessment document must not switch to camelCase ismsContextId"
    );

    let path = ir_fixture("assessment.json");
    let raw = fs::read_to_string(&path).unwrap();
    let golden: AssessmentDefinition =
        serde_json::from_str(&raw).expect("golden assessment.json must still decode");
    golden.validate().unwrap();
    assert!(
        golden.isms_context_id.is_none(),
        "missing isms_context_id must serde-default to None"
    );
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert!(
        parsed.get("isms_context_id").is_none(),
        "do not require isms_context_id on existing assessment.json"
    );

    let mut linked = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.linked"));
    linked.isms_context_id = Some(IsmsContextId::new("isms:acme"));
    let linked_json = serde_json::to_value(&linked).unwrap();
    assert!(
        linked_json.get("isms_context_id").is_some(),
        "optional pointer serializes as snake_case isms_context_id"
    );
    assert!(linked_json.get("ismsContextId").is_none());
}

/// CTX-T03: duplicate ids fail `validate()` with a deterministic `duplicate` error.
#[test]
fn ctx_t03_duplicate_ids_fail_closed() {
    let mut parties = golden_context();
    parties
        .interested_parties
        .push(parties.interested_parties[0].clone());
    assert_fails_with(parties.validate(), "duplicate");

    let mut obligations = golden_context();
    obligations
        .obligations
        .push(obligations.obligations[0].clone());
    assert_fails_with(obligations.validate(), "duplicate");

    let mut issues = golden_context();
    issues.issues.push(issues.issues[0].clone());
    assert_fails_with(issues.validate(), "duplicate");

    let mut objectives = golden_context();
    objectives.objectives.push(objectives.objectives[0].clone());
    assert_fails_with(objectives.validate(), "duplicate");

    let mut units = golden_context();
    units
        .organization
        .business_units
        .push(units.organization.business_units[0].clone());
    assert_fails_with(units.validate(), "duplicate");

    let mut self_successor = golden_context();
    self_successor.lifecycle = IsmsLifecycleStatus::Superseded;
    self_successor.superseded_by = Some(self_successor.id.clone());
    let err = self_successor
        .validate()
        .expect_err("self-successor must fail closed");
    let text = err_text(&err);
    assert!(
        text.contains("duplicate") || text.contains("lifecycle") || text.contains("dangling"),
        "self-successor must fail deterministically, got `{err}`"
    );
}

/// CTX-T04: dangling internal refs and pair-validator population mismatches fail closed.
#[test]
fn ctx_t04_dangling_refs_fail_closed() {
    let mut obligation_ids = golden_context();
    obligation_ids.interested_parties[0].obligation_ids =
        vec![ObligationId::new("obligation:missing")];
    assert_fails_with(obligation_ids.validate(), "dangling");

    let mut party_id = golden_context();
    party_id.obligations[0].interested_party_id = InterestedPartyId::new("party:missing");
    assert_fails_with(party_id.validate(), "dangling");

    let mut one_way = golden_context();
    one_way.interested_parties[0].obligation_ids.clear();
    assert_fails_with(one_way.validate(), "dangling");

    let mut parent = golden_context();
    parent.organization.business_units[1].parent_id =
        Some(BusinessUnitId::new("bu:does-not-exist"));
    assert_fails_with(parent.validate(), "dangling");

    let mut scope_mismatch = golden_context();
    scope_mismatch.organization.scope_id = ScopeId::new("scope:other");
    assert_fails_with(scope_mismatch.validate(), "dangling");

    let mut ctx = golden_context();
    ctx.asset_ids.insert(AssetId::new("asset:repo-payments"));
    ctx.vendor_ids.insert(VendorId::new("vendor:payroll"));
    ctx.identity_ids.insert(IdentityId::new("identity:alice"));

    let mut mismatched = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.pair"));
    mismatched.isms_context_id = Some(IsmsContextId::new("isms:other"));
    assert_fails_with(
        validate_assessment_against_context(&mismatched, &ctx),
        "dangling",
    );

    let mut missing_inventory =
        AssessmentDefinition::new(AssessmentId::new("assess.isms-context.inventory"));
    missing_inventory.isms_context_id = Some(ctx.id.clone());
    assert_fails_with(
        validate_assessment_against_context(&missing_inventory, &ctx),
        "dangling",
    );

    let mut standalone =
        AssessmentDefinition::new(AssessmentId::new("assess.isms-context.standalone"));
    standalone.isms_context_id = None;
    validate_assessment_against_context(&standalone, &ctx)
        .expect("pair validator is a no-op when assessment has no isms_context_id");

    let mut paired = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.paired-ok"));
    paired.isms_context_id = Some(ctx.id.clone());
    paired.assets.push(Asset::new(
        AssetId::new("asset:repo-payments"),
        AssetKind::Repository,
        "payments",
    ));
    paired
        .vendors
        .push(Vendor::new(VendorId::new("vendor:payroll"), "Payroll Co"));
    paired.identities.push(Identity::new(
        IdentityId::new("identity:alice"),
        IdentityKind::User,
    ));
    validate_assessment_against_context(&paired, &ctx)
        .expect("matching pointer and inventories must pair-validate");

    ctx.validate()
        .expect("standalone IsmsContext::validate must not require assessment inventories");
}

/// CTX-T05: empty / whitespace required identity and title fields fail closed.
#[test]
fn ctx_t05_empty_required_identity_and_title_fields_fail_closed() {
    let mut legal = golden_context();
    legal.organization.legal_name = "   ".into();
    assert_fails_with(legal.validate(), "empty");

    let mut issue = golden_context();
    issue.issues[0].title.clear();
    assert_fails_with(issue.validate(), "empty");

    let mut party = golden_context();
    party.interested_parties[0].name = " ".into();
    assert_fails_with(party.validate(), "empty");

    let mut obligation = golden_context();
    obligation.obligations[0].title.clear();
    assert_fails_with(obligation.validate(), "empty");

    let mut objective = golden_context();
    objective.objectives[0].title = "\t".into();
    assert_fails_with(objective.validate(), "empty");

    let mut scope = golden_context();
    scope.scope.title.clear();
    assert_fails_with(scope.validate(), "empty");
}

/// CTX-T06: impossible lifecycle combinations fail closed; Draft may omit
/// methodology and cadence; unknown lifecycle tags fail at deserialize.
#[test]
fn ctx_t06_impossible_lifecycle_states_fail_closed() {
    let mut missing_successor = golden_context();
    missing_successor.lifecycle = IsmsLifecycleStatus::Superseded;
    missing_successor.superseded_by = None;
    let err = missing_successor
        .validate()
        .expect_err("superseded without supersededBy");
    let text = err_text(&err);
    assert!(
        text.contains("lifecycle") || text.contains("superseded"),
        "expected lifecycle error, got `{err}`"
    );

    let mut successor_on_active = golden_context();
    successor_on_active.lifecycle = IsmsLifecycleStatus::Active;
    successor_on_active.superseded_by = Some(IsmsContextId::new("isms:acme-v2"));
    let err = successor_on_active
        .validate()
        .expect_err("supersededBy is illegal on active");
    let text = err_text(&err);
    assert!(
        text.contains("lifecycle") || text.contains("superseded"),
        "expected lifecycle error, got `{err}`"
    );

    let mut active_no_method = golden_context();
    active_no_method.lifecycle = IsmsLifecycleStatus::Active;
    active_no_method.risk_methodology_id = None;
    let err = active_no_method
        .validate()
        .expect_err("active requires riskMethodologyId");
    let text = err_text(&err);
    assert!(
        text.contains("lifecycle") || text.contains("methodology"),
        "expected lifecycle/methodology error, got `{err}`"
    );

    let mut review_no_cadence = golden_context();
    review_no_cadence.lifecycle = IsmsLifecycleStatus::UnderReview;
    review_no_cadence.cadence = None;
    let err = review_no_cadence
        .validate()
        .expect_err("underReview requires cadence");
    let text = err_text(&err);
    assert!(
        text.contains("lifecycle") || text.contains("cadence"),
        "expected lifecycle/cadence error, got `{err}`"
    );

    let mut zero = golden_context();
    if let Some(cadence) = zero.cadence.as_mut() {
        cadence.management_review.count = 0;
    }
    let err = zero.validate().expect_err("cadence count 0 is impossible");
    let text = err_text(&err);
    assert!(
        text.contains("lifecycle") || text.contains("cadence") || text.contains("count"),
        "expected cadence error, got `{err}`"
    );

    let mut draft = golden_context();
    draft.lifecycle = IsmsLifecycleStatus::Draft;
    draft.risk_methodology_id = None;
    draft.cadence = None;
    draft.superseded_by = None;
    draft
        .validate()
        .expect("Draft may omit methodology and cadence");

    let mut wire = serde_json::to_value(golden_context()).unwrap();
    wire["lifecycle"] = Value::String("archived".into());
    let decoded = serde_json::from_value::<IsmsContext>(wire);
    assert!(
        decoded.is_err(),
        "unknown IsmsLifecycleStatus tags must fail closed at deserialize"
    );
}

/// CTX-T07: generic IR JSON keys have no ISO/provider tokens; new IR module
/// stays free of network/SDK types.
#[test]
fn ctx_t07_generic_ir_is_provider_and_framework_neutral() {
    let json = serde_json::to_value(golden_context()).unwrap();
    let mut keys = BTreeSet::new();
    collect_object_keys(&json, &mut keys);
    let forbidden = [
        "annex",
        "soa",
        "clause",
        "iso27001",
        "iso-27001",
        "gdpr",
        "soc2",
        "nis2",
        "dora",
        "aws",
        "amazon",
        "github",
        "entra",
        "okta",
        "cloudflare",
        "gcp",
        "azure",
    ];
    let mut hits = Vec::new();
    for key in &keys {
        let folded = key.to_ascii_lowercase();
        for token in forbidden {
            if folded.contains(token) {
                hits.push(format!("{key} contains {token}"));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "generic IsmsContext JSON keys must not include ISO/provider tokens: {hits:?}"
    );

    let ir_src = crate_sources_joined("weeping-angel-assurance-ir");
    let mut context_files = Vec::new();
    walk_rs_files(&crate_src("weeping-angel-assurance-ir"), &mut context_files);
    let mut module_src = String::new();
    for path in &context_files {
        let src = fs::read_to_string(path).unwrap();
        if src.contains("struct IsmsContext") {
            module_src.push_str(&src);
            module_src.push('\n');
        }
    }
    assert!(
        !module_src.is_empty(),
        "IsmsContext must live in weeping-angel-assurance-ir (not a parallel GRC crate)"
    );
    for needle in ["reqwest::", "GitHubClient", "octocrab", "aws_sdk"] {
        assert!(
            !module_src.contains(needle),
            "ISMS context IR module must not contain `{needle}`"
        );
    }
    assert!(
        !ir_src.contains("pub const ISMS_IR_SCHEMA") && !ir_src.contains("pub const GRC_IR_SCHEMA"),
        "do not fork a parallel ISMS/GRC schema constant"
    );
}

/// CTX-T08: `weeping-angel-framework` stays network-free.
#[test]
fn ctx_t08_framework_crate_remains_network_free() {
    let toml = read_repo_file("crates/weeping-angel-framework/Cargo.toml");
    for forbidden in ["reqwest", "octocrab", "aws-sdk", "aws_sdk", "cloudflare"] {
        assert!(
            !toml.contains(forbidden),
            "weeping-angel-framework Cargo.toml must not mention `{forbidden}`"
        );
    }
    let src = crate_sources_joined("weeping-angel-framework");
    for needle in ["reqwest::", "GitHubClient", "octocrab", "aws_sdk"] {
        assert!(
            !src.contains(needle),
            "weeping-angel-framework sources must not contain `{needle}`"
        );
    }

    let workspace = read_repo_file("Cargo.toml");
    assert!(
        !workspace.contains("weeping-angel-grc"),
        "ISMS context IR must not ship a parallel GRC crate"
    );
}

/// CTX-T09: `explain_isms_context` is deterministic and names the graph.
#[test]
fn ctx_t09_explain_isms_context_is_deterministic() {
    let ctx = golden_context();
    let first = explain_isms_context(&ctx);
    let second = explain_isms_context(&ctx);
    assert_eq!(first, second, "explain_isms_context must be byte-stable");
    assert!(!first.is_empty());
    let folded = first.to_ascii_lowercase();
    for needle in [
        "isms:acme",
        "org:acme",
        "acme corp",
        "finance",
        "engineering",
        "scope:acme-ms",
        "internal",
        "external",
        "customer",
        "reduce",
        "risk-method:acme-v1",
        "active",
    ] {
        assert!(
            folded.contains(needle),
            "explain output must mention `{needle}`, got:\n{first}"
        );
    }
}

/// CTX-T10: `assetIds` insertion order does not change `canonical_digest`.
#[test]
fn ctx_t10_asset_id_set_digest_is_order_independent() {
    let mut first = golden_context();
    first.asset_ids = BTreeSet::new();
    first.asset_ids.insert(AssetId::new("asset:zulu"));
    first.asset_ids.insert(AssetId::new("asset:alpha"));

    let mut second = golden_context();
    second.asset_ids = BTreeSet::new();
    second.asset_ids.insert(AssetId::new("asset:alpha"));
    second.asset_ids.insert(AssetId::new("asset:zulu"));

    assert_eq!(
        canonical_digest(&first).unwrap(),
        canonical_digest(&second).unwrap()
    );
    let encoded = serde_json::to_value(&first).unwrap();
    let ids = encoded["assetIds"]
        .as_array()
        .expect("assetIds serializes as an array");
    let as_str: Vec<&str> = ids.iter().filter_map(Value::as_str).collect();
    assert_eq!(as_str, vec!["asset:alpha", "asset:zulu"]);
}

/// CTX-T11: context `schemaVersion` is `assurance-ir/v1`; no second schema.
#[test]
fn ctx_t11_schema_remains_assurance_ir_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let ctx = IsmsContext::new(
        IsmsContextId::new("isms:draft"),
        golden_organization(),
        golden_scope(),
    );
    assert_eq!(ctx.schema_version, ASSURANCE_IR_SCHEMA);
    assert_eq!(ctx.lifecycle, IsmsLifecycleStatus::Draft);
    assert!(ctx.issues.is_empty());
    assert!(ctx.interested_parties.is_empty());
    assert!(ctx.obligations.is_empty());
    assert!(ctx.objectives.is_empty());
    assert!(ctx.risk_methodology_id.is_none());
    assert!(ctx.cadence.is_none());
    assert!(ctx.superseded_by.is_none());
    assert!(ctx.asset_ids.is_empty());
    assert!(ctx.vendor_ids.is_empty());
    assert!(ctx.identity_ids.is_empty());
    ctx.validate()
        .expect("IsmsContext::new Draft graph must validate");

    let json = serde_json::to_value(&ctx).unwrap();
    assert_eq!(json["schemaVersion"], "assurance-ir/v1");
}

/// CTX-T12: durable definition must not carry assessment-result fields.
#[test]
fn ctx_t12_context_is_definition_not_assessment_results() {
    let json = serde_json::to_value(golden_context()).unwrap();
    let obj: &Map<String, Value> = json.as_object().unwrap();
    for forbidden in [
        "effectiveness",
        "residualScore",
        "residual_score",
        "statementOfApplicability",
        "statement_of_applicability",
        "controlTestResults",
        "control_test_results",
    ] {
        assert!(
            obj.get(forbidden).is_none(),
            "IsmsContext must not serialize `{forbidden}`"
        );
    }
}

/// CTX-T13: existing spine type names still resolve from the IR crate.
#[test]
fn ctx_t13_existing_spine_types_are_not_renamed() {
    let names = [
        std::any::type_name::<Asset>(),
        std::any::type_name::<Vendor>(),
        std::any::type_name::<Risk>(),
        std::any::type_name::<Control>(),
        std::any::type_name::<Requirement>(),
        std::any::type_name::<Mapping>(),
        std::any::type_name::<SubjectSelector>(),
        std::any::type_name::<EvidenceRequirement>(),
        std::any::type_name::<EvidenceType>(),
    ];
    for name in names {
        assert!(
            name.contains("weeping_angel_assurance_ir"),
            "spine type must remain in weeping-angel-assurance-ir, got {name}"
        );
    }
    assert!(
        names.iter().any(|n| n.ends_with("::Asset"))
            && names.iter().any(|n| n.ends_with("::Vendor"))
            && names.iter().any(|n| n.ends_with("::Risk"))
            && names.iter().any(|n| n.ends_with("::Control"))
            && names.iter().any(|n| n.ends_with("::Requirement"))
            && names.iter().any(|n| n.ends_with("::Mapping"))
            && names.iter().any(|n| n.ends_with("::SubjectSelector")),
        "Asset, Vendor, Risk, Control, Requirement, Mapping, SubjectSelector must not be broadly renamed"
    );
}

/// CTX-T14: dual-suite `[[test]]` names are listed in root Cargo.toml.
#[test]
fn ctx_t14_dual_suite_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("name = \"sdd_isms_context_baseline\"")
            && !toml.contains("path = \"tests/contracts/isms_context.baseline.rs\"")
            && toml.contains("name = \"sdd_isms_context_target\"")
            && toml.contains("path = \"tests/contracts/isms_context.target.rs\""),
        "ISMS context IR dual-suite must be explicitly listed (not auto-discovered)"
    );
}
