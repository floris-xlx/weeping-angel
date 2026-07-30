pub mod html;
pub mod json;
pub mod manifest;
pub mod openapi_gen;
pub mod sarif;
pub mod terminal;

use std::path::Path;

use anyhow::Result;

use crate::discovery;
use crate::finding::ScanReport;

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
                _ => None,
            })
            .collect()
    }
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
                let s = json::to_string(report)?;
                if let Some(path) = output {
                    let p = with_ext(path, "json");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
            Format::Sarif => {
                let s = sarif::to_string(report)?;
                if let Some(path) = output {
                    let p = with_ext(path, "sarif.json");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
            Format::Html => {
                let s = html::to_string(report);
                if let Some(path) = output {
                    let p = with_ext(path, "html");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
            Format::Manifest => {
                let s = manifest::to_string(report)?;
                if let Some(path) = output {
                    let p = with_suffix_ext(path, "manifest.json");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
            Format::OpenApi => {
                let s = openapi_gen::to_string(report)?;
                if let Some(path) = output {
                    let p = with_suffix_ext(path, "openapi.json");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
            Format::Images => {
                let s = if let Some(h) = &report.image_harvest {
                    discovery::image_harvest::to_string(h)?
                } else {
                    // empty shell so callers always get a file
                    let empty = crate::discovery::image_harvest::ImageHarvestManifest {
                        tool: report.tool.clone(),
                        version: report.version.clone(),
                        target: report.target.clone(),
                        generated_at: report.finished_at.to_rfc3339(),
                        ..Default::default()
                    };
                    crate::discovery::image_harvest::to_string(&empty)?
                };
                if let Some(path) = output {
                    let p = with_suffix_ext(path, "images.json");
                    std::fs::write(&p, s)?;
                    eprintln!("wrote {}", p.display());
                } else {
                    println!("{s}");
                }
            }
        }
    }
    Ok(())
}

fn with_suffix_ext(path: &Path, suffix: &str) -> std::path::PathBuf {
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
        std::path::PathBuf::from(format!("{stem}.{suffix}"))
    }
}

fn with_ext(path: &Path, ext: &str) -> std::path::PathBuf {
    // if user gave report.json and format json, keep; else append
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
        // report.json -> report.sarif.json style via file_stem
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("report");
        if let Some(parent) = path.parent() {
            return parent.join(format!("{stem}.{ext}"));
        }
        return std::path::PathBuf::from(format!("{stem}.{ext}"));
    }
    p
}
