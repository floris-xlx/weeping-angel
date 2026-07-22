use anyhow::Result;
use async_trait::async_trait;
use url::Url;
use uuid::Uuid;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct XssReflectProbe;

#[async_trait]
impl Check for XssReflectProbe {
    fn id(&self) -> &'static str {
        "xss"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Active
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let canary = format!("waXSS{}", Uuid::new_v4().simple());
        let payload = format!("\"'><{canary}>");

        let candidates = param_urls(ctx, 25);
        for mut url in candidates {
            // inject into each query param
            let pairs: Vec<(String, String)> = url
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            if pairs.is_empty() {
                // add a probe param
                url.query_pairs_mut().append_pair("q", &payload);
            } else {
                url.query_pairs_mut().clear();
                for (k, _) in &pairs {
                    url.query_pairs_mut().append_pair(k, &payload);
                }
            }

            let resp = match ctx.client.get(&url).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            let escaped_payload = html_escape(&payload);
            let escaped_canary = html_escape(&canary);
            let raw_payload = resp.body.contains(&payload);
            let raw_canary = resp.body.contains(&canary);
            let only_escaped = !raw_payload
                && !raw_canary
                && (resp.body.contains(&escaped_payload) || resp.body.contains(&escaped_canary));

            if raw_payload || (raw_canary && resp.body.contains(&format!("<{canary}"))) {
                findings.push(
                    Finding::builder(self.id(), "reflected-xss")
                        .title("Possible reflected XSS (canary reflected unescaped)")
                        .severity(Severity::High)
                        .url(url.as_str())
                        .description(
                            "Active probe payload/canary was reflected without HTML encoding.",
                        )
                        .remediation(
                            "Context-encode all user input in HTML/JS/attributes; deploy CSP.",
                        )
                        .cwe("CWE-79")
                        .evidence(Evidence::new("body", format!("canary {canary} reflected raw")))
                        .build(),
                );
            } else if raw_canary && !only_escaped {
                // canary text reflected (may still be XSS depending on context)
                findings.push(
                    Finding::builder(self.id(), "reflected-input")
                        .title("User input reflected in response")
                        .severity(Severity::Medium)
                        .url(url.as_str())
                        .description(
                            "Probe canary was reflected in the response. Verify encoding by context.",
                        )
                        .cwe("CWE-79")
                        .evidence(Evidence::new("body", format!("canary {canary} reflected")))
                        .build(),
                );
            }
        }

        Ok(findings)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn param_urls(ctx: &ScanContext, limit: usize) -> Vec<Url> {
    let mut out = Vec::new();
    for u in &ctx.discovered_urls {
        if let Ok(parsed) = Url::parse(u) {
            if parsed.query().is_some() {
                out.push(parsed);
            }
        }
        if out.len() >= limit {
            break;
        }
    }
    if out.is_empty() {
        out.push(ctx.seed.clone());
    }
    out.truncate(limit);
    out
}
