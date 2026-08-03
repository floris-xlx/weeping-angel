//! Git-backed change-set inventory for diff-mode code scans.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// How to select files for a diff-scoped scan.
#[derive(Debug, Clone)]
pub enum DiffTarget {
    /// `git diff --name-only <base>...<head>` (triple-dot merge-base style when available)
    Revisions { base: String, head: String },
    /// Staged + unstaged vs HEAD (working tree)
    WorkingTree { base: Option<String> },
}

#[derive(Debug, Clone)]
pub struct DiffInventory {
    pub files: Vec<String>,
    pub base_revision: Option<String>,
    pub head_revision: Option<String>,
    pub content_digest_hint: String,
}

/// List changed source-like files under `repo` for the given diff target.
pub fn inventory_diff(repo: &Path, target: &DiffTarget) -> Result<DiffInventory> {
    if !repo.join(".git").exists() && git_rev_parse(repo, "HEAD").is_err() {
        bail!(
            "{} is not a git repository (diff mode requires git)",
            repo.display()
        );
    }

    let (names, base, head) = match target {
        DiffTarget::Revisions { base, head } => {
            let mut files = git_name_only(repo, &format!("{base}...{head}"))?;
            if files.is_empty() {
                // fallback double-dot
                files = git_name_only(repo, &format!("{base}..{head}"))?;
            }
            // include untracked? no for revision mode
            let head_rev = git_rev_parse(repo, head).ok();
            let base_rev = git_rev_parse(repo, base).ok();
            (files, base_rev, head_rev)
        }
        DiffTarget::WorkingTree { base } => {
            let base_ref = base.as_deref().unwrap_or("HEAD");
            let mut files = BTreeSet::new();
            for f in git_name_only(repo, base_ref)? {
                files.insert(f);
            }
            // unstaged
            for f in git_name_only_args(repo, &["diff", "--name-only", "--diff-filter=ACMR"])? {
                files.insert(f);
            }
            // staged
            for f in git_name_only_args(
                repo,
                &["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
            )? {
                files.insert(f);
            }
            // untracked (source-like only later)
            for f in git_untracked(repo)? {
                files.insert(f);
            }
            let head_rev = git_rev_parse(repo, "HEAD").ok();
            let base_rev = git_rev_parse(repo, base_ref).ok();
            (files.into_iter().collect(), base_rev, head_rev)
        }
    };

    let mut files: Vec<String> = names
        .into_iter()
        .map(|p| p.replace('\\', "/"))
        .filter(|p| is_source_like(p) && !is_noise_path(p))
        .filter(|p| repo.join(p).is_file())
        .collect();
    files.sort();
    files.dedup();

    let hint = format!(
        "diff:{}:{}:{}files",
        base.as_deref().unwrap_or("?"),
        head.as_deref().unwrap_or("?"),
        files.len()
    );

    Ok(DiffInventory {
        files,
        base_revision: base,
        head_revision: head,
        content_digest_hint: hint,
    })
}

fn is_source_like(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "py" | "js" | "jsx" | "ts" | "tsx" | "go" | "java" | "kt" | "rb" | "php"
            | "c" | "cc" | "cpp" | "h" | "hpp" | "cs" | "swift" | "scala" | "sh" | "bash"
            | "yaml" | "yml" | "toml" | "json" | "sql" | "env" | "vue" | "svelte"
    )
}

fn is_noise_path(path: &str) -> bool {
    path.contains("/node_modules/")
        || path.contains("/target/")
        || path.contains("/.venv/")
        || path.starts_with("node_modules/")
        || path.starts_with("target/")
}

fn git_name_only(repo: &Path, range: &str) -> Result<Vec<String>> {
    git_name_only_args(
        repo,
        &["diff", "--name-only", "--diff-filter=ACMR", range],
    )
}

fn git_name_only_args(repo: &Path, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("run git {} in {}", args.join(" "), repo.display()))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!("git {} failed: {err}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn git_untracked(repo: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(repo)
        .output()
        .context("git ls-files --others")?;
    if !out.status.success() {
        return Ok(vec![]);
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn git_rev_parse(repo: &Path, rev: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(repo)
        .output()
        .with_context(|| format!("git rev-parse {rev}"))?;
    if !out.status.success() {
        bail!("git rev-parse {rev} failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Resolve repo root from a path (walk up looking for .git).
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_like_filters() {
        assert!(is_source_like("src/a.py"));
        assert!(!is_source_like("img.png"));
        assert!(is_noise_path("node_modules/x/index.js"));
    }
}
