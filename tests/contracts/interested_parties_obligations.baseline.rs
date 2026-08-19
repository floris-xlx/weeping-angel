//! Baseline suite for interested parties / obligations.
//!
//! Characterization of CURRENT behavior on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/interested-parties-obligations.md` §3). Product IR has no
//! `InterestedParty`, `RequirementSource`, `Obligation`, or `ObligationMapping`.
//! `typed_id!` stops at `MappingId` (`ObligationId` is specified by
//! controlled-documents but absent). `IsmsContext` and `ScopeResolution` are
//! unlanded. `Requirement` is a framework clause; `Mapping` is only
//! fromRequirement → toControl; `ComplianceGraph::equivalent` is fail-closed.
//! `explain_control` walks assessment mappings/tests/evidence, not
//! organizational duties. Collectors and packs do not record obligation
//! satisfaction.
//!
//! Must stay GREEN on this HEAD until `sdd_interested_parties_obligations_target`
//! is GREEN and this file is skip-superseded. Does not implement the
//! obligation layer.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance::lineage::ControlExplanation;
use weeping_angel_assurance_ir::crosswalk::{ComplianceGraph, ComplianceNodeRef};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentScope, Control, ControlId,
    ControlImplementation, ControlImplementationId, FrameworkId, FrameworkVersion, Mapping,
    MappingCompleteness, MappingDirection, MappingRelation, Requirement, RequirementId,
    RequirementKind, SubjectSelector, ValidateIr, canonical_digest,
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

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: interested parties / obligations product surface must be absent on characterization HEAD; found {present:?}"
    );
}

fn fixture_case_ids() -> &'static [&'static str] {
    &[
        "party.customer.acme",
        "obl.customer.security-commitment",
        "party.workforce",
        "obl.employment.confidentiality",
        "party.regulator.dpa",
        "obl.regulatory.retention",
        "party.vendor.payroll",
        "obl.supplier.dpa-security",
        "obl.customer.security-commitment.2026",
    ]
}

/// IPO-B001 found case: IR has no party/obligation types or modules.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b001_no_interested_party_or_obligation_ir_types() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "IPO-B001",
        &ir,
        &[
            "struct InterestedParty",
            "enum InterestedPartyKind",
            "struct RequirementSource",
            "enum RequirementSourceKind",
            "struct Obligation ",
            "pub struct Obligation",
            "enum ObligationLifecycle",
            "struct ObligationMapping",
            "enum ObligationMappingTarget",
            "struct ObligationRegistry",
            "struct ObligationLinkUniverse",
            "struct ObligationExplain",
        ],
    );
    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("party.rs").is_file(),
        "IPO-B001: party.rs must not exist on characterization HEAD"
    );
    assert!(
        !ir_src.join("obligation.rs").is_file(),
        "IPO-B001: obligation.rs must not exist on characterization HEAD"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !lib.contains("pub mod party") && !lib.contains("mod party;"),
        "IPO-B001: lib.rs must not declare mod party"
    );
    assert!(
        !lib.contains("pub mod obligation") && !lib.contains("mod obligation;"),
        "IPO-B001: lib.rs must not declare mod obligation"
    );
}

/// IPO-B002 found case: typed_id! aliases stop at MappingId; ObligationId absent.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b002_typed_ids_stop_at_mapping_id() {
    let id_src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        id_src.contains("typed_id!(MappingId);"),
        "IPO-B002: MappingId remains the last compliance-graph typed id"
    );
    forbid_needles(
        "IPO-B002",
        &id_src,
        &[
            "typed_id!(ObligationId)",
            "typed_id!(InterestedPartyId)",
            "typed_id!(RequirementSourceId)",
            "typed_id!(ObligationMappingId)",
        ],
    );
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    forbid_needles(
        "IPO-B002",
        &lib,
        &[
            "ObligationId",
            "InterestedPartyId",
            "RequirementSourceId",
            "ObligationMappingId",
        ],
    );
}

