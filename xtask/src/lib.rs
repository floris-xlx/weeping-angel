//! Repository health gate: `cargo xtask guard`.
//!
//! Phase 1 evaluation plane: `run_guard` loads one [`RepositoryModel`] (Cargo
//! workspace, package graph, filesystem, architecture manifests,
//! `docs/debt/register.toml`, `docs/adr` metadata, `docs/specs` metadata,
//! framework packs, catalog sources) and runs [`ArchitectureCheck::check`].
//! Checks are not independent filesystem greps.
//!
//! Implemented: 01, 02, 03, **04**, 13.
//! Remaining stubs 05–12 and 14–15 skip only with a live `DEBT-GUARD-NN`
//! finding (fail closed otherwise). No silent skips.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const ARCH_SCHEMA: &str = "weeping-angel/architecture/v1";
pub const INVARIANTS_SCHEMA: &str = "weeping-angel/architecture-invariants/v1";
pub const FORBIDDEN_SCHEMA: &str = "weeping-angel/forbidden-patterns/v1";
pub const DEBT_SCHEMA: &str = "weeping-angel/debt-register/v1";

const FORBIDDEN_PACKAGES: [&str; 2] = ["weeping-angel-catalog", "weeping-angel-assurance-cli"];

const ALLOWED_STATUS: [&str; 6] = [
    "open",
    "confirmed",
    "in-progress",
    "resolved",
    "rejected",
    "superseded",
];

const OWNERSHIP_KINDS: [&str; 5] = [
    "exclusive",
    "facade",
    "projection",
    "adapter",
    "shared-primitive",
];

/// Concept → (package name, required path needles).
pub const REQUIRED_OWNERSHIP: [(&str, &str, &[&str]); 7] = [
    (
        "catalog",
        "weeping-angel-canonical-catalog",
        &["crates/weeping-angel-canonical-catalog"],
    ),
    (
        "framework_compilation",
        "weeping-angel-framework",
        &["crates/weeping-angel-framework"],
    ),
    (
        "readiness_projection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/readiness.rs"],
    ),
    (
        "temporal_evidence_selection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/temporal.rs"],
    ),
    (
        "assessment_lineage",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/lineage.rs"],
    ),
    (
        "evidence_persistence",
        "weeping-angel-evidence",
        &["crates/weeping-angel-evidence"],
    ),
    (
        "assurance_cli",
        "weeping-angel",
        &["src/main.rs", "src/cli.rs"],
    ),
];

/// Remaining stubs after Guard 04 became a real ArchitectureCheck.
const REMAINING_STUBS: [(&str, &str); 10] = [
    ("05", "catalog-ssot"),
    ("06", "framework-pack-parse"),
    ("07", "framework-digest"),
    ("08", "readiness-ssot"),
    ("09", "temporal-evidence-selection"),
    ("10", "assessment-lineage-rebuild"),
    ("11", "evidence-latest-vs-current"),
    ("12", "soa-invariants"),
    ("14", "adr-graph"),
    ("15", "spec-lifecycle"),
];

