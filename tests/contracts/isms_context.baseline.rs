//! SUPERSEDED by `sdd_isms_context_target`.
//!
//! Historical characterization of SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (`docs/specs/isms-context.md` §3):
//! no `IsmsContext` graph, assessment inventories only, no context fixture.
//! The operational ISMS context IR is now the SSOT in the target suite.
//! Characterization tests are `#[ignore = "superseded by target suite"]` so CI
//! does not require the pre-implement absences. Dual-suite registration remains.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssetKind, CanonicalizationVersion,
    IdError, IdentityKind, MAX_ID_LEN, SubjectKind, ValidateIr, canonical_digest,
    typed_canonical_digest,
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

/// CTX-B01: product crate sources have no ISMS context IR types or ids.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b01_product_crates_have_no_isms_context_types() {
    let product = product_crate_sources_joined();
    for needle in [
        "struct IsmsContext",
        "IsmsContextId",
        "struct InterestedParty",
        "struct Obligation",
        "struct SecurityObjective",
        "struct ManagementSystemScope",
        "struct ContextIssue",
        "struct GovernanceCadence",
        "struct BusinessUnit",
        "RiskMethodologyId",
        "IsmsLifecycleStatus",
    ] {
        assert!(
            !product.contains(needle),
            "found-case: product crates must not contain `{needle}`"
        );
    }
    assert!(
        !product.contains("pub struct RiskMethodology")
            && !product.contains("struct RiskMethodology "),
        "found-case: RiskMethodology scoring type is absent in product crates"
    );

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !lib.contains("pub mod isms") && !lib.contains("IsmsContext"),
        "lib.rs currently does not re-export an ISMS context module"
    );
}

/// CTX-B02: `AssessmentDefinition::new` succeeds with schema v1 and no context pointer.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b02_assessment_definition_new_is_inventories_only() {
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.baseline"));
    assert_eq!(assessment.schema_version, "assurance-ir/v1");
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    assert!(assessment.requirements.is_empty());
    assert!(assessment.controls.is_empty());
    assert!(assessment.mappings.is_empty());
    assert!(assessment.evidence_requirements.is_empty());
    assert!(assessment.tests.is_empty());
    assert!(assessment.implementations.is_empty());
    assert!(assessment.scope.organizations.is_empty());
    assert!(assessment.scope.subjects.is_empty());
    assert!(assessment.scope.exclusions.is_empty());
    assert!(assessment.assets.is_empty());
    assert!(assessment.identities.is_empty());
    assert!(assessment.vendors.is_empty());
    assert!(assessment.risks.is_empty());
    assert!(assessment.exceptions.is_empty());
    assert!(assessment.processing_activities.is_empty());
    assessment
        .validate()
        .expect("empty AssessmentDefinition::new must validate today");

    let json = serde_json::to_value(&assessment).unwrap();
    let obj = json
        .as_object()
        .expect("AssessmentDefinition serializes as an object");
    assert!(
        obj.contains_key("schema_version"),
        "assessment document uses snake_case schema_version"
    );
    assert!(
        obj.get("isms_context_id").is_none() && obj.get("ismsContextId").is_none(),
        "found-case: AssessmentDefinition JSON has no isms_context_id pointer"
    );

    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !src.contains("isms_context_id"),
        "AssessmentDefinition currently has no isms_context_id field"
    );
}

/// CTX-B03: golden assessment.json decodes and validates.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b03_golden_assessment_json_decodes() {
    let path = ir_fixture("assessment.json");
    assert!(
        path.is_file(),
        "golden fixture must exist: {}",
        path.display()
    );
    let raw = fs::read_to_string(&path).unwrap();
    let assessment: AssessmentDefinition = serde_json::from_str(&raw).unwrap();
    assessment.validate().unwrap();
    assert_eq!(assessment.schema_version, ASSURANCE_IR_SCHEMA);
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert!(
        parsed.get("isms_context_id").is_none(),
        "golden assessment.json has no isms_context_id key"
    );
}

/// CTX-B04: schema constant is assurance-ir/v1 (do not fork).
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b04_assurance_ir_schema_is_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

/// CTX-B05: digest and typed-id reuse surface already exists.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b05_digest_and_typed_id_surface_exists() {
    assert_eq!(CanonicalizationVersion::CURRENT.as_str(), "canon/v1");
    assert_eq!(MAX_ID_LEN, 256);

    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.digest"));
    let digest = canonical_digest(&assessment).expect("canonical_digest must serialize");
    assert_eq!(digest.len(), 64);
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    let typed = typed_canonical_digest("AssessmentDefinition", &assessment)
        .expect("typed_canonical_digest must serialize");
    assert_eq!(typed.len(), 64);
    assert_ne!(digest, typed);

    assert_eq!(AssessmentId::try_new(""), Err(IdError::Empty));
    assert_eq!(AssessmentId::try_new("   "), Err(IdError::Empty));
}

/// CTX-B06: organization-shaped strings are not ISMS identity.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b06_organization_is_not_isms_identity() {
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.isms-context.scope"));
    let orgs: &Vec<String> = &assessment.scope.organizations;
    assert!(orgs.is_empty());

    let _asset_org = AssetKind::Organization;
    let _subject_org = SubjectKind::Organization;

    match IdentityKind::User {
        IdentityKind::User
        | IdentityKind::Service
        | IdentityKind::ServiceAccount
        | IdentityKind::Team
        | IdentityKind::Role
        | IdentityKind::Other => {}
    }

    let identity_src = read_repo_file("crates/weeping-angel-assurance-ir/src/identity.rs");
    assert!(
        identity_src.contains("Minimal principal identity"),
        "Identity remains an IAM principal, not a legal entity"
    );
    assert!(
        !identity_src.contains("Organization"),
        "IdentityKind currently has no Organization variant"
    );
}

/// CTX-B07: representative ISMS context fixture is absent.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b07_no_isms_context_fixture() {
    let path = ir_fixture("isms-context.json");
    assert!(
        !path.exists(),
        "found-case: tests/fixtures/assurance-ir/v1/isms-context.json must not exist"
    );
}

/// CTX-B08: framework crate Cargo.toml stays network-free.
#[test]
#[ignore = "superseded by target suite"]
fn ctx_b08_framework_crate_is_network_free() {
    let toml = read_repo_file("crates/weeping-angel-framework/Cargo.toml");
    for forbidden in ["reqwest", "octocrab", "aws-sdk", "cloudflare"] {
        assert!(
            !toml.contains(forbidden),
            "weeping-angel-framework Cargo.toml must not mention `{forbidden}`"
        );
    }
    assert!(
        toml.contains("weeping-angel-assurance-ir"),
        "framework crate currently depends on weeping-angel-assurance-ir only for IR"
    );
}
