//! Command injection / unsafe shell execution patterns.

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
            id: "command-injection.shell-true",
            anchor: "subprocess-shell-true",
            title: "Subprocess invoked with shell=True",
            severity: "high",
            re: Regex::new(r#"(?i)subprocess\.(Popen|call|run|check_output|check_call)\s*\([^)]*shell\s*=\s*True"#)
                .unwrap(),
            remediation: "Pass argument lists with shell=False; never interpolate untrusted input into a shell string.",
        },
        Rule {
            id: "command-injection.os-system",
            anchor: "os-system-or-popen",
            title: "os.system / os.popen executes a shell command",
            severity: "high",
            re: Regex::new(r#"(?i)\b(os\.system|os\.popen|commands\.getoutput|commands\.getstatusoutput)\s*\("#)
                .unwrap(),
            remediation: "Replace with subprocess and an argv list; validate or drop user-controlled segments.",
        },
        Rule {
            id: "command-injection.child-process-exec",
            anchor: "child-process-exec-shell",
            title: "child_process exec/execSync may run a shell",
            severity: "high",
            re: Regex::new(
                r#"(?i)(child_process\.)?(exec|execSync|execFile|execFileSync)\s*\(\s*[`'"][^`'"]*\$\{"#,
            )
            .unwrap(),
            remediation: "Use execFile/spawn with discrete args; avoid shell string interpolation.",
        },
        Rule {
            id: "command-injection.eval-shell",
            anchor: "shell-backtick-or-eval",
            title: "Shell backticks or eval of command string",
            severity: "critical",
            re: Regex::new(r#"(?i)\b(eval|Runtime\.getRuntime\(\)\.exec)\s*\("#).unwrap(),
            remediation: "Remove eval/exec of dynamic strings; use safe APIs for the intended operation.",
        },
        Rule {
            id: "command-injection.rust-command",
            anchor: "std-process-command-arg",
            title: "Process command built from concatenated string",
            severity: "medium",
            re: Regex::new(
                r#"(?i)(Command::new|std::process::Command::new)\s*\([^)]*\)\s*\.args?\s*\(\s*&?format!"#,
            )
            .unwrap(),
            remediation: "Pass discrete OsStr arguments; do not format a full shell line.",
        },
    ]
});

pub fn scan(rel_path: &str, content: &str) -> Vec<EngineHit> {
    let mut hits = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let line_no = (line_no + 1) as u32;
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
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
                        "{} at `{}:{}`.",
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
                        "Dangerous process/shell API pattern; attacker control of arguments not fully proven."
                            .into(),
                    category: "command-injection".into(),
                    cwe: vec!["CWE-78".into()],
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
    fn detects_shell_true() {
        let src = "import subprocess\nsubprocess.run(cmd, shell=True)\n";
        let hits = scan("run.py", src);
        assert!(hits.iter().any(|h| h.rule_id.contains("shell-true")));
    }
}
