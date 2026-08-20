# SDD: Weeping Angel Structural Reconciliation Program — Phase 0 + Phase 1

| Field | Value |
| --- | --- |
| Status | **Implemented** — Phase 0+1: `cargo xtask inventory`, Historical `baseline-2026-08.md`, mechanical `docs/debt/current.md`, RI/debt active-plane reconcile, active-spec drift (Guard 15). Target suite GREEN; baseline skip-superseded. |
| Program | Structural Reconciliation Program (subtractive honesty: docs + debt evidence match the live tree). |
| Slice | **Phase 0** (feature freeze + exit criteria + absence characterization) + **Phase 1** (`cargo xtask inventory`, historical baseline marker, mechanical `docs/debt/current.md`, RI active/Historical reconcile, active-spec drift guard). **Not** later phases (no new frameworks / collectors / ISMS / report formats / product scanners). |
| Dual-suite | `xtask/tests/sdd_structural_reconciliation_{baseline,target}.rs` (Cargo auto-discovers `cargo test -p xtask`). **Do not** create `tests/sdd/` ([ADR 0004](../adr/0004-documentation-architecture.md) / `FORBID-TESTS-SDD`). Do not invent `test/sdd/*.ts`. Integrity dual-suite remains `tests/contracts/repository_integrity.{baseline,target}.rs` (root `[[test]]`); amend assertions only where this slice requires live-plane honesty. |
| ADR | **Accepted** [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md). Do **not** mint another `0003-*` / colliding `0011-*`. |
| Predecessor law | [`docs/specs/repository-integrity.md`](repository-integrity.md), [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md), [`docs/specs/repository-hygiene.md`](repository-hygiene.md), ADRs 0009–0012. |
| Public contract | Assurance runtime public contract remains [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (untouched). |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) — human SSOT is **this file** under `docs/specs/`. Optional pointer under `docs/sdd/` only. Generated traces go to `.sdd/`. |
| Neighbors (must stay GREEN after implement) | `sdd_documentation_layout` (additive `CANONICAL_SPECS` row), Guard **15** (`architecture/spec-lifecycle.toml` row), `sdd_repository_integrity_target` / `sdd_architectural_cleanup_target` after assertion/comment honesty amendments, `cargo xtask guard` |
| Collision fence | Subtractive / documentary / inventory-only. No new assurance frameworks, collectors, ISMS engines, SARIF/report formats, or product scanners. Do not invent `weeping-angel-catalog` or `weeping-angel-assurance-cli`. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Current plane (characterization) | `cargo xtask` accepts `guard` and `inventory` (`--json` / `--markdown` / `--check`); `xtask/src/inventory.rs` present; `docs/debt/current.md` mechanical; live `cargo xtask guard` prints **pass** for checks **01–15** (`ProductLawCheck` 05–12; `DEBT-GUARD-05`…`12` **resolved**); active RI / debt README match the live pass plane (stub archaeology under Historical) |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| `adr_needed` | **true** — inventory CLI + mechanical debt snapshot + active-spec drift as repository law |
| Workspace verify (after implement) | `cargo fmt --all -- --check`; `cargo xtask guard`; `cargo clippy --workspace --all-targets --features demo -- -D warnings`; `cargo test --workspace --features demo --all-targets`; `cargo test -p xtask` |

This document is the durable human SSOT for **Phase 0 + Phase 1** of the Structural Reconciliation Program. It owns:

- Phase 0 feature freeze and exit criteria
- executable absence characterization for inventory / `current.md` / RI prose drift
- `cargo xtask inventory` (`--json` / `--markdown` / `--check`) contract and exclusions
- historical marking of `docs/debt/baseline-2026-08.md`
- mechanical generation / check of `docs/debt/current.md`
- reconciliation of [`docs/specs/repository-integrity.md`](repository-integrity.md) so **active** Guards **05–12** / ADR / baseline language matches the live repo (archaeology under **Historical**)
- an **active-spec drift guard** against superseded-state phrases in active specs
- dual-suite protocol under `xtask/tests/`

