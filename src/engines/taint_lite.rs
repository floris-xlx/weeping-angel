//! Intra-function light taint: track variables from common source patterns to sink lines.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;

use crate::engines::EngineHit;

static SOURCE_EXPR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)(req\.|request\.|params\.|query\.|body\.|headers\.|cookies\.|argv|sys\.argv|process\.argv|os\.environ|process\.env|urlsearchparams|location\.search|document\.cookie|\$_(GET|POST|REQUEST|COOKIE))"#,
    )
    .unwrap()
});

static ASSIGN: Lazy<Regex> = Lazy::new(|| {
    // name = expr  |  let/const/var name = expr  |  name := expr
    Regex::new(
        r#"(?i)^\s*(?:(?:let|const|var|auto)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::=\s*|=)\s*(.+)$"#,
    )
    .unwrap()
});

static IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaintVerdict {
    /// Source variable reaches the sink line in the same function window.
    Reaches,
    /// Sink matched but no source/taint chain found in the window.
    NoSourceInWindow,
    /// Secrets/static patterns don't need flow.
    NotApplicable,
}

#[derive(Debug, Clone)]
pub struct TaintResult {
    pub verdict: TaintVerdict,
    pub sources: Vec<String>,
    pub tainted_names: Vec<String>,
    pub method: String,
    pub confidence: &'static str,
    pub confidence_rationale: String,
    pub disposition: &'static str,
}

/// Enrich a hit with light taint analysis over the surrounding function window.
pub fn enrich_hit(hit: &mut EngineHit, file_content: &str) {
    // Secrets and pure static credentials: N/A for taint
    if hit.category == "secrets" {
        let r = TaintResult {
            verdict: TaintVerdict::NotApplicable,
            sources: vec![],
            tainted_names: vec![],
            method: "static-secret-pattern".into(),
            confidence: hit.confidence,
            confidence_rationale: hit.confidence_rationale.clone(),
            disposition: "reportable",
        };
        apply_validation(hit, &r);
        return;
    }

    let r = analyze_line_window(file_content, hit.start_line);
    apply_validation(hit, &r);

    // Adjust confidence on the hit itself for severity/fail_on consumers
    hit.confidence = r.confidence;
    hit.confidence_rationale = r.confidence_rationale.clone();
}

fn apply_validation(hit: &mut EngineHit, r: &TaintResult) {
    // Re-build validation/attack_path JSON into extensions path used at semantic conversion time
    // by storing in confidence fields; full JSON set in to_semantic after enrich.
    hit.evidence = format!(
        "{}; taint={:?}; sources={:?}; names={:?}",
        hit.evidence, r.verdict, r.sources, r.tainted_names
    );
}

/// Analyze whether tainted data can reach `sink_line` (1-based).
/// Uses local function window first, then same-file callees that accept tainted args.
pub fn analyze_line_window(content: &str, sink_line: u32) -> TaintResult {
    let lines: Vec<&str> = content.lines().collect();
    if sink_line == 0 || sink_line as usize > lines.len() {
        return TaintResult {
            verdict: TaintVerdict::NoSourceInWindow,
            sources: vec![],
            tainted_names: vec![],
            method: "static-taint-lite".into(),
            confidence: "low",
            confidence_rationale: "Sink line out of range.".into(),
            disposition: "deferred",
        };
    }

    let sink_idx = (sink_line as usize) - 1;

    // Pass 1: local function window
    let local = analyze_window_range(&lines, sink_idx, true);
    if local.verdict == TaintVerdict::Reaches {
        return local;
    }

    // Pass 2: same-file interprocedural — mark functions that return/use sources,
    // then if sink function calls them with any arg, upgrade confidence.
    let inter = analyze_interprocedural_file(&lines, sink_idx);
    if inter.verdict == TaintVerdict::Reaches {
        return inter;
    }

    // Prefer local detail if it had sources
    if !local.sources.is_empty() {
        local
    } else {
        inter
    }
}

