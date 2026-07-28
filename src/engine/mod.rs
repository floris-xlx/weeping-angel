pub mod scope;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};
use url::Url;

use crate::style;

use crate::authz::Authorization;
use crate::checks::{self, CheckKind, ScanContext};
use crate::config::Profile;
use crate::discovery::{self, DiscoveredAsset};
use crate::finding::{Finding, ScanReport, ScanStats, Severity};
use crate::http::{ClientConfig, HttpClient};
use crate::templates;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub targets: Vec<Url>,
    pub profile: Profile,
    pub modules: Vec<String>,
    pub depth: u32,
    pub max_urls: usize,
    pub ignore_robots: bool,
    pub wordlist: PathBuf,
    pub probes: Vec<String>,
    pub fail_on: Option<Severity>,
    pub templates_dir: PathBuf,
    pub compare_auth: bool,
}

pub async fn run_scan(
    authz: Authorization,
    client_cfg: ClientConfig,
    opts: ScanOptions,
) -> Result<ScanReport> {
    let started = Utc::now();
    let client = Arc::new(HttpClient::new(authz.clone(), client_cfg.clone())?);

    // Anonymous client: same scope/rate limits, no session cookie / auth headers stripped
    let anon_client = if opts.compare_auth {
        let mut anon_cfg = client_cfg;
        anon_cfg.cookie = None;
        // Drop Authorization-like headers
        anon_cfg
            .extra_headers
            .retain(|(k, _)| !k.eq_ignore_ascii_case("authorization") && !k.eq_ignore_ascii_case("cookie"));
        Some(Arc::new(HttpClient::new(authz.clone(), anon_cfg)?))
    } else {
        None
    };

    let seed = opts
        .targets
        .first()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no targets"))?;

    info!(
        target = %seed,
        profile = opts.profile.as_str(),
        "starting authorized scan"
    );
    let phase_t0 = Instant::now();
    progress(&format!(
        "scan start target={} profile={} (progress lines mean it is working; default ~5 req/s)",
        seed,
        opts.profile.as_str()
    ));

    let mut discovered: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(Url, u32)> = VecDeque::new();
    let mut assets: Vec<DiscoveredAsset> = Vec::new();
    let mut robots_disallow: Vec<String> = Vec::new();
    let mut response_cache: HashMap<String, crate::http::ResponseSnapshot> = HashMap::new();

    for t in &opts.targets {
        queue.push_back((t.clone(), 0));
        discovered.insert(t.as_str().to_string());
    }

    if module_enabled(&opts.modules, "discovery") {
        progress("phase: robots.txt / sitemaps");
        for t in &opts.targets {
            match discovery::robots::fetch_robots(&client, t).await {
                Ok(r) => {
                    if !opts.ignore_robots {
                        robots_disallow.extend(r.disallow);
                    }
                    for sm in r.sitemaps {
                        if let Ok(urls) = discovery::sitemap::fetch_sitemap(&client, &sm).await {
                            for u in urls {
                                try_enqueue(
                                    &authz,
                                    &mut discovered,
                                    &mut queue,
                                    u,
                                    0,
                                    opts.max_urls,
                                );
                            }
                        }
                    }
                }
                Err(e) => warn!("robots.txt: {e}"),
            }
        }
    }

    progress(&format!(
        "phase: crawl (queue={}, depth≤{}, requests so far={})",
        queue.len(),
        opts.depth,
        client.request_count()
    ));
    let mut crawl_done = 0usize;
    while let Some((url, depth)) = queue.pop_front() {
        if robots_blocked(&robots_disallow, url.path()) {
            continue;
        }

        match client.get(&url).await {
            Ok(resp) => {
                crawl_done += 1;
                if crawl_done == 1 || crawl_done % 10 == 0 {
                    progress(&format!(
                        "  crawl {crawl_done} fetched (queue left {}, reqs={})",
                        queue.len(),
                        client.request_count()
                    ));
                }
                let key = resp.final_url.as_str().to_string();
                assets.push(DiscoveredAsset {
                    url: resp.final_url.clone(),
                    status: resp.status.as_u16(),
                    content_type: resp.content_type.clone(),
                    source: "crawl".into(),
                });

                if module_enabled(&opts.modules, "discovery") && depth < opts.depth {
                    if resp.is_html() {
                        let links = discovery::crawl::extract_links(&resp.final_url, &resp.body);
                        for link in links {
                            try_enqueue(
                                &authz,
                                &mut discovered,
                                &mut queue,
                                link,
                                depth + 1,
                                opts.max_urls,
                            );
                        }
                        // Hosted images: img/srcset/meta/CSS (feeds pattern enumeration)
                        let images =
                            discovery::image_assets::extract_from_html(&resp.final_url, &resp.body);
                        for img in images {
                            try_enqueue(
                                &authz,
                                &mut discovered,
                                &mut queue,
                                img,
                                depth + 1,
                                opts.max_urls,
                            );
                        }
                        // SPA shells: __NEXT_DATA__, initial state, client routers
                        let spa_urls =
                            discovery::spa::extract_from_html(&resp.final_url, &resp.body);
                        for ep in spa_urls {
                            try_enqueue(
                                &authz,
                                &mut discovered,
                                &mut queue,
                                ep,
                                depth + 1,
                                opts.max_urls,
                            );
                        }
                        let scripts =
                            discovery::js_endpoints::script_srcs(&resp.final_url, &resp.body);
                        for script_url in scripts {
                            if !authz.url_in_scope(&script_url) {
                                continue;
                            }
                            if let Ok(js_resp) = client.get(&script_url).await {
                                let endpoints = discovery::js_endpoints::extract_endpoints(
                                    &script_url,
                                    &js_resp.body,
                                );
                                for ep in endpoints {
                                    try_enqueue(
                                        &authz,
                                        &mut discovered,
                                        &mut queue,
                                        ep,
                                        depth + 1,
                                        opts.max_urls,
                                    );
                                }
                                let spa_js = discovery::spa::extract_from_js(
                                    &script_url,
                                    &js_resp.body,
                                );
                                for ep in spa_js {
                                    try_enqueue(
                                        &authz,
                                        &mut discovered,
                                        &mut queue,
                                        ep,
                                        depth + 1,
                                        opts.max_urls,
                                    );
                                }
                                // Image path strings inside JS bundles (not filtered as noise)
                                let js_images = discovery::image_assets::extract_from_js(
                                    &script_url,
                                    &js_resp.body,
                                );
                                for img in js_images {
                                    try_enqueue(
                                        &authz,
                                        &mut discovered,
                                        &mut queue,
                                        img,
                                        depth + 1,
                                        opts.max_urls,
                                    );
                                }
                                response_cache.insert(script_url.as_str().to_string(), js_resp);
                            }
                        }
                    } else if discovery::image_assets::is_image_url(&resp.final_url)
                        || discovery::image_assets::is_image_path(resp.final_url.path())
                    {
                        // Hitting an image seed still allows pattern expansion later
                    }
                }

                response_cache.insert(key, resp);
            }
            Err(e) => warn!("fetch {url}: {e}"),
        }
    }
    progress(&format!(
        "phase: crawl done (assets={}, reqs={}, {:.1}s)",
        assets.len(),
        client.request_count(),
        phase_t0.elapsed().as_secs_f64()
    ));

    // Full image harvest: collect all img/srcset/CSS/JS paths → OPTIONS preflight → HEAD
    let mut image_harvest: Option<discovery::image_harvest::ImageHarvestManifest> = None;
    if module_enabled(&opts.modules, "discovery") {
        progress("phase: image harvest (collect → OPTIONS preflight → HEAD)");
        use discovery::image_harvest::{self, ImageCandidate, ImageSource};

        let mut cand_map: std::collections::HashMap<String, ImageCandidate> =
            std::collections::HashMap::new();

        // Seed URL itself if it is an image
        if discovery::image_assets::is_image_path(seed.path())
            || discovery::image_assets::is_image_url(&seed)
        {
            let mut sources = std::collections::HashSet::new();
            sources.insert(ImageSource::Seed);
            image_harvest::merge_candidates(
                &mut cand_map,
                vec![ImageCandidate {
                    url: seed.clone(),
                    sources,
                }],
            );
        }

        for a in &assets {
            if discovery::image_assets::is_image_url(&a.url)
                || discovery::image_assets::is_image_path(a.url.path())
            {
                let mut sources = std::collections::HashSet::new();
                sources.insert(if a.source == "crawl" {
                    ImageSource::Crawl
                } else if a.source == "wordlist" {
                    ImageSource::Wordlist
                } else {
                    ImageSource::Other(a.source.clone())
                });
                image_harvest::merge_candidates(
                    &mut cand_map,
                    vec![ImageCandidate {
                        url: a.url.clone(),
                        sources,
                    }],
                );
            }
        }

        for resp in response_cache.values() {
            if discovery::image_assets::is_image_path(resp.final_url.path())
                || discovery::image_assets::is_image_url(&resp.final_url)
            {
                let mut sources = std::collections::HashSet::new();
                sources.insert(ImageSource::Crawl);
                image_harvest::merge_candidates(
                    &mut cand_map,
                    vec![ImageCandidate {
                        url: resp.final_url.clone(),
                        sources,
                    }],
                );
            }
            if resp.is_html() {
                image_harvest::merge_candidates(
                    &mut cand_map,
                    image_harvest::collect_from_html(&resp.final_url, &resp.body),
                );
            }
            if resp.is_js()
                || resp
                    .content_type
                    .as_deref()
                    .map(|c| c.contains("css") || c.contains("javascript"))
                    .unwrap_or(false)
            {
                image_harvest::merge_candidates(
                    &mut cand_map,
                    image_harvest::collect_from_js(&resp.final_url, &resp.body),
                );
            }
        }

        let observed: Vec<Url> = cand_map.values().map(|c| c.url.clone()).collect();
        let observed_count = observed.len();
        let pattern_urls = discovery::image_assets::enumerate_patterns(&seed, &observed);
        for u in pattern_urls {
            if !authz.url_in_scope(&u) {
                continue;
            }
            let mut sources = std::collections::HashSet::new();
            sources.insert(ImageSource::PatternEnum);
            image_harvest::merge_candidates(
                &mut cand_map,
                vec![ImageCandidate { url: u, sources }],
            );
        }

        // Drop out-of-scope
        cand_map.retain(|_, c| authz.url_in_scope(&c.url));

        progress(&format!(
            "  image harvest: {} observed refs → {} total candidates (HEAD+OPTIONS, cap 160)",
            observed_count,
            cand_map.len()
        ));

        const MAX_IMAGE_PROBES: usize = 160;
        let harvest = image_harvest::harvest(
            client.as_ref(),
            &seed,
            cand_map.into_values(),
            MAX_IMAGE_PROBES,
            true, // OPTIONS preflight
        )
        .await;

        progress(&format!(
            "  image harvest HEAD: ok={} miss={} options_ok={} (reqs={})",
            harvest.stats.head_ok,
            harvest.stats.head_miss,
            harvest.stats.options_ok,
            client.request_count()
        ));

        // Fold successful images into discovery assets / URL set
        for img in &harvest.images {
            if !img.exists {
                continue;
            }
            if let Ok(u) = Url::parse(&img.url) {
                if discovered.insert(img.url.clone()) {
                    assets.push(DiscoveredAsset {
                        url: u,
                        status: img
                            .head
                            .as_ref()
                            .map(|h| h.status)
                            .or_else(|| img.get.as_ref().map(|g| g.status))
                            .unwrap_or(200),
                        content_type: img
                            .head
                            .as_ref()
                            .and_then(|h| h.content_type.clone())
                            .or_else(|| img.get.as_ref().and_then(|g| g.content_type.clone())),
                        source: "image-head".into(),
                    });
                }
            }
        }

        progress(&format!(
            "phase: image harvest done — {} paths, {} exist via HEAD/GET",
            harvest.all_paths.len(),
            harvest.stats.exists_total
        ));
        image_harvest = Some(harvest);
    }

    if module_enabled(&opts.modules, "wordlist") || module_enabled(&opts.modules, "exposures") {
        let origin = origin_of(&seed);
        let paths = discovery::wordlist::load_paths(&opts.wordlist).unwrap_or_default();
        let use_full_wordlist = module_enabled(&opts.modules, "wordlist");
        let paths: Vec<_> = if use_full_wordlist {
            paths
        } else {
            paths
                .into_iter()
                .filter(|p| discovery::wordlist::is_sensitive_path(p))
                .collect()
        };

        let total_paths = paths.len();
        progress(&format!(
            "phase: wordlist ({total_paths} paths; this is usually the long quiet stretch)"
        ));
        let mut probed = 0usize;
        for path in paths {
            if discovered.len() >= opts.max_urls {
                break;
            }
            let mut u = origin.clone();
            let p = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{path}")
            };
            u.set_path(&p);
            u.set_query(None);
            if robots_blocked(&robots_disallow, u.path()) {
                continue;
            }
            if discovered.contains(u.as_str()) {
                continue;
            }
            match client.get(&u).await {
                Ok(resp) => {
                    probed += 1;
                    if probed == 1 || probed % 25 == 0 || probed == total_paths {
                        progress(&format!(
                            "  wordlist {probed}/{total_paths} (hits={}, reqs={})",
                            assets.iter().filter(|a| a.source == "wordlist").count(),
                            client.request_count()
                        ));
                    }
                    let status = resp.status.as_u16();
                    if status < 400 || discovery::wordlist::is_interesting_status(status) {
                        discovered.insert(u.as_str().to_string());
                        assets.push(DiscoveredAsset {
                            url: resp.final_url.clone(),
                            status,
                            content_type: resp.content_type.clone(),
                            source: "wordlist".into(),
                        });
                        response_cache.insert(resp.final_url.as_str().to_string(), resp);
                    }
                }
                Err(e) => {
                    probed += 1;
                    if probed == 1 || probed % 25 == 0 {
                        progress(&format!(
                            "  wordlist {probed}/{total_paths} (last error, reqs={})",
                            client.request_count()
                        ));
                    }
                    tracing::debug!("wordlist miss {u}: {e}");
                }
            }
        }
        progress(&format!(
            "phase: wordlist done (probed={probed}, reqs={})",
            client.request_count()
        ));
    }

    if module_enabled(&opts.modules, "openapi") || module_enabled(&opts.modules, "discovery") {
        progress("phase: openapi candidates");
        // Cap path probing from huge OpenAPI docs so a large /openapi.yaml cannot
        // stall the scan for minutes with near-zero feedback.
        const MAX_OPENAPI_PATH_PROBES: usize = 40;
        let mut openapi_probes = 0usize;
        for candidate in discovery::openapi::candidate_urls(&seed) {
            if !authz.url_in_scope(&candidate) {
                continue;
            }
            progress(&format!("  openapi probe {}", candidate.path()));
            if let Ok(resp) = client.get(&candidate).await {
                if resp.status.is_success()
                    && (resp.is_json()
                        || resp.body.contains("openapi")
                        || resp.body.contains("swagger"))
                {
                    discovered.insert(candidate.as_str().to_string());
                    assets.push(DiscoveredAsset {
                        url: resp.final_url.clone(),
                        status: resp.status.as_u16(),
                        content_type: resp.content_type.clone(),
                        source: "openapi".into(),
                    });
                    let more = discovery::openapi::extract_paths(&seed, &resp.body);
                    let more_len = more.len();
                    if more_len > MAX_OPENAPI_PATH_PROBES {
                        progress(&format!(
                            "  openapi extracted {more_len} paths; probing first {MAX_OPENAPI_PATH_PROBES}"
                        ));
                    }
                    for ep in more.into_iter().take(MAX_OPENAPI_PATH_PROBES) {
                        if discovered.len() >= opts.max_urls {
                            break;
                        }
                        if authz.url_in_scope(&ep) && discovered.insert(ep.as_str().to_string()) {
                            openapi_probes += 1;
                            if let Ok(r2) = client.get(&ep).await {
                                assets.push(DiscoveredAsset {
                                    url: r2.final_url.clone(),
                                    status: r2.status.as_u16(),
                                    content_type: r2.content_type.clone(),
                                    source: "openapi-path".into(),
                                });
                                response_cache.insert(r2.final_url.as_str().to_string(), r2);
                            }
                        }
                    }
                    response_cache.insert(resp.final_url.as_str().to_string(), resp);
                }
            }
        }
        progress(&format!(
            "phase: openapi done (path probes={openapi_probes}, reqs={})",
            client.request_count()
        ));
    }

    let mut discovered_urls: Vec<String> = discovered.into_iter().collect();
    discovered_urls.sort();

    let ctx = ScanContext {
        client: client.clone(),
        anon_client: anon_client.clone(),
        seed: seed.clone(),
        assets: assets.clone(),
        responses: response_cache,
        discovered_urls: discovered_urls.clone(),
        probes: opts.probes.clone(),
        enable_active: authz.enable_active,
    };

    let mut findings: Vec<Finding> = Vec::new();

    if module_enabled(&opts.modules, "discovery") {
        let mut image_patterns: HashSet<String> = HashSet::new();
        for asset in &assets {
            findings.push(
                Finding::builder("discovery", "route-discovered")
                    .title(format!("Discovered route ({})", asset.source))
                    .severity(Severity::Info)
                    .url(asset.url.as_str())
                    .description(format!(
                        "URL discovered via {}. HTTP status {}.",
                        asset.source, asset.status
                    ))
                    .build(),
            );

            if asset.source == "image-pattern"
                || asset.source == "image-head"
                || discovery::image_assets::is_image_path(asset.url.path())
            {
                if let Some(info) = discovery::image_assets::describe_pattern(asset.url.path()) {
                    image_patterns.insert(info.directory.clone());
                    if (200..300).contains(&asset.status) {
                        findings.push(
                            Finding::builder("discovery", "image-asset")
                                .title(format!(
                                    "Hosted image asset ({}…/{}.{})",
                                    info.family.trim_end_matches('/'),
                                    info.stem,
                                    info.extension
                                ))
                                .severity(Severity::Info)
                                .url(asset.url.as_str())
                                .description(format!(
                                    "Image reachable via hosting pattern family `{}` \
                                     (section=`{}`, template=`{}`). Source: {}.",
                                    info.family, info.section, info.template, asset.source
                                ))
                                .build(),
                        );
                    }
                }
            }
        }
        for dir in image_patterns {
            findings.push(
                Finding::builder("discovery", "image-hosting-pattern")
                    .title(format!("Image hosting directory pattern: {dir}"))
                    .severity(Severity::Info)
                    .url(seed.join(dir.trim_start_matches('/')).unwrap_or(seed.clone()).as_str())
                    .description(format!(
                        "Enumerated static image tree under `{dir}`. \
                         Common layout example: /assets/images/home/dashboardpic.png \
                         (prefix + section + basename + extension)."
                    ))
                    .build(),
            );
        }
    }

    // YAML path templates (Nuclei-lite)
    if module_enabled(&opts.modules, "templates") {
        progress("phase: templates");
        match templates::load_templates(&opts.templates_dir) {
            Ok(tmpls) => {
                info!(count = tmpls.len(), "loaded templates");
                progress(&format!("  loaded {} templates", tmpls.len()));
                match templates::run_templates(client.as_ref(), &seed, &tmpls, 80).await {
                    Ok(mut f) => findings.append(&mut f),
                    Err(e) => warn!("templates: {e}"),
                }
            }
            Err(e) => warn!("load templates: {e}"),
        }
    }

    progress("phase: passive checks");
    for check in checks::registry() {
        if !should_run_check(check.as_ref(), &opts, authz.enable_active) {
            continue;
        }
        // auth-compare only when compare_auth is on
        if check.id() == "auth-compare" && !opts.compare_auth {
            continue;
        }
        progress(&format!("  check: {}", check.id()));
        match check.run(&ctx).await {
            Ok(mut f) => findings.append(&mut f),
            Err(e) => warn!("check {}: {e}", check.id()),
        }
    }

    // Emit findings for every HEAD-ok image path
    if let Some(ref harvest) = image_harvest {
        for img in &harvest.images {
            if !img.exists {
                continue;
            }
            let head_st = img.head.as_ref().map(|h| h.status).unwrap_or(0);
            let ct = img
                .head
                .as_ref()
                .and_then(|h| h.content_type.clone())
                .unwrap_or_else(|| "unknown".into());
            let cl = img
                .head
                .as_ref()
                .and_then(|h| h.content_length)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into());
            findings.push(
                Finding::builder("discovery", "image-head-ok")
                    .title(format!("Image HEAD ok: {}", img.path))
                    .severity(Severity::Info)
                    .url(&img.url)
                    .description(format!(
                        "HEAD status {head_st}, content-type={ct}, content-length={cl}. \
                         sources=[{}]. OPTIONS={}",
                        img.sources.join(","),
                        img.options
                            .as_ref()
                            .map(|o| format!("{}", o.status))
                            .unwrap_or_else(|| "n/a".into())
                    ))
                    .build(),
            );
        }
        findings.push(
            Finding::builder("discovery", "image-harvest-summary")
                .title(format!(
                    "Image harvest: {} paths, {} HEAD-ok, {} img-tag refs",
                    harvest.stats.candidates,
                    harvest.stats.exists_total,
                    harvest.stats.img_tag_refs
                ))
                .severity(Severity::Info)
                .url(seed.as_str())
                .description(format!(
                    "Harvested all image paths via OPTIONS preflight + HEAD. \
                     head_probes={} head_ok={} head_miss={} options_ok={}. \
                     Manifest: use --format images,manifest",
                    harvest.stats.head_probes,
                    harvest.stats.head_ok,
                    harvest.stats.head_miss,
                    harvest.stats.options_ok
                ))
                .build(),
        );
    }

    findings = dedupe_findings(findings);

    let finished = Utc::now();
    let mut requests = client.request_count();
    if let Some(a) = &anon_client {
        requests += a.request_count();
    }
    let stats = ScanStats::from_findings(&findings, requests, discovered_urls.len());
    progress(&format!(
        "scan finished in {:.1}s — {} requests, {} urls, {} findings",
        phase_t0.elapsed().as_secs_f64(),
        requests,
        discovered_urls.len(),
        findings.len()
    ));

    Ok(ScanReport {
        tool: "weeping-angel".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        target: seed.as_str().to_string(),
        started_at: started,
        finished_at: finished,
        profile: opts.profile.as_str().to_string(),
        modules: opts.modules,
        discovered_urls,
        findings,
        stats,
        image_harvest,
    })
}

