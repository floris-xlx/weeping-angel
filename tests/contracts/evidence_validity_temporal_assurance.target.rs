//! SUPERSEDED by `sdd_temporal_assurance_target`.
//!
//! Registration-only sibling harness. The SSOT is
//! `tests/contracts/temporal_assurance.target.rs` (TMP-001…012).
//! Tests are `#[ignore]` so this stub is not a second green bar.

use std::fs;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

#[test]
#[ignore = "superseded by target suite"]
fn dual_suite_target_is_registered() {
    let _toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        harness_src().contains("evidence_validity_temporal_assurance.target.rs")
            && harness_src().contains("evidence_validity_temporal_assurance.target.rs"),
        "target suite must be wired as a harness module"
    );
    assert!(
        !manifest_dir()
            .join("tests/contracts/evidence_validity_temporal_assurance.baseline.rs")
            .exists(),
        "superseded baseline must be deleted"
    );
}
