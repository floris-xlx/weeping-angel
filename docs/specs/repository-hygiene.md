# SDD: Test surface, panic budget, schema and repository hygiene

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_repository_hygiene_target` is law. Baseline skip-superseded. **Schema SSOT closed (Phase 3):** only `schemas/codex-security/`; `DEBT-SCHEMA-DUP` resolved; `codex-security/schemas/` must not be reintroduced. C01 later extracted contract-test `require_needles` into `tests/support/mod.rs` (DUP-002); hygiene-owned suites still must not call it. |
| Program | Repository cleanup (concurrent Prompts 1–4) |
| Slice | Prompt 4 — ignored-test retirement, dual-suite collapse (non-colliding), panic budget, Codex Security schema SSOT, generated-artifact policy, `.gitignore` / README hygiene |
| Dual-suite | `sdd_repository_hygiene_baseline` · `sdd_repository_hygiene_target` (`tests/contracts/repository_hygiene.{baseline,target}.rs`) — registered as `[[test]]` in root `Cargo.toml`. **Do not** create `tests/sdd/` ([ADR 0004](../adr/0004-documentation-architecture.md)) |
| ADR | **Accepted** [`docs/adr/0012-repository-hygiene.md`](../adr/0012-repository-hygiene.md). Concurrent Prompts 1–3 drafted `docs/adr/0011-*.md`; this slice did **not** mint another `0011-*` or a `0003-*`. |
| Public contract | Assurance runtime remains [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (untouched). |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) — human SSOT is **this file**. `docs/sdd/` is a stub. Generated traces go to `.sdd/runs/` and `.sdd/artifacts/` only. Before/after counts live in this spec (§3 / §12) and in `.sdd/runs/` after implement — **not** `docs/debt/register.toml`. |
| Neighbors (must stay GREEN after implement) | `sdd_documentation_layout` (additive `CANONICAL_SPECS` row for this path), existing GREEN `sdd_*_target` suites this slice does not edit |
| Collision fence | See §0. Skip files actively owned by Prompts 1–3; report rather than collide. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `0015f6395e7ead042e3cfd3066fefde3d39aa36b` (working tree 2026-08-19; live counts exclude `target/`, `target-*`, `node_modules/`, `.sdd/`) |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) — this slice does not edit IR |
| `adr_needed` | **true** — schema SSOT, generated-artifact policy (extends ADR 0004), production panic budget, dual-suite collapse vs permanent `#[ignore]` |
| Workspace verify (after implement) | `cargo fmt --all -- --check`; `cargo check --workspace --all-targets`; `cargo test --test sdd_documentation_layout`; `cargo test --test sdd_repository_hygiene_baseline`; `cargo test --test sdd_repository_hygiene_target`. CI today is `cargo test --features demo --all-targets` (not `--workspace`). |

This document is the durable human SSOT for Prompt 4. It owns:

- ignored-test inventory and retirement policy
- collapse of **completed, non-colliding** baseline+target dual suites
- source-grep (`require_needles`) hygiene for **hygiene-owned** tests
- production panic budget (scanner `src/**` runtime paths) and independent enforcement
- Codex Security JSON Schema single source of truth
- generated audit / execution artifact policy
- `.gitignore` / admission hygiene
- README and documentation-index slimming
- before/after hygiene metrics (outside the debt register)

It does **not** own the repository guard engine (Prompt 1), catalog/framework/readiness semantics (Prompt 2), or temporal/lineage/evidence/SoA semantics (Prompt 3).

---

## 0. Collision fence (concurrent SDD)

Prompts 1–3 run concurrently. If a file is actively owned, **skip it and continue**. Do not use cross-cutting formatting or mass renames.

| Do not touch | Owner |
| --- | --- |
| `xtask/**`, `architecture/**`, `docs/debt/register.toml` | Prompt 1 |
| `tests/contracts/repository_integrity.{baseline,target}.rs` | Prompt 1 |
| `xtask/tests/sdd_architectural_cleanup_*` | Prompt 1 |
| Canonical catalog / framework / readiness product code (`crates/weeping-angel-canonical-catalog/**`, `crates/weeping-angel-framework/**`, `crates/weeping-angel-assurance/src/readiness.rs`, `catalog/**`, `frameworks/**`) and those **active** `*.target.rs` suites | Prompt 2 |
| Temporal / lineage / evidence / SoA product code (`crates/weeping-angel-assurance/src/{temporal,lineage,soa}.rs`, `crates/weeping-angel-evidence/**`) and those **active** `*.target.rs` suites | Prompt 3 |
| `tests/sdd/` | Forbidden (ADR 0004) |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | Never invent |
| Existing `*target.rs` that define `require_needles` (all 16 files) | Skip rewrite while concurrent — add `tests/support` only for hygiene-owned replacements |
| `docs/adr/0011-*.md` drafts already created by Prompts 1–3 | Skip |

**Allowed this slice**

