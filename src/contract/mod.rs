//! Codex Security–compatible scan contract spine.
//!
//! Algorithmic substrate for dual-domain (web + code) scans: artifact layout,
//! stable fingerprints, candidate ledgers, severity policy, and finalize/seal.

pub mod fingerprint;
pub mod ledger;
pub mod paths;
pub mod report_md;
pub mod sarif;
pub mod severity_policy;
pub mod types;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::contract::fingerprint::{populate_finding_identities, sha256_bytes, sha256_file};
use crate::contract::paths::{
    ARTIFACTS_DIR, CONTEXT_DIR, COVERAGE_FILE, COVERAGE_DIR, DISCOVERY_DIR, FINDINGS_DIR,
    FINDINGS_FILE, MANIFEST_FILE, RECONCILIATION_DIR, REPORT_MD,
};
use crate::contract::report_md::project_report_md;

pub use fingerprint::{
    derive_fingerprint, finding_id_from_fingerprint, occurrence_id_from, sha256_text, stable_id,
    FINGERPRINT_ALGORITHM,
};
pub use ledger::{
    combine_candidates, normalize_raw_candidate, write_candidate_ledger, Candidate,
    CandidateLocation,
};
pub use severity_policy::{
    apply_severity_matrix, Impact, Likelihood, PolicyDecision, SeverityLevel,
};
pub use types::*;

/// Create the numbered scan artifact tree under `scan_dir`.
pub fn ensure_scan_layout(scan_dir: &Path) -> Result<()> {
    for rel in [
        ARTIFACTS_DIR,
        CONTEXT_DIR,
        DISCOVERY_DIR,
        COVERAGE_DIR,
        RECONCILIATION_DIR,
        FINDINGS_DIR,
    ] {
        fs::create_dir_all(scan_dir.join(rel))
            .with_context(|| format!("create {}", scan_dir.join(rel).display()))?;
    }
    Ok(())
}

