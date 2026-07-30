use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct HeadersCheck;

const RECOMMENDED: &[(&str, Severity, &str)] = &[
    (
        "content-security-policy",
        Severity::Medium,
        "Missing Content-Security-Policy helps mitigate XSS and data injection.",
    ),
    (
        "strict-transport-security",
        Severity::Medium,
        "Missing HSTS allows SSL stripping on HTTPS sites.",
    ),
    (
        "x-content-type-options",
        Severity::Low,
        "Missing X-Content-Type-Options (nosniff) enables MIME sniffing attacks.",
    ),
    (
        "x-frame-options",
        Severity::Low,
        "Missing X-Frame-Options (or CSP frame-ancestors) increases clickjacking risk.",
    ),
    (
        "referrer-policy",
        Severity::Info,
        "Missing Referrer-Policy may leak sensitive URL paths to third parties.",
    ),
    (
        "permissions-policy",
        Severity::Info,
        "Missing Permissions-Policy leaves browser features unrestricted.",
    ),
];

#[async_trait]
impl Check for HeadersCheck {
    fn id(&self) -> &'static str {
        "headers"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        // Evaluate on seed / primary HTML responses
        let samples: Vec<_> = ctx
            .responses
            .values()
            .filter(|r| r.status.is_success() && (r.is_html() || r.url.path() == "/" || r.url == ctx.seed))
            .take(5)
            .collect();

        let responses = if samples.is_empty() {
            ctx.responses.values().take(3).collect::<Vec<_>>()
        } else {
            samples
        };

        for resp in responses {
            for (header, severity, desc) in RECOMMENDED {
                if resp.header(header).is_none() {
                    // HSTS only meaningful for https
                    if *header == "strict-transport-security" && resp.final_url.scheme() != "https"
                    {
                        continue;
                    }
                    // CSP: also accept report-only as partial
                    if *header == "content-security-policy"
                        && resp.header("content-security-policy-report-only").is_some()
                    {
                        findings.push(
                            Finding::builder(self.id(), "csp-report-only")
                                .title("CSP is report-only")
                                .severity(Severity::Low)
                                .url(resp.final_url.as_str())
                                .description(
                                    "Content-Security-Policy-Report-Only is present but not enforced.",
                                )
                                .remediation("Deploy an enforcing Content-Security-Policy.")
                                .cwe("CWE-693")
                                .build(),
                        );
                        continue;
                    }
                    findings.push(
                        Finding::builder(self.id(), format!("missing-{header}"))
                            .title(format!("Missing security header: {header}"))
                            .severity(*severity)
                            .url(resp.final_url.as_str())
                            .description(*desc)
                            .remediation(format!("Add a strong {header} response header."))
                            .cwe("CWE-693")
                            .evidence(Evidence::new(
                                "response headers",
                                format!("header `{header}` not present"),
                            ))
                            .build(),
                    );
                }
            }

            if let Some(csp) = resp.header("content-security-policy") {
                if csp.contains("unsafe-inline") || csp.contains("unsafe-eval") {
                    findings.push(
                        Finding::builder(self.id(), "csp-unsafe")
                            .title("CSP allows unsafe-inline or unsafe-eval")
                            .severity(Severity::Medium)
                            .url(resp.final_url.as_str())
                            .description(
                                "CSP contains unsafe-inline and/or unsafe-eval, weakening XSS defenses.",
                            )
                            .remediation("Remove unsafe-inline/unsafe-eval; use nonces or hashes.")
                            .cwe("CWE-693")
                            .evidence(Evidence::new("content-security-policy", csp))
                            .build(),
                    );
                }
            }

            if let Some(xcto) = resp.header("x-content-type-options") {
                if !xcto.eq_ignore_ascii_case("nosniff") {
                    findings.push(
                        Finding::builder(self.id(), "xcto-invalid")
                            .title("X-Content-Type-Options is not nosniff")
                            .severity(Severity::Low)
                            .url(resp.final_url.as_str())
                            .description(format!("Value is `{xcto}`, expected `nosniff`."))
                            .build(),
                    );
                }
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checks::test_util::{context_with_responses, snapshot};
    use std::collections::HashMap;

    #[tokio::test]
    async fn flags_missing_security_headers_on_html() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[("content-type", "text/html")],
                "<html><body>hi</body></html>",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = HeadersCheck.run(&ctx).await.unwrap();
        let ids: Vec<_> = findings.iter().map(|f| f.id.as_str()).collect();
        assert!(
            ids.iter().any(|id| id.contains("content-security-policy")),
            "ids={ids:?}"
        );
        assert!(
            ids.iter().any(|id| id.contains("x-content-type-options")),
            "ids={ids:?}"
        );
        // HSTS skipped for non-https — seed is https so HSTS should fire
        assert!(
            ids.iter().any(|id| id.contains("strict-transport-security")),
            "ids={ids:?}"
        );
    }

    #[tokio::test]
    async fn flags_unsafe_csp() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[
                    ("content-type", "text/html"),
                    (
                        "content-security-policy",
                        "default-src 'self' 'unsafe-inline'",
                    ),
                    ("x-content-type-options", "nosniff"),
                    ("strict-transport-security", "max-age=31536000"),
                    ("x-frame-options", "DENY"),
                    ("referrer-policy", "no-referrer"),
                    ("permissions-policy", "geolocation=()"),
                ],
                "<html></html>",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = HeadersCheck.run(&ctx).await.unwrap();
        assert!(
            findings.iter().any(|f| f.id == "csp-unsafe"),
            "findings={:?}",
            findings.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn flags_invalid_xcto() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[
                    ("content-type", "text/html"),
                    ("x-content-type-options", "sniff"),
                    ("content-security-policy", "default-src 'self'"),
                    ("strict-transport-security", "max-age=1"),
                    ("x-frame-options", "DENY"),
                    ("referrer-policy", "no-referrer"),
                    ("permissions-policy", "geolocation=()"),
                ],
                "<html></html>",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = HeadersCheck.run(&ctx).await.unwrap();
        assert!(findings.iter().any(|f| f.id == "xcto-invalid"));
    }
}
