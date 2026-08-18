//! Scan-only DepCheck web UI (no exploit / credential surfaces).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{Multipart, Path as AxumPath, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::registry::{HttpRegistry, RegistryClient};
use super::types::{FileKind, ManifestInput, ScanOptions, ScanSummary};
use super::{load_url_body, scan_manifest};
use crate::style;

const INDEX_HTML: &str = include_str!("web_index.html");

#[derive(Clone)]
struct AppState {
    scans: Arc<RwLock<HashMap<String, ScanJob>>>,
    history: Arc<RwLock<Vec<HistoryEntry>>>,
}

#[derive(Clone, Serialize)]
struct HistoryEntry {
    id: String,
    file: String,
    ecosystem: String,
    total: usize,
    vulnerable: usize,
    duration: f64,
}

#[derive(Clone)]
struct ScanJob {
    status: String,
    progress: u8,
    result: Option<ScanSummary>,
    error: Option<String>,
    started: Instant,
}

#[derive(Deserialize)]
struct UrlScanBody {
    url: String,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    threads: Option<usize>,
    #[serde(default)]
    timeout: Option<u64>,
}

/// Start DepCheck web UI on `bind:port`.
pub async fn start_web(bind: &str, port: u16) -> Result<()> {
    let state = AppState {
        scans: Arc::new(RwLock::new(HashMap::new())),
        history: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/info", get(api_info))
        .route("/api/history", get(api_history))
        .route("/api/scan", post(api_scan_url))
        .route("/api/scan/upload", post(api_scan_upload))
        .route("/api/scan/{id}", get(api_scan_status))
        .with_state(state);

    let addr: SocketAddr = format!("{bind}:{port}").parse()?;
    eprintln!(
        "{} DepCheck web UI (scan-only) on http://{addr}",
        style::brand("weeping-angel")
    );
    eprintln!("  Detection only — no auto-exploit / publish.");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn api_info() -> Json<serde_json::Value> {
    let parsers: Vec<&str> = FileKind::all_known().iter().map(|k| k.as_str()).collect();
    Json(json!({
        "name": "weeping-angel-depcheck",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "scan-only",
        "parsers": parsers,
        "ecosystems": ["npm","pip","composer","rubygems","nuget","cargo","go","maven"],
    }))
}

async fn api_history(State(state): State<AppState>) -> Json<Vec<HistoryEntry>> {
    let hist = state.history.read().await;
    Json(hist.clone())
}

async fn api_scan_status(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> impl IntoResponse {
    let scans = state.scans.read().await;
    match scans.get(&id) {
        Some(job) => {
            let body = json!({
                "id": id,
                "status": job.status,
                "progress": job.progress,
                "error": job.error,
                "result": job.result,
                "elapsed_secs": job.started.elapsed().as_secs_f64(),
            });
            (StatusCode::OK, Json(body)).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(json!({"error":"not found"}))).into_response(),
    }
}

async fn api_scan_url(
    State(state): State<AppState>,
    Json(body): Json<UrlScanBody>,
) -> impl IntoResponse {
    let kind = body
        .r#type
        .as_deref()
        .and_then(FileKind::from_str_loose);
    let opts = ScanOptions {
        threads: body.threads.unwrap_or(20),
        timeout_secs: body.timeout.unwrap_or(10),
        quiet: true,
        kind_override: kind,
        secure_namespaces: Vec::new(),
        verbose: false,
    };

    let id = Uuid::new_v4().to_string();
    {
        let mut scans = state.scans.write().await;
        scans.insert(
            id.clone(),
            ScanJob {
                status: "running".into(),
                progress: 0,
                result: None,
                error: None,
                started: Instant::now(),
            },
        );
    }

    let state2 = state.clone();
    let id2 = id.clone();
    let url = body.url.clone();
    tokio::spawn(async move {
        let outcome = fetch_and_scan(&url, opts).await;
        finish_job(state2, id2, outcome).await;
    });

    (StatusCode::ACCEPTED, Json(json!({ "id": id, "status": "running" })))
}

async fn api_scan_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut filename = "upload".to_string();
    let mut content = String::new();
    let mut type_override: Option<FileKind> = None;
    let mut threads = 20usize;
    let mut timeout = 10u64;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                    filename = fname;
                }
                match field.text().await {
                    Ok(t) => content = t,
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": e.to_string()})),
                        )
                            .into_response();
                    }
                }
            }
            "type" => {
                if let Ok(t) = field.text().await {
                    type_override = FileKind::from_str_loose(&t);
                }
            }
            "threads" => {
                if let Ok(t) = field.text().await {
                    threads = t.parse().unwrap_or(20);
                }
            }
            "timeout" => {
                if let Ok(t) = field.text().await {
                    timeout = t.parse().unwrap_or(10);
                }
            }
            _ => {}
        }
    }

    if content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing file"})),
        )
            .into_response();
    }

    let kind = type_override.unwrap_or_else(|| {
        let hint = std::path::Path::new(&filename);
        super::detect::detect_file_type(hint, Some(&content))
    });
    if kind == FileKind::Unknown {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "unknown file type; pass type="})),
        )
            .into_response();
    }

    let input = ManifestInput {
        display: filename,
        path: None,
        content,
        kind,
    };
    let opts = ScanOptions {
        threads,
        timeout_secs: timeout,
        quiet: true,
        kind_override: Some(kind),
        secure_namespaces: Vec::new(),
        verbose: false,
    };

    let id = Uuid::new_v4().to_string();
    {
        let mut scans = state.scans.write().await;
        scans.insert(
            id.clone(),
            ScanJob {
                status: "running".into(),
                progress: 0,
                result: None,
                error: None,
                started: Instant::now(),
            },
        );
    }

    let state2 = state.clone();
    let id2 = id.clone();
    tokio::spawn(async move {
        let client: Arc<dyn RegistryClient> =
            match HttpRegistry::new(opts.timeout_secs) {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    finish_job(state2, id2, Err(e.to_string())).await;
                    return;
                }
            };
        let outcome = scan_manifest(&input, &opts, client)
            .await
            .map_err(|e| e.to_string());
        finish_job(state2, id2, outcome).await;
    });

    (StatusCode::ACCEPTED, Json(json!({ "id": id, "status": "running" }))).into_response()
}

async fn fetch_and_scan(url: &str, opts: ScanOptions) -> Result<ScanSummary, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(opts.timeout_secs.max(5)))
        .user_agent(concat!("weeping-angel-depcheck/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let body = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let input = load_url_body(url, body, opts.kind_override).map_err(|e| e.to_string())?;
    let registry: Arc<dyn RegistryClient> =
        Arc::new(HttpRegistry::new(opts.timeout_secs).map_err(|e| e.to_string())?);
    scan_manifest(&input, &opts, registry)
        .await
        .map_err(|e| e.to_string())
}

async fn finish_job(state: AppState, id: String, outcome: Result<ScanSummary, String>) {
    let mut scans = state.scans.write().await;
    if let Some(job) = scans.get_mut(&id) {
        match outcome {
            Ok(summary) => {
                let entry = HistoryEntry {
                    id: id.clone(),
                    file: summary.file.clone(),
                    ecosystem: summary.ecosystem.to_string(),
                    total: summary.total(),
                    vulnerable: summary.vulnerable.len(),
                    duration: summary.duration_secs,
                };
                job.status = "done".into();
                job.progress = 100;
                job.result = Some(summary);
                drop(scans);
                let mut hist = state.history.write().await;
                hist.insert(0, entry);
                hist.truncate(50);
            }
            Err(e) => {
                job.status = "error".into();
                job.error = Some(e);
            }
        }
    }
}

