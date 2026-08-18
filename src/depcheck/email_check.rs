//! Owner-email takeover heuristics (DepFuzzer `--check-email`, detection only).
//!
//! For packages that **exist** on the public registry, recover maintainer emails
//! (npm / PyPI / crates.io), then flag:
//! - disposable email providers
//! - custom domains with no MX / no RDAP registration (possibly purchasable)
//!
//! Does not attempt to register domains, reset accounts, or publish packages.

use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

use super::types::Ecosystem;

const UA: &str = concat!("weeping-angel-depcheck/", env!("CARGO_PKG_VERSION"));
const DISPOSABLE_URL: &str = "https://disposable.github.io/disposable-email-domains/domains.txt";

const KNOWN_FREEMAIL: &[&str] = &[
    "gmail.com",
    "googlemail.com",
    "outlook.com",
    "hotmail.com",
    "live.com",
    "msn.com",
    "protonmail.com",
    "proton.me",
    "yahoo.com",
    "ymail.com",
    "icloud.com",
    "me.com",
];

/// One email-related finding for a package that exists on the public registry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EmailFinding {
    pub package: String,
    pub ecosystem: String,
    pub email: String,
    pub domain: String,
    pub kind: EmailFindingKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailFindingKind {
    /// Custom domain appears unregistered / no MX — possibly purchasable.
    DomainPossiblyPurchasable,
    /// Maintainer uses a known disposable email provider.
    DisposableEmail,
}

/// Check maintainer emails for packages that exist (npm / pip / cargo only).
pub async fn check_package_emails(
    client: &Client,
    ecosystem: Ecosystem,
    package: &str,
) -> Result<Vec<EmailFinding>> {
    let emails = match ecosystem {
        Ecosystem::Npm => fetch_npm_emails(client, package).await?,
        Ecosystem::Pip => fetch_pypi_emails(client, package).await?,
        Ecosystem::Cargo => fetch_cargo_emails(client, package).await?,
        _ => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for email in emails {
        let Some(domain) = extract_domain(&email) else {
            continue;
        };
        if is_freemail(&domain) {
            continue;
        }
        if is_disposable(client, &domain).await {
            out.push(EmailFinding {
                package: package.to_string(),
                ecosystem: ecosystem.to_string(),
                email: email.clone(),
                domain: domain.clone(),
                kind: EmailFindingKind::DisposableEmail,
                detail: format!("maintainer {email} uses disposable provider {domain}"),
            });
            continue;
        }
        if domain_possibly_purchasable(client, &domain).await {
            out.push(EmailFinding {
                package: package.to_string(),
                ecosystem: ecosystem.to_string(),
                email: email.clone(),
                domain: domain.clone(),
                kind: EmailFindingKind::DomainPossiblyPurchasable,
                detail: format!(
                    "account associated with {package} is {email}; domain {domain} might be purchasable (no MX / no RDAP registration)"
                ),
            });
        }
    }
    Ok(out)
}

pub fn http_client(timeout_secs: u64) -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(timeout_secs.max(5)))
        .user_agent(UA)
        .build()?)
}

async fn fetch_npm_emails(client: &Client, package: &str) -> Result<Vec<String>> {
    let encoded = package.replace('/', "%2f");
    let url = format!("https://registry.npmjs.org/{encoded}");
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut emails = Vec::new();
    if let Some(arr) = data.get("maintainers").and_then(|v| v.as_array()) {
        for m in arr {
            if let Some(e) = m.get("email").and_then(|v| v.as_str()) {
                emails.push(e.to_string());
            }
        }
    }
    if let Some(arr) = data.get("contributors").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(e) = c.get("email").and_then(|v| v.as_str()) {
                emails.push(e.to_string());
            }
        }
    }
    if let Some(e) = data
        .get("author")
        .and_then(|a| a.get("email"))
        .and_then(|v| v.as_str())
    {
        emails.push(e.to_string());
    }
    Ok(emails)
}

async fn fetch_pypi_emails(client: &Client, package: &str) -> Result<Vec<String>> {
    let url = format!("https://pypi.org/pypi/{package}/json");
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut emails = Vec::new();
    if let Some(e) = data
        .pointer("/info/author_email")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        emails.push(e.to_string());
    }
    if let Some(e) = data
        .pointer("/info/maintainer_email")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        emails.push(e.to_string());
    }
    Ok(emails)
}

