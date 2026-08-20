//! SQL injection sink patterns (string-built queries).

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
            id: "sql-injection.format-fstring",
            anchor: "sql-query-fstring-or-format",
            title: "SQL query built with string formatting",
            severity: "high",
            re: Regex::new(
                r#"(?i)(execute|executemany|raw|query|cursor\.execute)\s*\(\s*(f["']|["'].*%[sdf]|["'].*\.format\(|["'].*\+|`[^`]*\$\{)"#,
            )
            .unwrap(),
            remediation: "Use parameterized queries / bound placeholders; never interpolate untrusted input into SQL text.",
        },
        Rule {
            id: "sql-injection.concat-select",
            anchor: "sql-string-concat-select",
            title: "SQL SELECT/INSERT string concatenation",
            severity: "high",
            re: Regex::new(
                r#"(?i)(["'`])\s*(SELECT|INSERT|UPDATE|DELETE|WHERE)\b[^"'`]*(['"`]\s*\+|f["']|\$\{)"#,
            )
            .unwrap(),
            remediation: "Replace concatenation with prepared statements and bound parameters.",
        },
        Rule {
            id: "sql-injection.raw-query",
            anchor: "orm-raw-sql",
            title: "ORM raw SQL with dynamic fragments",
            severity: "high",
            re: Regex::new(
                r#"(?i)\.(raw|Raw|executeRaw|queryRaw|fromRaw|whereRaw)\s*\([^)]*(req\.|request\.|params\.|query\.|argv|user|input|body)"#,
            )
            .unwrap(),
            remediation: "Avoid raw SQL with user input; use the query builder with parameters.",
        },
        Rule {
            id: "sql-injection.php-mysql",
            anchor: "php-mysql-query-concat",
            title: "PHP mysqli/mysql query with concatenated input",
            severity: "critical",
            re: Regex::new(
                r#"(?i)(mysqli_query|mysql_query|\$\w+->query)\s*\(\s*[^,)]*\$_(GET|POST|REQUEST|COOKIE)"#,
            )
            .unwrap(),
            remediation: "Use prepared statements (mysqli_prepare / PDO) with bound parameters.",
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
                hits.push(hit(rule, rel_path, line_no, line, m.start(), m.end()));
            }
        }
    }
    hits
}

fn hit(
    rule: &Rule,
    rel_path: &str,
    line_no: u32,
    line: &str,
    start: usize,
    end: usize,
) -> EngineHit {
    EngineHit {
        rule_id: rule.id.into(),
        anchor: rule.anchor.into(),
        instance: Some(format!("{}-l{}", slug_path(rel_path), line_no)),
        title: rule.title.into(),
        summary: format!("{} at `{}:{}`.", rule.title, rel_path, line_no),
        evidence: format!(
            "Matched `{}` on line {line_no}: `{}`",
            rule.id,
            &line[start..end.min(line.len())]
        ),
        severity: rule.severity,
        confidence: "medium",
        confidence_rationale:
            "SQL sink/construction pattern with dynamic content; parameterization not proven."
                .into(),
        category: "sql-injection".into(),
        cwe: vec!["CWE-89".into()],
        remediation: rule.remediation.into(),
        path: rel_path.replace('\\', "/"),
        start_line: line_no,
        end_line: Some(line_no),
        role: "sink",
        snippet: line.trim().chars().take(240).collect(),
        validation_json: None,
        attack_path_json: None,
    }
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
    fn detects_fstring_sql() {
        let src = "cur.execute(f\"SELECT * FROM users WHERE id={uid}\")\n";
        let hits = scan("db.py", src);
        assert!(!hits.is_empty());
    }
}
