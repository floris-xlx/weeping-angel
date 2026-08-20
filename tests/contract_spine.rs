//! Contract spine: fingerprints, finalize, ledger.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("apps/cli CARGO_MANIFEST_DIR")
        .to_path_buf()
}

use tempfile::tempdir;
use weeping_angel::contract::{
    CoverageDocument, FindingsDocument, ManifestDocument, derive_fingerprint, ensure_scan_layout,
    finalize_scan, finding_id_from_fingerprint, occurrence_id_from, paths,
    write_no_findings_bundle,
};

#[test]
fn example_fixture_fingerprint_identity() {
    let fp = derive_fingerprint(
        "target_sha256_example",
        "path-traversal.archive-extraction",
        "archive-entry-write-without-containment",
        None,
    )
    .unwrap();
    assert_eq!(
        fp,
        "codex-security/v1:sha256:990a4a6a2ec18440dd47eac4d7256c0ee2c02db1b43104720cab3cbe9db706ca"
    );
    assert_eq!(
        finding_id_from_fingerprint(&fp),
        "csf_852f90d6e1177502ff113d4a"
    );
    assert_eq!(
        occurrence_id_from("scan_example_001", &fp),
        "occ_e79cb19591e696572a1c22be"
    );
}

#[test]
fn finalize_example_fixture_writes_report() {
    let dir = tempdir().unwrap();
    let scan_dir = dir.path().join("scan");
    fs::create_dir_all(&scan_dir).unwrap();

    // Copy example completed-scan as draft (already sealed; re-seal is fine)
    let fixture = repo_root().join("tests/fixtures/completed-scan");
    for name in ["scan-manifest.json", "findings.json", "coverage.json"] {
        fs::copy(fixture.join(name), scan_dir.join(name)).unwrap();
    }

    let report = finalize_scan(&scan_dir, "0.1.2").unwrap();
    assert!(report.exists());
    let md = fs::read_to_string(&report).unwrap();
    assert!(md.contains("# Security Review:"));
    assert!(md.contains("Unsafe archive extraction"));

    let findings: FindingsDocument =
        serde_json::from_str(&fs::read_to_string(scan_dir.join(paths::FINDINGS_FILE)).unwrap())
            .unwrap();
    assert_eq!(findings.findings.len(), 1);
    assert_eq!(
        findings.findings[0].finding_id,
        "csf_852f90d6e1177502ff113d4a"
    );

    let manifest: ManifestDocument =
        serde_json::from_str(&fs::read_to_string(scan_dir.join(paths::MANIFEST_FILE)).unwrap())
            .unwrap();
    assert_eq!(manifest.scan.status, "completed");
    assert!(!manifest.scan.artifacts.is_empty());
}

#[test]
fn no_findings_bundle_seals() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

    let scan_dir = dir.path().join("out");
    ensure_scan_layout(&scan_dir).unwrap();
    let report = write_no_findings_bundle(
        &scan_dir,
        &root,
        "wa_test_001",
        "toy",
        vec!["src/".into()],
        "0.1.2",
    )
    .unwrap();
    assert!(report.exists());

    let coverage: CoverageDocument =
        serde_json::from_str(&fs::read_to_string(scan_dir.join(paths::COVERAGE_FILE)).unwrap())
            .unwrap();
    assert_eq!(coverage.completeness, "complete");
    assert!(
        coverage
            .surfaces
            .iter()
            .any(|s| s.disposition == "no_issue_found")
    );
}
