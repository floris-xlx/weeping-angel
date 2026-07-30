//! Lighter e2e: recon profile asserts phases/surface/timing without active probes.

use std::time::Duration;

use weeping_angel::authz::Authorization;
use weeping_angel::config::Profile;
use weeping_angel::engine::{run_scan, ScanOptions};
use weeping_angel::finding::Severity;
use weeping_angel::http::ClientConfig;
use weeping_angel::parse::LogHttp;
use weeping_angel::report::{Format, html, json, write_reports};

#[tokio::test]
async fn recon_scan_emits_wide_report_fields() {
    let app = demo_lab::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    tokio::time::sleep(Duration::from_millis(40)).await;

    let base = format!("http://{addr}/");
    let host = addr.ip().to_string();

    let authz = Authorization::new(true, [host, "127.0.0.1".into()], false, false);
    let client_cfg = ClientConfig {
        timeout: Duration::from_secs(5),
        concurrency: 12,
        rps: 80.0,
        log_http: LogHttp::Off,
        ..ClientConfig::default()
    };

    let opts = ScanOptions {
        targets: vec![url::Url::parse(&base).unwrap()],
        profile: Profile::Recon,
        modules: Profile::Recon
            .default_modules()
            .into_iter()
            .map(str::to_string)
            .collect(),
        depth: 1,
        max_urls: 40,
        ignore_robots: true,
        wordlist: std::path::PathBuf::from("wordlists/common-paths.txt"),
        probes: vec![],
        fail_on: Some(Severity::Critical),
        templates_dir: std::path::PathBuf::from("templates"),
        compare_auth: false,
        skip_image_options: true,
        max_terminal_routes: 20,
        report_width: 100,
    };

    let report = run_scan(authz, client_cfg, opts).await.expect("scan");

    assert!(report.stats.urls_discovered >= 1);
    assert!(
        !report.phases.is_empty(),
        "expected phase timings, phases={:?}",
        report.phases
    );
    assert!(
        report.phases.iter().any(|p| p.name.contains("crawl")),
        "phases={:?}",
        report.phases.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    assert!(report.timing.wall_seconds > 0.0);
    assert!(report.timing.requests > 0);
    assert!(report.surface.total_routes > 0 || !report.discovered_urls.is_empty());
    assert!(!report.module_results.is_empty());

    // JSON includes wide fields
    let js = json::to_string(&report).unwrap();
    assert!(js.contains("phases"));
    assert!(js.contains("timing"));

    // HTML renders
    let h = html::to_string(&report);
    assert!(h.contains("weeping-angel"));

    // write_reports smoke
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("recon");
    write_reports(
        &report,
        &[Format::Json, Format::Html, Format::Terminal],
        Some(&out),
        20,
        80,
    )
    .unwrap();
}

#[tokio::test]
async fn out_of_scope_host_fails_before_network() {
    let authz = Authorization::new(true, ["allowed.test".into()], false, false);
    let err = authz
        .validate_targets(&["https://not-allowed.test/".into()])
        .unwrap_err();
    assert!(matches!(
        err,
        weeping_angel::authz::AuthzError::HostNotAllowed { .. }
    ));
}

#[tokio::test]
async fn standard_scan_wordlist_phase_present() {
    let app = demo_lab::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    tokio::time::sleep(Duration::from_millis(40)).await;

    let base = format!("http://{addr}/");
    let host = addr.ip().to_string();
    let authz = Authorization::new(true, [host, "127.0.0.1".into()], false, false);
    let client_cfg = ClientConfig {
        timeout: Duration::from_secs(5),
        concurrency: 16,
        rps: 100.0,
        log_http: LogHttp::Summary,
        ..ClientConfig::default()
    };

    let opts = ScanOptions {
        targets: vec![url::Url::parse(&base).unwrap()],
        profile: Profile::Standard,
        modules: vec![
            "discovery".into(),
            "wordlist".into(),
            "templates".into(),
            "headers".into(),
            "secrets".into(),
            "exposures".into(),
        ],
        depth: 1,
        max_urls: 50,
        ignore_robots: true,
        wordlist: std::path::PathBuf::from("wordlists/common-paths.txt"),
        probes: vec![],
        fail_on: Some(Severity::Critical),
        templates_dir: std::path::PathBuf::from("templates"),
        compare_auth: false,
        skip_image_options: true,
        max_terminal_routes: 10,
        report_width: 80,
    };

    let report = run_scan(authz, client_cfg, opts).await.expect("scan");
    assert!(
        report.phases.iter().any(|p| p.name == "wordlist" || p.name == "templates"),
        "phases={:?}",
        report.phases
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.module == "templates" || f.id.contains("env") || f.module == "secrets"),
        "expected exposure/secret/template findings"
    );
}

// Shared minimal lab (subset of e2e_demo)
mod demo_lab {
    use axum::extract::{Query, Request};
    use axum::http::{header, StatusCode};
    use axum::response::{Html, IntoResponse, Response};
    use axum::routing::get;
    use axum::Router;
    use serde::Deserialize;

    pub fn router() -> Router {
        Router::new()
            .route("/", get(home))
            .route("/login", get(|| async { Html(r#"<form><input type="password"></form>"#) }))
            .route("/.env", get(|| async { "API_SECRET=x\nDATABASE_URL=postgres://a:b@h/db\n" }))
            .route("/.git/HEAD", get(|| async { "ref: refs/heads/main\n" }))
            .route("/assets/app.js", get(app_js))
            .route("/search", get(search))
            .route("/robots.txt", get(|| async { "User-agent: *\nDisallow:\n" }))
            .route("/api/v1/me", get(api_me))
    }

    async fn home() -> impl IntoResponse {
        (
            [
                (header::SET_COOKIE, "session=guest; Path=/"),
                (header::SERVER, "lab"),
            ],
            Html(
                r#"<!DOCTYPE html><html><body>
                <a href="/login">L</a><a href="/search?q=hi">S</a>
                <script src="/assets/app.js"></script>
                <script>const STRIPE_KEY="sk_live_51DemoPlantedSecretKey000000";</script>
                </body></html>"#,
            ),
        )
    }

    async fn app_js() -> impl IntoResponse {
        (
            [(header::CONTENT_TYPE, "application/javascript")],
            r#"fetch("/api/v1/me"); const T="ghp_demoPlantedTokenNotReal000000000001";"#,
        )
    }

    #[derive(Deserialize)]
    struct Q {
        q: Option<String>,
    }
    async fn search(Query(q): Query<Q>) -> Html<String> {
        Html(format!("Results for: {}", q.q.unwrap_or_default()))
    }

    async fn api_me(req: Request) -> Response {
        let admin = req
            .headers()
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|c| c.contains("admin"))
            .unwrap_or(false);
        if admin {
            (StatusCode::OK, r#"{"role":"admin"}"#).into_response()
        } else {
            (StatusCode::UNAUTHORIZED, "no").into_response()
        }
    }
}