fn analyze_window_range(lines: &[&str], sink_idx: usize, local_only: bool) -> TaintResult {
    let (start, end) = function_window(lines, sink_idx);
    let window = &lines[start..=end];

    let mut tainted: HashSet<String> = HashSet::new();
    let mut sources: Vec<String> = Vec::new();

    for line in window.iter() {
        if SOURCE_EXPR.is_match(line) {
            sources.push(line.trim().chars().take(120).collect());
            if let Some(caps) = ASSIGN.captures(line.trim()) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let rhs = caps.get(2).unwrap().as_str();
                if SOURCE_EXPR.is_match(rhs) || idents_intersect(rhs, &tainted) {
                    tainted.insert(name);
                }
            }
            tainted.insert("__source_expr__".into());
        } else if let Some(caps) = ASSIGN.captures(line.trim()) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let rhs = caps.get(2).unwrap().as_str();
            if idents_intersect(rhs, &tainted) || SOURCE_EXPR.is_match(rhs) {
                tainted.insert(name);
            }
        }
    }

    let sink = lines[sink_idx];
    let reaches = SOURCE_EXPR.is_match(sink)
        || idents_intersect(sink, &tainted)
        || (!tainted.is_empty() && sink_uses_any(sink, &tainted));

    let method = if local_only {
        "static-taint-lite"
    } else {
        "static-taint-file"
    };

    if reaches {
        TaintResult {
            verdict: TaintVerdict::Reaches,
            sources: sources.into_iter().take(5).collect(),
            tainted_names: tainted
                .into_iter()
                .filter(|n| n != "__source_expr__")
                .take(12)
                .collect(),
            method: method.into(),
            confidence: "high",
            confidence_rationale:
                "Attacker-source pattern and/or tainted identifiers reach the sink line."
                    .into(),
            disposition: "reportable",
        }
    } else if !sources.is_empty() {
        TaintResult {
            verdict: TaintVerdict::NoSourceInWindow,
            sources: sources.into_iter().take(5).collect(),
            tainted_names: tainted.into_iter().filter(|n| n != "__source_expr__").collect(),
            method: method.into(),
            confidence: "low",
            confidence_rationale:
                "Sources exist nearby but no clear identifier flow to the sink line.".into(),
            disposition: "reportable",
        }
    } else {
        TaintResult {
            verdict: TaintVerdict::NoSourceInWindow,
            sources: vec![],
            tainted_names: vec![],
            method: method.into(),
            confidence: "low",
            confidence_rationale: "No attacker-source pattern in the analysis window.".into(),
            disposition: "reportable",
        }
    }
}

/// Same-file: if sink function calls a helper that reads sources / takes params used at sink.
fn analyze_interprocedural_file(lines: &[&str], sink_idx: usize) -> TaintResult {
    // Collect function names that contain SOURCE_EXPR
    let funcs = split_functions(lines);
    let mut source_funcs: HashSet<String> = HashSet::new();
    for (name, range) in &funcs {
        for line in &lines[range.0..=range.1] {
            if SOURCE_EXPR.is_match(line) {
                source_funcs.insert(name.clone());
                break;
            }
        }
    }

    let sink_fn = enclosing_function_name(lines, sink_idx);
    let (start, end) = function_window(lines, sink_idx);
    let mut sources = Vec::new();
    let mut tainted_names = Vec::new();
    let mut reaches = false;

    // Calls inside sink function to source_funcs
    static CALL: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());

    for line in &lines[start..=end] {
        for caps in CALL.captures_iter(line) {
            let callee = caps.get(1).unwrap().as_str();
            if source_funcs.contains(callee) {
                reaches = true;
                sources.push(format!("call→{callee} (function contains source pattern)"));
                tainted_names.push(callee.to_string());
            }
        }
        // Direct source still
        if SOURCE_EXPR.is_match(line) {
            sources.push(line.trim().chars().take(120).collect());
            reaches = true;
        }
    }

    // If sink is inside a source function (param-based): params treated as tainted
    if let Some(name) = &sink_fn {
        if let Some(range) = funcs.get(name) {
            let header = lines[range.0];
            if header.contains('(') {
                // mark param names
                if let Some(params) = header.split_once('(').and_then(|(_, r)| r.split_once(')')) {
                    for p in params.0.split(',') {
                        let p = p
                            .trim()
                            .split(':')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .split('=')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if !p.is_empty() && p != "self" && p != "cls" {
                            tainted_names.push(p.to_string());
                        }
                    }
                }
            }
            // re-run local with synthetic sources if params used with sink and any caller exists
            let mut tainted: HashSet<String> = tainted_names.iter().cloned().collect();
            let sink = lines[sink_idx];
            if idents_intersect(sink, &tainted) {
                // only upgrade if something in file calls this function with source-ish args
                let mut called_with_source = false;
                let call_re = Regex::new(&format!(r"\b{}\s*\(", regex::escape(name))).unwrap();
                for (i, line) in lines.iter().enumerate() {
                    if i >= range.0 && i <= range.1 {
                        continue;
                    }
                    if call_re.is_match(line) && SOURCE_EXPR.is_match(line) {
                        called_with_source = true;
                        sources.push(line.trim().chars().take(120).collect());
                        break;
                    }
                    // also: x = source; foo(x)
                    if call_re.is_match(line) {
                        // check previous 5 lines for source assign
                        let from = i.saturating_sub(5);
                        for prev in &lines[from..i] {
                            if SOURCE_EXPR.is_match(prev) {
                                called_with_source = true;
                                sources.push(prev.trim().chars().take(120).collect());
                                break;
                            }
                        }
                    }
                }
                if called_with_source {
                    reaches = true;
                    let _ = &mut tainted;
                }
            }
        }
    }

    if reaches {
        TaintResult {
            verdict: TaintVerdict::Reaches,
            sources: sources.into_iter().take(5).collect(),
            tainted_names: tainted_names.into_iter().take(12).collect(),
            method: "static-taint-file".into(),
            confidence: "high",
            confidence_rationale:
                "Same-file interprocedural: sink reached via helper that reads attacker sources and/or call with source args."
                    .into(),
            disposition: "reportable",
        }
    } else {
        TaintResult {
            verdict: TaintVerdict::NoSourceInWindow,
            sources,
            tainted_names,
            method: "static-taint-file".into(),
            confidence: "low",
            confidence_rationale: "No same-file call-chain from source-bearing functions.".into(),
            disposition: "reportable",
        }
    }
}

