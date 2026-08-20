//! Shared CLI value parsers: booleans, CSV lists, headers, consent flags.

use anyhow::{Result, bail};

/// Parse a loose boolean: true/yes/y/1/on vs false/no/n/0/off.
pub fn parse_bool_loose(s: &str) -> Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "y" | "1" | "on" => Ok(true),
        "false" | "no" | "n" | "0" | "off" => Ok(false),
        other => Err(format!(
            "invalid boolean '{other}' (expected true|false|yes|no|1|0|on|off)"
        )),
    }
}

/// Consent flag: truthy values only. Explicit false is an error (refuse loudly).
pub fn parse_consent(s: &str) -> Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "" | "true" | "yes" | "y" | "1" | "on" | "i-own-this" | "owned" | "authorized" => {
            Ok(true)
        }
        "false" | "no" | "n" | "0" | "off" => Err(
            "consent explicitly set to false — pass --i-own-this (or --i-own-this=true) only when you own or have written permission to test the target"
                .into(),
        ),
        other => Err(format!(
            "invalid --i-own-this value '{other}' (use bare --i-own-this or =true|yes|1|on)"
        )),
    }
}

/// Optional safety bool that accepts bare flag or =true/false.
pub fn parse_optional_bool(s: &str) -> Result<bool, String> {
    parse_bool_loose(if s.is_empty() { "true" } else { s })
}

/// Split comma/whitespace/semicolon separated lists; drop empties.
pub fn split_list(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| p.trim_matches(|c| c == '"' || c == '\'').to_string())
        .collect()
}

/// Expand a list of raw CLI values that may each contain CSV segments.
pub fn expand_list_args(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|v| split_list(v))
        .filter(|s| !s.is_empty())
        .collect()
}

/// True when a token already looks like `Name=Value` or `Name: Value`.
pub fn looks_like_kv(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    split_kv(s).is_some()
}

/// Split `Name=Value`, `Name: Value`, or `Name Value` into (name, value).
pub fn split_kv(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (k, v) = if let Some((k, v)) = s.split_once('=') {
        (k, v)
    } else if let Some((k, v)) = s.split_once(':') {
        (k, v)
    } else if let Some((k, v)) = s.split_once(char::is_whitespace) {
        (k, v)
    } else {
        return None;
    };
    let k = k.trim();
    let v = v.trim();
    if k.is_empty() {
        return None;
    }
    Some((k, v))
}

/// Pair a bare key with the following token (`Name` + `Value` → `Name=Value`).
/// Tokens that already contain `=` or `:` are left intact.
pub fn coalesce_kv_tokens(raw: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let cur = raw[i].trim();
        if cur.is_empty() {
            i += 1;
            continue;
        }
        if looks_like_kv(cur) {
            out.push(cur.to_string());
            i += 1;
            continue;
        }
        if i + 1 < raw.len() {
            let nxt = raw[i + 1].trim();
            if !nxt.is_empty() && !looks_like_kv(nxt) {
                out.push(format!("{cur}={nxt}"));
                i += 2;
                continue;
            }
        }
        out.push(cur.to_string());
        i += 1;
    }
    out
}

/// Parse `Name: Value`, `Name=Value`, or `Name Value` header lines.
/// Adjacent bare tokens (`Name`, `Value`) are paired as `Name=Value`.
pub fn parse_header_lines(lines: &[String]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for line in coalesce_kv_tokens(lines) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((k, v)) = split_kv(line) else {
            bail!(
                "invalid --header (expected 'Name: Value', 'Name=Value', or 'Name Value'): {line}"
            );
        };
        out.push((k.to_string(), v.to_string()));
    }
    Ok(out)
}

