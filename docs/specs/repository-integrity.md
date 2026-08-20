# SDD: Repository Integrity & Technical-Debt Reconciliation

| Field | Value |
| --- | --- |
| Status | **Increment 1 implemented.** **Increment 2 implemented** — modular guard engine, architecture policy SSOT, real Guards 14/15, fail-closed debt expiry. |
| Program | Repository Integrity & Technical-Debt Reconciliation |
| Slice | **Increment 2** — Repository Guard and Governance Hardening (Prompt 1). Extends increment 1 (§0–§9) rather than forking a second SSOT. |
| Dual-suite | `sdd_repository_integrity_baseline` · `sdd_repository_integrity_target` (`tests/contracts/repository_integrity.{baseline,target}.rs`) — **not** auto-discovered; listed as `[[test]]` in root `Cargo.toml`. Increment-1 absence tests RI-B01–B10 and increment-2 characterization RI-B11–B18 stay `#[ignore]`-superseded (do **not** re-enable). Desired increment-2 behavior is additive target IDs RI-T18–T31. Architectural-cleanup dual-suite remains `xtask/tests/sdd_architectural_cleanup_{baseline,target}.rs` (`cargo test -p xtask`). `tests/sdd/` is forbidden ([ADR 0004](../adr/0004-documentation-architecture.md)). |
| ADR | Increment 1: **Accepted** [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md), **Accepted** [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md). Increment 2: **Accepted** [`docs/adr/0011-repository-guard-governance.md`](../adr/0011-repository-guard-governance.md). Duplicate `0003-*` / `0005-*` / `0007-*` / `0008-*` (and pinned concurrent `0011-*`) IDs remain grandfathered debt (`DEBT-DUP-ADR`); do **not** silently renumber. Do **not** rewrite 0009/0010 decision bodies. |
| Public contract | This file. Assurance runtime public contract remains [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (untouched). |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — human SSOT is this file under `docs/specs/`. `docs/sdd/repository-integrity.md` is a pointer stub only. Generated traces go to `.sdd/runs/` and `.sdd/artifacts/` only. |
| Neighbors (must stay GREEN after implement) | `sdd_documentation_layout` (this path already in `CANONICAL_SPECS`), `sdd_assurance_runtime_target`, `sdd_canonical_assurance_catalog_target`, `sdd_iso27001_assurance_target`; ACP target suite under `xtask/tests` after its increment-2 assertion updates |
| Collision fence | Prompt 1 exclusive surfaces only (see [§10](#10-increment-2-collision-fence)). Guards **01–15** are real `ArchitectureCheck`s on the healthy tree (`ProductLawCheck` for **05–12**). Prompt 4 owns panic budget, broad test retirement, schemas, README/audit hygiene. Do not invent `weeping-angel-catalog` or `weeping-angel-assurance-cli`. Stub/skip archaeology lives under **Historical**. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Increment-1 characterization SHA | `f560196c57e77df2573cfb9a4b384d3cf1c21e8a` |
| Increment-2 current plane | Modular `xtask` (`model` / `architecture` / `debt` / `checks` / `report`); Guards **01–15** real / pass on the healthy tree; `DEBT-GUARD-05`…`12` (and 14/15) **resolved** |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) — this program does not edit IR |
| `adr_needed` | **true** (increment 1: 0009/0010). Increment 2: **Accepted** ADR **0011** (modular guard engine, policy-in-`architecture/`, real Guards 14/15, debt-expiry, additive JSON). |
| Workspace verify | `cargo fmt --all -- --check`; `cargo test -p xtask`; `cargo xtask guard`; `cargo test --test sdd_repository_integrity_target`; `cargo test --test sdd_documentation_layout` |

This document is the durable human SSOT for the Repository Integrity program. Increment 1 owns **architecture manifests**, **canonical concept ownership**, **forbidden-pattern declarations**, **the technical-debt register**, **the 2026-08 live baseline snapshot**, **`cargo xtask guard` as the single repository health command**, and **mandatory CI wiring**. Increment 2 (this revision) owns turning that gate into a **durable architectural enforcement system**: modular guard engine, single-load `RepositoryModel` with cached source, versioned policy under `architecture/`, real ADR identity/graph (Guard 14), real spec lifecycle (Guard 15), and fail-closed debt expiry.

It does **not** own P0 product remediations owned by Prompts 2–4 (framework expression preservation, catalog SSOT migration, lineage rebuild, SoA, persistence, package-install tests, ADR *mass-renumber*, deleting obsolete baseline suites, a test-support crate, panic budget, schema fixtures). Increment-1 remainder is [§8](#8-remaining_backlog-out-of-scope-for-this-slice); increment-2 remainder is [§17](#17-increment-2-out-of-scope).

Architecture law (frozen, unchanged):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

This slice adds a **repository-level** law beside that graph:

```text
architecture.toml (ownership)
  + invariants.toml (declared this slice; evaluated as check 04 by ADR 0010)
  + forbidden-patterns.toml
  + docs/debt/register.toml
        ↓
cargo xtask guard   (fail-closed; remaining stubs are not silent passes)
        ↓
CI must run it
```

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

This slice may add **only** the health-gate surfaces listed in §4. It must not rewrite assurance engines, catalog TOML, framework packs, IR types, or scanner CLI behavior.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap — remaining_backlog |
| `frameworks/**` pack parsing, digest algorithm, expression preservation | Framework crate — remaining_backlog |
| `crates/weeping-angel-assurance/src/{readiness,lineage,soa,temporal}.rs` semantics | Assurance facade — remaining_backlog |
| `crates/weeping-angel-evidence` ledger latest-vs-current / persistence invariants | Evidence crate — remaining_backlog |
| `crates/weeping-angel-control-test` `select_latest_as_of` rewrite | Control-test — remaining_backlog (ownership *declared* here; behavior not changed) |
| Scanner `src/finding.rs`, recon engines, Codex Security seal | Root product |
| Existing dual-suite bodies except additive `Cargo.toml` `[[test]]` rows and `CANONICAL_SPECS` | Neighbors stay GREEN |
| `tests/sdd/` | ADR 0004 forbids this path |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | **Do not invent** |
| Guard checks 05–12 product semantics (historical row; now landed) | Prompts 2/3 owned product semantics. Check **04** is [ADR 0010](../adr/0010-architecture-as-law.md). Checks **14–15** are increment 2 ([ADR 0011](../adr/0011-repository-guard-governance.md)). Live tree: **05–12** are `ProductLawCheck` and pass. |

Suggested **implement** surfaces (new files only, plus tiny wiring):

| Concern | Home |
| --- | --- |
| Concept ownership + schema version | `architecture/architecture.toml` |
| Declared invariants (file present this slice; evaluation later) | `architecture/invariants.toml` |
| Forbidden patterns catalog | `architecture/forbidden-patterns.toml` |
| Debt register + README | `docs/debt/register.toml`, `docs/debt/README.md` |
| Live counts snapshot | `docs/debt/baseline-2026-08.md` |
| Authoritative health command | workspace member `xtask/` (`cargo xtask guard`) |
| Alias | `.cargo/config.toml` `[alias] xtask = "run --package xtask --"` |
| CI gate | `.github/workflows/ci.yml` mandatory `cargo xtask guard` step |
| Dual-suite | `tests/contracts/repository_integrity.{baseline,target}.rs` + root `[[test]]` |
| Spec index | `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` |

Increment 1 landed those surfaces. Do **not** change `edition`, IR schema, catalog IDs, or CI's existing fmt/clippy/test commands except the mandatory guard step (do not remove fmt/clippy/test).

---

## 1. Problem / user-visible goal

The assurance program now spans seven workspace crates plus a root CLI, dozens of dual-suite contracts, duplicated ADR numbers, catalog TOML, and two framework packs — with **no executable ownership map and no fail-closed repository health command**.

On characterization SHA `f560196c57e77df2573cfb9a4b384d3cf1c21e8a`:

- There is **no** `architecture/` directory, **no** `architecture.toml` / `invariants.toml` / `forbidden-patterns.toml`.
- There is **no** `docs/debt/` tree, register, or live baseline snapshot.
- There is **no** `xtask` crate, **no** `.cargo/config.toml`, and **no** string `xtask` anywhere in `Cargo.toml`, workflows, or docs.
- `cargo xtask` and `cargo run -p xtask` fail (unknown alias / unknown package).
- GitHub Actions [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo fmt --all -- --check`, `cargo clippy --all-targets --features demo -- -D warnings`, and `cargo test --features demo --all-targets`. It does **not** use `--workspace` and does **not** run `cargo xtask guard`.
- Concept ownership is tribal: catalog lives in `weeping-angel-canonical-catalog`, compilation in `weeping-angel-framework`, readiness/lineage/temporal projection in `weeping-angel-assurance`, persistence in `weeping-angel-evidence`, CLI in root `src/main.rs` + `src/cli.rs`. Nothing machine-checks that mapping. Hypothetical names `weeping-angel-catalog` / `weeping-angel-assurance-cli` **do not exist**.
- ADR files reuse IDs: 25 files are `0003-*`; also `0005-*` (5), `0007-*` (2), `0008-*` (4). A later “P0 uniqueness rewrite” cannot land without a gate that would otherwise miss it.
- Later P0 remediations (pack parsing, digest, lineage, SoA, persistence) have nowhere to record debt with **proof-of-resolution**. A finding could be marked resolved with no regression test.

**User-visible goal:** a contributor (or CI) can run **one** command and know whether the repository’s architecture manifests, ownership table, forbidden-pattern file, and debt register are present, parseable, and honest:

```text
cargo xtask guard
  → 01 architecture.toml present + parseable
  → 02 canonical ownership table present (live crate names only)
  → 03 forbidden-patterns.toml present
  → 13 debt register schema + unique finding IDs
       + status=resolved requires regression_tests or repository_guard
  → 04 evaluates invariants (ADR 0010); increment-1 stubs 05–12 / 14–15 fail closed or skip only with a registered debt finding
  → non-zero exit if any implemented check fails
```

Increment 2 ([ADR 0011](../adr/0011-repository-guard-governance.md)) keeps that command and makes **14–15** real; **05–12** stay skip-with-debt. Definition of done for increment 1: *later P0 remediations cannot land without regression controls, because resolved debt without proof is rejected and CI cannot merge without the guard.*

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `f560196c57e77df2573cfb9a4b384d3cf1c21e8a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Workspace members | root [`Cargo.toml`](../../Cargo.toml) | Keep the seven crates. **Add** `xtask` only. Do not rename members. |
| Root package | `name = "weeping-angel"`, `src/main.rs` | Stays the scanner + `assurance` clap family. Not a new CLI crate. |
| `AssuranceCommand` | [`src/cli.rs`](../../src/cli.rs) | Ownership concept `assurance_cli` maps here + `src/main.rs`. Do not extract a `weeping-angel-assurance-cli` crate. |
| Canonical catalog crate | `crates/weeping-angel-canonical-catalog` | Live owner of concept `catalog`. Package name is **not** `weeping-angel-catalog`. |
| Framework crate | `crates/weeping-angel-framework` | Live owner of `framework_compilation`. |
| Assurance facade | `crates/weeping-angel-assurance` | Live owner of `readiness_projection`, `temporal_evidence_selection`, `assessment_lineage`. |
| Evidence crate | `crates/weeping-angel-evidence` | Live owner of `evidence_persistence`. |
| Dual-suite discovery | root `Cargo.toml` `[[test]]` | Dual-suites are **not** auto-discovered. Register `sdd_repository_integrity_{baseline,target}` the same way as existing `sdd_*` rows. |
| Docs layout | `tests/contracts/documentation_layout.rs` | Human spec path must be added to `CANONICAL_SPECS` at implement so `sdd_documentation_layout` stays GREEN *and* lists this SSOT. |
| CI job | `.github/workflows/ci.yml` job `test` | **Add** a mandatory `cargo xtask guard` step. Keep existing fmt / clippy / test / demo-example steps. |
| ADR numbering | `docs/adr/` | Next unused unique number is **0009**. Cite by path. Do not add `0003-repository-integrity.md`. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Untouched. |

Serde / TOML compatibility law:

- Architecture and debt files use explicit `schema` strings (see §4). Unknown required fields fail closed.
- Existing product JSON/TOML (catalog, packs, IR fixtures) is not migrated in this slice.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Executable characterization lives in `sdd_repository_integrity_baseline` (to be registered at implement). This section is the **historical** contract of SHA `f560196c`. After target GREEN the absence tests are skip-superseded (`#[ignore = "superseded by sdd_repository_integrity_target"]`).

### 3.1 No architecture manifests

Relative to repo root, these paths **do not exist**:

```text
architecture/
architecture/architecture.toml
architecture/invariants.toml
architecture/forbidden-patterns.toml
```

Ripgrep over `Cargo.toml`, `*.yml`, `*.md`, `*.rs`, `*.toml` finds **no** `architecture.toml` / `xtask` / `docs/debt` references.

### 3.2 No debt register

```text
docs/debt/
docs/debt/register.toml
docs/debt/README.md
docs/debt/baseline-2026-08.md
```

are absent. There is no machine schema for finding `id` / `status`, and nothing rejects `status = "resolved"` without proof.

### 3.3 No xtask member or cargo alias

Workspace members (exact, current):

```text
crates/weeping-angel-assurance-ir
crates/weeping-angel-framework
crates/weeping-angel-evidence
crates/weeping-angel-collector
crates/weeping-angel-control-test
crates/weeping-angel-assurance
crates/weeping-angel-canonical-catalog
```

- `.cargo/` (and `.cargo/config.toml`) **do not exist**.
- `cargo xtask` fails (no `[alias] xtask`).
- `cargo run -p xtask` fails (no package `xtask`).
- `cargo test -p xtask` fails for the same reason.

### 3.4 CI does not run a repository health gate

[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml):

```text
cargo fmt --all -- --check
cargo clippy --all-targets --features demo -- -D warnings
cargo test --features demo --all-targets
cargo build --example weeping-angel-demo --features demo
```

The workflow file does **not** contain the substring `xtask guard`. Clippy/test are **not** `--workspace`.

### 3.5 Live crate map (do not invent names)

| Concept (to be owned) | Live home today | Does **not** exist |
| --- | --- | --- |
| `catalog` | package `weeping-angel-canonical-catalog` (`crates/weeping-angel-canonical-catalog`) | `weeping-angel-catalog` |
| `framework_compilation` | package `weeping-angel-framework` | — |
| `readiness_projection` | `weeping-angel-assurance::readiness` (`src/readiness.rs`) | — |
| `temporal_evidence_selection` | `weeping-angel-assurance::temporal` (`src/temporal.rs`); primitives also in `weeping-angel-control-test::temporal::select_latest_as_of` | ownership **declared** on assurance this slice; do not move code |
| `assessment_lineage` | `weeping-angel-assurance::lineage` | — |
| `evidence_persistence` | `weeping-angel-evidence` (`src/ledger.rs`) | — |
| `assurance_cli` | root package `weeping-angel`: [`src/main.rs`](../../src/main.rs) + [`src/cli.rs`](../../src/cli.rs) (`Commands::Assurance` / `AssuranceCommand`) | `weeping-angel-assurance-cli` |

### 3.6 Live counts snapshot (spec-first measurement; implement must re-record into `docs/debt/baseline-2026-08.md`)

Measured on SHA `f560196c` (Windows, exclude `target/` and `node_modules/`):

| Metric | Count / status | Notes |
| --- | --- | --- |
| Root `[[test]]` binaries | **78** | Dual-suites + `e2e_demo` + `e2e_recon`. Not auto-discovered for `tests/contracts/`. |
| `tests/*.rs` (root of `tests/`, auto-discovered) | **16** | Includes `e2e_demo.rs`, `e2e_recon.rs`, `contract_spine.rs`, … |
| `tests/contracts/*.rs` | **76** | 37 `*.baseline.rs`, 38 `*.target.rs`, 1 `documentation_layout.rs` |
| `#[ignore` attributes in `*.rs` | **659** | Mostly superseded baseline suites |
| `.unwrap()` in `*.rs` | **1726** | Includes tests |
| `.expect(` in `*.rs` | **776** | Includes tests |
| unwrap+expect | **2502** | Combined |
| Source-grep contract helpers | **16** files define `fn require_needles`; **203** `require_needles(` occurrences | Dual-suite needle greps, not a dedicated `source-grep` crate |
| ADR markdown files | **40** | `docs/adr/*.md` |
| ADR ID prefixes | `0001`(1), `0002`(1), `0003`(25), `0004`(1), `0005`(5), `0006`(1), `0007`(2), `0008`(4) | Next unused unique number: **0009** |
| Duplicate ADR ID prefixes | **4** prefixes (`0003`,`0005`,`0007`,`0008`); **36** files under those prefixes | Capture as debt; do **not** renumber this slice |
| Catalog test TOML | **13** | `catalog/canonical/v1/tests/*.toml` (same count for controls/ and evidence/) |
| Framework packs | **2** | `frameworks/iso-27001/2022`, `frameworks/wa-baseline/1` (manifest.toml) |
| `*.schema.json` files | **6** | 3 under `schemas/codex-security/` duplicated in `codex-security/schemas/` |
| In-crate schema string constants | `ASSURANCE_IR_SCHEMA`, `CATALOG_SCHEMA`, `FRAMEWORK_PACK_SCHEMA`, `EVIDENCE_SCHEMA`, `EVIDENCE_VALUE_SCHEMA`, `EVIDENCE_VALIDITY_SCHEMA`, `LINEAGE_SNAPSHOT_SCHEMA`, others | Not JSON Schema fixtures — remaining_backlog |
| `cargo fmt --all -- --check` | **pass** (exit 0) | Runnable at characterization |
| `cargo check --workspace --offline` | **pass** (exit 0, ~17s) | Runnable at characterization |
| `cargo test` / `cargo clippy` full workspace | **not re-run in spec-first** | CI currently uses `--features demo --all-targets` (not `--workspace`). Implement must record live results in `docs/debt/baseline-2026-08.md` if runnable. |

### 3.7 Baseline suite obligations (must be GREEN on CURRENT absences)

The baseline binary encodes the found case, not the desired gate. After implement it is `#[ignore]`-superseded; until then it must pass on SHA `f560196c` *and* on the tree immediately before product files exist.

| ID | Assertion on CURRENT tree |
| --- | --- |
| RI-B01 | `architecture/architecture.toml` is not a file |
| RI-B02 | `architecture/invariants.toml` is not a file |
| RI-B03 | `architecture/forbidden-patterns.toml` is not a file |
| RI-B04 | `docs/debt/register.toml` is not a file |
| RI-B05 | Root `Cargo.toml` `[workspace].members` does not contain `xtask` |
| RI-B06 | `.cargo/config.toml` is absent **or** does not define alias `xtask` |
| RI-B07 | `cargo xtask guard` fails (nonzero / command not found) |
| RI-B08 | `cargo run -p xtask -- guard` fails (unknown package) |
| RI-B09 | `.github/workflows/ci.yml` does not contain the substring `xtask guard` |
| RI-B10 | No package named `weeping-angel-catalog` or `weeping-angel-assurance-cli` exists (this remains true after implement) |

---

## 4. Desired behavior (target — RED on current tree, GREEN after implement)

### 4.1 Architecture manifests

Create:

```text
architecture/architecture.toml
architecture/invariants.toml
architecture/forbidden-patterns.toml
```

#### 4.1.1 `architecture/architecture.toml` (check 01 + 02)

Must be UTF-8 TOML, parseable by the xtask TOML parser used in production (not a bespoke half-parser).

Required top-level:

```toml
schema = "weeping-angel/architecture/v1"
```

Required table `ownership` (canonical concept ownership). Every key below is **mandatory**. Values MUST name live packages/paths. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli` is a check-02 failure.

| Concept key | `crate` (package name) | Canonical paths (non-empty) |
| --- | --- | --- |
| `catalog` | `weeping-angel-canonical-catalog` | `crates/weeping-angel-canonical-catalog` |
| `framework_compilation` | `weeping-angel-framework` | `crates/weeping-angel-framework` |
| `readiness_projection` | `weeping-angel-assurance` | include `crates/weeping-angel-assurance/src/readiness.rs` |
| `temporal_evidence_selection` | `weeping-angel-assurance` | include `crates/weeping-angel-assurance/src/temporal.rs` |
| `assessment_lineage` | `weeping-angel-assurance` | include `crates/weeping-angel-assurance/src/lineage.rs` |
| `evidence_persistence` | `weeping-angel-evidence` | `crates/weeping-angel-evidence` |
| `assurance_cli` | `weeping-angel` | `src/main.rs` **and** `src/cli.rs` |

Each ownership entry SHALL include at least:

- `crate` — Cargo package name as in that crate’s `Cargo.toml` / root package name
- `paths` — array of repo-relative directories or files that exist on disk

Check **01**: file present + TOML parse + `schema == "weeping-angel/architecture/v1"`.

Check **02**: all seven concept keys present; `crate` matches the table above; listed `paths` exist; no extra required concept may bind to a non-existent package.

#### 4.1.2 `architecture/invariants.toml`

Must exist and be parseable TOML with:

```toml
schema = "weeping-angel/architecture-invariants/v1"
```

It MAY list named invariants (IDs, prose, optional `guard_check` references) for later slices. **Evaluating** those invariants is check **04** (stub in this health-gate slice; implemented by [ADR 0010](../adr/0010-architecture-as-law.md)). Presence/parse is **not** silently “all invariants hold.”

#### 4.1.3 `architecture/forbidden-patterns.toml` (check 03)

Must exist and be parseable TOML with:

```toml
schema = "weeping-angel/forbidden-patterns/v1"
```

This increment requires the **file** and a parseable schema. Executing `kind` against the tree is [ADR 0010](../adr/0010-architecture-as-law.md) check **03** (not a new grep crate). The file SHOULD declare at least the hypothetical crate names as forbidden patterns so enforcement has a seed:

- package name `weeping-angel-catalog`
- package name `weeping-angel-assurance-cli`
- path `tests/sdd/` (ADR 0004)

Check **03**: file present (parseable TOML with the schema string).

### 4.2 Debt register (check 13)

Create:

```text
docs/debt/README.md
docs/debt/register.toml
docs/debt/baseline-2026-08.md
```

#### 4.2.1 Register schema

```toml
schema = "weeping-angel/debt-register/v1"

[[finding]]
id = "DEBT-0001"          # required, unique, non-empty
title = "…"               # required
status = "open"           # required: open|confirmed|in-progress|resolved|rejected|superseded
summary = "…"             # required
# optional:
# owner = "…"
# source = "…"
# regression_tests = ["sdd_…", "xtask::…"]
# repository_guard = "13"  # or check id / true
```

**Required fields per finding:** `id`, `title`, `status`, `summary`.

**Status enum (closed set):** `open` | `confirmed` | `in-progress` | `resolved` | `rejected` | `superseded`.

**Resolution proof law:** `status = "resolved"` is **illegal** unless the finding lists a non-empty `regression_tests` array **or** a non-empty `repository_guard` value (string check id or boolean `true` meaning “the guard itself covers this”). Check 13 must reject resolved-without-proof. `rejected` and `superseded` do **not** require proof arrays.

**Unique IDs:** duplicate `finding.id` values fail check 13.

`docs/debt/README.md` explains the status machine and the proof law. It is not a second register.

#### 4.2.2 Seed findings (this increment)

The register MUST contain enough rows that stubbed guard checks can skip **only** by citing a real id. Suggested seed (implement may add more; ids must stay unique):

| id | status | Purpose |
| --- | --- | --- |
| `DEBT-DUP-ADR` | `confirmed` | Duplicate ADR ID prefixes (`0003`×25, `0005`×5, `0007`×2, `0008`×4) |
| `DEBT-UNWRAP` | `open` | High `.unwrap()` / `.expect(` volume in `*.rs` |
| `DEBT-IGNORE` | `open` | Large `#[ignore]` baseline surface |
| `DEBT-SCHEMA-DUP` | `open` | Duplicated Codex Security JSON schemas (`schemas/codex-security` vs `codex-security/schemas`) |
| `DEBT-GUARD-04` … `DEBT-GUARD-12`, `DEBT-GUARD-14`, `DEBT-GUARD-15` | `open` or `confirmed` | One finding per stubbed check so a skip is attributable |

P0 remediations listed in §8 MAY be entered as `open`/`confirmed` findings **without** marking them `resolved`.

#### 4.2.3 `docs/debt/baseline-2026-08.md`

Human snapshot of live counts from §3.6, re-measured at implement. Must include: test binaries, ignored tests, unwrap/expect, source-grep contract tests, ADR IDs, duplicate ADR IDs, catalog tests, framework packs, schemas, and `cargo fmt` / `check` / `test` / `clippy` workspace status **if runnable**. This file is evidence, not the register.

### 4.3 `cargo xtask guard` (authoritative health command)

Introduce workspace member **`xtask`** (directory `xtask/`, package name `xtask`, `publish = false`).

`.cargo/config.toml`:

```toml
[alias]
xtask = "run --package xtask --"
```

`cargo xtask guard` MUST:

1. Discover repo root (xtask `CARGO_MANIFEST_DIR` parent).
2. Run implemented checks **01, 02, 03, 13** for real.
3. For checks **04–12** and **14–15**: either **fail closed** (nonzero, message `not-yet-implemented: check NN`) **or** **skip** with an explicit line that cites a registered `docs/debt/register.toml` finding id. Silent pass is forbidden.
4. Exit **0** only if every implemented check passed and every stub is skipped with debt (or would fail closed). **Shipped policy:** implemented checks gate the process; stubs skip-with-debt (`skip(DEBT-GUARD-NN)`) and are listed in the report; process exit 0 if implemented checks pass. If a stub has **no** matching debt finding, that stub **fails closed**.
5. Print a stable report: check id, name, `pass` / `fail` / `skip(DEBT-…)`.

`cargo test -p xtask` covers TOML parse helpers and the resolved-without-proof / duplicate-id rejection with fixture registers (tempdir). Product crates are not the place for these tests.

#### 4.3.1 Guard check catalog (IDs are stable)

| ID | Name | This slice |
| --- | --- | --- |
| 01 | Architecture manifest present + parseable | **Implement** |
| 02 | Canonical ownership table present (seven concepts → live crates) | **Implement** |
| 03 | Forbidden-patterns file present + parseable | **Implement** |
| 04 | Architecture invariants evaluated | Stub this slice; **real** in [ADR 0010](../adr/0010-architecture-as-law.md) |
| 05 | Catalog SSOT (no dual catalog sources) | Stub |
| 06 | Framework pack parse fail-closed | Stub |
| 07 | Framework digest redesign | Stub |
| 08 | Readiness projection SSOT | Stub |
| 09 | Temporal evidence selection law | Stub |
| 10 | Assessment lineage rebuild | Stub |
| 11 | Evidence latest vs current | Stub |
| 12 | Statement of Applicability invariants | Stub |
| 13 | Debt register schema + unique IDs + resolved-without-proof rejected | **Implement** |
| 14 | ADR graph / unique ADR IDs | Stub this slice; **real** in [ADR 0011](../adr/0011-repository-guard-governance.md) |
| 15 | Spec lifecycle states / crate dependency graph policy | Stub this slice; spec-lifecycle **real** in [ADR 0011](../adr/0011-repository-guard-governance.md) (crate-graph product law remains remaining_backlog) |

This slice did **not** implement 04–12 / 14–15 beyond the stub contract. Successor [ADR 0010](../adr/0010-architecture-as-law.md) implements **04** only. Increment 2 implements **14–15**.

### 4.4 CI

Edit [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) so a step **must** run:

```text
cargo xtask guard
```

Shipped: CI job `test` runs a mandatory `repository health gate` step (`cargo xtask guard`) after rustfmt and before clippy. Failure fails the job.

Do not switch clippy/test to `--workspace` in this slice (that is remaining_backlog / check 15 adjacent). Do not remove `--features demo`.

### 4.5 Dual-suite protocol (mandatory)

Protocol (this program, not optional):

1. **Spec first** (this file + draft ADR 0009). No product code in the spec-first commit.
2. **Baseline GREEN** on CURRENT absences (RI-B01–B10).
3. **Target RED** on CURRENT absences proving §4 (RI-T01–T16 below).
4. **Implement** xtask + alias + manifests + debt + CI until target GREEN.
5. **Supersede** baseline with `#[ignore = "superseded by sdd_repository_integrity_target"]`.
6. Add `docs/specs/repository-integrity.md` to `CANONICAL_SPECS`.
7. Set ADR 0009 to **Accepted**.

Increment 1 completed this protocol. Target suite GREEN; baseline ignore-superseded; ADR 0009 Accepted.

Target tests (must fail before product files exist; pass after):

| ID | Assertion |
| --- | --- |
| RI-T01 | `architecture/architecture.toml` exists and parses; `schema` is `weeping-angel/architecture/v1` |
| RI-T02 | Ownership table lists all seven concepts with live crate names in §4.1.1 |
| RI-T03 | `catalog` crate is `weeping-angel-canonical-catalog` (not `weeping-angel-catalog`) |
| RI-T04 | `assurance_cli` crate is `weeping-angel` with paths `src/main.rs` and `src/cli.rs` (not `weeping-angel-assurance-cli`) |
| RI-T05 | `architecture/forbidden-patterns.toml` exists and parses |
| RI-T06 | `architecture/invariants.toml` exists and parses |
| RI-T07 | `docs/debt/register.toml` exists; every finding has `id` and `status` |
| RI-T08 | Finding IDs in the register are unique |
| RI-T09 | A fixture/register with `status="resolved"` and neither `regression_tests` nor `repository_guard` is rejected by the guard/parser |
| RI-T10 | Workspace members include `xtask`; `.cargo/config.toml` aliases `xtask` |
| RI-T11 | `cargo xtask guard` is invocable (process starts; implemented checks run) |
| RI-T12 | Checks 01, 02, 03, 13 actually execute (report lines, not comments) |
| RI-T13 | Check **04** pass/evaluated (ADR 0010). Checks **05–12** and **14–15** do not silently pass: skip cites `DEBT-GUARD-NN` **or** fail `not-yet-implemented` |
| RI-T14 | `.github/workflows/ci.yml` contains a step running `cargo xtask guard` |
| RI-T15 | Dual-suite names `sdd_repository_integrity_{baseline,target}` are listed in root `Cargo.toml` |
| RI-T16 | This spec path is in `CANONICAL_SPECS` |

---

## 5. Acceptance criteria (testable)

- [x] `architecture/architecture.toml`, `architecture/invariants.toml`, and `architecture/forbidden-patterns.toml` exist and parse as TOML with the schema strings in §4.1.
- [x] Ownership table contains `catalog`, `framework_compilation`, `readiness_projection`, `temporal_evidence_selection`, `assessment_lineage`, `evidence_persistence`, `assurance_cli` mapped to live packages (canonical-catalog, framework, assurance, evidence, root `weeping-angel` + `src/main.rs`/`src/cli.rs`).
- [x] `docs/debt/register.toml` requires `id` + `status`; unique ids; `resolved` without `regression_tests` or `repository_guard` is rejected.
- [x] `docs/debt/README.md` and `docs/debt/baseline-2026-08.md` exist; baseline records the live counts in §3.6 (re-measured).
- [x] Workspace member `xtask` exists; `.cargo/config.toml` defines `xtask` alias; `cargo xtask guard` runs checks 01, 02, 03, 13.
- [x] Guard checks 05–12 and 14–15 are stubs: fail closed or skip only with a registered debt finding — never a silent pass. Check **04** is evaluated by [ADR 0010](../adr/0010-architecture-as-law.md).
- [x] CI workflow contains a mandatory `cargo xtask guard` step.
- [x] Dual-suite `sdd_repository_integrity_{baseline,target}` is registered in root `Cargo.toml`; baseline skip-superseded after target GREEN.
- [x] `sdd_documentation_layout` stays GREEN with this spec in `CANONICAL_SPECS`.
- [x] No new package named `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
- [x] ADR 0009 exists (Accepted). Remaining_backlog items are **not** implemented.

---

## 6. Risks

- Stub skips with debt could become a permanent “skip everything” hatch if seed findings are too generic; each stub needs its **own** `DEBT-GUARD-NN` id.
- Exit-0-with-skips vs fail-closed-stubs: implementers might accidentally fail CI on stubs. The recommended policy (implemented checks gate exit; stubs skip-with-debt) must be followed or CI cannot go green.
- Duplicate ADR IDs: minting `0003-*` again would worsen debt; 0009 is mandatory.
- `temporal_evidence_selection` also lives in `weeping-angel-control-test`; declaring assurance as owner without moving code can confuse later slices — record as comment in architecture.toml, do not move.
- Adding `xtask` to workspace may interact with `cargo dist` / packager metadata; `xtask` must be `publish = false` and must not become a distributed binary.
- CI currently is not `--workspace`; `cargo test -p xtask` is not in CI unless `cargo test --all-targets` at the root picks up workspace members — **root `cargo test --all-targets` does not build other workspace members’ tests**. Implement MUST either run `cargo xtask guard` as its own CI step (required) and/or document that `cargo test -p xtask` is implied by the guard step, not by the existing test step.
- Counting unwrap/ignore in tests vs production can inflate debt; baseline-2026-08.md should state the inclusion rule (all `*.rs` excluding `target/` / `node_modules/`).

---

## 7. Dual-suite and verify commands

```text
cargo test --test sdd_repository_integrity_target
cargo test --test sdd_repository_integrity_baseline -- --ignored
cargo test -p xtask
cargo xtask guard
cargo test --test sdd_documentation_layout
cargo fmt --all -- --check
```

Shipped stub policy after ADR 0010: checks **05–12 / 14–15** printed `skip(DEBT-GUARD-NN)` when that finding existed; otherwise fail closed. After increment 2, checks **14–15** never skip; **05–12** still skip only with a non-expired, fully populated exemption. Check **04** passes when every invariant evaluates. Guard exit 0 if implemented checks pass. `cargo clippy` / full `cargo test --features demo --all-targets` remain the existing CI commands (not `--workspace`); CI additionally runs `cargo xtask guard`. Root `cargo test --all-targets` does **not** run `cargo test -p xtask`.

---

## 8. remaining_backlog (out of scope for this slice)

Do **not** implement these in the health-gate slice. Record them in the spec and optionally as `open`/`confirmed` debt rows. They correspond to stubbed checks **05–12 / 14–15** and P0 remediations. Check **04** is closed by [ADR 0010](../adr/0010-architecture-as-law.md).

1. P0 framework expression preservation
2. Fail-closed pack parsing (check 06)
3. Catalog SSOT migration (check 05)
4. Framework digest redesign (check 07)
5. Readiness SSOT (check 08)
6. Lineage rebuild (check 10)
7. Evidence latest vs current (check 11)
8. Statement of Applicability invariants (check 12)
9. Persistence invariants (evidence ledger)
10. Package install tests
11. Crate dependency graph policy (check 15, partial)
12. Schema fixtures (JSON Schema for IR/catalog/packs)
13. ADR graph validator / unique ADR IDs (check 14) — **pulled into increment 2** ([§13.4](#134-guard-14--adr-identity-and-graph)); still **do not mass-renumber** existing `0003-*` files (`DEBT-DUP-ADR`)
14. Spec lifecycle states — **pulled into increment 2** ([§13.5](#135-guard-15--spec-lifecycle-and-dependency-policy))
15. Deleting obsolete baseline suites
16. Test-support crate
17. Remaining guard checks **05–12** beyond stubs (check **04** evaluates `invariants.toml` — ADR 0010; checks **14–15** become real in increment 2). Do **not** encode Prompt 2/3 product semantics in xtask.
18. Switching CI clippy/test to `--workspace`

---

## 9. Related

- Decision (Accepted): [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md)
- Successor (architecture-as-law): [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md), [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md)
- Increment 2 (Accepted): [`docs/adr/0011-repository-guard-governance.md`](../adr/0011-repository-guard-governance.md) — [§10+](#10-increment-2-collision-fence)
- Docs layout: [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md)
- Crate graph: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md)
- Catalog crate: [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md)
- Debt register: [`docs/debt/register.toml`](../debt/register.toml)
- Pointer stub: [`docs/sdd/repository-integrity.md`](../sdd/repository-integrity.md)

---

# Increment 2 — Repository Guard and Governance Hardening

Field-level law for Prompt 1. Dual-suite protocol completed: spec first → increment-2 baseline GREEN on increment-1/ADR-0010 → increment-2 target RED for the right reasons → implement → [ADR 0011](../adr/0011-repository-guard-governance.md) Accepted → target GREEN → increment-2 baseline RI-B11–B18 `#[ignore]`-superseded → target still GREEN.

Do not paper over with new `#[ignore]`, broad allowlists, or new debt unless unavoidable, narrowly scoped, owned, expiring, and justified.

Architecture law (unchanged):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Repository-level law after increment 2:

```text
architecture/*.toml          (ownership + policy kinds/concepts; invariants; forbidden)
architecture/adr-identity.toml
architecture/spec-lifecycle.toml
docs/adr/*                   (identity + machine-readable lifecycle metadata)
docs/debt/register.toml      (exemptions expire; resolved still needs proof)
        ↓
RepositoryModel              (single load; cached source index; deterministic order)
        ↓
cargo xtask guard            (modular checks; 01–04, 13–15 real; 05–12 stub/plumbing)
        ↓
CI must run it, no path-filter bypass on governance surfaces
```

---

## 10. Increment-2 collision fence

Concurrent Prompts 2–4 run on the same tree. This increment may modify **only**:

| Surface | Allowed work |
| --- | --- |
| `xtask/**` | Decompose engine; real Guards 14/15; debt-expiry; additive JSON; model cache |
| `architecture/**` | Versioned policy (kinds, required concepts), ADR identity file, spec-lifecycle file |
| `.cargo/**` | Only if required to keep `cargo xtask` working |
| `.github/workflows/**` | Only repository-health-gate enforcement (must keep `cargo xtask guard`; no bypass) |
| `docs/adr/**` | Guard 14 metadata/identity **normalization only**; do **not** rewrite 0009/0010 (or any) decision bodies |
| `docs/specs/repository-integrity.md` | This SSOT (extend, do not fork) |
| `docs/debt/register.toml` | Exemption fields; resolve `DEBT-GUARD-14` / `DEBT-GUARD-15` with proof |
| `tests/contracts/repository_integrity*` | Increment-2 baseline + target IDs |
| `tests/contracts/architectural_cleanup*` | Only if created; **today ACP lives in `xtask/tests/`** — update those assertions, do not move them to `tests/sdd/` |

**Forbidden:** `src/**`, `crates/**`, other contract suites, `schemas/**`, README, generated audit artifacts, `tests/sdd/`, inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`.

Guards **05–12** may gain plumbing/interfaces only if they do **not** encode incomplete catalog/framework/readiness/temporal/lineage/evidence/SoA product semantics (Prompts 2/3). Prompt 4 owns panic budget, broad test retirement, schemas, README/audit hygiene.

`docs/sdd/` remains a stub. Traces belong under `.sdd/runs/` and `.sdd/artifacts/` only (ADR 0004).

---

## 11. Increment-2 problem / user-visible goal

Increment 1 + ADR 0010 shipped a working health gate, but the enforcement system is not durable:

1. `xtask/src/lib.rs` is a ~1434-line monolith (`lib.rs` + `main.rs` only). Adding Guards 14/15 and debt-expiry on that file repeats the “hand-written grep framework” failure mode ADR 0010 was meant to stop.
2. `run_guard` loads one `RepositoryModel`, but `source_files` is only a path list. `source_contains` and `kind=symbol` `in_crate` **reread disk per check**. Public `check_01_*`…`check_04_*` each `RepositoryModel::load` again (allowed as test wrappers; the evaluation plane for `run_guard` must not).
3. Policy is duplicated as Rust constants: `REQUIRED_OWNERSHIP`, `OWNERSHIP_KINDS`, `FORBIDDEN_PACKAGES`, `REMAINING_STUBS`. Those values already live (or should live) under `architecture/`. Rust must **validate and interpret**, not be a second SSOT.
4. Guard **14** is `StubArchitectureCheck` → `skip(DEBT-GUARD-14)`. Historical ADR prefix collisions (`0003`×25, `0005`×5, `0007`×2, `0008`×4) are recorded as `DEBT-DUP-ADR` but a **new** duplicate `0010-*` or `0003-*` can land. Nothing parses ADR lifecycle metadata or graph edges.
5. Guard **15** is `skip(DEBT-GUARD-15)`. Specs have no machine lifecycle; superseded/retired text can still be treated as active law; there is no spec→ownership binding.
6. Debt validation requires only `id` / `title` / `status` / `summary` + unique IDs + resolved proof. Live skip exemptions (`DEBT-GUARD-05`…`12`, `14`, `15`) have no owner, introduced date, severity, remediation, associated check, or expiry. Expired debt cannot fail CI because expiry does not exist.
7. `GuardReport` JSON is `{ checks, violations, skipped, debt_exemptions, duration }`. `duration` is wall-clock (`Instant::now`). There is no `schema` / `version` / aggregate counts. Equality-sensitive fixtures that snap JSON will flake on `duration`.
8. CI [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) **already** always runs `cargo xtask guard` (no `paths-ignore` today). Increment 2 must **keep** that property: introducing a path filter must not bypass the gate when architecture, ADRs, specs, debt, workspace manifests, frameworks/catalog, or Rust source change.

**User-visible goal:** a contributor or CI runs **one** command and gets a fail-closed, modular, policy-driven verdict — including ADR identity/graph and spec lifecycle — without rereading the tree per check and without silent-pass on missing metadata:

```text
cargo xtask guard
  → 01–04, 13 real (behavior retained unless deliberately strengthened)
  → 14 ADR identity + graph (real; historical dupes via DEBT-DUP-ADR only)
  → 15 spec lifecycle + spec-dependency policy (real)
  → 05–12 skip(DEBT-GUARD-NN) with owned, dated, expiring exemptions — or fail closed
  → expired exemption → fail
  → missing/malformed architecture, invariants, forbidden-patterns, debt, ADR metadata, spec-lifecycle → fail
  → JSON: schema/version + counts + current fields (duration not used for equality)
```

---

## 12. Increment-2 current behavior (baseline — GREEN on CURRENT increment-1 / ADR-0010)

Characterize **today’s shipped tree**, not SHA `f560196c` absences. Do **not** re-enable RI-B01–B10 (those stay `#[ignore = "superseded by sdd_repository_integrity_target"]`).

Executable characterization: additive tests in `tests/contracts/repository_integrity.baseline.rs` (RI-B11+) and, if needed, ACP baseline notes in `xtask/tests/sdd_architectural_cleanup_baseline.rs` (already increment-1-superseded; do not revive ACP-B01–B06 as live law).

### 12.1 xtask is a two-file monolith

Workspace member `xtask` (`publish = false`). Sources:

```text
xtask/src/lib.rs     (~1434 lines — model, architecture parse, debt, checks, report, CLI)
xtask/src/main.rs    (exit(xtask::main_with_args(...)))
```

No `xtask/src/{model,architecture,debt,checks,report}.rs` modules. `cargo test -p xtask` also runs `xtask/tests/{debt_register.rs,sdd_architectural_cleanup_{baseline,target}.rs}`.

### 12.2 RepositoryModel is single-load for `run_guard`, but source is not cached

`run_guard` / `run_guard_with_options` construct **one** `RepositoryModel` then `ArchitectureCheck::check`.

`RepositoryModel` fields today: `root`, `workspace_members`, `package_graph`, `package_names`, `filesystem`, architecture/invariants/forbidden + `*_error`, `debt_ids` + `debt_error`, `adr_files` (filename list only), `spec_files` (filename list only), `framework_packs`, `catalog_sources`, `source_files` (`Vec<String>` of `src/**` + `crates/**` `.rs` paths).

`source_contains` loops `source_files` and `fs::read_to_string` each call. `kind=symbol` with `in_crate` reads those paths again. There is no normalized source text map and no inverted index.

`adr_files` / `spec_files` are sorted basenames; no lifecycle metadata is parsed.

Public wrappers `check_01_architecture_manifest`, `check_02_ownership`, `check_03_forbidden_patterns`, `check_04_architecture_invariants` each call `RepositoryModel::load` again (test-facing; preserve signature).

### 12.3 Policy is hard-coded in Rust

| Constant | Role | Duplicates |
| --- | --- | --- |
| `REQUIRED_OWNERSHIP` (pub) | Seven concepts → package + path needles | `architecture/architecture.toml` `[ownership.*]` |
| `OWNERSHIP_KINDS` | `exclusive \| facade \| projection \| adapter \| shared-primitive` | kinds already required on rows; not listed as a versioned table |
| `FORBIDDEN_PACKAGES` | `weeping-angel-catalog`, `weeping-angel-assurance-cli` | `architecture/forbidden-patterns.toml` `kind = "package"` |
| `REMAINING_STUBS` | 05–12, 14, 15 names | debt `DEBT-GUARD-NN` rows |
| Schema string consts | `ARCH_SCHEMA` / `INVARIANTS_SCHEMA` / `FORBIDDEN_SCHEMA` / `DEBT_SCHEMA` | matching `schema =` in the TOML files (schema IDs may stay in Rust as accepted constants) |

ACP-T01 / ACP-T02 **grep only** `xtask/src/lib.rs` for `struct RepositoryModel` and `trait ArchitectureCheck`.

### 12.4 Guard catalog (shipped)

| ID | Name | Current |
| --- | --- | --- |
| 01 | `architecture-manifest` | Real: file + schema `weeping-angel/architecture/v1` |
| 02 | `canonical-ownership` | Real: seven concepts, live crates/paths, required `kind` |
| 03 | `forbidden-patterns` | Real: executes `package \| path \| dependency \| symbol \| source-pattern` |
| 04 | `architecture-invariants` | Real: evaluates every `[[invariant]]`; unknown id fails |
| 05–12 | catalog-ssot … soa-invariants | `StubArchitectureCheck` → `skip(DEBT-GUARD-NN)` |
| 13 | `debt-register` | Real: schema, unique ids, resolved-without-proof rejected |
| 14 | `adr-graph` | **Stub** `skip(DEBT-GUARD-14)` |
| 15 | `spec-lifecycle` | **Stub** `skip(DEBT-GUARD-15)` |

Human render (preserved): `NN  <name>  pass|fail|skip(DEBT-…)`.

CLI (preserved): `cargo xtask guard [--json] [--check NN] [--explain INV-…]`. Unknown check id fails closed. Exit 0 iff no `Fail`.

### 12.5 Debt register (check 13 today)

`schema = "weeping-angel/debt-register/v1"`. Required per `[[finding]]`: `id`, `title`, `status`, `summary`. Status ∈ `open|confirmed|in-progress|resolved|rejected|superseded`. Unique ids. `resolved` requires `regression_tests` or `repository_guard`. **Not** required: `owner`, `introduced`, `severity`, `remediation`, associated guard, `expires` / `review_by`.

Live seed at increment-2 start: `DEBT-GUARD-05`…`12`, `14`, `15` are `open` without those fields. `DEBT-GUARD-04` is `resolved` with `repository_guard = "04"` and `regression_tests = ["sdd_architectural_cleanup_target"]`. `DEBT-DUP-ADR` is `confirmed` (historical prefix collisions). After implement, `DEBT-GUARD-14` / `DEBT-GUARD-15` are `resolved` with live checks + named tests; `DEBT-DUP-ADR` stays `confirmed` with exemption fields and pins `0003`/`0005`/`0007`/`0008`/`0011`.

### 12.6 ADR / spec metadata today

- `docs/adr/*.md`: human Status tables; **no** machine-readable `supersedes` / `superseded_by` / `depends_on` block that Guard 14 parses.
- Prefix collisions confirmed: `0003`×25, `0005`×5, `0007`×2, `0008`×4. Unique: `0001`, `0002`, `0004`, `0006`, `0009`, `0010`.
- `docs/specs/*.md`: no `architecture/spec-lifecycle.toml`. `CANONICAL_SPECS` in `tests/contracts/documentation_layout.rs` already lists this file.
- `docs/sdd/repository-integrity.md` is a pointer stub.

### 12.7 GuardReport JSON today

```text
{ checks, violations, skipped, debt_exemptions, duration: { secs, nanos, as_secs_f64 } }
```

No `schema`, no `version`, no aggregate counts, no separate `failed` array (violations is the fail list). Zero-duration coerced to 1 ns. `duration` is wall-clock.

### 12.8 CI today

[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) job `test` always runs `cargo xtask guard` after rustfmt, before clippy. No `paths-ignore` / `paths` filter on the workflow. Clippy/test remain `--features demo --all-targets` (not `--workspace`).

### 12.9 Neighbor assertions that increment 2 must update (not weaken 01–04)

| Test | Current assertion | Increment-2 obligation |
| --- | --- | --- |
| RI-T13 | 04 pass; **05–12 and 14–15** skip-or-nyi | Keep 01–04/13; **14/15 become real** (update assertion) |
| RI-T17 | 05–12 / 14–15 stay stubs | 14/15 no longer stubs; 05–12 remain stubs |
| ACP-T01 / T02 | grep `xtask/src/lib.rs` only | Grep `xtask/src/**` after the split |
| ACP-T03 / T11 | remaining stubs include **14, 15** skip-with-debt | 14/15 pass on increment-1 fixture + new metadata files; 05–12 still skip |

### 12.10 Increment-2 baseline IDs (must be GREEN on CURRENT code)

| ID | Assertion on CURRENT increment-1 / ADR-0010 tree |
| --- | --- |
| RI-B11 | `xtask/src/` Rust sources are only `lib.rs` and `main.rs` (no `model.rs` / `architecture.rs` / `debt.rs` / `checks*` / `report.rs`) |
| RI-B12 | xtask sources contain `REQUIRED_OWNERSHIP`, `OWNERSHIP_KINDS`, `FORBIDDEN_PACKAGES`, and remaining-stub entries for `"14"` and `"15"` |
| RI-B13 | `RepositoryModel` exposes `source_files`; evaluation of source needles uses `fs::read_to_string` (no cached source map / index field) |
| RI-B14 | Live `cargo xtask guard` report contains `skip(DEBT-GUARD-14)` and `skip(DEBT-GUARD-15)` (or equivalent skip+debt_id) |
| RI-B15 | `docs/debt/register.toml` live `DEBT-GUARD-05`…`12`/`14`/`15` rows omit at least one of `owner`, `introduced`, `severity`, `remediation`, `expires`/`review_by` |
| RI-B16 | `cargo xtask guard --json` includes `checks` / `violations` / `skipped` / `debt_exemptions` / `duration` and does **not** include a report `schema` / `version` / aggregate `counts` object |
| RI-B17 | `DEBT-GUARD-14` and `DEBT-GUARD-15` have `status = "open"` |
| RI-B18 | `architecture/spec-lifecycle.toml` and `architecture/adr-identity.toml` are **not** files |

---

## 13. Increment-2 desired behavior (target — RED on CURRENT, GREEN after implement)

### 13.1 Modular xtask (preserve public `cargo xtask guard`)

Decompose `xtask/src/lib.rs` into focused modules. Required module boundaries (names may be `mod` files or a `checks/` directory):

| Module | Owns |
| --- | --- |
| `model` | `RepositoryModel::load`, workspace/package graph, filesystem index, **cached** source text or lightweight index, deterministic `BTree*` ordering |
| `architecture` | Parse/validate `architecture/*.toml` (manifest, invariants, forbidden, adr-identity, spec-lifecycle); types `ArchitectureManifest`, `OwnershipRow`, `ArchitectureInvariant`, `ForbiddenPattern` |
| `debt` | Parse/validate `docs/debt/register.toml`; exemption + expiry + orphan/duplicate rules |
| `checks` | Individual `ArchitectureCheck` impls (01–15). Stubs 05–12 remain `StubArchitectureCheck` (or equivalent plumbing) |
| `report` | `CheckResult`, `GuardReport`, `render()`, `to_json()`, human line format |

`lib.rs` re-exports the public surface used by tests and `main.rs`. `main.rs` stays a thin `main_with_args` wrapper.

**Preserved public behavior:**

- CLI: `guard`, `--json`, `--check NN`, `--explain INV-…`, usage exit 2, fail exit 1, success exit 0
- Human lines: `NN  <name>  pass|fail|skip(DEBT-…)`
- JSON **existing keys** remain: `checks`, `violations`, `skipped`, `debt_exemptions`, `duration`
- Types/functions: `run_guard`, `main_with_args`, `CheckStatus`, `CheckResult`, `GuardReport`, `ArchitectureCheck`, `RepositoryModel`, `check_01_*`…`check_04_*`, schema consts
- `REQUIRED_OWNERSHIP` may become a **derived** view from loaded policy (same tuple shape) so ACP tests that still import it keep compiling, **or** those tests are updated in `xtask/tests` to read `architecture.toml`. Rust must not remain the policy SSOT.

Checks **01–04** and **13** retain behavior unless deliberately strengthened; any strengthening needs a regression in the dual-suite.

### 13.2 Single-load evaluation plane + cached source

`run_guard` / `run_guard_with_options` load **one** `RepositoryModel`.

At construction the model MUST cache either:

- normalized source text keyed by repo-relative path (`BTreeMap<String, String>`), or
- a deterministic lightweight index sufficient for `source-pattern` / `symbol` / `in_crate`

Checks MUST NOT walk/reread the whole `src/` + `crates/` tree again. `in_crate` filters the cache. Ordering of `source_files`, `adr_files`, `spec_files`, `workspace_members`, and report checks is deterministic (sorted / `BTreeMap` insertion from sorted walks).

Public `check_01_*`…`check_04_*` may still reload the model (documented test wrappers).

Missing/malformed architecture manifest, invariants, forbidden-patterns, debt register, ADR identity/metadata set, or spec-lifecycle file is recorded on the model as `*_error` and the corresponding check **fails** — never a silent pass, never skip-without-debt.

### 13.3 Policy lives under `architecture/` (Rust interprets)

Extend [`architecture/architecture.toml`](../../architecture/architecture.toml) with a required `[policy]` table (same file, same `schema = "weeping-angel/architecture/v1"`):

```toml
[policy]
ownership_kinds = ["exclusive", "facade", "projection", "adapter", "shared-primitive"]
required_concepts = [
  "catalog",
  "framework_compilation",
  "readiness_projection",
  "temporal_evidence_selection",
  "assessment_lineage",
  "evidence_persistence",
  "assurance_cli",
]
```

Rules:

- Missing `[policy]`, empty `ownership_kinds`, or empty `required_concepts` → check **01** or **02** fails closed.
- Check **02** reads required concept keys and allowed kinds from this table. Row `crate` / `paths` / `kind` stay on `[ownership.*]`. Live crate-name law (canonical-catalog, root `weeping-angel`, …) remains this spec + the TOML rows + RI-T02–T04; Rust does not re-list those packages as the SSOT.
- Hypothetical package names are **only** forbidden via `architecture/forbidden-patterns.toml` `kind = "package"` (plus check 03 execution). Delete `FORBIDDEN_PACKAGES` as policy SSOT.
- Optional extra ownership row (allowed today) for `repository_guard` → package `xtask`, `kind = "exclusive"`, `paths = ["xtask"]` MAY be added so Guard 15 can bind this spec to an existing concept. Do not invent hypothetical crates.

Create [`architecture/adr-identity.toml`](../../architecture/adr-identity.toml):

```toml
schema = "weeping-angel/adr-identity/v1"
grandfathered_debt = "DEBT-DUP-ADR"
grandfathered_prefixes = ["0003", "0005", "0007", "0008", "0011"]
# grandfathered_files pins the historical (and concurrent increment-2) set.
# A new path that reuses one of these prefixes but is not listed fails Guard 14.
```

Create [`architecture/spec-lifecycle.toml`](../../architecture/spec-lifecycle.toml) — see §13.5.

Missing/wrong schema on any of these files fails the owning check (01/14/15), not skip.

### 13.4 Guard 14 — ADR identity and graph (real)

Check **14** (`adr-graph`) is an `ArchitectureCheck`, not a stub. `DEBT-GUARD-14` becomes `resolved` with `repository_guard = "14"` and named regression tests (`sdd_repository_integrity_target` and/or `sdd_architectural_cleanup_target`).

#### 13.4.1 Identity

- Every `docs/adr/*.md` filename MUST match `^(\d{4})-.+\.md$`.
- **Prefix identity** = the four digits. A prefix that appears more than once **fails** unless:
  1. the prefix is listed in `architecture/adr-identity.toml` `grandfathered_prefixes`, **and**
  2. `DEBT-DUP-ADR` is a live (not resolved-without-proof) finding, **and**
  3. the colliding files are listed in `grandfathered_files` (historical `0003`/`0005`/`0007`/`0008` plus the three concurrent increment-2 `0011-*` drafts). A **new** file that reuses `0003` / `0005` / `0007` / `0008` / `0011` / any already-used unique prefix (`0001`, `0002`, `0004`, `0006`, `0009`, `0010`, `0012`, …) **fails**.
- Do **not** silently renumber historical ADRs. Mass-renumber remains remaining_backlog (architectural-cleanup Phase 27).
- Missing `architecture/adr-identity.toml` or malformed schema → check 14 fails closed (historical dupes then have no legal grandfather).

#### 13.4.2 Machine-readable lifecycle metadata

Every ADR file MUST contain exactly one parseable metadata block. Format (HTML comment so decision prose is not rewritten):

```html
<!-- weeping-angel-adr-meta
id = "0011"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0009-repository-health-gate", "0010-architecture-as-law"]
-->
```

| Field | Rule |
| --- | --- |
| `id` | Four-digit prefix; MUST match the filename prefix |
| `status` | Closed set: `draft` \| `proposed` \| `accepted` \| `superseded` \| `rejected` \| `deprecated` |
| `supersedes` | Array of **filename stems** (preferred) or unambiguous unique prefixes |
| `superseded_by` | Same |
| `depends_on` | Same; must be acyclic |

**Graph node id** is the filename stem (`0003-applicability-engine`), so grandfathered prefix collisions remain addressable.

Implement may add this block to existing ADRs **without changing decision sections**. That is the only permitted `docs/adr/**` edit besides the new 0011 file.

Missing block, unparseable TOML, `id` mismatch, or illegal status → check 14 fails for that file.

#### 13.4.3 Graph validation

- Every edge target MUST resolve to an existing `docs/adr/<stem>.md` (dangling `supersedes` / `superseded_by` / `depends_on` fail).
- `supersedes` / `superseded_by` / `depends_on` MUST be acyclic (DFS/Kahn on the directed graph). A cycle fails closed.
- If A lists `supersedes = ["B"]`, B SHOULD list A in `superseded_by` (inverse consistency). Missing inverse is a check-14 failure (fail-closed; do not silently repair).
- Evaluation is offline, repository-bound, deterministic (sorted stems).

Live tree after metadata landing: check 14 **pass** (grandfathered prefix set via `DEBT-DUP-ADR`, no new collisions, graph acyclic).

### 13.5 Guard 15 — spec lifecycle and dependency policy (real)

Check **15** (`spec-lifecycle`) is an `ArchitectureCheck`, not a stub. `DEBT-GUARD-15` becomes `resolved` with `repository_guard = "15"` and named regression tests.

[`architecture/spec-lifecycle.toml`](../../architecture/spec-lifecycle.toml):

```toml
schema = "weeping-angel/spec-lifecycle/v1"

[[spec]]
path = "docs/specs/repository-integrity.md"
state = "active"
ownership = ["repository_guard"]  # or another existing architecture.toml concept key
depends_on = ["docs/specs/assurance-runtime-spine.md"]
supersedes = []
successor = ""
```

| Field | Rule |
| --- | --- |
| `path` | Repo-relative path that exists; unique in the file |
| `state` | Closed set: `draft` \| `active` \| `superseded` \| `retired` |
| `ownership` | Non-empty when `state = "active"`; every key MUST exist in `architecture.toml` `[ownership]` |
| `depends_on` | Array of spec paths; must exist; graph acyclic |
| `supersedes` / `successor` | Paths; dangling fails |

**Coverage:** every `docs/specs/*.md` file MUST appear as a `[[spec]]` row. An on-disk spec missing from the lifecycle file fails. A row whose `path` is not a file fails.

**Transitions** (documented; Guard 15 enforces *state consistency*, not git history):

```text
draft → active | retired
active → superseded | retired
superseded → retired
retired → (terminal)
```

Illegal advertised transitions: a `superseded` or `retired` row MUST NOT use `state = "active"`. A `successor` is required when `state = "superseded"`. Active specs cannot set `successor` to themselves.

**Masquerade rule:** if `state ∈ {superseded, retired}`, check 15 fails if the row still claims to be the active SSOT (`state = "active"` is the only “active requirements” signal). Human Status tables are not a second machine state.

**Dependency policy** in this increment is **spec-to-spec** `depends_on` plus spec-to-ownership bindings. Full ADR 0001 crate-edge catalog remains architectural-cleanup Phase 19 / remaining_backlog. Forbidden crate edges already executable via check 03 `kind = "dependency"` stay there. Do not encode Prompt 2/3 product crate semantics in check 15.

Missing/malformed `architecture/spec-lifecycle.toml` → check 15 fails closed (never skip).

### 13.6 Debt exemptions harden (check 13, fail-closed expiry)

Keep `schema = "weeping-angel/debt-register/v1"` (additive required fields; do not invent a Prompt-4 JSON Schema file).

**Every live guard exemption** — a finding cited by a skip (`DEBT-GUARD-NN` still used by checks 05–12) **or** a finding that lists `skip_check` / is the registered stub exemption — MUST have:

| Field | Rule |
| --- | --- |
| `owner` | Non-empty string |
| `introduced` | ISO date `YYYY-MM-DD` |
| `severity` | `low` \| `medium` \| `high` \| `critical` |
| `remediation` | Non-empty statement |
| `repository_guard` | Associated check id (e.g. `"05"`) |
| `expires` or `review_by` | ISO date `YYYY-MM-DD` (at least one) |

`DEBT-DUP-ADR` is grandfathered identity debt (not a whole-check skip) but is a **live exemption** for prefix collisions: it MUST also carry owner / introduced / severity / remediation / `repository_guard = "14"` / expiry-or-review.

**Expiry law:** if `expires` or `review_by` is **strictly before** the evaluation date, check 13 **fails** (`expired debt <id>`). Evaluation date is UTC calendar date, overridable by env `WEEPING_ANGEL_GUARD_AS_OF=YYYY-MM-DD` so fixtures are deterministic (no wall-clock in equality-sensitive JSON).

**Resolved proof (retained + tightened):** `status = "resolved"` still requires non-empty `regression_tests` **or** `repository_guard`. For increment-2 closures (`DEBT-GUARD-14`, `DEBT-GUARD-15`), require the live check to actually be implemented (not still a stub) **and** named regression tests.

**Reject:**

- malformed fields / illegal status / illegal severity / unparseable dates
- duplicate `finding.id`
- **orphaned** debt IDs: a skip cites an id not in the register; a `repository_guard` / `skip_check` names an unknown check id; a stub check has no matching finding (existing fail-closed)

Non-exemption rows (e.g. `DEBT-UNWRAP`) MAY omit exemption fields; they are not skip hatches. If they later become skip exemptions, they must gain the fields.

### 13.7 Additive machine-readable JSON

`GuardReport::to_json()` MUST remain a single JSON object and MUST keep current keys. Additive keys:

```json
{
  "schema": "weeping-angel/guard-report/v1",
  "version": 1,
  "checks": [ { "id": "01", "name": "architecture-manifest", "status": { "kind": "pass" } } ],
  "violations": [ { "check_id": "…", "message": "…" } ],
  "failed": [ { "check_id": "…", "message": "…" } ],
  "skipped": [ { "check_id": "15", "debt_id": "DEBT-GUARD-15" } ],
  "debt_exemptions": ["DEBT-GUARD-05"],
  "counts": { "total": 15, "pass": 7, "fail": 0, "skip": 8 },
  "duration": { "secs": 0, "nanos": 1, "as_secs_f64": 1e-9 }
}
```

- Check `id` values are the stable two-digit strings (`"01"`…`"15"`).
- `failed` may equal `violations` (duplicate view) so CI consumers have an obvious key; do not drop `violations`.
- `counts` are derived from `checks` in deterministic order.
- **Equality-sensitive fixtures MUST NOT assert `duration`.** Tests compare schema/version/ids/counts/status kinds only.
- Do not embed `SystemTime` / ISO timestamps in the report object.

### 13.8 CI health-gate enforcement

[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) MUST continue to run `cargo xtask guard` as a mandatory step (not `continue-on-error`).

If a `paths` / `paths-ignore` filter is introduced (this increment or later), the job that runs the guard MUST still execute when any of these change:

```text
architecture/**
docs/adr/**
docs/specs/**
docs/debt/**
Cargo.toml
**/Cargo.toml
.cargo/**
xtask/**
frameworks/**
catalog/**
src/**
crates/**
tests/contracts/repository_integrity*
tests/contracts/architectural_cleanup*
```

Today there is no path filter — **keep that**, or add an explicit always-on health-gate job. Do not switch clippy/test to `--workspace` (Prompt 4 / remaining_backlog).

### 13.9 Dual-suite IDs (target — RED on CURRENT)

Additive tests in `tests/contracts/repository_integrity.target.rs`. Update RI-T13 / RI-T17 rather than leaving 14/15 as stubs.

| ID | Assertion |
| --- | --- |
| RI-T18 | `xtask/src/` contains separate modules for model, architecture, debt, checks, and report (`lib.rs` is not the only implementation file) |
| RI-T19 | `RepositoryModel` caches normalized source text or a lightweight index at load; check evaluation does not `read_to_string` every `source_files` entry on each `source_contains` / symbol scan |
| RI-T20 | `[policy]` in `architecture.toml` lists `ownership_kinds` and `required_concepts`; xtask does not treat `REQUIRED_OWNERSHIP` / `OWNERSHIP_KINDS` / `FORBIDDEN_PACKAGES` as the policy SSOT |
| RI-T21 | Live guard check **14** is `pass` (not `skip(DEBT-GUARD-14)` / not-yet-implemented) |
| RI-T22 | Fixture repo that adds a new ADR file reusing prefix `0010` or `0003` fails check 14 |
| RI-T23 | Fixture with dangling `depends_on` / `supersedes`, or a cycle, fails check 14 |
| RI-T24 | Live guard check **15** is `pass`; `architecture/spec-lifecycle.toml` exists and lists every `docs/specs/*.md` |
| RI-T25 | Fixture with a spec `state = "active"` while also `superseded`, or a superseded row without successor, fails check 15; missing lifecycle file fails 15 |
| RI-T26 | Active spec `ownership` keys must exist in `architecture.toml`; unknown concept fails 15 |
| RI-T27 | Live skip exemptions include owner / introduced / severity / remediation / repository_guard / expires-or-review_by; fixture with yesterday’s `expires` fails 13 when `WEEPING_ANGEL_GUARD_AS_OF` is today |
| RI-T28 | `DEBT-GUARD-14` and `DEBT-GUARD-15` are `resolved` with proof; duplicate / orphan / malformed debt ids fail 13 |
| RI-T29 | `guard --json` includes `schema`, `version`, `counts`, `failed`, existing keys; tests do not equality-compare `duration` |
| RI-T30 | `.github/workflows/ci.yml` still contains a mandatory `cargo xtask guard` step and does not path-filter-bypass the surfaces in §13.8 |
| RI-T31 | Checks 01–04 and 13 still pass on the live tree; 05–12 still skip-with-debt or fail closed (no silent pass, no product-semantic implementation) |

ACP-T01/T02 must be updated to discover `RepositoryModel` / `ArchitectureCheck` under `xtask/src/**`. ACP-T03 remaining-stub list must drop 14/15. Increment-1 ACP-T08–T10 (forbidden kinds) stay GREEN.

### 13.10 Guard check catalog after increment 2

| ID | Name | Increment 2 |
| --- | --- | --- |
| 01 | architecture-manifest | Real (+ fail on missing `[policy]` if owned here) |
| 02 | canonical-ownership | Real (kinds/concepts from `[policy]`) |
| 03 | forbidden-patterns | Real (unchanged kinds) |
| 04 | architecture-invariants | Real (unchanged predicates unless a new invariant is added with a predicate) |
| 05–12 | product-semantic checks | **Stub / plumbing only** |
| 13 | debt-register | Real + exemption fields + expiry + orphans |
| 14 | adr-graph | **Real** |
| 15 | spec-lifecycle | **Real** |

---

## 14. Increment-2 acceptance criteria (testable)

- [x] `xtask` is modular (`model` / `architecture` / `debt` / `checks` / `report`); not a single 1400+ line `lib.rs` implementation.
- [x] `run_guard` loads one `RepositoryModel`; source text or a lightweight index is cached at load; check order is deterministic; checks do not reread the whole Rust tree.
- [x] Ownership kinds, required concepts, and forbidden package names are versioned under `architecture/`; Rust interprets only.
- [x] Guard **14** is real: unique **new** ADR prefixes; historical `0003`/`0005`/`0007`/`0008` contained by `DEBT-DUP-ADR` + `adr-identity.toml`; metadata parsed; dangling edges and cycles fail; no silent renumber.
- [x] Guard **15** is real: `architecture/spec-lifecycle.toml` required; explicit states; valid transitions; active specs reference existing ownership; superseded/retired cannot be `active`; missing file fails closed.
- [x] Live guard exemptions require owner, introduced, severity, remediation, associated check, and expiry/review; expired exemption fails CI; resolved still needs live guard or named tests; malformed/duplicate/orphaned ids fail.
- [x] Missing/malformed architecture manifest, invariants, forbidden-patterns, debt register, ADR metadata set, or spec-lifecycle file never silent-pass.
- [x] JSON is additive: `schema` / `version` / `counts` / `failed` + current keys; deterministic check ids; no unstable clocks in equality fixtures.
- [x] CI requires `cargo xtask guard` and cannot path-filter-bypass architecture, ADRs, specs, debt, workspace manifests, frameworks/catalog, or Rust source.
- [x] Checks 01–04 and 13 retain behavior (or are strengthened with regressions). 05–12 remain stubs. 14/15 are not debt-backed stubs.
- [x] `cargo fmt --all -- --check` passes; `cargo test -p xtask` passes; `sdd_repository_integrity_target` and ACP target pass after implement; `sdd_documentation_layout` stays GREEN.
- [x] `cargo xtask guard` passes on the integrated tree, or any fail is clearly a concurrent Prompt 2/3 **product-semantic** check — not a skipped/weakened 14/15/13.
- [x] No new `#[ignore]`, broad allowlist, or unjustified debt. No `weeping-angel-catalog` / `weeping-angel-assurance-cli`. ADR 0011 exists (Accepted at implement).

---

## 15. Increment-2 risks

- ACP-T01/T02/T03 and RI-T13 still encode increment-1 “14/15 are stubs” and `lib.rs`-only greps; forgetting to update them looks like a regression. Update assertions; do not weaken 01–04.
- Adding ADR HTML metadata to 40+ files is mechanical but merge-noisy under concurrent Prompt work — touch **only** the metadata block.
- Grandfather logic that keys on “files that existed at freeze” can drift; pin prefixes in `adr-identity.toml` and treat any **new** path with a colliding prefix as fail.
- Date expiry that uses `SystemTime` without `WEEPING_ANGEL_GUARD_AS_OF` makes fixtures timezone-flaky near midnight UTC.
- Pulling architectural-cleanup Phases 17–18 forward (Guards 14/15) while that spec still says “later” can confuse Prompt 2/3; this SSOT + ADR 0011 are the amendment. Do not rewrite ADR 0010’s non-goals paragraph.
- Encoding crate-graph product law in Guard 15 would collide with Prompts 2/3; keep 15 to spec lifecycle + spec dependencies.
- Additive JSON that changes key order or drops `duration` can break unknown consumers; keep keys and document `duration` as non-equality.
- Expired `DEBT-GUARD-05`…`12` will fail CI if implement sets short expiries; pick review dates that survive concurrent remediations.

---

## 16. Increment-2 dual-suite and verify commands

```text
cargo test --test sdd_repository_integrity_baseline -- --ignored
# increment-1 RI-B01–B10 and increment-2 RI-B11–B18 are ignore-superseded

cargo test --test sdd_repository_integrity_target
cargo test -p xtask
cargo xtask guard
cargo test --test sdd_documentation_layout
cargo fmt --all -- --check
```

Protocol recap (completed):

1. Spec first (this file + ADR 0011).
2. Baseline GREEN on increment-1 monolith/stubs/weak debt/JSON (RI-B11–B18); RI-B01–B10 stayed ignored.
3. Target RED on that tree for RI-T18–T31 (and updated RI-T13).
4. Implement exclusive surfaces until target GREEN.
5. Increment-2 baseline now fails on the new tree; those tests are `#[ignore = "superseded by sdd_repository_integrity_target"]`.
6. Target still GREEN. ADR 0011 is **Accepted**.

Shipped stub policy after increment 2: checks **05–12** print `skip(DEBT-GUARD-NN)` when a **non-expired**, fully populated exemption exists; otherwise fail closed. Checks **14–15** never skip.

---

## 17. Increment-2 out of scope

Do **not** do these in Prompt 1:

1. Guards **05–12** product semantics (catalog SSOT, pack parse/digest, readiness, temporal law, lineage, evidence latest-vs-current, SoA)
2. Moving `select_latest_as_of` or any `crates/**` / `src/**` code
3. ADR mass-renumber of historical `0003-*` / `0005-*` / `0007-*` / `0008-*` (Phase 27)
4. Deleting obsolete baseline suites / panic-budget / README / audit hygiene (Prompt 4)
5. JSON Schema fixtures under `schemas/` (Prompt 4)
6. Switching CI clippy/test to `--workspace`
7. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`
8. A second human SSOT under `docs/sdd/` or a new `docs/specs/repository-integrity-increment-2.md`
9. Rewriting ADR 0009 / 0010 decision bodies
10. Full ADR 0001 crate-dependency catalog as Guard 15 (Phase 19)
11. Creating `tests/sdd/`
12. Broad new debt rows except unavoidable, owned, expiring, justified exemptions for 05–12
13. Registering a new root `[[test]]` beyond the existing repository-integrity pair (ACP stays in `xtask/tests`)
14. Editing neighbor contract suites, IR, catalog TOML, or framework packs

---

## 18. Increment-2 related

- Accepted decision: [`docs/adr/0011-repository-guard-governance.md`](../adr/0011-repository-guard-governance.md)
- Accepted: [ADR 0009](../adr/0009-repository-health-gate.md), [ADR 0010](../adr/0010-architecture-as-law.md), [ADR 0004](../adr/0004-documentation-architecture.md)
- Neighbor program (Phases 17–18 pulled forward here): [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md)
- Prompt: [`docs/prompts/repository-cleanup-concurrent/01-repository-guard-governance.md`](../prompts/repository-cleanup-concurrent/01-repository-guard-governance.md)