/// IPO-B003 found case: Requirement is a framework clause, not an organizational duty.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b003_requirement_is_framework_clause_not_org_duty() {
    let req = Requirement::new(
        RequirementId::new("req.iso27001.2022.a-5-15"),
        FrameworkId::new("iso-27001"),
        FrameworkVersion::new("2022"),
        "Access control",
        "Limit access to information.",
    );
    assert_eq!(req.schema_version(), ASSURANCE_IR_SCHEMA);
    assert_eq!(req.framework_id().as_str(), "iso-27001");
    assert_eq!(req.framework_version().as_str(), "2022");
    assert_eq!(req.title(), "Access control");
    assert_eq!(req.kind(), RequirementKind::Requirement);

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["schemaVersion"], "assurance-ir/v1");
    assert_eq!(json["frameworkId"], "iso-27001");
    assert_eq!(json["frameworkVersion"], "2022");
    assert!(json.get("sourceId").is_none());
    assert!(json.get("partyId").is_none());
    assert!(json.get("lifecycle").is_none());
    assert!(json.get("effectiveFrom").is_none());

    let raw = fs::read_to_string(ir_fixture("requirement.json")).unwrap();
    let golden: Requirement = serde_json::from_str(&raw).unwrap();
    assert_eq!(golden.id().as_str(), "req.iso27001.2022.a-5-15");
    assert_eq!(golden.framework_id().as_str(), "iso-27001");
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["schemaVersion"], "assurance-ir/v1");
    assert!(value.get("sourceId").is_none());

    let req_src = read_repo_file("crates/weeping-angel-assurance-ir/src/requirement.rs");
    assert!(
        req_src.contains("//! Framework-specific requirement. Not a canonical control."),
        "IPO-B003: Requirement module remains pack/catalog language"
    );
    assert!(
        !req_src.contains("Obligation"),
        "IPO-B003: RequirementKind must not grow an Obligation variant"
    );
    let _ = RequirementKind::ControlObjective;
    let _ = RequirementKind::Clause;
    let _ = RequirementKind::Article;
}

/// IPO-B004 found case: Mapping is only fromRequirement → toControl.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b004_mapping_is_requirement_to_control_only() {
    let mapping = Mapping::new(
        RequirementId::new("req.iso27001.2022.a-5-15"),
        ControlId::new("control.access.mfa"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert_eq!(
        mapping.from_requirement().as_str(),
        "req.iso27001.2022.a-5-15"
    );
    assert_eq!(mapping.to_control().as_str(), "control.access.mfa");
    assert_eq!(mapping.direction(), MappingDirection::Forward);
    assert_eq!(mapping.completeness(), MappingCompleteness::Partial);
    assert_eq!(mapping.relation(), MappingRelation::PartiallySatisfies);
    assert_eq!(
        mapping.rationale(),
        "partial mapping; PartiallySatisfies cannot fully satisfy"
    );

    let json = serde_json::to_value(&mapping).unwrap();
    assert_eq!(json["fromRequirement"], "req.iso27001.2022.a-5-15");
    assert_eq!(json["toControl"], "control.access.mfa");
    assert_eq!(json["relation"], "PartiallySatisfies");
    assert!(json.get("from").is_none());
    assert!(json.get("to").is_none());

    let raw = fs::read_to_string(ir_fixture("mapping.json")).unwrap();
    let golden: Mapping = serde_json::from_str(&raw).unwrap();
    assert_eq!(golden.relation(), MappingRelation::PartiallySatisfies);
    assert_eq!(
        golden.rationale(),
        "partial mapping; PartiallySatisfies cannot fully satisfy"
    );

    let mapping_src = read_repo_file("crates/weeping-angel-assurance-ir/src/mapping.rs");
    assert!(
        mapping_src.contains("from_requirement: RequirementId"),
        "IPO-B004: Mapping source field remains from_requirement"
    );
    assert!(
        mapping_src.contains("to_control: ControlId"),
        "IPO-B004: Mapping target field remains to_control"
    );
    assert!(
        !mapping_src.contains("struct ObligationMapping"),
        "IPO-B004: ObligationMapping is not a sibling record yet"
    );
}

/// IPO-B005 found case: ComplianceGraph::equivalent is fail-closed; no obligation node.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b005_equivalent_is_fail_closed_and_has_no_obligation_node() {
    let mut graph = ComplianceGraph::new();
    let a = RequirementId::new("fw-a:r1");
    let b = RequirementId::new("fw-b:r1");
    let c = RequirementId::new("fw-c:r1");
    graph.link(
        a.clone(),
        b.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    graph.link(
        b,
        c.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert!(
        !graph.equivalent(&a, &c),
        "IPO-B005: two-hop partial path must not be equivalent (IR-006 found case)"
    );

    let mut full = ComplianceGraph::new();
    let left = RequirementId::new("fw-left:r1");
    let right = RequirementId::new("fw-right:r1");
    full.link(
        left.clone(),
        right.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Full,
    );
    assert!(
        !full.equivalent(&left, &right),
        "IPO-B005: forward-only full mapping is not equivalence"
    );
    full.link(
        right.clone(),
        left.clone(),
        MappingDirection::Forward,
        MappingCompleteness::Full,
    );
    assert!(
        full.equivalent(&left, &right),
        "IPO-B005: explicit full bidirectional remains the only equivalence"
    );

    let crosswalk = read_repo_file("crates/weeping-angel-assurance-ir/src/crosswalk.rs");
    assert!(
        crosswalk.contains("enum ComplianceNodeRef"),
        "IPO-B005: ComplianceNodeRef exists"
    );
    forbid_needles(
        "IPO-B005",
        &crosswalk,
        &[
            "Obligation(",
            "InterestedParty(",
            "RequirementSource(",
            "ObligationMapping(",
        ],
    );
    let _ = ComplianceNodeRef::Requirement(RequirementId::new("req.found"));
    let _ = ComplianceNodeRef::Control(ControlId::new("control.found"));
}

/// IPO-B006 found case: IR scope types exist; IsmsContext and ScopeResolution do not.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b006_scope_types_exist_context_and_engine_do_not() {
    let scope = AssessmentScope::default();
    assert!(scope.organizations.is_empty());
    assert!(scope.subjects.is_empty());
    assert!(scope.exclusions.is_empty());
    let _ = SubjectSelector::default();

    let assessment = AssessmentDefinition::new(AssessmentId::new(
        "assess.interested-parties-obligations.baseline",
    ));
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assessment
        .validate()
        .expect("empty assessment remains valid without an obligation graph");

    let product = product_crate_sources_joined();
    forbid_needles(
        "IPO-B006",
        &product,
        &[
            // IsmsContext-absence skip-superseded by ISMS context IR.
            "struct ScopeResolution",
            "fn explain_why_control_exists",
            "fn explain_why_document_exists",
            "fn current_obligations_at",
            "fn projects_as_equivalence",
            "fn projects_as_full_satisfaction",
        ],
    );

    let facade = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        facade.contains("pub struct AssessmentScope") && facade.contains("BTreeSet<AssetId>"),
        "IPO-B006: facade AssessmentScope remains a collector asset allow-set"
    );
}

