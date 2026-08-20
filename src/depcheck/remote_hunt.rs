//! Remote dependency-manifest hunting (DepenFusion-style, multi-ecosystem).
//!
//! Reads hosts/URLs, probes common dependency file paths, extracts packages,
//! and checks public registries for free namespaces.
//!
//! Does **not** publish packages or exploit targets.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use super::detect::detect_from_content;
use super::extract_packages;
use super::registry::{RegistryClient, check_many};
use super::types::{CheckStatus, Ecosystem, FileKind, PackageRef, ScanOptions};

const UA: &str = concat!("weeping-angel-depcheck/", env!("CARGO_PKG_VERSION"));

/// Options for bulk remote hunting (DepenFusion-compatible knobs).
#[derive(Debug, Clone)]
pub struct HuntOptions {
    pub threads: usize,
    pub timeout_secs: u64,
    /// Appended to each probe URL (e.g. `?token=foo`).
    pub append: String,
    /// If true, drop the path from the input URL (probe host root only).
    pub strip_path: bool,
    pub verbose: u8,
    pub silent: bool,
    /// Print full public-registry links for missing packages.
    pub show_link: bool,
    /// Extra relative paths to probe (defaults to guide’s common list).
    pub extra_paths: Vec<String>,
}

impl Default for HuntOptions {
    fn default() -> Self {
        Self {
            threads: 10,
            timeout_secs: 15,
            append: String::new(),
            strip_path: false,
            verbose: 0,
            silent: false,
            show_link: false,
            extra_paths: Vec::new(),
        }
    }
}

/// Common dependency / config paths from dependency-confusion recon guides.
pub const DEFAULT_PROBE_PATHS: &[&str] = &[
    "package.json",
    "package-lock.json",
    ".package-lock.json",
    "npm-shrinkwrap.json",
    "node_modules/.package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "requirements.txt",
    "Pipfile",
    "Pipfile.lock",
    "pyproject.toml",
    "composer.json",
    "composer.lock",
    "Gemfile",
    "Gemfile.lock",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "go.mod",
    "Cargo.toml",
    "Cargo.lock",
    "packages.config",
];

#[derive(Debug, Clone)]
struct TargetBase {
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntHit {
    pub package: String,
    pub version: String,
    pub ecosystem: String,
    pub registry_url: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntReport {
    pub probed_urls: usize,
    pub manifests_found: usize,
    pub packages_checked: usize,
    pub vulnerable: Vec<HuntHit>,
    pub duration_secs: f64,
}

/// Parse a host/URL line into a normalized base URL for probing.
fn parse_target_line(line: &str, strip_path: bool) -> Option<TargetBase> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    static RE: OnceLock<Regex> = OnceLock::new();
    let re =
        RE.get_or_init(|| Regex::new(r"^(https?://)?([^/\s]+)(/.*)?$").expect("url line regex"));
    let caps = re.captures(line)?;
    let protocol = caps
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "https://".into());
    let domain = caps.get(2)?.as_str();
    let path = if strip_path {
        "/".to_string()
    } else {
        caps.get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "/".into())
    };
    let mut base = format!("{protocol}{domain}{path}");
    if !base.ends_with('/') {
        base.push('/');
    }
    Some(TargetBase { base_url: base })
}

fn append_suffix(append: &str) -> String {
    let a = append.trim();
    if a.is_empty() {
        String::new()
    } else if a.starts_with('?') || a.starts_with('&') {
        a.to_string()
    } else {
        format!("?{a}")
    }
}

fn probe_urls(base: &TargetBase, append: &str, extra: &[String]) -> Vec<String> {
    let suffix = append_suffix(append);
    let mut paths: Vec<String> = DEFAULT_PROBE_PATHS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    for p in extra {
        let p = p.trim_start_matches('/');
        if !p.is_empty() && !paths.iter().any(|x| x == p) {
            paths.push(p.to_string());
        }
    }
    paths
        .into_iter()
        .map(|rel| format!("{}{}{}", base.base_url, rel, suffix))
        .collect()
}

fn registry_url_for(eco: Ecosystem, name: &str) -> String {
    match eco {
        Ecosystem::Npm => format!("https://registry.npmjs.org/{}", name.replace('/', "%2f")),
        Ecosystem::Pip => format!("https://pypi.org/project/{name}/"),
        Ecosystem::Composer => format!("https://packagist.org/packages/{name}"),
        Ecosystem::Rubygems => format!("https://rubygems.org/gems/{name}"),
        Ecosystem::Cargo => format!("https://crates.io/crates/{name}"),
        Ecosystem::Go => format!("https://pkg.go.dev/{name}"),
        Ecosystem::Maven => format!("https://search.maven.org/search?q={name}"),
        Ecosystem::Nuget => format!("https://www.nuget.org/packages/{name}"),
    }
}