async fn fetch_cargo_emails(client: &Client, package: &str) -> Result<Vec<String>> {
    // crates.io owners API (no email in public owners list typically) — fall back to crate metadata authors
    let url = format!("https://crates.io/api/v1/crates/{package}");
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut emails = Vec::new();
    if let Some(authors) = data
        .pointer("/versions/0/crate")
        .or_else(|| data.get("crate"))
        .and_then(|c| c.get("description"))
    {
        let _ = authors; // authors often not on summary
    }
    // Prefer owners → user — crates.io rarely exposes emails publicly; try authors string on versions
    if let Some(vers) = data.get("versions").and_then(|v| v.as_array()) {
        if let Some(authors) = vers.first().and_then(|v| v.get("crate_size"))
        // placeholder to keep structure
        {
            let _ = authors;
        }
    }
    // Parse any email-looking strings from crate homepage/repository is weak; use authors if present
    if let Some(authors) = data
        .get("versions")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("published_by"))
        .and_then(|p| p.get("email"))
        .and_then(|v| v.as_str())
    {
        emails.push(authors.to_string());
    }
    Ok(emails)
}

fn extract_domain(raw: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE
        .get_or_init(|| Regex::new(r"(?i)[\w.+-]+@([\w-]+\.[\w.-]+)").expect("email domain regex"));
    re.captures(raw)
        .map(|c| c[1].trim_end_matches('.').to_ascii_lowercase())
}

fn is_freemail(domain: &str) -> bool {
    KNOWN_FREEMAIL
        .iter()
        .any(|d| domain.eq_ignore_ascii_case(d))
}

async fn disposable_set(client: &Client) -> HashSet<String> {
    static CACHE: OnceLock<tokio::sync::Mutex<Option<HashSet<String>>>> = OnceLock::new();
    let lock = CACHE.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut guard = lock.lock().await;
    if let Some(set) = guard.as_ref() {
        return set.clone();
    }
    let set = match client.get(DISPOSABLE_URL).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(text) => text
                .lines()
                .map(|l| l.trim().to_ascii_lowercase())
                .filter(|l| !l.is_empty())
                .collect(),
            Err(_) => HashSet::new(),
        },
        _ => HashSet::new(),
    };
    *guard = Some(set.clone());
    set
}

async fn is_disposable(client: &Client, domain: &str) -> bool {
    let set = disposable_set(client).await;
    set.contains(&domain.to_ascii_lowercase())
}

/// True when domain looks unregistered / without mail exchange (DepFuzzer-style heuristic).
async fn domain_possibly_purchasable(client: &Client, domain: &str) -> bool {
    let has_mx = doh_has_mx(client, domain).await.unwrap_or(true);
    if has_mx {
        return false;
    }
    // No MX — check RDAP; missing registration strengthens the signal
    match client
        .get(format!("https://rdap.org/domain/{domain}"))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().as_u16() == 404 {
                return true;
            }
            if let Ok(v) = resp.json::<Value>().await {
                // No entities / status redacted still may be registered
                let status = v
                    .get("status")
                    .and_then(|s| s.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(false);
                let entities_empty = v
                    .get("entities")
                    .and_then(|e| e.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(true);
                return status && entities_empty;
            }
            true
        }
        Err(_) => true, // DepFuzzer treats whois failure as takeoverable
    }
}

async fn doh_has_mx(client: &Client, domain: &str) -> Result<bool> {
    // Cloudflare DNS-over-HTTPS
    let url = format!("https://cloudflare-dns.com/dns-query?name={domain}&type=MX");
    let resp = client
        .get(&url)
        .header("Accept", "application/dns-json")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(true); // fail closed: don't flag on DoH errors
    }
    let data: Value = resp.json().await?;
    let status = data.get("Status").and_then(|v| v.as_u64()).unwrap_or(0);
    // 0 = NOERROR, 3 = NXDOMAIN
    if status == 3 {
        return Ok(false);
    }
    let answers = data
        .get("Answer")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_domain_from_angle_email() {
        assert_eq!(
            extract_domain("Name <foo@Example.COM>").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn freemail_skipped() {
        assert!(is_freemail("gmail.com"));
        assert!(!is_freemail("acme-corp.example"));
    }
}
