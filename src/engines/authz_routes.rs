//! Missing-auth heuristics on mutation/admin routes (Express/Next/Django-ish).

use once_cell::sync::Lazy;
use regex::Regex;

use crate::engines::EngineHit;

/// Route registration that looks like a privileged action without nearby auth middleware names.
static ROUTE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(app|router|api)\.(post|put|patch|delete)\s*\(\s*['"`]([^'"`]+)['"`]"#,
    )
    .unwrap()
});

static DJANGO_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)path\s*\(\s*['"`]([^'"`]*(admin|delete|reset|create|upload)[^'"`]*)['"`]"#)
        .unwrap()
});

static NEXT_ROUTE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)export\s+(async\s+)?function\s+(POST|PUT|PATCH|DELETE)\s*\("#,
    )
    .unwrap()
});

static AUTH_HINT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(requireAuth|requireSession|isAuthenticated|ensureAuth|checkAuth|passport\.authenticate|authMiddleware|verifyToken|getServerSession|require_user|login_required|permission_required|@login_required|authorize\()"#,
    )
    .unwrap()
});

static PRIV_PATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)/(admin|internal|delete|reset|create|upload|manage|billing|api-key)"#)
        .unwrap()
});

pub fn scan(rel_path: &str, content: &str) -> Vec<EngineHit> {
    let lower_path = rel_path.replace('\\', "/").to_ascii_lowercase();
    // Focus on route-ish files
    let routeish = lower_path.contains("route")
        || lower_path.contains("router")
        || lower_path.contains("urls")
        || lower_path.contains("api/")
        || lower_path.ends_with("app.py")
        || lower_path.ends_with("server.ts")
        || lower_path.ends_with("server.js")
        || lower_path.ends_with("main.ts")
        || lower_path.ends_with("main.js")
        || lower_path.contains("handler");

    if !routeish && !content.contains("app.post") && !content.contains("router.") {
        // still scan for Next handlers
        if !NEXT_ROUTE.is_match(content) && !DJANGO_URL.is_match(content) {
            return vec![];
        }
    }

    let mut hits = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_no = (i + 1) as u32;
        let window = window_around(&lines, i, 8);

        if let Some(caps) = ROUTE_RE.captures(line) {
            let path = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            if PRIV_PATH.is_match(path) && !AUTH_HINT.is_match(&window) {
                hits.push(make_hit(
                    "authorization-bypass.express-mutation-no-auth",
                    "express-mutation-without-auth-hint",
                    "Mutation route looks privileged without auth middleware nearby",
                    "medium",
                    rel_path,
                    line_no,
                    line,
                    "Attach auth middleware (e.g. requireAuth) before privileged POST/PUT/PATCH/DELETE handlers.",
                ));
            }
        }

        if let Some(caps) = DJANGO_URL.captures(line) {
            let path = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !AUTH_HINT.is_match(&window) && !line.contains("login_required") {
                hits.push(make_hit(
                    "authorization-bypass.django-url-no-auth",
                    "django-privileged-url-without-auth",
                    &format!("Django path `{path}` may lack auth decorator in nearby code"),
                    "medium",
                    rel_path,
                    line_no,
                    line,
                    "Use login_required / permission_required on views for privileged URL patterns.",
                ));
            }
        }

        if NEXT_ROUTE.is_match(line) {
            // App Router route handlers — check file for session helpers
            if !AUTH_HINT.is_match(content)
                && (lower_path.contains("admin")
                    || lower_path.contains("api/")
                    || PRIV_PATH.is_match(rel_path))
            {
                hits.push(make_hit(
                    "authorization-bypass.next-route-no-session",
                    "next-route-handler-without-session",
                    "Next.js route handler without session/auth helper in file",
                    "medium",
                    rel_path,
                    line_no,
                    line,
                    "Call getServerSession / requireSession (or equivalent) before privileged mutations.",
                ));
            }
        }
    }

    hits
}

fn window_around(lines: &[&str], idx: usize, radius: usize) -> String {
    let start = idx.saturating_sub(radius);
    let end = (idx + radius + 1).min(lines.len());
    lines[start..end].join("\n")
}

fn make_hit(
    rule_id: &str,
    anchor: &str,
    title: &str,
    severity: &'static str,
    rel_path: &str,
    line_no: u32,
    line: &str,
    remediation: &str,
) -> EngineHit {
    EngineHit {
        rule_id: rule_id.into(),
        anchor: anchor.into(),
        instance: Some(format!("{}-l{}", slug_path(rel_path), line_no)),
        title: title.into(),
        summary: format!("{title} at `{rel_path}:{line_no}`."),
        evidence: format!("Route-like line without nearby auth hint: `{}`", line.trim()),
        severity,
        confidence: "low",
        confidence_rationale:
            "Heuristic: privileged path/method without auth identifier in a small source window; may be false positive if auth is applied globally."
                .into(),
        category: "authorization-bypass".into(),
        cwe: vec!["CWE-862".into()],
        remediation: remediation.into(),
        path: rel_path.replace('\\', "/"),
        start_line: line_no,
        end_line: Some(line_no),
        role: "entrypoint",
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
    fn flags_express_admin_post() {
        let src = r#"
const app = express();
app.post('/admin/delete', (req, res) => {
  res.send('ok');
});
"#;
        let hits = scan("server.js", src);
        assert!(!hits.is_empty());
    }

    #[test]
    fn skips_when_auth_nearby() {
        let src = r#"
app.post('/admin/delete', requireAuth, (req, res) => {
  res.send('ok');
});
"#;
        let hits = scan("server.js", src);
        assert!(hits.is_empty());
    }
}
