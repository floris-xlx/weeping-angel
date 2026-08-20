use scraper::{Html, Selector};
use url::Url;

use crate::engine::scope::resolve_link;

pub fn extract_links(base: &Url, html: &str) -> Vec<Url> {
    let document = Html::parse_document(html);
    let mut out = Vec::new();

    let selectors = [
        ("a", "href"),
        ("link", "href"),
        ("area", "href"),
        ("form", "action"),
        ("iframe", "src"),
        ("img", "src"),
        ("img", "data-src"),
        ("img", "data-lazy-src"),
        ("source", "src"),
        ("video", "poster"),
    ];

    for (tag, attr) in selectors {
        let Ok(sel) = Selector::parse(tag) else {
            continue;
        };
        for el in document.select(&sel) {
            if let Some(href) = el.value().attr(attr)
                && let Some(u) = resolve_link(base, href)
            {
                out.push(u);
            }
        }
    }

    // srcset lists (responsive images)
    if let Ok(sel) = Selector::parse("[srcset], [data-srcset]") {
        for el in document.select(&sel) {
            for attr in ["srcset", "data-srcset"] {
                if let Some(srcset) = el.value().attr(attr) {
                    for part in srcset.split(',') {
                        let token = part.split_whitespace().next().unwrap_or("");
                        if token.is_empty() {
                            continue;
                        }
                        if let Some(u) = resolve_link(base, token) {
                            out.push(u);
                        }
                    }
                }
            }
        }
    }

    dedupe(out)
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
    fn extracts_anchor_links() {
        let base = Url::parse("https://example.com/app/").unwrap();
        let html = r#"<html><a href="/login">L</a><a href="https://example.com/api/v1">A</a><a href="https://evil.com/x">X</a></html>"#;
        let links = extract_links(&base, html);
        assert!(links.iter().any(|u| u.path() == "/login"));
        assert!(links.iter().any(|u| u.path() == "/api/v1"));
    }
}
