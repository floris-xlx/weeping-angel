use anyhow::Result;
use regex::Regex;
use url::Url;

use crate::http::HttpClient;

pub async fn fetch_sitemap(client: &HttpClient, sitemap_url: &Url) -> Result<Vec<Url>> {
    let resp = client.get(sitemap_url).await?;
    if !resp.status.is_success() {
        return Ok(Vec::new());
    }
    Ok(parse_sitemap(sitemap_url, &resp.body))
}

pub fn parse_sitemap(base: &Url, body: &str) -> Vec<Url> {
    // lightweight XML loc extraction (no full XML parser dependency)
    let re = Regex::new(r"(?i)<loc>\s*([^<]+)\s*</loc>").unwrap();
    let mut out = Vec::new();
    for cap in re.captures_iter(body) {
        let raw = cap[1].trim();
        if let Ok(u) = Url::parse(raw) {
            out.push(u);
        } else if let Ok(u) = base.join(raw) {
            out.push(u);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_locs() {
        let base = Url::parse("https://example.com/sitemap.xml").unwrap();
        let body = r#"<?xml version="1.0"?><urlset><url><loc>https://example.com/a</loc></url><url><loc>/b</loc></url></urlset>"#;
        let urls = parse_sitemap(&base, body);
        assert!(urls.iter().any(|u| u.as_str().contains("/a")));
    }
}
