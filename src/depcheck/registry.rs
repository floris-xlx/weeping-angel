//! Concurrent public-registry existence checks.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::{Client, StatusCode};
use tokio::sync::Semaphore;
use tokio::time::sleep;

use super::types::{CheckStatus, Ecosystem, PackageRef, PackageResult};

const UA: &str = concat!("weeping-angel-depcheck/", env!("CARGO_PKG_VERSION"));

/// Pluggable registry HTTP client (real or stub).
#[async_trait]
pub trait RegistryClient: Send + Sync {
    async fn check(&self, ecosystem: Ecosystem, name: &str) -> PackageResult;
}

/// Live HTTP registry client.
#[derive(Clone)]
pub struct HttpRegistry {
    client: Client,
    timeout: Duration,
    retries: u32,
}

impl HttpRegistry {
    pub fn new(timeout_secs: u64) -> anyhow::Result<Self> {
        let timeout = Duration::from_secs(timeout_secs.max(1));
        let client = Client::builder().timeout(timeout).user_agent(UA).build()?;
        Ok(Self {
            client,
            timeout,
            retries: 3,
        })
    }
}

#[async_trait]
impl RegistryClient for HttpRegistry {
    async fn check(&self, ecosystem: Ecosystem, name: &str) -> PackageResult {
        for attempt in 0..self.retries {
            match self.check_once(ecosystem, name).await {
                Ok(status) => {
                    return PackageResult {
                        name: name.to_string(),
                        version: String::new(),
                        status,
                        detail: None,
                    };
                }
                Err(Retryable::RateLimited) => {
                    sleep(Duration::from_secs(2 * (attempt as u64 + 1))).await;
                }
                Err(Retryable::Transient(msg)) => {
                    if attempt + 1 < self.retries {
                        sleep(Duration::from_secs(attempt as u64 + 1)).await;
                        continue;
                    }
                    return PackageResult {
                        name: name.to_string(),
                        version: String::new(),
                        status: CheckStatus::Error,
                        detail: Some(msg),
                    };
                }
                Err(Retryable::Fatal(msg)) => {
                    return PackageResult {
                        name: name.to_string(),
                        version: String::new(),
                        status: CheckStatus::Error,
                        detail: Some(msg),
                    };
                }
            }
        }
        PackageResult {
            name: name.to_string(),
            version: String::new(),
            status: CheckStatus::Error,
            detail: Some("retries exhausted".into()),
        }
    }
}

enum Retryable {
    RateLimited,
    Transient(String),
    Fatal(String),
}

impl HttpRegistry {
    async fn check_once(&self, ecosystem: Ecosystem, name: &str) -> Result<CheckStatus, Retryable> {
        let url = match build_url(ecosystem, name) {
            Some(u) => u,
            None => return Ok(CheckStatus::Safe),
        };

        let resp = self
            .client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| Retryable::Transient(e.to_string()))?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(CheckStatus::Vulnerable);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(Retryable::RateLimited);
        }
        if !status.is_success() {
            return Err(Retryable::Transient(format!("HTTP {status}")));
        }

        if ecosystem == Ecosystem::Maven {
            let body = resp
                .text()
                .await
                .map_err(|e| Retryable::Transient(e.to_string()))?;
            let data: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| Retryable::Fatal(format!("maven json: {e}")))?;
            let found = data
                .pointer("/response/numFound")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if found == 0 {
                return Ok(CheckStatus::Vulnerable);
            }
        }

        Ok(CheckStatus::Safe)
    }
}

pub fn build_url(ecosystem: Ecosystem, name: &str) -> Option<String> {
    match ecosystem {
        Ecosystem::Npm => {
            let encoded = name.replace('/', "%2f");
            Some(format!("https://registry.npmjs.org/{encoded}"))
        }
        Ecosystem::Pip => Some(format!("https://pypi.org/pypi/{name}/json")),
        Ecosystem::Composer => Some(format!("https://repo.packagist.org/p2/{name}.json")),
        Ecosystem::Rubygems => Some(format!("https://rubygems.org/api/v1/gems/{name}.json")),
        Ecosystem::Nuget => {
            let lower = name.to_ascii_lowercase();
            Some(format!(
                "https://api.nuget.org/v3-flatcontainer/{lower}/index.json"
            ))
        }
        Ecosystem::Cargo => Some(format!("https://crates.io/api/v1/crates/{name}")),
        Ecosystem::Go => Some(format!("https://proxy.golang.org/{name}/@latest")),
        Ecosystem::Maven => {
            let parts: Vec<&str> = name.splitn(2, ':').collect();
            if parts.len() != 2 {
                return None;
            }
            let (group, artifact) = (parts[0], parts[1]);
            Some(format!(
                "https://search.maven.org/solrsearch/select?q=a:{artifact}%20AND%20g:{group}&rows=1&wt=json"
            ))
        }
    }
}

/// Check many packages concurrently.
pub async fn check_many(
    client: Arc<dyn RegistryClient>,
    ecosystem: Ecosystem,
    packages: &[PackageRef],
    threads: usize,
) -> Vec<PackageResult> {
    let sem = Arc::new(Semaphore::new(threads.max(1)));
    let mut futs = FuturesUnordered::new();

    for pkg in packages {
        let client = Arc::clone(&client);
        let sem = Arc::clone(&sem);
        let name = pkg.name.clone();
        let version = pkg.version.clone();
        futs.push(async move {
            let _permit = sem.acquire().await.expect("semaphore");
            let mut result = client.check(ecosystem, &name).await;
            result.version = version;
            result
        });
    }

    let mut out = Vec::with_capacity(packages.len());
    while let Some(r) = futs.next().await {
        out.push(r);
    }
    out
}

/// Test double: names in `missing` → Vulnerable, else Safe.
#[derive(Debug, Default)]
pub struct StubRegistry {
    pub missing: std::collections::HashSet<String>,
}

#[async_trait]
impl RegistryClient for StubRegistry {
    async fn check(&self, _ecosystem: Ecosystem, name: &str) -> PackageResult {
        let status = if self.missing.contains(name) {
            CheckStatus::Vulnerable
        } else {
            CheckStatus::Safe
        };
        PackageResult {
            name: name.to_string(),
            version: String::new(),
            status,
            detail: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npm_scoped_url_encodes_slash() {
        let u = build_url(Ecosystem::Npm, "@scope/pkg").unwrap();
        assert!(u.contains("%2f"));
    }

    #[test]
    fn maven_requires_colon() {
        assert!(build_url(Ecosystem::Maven, "only-artifact").is_none());
        assert!(build_url(Ecosystem::Maven, "g:a").is_some());
    }

    #[tokio::test]
    async fn stub_marks_missing_vulnerable() {
        let mut stub = StubRegistry::default();
        stub.missing.insert("acme-private".into());
        let client: Arc<dyn RegistryClient> = Arc::new(stub);
        let pkgs = vec![
            PackageRef::new("react", "1"),
            PackageRef::new("acme-private", "1"),
        ];
        let results = check_many(client, Ecosystem::Npm, &pkgs, 4).await;
        let acme = results.iter().find(|r| r.name == "acme-private").unwrap();
        let react = results.iter().find(|r| r.name == "react").unwrap();
        assert_eq!(acme.status, CheckStatus::Vulnerable);
        assert_eq!(react.status, CheckStatus::Safe);
    }
}
