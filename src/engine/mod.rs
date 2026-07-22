pub mod scope;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};
use url::Url;

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

    while let Some((url, depth)) = queue.pop_front() {
        if robots_blocked(&robots_disallow, url.path()) {
            continue;
        }

        match client.get(&url).await {
            Ok(resp) => {
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
                                response_cache.insert(script_url.as_str().to_string(), js_resp);
                            }
                        }
                    }
                }

                response_cache.insert(key, resp);
            }
            Err(e) => warn!("fetch {url}: {e}"),
        }
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
                Err(e) => tracing::debug!("wordlist miss {u}: {e}"),
            }
        }
    }

    if module_enabled(&opts.modules, "openapi") || module_enabled(&opts.modules, "discovery") {
        for candidate in discovery::openapi::candidate_urls(&seed) {
            if !authz.url_in_scope(&candidate) {
                continue;
            }
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
                    for ep in more {
                        if discovered.len() >= opts.max_urls {
                            break;
                        }
                        if authz.url_in_scope(&ep) && discovered.insert(ep.as_str().to_string()) {
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
        }
    }

    // YAML path templates (Nuclei-lite)
    if module_enabled(&opts.modules, "templates") {
        match templates::load_templates(&opts.templates_dir) {
            Ok(tmpls) => {
                info!(count = tmpls.len(), "loaded templates");
                match templates::run_templates(client.as_ref(), &seed, &tmpls, 80).await {
                    Ok(mut f) => findings.append(&mut f),
                    Err(e) => warn!("templates: {e}"),
                }
            }
            Err(e) => warn!("load templates: {e}"),
        }
    }

    for check in checks::registry() {
        if !should_run_check(check.as_ref(), &opts, authz.enable_active) {
            continue;
        }
        // auth-compare only when compare_auth is on
        if check.id() == "auth-compare" && !opts.compare_auth {
            continue;
        }
        match check.run(&ctx).await {
            Ok(mut f) => findings.append(&mut f),
            Err(e) => warn!("check {}: {e}", check.id()),
        }
    }

    findings = dedupe_findings(findings);

    let finished = Utc::now();
    let mut requests = client.request_count();
    if let Some(a) = &anon_client {
        requests += a.request_count();
    }
    let stats = ScanStats::from_findings(&findings, requests, discovered_urls.len());

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
    })
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
