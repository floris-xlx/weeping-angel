//! Synthesize an OpenAPI 3.0 document from discovered routes and findings.
//!
//! This is recon-derived (not a replacement for vendor specs). Paths, response
//! status codes, content types, and auth/rate-limit annotations come from the scan.

use serde_json::{json, Map, Value};
use url::Url;

use crate::finding::ScanReport;
use crate::report::manifest::{self, AuthGuess, RateLimitGuess};

pub fn from_report(report: &ScanReport) -> Value {
    let manifest = manifest::from_report(report);
    let base = Url::parse(&report.target).ok();
    let server_url = base
        .as_ref()
        .map(|u| {
            let mut s = format!("{}://{}", u.scheme(), u.host_str().unwrap_or("localhost"));
            if let Some(port) = u.port() {
                s.push_str(&format!(":{port}"));
            }
            s
        })
        .unwrap_or_else(|| report.target.clone());

    let mut paths = Map::new();

    for route in &manifest.routes {
        let path = normalize_path(&route.path);
        if path.is_empty() || path == "/" && route.url.contains('?') {
            // keep root
        }
        // skip pure static assets noise somewhat
        if is_static_asset(&path) {
            continue;
        }

        let mut responses = Map::new();
        let status_key = route
            .status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".into());

        let mut resp_obj = Map::new();
        resp_obj.insert(
            "description".into(),
            json!(format!(
                "Observed during recon (source: {})",
                route.source.as_deref().unwrap_or("unknown")
            )),
        );
        if let Some(ct) = &route.content_type {
            resp_obj.insert(
                "content".into(),
                json!({
                    ct: {
                        "schema": { "type": "string", "description": "body not captured in full" }
                    }
                }),
            );
        }

        responses.insert(status_key, Value::Object(resp_obj));

        // Auth annotation
        let security = match route.auth {
            AuthGuess::AuthRequired => Some(json!([{ "sessionCookie": [] }])),
            AuthGuess::UnauthenticatedSensitive | AuthGuess::PublicAuthForm => None,
            AuthGuess::UnauthenticatedOk => None,
            AuthGuess::Unknown => None,
        };

        let mut op = Map::new();
        op.insert(
            "summary".into(),
            json!(summarize_route(&path, &route.tags)),
        );
        op.insert(
            "operationId".into(),
            json!(operation_id(&route.method, &path)),
        );
        op.insert(
            "tags".into(),
            json!(if route.tags.is_empty() {
                vec!["discovered".to_string()]
            } else {
                route.tags.clone()
            }),
        );
        op.insert("responses".into(), Value::Object(responses));

        let mut x_weeping = Map::new();
        x_weeping.insert(
            "auth_guess".into(),
            json!(auth_guess_str(&route.auth)),
        );
        x_weeping.insert(
            "rate_limit_guess".into(),
            json!(rate_guess_str(&route.rate_limit)),
        );
        if let Some(src) = &route.source {
            x_weeping.insert("discovery_source".into(), json!(src));
        }
        x_weeping.insert("observed_url".into(), json!(&route.url));
        op.insert("x-weeping-angel".into(), Value::Object(x_weeping));

        if let Some(sec) = security {
            op.insert("security".into(), sec);
        }

        // Mark weak unauth sensitive
        if matches!(route.auth, AuthGuess::UnauthenticatedSensitive) {
            op.insert(
                "description".into(),
                json!("WARNING: scan suggests this path may expose sensitive data without authentication."),
            );
        }

        // rate limit extension
        if matches!(
            route.rate_limit,
            RateLimitGuess::HeadersPresent | RateLimitGuess::Http429
        ) {
            op.insert(
                "x-rate-limit-observed".into(),
                json!(true),
            );
        }

        let method = route.method.to_ascii_lowercase();
        let entry = paths.entry(path.clone()).or_insert_with(|| json!({}));
        if let Some(obj) = entry.as_object_mut() {
            obj.insert(method, Value::Object(op));
        }
    }

    // Document auth surfaces as path items if missing
    for auth in &manifest.auth_surfaces {
        if let Ok(u) = Url::parse(&auth.url) {
            let path = normalize_path(u.path());
            if paths.contains_key(&path) {
                continue;
            }
            paths.insert(
                path.clone(),
                json!({
                    "get": {
                        "summary": format!("Auth surface: {}", auth.kind),
                        "tags": ["auth-surface"],
                        "description": auth.detail,
                        "x-weeping-angel": {
                            "auth_status": auth.status,
                            "guarded": auth.guarded
                        },
                        "responses": {
                            "200": { "description": "Observed during recon" }
                        }
                    }
                }),
            );
        }
    }

    let mut info_desc = format!(
        "Synthesized by weeping-angel from authorized recon of {}.\n\
         Not an official API contract — paths and status codes are observed, not asserted complete.\n\
         Auth guesses and rate-limit notes are advisory.",
        report.target
    );
    if manifest.firebase.detected {
        info_desc.push_str("\n\nFirebase/Firestore signals detected.");
        if !manifest.firebase.project_ids.is_empty() {
            info_desc.push_str(&format!(
                " projectIds: {}.",
                manifest.firebase.project_ids.join(", ")
            ));
        }
    }

    json!({
        "openapi": "3.0.3",
        "info": {
            "title": format!("weeping-angel recon: {}", host_label(&report.target)),
            "version": report.version,
            "description": info_desc,
            "x-generated-by": {
                "tool": report.tool,
                "version": report.version,
                "profile": report.profile,
                "finished_at": report.finished_at.to_rfc3339()
            }
        },
        "servers": [
            { "url": server_url, "description": "Scan target origin" }
        ],
        "paths": paths,
        "components": {
            "securitySchemes": {
                "sessionCookie": {
                    "type": "apiKey",
                    "in": "cookie",
                    "name": "session",
                    "description": "Inferred session cookie; actual cookie name may differ."
                },
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "tags": [
            { "name": "discovered", "description": "Routes found during recon" },
            { "name": "login", "description": "Login-related surfaces" },
            { "name": "signup", "description": "Registration surfaces" },
            { "name": "api", "description": "API-like paths" },
            { "name": "admin", "description": "Administrative surfaces" },
            { "name": "firebase", "description": "Firebase/Firestore related" },
            { "name": "auth-surface", "description": "Authentication UX / endpoints" }
        ],
        "x-weeping-angel-manifest-stats": {
            "routes": manifest.stats.route_count,
            "findings": manifest.stats.finding_count,
            "firebase_detected": manifest.firebase.detected,
            "rate_limit_with_signal": manifest.rate_limits.routes_with_signals.len(),
            "rate_limit_without_signal": manifest.rate_limits.routes_without_signals.len()
        }
    })
}

pub fn to_string(report: &ScanReport) -> anyhow::Result<String> {
    let doc = from_report(report);
    Ok(serde_json::to_string_pretty(&doc)?)
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".into();
    }
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn is_static_asset(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".js")
        || lower.ends_with(".css")
        || lower.ends_with(".map")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".svg")
        || lower.ends_with(".ico")
        || lower.ends_with(".woff")
        || lower.ends_with(".woff2")
}

