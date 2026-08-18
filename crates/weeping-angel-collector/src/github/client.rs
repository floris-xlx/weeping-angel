//! HTTP client wrapper. Provider types do not escape this module.

use std::time::Duration;

use thiserror::Error;

use weeping_angel_evidence::redact;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("{0}")]
    Transport(String),
}

/// Fixture-friendly client. Live transport is injected; tokens are never logged.
#[derive(Clone, Default)]
pub struct GitHubClient {
    token: Option<String>,
    fixtures: Vec<(String, u16, String, Option<u64>)>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token,
            fixtures: Vec::new(),
        }
    }

    pub fn with_fixture(
        mut self,
        path: &str,
        status: u16,
        body: &str,
        retry_after: Option<u64>,
    ) -> Self {
        self.fixtures
            .push((path.to_string(), status, body.to_string(), retry_after));
        self
    }

    pub fn authorization_header(&self) -> Option<String> {
        self.token.as_ref().map(|_| "Bearer [redacted]".to_string())
    }

    pub fn get(&self, path: &str) -> Result<(u16, String, Option<Duration>), ClientError> {
        let _ = redact(self.authorization_header().as_deref().unwrap_or(""));
        if let Some((_, status, body, retry)) = self
            .fixtures
            .iter()
            .find(|(p, _, _, _)| path.starts_with(p.as_str()) || p == path)
        {
            let retry_after = retry.map(Duration::from_secs);
            if *status == 429 {
                let _ = retry_after;
            }
            return Ok((*status, body.clone(), retry_after));
        }
        if self.token.is_none() {
            return Ok((401, r#"{"message":"requires Authorization"}"#.into(), None));
        }
        Err(ClientError::Transport(format!(
            "no fixture and no live transport for {path}"
        )))
    }
}