| Surface | Rule |
| --- | --- |
| `tests/contracts/repository_hygiene.{baseline,target}.rs` | New hygiene dual-suite only |
| `tests/support/**` | Shared helpers that replace source-grep in **hygiene-owned** tests |
| `schemas/**`, `codex-security/schemas/**` | Schema SSOT + generated/packaged second copy |
| Root `.gitignore` | Harden; keep ADR 0004 `.sdd/runs/` + `.sdd/artifacts/` strings |
| `audit.txt` | Untrack; replace with compact manifest or CI artifact policy |
| `README.md`, `docs/contracts/README.md`, `tests/contracts/README.md`, `docs/README.md` | Slim inventories; do not rewrite product specs |
| Root `Cargo.toml` | Additive `[[test]]` rows + optional root `[lints]` for panic budget. Do not mass-reorder existing `sdd_*` rows. |
| `tests/contracts/documentation_layout.rs` | Additive `CANONICAL_SPECS` entry only |
| Production `src/**` | Narrow panic/error-handling only; no assurance-semantic change |
| Tracked `__pycache__/*.pyc` | Untrack (generated) |

---

## 1. Problem / user-visible goal

The assurance program left a large **maintenance tax** that is not product semantics:

1. **661-era ignored tests.** Live tree (this SHA): **632** line-starting `#[ignore…]` attributes (686 substring hits including comments/string mentions). Debt snapshot was 661. **Every** line-starting ignore is a “superseded by …” characterization leftover. Cargo still compiles 38 `*.baseline.rs` binaries that never run in default CI.
2. **Permanent dual-suite scaffolding.** 38 `*.baseline.rs` + 39 `*.target.rs` + `documentation_layout.rs`. 80 root `[[test]]` rows (78 `sdd_*` + `e2e_demo` / `e2e_recon`). Completed targets are already law; ignored baselines remain as compile-time debt.
3. **Brittle source-grep contracts.** 16 `fn require_needles` definitions and **203** `require_needles(` matches (16 defs + 187 call sites), all in existing `*.target.rs` files. Those files are Prompt 2/3-owned during this concurrent run.
4. **Production panic paths.** Root `src/`: **174** `.unwrap()` and **60** `.expect(` (≈88 / 52 outside `#[cfg(test)]`). Most regex compiles are statically closed; several CLI/IO/parser/workbench paths still panic on runtime failure. Workspace crates: **1** `.unwrap()` and **29** `.expect(` under `crates/**/src` (Prompt 2/3 — skip). No Clippy `unwrap_used` / `expect_used`. No `[lints]` table.
5. **Duplicate Codex Security schemas.** Three JSON Schemas exist in two tracked trees (`schemas/codex-security/` and `codex-security/schemas/`). SHA-256 of each pair **matches**. Debt `DEBT-SCHEMA-DUP` (Prompt 1 register — do not edit).
6. **Generated source-of-truth.** Tracked `audit.txt` is raw `xbp audit` output (~1.1 MiB, 18 989 lines, 2 710 findings). 21 tracked `codex-security/scripts/**/__pycache__/*.pyc` files.
7. **`.gitignore` gaps.** Ignores `.env` / `.env.local` / `.env.*.local` but not `.env*`; `/target` but not `target*`; `apps/docs/node_modules/` and `/node_modules`; `.sdd/runs/` + `.sdd/artifacts/` + `/.sdd`. Missing typical `*.pem` / `*.key` / `*.sqlite` / `__pycache__` / `.idea`. Per-worktree `/target-<name>/debug` lines are enumerative rather than `target*/`.
8. **Manually synchronized inventory.** Root `README.md` is already capability-oriented. [`docs/contracts/README.md`](../contracts/README.md) enumerates many dual-suites by hand.

**User-visible goal:** a contributor can clone the repo without compiling hundreds of ignored characterization tests, without treating raw audit logs as source, without two schema homes, and without discovering secrets/build caches as “source.” Panic-prone scanner/runtime failures return typed errors. Hygiene is fail-closed via **this slice’s tests and root lints**, not via Prompt 1’s `xtask`.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `0015f6395e7ead042e3cfd3066fefde3d39aa36b`.

| Surface | Location | Rule |
| --- | --- | --- |
| Dual-suite discovery | root `Cargo.toml` `[[test]]` | `tests/contracts` is **not** auto-discovered (ADR 0004). Register `sdd_repository_hygiene_{baseline,target}` additively. |
| Auto-discovered tests | `tests/*.rs` | Cargo already owns these. `e2e_demo` / `e2e_recon` **must** stay explicit `[[test]]` because they set `required-features = ["demo"]`. |
| Docs layout | `tests/contracts/documentation_layout.rs` | Add `docs/specs/repository-hygiene.md` to `CANONICAL_SPECS` at implement. Keep `.sdd/runs/` and `.sdd/artifacts/` gitignore assertions. |
| CI | `.github/workflows/ci.yml` | Today: `cargo fmt --all -- --check`; `cargo xtask guard`; `cargo clippy --all-targets --features demo -- -D warnings`; `cargo test --features demo --all-targets`. This slice does **not** edit CI (Prompt 1 may). New hygiene `[[test]]` is picked up by `--all-targets`. |
| Debt register | `docs/debt/register.toml` | Prompt 1. Do **not** mark `DEBT-IGNORE` / `DEBT-UNWRAP` / `DEBT-SCHEMA-DUP` resolved here. Record proof in `.sdd/runs/` so Prompt 1 can close later. |
| Architectural-cleanup Phase 23 | [`architectural-cleanup-program.md`](architectural-cleanup-program.md) | “Deleting `#[ignore]` superseded baselines” is a later program phase. This slice performs the **hygiene subset** that does not collide with Prompts 1–3. |
| Schema `$id` | `https://openai.com/codex-security/schemas/*.schema.json` | Keep `$id` strings; SSOT is the file location, not the URL. |
| Fixtures | `tests/fixtures/**`, `codex-security/examples/**`, `schemas/**` examples | Must remain tracked and **not** gitignored. |

