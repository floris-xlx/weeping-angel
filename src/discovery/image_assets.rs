//! Enumerate static / hosting image URL patterns.
//!
//! Given seeds like `https://depotfox.com/assets/images/home/dashboardpic.png`,
//! extract observed image URLs from HTML/JS and expand common hosting layouts:
//! `/assets/images/{section}/{name}.{ext}`, `/static/…`, `/img/…`, `/media/…`, etc.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::engine::scope::resolve_link;

/// Max candidate image URLs to generate from pattern expansion (before scope filter).
pub const MAX_ENUMERATED: usize = 200;

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "svg", "gif", "avif", "ico"];

/// Common path prefixes used by app hosts / CDNs for marketing & product imagery.
const HOSTING_PREFIXES: &[&str] = &[
    "/assets/images/",
    "/assets/img/",
    "/assets/media/",
    "/static/images/",
    "/static/img/",
    "/static/media/",
    "/images/",
    "/img/",
    "/media/",
    "/media/images/",
    "/uploads/",
    "/uploads/images/",
    "/content/images/",
    "/content/assets/",
    "/public/images/",
    "/public/assets/images/",
    "/_next/static/media/",
    "/_next/image",
    "/cdn/images/",
    "/files/images/",
    "/storage/images/",
    "/wp-content/uploads/",
];

/// Typical first-level folders under `/assets/images/`-style trees.
const SECTIONS: &[&str] = &[
    "home",
    "landing",
    "hero",
    "marketing",
    "product",
    "products",
    "features",
    "feature",
    "about",
    "blog",
    "docs",
    "dashboard",
    "app",
    "auth",
    "login",
    "signup",
    "pricing",
    "icons",
    "logo",
    "logos",
    "screenshots",
    "og",
    "social",
    "backgrounds",
    "bg",
    "misc",
    "common",
    "shared",
];

/// Basenames frequently used next to dashboard/hero marketing shots.
const BASENAMES: &[&str] = &[
    "dashboardpic",
    "dashboard",
    "dashboard-pic",
    "dashboard_pic",
    "dashboard-preview",
    "dashboard-screenshot",
    "hero",
    "hero-image",
    "banner",
    "preview",
    "screenshot",
    "screenshot-1",
    "screenshot1",
    "logo",
    "logo-dark",
    "logo-light",
    "favicon",
    "og",
    "og-image",
    "opengraph",
    "twitter",
    "feature",
    "feature-1",
    "features",
    "bg",
    "background",
    "cover",
    "thumbnail",
    "thumb",
    "placeholder",
    "avatar",
    "icon",
    "app-preview",
    "ui",
    "ui-preview",
    "mockup",
    "product",
    "main",
    "index",
    "home",
];

static CSS_URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)url\(\s*['"]?([^'")\s]+)['"]?\s*\)"#).unwrap());

static QUOTED_IMAGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)["']((?:https?://[^"']+|/[^"']+)\.(?:png|jpe?g|webp|gif|svg|avif|ico)(?:\?[^"']*)?)["']"#,
    )
    .unwrap()
});

static SRCSET_PART_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(\S+\.(?:png|jpe?g|webp|gif|svg|avif)(?:\?[^\s,]*)?)"#).unwrap()
});

