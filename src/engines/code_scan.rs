//! Orchestrate algorithmic code scan → ledger → sealed contract.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::contract::{
    CoverageDocument, CoverageSurface, FindingsDocument, ManifestDocument, Producer, ScanBody,
    ScanScope, ScanTarget, SemanticFinding, combine_candidates, ensure_scan_layout, finalize_scan,
    normalize_raw_candidate,
    paths::{CANDIDATE_LEDGER, IN_SCOPE_FILES},
    sha256_path_inventory, snapshot_digest_v1, target_id_from_display, write_candidate_ledger,
    write_scan_bundle,
};
use crate::engines::{EngineHit, MAX_ENGINE_FILE_BYTES, scan_source_file};

#[derive(Debug, Clone)]
pub struct CodeScanResult {
    pub scan_id: String,
    pub files_scanned: usize,
    pub hit_count: usize,
    pub finding_count: usize,
    /// Highest finding severity level present (or "none").
    pub max_severity: String,
    pub report_path: PathBuf,
    pub mode: String,
}

/// Options for algorithmic code / diff scans.
#[derive(Debug, Clone)]
pub struct CodeScanOpts {
    pub scope_prefix: Option<String>,
    /// When set, only these relative paths are scanned (diff mode).
    pub file_list: Option<Vec<String>>,
    pub mode: String,
    pub inventory_strategy: String,
    pub target_kind: String,
    pub base_revision: Option<String>,
    pub head_revision: Option<String>,
    pub summary_prefix: Option<String>,
}

impl Default for CodeScanOpts {
    fn default() -> Self {
        Self {
            scope_prefix: None,
            file_list: None,
            mode: "repository".into(),
            inventory_strategy: "directory".into(),
            target_kind: "directory_snapshot".into(),
            base_revision: None,
            head_revision: None,
            summary_prefix: None,
        }
    }
}

pub fn run_code_scan(
    root: &Path,
    scan_dir: &Path,
    scope_prefix: Option<&str>,
    producer_version: &str,
) -> Result<CodeScanResult> {
    let mut opts = CodeScanOpts::default();
    if let Some(s) = scope_prefix {
        opts.scope_prefix = Some(s.to_string());
        opts.mode = "scoped_path".into();
        opts.inventory_strategy = "scoped_path".into();
    }
    run_code_scan_with_opts(root, scan_dir, opts, producer_version)
}

