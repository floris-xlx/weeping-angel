use crate::finding::{ScanReport, Severity};

pub fn to_string(report: &ScanReport) -> String {
    let mut rows = String::new();
    let mut findings = report.findings.clone();
    findings.sort_by(|a, b| b.severity.cmp(&a.severity));

    for f in &findings {
        if f.id == "route-discovered" {
            continue;
        }
        rows.push_str(&format!(
            r#"<tr class="{sev}"><td>{sev}</td><td>{module}</td><td>{title}</td><td><a href="{url}">{url}</a></td><td>{desc}</td></tr>"#,
            sev = f.severity.as_str(),
            module = escape(&f.module),
            title = escape(&f.title),
            url = escape(&f.url),
            desc = escape(&f.description),
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<title>weeping-angel report — {target}</title>
<style>
body {{ font-family: ui-sans-serif, system-ui, sans-serif; margin: 2rem; background: #0f1419; color: #e7ecf1; }}
h1 {{ font-weight: 600; }}
.meta {{ color: #9aa7b5; margin-bottom: 1.5rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border-bottom: 1px solid #243040; padding: 0.6rem 0.5rem; text-align: left; vertical-align: top; }}
th {{ color: #9aa7b5; font-size: 0.85rem; }}
a {{ color: #7cb7ff; }}
.critical td:first-child {{ color: #ff6b6b; font-weight: 700; }}
.high td:first-child {{ color: #ff9f43; font-weight: 700; }}
.medium td:first-child {{ color: #feca57; }}
.low td:first-child {{ color: #54a0ff; }}
.info td:first-child {{ color: #9aa7b5; }}
</style>
</head>
<body>
<h1>weeping-angel</h1>
<div class="meta">
  <div>Target: {target}</div>
  <div>Profile: {profile} · Requests: {requests} · URLs: {urls} · Findings: {total}</div>
  <div>Critical: {c} · High: {h} · Medium: {m} · Low: {l} · Info: {i}</div>
</div>
<table>
<thead><tr><th>Severity</th><th>Module</th><th>Title</th><th>URL</th><th>Description</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
</body>
</html>"#,
        target = escape(&report.target),
        profile = escape(&report.profile),
        requests = report.stats.requests,
        urls = report.stats.urls_discovered,
        total = report.stats.findings_total,
        c = report.stats.by_severity.critical,
        h = report.stats.by_severity.high,
        m = report.stats.by_severity.medium,
        l = report.stats.by_severity.low,
        i = report.stats.by_severity.info,
        rows = rows,
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[allow(dead_code)]
fn _sev_class(s: Severity) -> &'static str {
    s.as_str()
}
