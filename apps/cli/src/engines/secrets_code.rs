//! Hardcoded secrets / credentials in source trees.

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
            id: "secrets.aws-access-key",
            anchor: "aws-access-key-id-literal",
            title: "Possible AWS Access Key ID in source",
            severity: "high",
            re: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            remediation: "Rotate the key; load credentials from a secret store or environment, not source.",
        },
        Rule {
            id: "secrets.github-pat",
            anchor: "github-pat-literal",
            title: "Possible GitHub personal access token in source",
            severity: "critical",
            re: Regex::new(r"\bghp_[A-Za-z0-9]{36,}\b").unwrap(),
            remediation: "Revoke the token and store replacements outside the repository.",
        },
        Rule {
            id: "secrets.github-fine-grained",
            anchor: "github-fine-grained-pat",
            title: "Possible GitHub fine-grained PAT in source",
            severity: "critical",
            re: Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b").unwrap(),
            remediation: "Revoke the token; never commit PATs.",
        },
        Rule {
            id: "secrets.stripe-live",
            anchor: "stripe-live-secret",
            title: "Possible Stripe live secret key in source",
            severity: "critical",
            re: Regex::new(r"\bsk_live_[A-Za-z0-9]{16,}\b").unwrap(),
            remediation: "Rotate the Stripe secret; inject via secrets manager.",
        },
        Rule {
            id: "secrets.private-key-pem",
            anchor: "pem-private-key-block",
            title: "PEM private key material in source",
            severity: "critical",
            re: Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----").unwrap(),
            remediation: "Remove the key from git history if committed; use a secrets vault.",
        },
        Rule {
            id: "secrets.generic-api-key-assign",
            anchor: "api-key-assignment-literal",
            title: "Hardcoded API key / secret assignment",
            severity: "high",
            re: Regex::new(
                r#"(?i)(api[_-]?key|apikey|secret[_-]?key|access[_-]?token|auth[_-]?token)\s*[:=]\s*['"][A-Za-z0-9_\-]{16,}['"]"#,
            )
            .unwrap(),
            remediation: "Move secrets to environment or a vault; keep only non-secret config in source.",
        },
        Rule {
            id: "secrets.slack-token",
            anchor: "slack-token-literal",
            title: "Possible Slack token in source",
            severity: "high",
            re: Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b").unwrap(),
            remediation: "Revoke the Slack token and load from secrets at runtime.",
        },
    ]
});

pub fn scan(rel_path: &str, content: &str) -> Vec<EngineHit> {
    // Skip obvious lock/vendor noise for generic patterns; still catch PEMs/tokens.
    let lower = rel_path.replace('\\', "/").to_ascii_lowercase();
    let skip_generic = lower.contains("/vendor/")
        || lower.contains("/node_modules/")
        || lower.ends_with("package-lock.json")
        || lower.ends_with("cargo.lock")
        || lower.ends_with("yarn.lock")
        || lower.ends_with("pnpm-lock.yaml");

    let mut hits = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let line_no = (line_no + 1) as u32;
        for rule in RULES.iter() {
            if skip_generic && rule.id == "secrets.generic-api-key-assign" {
                continue;
            }
            if let Some(m) = rule.re.find(line) {
                let snippet = redact_line(line.trim());
                hits.push(EngineHit {
                    rule_id: rule.id.into(),
                    anchor: rule.anchor.into(),
                    instance: Some(format!("{}-l{}", slug_path(rel_path), line_no)),
                    title: rule.title.into(),
                    summary: format!("{} at `{}:{}`.", rule.title, rel_path, line_no),
                    evidence: format!(
                        "Matched `{}` on line {line_no} (secret value redacted in report).",
                        rule.id
                    ),
                    severity: rule.severity,
                    confidence: if rule.id.contains("generic") {
                        "low"
                    } else {
                        "high"
                    },
                    confidence_rationale: if rule.id.contains("generic") {
                        "Generic assignment pattern; may be a placeholder or public test key."
                            .into()
                    } else {
                        "High-signal secret format match in source.".into()
                    },
                    category: "secrets".into(),
                    cwe: vec!["CWE-798".into()],
                    remediation: rule.remediation.into(),
                    path: rel_path.replace('\\', "/"),
                    start_line: line_no,
                    end_line: Some(line_no),
                    role: "sink",
                    snippet,
                    validation_json: None,
                    attack_path_json: None,
                });
                let _ = m; // silence
            }
        }
    }
    hits
}

fn redact_line(line: &str) -> String {
    let mut out: String = line.chars().take(200).collect();
    // crude redact of long tokens
    if out.len() > 40 {
        let mid = out.len() / 2;
        out.replace_range(mid.saturating_sub(8)..mid.saturating_add(8), "********");
    }
    out
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
    fn detects_github_pat() {
        let src = "const t = \"ghp_abcdefghijklmnopqrstuvwxyz0123456789\";\n";
        let hits = scan("cfg.ts", src);
        assert!(hits.iter().any(|h| h.rule_id.contains("github-pat")));
    }
}
