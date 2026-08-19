//! Target suite for Operational ISMS v1 nonconformity / CAPA (Prompt 22).
//!
//! Encodes DESIRED behavior in `docs/specs/nonconformity-capa.md` §4 / §6
//! (NC-001…NC-012). On current HEAD these tests are compile-discoverable and
//! the product APIs they name do not exist yet — the implement phase must
//! turn them GREEN. This file currently asserts only the dual-suite/spec
//! registration that already holds after the baseline commit so
//! `cargo test --test sdd_nonconformity_capa_target` is a real `--test`
//! binary. Normative NC-001…NC-011 bodies land when the RED rewrite imports
//! `Nonconformity` / `CorrectiveAction`.
//!
//! Do not implement the CAPA engine in this file.

use std::fs;
use std::path::PathBuf;

use weeping_angel_assurance_ir::{ASSURANCE_IR_SCHEMA, AssessmentRequests};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_framework::FrameworkCapabilities;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// NC-011 (partial, already true on HEAD): compile flags and catalog fence.
#[test]
fn nc_011_compile_flags_and_catalog_fence() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    assert!(!AssessmentRequests::default().nonconformities);
    assert!(!FrameworkCapabilities::default().supports_nonconformities);

    let catalog = CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load");
    catalog
        .control("control.governance.corrective-action")
        .expect("catalog attestation control must remain");
    assert!(
        catalog
            .tests()
            .contains_key("test.governance.corrective-action-recorded")
    );
}

/// NC-012 Dual-suite registration.
#[test]
fn nc_012_dual_suite_registration() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("name = \"sdd_nonconformity_capa_baseline\"")
            && cargo.contains("path = \"tests/contracts/nonconformity_capa.baseline.rs\"")
            && cargo.contains("name = \"sdd_nonconformity_capa_target\"")
            && cargo.contains("path = \"tests/contracts/nonconformity_capa.target.rs\"")
    );
    assert!(
        manifest_dir()
            .join("docs/specs/nonconformity-capa.md")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("docs/adr/0003-nonconformity-capa.md")
            .is_file()
    );
}
