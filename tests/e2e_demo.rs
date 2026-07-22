//! End-to-end scan against the in-process lab demo app.

use std::time::Duration;

use weeping_angel::authz::Authorization;
use weeping_angel::config::Profile;
use weeping_angel::engine::{run_scan, ScanOptions};
use weeping_angel::finding::Severity;
use weeping_angel::http::ClientConfig;

#[tokio::test]
async fn scan_lab_demo_finds_core_issues() {
    let app = weeping_angel_demo_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    // tiny settle
    tokio::time::sleep(Duration::from_millis(50)).await;

    let base = format!("http://{addr}/");
    let host = addr.ip().to_string();

    let authz = Authorization::new(true, [host.clone(), "127.0.0.1".into()], true, false);
    let client_cfg = ClientConfig {
        timeout: Duration::from_secs(5),
        concurrency: 8,
        rps: 50.0,
        cookie: Some("session=admin-session".into()),
        ..ClientConfig::default()
    };

    let opts = ScanOptions {
        targets: vec![url::Url::parse(&base).unwrap()],
        profile: Profile::Deep,
        modules: Profile::Deep
            .default_modules()
            .into_iter()
            .map(str::to_string)
            .chain(std::iter::once("auth-compare".into()))
            .collect(),
        depth: 2,
        max_urls: 80,
        ignore_robots: true,
        wordlist: std::path::PathBuf::from("wordlists/common-paths.txt"),
        probes: vec![
            "xss".into(),
            "sqli".into(),
            "open-redirect".into(),
            "path-traversal".into(),
        ],
        fail_on: Some(Severity::Medium),
        templates_dir: std::path::PathBuf::from("templates"),
        compare_auth: true,
    };

    let report = run_scan(authz, client_cfg, opts)
        .await
        .expect("scan should succeed");

    assert!(
        report.stats.urls_discovered > 3,
        "expected multiple discovered urls, got {}",
        report.stats.urls_discovered
    );

    let ids: Vec<_> = report.findings.iter().map(|f| f.id.as_str()).collect();
    let modules: Vec<_> = report.findings.iter().map(|f| f.module.as_str()).collect();

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.id.contains("env") || f.id == "exposed-env" || f.module == "templates"),
        "expected env exposure, ids={ids:?}"
    );
    assert!(
        report.findings.iter().any(|f| f.module == "secrets"),
        "expected secrets findings"
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.module == "headers" && f.severity >= Severity::Low),
        "expected header findings"
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.id.contains("xss") || f.id == "reflected-xss" || f.id == "reflected-input"),
        "expected XSS probe hit on /search, modules={modules:?} ids={ids:?}"
    );

    // auth-compare or auth-surface should notice admin / api
    assert!(
        report.findings.iter().any(|f| {
            f.module == "auth-compare"
                || f.module == "auth-surface"
                || f.id.contains("admin")
                || f.id.contains("anon")
        }),
        "expected auth surface findings, ids={ids:?}"
    );

    }

/// Re-export demo router without requiring binary link tricks.
fn weeping_angel_demo_router() -> axum::Router {
    // Inline minimal duplicate of demo routes used for tests is heavy —
    // instead include the demo module paths via shared factory.
    demo_lab::router()
}

mod demo_lab {
    use axum::extract::{Query, Request};
    use axum::http::{header, StatusCode};
    use axum::response::{Html, IntoResponse, Redirect, Response};
    use axum::routing::get;
    use axum::Router;
    use serde::Deserialize;

    pub fn router() -> Router {
        Router::new()
            .route("/", get(home))
            .route("/spa", get(spa))
            .route("/assets/app.js", get(app_js))
            .route("/login", get(login))
            .route("/admin", get(admin))
            .route("/search", get(search))
            .route("/redirect", get(redir))
            .route("/file", get(file))
            .route("/.env", get(|| async { "API_SECRET=x\nDATABASE_URL=postgres://a:b@h/db\n" }))
            .route("/.git/HEAD", get(|| async { "ref: refs/heads/main\n" }))
            .route("/api/config", get(api_config))
            .route("/api/v1/users", get(api_users))
            .route("/api/v1/me", get(api_me))
            .route("/openapi.json", get(openapi))
            .route("/robots.txt", get(|| async { "User-agent: *\nDisallow:\n" }))
            .route("/sitemap.xml", get(sitemap))
    }

    async fn home() -> impl IntoResponse {
        (
            [
                (header::SET_COOKIE, "session=guest; Path=/"),
                (header::SERVER, "lab"),
                (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            ],
            Html(
                r#"<!DOCTYPE html><html><body>
                <a href="/login">L</a><a href="/admin">A</a><a href="/spa">S</a>
                <a href="/search?q=hi">Search</a>
                <script src="/assets/app.js"></script>
                <script>const STRIPE_KEY="sk_live_51DemoPlantedSecretKey000000";
                window.__INITIAL_STATE__={routes:["/api/v1/me","/api/v1/users"]};</script>
                </body></html>"#,
            ),
        )
    }

    async fn spa() -> Html<&'static str> {
        Html(
            r#"<script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{"paths":["/api/v1/me","/api/internal/debug"]}}}</script>
            <script src="/assets/app.js"></script>"#,
        )
    }

    async fn app_js() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/javascript")],
            r#"const routes=["/api/v1/items"]; fetch("/api/v1/me"); const T="ghp_demoPlantedTokenNotReal000000000001";"#,
        )
    }

    async fn login() -> Html<&'static str> {
        Html(r#"<form><input type="password" name="password"></form>"#)
    }

    fn is_admin(req: &Request) -> bool {
        req.headers()
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|c| c.contains("session=admin-session"))
            .unwrap_or(false)
    }

    async fn admin(req: Request) -> Html<&'static str> {
        let _ = is_admin(&req);
        Html("<html><body><h1>Admin</h1><p>open admin</p></body></html>")
    }

    #[derive(Deserialize)]
    struct Q {
        q: Option<String>,
    }
    async fn search(Query(q): Query<Q>) -> Html<String> {
        Html(format!("Results for: {}", q.q.unwrap_or_default()))
    }

    #[derive(Deserialize)]
    struct R {
        next: Option<String>,
    }
    async fn redir(Query(q): Query<R>) -> Response {
        Redirect::temporary(q.next.as_deref().unwrap_or("/")).into_response()
    }

    #[derive(Deserialize)]
    struct F {
        file: Option<String>,
    }
    async fn file(Query(q): Query<F>) -> Response {
        let n = q.file.unwrap_or_default();
        if n.contains("passwd") || n.contains("..") {
            (StatusCode::OK, "root:x:0:0:root:/root:/bin/bash\n").into_response()
        } else {
            (StatusCode::NOT_FOUND, "no").into_response()
        }
    }

    async fn api_config() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"aws_key":"AKIAIOSFODNN7EXAMPLE","jwt":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIn0.xxx"}"#,
        )
    }

    async fn api_users() -> impl IntoResponse {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"users":[{"email":"a@lab"}]}"#,
        )
    }

    async fn api_me(req: Request) -> Response {
        if is_admin(&req) {
            (StatusCode::OK, r#"{"role":"admin"}"#).into_response()
        } else {
            (StatusCode::UNAUTHORIZED, "no").into_response()
        }
    }

    async fn openapi() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"openapi":"3.0.0","paths":{"/api/v1/me":{},"/api/v1/users":{}}}"#,
        )
    }

    async fn sitemap() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/xml")],
            r#"<?xml version="1.0"?><urlset><url><loc>/login</loc></url><url><loc>/search?q=x</loc></url></urlset>"#,
        )
    }
}