const SKIP_DIR_NAMES: [&str; 5] = ["target", "node_modules", ".git", "__pycache__", "apps"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Skip { debt_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: CheckStatus,
}

impl CheckResult {
    fn pass(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Pass,
        }
    }

    fn fail(id: &str, name: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Fail(message.into()),
        }
    }

    fn skip(id: &str, name: &str, debt_id: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Skip {
                debt_id: debt_id.into(),
            },
        }
    }

    pub fn report_line(&self) -> String {
        match &self.status {
            CheckStatus::Pass => format!("{}  {}  pass", self.id, self.name),
            CheckStatus::Fail(msg) => format!("{}  {}  fail  {msg}", self.id, self.name),
            CheckStatus::Skip { debt_id } => {
                format!("{}  {}  skip({debt_id})", self.id, self.name)
            }
        }
    }

    pub fn is_fail(&self) -> bool {
        matches!(self.status, CheckStatus::Fail(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardViolation {
    pub check_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardSkip {
    pub check_id: String,
    pub debt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardReport {
    pub checks: Vec<CheckResult>,
    pub violations: Vec<GuardViolation>,
    pub skipped: Vec<GuardSkip>,
    pub debt_exemptions: Vec<String>,
    pub duration: Duration,
}

impl GuardReport {
    fn from_checks(checks: Vec<CheckResult>, duration: Duration) -> Self {
        let duration = if duration.is_zero() {
            Duration::from_nanos(1)
        } else {
            duration
        };
        let mut violations = Vec::new();
        let mut skipped = Vec::new();
        let mut debt_exemptions = Vec::new();
        for check in &checks {
            match &check.status {
                CheckStatus::Fail(message) => violations.push(GuardViolation {
                    check_id: check.id.clone(),
                    message: message.clone(),
                }),
                CheckStatus::Skip { debt_id } => {
                    skipped.push(GuardSkip {
                        check_id: check.id.clone(),
                        debt_id: debt_id.clone(),
                    });
                    if !debt_exemptions.iter().any(|id| id == debt_id) {
                        debt_exemptions.push(debt_id.clone());
                    }
                }
                CheckStatus::Pass => {}
            }
        }
        Self {
            checks,
            violations,
            skipped,
            debt_exemptions,
            duration,
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::from("cargo xtask guard\n");
        for check in &self.checks {
            out.push_str(&check.report_line());
            out.push('\n');
        }
        out
    }

    pub fn failed(&self) -> bool {
        self.checks.iter().any(CheckResult::is_fail)
    }

    pub fn to_json(&self) -> String {
        let checks: Vec<serde_json::Value> = self
            .checks
            .iter()
            .map(|c| {
                let status = match &c.status {
                    CheckStatus::Pass => serde_json::json!({"kind": "pass"}),
                    CheckStatus::Fail(msg) => serde_json::json!({"kind": "fail", "message": msg}),
                    CheckStatus::Skip { debt_id } => {
                        serde_json::json!({"kind": "skip", "debt_id": debt_id})
                    }
                };
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "status": status,
                })
            })
            .collect();
        let violations: Vec<serde_json::Value> = self
            .violations
            .iter()
            .map(|v| serde_json::json!({"check_id": v.check_id, "message": v.message}))
            .collect();
        let skipped: Vec<serde_json::Value> = self
            .skipped
            .iter()
            .map(|s| serde_json::json!({"check_id": s.check_id, "debt_id": s.debt_id}))
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "checks": checks,
            "violations": violations,
            "skipped": skipped,
            "debt_exemptions": self.debt_exemptions,
            "duration": {
                "secs": self.duration.as_secs(),
                "nanos": self.duration.subsec_nanos(),
                "as_secs_f64": self.duration.as_secs_f64(),
            },
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

/// Ownership row from `architecture/architecture.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipRow {
    pub crate_name: String,
    pub kind: Option<String>,
    pub paths: Vec<String>,
}

/// Parsed `architecture/architecture.toml` including ownership kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureManifest {
    pub schema: String,
    pub ownership: BTreeMap<String, OwnershipRow>,
}

/// One `[[invariant]]` row from `architecture/invariants.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureInvariant {
    pub id: String,
    pub summary: String,
    pub guard_check: String,
}

/// Per-row outcome of Guard 04 evaluation against [`RepositoryModel`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantResult {
    pub id: String,
    pub summary: String,
    pub guard_check: String,
    pub passed: bool,
    pub evidence: String,
}

/// One `[[pattern]]` row from `architecture/forbidden-patterns.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenPattern {
    pub id: String,
    pub kind: Option<String>,
    pub value: String,
    pub extra: BTreeMap<String, String>,
}

/// Snapshot loaded once per `run_guard`: workspace, package graph, filesystem,
/// architecture manifests, debt register, ADR/spec metadata, framework packs,
/// catalog sources.
#[derive(Debug, Clone)]
pub struct RepositoryModel {
    pub root: PathBuf,
    pub workspace_members: Vec<String>,
    /// package graph: crate name → direct dependency names
    pub package_graph: BTreeMap<String, BTreeSet<String>>,
    pub package_names: BTreeSet<String>,
    pub filesystem: BTreeSet<String>,
    pub architecture: Option<ArchitectureManifest>,
    pub architecture_error: Option<String>,
    pub invariants: Vec<ArchitectureInvariant>,
    pub invariants_error: Option<String>,
    pub forbidden: Vec<ForbiddenPattern>,
    pub forbidden_error: Option<String>,
    pub debt_ids: BTreeSet<String>,
    pub debt_error: Option<String>,
    pub adr_files: Vec<String>,
    pub spec_files: Vec<String>,
    pub framework_packs: Vec<String>,
    pub catalog_sources: Vec<String>,
    pub source_files: Vec<String>,
}

impl RepositoryModel {
    pub fn load(root: &Path) -> Self {
        let root = root.to_path_buf();
        let (workspace_members, package_names, package_graph) = load_workspace(&root);
        let mut filesystem = BTreeSet::new();
        for rel in [
            "architecture",
            "docs/adr",
            "docs/specs",
            "docs/debt",
            "frameworks",
            "catalog",
            "crates",
            "src",
            "tests",
        ] {
            index_tree(&root, &root.join(rel), &mut filesystem);
        }

        let (architecture, architecture_error) = match load_architecture_manifest(&root) {
            Ok(m) => (Some(m), None),
            Err(e) => (None, Some(e)),
        };
        let (invariants, invariants_error) = match load_invariants(&root) {
            Ok(rows) => (rows, None),
            Err(e) => (Vec::new(), Some(e)),
        };
        let (forbidden, forbidden_error) = match load_forbidden_patterns(&root) {
            Ok(rows) => (rows, None),
            Err(e) => (Vec::new(), Some(e)),
        };
        let (debt_ids, debt_error) = match load_and_validate_debt_register(&root) {
            Ok(ids) => (ids, None),
            Err(e) => (BTreeSet::new(), Some(e)),
        };

        let adr_files = list_dir_files(&root.join("docs/adr"), "md");
        let spec_files = list_dir_files(&root.join("docs/specs"), "md");
        let mut framework_packs = Vec::new();
        collect_files(&root, &root.join("frameworks"), &mut framework_packs);
        let mut catalog_sources = Vec::new();
        collect_files(&root, &root.join("catalog"), &mut catalog_sources);
        if filesystem
            .iter()
            .any(|p| p.starts_with("crates/weeping-angel-canonical-catalog"))
        {
            catalog_sources.push("crates/weeping-angel-canonical-catalog".into());
        }
        catalog_sources.sort();
        catalog_sources.dedup();

        let source_files: Vec<String> = filesystem
            .iter()
            .filter(|p| p.ends_with(".rs") && (p.starts_with("src/") || p.starts_with("crates/")))
            .cloned()
            .collect();

        Self {
            root,
            workspace_members,
            package_graph,
            package_names,
            filesystem,
            architecture,
            architecture_error,
            invariants,
            invariants_error,
            forbidden,
            forbidden_error,
            debt_ids,
            debt_error,
            adr_files,
            spec_files,
            framework_packs,
            catalog_sources,
            source_files,
        }
    }

    fn rel_exists(&self, rel: &str) -> bool {
        let trimmed = rel.trim_end_matches(['/', '\\']);
        self.root.join(trimmed).exists()
            || self.filesystem.contains(rel)
            || self.filesystem.contains(trimmed)
    }

    fn source_contains(&self, needle: &str) -> bool {
        for rel in &self.source_files {
            if let Ok(text) = fs::read_to_string(self.root.join(rel)) {
                if text.contains(needle) {
                    return true;
                }
            }
        }
        false
    }
}

/// Shared evaluation plane: every guard check takes the loaded model.
pub trait ArchitectureCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult;
}

