//! Loki-style npm hardening recon (analysis only — no attack / publish / reverse shell).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{Ecosystem, PackageResult};

/// One hardening / misconfiguration finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningFinding {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub remediation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Aggregated npm / Node supply-chain hardening report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardeningReport {
    pub findings: Vec<HardeningFinding>,
    pub npmrc_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint_exists: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
}

/// Analyze a project directory (and optional package.json) for hybrid-registry / range risks.
pub fn analyze_npm_project(
    project_root: &Path,
    package_json: Option<&Path>,
    vulnerable: &[PackageResult],
    entrypoint_override: Option<&str>,
) -> HardeningReport {
    let mut report = HardeningReport::default();

    // Collect .npmrc files (project + parents up to filesystem root, capped)
    report.npmrc_paths = find_npmrc_chain(project_root);
    let npmrc_snapshot = report.npmrc_paths.clone();
    for p in &npmrc_snapshot {
        if let Ok(text) = fs::read_to_string(p) {
            analyze_npmrc(&text, p, &mut report);
        }
    }

    let pkg_path = package_json.map(|p| p.to_path_buf()).or_else(|| {
        let p = project_root.join("package.json");
        p.is_file().then_some(p)
    });

    if let Some(ref pj) = pkg_path {
        if let Ok(text) = fs::read_to_string(pj) {
            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                report.package_name = data
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                analyze_package_json(
                    &data,
                    pj,
                    vulnerable,
                    entrypoint_override,
                    project_root,
                    &mut report,
                );
            }
        }
    } else if let Some(ep) = entrypoint_override {
        report.entrypoint = Some(ep.to_string());
        report.entrypoint_exists = Some(project_root.join(ep).is_file());
    }

    // Vulnerable scoped packages without matching @scope:registry=
    for v in vulnerable {
        if let Some(scope) = npm_scope(&v.name) {
            let key = format!("{scope}:registry");
            let has_scope_registry = report.npmrc_paths.iter().any(|p| {
                fs::read_to_string(p)
                    .map(|t| t.lines().any(|l| l.trim_start().starts_with(&key)))
                    .unwrap_or(false)
            });
            if !has_scope_registry {
                report.findings.push(HardeningFinding {
                    id: "npm.scope-without-private-registry".into(),
                    severity: "high".into(),
                    title: format!("Scoped package `{name}` has no `{key}` in .npmrc", name = v.name),
                    detail: format!(
                        "Free-namespace / missing-on-public hit for scoped dependency `{}`. \
                         Without `{key}=https://your-private-registry/`, installs may fall back to the public npm registry.",
                        v.name
                    ),
                    remediation: format!(
                        "Claim the npm scope `{scope}`, set `{key}` to your private registry, \
                         and configure the proxy never to fetch unknown packages from the public realm (e.g. Verdaccio uplink policy)."
                    ),
                    path: Some(v.name.clone()),
                });
            }
        }

        if is_floating_range(&v.version) {
            report.findings.push(HardeningFinding {
                id: "npm.floating-range-on-private-candidate".into(),
                severity: "high".into(),
                title: format!("Floating range on missing package `{}`", v.name),
                detail: format!(
                    "`{}` @ `{}` is not on the public registry, but the range allows newer versions. \
                     In hybrid private+public setups, a public package with a higher version can win (dependency confusion).",
                    v.name, v.version
                ),
                remediation: "Pin exact versions (no ^ or ~) for private packages, or use a lockfile + private-only resolution."
                    .into(),
                path: Some(v.name.clone()),
            });
        }
    }

    if vulnerable.iter().any(|v| !v.name.starts_with('@')) {
        report.findings.push(HardeningFinding {
            id: "npm.unscoped-private-candidate".into(),
            severity: "medium".into(),
            title: "Unscoped packages missing from public registry".into(),
            detail: "One or more unscoped names were not found on the public npm registry. \
                     Unscoped names are especially exposed to typosquatting and confusion attacks."
                .into(),
            remediation: "Prefer scoped packages (@company/…), pin versions, and ensure private registry resolution never falls through to npmjs.org for unknown names."
                .into(),
            path: None,
        });
    }

    report
}

