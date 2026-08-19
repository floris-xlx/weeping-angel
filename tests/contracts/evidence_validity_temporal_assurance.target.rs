//! SUPERSEDED by `sdd_temporal_assurance_target`.
//!
//! Registration-only sibling harness. The SSOT is
//! `tests/contracts/temporal_assurance.target.rs` (TMP-001…012).
//! Tests are `#[ignore]` so this stub is not a second green bar.

use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
#[ignore = "superseded by target suite"]
fn dual_suite_target_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_evidence_validity_temporal_assurance_target")
            && toml.contains("tests/contracts/evidence_validity_temporal_assurance.target.rs"),
        "target suite must be listed in root Cargo.toml"
    );
    assert!(
        !manifest_dir()
            .join("tests/contracts/evidence_validity_temporal_assurance.baseline.rs")
            .exists(),
        "superseded baseline must be deleted"
    );
}