Serde / TOML / IR compatibility law: this slice does not migrate product JSON/TOML or fork `assurance-ir/v1`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Executable characterization will live in `sdd_repository_hygiene_baseline` (register at implement). This section is the **historical** contract of SHA `0015f63`. After target GREEN, absence/debt tests are skip-superseded (`#[ignore = "superseded by sdd_repository_hygiene_target"]`) **or** the baseline file is deleted if it has no remaining found-case value.

Inclusion rule for all counts: matching files under the repo **excluding** `target/`, `target-*`, `node_modules/`, `.sdd/`.

### 3.1 Ignored tests

| Metric | Live count | Notes |
| --- | --- | --- |
| Line-starting `#[ignore` | **632** | Authoritative ignore-attr count |
| Substring `#[ignore` | **686** | Includes comments / string assertions (e.g. target suites asserting baseline ignore text) |
| Debt snapshot | 661 | `docs/debt/baseline-2026-08.md` — stale vs live 632 |
| Files with line-starting ignore | **43** | 38 `tests/contracts/*.baseline.rs` + `xtask/tests/sdd_architectural_cleanup_baseline.rs` (11) + 4 target leftovers |
| Non-superseded ignores | **0** | Every line-starting ignore reason matches `superseded by …` |

**Classification (this SHA)**

| Class | Count (approx) | Policy |
| --- | --- | --- |
| Obsolete migration baseline (ignore-superseded dual-suite) | 616 in `tests/contracts/*.baseline.rs` + 11 in xtask baseline | **Remove** the suite (file + `[[test]]`) when the matching target is GREEN **and** the file is not Prompt 1–3 owned. Prompt 1 xtask baseline: **skip**. Prompt 2/3 baselines: **skip and report**. |
| Intentionally characterization-only (still documents a found case) | 5 leftover ignores **inside** `*.target.rs` (`iam_catalog` ×2, `vulnerability_catalog` ×1, `canonical_assurance_catalog` ×1, `evidence_validity_temporal_assurance` ×1) | Prompt 2/3 — **do not delete**. They encode remap/stub supersession. |
| Temporarily blocked | 0 | Do not invent `#[ignore]` to park failures. |
| Still-valuable default-run regression | 0 among ignored tests | Valuable coverage already lives in GREEN `*.target.rs` and `tests/*.rs`. |

Top ignore-reason buckets (line-starting):

- 246 × `superseded by target suite`
- remainder named `superseded by sdd_<suite>_target` (github_collector 30, governance_catalog 28, vulnerability_catalog 23, iso27001_assurance 21, … repository_integrity 12, architectural_cleanup 11)

### 3.2 Dual-suite pair count

| Metric | Count |
| --- | --- |
| `tests/contracts/*.baseline.rs` | 38 |
| `tests/contracts/*.target.rs` | 39 (`compliance_ir.target.rs` has no baseline) |
| `documentation_layout.rs` | 1 (not dual) |
| Root `[[test]]` | 80 |
| `tests/*.rs` auto-discovered | 16 (includes `e2e_demo.rs`, `e2e_recon.rs`) |
| `tests/sdd/` | **absent** (must stay absent) |

Pairs (basename): `applicability_engine`, `assessment_lineage`, `assurance_runtime`, `canonical_assurance_catalog`, `continuity_resilience`, `continuous_assurance_scheduler`, `controlled_documents`, `control_implementation_registry`, `evidence_validity_temporal_assurance`, `github_collector`, `governance_catalog`, `iam_catalog`, `incident_governance`, `infrastructure_catalog`, `interested_parties_obligations`, `internal_audit`, `isms_context`, `isms_events_drift`, `iso27001_assurance`, `iso27001_remap`, `nonconformity_capa`, `operational_soa`, `personnel_security`, `population_runtime`, `remediation_engine`, `repository_integrity`, `residual_risk`, `risk_identification`, `risk_methodology`, `risk_register`, `risk_treatment`, `scope_engine`, `sdlc_catalog`, `security_objectives`, `supplier_risk`, `temporal_assurance`, `typed_evidence`, `vulnerability_catalog`.

### 3.3 `require_needles`

| Metric | Count |
| --- | --- |
| `fn require_needles` | 16 |
| `require_needles(` matches | 203 |

All 16 definitions are in `tests/contracts/*.target.rs`:

