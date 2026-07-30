use crate::finding::ScanReport;
use crate::report::{executive_summary, findings_for_display};

pub fn to_string(report: &ScanReport) -> String {
    let mut finding_rows = String::new();
    let findings = findings_for_display(report);

    for f in &findings {
        let evidence = f
            .evidence
            .iter()
            .map(|e| {
                format!(
                    "<div class=\"ev\"><code>{}</code> {}</div>",
                    escape(&e.location),
                    escape(&e.snippet)
                )
            })
            .collect::<String>();
        let rem = f
            .remediation
            .as_deref()
            .map(|r| format!("<div class=\"rem\"><strong>Fix:</strong> {}</div>", escape(r)))
            .unwrap_or_default();
        let cwe = f
            .cwe
            .as_deref()
            .map(|c| format!(" <span class=\"cwe\">{}</span>", escape(c)))
            .unwrap_or_default();
        finding_rows.push_str(&format!(
            r#"<tr class="sev-{sev}" data-sev="{sev}" data-module="{module}">
<td><span class="badge {sev}">{sev}</span></td>
<td>{module}</td>
<td><code class="fid">{id}</code> <strong>{title}</strong>{cwe}<div class="desc">{desc}</div>{rem}{evidence}</td>
<td><a href="{url}">{url}</a></td>
</tr>"#,
            sev = f.severity.as_str(),
            module = escape(&f.module),
            id = escape(&f.id),
            title = escape(&f.title),
            cwe = cwe,
            desc = escape(&f.description),
            rem = rem,
            evidence = evidence,
            url = escape(&f.url),
        ));
    }

    let phase_rows: String = report
        .phases
        .iter()
        .map(|p| {
            format!(
                "<tr><td>{}</td><td>{:.2}s</td><td>{}</td></tr>",
                escape(&p.name),
                p.seconds,
                escape(p.detail.as_deref().unwrap_or(""))
            )
        })
        .collect();

    let source_chips: String = report
        .surface
        .routes_by_source
        .iter()
        .map(|s| {
            format!(
                "<span class=\"chip\">{} <b>{}</b></span>",
                escape(&s.name),
                s.count
            )
        })
        .collect();

    let status_chips: String = report
        .surface
        .status_histogram
        .iter()
        .map(|s| {
            format!(
                "<span class=\"chip status\">{} ×{}</span>",
                s.status, s.count
            )
        })
        .collect();

    let module_rows: String = report
        .module_results
        .iter()
        .map(|m| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(&m.id),
                if m.ran { "yes" } else { "no" },
                m.findings
            )
        })
        .collect();

    let route_rows: String = if !report.routes.is_empty() {
        report
            .routes
            .iter()
            .take(500)
            .map(|r| {
                format!(
                    "<tr><td><a href=\"{u}\">{u}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    escape(&r.source),
                    r.status
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "—".into()),
                    escape(r.content_type.as_deref().unwrap_or("—")),
                    u = escape(&r.url),
                )
            })
            .collect()
    } else {
        report
            .findings
            .iter()
            .filter(|f| f.id == "route-discovered")
            .take(500)
            .map(|f| {
                format!(
                    "<tr><td><a href=\"{u}\">{u}</a></td><td colspan=\"3\">{}</td></tr>",
                    escape(&f.description),
                    u = escape(&f.url),
                )
            })
            .collect()
    };

    let tech_list: String = report
        .tech_stack
        .iter()
        .map(|t| format!("<li>{}</li>", escape(t)))
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>weeping-angel report — {target}</title>
<style>
:root {{
  --bg: #0b1016; --panel: #121a24; --border: #243040; --text: #e7ecf1; --muted: #9aa7b5;
  --crit: #ff6b6b; --high: #ff9f43; --med: #feca57; --low: #54a0ff; --info: #9aa7b5; --accent: #7cb7ff;
}}
* {{ box-sizing: border-box; }}
body {{ font-family: ui-sans-serif, system-ui, sans-serif; margin: 0; background: var(--bg); color: var(--text); line-height: 1.45; }}
header {{ position: sticky; top: 0; z-index: 10; background: rgba(11,16,22,.92); backdrop-filter: blur(8px); border-bottom: 1px solid var(--border); padding: 1rem 1.5rem; }}
header h1 {{ margin: 0 0 .35rem; font-size: 1.25rem; font-weight: 650; }}
.meta {{ color: var(--muted); font-size: .9rem; display: flex; flex-wrap: wrap; gap: .75rem 1.25rem; }}
.badges {{ display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .65rem; }}
.badge {{ display: inline-block; padding: .15rem .5rem; border-radius: 4px; font-size: .75rem; font-weight: 700; text-transform: uppercase; }}
.badge.critical, .sev-critical .badge {{ background: var(--crit); color: #1a0505; }}
.badge.high {{ background: #ff9f43; color: #1a1005; }}
.badge.medium {{ background: #feca57; color: #1a1505; }}
.badge.low {{ background: #54a0ff; color: #05101a; }}
.badge.info {{ background: #2a3544; color: var(--muted); }}
main {{ padding: 1.25rem 1.5rem 3rem; max-width: 1400px; margin: 0 auto; }}
section {{ background: var(--panel); border: 1px solid var(--border); border-radius: 10px; padding: 1rem 1.1rem; margin-bottom: 1rem; }}
section h2 {{ margin: 0 0 .75rem; font-size: 1rem; color: var(--accent); font-weight: 600; }}
table {{ border-collapse: collapse; width: 100%; font-size: .9rem; }}
th, td {{ border-bottom: 1px solid var(--border); padding: .55rem .45rem; text-align: left; vertical-align: top; }}
th {{ color: var(--muted); font-size: .8rem; font-weight: 600; }}
a {{ color: var(--accent); word-break: break-all; }}
.desc {{ color: var(--muted); margin-top: .25rem; font-size: .85rem; }}
.rem {{ margin-top: .35rem; color: #8fd19e; font-size: .85rem; }}
.ev {{ margin-top: .25rem; font-size: .8rem; color: var(--muted); }}
.ev code {{ color: #feca57; }}
.cwe {{ color: #feca57; font-size: .8rem; }}
.fid {{ color: var(--muted); font-size: .75rem; margin-right: .35rem; }}
.chip {{ display: inline-block; background: #1a2433; border: 1px solid var(--border); border-radius: 999px; padding: .15rem .55rem; margin: .15rem; font-size: .8rem; }}
.chip b {{ color: var(--accent); }}
.filters {{ display: flex; flex-wrap: wrap; gap: .4rem; margin-bottom: .75rem; }}
.filters button {{ background: #1a2433; border: 1px solid var(--border); color: var(--text); border-radius: 6px; padding: .3rem .65rem; cursor: pointer; font-size: .8rem; }}
.filters button.active, .filters button:hover {{ border-color: var(--accent); color: var(--accent); }}
.grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }}
@media (max-width: 900px) {{ .grid {{ grid-template-columns: 1fr; }} }}
ul.tech {{ margin: 0; padding-left: 1.1rem; color: var(--muted); }}
.hidden {{ display: none !important; }}
footer {{ color: var(--muted); font-size: .8rem; padding: 1rem 1.5rem 2rem; text-align: center; }}
</style>
</head>
<body>
<header>
  <h1>weeping-angel <span style="font-weight:400;color:var(--muted);font-size:.85rem">v{version}</span></h1>
  <div class="meta">
    <span><strong>Target</strong> {target}</span>
    <span><strong>Profile</strong> {profile}</span>
    <span><strong>Duration</strong> {wall:.1}s</span>
    <span><strong>Requests</strong> {requests}</span>
    <span><strong>URLs</strong> {urls}</span>
    <span><strong>Findings</strong> {total}</span>
  </div>
  <div class="badges">
    <span class="badge critical">critical {c}</span>
    <span class="badge high">high {h}</span>
    <span class="badge medium">medium {m}</span>
    <span class="badge low">low {l}</span>
    <span class="badge info">info {i}</span>
  </div>
</header>
<main>
<section>
  <h2>Executive summary</h2>
  <p class="desc">{exec_summary}</p>
  <div>{source_chips}</div>
  <div style="margin-top:.5rem">{status_chips}</div>
</section>
<div class="grid">
<section>
  <h2>Phase timings</h2>
  <table><thead><tr><th>Phase</th><th>Time</th><th>Detail</th></tr></thead><tbody>{phase_rows}</tbody></table>
</section>
<section>
  <h2>Modules</h2>
  <table><thead><tr><th>Module</th><th>Ran</th><th>Findings</th></tr></thead><tbody>{module_rows}</tbody></table>
  <h2 style="margin-top:1rem">Tech</h2>
  <ul class="tech">{tech_list}</ul>
</section>
</div>
<section>
  <h2>Findings</h2>
  <div class="filters" id="sev-filters">
    <button type="button" data-sev="all" class="active">all</button>
    <button type="button" data-sev="critical">critical</button>
    <button type="button" data-sev="high">high</button>
    <button type="button" data-sev="medium">medium</button>
    <button type="button" data-sev="low">low</button>
    <button type="button" data-sev="info">info</button>
  </div>
  <table>
    <thead><tr><th>Severity</th><th>Module</th><th>Issue</th><th>URL</th></tr></thead>
    <tbody id="findings-body">{finding_rows}</tbody>
  </table>
</section>
<section>
  <h2>Routes (sample)</h2>
  <table><thead><tr><th>URL</th><th>Source</th><th>Status</th><th>Content-Type</th></tr></thead><tbody>{route_rows}</tbody></table>
</section>
</main>
<footer>weeping-angel v{version} · generated {generated} · {surface_n} surface routes · {mod_n} modules</footer>
<script>
(function(){{
  const buttons = document.querySelectorAll('#sev-filters button');
  const rows = document.querySelectorAll('#findings-body tr');
  buttons.forEach(btn => btn.addEventListener('click', () => {{
    buttons.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    const sev = btn.getAttribute('data-sev');
    rows.forEach(r => {{
      if (sev === 'all' || r.getAttribute('data-sev') === sev) r.classList.remove('hidden');
      else r.classList.add('hidden');
    }});
  }}));
}})();
</script>
</body>
</html>"#,
        target = escape(&report.target),
        profile = escape(&report.profile),
        version = escape(&report.version),
        generated = escape(&report.finished_at.to_rfc3339()),
        wall = report.timing.wall_seconds,
        requests = report.stats.requests,
        urls = report.stats.urls_discovered,
        total = report.stats.findings_total,
        c = report.stats.by_severity.critical,
        h = report.stats.by_severity.high,
        m = report.stats.by_severity.medium,
        l = report.stats.by_severity.low,
        i = report.stats.by_severity.info,
        exec_summary = escape(&executive_summary(report)),
        surface_n = report.surface.total_routes.max(report.routes.len()),
        mod_n = report.module_results.len(),
        source_chips = source_chips,
        status_chips = status_chips,
        phase_rows = phase_rows,
        module_rows = module_rows,
        tech_list = if tech_list.is_empty() {
            "<li class=\"desc\">(none)</li>".into()
        } else {
            tech_list
        },
        finding_rows = finding_rows,
        route_rows = route_rows,
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
