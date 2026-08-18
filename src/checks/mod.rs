pub mod active;
pub mod auth_compare;
pub mod auth_surface;
pub mod cookies;
pub mod cors;
pub mod exposures;
pub mod firebase;
pub mod headers;
pub mod rate_limits;
pub mod secrets;
pub mod tech;
pub mod tls;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::discovery::DiscoveredAsset;
use crate::finding::Finding;
use crate::http::{HttpClient, ResponseSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckKind {
    Passive,
    Active,
}

#[derive(Clone)]
pub struct ScanContext {
    pub client: Arc<HttpClient>,
    /// Unauthenticated client (no session cookie) when `--compare-auth` is set.
    pub anon_client: Option<Arc<HttpClient>>,
    pub seed: Url,
    pub assets: Vec<DiscoveredAsset>,
    pub responses: HashMap<String, ResponseSnapshot>,
    pub discovered_urls: Vec<String>,
    pub probes: Vec<String>,
    pub enable_active: bool,
}

#[async_trait]
pub trait Check: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> CheckKind;
    async fn run(&self, ctx: &ScanContext) -> Result<Vec<Finding>>;
}

pub fn registry() -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = vec![
        Box::new(headers::HeadersCheck),
        Box::new(tls::TlsCheck),
        Box::new(cookies::CookiesCheck),
        Box::new(secrets::SecretsCheck),
        Box::new(exposures::ExposuresCheck),
        Box::new(tech::TechCheck),
        Box::new(cors::CorsCheck),
        Box::new(auth_surface::AuthSurfaceCheck),
        Box::new(auth_compare::AuthCompareCheck),
        Box::new(firebase::FirebaseCheck),
        Box::new(rate_limits::RateLimitsCheck),
    ];
    checks.extend(active::registry());
    checks
}

/// Test helpers for building a minimal `ScanContext` without network I/O.
#[cfg(test)]
pub mod test_util {
    use std::collections::HashMap;
    use std::sync::Arc;

    use reqwest::StatusCode;
    use url::Url;

    use crate::authz::Authorization;
    use crate::checks::ScanContext;
    use crate::discovery::DiscoveredAsset;
    use crate::http::{ClientConfig, HttpClient, ResponseSnapshot};

    pub fn dummy_client() -> Arc<HttpClient> {
        let authz = Authorization::new(
            true,
            vec!["example.com".into(), "127.0.0.1".into()],
            false,
            false,
        );
        Arc::new(HttpClient::new(authz, ClientConfig::default()).expect("test client"))
    }

    pub fn snapshot(
        url: &str,
        status: u16,
        headers: &[(&str, &str)],
        body: &str,
    ) -> ResponseSnapshot {
        let u = Url::parse(url).expect("url");
        let mut map = HashMap::new();
        let mut content_type = None;
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("content-type") {
                content_type = Some((*v).to_string());
            }
            map.insert((*k).to_string(), (*v).to_string());
        }
        ResponseSnapshot {
            url: u.clone(),
            final_url: u,
            status: StatusCode::from_u16(status).unwrap_or(StatusCode::OK),
            headers: map,
            body: body.to_string(),
            content_type,
        }
    }

    pub fn context_with_responses(responses: HashMap<String, ResponseSnapshot>) -> ScanContext {
        let seed = Url::parse("https://example.com/").unwrap();
        let assets: Vec<DiscoveredAsset> = responses
            .values()
            .map(|r| DiscoveredAsset {
                url: r.final_url.clone(),
                status: r.status.as_u16(),
                content_type: r.content_type.clone(),
                source: "test".into(),
            })
            .collect();
        let discovered_urls: Vec<String> = responses.keys().cloned().collect();
        ScanContext {
            client: dummy_client(),
            anon_client: None,
            seed,
            assets,
            responses,
            discovered_urls,
            probes: vec![],
            enable_active: false,
        }
    }
}