struct ArchitectureManifestCheck;
struct CanonicalOwnershipCheck;
struct ForbiddenPatternsCheck;
struct ArchitectureInvariantsCheck;
struct DebtRegisterCheck;
struct StubArchitectureCheck {
    id: &'static str,
    name: &'static str,
}

impl ArchitectureCheck for ArchitectureManifestCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        match &repo.architecture_error {
            Some(err) => CheckResult::fail("01", "architecture-manifest", err.clone()),
            None if repo.architecture.is_some() => CheckResult::pass("01", "architecture-manifest"),
            None => CheckResult::fail(
                "01",
                "architecture-manifest",
                "architecture/architecture.toml is not a file",
            ),
        }
    }
}

impl ArchitectureCheck for CanonicalOwnershipCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        match check_02_on_model(repo) {
            Ok(()) => CheckResult::pass("02", "canonical-ownership"),
            Err(err) => CheckResult::fail("02", "canonical-ownership", err),
        }
    }
}

impl ArchitectureCheck for ForbiddenPatternsCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        match check_03_on_model(repo) {
            Ok(()) => CheckResult::pass("03", "forbidden-patterns"),
            Err(err) => CheckResult::fail("03", "forbidden-patterns", err),
        }
    }
}

impl ArchitectureCheck for ArchitectureInvariantsCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        check_04(repo)
    }
}

impl ArchitectureCheck for DebtRegisterCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        match &repo.debt_error {
            Some(err) => CheckResult::fail("13", "debt-register", err.clone()),
            None => CheckResult::pass("13", "debt-register"),
        }
    }
}

impl ArchitectureCheck for StubArchitectureCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        stub_check(self.id, self.name, &repo.debt_ids)
    }
}

/// Workspace root: parent of the `xtask` crate.
pub fn repo_root_from_xtask_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask crate lives under the workspace root")
        .to_path_buf()
}

