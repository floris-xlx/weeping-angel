pub mod html;
pub mod json;
pub mod manifest;
pub mod openapi_gen;
pub mod sarif;
pub mod terminal;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::discovery;
use crate::finding::{Finding, ScanReport, Severity, is_inventory_finding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Terminal,
    Json,
    Sarif,
    Html,
    /// Surface / route manifest (JSON)
    Manifest,
    /// Synthesized OpenAPI 3 from recon
    OpenApi,
    /// Image harvest manifest (all img paths + HEAD/OPTIONS probes)
    Images,
}

impl Format {
    pub fn parse_list(s: &str) -> Vec<Self> {
        s.split(',')
            .filter_map(|p| match p.trim().to_ascii_lowercase().as_str() {
                "terminal" | "term" | "text" => Some(Self::Terminal),
                "json" => Some(Self::Json),
                "sarif" => Some(Self::Sarif),
                "html" => Some(Self::Html),
                "manifest" | "surface" => Some(Self::Manifest),
                "openapi" | "oas" | "swagger" => Some(Self::OpenApi),
                "images" | "image" | "image-manifest" | "img" => Some(Self::Images),
                "" => None,
                other => {
                    eprintln!("weeping-angel: warning: unknown report format `{other}` (ignored)");
                    None
                }
            })
            .collect()
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Terminal => "",
            Self::Json => "json",
            Self::Sarif => "sarif.json",
            Self::Html => "html",
            Self::Manifest => "manifest.json",
            Self::OpenApi => "openapi.json",
            Self::Images => "images.json",
        }
    }
}

/// Findings shown in human-facing reports (HTML/terminal): drop inventory noise.
pub fn findings_for_display(report: &ScanReport) -> Vec<&Finding> {
    let mut out: Vec<&Finding> = report
        .findings
        .iter()
        .filter(|f| !is_inventory_finding(f))
        .collect();
    sort_findings(&mut out);
    out
}

/// Findings suitable for SARIF / security tooling (exclude pure inventory).
pub fn security_findings(report: &ScanReport) -> Vec<&Finding> {
    let mut out: Vec<&Finding> = report
        .findings
        .iter()
        .filter(|f| !is_inventory_finding(f))
        .collect();
    sort_findings(&mut out);
    out
}

pub fn sort_findings(findings: &mut [&Finding]) {
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then(a.module.cmp(&b.module))
            .then(a.id.cmp(&b.id))
            .then(a.url.cmp(&b.url))
    });
}

pub fn write_reports(
    report: &ScanReport,
    formats: &[Format],
    output: Option<&Path>,
    max_terminal_routes: usize,
    report_width: usize,
) -> Result<()> {
    for fmt in formats {
        match fmt {
            Format::Terminal => terminal::print_report(report, max_terminal_routes, report_width),
            Format::Json => {
                write_or_print(output, fmt.extension(), &json::to_string(report)?)?;
            }
            Format::Sarif => {
                write_or_print(output, fmt.extension(), &sarif::to_string(report)?)?;
            }
            Format::Html => {
                write_or_print(output, fmt.extension(), &html::to_string(report))?;
            }
            Format::Manifest => {
                write_or_print(output, fmt.extension(), &manifest::to_string(report)?)?;
            }
            Format::OpenApi => {
                write_or_print(output, fmt.extension(), &openapi_gen::to_string(report)?)?;
            }
            Format::Images => {
                let s = if let Some(h) = &report.image_harvest {
                    discovery::image_harvest::to_string(h)?
                } else {
                    let empty = crate::discovery::image_harvest::ImageHarvestManifest {
                        tool: report.tool.clone(),
                        version: report.version.clone(),
                        target: report.target.clone(),
                        generated_at: report.finished_at.to_rfc3339(),
                        ..Default::default()
                    };
                    crate::discovery::image_harvest::to_string(&empty)?
                };
                write_or_print(output, fmt.extension(), &s)?;
            }
        }
    }
    Ok(())
}

fn write_or_print(output: Option<&Path>, ext: &str, content: &str) -> Result<()> {
    if let Some(path) = output {
        let p = if ext.contains('.') {
            with_suffix_ext(path, ext)
        } else {
            with_ext(path, ext)
        };
        std::fs::write(&p, content)?;
        eprintln!("wrote {}", p.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

fn with_suffix_ext(path: &Path, suffix: &str) -> PathBuf {
    if path.extension().is_none() {
        let mut p = path.to_path_buf();
        p.set_extension(suffix);
        return p;
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("report");
    if let Some(parent) = path.parent() {
        parent.join(format!("{stem}.{suffix}"))
    } else {
        PathBuf::from(format!("{stem}.{suffix}"))
    }
}

fn with_ext(path: &Path, ext: &str) -> PathBuf {
    if path.extension().is_some() && ext == "json" {
        return path.to_path_buf();
    }
    if path.extension().and_then(|e| e.to_str()) == Some("html") && ext == "html" {
        return path.to_path_buf();
    }
    let mut p = path.to_path_buf();
    if path.extension().is_none() {
        p.set_extension(ext);
    } else if ext != "json" {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("report");
        if let Some(parent) = path.parent() {
            return parent.join(format!("{stem}.{ext}"));
        }
        return PathBuf::from(format!("{stem}.{ext}"));
    }
    p
}

/// Shared executive summary line for HTML/terminal.
pub fn executive_summary(report: &ScanReport) -> String {
    let top = findings_for_display(report)
        .into_iter()
        .filter(|f| f.severity >= Severity::Medium)
        .take(3)
        .map(|f| format!("{} ({})", f.title, f.severity.as_str()))
        .collect::<Vec<_>>();
    let top_s = if top.is_empty() {
        "no medium+ findings".into()
    } else {
        top.join("; ")
    };
    format!(
        "Wall {:.1}s · ~{:.1} req/s · {} routes · {} modules · top: {}",
        report.timing.wall_seconds,
        report.timing.effective_rps.unwrap_or(0.0),
        report.surface.total_routes.max(report.routes.len()),
        report.module_results.len(),
        top_s
    )
}
