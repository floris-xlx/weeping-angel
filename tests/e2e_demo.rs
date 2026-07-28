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

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.module == "firebase" || f.id.contains("firebase") || f.id.contains("firestore")),
        "expected firebase/firestore findings, ids={ids:?}"
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.module == "rate-limits" || f.id.contains("rate-limit")),
        "expected rate-limit findings, ids={ids:?}"
    );
    assert!(
        report.findings.iter().any(|f| f.id == "auth-guard-summary"
            || f.id == "login-unguarded"
            || f.id == "signup-form"
            || f.id == "signup-unguarded"),
        "expected login/signup guard classification, ids={ids:?}"
    );
    assert!(
        report.discovered_urls.iter().any(|u| u.contains("dashboardpic")
            || u.contains("/assets/images/")),
        "expected image hosting path enumeration, urls={:?}",
        report.discovered_urls
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.id == "image-asset"
                || f.id == "image-hosting-pattern"
                || f.description.contains("image")
                    && f.module == "discovery"),
        "expected image pattern findings, ids={ids:?}"
    );

    let manifest = weeping_angel::report::manifest::from_report(&report);
    assert!(manifest.routes.len() > 2);
    assert!(manifest.firebase.detected, "manifest should mark firebase");

    let oas = weeping_angel::report::openapi_gen::from_report(&report);
    assert_eq!(oas["openapi"], "3.0.3");
    assert!(oas["paths"].as_object().map(|p| !p.is_empty()).unwrap_or(false));

    let harvest = report
        .image_harvest
        .as_ref()
        .expect("image_harvest manifest should be present");
    assert!(
        !harvest.all_paths.is_empty(),
        "expected harvested image paths"
    );
    assert!(
        harvest.stats.head_probes > 0,
        "expected HEAD probes on image paths"
    );
    assert!(
        harvest
            .images
            .iter()
            .any(|i| i.path.contains("dashboardpic") && i.exists),
        "expected dashboardpic HEAD-ok in harvest, sample={:?}",
        harvest
            .images
            .iter()
            .filter(|i| i.exists)
            .map(|i| i.path.as_str())
            .take(10)
            .collect::<Vec<_>>()
    );
    assert!(
        harvest
            .images
            .iter()
            .any(|i| i.head.is_some() || i.options.is_some()),
        "expected HEAD and/or OPTIONS probe records"
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
            .route(
                "/assets/images/home/dashboardpic.png",
                get(dashboard_pic),
            )
            .route("/assets/images/home/hero.png", get(dashboard_pic))
            .route("/login", get(login))
            .route("/signup", get(signup))
            .route("/admin", get(admin))
            .route("/search", get(search))
            .route("/redirect", get(redir))
            .route("/file", get(file))
            .route("/.env", get(|| async { "API_SECRET=x\nDATABASE_URL=postgres://a:b@h/db\n" }))
            .route("/.git/HEAD", get(|| async { "ref: refs/heads/main\n" }))
            .route("/api/config", get(api_config))
            .route("/api/v1/users", get(api_users))
            .route("/api/v1/me", get(api_me))
            .route("/api/limited", get(api_limited))
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
                <img src="/assets/images/home/dashboardpic.png" alt="dash"/>
                <script src="/assets/app.js"></script>
                <script>const STRIPE_KEY="sk_live_51DemoPlantedSecretKey000000";
                const firebaseConfig={apiKey:"AIzaSyA-demoKeyNotReal0123456789ABCD",authDomain:"lab.firebaseapp.com",projectId:"weeping-angel-lab"};
                window.__INITIAL_STATE__={routes:["/api/v1/me","/api/v1/users","/signup"]};
                // firebase/firestore marker
                </script>
                </body></html>"#,
            ),
        )
    }

    async fn dashboard_pic() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "image/png")],
            // minimal PNG header bytes — enough for content-type checks
            &[0x89u8, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a][..],
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
        Html(r#"<form action="/login"><input type="password" name="password"></form>"#)
    }

    async fn signup() -> Html<&'static str> {
        Html(r#"<form action="/signup"><input name="email"><input type="password" name="password"><button>Sign up</button></form>"#)
    }

    async fn api_limited() -> Response {
        let mut res = Response::new(r#"{"ok":true}"#.into());
        *res.status_mut() = StatusCode::OK;
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        res.headers_mut().insert(
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderValue::from_static("10"),
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
