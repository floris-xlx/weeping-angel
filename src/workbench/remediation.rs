//! Pattern-based remediation: generate unified diffs for known rule families.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationRequest {
    pub finding_id: String,
    pub rule_id: String,
    pub path: String,
    pub start_line: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemediationResult {
    pub finding_id: String,
    pub rule_id: String,
    pub strategy: String,
    pub state: String,
    pub summary: String,
    pub patch_path: Option<String>,
    pub patch_preview: Option<String>,
    pub files_touched: Vec<String>,
}

/// Load reportable findings from a sealed scan as remediation requests.
pub fn requests_from_scan(scan_dir: &Path) -> Result<Vec<RemediationRequest>> {
    let findings: Value =
        serde_json::from_str(&fs::read_to_string(scan_dir.join("findings.json"))?)?;
    let mut out = Vec::new();
    if let Some(arr) = findings["findings"].as_array() {
        for f in arr {
            let loc = f["locations"]
                .as_array()
                .and_then(|a| a.first())
                .cloned()
                .unwrap_or(Value::Null);
            let path = loc["path"].as_str().unwrap_or("").to_string();
            let start_line = loc["startLine"].as_u64().unwrap_or(1) as u32;
            if path.is_empty() {
                continue;
            }
            out.push(RemediationRequest {
                finding_id: f["findingId"].as_str().unwrap_or("").into(),
                rule_id: f["ruleId"].as_str().unwrap_or("").into(),
                path,
                start_line,
                title: f["title"].as_str().unwrap_or("").into(),
            });
        }
    }
    Ok(out)
}

/// Generate a unified diff patch for one finding (does not modify source).
pub fn generate_patch(
    source_root: &Path,
    scan_dir: &Path,
    req: &RemediationRequest,
) -> Result<RemediationResult> {
    let src_path = source_root.join(&req.path);
    if !src_path.is_file() {
        bail!("source file missing: {}", src_path.display());
    }
    let original =
        fs::read_to_string(&src_path).with_context(|| format!("read {}", src_path.display()))?;
    let (patched, strategy, summary) = apply_rule_fix(&original, req)?;

    if patched == original {
        return Ok(RemediationResult {
            finding_id: req.finding_id.clone(),
            rule_id: req.rule_id.clone(),
            strategy: strategy.clone(),
            state: "failed".into(),
            summary: format!("No safe algorithmic patch for `{}`: {summary}", req.rule_id),
            patch_path: None,
            patch_preview: None,
            files_touched: vec![],
        });
    }

    let patch = unified_diff(&req.path, &original, &patched);
    let patch_dir = scan_dir.join("remediation").join(&req.finding_id);
    fs::create_dir_all(&patch_dir)?;
    let patch_path = patch_dir.join("fix.patch");
    fs::write(&patch_path, &patch)?;
    fs::write(
        patch_dir.join("summary.json"),
        serde_json::to_string_pretty(&json_meta(req, &strategy, &summary))?,
    )?;

    let preview: String = patch.chars().take(4000).collect();
    Ok(RemediationResult {
        finding_id: req.finding_id.clone(),
        rule_id: req.rule_id.clone(),
        strategy,
        state: "generated".into(),
        summary,
        patch_path: Some(patch_path.display().to_string()),
        patch_preview: Some(preview),
        files_touched: vec![req.path.clone()],
    })
}

/// Apply a previously generated patch to source_root (best-effort line rewrite for our generators).
pub fn apply_patch(source_root: &Path, patch_path: &Path) -> Result<RemediationResult> {
    let patch = fs::read_to_string(patch_path)?;
    // Parse minimal unified diff for single-file full rewrites we emit
    let (rel, new_body) = parse_simple_full_file_patch(&patch)?;
    let dest = source_root.join(&rel);
    if !dest.is_file() {
        bail!("target missing: {}", dest.display());
    }
    // backup
    let bak = dest.with_extension(format!(
        "{}.wa.bak",
        dest.extension().and_then(|e| e.to_str()).unwrap_or("bak")
    ));
    fs::copy(&dest, &bak)?;
    fs::write(&dest, new_body)?;
    Ok(RemediationResult {
        finding_id: String::new(),
        rule_id: String::new(),
        strategy: "apply-unified-diff".into(),
        state: "applied".into(),
        summary: format!("Applied patch to `{rel}` (backup {})", bak.display()),
        patch_path: Some(patch_path.display().to_string()),
        patch_preview: None,
        files_touched: vec![rel],
    })
}

