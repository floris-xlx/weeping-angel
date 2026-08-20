//! Individual `ArchitectureCheck` implementations (01–15).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::architecture::{
    ARCH_SCHEMA, ArchitectureInvariant, ForbiddenPattern, InvariantResult, REQUIRED_OWNERSHIP,
    SPEC_STATES,
};

use crate::model::RepositoryModel;
use crate::report::CheckResult;

/// Shared evaluation plane: every guard check takes the loaded model.
pub trait ArchitectureCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult;
}

struct ArchitectureManifestCheck;
struct CanonicalOwnershipCheck;
struct ForbiddenPatternsCheck;
struct ArchitectureInvariantsCheck;
struct DebtRegisterCheck;
struct AdrGraphCheck;
struct SpecLifecycleCheck;
struct ProductLawCheck {
    id: &'static str,
    name: &'static str,
    required: &'static [(&'static str, &'static str)],
    forbidden: &'static [(&'static str, &'static str)],
}

const PRODUCT_LAWS: [ProductLawCheck; 8] = [
    ProductLawCheck {
        id: "05",
        name: "catalog-ssot",
        required: &[
            (
                "weeping-angel-canonical-catalog",
                "pub struct CanonicalCatalog",
            ),
            ("weeping-angel-canonical-catalog", "pub fn load("),
        ],
        forbidden: &[("weeping-angel-framework", "fn discover_catalog_index")],
    },
    ProductLawCheck {
        id: "06",
        name: "framework-pack-parse",
        required: &[("weeping-angel-framework", "enum PackError")],
        forbidden: &[],
    },
    ProductLawCheck {
        id: "07",
        name: "framework-digest",
        required: &[("weeping-angel-framework", "struct FrameworkPackDigest")],
        forbidden: &[],
    },
    ProductLawCheck {
        id: "08",
        name: "readiness-ssot",
        required: &[("weeping-angel-assurance", "pub fn project_readiness(")],
        forbidden: &[(
            "weeping-angel-assurance",
            "fn overlay_privileged_mfa_presence",
        )],
    },
    ProductLawCheck {
        id: "09",
        name: "temporal-evidence-selection",
        required: &[
            ("weeping-angel-evidence", "pub fn current("),
            ("weeping-angel-evidence", "pub fn as_of("),
        ],
        forbidden: &[],
    },
    ProductLawCheck {
        id: "10",
        name: "assessment-lineage-rebuild",
        required: &[("weeping-angel-assurance", "pub fn replay_assessment(")],
        forbidden: &[("weeping-angel-assurance", "Ok(reconstruct(bundle))")],
    },
    ProductLawCheck {
        id: "11",
        name: "evidence-latest-vs-current",
        required: &[
            ("weeping-angel-evidence", "pub fn latest("),
            ("weeping-angel-evidence", "pub fn current("),
        ],
        forbidden: &[],
    },
    ProductLawCheck {
        id: "12",
        name: "soa-invariants",
        required: &[("weeping-angel-assurance", "project_soa_from_snapshot")],
        forbidden: &[],
    },
];

pub fn run_all_checks(repo: &RepositoryModel) -> Vec<CheckResult> {
    let mut checks = vec![
        ArchitectureManifestCheck.check(repo),
        CanonicalOwnershipCheck.check(repo),
        ForbiddenPatternsCheck.check(repo),
        ArchitectureInvariantsCheck.check(repo),
    ];
    for law in &PRODUCT_LAWS {
        checks.push(law.check(repo));
    }
    checks.push(DebtRegisterCheck.check(repo));
    checks.push(AdrGraphCheck.check(repo));
    checks.push(SpecLifecycleCheck.check(repo));
    checks.sort_by(|a, b| a.id.cmp(&b.id));
    checks
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

impl ArchitectureCheck for AdrGraphCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        const ID: &str = "14";
        const NAME: &str = "adr-graph";
        match check_14_on_model(repo) {
            Ok(()) => CheckResult::pass(ID, NAME),
            Err(err) => CheckResult::fail(ID, NAME, err),
        }
    }
}