pub fn main_with_args<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    match args.first().map(String::as_str) {
        Some("guard") => {
            let mut json = false;
            let mut selected: Option<String> = None;
            let mut explain: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" => json = true,
                    "--check" => {
                        i += 1;
                        match args.get(i) {
                            Some(id) => selected = Some(id.clone()),
                            None => {
                                eprintln!(
                                    "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                                );
                                return 2;
                            }
                        }
                    }
                    "--explain" => {
                        i += 1;
                        match args.get(i) {
                            Some(id) => explain = Some(id.clone()),
                            None => {
                                eprintln!(
                                    "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                                );
                                return 2;
                            }
                        }
                    }
                    other => {
                        eprintln!("unrecognized argument: {other}");
                        eprintln!(
                            "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                        );
                        return 2;
                    }
                }
                i += 1;
            }

            let root = repo_root_from_xtask_manifest();
            if let Some(inv_id) = explain {
                match explain_invariant(&root, &inv_id) {
                    Ok(text) => {
                        print!("{text}");
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                let report = run_guard_with_options(&root, selected.as_deref());
                if json {
                    println!("{}", report.to_json());
                } else {
                    print!("{}", report.render());
                }
                if report.failed() { 1 } else { 0 }
            }
        }
        _ => {
            eprintln!("usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]");
            2
        }
    }
}

pub fn run_guard(root: &Path) -> GuardReport {
    run_guard_with_options(root, None)
}

fn run_guard_with_options(root: &Path, selected: Option<&str>) -> GuardReport {
    let started = Instant::now();
    let repo = RepositoryModel::load(root);
    let mut checks = run_all_checks(&repo);
    if let Some(id) = selected {
        // CLI --check NN runs the selected check (model + debt already loaded).
        checks.retain(|c| c.id == id);
        if checks.is_empty() {
            checks.push(CheckResult::fail(
                id,
                "unknown-check",
                format!("unknown check {id}"),
            ));
        }
    }
    GuardReport::from_checks(checks, started.elapsed())
}

fn run_all_checks(repo: &RepositoryModel) -> Vec<CheckResult> {
    let mut checks = Vec::new();
    checks.push(ArchitectureManifestCheck.check(repo));
    checks.push(CanonicalOwnershipCheck.check(repo));
    checks.push(ForbiddenPatternsCheck.check(repo));
    checks.push(ArchitectureInvariantsCheck.check(repo));
    checks.push(DebtRegisterCheck.check(repo));
    for (id, name) in REMAINING_STUBS {
        checks.push(StubArchitectureCheck { id, name }.check(repo));
    }
    checks
}

fn stub_check(id: &str, name: &str, finding_ids: &BTreeSet<String>) -> CheckResult {
    let debt_id = format!("DEBT-GUARD-{id}");
    if finding_ids.contains(&debt_id) {
        CheckResult::skip(id, name, debt_id)
    } else {
        CheckResult::fail(
            id,
            name,
            format!("not-yet-implemented: check {id} (no registered {debt_id} finding)"),
        )
    }
}

fn read_toml(path: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("parse {}: {e}", path.display()))
}

fn require_schema(value: &toml::Value, expected: &str, path: &Path) -> Result<(), String> {
    let got = value.get("schema").and_then(|s| s.as_str());
    if got == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "{} schema must be {expected}, got {got:?}",
            path.display()
        ))
    }
}

pub fn check_01_architecture_manifest(root: &Path) -> CheckResult {
    ArchitectureManifestCheck.check(&RepositoryModel::load(root))
}

pub fn check_02_ownership(root: &Path) -> CheckResult {
    CanonicalOwnershipCheck.check(&RepositoryModel::load(root))
}

pub fn check_03_forbidden_patterns(root: &Path) -> CheckResult {
    ForbiddenPatternsCheck.check(&RepositoryModel::load(root))
}

/// Guard 04: parse `architecture/invariants.toml` and evaluate every `[[invariant]]`.
pub fn check_04_architecture_invariants(root: &Path) -> CheckResult {
    ArchitectureInvariantsCheck.check(&RepositoryModel::load(root))
}

fn check_04(repo: &RepositoryModel) -> CheckResult {
    match evaluate_all_invariants(repo) {
        Ok(results) => {
            let failed: Vec<&InvariantResult> = results.iter().filter(|r| !r.passed).collect();
            if failed.is_empty() {
                CheckResult::pass("04", "architecture-invariants")
            } else {
                let msg = failed
                    .iter()
                    .map(|r| format!("{}: {}", r.id, r.evidence))
                    .collect::<Vec<_>>()
                    .join("; ");
                CheckResult::fail("04", "architecture-invariants", msg)
            }
        }
        Err(err) => CheckResult::fail("04", "architecture-invariants", err),
    }
}

pub fn explain_invariant(root: &Path, inv_id: &str) -> Result<String, String> {
    let repo = RepositoryModel::load(root);
    let results = evaluate_all_invariants(&repo)?;
    let found = results
        .into_iter()
        .find(|r| r.id == inv_id)
        .ok_or_else(|| format!("unknown invariant {inv_id}"))?;
    Ok(format!(
        "id: {}\nsummary: {}\nguard_check: {}\nresult: {}\nevidence: {}\n",
        found.id,
        found.summary,
        found.guard_check,
        if found.passed { "pass" } else { "fail" },
        found.evidence
    ))
}

