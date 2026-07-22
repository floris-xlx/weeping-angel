use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::{Client, Method, redirect::Policy};
use tokio::sync::Semaphore;
use tokio::time::{interval, MissedTickBehavior};
use url::Url;

use crate::authz::Authorization;
use crate::http::response::ResponseSnapshot;

const DEFAULT_UA: &str = "weeping-angel/0.1 (+authorized-security-scan; polite)";

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub max_body_bytes: usize,
    pub concurrency: usize,
    pub rps: f64,
    pub extra_headers: Vec<(String, String)>,
    pub cookie: Option<String>,
    pub insecure_tls: bool,
}

impl ClientConfig {
    pub fn lab_defaults() -> Self {
        Self {
            timeout: Duration::from_secs(15),
            max_redirects: 5,
            max_body_bytes: 2 * 1024 * 1024,
            concurrency: 10,
            rps: 5.0,
            extra_headers: Vec::new(),
            cookie: None,
            insecure_tls: false,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::lab_defaults()
    }
}

pub struct HttpClient {
    inner: Client,
    authz: Authorization,
    max_body_bytes: usize,
    semaphore: Arc<Semaphore>,
    rate: Arc<RateLimiter>,
    requests: AtomicU64,
    allow_write: bool,
}

impl HttpClient {
    pub fn new(authz: Authorization, cfg: ClientConfig) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(cfg.timeout)
            .user_agent(DEFAULT_UA)
            .redirect(Policy::limited(cfg.max_redirects))
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .pool_max_idle_per_host(cfg.concurrency);

        if cfg.insecure_tls {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let mut default_headers = reqwest::header::HeaderMap::new();
        if let Some(cookie) = &cfg.cookie {
            default_headers.insert(
                reqwest::header::COOKIE,
                cookie
                    .parse()
                    .map_err(|e| anyhow!("invalid cookie header: {e}"))?,
            );
        }
        for (k, v) in &cfg.extra_headers {
            let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| anyhow!("invalid header name {k}: {e}"))?;
            let value = reqwest::header::HeaderValue::from_str(v)
                .map_err(|e| anyhow!("invalid header value for {k}: {e}"))?;
            default_headers.insert(name, value);
        }
        builder = builder.default_headers(default_headers);

        let inner = builder.build().context("build HTTP client")?;
        let rps = if cfg.rps <= 0.0 { 1.0 } else { cfg.rps };
        let allow_write = authz.allow_write_methods;

        Ok(Self {
            inner,
            authz,
            max_body_bytes: cfg.max_body_bytes,
            semaphore: Arc::new(Semaphore::new(cfg.concurrency.max(1))),
            rate: Arc::new(RateLimiter::new(rps)),
            requests: AtomicU64::new(0),
            allow_write,
        })
    }

    pub fn authz(&self) -> &Authorization {
        &self.authz
    }

    pub fn request_count(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }

    pub async fn get(&self, url: &Url) -> Result<ResponseSnapshot> {
        self.request(Method::GET, url, None, None).await
    }

    pub async fn request(
        &self,
        method: Method,
        url: &Url,
        body: Option<String>,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Result<ResponseSnapshot> {
        if !self.authz.url_in_scope(url) {
            return Err(anyhow!("URL out of scope: {url}"));
        }
        if is_write_method(&method) && !self.allow_write {
            return Err(anyhow!(
                "write method {} blocked (pass --allow-write-methods)",
                method
            ));
        }

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| anyhow!("concurrency semaphore closed"))?;
        self.rate.until_ready().await;

        let mut req = self.inner.request(method.clone(), url.clone());
        if let Some(headers) = extra_headers {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }
        if let Some(b) = body {
            req = req.body(b);
        }

        self.requests.fetch_add(1, Ordering::Relaxed);
        let resp = req.send().await.with_context(|| format!("{method} {url}"))?;

        // Re-validate final URL after redirects
        let final_url = resp.url().clone();
        if !self.authz.url_in_scope(&final_url) {
            return Err(anyhow!(
                "redirect left authorized scope: {url} -> {final_url}"
            ));
        }

        let status = resp.status();
        let mut headers = HashMap::new();
        for (k, v) in resp.headers().iter() {
            if let Ok(val) = v.to_str() {
                // preserve multiple set-cookie by joining last-wins for map; also store individually if needed
                headers
                    .entry(k.as_str().to_string())
                    .and_modify(|existing: &mut String| {
                        if k.as_str().eq_ignore_ascii_case("set-cookie") {
                            existing.push('\n');
                            existing.push_str(val);
                        }
                    })
                    .or_insert_with(|| val.to_string());
            }
        }
        let content_type = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone());

        let bytes = resp.bytes().await.context("read body")?;
        let truncated = if bytes.len() > self.max_body_bytes {
            &bytes[..self.max_body_bytes]
        } else {
            &bytes[..]
        };
        let body = String::from_utf8_lossy(truncated).into_owned();

        Ok(ResponseSnapshot {
            url: url.clone(),
            final_url,
            status,
            headers,
            body,
            content_type,
        })
    }
}

fn is_write_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

struct RateLimiter {
    interval_ms: u64,
    ticker: tokio::sync::Mutex<tokio::time::Interval>,
}

impl RateLimiter {
    fn new(rps: f64) -> Self {
        let interval_ms = (1000.0 / rps).ceil().max(1.0) as u64;
        let mut ticker = interval(Duration::from_millis(interval_ms));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // consume first immediate tick
        ticker.reset();
        Self {
            interval_ms,
            ticker: tokio::sync::Mutex::new(ticker),
        }
    }

    async fn until_ready(&self) {
        let mut t = self.ticker.lock().await;
        t.tick().await;
        let _ = self.interval_ms;
    }
}
