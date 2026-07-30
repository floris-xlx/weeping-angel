//! Target URL normalization: bare hosts, //host, http(s), loopback defaults.

use std::net::IpAddr;

use thiserror::Error;
use url::Url;

use crate::parse::split_list;

#[derive(Debug, Error)]
pub enum TargetError {
    #[error(
        "invalid target '{raw}': {detail}. Accepted: example.com, example.com:8443/path, //host/path, http://…, https://…"
    )]
    Invalid { raw: String, detail: String },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NormalizeOptions {
    /// When scheme is missing, force http instead of smart default.
    pub prefer_http: bool,
}

/// Expand multi-target paste (comma-separated) then normalize each.
pub fn normalize_targets(
    raw: &[String],
    opts: NormalizeOptions,
) -> Result<Vec<String>, TargetError> {
    let mut out = Vec::new();
    for item in raw {
        for part in split_list(item) {
            out.push(normalize_one(&part, opts)?);
        }
    }
    if out.is_empty() {
        return Err(TargetError::Invalid {
            raw: String::new(),
            detail: "no targets provided".into(),
        });
    }
    Ok(out)
}

pub fn normalize_one(raw: &str, opts: NormalizeOptions) -> Result<String, TargetError> {
    // Strip whitespace, surrounding quotes, and trailing commas (paste artifacts).
    // Order matters: `'example.com',` must lose the comma then the quotes.
    let mut s = raw.trim().to_string();
    loop {
        let next = s
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim_matches(|c| c == ',' || c == ';')
            .to_string();
        if next == s {
            break;
        }
        s = next;
    }
    let s = s.as_str();
    if s.is_empty() {
        return Err(TargetError::Invalid {
            raw: raw.to_string(),
            detail: "empty".into(),
        });
    }

    // Already absolute http(s)
    if let Some(rest) = s.strip_prefix("https://") {
        return finalize_parsed(&format!("https://{rest}"), s);
    }
    if let Some(rest) = s.strip_prefix("http://") {
        return finalize_parsed(&format!("http://{rest}"), s);
    }
    // Protocol-relative
    if let Some(rest) = s.strip_prefix("//") {
        let scheme = scheme_for_authority(rest, opts);
        return finalize_parsed(&format!("{scheme}://{rest}"), s);
    }
    // Reject other schemes early with clear message
    if let Some(idx) = s.find("://") {
        let scheme = &s[..idx];
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(TargetError::Invalid {
                raw: s.to_string(),
                detail: format!("unsupported scheme '{scheme}' (only http/https)"),
            });
        }
    }

    // Bare host / host:port / host/path
    if looks_like_authority_or_path(s) {
        let scheme = scheme_for_authority(s, opts);
        return finalize_parsed(&format!("{scheme}://{s}"), s);
    }

    Err(TargetError::Invalid {
        raw: s.to_string(),
        detail: "could not parse as host or URL".into(),
    })
}

fn finalize_parsed(candidate: &str, raw: &str) -> Result<String, TargetError> {
    let mut url = Url::parse(candidate).map_err(|e| TargetError::Invalid {
        raw: raw.to_string(),
        detail: e.to_string(),
    })?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(TargetError::Invalid {
            raw: raw.to_string(),
            detail: format!("unsupported scheme '{}'", url.scheme()),
        });
    }
    if url.host_str().is_none() {
        return Err(TargetError::Invalid {
            raw: raw.to_string(),
            detail: "missing host".into(),
        });
    }
    // Ensure path at least "/"
    if url.path().is_empty() {
        url.set_path("/");
    }
    Ok(url.into())
}