It does **not** own new product collectors, framework packs, ISMS modules, scanner engines, or mass ADR renumber.

---

## 0. Collision fence

Phase 0+1 may change **only**:

| Concern | Home |
| --- | --- |
| This SSOT | `docs/specs/structural-reconciliation.md` |
| Draft/Accepted ADR | `docs/adr/0048-structural-reconciliation.md` |
| Inventory module + CLI wiring | `xtask/src/inventory.rs` + thin `xtask/src/lib.rs` / `main_with_args` registration |
| Debt evidence | `docs/debt/baseline-2026-08.md` (historical marker), `docs/debt/current.md` (generated), `docs/debt/README.md` (pointer honesty) |
| RI reconcile | `docs/specs/repository-integrity.md` active header / collision fence / current-plane language; archaeology moved under **Historical** |
| Active-spec drift | Guard check or inventory `--check` path defined in §4.6 (no new scanner product) |
| Dual-suite | `xtask/tests/sdd_structural_reconciliation_{baseline,target}.rs` |
| Neighbor index | `architecture/spec-lifecycle.toml`, `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` |
| Stale comments | RI / ACP / `docs/debt/README.md` / xtask comments that still say 05–12 are stubs **when claiming current law** |

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**`, framework pack semantics | Catalog / framework programs |
| `crates/weeping-angel-*` assurance/collector/evidence engines | Product programs |
| Scanner `src/**` recon / report / SARIF behavior | Root product |
| New Guard **product-semantic** laws beyond drift against superseded phrases | Out of scope |
| New frameworks, collectors, ISMS modules, report formats, product scanners | Never this program |
| `tests/sdd/`, `test/sdd/*.ts` | Forbidden |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | Never invent |
| Mass ADR renumber / ignore-baseline mass deletion | Hygiene / later ACP phases |

Toolchain scope: **Cargo** (workspace members + root `weeping-angel` + `xtask`) is primary. **pnpm** / `apps/docs` is out of scope.

---

## 1. Problem / user-visible goal

The live repository already enforces architecture as law: `cargo xtask guard` prints **pass** for checks **01–15**, including `ProductLawCheck` implementations for **05–12**, and `docs/debt/register.toml` marks `DEBT-GUARD-05`…`12` (and 14/15) **resolved**. Contributors and CI therefore treat product-law checks as real.

Yet **active** human surfaces still advertise archaeology as current law:

1. [`docs/specs/repository-integrity.md`](repository-integrity.md) header **Collision fence** and **Increment-2 current plane** still say Guards **05–12** stay stubs / **skip-with-debt**.
2. [`docs/debt/README.md`](../debt/README.md) **Stubbed guard checks** still says checks **05–12** and **14–15** may skip with `DEBT-GUARD-NN`.
3. [`docs/debt/baseline-2026-08.md`](../debt/baseline-2026-08.md) is still titled as a **live** counts snapshot, not a historical evidence artifact.
4. There is **no** `cargo xtask inventory`, **no** `xtask/src/inventory.rs`, and **no** mechanical `docs/debt/current.md` — so debt/count evidence cannot be regenerated or `--check`ed against the tree.
5. Stale “current plane” phrases in **active** specs (and some ACP comments) can reintroduce skip/stub archaeology without failing any guard.

**User-visible goal (program):** subtractive honesty — what active docs claim matches what `cargo xtask guard` and the debt register already do.

**User-visible goal (Phase 0+1):**

```text
Phase 0 freeze
  → no new frameworks / collectors / ISMS / report scanners
  → exit criteria locked in this SSOT

Phase 1
  cargo xtask inventory [--json|--markdown|--check]
        ↓
  docs/debt/current.md   (mechanical)
  docs/debt/baseline-2026-08.md  (Historical marker)
  docs/specs/repository-integrity.md  (active = live; archaeology = Historical)
  active-spec drift guard  (superseded-state phrases fail closed)
```

Definition of done for Phase 0+1: *a contributor can inventory the repo, regenerate or verify `docs/debt/current.md`, and trust that active RI/debt prose no longer claims Guards 05–12 are stubs — without adding new product frameworks.*

---

## 2. Compatibility / dependencies

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Health command | `cargo xtask guard` | Preserved. Inventory is an **additional** subcommand, not a replacement. |
| xtask CLI today | `main_with_args` | Only `guard [--json\|--check NN\|--explain INV-…]`. Extend with `inventory`. Unknown subcommands stay exit 2. |
| Debt register | `docs/debt/register.toml` | Do not reopen resolved `DEBT-GUARD-05`…`12`. Do not invent new product-semantic debt rows for this slice unless a live exemption is unavoidable. |
| RI dual-suite | `tests/contracts/repository_integrity.*` | Amend only assertions/comments that still encode 05–12 skip-as-current-law if they fail honesty after reconcile. Prefer moving archaeology to Historical in the spec over weakening live checks. |
| ACP suite | `xtask/tests/sdd_architectural_cleanup_*.rs` | Fix stale **comments**/messages that claim live 05–12 skip when the body already expects **pass**. Do not reintroduce stub skips. |
| Docs layout | `tests/contracts/documentation_layout.rs` | This path must be in `CANONICAL_SPECS`. |
| Spec lifecycle | `architecture/spec-lifecycle.toml` | This path must be listed (`state = "active"`, `ownership = ["repository_guard"]`, `depends_on` includes repository-integrity and/or architectural-cleanup). |
| Dual-suite discovery | `xtask/tests/*.rs` | Auto-discovered. No root `[[test]]` for this program’s executable law. |
| CI verify | existing | Keep green: `cargo fmt --all -- --check`; `cargo xtask guard`; `cargo clippy --workspace --all-targets --features demo -- -D warnings`; `cargo test --workspace --features demo --all-targets`. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Untouched. |