`assessment_lineage`, `continuity_resilience`, `control_implementation_registry`, `controlled_documents`, `incident_governance`, `interested_parties_obligations`, `internal_audit`, `iso27001_assurance`, `nonconformity_capa`, `operational_soa`, `population_runtime`, `remediation_engine`, `risk_register`, `supplier_risk`, `temporal_assurance`, `typed_evidence`.

Prompt 4 characterization: `tests/support/` **did not exist**. **Live (C01 / DUP-002):** crate-private `fn require_needles` lives in `tests/support/mod.rs`; the 17 contract binaries `include!` it. Hygiene-owned suites still must not call it. Do not add `tests/support.rs`.

### 3.4 Production unwrap / expect

| Metric | Count |
| --- | --- |
| `src/**` `.unwrap()` | 174 |
| `src/**` `.expect(` | 60 |
| of which outside `#[cfg(test)]` | ≈88 unwrap / 52 expect |
| `crates/**/src` `.unwrap()` | 1 (`weeping-angel-collector`) |
| `crates/**/src` `.expect(` | 29 (mostly Prompt 2/3 assurance/IR/control-test/evidence/scheduler) |
| Clippy `unwrap_used` / `expect_used` | **not configured** |
| Root / workspace `[lints]` | **absent** |

Observed production classes (not exhaustive):

- **Statically closed regex compile** (`Regex::new(literal).unwrap()` / `.expect("… regex")`) in `src/checks/secrets.rs`, `src/depcheck/detect.rs`, engines, templates. Failure only if the literal is invalid — a programmer error.
- **CLI control-flow unwrap** — `src/lib.rs` `args.base.clone().unwrap()` after `args.base.is_none()` is checked (statically closed on that branch).
- **Runtime IO / parse / workbench** — workbench SQLite, sealed-scan JSON, report/remediation paths generally use `Result` already; remaining `unwrap_or` on JSON fields is Option-default, not panic. Hygiene implement must **re-audit** scanner/parser/network/auth/report/workbench for true panicking unwraps on external input and convert those only.
- **In-module `#[cfg(test)]`** — ≈86 unwrap / 8 expect in `src/**`. Leave them.

### 3.5 Schema duplication

| Path | SHA-256 |
| --- | --- |
| `schemas/codex-security/coverage.schema.json` | `7964B132998CA4DCDD19C75F5D92483E1D44CB71462237709B968EC548C10652` |
| `codex-security/schemas/coverage.schema.json` | same |
| `schemas/codex-security/findings.schema.json` | `BD16DFE9A68C9B0485CAD15B4EA3B037B0006DFB76A0549ED65A60AB8B062AC4` |
| `codex-security/schemas/findings.schema.json` | same |
| `schemas/codex-security/scan-manifest.schema.json` | `20D6801775AE1B056D10114C3AF5E07C5EDFEF27468218611411231A95C7C55E` |
| `codex-security/schemas/scan-manifest.schema.json` | same |

Both trees are **tracked**. No generator, no equivalence test. `DEBT-SCHEMA-DUP` is `open` in the Prompt 1 register.

### 3.6 Tracked generated artifacts

| Artifact | State |
| --- | --- |
| `audit.txt` | Tracked. Header: `xbp audit errors — 2,710 findings — 293 files`. Not a schema. |
| `codex-security/scripts/**/__pycache__/*.pyc` | **21** tracked bytecode files |
| `report-lab*`, `scan-mirror3*` | gitignored (`.gitignore` names them) and **not** in `git ls-files` |
| `.sdd/runs/`, `.sdd/artifacts/` | gitignored (ADR 0004) |

### 3.7 `.gitignore` (current)

Present: `/target`, `/report-lab*`, `/report.*`, `/weeping-angel.toml`, `/node_modules`, `apps/docs/node_modules/`, `apps/docs/.next/`, `apps/docs/.source/`, `apps/docs/out/`, `.sdd/runs/`, `.sdd/artifacts/`, `.env`, `.env.local`, `.env.*.local`, named `scan-mirror3*`, some `.xbp/` locals, `/.sdd`, enumerative `/target-<worktree>/debug` lines.

Absent: `.env*` (generic), `target*` / `target-*/`, `__pycache__/`, `*.pyc`, `*.pem`, `*.key`, `*.sqlite` / `*.sqlite3`, `.idea/`, `.vscode/` (optional), `audit.txt`, generic `*.log`.

Does **not** ignore `tests/fixtures/**` or schema examples (correct).

`sdd_documentation_layout::gitignore_excludes_generated_sdd_traces` requires the strings `.sdd/runs/` and `.sdd/artifacts/` to remain.

### 3.8 README / indexes

- Root [`README.md`](../../README.md): capabilities, CLI, install, workspace crate graph, docs map. **Not** a per-suite inventory. Keep that shape.
- [`docs/contracts/README.md`](../contracts/README.md): “Moved” stub that still **hand-lists** many dual-suites (ISMS, risk, audit, …). This is the inventory to slim.
- [`tests/contracts/README.md`](../../tests/contracts/README.md): short dual-suite note (keep, or shorten further).
- [`docs/README.md`](../README.md): path map (keep).

