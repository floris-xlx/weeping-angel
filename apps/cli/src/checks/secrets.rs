use anyhow::Result;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct SecretsCheck;

struct Pattern {
    id: &'static str,
    title: &'static str,
    severity: Severity,
    re: Regex,
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    vec![
        Pattern {
            id: "aws-access-key",
            title: "Possible AWS Access Key ID",
            severity: Severity::High,
            re: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
        },
        Pattern {
            id: "aws-secret-assign",
            title: "Possible AWS secret access key assignment",
            severity: Severity::Critical,
            re: Regex::new(
                r#"(?i)(aws_secret_access_key|secret_access_key)\s*[:=]\s*['"]([A-Za-z0-9/+=]{30,})['"]"#,
            )
            .unwrap(),
        },
        Pattern {
            id: "github-pat",
            title: "Possible GitHub personal access token",
            severity: Severity::Critical,
            re: Regex::new(r"\bghp_[A-Za-z0-9]{36,}\b").unwrap(),
        },
        Pattern {
            id: "github-oauth",
            title: "Possible GitHub OAuth token",
            severity: Severity::Critical,
            re: Regex::new(r"\bgho_[A-Za-z0-9]{36,}\b").unwrap(),
        },
        Pattern {
            id: "github-fine-grained",
            title: "Possible GitHub fine-grained PAT",
            severity: Severity::Critical,
            re: Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b").unwrap(),
        },
        Pattern {
            id: "gitlab-pat",
            title: "Possible GitLab personal access token",
            severity: Severity::Critical,
            re: Regex::new(r"\bglpat-[A-Za-z0-9\-_]{20,}\b").unwrap(),
        },
        Pattern {
            id: "slack-token",
            title: "Possible Slack token",
            severity: Severity::High,
            re: Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b").unwrap(),
        },
        Pattern {
            id: "slack-webhook",
            title: "Possible Slack webhook URL",
            severity: Severity::High,
            re: Regex::new(r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+")
                .unwrap(),
        },
        Pattern {
            id: "stripe-live",
            title: "Possible Stripe live secret key",
            severity: Severity::Critical,
            re: Regex::new(r"\bsk_live_[A-Za-z0-9]{16,}\b").unwrap(),
        },
        Pattern {
            id: "stripe-test",
            title: "Possible Stripe test secret key",
            severity: Severity::Medium,
            re: Regex::new(r"\bsk_test_[A-Za-z0-9]{16,}\b").unwrap(),
        },
        Pattern {
            id: "google-api-key",
            title: "Possible Google API key",
            severity: Severity::High,
            re: Regex::new(r"\bAIza[0-9A-Za-z\-_]{35}\b").unwrap(),
        },
        Pattern {
            id: "sendgrid",
            title: "Possible SendGrid API key",
            severity: Severity::High,
            re: Regex::new(r"\bSG\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}\b").unwrap(),
        },
        Pattern {
            id: "mailgun",
            title: "Possible Mailgun API key",
            severity: Severity::High,
            re: Regex::new(r"\bkey-[0-9a-zA-Z]{32}\b").unwrap(),
        },
        Pattern {
            id: "twilio-sid",
            title: "Possible Twilio Account SID",
            severity: Severity::Medium,
            re: Regex::new(r"\bAC[0-9a-fA-F]{32}\b").unwrap(),
        },
        Pattern {
            id: "npm-token",
            title: "Possible npm access token",
            severity: Severity::High,
            re: Regex::new(r"\bnpm_[A-Za-z0-9]{36,}\b").unwrap(),
        },
        Pattern {
            id: "pypi-token",
            title: "Possible PyPI token",
            severity: Severity::High,
            re: Regex::new(r"\bpypi-AgEIcHlwaS5vcmc[A-Za-z0-9\-_]{20,}\b").unwrap(),
        },
        Pattern {
            id: "private-key",
            title: "PEM private key material",
            severity: Severity::Critical,
            re: Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----").unwrap(),
        },
        Pattern {
            id: "jwt",
            title: "JWT-like token",
            severity: Severity::Medium,
            re: Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
                .unwrap(),
        },
        Pattern {
            id: "connection-string",
            title: "Possible database connection string",
            severity: Severity::High,
            re: Regex::new(
                r#"(?i)\b(postgres|mysql|mongodb|redis|amqp)://[^\s"'<>]{8,}"#,
            )
            .unwrap(),
        },
        Pattern {
            id: "generic-api-key-assign",
            title: "Possible API key assignment",
            severity: Severity::Medium,
            re: Regex::new(
                r#"(?i)(api[_-]?key|apikey|secret[_-]?key|auth[_-]?token|access[_-]?token)\s*[:=]\s*['"]([A-Za-z0-9_\-]{16,})['"]"#,
            )
            .unwrap(),
        },
        Pattern {
            id: "bearer-hardcoded",
            title: "Hardcoded Bearer token",
            // Require a long token-shaped payload so prose like "Bearer sessions"
            // in OpenAPI docs does not false-positive as a live secret.
            severity: Severity::High,
            re: Regex::new(r#"(?i)\bbearer\s+[A-Za-z0-9\-._~+/]{20,}=*"#).unwrap(),
        },
    ]
});

#[async_trait]
impl Check for SecretsCheck {
    fn id(&self) -> &'static str {
        "secrets"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for resp in ctx.responses.values() {
            // headers
            for (k, v) in &resp.headers {
                scan_text(
                    &mut findings,
                    self.id(),
                    resp.final_url.as_str(),
                    &format!("header:{k}"),
                    v,
                );
            }
            scan_text(
                &mut findings,
                self.id(),
                resp.final_url.as_str(),
                "body",
                &resp.body,
            );
        }

