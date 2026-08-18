//! Algorithmic code engines + scan-code sealing.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use weeping_angel::contract::{FindingsDocument, paths};
use weeping_angel::engines::code_scan::run_code_scan;
use weeping_angel::engines::{
    authz_routes, cmd_injection, findings_meet_fail_on, path_traversal, secrets_code,
    sql_injection, ssrf, xss_template,
};

#[test]
fn engines_detect_toy_patterns() {
    let src = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/code-toy/src/app.py"),
    )
    .unwrap();
    let path_hits = path_traversal::scan("src/app.py", &src);
    let cmd_hits = cmd_injection::scan("src/app.py", &src);
    let sec_hits = secrets_code::scan("src/app.py", &src);
    let sql_hits = sql_injection::scan("src/app.py", &src);
    let ssrf_hits = ssrf::scan("src/app.py", &src);
    let xss_hits = xss_template::scan("src/app.py", &src);
    assert!(
        !path_hits.is_empty(),
        "expected path traversal hits, got none"
    );
    assert!(!cmd_hits.is_empty(), "expected cmd injection hits");
    assert!(!sec_hits.is_empty(), "expected secrets hits");
    assert!(!sql_hits.is_empty(), "expected sql injection hits");
    assert!(!ssrf_hits.is_empty(), "expected ssrf hits");
    assert!(!xss_hits.is_empty(), "expected xss hits");
}

#[test]
fn scan_code_seals_findings_from_toy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/code-toy");
    let dir = tempdir().unwrap();
    let scan_dir = dir.path().join("out");

    let result = run_code_scan(&root, &scan_dir, None, "0.1.2").unwrap();
    assert!(result.finding_count >= 2, "expected multiple findings");
    assert!(result.report_path.exists());

    let findings: FindingsDocument =
        serde_json::from_str(&fs::read_to_string(scan_dir.join(paths::FINDINGS_FILE)).unwrap())
            .unwrap();
    assert!(!findings.findings.is_empty());
    assert!(
        findings
            .findings
            .iter()
            .all(|f| f.finding_id.starts_with("csf_"))
    );
    assert!(
        findings
            .findings
            .iter()
            .any(|f| f.taxonomy.category == "secrets"
                || f.rule_id.contains("command")
                || f.rule_id.contains("path"))
    );

    let md = fs::read_to_string(scan_dir.join(paths::REPORT_MD)).unwrap();
    assert!(md.contains("## Findings"));

    assert!(scan_dir.join(paths::SECURITY_GUIDANCE_MD).exists());
    assert!(findings_meet_fail_on(&result.max_severity, "low"));
    assert!(!findings_meet_fail_on("none", "medium"));
    assert!(
        scan_dir.join("findings.sarif.json").exists(),
        "expected SARIF export"
    );
    let sarif = fs::read_to_string(scan_dir.join("findings.sarif.json")).unwrap();
    assert!(sarif.contains("\"version\": \"2.1.0\"") || sarif.contains("\"version\":\"2.1.0\""));
    assert!(sarif.contains("runs"));
}

#[test]
fn authz_engine_flags_and_clears() {
    let bad = "app.post('/admin/delete', (req,res)=>res.end())\n";
    assert!(!authz_routes::scan("server.js", bad).is_empty());
    let good = "app.post('/admin/delete', requireAuth, (req,res)=>res.end())\n";
    assert!(authz_routes::scan("server.js", good).is_empty());
}