pub fn run_code_scan_with_opts(
    root: &Path,
    scan_dir: &Path,
    opts: CodeScanOpts,
    producer_version: &str,
) -> Result<CodeScanResult> {
    ensure_scan_layout(scan_dir)?;

    // Resolve SECURITY.md guidance for audit (non-fatal)
    let guidance_scope = opts.scope_prefix.as_deref().unwrap_or(".");
    let _ = crate::engines::security_md::write_security_guidance(root, guidance_scope, scan_dir);

    let files = if let Some(list) = &opts.file_list {
        list.clone()
    } else {
        inventory_source_files(root, opts.scope_prefix.as_deref())?
    };
    {
        let mut f = File::create(scan_dir.join(IN_SCOPE_FILES))?;
        for line in &files {
            writeln!(f, "{line}")?;
        }
    }

    let scope: BTreeSet<String> = files.iter().cloned().collect();
    let mut hits: Vec<EngineHit> = Vec::new();
    let mut file_contents: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for rel in &files {
        let path = root.join(rel);
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() > MAX_ENGINE_FILE_BYTES {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue, // binary / encoding
        };
        hits.extend(scan_source_file(rel, &content));
        file_contents.insert(rel.replace('\\', "/"), content);
    }

    // Dependency confusion (live registry checks; skip with WA_DEPCHECK_SKIP_NETWORK=1)
    hits.extend(crate::engines::depcheck_engine::scan_tree_for_confusion(
        root, &files,
    ));

    // Intra-function light taint: upgrade confidence + structured validation
    crate::engines::taint_lite::enrich_hits(&mut hits, &file_contents);

    // Normalize + combine as ledger
    let mut candidates = Vec::new();
    for hit in &hits {
        let raw = hit.to_raw_candidate();
        if let Ok(c) = normalize_raw_candidate(&raw, &scope) {
            candidates.push(c);
        }
    }
    let combined = combine_candidates(candidates);
    write_candidate_ledger(&scan_dir.join(CANDIDATE_LEDGER), &combined)?;

    // Build findings: one SemanticFinding per engine hit (instance preserved)
    // Re-scan mapping: match hits to combined ledger for candidate_id in extensions
    let mut findings: Vec<SemanticFinding> = Vec::new();
    for hit in &hits {
        let mut f = hit.to_semantic_finding();
        // attach candidate id when identity matches
        let key = identity_key(hit);
        if let Some(c) = combined.iter().find(|c| {
            c.locations
                .first()
                .map(|l| l.path == hit.path && l.start_line == hit.start_line)
                .unwrap_or(false)
                && c.cwe_ids == hit.cwe
        }) {
            f.extensions = serde_json::json!({
                "engine": "algorithmic",
                "candidateId": c.candidate_id,
                "snippet": hit.snippet,
                "identityKey": key,
            });
        }
        findings.push(f);
    }

    let scan_id = format!(
        "wa_{}",
        &hex::encode(Sha256::digest(
            format!("{}:{}", root.display(), UtcLike::now()).as_bytes()
        ))[..12]
    );
    // Prefer uuid if available — use simple hash of path + count for stability in tests when WA_SCAN_ID set
    let scan_id = std::env::var("WA_SCAN_ID").unwrap_or(scan_id);

    let display = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string());
    let include = if let Some(s) = &opts.scope_prefix {
        vec![s.clone()]
    } else if opts.file_list.is_some() {
        files.clone()
    } else {
        vec![".".into()]
    };

    let inventory = sha256_path_inventory(root).unwrap_or_else(|_| "0".repeat(64));
    let snap = snapshot_digest_v1(&inventory);

    let surfaces = engine_surfaces(&findings);
    let prefix = opts
        .summary_prefix
        .clone()
        .unwrap_or_else(|| "Algorithmic code scan".into());

    let manifest = ManifestDocument {
        document_type: "codex-security.scan-manifest".into(),
        schema_version: "1.0".into(),
        scan: ScanBody {
            id: scan_id.clone(),
            producer: Producer {
                name: "weeping-angel".into(),
                version: producer_version.into(),
            },
            status: "completed".into(),
            started_at: String::new(),
            completed_at: String::new(),
            sealed_at: String::new(),
            target: ScanTarget {
                kind: opts.target_kind.clone(),
                target_id: target_id_from_display(&display),
                display_name: display,
                remote: None,
                revision: opts.head_revision.clone(),
                base_revision: opts.base_revision.clone(),
                head_revision: opts.head_revision.clone(),
                snapshot_digest: Some(snap),
            },
            scope: ScanScope {
                include_paths: include.clone(),
                exclude_paths: vec![],
                summary: Some(format!(
                    "{prefix}: {} files, {} engine hits, {} reportable findings.",
                    files.len(),
                    hits.len(),
                    findings.len()
                )),
                artifacts_reviewed: Some(vec![IN_SCOPE_FILES.into(), CANDIDATE_LEDGER.into()]),
                runtime_status: Some("not-run".into()),
                validation_mode: Some("static-pattern".into()),
                context: None,
                limitations: Some(vec![
                    "Pattern/static engines only; no AI semantic review.".into(),
                    "No full interprocedural taint; confidence often medium.".into(),
                    "DepCheck registry checks in scan-code are opt-in via WA_DEPCHECK_NETWORK=1."
                        .into(),
                ]),
            },
            threat_model: None,
            hardening: None,
            coverage_ref: "coverage.json".into(),
            findings_ref: "findings.json".into(),
            artifacts: vec![],
        },
    };

    let findings_doc = FindingsDocument {
        document_type: "codex-security.findings".into(),
        schema_version: "1.0".into(),
        scan_id: scan_id.clone(),
        findings,
    };

    let coverage = CoverageDocument {
        document_type: "codex-security.coverage".into(),
        schema_version: "1.0".into(),
        scan_id: scan_id.clone(),
        mode: opts.mode.clone(),
        completeness: "complete".into(),
        inventory_strategy: opts.inventory_strategy.clone(),
        include_paths: include,
        exclude_paths: vec![],
        surfaces,
        explicit_exclusions: vec![],
        deferred: vec![],
        open_questions: vec![],
    };

    let finding_count = findings_doc.findings.len();
    let max_severity = findings_doc
        .findings
        .iter()
        .map(|f| f.severity.level.as_str())
        .max_by_key(|l| crate::engines::severity_rank(l))
        .unwrap_or("none")
        .to_string();

    write_scan_bundle(scan_dir, &manifest, &findings_doc, &coverage)?;
    let report_path = finalize_scan(scan_dir, producer_version)?;

    // SARIF from sealed findings (fingerprints assigned by finalize)
    if let (Ok(fraw), Ok(mraw)) = (
        fs::read_to_string(scan_dir.join("findings.json")),
        fs::read_to_string(scan_dir.join("scan-manifest.json")),
    ) {
        if let (Ok(sealed_f), Ok(sealed_m)) = (
            serde_json::from_str::<FindingsDocument>(&fraw),
            serde_json::from_str::<ManifestDocument>(&mraw),
        ) {
            if let Ok(sarif) =
                crate::contract::sarif::findings_to_sarif(&sealed_f, &sealed_m, producer_version)
            {
                let _ = fs::write(scan_dir.join("findings.sarif.json"), sarif);
            }
        }
    }

    Ok(CodeScanResult {
        finding_count,
        max_severity,
        hit_count: hits.len(),
        files_scanned: files.len(),
        scan_id,
        report_path,
        mode: opts.mode,
    })
}

