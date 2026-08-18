//! SPA / client-router aware endpoint extraction (no headless browser required).
//! Pulls routes from Next.js payloads, common initial state blobs, and JS router tables.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::engine::scope::resolve_link;

static NEXT_DATA: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<script[^>]*id=["']__NEXT_DATA__["'][^>]*>(.*?)</script>"#).unwrap()
});

static INITIAL_STATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?s)window\.__INITIAL_STATE__\s*=\s*(\{.*?\});"#).unwrap());

static ROUTE_ARRAY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(?:routes|paths|pages)\s*[:=]\s*\[([^\]]{0,2000})\]"#).unwrap());

static QUOTED_PATH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""(/[A-Za-z0-9][A-Za-z0-9._/-]{0,120})""#).unwrap());

static HASH_ROUTE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"["']#(/[A-Za-z0-9/_-]{1,80})["']"#).unwrap());

/// Extract candidate URLs from an HTML document that may host an SPA shell.
pub fn extract_from_html(base: &Url, html: &str) -> Vec<Url> {
    let mut out = Vec::new();

    if let Some(cap) = NEXT_DATA.captures(html) {
        out.extend(paths_from_jsonish(base, &cap[1]));
    }
    if let Some(cap) = INITIAL_STATE.captures(html) {
        out.extend(paths_from_jsonish(base, &cap[1]));
    }

    // script tags with type application/json
    let document = Html::parse_document(html);
    if let Ok(sel) = Selector::parse("script[type='application/json'], script#__NEXT_DATA__") {
        for el in document.select(&sel) {
            let text = el.text().collect::<String>();
            out.extend(paths_from_jsonish(base, &text));
        }
    }

    // inline scripts
    if let Ok(sel) = Selector::parse("script:not([src])") {
        for el in document.select(&sel) {
            let text = el.text().collect::<String>();
            out.extend(extract_from_js(base, &text));
        }
    }

    dedupe(out)
}

pub fn extract_from_js(base: &Url, js: &str) -> Vec<Url> {
    let mut out = Vec::new();

    for cap in ROUTE_ARRAY.captures_iter(js) {
        out.extend(paths_from_jsonish(base, &cap[1]));
    }
    for cap in QUOTED_PATH.captures_iter(js) {
        let p = &cap[1];
        if is_useful_path(p) {
            if let Some(u) = resolve_link(base, p) {
                out.push(u);
            }
        }
    }
    for cap in HASH_ROUTE.captures_iter(js) {
        if let Some(u) = resolve_link(base, &cap[1]) {
            out.push(u);
        }
    }

    // createBrowserRouter / path: '/x'
    let path_prop = Regex::new(r#"path\s*:\s*['"](/[^'"]+)['"]"#).unwrap();
    for cap in path_prop.captures_iter(js) {
        if let Some(u) = resolve_link(base, &cap[1]) {
            out.push(u);
        }
    }

    dedupe(out)
}

fn paths_from_jsonish(base: &Url, blob: &str) -> Vec<Url> {
    let mut out = Vec::new();
    for cap in QUOTED_PATH.captures_iter(blob) {
        let p = &cap[1];
        if is_useful_path(p) {
            if let Some(u) = resolve_link(base, p) {
                out.push(u);
            }
        }
    }
    // also absolute URLs
    let abs = Regex::new(r#""(https?://[^"]+)""#).unwrap();
    for cap in abs.captures_iter(blob) {
        if let Ok(u) = Url::parse(&cap[1]) {
            out.push(u);
        }
    }
    out
}

fn is_useful_path(p: &str) -> bool {
    let lower = p.to_ascii_lowercase();
    if p.len() < 2 || p.len() > 160 {
        return false;
    }
    if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".css")
        || lower.ends_with(".woff")
        || lower.ends_with(".svg")
        || lower.ends_with(".map")
    {
        return false;
    }
    // prefer API / app routes
    lower.starts_with("/api")
        || lower.starts_with("/v1")
        || lower.starts_with("/v2")
        || lower.starts_with("/v3")
        || lower.contains("admin")
        || lower.contains("dashboard")
        || lower.contains("auth")
        || lower.contains("user")
        || lower.contains("internal")
        || lower.contains("debug")
        || lower.contains("settings")
        || lower.matches('/').count() >= 1
}

fn dedupe(urls: Vec<Url>) -> Vec<Url> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for u in urls {
        if seen.insert(u.as_str().to_string()) {
            out.push(u);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_data_paths() {
        let base = Url::parse("https://example.com/").unwrap();
        let html = r#"<script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{"paths":["/api/v1/me","/dashboard"]}}}</script>"#;
        let urls = extract_from_html(&base, html);
        assert!(urls.iter().any(|u| u.path() == "/api/v1/me"));
    }

    #[test]
    fn js_router_paths() {
        let base = Url::parse("https://example.com/").unwrap();
        let js = r#"const routes = ["/api/v1/items", "/settings"]; path: '/admin/panel'"#;
        let urls = extract_from_js(&base, js);
        assert!(urls.iter().any(|u| u.path() == "/api/v1/items"));
        assert!(urls.iter().any(|u| u.path() == "/admin/panel"));
    }
}
