pub mod crawl;
pub mod image_assets;
pub mod image_harvest;
pub mod js_endpoints;
pub mod openapi;
pub mod robots;
pub mod sitemap;
pub mod spa;
pub mod wordlist;

use url::Url;

#[derive(Debug, Clone)]
pub struct DiscoveredAsset {
    pub url: Url,
    pub status: u16,
    pub content_type: Option<String>,
    pub source: String,
}