fn evaluate_all_invariants(repo: &RepositoryModel) -> Result<Vec<InvariantResult>, String> {
    if let Some(err) = &repo.invariants_error {
        return Err(err.clone());
    }
    if repo.invariants.is_empty() {
        return Err("architecture/invariants.toml [[invariant]] array must be non-empty".into());
    }
    let mut results = Vec::with_capacity(repo.invariants.len());
    for inv in &repo.invariants {
        if inv.id.is_empty() || inv.summary.is_empty() || inv.guard_check.is_empty() {
            return Err(format!(
                "invariant row missing required non-empty id/summary/guard_check ({})",
                inv.id
            ));
        }
        results.push(evaluate_invariant(repo, inv, repo.invariants.len()));
    }
    Ok(results)
}

fn evaluate_invariant(
    repo: &RepositoryModel,
    inv: &ArchitectureInvariant,
    total: usize,
) -> InvariantResult {
    let (passed, evidence) = match inv.id.as_str() {
        "INV-OWNERSHIP-LIVE-CRATES" => eval_ownership_live_crates(repo),
        "INV-NO-HYPOTHETICAL-PACKAGES" => eval_no_hypothetical_packages(repo),
        "INV-DEBT-RESOLVED-HAS-PROOF" => eval_debt_resolved_has_proof(repo),
        "INV-INVARIANTS-EVALUATED" => {
            let backlog = inv
                .summary
                .to_ascii_lowercase()
                .contains("remaining_backlog");
            if backlog {
                (
                    false,
                    "INV-INVARIANTS-EVALUATED must not claim remaining_backlog".into(),
                )
            } else if total == 0 {
                (false, "no invariants evaluated".into())
            } else {
                (
                    true,
                    format!(
                        "every [[invariant]] ({total}) is evaluated against RepositoryModel; skip is illegal without a live debt id"
                    ),
                )
            }
        }
        other => (
            false,
            format!("unknown invariant {other} has no evaluation predicate"),
        ),
    };
    InvariantResult {
        id: inv.id.clone(),
        summary: inv.summary.clone(),
        guard_check: inv.guard_check.clone(),
        passed,
        evidence,
    }
}

fn eval_ownership_live_crates(repo: &RepositoryModel) -> (bool, String) {
    let Some(arch) = &repo.architecture else {
        return (
            false,
            repo.architecture_error
                .clone()
                .unwrap_or_else(|| "architecture manifest missing".into()),
        );
    };
    let mut problems = Vec::new();
    for (concept, crate_name, required_paths) in REQUIRED_OWNERSHIP {
        let Some(row) = arch.ownership.get(concept) else {
            problems.push(format!("ownership.{concept} missing"));
            continue;
        };
        if row.crate_name != crate_name {
            problems.push(format!(
                "ownership.{concept}.crate must be {crate_name}, got {}",
                row.crate_name
            ));
        }
        if !repo.package_names.contains(&row.crate_name) {
            problems.push(format!(
                "ownership.{concept}.crate {} is not a workspace package",
                row.crate_name
            ));
        }
        for needle in required_paths {
            if !row.paths.iter().any(|p| p == needle || p.contains(needle)) {
                problems.push(format!("ownership.{concept}.paths must include {needle}"));
            }
        }
        for rel in &row.paths {
            if !repo.rel_exists(rel) {
                problems.push(format!("ownership.{concept} path {rel} does not exist"));
            }
        }
    }
    if problems.is_empty() {
        (
            true,
            "ownership crates are workspace members and paths exist".into(),
        )
    } else {
        (false, problems.join("; "))
    }
}

fn eval_no_hypothetical_packages(repo: &RepositoryModel) -> (bool, String) {
    let mut hits = Vec::new();
    for name in FORBIDDEN_PACKAGES {
        if repo.package_names.iter().any(|p| p == name) {
            hits.push(name.to_string());
        }
    }
    if hits.is_empty() {
        (
            true,
            "no workspace member named weeping-angel-catalog or weeping-angel-assurance-cli".into(),
        )
    } else {
        (
            false,
            format!("hypothetical packages present: {}", hits.join(", ")),
        )
    }
}

fn eval_debt_resolved_has_proof(repo: &RepositoryModel) -> (bool, String) {
    match &repo.debt_error {
        Some(err) => (false, err.clone()),
        None => (
            true,
            "debt register validates resolved-without-proof law (check 13)".into(),
        ),
    }
}

