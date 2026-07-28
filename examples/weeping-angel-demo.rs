//! Intentionally vulnerable local lab for weeping-angel demos.
//! Bind: 127.0.0.1 only. Do not expose to the network.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::{Query, Request};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);

    let app: Router = demo_router();
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("weeping-angel-demo listening on http://{addr}");
    eprintln!("Scan with:");
    eprintln!(
        "  cargo run --bin weeping-angel -- scan http://127.0.0.1:{port}/ --i-own-this --allow-host 127.0.0.1 --profile deep --enable-active --cookie \"session=admin-session\" --compare-auth"
    );

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

pub fn demo_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/spa", get(spa_shell))
        .route("/assets/app.js", get(app_js))
        .route(
            "/assets/images/home/dashboardpic.png",
            get(dashboard_pic),
        )
        .route("/assets/images/home/hero.png", get(dashboard_pic))
        .route("/login", get(login))
        .route("/signup", get(signup))
        .route("/register", get(signup))
        .route("/admin", get(admin))
        .route("/admin/users", get(admin_users))
        .route("/api/config", get(api_config))
        .route("/api/v1/me", get(api_me))
        .route("/api/v1/users", get(api_users))
        .route("/api/auth/login", get(api_auth_login))
        .route("/api/limited", get(api_limited))
        .route("/search", get(search))
        .route("/redirect", get(open_redirect))
        .route("/file", get(file_read))
        .route("/.env", get(exposed_env))
        .route("/.git/HEAD", get(git_head))
        .route("/openapi.json", get(openapi))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/private/secret", get(|| async { (StatusCode::FORBIDDEN, "nope") }))
        .fallback(not_found)
}

async fn home() -> impl IntoResponse {
    let body = r#"<!DOCTYPE html>
<html>
<head><title>Weeping Angel Lab</title>
<meta name="generator" content="lab-demo"/>
</head>
<body>
<h1>Weeping Angel Demo Lab</h1>
<p>Intentionally weak app for authorized local testing.</p>
<ul>
  <li><a href="/login">Login</a></li>
  <li><a href="/signup">Signup</a></li>
  <li><a href="/admin">Admin</a></li>
  <li><a href="/spa">SPA shell</a></li>
  <li><a href="/search?q=hello">Search</a></li>
  <li><a href="/api/config">API config</a></li>
  <li><a href="/api/limited">Rate-limited API</a></li>
  <li><a href="/openapi.json">OpenAPI</a></li>
  <li><a href="/assets/images/home/dashboardpic.png">Dashboard pic</a></li>
</ul>
<img src="/assets/images/home/dashboardpic.png" alt="dashboard" />
<script src="/assets/app.js"></script>
<script>
  // planted secret for scanner demos — not real credentials
  const STRIPE_KEY = "sk_live_51DemoPlantedSecretKey000000";
  // planted Firebase client config (demo only)
  const firebaseConfig = {
    apiKey: "AIzaSyA-demoKeyNotReal0123456789ABCD",
    authDomain: "weeping-angel-lab.firebaseapp.com",
    projectId: "weeping-angel-lab",
    storageBucket: "weeping-angel-lab.appspot.com",
    appId: "1:1234567890:web:abcdef"
  };
  // modular SDK-style markers
  // import { getFirestore } from "firebase/firestore"
  window.__FIREBASE__ = firebaseConfig;
  window.__INITIAL_STATE__ = { apiBase: "/api/v1", routes: ["/api/v1/me", "/api/v1/users", "/signup", "/api/limited"] };
</script>
</body>
</html>"#;
    (
        [
            (header::SET_COOKIE, "session=guest; Path=/"),
            (header::SERVER, "lab-demo/0.1"),
            (header::HeaderName::from_static("x-powered-by"), "Express-ish"),
            (
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                "*",
            ),
        ],
        Html(body),
    )
}

async fn spa_shell() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html><head><title>SPA Lab</title></head>
<body>
<div id="root">Loading SPA…</div>
<script id="__NEXT_DATA__" type="application/json">
{"page":"/dashboard","buildId":"lab","runtimeConfig":{"apiUrl":"/api/v1"},"props":{"pageProps":{"paths":["/api/v1/me","/api/internal/debug"]}}}
</script>
<script src="/assets/app.js"></script>
</body></html>"#,
    )
}

async fn dashboard_pic() -> impl IntoResponse {
    // Minimal valid 1x1 PNG for image-pattern enumeration demos
    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xcf, 0xc0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x05, 0xfe, 0xd4, 0xef, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    (
        [(header::CONTENT_TYPE, "image/png")],
        PNG_1X1,
    )
}

async fn app_js() -> impl IntoResponse {
    let js = r#"
// client router paths
const routes = ["/dashboard", "/settings", "/api/v1/items"];
export async function loadUser() {
  return fetch("/api/v1/me");
}
export async function listUsers() {
  return fetch("/api/v1/users");
}
const HERO = "/assets/images/home/dashboardpic.png";
const GITHUB_TOKEN = "ghp_demoPlantedTokenNotReal000000000001";
"#;
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        js,
    )
}

async fn login() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html><body>
<h1>Login</h1>
<form method="POST" action="/login">
  <label>User <input name="username"></label>
  <label>Password <input type="password" name="password"></label>
  <button type="submit">Sign in</button>
</form>
</body></html>"#,
    )
}

