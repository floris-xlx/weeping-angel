//! Report format serializers + format list parsing.

use chrono::Utc;
use weeping_angel::finding::{
    Evidence, Finding, ModuleSummary, PhaseTiming, ScanReport, ScanStats, Severity, SeverityCounts,
    SourceCount, StatusCount, SurfaceInventory, TimingSummary,
};
use weeping_angel::report::{Format, html, json, manifest, openapi_gen, sarif, write_reports};

fn sample_report() -> ScanReport {
    ScanReport {
        tool: "weeping-angel".into(),
        version: "0.1.2".into(),
        target: "https://lab.example/".into(),
        started_at: Utc::now(),
        finished_at: Utc::now(),
        profile: "standard".into(),
        modules: vec!["discovery".into(), "headers".into(), "secrets".into()],
        discovered_urls: vec![
            "https://lab.example/".into(),
            "https://lab.example/login".into(),
            "https://lab.example/api/v1/me".into(),
        ],
        findings: vec![
            Finding::builder("secrets", "aws-key")
                .title("AWS access key")
                .severity(Severity::Critical)
                .url("https://lab.example/")
                .description("Found AKIA…")
                .remediation("Rotate keys")
                .cwe("CWE-798")
                .evidence(Evidence::new("body", "AKIAIOSFODNN7EXAMPLE"))
                .build(),
            Finding::builder("headers", "missing-csp")
                .title("Missing CSP")
                .severity(Severity::Medium)
                .url("https://lab.example/")
                .description("No Content-Security-Policy")
                .build(),
            Finding::builder("discovery", "route-discovered")
                .title("Discovered route (crawl)")
                .severity(Severity::Info)
                .url("https://lab.example/login")
                .description("URL discovered via crawl. HTTP status 200.")
                .build(),
            Finding::builder("tech", "server-header")
                .title("Server: lab")
                .severity(Severity::Info)
                .url("https://lab.example/")
                .description("fingerprint")
                .build(),
        ],
        stats: ScanStats {
            requests: 42,
            urls_discovered: 3,
            findings_total: 4,
            by_severity: SeverityCounts {
                critical: 1,
                high: 0,
                medium: 1,
                low: 0,
                info: 2,
            },
        },
        image_harvest: None,
        phases: vec![
            PhaseTiming {
                name: "crawl".into(),
                seconds: 1.2,
                detail: Some("assets=3".into()),
            },
            PhaseTiming {
                name: "wordlist".into(),
                seconds: 3.4,
                detail: None,
            },
        ],
        module_results: vec![
            ModuleSummary {
                id: "discovery".into(),
                ran: true,
                findings: 1,
                note: None,
            },
            ModuleSummary {
                id: "secrets".into(),
                ran: true,
                findings: 1,
                note: None,
            },
        ],
        surface: SurfaceInventory {
            total_routes: 3,
            routes_by_source: vec![SourceCount {
                name: "crawl".into(),
                count: 3,
            }],
            status_histogram: vec![StatusCount {
                status: 200,
                count: 3,
            }],
            content_types: vec![SourceCount {
                name: "text/html".into(),
                count: 2,
            }],
        },
        tech_stack: vec!["Server: lab".into()],
        timing: TimingSummary {
            wall_seconds: 5.5,
            requests: 42,
            effective_rps: Some(7.6),
        },
    }
}

#[test]
fn format_parse_list_aliases() {
    let spaced = "terminal, JSON ,html".to_string();
    let f = Format::parse_list(&spaced);
    assert!(f.contains(&Format::Terminal));
    assert!(f.contains(&Format::Json));
    assert!(f.contains(&Format::Html));

    let f = Format::parse_list(
        "terminal,json,sarif,html,manifest,openapi,images,term,text,oas,swagger,surface,img",
    );
    assert!(f.contains(&Format::Terminal));
    assert!(f.contains(&Format::Json));
    assert!(f.contains(&Format::Sarif));
    assert!(f.contains(&Format::Html));
    assert!(f.contains(&Format::Manifest));
    assert!(f.contains(&Format::OpenApi));
    assert!(f.contains(&Format::Images));
    assert!(Format::parse_list("").is_empty());
    assert!(Format::parse_list("nope").is_empty());
}