---

## 3. Current behavior (baseline — GREEN on CURRENT code before Phase 1 product)

§3 is the **absence / drift characterization**. Executable baseline tests in `xtask/tests/sdd_structural_reconciliation_baseline.rs` must **PASS on the pre-inventory tree** (found-case record). After Phase 1 target GREEN, that baseline suite is **deleted** (superseded by `sdd_structural_reconciliation_target`; INV-NO-SUPERSEDED-BASELINES).

Inclusion rule for counts (inventory + baseline snapshots): all matching paths under the repo root **excluding** `target/`, `target-*`, and `node_modules/` (and optionally `.sdd/` for walk performance; document in JSON `exclusions`).

### 3.1 xtask CLI surface

- `cargo xtask` / `main_with_args` accepts **only** `guard` with `--json`, `--check NN`, `--explain INV-…`.
- Any other first argument (including `inventory`) prints usage and exits **2**.
- There is **no** `xtask/src/inventory.rs` module and **no** `pub mod inventory` in `xtask/src/lib.rs`.

### 3.2 Debt evidence surfaces

| Path | Live state |
| --- | --- |
| `docs/debt/register.toml` | Present; `DEBT-GUARD-05`…`12`, `14`, `15` are `status = "resolved"` with `repository_guard` + regression tests |
| `docs/debt/baseline-2026-08.md` | Present; title reads **Live repository counts — 2026-08 implement snapshot** (not Historical) |
| `docs/debt/current.md` | **Absent** |
| `docs/debt/README.md` | Points at baseline as evidence; **Stubbed guard checks** section still claims 05–12 / 14–15 may skip |

### 3.3 Guard live plane vs active prose

Live `cargo xtask guard` (human render):

```text
01  architecture-manifest  pass
02  canonical-ownership  pass
03  forbidden-patterns  pass
04  architecture-invariants  pass
05  catalog-ssot  pass
06  framework-pack-parse  pass
07  framework-digest  pass
08  readiness-ssot  pass
09  temporal-evidence-selection  pass
10  assessment-lineage-rebuild  pass
11  evidence-latest-vs-current  pass
12  soa-invariants  pass
13  debt-register  pass
14  adr-graph  pass
15  spec-lifecycle  pass
```

