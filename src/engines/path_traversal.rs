//! Path traversal / unsafe path join and archive extract patterns.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::engines::EngineHit;

struct Rule {
    id: &'static str,
    anchor: &'static str,
    title: &'static str,
    severity: &'static str,
    re: Regex,
    remediation: &'static str,
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        Rule {
            id: "path-traversal.os-path-join-user",
            anchor: "os-path-join-with-user-path",
            title: "Path join may allow traversal from user-controlled input",
            severity: "high",
            re: Regex::new(
                r#"(?i)(os\.path\.join|path\.join|Paths\.get|filepath\.Join|Path\.Combine)\s*\([^)]*(req\.|request\.|params\.|query\.|body\.|argv|sys\.argv|user|filename|filepath|file_path|path_param)"#,
            )
            .unwrap(),
            remediation:
                "Resolve and canonicalize the destination, then reject paths that escape the allowed root.",
        },
        Rule {
            id: "path-traversal.archive-extraction",
            anchor: "archive-extract-without-containment",
            title: "Archive extraction may write outside the target directory",
            severity: "high",
            re: Regex::new(
                r#"(?i)(zipfile\.ZipFile|tarfile\.open|ZipInputStream|zip\.OpenReader|archive/zip).{0,200}(extract|extractall|ExtractTo|unzip)"#,
            )
            .unwrap(),
            remediation:
                "Validate each archive member name; join under a fixed root and reject `..` / absolute paths before write.",
        },
        Rule {
            id: "path-traversal.open-user-path",
            anchor: "open-user-supplied-path",
            title: "File open uses potentially attacker-controlled path",
            severity: "high",
            re: Regex::new(
                r#"(?i)(open|fs\.readFile|fs\.readFileSync|File::open|std::fs::read|ioutil\.ReadFile|os\.ReadFile)\s*\(\s*(req\.|request\.|params\.|query\.|argv|sys\.argv|filename|filepath|file_path)"#,
            )
            .unwrap(),
            remediation:
                "Do not open user-supplied paths directly; map to allowlisted IDs or enforce a strict root prefix after normalization.",
        },
        Rule {
            id: "path-traversal.send-file",
            anchor: "static-send-file-user-path",
            title: "Static file send may expose traversal",
            severity: "high",
            re: Regex::new(
                r#"(?i)(send_file|sendFile|res\.sendFile|StaticFiles|FileResponse)\s*\([^)]*(req\.|request\.|params\.|query\.|path)"#,
            )
            .unwrap(),
            remediation:
                "Serve files only from a fixed directory after resolving and verifying the path stays inside it.",
        },
    ]
});

pub fn scan(rel_path: &str, content: &str) -> Vec<EngineHit> {
    let mut hits = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let line_no = (line_no + 1) as u32;
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with('*') {
            continue;
        }
        for rule in RULES.iter() {
            if let Some(m) = rule.re.find(line) {
                let snippet = line.trim().chars().take(240).collect::<String>();
                hits.push(EngineHit {
                    rule_id: rule.id.into(),
                    anchor: rule.anchor.into(),
                    instance: Some(format!("{}-l{}", slug_path(rel_path), line_no)),
                    title: rule.title.into(),
                    summary: format!(
                        "{} at `{}:{}` — pattern suggests a path/file operation on attacker-influenced input.",
                        rule.title, rel_path, line_no
                    ),
                    evidence: format!(
                        "Matched `{}` on line {line_no}: `{}`",
                        rule.id,
                        &line[m.start()..m.end().min(line.len())]
                    ),
                    severity: rule.severity,
                    confidence: "medium",
                    confidence_rationale:
                        "Static sink/path pattern with user-input-like identifiers nearby; no full taint proof."
                            .into(),
                    category: "path-traversal".into(),
                    cwe: vec!["CWE-22".into()],
                    remediation: rule.remediation.into(),
                    path: rel_path.replace('\\', "/"),
                    start_line: line_no,
                    end_line: Some(line_no),
                    role: "sink",
                    snippet,
                    validation_json: None,
                    attack_path_json: None,
                });
            }
        }
    }
    hits
}

fn slug_path(path: &str) -> String {
    path.replace('\\', "/")
        .replace(['/', '.', ' '], "-")
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_os_path_join() {
        let src = "def load(req):\n    p = os.path.join(BASE, req.args['file'])\n    return open(p).read()\n";
        let hits = scan("app.py", src);
        assert!(hits.iter().any(|h| h.rule_id.contains("path-join") || h.rule_id.contains("open-user")));
    }
}
