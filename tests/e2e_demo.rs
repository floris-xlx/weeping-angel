//! End-to-end scan against the shared lab demo app (`weeping_angel::lab`).

use std::time::Duration;

use weeping_angel::authz::Authorization;
use weeping_angel::config::Profile;
use weeping_angel::engine::{ScanOptions, run_scan};
use weeping_angel::finding::Severity;
use weeping_angel::http::ClientConfig;

#[tokio::test]
async fn scan_lab_demo_finds_core_issues() {
    let app = weeping_angel::lab::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
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
        skip_image_options: true,
        max_terminal_routes: 120,
        report_width: 100,
    };

    let report = run_scan(authz, client_cfg, opts)
        .await
        .expect("scan should succeed");

    assert!(
        report.stats.urls_discovered > 3,
        "expected multiple discovered urls, got {}",
        report.stats.urls_discovered
    );

    // Structured routes (RouteRecord) should mirror discovery
    assert!(
        !report.routes.is_empty(),
        "expected structured route inventory"
    );
    assert!(
        report.routes.iter().any(|r| !r.source.is_empty()),
        "routes should carry discovery source"
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
        report.findings.iter().any(|f| f.module == "firebase"
            || f.id.contains("firebase")
            || f.id.contains("firestore")),
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
        report
            .discovered_urls
            .iter()
            .any(|u| u.contains("dashboardpic") || u.contains("/assets/images/")),
        "expected image hosting path enumeration, urls={:?}",
        report.discovered_urls
    );
    assert!(
        report.findings.iter().any(|f| f.id == "image-asset"
            || f.id == "image-hosting-pattern"
            || f.description.contains("image") && f.module == "discovery"),
        "expected image pattern findings, ids={ids:?}"
    );

    let manifest = weeping_angel::report::manifest::from_report(&report);
    assert!(manifest.routes.len() > 2);
    assert!(manifest.firebase.detected, "manifest should mark firebase");
    // content_type should be filled from RouteRecord when available
    assert!(
        manifest.routes.iter().any(|r| r.status.is_some()),
        "manifest routes should carry status from RouteRecord"
    );

    let oas = weeping_angel::report::openapi_gen::from_report(&report);
    assert_eq!(oas["openapi"], "3.0.3");
    assert!(
        oas["paths"]
            .as_object()
            .map(|p| !p.is_empty())
            .unwrap_or(false)
    );

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

    assert!(
        !report.phases.is_empty(),
        "expected phase timings on deep scan"
    );
    assert!(report.timing.wall_seconds > 0.0);
    assert!(report.timing.requests >= report.stats.requests.min(1));
    assert!(!report.module_results.is_empty());
    assert!(
        report.surface.total_routes > 0 || !report.discovered_urls.is_empty(),
        "surface/urls empty"
    );

    let html = weeping_angel::report::html::to_string(&report);
    assert!(html.contains("weeping-angel"));
    assert!(html.contains("Phase") || html.contains("phase") || html.contains("Findings"));

    let js = weeping_angel::report::json::to_string(&report).unwrap();
    assert!(js.contains("\"phases\""));
    assert!(js.contains("\"timing\""));
    assert!(js.contains("\"surface\""));
    assert!(js.contains("\"routes\""));

    let sarif = weeping_angel::report::sarif::to_string(&report).unwrap();
    assert!(sarif.contains("floris-xlx/weeping-angel") || sarif.contains("weeping-angel"));
    // inventory routes should not dominate SARIF results
    assert!(
        !sarif.contains("route-discovered")
            || sarif.matches("ruleId").count() < report.findings.len(),
        "SARIF should prefer security findings over inventory noise"
    );
}
