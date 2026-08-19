//! HTTP client wrapper. Provider types do not escape this module.

use std::collections::BTreeSet;
use std::time::Duration;

use serde_json::Value;
use thiserror::Error;

use weeping_angel_evidence::redact;

use super::error::sanitize_diagnostic;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("{0}")]
    Transport(String),
}

/// Default GitHub list page size used by the walker (`per_page`).
pub const DEFAULT_PER_PAGE: u32 = 100;

#[derive(Clone)]
struct Fixture {
    path: String,
    status: u16,
    body: String,
    retry_after: Option<u64>,
    link: Option<String>,
}

/// One HTTP result. `Link` (rel=next) drives pagination when present.
#[derive(Debug, Clone)]
pub struct GitHubResponse {
    pub status: u16,
    pub body: String,
    pub retry_after: Option<Duration>,
    pub link: Option<String>,
}

pub struct PageWalk {
    pub items: Vec<Value>,
    pub complete: bool,
    pub error: Option<String>,
}

/// Fixture-friendly client. Live transport is injected; tokens are never logged.
#[derive(Clone, Default)]
pub struct GitHubClient {
    token: Option<String>,
    fixtures: Vec<Fixture>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token,
            fixtures: Vec::new(),
        }
    }

    pub fn with_fixture(
        mut self,
        path: &str,
        status: u16,
        body: &str,
        retry_after: Option<u64>,
    ) -> Self {
        let (body, link) = split_fixture_body(body);
        self.fixtures.push(Fixture {
            path: path.to_string(),
            status,
            body,
            retry_after,
            link,
        });
        self
    }

    pub fn authorization_header(&self) -> Option<String> {
        self.token.as_ref().map(|_| "Bearer [redacted]".to_string())
    }

    pub fn transport_mode(&self) -> &'static str {
        if self.fixtures.is_empty() {
            "unconfigured"
        } else {
            "fixture"
        }
    }

    pub fn get(&self, path: &str) -> Result<(u16, String, Option<Duration>), ClientError> {
        let r = self.get_response(path)?;
        Ok((r.status, r.body, r.retry_after))
    }

    pub fn get_response(&self, path: &str) -> Result<GitHubResponse, ClientError> {
        let _ = redact(self.authorization_header().as_deref().unwrap_or(""));
        let mut attempt = 0u32;
        loop {
            let Some(fx) = self.match_fixture(path, attempt) else {
                if self.token.is_none() {
                    return Ok(GitHubResponse {
                        status: 401,
                        body: r#"{"message":"requires Authorization"}"#.into(),
                        retry_after: None,
                        link: None,
                    });
                }
                return Err(ClientError::Transport(sanitize_diagnostic(&format!(
                    "no fixture and no live transport for {path}"
                ))));
            };
            if fx.status == 429 && self.match_fixture(path, attempt + 1).is_some() {
                attempt += 1;
                continue;
            }
            return Ok(GitHubResponse {
                status: fx.status,
                body: fx.body.clone(),
                retry_after: fx.retry_after.map(Duration::from_secs),
                link: fx.link.clone(),
            });
        }
    }

    /// Walk `base` then `base?page=` using `Link` rel=next or a full `per_page` page.
    pub fn get_pages(&self, base: &str) -> PageWalk {
        let mut items = Vec::new();
        let mut path = base.to_string();
        let mut page: u32 = 1;
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(path.clone()) {
                return PageWalk {
                    items,
                    complete: true,
                    error: None,
                };
            }
            match self.get_response(&path) {
                Ok(resp) if resp.status == 200 => {
                    let json = match serde_json::from_str::<Value>(&resp.body) {
                        Ok(v) => v,
                        Err(e) => {
                            return PageWalk {
                                items,
                                complete: false,
                                error: Some(format!("normalize failed: {e}")),
                            };
                        }
                    };
                    let batch = extract_list_items(&json);
                    let n = batch.len();
                    items.extend(batch);
                    if let Some(next) = resp.link.as_deref().and_then(next_from_link) {
                        path = next;
                        page += 1;
                        continue;
                    }
                    if n >= DEFAULT_PER_PAGE as usize {
                        page += 1;
                        path = format!("{base}?page={page}");
                        continue;
                    }
                    return PageWalk {
                        items,
                        complete: true,
                        error: None,
                    };
                }
                Ok(resp) if resp.status == 401 || resp.status == 403 => {
                    return PageWalk {
                        items,
                        complete: false,
                        error: Some(format!("PermissionDenied: {} listing {path}", resp.status)),
                    };
                }
                Ok(resp) if resp.status == 404 => {
                    return PageWalk {
                        items,
                        complete: page == 1 || page > 1,
                        error: None,
                    };
                }
                Ok(resp) => {
                    return PageWalk {
                        items,
                        complete: false,
                        error: Some(format!("unexpected status {} listing {path}", resp.status)),
                    };
                }
                Err(ClientError::Transport(e)) if page == 1 => {
                    return PageWalk {
                        items,
                        complete: false,
                        error: Some(e),
                    };
                }
                Err(ClientError::Transport(_)) => {
                    // Followed `Link` rel=next into a missing page → not complete.
                    // A probe after a short page is not used; only Link or full pages advance.
                    return PageWalk {
                        items,
                        complete: false,
                        error: Some(format!(
                            "partial pagination: missing page {page} for {base}"
                        )),
                    };
                }
            }
        }
    }

    fn match_fixture(&self, path: &str, attempt: u32) -> Option<&Fixture> {
        let mut best_len = 0usize;
        let mut matched: Vec<&Fixture> = Vec::new();
        for fx in &self.fixtures {
            if path == fx.path || path.starts_with(&fx.path) {
                let len = fx.path.len();
                if len > best_len {
                    best_len = len;
                    matched.clear();
                    matched.push(fx);
                } else if len == best_len {
                    matched.push(fx);
                }
            }
        }
        if matched.is_empty() {
            return None;
        }
        let idx = attempt as usize;
        if idx < matched.len() {
            Some(matched[idx])
        } else {
            matched.last().copied()
        }
    }
}

fn split_fixture_body(body: &str) -> (String, Option<String>) {
    let trimmed = body.trim_start();
    let Some(rest) = trimmed.strip_prefix("LINK: ") else {
        return (body.to_string(), None);
    };
    if let Some((link, json)) = rest.split_once('\n') {
        (json.to_string(), Some(link.trim().to_string()))
    } else {
        (String::new(), Some(rest.trim().to_string()))
    }
}

fn next_from_link(link: &str) -> Option<String> {
    for part in link.split(',') {
        if part.contains("rel=\"next\"") || part.contains("rel=next") {
            let start = part.find('<')?;
            let end = part[start..].find('>')?;
            let url = &part[start + 1..start + end];
            return Some(path_of_url(url));
        }
    }
    None
}

fn path_of_url(url: &str) -> String {
    if let Some(scheme) = url.find("://") {
        let after_host = &url[scheme + 3..];
        if let Some(slash) = after_host.find('/') {
            return after_host[slash..].to_string();
        }
    }
    url.to_string()
}

fn extract_list_items(value: &Value) -> Vec<Value> {
    if let Some(arr) = value.as_array() {
        return arr.clone();
    }
    for key in ["environments", "repositories", "items", "keys"] {
        if let Some(arr) = value.get(key).and_then(Value::as_array) {
            return arr.clone();
        }
    }
    Vec::new()
}