/// Verify by re-running engines on a single file and checking the fingerprint family is gone or reduced.
pub fn verify_file_clean(
    source_root: &Path,
    rel_path: &str,
    rule_id: &str,
) -> Result<RemediationResult> {
    let path = source_root.join(rel_path);
    let content = fs::read_to_string(&path)?;
    let hits = crate::engines::scan_source_file(rel_path, &content);
    let remaining = hits.iter().filter(|h| h.rule_id == rule_id).count();
    let state = if remaining == 0 { "verified" } else { "failed" };
    Ok(RemediationResult {
        finding_id: String::new(),
        rule_id: rule_id.into(),
        strategy: "re-scan-engine".into(),
        state: state.into(),
        summary: if remaining == 0 {
            format!("Rule `{rule_id}` no longer matches `{rel_path}`")
        } else {
            format!("Rule `{rule_id}` still has {remaining} hit(s) on `{rel_path}`")
        },
        patch_path: None,
        patch_preview: None,
        files_touched: vec![rel_path.into()],
    })
}

fn apply_rule_fix(original: &str, req: &RemediationRequest) -> Result<(String, String, String)> {
    let line_idx = req.start_line.saturating_sub(1) as usize;
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    if line_idx >= lines.len() {
        bail!(
            "start_line {} out of range for {}",
            req.start_line,
            req.path
        );
    }
    let line = lines[line_idx].clone();

    // Family-specific safe transforms
    if req.rule_id.contains("command-injection.shell-true")
        || line.to_ascii_lowercase().contains("shell=true")
    {
        let new_line = line
            .replace("shell=True", "shell=False")
            .replace("shell=true", "shell=false");
        if new_line != line {
            lines[line_idx] = new_line;
            return Ok((
                join_preserve(original, &lines),
                "shell-false".into(),
                "Set shell=False to avoid shell injection surface.".into(),
            ));
        }
    }

    if req.rule_id.contains("secrets.") || req.rule_id.contains("github-pat") {
        // Redact literal assignment values on that line
        let redacted = redact_string_literals(&line);
        if redacted != line {
            lines[line_idx] =
                format!("{redacted}  # weeping-angel: secret removed — load from env");
            return Ok((
                join_preserve(original, &lines),
                "secret-redact".into(),
                "Replaced secret literal with placeholder; rotate the leaked credential.".into(),
            ));
        }
    }

    if req.rule_id.contains("xss.innerhtml") || line.contains("innerHTML") {
        let new_line = line.replace("innerHTML", "textContent");
        if new_line != line {
            lines[line_idx] = new_line;
            return Ok((
                join_preserve(original, &lines),
                "innerhtml-to-textcontent".into(),
                "Prefer textContent over innerHTML for untrusted data.".into(),
            ));
        }
    }

    if req.rule_id.contains("sql-injection") {
        // Comment the line and add TODO for parameterization — never invent SQL rewrite
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        lines[line_idx] = format!(
            "{indent}# weeping-angel: blocked unsafe SQL construction — use bound parameters\n{indent}raise RuntimeError(\"sql construction blocked pending parameterized rewrite\")  # was: {}",
            line.trim()
        );
        return Ok((
            join_preserve(original, &lines),
            "sql-fail-closed".into(),
            "Fail-closed stub: force parameterization instead of string SQL.".into(),
        ));
    }

    Ok((
        original.to_string(),
        "none".into(),
        "No codemod for this rule family yet.".into(),
    ))
}

