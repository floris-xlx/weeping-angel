use std::path::Path;

use anyhow::{Context, Result};

const EMBEDDED: &str = include_str!("../../wordlists/common-paths.txt");

pub fn load_paths(path: &Path) -> Result<Vec<String>> {
    let raw = if path.exists() {
        std::fs::read_to_string(path).with_context(|| format!("read wordlist {}", path.display()))?
    } else {
        EMBEDDED.to_string()
    };
    Ok(parse_wordlist(&raw))
}

pub fn parse_wordlist(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn is_sensitive_path(path: &str) -> bool {
    let p = path.to_ascii_lowercase();
    const KEYS: &[&str] = &[
        ".env",
        ".git",
        ".svn",
        ".htpasswd",
        "phpinfo",
        "actuator",
        "server-status",
        "swagger",
        "openapi",
        "graphql",
        "backup",
        "dump.sql",
        "id_rsa",
        "credentials",
        "config.json",
        "web.config",
        "debug",
        "trace.axd",
        "elmah",
        "_profiler",
        "aws/credentials",
        "docker-compose",
        "wp-config",
    ];
    KEYS.iter().any(|k| p.contains(k))
}

pub fn is_interesting_status(status: u16) -> bool {
    matches!(status, 200..=399 | 401 | 403 | 405 | 500)
}
