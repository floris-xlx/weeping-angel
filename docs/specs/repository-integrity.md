# SDD: Repository Integrity & Technical-Debt Reconciliation — Increment 1 (health gate)

| Field | Value |
| --- | --- |
| Status | **Implemented** — increment 1 health gate (`architecture/*.toml`, `docs/debt/register.toml`, `cargo xtask guard` checks 01/02/03/13, CI). Dual-suite target GREEN; baseline ignore-superseded. |
| Program | Repository Integrity & Technical-Debt Reconciliation |
| Slice | Increment 1 — sections 1–4: executable architecture health model (manifests + debt register + `xtask guard` + CI). **Not** P0 remediations. |
| Dual-suite | `sdd_repository_integrity_baseline` · `sdd_repository_integrity_target` (`tests/contracts/repository_integrity.{baseline,target}.rs`) — **not** auto-discovered; listed as `[[test]]` in root `Cargo.toml`. Baseline `#[ignore]`-superseded. `tests/sdd/` is forbidden ([ADR 0004](../adr/0004-documentation-architecture.md)) |
| ADR | **Accepted** [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md). Duplicate `0003-*` / `0005-*` / `0007-*` / `0008-*` IDs remain debt (`DEBT-DUP-ADR`); do **not** mint another `0003-*`. Successor: [ADR 0010](../adr/0010-architecture-as-law.md) (architecture-as-law). Next unique number is **0011**. |
| Public contract | This file. Assurance runtime public contract remains [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (untouched). |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — human SSOT is this file under `docs/specs/`. `docs/sdd/repository-integrity.md` is a pointer stub only. Generated traces go to `.sdd/`. |
| Neighbors (must stay GREEN after implement) | `sdd_documentation_layout`, `sdd_assurance_runtime_target`, `sdd_canonical_assurance_catalog_target`, `sdd_iso27001_assurance_target` |
| Collision fence | This slice does not implement P0 remediations. Guard **04** is implemented by the successor program ([ADR 0010](../adr/0010-architecture-as-law.md)); checks **05–12 / 14–15** stay stubs. Do not invent crates named `weeping-angel-catalog` or `weeping-angel-assurance-cli`. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `f560196c57e77df2573cfb9a4b384d3cf1c21e8a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) — this slice does not edit IR |
| `adr_needed` | **true** — new repo health gate and concept-ownership SSOT |
| Workspace verify | `cargo test -p xtask`; `cargo xtask guard`; `cargo test --test sdd_repository_integrity_target`; `cargo test --test sdd_repository_integrity_baseline -- --ignored`; `sdd_documentation_layout` GREEN (`CANONICAL_SPECS` includes this path) |

This document is the durable human SSOT for increment 1 of the Repository Integrity program. It owns **architecture manifests**, **canonical concept ownership**, **forbidden-pattern declarations**, **the technical-debt register**, **the 2026-08 live baseline snapshot**, **`cargo xtask guard` as the single repository health command**, and **mandatory CI wiring**.

It does **not** own P0 product remediations (framework expression preservation, catalog SSOT migration, lineage rebuild, SoA, persistence, package-install tests, ADR graph uniqueness as an *enforced* uniqueness rewrite, spec lifecycle states, deleting obsolete baseline suites, or a test-support crate). Those stay in [§8 remaining_backlog](#8-remaining_backlog-out-of-scope-for-this-slice).

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
| Guard checks 05–12 and 14–15 as real implementations | remaining_backlog — stub fail-closed or skip-with-debt only. Check **04** is [ADR 0010](../adr/0010-architecture-as-law.md). |

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
  → 04 evaluates invariants (ADR 0010); 05–12, 14–15 are explicit stubs (fail closed or skip only with a registered debt finding)
  → non-zero exit if any implemented check fails
```

Definition of done for this increment: *later P0 remediations cannot land without regression controls, because resolved debt without proof is rejected and CI cannot merge without the guard.*

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
| 14 | ADR graph / unique ADR IDs | Stub |
| 15 | Spec lifecycle states / crate dependency graph policy | Stub |

This slice did **not** implement 04–12 / 14–15 beyond the stub contract. Successor [ADR 0010](../adr/0010-architecture-as-law.md) implements **04** only.

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

Shipped stub policy after ADR 0010: checks **05–12 / 14–15** print `skip(DEBT-GUARD-NN)` when that finding exists; otherwise fail closed. Check **04** passes when every invariant evaluates. Guard exit 0 if implemented checks pass. `cargo clippy` / full `cargo test --features demo --all-targets` remain the existing CI commands (not `--workspace`); CI additionally runs `cargo xtask guard`. Root `cargo test --all-targets` does **not** run `cargo test -p xtask`.

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
13. ADR graph validator / unique ADR IDs (check 14) — **do not renumber** existing `0003-*` files here
14. Spec lifecycle states
15. Deleting obsolete baseline suites
16. Test-support crate
17. Remaining guard checks **05–12** and **14–15** beyond stubs (check **04** evaluates `invariants.toml` — ADR 0010)
18. Switching CI clippy/test to `--workspace`

---

## 9. Related

- Decision (Accepted): [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md)
- Successor (architecture-as-law): [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md), [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md)
- Docs layout: [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md)
- Crate graph: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md)
- Catalog crate: [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md)
- Debt register: [`docs/debt/register.toml`](../debt/register.toml)
- Pointer stub: [`docs/sdd/repository-integrity.md`](../sdd/repository-integrity.md)
