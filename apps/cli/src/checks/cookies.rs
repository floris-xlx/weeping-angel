use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct CookiesCheck;

#[async_trait]
impl Check for CookiesCheck {
    fn id(&self) -> &'static str {
        "cookies"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for resp in ctx.responses.values() {
            for set_cookie in resp.set_cookies() {
                for line in set_cookie.split('\n') {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let name = line.split('=').next().unwrap_or("cookie");
                    let lower = line.to_ascii_lowercase();
                    let is_session_like = name.to_ascii_lowercase().contains("session")
                        || name.to_ascii_lowercase().contains("sid")
                        || name.to_ascii_lowercase().contains("token")
                        || name.to_ascii_lowercase().contains("auth")
                        || name.eq_ignore_ascii_case("jwt");

                    if !lower.contains("httponly") && is_session_like {
                        findings.push(
                            Finding::builder(self.id(), "cookie-missing-httponly")
                                .title(format!("Cookie `{name}` missing HttpOnly"))
                                .severity(Severity::Medium)
                                .url(resp.final_url.as_str())
                                .description(
                                    "Session-like cookie without HttpOnly is readable by JavaScript (XSS impact).",
                                )
                                .remediation("Set the HttpOnly flag on session cookies.")
                                .cwe("CWE-1004")
                                .evidence(Evidence::new("set-cookie", line))
                                .build(),
                        );
                    }

                    if resp.final_url.scheme() == "https" && !lower.contains("secure") {
                        findings.push(
                            Finding::builder(self.id(), "cookie-missing-secure")
                                .title(format!("Cookie `{name}` missing Secure on HTTPS"))
                                .severity(if is_session_like {
                                    Severity::Medium
                                } else {
                                    Severity::Low
                                })
                                .url(resp.final_url.as_str())
                                .description(
                                    "Cookie without Secure may be sent over HTTP if the user is downgraded.",
                                )
                                .remediation("Set the Secure flag on cookies used over HTTPS.")
                                .cwe("CWE-614")
                                .evidence(Evidence::new("set-cookie", line))
                                .build(),
                        );
                    }

                    if !lower.contains("samesite") && is_session_like {
                        findings.push(
                            Finding::builder(self.id(), "cookie-missing-samesite")
                                .title(format!("Cookie `{name}` missing SameSite"))
                                .severity(Severity::Low)
                                .url(resp.final_url.as_str())
                                .description(
                                    "Missing SameSite increases CSRF risk for session cookies.",
                                )
                                .remediation("Set SameSite=Lax or Strict as appropriate.")
                                .cwe("CWE-1275")
                                .evidence(Evidence::new("set-cookie", line))
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

    #[tokio::test]
    async fn flags_session_cookie_missing_flags_on_https() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[
                    ("content-type", "text/html"),
                    ("set-cookie", "session=abc; Path=/"),
                ],
                "<html></html>",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = CookiesCheck.run(&ctx).await.unwrap();
        let ids: Vec<_> = findings.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"cookie-missing-httponly"), "ids={ids:?}");
        assert!(ids.contains(&"cookie-missing-secure"), "ids={ids:?}");
        assert!(ids.contains(&"cookie-missing-samesite"), "ids={ids:?}");
    }

    #[tokio::test]
    async fn accepts_hardened_session_cookie() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/".into(),
            snapshot(
                "https://example.com/",
                200,
                &[
                    ("content-type", "text/html"),
                    (
                        "set-cookie",
                        "session=abc; Path=/; HttpOnly; Secure; SameSite=Lax",
                    ),
                ],
                "<html></html>",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = CookiesCheck.run(&ctx).await.unwrap();
        assert!(
            findings.is_empty(),
            "unexpected={:?}",
            findings.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
    }
}