### 3.9 Test discovery

Cargo auto-discovers `tests/*.rs`. `tests/contracts/*.rs` is **not** auto-discovered. Explicit `[[test]]` is required for every contract suite. Redundant explicit rows today: none except `e2e_*` which are **justified** (`required-features`).

---

## 4. Desired behavior (target — RED on characterization SHA)

### 4.1 Ignored-test retirement

- Obsolete ignore-superseded **hygiene-owned** baseline suites are **deleted** (file + `[[test]]` row), not re-ignored.
- Prompt 1 / 2 / 3 owned baselines stay until those owners collapse them. Hygiene target **reports** the skip list; it does not fail the build solely because skipped owners still have ignored baselines.
- No new `#[ignore]` is introduced as a shortcut for a red test.
- The 5 leftover ignores inside Prompt 2/3 `*.target.rs` stay (skip).
- Hygiene-owned default-run ignore count for **this slice’s files** is 0 after supersede (baseline file may keep a single skip-supersede attr if retained as additive documentation; prefer deletion).

### 4.2 Dual-suite collapse

Collapse (delete `*.baseline.rs` + its `[[test]]`, keep `*.target.rs` as the durable contract) **only** when all of:

1. Matching `sdd_*_target` is GREEN on current HEAD.
2. Baseline tests are ignore-superseded characterizations of a **completed** migration (not an open product defect).
3. The files are **not** on the Prompt 1–3 skip list.
4. No skipped owner’s target asserts the baseline file’s presence (Prompt 1 `repository_integrity.target.rs` **does** read its baseline — never delete that pair here).

If no completed pair is safely owned this increment, **do not** delete foreign baselines just to move the metric. The hygiene dual-suite itself follows the full protocol (baseline GREEN → target RED → implement → target GREEN → supersede baseline).

### 4.3 Source-grep policy

- Hygiene-owned tests (`repository_hygiene.*`) **must not** call `require_needles`.
- Prefer public API behavior, typed metadata, serialized schema bytes, compile-time boundaries, or filesystem/hash equality.
- Keep source-grep only when **exact source structure is the invariant**, and comment why (one-line `// source-structure invariant: …`).
- Concurrent Prompt 2/3 skip (this slice): do **not** rewrite the existing `require_needles` target files. **Later non-concurrent C01** moved the helper into `tests/support/mod.rs` for those 17 contract binaries (DUP-002). Needles in those files remain product-surface invariants; uniqueness is inventory + C01 target, not a second hygiene grep.

### 4.4 Panic budget

**Scope:** production runtime in root `src/**` excluding `#[cfg(test)]` modules and the `demo` lab.

| Class | Policy |
| --- | --- |
| External/input/runtime failure (IO, parse of untrusted/user/scan input, network, authz decisions, report emission, workbench DB, CLI argument combinations that clap cannot make impossible) | `Result` / typed error with context. No `.unwrap()` / `.expect()`. |
| Statically closed programmer errors (invalid regex **literals**, `unwrap` after an exhaustive branch, `OnceLock` of constants) | May remain. Mark `// panic-ok: <reason>` (or `#[expect(clippy::unwrap_used)]` if lints are enabled). |
| Test / example / build-script unwraps | Out of budget. Do not churn. |

**Enforcement (must not edit `xtask/`):**

1. Hygiene target walks `src/**/*.rs`, strips `#[cfg(test)]` tails, and fails on unmarked `.unwrap()` / `.expect(` in **budgeted modules** (`parse`, `http`, `authz`, `report`, `workbench`, `cli`/`lib` command entry, `contract` seal/report, `discovery` IO, `depcheck` file parsers that read user manifests). Regex-literal compiles may be allowlisted by `panic-ok` or by matching `Regex::new("…")`.
2. Optional root `[lints.clippy] unwrap_used` / `expect_used` = `"warn"` or `"deny"` **only if** it can be limited so workspace tests and Prompt 2/3 crates do not become a mass-allow festival. Prefer the hygiene test as SSOT if Clippy cannot be scoped without editing `xtask` or foreign crates.
3. Narrow exceptions are explicit in source next to the call.

Do **not** convert crate unwraps owned by Prompts 2/3.

### 4.5 Schema SSOT

- **Authoritative location:** `schemas/codex-security/{coverage,findings,scan-manifest}.schema.json`.
- `codex-security/schemas/` is either **deleted** (callers retargeted to SSOT) or **generated/packaged** from the SSOT (copy step or build script). It is not a second hand-edited source.
- Hygiene target: for each of the three names, if a second path exists it is **byte-identical** to the SSOT (SHA-256). Prefer a comment or tiny stamp file proving generation; do not invent a new crate.
- Update hygiene-owned references only. If a Prompt 2/3 test reads `codex-security/schemas/…` (e.g. `vulnerability_catalog.baseline.rs`), **skip that file**; keep a second path as a generated copy so those tests stay GREEN.

### 4.6 Generated artifacts

