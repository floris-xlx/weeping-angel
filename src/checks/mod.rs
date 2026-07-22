pub mod auth_compare;
pub mod auth_surface;
pub mod cookies;
pub mod cors;
pub mod exposures;
pub mod headers;
pub mod secrets;
pub mod tech;
pub mod tls;
pub mod active;

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
    ];
    checks.extend(active::registry());
    checks
}