#[test]
fn json_roundtrip_includes_wide_fields() {
    let report = sample_report();
    let s = json::to_string(&report).unwrap();
    assert!(s.contains("\"phases\""));
    assert!(s.contains("\"surface\""));
    assert!(s.contains("\"tech_stack\""));
    assert!(s.contains("\"module_results\""));
    assert!(s.contains("\"timing\""));
    assert!(s.contains("aws-key"));
    let back: ScanReport = serde_json::from_str(&s).unwrap();
    assert_eq!(back.phases.len(), 2);
    assert_eq!(back.surface.total_routes, 3);
    assert_eq!(back.stats.by_severity.critical, 1);
}

#[test]
fn html_contains_dashboard_sections() {
    let html = html::to_string(&sample_report());
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Executive summary") || html.contains("executive"));
    assert!(html.contains("Phase timings") || html.contains("phase"));
    assert!(html.contains("critical"));
    assert!(html.contains("AWS access key") || html.contains("aws"));
    assert!(html.contains("sev-filters") || html.contains("data-sev"));
    assert!(html.contains("lab.example"));
}

#[test]
fn sarif_is_valid_shape() {
    let s = sarif::to_string(&sample_report()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("runs").is_some() || v.get("$schema").is_some() || v.get("version").is_some());
    assert!(s.contains("secrets:aws-key") || s.contains("aws-key") || s.contains("AWS"));
}

#[test]
fn manifest_has_routes_and_stats() {
    let m = manifest::from_report(&sample_report());
    assert!(!m.routes.is_empty());
    assert_eq!(m.target, "https://lab.example/");
    let s = manifest::to_string(&sample_report()).unwrap();
    assert!(s.contains("routes"));
}

#[test]
fn openapi_has_paths_from_discovery() {
    let doc = openapi_gen::from_report(&sample_report());
    assert_eq!(doc["openapi"], "3.0.3");
    let paths = doc["paths"].as_object().unwrap();
    assert!(
        paths.keys().any(|k| k.contains("login") || k == "/login" || k.contains("api")),
        "paths={:?}",
        paths.keys().collect::<Vec<_>>()
    );
}

#[test]
fn write_reports_to_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let prefix = dir.path().join("out");
    let report = sample_report();
    write_reports(
        &report,
        &[Format::Json, Format::Html, Format::Sarif, Format::Manifest, Format::OpenApi],
        Some(&prefix),
        50,
        100,
    )
    .unwrap();

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        entries.iter().any(|n| n.contains("json") || n.ends_with(".json")),
        "entries={entries:?}"
    );
    assert!(
        entries.iter().any(|n| n.contains("html")),
        "entries={entries:?}"
    );
}

#[test]
fn terminal_print_does_not_panic() {
    // Should not panic; writes to stderr
    weeping_angel::report::terminal::print_report(&sample_report(), 10, 80);
    weeping_angel::report::terminal::print_report(&sample_report(), 0, 0);
}

#[test]
fn evidence_truncates_long_snippets() {
    let long = "x".repeat(2000);
    let e = Evidence::new("body", long);
    assert!(e.snippet.chars().count() <= 501);
    assert!(e.snippet.ends_with('…') || e.snippet.len() <= 500);
}

#[test]
fn severity_ordering_and_parse() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert_eq!(Severity::from_str_loose("crit"), Some(Severity::Critical));
    assert_eq!(Severity::from_str_loose("med"), Some(Severity::Medium));
    assert_eq!(Severity::from_str_loose("informational"), Some(Severity::Info));
    assert_eq!(Severity::from_str_loose("nope"), None);
    assert_eq!(Severity::Low.as_str(), "low");
    assert_eq!(format!("{}", Severity::High), "high");
}

#[test]
fn scan_stats_from_findings() {
    let report = sample_report();
    let stats = ScanStats::from_findings(&report.findings, 99, 5);
    assert_eq!(stats.requests, 99);
    assert_eq!(stats.urls_discovered, 5);
    assert_eq!(stats.findings_total, 4);
    assert_eq!(stats.by_severity.critical, 1);
    assert_eq!(stats.by_severity.medium, 1);
}
