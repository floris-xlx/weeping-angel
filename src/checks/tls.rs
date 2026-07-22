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
            if let Ok(resp) = ctx.client.get(&https).await {
                if resp.status.is_success() {
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