/// SHA-256 hex of a path for snapshot digests / target ids.
pub fn sha256_path_inventory(root: &Path) -> Result<String> {
    let mut entries: Vec<(String, String)> = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.starts_with(".git/") || rel.contains("/.git/") {
            continue;
        }
        let digest = sha256_file(path)?;
        entries.push((rel, digest));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (rel, digest) in &entries {
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        hasher.update(digest.as_bytes());
        hasher.update([0]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn snapshot_digest_v1(inventory_sha256: &str) -> String {
    format!("codex-security-snapshot/v1:sha256:{inventory_sha256}")
}

pub fn target_id_from_display(display: &str) -> String {
    format!("target_sha256_{}", &sha256_bytes(display.as_bytes())[..32])
}

/// Validate + seal a completed-scan bundle and project `report.md`.
///
/// Expects unsealed (or draft) `scan-manifest.json`, `findings.json`, and
/// `coverage.json` under `scan_dir`. Overwrites them with sealed identities
/// and digests, then writes unsealed `report.md`.
pub fn finalize_scan(scan_dir: &Path, producer_version: &str) -> Result<PathBuf> {
    if !scan_dir.is_dir() {
        bail!("scan-dir is not a directory: {}", scan_dir.display());
    }

    let manifest_path = scan_dir.join(MANIFEST_FILE);
    let findings_path = scan_dir.join(FINDINGS_FILE);
    let coverage_path = scan_dir.join(COVERAGE_FILE);

    let mut manifest: ManifestDocument = read_json(&manifest_path)?;
    let mut findings: FindingsDocument = read_json(&findings_path)?;
    let mut coverage: CoverageDocument = read_json(&coverage_path)?;

    // Producer identity for weeping-angel
    if manifest.scan.producer.name.is_empty() {
        manifest.scan.producer = Producer {
            name: "weeping-angel".into(),
            version: producer_version.into(),
        };
    }

    // Align scan ids
    let scan_id = manifest.scan.id.clone();
    findings.scan_id = scan_id.clone();
    coverage.scan_id = scan_id.clone();

    // Derive stable finding identities
    populate_finding_identities(&manifest, &mut findings)?;

    // Sort findings by severity desc then title
    findings.findings.sort_by(|a, b| {
        severity_rank(&b.severity.level)
            .cmp(&severity_rank(&a.severity.level))
            .then_with(|| a.title.cmp(&b.title))
    });

    // Timestamps
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    if manifest.scan.started_at.is_empty() {
        manifest.scan.started_at = now.clone();
    }
    manifest.scan.completed_at = now.clone();
    manifest.scan.sealed_at = now;
    manifest.scan.status = "completed".into();
    manifest.scan.coverage_ref = COVERAGE_FILE.into();
    manifest.scan.findings_ref = FINDINGS_FILE.into();

    // Write findings + coverage first so digests match sealed bytes
    write_json(&findings_path, &findings)?;
    write_json(&coverage_path, &coverage)?;

    let findings_sha = sha256_file(&findings_path)?;
    let coverage_sha = sha256_file(&coverage_path)?;

    // Temporarily write manifest without artifacts, then seal with digests
    // of the three canonical docs. Include findings + coverage; re-hash
    // after embedding artifact list of those two, then add self? Codex
    // seals findings + coverage only (example has no self hash of manifest).
    manifest.scan.artifacts = vec![
        ArtifactRecord {
            path: FINDINGS_FILE.into(),
            sha256: findings_sha,
            media_type: "application/json".into(),
        },
        ArtifactRecord {
            path: COVERAGE_FILE.into(),
            sha256: coverage_sha,
            media_type: "application/json".into(),
        },
    ];
    write_json(&manifest_path, &manifest)?;

    let report = project_report_md(&manifest, &findings, &coverage);
    let report_path = scan_dir.join(REPORT_MD);
    fs::write(&report_path, report)
        .with_context(|| format!("write {}", report_path.display()))?;

    Ok(report_path)
}

fn severity_rank(level: &str) -> u8 {
    match level {
        "critical" => 5,
        "high" => 4,
        "medium" => 3,
        "low" => 2,
        "informational" => 1,
        _ => 0,
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Write unsealed canonical JSON documents into `scan_dir` (caller finalizes).
pub fn write_scan_bundle(
    scan_dir: &Path,
    manifest: &ManifestDocument,
    findings: &FindingsDocument,
    coverage: &CoverageDocument,
) -> Result<()> {
    ensure_scan_layout(scan_dir)?;
    write_json(&scan_dir.join(MANIFEST_FILE), manifest)?;
    write_json(&scan_dir.join(FINDINGS_FILE), findings)?;
    write_json(&scan_dir.join(COVERAGE_FILE), coverage)?;
    Ok(())
}

/// Build a minimal sealed empty findings bundle (no issues) for a directory target.
pub fn write_no_findings_bundle(
    scan_dir: &Path,
    source_root: &Path,
    scan_id: &str,
    display_name: &str,
    include_paths: Vec<String>,
    producer_version: &str,
) -> Result<PathBuf> {
    ensure_scan_layout(scan_dir)?;
    let inventory = sha256_path_inventory(source_root).unwrap_or_else(|_| "0".repeat(64));
    let snap = snapshot_digest_v1(&inventory);

    let manifest = ManifestDocument {
        document_type: "codex-security.scan-manifest".into(),
        schema_version: "1.0".into(),
        scan: ScanBody {
            id: scan_id.into(),
            producer: Producer {
                name: "weeping-angel".into(),
                version: producer_version.into(),
            },
            status: "completed".into(),
            started_at: String::new(),
            completed_at: String::new(),
            sealed_at: String::new(),
            target: ScanTarget {
                kind: "directory_snapshot".into(),
                target_id: target_id_from_display(display_name),
                display_name: display_name.into(),
                remote: None,
                revision: None,
                base_revision: None,
                head_revision: None,
                snapshot_digest: Some(snap),
            },
            scope: ScanScope {
                include_paths: include_paths.clone(),
                exclude_paths: vec![],
                summary: Some("Algorithmic weeping-angel scan (no findings).".into()),
                artifacts_reviewed: None,
                runtime_status: None,
                validation_mode: Some("static".into()),
                context: None,
                limitations: Some(vec![
                    "Autonomous rule engines only; not full semantic AI review.".into(),
                ]),
            },
            threat_model: None,
            hardening: None,
            coverage_ref: COVERAGE_FILE.into(),
            findings_ref: FINDINGS_FILE.into(),
            artifacts: vec![],
        },
    };

    let findings = FindingsDocument {
        document_type: "codex-security.findings".into(),
        schema_version: "1.0".into(),
        scan_id: scan_id.into(),
        findings: vec![],
    };

    let coverage = CoverageDocument {
        document_type: "codex-security.coverage".into(),
        schema_version: "1.0".into(),
        scan_id: scan_id.into(),
        mode: "repository".into(),
        completeness: "complete".into(),
        inventory_strategy: "directory".into(),
        include_paths,
        exclude_paths: vec![],
        surfaces: vec![CoverageSurface {
            id: "surface_algorithmic_engines".into(),
            label: "Algorithmic engines".into(),
            disposition: "no_issue_found".into(),
            receipt_refs: vec![],
            risk_area: None,
            notes: Some("No reportable candidates after rule pass.".into()),
        }],
        explicit_exclusions: vec![],
        deferred: vec![],
        open_questions: vec![],
    };

    write_scan_bundle(scan_dir, &manifest, &findings, &coverage)?;
    finalize_scan(scan_dir, producer_version)
}