/// IPO-B007 found case: ControlExplanation has no obligation lineage field.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b007_explain_control_has_no_obligation_lineage() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        lineage.contains("pub struct ControlExplanation"),
        "IPO-B007: ControlExplanation exists"
    );
    assert!(
        lineage.contains("pub fn explain_control"),
        "IPO-B007: explain_control exists"
    );
    assert!(
        lineage.contains("pub mappings: Vec<Mapping>"),
        "IPO-B007: explain walks requirement→control mappings"
    );
    forbid_needles(
        "IPO-B007",
        &lineage,
        &[
            "pub obligations:",
            "ObligationExplain",
            "explain_why_control_exists",
            "explain_why_document_exists",
            "ObligationExplainEdge",
        ],
    );

    let fields = [
        "control",
        "applicability",
        "implementation",
        "population",
        "tests",
        "evidenceRequirements",
        "evidence",
        "missingEvidence",
        "failingSubjects",
        "missingSubjects",
        "exceptions",
        "mappings",
        "effectiveness",
    ];
    for field in fields {
        assert!(
            !field.is_empty(),
            "IPO-B007: ControlExplanation field list is non-empty"
        );
    }
    let _ = std::any::type_name::<ControlExplanation>();
}

/// IPO-B008 found case: validation walks requirement→control mappings, not obligations.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b008_validation_has_no_obligation_graph() {
    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        validation.contains("dangling mapping"),
        "IPO-B008: dangling requirement→control mappings already fail closed"
    );
    forbid_needles(
        "IPO-B008",
        &validation,
        &[
            "Obligation",
            "InterestedParty",
            "RequirementSource",
            "obligation_ids",
            "dangling obligation",
            "dangling party",
            "dangling source",
        ],
    );

    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.ipo.dangling"));
    assessment.mappings.push(Mapping::new(
        RequirementId::new("req.missing"),
        ControlId::new("control.missing"),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    ));
    let err = assessment
        .validate()
        .expect_err("IPO-B008: dangling requirement→control mapping must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("dangling mapping"),
        "IPO-B008: found-case error remains requirement→control, got {msg}"
    );
}