/// Human-visible progress on stderr (always, not only with -v). Colored + flushed.
fn progress(msg: &str) {
    style::log_progress(msg);
    info!("{msg}");
}

fn should_run_check(
    check: &dyn checks::Check,
    opts: &ScanOptions,
    enable_active: bool,
) -> bool {
    match check.kind() {
        CheckKind::Passive => module_enabled(&opts.modules, check.id()),
        CheckKind::Active => {
            if !enable_active {
                return false;
            }
            if !opts.probes.is_empty() {
                return opts.probes.iter().any(|p| p == check.id());
            }
            // no probe filter: run all active when module "active" selected or deep profile
            module_enabled(&opts.modules, "active") || opts.profile == Profile::Deep
        }
    }
}

fn module_enabled(modules: &[String], id: &str) -> bool {
    modules.iter().any(|m| m == id)
}

fn try_enqueue(
    authz: &Authorization,
    discovered: &mut HashSet<String>,
    queue: &mut VecDeque<(Url, u32)>,
    url: Url,
    depth: u32,
    max_urls: usize,
) {
    if discovered.len() >= max_urls {
        return;
    }
    if !authz.url_in_scope(&url) {
        return;
    }
    if url.scheme() != "http" && url.scheme() != "https" {
        return;
    }
    let key = url.as_str().to_string();
    if discovered.insert(key) {
        queue.push_back((url, depth));
    }
}

fn robots_blocked(disallow: &[String], path: &str) -> bool {
    disallow.iter().any(|d| {
        if d.is_empty() {
            return false;
        }
        path.starts_with(d.as_str())
    })
}

fn origin_of(url: &Url) -> Url {
    let mut u = url.clone();
    u.set_path("/");
    u.set_query(None);
    u.set_fragment(None);
    u
}

fn dedupe_findings(findings: Vec<Finding>) -> Vec<Finding> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for f in findings {
        let key = format!("{}|{}|{}|{}", f.module, f.id, f.url, f.title);
        if seen.insert(key) {
            out.push(f);
        }
    }
    out
}