fn summarize_route(path: &str, tags: &[String]) -> String {
    if tags.iter().any(|t| t == "login") {
        return format!("Login surface {path}");
    }
    if tags.iter().any(|t| t == "signup") {
        return format!("Signup surface {path}");
    }
    if tags.iter().any(|t| t == "admin") {
        return format!("Admin surface {path}");
    }
    if tags.iter().any(|t| t == "api") {
        return format!("API {path}");
    }
    format!("Discovered {path}")
}

fn operation_id(method: &str, path: &str) -> String {
    let mut s = format!(
        "{}_{}",
        method.to_ascii_lowercase(),
        path.trim_matches('/').replace('/', "_")
    );
    if s.ends_with('_') {
        s.push_str("root");
    }
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

fn host_label(target: &str) -> String {
    Url::parse(target)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "target".into())
}

fn auth_guess_str(a: &AuthGuess) -> &'static str {
    match a {
        AuthGuess::Unknown => "unknown",
        AuthGuess::UnauthenticatedOk => "unauthenticated_ok",
        AuthGuess::AuthRequired => "auth_required",
        AuthGuess::PublicAuthForm => "public_auth_form",
        AuthGuess::UnauthenticatedSensitive => "unauthenticated_sensitive",
    }
}

fn rate_guess_str(r: &RateLimitGuess) -> &'static str {
    match r {
        RateLimitGuess::Unknown => "unknown",
        RateLimitGuess::HeadersPresent => "headers_present",
        RateLimitGuess::Http429 => "http_429",
        RateLimitGuess::NoSignal => "no_signal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::finding::{Finding, ScanStats, Severity};

    #[test]
    fn builds_openapi_paths() {
        let report = ScanReport {
            tool: "weeping-angel".into(),
            version: "0.1.1".into(),
            target: "http://127.0.0.1:8787/".into(),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            profile: "deep".into(),
            modules: vec![],
            discovered_urls: vec![
                "http://127.0.0.1:8787/login".into(),
                "http://127.0.0.1:8787/api/v1/users".into(),
            ],
            routes: vec![
                crate::finding::RouteRecord {
                    url: "http://127.0.0.1:8787/login".into(),
                    path: "/login".into(),
                    method: "GET".into(),
                    status: Some(200),
                    source: "crawl".into(),
                    content_type: Some("text/html".into()),
                    tags: vec!["login".into()],
                },
                crate::finding::RouteRecord {
                    url: "http://127.0.0.1:8787/api/v1/users".into(),
                    path: "/api/v1/users".into(),
                    method: "GET".into(),
                    status: Some(200),
                    source: "wordlist".into(),
                    content_type: Some("application/json".into()),
                    tags: vec!["api".into()],
                },
            ],
            findings: vec![
                Finding::builder("discovery", "route-discovered")
                    .title("Discovered route (crawl)")
                    .severity(Severity::Info)
                    .url("http://127.0.0.1:8787/login")
                    .description("URL discovered via crawl. HTTP status 200.")
                    .build(),
                Finding::builder("auth-surface", "login-form")
                    .title("Login form")
                    .severity(Severity::Info)
                    .url("http://127.0.0.1:8787/login")
                    .description("form")
                    .build(),
            ],
            stats: ScanStats::default(),
            image_harvest: None,
            phases: vec![],
            module_results: vec![],
            surface: Default::default(),
            tech_stack: vec![],
            timing: Default::default(),
        };
        let doc = from_report(&report);
        assert_eq!(doc["openapi"], "3.0.3");
        assert!(doc["paths"].get("/login").is_some());
        assert!(doc["paths"].get("/api/v1/users").is_some());
    }
}
