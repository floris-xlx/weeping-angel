use crate::finding::{ScanReport, Severity};
use crate::style;

pub fn print_report(report: &ScanReport) {
    let bar = style::magenta("═══════════════════════════════════════════════════════════");
    style::eprint_line("");
    style::eprint_line(&bar);
    style::eprint_line(&format!(
        "  {}  {}",
        style::brand("weeping-angel"),
        style::dim(&format!("v{}", report.version))
    ));
    style::eprint_line(&format!(
        "  {}:  {}",
        style::cyan("target"),
        style::bold(&report.target)
    ));
    style::eprint_line(&format!(
        "  {}: {}",
        style::cyan("profile"),
        style::bright_magenta(&report.profile)
    ));
    style::eprint_line(&format!(
        "  {}: {}   {}: {}   {}: {}",
        style::cyan("urls"),
        style::bold(&report.stats.urls_discovered.to_string()),
        style::cyan("requests"),
        style::bold(&report.stats.requests.to_string()),
        style::cyan("findings"),
        style::bold(&report.stats.findings_total.to_string()),
    ));
    style::eprint_line(&format!(
        "  severity: {}={} {}={} {}={} {}={} {}={}",
        style::severity_badge(Severity::Critical),
        report.stats.by_severity.critical,
        style::severity_badge(Severity::High),
        report.stats.by_severity.high,
        style::severity_badge(Severity::Medium),
        report.stats.by_severity.medium,
        style::severity_badge(Severity::Low),
        report.stats.by_severity.low,
        style::severity_badge(Severity::Info),
        report.stats.by_severity.info,
    ));
    style::eprint_line(&bar);

    let mut findings = report.findings.clone();
    findings.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.module.cmp(&b.module)));

    for f in &findings {
        if f.severity == Severity::Info && f.id == "route-discovered" {
            continue; // keep terminal quiet; still in JSON
        }
        let badge = style::severity_badge(f.severity);
        let module = style::cyan(&format!("[{}]", f.module));
        let title = style::bold(&f.title);
        let sev = style::severity_name(f.severity);
        style::eprint_line("");
        style::eprint_line(&format!("{badge} {module} {title}  ({sev})"));
        style::eprint_line(&format!(
            "  {}  {}",
            style::dim("url:"),
            style::bright_blue(&f.url)
        ));
        style::eprint_line(&format!(
            "  {}   {}",
            style::dim("id:"),
            style::dim(&f.id)
        ));
        if !f.description.is_empty() {
            style::eprint_line(&format!("  {}", f.description));
        }
        if let Some(rem) = &f.remediation {
            style::eprint_line(&format!(
                "  {}  {}",
                style::green("fix:"),
                rem
            ));
        }
        for ev in &f.evidence {
            style::eprint_line(&format!(
                "  {}{}{}: {}",
                style::yellow("evidence@"),
                style::yellow(&ev.location),
                style::dim(""),
                style::dim(&ev.snippet)
            ));
        }
    }

    let routes: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.id == "route-discovered")
        .collect();
    if !routes.is_empty() {
        style::eprint_line(&format!(
            "\n{} {} {}",
            style::dim("──"),
            style::phase(&format!("discovered routes ({})", routes.len())),
            style::dim("──")
        ));
        for r in routes.iter().take(50) {
            style::eprint_line(&format!(
                "  {} {}",
                style::green("•"),
                style::bright_blue(&r.url)
            ));
        }
        if routes.len() > 50 {
            style::eprint_line(&format!(
                "  {} {}",
                style::dim("…"),
                style::dim(&format!(
                    "and {} more (see JSON report)",
                    routes.len() - 50
                ))
            ));
        }
    }
    style::eprint_line("");
}
