use std::collections::BTreeMap;

use crate::finding::ScanReport;
use crate::report::{executive_summary, findings_for_display};
use crate::style;

pub fn print_report(report: &ScanReport, max_routes: usize, report_width: usize) {
    let width = style::terminal_width(report_width);
    let bar = style::rule(width, '═');

    style::eprint_line("");
    style::eprint_line(&bar);
    style::eprint_line(&format!(
        "  {}  {}  {}",
        style::brand("weeping-angel"),
        style::dim(&format!("v{}", report.version)),
        style::dim(&format!("{:.1}s", report.timing.wall_seconds))
    ));
    style::eprint_line(&format!(
        "  {}:  {}",
        style::cyan("target"),
        style::bold(&report.target)
    ));
    style::eprint_line(&format!(
        "  {}: {}   {}: {}   {}: {}   {}: {}",
        style::cyan("profile"),
        style::bright_magenta(&report.profile),
        style::cyan("urls"),
        style::bold(&report.stats.urls_discovered.to_string()),
        style::cyan("requests"),
        style::bold(&report.stats.requests.to_string()),
        style::cyan("findings"),
        style::bold(&report.stats.findings_total.to_string()),
    ));
    if let Some(rps) = report.timing.effective_rps {
        style::eprint_line(&format!(
            "  {}: {:.1} req/s wall",
            style::cyan("throughput"),
            rps
        ));
    }
    style::eprint_line(&format!(
        "  severity: {}",
        style::severity_heat(
            report.stats.by_severity.critical,
            report.stats.by_severity.high,
            report.stats.by_severity.medium,
            report.stats.by_severity.low,
            report.stats.by_severity.info,
        )
    ));
    style::eprint_line(&bar);

    // Phase timings
    if !report.phases.is_empty() {
        style::eprint_line(&style::section_title(width, "phase timings"));
        for p in &report.phases {
            let detail = p
                .detail
                .as_deref()
                .map(|d| format!("  {}", style::dim(d)))
                .unwrap_or_default();
            style::eprint_line(&format!(
                "  {} {:>7.2}s  {}{}",
                style::bright_cyan(&format!("{:<16}", p.name)),
                p.seconds,
                style::dim("│"),
                detail
            ));
        }
    }

    // Surface inventory
    if report.surface.total_routes > 0 {
        style::eprint_line(&style::section_title(
            width,
            &format!("surface ({} routes)", report.surface.total_routes),
        ));
        if !report.surface.routes_by_source.is_empty() {
            let parts: Vec<String> = report
                .surface
                .routes_by_source
                .iter()
                .map(|s| {
                    format!(
                        "{}={}",
                        style::cyan(&s.name),
                        style::bold(&s.count.to_string())
                    )
                })
                .collect();
            style::eprint_line(&format!("  by source: {}", parts.join("  ")));
        }
        if !report.surface.status_histogram.is_empty() {
            let parts: Vec<String> = report
                .surface
                .status_histogram
                .iter()
                .take(12)
                .map(|s| {
                    format!(
                        "{}×{}",
                        style::http_status(s.status),
                        style::bold(&s.count.to_string())
                    )
                })
                .collect();
            style::eprint_line(&format!("  status: {}", parts.join("  ")));
        }
        if !report.surface.content_types.is_empty() {
            let parts: Vec<String> = report
                .surface
                .content_types
                .iter()
                .take(8)
                .map(|s| format!("{}({})", style::dim(&s.name), s.count))
                .collect();
            style::eprint_line(&format!("  types: {}", parts.join("  ")));
        }
    }

    // Module results
    if !report.module_results.is_empty() {
        style::eprint_line(&style::section_title(width, "modules"));
        let mut line = String::from("  ");
        for (i, m) in report.module_results.iter().enumerate() {
            let cell = format!(
                "{}:{}",
                style::cyan(&m.id),
                if m.findings > 0 {
                    style::bold(&m.findings.to_string())
                } else {
                    style::dim("0")
                }
            );
            if i > 0 && i % 5 == 0 {
                style::eprint_line(&line);
                line = String::from("  ");
            }
            if !line.trim().is_empty() && line != "  " {
                line.push_str("  ");
            }
            line.push_str(&cell);
        }
        if line.trim().len() > 0 {
            style::eprint_line(&line);
        }
    }

    // Tech stack
    if !report.tech_stack.is_empty() {
        style::eprint_line(&style::section_title(width, "tech"));
        for t in report.tech_stack.iter().take(20) {
            style::eprint_line(&format!("  {} {}", style::green("•"), t));
        }
    }

    style::eprint_line(&style::section_title(width, "summary"));
    style::eprint_line(&format!("  {}", style::dim(&executive_summary(report))));

    // Findings (non-inventory)
    style::eprint_line(&style::section_title(width, "findings"));
    let findings = findings_for_display(report);

    let mut shown = 0usize;
    for f in &findings {
        shown += 1;
        let badge = style::severity_badge(f.severity);
        let module = style::cyan(&format!("[{}]", f.module));
        let title = style::bold(&f.title);
        let sev = style::severity_name(f.severity);
        style::eprint_line("");
        style::eprint_line(&format!("{badge} {module} {title}  ({sev})"));
        style::eprint_line(&format!(
            "  {}  {}",
            style::dim("url:"),
            style::bright_blue(&style::truncate_url(&f.url, width.saturating_sub(12)))
        ));
        style::eprint_line(&format!("  {}   {}", style::dim("id:"), style::dim(&f.id)));
        if let Some(cwe) = &f.cwe {
            style::eprint_line(&format!("  {}  {}", style::dim("cwe:"), style::yellow(cwe)));
        }
        if !f.description.is_empty() {
            for chunk in wrap_text(&f.description, width.saturating_sub(4)) {
                style::eprint_line(&format!("  {chunk}"));
            }
        }
        if let Some(rem) = &f.remediation {
            style::eprint_line(&format!("  {}  {}", style::green("fix:"), rem));
        }
        for ev in &f.evidence {
            style::eprint_line(&format!(
                "  {}{}: {}",
                style::yellow("evidence@"),
                style::yellow(&ev.location),
                style::dim(&style::truncate_url(&ev.snippet, width.saturating_sub(20)))
            ));
        }
    }
    if shown == 0 {
        style::eprint_line(&format!("  {}", style::dim("(no security findings)")));
    }

    // Routes from structured inventory
    let route_count = if !report.routes.is_empty() {
        report.routes.len()
    } else {
        report
            .findings
            .iter()
            .filter(|f| f.id == "route-discovered")
            .count()
    };
    if route_count > 0 {
        style::eprint_line(&style::section_title(
            width,
            &format!("discovered routes ({route_count})"),
        ));
        let mut by_src: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        if !report.routes.is_empty() {
            for r in &report.routes {
                let src = if r.source.is_empty() {
                    "other".into()
                } else {
                    r.source.clone()
                };
                by_src.entry(src).or_default().push(r.url.as_str());
            }
        } else {
            for f in report
                .findings
                .iter()
                .filter(|f| f.id == "route-discovered")
            {
                by_src
                    .entry("discovery".into())
                    .or_default()
                    .push(f.url.as_str());
            }
        }
        let mut printed = 0usize;
        for (src, urls) in &by_src {
            style::eprint_line(&format!(
                "  {} {} ({})",
                style::dim("▸"),
                style::bright_magenta(src),
                urls.len()
            ));
            for u in urls {
                if printed >= max_routes {
                    break;
                }
                style::eprint_line(&format!(
                    "    {} {}",
                    style::green("•"),
                    style::bright_blue(&style::truncate_url(u, width.saturating_sub(8)))
                ));
                printed += 1;
            }
            if printed >= max_routes {
                break;
            }
        }
        if route_count > printed {
            style::eprint_line(&format!(
                "  {} {}",
                style::dim("…"),
                style::dim(&format!(
                    "and {} more (raise --max-terminal-routes or see JSON)",
                    route_count - printed
                ))
            ));
        }
    }

    style::eprint_line("");
    style::eprint_line(&bar);
    style::eprint_line("");
}

fn wrap_text(s: &str, width: usize) -> Vec<String> {
    if width < 20 {
        return vec![s.to_string()];
    }
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.len() + 1 + word.len() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
