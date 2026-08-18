//! False-positive filters for non-registry / non-confusable dependencies.

use regex::Regex;
use std::sync::OnceLock;

use super::types::{Ecosystem, PackageRef};

/// Return true when `spec` looks like a VCS / URL / path dependency (skip).
pub fn is_remote_or_path_spec(spec: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^(git\+|git://|https?://|ssh://|file:|github:|gitlab:|bitbucket:|hg\+|svn\+)")
            .expect("remote spec regex")
    });
    re.is_match(spec.trim())
}

/// Resolve `npm:real@ver` aliases → (name, version). Returns None if not an alias.
pub fn resolve_npm_alias(version_str: &str) -> Option<(String, String)> {
    let rest = version_str.strip_prefix("npm:")?;
    let (name, ver) = if rest.starts_with('@') {
        if let Some(idx) = rest[1..].find('@') {
            let split = idx + 1;
            (&rest[..split], &rest[split + 1..])
        } else {
            (rest, "*")
        }
    } else if let Some(idx) = rest.find('@') {
        (&rest[..idx], &rest[idx + 1..])
    } else {
        (rest, "*")
    };
    if name.is_empty() {
        return None;
    }
    Some((
        name.to_string(),
        if ver.is_empty() {
            "*".into()
        } else {
            ver.into()
        },
    ))
}

/// Composer platform packages that cannot be confused on Packagist.
pub fn is_composer_platform(name: &str) -> bool {
    name == "php"
        || name == "composer-plugin-api"
        || name.starts_with("ext-")
        || name.starts_with("lib-")
}

/// Filter a parsed package map for registry checking.
pub fn filter_packages(
    ecosystem: Ecosystem,
    packages: impl IntoIterator<Item = PackageRef>,
) -> Vec<PackageRef> {
    packages
        .into_iter()
        .filter(|p| !should_skip(ecosystem, &p.name, &p.version))
        .collect()
}

/// Split packages into (to_check, known_secure) using confused-style `-s` patterns.
///
/// Patterns support `*` wildcards (e.g. `@mycompany/*`, `com.acme.*`).
pub fn partition_secure_namespaces(
    packages: Vec<PackageRef>,
    patterns: &[String],
) -> (Vec<PackageRef>, Vec<PackageRef>) {
    if patterns.is_empty() {
        return (packages, Vec::new());
    }
    let mut check = Vec::new();
    let mut secure = Vec::new();
    for pkg in packages {
        if matches_any_namespace(&pkg.name, patterns) {
            secure.push(pkg);
        } else {
            check.push(pkg);
        }
    }
    (check, secure)
}

/// True when `name` matches any known-secure namespace pattern.
pub fn matches_any_namespace(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| matches_secure_namespace(name, p))
}

/// Glob-style match: `*` matches any sequence (including `/` and `@` segments).
pub fn matches_secure_namespace(name: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return false;
    }
    if pattern == "*" {
        return true;
    }
    // Exact match fast-path
    if !pattern.contains('*') {
        return name.eq_ignore_ascii_case(pattern);
    }
    wildcard_match(name, pattern)
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    // Case-insensitive segment match with * wildcards
    let text = text.to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return text == pattern;
    }
    let mut rest = text.as_str();
    if !parts[0].is_empty() {
        if !rest.starts_with(parts[0]) {
            return false;
        }
        rest = &rest[parts[0].len()..];
    }
    for (i, part) in parts.iter().enumerate().skip(1) {
        if part.is_empty() {
            if i == parts.len() - 1 {
                return true;
            }
            continue;
        }
        if i == parts.len() - 1 {
            return rest.ends_with(part);
        }
        if let Some(idx) = rest.find(part) {
            rest = &rest[idx + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Parse confused-style comma-separated `-s` values into patterns.
pub fn parse_secure_namespace_list(raw: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for item in raw {
        for part in item.split(',') {
            let p = part.trim();
            if !p.is_empty() {
                out.push(p.to_string());
            }
        }
    }
    out
}

fn should_skip(ecosystem: Ecosystem, name: &str, version: &str) -> bool {
    if name.is_empty() || name.starts_with('.') {
        return true;
    }
    match ecosystem {
        Ecosystem::Composer if is_composer_platform(name) => true,
        Ecosystem::Npm if is_remote_or_path_spec(version) => true,
        Ecosystem::Npm if version.starts_with("link:") || version.starts_with("workspace:") => true,
        Ecosystem::Pip if is_remote_or_path_spec(version) => true,
        Ecosystem::Pip if version.starts_with('.') || version.starts_with('/') => true,
        Ecosystem::Rubygems if is_remote_or_path_spec(version) => true,
        Ecosystem::Cargo if version.contains("path") || version.contains("git") => {
            // Cargo.toml path/git already filtered in parser; keep defensive.
            version.contains("path =") || version.contains("git =")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_scoped_npm_alias() {
        let (n, v) = resolve_npm_alias("npm:@scope/pkg@^1.0.0").unwrap();
        assert_eq!(n, "@scope/pkg");
        assert_eq!(v, "^1.0.0");
    }

    #[test]
    fn skips_composer_platform() {
        assert!(is_composer_platform("php"));
        assert!(is_composer_platform("ext-json"));
        assert!(!is_composer_platform("vendor/pkg"));
    }

    #[test]
    fn secure_namespace_wildcards() {
        assert!(matches_secure_namespace(
            "@mycompany/internal_package1",
            "@mycompany/*"
        ));
        assert!(matches_secure_namespace("com.acme.billing", "com.acme.*"));
        assert!(!matches_secure_namespace(
            "@other/internal_package1",
            "@mycompany/*"
        ));
        assert!(matches_secure_namespace("exact-name", "exact-name"));
        let pkgs = vec![
            PackageRef::new("@mycompany/a", "1"),
            PackageRef::new("orphan-internal", "1"),
        ];
        let (check, secure) = partition_secure_namespaces(pkgs, &["@mycompany/*".into()]);
        assert_eq!(secure.len(), 1);
        assert_eq!(check.len(), 1);
        assert_eq!(check[0].name, "orphan-internal");
    }
}