fn redact_string_literals(line: &str) -> String {
    // Replace long quoted tokens
    let mut out = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' || c == '\'' {
            let quote = c;
            let mut lit = String::new();
            while let Some(&n) = chars.peek() {
                chars.next();
                if n == quote {
                    break;
                }
                lit.push(n);
            }
            if lit.len() >= 16 {
                out.push(quote);
                out.push_str("REDACTED_USE_ENV");
                out.push(quote);
            } else {
                out.push(quote);
                out.push_str(&lit);
                out.push(quote);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn join_preserve(original: &str, lines: &[String]) -> String {
    let ends_nl = original.ends_with('\n');
    let mut s = lines.join("\n");
    if ends_nl {
        s.push('\n');
    }
    s
}

fn unified_diff(path: &str, old: &str, new: &str) -> String {
    // Emit a simple full-file replacement style diff for reliability
    let mut out = String::new();
    out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));
    out.push_str("@@ full-file @@\n");
    for line in old.lines() {
        out.push_str(&format!("-{line}\n"));
    }
    for line in new.lines() {
        out.push_str(&format!("+{line}\n"));
    }
    if new.ends_with('\n') && !out.ends_with("+\n") {
        // keep
    }
    out
}

fn parse_simple_full_file_patch(patch: &str) -> Result<(String, String)> {
    let mut path = String::new();
    let mut new_lines = Vec::new();
    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            path = rest.to_string();
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            new_lines.push(line[1..].to_string());
        }
    }
    if path.is_empty() {
        bail!("patch missing +++ b/ path");
    }
    let mut body = new_lines.join("\n");
    body.push('\n');
    Ok((path, body))
}

fn json_meta(req: &RemediationRequest, strategy: &str, summary: &str) -> Value {
    serde_json::json!({
        "findingId": req.finding_id,
        "ruleId": req.rule_id,
        "path": req.path,
        "startLine": req.start_line,
        "strategy": strategy,
        "summary": summary,
    })
}

/// Batch generate for all findings in a scan.
pub fn generate_all(source_root: &Path, scan_dir: &Path) -> Result<Vec<RemediationResult>> {
    let reqs = requests_from_scan(scan_dir)?;
    let mut out = Vec::new();
    for req in reqs {
        match generate_patch(source_root, scan_dir, &req) {
            Ok(r) => out.push(r),
            Err(e) => out.push(RemediationResult {
                finding_id: req.finding_id,
                rule_id: req.rule_id,
                strategy: "error".into(),
                state: "failed".into(),
                summary: e.to_string(),
                patch_path: None,
                patch_preview: None,
                files_touched: vec![],
            }),
        }
    }
    let report_path = scan_dir.join("remediation").join("index.json");
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, serde_json::to_string_pretty(&out)?)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generates_shell_false_patch() {
        let dir = tempdir().unwrap();
        let src_root = dir.path();
        fs::create_dir_all(src_root.join("pkg")).unwrap();
        fs::write(
            src_root.join("pkg/run.py"),
            "import subprocess\nsubprocess.run(cmd, shell=True)\n",
        )
        .unwrap();
        let scan = dir.path().join("scan");
        fs::create_dir_all(&scan).unwrap();
        let req = RemediationRequest {
            finding_id: "csf_test".into(),
            rule_id: "command-injection.shell-true".into(),
            path: "pkg/run.py".into(),
            start_line: 2,
            title: "shell".into(),
        };
        let r = generate_patch(src_root, &scan, &req).unwrap();
        assert_eq!(r.state, "generated");
        assert!(r.patch_path.is_some());
        let patch_path = r.patch_path.unwrap();
        let patch = fs::read_to_string(&patch_path).unwrap();
        assert!(patch.contains("shell=False") || patch.contains("shell=false"));

        let applied = apply_patch(src_root, Path::new(&patch_path)).unwrap();
        assert_eq!(applied.state, "applied");
        let body = fs::read_to_string(src_root.join("pkg/run.py")).unwrap();
        assert!(body.contains("shell=False") || body.contains("shell=false"));

        let v = verify_file_clean(src_root, "pkg/run.py", "command-injection.shell-true").unwrap();
        assert_eq!(v.state, "verified", "{}", v.summary);
    }

    #[test]
    fn generate_all_writes_index() {
        let dir = tempdir().unwrap();
        let src_root = dir.path().join("src");
        fs::create_dir_all(&src_root).unwrap();
        fs::write(src_root.join("x.js"), "el.innerHTML = userInput;\n").unwrap();
        let scan = dir.path().join("scan");
        fs::create_dir_all(&scan).unwrap();
        fs::write(
            scan.join("findings.json"),
            r#"{"documentType":"codex-security.findings","schemaVersion":"1.0","scanId":"s","findings":[{"findingId":"csf_x","ruleId":"xss.innerhtml","title":"xss","locations":[{"path":"x.js","startLine":1}]}]}"#,
        )
        .unwrap();
        let results = generate_all(&src_root, &scan).unwrap();
        assert!(!results.is_empty());
        assert!(scan.join("remediation").join("index.json").is_file());
    }
}
