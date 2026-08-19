use thiserror::Error;

use weeping_angel_evidence::redact;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("rate limited; honor Retry-After")]
    RateLimited,
}

/// Fold GitHub token prefixes (including installation `ghs_`) after shared redact.
pub fn sanitize_diagnostic(text: &str) -> String {
    let mut out = redact(text);
    for needle in ["ghs_", "ghu_", "ghr_"] {
        fold_prefix(&mut out, needle);
    }
    out
}

fn fold_prefix(out: &mut String, needle: &str) {
    if let Some(idx) = out.find(needle) {
        let rest = &out[idx + needle.len()..];
        let cut = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(rest.len());
        out.replace_range(idx..idx + needle.len() + cut, "[redacted]");
    }
}
