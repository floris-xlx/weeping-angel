//! Deterministic report.md projection from sealed canonical JSON.

use crate::contract::types::{CoverageDocument, FindingsDocument, ManifestDocument};

pub fn project_report_md(
    manifest: &ManifestDocument,
    findings: &FindingsDocument,
    coverage: &CoverageDocument,
) -> String {
    let name = &manifest.scan.target.display_name;
    let mut out = String::new();
    out.push_str(&format!("# Security Review: {name}\n\n"));

    out.push_str("## Scope\n\n");
    if let Some(summary) = &manifest.scan.scope.summary {
        out.push_str(summary);
        out.push_str("\n\n");
    }
    out.push_str("| Field | Value |\n| --- | --- |\n");
    out.push_str(&format!(
        "| Scan ID | `{}` |\n",
        escape_cell(&manifest.scan.id)
    ));
    out.push_str(&format!(
        "| Producer | {} {} |\n",
        escape_cell(&manifest.scan.producer.name),
        escape_cell(&manifest.scan.producer.version)
    ));
    out.push_str(&format!(
        "| Target kind | `{}` |\n",
        escape_cell(&manifest.scan.target.kind)
    ));
    out.push_str(&format!("| Mode | `{}` |\n", escape_cell(&coverage.mode)));
    out.push_str(&format!(
        "| Completeness | `{}` |\n",
        escape_cell(&coverage.completeness)
    ));
    out.push_str(&format!(
        "| Include paths | {} |\n",
        if manifest.scan.scope.include_paths.is_empty() {
            "_(none)_".into()
        } else {
            manifest
                .scan
                .scope
                .include_paths
                .iter()
                .map(|p| format!("`{p}`"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    ));
    out.push_str(&format!(
        "| Reportable findings | {} |\n\n",
        findings.findings.len()
    ));

    if let Some(tm) = &manifest.scan.threat_model {
        out.push_str("## Threat Model\n\n");
        if let Some(s) = &tm.summary {
            out.push_str(s);
            out.push_str("\n\n");
        }
        if let Some(assets) = &tm.assets {
            if !assets.is_empty() {
                out.push_str("**Assets**\n\n");
                for a in assets {
                    out.push_str(&format!("- {a}\n"));
                }
                out.push('\n');
            }
        }
    }

    out.push_str("## Findings\n\n");
    if findings.findings.is_empty() {
        out.push_str("### No findings\n\n");
        out.push_str(
            "No reportable findings survived discovery, validation, and attack-path gates.\n\n",
        );
    } else {
        out.push_str("| Finding | Severity | Confidence |\n| --- | --- | --- |\n");
        for (i, f) in findings.findings.iter().enumerate() {
            out.push_str(&format!(
                "| [{}] {} | {} | {} |\n",
                i + 1,
                escape_cell(&f.title),
                escape_cell(&f.severity.level),
                escape_cell(&f.confidence.level)
            ));
        }
        out.push('\n');

        for (i, f) in findings.findings.iter().enumerate() {
            out.push_str(&format!("### [{}] {}\n\n", i + 1, f.title));
            out.push_str("| Field | Value |\n| --- | --- |\n");
            out.push_str(&format!(
                "| Severity | {} |\n",
                escape_cell(&f.severity.level)
            ));
            out.push_str(&format!(
                "| Confidence | {} |\n",
                escape_cell(&f.confidence.level)
            ));
            out.push_str(&format!(
                "| Confidence rationale | {} |\n",
                escape_cell(&f.confidence.rationale)
            ));
            out.push_str(&format!("| Rule ID | `{}` |\n", escape_cell(&f.rule_id)));
            out.push_str(&format!(
                "| Finding ID | `{}` |\n",
                escape_cell(&f.finding_id)
            ));
            if !f.taxonomy.cwe.is_empty() {
                out.push_str(&format!(
                    "| CWE | {} |\n",
                    f.taxonomy
                        .cwe
                        .iter()
                        .map(|c| format!("`{c}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            out.push('\n');
            out.push_str(&f.summary);
            out.push_str("\n\n");

            out.push_str("**Locations**\n\n");
            for loc in &f.locations {
                let role = loc.role.as_deref().unwrap_or("location");
                let end = loc.end_line.map(|e| format!("-{e}")).unwrap_or_default();
                out.push_str(&format!(
                    "- `{}:{}{}` ({role})\n",
                    loc.path, loc.start_line, end
                ));
            }
            out.push('\n');

            out.push_str("**Remediation**\n\n");
            out.push_str(&f.remediation);
            out.push_str("\n\n");
        }
    }

    out.push_str("## Reviewed Surfaces\n\n");
    if coverage.surfaces.is_empty() {
        out.push_str("_(no surfaces recorded)_\n\n");
    } else {
        out.push_str("| Surface | Disposition | Notes |\n| --- | --- | --- |\n");
        for s in &coverage.surfaces {
            out.push_str(&format!(
                "| {} | `{}` | {} |\n",
                escape_cell(&s.label),
                escape_cell(&s.disposition),
                escape_cell(s.notes.as_deref().unwrap_or(""))
            ));
        }
        out.push('\n');
    }

    if !coverage.open_questions.is_empty() {
        out.push_str("## Open Questions And Follow Up\n\n");
        for q in &coverage.open_questions {
            out.push_str(&format!("- {}\n", q.question));
        }
        out.push('\n');
    }

    out.push_str("---\n\n");
    out.push_str("*Generated deterministically by weeping-angel from sealed canonical JSON.*\n");
    out
}

fn escape_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}
