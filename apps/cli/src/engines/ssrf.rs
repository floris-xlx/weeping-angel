//! SSRF / attacker-controlled outbound URL patterns.

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
            id: "ssrf.http-client-user-url",
            anchor: "http-get-user-url",
            title: "HTTP client uses potentially attacker-controlled URL",
            severity: "high",
            re: Regex::new(
                r#"(?i)(requests\.(get|post|put|delete|request|head)|httpx\.(get|post|request)|urllib\.request\.urlopen|fetch\s*\(|axios\.(get|post|request)|got\s*\(|reqwest::(get|Client)|HttpClient|RestTemplate)\s*[\.(][^;)]*(req\.|request\.|params\.|query\.|body\.|url|target|webhook|callback)"#,
            )
            .unwrap(),
            remediation:
                "Allowlist destinations; block link-local/metadata IPs; do not pass raw user URLs to HTTP clients.",
        },
        Rule {
            id: "ssrf.curl-exec-url",
            anchor: "curl-user-url",
            title: "curl/wget invoked with dynamic URL",
            severity: "high",
            re: Regex::new(
                r#"(?i)(curl|wget)\s+[^;\n]*(req\.|request\.|params\.|\$\{|f["']|format!)"#,
            )
            .unwrap(),
            remediation: "Avoid shelling out to curl with user URLs; use an HTTP library with SSRF controls.",
        },
        Rule {
            id: "ssrf.open-url-redirect",
            anchor: "urlopen-or-redirect-follow",
            title: "URL open / redirect-following client may reach internal hosts",
            severity: "medium",
            re: Regex::new(
                r#"(?i)(urlopen|OpenUri|WebClient\.Download|HttpURLConnection|followRedirects\s*=\s*true).{0,80}(req\.|request\.|user|url)"#,
            )
            .unwrap(),
            remediation: "Disable open redirects on the client; validate scheme/host before connect.",
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
                        "Outbound HTTP/URL sink near user-influenced identifiers; allowlist not proven."
                            .into(),
                    category: "ssrf".into(),
                    cwe: vec!["CWE-918".into()],
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
    fn detects_requests_get() {
        let src = "r = requests.get(request.args['url'])\n";
        let hits = scan("proxy.py", src);
        assert!(!hits.is_empty());
    }
}
