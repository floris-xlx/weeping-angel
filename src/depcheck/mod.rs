//! DepCheck — universal dependency-confusion **scanner** (detection only).
//!
//! Parses many package-manager manifests, filters non-registry deps, and checks
//! whether each package name exists on the public registry. Does **not** publish
//! packages, generate install-hook payloads, or auto-exploit.

pub mod convert;
pub mod depsdev;
pub mod detect;
pub mod discover;
pub mod email_check;
pub mod filter;
pub mod hardening;
pub mod inspect;
pub mod parsers;
pub mod registry;
pub mod remote_hunt;
pub mod report;
pub mod types;
pub mod web;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Context, Result};

use convert::packages_to_map;
use detect::detect_file_type;
use filter::{filter_packages, partition_secure_namespaces};
use parsers::parse_manifest;
use registry::{check_many, HttpRegistry, RegistryClient};
use report::partition_results;
use types::{
    CheckStatus, Ecosystem, FileKind, ManifestInput, PackageRef, ScanOptions, ScanSummary,
};

/// Parse + filter a single manifest (no network).
pub fn extract_packages(
    kind: FileKind,
    content: &str,
) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let (raw, eco) = parse_manifest(kind, content)?;
    let filtered = filter_packages(eco, raw);
    Ok((filtered, eco))
}

/// Load a local file into a [`ManifestInput`].
pub fn load_path(path: &Path, kind_override: Option<FileKind>) -> Result<ManifestInput> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let kind = kind_override.unwrap_or_else(|| detect_file_type(path, Some(&content)));
    if kind == FileKind::Unknown {
        bail!(
            "could not detect dependency file type for {} (use --type)",
            path.display()
        );
    }
    Ok(ManifestInput {
        display: path.display().to_string(),
        path: Some(path.to_path_buf()),
        content,
        kind,
    })
}

/// Build a manifest input from fetched URL body.
pub fn load_url_body(
    url: &str,
    body: String,
    kind_override: Option<FileKind>,
) -> Result<ManifestInput> {
    let kind = kind_override.unwrap_or_else(|| {
        let path_hint = url::Url::parse(url)
            .ok()
            .and_then(|u| {
                PathBuf::from(u.path())
                    .file_name()
                    .map(|s| PathBuf::from(s))
            })
            .unwrap_or_else(|| PathBuf::from("remote.json"));
        detect_file_type(&path_hint, Some(&body))
    });
    if kind == FileKind::Unknown {
        bail!("could not detect dependency file type from URL body (use --type)");
    }
    Ok(ManifestInput {
        display: url.to_string(),
        path: None,
        content: body,
        kind,
    })
}

/// Scan one manifest with the given registry client.
pub async fn scan_manifest(
    input: &ManifestInput,
    opts: &ScanOptions,
    client: Arc<dyn RegistryClient>,
) -> Result<ScanSummary> {
    let started = Instant::now();
    let (packages, ecosystem) = extract_packages(input.kind, &input.content)?;
    let all_packages = packages_to_map(&packages);

    // confused `-s`: skip registry checks for claimed scopes / namespaces
    let (to_check, known_secure) =
        partition_secure_namespaces(packages, &opts.secure_namespaces);

    if opts.verbose && !opts.quiet {
        for p in &known_secure {
            eprintln!(
                "  [*] skipping known-secure namespace match: {}",
                p.name
            );
        }
        eprintln!(
            "  [*] checking {} package(s) against {} …",
            to_check.len(),
            ecosystem
        );
    }

    let results = check_many(client, ecosystem, &to_check, opts.threads).await;
    let (vulnerable, mut safe, errors) = partition_results(results);

    // Known-secure packages count as safe for reporting totals
    let mut suppressed = Vec::new();
    for p in known_secure {
        suppressed.push(types::PackageResult {
            name: p.name.clone(),
            version: p.version.clone(),
            status: CheckStatus::Safe,
            detail: Some("secure-namespace".into()),
        });
        safe.push(types::PackageResult {
            name: p.name,
            version: p.version,
            status: CheckStatus::Safe,
            detail: Some("secure-namespace".into()),
        });
    }
    safe.sort_by(|a, b| a.name.cmp(&b.name));
    suppressed.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ScanSummary {
        file: input.display.clone(),
        file_kind: input.kind,
        ecosystem,
        packages: all_packages,
        vulnerable,
        safe,
        errors,
        suppressed,
        introductions: Vec::new(),
        hardening: None,
        duration_secs: started.elapsed().as_secs_f64(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Convenience: HTTP registry + scan one path.
pub async fn scan_path(path: &Path, opts: ScanOptions) -> Result<ScanSummary> {
    let input = load_path(path, opts.kind_override)?;
    let client: Arc<dyn RegistryClient> = Arc::new(HttpRegistry::new(opts.timeout_secs)?);
    scan_manifest(&input, &opts, client).await
}

/// List packages only (no registry).
pub fn list_only(input: &ManifestInput) -> Result<(BTreeMap<String, String>, Ecosystem, FileKind)> {
    let (packages, eco) = extract_packages(input.kind, &input.content)?;
    Ok((packages_to_map(&packages), eco, input.kind))
}

/// Exit code helper: 1 if any vulnerable, else 0.
pub fn exit_code(summary: &ScanSummary) -> i32 {
    if summary.vulnerable.is_empty() {
        0
    } else {
        1
    }
}

/// Collect EngineHit-ready vulnerable package names from a summary.
pub fn vulnerable_names(summary: &ScanSummary) -> Vec<(String, String)> {
    summary
        .vulnerable
        .iter()
        .filter(|p| p.status == CheckStatus::Vulnerable)
        .map(|p| (p.name.clone(), p.version.clone()))
        .collect()
}

/// Resolve which paths to scan for a CLI target (file or directory).
pub fn resolve_targets(target: &Path) -> Result<Vec<PathBuf>> {
    if target.is_dir() {
        let files = discover::find_dep_files(target);
        if files.is_empty() {
            bail!("no dependency files found under {}", target.display());
        }
        Ok(files)
    } else if target.is_file() {
        Ok(vec![target.to_path_buf()])
    } else {
        bail!("path not found: {}", target.display());
    }
}
