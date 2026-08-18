use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("rate limited; honor Retry-After")]
    RateLimited,
}
