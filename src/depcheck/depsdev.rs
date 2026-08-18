//! Thin client for deps.dev (used by DepFuzzer-style transitive walks).

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

use super::types::{Ecosystem, PackageRef};

const UA: &str = concat!("weeping-angel-depcheck/", env!("CARGO_PKG_VERSION"));

fn system_name(eco: Ecosystem) -> Option<&'static str> {
    match eco {
        Ecosystem::Npm => Some("npm"),
        Ecosystem::Pip => Some("pypi"),
        Ecosystem::Cargo => Some("cargo"),
        Ecosystem::Go => Some("go"),
        Ecosystem::Maven => Some("maven"),
        Ecosystem::Rubygems => Some("rubygems"),
        Ecosystem::Nuget => Some("nuget"),
        Ecosystem::Composer => None, // not on deps.dev systems list
    }
}

/// Fetch direct dependency names for `name@version` via deps.dev legacy JSON API
/// (`https://deps.dev/_/s/{system}/p/{pkg}/v/{ver}/dependencies`).
pub async fn fetch_transitive(
    client: &Client,
    eco: Ecosystem,
    name: &str,
    version: &str,
) -> Result<Vec<PackageRef>> {
    let Some(system) = system_name(eco) else {
        return Ok(Vec::new());
    };
    let package = urlencoding_path(name);
    let ver = sanitize_version(version);
    let url = if ver.is_empty() {
        format!("https://deps.dev/_/s/{system}/p/{package}/v/")
    } else {
        format!("https://deps.dev/_/s/{system}/p/{package}/v/{ver}/dependencies")
    };

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(20))
        .header("User-Agent", UA)
        .send()
        .await
        .with_context(|| format!("deps.dev fetch {url}"))?;

    if resp.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let data: Value = resp.json().await?;
    let mut out = Vec::new();
    if let Some(deps) = data.get("dependencies").and_then(|v| v.as_array()) {
        // First node is often the root; skip index 0 when present
        let iter = if deps.len() > 1 {
            &deps[1..]
        } else {
            deps.as_slice()
        };
        for dep in iter {
            let pname = dep
                .pointer("/package/name")
                .or_else(|| dep.get("name"))
                .and_then(|v| v.as_str());
            let pver = dep.get("version").and_then(|v| v.as_str()).unwrap_or("*");
            if let Some(n) = pname {
                if !n.is_empty() {
                    out.push(PackageRef::new(n, pver));
                }
            }
        }
    }
    Ok(out)
}

pub fn client(timeout_secs: u64) -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(timeout_secs.max(5)))
        .user_agent(UA)
        .build()?)
}

fn urlencoding_path(name: &str) -> String {
    // percent-encode everything except unreserved (deps.dev uses quote(safe=''))
    let mut out = String::new();
    for b in name.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn sanitize_version(version: &str) -> String {
    version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
        .collect()
}

/// NuGet IDs that match reserved publisher prefixes (DepFuzzer skip list).
pub fn is_nuget_reserved(package_id: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "Microsoft",
        "System",
        "Azure",
        "Serilog",
        "Newtonsoft",
        "Xamarin",
        "xunit",
        "OpenTelemetry",
        "Spectre",
        "Grpc",
        "NuGet",
        "Google",
        "AWSSDK",
        "Castle",
        "Polly",
        "Moq",
        "AutoMapper",
    ];
    let lower = package_id.to_ascii_lowercase();
    PREFIXES.iter().any(|p| {
        let pl = p.to_ascii_lowercase();
        lower == pl || lower.starts_with(&format!("{pl}."))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_scoped_npm() {
        let e = urlencoding_path("@scope/pkg");
        assert!(e.contains("%40"));
        assert!(e.contains("%2F") || e.contains("%2f"));
    }

    #[test]
    fn nuget_reserved_microsoft() {
        assert!(is_nuget_reserved("Microsoft.Extensions.Logging"));
        assert!(!is_nuget_reserved("Acme.Internal.Auth"));
    }
}