fn analyze_npmrc(text: &str, path: &str, report: &mut HardeningReport) {
    let mut has_public = false;
    let mut has_always_auth = false;
    let mut scope_regs = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with("registry=") {
            let val = line.split_once('=').map(|(_, v)| v.trim()).unwrap_or("");
            if val.contains("registry.npmjs.org") || val.contains("registry.npmjs.com") {
                has_public = true;
            }
        }
        if line.contains(":registry=") {
            scope_regs += 1;
        }
        if line.starts_with("always-auth=") {
            has_always_auth = line.contains("true");
        }
        // Explicit public registry auth token lines are normal; note hybrid risk when mixed
        if line.contains("//registry.npmjs.org/") {
            has_public = true;
        }
    }

    if has_public && scope_regs == 0 {
        report.findings.push(HardeningFinding {
            id: "npm.npmrc-public-default-no-scopes".into(),
            severity: "medium".into(),
            title: "Default registry points at public npm without scoped remaps".into(),
            detail: format!(
                "`{path}` configures the public npm registry without `@scope:registry=` entries. \
                 Hybrid installs can resolve missing private names from the public registry."
            ),
            remediation: "Map private scopes to an internal registry and disable public fallback for unknown packages (Verdaccio / Artifactory / npm Enterprise)."
                .into(),
            path: Some(path.to_string()),
        });
    }

    if has_public && !has_always_auth && scope_regs > 0 {
        report.findings.push(HardeningFinding {
            id: "npm.npmrc-hybrid-scopes".into(),
            severity: "info".into(),
            title: "Hybrid scoped + public registry configuration detected".into(),
            detail: format!(
                "`{path}` combines scoped private registries with public npm. Ensure the private proxy never uplinks missing packages to the public realm."
            ),
            remediation: "Review Verdaccio/uplink `max_fails` / `cache` / package access rules so unpublished private names never resolve publicly."
                .into(),
            path: Some(path.to_string()),
        });
    }
}

fn analyze_package_json(
    data: &Value,
    pj: &Path,
    vulnerable: &[PackageResult],
    entrypoint_override: Option<&str>,
    project_root: &Path,
    report: &mut HardeningReport,
) {
    let entry = entrypoint_override
        .map(str::to_string)
        .or_else(|| {
            data.get("main")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            data.get("module")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| Some("index.js".into()));

    if let Some(ep) = entry {
        let exists = project_root.join(&ep).is_file()
            || pj.parent().map(|p| p.join(&ep).is_file()).unwrap_or(false);
        report.entrypoint = Some(ep);
        report.entrypoint_exists = Some(exists);
        if !exists {
            report.findings.push(HardeningFinding {
                id: "npm.entrypoint-missing".into(),
                severity: "low".into(),
                title: "Configured entrypoint file not found".into(),
                detail: format!(
                    "Entrypoint `{}` (from --entrypoint / package.json main|module) was not found under the project root.",
                    report.entrypoint.as_deref().unwrap_or("?")
                ),
                remediation: "Confirm the application entry file path for impact analysis.".into(),
                path: report.entrypoint.clone(),
            });
        }
    }

    // Import-name ≠ install-name hint: look for require/import of vulnerable names in entrypoint is heavy;
    // instead warn when package.json dependency key looks like an import-style path.
    for v in vulnerable {
        if v.name.contains('/') && !v.name.starts_with('@') {
            report.findings.push(HardeningFinding {
                id: "npm.install-vs-import-name".into(),
                severity: "medium".into(),
                title: format!("Dependency key `{}` looks like a path/import name", v.name),
                detail: "Package install names sometimes differ from import names (similar to Python opencv-python vs cv2). \
                         A mistaken install name can resolve a public typosquat."
                    .into(),
                remediation: "Verify the published package name on your private registry matches package.json keys."
                    .into(),
                path: Some(v.name.clone()),
            });
        }
    }

    let _ = Ecosystem::Npm; // keep ecosystem import meaningful for future expansion
}

fn find_npmrc_chain(start: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };
    for _ in 0..8 {
        let p = cur.join(".npmrc");
        if p.is_file() {
            out.push(p.display().to_string());
        }
        if cur.join(".git").exists() {
            break;
        }
        if !cur.pop() {
            break;
        }
    }
    out
}

fn npm_scope(name: &str) -> Option<String> {
    if !name.starts_with('@') {
        return None;
    }
    let rest = &name[1..];
    let scope = rest.split('/').next().unwrap_or("");
    if scope.is_empty() {
        None
    } else {
        Some(format!("@{scope}"))
    }
}

/// True for npm-style floating ranges that allow newer versions.
pub fn is_floating_range(version: &str) -> bool {
    let v = version.trim();
    v.starts_with('^')
        || v.starts_with('~')
        || v == "*"
        || v == "latest"
        || v.starts_with(">=")
        || v.starts_with('>')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_ranges() {
        assert!(is_floating_range("^1.2.3"));
        assert!(is_floating_range("~1.2.3"));
        assert!(!is_floating_range("1.2.3"));
    }

    #[test]
    fn scope_parse() {
        assert_eq!(npm_scope("@acme/pkg").as_deref(), Some("@acme"));
        assert_eq!(npm_scope("lodash"), None);
    }
}