fn check_02_on_model(repo: &RepositoryModel) -> Result<(), String> {
    let arch = repo.architecture.as_ref().ok_or_else(|| {
        repo.architecture_error
            .clone()
            .unwrap_or_else(|| "architecture/architecture.toml is not a file".into())
    })?;
    if arch.schema != ARCH_SCHEMA {
        return Err(format!(
            "architecture.toml schema must be {ARCH_SCHEMA}, got {}",
            arch.schema
        ));
    }
    if arch.ownership.is_empty() {
        return Err("architecture.toml missing [ownership] table".into());
    }

    for (concept, crate_name, required_paths) in REQUIRED_OWNERSHIP {
        let entry = arch
            .ownership
            .get(concept)
            .ok_or_else(|| format!("ownership.{concept} is mandatory"))?;
        if entry.crate_name != crate_name {
            return Err(format!(
                "ownership.{concept}.crate must be {crate_name}, got {}",
                entry.crate_name
            ));
        }
        if FORBIDDEN_PACKAGES.contains(&entry.crate_name.as_str()) {
            return Err(format!(
                "ownership.{concept}.crate must not be hypothetical package {}",
                entry.crate_name
            ));
        }
        let kind = entry
            .kind
            .as_deref()
            .ok_or_else(|| format!("ownership.{concept}.kind is required"))?;
        if !OWNERSHIP_KINDS.contains(&kind) {
            return Err(format!(
                "ownership.{concept}.kind must be one of exclusive|facade|projection|adapter|shared-primitive, got {kind}"
            ));
        }
        if entry.paths.is_empty() {
            return Err(format!("ownership.{concept}.paths must be non-empty"));
        }
        for needle in required_paths {
            if !entry
                .paths
                .iter()
                .any(|p| p == needle || p.contains(needle))
            {
                return Err(format!("ownership.{concept}.paths must include {needle}"));
            }
        }
        for rel in &entry.paths {
            if !repo.rel_exists(rel) {
                return Err(format!(
                    "ownership.{concept} path {rel} does not exist on disk"
                ));
            }
        }
    }

    for (concept, entry) in &arch.ownership {
        if FORBIDDEN_PACKAGES.contains(&entry.crate_name.as_str()) {
            return Err(format!(
                "ownership.{concept} binds hypothetical package {}",
                entry.crate_name
            ));
        }
        match entry.kind.as_deref() {
            Some(kind) if OWNERSHIP_KINDS.contains(&kind) => {}
            Some(kind) => {
                return Err(format!(
                    "ownership.{concept}.kind must be one of exclusive|facade|projection|adapter|shared-primitive, got {kind}"
                ));
            }
            None => {
                return Err(format!("ownership.{concept}.kind is required"));
            }
        }
        for rel in &entry.paths {
            if !repo.rel_exists(rel) {
                return Err(format!(
                    "ownership.{concept} path {rel} does not exist on disk"
                ));
            }
        }
    }
    Ok(())
}

fn check_03_on_model(repo: &RepositoryModel) -> Result<(), String> {
    if let Some(err) = &repo.forbidden_error {
        return Err(err.clone());
    }
    for pattern in &repo.forbidden {
        evaluate_forbidden_pattern(repo, pattern)?;
    }
    Ok(())
}

fn evaluate_forbidden_pattern(
    repo: &RepositoryModel,
    pattern: &ForbiddenPattern,
) -> Result<(), String> {
    let kind = pattern
        .kind
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| format!("pattern {} missing required kind", pattern.id))?;
    if pattern.value.is_empty() {
        return Err(format!("pattern {} has empty value", pattern.id));
    }
    match kind {
        "package" => {
            if repo.package_names.iter().any(|n| n == &pattern.value) {
                return Err(format!(
                    "{} forbids package {} (present in workspace)",
                    pattern.id, pattern.value
                ));
            }
            Ok(())
        }
        "path" => {
            if repo.rel_exists(&pattern.value) {
                return Err(format!(
                    "{} forbids path {} (exists on disk)",
                    pattern.id, pattern.value
                ));
            }
            Ok(())
        }
        "dependency" => {
            let (from, to) = parse_dependency_edge(&pattern.value).ok_or_else(|| {
                format!(
                    "{} dependency value must be `from -> to`, got {}",
                    pattern.id, pattern.value
                )
            })?;
            if let Some(deps) = repo.package_graph.get(&from) {
                if deps.contains(&to) {
                    return Err(format!("{} forbids dependency {from} -> {to}", pattern.id));
                }
            }
            Ok(())
        }
        "symbol" => {
            let search = pattern
                .extra
                .get("symbol")
                .cloned()
                .unwrap_or_else(|| pattern.value.clone());
            if repo.source_contains(&search) {
                if let Some(only) = pattern.extra.get("in_crate") {
                    let prefix = format!("crates/{only}/");
                    let hit = repo.source_files.iter().any(|rel| {
                        rel.starts_with(&prefix)
                            && fs::read_to_string(repo.root.join(rel))
                                .is_ok_and(|t| t.contains(&search))
                    });
                    if hit {
                        return Err(format!(
                            "{} forbids symbol {search} in crate {only}",
                            pattern.id
                        ));
                    }
                } else {
                    return Err(format!("{} forbids symbol {search} in source", pattern.id));
                }
            }
            Ok(())
        }
        "source-pattern" => {
            if repo.source_contains(&pattern.value) {
                return Err(format!(
                    "{} source-pattern {} matched RepositoryModel source index",
                    pattern.id, pattern.value
                ));
            }
            Ok(())
        }
        other => Err(format!(
            "pattern {} has unknown kind {other} (allowed: package|path|dependency|symbol|source-pattern)",
            pattern.id
        )),
    }
}

