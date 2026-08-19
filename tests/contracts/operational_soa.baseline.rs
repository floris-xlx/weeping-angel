//! Baseline characterization of live Statement-of-Applicability projection
//! (`docs/specs/operational-soa.md` §3 / §6.1).
//!
//! SUPERSEDED by `sdd_operational_soa_target`.
//!
//! Historical characterization: `project_soa` rereads pack `applicability.toml`,
//! three-state `from_pack` (including Unresolved), hardcoded
//! `implementation_state = "assessed"`, empty evidence/exceptions, snapshot
//! clone not crate-root exported, CLI catch-all, no SoA cause taxonomy.
//! All tests are `#[ignore = "superseded by sdd_operational_soa_target"]` so CI
//! does not require the pre-operational-graph shortcut. Dual-suite registration
//! remains.
//!
//! Do **not** assert the older remap-baseline lie that Unresolved is absent
//! or that `applicability.toml` is only `applicable = true`.

use std::fs;
use std::path::{Path, PathBuf};

use weeping_angel_assurance::lineage::StatementOfApplicabilitySnapshot;
use weeping_angel_assurance::soa::{Applicability, project_soa_from_snapshot};
use weeping_angel_assurance::{StatementOfApplicability, project_soa};

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
    manifest_dir().join("crates").join(name).join("src")
}

fn fn_project_soa(src: &str) -> &str {
    let start = src
        .find("pub fn project_soa(")
        .expect("soa.rs must expose project_soa");
    &src[start..]
}

fn live_iso_soa() -> StatementOfApplicability {
    project_soa("iso-27001", "2022")
}