`xtask/src/lib.rs` states implemented **01–15** with **no debt-backed skips**. `ProductLawCheck` covers 05–12. `STUB_EXEMPTION_CHECKS` is empty.

Active [`docs/specs/repository-integrity.md`](repository-integrity.md) still contains non-Historical claims such as:

- Collision fence: Guards **05–12** stay stubs/plumbing
- Increment-2 current plane: **05–12** skip-with-debt
- Shipped stub policy paragraphs that present 05–12 skip as current gate behavior

[`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md) Status/header still advertise 05–12 / 14–15 `skip(DEBT-GUARD-NN)` as shipped increment-1 current law without a Historical fence for the post–product-law plane.

### 3.4 Spec / inventory absences (found case)

Before this SSOT landed: no `docs/specs/structural-reconciliation.md`. After SSOT + lifecycle registration: the **spec** exists; **inventory product** and **`current.md`** remain absent until Phase 1 implement.

Baseline tests must still encode:

- no `inventory` subcommand behavior
- no `xtask/src/inventory.rs`
- no `docs/debt/current.md`
- active RI/debt README still contain superseded-state phrases listed in §4.6 (until reconcile)

### 3.5 Baseline suite IDs (must be GREEN on CURRENT pre-inventory code)

| ID | Characterization |
| --- | --- |
| SR-B01 | `main_with_args(["inventory"])` (and `--json` / `--markdown` / `--check`) exits 2 / usage; not a successful inventory |
| SR-B02 | `xtask/src/inventory.rs` is not a file |
| SR-B03 | `docs/debt/current.md` is not a file |
| SR-B04 | `docs/debt/baseline-2026-08.md` title/body present themselves as live counts (no Historical banner required by law yet) |
| SR-B05 | Active RI header/collision fence / current-plane strings still claim 05–12 stub or skip-with-debt |
| SR-B06 | `docs/debt/README.md` stub section still mentions 05–12 / 14–15 skip-with-debt |
| SR-B07 | Live guard checks 01–15 are `pass` (honesty hinge: code ahead of docs) |

---

## 4. Desired behavior (target — RED on CURRENT, GREEN after Phase 1 implement)

### 4.1 Phase 0 — feature freeze + exit criteria

**Freeze (no landing in Phase 0+1):**

1. New assurance frameworks or framework-pack formats
2. New collectors / provider adapters
3. New ISMS / risk / SoA product modules
4. New report formats or product security scanners
5. New Guard **product-semantic** checks beyond the active-spec drift rule in §4.6
6. Mass ADR renumber; mass `#[ignore]` baseline deletion
7. pnpm / `apps/docs` restructuring
8. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`

**Exit criteria (Phase 0+1 done only when all hold):**

1. `docs/specs/structural-reconciliation.md` is SSOT; registered in `CANONICAL_SPECS` + `architecture/spec-lifecycle.toml`
2. Draft ADR 0048 finalized **Accepted** with target GREEN proof
3. `cargo xtask inventory` works with `--json`, `--markdown`, and `--check` per §4.2–§4.4
4. `docs/debt/baseline-2026-08.md` is explicitly **Historical**
5. `docs/debt/current.md` exists and is mechanical (§4.5); `--check` fails on drift
6. Active RI / debt README language matches live Guards 01–15 pass; archaeology lives under **Historical**
7. Active-spec drift guard fails closed on superseded-state phrases in active specs (§4.6)
8. Dual-suite: baseline superseded; target GREEN under `xtask/tests/`
9. Verify set green: fmt, `cargo xtask guard`, clippy `--workspace … -D warnings`, `cargo test --workspace --features demo --all-targets`

### 4.2 `cargo xtask inventory` CLI

```text
cargo xtask inventory [--json | --markdown | --check]
```

| Mode | Behavior |
| --- | --- |
| (default) | Human summary to stdout (same metrics as JSON `counts`); exit 0 on success |
| `--json` | Print one JSON object (§4.3) to stdout; exit 0 |
| `--markdown` | Print markdown suitable for `docs/debt/current.md` body (§4.5); exit 0 |
| `--check` | Recompute inventory; compare to committed `docs/debt/current.md` (and enforce §4.6 if drift is wired here **or** via guard — see §4.6). Exit **0** if in sync; exit **1** on drift; exit **2** on usage error |

Mutually exclusive flags: if more than one of `--json` / `--markdown` / `--check` is passed, exit 2 with usage. Combining with `guard` flags is invalid.

Implementation home: `xtask/src/inventory.rs`, registered from `lib.rs` / `main_with_args`. Reuse walk helpers from `RepositoryModel` where practical; **do not** create a second filesystem framework crate.

### 4.3 Inventory JSON contract

Schema string: `weeping-angel/inventory/v1`.

Required top-level fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | string | Exactly `weeping-angel/inventory/v1` |
| `exclusions` | string array | At least `target/`, `target-*`, `node_modules/` |
| `counts` | object | Metrics below |
| `absences` | object | Booleans characterizing Phase 0 found-case **or** post-implement presence (see note) |

Required `counts` keys (integers ≥ 0):

| Key | Counts |
| --- | --- |
| `root_test_binaries` | Root `Cargo.toml` `[[test]]` tables |
| `tests_rs_autodiscovered` | `tests/*.rs` files (non-recursive) |
| `tests_contracts_rs` | `tests/contracts/*.rs` |
| `ignored_test_attrs` | Line-starting `#[ignore` attributes in `*.rs` under inclusion rule |
| `unwrap_calls` | `.unwrap()` in `*.rs` |
| `expect_calls` | `.expect(` in `*.rs` |
| `unwrap_plus_expect` | Sum of the two |
| `require_needles_fns` | Files / defs defining `fn require_needles` (match baseline methodology; document as file count of defs) |
| `require_needles_calls` | `require_needles(` occurrences |
| `adr_markdown_files` | `docs/adr/*.md` |
| `catalog_test_toml` | `catalog/canonical/v1/tests/*.toml` |
| `framework_packs` | Framework pack roots with `manifest.toml` (iso-27001/2022, wa-baseline/1 → **2** on current tree) |
| `schema_json_files` | `*.schema.json` under inclusion rule |

Required `absences` keys (bool) — **baseline found-case** uses `true` for missing product surfaces; after implement, inventory may omit claiming absences as debt, but `--json` must still expose:

| Key | Meaning |
| --- | --- |
| `inventory_module` | `xtask/src/inventory.rs` missing |
| `debt_current_md` | `docs/debt/current.md` missing |
| `structural_reconciliation_spec` | SSOT missing (false once this file exists) |

Optional but recommended: `git_sha` (string or null), `generated_at` (ISO-8601 UTC). Equality-sensitive `--check` must ignore wall-clock `generated_at` when comparing to `current.md` (compare counts + stable sections only).

### 4.4 Exclusions

Walks **must not** descend into:

- `target/`
- any `target-*` directory
- `node_modules/` (any depth)

Do not invent additional silent exclusions without documenting them in JSON `exclusions` and this SSOT.

### 4.5 `docs/debt/current.md` (mechanical)

- Generated by `cargo xtask inventory --markdown` (implement may also write a thin `include` header).
- Committed artifact; CI / contributors run `cargo xtask inventory --check`.
- Must state it is the **current** mechanical snapshot and that `baseline-2026-08.md` is **Historical**.
- Must embed the same `counts` as JSON (table form acceptable).
- Must not be hand-edited as SSOT; drift fails `--check`.

### 4.6 Historical baseline + RI reconcile + active-spec drift

**baseline-2026-08.md:** retitle / banner as **Historical** evidence from the 2026-08 implement snapshot. Not live status. `docs/debt/README.md` must point `current.md` as current counts and baseline as historical.

**repository-integrity.md:**

- Active header fields (Status, Collision fence, Increment-2 current plane, verify blurb) MUST match live law: Guards **01–15** implemented / pass on the healthy tree; `DEBT-GUARD-05`…`12` resolved; **no** active claim that 05–12 are stubs or skip-with-debt.
- Stub / skip-with-debt / increment-1 monolith archaeology MUST move under an explicit **Historical** section (or already-historical §3 / §12 characterization) so readers cannot mistake it for current gate behavior.
- Do not rewrite Accepted ADR 0009/0010/0011 decision bodies; amend RI prose and cross-links only.

**Active-spec drift guard:**

Fail closed when an **active** spec file (`architecture/spec-lifecycle.toml` `state = "active"`, or equivalently every `docs/specs/*.md` not marked superseded/retired) contains **superseded-state phrases** outside a Historical / characterization fence.

Minimum phrase set (case-sensitive enough to catch RI header drift; implement may use line/section heuristics documented in the target suite):

| Phrase / pattern | Why banned in active voice |
| --- | --- |
| `05–12` + `skip-with-debt` (same active paragraph/header) | Live 05–12 pass |
| `Guards **05–12** stay stubs` / `05–12 stay stubs` | ProductLawCheck landed |
| `Increment-2 current plane` claiming `05–12` skip | Plane superseded |
| `checks **05–12** and **14–15** may skip` as present tense debt README law | 14–15 and 05–12 resolved |
| `skip(DEBT-GUARD-05)` … `skip(DEBT-GUARD-12)` as describing live default gate | No longer default |

Allowed: those phrases inside sections explicitly titled **Historical**, **characterization**, **baseline (GREEN on …)**, or quoted as superseded found-case in dual-suite comments that are not human SSOT.

Enforcement seat (**shipped**; [ADR 0048](../adr/0048-structural-reconciliation.md)):

1. Folded into Guard **15** (`check_active_spec_drift_on_model` after lifecycle validation).
2. The same helper also runs from `cargo xtask inventory --check`.
3. No new Guard id; no product scanner.

### 4.7 Dual-suite target IDs

| ID | Obligation |
| --- | --- |
| SR-T01 | `inventory` module exists; `main_with_args(["inventory", …])` succeeds for json/markdown/check paths |
| SR-T02 | `--json` includes schema + required counts + exclusions |
| SR-T03 | `--markdown` matches committed `docs/debt/current.md` stable sections |
| SR-T04 | `--check` exit 0 on synced tree; exit 1 if `current.md` counts tampered in a fixture |
| SR-T05 | `baseline-2026-08.md` carries Historical marker; README points at `current.md` |
| SR-T06 | Active RI header/collision fence / current-plane do **not** claim 05–12 stub/skip; archaeology under Historical |
| SR-T07 | Active-spec drift: fixture/active text with banned phrases fails the drift check; Historical-fenced text does not |
| SR-T08 | Live `cargo xtask guard` still 01–15 pass; fmt/clippy/test verify set documented in header stays green |
| SR-T09 | Spec registered in `CANONICAL_SPECS` + `spec-lifecycle.toml` |
| SR-T10 | No `tests/sdd/`; suites live under `xtask/tests/` |

Protocol: write baseline (GREEN) → write target (RED) → implement → target GREEN → supersede baseline.

---

## 5. Acceptance criteria (testable)

- [x] Phase 0 freeze list (§4.1) is not violated by the Phase 1 diff (no new frameworks/collectors/ISMS/report scanners).
- [x] `xtask/src/inventory.rs` exists; `cargo xtask inventory` supports `--json`, `--markdown`, `--check` with exit codes in §4.2.
- [x] JSON matches §4.3 field set; exclusions include `target/`, `target-*`, `node_modules/`.
- [x] `docs/debt/current.md` is mechanical and `--check` fails on drift.
- [x] `docs/debt/baseline-2026-08.md` is Historical; README distinguishes historical vs current.
- [x] Active `docs/specs/repository-integrity.md` + `docs/debt/README.md` match live Guards 01–15 / resolved DEBT-GUARD-05…12; stub archaeology is Historical only.
- [x] Active-spec drift guard fails closed on superseded-state phrases (§4.6).
- [x] Dual-suite `sdd_structural_reconciliation_{baseline,target}` under `xtask/tests/`; baseline superseded after target GREEN.
- [x] Neighbor registration: `CANONICAL_SPECS` + `architecture/spec-lifecycle.toml`.
- [x] ADR 0048 Accepted; `cargo fmt --all -- --check`, `cargo xtask guard`, clippy workspace `-D warnings`, `cargo test --workspace --features demo --all-targets` green.
- [x] Subtractive-only: no new product scanners/frameworks/collectors/ISMS modules in the Phase 1 diff.

---

## 6. Out of scope

1. New framework packs, catalog families, or collectors
2. New ISMS / risk / remediation / audit product engines
3. New SARIF/report formats or root scanner features
4. Mass ADR renumber (`DEBT-DUP-ADR` remains)
5. Mass deletion of ignore-superseded baselines (ACP Phase 23 / hygiene)
6. Reopening resolved `DEBT-GUARD-05`…`12` as skip hatches
7. pnpm / `apps/docs` changes
8. Inventing `tests/sdd/` or `test/sdd/*.ts`
9. Inventing `weeping-angel-catalog` / `weeping-angel-assurance-cli`
10. Forking `assurance-ir/v1`
11. Replacing `cargo xtask guard` with inventory
12. Broad README capability rewrites unrelated to debt/inventory honesty
13. Implementing later Structural Reconciliation phases beyond 0+1 (if any are later catalogued)
14. Changing ProductLawCheck product semantics (05–12 stay real; this slice only reconciles docs/evidence)

---

## 7. Risks

- Hand-maintained `current.md` without `--check` in CI reintroduces drift; wire verify docs/scripts or rely on xtask tests calling `--check`.
- Over-broad phrase bans flag legitimate Historical sections; require an explicit Historical/characterization fence heuristic.
- Editing RI without moving archaeology may break citation anchors; prefer move/clarify over delete of Historical § numbers where dual-suites quote them.
- ACP/RI comment churn can look like product regressions; keep assertion bodies aligned with live **pass** semantics.
- Inventory walks that enter `target/` inflate counts and flake; exclusions are mandatory.
- Treating inventory as a new “scanner product” scope-creeps; keep metrics documentary.
- Finalizing ADR 0048 before target GREEN invites Accepted-without-proof; keep Draft until SR-T\* green.

---

## 8. Dual-suite and verify commands

```text
cargo test -p xtask --test sdd_structural_reconciliation_target
cargo xtask inventory --json
cargo xtask inventory --markdown
cargo xtask inventory --check
cargo xtask guard
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features demo -- -D warnings
cargo test --workspace --features demo --all-targets
```

Protocol:

1. Spec first (this file) + Draft ADR 0048.
2. Baseline GREEN on absences / prose drift (SR-B01–B07).
3. Target RED (SR-T01–T10).
4. Implement inventory + debt/RI reconcile + drift guard until target GREEN.
5. Delete superseded baseline; Accept ADR 0048.

---

## 9. Related

- Draft decision: [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md)
- [`docs/specs/repository-integrity.md`](repository-integrity.md)
- [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md)
- [`docs/specs/repository-hygiene.md`](repository-hygiene.md)
- [ADR 0004](../adr/0004-documentation-architecture.md), [ADR 0009](../adr/0009-repository-health-gate.md), [ADR 0010](../adr/0010-architecture-as-law.md), [ADR 0011](../adr/0011-repository-guard-governance.md), [ADR 0012](../adr/0012-repository-hygiene.md)
