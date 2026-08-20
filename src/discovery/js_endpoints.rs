use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::engine::scope::resolve_link;

// Rust `regex` crate has no backreferences — match each quote style separately.
static PATH_RE_DQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""(https?://[^"\s]+|/api[^"\s]*|/[A-Za-z0-9][A-Za-z0-9._/-]{1,120})""#)
        .expect("PATH_RE_DQ") // panic-ok: regex literal
});
static PATH_RE_SQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"'(https?://[^'\s]+|/api[^'\s]*|/[A-Za-z0-9][A-Za-z0-9._/-]{1,120})'"#)
        .expect("PATH_RE_SQ") // panic-ok: regex literal
});

static FETCH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:fetch|axios\.(?:get|post|put|delete|patch)|\$\.(?:get|post))\s*\(\s*["']([^"']+)["']"#)
        .expect("FETCH_RE") // panic-ok: regex literal
});

pub fn script_srcs(base: &Url, html: &str) -> Vec<Url> {
    let document = Html::parse_document(html);
    let Ok(sel) = Selector::parse("script[src]") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for el in document.select(&sel) {
        if let Some(src) = el.value().attr("src")
            && let Some(u) = resolve_link(base, src)
        {
            out.push(u);
        }
    }
    out
}

pub fn extract_endpoints(base: &Url, js: &str) -> Vec<Url> {
    let mut out = Vec::new();

    for re in [&*PATH_RE_DQ, &*PATH_RE_SQ] {
        for cap in re.captures_iter(js) {
            let path = &cap[1];
            if looks_like_noise(path) {
                continue;
            }
            if let Some(u) = resolve_js_url(base, path) {
                out.push(u);
            }
        }
    }

    for cap in FETCH_RE.captures_iter(js) {
        let path = &cap[1];
        if let Some(u) = resolve_js_url(base, path) {
            out.push(u);
        }
    }

    dedupe(out)
}

fn resolve_js_url(base: &Url, raw: &str) -> Option<Url> {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        Url::parse(raw).ok()
    } else {
        resolve_link(base, raw)
    }
}

fn looks_like_noise(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".svg")
        || lower.ends_with(".woff")
        || lower.ends_with(".woff2")
        || lower.ends_with(".css")
        || lower.ends_with(".map")
        || s.len() > 200
        || s.contains("${")
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
    fn extracts_api_paths() {
        let base = Url::parse("https://example.com/").unwrap();
        let js = r#"const x = "/api/users"; fetch('/api/v2/items');"#;
        let eps = extract_endpoints(&base, js);
        assert!(eps.iter().any(|u| u.path() == "/api/users"));
        assert!(eps.iter().any(|u| u.path() == "/api/v2/items"));
    }
}