- `audit.txt` is **not** source. Untrack it. Gitignore `audit.txt` and `/audit.txt`.
- Replacement: a compact structured manifest **or** a CI-artifact policy documented in this spec / README. Manifest fields if a file is added: `generator`, `schema_version`, `source_commit`, `digest`, `finding_summary` (counts only). Path options: untracked under `.sdd/artifacts/` (preferred) or a tiny tracked `codex-security/audit.manifest.json` **only if** a consumer requires a committed pointer. Do **not** commit 1 MiB logs.
- Untrack `codex-security/scripts/**/__pycache__/**`. Gitignore `__pycache__/` and `*.pyc`.
- Raw scan dumps (`report-lab*`, `scan-mirror3*`) stay ignored.

### 4.7 `.gitignore` target

Must ignore (without hiding fixtures/schemas):

| Pattern | Intent |
| --- | --- |
| `.env`, `.env.*` (and keep existing `.env.local` / `.env.*.local`) | Secrets. Do **not** ignore `.env.example` if one is added later — today none. |
| `node_modules/` | JS deps (root + apps) |
| `/target`, `target/` , `target-*/` | Rust build dirs including worktree isolates (replaces enumerative `/target-foo/debug` lines) |
| `.sdd/runs/`, `.sdd/artifacts/`, `/.sdd` | ADR 0004 (keep exact `runs`/`artifacts` strings) |
| `*.sqlite`, `*.sqlite3` | Local workbench DBs |
| `*.pem`, `*.key` | Private keys/certs |
| `__pycache__/`, `*.pyc` | Python caches |
| `.idea/` | Editor |
| `audit.txt` | Generated xbp output |
| existing `/report-lab*`, `/report.*`, named scan-mirror files | Local scan dumps |

Must **not** ignore: `tests/fixtures/**`, `fixtures/**`, `schemas/**`, `codex-security/examples/**`, `codex-security/schemas/*.schema.json` if that path remains a generated tracked copy.

### 4.8 README / indexes

- Root README stays capability + architecture + canonical commands. It must **not** grow a hand-maintained list of `sdd_*` suites.
- `docs/contracts/README.md` shrinks to a pointer: specs live in `docs/specs/`, invariants in `tests/contracts/`, discovery via root `Cargo.toml` `[[test]]` / `sdd_documentation_layout`. Optional: “generate the inventory with `rg '^name = \"sdd_' Cargo.toml`” — do not paste dozens of rows.
- Do not write generated reports under `docs/sdd/`.

### 4.9 Test discovery

- New hygiene suites: explicit `[[test]]` (required).
- Do not remove `e2e_*` explicit rows.
- Do not rename CI-consumed test binary names without updating consumers in this ownership boundary (none besides Cargo.toml).
- Do not add `[[test]]` for auto-discovered `tests/*.rs` that have no extra metadata.

### 4.10 Hygiene dual-suite semantics

**Baseline (GREEN now):** asserts the **debt exists** — ignore-attr count ≥ 600; ≥ 38 `*.baseline.rs`; `require_needles` definitions == 16; `src/` unwrap+expect ≥ 200 combined; two schema trees exist and are byte-identical; `audit.txt` is tracked; `.gitignore` lacks at least one of `{.env*, target-*/, __pycache__, *.pem, *.sqlite}`; `docs/contracts/README.md` still names multiple `tests/contracts/*.baseline.rs` paths.

**Target (RED now, GREEN after implement):** asserts **fail-closed hygiene** —

- `audit.txt` is not tracked (`git ls-files` empty) and is gitignored.
- Schema SSOT is `schemas/codex-security/`; any second copy is byte-identical and documented as generated.
- `.gitignore` contains hardened patterns listed in §4.7 (and still contains `.sdd/runs/` + `.sdd/artifacts/`).
- `__pycache__` / `*.pyc` are not tracked.
- Hygiene-owned tests contain zero `require_needles`.
- Production budgeted modules have no unmarked panic-on-input unwrap/expect.
- `docs/contracts/README.md` is not a dual-suite inventory (no long list of `sdd_*` / `*.baseline.rs` rows).
- `CANONICAL_SPECS` includes this spec path.
- No new `#[ignore]` without `superseded by sdd_repository_hygiene_target` (and only on the hygiene baseline after supersede).
- Before/after counts written under `.sdd/runs/` (gitignored) **or** updated tables in this spec §12 — not `docs/debt/register.toml`.

---

## 5. Dual-suite protocol (mandatory)

```text
spec first (this file; no product feature code)
  → baseline GREEN on CURRENT code (characterize debt)
  → target RED on CURRENT code (desired hygiene)
  → implement (within ownership)
  → draft ADR 0012 → Accepted at implement
  → target GREEN
  → prove baseline fails or is additive-documented
  → supersede / delete hygiene baseline
  → target still GREEN
```

Prefer deletion and simplification over replacing one scaffold with another. **No new `#[ignore]` as a shortcut.**

Register at implement:

```toml
[[test]]
name = "sdd_repository_hygiene_baseline"
path = "tests/contracts/repository_hygiene.baseline.rs"

[[test]]
name = "sdd_repository_hygiene_target"
path = "tests/contracts/repository_hygiene.target.rs"
```

Add `docs/specs/repository-hygiene.md` to `CANONICAL_SPECS`.