async fn signup() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html><body>
<h1>Create account</h1>
<form method="POST" action="/signup">
  <label>Email <input name="email" type="email"></label>
  <label>Password <input type="password" name="password"></label>
  <button type="submit">Sign up</button>
</form>
<p>Firebase Auth signUp is also referenced client-side.</p>
</body></html>"#,
    )
}

async fn api_auth_login() -> Response {
    // JSON login API without credentials still 401 — guarded API shape
    (StatusCode::UNAUTHORIZED, "credentials required").into_response()
}

async fn api_limited() -> Response {
    // Advertise rate limits (lab); always 200 with headers for passive detection
    let mut res = Response::new(Body::from(r#"{"ok":true,"note":"rate limited surface"}"#));
    *res.status_mut() = StatusCode::OK;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_static("60"),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_static("59"),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_static("60"),
    );
    res
}

fn is_admin(req: &Request) -> bool {
    req.headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|c| c.contains("session=admin-session"))
        .unwrap_or(false)
}

async fn admin(req: Request) -> Response {
    if !is_admin(&req) {
        return (
            StatusCode::OK,
            Html("<html><body><h1>Admin</h1><p>Welcome, unauthenticated admin panel (bad).</p><a href=\"/admin/users\">Users</a></body></html>"),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Html("<html><body><h1>Admin (auth)</h1><p>Authenticated view.</p><a href=\"/admin/users\">Users</a></body></html>"),
    )
        .into_response()
}

async fn admin_users(req: Request) -> Response {
    if !is_admin(&req) {
        return (StatusCode::UNAUTHORIZED, "login required").into_response();
    }
    (StatusCode::OK, r#"{"users":[{"id":1,"role":"admin"}]}"#).into_response()
}

async fn api_config() -> impl IntoResponse {
    let body = r#"{
  "env": "production",
  "aws_key": "AKIAIOSFODNN7EXAMPLE",
  "debug": true,
  "jwt_sample": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"
}"#;
    (
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
}

async fn api_me(req: Request) -> Response {
    if is_admin(&req) {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"id":1,"name":"admin","role":"admin"}"#,
        )
            .into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
    }
}

async fn api_users(req: Request) -> Response {
    // IDOR-ish: returns data without auth (bad)
    let _ = req;
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"users":[{"id":1,"email":"admin@lab.local"},{"id":2,"email":"user@lab.local"}]}"#,
    )
        .into_response()
}

#[derive(Deserialize)]
struct SearchQ {
    q: Option<String>,
}

async fn search(Query(q): Query<SearchQ>) -> Html<String> {
    let term = q.q.unwrap_or_default();
    // Reflected XSS (unescaped)
    Html(format!(
        "<html><body><h1>Search</h1><p>Results for: {term}</p></body></html>"
    ))
}

#[derive(Deserialize)]
struct RedirQ {
    next: Option<String>,
    url: Option<String>,
}

async fn open_redirect(Query(q): Query<RedirQ>) -> Response {
    let target = q.next.or(q.url).unwrap_or_else(|| "/".into());
    Redirect::temporary(&target).into_response()
}

#[derive(Deserialize)]
struct FileQ {
    file: Option<String>,
    path: Option<String>,
}

async fn file_read(Query(q): Query<FileQ>) -> Response {
    let name = q.file.or(q.path).unwrap_or_default();
    if name.contains("passwd") || name.contains("..") {
        // fake /etc/passwd signature for lab
        return (
            StatusCode::OK,
            "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n",
        )
            .into_response();
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

async fn exposed_env() -> &'static str {
    "DATABASE_URL=postgres://lab:lab@localhost/lab\nAPI_SECRET=super-secret-lab-key-123456\nAWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n"
}

async fn git_head() -> &'static str {
    "ref: refs/heads/main\n"
}

async fn openapi() -> impl IntoResponse {
    let body = r#"{
  "openapi": "3.0.0",
  "info": {"title": "Lab API", "version": "1.0.0"},
  "paths": {
    "/api/v1/me": {"get": {"summary": "me"}},
    "/api/v1/users": {"get": {"summary": "users"}},
    "/api/internal/debug": {"get": {"summary": "debug"}},
    "/api/limited": {"get": {"summary": "rate limited"}},
    "/api/auth/login": {"post": {"summary": "login"}},
    "/signup": {"get": {"summary": "signup page"}}
  }
}"#;
    ([(header::CONTENT_TYPE, "application/json")], body)
}

async fn robots() -> &'static str {
    "User-agent: *\nDisallow: /private\nSitemap: /sitemap.xml\n"
}

async fn sitemap() -> impl IntoResponse {
    let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>/login</loc></url>
  <url><loc>/search?q=test</loc></url>
  <url><loc>/admin</loc></url>
</urlset>"#;
    ([(header::CONTENT_TYPE, "application/xml")], body)
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "not found")
}

/// Build a response with optional headers helper for tests.
#[allow(dead_code)]
fn with_headers(status: StatusCode, headers: HeaderMap, body: Body) -> Response {
    let mut res = Response::new(body);
    *res.status_mut() = status;
    *res.headers_mut() = headers;
    res
}

#[allow(dead_code)]
fn hv(s: &str) -> HeaderValue {
    HeaderValue::from_str(s).unwrap_or_else(|_| HeaderValue::from_static(""))
}
