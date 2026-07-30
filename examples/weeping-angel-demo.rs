//! Intentionally vulnerable local lab for weeping-angel demos.
//! Bind: 127.0.0.1 only. Do not expose to the network.
//!
//! Router lives in `weeping_angel::lab` (feature `demo`).

use std::net::SocketAddr;

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

    let app = weeping_angel::lab::router();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("weeping-angel-demo listening on http://{addr}");
    eprintln!("Scan with:");
    eprintln!(
        "  cargo run --bin weeping-angel -- scan http://127.0.0.1:{port}/ --i-own-this --allow-host 127.0.0.1 --profile deep --enable-active --cookie \"session=admin-session\" --compare-auth"
    );

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