/// Run remote dependency-manifest hunt over host/URL lines (multi-ecosystem).
pub async fn hunt_remote_npm(
    lines: &[String],
    hunt: &HuntOptions,
    scan: &ScanOptions,
    registry: Arc<dyn RegistryClient>,
) -> Result<HuntReport> {
    hunt_remote(lines, hunt, scan, registry).await
}

/// Multi-ecosystem remote hunt (preferred name).
pub async fn hunt_remote(
    lines: &[String],
    hunt: &HuntOptions,
    scan: &ScanOptions,
    registry: Arc<dyn RegistryClient>,
) -> Result<HuntReport> {
    let started = std::time::Instant::now();
    let client = Client::builder()
        .timeout(Duration::from_secs(hunt.timeout_secs.max(3)))
        .user_agent(UA)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let mut bases = Vec::new();
    for line in lines {
        if let Some(t) = parse_target_line(line, hunt.strip_path) {
            bases.push(t);
        } else if hunt.verbose >= 1 && !hunt.silent {
            eprintln!("[!] invalid url: {line}");
        }
    }
    if bases.is_empty() {
        bail!("no valid hosts/URLs to probe");
    }

    let mut urls = Vec::new();
    let mut url_owner: HashMap<String, String> = HashMap::new();
    for b in &bases {
        for u in probe_urls(b, &hunt.append, &hunt.extra_paths) {
            url_owner.insert(u.clone(), b.base_url.clone());
            urls.push(u);
        }
    }

    if !hunt.silent {
        eprintln!(
            "[*] Remote dependency hunt: {} hosts → {} probe URLs ({} path templates, threads={})",
            bases.len(),
            urls.len(),
            DEFAULT_PROBE_PATHS.len() + hunt.extra_paths.len(),
            hunt.threads
        );
    }

    let probed = urls.len();
    let sem = Arc::new(Semaphore::new(hunt.threads.max(1)));
    let mut futs = FuturesUnordered::new();
    for url in urls {
        let client = client.clone();
        let sem = Arc::clone(&sem);
        futs.push(async move {
            let _p = sem.acquire().await.expect("sem");
            let resp = client.get(&url).send().await;
            (url, resp)
        });
    }

    // (ecosystem, name) → (version, sources)
    let mut pkg_sources: BTreeMap<(Ecosystem, String), (String, BTreeSet<String>)> =
        BTreeMap::new();
    let mut manifests_found = 0usize;

    while let Some((url, resp)) = futs.next().await {
        let Ok(resp) = resp else {
            if hunt.verbose >= 2 && !hunt.silent {
                eprintln!("  miss/err {url}");
            }
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(body) = resp.text().await else {
            continue;
        };
        let mut kind = detect_from_content(&body);
        // Filename hint from URL path
        if kind == FileKind::Unknown
            && let Some(name) = url.split('?').next().and_then(|u| u.rsplit('/').next())
        {
            kind = super::detect::detect_file_type(std::path::Path::new(name), Some(&body));
        }
        if kind == FileKind::Unknown {
            continue;
        }
        let Ok((packages, eco)) = extract_packages(kind, &body) else {
            continue;
        };
        if packages.is_empty() {
            continue;
        }
        manifests_found += 1;
        let source = url_owner.get(&url).cloned().unwrap_or_else(|| url.clone());
        if hunt.verbose >= 1 && !hunt.silent {
            eprintln!(
                "  [+] {} → {} ({} pkgs, {})",
                url,
                kind,
                packages.len(),
                eco
            );
        }
        for p in packages {
            let key = (eco, p.name.clone());
            let entry = pkg_sources
                .entry(key)
                .or_insert_with(|| (p.version.clone(), BTreeSet::new()));
            if entry.0 == "*" || entry.0.is_empty() {
                entry.0 = p.version;
            }
            entry.1.insert(source.clone());
            entry.1.insert(url.clone());
        }
    }

    if manifests_found == 0 {
        if !hunt.silent {
            eprintln!("[-] No dependency manifests found on probed paths.");
        }
        return Ok(HuntReport {
            probed_urls: probed,
            manifests_found: 0,
            packages_checked: 0,
            vulnerable: Vec::new(),
            duration_secs: started.elapsed().as_secs_f64(),
        });
    }

    // Group by ecosystem for registry checks
    let mut by_eco: BTreeMap<Ecosystem, Vec<PackageRef>> = BTreeMap::new();
    for ((eco, name), (ver, _)) in &pkg_sources {
        by_eco
            .entry(*eco)
            .or_default()
            .push(PackageRef::new(name, ver));
    }

    let mut packages_checked = 0usize;
    let mut vulnerable = Vec::new();
    for (eco, pkgs) in by_eco {
        packages_checked += pkgs.len();
        if !hunt.silent {
            eprintln!(
                "[*] Checking {} {} package(s) against public registry…",
                pkgs.len(),
                eco
            );
        }
        let results = check_many(
            Arc::clone(&registry),
            eco,
            &pkgs,
            scan.threads.max(hunt.threads),
        )
        .await;
        for r in results {
            if r.status != CheckStatus::Vulnerable {
                continue;
            }
            let (ver, sources) = pkg_sources
                .get(&(eco, r.name.clone()))
                .cloned()
                .unwrap_or_else(|| (r.version.clone(), BTreeSet::new()));
            vulnerable.push(HuntHit {
                package: r.name.clone(),
                version: ver,
                ecosystem: eco.to_string(),
                registry_url: registry_url_for(eco, &r.name),
                sources: sources.into_iter().collect(),
            });
        }
    }
    vulnerable.sort_by(|a, b| {
        a.ecosystem
            .cmp(&b.ecosystem)
            .then_with(|| a.package.cmp(&b.package))
    });

    Ok(HuntReport {
        probed_urls: probed,
        manifests_found,
        packages_checked,
        vulnerable,
        duration_secs: started.elapsed().as_secs_f64(),
    })
}

/// Print DepenFusion-style results to stdout.
pub fn print_hunt_report(report: &HuntReport, hunt: &HuntOptions) {
    if report.vulnerable.is_empty() {
        if !hunt.silent {
            eprintln!(
                "[+] No free-namespace npm packages found (checked {}, manifests {}).",
                report.packages_checked, report.manifests_found
            );
        }
        return;
    }
    for hit in &report.vulnerable {
        if hunt.show_link {
            println!("{}", hit.registry_url);
        } else {
            println!("{} [{}]", hit.package, hit.ecosystem);
        }
        if !hunt.silent {
            for src in &hit.sources {
                eprintln!("  => {src}");
            }
        }
    }
    if !hunt.silent {
        eprintln!(
            "[*] Hunt done: {} vulnerable / {} checked in {:.1}s",
            report.vulnerable.len(),
            report.packages_checked,
            report.duration_secs
        );
    }
}

/// Read host lines from stdin (non-blocking if piped).
pub fn read_stdin_lines() -> Result<Vec<String>> {
    use std::io::{self, BufRead, IsTerminal};
    if io::stdin().is_terminal() {
        bail!("--stdin requires hosts/URLs piped on stdin (tty detected)");
    }
    let mut lines = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = line.context("read stdin")?;
        if !line.trim().is_empty() {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        bail!("stdin was empty — provide hosts/URLs");
    }
    Ok(lines)
}

/// Read host lines from a file (one per line).
pub fn read_hosts_file(path: &std::path::Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read hosts file {}", path.display()))?;
    Ok(text
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_host_https() {
        let t = parse_target_line("app.example.com", false).expect("parse");
        assert_eq!(t.base_url, "https://app.example.com/");
    }

    #[test]
    fn parses_http_with_path() {
        let t = parse_target_line("http://x.test/foo", false).expect("parse");
        assert_eq!(t.base_url, "http://x.test/foo/");
    }

    #[test]
    fn strip_path_uses_root() {
        let t = parse_target_line("https://x.test/app/v1", true).expect("parse");
        assert_eq!(t.base_url, "https://x.test/");
    }

    #[test]
    fn probe_urls_include_guide_paths() {
        let t = parse_target_line("https://x.test/", false).expect("parse");
        let u = probe_urls(&t, "token=abc", &[]);
        assert!(u.iter().any(|x| x.contains("package.json?token=abc")));
        assert!(u.iter().any(|x| x.contains("requirements.txt")));
        assert!(u.iter().any(|x| x.contains("Cargo.toml")));
        assert!(u.iter().any(|x| x.contains("composer.lock")));
        assert!(u.len() >= DEFAULT_PROBE_PATHS.len());
    }
}
