//! Compare unauthenticated vs authenticated responses for the same URLs.
//! Surfaces likely auth bypass / IDOR-style exposure when anon gets 200 on sensitive paths
//! or body content that should be protected.

use anyhow::Result;
use async_trait::async_trait;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct AuthCompareCheck;

const SENSITIVE_HINTS: &[&str] = &[
    "admin", "user", "users", "account", "me", "private", "internal", "dashboard", "manage",
    "settings", "billing", "secret", "token", "config",
];

#[async_trait]
impl Check for AuthCompareCheck {
    fn id(&self) -> &'static str {
        "auth-compare"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        let Some(anon) = &ctx.anon_client else {
            // no comparison requested
            return Ok(findings);
        };

        // Prefer paths that look sensitive + a sample of discovered URLs
        let mut candidates: Vec<url::Url> = Vec::new();
        for u in &ctx.discovered_urls {
            let Ok(parsed) = url::Url::parse(u) else {
                continue;
            };
            let path = parsed.path().to_ascii_lowercase();
            if SENSITIVE_HINTS.iter().any(|h| path.contains(h)) {
                candidates.push(parsed);
            }
        }
        if candidates.is_empty() {
            for u in ctx.discovered_urls.iter().take(15) {
                if let Ok(parsed) = url::Url::parse(u) {
                    candidates.push(parsed);
                }
            }
        }
        candidates.truncate(40);

        for url in candidates {
            let authed = match ctx.client.get(&url).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let unauth = match anon.get(&url).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            let a_status = authed.status.as_u16();
            let u_status = unauth.status.as_u16();
            let path = url.path().to_ascii_lowercase();
            let sensitive = SENSITIVE_HINTS.iter().any(|h| path.contains(h));

            // Anon gets success on sensitive path
            if sensitive && (200..300).contains(&u_status) {
                let body_looks_data = unauth.is_json()
                    || unauth.body.contains("email")
                    || unauth.body.contains("role")
                    || unauth.body.contains("users")
                    || unauth.body.len() > 40;

                if body_looks_data {
                    findings.push(
                        Finding::builder(self.id(), "anon-access-sensitive")
                            .title("Sensitive path accessible without authentication")
                            .severity(Severity::High)
                            .url(url.as_str())
                            .description(format!(
                                "Unauthenticated request returned HTTP {u_status} on a sensitive-looking path."
                            ))
                            .remediation(
                                "Enforce authentication and authorization; return 401/403 for anonymous callers.",
                            )
                            .cwe("CWE-306")
                            .evidence(Evidence::new(
                                "anon body",
                                unauth.body.chars().take(160).collect::<String>(),
                            ))
                            .build(),
                    );
                }
            }

            // Auth gets more data than anon (interesting but expected) vs same body (maybe public)
            if (200..300).contains(&a_status)
                && (200..300).contains(&u_status)
                && sensitive
                && similar_body(&authed.body, &unauth.body)
            {
                findings.push(
                    Finding::builder(self.id(), "auth-no-difference")
                        .title("Authenticated and anonymous responses look identical")
                        .severity(Severity::Medium)
                        .url(url.as_str())
                        .description(
                            "Session cookie does not change the response on a sensitive path — data may be fully public or auth ignored.",
                        )
                        .remediation(
                            "Verify the route requires auth and returns different content for privileged users.",
                        )
                        .cwe("CWE-862")
                        .build(),
                );
            }

            // Anon 401/403 but still leaks data in body
            if matches!(u_status, 401 | 403)
                && (unauth.body.contains("email")
                    || unauth.body.contains("stack")
                    || unauth.body.contains("password"))
            {
                findings.push(
                    Finding::builder(self.id(), "error-body-leak")
                        .title("Error response may leak sensitive fields")
                        .severity(Severity::Low)
                        .url(url.as_str())
                        .description(format!(
                            "HTTP {u_status} body appears to include sensitive-looking content."
                        ))
                        .build(),
                );
            }
        }

        Ok(findings)
    }
}

fn similar_body(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() || b.is_empty() {
        return false;
    }
    // crude similarity: first 200 chars
    let aa: String = a.chars().take(200).collect();
    let bb: String = b.chars().take(200).collect();
    aa == bb
}
