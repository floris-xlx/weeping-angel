//! Route / surface manifest generated from scan discovery + findings.

use serde::Serialize;
use url::Url;

use crate::finding::{Finding, ScanReport};

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceManifest {
    pub tool: String,
    pub version: String,
    pub target: String,
    pub generated_at: String,
    pub routes: Vec<ManifestRoute>,
    pub auth_surfaces: Vec<AuthSurfaceEntry>,
    pub tech: Vec<String>,
    pub firebase: FirebaseManifest,
    pub rate_limits: RateLimitManifest,
    /// Embedded image harvest (paths + HEAD/OPTIONS) when available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<crate::discovery::image_harvest::ImageHarvestManifest>,
    pub stats: ManifestStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManifestRoute {
    pub url: String,
    pub path: String,
    pub method: String,
    pub status: Option<u16>,
    pub source: Option<String>,
    pub content_type: Option<String>,
    pub auth: AuthGuess,
    pub rate_limit: RateLimitGuess,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthGuess {
    Unknown,
    /// Successfully fetched without session (public)
    UnauthenticatedOk,
    /// 401/403 without session
    AuthRequired,
    /// Login/signup form publicly reachable
    PublicAuthForm,
    /// Sensitive data reachable without auth (weak)
    UnauthenticatedSensitive,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitGuess {
    Unknown,
    HeadersPresent,
    Http429,
    NoSignal,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthSurfaceEntry {
    pub kind: String,
    pub url: String,
    pub status: String,
    pub guarded: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FirebaseManifest {
    pub detected: bool,
    pub project_ids: Vec<String>,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RateLimitManifest {
    pub routes_with_signals: Vec<String>,
    pub routes_without_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManifestStats {
    pub route_count: usize,
    pub finding_count: usize,
    pub auth_surfaces: usize,
    #[serde(default)]
    pub by_severity: crate::finding::SeverityCounts,
}

pub fn from_report(report: &ScanReport) -> SurfaceManifest {
    let mut tech = Vec::new();
    let mut firebase = FirebaseManifest::default();
    let mut rate_limits = RateLimitManifest::default();
    let mut auth_surfaces = Vec::new();

    for f in &report.findings {
        match f.module.as_str() {
            "tech" => {
                if !tech.iter().any(|t| t == &f.title) {
                    tech.push(f.title.clone());
                }
            }
            "firebase" => {
                firebase.detected = true;
                firebase.signals.push(format!("{}: {}", f.id, f.title));
                if f.id == "firebase-project-id" {
                    if let Some(pid) = f.evidence.iter().find(|e| e.location == "projectId") {
                        if !firebase.project_ids.contains(&pid.snippet) {
                            firebase.project_ids.push(pid.snippet.clone());
                        }
                    }
                }
            }
            "rate-limits" => match f.id.as_str() {
                "rate-limit-headers" | "http-429" | "burst-triggered-429" => {
                    if !rate_limits.routes_with_signals.contains(&f.url) {
                        rate_limits.routes_with_signals.push(f.url.clone());
                    }
                }
                "no-rate-limit-signal" | "burst-no-throttle" => {
                    if !rate_limits.routes_without_signals.contains(&f.url) {
                        rate_limits.routes_without_signals.push(f.url.clone());
                    }
                }
                _ => {}
            },
            "auth-surface" => {
                if matches!(
                    f.id.as_str(),
                    "login-form"
                        | "login-unguarded"
                        | "login-guarded"
                        | "signup-form"
                        | "signup-unguarded"
                        | "signup-guarded"
                        | "auth-endpoint-unauthenticated"
                        | "auth-endpoint-protected"
                        | "admin-200"
                        | "admin-login"
                ) {
                    let guarded = f.id.contains("guarded") && !f.id.contains("unguarded")
                        || f.id.contains("protected");
                    auth_surfaces.push(AuthSurfaceEntry {
                        kind: f.id.clone(),
                        url: f.url.clone(),
                        status: if guarded {
                            "guarded".into()
                        } else if f.id.contains("unguarded")
                            || f.id.contains("unauthenticated")
                            || f.id == "login-form"
                            || f.id == "signup-form"
                            || f.id == "admin-200"
                        {
                            "unauthenticated_or_public".into()
                        } else {
                            "info".into()
                        },
                        guarded,
                        detail: f.description.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    // Prefer structured RouteRecord; fall back to discovered_urls only if empty.
    let mut routes: Vec<ManifestRoute> = Vec::new();
    if !report.routes.is_empty() {
        for r in &report.routes {
            let mut tags = r.tags.clone();
            if tags.is_empty() {
                tags = classify_tags(&r.url, &report.findings);
            }
            routes.push(ManifestRoute {
                url: r.url.clone(),
                path: r.path.clone(),
                method: r.method.clone(),
                status: r.status,
                source: Some(r.source.clone()).filter(|s| !s.is_empty()),
                content_type: r.content_type.clone(),
                auth: guess_auth(&r.url, &report.findings),
                rate_limit: guess_rate(&r.url, &report.findings),
                tags,
            });
        }
    } else {
        for u in &report.discovered_urls {
            let path = Url::parse(u)
                .map(|p| p.path().to_string())
                .unwrap_or_else(|_| u.clone());
            let tags = classify_tags(u, &report.findings);
            routes.push(ManifestRoute {
                url: u.clone(),
                path,
                method: "GET".into(),
                status: None,
                source: None,
                content_type: None,
                auth: guess_auth(u, &report.findings),
                rate_limit: guess_rate(u, &report.findings),
                tags,
            });
        }
    }

    SurfaceManifest {
        tool: report.tool.clone(),
        version: report.version.clone(),
        target: report.target.clone(),
        generated_at: report.finished_at.to_rfc3339(),
        stats: ManifestStats {
            route_count: routes.len(),
            finding_count: report.findings.len(),
            auth_surfaces: auth_surfaces.len(),
            by_severity: report.stats.by_severity.clone(),
        },
        routes,
        auth_surfaces,
        tech,
        firebase,
        rate_limits,
        images: report.image_harvest.clone(),
    }
}

pub fn to_string(report: &ScanReport) -> anyhow::Result<String> {
    let m = from_report(report);
    Ok(serde_json::to_string_pretty(&m)?)
}

fn classify_tags(url: &str, findings: &[Finding]) -> Vec<String> {
    let mut tags = Vec::new();
    let path = Url::parse(url)
        .map(|u| u.path().to_ascii_lowercase())
        .unwrap_or_default();
    if path.contains("login") || path.contains("signin") || path.contains("sign-in") {
        tags.push("login".into());
    }
    if path.contains("signup")
        || path.contains("sign-up")
        || path.contains("register")
        || path.contains("sign_up")
    {
        tags.push("signup".into());
    }
    if path.contains("admin") || path.contains("dashboard") {
        tags.push("admin".into());
    }
    if path.contains("/api") || path.contains("graphql") {
        tags.push("api".into());
    }
    if path.contains("firebase") || path.contains("firestore") {
        tags.push("firebase".into());
    }
    if path.contains("/assets/images/")
        || path.contains("/static/images/")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
    {
        tags.push("image-asset".into());
    }
    for f in findings {
        if f.url == url && f.module == "firebase" {
            tags.push("firebase-finding".into());
            break;
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn guess_auth(url: &str, findings: &[Finding]) -> AuthGuess {
    for f in findings {
        if f.url != url {
            // also match path-level for auth findings that share path
            continue;
        }
        match f.id.as_str() {
            "anon-access-sensitive" => return AuthGuess::UnauthenticatedSensitive,
            "auth-endpoint-protected" | "login-guarded" | "signup-guarded" => {
                return AuthGuess::AuthRequired;
            }
            "login-form" | "signup-form" | "login-unguarded" | "signup-unguarded"
            | "auth-endpoint-unauthenticated" => {
                return AuthGuess::PublicAuthForm;
            }
            "admin-200" => return AuthGuess::UnauthenticatedSensitive,
            _ => {}
        }
    }
    // any finding on same path
    for f in findings {
        if !urls_same_path(url, &f.url) {
            continue;
        }
        match f.id.as_str() {
            "anon-access-sensitive" => return AuthGuess::UnauthenticatedSensitive,
            "auth-endpoint-protected" => return AuthGuess::AuthRequired,
            "login-form" | "signup-form" => return AuthGuess::PublicAuthForm,
            _ => {}
        }
    }
    AuthGuess::Unknown
}

fn guess_rate(url: &str, findings: &[Finding]) -> RateLimitGuess {
    for f in findings {
        if f.url != url && !urls_same_path(url, &f.url) {
            continue;
        }
        match f.id.as_str() {
            "http-429" | "burst-triggered-429" => return RateLimitGuess::Http429,
            "rate-limit-headers" => return RateLimitGuess::HeadersPresent,
            "no-rate-limit-signal" | "burst-no-throttle" => return RateLimitGuess::NoSignal,
            _ => {}
        }
    }
    RateLimitGuess::Unknown
}

fn urls_same_path(a: &str, b: &str) -> bool {
    match (Url::parse(a), Url::parse(b)) {
        (Ok(ua), Ok(ub)) => ua.path() == ub.path() && ua.host_str() == ub.host_str(),
        _ => a == b,
    }
}
