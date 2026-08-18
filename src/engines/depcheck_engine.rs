//! Dependency-confusion engine: parse manifests + optional registry checks.

use std::path::Path;
use std::sync::Arc;

use crate::depcheck::detect::detect_file_type;
use crate::depcheck::extract_packages;
use crate::depcheck::load_path;
use crate::depcheck::registry::{check_many, HttpRegistry, RegistryClient};
use crate::depcheck::types::{CheckStatus, FileKind, PackageRef, ScanOptions};
use crate::engines::EngineHit;

/// Live registry checks during `scan-code` / `scan-diff` are **opt-in**.
///
/// Set `WA_DEPCHECK_NETWORK=1` to enable. The dedicated `depcheck` subcommand
/// always performs registry checks and ignores this flag.
///
/// `WA_DEPCHECK_SKIP_NETWORK=1` always disables (wins over NETWORK).
pub fn network_disabled() -> bool {
    let skip = matches!(
        std::env::var("WA_DEPCHECK_SKIP_NETWORK")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );
    if skip {
        return true;
    }
    !matches!(
        std::env::var("WA_DEPCHECK_NETWORK")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Scan dependency manifests under `root` for packages missing on public registries.
///
/// Only paths present in `relative_files` (code-scan inventory) are checked.
pub fn scan_tree_for_confusion(root: &Path, relative_files: &[String]) -> Vec<EngineHit> {
    if network_disabled() {
        return Vec::new();
    }

    let scope: std::collections::HashSet<&str> =
        relative_files.iter().map(String::as_str).collect();
    let mut hits = Vec::new();
    for rel in relative_files {
        if !scope.contains(rel.as_str()) {
            continue;
        }
        let path = root.join(rel);
        let kind = detect_file_type(&path, None);
        if kind == FileKind::Unknown {
            continue;
        }
        hits.extend(scan_manifest_path(&path, rel));
    }
    hits
}

fn scan_manifest_path(path: &Path, rel: &str) -> Vec<EngineHit> {
    let input = match load_path(path, None) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let (packages, ecosystem) = match extract_packages(input.kind, &input.content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    if packages.is_empty() {
        return Vec::new();
    }

    let opts = ScanOptions {
        threads: 10,
        timeout_secs: 8,
        quiet: true,
        kind_override: Some(input.kind),
        secure_namespaces: Vec::new(),
        verbose: false,
    };
    let client: Arc<dyn RegistryClient> = match HttpRegistry::new(opts.timeout_secs) {
        Ok(c) => Arc::new(c),
        Err(_) => return Vec::new(),
    };

    let results = block_on(check_many(client, ecosystem, &packages, opts.threads));
    results
        .into_iter()
        .filter(|r| r.status == CheckStatus::Vulnerable)
        .map(|r| hit_for(rel, &r.name, &r.version, ecosystem.as_str()))
        .collect()
}

/// Offline helper used by tests: build hits from an already-known missing set.
pub fn hits_for_missing(
    rel: &str,
    ecosystem: &str,
    missing: &[(String, String)],
) -> Vec<EngineHit> {
    missing
        .iter()
        .map(|(name, ver)| hit_for(rel, name, ver, ecosystem))
        .collect()
}

fn hit_for(rel: &str, name: &str, version: &str, ecosystem: &str) -> EngineHit {
    EngineHit {
        rule_id: "depcheck.confusion.public-registry-missing".into(),
        anchor: format!("depcheck:{ecosystem}:{name}"),
        instance: Some(name.to_string()),
        title: format!("Dependency confusion: `{name}` missing on public {ecosystem} registry"),
        summary: format!(
            "Package `{name}`@{version} from `{rel}` was not found on the public {ecosystem} registry. \
             An attacker who registers this name could supply a higher version to consumers that resolve public-first."
        ),
        evidence: format!("registry_checked=true ecosystem={ecosystem} package={name} version={version}"),
        severity: "high",
        confidence: "high",
        confidence_rationale: "Public registry returned not-found for this exact package name.".into(),
        category: "supply-chain".into(),
        cwe: vec!["CWE-427".into()],
        remediation: format!(
            "Claim/reserve `{name}` on the public {ecosystem} registry, or configure installs to use a private registry \
             with public fallback disabled (e.g. npm scopes, pip --index-url only, Cargo [source] replace-with)."
        ),
        path: rel.replace('\\', "/"),
        start_line: 1,
        end_line: None,
        role: "manifest",
        snippet: format!("{name} = \"{version}\""),
        validation_json: None,
        attack_path_json: None,
    }
}

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(fut))
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("depcheck runtime")
            .block_on(fut)
    }
}

/// Parse-only (no network) — useful for unit tests.
pub fn parse_packages_offline(rel: &str, content: &str) -> Vec<PackageRef> {
    let kind = detect_file_type(Path::new(rel), Some(content));
    if kind == FileKind::Unknown {
        return Vec::new();
    }
    extract_packages(kind, content)
        .map(|(p, _)| p)
        .unwrap_or_default()
}
