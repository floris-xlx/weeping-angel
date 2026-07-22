use crate::finding::{ScanReport, Severity};

pub fn print_report(report: &ScanReport) {
    eprintln!();
    eprintln!("═══════════════════════════════════════════════════════════");
    eprintln!("  weeping-angel  v{}", report.version);
    eprintln!("  target:  {}", report.target);
    eprintln!("  profile: {}", report.profile);
    eprintln!(
        "  urls: {}   requests: {}   findings: {}",
        report.stats.urls_discovered, report.stats.requests, report.stats.findings_total
    );
    eprintln!(
        "  severity: crit={} high={} med={} low={} info={}",
        report.stats.by_severity.critical,
        report.stats.by_severity.high,
        report.stats.by_severity.medium,
        report.stats.by_severity.low,
        report.stats.by_severity.info
    );
    eprintln!("═══════════════════════════════════════════════════════════");

    let mut findings = report.findings.clone();
    findings.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.module.cmp(&b.module)));

    for f in &findings {
        if f.severity == Severity::Info && f.id == "route-discovered" {
            continue; // keep terminal quiet; still in JSON
        }
        let badge = severity_badge(f.severity);
        eprintln!(
            "\n{badge} [{}] {}  ({})",
            f.module, f.title, f.severity
        );
        eprintln!("  url:  {}", f.url);
        eprintln!("  id:   {}", f.id);
        if !f.description.is_empty() {
            eprintln!("  {}", f.description);
        }
        if let Some(rem) = &f.remediation {
            eprintln!("  fix:  {rem}");
        }
        for ev in &f.evidence {
            eprintln!("  evidence@{}: {}", ev.location, ev.snippet);
        }
    }

    let routes: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.id == "route-discovered")
        .collect();
    if !routes.is_empty() {
        eprintln!("\n── discovered routes ({}) ──", routes.len());
        for r in routes.iter().take(50) {
            eprintln!("  • {}", r.url);
        }
        if routes.len() > 50 {
            eprintln!("  … and {} more (see JSON report)", routes.len() - 50);
        }
    }
    eprintln!();
}

fn severity_badge(s: Severity) -> &'static str {
    match s {
        Severity::Critical => "[CRIT]",
        Severity::High => "[HIGH]",
        Severity::Medium => "[MED ]",
        Severity::Low => "[LOW ]",
        Severity::Info => "[INFO]",
    }
}