        Ok(findings)
    }
}

fn scan_text(findings: &mut Vec<Finding>, module: &str, url: &str, location: &str, text: &str) {
    for pat in PATTERNS.iter() {
        for mat in pat.re.find_iter(text) {
            let snippet = redact(mat.as_str());
            findings.push(
                Finding::builder(module, pat.id)
                    .title(pat.title)
                    .severity(pat.severity)
                    .url(url)
                    .description(format!(
                        "Pattern `{}` matched in {location}. Verify and rotate if real.",
                        pat.id
                    ))
                    .remediation(
                        "Remove secrets from client-accessible responses; rotate exposed credentials.",
                    )
                    .cwe("CWE-798")
                    .evidence(Evidence::new(location, snippet))
                    .build(),
            );
        }
    }
}

fn redact(s: &str) -> String {
    if s.len() <= 8 {
        return "***".into();
    }
    let keep = 4.min(s.len() / 4);
    format!("{}…{}", &s[..keep], &s[s.len() - keep..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_aws_key() {
        let mut findings = Vec::new();
        scan_text(
            &mut findings,
            "secrets",
            "https://example.com",
            "body",
            "key=AKIAIOSFODNN7EXAMPLE",
        );
        assert!(findings.iter().any(|f| f.id == "aws-access-key"));
    }

    #[test]
    fn bearer_prose_is_not_a_finding() {
        let mut findings = Vec::new();
        scan_text(
            &mut findings,
            "secrets",
            "https://example.com/openapi.yaml",
            "body",
            "Authorization: Bearer sessions and admin bearer sessions are supported.",
        );
        assert!(
            findings.iter().all(|f| f.id != "bearer-hardcoded"),
            "OpenAPI prose must not trip bearer-hardcoded"
        );
    }

    #[test]
    fn bearer_token_shaped_value_is_a_finding() {
        let mut findings = Vec::new();
        scan_text(
            &mut findings,
            "secrets",
            "https://example.com",
            "body",
            "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.abc",
        );
        assert!(findings.iter().any(|f| f.id == "bearer-hardcoded"));
    }
}
