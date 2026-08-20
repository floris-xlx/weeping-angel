//! XSS / unsafe HTML template rendering patterns.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::engines::EngineHit;

struct Rule {
    id: &'static str,
    anchor: &'static str,
    title: &'static str,
    severity: &'static str,
    re: Regex,
    remediation: &'static str,
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        Rule {
            id: "xss.innerhtml-user",
            anchor: "innerhtml-assignment",
            title: "innerHTML assignment may enable XSS",
            severity: "high",
            re: Regex::new(
                r#"(?i)\.innerHTML\s*=\s*[^;]*(req\.|request\.|params\.|query\.|user|input|location\.|searchParams|document\.cookie)"#,
            )
            .unwrap(),
            remediation: "Use textContent or a safe sanitizer; never assign untrusted HTML to innerHTML.",
        },
        Rule {
            id: "xss.dangerously-set-html",
            anchor: "react-dangerously-set-innerhtml",
            title: "React dangerouslySetInnerHTML with dynamic content",
            severity: "high",
            re: Regex::new(r#"(?i)dangerouslySetInnerHTML\s*=\s*\{\s*\{\s*__html\s*:"#).unwrap(),
            remediation: "Sanitize HTML with a trusted library or avoid raw HTML entirely.",
        },
        Rule {
            id: "xss.jinja-safe",
            anchor: "jinja-safe-or-markup",
            title: "Template marks content as safe without clear sanitization",
            severity: "medium",
            re: Regex::new(r#"(?i)(\|\s*safe\b|Markup\s*\(|raw\s*\()"#).unwrap(),
            remediation: "Do not mark untrusted data as safe; escape by default.",
        },
        Rule {
            id: "xss.document-write",
            anchor: "document-write",
            title: "document.write may inject script",
            severity: "medium",
            re: Regex::new(r#"(?i)document\.write\s*\("#).unwrap(),
            remediation: "Avoid document.write; use DOM APIs with safe text nodes.",
        },
        Rule {
            id: "xss.v-html",
            anchor: "vue-v-html",
            title: "Vue v-html renders raw HTML",
            severity: "high",
            re: Regex::new(r#"v-html\s*="#).unwrap(),
            remediation: "Prefer text interpolation; sanitize if HTML is required.",
        },
    ]
});

pub fn scan(rel_path: &str, content: &str) -> Vec<EngineHit> {
    let mut hits = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let line_no = (line_no + 1) as u32;
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        for rule in RULES.iter() {
            if let Some(m) = rule.re.find(line) {
                hits.push(EngineHit {
                    rule_id: rule.id.into(),
                    anchor: rule.anchor.into(),
                    instance: Some(format!("{}-l{}", slug_path(rel_path), line_no)),
                    title: rule.title.into(),
                    summary: format!("{} at `{}:{}`.", rule.title, rel_path, line_no),
                    evidence: format!(
                        "Matched `{}` on line {line_no}: `{}`",
                        rule.id,
                        &line[m.start()..m.end().min(line.len())]
                    ),
                    severity: rule.severity,
                    confidence: "medium",
                    confidence_rationale:
                        "Unsafe HTML/template sink pattern; attacker control and sanitizers not fully proven."
                            .into(),
                    category: "xss".into(),
                    cwe: vec!["CWE-79".into()],
                    remediation: rule.remediation.into(),
                    path: rel_path.replace('\\', "/"),
                    start_line: line_no,
                    end_line: Some(line_no),
                    role: "sink",
                    snippet: line.trim().chars().take(240).collect(),
        validation_json: None,
        attack_path_json: None,
                });
            }
        }
    }
    hits
}

fn slug_path(path: &str) -> String {
    path.replace('\\', "/")
        .replace(['/', '.', ' '], "-")
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_innerhtml() {
        let src = "el.innerHTML = request.query.q;\n";
        let hits = scan("ui.js", src);
        assert!(!hits.is_empty());
    }
}