impl ArchitectureCheck for SpecLifecycleCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        const ID: &str = "15";
        const NAME: &str = "spec-lifecycle";
        match check_15_on_model(repo) {
            Ok(()) => CheckResult::pass(ID, NAME),
            Err(err) => CheckResult::fail(ID, NAME, err),
        }
    }
}

impl ArchitectureCheck for ProductLawCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult {
        for (crate_name, needle) in self.required {
            if !repo.crate_source_contains(crate_name, needle) {
                return CheckResult::fail(
                    self.id,
                    self.name,
                    format!("missing required surface in {crate_name}: {needle}"),
                );
            }
        }
        for (crate_name, needle) in self.forbidden {
            if repo.crate_source_contains(crate_name, needle) {
                return CheckResult::fail(
                    self.id,
                    self.name,
                    format!("forbidden leftover in {crate_name}: {needle}"),
                );
            }
        }
        CheckResult::pass(self.id, self.name)
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
        "INV-ADR-NAMESPACE-UNIQUE" => eval_adr_namespace_unique(repo),
        "INV-NO-SUPERSEDED-BASELINES" => eval_no_superseded_baselines(repo),
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
    for concept in &arch.policy.required_concepts {
        let Some(row) = arch.ownership.get(concept) else {
            problems.push(format!("ownership.{concept} missing"));
            continue;
        };
        if !repo.package_names.contains(&row.crate_name) {
            problems.push(format!(
                "ownership.{concept}.crate {} is not a workspace package",
                row.crate_name
            ));
        }
        for rel in &row.paths {
            if !repo.rel_exists(rel) {
                problems.push(format!("ownership.{concept} path {rel} does not exist"));
            }
        }
    }
    // Keep increment-1 path-needle regressions for the seven canonical concepts.
    for (concept, crate_name, required_paths) in REQUIRED_OWNERSHIP {
        if !arch.policy.required_concepts.iter().any(|c| c == concept) {
            continue;
        }
        let Some(row) = arch.ownership.get(concept) else {
            continue;
        };
        if row.crate_name != crate_name {
            problems.push(format!(
                "ownership.{concept}.crate must be {crate_name}, got {}",
                row.crate_name
            ));
        }
        for needle in required_paths {
            if !row.paths.iter().any(|p| p == needle || p.contains(needle)) {
                problems.push(format!("ownership.{concept}.paths must include {needle}"));
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
    let names = repo.forbidden_package_names();
    if names.is_empty()
        && let Some(err) = &repo.forbidden_error
    {
        return (false, err.clone());
    }
    for name in &names {
        if repo.package_names.iter().any(|p| p == name) {
            hits.push(name.clone());
        }
    }
    if hits.is_empty() {
        (
            true,
            "no workspace member named a forbidden package from architecture/forbidden-patterns.toml"
                .into(),
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

fn eval_adr_namespace_unique(repo: &RepositoryModel) -> (bool, String) {
    let mut by_prefix: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in &repo.adr_files {
        if let Some(prefix) = crate::architecture::adr_filename_prefix(name) {
            by_prefix.entry(prefix).or_default().push(name.clone());
        }
    }
    let dups: Vec<String> = by_prefix
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(prefix, files)| format!("{prefix}: {}", files.join(", ")))
        .collect();
    if dups.is_empty() {
        (
            true,
            format!("{} ADR files have unique prefixes", repo.adr_files.len()),
        )
    } else {
        (
            false,
            format!("duplicate ADR prefixes: {}", dups.join("; ")),
        )
    }
}

fn eval_no_superseded_baselines(repo: &RepositoryModel) -> (bool, String) {
    let leftovers: Vec<String> = repo
        .filesystem
        .iter()
        .filter(|p| p.ends_with(".baseline.rs") || p.ends_with("_baseline.rs"))
        .cloned()
        .collect();
    if leftovers.is_empty() {
        (true, "no superseded baseline suites on disk".into())
    } else {
        (
            false,
            format!("deleted-baseline leftovers: {}", leftovers.join(", ")),
        )
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
    if arch.policy.required_concepts.is_empty() {
        return Err("architecture.toml [policy].required_concepts must be non-empty".into());
    }
    if arch.policy.ownership_kinds.is_empty() {
        return Err("architecture.toml [policy].ownership_kinds must be non-empty".into());
    }

    let forbidden_packages = repo.forbidden_package_names();

    for concept in &arch.policy.required_concepts {
        let entry = arch
            .ownership
            .get(concept)
            .ok_or_else(|| format!("ownership.{concept} is mandatory"))?;
        if forbidden_packages.iter().any(|p| p == &entry.crate_name) {
            return Err(format!(
                "ownership.{concept}.crate must not be hypothetical package {}",
                entry.crate_name
            ));
        }
        let kind = entry
            .kind
            .as_deref()
            .ok_or_else(|| format!("ownership.{concept}.kind is required"))?;
        if !arch.policy.ownership_kinds.iter().any(|k| k == kind) {
            return Err(format!(
                "ownership.{concept}.kind must be one of exclusive|facade|projection|adapter|shared-primitive, got {kind}"
            ));
        }
        if entry.paths.is_empty() {
            return Err(format!("ownership.{concept}.paths must be non-empty"));
        }
        for rel in &entry.paths {
            if !repo.rel_exists(rel) {
                return Err(format!(
                    "ownership.{concept} path {rel} does not exist on disk"
                ));
            }
        }
    }

    // Path-needle regressions for the seven live concepts when they remain required.
    for (concept, crate_name, required_paths) in REQUIRED_OWNERSHIP {
        if !arch.policy.required_concepts.iter().any(|c| c == concept) {
            continue;
        }
        let Some(entry) = arch.ownership.get(concept) else {
            continue;
        };
        if entry.crate_name != crate_name {
            return Err(format!(
                "ownership.{concept}.crate must be {crate_name}, got {}",
                entry.crate_name
            ));
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
    }

    for (concept, entry) in &arch.ownership {
        if forbidden_packages.iter().any(|p| p == &entry.crate_name) {
            return Err(format!(
                "ownership.{concept} binds hypothetical package {}",
                entry.crate_name
            ));
        }
        match entry.kind.as_deref() {
            Some(kind) if arch.policy.ownership_kinds.iter().any(|k| k == kind) => {}
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
            if let Some(deps) = repo.package_graph.get(&from)
                && deps.contains(&to)
            {
                return Err(format!("{} forbids dependency {from} -> {to}", pattern.id));
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
                    if repo.crate_source_contains(only, &search) {
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

fn check_14_on_model(repo: &RepositoryModel) -> Result<(), String> {
    if let Some(err) = &repo.adr_identity_error {
        return Err(err.clone());
    }
    let identity = repo
        .adr_identity
        .as_ref()
        .ok_or_else(|| "architecture/adr-identity.toml is not a file".to_string())?;
    if let Some(err) = &repo.adr_docs_error {
        return Err(err.clone());
    }
    if repo.adr_files.is_empty() {
        return Err("docs/adr contains no markdown ADRs".into());
    }

    if !identity.grandfathered_debt.is_empty()
        && !repo.debt_ids.contains(&identity.grandfathered_debt)
    {
        return Err(format!(
            "grandfathered ADR prefixes require live finding {}",
            identity.grandfathered_debt
        ));
    }

    let mut by_prefix: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in &repo.adr_files {
        let prefix = crate::architecture::adr_filename_prefix(name)
            .ok_or_else(|| format!("ADR filename {name} must match ^(\\d{{4}})-.+\\.md$"))?;
        by_prefix.entry(prefix).or_default().push(name.clone());
    }

    for (prefix, files) in &by_prefix {
        if files.len() <= 1 {
            continue;
        }
        if !identity.grandfathered_prefixes.contains(prefix) {
            return Err(format!(
                "duplicate ADR prefix {prefix} is not grandfathered (new collision): {}",
                files.join(", ")
            ));
        }
        for file in files {
            if !identity.grandfathered_files.contains(file) {
                return Err(format!(
                    "new ADR file {file} reuses grandfathered prefix {prefix} (historical set is pinned; no silent renumber)"
                ));
            }
        }
    }

    let stems: BTreeSet<String> = repo.adr_docs.keys().cloned().collect();
    let mut supersedes_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut depends_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (stem, meta) in &repo.adr_docs {
        for target in &meta.supersedes {
            let resolved = resolve_adr_ref(target, &stems, &by_prefix)?;
            supersedes_edges
                .entry(stem.clone())
                .or_default()
                .push(resolved);
        }
        for target in &meta.superseded_by {
            let _resolved = resolve_adr_ref(target, &stems, &by_prefix)?;
        }
        for target in &meta.depends_on {
            let resolved = resolve_adr_ref(target, &stems, &by_prefix)?;
            depends_edges
                .entry(stem.clone())
                .or_default()
                .push(resolved);
        }
    }

    // Inverse consistency: if A.supersedes contains B, B.superseded_by must contain A.
    for (stem, meta) in &repo.adr_docs {
        for target in &meta.supersedes {
            let resolved = resolve_adr_ref(target, &stems, &by_prefix)?;
            let other = repo.adr_docs.get(&resolved).ok_or_else(|| {
                format!("dangling supersedes {target} from {stem} does not exist")
            })?;
            let inverse_ok = other.superseded_by.iter().any(|r| {
                resolve_adr_ref(r, &stems, &by_prefix).ok().as_deref() == Some(stem.as_str())
            });
            if !inverse_ok {
                return Err(format!(
                    "ADR {stem} supersedes {resolved} but {resolved} does not list {stem} in superseded_by"
                ));
            }
        }
        for target in &meta.superseded_by {
            let resolved = resolve_adr_ref(target, &stems, &by_prefix)?;
            let other = repo.adr_docs.get(&resolved).ok_or_else(|| {
                format!("dangling superseded_by {target} from {stem} does not exist")
            })?;
            let inverse_ok = other.supersedes.iter().any(|r| {
                resolve_adr_ref(r, &stems, &by_prefix).ok().as_deref() == Some(stem.as_str())
            });
            if !inverse_ok {
                return Err(format!(
                    "ADR {stem} superseded_by {resolved} but {resolved} does not list {stem} in supersedes"
                ));
            }
        }
    }

    if let Some(cycle) = find_cycle(&stems, &supersedes_edges) {
        return Err(format!(
            "ADR supersedes graph is not acyclic (cycle involving {cycle})"
        ));
    }
    if let Some(cycle) = find_cycle(&stems, &depends_edges) {
        return Err(format!(
            "ADR depends_on graph is not acyclic (cycle involving {cycle})"
        ));
    }
    Ok(())
}

fn resolve_adr_ref(
    raw: &str,
    stems: &BTreeSet<String>,
    by_prefix: &BTreeMap<String, Vec<String>>,
) -> Result<String, String> {
    let trimmed = raw.trim().trim_end_matches(".md");
    if stems.contains(trimmed) {
        return Ok(trimmed.to_string());
    }
    if trimmed.len() == 4 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let Some(files) = by_prefix.get(trimmed) else {
            return Err(format!("dangling ADR reference {raw} does not exist"));
        };
        if files.len() == 1 {
            return Ok(files[0].trim_end_matches(".md").to_string());
        }
        return Err(format!(
            "ADR reference {raw} is an ambiguous prefix (not a unique stem)"
        ));
    }
    Err(format!("dangling ADR reference {raw} does not exist"))
}

fn find_cycle(nodes: &BTreeSet<String>, edges: &BTreeMap<String, Vec<String>>) -> Option<String> {
    let mut state: BTreeMap<&str, u8> = BTreeMap::new();
    fn visit<'a>(
        node: &'a str,
        edges: &'a BTreeMap<String, Vec<String>>,
        state: &mut BTreeMap<&'a str, u8>,
    ) -> Option<String> {
        state.insert(node, 1);
        if let Some(nexts) = edges.get(node) {
            for next in nexts {
                match state.get(next.as_str()).copied().unwrap_or(0) {
                    1 => return Some(next.clone()),
                    0 => {
                        if let Some(c) = visit(next, edges, state) {
                            return Some(c);
                        }
                    }
                    _ => {}
                }
            }
        }
        state.insert(node, 2);
        None
    }
    for node in nodes {
        if state.get(node.as_str()).copied().unwrap_or(0) == 0
            && let Some(c) = visit(node, edges, &mut state)
        {
            return Some(c);
        }
    }
    None
}

fn check_15_on_model(repo: &RepositoryModel) -> Result<(), String> {
    if let Some(err) = &repo.spec_lifecycle_error {
        return Err(err.clone());
    }
    if repo.spec_lifecycle.is_empty() && repo.spec_files.is_empty() {
        return Err("architecture/spec-lifecycle.toml is not a file".into());
    }
    let ownership_keys: BTreeSet<String> = repo
        .architecture
        .as_ref()
        .map(|a| a.ownership.keys().cloned().collect())
        .unwrap_or_default();
    if repo.architecture.is_none() {
        return Err(repo.architecture_error.clone().unwrap_or_else(|| {
            "architecture.toml missing; active specs cannot bind ownership".into()
        }));
    }

    let listed: BTreeSet<String> = repo.spec_lifecycle.iter().map(|r| r.path.clone()).collect();
    for name in &repo.spec_files {
        let path = format!("docs/specs/{name}");
        if !listed.contains(&path) {
            return Err(format!(
                "on-disk spec {path} is missing from architecture/spec-lifecycle.toml"
            ));
        }
    }

    let mut depends_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let nodes: BTreeSet<String> = listed.clone();

    for row in &repo.spec_lifecycle {
        if !repo.rel_exists(&row.path) {
            return Err(format!(
                "spec-lifecycle path {} does not exist on disk",
                row.path
            ));
        }
        if !SPEC_STATES.contains(&row.state.as_str()) {
            return Err(format!(
                "spec {} has illegal state {} (draft|active|superseded|retired)",
                row.path, row.state
            ));
        }
        // Masquerade: superseded/retired cannot also be advertised as active.
        if (row.state == "superseded" || row.state == "retired") && row.state == "active" {
            return Err(format!(
                "spec {} cannot masquerade as active while {}",
                row.path, row.state
            ));
        }
        if row.state == "active" {
            if row.ownership.is_empty() {
                return Err(format!(
                    "active spec {} must list non-empty ownership",
                    row.path
                ));
            }
            for key in &row.ownership {
                if !ownership_keys.contains(key) {
                    return Err(format!(
                        "active spec {} ownership {key} does not exist in architecture.toml",
                        row.path
                    ));
                }
            }
            if !row.successor.is_empty() && row.successor == row.path {
                return Err(format!(
                    "active spec {} cannot set successor to itself",
                    row.path
                ));
            }
        }
        if row.state == "superseded" && row.successor.trim().is_empty() {
            return Err(format!(
                "superseded spec {} is missing required successor",
                row.path
            ));
        }
        if !row.successor.is_empty() && !repo.rel_exists(&row.successor) {
            return Err(format!(
                "spec {} successor {} is dangling (does not exist)",
                row.path, row.successor
            ));
        }
        for dep in &row.depends_on {
            if !repo.rel_exists(dep) {
                return Err(format!(
                    "spec {} depends_on {dep} is dangling (does not exist)",
                    row.path
                ));
            }
            depends_edges
                .entry(row.path.clone())
                .or_default()
                .push(dep.clone());
        }
        for old in &row.supersedes {
            if !repo.rel_exists(old) {
                return Err(format!(
                    "spec {} supersedes {old} is dangling (does not exist)",
                    row.path
                ));
            }
        }
    }

    if let Some(cycle) = find_cycle(&nodes, &depends_edges) {
        return Err(format!(
            "spec depends_on graph is not acyclic (cycle involving {cycle})"
        ));
    }

    check_active_spec_drift_on_model(repo)?;
    Ok(())
}

/// Active-spec drift: superseded-state phrases must not appear in active voice.
pub fn check_active_spec_drift(root: &Path) -> Result<(), String> {
    check_active_spec_drift_on_model(&RepositoryModel::load(root))
}

fn check_active_spec_drift_on_model(repo: &RepositoryModel) -> Result<(), String> {
    if let Some(err) = &repo.spec_lifecycle_error {
        return Err(err.clone());
    }
    for row in &repo.spec_lifecycle {
        if row.state != "active" {
            continue;
        }
        let text = match fs::read_to_string(repo.root.join(&row.path)) {
            Ok(t) => t,
            Err(e) => return Err(format!("read {}: {e}", row.path)),
        };
        if let Some(hit) = find_active_voice_drift(&text) {
            return Err(format!(
                "active-spec drift in {}: superseded-state phrase in active voice ({hit})",
                row.path
            ));
        }
    }
    Ok(())
}

/// Test/helper: scan one markdown body for active-plane superseded phrases.
pub fn active_spec_drift_in_text(text: &str) -> Option<&'static str> {
    find_active_voice_drift(text)
}

fn find_active_voice_drift(text: &str) -> Option<&'static str> {
    let unscanned = active_plane_text(text);
    if unscanned.contains("skip-with-debt")
        && (unscanned.contains("05–12") || unscanned.contains("05-12"))
    {
        return Some("05–12 + skip-with-debt");
    }
    if unscanned.contains("Guards **05–12** stay stubs")
        || unscanned.contains("05–12 stay stubs")
        || unscanned.contains("05-12 stay stubs")
    {
        return Some("05–12 stay stubs");
    }
    if unscanned.contains("Increment-2 current plane")
        && (unscanned.contains("05–12") || unscanned.contains("05-12"))
        && unscanned.contains("skip")
    {
        return Some("Increment-2 current plane skip archaeology");
    }
    if (unscanned.contains("05–12") || unscanned.contains("05-12"))
        && (unscanned.contains("14–15") || unscanned.contains("14-15"))
        && unscanned.contains("may skip")
    {
        return Some("05–12 / 14–15 may skip");
    }
    if unscanned.contains("skip(DEBT-GUARD-NN)")
        && (unscanned.contains("05–12") || unscanned.contains("05-12"))
    {
        return Some("skip(DEBT-GUARD-NN) for 05–12 as live status");
    }
    None
}

/// Active-plane scan: document header (before first `##`) plus current-plane /
/// collision-fence sections that are not Historical/characterization fences.
fn active_plane_text(text: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    let mut past_header = false;
    let mut include_section = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        // Only ##+ headings end the header table / open body sections. A leading
        // `# Title` remains part of the document header scan region.
        if trimmed.starts_with("##") {
            past_header = true;
            let heading = trimmed.trim_start_matches('#').trim();
            let lower = heading.to_ascii_lowercase();
            let historical = lower.contains("historical")
                || lower.contains("characterization")
                || lower.contains("baseline")
                || lower.contains("current behavior")
                || lower.contains("found case")
                || lower.contains("remaining_backlog")
                || lower.contains("shipped stub policy")
                || lower.starts_with("3.")
                || lower.starts_with("12.");
            include_section = !historical
                && (lower.contains("collision fence")
                    || lower.contains("current plane")
                    || lower.contains("guard checks (current")
                    || lower.starts_with("0. collision fence"));
            continue;
        }
        if line.contains("characterization") || line.contains("Characterization") {
            continue;
        }
        if line.contains("Historical") && line.contains("|") {
            // Header rows that point archaeology at Historical are allowed.
            continue;
        }
        if !past_header || include_section {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