fn split_functions(lines: &[&str]) -> HashMap<String, (usize, usize)> {
    let mut map = HashMap::new();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        let name = if let Some(rest) = t.strip_prefix("def ") {
            rest.split('(').next().map(|s| s.trim().to_string())
        } else if let Some(rest) = t.strip_prefix("function ") {
            rest.split('(').next().map(|s| s.trim().to_string())
        } else if let Some(rest) = t.strip_prefix("async function ") {
            rest.split('(').next().map(|s| s.trim().to_string())
        } else if t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("async fn ") {
            t.split_whitespace()
                .nth(if t.starts_with("pub") || t.starts_with("async") {
                    2
                } else {
                    1
                })
                .and_then(|s| s.split('(').next())
                .map(|s| s.to_string())
        } else {
            None
        };
        if let Some(name) = name {
            if !name.is_empty() {
                let start = i;
                let mut end = lines.len().saturating_sub(1);
                for j in (i + 1)..lines.len() {
                    let u = lines[j].trim();
                    if u.starts_with("def ")
                        || u.starts_with("function ")
                        || u.starts_with("fn ")
                        || u.starts_with("pub fn ")
                        || u.starts_with("async fn ")
                        || u.starts_with("async function ")
                    {
                        end = j.saturating_sub(1);
                        break;
                    }
                }
                map.insert(name, (start, end));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    map
}

fn enclosing_function_name(lines: &[&str], sink_idx: usize) -> Option<String> {
    let funcs = split_functions(lines);
    funcs
        .into_iter()
        .find(|(_, r)| sink_idx >= r.0 && sink_idx <= r.1)
        .map(|(n, _)| n)
}

fn function_window(lines: &[&str], sink_idx: usize) -> (usize, usize) {
    // Walk up for def/function/{ or blank+indent drop; cap 80 lines back, 40 forward
    let mut start = sink_idx.saturating_sub(80);
    for i in (start..=sink_idx).rev() {
        let t = lines[i].trim();
        if t.starts_with("def ")
            || t.starts_with("fn ")
            || t.starts_with("function ")
            || t.starts_with("async function ")
            || t.starts_with("pub fn ")
            || t.starts_with("async fn ")
            || (t.contains("=>") && t.contains('('))
        {
            start = i;
            break;
        }
    }
    let mut end = (sink_idx + 40).min(lines.len().saturating_sub(1));
    // Prefer not to cross next top-level def
    for i in (sink_idx + 1)..=end {
        let t = lines[i].trim();
        if t.starts_with("def ") || t.starts_with("fn ") || t.starts_with("function ") {
            end = i.saturating_sub(1);
            break;
        }
    }
    (start, end)
}

fn idents_intersect(expr: &str, tainted: &HashSet<String>) -> bool {
    for m in IDENT.find_iter(expr) {
        if tainted.contains(m.as_str()) {
            return true;
        }
    }
    false
}

fn sink_uses_any(sink: &str, tainted: &HashSet<String>) -> bool {
    idents_intersect(sink, tainted)
}

/// Build validation + attack_path JSON for a hit after taint enrichment.
pub fn validation_json(hit: &EngineHit, taint: &TaintResult) -> (serde_json::Value, serde_json::Value) {
    let validation = json!({
        "disposition": taint.disposition,
        "method": taint.method,
        "confidence": taint.confidence,
        "confidence_rationale": taint.confidence_rationale,
        "rubric": [
            "sink pattern match",
            "intra-function source/taint window",
            "identifier flow heuristic"
        ],
        "evidence": hit.evidence,
        "source": taint.sources,
        "tainted_names": taint.tainted_names,
        "taint_verdict": format!("{:?}", taint.verdict),
        "counterevidence_or_proof_gap": match taint.verdict {
            TaintVerdict::Reaches => "No dynamic PoC; interprocedural flow not proven.",
            TaintVerdict::NoSourceInWindow => "Missing clear source-to-sink identifier chain in window.",
            TaintVerdict::NotApplicable => "Flow analysis not required for this rule family.",
        },
        "remaining_uncertainty": "Sanitizers, framework middleware, and cross-function flow are not modeled.",
    });

    let attack_path = json!({
        "decision": if taint.disposition == "reportable" { "reportable" } else { "deferred" },
        "dataflow": {
            "source": taint.sources.first().cloned().unwrap_or_else(|| "unknown-or-static".into()),
            "transformations": taint.tainted_names,
            "sink": hit.rule_id,
            "narrative": hit.summary,
        },
        "reachability": {
            "attacker": "depends on product surface and entrypoint",
            "preconditions": "code path reachable with attacker-controlled input",
            "narrative": taint.confidence_rationale,
        },
        "severity": hit.severity,
        "severity_rationale": "Rule default severity adjusted only by confidence, not impact matrix.",
        "impact": if hit.severity == "critical" || hit.severity == "high" { "high" } else { "medium" },
        "likelihood": match taint.verdict {
            TaintVerdict::Reaches => "medium",
            TaintVerdict::NotApplicable => "medium",
            TaintVerdict::NoSourceInWindow => "low",
        },
    });

    (validation, attack_path)
}

/// Run taint enrichment for all hits using per-file content cache.
pub fn enrich_hits(hits: &mut [EngineHit], file_contents: &HashMap<String, String>) {
    for hit in hits.iter_mut() {
        if let Some(content) = file_contents.get(&hit.path) {
            let taint = if hit.category == "secrets" {
                TaintResult {
                    verdict: TaintVerdict::NotApplicable,
                    sources: vec![],
                    tainted_names: vec![],
                    method: "static-secret-pattern".into(),
                    confidence: "high",
                    confidence_rationale: "High-signal secret format in source.".into(),
                    disposition: "reportable",
                }
            } else {
                analyze_line_window(content, hit.start_line)
            };
            // store for later semantic conversion via confidence fields + evidence
            hit.confidence = taint.confidence;
            hit.confidence_rationale = taint.confidence_rationale.clone();
            let (v, a) = validation_json(hit, &taint);
            // piggy-back in snippet? better: use evidence append + extensions at convert
            hit.evidence = format!(
                "{} | taint={:?} conf={}",
                hit.evidence, taint.verdict, taint.confidence
            );
            // stash JSON in extensions-like way: encode in summary? 
            // Use a side channel via EngineHit — add optional fields
            hit.validation_json = Some(v);
            hit.attack_path_json = Some(a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taint_reaches_sql_sink() {
        let src = r#"
def q(req):
    uid = req.args["id"]
    cur.execute(f"SELECT * FROM users WHERE id={uid}")
"#;
        let r = analyze_line_window(src, 4);
        assert_eq!(r.verdict, TaintVerdict::Reaches);
        assert_eq!(r.confidence, "high");
    }

    #[test]
    fn taint_misses_without_source() {
        let src = r#"
def q():
    uid = "1"
    cur.execute(f"SELECT * FROM users WHERE id={uid}")
"#;
        let r = analyze_line_window(src, 4);
        assert_eq!(r.verdict, TaintVerdict::NoSourceInWindow);
    }

    #[test]
    fn same_file_interprocedural_via_source_helper() {
        // Sink function only calls helper; source lives in get_uid
        let src = r#"
def get_uid(req):
    return req.args["id"]

def q(req):
    uid = get_uid(req)
    cur.execute(f"SELECT * FROM users WHERE id={uid}")
"#;
        // sink is line 7 (cur.execute)
        let r = analyze_line_window(src, 7);
        assert_eq!(r.verdict, TaintVerdict::Reaches, "{r:?}");
        assert_eq!(r.method, "static-taint-file");
        assert_eq!(r.confidence, "high");
    }

    #[test]
    fn same_file_interprocedural_call_with_source_args() {
        let src = r#"
def run_sql(uid):
    cur.execute(f"SELECT * FROM t WHERE id={uid}")

def handler(req):
    run_sql(req.args["id"])
"#;
        // sink inside run_sql at line 3
        let r = analyze_line_window(src, 3);
        assert_eq!(r.verdict, TaintVerdict::Reaches, "{r:?}");
    }
}