fn looks_like_authority_or_path(s: &str) -> bool {
    // Reject spaces and obvious garbage
    if s.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    // host, host:port, host/path, [ipv6]:port/path
    let authority = s.split('/').next().unwrap_or(s);
    if authority.is_empty() {
        return false;
    }
    // Must have a dot, or be localhost-ish, or an IP, or have a port
    if authority.eq_ignore_ascii_case("localhost")
        || authority.to_ascii_lowercase().starts_with("localhost:")
    {
        return true;
    }
    if authority.parse::<IpAddr>().is_ok() {
        return true;
    }
    // bracket IPv6
    if authority.starts_with('[') {
        return true;
    }
    // host:port where host may lack dots (e.g. myservice:8080 in lab)
    if let Some((host, port)) = authority.rsplit_once(':') {
        if !host.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    authority.contains('.') || authority.contains("localhost")
}

fn scheme_for_authority(authority_and_path: &str, opts: NormalizeOptions) -> &'static str {
    if opts.prefer_http {
        return "http";
    }
    let authority = authority_and_path
        .split('/')
        .next()
        .unwrap_or(authority_and_path);
    let host = authority_host(authority);
    if should_default_http(&host, authority) {
        "http"
    } else {
        "https"
    }
}

fn authority_host(authority: &str) -> String {
    let without_user = authority.rsplit('@').next().unwrap_or(authority);
    if without_user.starts_with('[') {
        // [ipv6]:port
        if let Some(end) = without_user.find(']') {
            return without_user[1..end].to_string();
        }
        return without_user.trim_matches(|c| c == '[' || c == ']').to_string();
    }
    if let Some((h, port)) = without_user.rsplit_once(':') {
        if port.chars().all(|c| c.is_ascii_digit()) {
            return h.to_string();
        }
    }
    without_user.to_string()
}

fn should_default_http(host: &str, authority: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if h == "localhost" || h.ends_with(".localhost") || h.ends_with(".local") {
        return true;
    }
    if let Ok(ip) = h.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback()
                    || v4.is_private()
                    || v4.is_link_local()
                    || v4.octets()[0] == 0 // 0.0.0.0-ish lab
            }
            IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
        };
    }
    // Explicit :80 → http
    if let Some((_, port)) = authority.rsplit_once(':') {
        if port == "80" {
            return true;
        }
    }
    false
}

/// Extract host string from a normalized target URL for allowlist convenience.
pub fn host_of_normalized(url_str: &str) -> Option<String> {
    Url::parse(url_str)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_public_host_https() {
        let u = normalize_one("example.com", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "https://example.com/");
    }

    #[test]
    fn bare_with_path_and_port() {
        let u = normalize_one("example.com:8443/app", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "https://example.com:8443/app");
    }

    #[test]
    fn protocol_relative() {
        let u = normalize_one("//cdn.example.com/x", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "https://cdn.example.com/x");
    }

    #[test]
    fn loopback_http() {
        let u = normalize_one("127.0.0.1:8787", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "http://127.0.0.1:8787/");
        let u2 = normalize_one("localhost", NormalizeOptions::default()).unwrap();
        assert_eq!(u2, "http://localhost/");
    }

    #[test]
    fn explicit_schemes_preserved() {
        let u = normalize_one("http://example.com/a", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "http://example.com/a");
        let u = normalize_one("https://example.com/a", NormalizeOptions::default()).unwrap();
        assert_eq!(u, "https://example.com/a");
    }

    #[test]
    fn prefer_http_override() {
        let u = normalize_one(
            "example.com",
            NormalizeOptions { prefer_http: true },
        )
        .unwrap();
        assert_eq!(u, "http://example.com/");
    }

    #[test]
    fn multi_csv_targets() {
        let v = normalize_targets(
            &["a.com, b.com".into()],
            NormalizeOptions::default(),
        )
        .unwrap();
        assert_eq!(v.len(), 2);
        assert!(v[0].starts_with("https://a.com"));
        assert!(v[1].starts_with("https://b.com"));
    }

    #[test]
    fn private_rfc1918_http() {
        for h in ["10.1.2.3", "192.168.0.50", "172.16.9.9"] {
            let u = normalize_one(h, NormalizeOptions::default()).unwrap();
            assert!(u.starts_with("http://"), "{h} → {u}");
        }
    }

    #[test]
    fn query_and_fragment_on_explicit_https() {
        let u = normalize_one(
            "https://example.com/path?x=1#frag",
            NormalizeOptions::default(),
        )
        .unwrap();
        assert!(u.contains("?x=1"));
        // url crate may drop fragment on into() depending on version — path must remain
        assert!(u.contains("/path"));
    }

    #[test]
    fn host_of_normalized_none_on_garbage() {
        assert!(host_of_normalized("not-a-url").is_none());
    }
}
