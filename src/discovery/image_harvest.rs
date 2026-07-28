//! Full image-path harvest: collect every image reference, OPTIONS preflight,
//! HEAD existence probes, and build a structured image manifest.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::discovery::image_assets::{self, ImagePatternInfo};
use crate::http::{HttpClient, ResponseSnapshot};

/// How an image path was first observed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ImageSource {
    ImgTag,
    Srcset,
    DataSrc,
    MetaOg,
    CssUrl,
    JsBundle,
    LinkIcon,
    PatternEnum,
    Wordlist,
    Crawl,
    Seed,
    Other(String),
}

impl ImageSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ImgTag => "img-tag",
            Self::Srcset => "srcset",
            Self::DataSrc => "data-src",
            Self::MetaOg => "meta-og",
            Self::CssUrl => "css-url",
            Self::JsBundle => "js-bundle",
            Self::LinkIcon => "link-icon",
            Self::PatternEnum => "pattern-enum",
            Self::Wordlist => "wordlist",
            Self::Crawl => "crawl",
            Self::Seed => "seed",
            Self::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodProbe {
    pub method: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_ranges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// CORS / preflight related headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cors_headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<String>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl MethodProbe {
    pub fn from_response(method: &str, resp: &ResponseSnapshot) -> Self {
        let status = resp.status.as_u16();
        let content_length = resp
            .header("content-length")
            .and_then(|v| v.parse().ok())
            .or_else(|| {
                // some servers omit length on HEAD; still record body len if any
                if !resp.body.is_empty() {
                    Some(resp.body.len() as u64)
                } else {
                    None
                }
            });
        let mut cors_headers = HashMap::new();
        for (k, v) in &resp.headers {
            let kl = k.to_ascii_lowercase();
            if kl.starts_with("access-control-") || kl == "vary" && v.to_ascii_lowercase().contains("origin")
            {
                cors_headers.insert(k.clone(), v.clone());
            }
        }
        Self {
            method: method.into(),
            status,
            content_type: resp.content_type.clone().or_else(|| {
                resp.header("content-type").map(|s| s.to_string())
            }),
            content_length,
            accept_ranges: resp.header("accept-ranges").map(|s| s.to_string()),
            cache_control: resp.header("cache-control").map(|s| s.to_string()),
            etag: resp.header("etag").map(|s| s.to_string()),
            last_modified: resp.header("last-modified").map(|s| s.to_string()),
            cors_headers,
            allow: resp.header("allow").map(|s| s.to_string()),
            ok: (200..400).contains(&status),
            error: None,
        }
    }

    pub fn from_error(method: &str, err: &str) -> Self {
        Self {
            method: method.into(),
            status: 0,
            content_type: None,
            content_length: None,
            accept_ranges: None,
            cache_control: None,
            etag: None,
            last_modified: None,
            cors_headers: HashMap::new(),
            allow: None,
            ok: false,
            error: Some(err.chars().take(200).collect()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestedImage {
    pub url: String,
    pub path: String,
    pub sources: Vec<String>,
    /// True when HEAD (or GET fallback) returned 2xx/3xx with image CT or image path
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<MethodProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MethodProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<MethodProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<ImagePatternSerde>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePatternSerde {
    pub directory: String,
    pub section: String,
    pub stem: String,
    pub extension: String,
    pub family: String,
    pub template: String,
}

impl From<&ImagePatternInfo> for ImagePatternSerde {
    fn from(i: &ImagePatternInfo) -> Self {
        Self {
            directory: i.directory.clone(),
            section: i.section.clone(),
            stem: i.stem.clone(),
            extension: i.extension.clone(),
            family: i.family.clone(),
            template: i.template.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageHarvestManifest {
    pub tool: String,
    pub version: String,
    pub target: String,
    pub generated_at: String,
    /// Every unique image path harvested (exists or not)
    pub all_paths: Vec<String>,
    /// Paths that returned success on HEAD
    pub head_ok_paths: Vec<String>,
    /// Paths that failed HEAD / 404
    pub head_miss_paths: Vec<String>,
    pub images: Vec<HarvestedImage>,
    pub stats: ImageHarvestStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageHarvestStats {
    pub candidates: usize,
    pub head_probes: usize,
    pub head_ok: usize,
    pub head_miss: usize,
    pub options_probes: usize,
    pub options_ok: usize,
    pub img_tag_refs: usize,
    pub pattern_enum: usize,
    pub exists_total: usize,
}

/// Candidate with provenance before probing.
#[derive(Debug, Clone)]
pub struct ImageCandidate {
    pub url: Url,
    pub sources: HashSet<ImageSource>,
}

/// Collect image URLs from HTML with source labels.
pub fn collect_from_html(base: &Url, html: &str) -> Vec<ImageCandidate> {
    use scraper::{Html, Selector};
    use once_cell::sync::Lazy;
    use regex::Regex;

    static CSS_URL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(?i)url\(\s*['"]?([^'")\s]+)['"]?\s*\)"#).unwrap());
    static QUOTED_IMAGE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?i)["']((?:https?://[^"']+|/[^"']+)\.(?:png|jpe?g|webp|gif|svg|avif|ico)(?:\?[^"']*)?)["']"#,
        )
        .unwrap()
    });

    let mut map: HashMap<String, ImageCandidate> = HashMap::new();
    let document = Html::parse_document(html);

    let push = |map: &mut HashMap<String, ImageCandidate>, raw: &str, src: ImageSource| {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with("data:") || raw.starts_with("blob:") {
            return;
        }
        let u = if raw.starts_with("http://") || raw.starts_with("https://") {
            Url::parse(raw).ok()
        } else {
            crate::engine::scope::resolve_link(base, raw)
        };
        let Some(url) = u else {
            return;
        };
        if !(image_assets::is_image_path(url.path()) || image_assets::is_image_url(&url)) {
            // still keep if looks like image path-ish or srcset token without ext (skip)
            if !url.path().contains("/_next/image") {
                return;
            }
        }
        let key = url.as_str().to_string();
        map.entry(key)
            .and_modify(|c| {
                c.sources.insert(src.clone());
            })
            .or_insert_with(|| {
                let mut sources = HashSet::new();
                sources.insert(src);
                ImageCandidate { url, sources }
            });
    };

    if let Ok(sel) = Selector::parse("img") {
        for el in document.select(&sel) {
            if let Some(s) = el.value().attr("src") {
                push(&mut map, s, ImageSource::ImgTag);
            }
            for attr in ["data-src", "data-lazy-src", "data-original"] {
                if let Some(s) = el.value().attr(attr) {
                    push(&mut map, s, ImageSource::DataSrc);
                }
            }
            for attr in ["srcset", "data-srcset"] {
                if let Some(srcset) = el.value().attr(attr) {
                    for part in srcset.split(',') {
                        if let Some(token) = part.split_whitespace().next() {
                            push(&mut map, token, ImageSource::Srcset);
                        }
                    }
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("source[srcset], source[src], source[data-src]") {
        for el in document.select(&sel) {
            if let Some(s) = el.value().attr("src") {
                push(&mut map, s, ImageSource::Srcset);
            }
            if let Some(s) = el.value().attr("data-src") {
                push(&mut map, s, ImageSource::DataSrc);
            }
            if let Some(srcset) = el.value().attr("srcset") {
                for part in srcset.split(',') {
                    if let Some(token) = part.split_whitespace().next() {
                        push(&mut map, token, ImageSource::Srcset);
                    }
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("meta[property], meta[name]") {
        for el in document.select(&sel) {
            let prop = el
                .value()
                .attr("property")
                .or_else(|| el.value().attr("name"))
                .unwrap_or("")
                .to_ascii_lowercase();
            if prop.contains("image") || prop.contains("thumbnail") {
                if let Some(c) = el.value().attr("content") {
                    push(&mut map, c, ImageSource::MetaOg);
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("link[rel]") {
        for el in document.select(&sel) {
            let rel = el.value().attr("rel").unwrap_or("").to_ascii_lowercase();
            if rel.contains("icon") || rel.contains("apple-touch") || rel.contains("image_src") {
                if let Some(h) = el.value().attr("href") {
                    push(&mut map, h, ImageSource::LinkIcon);
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("[style]") {
        for el in document.select(&sel) {
            if let Some(style) = el.value().attr("style") {
                for cap in CSS_URL_RE.captures_iter(style) {
                    push(&mut map, &cap[1], ImageSource::CssUrl);
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("style") {
        for el in document.select(&sel) {
            let text = el.text().collect::<String>();
            for cap in CSS_URL_RE.captures_iter(&text) {
                push(&mut map, &cap[1], ImageSource::CssUrl);
            }
        }
    }
    for cap in QUOTED_IMAGE_RE.captures_iter(html) {
        push(&mut map, &cap[1], ImageSource::Other("html-quoted".into()));
    }

    map.into_values().collect()
}

pub fn collect_from_js(base: &Url, body: &str) -> Vec<ImageCandidate> {
    image_assets::extract_from_js(base, body)
        .into_iter()
        .map(|url| {
            let mut sources = HashSet::new();
            sources.insert(ImageSource::JsBundle);
            ImageCandidate { url, sources }
        })
        .collect()
}

pub fn merge_candidates(into: &mut HashMap<String, ImageCandidate>, extra: Vec<ImageCandidate>) {
    for c in extra {
        let key = c.url.as_str().to_string();
        into.entry(key)
            .and_modify(|e| e.sources.extend(c.sources.iter().cloned()))
            .or_insert(c);
    }
}

/// Probe each candidate: OPTIONS preflight → HEAD (primary harvest signal).
/// Optionally light GET only when HEAD is ambiguous (405/501).
pub async fn harvest(
    client: &HttpClient,
    seed: &Url,
    candidates: impl IntoIterator<Item = ImageCandidate>,
    max_probes: usize,
    do_options: bool,
) -> ImageHarvestManifest {
    let origin = format!(
        "{}://{}",
        seed.scheme(),
        seed.host_str().unwrap_or("localhost")
    );
    let origin = if let Some(port) = seed.port() {
        format!("{origin}:{port}")
    } else {
        origin
    };

    let mut by_url: HashMap<String, ImageCandidate> = HashMap::new();
    for c in candidates {
        merge_candidates(&mut by_url, vec![c]);
    }

    // Stable order: img-tag first, then others, then pattern-enum last
    let mut list: Vec<ImageCandidate> = by_url.into_values().collect();
    list.sort_by(|a, b| {
        let score = |c: &ImageCandidate| {
            if c.sources.contains(&ImageSource::ImgTag) {
                0
            } else if c.sources.contains(&ImageSource::Srcset)
                || c.sources.contains(&ImageSource::DataSrc)
            {
                1
            } else if c.sources.contains(&ImageSource::PatternEnum) {
                3
            } else {
                2
            }
        };
        score(a)
            .cmp(&score(b))
            .then_with(|| a.url.as_str().cmp(b.url.as_str()))
    });

    let mut stats = ImageHarvestStats {
        candidates: list.len(),
        ..Default::default()
    };
    stats.img_tag_refs = list
        .iter()
        .filter(|c| c.sources.contains(&ImageSource::ImgTag))
        .count();
    stats.pattern_enum = list
        .iter()
        .filter(|c| c.sources.contains(&ImageSource::PatternEnum))
        .count();

    let mut images: Vec<HarvestedImage> = Vec::new();
    let mut head_ok_paths = Vec::new();
    let mut head_miss_paths = Vec::new();
    let mut all_paths = Vec::new();
    let mut probed = 0usize;

    for cand in list {
        if probed >= max_probes {
            // still record path without probe if over budget? record unprobed as path-only
            all_paths.push(cand.url.path().to_string());
            images.push(HarvestedImage {
                url: cand.url.as_str().into(),
                path: cand.url.path().into(),
                sources: cand.sources.iter().map(|s| s.as_str().to_string()).collect(),
                exists: false,
                head: None,
                options: None,
                get: None,
                pattern: image_assets::describe_pattern(cand.url.path()).as_ref().map(Into::into),
            });
            continue;
        }

        all_paths.push(cand.url.path().to_string());
        let sources: Vec<String> = {
            let mut s: Vec<_> = cand.sources.iter().map(|s| s.as_str().to_string()).collect();
            s.sort();
            s.dedup();
            s
        };
        let pattern = image_assets::describe_pattern(cand.url.path())
            .as_ref()
            .map(Into::into);

        let mut options_probe = None;
        if do_options {
            stats.options_probes += 1;
            match client
                .options(&cand.url, Some(&origin), "GET")
                .await
            {
                Ok(resp) => {
                    let p = MethodProbe::from_response("OPTIONS", &resp);
                    if p.ok {
                        stats.options_ok += 1;
                    }
                    options_probe = Some(p);
                }
                Err(e) => {
                    options_probe = Some(MethodProbe::from_error("OPTIONS", &e.to_string()));
                }
            }
        }

        stats.head_probes += 1;
        probed += 1;
        let (head_probe, mut exists, need_get) = match client.head(&cand.url).await {
            Ok(resp) => {
                let p = MethodProbe::from_response("HEAD", &resp);
                let ct_img = p
                    .content_type
                    .as_deref()
                    .map(|c| c.to_ascii_lowercase().starts_with("image/"))
                    .unwrap_or(false);
                let path_img = image_assets::is_image_path(cand.url.path());
                let ok = p.ok && (ct_img || path_img || p.status == 200 || (200..300).contains(&p.status));
                // Some stacks return 405 Method Not Allowed on HEAD for static files
                let need_get = p.status == 405 || p.status == 501 || p.status == 403 && path_img;
                (Some(p), ok, need_get)
            }
            Err(e) => (Some(MethodProbe::from_error("HEAD", &e.to_string())), false, true),
        };

        let mut get_probe = None;
        if need_get && !exists {
            match client.get(&cand.url).await {
                Ok(resp) => {
                    let p = MethodProbe::from_response("GET", &resp);
                    let ct_img = p
                        .content_type
                        .as_deref()
                        .map(|c| c.to_ascii_lowercase().starts_with("image/"))
                        .unwrap_or(false);
                    if p.ok && (ct_img || image_assets::is_image_path(cand.url.path())) {
                        exists = true;
                    }
                    get_probe = Some(p);
                }
                Err(e) => {
                    get_probe = Some(MethodProbe::from_error("GET", &e.to_string()));
                }
            }
        }

        if exists {
            stats.head_ok += 1;
            stats.exists_total += 1;
            head_ok_paths.push(cand.url.path().to_string());
        } else {
            stats.head_miss += 1;
            head_miss_paths.push(cand.url.path().to_string());
        }

        images.push(HarvestedImage {
            url: cand.url.as_str().into(),
            path: cand.url.path().into(),
            sources,
            exists,
            head: head_probe,
            options: options_probe,
            get: get_probe,
            pattern,
        });
    }

    all_paths.sort();
    all_paths.dedup();
    head_ok_paths.sort();
    head_ok_paths.dedup();
    head_miss_paths.sort();
    head_miss_paths.dedup();

    // Prefer existing images first in manifest
    images.sort_by(|a, b| {
        b.exists
            .cmp(&a.exists)
            .then_with(|| a.path.cmp(&b.path))
    });

    ImageHarvestManifest {
        tool: "weeping-angel".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        target: seed.as_str().into(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        all_paths,
        head_ok_paths,
        head_miss_paths,
        images,
        stats,
    }
}

pub fn to_string(m: &ImageHarvestManifest) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(m)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_img_tags_with_source() {
        let base = Url::parse("https://depotfox.com/").unwrap();
        let html = r#"<img src="/assets/images/home/dashboardpic.png" data-src="/assets/images/home/lazy.png" />"#;
        let c = collect_from_html(&base, html);
        assert!(c.iter().any(|x| {
            x.url.path() == "/assets/images/home/dashboardpic.png"
                && x.sources.contains(&ImageSource::ImgTag)
        }));
        assert!(c.iter().any(|x| {
            x.url.path() == "/assets/images/home/lazy.png"
                && x.sources.contains(&ImageSource::DataSrc)
        }));
    }
}
