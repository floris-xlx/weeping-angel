use std::collections::HashMap;

use reqwest::StatusCode;
use url::Url;

#[derive(Debug, Clone)]
pub struct ResponseSnapshot {
    pub url: Url,
    pub final_url: Url,
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub content_type: Option<String>,
}

impl ResponseSnapshot {
    pub fn header(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lower)
            .map(|(_, v)| v.as_str())
    }

    pub fn headers_with_prefix(&self, prefix: &str) -> Vec<(&str, &str)> {
        let p = prefix.to_ascii_lowercase();
        self.headers
            .iter()
            .filter(|(k, _)| k.to_ascii_lowercase().starts_with(&p))
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    pub fn is_html(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|c| c.to_ascii_lowercase().contains("text/html"))
            .unwrap_or_else(|| {
                let t = self.body.trim_start();
                t.starts_with("<!DOCTYPE") || t.starts_with("<html") || t.starts_with("<HTML")
            })
    }

    pub fn is_js(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                c.contains("javascript") || c.contains("ecmascript")
            })
            .unwrap_or_else(|| {
                self.final_url
                    .path()
                    .ends_with(".js")
            })
    }

    pub fn is_json(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|c| c.to_ascii_lowercase().contains("json"))
            .unwrap_or(false)
    }

    pub fn set_cookies(&self) -> Vec<String> {
        self.headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
            .map(|(_, v)| v.clone())
            .collect()
    }
}