fn engine_surfaces(findings: &[SemanticFinding]) -> Vec<CoverageSurface> {
    let packs = [
        (
            "surface_path_traversal",
            "Path traversal engines",
            "path-traversal",
        ),
        (
            "surface_command_injection",
            "Command injection engines",
            "command-injection",
        ),
        ("surface_secrets", "Secrets-in-code engines", "secrets"),
        (
            "surface_sql_injection",
            "SQL injection engines",
            "sql-injection",
        ),
        ("surface_ssrf", "SSRF engines", "ssrf"),
        ("surface_xss", "XSS / template engines", "xss"),
        (
            "surface_authz",
            "Authorization / route guard engines",
            "authorization-bypass",
        ),
    ];
    packs
        .iter()
        .map(|(id, label, cat)| {
            let reported = findings.iter().any(|f| f.taxonomy.category == *cat);
            CoverageSurface {
                id: (*id).into(),
                label: (*label).into(),
                disposition: if reported {
                    "reported".into()
                } else {
                    "no_issue_found".into()
                },
                receipt_refs: vec![],
                risk_area: Some((*cat).into()),
                notes: None,
            }
        })
        .collect()
}

fn identity_key(hit: &EngineHit) -> String {
    format!(
        "{}|{}|{}|{}",
        hit.rule_id,
        hit.path,
        hit.start_line,
        hit.instance.as_deref().unwrap_or("")
    )
}

pub fn inventory_source_files(root: &Path, scope_prefix: Option<&str>) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        if rel.starts_with(".git/") || rel.contains("/.git/") {
            continue;
        }
        if rel.contains("/node_modules/")
            || rel.contains("/target/")
            || rel.contains("/.venv/")
            || rel.contains("/dist/")
            || rel.contains("/build/")
        {
            continue;
        }
        if let Some(prefix) = scope_prefix {
            if !(rel == *prefix || rel.starts_with(&format!("{prefix}/"))) {
                continue;
            }
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let fname = entry
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let is_dep_manifest = matches!(
            fname.as_str(),
            "package.json"
                | "package-lock.json"
                | "npm-shrinkwrap.json"
                | "yarn.lock"
                | "pnpm-lock.yaml"
                | "requirements.txt"
                | "pipfile"
                | "pipfile.lock"
                | "pyproject.toml"
                | "composer.json"
                | "composer.lock"
                | "gemfile"
                | "gemfile.lock"
                | "pom.xml"
                | "build.gradle"
                | "build.gradle.kts"
                | "go.mod"
                | "go.sum"
                | "cargo.toml"
                | "cargo.lock"
                | "packages.config"
        ) || fname.ends_with(".csproj");
        let ok = is_dep_manifest
            || matches!(
                ext.as_str(),
                "rs" | "py"
                    | "js"
                    | "jsx"
                    | "ts"
                    | "tsx"
                    | "go"
                    | "java"
                    | "kt"
                    | "rb"
                    | "php"
                    | "c"
                    | "cc"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "cs"
                    | "swift"
                    | "scala"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "json"
                    | "sql"
                    | "env"
            );
        if ok {
            files.push(rel);
        }
    }
    files.sort();
    Ok(files)
}

/// Tiny helper so we don't depend on chrono for scan id entropy here.
struct UtcLike;
impl UtcLike {
    fn now() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis().to_string())
            .unwrap_or_else(|_| "0".into())
    }
}