fn entry<'a>(
    soa: &'a StatementOfApplicability,
    reference: &str,
) -> &'a weeping_angel_assurance::soa::SoaEntry {
    soa.entries
        .iter()
        .find(|e| e.reference == reference)
        .unwrap_or_else(|| panic!("expected SoA row {reference}"))
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b01_dual_suite_and_spec_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("name = \"sdd_operational_soa_baseline\""),
        "SOA-B01: register sdd_operational_soa_baseline in root Cargo.toml"
    );
    assert!(
        cargo.contains("path = \"tests/contracts/operational_soa.baseline.rs\""),
        "SOA-B01: baseline path must be tests/contracts/operational_soa.baseline.rs"
    );
    assert!(
        cargo.contains("name = \"sdd_operational_soa_target\""),
        "SOA-B01: register sdd_operational_soa_target in root Cargo.toml"
    );
    assert!(
        cargo.contains("path = \"tests/contracts/operational_soa.target.rs\""),
        "SOA-B01: target path must be tests/contracts/operational_soa.target.rs"
    );
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/operational-soa.md"),
        "SOA-B01: CANONICAL_SPECS must include docs/specs/operational-soa.md"
    );
    assert!(
        manifest_dir()
            .join("docs/specs/operational-soa.md")
            .is_file(),
        "SOA-B01: human spec must exist"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b02_project_soa_rereads_live_pack_toml() {
    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    let body = fn_project_soa(&soa_src);
    assert!(
        body.contains("resolve_pack_dir"),
        "SOA-B02: project_soa currently rereads via resolve_pack_dir"
    );
    assert!(
        body.contains("applicability.toml"),
        "SOA-B02: project_soa currently reads live applicability.toml"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b03_from_pack_is_three_state_including_unresolved() {
    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa_src.contains("fn from_pack"),
        "SOA-B03: Applicability::from_pack must exist"
    );
    assert!(
        soa_src.contains("Unresolved") && soa_src.contains("NotApplicable"),
        "SOA-B03: three-state enum including Unresolved (not the old boolean-only lie)"
    );
    assert!(
        soa_src.contains("manualdeterminationrequired")
            || soa_src.contains("ManualDeterminationRequired"),
        "SOA-B03: from_pack maps manual determination to Unresolved"
    );
    assert_eq!(Applicability::Applicable.as_str(), "applicable");
    assert_eq!(Applicability::NotApplicable.as_str(), "notApplicable");
    assert_eq!(Applicability::Unresolved.as_str(), "unresolved");
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b04_live_iso_three_state_pack_defaults() {
    let soa = live_iso_soa();
    assert!(
        !soa.entries.is_empty(),
        "SOA-B04: live ISO SoA must have pack rows"
    );
    assert_eq!(
        entry(&soa, "A.5.19").applicability,
        Applicability::NotApplicable,
        "SOA-B04: pack A.5.19 is not-applicable"
    );
    assert_eq!(
        entry(&soa, "A.8.13").applicability,
        Applicability::Unresolved,
        "SOA-B04: pack A.8.13 is unresolved — Unresolved is representable"
    );
    assert_eq!(
        entry(&soa, "A.5.1").applicability,
        Applicability::Applicable,
        "SOA-B04: pack A.5.1 is applicable"
    );
    let pack = read_repo_file("frameworks/iso-27001/2022/applicability.toml");
    assert!(
        pack.contains("not-applicable") && pack.contains("unresolved"),
        "SOA-B04: applicability.toml is three-state, not only applicable=true"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b05_implementation_hardcoded_assessed_empty_evidence() {
    let soa = live_iso_soa();
    for row in &soa.entries {
        assert_eq!(
            row.implementation_state, "assessed",
            "SOA-B05: implementation_state is hardcoded assessed (row={})",
            row.reference
        );
        assert!(
            row.automated_effectiveness.is_none(),
            "SOA-B05: automated_effectiveness is None (row={})",
            row.reference
        );
        assert!(
            row.evidence.is_empty(),
            "SOA-B05: evidence empty (row={})",
            row.reference
        );
        assert!(
            row.exceptions.is_empty(),
            "SOA-B05: exceptions empty (row={})",
            row.reference
        );
    }
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b06_mapped_controls_come_from_pack_mappings() {
    let soa = live_iso_soa();
    let mapped = soa.entries.iter().any(|e| !e.mapped_controls.is_empty());
    assert!(
        mapped,
        "SOA-B06: at least one live SoA row must list pack mapped_controls"
    );
    let a85 = entry(&soa, "A.8.5");
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.starts_with("control.identity.")),
        "SOA-B06: A.8.5 mapped_controls come from catalog-targeted pack mappings, got {:?}",
        a85.mapped_controls
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b07_disclaimer_is_readiness_not_certification() {
    let soa = live_iso_soa();
    let d = soa.disclaimer.to_ascii_lowercase();
    assert!(
        d.contains("readiness") && d.contains("not certification"),
        "SOA-B07: disclaimer must be a readiness aid, not certification, got {}",
        soa.disclaimer
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b08_snapshot_clone_exists_but_is_not_crate_root_exported() {
    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa_src.contains("pub fn project_soa_from_snapshot"),
        "SOA-B08: project_soa_from_snapshot exists in soa.rs"
    );
    let live = live_iso_soa();
    let snap = StatementOfApplicabilitySnapshot {
        schema: "weeping-angel/assessment-lineage/v1".into(),
        digest: "caller-supplied".into(),
        framework_pack_digest: "unpinned".into(),
        soa: live.clone(),
    };
    let restored = project_soa_from_snapshot(&snap);
    assert_eq!(
        restored.entries.len(),
        live.entries.len(),
        "SOA-B08: from_snapshot clones snapshot.soa"
    );
    assert_eq!(restored.disclaimer, live.disclaimer);
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let export_line = lib
        .lines()
        .find(|l| l.contains("pub use soa::"))
        .unwrap_or("");
    assert!(
        !export_line.contains("project_soa_from_snapshot"),
        "SOA-B08: project_soa_from_snapshot is not crate-root re-exported today ({export_line})"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b09_statement_of_applicability_snapshot_shape() {
    let mut files = Vec::new();
    walk_rs_files(&crate_src("weeping-angel-assurance"), &mut files);
    let lineage = files
        .iter()
        .find(|p| p.ends_with("lineage.rs"))
        .and_then(|p| fs::read_to_string(p).ok())
        .expect("lineage.rs");
    assert!(
        lineage.contains("struct StatementOfApplicabilitySnapshot"),
        "SOA-B09: StatementOfApplicabilitySnapshot exists"
    );
    assert!(
        lineage.contains("framework_pack_digest") && lineage.contains("pub soa:"),
        "SOA-B09: snapshot fields include schema/digest/framework_pack_digest/soa"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b10_cli_soa_is_still_banner_and_exit_zero() {
    let main_src = read_repo_file("src/main.rs");
    assert!(
        !main_src.contains("AssuranceCommand::Soa"),
        "SOA-B10: Soa is not dispatched like Catalog/Explain"
    );
    assert!(
        main_src.contains("This is a readiness assessment and is not certification."),
        "SOA-B10: catch-all still prints the non-certification banner"
    );
    let cli = read_repo_file("src/cli.rs");
    assert!(
        cli.contains("Soa(AssuranceSoaArgs)"),
        "SOA-B10: parser still exposes assurance soa"
    );
    assert!(
        !manifest_dir().join("src/assurance_soa.rs").is_file(),
        "SOA-B10: no assurance_soa.rs sibling yet"
    );
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b11_snapshot_diff_has_no_soa_cause_taxonomy() {
    let snap = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    for needle in [
        "ApplicabilityChange",
        "ImplementationChange",
        "EffectivenessRegression",
        "ExceptionExpiry",
        "MappingChange",
        "TreatmentChange",
        "SoaDiffCause",
    ] {
        assert!(
            !snap.contains(needle),
            "SOA-B11: SnapshotDiff must not yet name SoA cause {needle}"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_operational_soa_target"]
fn soa_b12_missing_implementation_is_not_first_class() {
    let soa = live_iso_soa();
    let a51 = entry(&soa, "A.5.1");
    assert_eq!(a51.applicability, Applicability::Applicable);
    assert_ne!(
        a51.implementation_state
            .to_ascii_lowercase()
            .replace('_', ""),
        "notimplemented",
        "SOA-B12: live path does not yet emit first-class notImplemented"
    );
    assert_eq!(a51.implementation_state, "assessed");
    let json = serde_json::to_value(&soa).expect("serialize");
    let blob = json.to_string().to_ascii_lowercase();
    assert!(
        !blob.contains("\"implementationstatus\":\"notimplemented\"")
            && !blob.contains("\"implementationstate\":\"notimplemented\""),
        "SOA-B12: no live row uses notImplemented as implementation status"
    );
}
