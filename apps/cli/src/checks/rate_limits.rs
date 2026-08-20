//! Detect and probe per-route rate limiting.
//!
//! Passive phase: inspect cached responses for 429, `Retry-After`, and
//! `RateLimit-*` / `X-RateLimit-*` headers.
//!
//! Light active phase (when `enable_active`): small burst against auth and API
//! candidates to see if the target enforces throttling.

use std::collections::HashSet;
use std::time::Instant;

use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::checks::{Check, CheckKind, ScanContext};
use crate::finding::{Evidence, Finding, Severity};

pub struct RateLimitsCheck;

const AUTH_PATH_HINTS: &[&str] = &[
    "login", "signin", "sign-in", "signup", "sign-up", "register", "auth", "oauth", "token",
    "password", "session",
];

const RATE_HEADER_PREFIXES: &[&str] = &[
    "ratelimit-",
    "x-ratelimit-",
    "x-rate-limit-",
    "retry-after",
    "x-ratelimit",
];

#[async_trait]
impl Check for RateLimitsCheck {
    fn id(&self) -> &'static str {
        "rate-limits"
    }

    fn kind(&self) -> CheckKind {
        CheckKind::Passive
    }

    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        let mut seen = HashSet::new();
        let mut routes_with_limits: HashSet<String> = HashSet::new();
        let mut routes_checked: HashSet<String> = HashSet::new();

        // --- Passive: headers / status on cached responses ---
        for resp in ctx.responses.values() {
            let url = resp.final_url.as_str().to_string();
            routes_checked.insert(url.clone());
            let status = resp.status.as_u16();

            if status == 429 && seen.insert(format!("429:{url}")) {
                routes_with_limits.insert(url.clone());
                let retry = resp.header("retry-after").unwrap_or("(none)");
                findings.push(
                    Finding::builder(self.id(), "http-429")
                        .title("HTTP 429 Too Many Requests observed")
                        .severity(Severity::Info)
                        .url(&url)
                        .description(format!(
                            "Target returned 429 (rate limited). Retry-After: {retry}."
                        ))
                        .evidence(Evidence::new("status", "429"))
                        .build(),
                );
            }

            let mut rate_headers: Vec<(String, String)> = Vec::new();
            for (k, v) in &resp.headers {
                let kl = k.to_ascii_lowercase();
                if RATE_HEADER_PREFIXES
                    .iter()
                    .any(|p| kl == *p || kl.starts_with(p.trim_end_matches('-')))
                    || kl.contains("ratelimit")
                    || kl.contains("rate-limit")
                {
                    rate_headers.push((k.clone(), v.clone()));
                }
            }

            // Also explicit common names
            for name in [
                "retry-after",
                "x-ratelimit-limit",
                "x-ratelimit-remaining",
                "x-ratelimit-reset",
                "ratelimit-limit",
                "ratelimit-remaining",
                "ratelimit-reset",
                "ratelimit-policy",
            ] {
                if let Some(v) = resp.header(name)
                    && !rate_headers
                        .iter()
                        .any(|(k, _)| k.eq_ignore_ascii_case(name))
                {
                    rate_headers.push((name.into(), v.into()));
                }
            }

            if !rate_headers.is_empty() && seen.insert(format!("hdr:{url}")) {
                routes_with_limits.insert(url.clone());
                let summary = rate_headers
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                findings.push(
                    Finding::builder(self.id(), "rate-limit-headers")
                        .title("Rate-limit headers present")
                        .severity(Severity::Info)
                        .url(&url)
                        .description(format!(
                            "Response advertises rate limiting metadata: {summary}"
                        ))
                        .evidence(Evidence::new(
                            "headers",
                            summary.chars().take(400).collect::<String>(),
                        ))
                        .build(),
                );
            }
        }

        // --- Candidate routes for missing-limit analysis + optional burst ---
        let mut candidates: Vec<Url> = Vec::new();
        for u in &ctx.discovered_urls {
            let Ok(parsed) = Url::parse(u) else {
                continue;
            };
            let path = parsed.path().to_ascii_lowercase();
            let interesting = AUTH_PATH_HINTS.iter().any(|h| path.contains(h))
                || path.contains("/api/")
                || path.starts_with("/api")
                || path.contains("graphql");
            if interesting {
                candidates.push(parsed);
            }
        }
        candidates.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        candidates.dedup();
        candidates.truncate(20);

        // Annotate auth/API routes with no observed rate-limit signal (informational)
        for url in &candidates {
            let key = url.as_str();
            if routes_with_limits
                .iter()
                .any(|r| r.starts_with(key) || key.starts_with(r.as_str()))
            {
                continue;
            }
            // Check if any cached response for this exact URL had limits
            let had = ctx.responses.get(key).map(|r| {
                r.status.as_u16() == 429
                    || r.headers.keys().any(|h| {
                        let hl = h.to_ascii_lowercase();
                        hl.contains("ratelimit") || hl.contains("rate-limit") || hl == "retry-after"
                    })
            });
            if had == Some(true) {
                continue;
            }
            if seen.insert(format!("missing:{key}")) {
                let path = url.path().to_ascii_lowercase();
                let is_auth = AUTH_PATH_HINTS.iter().any(|h| path.contains(h));
                findings.push(
                    Finding::builder(self.id(), "no-rate-limit-signal")
                        .title(if is_auth {
                            "Auth-related route: no rate-limit signal observed"
                        } else {
                            "API route: no rate-limit signal observed"
                        })
                        .severity(if is_auth {
                            Severity::Medium
                        } else {
                            Severity::Low
                        })
                        .url(key)
                        .description(
                            "During recon, this route did not return 429 or standard rate-limit headers. \
                             Absence of headers does not prove unlimited access, but auth endpoints without \
                             throttling are high-value for credential stuffing.",
                        )
                        .remediation(
                            "Apply IP/user rate limits, progressive delays, CAPTCHA/App Check, and WAF rules on login/signup/token routes.",
                        )
                        .cwe("CWE-770")
                        .build(),
                );
            }
        }

        // --- Light burst when active scanning enabled ---
        if ctx.enable_active {
            let burst_targets: Vec<Url> = candidates
                .into_iter()
                .filter(|u| {
                    let p = u.path().to_ascii_lowercase();
                    AUTH_PATH_HINTS.iter().any(|h| p.contains(h))
                })
                .take(5)
                .collect();

            const BURST: usize = 8;
            for url in burst_targets {
                let mut got_429 = false;
                let mut last_status = 0u16;
                let t0 = Instant::now();
                for _ in 0..BURST {
                    match ctx.client.get(&url).await {
                        Ok(r) => {
                            last_status = r.status.as_u16();
                            if last_status == 429 {
                                got_429 = true;
                                let retry = r.header("retry-after").unwrap_or("?");
                                if seen.insert(format!("burst429:{}", url.as_str())) {
                                    routes_with_limits.insert(url.as_str().into());
                                    findings.push(
                                        Finding::builder(self.id(), "burst-triggered-429")
                                            .title("Rate limit engaged under light burst")
                                            .severity(Severity::Info)
                                            .url(url.as_str())
                                            .description(format!(
                                                "A short burst of {BURST} GETs produced HTTP 429 after {:.0}ms. Retry-After: {retry}.",
                                                t0.elapsed().as_millis()
                                            ))
                                            .build(),
                                    );
                                }
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                if !got_429 && seen.insert(format!("burst-ok:{}", url.as_str())) {
                    findings.push(
                        Finding::builder(self.id(), "burst-no-throttle")
                            .title("Auth route accepted rapid sequential GETs without 429")
                            .severity(Severity::Medium)
                            .url(url.as_str())
                            .description(format!(
                                "Sent {BURST} sequential GETs in {:.0}ms; last status was {last_status}. \
                                 No 429 observed (POST login may still be throttled).",
                                t0.elapsed().as_millis()
                            ))
                            .remediation(
                                "Throttle authentication endpoints by IP, account, and fingerprint; consider CAPTCHA after N failures.",
                            )
                            .cwe("CWE-770")
                            .build(),
                    );
                }
            }
        }

        // Aggregate summary finding
        if !routes_checked.is_empty() {
            findings.push(
                Finding::builder(self.id(), "rate-limit-summary")
                    .title(format!(
                        "Rate-limit summary: {}/{} observed routes show limit signals",
                        routes_with_limits.len(),
                        routes_checked.len().max(1)
                    ))
                    .severity(Severity::Info)
                    .url(ctx.seed.as_str())
                    .description(format!(
                        "Routes with rate-limit headers or 429: {}. \
                         Review auth and write-like APIs for missing throttling.",
                        if routes_with_limits.is_empty() {
                            "(none)".into()
                        } else {
                            routes_with_limits
                                .iter()
                                .take(12)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .build(),
            );
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
    async fn detects_rate_limit_headers_and_429() {
        let mut responses = HashMap::new();
        responses.insert(
            "https://example.com/api/limited".into(),
            snapshot(
                "https://example.com/api/limited",
                200,
                &[
                    ("content-type", "application/json"),
                    ("x-ratelimit-limit", "60"),
                    ("x-ratelimit-remaining", "59"),
                ],
                r#"{"ok":true}"#,
            ),
        );
        responses.insert(
            "https://example.com/api/burst".into(),
            snapshot(
                "https://example.com/api/burst",
                429,
                &[("retry-after", "30")],
                "slow down",
            ),
        );
        let ctx = context_with_responses(responses);
        let findings = RateLimitsCheck.run(&ctx).await.unwrap();
        assert!(findings.iter().any(|f| f.id == "rate-limit-headers"));
        assert!(findings.iter().any(|f| f.id == "http-429"));
    }
}