fn parse_dependency_edge(value: &str) -> Option<(String, String)> {
    let (left, right) = value.split_once("->")?;
    let from = left.trim().to_string();
    let to = right.trim().to_string();
    if from.is_empty() || to.is_empty() {
        None
    } else {
        Some((from, to))
    }
}

fn load_architecture_manifest(root: &Path) -> Result<ArchitectureManifest, String> {
    let path = root.join("architecture/architecture.toml");
    if !path.is_file() {
        return Err("architecture/architecture.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, ARCH_SCHEMA, &path)?;
    let ownership_table = value
        .get("ownership")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "architecture.toml missing [ownership] table".to_string())?;
    let mut ownership = BTreeMap::new();
    for (concept, entry) in ownership_table {
        let crate_name = entry
            .get("crate")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let kind = entry
            .get("kind")
            .and_then(|c| c.as_str())
            .map(str::to_string);
        let paths = entry
            .get("paths")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        ownership.insert(
            concept.clone(),
            OwnershipRow {
                crate_name,
                kind,
                paths,
            },
        );
    }
    Ok(ArchitectureManifest {
        schema: ARCH_SCHEMA.to_string(),
        ownership,
    })
}

fn load_invariants(root: &Path) -> Result<Vec<ArchitectureInvariant>, String> {
    let path = root.join("architecture/invariants.toml");
    if !path.is_file() {
        return Err("architecture/invariants.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, INVARIANTS_SCHEMA, &path)?;
    let rows = value
        .get("invariant")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "architecture/invariants.toml missing [[invariant]] array".to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(ArchitectureInvariant {
            id: row
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            summary: row
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            guard_check: row
                .get("guard_check")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(out)
}

fn load_forbidden_patterns(root: &Path) -> Result<Vec<ForbiddenPattern>, String> {
    let path = root.join("architecture/forbidden-patterns.toml");
    if !path.is_file() {
        return Err("architecture/forbidden-patterns.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, FORBIDDEN_SCHEMA, &path)?;
    let Some(rows) = value.get("pattern").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for row in rows {
        let mut extra = BTreeMap::new();
        if let Some(table) = row.as_table() {
            for (k, v) in table {
                if matches!(k.as_str(), "id" | "kind" | "value" | "rationale") {
                    continue;
                }
                if let Some(s) = v.as_str() {
                    extra.insert(k.clone(), s.to_string());
                }
            }
        }
        out.push(ForbiddenPattern {
            id: row
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            kind: row.get("kind").and_then(|v| v.as_str()).map(str::to_string),
            value: row
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            extra,
        });
    }
    Ok(out)
}

fn load_workspace(
    root: &Path,
) -> (
    Vec<String>,
    BTreeSet<String>,
    BTreeMap<String, BTreeSet<String>>,
) {
    let mut members = Vec::new();
    let mut names = BTreeSet::new();
    let mut graph = BTreeMap::new();
    let cargo_path = root.join("Cargo.toml");
    let Ok(value) = read_toml(&cargo_path) else {
        return (members, names, graph);
    };
    if let Some(name) = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        names.insert(name.to_string());
        graph.insert(name.to_string(), collect_dep_names(&value));
        members.push(".".to_string());
    }
    if let Some(listed) = value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
    {
        for item in listed {
            let Some(rel) = item.as_str() else { continue };
            members.push(rel.to_string());
            let member_cargo = root.join(rel).join("Cargo.toml");
            if let Ok(member) = read_toml(&member_cargo) {
                if let Some(name) = member
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                {
                    names.insert(name.to_string());
                    graph.insert(name.to_string(), collect_dep_names(&member));
                }
            }
        }
    }
    (members, names, graph)
}

fn collect_dep_names(value: &toml::Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = value.get(key).and_then(|v| v.as_table()) {
            out.extend(table.keys().cloned());
        }
    }
    out
}

fn index_tree(root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if SKIP_DIR_NAMES.iter().any(|s| *s == name_str) || name_str.starts_with("target") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(root) {
            out.insert(rel.to_string_lossy().replace('\\', "/"));
        }
        if path.is_dir() {
            index_tree(root, &path, out);
        }
    }
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if SKIP_DIR_NAMES.iter().any(|s| *s == name) {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn list_dir_files(dir: &Path, ext: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out
}

/// Validate a debt-register TOML string. Rejects duplicate ids and
/// `status = "resolved"` without `regression_tests` or `repository_guard`.
pub fn validate_debt_register_str(text: &str) -> Result<BTreeSet<String>, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("debt register is not parseable TOML: {e}"))?;
    validate_debt_register_value(&value)
}

pub fn validate_debt_register_file(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    validate_debt_register_str(&text)
}

fn load_and_validate_debt_register(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("docs/debt/register.toml");
    if !path.is_file() {
        return Err("docs/debt/register.toml is not a file".into());
    }
    validate_debt_register_file(&path)
}

fn validate_debt_register_value(value: &toml::Value) -> Result<BTreeSet<String>, String> {
    let schema = value.get("schema").and_then(|s| s.as_str());
    if schema != Some(DEBT_SCHEMA) {
        return Err(format!(
            "debt register schema must be {DEBT_SCHEMA}, got {schema:?}"
        ));
    }
    let findings = value
        .get("finding")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "debt register must contain [[finding]] rows".to_string())?;

    let mut ids = BTreeSet::new();
    for (idx, finding) in findings.iter().enumerate() {
        let id = required_nonempty(finding, "id", idx)?;
        let _title = required_nonempty(finding, "title", idx)?;
        let _summary = required_nonempty(finding, "summary", idx)?;
        let status = required_nonempty(finding, "status", idx)?;
        if !ALLOWED_STATUS.contains(&status.as_str()) {
            return Err(format!("finding {id} has illegal status {status}"));
        }
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate finding id {id} (ids must be unique)"));
        }
        if status == "resolved" && !has_resolution_proof(finding) {
            return Err(format!(
                "resolved finding {id} is rejected without proof: list non-empty regression_tests or repository_guard"
            ));
        }
    }
    Ok(ids)
}

fn required_nonempty(finding: &toml::Value, field: &str, idx: usize) -> Result<String, String> {
    match finding.get(field).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(format!("finding[{idx}] missing required non-empty {field}")),
    }
}

fn has_resolution_proof(finding: &toml::Value) -> bool {
    let tests = finding
        .get("regression_tests")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().any(|x| x.as_str().is_some_and(|s| !s.is_empty())))
        .unwrap_or(false);
    let guard = match finding.get("repository_guard") {
        Some(toml::Value::String(s)) => !s.is_empty(),
        Some(toml::Value::Boolean(true)) => true,
        Some(toml::Value::Integer(_)) => true,
        _ => false,
    };
    tests || guard
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "schema = \"weeping-angel/debt-register/v1\"\n";

    fn finding(id: &str, status: &str, extra: &str) -> String {
        format!(
            r#"
[[finding]]
id = "{id}"
title = "t"
status = "{status}"
summary = "s"
{extra}
"#
        )
    }

    #[test]
    fn schema_constants_match_spec() {
        assert_eq!(ARCH_SCHEMA, "weeping-angel/architecture/v1");
        assert_eq!(
            INVARIANTS_SCHEMA,
            "weeping-angel/architecture-invariants/v1"
        );
        assert_eq!(FORBIDDEN_SCHEMA, "weeping-angel/forbidden-patterns/v1");
        assert_eq!(DEBT_SCHEMA, "weeping-angel/debt-register/v1");
    }

    #[test]
    fn accepts_open_finding() {
        let text = format!("{}{}", HEADER, finding("DEBT-1", "open", ""));
        let ids = validate_debt_register_str(&text).expect("valid");
        assert!(ids.contains("DEBT-1"));
    }

    #[test]
    fn rejects_duplicate_finding_ids() {
        let text = format!(
            "{}{}{}",
            HEADER,
            finding("DEBT-1", "open", ""),
            finding("DEBT-1", "confirmed", "")
        );
        let err = validate_debt_register_str(&text).expect_err("duplicate");
        assert!(err.contains("duplicate") || err.contains("unique"), "{err}");
    }

    #[test]
    fn rejects_resolved_without_proof() {
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", ""));
        let err = validate_debt_register_str(&text).expect_err("resolved without proof");
        assert!(err.contains("resolved"), "{err}");
        assert!(
            err.contains("regression_tests") && err.contains("repository_guard"),
            "{err}"
        );
    }

    #[test]
    fn accepts_resolved_with_regression_tests() {
        let extra = r#"regression_tests = ["sdd_repository_integrity_target"]"#;
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", extra));
        validate_debt_register_str(&text).expect("proof via regression_tests");
    }

    #[test]
    fn accepts_resolved_with_repository_guard() {
        let extra = r#"repository_guard = "13""#;
        let text = format!("{}{}", HEADER, finding("DEBT-X", "resolved", extra));
        validate_debt_register_str(&text).expect("proof via repository_guard");
    }
}