/// Extract image URLs referenced from HTML (img, srcset, meta, link, inline CSS).
pub fn extract_from_html(base: &Url, html: &str) -> Vec<Url> {
    let document = Html::parse_document(html);
    let mut out = Vec::new();

    let tag_attrs = [
        ("img", "src"),
        ("img", "data-src"),
        ("img", "data-lazy-src"),
        ("img", "data-original"),
        ("source", "src"),
        ("source", "data-src"),
        ("video", "poster"),
        ("image", "href"), // SVG
        ("meta", "content"),
        ("link", "href"),
        ("use", "href"),
    ];

    for (tag, attr) in tag_attrs {
        let Ok(sel) = Selector::parse(tag) else {
            continue;
        };
        for el in document.select(&sel) {
            let Some(val) = el.value().attr(attr) else {
                continue;
            };
            // meta: only image-ish properties
            if tag == "meta" {
                let prop = el
                    .value()
                    .attr("property")
                    .or_else(|| el.value().attr("name"))
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if !(prop.contains("image")
                    || prop.contains("thumbnail")
                    || prop == "twitter:image")
                {
                    continue;
                }
            }
            if tag == "link" {
                let rel = el.value().attr("rel").unwrap_or("").to_ascii_lowercase();
                if !(rel.contains("icon")
                    || rel.contains("apple-touch")
                    || rel.contains("image_src")
                    || is_image_path(val))
                {
                    continue;
                }
            }
            push_resolved(&mut out, base, val);
        }
    }

    // srcset / data-srcset
    if let Ok(sel) = Selector::parse("[srcset], [data-srcset]") {
        for el in document.select(&sel) {
            for attr in ["srcset", "data-srcset"] {
                if let Some(srcset) = el.value().attr(attr) {
                    for cap in SRCSET_PART_RE.captures_iter(srcset) {
                        push_resolved(&mut out, base, &cap[1]);
                    }
                }
            }
        }
    }

    // style="background-image: url(...)" and inline style blocks
    if let Ok(sel) = Selector::parse("[style]") {
        for el in document.select(&sel) {
            if let Some(style) = el.value().attr("style") {
                for cap in CSS_URL_RE.captures_iter(style) {
                    let u = &cap[1];
                    if is_image_path(u) || u.starts_with("data:") {
                        if !u.starts_with("data:") {
                            push_resolved(&mut out, base, u);
                        }
                    }
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("style") {
        for el in document.select(&sel) {
            let text = el.text().collect::<String>();
            for cap in CSS_URL_RE.captures_iter(&text) {
                let u = &cap[1];
                if is_image_path(u) {
                    push_resolved(&mut out, base, u);
                }
            }
        }
    }

    // raw quoted image paths anywhere in HTML
    for cap in QUOTED_IMAGE_RE.captures_iter(html) {
        push_resolved(&mut out, base, &cap[1]);
    }

    dedupe(out)
}

/// Extract image path strings from JS/CSS bundles (not treated as noise).
pub fn extract_from_js(base: &Url, body: &str) -> Vec<Url> {
    let mut out = Vec::new();
    for cap in QUOTED_IMAGE_RE.captures_iter(body) {
        push_resolved(&mut out, base, &cap[1]);
    }
    for cap in CSS_URL_RE.captures_iter(body) {
        let u = &cap[1];
        if is_image_path(u) {
            push_resolved(&mut out, base, u);
        }
    }
    dedupe(out)
}

/// Expand observed image URLs into sibling candidates using hosting patterns.
pub fn enumerate_patterns(seed: &Url, observed: &[Url]) -> Vec<Url> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Seed itself may be an image URL
    let mut seeds: Vec<Url> = Vec::new();
    if is_image_path(seed.path()) || is_image_url(seed) {
        seeds.push(seed.clone());
    }
    for u in observed {
        if is_image_url(u) || is_image_path(u.path()) {
            seeds.push(u.clone());
        }
    }

    for img in &seeds {
        push_unique(&mut out, &mut seen, img.clone());
        for cand in expand_from_observed(seed, img) {
            if out.len() >= MAX_ENUMERATED {
                return out;
            }
            push_unique(&mut out, &mut seen, cand);
        }
    }

    // Always add generic hosting prefix probes under the origin (lightweight basename set)
    if out.len() < MAX_ENUMERATED {
        for prefix in HOSTING_PREFIXES {
            if prefix.contains("_next/image") {
                continue; // query-based; skip blind probe
            }
            for name in [
                "logo",
                "hero",
                "banner",
                "dashboard",
                "dashboardpic",
                "og-image",
                "favicon",
            ] {
                for ext in ["png", "jpg", "webp", "svg"] {
                    // Prefer sectioned tree for assets/images
                    if prefix.ends_with("images/") || prefix.ends_with("img/") {
                        for section in ["home", "hero", "common", "logo"] {
                            if let Some(u) =
                                join_path(seed, &format!("{prefix}{section}/{name}.{ext}"))
                            {
                                push_unique(&mut out, &mut seen, u);
                            }
                            if out.len() >= MAX_ENUMERATED {
                                return out;
                            }
                        }
                    }
                    if let Some(u) = join_path(seed, &format!("{prefix}{name}.{ext}")) {
                        push_unique(&mut out, &mut seen, u);
                    }
                    if out.len() >= MAX_ENUMERATED {
                        return out;
                    }
                }
            }
        }
    }

    out
}

/// Describe hosting pattern families inferred from a path (for findings/manifest).
pub fn describe_pattern(path: &str) -> Option<ImagePatternInfo> {
    let path = path.split('?').next().unwrap_or(path);
    if !is_image_path(path) {
        return None;
    }
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let dir = path_parent(path);
    let file = path.rsplit('/').next().unwrap_or("");
    let stem = file
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(file)
        .to_string();
    let section = dir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    let family = HOSTING_PREFIXES
        .iter()
        .find(|p| path.starts_with(*p) || path.contains(p.trim_start_matches('/')))
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if dir.is_empty() {
                "/".into()
            } else {
                format!("{dir}/")
            }
        });

    Some(ImagePatternInfo {
        path: path.to_string(),
        directory: if dir.is_empty() {
            "/".into()
        } else {
            format!("{dir}/")
        },
        section,
        stem,
        extension: ext,
        family,
        template: if dir.is_empty() {
            "/{name}.{ext}".into()
        } else {
            format!("{dir}/{{name}}.{{ext}}")
        },
    })
}

#[derive(Debug, Clone)]
pub struct ImagePatternInfo {
    pub path: String,
    pub directory: String,
    pub section: String,
    pub stem: String,
    pub extension: String,
    pub family: String,
    pub template: String,
}

fn expand_from_observed(origin_seed: &Url, img: &Url) -> Vec<Url> {
    // Priority order matters: probe budget is limited, so high-signal paths first.
    let mut priority: Vec<Url> = Vec::new();
    let mut later: Vec<Url> = Vec::new();
    let path = img.path();
    let Some(info) = describe_pattern(path) else {
        return priority;
    };

    let dir = info.directory.trim_end_matches('/');
    let parent = path_parent(dir);
    let common_exts = ["png", "jpg", "webp", "svg"];

    // 1) Same stem, alternate extensions + responsive/dark variants (png/webp first)
    for ext in common_exts {
        if let Some(u) = join_path(origin_seed, &format!("{dir}/{}.{ext}", info.stem)) {
            priority.push(u);
        }
    }
    for suffix in ["@2x", "@3x", "-dark", "-light", "-mobile", "-desktop"] {
        for ext in ["png", "webp"] {
            if let Some(u) = join_path(origin_seed, &format!("{dir}/{}{suffix}.{ext}", info.stem)) {
                priority.push(u);
            }
        }
    }

    // 2) Same directory: high-value basenames (primary exts only)
    const PRIORITY_NAMES: &[&str] = &[
        "dashboardpic",
        "dashboard",
        "hero",
        "banner",
        "logo",
        "og",
        "og-image",
        "preview",
        "screenshot",
        "feature",
        "mockup",
        "ui",
        "app-preview",
        "favicon",
    ];
    for base in PRIORITY_NAMES {
        for ext in common_exts {
            if let Some(u) = join_path(origin_seed, &format!("{dir}/{base}.{ext}")) {
                priority.push(u);
            }
        }
    }

    // 3) Sibling sections under parent (…/images/{section}/…) — compact set
    const PRIORITY_SECTIONS: &[&str] = &[
        "home",
        "hero",
        "landing",
        "marketing",
        "product",
        "features",
        "dashboard",
        "logo",
        "screenshots",
        "og",
        "common",
        "shared",
    ];
    if !parent.is_empty() {
        for section in PRIORITY_SECTIONS {
            for name in [info.stem.as_str(), "dashboardpic", "hero", "logo"] {
                for ext in ["png", "webp"] {
                    if let Some(u) =
                        join_path(origin_seed, &format!("{parent}/{section}/{name}.{ext}"))
                    {
                        priority.push(u);
                    }
                }
            }
        }
    }

    // 4) Alternate hosting prefixes (depotfox-style → /static/images, /img, …)
    //    Interleaved early so probe budget still covers them.
    let section = if info.section.is_empty() {
        "home"
    } else {
        info.section.as_str()
    };
    for prefix in HOSTING_PREFIXES {
        if prefix.contains("_next/image") {
            continue;
        }
        for name in [info.stem.as_str(), "dashboardpic", "hero", "logo"] {
            if let Some(u) = join_path(origin_seed, &format!("{prefix}{section}/{name}.png")) {
                priority.push(u);
            }
            if let Some(u) = join_path(origin_seed, &format!("{prefix}{name}.png")) {
                priority.push(u);
            }
            if let Some(u) = join_path(origin_seed, &format!("{prefix}{section}/{name}.webp")) {
                later.push(u);
            }
        }
    }

    // Remaining sections (lower priority)
    if !parent.is_empty() {
        for section in SECTIONS {
            if PRIORITY_SECTIONS.contains(section) {
                continue;
            }
            if let Some(u) = join_path(
                origin_seed,
                &format!("{parent}/{section}/{}.png", info.stem),
            ) {
                later.push(u);
            }
        }
    }

    // 5) Remaining basenames (lower priority)
    for base in BASENAMES {
        if PRIORITY_NAMES.contains(base) {
            continue;
        }
        for ext in ["png", "webp"] {
            if let Some(u) = join_path(origin_seed, &format!("{dir}/{base}.{ext}")) {
                later.push(u);
            }
        }
    }

    priority.extend(later);
    priority
}

fn push_resolved(out: &mut Vec<Url>, base: &Url, raw: &str) {
    let raw = raw.trim();
    if raw.is_empty() || raw.starts_with("data:") || raw.starts_with("blob:") {
        return;
    }
    if let Some(u) = resolve_link(base, raw) {
        out.push(u);
    } else if raw.starts_with("http://") || raw.starts_with("https://") {
        if let Ok(u) = Url::parse(raw) {
            out.push(u);
        }
    }
}

fn push_unique(out: &mut Vec<Url>, seen: &mut std::collections::HashSet<String>, u: Url) {
    if seen.insert(u.as_str().to_string()) {
        out.push(u);
    }
}

fn join_path(seed: &Url, path: &str) -> Option<Url> {
    let mut u = seed.clone();
    let p = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    u.set_path(&p);
    u.set_query(None);
    u.set_fragment(None);
    Some(u)
}

fn path_parent(path: &str) -> String {
    let path = path.trim_end_matches('/');
    match path.rfind('/') {
        Some(0) => "".into(),
        Some(i) => path[..i].to_string(),
        None => "".into(),
    }
}

pub fn is_image_path(s: &str) -> bool {
    let lower = s.split('?').next().unwrap_or(s).to_ascii_lowercase();
    IMAGE_EXTS.iter().any(|e| lower.ends_with(&format!(".{e}")))
}

pub fn is_image_url(u: &Url) -> bool {
    is_image_path(u.path())
        || u.path().contains("/_next/image")
        || u.query_pairs()
            .any(|(k, v)| k == "url" && is_image_path(&v))
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
    fn extracts_img_and_srcset() {
        let base = Url::parse("https://depotfox.com/").unwrap();
        let html = r#"
          <img src="/assets/images/home/dashboardpic.png" />
          <img srcset="/assets/images/home/hero.png 1x, /assets/images/home/hero@2x.png 2x" />
          <meta property="og:image" content="https://depotfox.com/assets/images/home/og.png" />
        "#;
        let urls = extract_from_html(&base, html);
        assert!(
            urls.iter()
                .any(|u| u.path() == "/assets/images/home/dashboardpic.png"),
            "{urls:?}"
        );
        assert!(
            urls.iter()
                .any(|u| u.path() == "/assets/images/home/hero.png")
        );
        assert!(
            urls.iter()
                .any(|u| u.path() == "/assets/images/home/og.png")
        );
    }

    #[test]
    fn enumerates_depotfox_style_pattern() {
        let seed = Url::parse("https://depotfox.com/assets/images/home/dashboardpic.png").unwrap();
        let cands = enumerate_patterns(&seed, &[]);
        assert!(
            cands
                .iter()
                .any(|u| u.path() == "/assets/images/home/dashboardpic.webp"),
            "expected extension swap, sample={:?}",
            cands
                .iter()
                .take(12)
                .map(|u| u.path().to_string())
                .collect::<Vec<_>>()
        );
        assert!(
            cands
                .iter()
                .any(|u| u.path() == "/assets/images/home/hero.png"),
            "expected same-dir basename expansion"
        );
        assert!(
            cands
                .iter()
                .any(|u| u.path().starts_with("/assets/images/hero/")
                    || u.path().starts_with("/assets/images/landing/")),
            "expected sibling section expansion, sample={:?}",
            cands
                .iter()
                .filter(|u| u.path().contains("/assets/images/"))
                .take(20)
                .map(|u| u.path().to_string())
                .collect::<Vec<_>>()
        );
        assert!(
            cands.iter().any(|u| u.path().starts_with("/static/images/")
                || u.path().starts_with("/images/")
                || u.path().starts_with("/img/")),
            "expected alternate hosting prefixes"
        );
        let info = describe_pattern("/assets/images/home/dashboardpic.png").unwrap();
        assert_eq!(info.section, "home");
        assert_eq!(info.stem, "dashboardpic");
        assert_eq!(info.extension, "png");
        assert!(info.family.contains("assets/images") || info.directory.contains("assets/images"));
    }

    #[test]
    fn extracts_from_js_quoted_images() {
        let base = Url::parse("https://depotfox.com/").unwrap();
        let js = r#"const img = "/assets/images/home/dashboardpic.png"; background: url('/static/images/logo.svg')"#;
        let urls = extract_from_js(&base, js);
        assert!(urls.iter().any(|u| u.path().contains("dashboardpic")));
        assert!(urls.iter().any(|u| u.path().contains("logo.svg")));
    }
}