---

## 6. Collision report (pre-implement; update at implement)

| Item | Owner | Action |
| --- | --- | --- |
| `tests/contracts/repository_integrity.*` | Prompt 1 | Skip collapse. Target asserts baseline ignore text. |
| `xtask/tests/sdd_architectural_cleanup_*` | Prompt 1 | Skip. 11 ignored tests stay. |
| `docs/debt/register.toml` (`DEBT-IGNORE`, `DEBT-UNWRAP`, `DEBT-SCHEMA-DUP`) | Prompt 1 | Skip. Proof goes to `.sdd/runs/`. |
| All 16 `require_needles` `*.target.rs` | Prompts 2/3 (semantic targets) | Skip rewrite. |
| Catalog/framework/readiness `*.{baseline,target}.rs` | Prompt 2 | Skip collapse. |
| `temporal_assurance`, `evidence_validity_temporal_assurance`, `assessment_lineage`, `operational_soa`, `continuous_assurance_scheduler` | Prompt 3 | Skip collapse. |
| 5 ignored tests inside Prompt 2/3 targets | Prompts 2/3 | Skip. |
| `docs/adr/0011-*.md`, `docs/specs/catalog-framework-readiness-trust-boundary.md`, `docs/specs/temporal-lineage-evidence-soa.md` | Prompts 1–3 drafts | Skip. Hygiene ADR is **0012**. |
| `crates/weeping-angel-assurance/**`, `*-evidence/**`, `*-canonical-catalog/**`, `*-framework/**` unwraps | Prompts 2/3 | Skip panic conversion. |

**Collapse candidates — skipped at implement.** Nearly every GREEN `*.target.rs` (including the operational-ISMS pair listed below) still **asserts its `*.baseline.rs` path in `Cargo.toml`**. Deleting those files would fail Prompt 2/3 targets. Hygiene does not edit those targets.

Skipped collapse (target asserts baseline presence): `controlled_documents`, `continuity_resilience`, `control_implementation_registry`, `incident_governance`, `interested_parties_obligations`, `internal_audit`, `isms_context`, `isms_events_drift`, `nonconformity_capa`, `personnel_security`, `remediation_engine`, `residual_risk`, `risk_identification`, `risk_methodology`, `risk_register`, `risk_treatment`, `scope_engine`, `security_objectives`, `supplier_risk`.

`assurance_runtime` remains the public-contract neighbor — not collapsed. No hygiene-owned ignored baseline existed to delete besides this slice’s own characterization suite, which is skip-superseded (`#[ignore = "superseded by sdd_repository_hygiene_target"]`), not left as failing CI.

---

## 7. Implement surfaces (landed)

| Concern | Home | Landed |
| --- | --- | --- |
| Hygiene dual-suite | `tests/contracts/repository_hygiene.{baseline,target}.rs` + root `[[test]]` | Yes. Target GREEN; baseline skip-superseded. |
| Shared test helpers | `tests/support/` | Hygiene did **not** add this directory. C01 later placed crate-private `require_needles` here for contract binaries (DUP-002). Hygiene suites still do not call it. |
| Schema SSOT | keep `schemas/codex-security/`; generate or retarget `codex-security/schemas/` | SSOT kept. Second tree stamped `codex-security/schemas/GENERATED_FROM_SSOT`. |
| Equivalence test | hygiene target (SHA-256) | Yes. |
| Panic conversion | narrow `src/**` Result paths | `scan-diff` missing `--base`; Pipfile `split_once`. Regex literals marked `panic-ok`. |
| Panic budget gate | hygiene target + optional root `[lints]` | Hygiene target only. No workspace `unwrap_used` deny. |
| Untrack generated | `git rm --cached audit.txt` and `*.pyc`; gitignore | Yes. |
| `.gitignore` | root file | Hardened (`.env.*`, `target-*/`, `__pycache__/`, keys, sqlite, `audit.txt`). |
| Slim index | `docs/contracts/README.md` | Pointer, not a dual-suite inventory. |
| Spec index | `documentation_layout.rs` `CANONICAL_SPECS` | `docs/specs/repository-hygiene.md` listed. |
| Decision | [ADR 0012](../adr/0012-repository-hygiene.md) Accepted | Yes. |
| Metrics | this spec §12 + `.sdd/runs/repository-hygiene-counts.md` | §12 filled. Debt register untouched. |

---

## 8. Out of scope

- Prompt 1 guard engine, architecture TOML, debt register, Guards 14–15.
- Prompt 2 catalog SSOT / framework digest / readiness projection.
- Prompt 3 temporal / lineage / evidence `current` vs `latest` / SoA semantics.
- Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
- Creating `tests/sdd/`.
- Mass rustfmt-only or rename-only churn across foreign files.
- Mechanical replacement of every test unwrap / every statically closed regex unwrap.
- Closing `DEBT-*` rows in `docs/debt/register.toml`.
- Changing CI commands (unless a hygiene-owned file must; prefer not).
- Writing generated reports under `docs/sdd/`.
- ADR mass-renumber (`DEBT-DUP-ADR`).

---

## 9. Risks