/// Normalize cookie CLI tokens to `name=value` pairs, then join with `; `.
pub fn cookie_header_from_args(raw: &[String]) -> Option<String> {
    let parts: Vec<String> = coalesce_kv_tokens(raw)
        .into_iter()
        .map(|tok| {
            if let Some((k, v)) = split_kv(&tok) {
                format!("{k}={v}")
            } else {
                tok
            }
        })
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

/// Normalize allow-host entries: extract host from URL, expand CSV, lower-case.
pub fn normalize_allow_hosts(raw: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in raw {
        for part in split_list(&item) {
            if let Some(h) = host_from_allow_entry(&part) {
                out.push(h);
            }
        }
    }
    out
}

fn host_from_allow_entry(raw: &str) -> Option<String> {
    let s = raw
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim_end_matches('/');
    if s.is_empty() {
        return None;
    }

    // Full URL → host
    if s.contains("://") || s.starts_with("//") {
        let candidate = if s.starts_with("//") {
            format!("https:{s}")
        } else {
            s.to_string()
        };
        if let Ok(u) = url::Url::parse(&candidate)
            && let Some(h) = u.host_str()
        {
            return Some(normalize_host_token(h));
        }
    }

    // Strip path if someone passed host/path without scheme
    let host_part = s.split('/').next().unwrap_or(s);
    // Drop userinfo
    let host_part = host_part.rsplit('@').next().unwrap_or(host_part);
    // Keep wildcard prefix; strip port for exact hosts but keep *. patterns
    if host_part.starts_with("*.") || host_part.starts_with('.') {
        return Some(normalize_host_token(host_part));
    }
    let host_only = if let Some((h, port)) = host_part.rsplit_once(':') {
        // IPv6 in brackets
        if host_part.starts_with('[') {
            host_part
        } else if port.chars().all(|c| c.is_ascii_digit()) {
            h
        } else {
            host_part
        }
    } else {
        host_part
    };
    Some(normalize_host_token(host_only))
}

fn normalize_host_token(host: &str) -> String {
    host.trim()
        .trim_end_matches('.')
        .trim_matches(|c| c == '[' || c == ']')
        .to_ascii_lowercase()
}

/// Log HTTP verbosity for live request lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogHttp {
    Full,
    #[default]
    Compact,
    Summary,
    Off,
}

impl LogHttp {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "full" | "all" | "verbose" => Some(Self::Full),
            "compact" | "short" | "default" => Some(Self::Compact),
            "summary" | "sum" | "phase" => Some(Self::Summary),
            "off" | "none" | "quiet" | "0" | "false" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Compact => "compact",
            Self::Summary => "summary",
            Self::Off => "off",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_truthy() {
        assert!(parse_consent("yes").unwrap());
        assert!(parse_consent("true").unwrap());
        assert!(parse_consent("1").unwrap());
    }

    #[test]
    fn consent_false_errors() {
        assert!(parse_consent("false").is_err());
        assert!(parse_consent("no").is_err());
    }

    #[test]
    fn split_csv_and_space() {
        assert_eq!(split_list("a, b;c\td"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn allow_host_from_url() {
        let h = normalize_allow_hosts(vec![
            "https://App.Example.com/path".into(),
            "foo.com,bar.com".into(),
            "*.cdn.example.com".into(),
        ]);
        assert!(h.contains(&"app.example.com".to_string()));
        assert!(h.contains(&"foo.com".to_string()));
        assert!(h.contains(&"bar.com".to_string()));
        assert!(h.contains(&"*.cdn.example.com".to_string()));
    }

    #[test]
    fn header_colon_and_eq() {
        let h = parse_header_lines(&[
            "Authorization: Bearer x".into(),
            "X-Test=1".into(),
            "X-Space secret".into(),
        ])
        .unwrap();
        assert_eq!(h[0].0, "Authorization");
        assert_eq!(h[0].1, "Bearer x");
        assert_eq!(h[1].0, "X-Test");
        assert_eq!(h[1].1, "1");
        assert_eq!(h[2].0, "X-Space");
        assert_eq!(h[2].1, "secret");
    }

    #[test]
    fn header_pairs_adjacent_tokens() {
        let h = parse_header_lines(&["X-Api-Key".into(), "secret".into(), "X-B=2".into()]).unwrap();
        assert_eq!(
            h,
            vec![
                ("X-Api-Key".into(), "secret".into()),
                ("X-B".into(), "2".into())
            ]
        );
    }

    #[test]
    fn cookie_space_and_equals() {
        assert_eq!(
            cookie_header_from_args(&["session=admin".into()]).as_deref(),
            Some("session=admin")
        );
        assert_eq!(
            cookie_header_from_args(&["session".into(), "admin".into(), "role=ops".into()])
                .as_deref(),
            Some("session=admin; role=ops")
        );
    }

    #[test]
    fn expand_list_args_dedupes_not_required() {
        let v = expand_list_args(&["a,a".into(), "b".into()]);
        assert_eq!(v, vec!["a", "a", "b"]);
    }

    #[test]
    fn log_http_aliases() {
        assert_eq!(LogHttp::parse("verbose"), Some(LogHttp::Full));
        assert_eq!(LogHttp::parse("default"), Some(LogHttp::Compact));
        assert_eq!(LogHttp::parse("phase"), Some(LogHttp::Summary));
        assert_eq!(LogHttp::parse("0"), Some(LogHttp::Off));
    }

    #[test]
    fn parse_bool_rejects_empty() {
        assert!(parse_bool_loose("").is_err());
        assert!(parse_bool_loose("  ").is_err());
    }
}