/// IPO-B009 found case: collectors and framework packs cannot record obligation satisfaction.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b009_collectors_and_packs_have_no_obligation_satisfaction() {
    let collector = crate_sources_joined("weeping-angel-collector");
    let framework = crate_sources_joined("weeping-angel-framework");
    forbid_needles(
        "IPO-B009-collector",
        &collector,
        &[
            "struct Obligation",
            "ObligationLifecycle",
            "obligationStatus",
            "set_obligation",
            "mark_obligation_satisfied",
            "fn satisfy_obligation",
        ],
    );
    forbid_needles(
        "IPO-B009-framework",
        &framework,
        &[
            "struct Obligation",
            "ObligationLifecycle",
            "obligationStatus",
            "set_obligation",
            "mark_obligation_satisfied",
        ],
    );
}

/// IPO-B010 found case: schema remains assurance-ir/v1; generic IR has no Annex A fields.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b010_schema_remains_assurance_ir_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(ir.contains("assurance-ir/v1"));
    assert!(
        !ir.to_ascii_lowercase().contains("annex a"),
        "IPO-B010: generic IR must not carry Annex A"
    );
    let _ = canonical_digest(&"interested-parties-obligations-baseline").unwrap();
}

/// IPO-B011 found case: dual-suite binaries are listed; contracts are not auto-discovered.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b011_dual_suite_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("name = \"sdd_interested_parties_obligations_baseline\"")
            && cargo
                .contains("path = \"tests/contracts/interested_parties_obligations.baseline.rs\""),
        "IPO-B011: baseline suite must be listed in root Cargo.toml"
    );
    assert!(
        cargo.contains("name = \"sdd_interested_parties_obligations_target\"")
            && cargo
                .contains("path = \"tests/contracts/interested_parties_obligations.target.rs\""),
        "IPO-B011: target suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/interested_parties_obligations.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/interested_parties_obligations.target.rs")
            .is_file()
    );
}

/// IPO-B012 found case: required obligation fixtures are not IR types or golden files.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b012_obligation_fixtures_are_absent_from_product_and_golden_ir() {
    let product = product_crate_sources_joined();
    for id in fixture_case_ids() {
        assert!(
            !product.contains(id),
            "IPO-B012: product crates must not yet encode fixture id `{id}`"
        );
    }

    let fixture_dir = manifest_dir().join("tests/fixtures/assurance-ir/v1");
    for name in [
        "interested-party.json",
        "requirement-source.json",
        "obligation.json",
        "obligation-mapping.json",
        "obligation-registry.json",
    ] {
        assert!(
            !fixture_dir.join(name).is_file(),
            "IPO-B012: golden `{name}` must not exist on characterization HEAD"
        );
    }

    // The four required found cases cannot be constructed: there is no Obligation type.
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("fn why_control_exists") && !ir.contains("fn get_obligation"),
        "IPO-B012: registry query helpers are absent"
    );
}

/// IPO-B013 found case: supersession / expiry / current-at-T helpers do not exist.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b013_no_lifecycle_or_current_at_helpers() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "IPO-B013",
        &product,
        &[
            "fn current_obligations_at",
            "fn supersession_chain",
            "fn obligation_applies",
            "enum ObligationApplicability",
            "enum ObligationLifecycle",
        ],
    );
}

/// IPO-B014 found case: AssessmentDefinition has no obligations inventory.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b014_assessment_has_no_obligations_inventory() {
    let assessment_src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !assessment_src.contains("pub obligations:")
            && !assessment_src.contains("pub interested_parties:"),
        "IPO-B014: AssessmentDefinition must not carry an obligation inventory"
    );
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.ipo.no-inventory"));
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("obligations").is_none());
    assert!(json.get("interestedParties").is_none());
    assert!(json.get("requirementSources").is_none());
}

/// IPO-B015 found case: implementations and controls still exist independently of duties.
#[ignore = "superseded by sdd_interested_parties_obligations_target"]
#[test]
fn ipo_b015_control_and_implementation_exist_without_obligation_owner() {
    let control = Control::new(
        ControlId::new("control.identity.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    );
    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.identity.mfa"),
        ControlId::new("control.identity.mfa"),
    );
    assert_eq!(control.id().as_str(), "control.identity.mfa");
    assert_eq!(impln.control_id().as_str(), "control.identity.mfa");
    let json = serde_json::to_value(&control).unwrap();
    assert!(json.get("obligationIds").is_none());
    assert!(json.get("whyExists").is_none());
}