- **Concurrent overwrite.** Deleting a baseline that Prompt 2/3 still treat as characterization, or that Prompt 1 asserts, will fail their targets. Mitigation: skip list in §6; delete only hygiene-owned files.
- **Schema path drift.** Callers and Prompt 2 tests that hard-code `codex-security/schemas/` break if the second tree is removed without a generated copy. Mitigation: keep generated byte-identical copy until those tests move.
- **Clippy `unwrap_used` deny.** Workspace `--all-targets` would explode on tests. Mitigation: hygiene test as SSOT; do not deny workspace-wide without a scoped allow.
- **False-green ignore metric.** Deleting coverage to lower 632 is forbidden. Collapse only superseded characterizations whose target remains.
- **`git rm --cached audit.txt`.** Local developers may still generate it; gitignore prevents re-add.
- **Worktree `target-*` ignore.** Must not ignore source dirs named `target` under fixtures (none today). Use `target/` and `target-*/` not a bare `target` glob over `tests/`.
- **ADR number race.** 0011 is already claimed by concurrent drafts. 0012 avoids a fourth `0011-*`.
- **documentation_layout.** Forgetting the `CANONICAL_SPECS` add is not a fail of that suite (it only checks listed files exist). Implement **must** add the row so the spec is indexed.

---

## 10. Acceptance criteria

- Obsolete ignored **hygiene-owned** baseline suites are removed, not merely left ignored.
- Valuable characterization / GREEN target coverage remains (including all skipped Prompt 1–3 suites).
- Hygiene-owned tests do not use `require_needles`. Contract binaries share one helper in `tests/support/mod.rs` (C01 / DUP-002); that is not hygiene-owned surface.
- Panic-prone production paths in budgeted `src/**` modules return typed errors; remaining unwraps are `panic-ok` or test-only.
- Codex Security schemas have one SSOT; any second copy is generated and byte-equivalent (tested).
- Raw generated audit/execution artifacts (`audit.txt`, `*.pyc`) are not hand-maintained source.
- `.gitignore` prevents recurrence of `.env*`, `node_modules`, `target*`, SDD runtime state, local DBs, keys, editor caches, raw audit output — without hiding fixtures or schema examples.
- README is not a manually synchronized test inventory; `docs/contracts/README.md` is a pointer.
- Before/after counts recorded in this spec and/or `.sdd/runs/` — not `docs/debt/register.toml`.
- `cargo fmt --all -- --check` passes.
- `cargo check --workspace --all-targets` passes.
- `cargo test --test sdd_documentation_layout` stays GREEN.
- `cargo test --test sdd_repository_hygiene_target` passes after implement; baseline is GREEN before implement and superseded or deleted after.
- No new `#[ignore]` as a shortcut.

---

## 11. Verify commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --test sdd_documentation_layout
cargo test --test sdd_repository_hygiene_target
# baseline is skip-superseded; found-case fails if run with --ignored
cargo test --test sdd_repository_hygiene_baseline
```

Skip files owned by Prompts 1–3 if they change mid-run.

---

## 12. Before / after counts

Record live numbers here at spec-first. After column filled at implement. Snapshot also written to `.sdd/runs/repository-hygiene-counts.md` (gitignored).

| Metric | Before (SHA `0015f63`) | After (implement) |
| --- | --- | --- |
| Line-starting `#[ignore` (excl. target/, target-*, node_modules, .sdd) | 632 | 656 (Prompt 3 added suites; hygiene baseline skip-supersede +16; **no GREEN coverage deleted**) |
| Substring `#[ignore` | 686 | 710 |
| `tests/contracts/*.baseline.rs` | 38 | 40 (hygiene + Prompt 3 `temporal_lineage_evidence_soa`; **0 collapses** — targets still assert baselines) |
| `tests/contracts/*.target.rs` | 39 | 41 (same two additions) |
| Root `[[test]]` | 80 | 84 (hygiene pair + Prompt 3 pair; no collapsed rows) |
| `fn require_needles` | 16 | 16 (Prompt 2/3 files skipped). **Live after C01:** 1 (`tests/support/mod.rs`; see `docs/debt/current.md`) |
| `require_needles(` matches | 203 | 203 (hygiene files added 0). **Live after C01:** 206 |
| `src/**` `.unwrap()` | 174 | 172 (`lib.rs` scan-diff base + Pipfile `split_once` converted) |
| `src/**` `.expect(` | 60 | 60 (regex-literal continuation lines marked `panic-ok`) |
| `src/**` unwrap/expect outside `#[cfg(test)]` | ≈88 / 52 | 86 / 52; **unmarked budgeted-module panics = 0** |
| Codex Security schema trees | 2 × 3 files, SHA-identical | **1 SSOT only** — `schemas/codex-security/` (3 files); `codex-security/schemas/` deleted; Guard 03 + hygiene forbid reintroduction (`DEBT-SCHEMA-DUP` resolved) |
| Tracked `audit.txt` | 1 | 0 (untracked + gitignored) |
| Tracked `*.pyc` | 21 | 0 (untracked + gitignored) |

Do not optimize a metric at the expense of coverage.
