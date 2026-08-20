use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct TlsCheck;

#[async_trait]
impl Check for TlsCheck {
    fn id(&self) -> &'static str {
        "tls"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        if ctx.seed.scheme() == "http" {
            findings.push(
                Finding::builder(self.id(), "http-only-seed")
                    .title("Target uses HTTP (no TLS on seed URL)")
                    .severity(Severity::Medium)
                    .url(ctx.seed.as_str())
                    .description(
                        "The scan seed is plain HTTP. Credentials and cookies may be sent in cleartext.",
                    )
                    .remediation("Serve the application exclusively over HTTPS with HSTS.")
                    .cwe("CWE-319")
                    .build(),
            );

            // check if HTTPS variant works
            let mut https = ctx.seed.clone();
            let _ = https.set_scheme("https");
            if let Ok(resp) = ctx.client.get(&https).await
                && resp.status.is_success()
            {
                findings.push(
                        Finding::builder(self.id(), "https-available")
                            .title("HTTPS is available; prefer it as primary")
                            .severity(Severity::Low)
                            .url(https.as_str())
                            .description(
                                "HTTPS responded successfully while seed was HTTP. Redirect users to HTTPS.",
                            )
                            .build(),
                    );
            }
        }

        // Cleartext cookie over http already covered in cookies; flag mixed content-ish absolute http resources in https pages
        if ctx.seed.scheme() == "https" {
            for resp in ctx.responses.values().filter(|r| r.is_html()) {
                if resp.body.contains("http://") {
                    let count = resp.body.matches("http://").count();
                    if count > 0 {
                        findings.push(
                            Finding::builder(self.id(), "mixed-content-hint")
                                .title("Possible mixed content (http:// references on HTTPS page)")
                                .severity(Severity::Low)
                                .url(resp.final_url.as_str())
                                .description(format!(
                                    "Found {count} occurrence(s) of http:// in an HTTPS HTML response."
                                ))
                                .remediation("Use HTTPS or protocol-relative URLs for all subresources.")
                                .cwe("CWE-311")
                                .evidence(Evidence::new("html body", "http:// references present"))
                                .build(),
                        );
                    }
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
    use url::Url;

    #[tokio::test]
    async fn http_seed_flags_no_tls() {
        let mut responses = HashMap::new();
        responses.insert(
            "http://example.com/".into(),
            snapshot(
                "http://example.com/",
                200,
                &[("content-type", "text/html")],
                "<html></html>",
            ),
        );
        let mut ctx = context_with_responses(responses);
        ctx.seed = Url::parse("http://example.com/").unwrap();
        let findings = TlsCheck.run(&ctx).await.unwrap();
        assert!(findings.iter().any(|f| f.id == "http-only-seed"));
    }

    #[tokio::test]
    async fn https_page_with_http_refs_mixed_content() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[("content-type", "text/html")],
                r#"<html><img src="http://cdn.example/a.png"></html>"#,
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = TlsCheck.run(&ctx).await.unwrap();
        assert!(findings.iter().any(|f| f.id == "mixed-content-hint"));
    }
}
