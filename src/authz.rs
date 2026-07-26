use std::collections::HashSet;

use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum AuthzError {
    #[error(
        "refusing to scan: missing ownership consent. Re-run with --i-own-this only against systems you own or have written permission to test."
    )]
    MissingConsent,

    #[error(
        "refusing to scan: no allowlisted hosts. Pass --allow-host <host> (repeatable) or set allow_hosts in config."
    )]
    EmptyAllowlist,

    #[error("target host '{host}' is not in the allowlist ({allowed})")]
    HostNotAllowed { host: String, allowed: String },

    #[error("invalid target URL: {0}")]
    InvalidUrl(String),

    #[error(
        "active probes requested but --enable-active was not set. Passive recon only by default."
    )]
    ActiveNotEnabled,

    #[error("write HTTP methods requested but --allow-write-methods was not set")]
    WriteNotAllowed,
}

#[derive(Debug, Clone)]
pub struct Authorization {
    pub i_own_this: bool,
    pub allow_hosts: HashSet<String>,
    pub enable_active: bool,
    pub allow_write_methods: bool,
}

impl Authorization {
    pub fn new(
        i_own_this: bool,
        allow_hosts: impl IntoIterator<Item = String>,
        enable_active: bool,
        allow_write_methods: bool,
    ) -> Self {
        let allow_hosts: HashSet<String> = allow_hosts
            .into_iter()
            .map(|h| normalize_host(&h))
            .filter(|h| !h.is_empty())
            .collect();
        Self {
            i_own_this,
            allow_hosts,
            enable_active,
            allow_write_methods,
        }
    }

    /// Validate consent and that every seed target is in scope. No network I/O.
    pub fn validate_targets(&self, targets: &[String]) -> Result<Vec<Url>, AuthzError> {
        if !self.i_own_this {
            return Err(AuthzError::MissingConsent);
        }
        if self.allow_hosts.is_empty() {
            return Err(AuthzError::EmptyAllowlist);
        }

        let mut urls: Vec<Url> = Vec::with_capacity(targets.len());
        for t in targets {
            let url: Url =
                Url::parse(t).map_err(|e| AuthzError::InvalidUrl(format!("{t}: {e}")))?;
            if url.scheme() != "http" && url.scheme() != "https" {
                return Err(AuthzError::InvalidUrl(format!(
                    "{t}: only http/https schemes are supported"
                )));
            }
            let host: &str = url
                .host_str()
                .ok_or_else(|| AuthzError::InvalidUrl(format!("{t}: missing host")))?;
            self.ensure_host_allowed(host)?;
            urls.push(url);
        }
        Ok(urls)
    }

    pub fn ensure_host_allowed(&self, host: &str) -> Result<(), AuthzError> {
        let host: String = normalize_host(host);
        if self.host_matches(&host) {
            Ok(())
        } else {
            let mut allowed: Vec<_> = self.allow_hosts.iter().cloned().collect();
            allowed.sort();
            Err(AuthzError::HostNotAllowed {
                host,
                allowed: allowed.join(", "),
            })
        }
    }

    pub fn url_in_scope(&self, url: &Url) -> bool {
        match url.host_str() {
            Some(h) => self.host_matches(&normalize_host(h)),
            None => false,
        }
    }

    fn host_matches(&self, host: &str) -> bool {
        if self.allow_hosts.contains(host) {
            return true;
        }
        // allow "*.example.com" style: stored as ".example.com" or "*.example.com"
        for pattern in &self.allow_hosts {
            if let Some(suffix) = pattern.strip_prefix("*.") {
                if host == suffix || host.ends_with(&format!(".{suffix}")) {
                    return true;
                }
            }
            if let Some(suffix) = pattern.strip_prefix('.') {
                if host == suffix || host.ends_with(&format!(".{suffix}")) {
                    return true;
                }
            }
        }
        false
    }

    pub fn require_active(&self) -> Result<(), AuthzError> {
        if self.enable_active {
            Ok(())
        } else {
            Err(AuthzError::ActiveNotEnabled)
        }
    }

    pub fn require_write(&self) -> Result<(), AuthzError> {
        if self.allow_write_methods {
            Ok(())
        } else {
            Err(AuthzError::WriteNotAllowed)
        }
    }
}

fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_without_consent() {
        let authz: Authorization = Authorization::new(false, ["example.com".into()], false, false);
        let err: AuthzError = authz
            .validate_targets(&["https://example.com".into()])
            .unwrap_err();
        assert!(matches!(err, AuthzError::MissingConsent));
    }

    #[test]
    fn rejects_empty_allowlist() {
        let authz: Authorization = Authorization::new(true, Vec::<String>::new(), false, false);
        let err: AuthzError = authz
            .validate_targets(&["https://example.com".into()])
            .unwrap_err();
        assert!(matches!(err, AuthzError::EmptyAllowlist));
    }

    #[test]
    fn rejects_host_not_allowlisted() {
        let authz: Authorization = Authorization::new(true, ["allowed.test".into()], false, false);
        let err: AuthzError = authz
            .validate_targets(&["https://evil.test".into()])
            .unwrap_err();
        assert!(matches!(err, AuthzError::HostNotAllowed { .. }));
    }

    #[test]
    fn accepts_allowlisted_host() {
        let authz: Authorization = Authorization::new(true, ["example.com".into()], false, false);
        let urls: Vec<Url> = authz
            .validate_targets(&["https://example.com/app".into()])
            .unwrap();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn wildcard_subdomain() {
        let authz: Authorization = Authorization::new(true, ["*.example.com".into()], false, false);
        assert!(authz.url_in_scope(&Url::parse("https://a.example.com").unwrap()));
        assert!(authz.url_in_scope(&Url::parse("https://example.com").unwrap()));
        assert!(!authz.url_in_scope(&Url::parse("https://evil.com").unwrap()));
    }
}
