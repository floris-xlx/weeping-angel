//! Loki-style inspector: git commit that introduced a vulnerable dependency name.
//!
//! Recon/analysis only — no payload injection or attack mode.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::engines::git_diff::find_git_root;

/// First commit that appears to introduce `package` into a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntroductionCommit {
    pub package: String,
    pub commit: String,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub file: String,
}

/// Find git introductions for each package name under `start` (walks to repo root).
pub fn inspect_introductions(
    start: &Path,
    packages: &[String],
    manifest_hint: Option<&Path>,
) -> Result<Vec<IntroductionCommit>> {
    let Some(repo) = find_git_root(start) else {
        bail!("not a git repository (inspector requires git)");
    };

    let search_files = manifest_candidates(&repo, manifest_hint);
    let mut out = Vec::new();
    for pkg in packages {
        if let Some(hit) = find_introduction(&repo, pkg, &search_files)? {
            out.push(hit);
        }
    }
    Ok(out)
}

fn manifest_candidates(repo: &Path, hint: Option<&Path>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(h) = hint {
        if let Ok(rel) = h.strip_prefix(repo) {
            files.push(rel.to_path_buf());
        } else if h.is_file() {
            files.push(h.to_path_buf());
        }
    }
    for name in [
        "package.json",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "pnpm-lock.yaml",
    ] {
        let p = PathBuf::from(name);
        if repo.join(&p).is_file() && !files.iter().any(|f| f == &p) {
            files.push(p);
        }
    }
    files
}

fn find_introduction(
    repo: &Path,
    package: &str,
    files: &[PathBuf],
) -> Result<Option<IntroductionCommit>> {
    // Prefer pickaxe search for the package name string in manifests.
    let needle = package.to_string();
    let mut args: Vec<String> = vec![
        "log".into(),
        "-S".into(),
        needle.clone(),
        "--pretty=format:%H\t%s\t%an\t%ad".into(),
        "--date=iso".into(),
        "--reverse".into(),
        "--".into(),
    ];
    for f in files {
        args.push(f.display().to_string());
    }

    let out = Command::new("git")
        .args(&args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("git log -S in {}", repo.display()))?;

    if !out.status.success() {
        // Fallback: -G regex
        return find_introduction_regex(repo, package, files);
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(first) = stdout.lines().next().filter(|l| !l.trim().is_empty()) else {
        return find_introduction_regex(repo, package, files);
    };
    Ok(Some(parse_log_line(package, first, files)?))
}

fn find_introduction_regex(
    repo: &Path,
    package: &str,
    files: &[PathBuf],
) -> Result<Option<IntroductionCommit>> {
    let escaped = regex_escape(package);
    let mut args: Vec<String> = vec![
        "log".into(),
        "-G".into(),
        format!(r#"[\"']{}[\"']"#, escaped),
        "--pretty=format:%H\t%s\t%an\t%ad".into(),
        "--date=iso".into(),
        "--reverse".into(),
        "--".into(),
    ];
    for f in files {
        args.push(f.display().to_string());
    }
    let out = Command::new("git")
        .args(&args)
        .current_dir(repo)
        .output()
        .context("git log -G")?;
    if !out.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(first) = stdout.lines().next().filter(|l| !l.trim().is_empty()) else {
        return Ok(None);
    };
    Ok(Some(parse_log_line(package, first, files)?))
}

fn parse_log_line(package: &str, line: &str, files: &[PathBuf]) -> Result<IntroductionCommit> {
    let parts: Vec<&str> = line.splitn(4, '\t').collect();
    if parts.len() < 4 {
        bail!("unexpected git log line: {line}");
    }
    let file = files
        .first()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "package.json".into());
    Ok(IntroductionCommit {
        package: package.to_string(),
        commit: parts[0].to_string(),
        subject: parts[1].to_string(),
        author: parts[2].to_string(),
        date: parts[3].to_string(),
        file,
    })
}

fn regex_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if matches!(
            c,
            '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_escapes_scope() {
        let e = regex_escape("@scope/pkg");
        assert!(e.contains(r"@scope/pkg") || e.contains("scope"));
    }
}
