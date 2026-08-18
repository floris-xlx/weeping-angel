//! Terminal + JSON reporting for depcheck scans.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};

use super::types::{CheckStatus, ScanSummary};
use crate::style;

pub fn print_summary(summary: &ScanSummary, quiet: bool) {
    if quiet {
        for v in &summary.vulnerable {
            println!("{}", v.name);
        }
        return;
    }

    let width = style::terminal_width(0);
    eprintln!("{}", style::rule(width, '='));
    eprintln!("  {}", style::phase("SCAN RESULTS"));
    eprintln!("{}", style::rule(width, '='));
    eprintln!("  File:       {}", summary.file);
    eprintln!("  Ecosystem:  {}", summary.ecosystem);
    eprintln!("  Kind:       {}", summary.file_kind);
    eprintln!("  Total:      {}", summary.total());
    eprintln!("  Safe:       {}", summary.safe.len());
    eprintln!(
        "  Vulnerable: {}",
        if summary.vulnerable.is_empty() {
            style::ok(&summary.vulnerable.len().to_string())
        } else {
            style::err(&summary.vulnerable.len().to_string())
        }
    );
    if !summary.suppressed.is_empty() {
        eprintln!(
            "  Suppressed: {} (known-secure namespaces)",
            summary.suppressed.len()
        );
    }
    eprintln!("  Errors:     {}", summary.errors.len());
    eprintln!("  Duration:   {:.1}s", summary.duration_secs);
    eprintln!("{}", style::rule(width, '='));

    if !summary.vulnerable.is_empty() {
        eprintln!();
        eprintln!(
            "  {} Issues found — packages not available in public repositories:",
            style::err("[!]")
        );
        eprintln!();
        for v in &summary.vulnerable {
            eprintln!("  {} {} @ {}", style::err("[!]"), v.name, v.version);
        }
        eprintln!();
        eprintln!(
            "  These names are free on the public {} registry (lingering namespace).",
            summary.ecosystem
        );
        eprintln!(
            "  If you use a private registry, claim/register these namespaces with a trusted party"
        );
        eprintln!(
            "  (typically your company) so an attacker cannot publish a higher version first."
        );
        if summary.ecosystem.as_str() == "npm" {
            eprintln!();
            eprintln!("  Note: npm scopes (@org/…) are not always publicly visible.");
            eprintln!("  If you already own a scope, pass -s '@org/*' (confused-compatible).");
        }
    } else if summary.errors.is_empty() {
        eprintln!();
        eprintln!(
            "  {} All checked packages exist on the public {} registry",
            style::ok("[OK]"),
            summary.ecosystem
        );
        if !summary.suppressed.is_empty() {
            eprintln!(
                "  ({} suppressed via known-secure namespaces).",
                summary.suppressed.len()
            );
        } else {
            eprintln!("  No dependency confusion / free-namespace issue found.");
        }
    }

    if !summary.errors.is_empty() {
        eprintln!();
        eprintln!(
            "  {} Could not verify {} package(s) (network / registry errors):",
            style::warn("[?]"),
            summary.errors.len()
        );
        for e in &summary.errors {
            let detail = e.detail.as_deref().unwrap_or("error");
            eprintln!("  {} {} ({})", style::warn("[?]"), e.name, detail);
        }
    }

    if !summary.introductions.is_empty() {
        eprintln!();
        eprintln!("  {}", style::phase("INSPECTOR (git introduction)"));
        for intro in &summary.introductions {
            eprintln!(
                "  {} {} ← {} ({})",
                style::warn("[i]"),
                intro.package,
                &intro.commit[..intro.commit.len().min(12)],
                intro.date
            );
            eprintln!(
                "      \"{}\" — {} — {}",
                intro.subject, intro.author, intro.file
            );
        }
    }

    if let Some(h) = &summary.hardening {
        if let Some(ep) = &h.entrypoint {
            eprintln!();
            eprintln!(
                "  Entrypoint: {} ({})",
                ep,
                if h.entrypoint_exists == Some(true) {
                    "found"
                } else {
                    "missing"
                }
            );
        }
        if !h.findings.is_empty() {
            eprintln!();
            eprintln!("  {}", style::phase("HARDENING RECON"));
            for f in &h.findings {
                let sev = match f.severity.as_str() {
                    "high" | "critical" => style::err(&f.severity),
                    "medium" => style::warn(&f.severity),
                    _ => style::dim(&f.severity),
                };
                eprintln!("  [{sev}] {}", f.title);
                eprintln!("      {}", f.detail);
                eprintln!("      → {}", f.remediation);
            }
        }
        if !h.npmrc_paths.is_empty() {
            eprintln!("  .npmrc: {}", h.npmrc_paths.join(", "));
        }
    }

    eprintln!();
    let _ = io::stderr().flush();
}

pub fn export_json(path: &Path, summary: &ScanSummary) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_string_pretty(summary)?;
    fs::write(path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn list_packages(summary: &ScanSummary) {
    for (name, version) in &summary.packages {
        println!("{name} @ {version}");
    }
}

pub fn partition_results(
    results: Vec<super::types::PackageResult>,
) -> (
    Vec<super::types::PackageResult>,
    Vec<super::types::PackageResult>,
    Vec<super::types::PackageResult>,
) {
    let mut vulnerable = Vec::new();
    let mut safe = Vec::new();
    let mut errors = Vec::new();
    for r in results {
        match r.status {
            CheckStatus::Vulnerable => vulnerable.push(r),
            CheckStatus::Safe => safe.push(r),
            CheckStatus::Error => errors.push(r),
        }
    }
    vulnerable.sort_by(|a, b| a.name.cmp(&b.name));
    safe.sort_by(|a, b| a.name.cmp(&b.name));
    errors.sort_by(|a, b| a.name.cmp(&b.name));
    (vulnerable, safe, errors)
}
