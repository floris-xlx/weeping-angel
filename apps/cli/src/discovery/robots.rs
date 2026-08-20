use anyhow::Result;
use url::Url;

use crate::http::HttpClient;

#[derive(Debug, Default)]
pub struct RobotsRules {
    pub disallow: Vec<String>,
    pub sitemaps: Vec<Url>,
}

pub async fn fetch_robots(client: &HttpClient, seed: &Url) -> Result<RobotsRules> {
    let mut robots_url = seed.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let resp = client.get(&robots_url).await?;
    if !resp.status.is_success() {
        return Ok(RobotsRules::default());
    }
    Ok(parse_robots(seed, &resp.body))
}

pub fn parse_robots(seed: &Url, body: &str) -> RobotsRules {
    let mut rules = RobotsRules::default();
    let mut applies = true; // treat as global unless user-agent specific blocks us; simple: all agents

    for line in body.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        match key.as_str() {
            "user-agent" => {
                applies = value == "*" || value.to_ascii_lowercase().contains("weeping");
            }
            "disallow" if applies => {
                if !value.is_empty() {
                    rules.disallow.push(value.to_string());
                }
            }
            "sitemap" => {
                if let Ok(u) = Url::parse(value) {
                    rules.sitemaps.push(u);
                } else if let Ok(u) = seed.join(value) {
                    rules.sitemaps.push(u);
                }
            }
            _ => {}
        }
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_disallow_and_sitemap() {
        let seed = Url::parse("https://example.com/").unwrap();
        let body = "User-agent: *\nDisallow: /private\nSitemap: https://example.com/sitemap.xml\n";
        let r = parse_robots(&seed, body);
        assert!(r.disallow.iter().any(|d| d == "/private"));
        assert_eq!(r.sitemaps.len(), 1);
    }
}
