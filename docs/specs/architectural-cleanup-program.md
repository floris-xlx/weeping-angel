# SDD: Weeping Angel architectural-cleanup PROGRAM — Increment 1 (Phase 0 freeze + Phase 1 architecture-as-law)

| Field | Value |
| --- | --- |
| Status | **Implemented** — increment 1 (Phase 0 freeze + Phase 1 architecture-as-law). Post–product-law plane: Guards **01–15** pass on the healthy tree (`ProductLawCheck` 05–12; 14/15 real). Historical increment-1 stub-skip archaeology is not current law. |
| Program | Architectural-cleanup PROGRAM (29 phases: **0–28**). One coordinated program, not unrelated refactors. |
| Slice | Increment 1 — **Phase 0** (freeze architectural expansion) + **Phase 1** (architecture-as-law: Guard 04 evaluation, ownership kinds, executable forbidden patterns, structured guard report/CLI). **Not** phases 2–28. |
| Dual-suite | `xtask/tests/*.rs` (Cargo auto-discovers `cargo test -p xtask`). **Do not** create `tests/sdd/` ([ADR 0004](../adr/0004-documentation-architecture.md) / `FORBID-TESTS-SDD`). Do not invent `test/sdd/*.ts`. Repo-wide SDD contracts historically live in `tests/contracts/` with root `Cargo.toml` `[[test]]` rows; **this increment’s executable law is `cargo test -p xtask`**. Neighbor `sdd_repository_integrity_target` expects check **04** pass/evaluated and live product-law checks **05–12** / **14–15** pass on the healthy tree. |
| ADR | **Accepted** [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md). Next unique number after [ADR 0009](../adr/0009-repository-health-gate.md). Do **not** mint another `0003-*`. Duplicate prefixes remain `DEBT-DUP-ADR`. |
| Predecessor law | [`docs/specs/repository-integrity.md`](repository-integrity.md) + ADR 0009 (health gate increment 1: manifests + debt + checks 01/02/03/13). This program **extends** that gate; it does not replace the assurance spine. |
| Public contract | Assurance runtime public contract remains [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (untouched this increment). |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) — human SSOT is **this file** under `docs/specs/`. [`docs/sdd/architectural-cleanup-program.md`](../sdd/architectural-cleanup-program.md) is the SDD run report, not a second SSOT. Generated traces go to `.sdd/`. |
| Neighbors (must stay GREEN after implement) | `sdd_documentation_layout` (`CANONICAL_SPECS` includes this path), `sdd_repository_integrity_target` (RI-T13 updated as specified), `sdd_assurance_runtime_target` |
| Collision fence | Do not implement catalog SSOT (Phase 2), framework parser/digest (Phase 3), evidence ledger `current()`/`as_of(t)` law (Phase 4), temporal move (Phase 5), AssessmentRun rebuild (Phase 6), readiness/SoA/explain (Phases 7–9), remaining guards 05–12 / 14–15 as real, ADR mass-renumber, or ignore-baseline deletion. Do not invent `weeping-angel-catalog` or `weeping-angel-assurance-cli`. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| `adr_needed` | **true** — architecture-as-law: `RepositoryModel` + `ArchitectureCheck`, ownership kinds, executable forbidden kinds, structured `GuardReport`/CLI |
| Workspace verify (after implement) | `cargo test -p xtask -- --nocapture`; `cargo xtask guard` (expect **01–15 pass**, exit 0) |

This document is the durable human SSOT for the **full 29-phase architectural-cleanup program** and for **increment 1 acceptance**. It owns:

- the program end-state pipeline and phase catalog (0–28)
- Phase 0 freeze (no new semantic SSoTs / engines / locations / numbering / grep frameworks)
- Phase 1 architecture-as-law (`RepositoryModel`, Guard **04** evaluating `architecture/invariants.toml`, ownership `kind`, executable `forbidden-patterns.toml`, structured guard CLI/report)
- dual-suite protocol under `xtask/tests/`
- debt-close rules for `DEBT-GUARD-04`

It does **not** own P0 product remediations listed in [§8](#8-out-of-scope-phases-2-28-and-phase-0-freeze). Those stay documented as later phases.

---

## 0. Collision fence (concurrent SDD)

Increment 1 may change **only** xtask + `architecture/*.toml` (+ debt/ADR/spec/neighbor RI-T13 as named). It must not rewrite assurance engines, catalog TOML, framework packs, IR types, or scanner CLI behavior.

| Do not touch | Owner / later phase |
| --- | --- |
| `catalog/canonical/v1/**`, dual catalog sources | Phase 2 / Guard 05 |
| `frameworks/**` pack parse, digest, expression preservation | Phase 3 / Guards 06–07 |
| Evidence ledger `current()` / `as_of(t)` as exclusive read APIs | Phase 4 / Guard 11 |
| Move `select_latest_as_of` into `weeping-angel-evidence` | Phase 5 / Guard 09 |
| `AssessmentRun` rebuild / immutable lineage | Phase 6 / Guard 10 |
| `readiness.rs` / `soa.rs` / explain as new implementations | Phases 7–9 / Guards 08, 12 |
| Remaining guard checks 05–12 and 14–15 as real (keep `DEBT-GUARD-*` skips) | Phases 2–8, 17–19 |
| ADR mass-renumber of existing `0003-*` / `0005-*` / `0007-*` / `0008-*` | Phase 27 (after Guard 14) |
| Deleting `#[ignore]` superseded baselines | Phase 23 |
| `tests/sdd/`, `test/sdd/*.ts` | Forbidden (ADR 0004 / `FORBID-TESTS-SDD`) |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | Never |

Suggested **implement** surfaces (increment 1 only):

| Concern | Home |
| --- | --- |
| `RepositoryModel`, `ArchitectureManifest`, `ArchitectureInvariant`, `ArchitectureCheck` | `xtask/src/` (modules; not independent greps) |
| Guard 04 evaluate every `[[invariant]]` | `xtask` check 04 |
| Ownership `kind` | `architecture/architecture.toml` + check 02 |
| Executable forbidden kinds | `architecture/forbidden-patterns.toml` + check 03 |
| Structured CLI/report | `xtask` `main_with_args` + `GuardReport` |
| Dual-suite | `xtask/tests/*baseline*.rs` + `xtask/tests/*target*.rs` |
| Close `DEBT-GUARD-04` | `docs/debt/register.toml` **after** Guard 04 tests exist and evaluate invariants |
| Neighbor RI-T13 | `tests/contracts/repository_integrity.target.rs` |
| Decision | ADR 0010 Accepted at implement |

---

## 1. Problem / user-visible goal

The workspace already has an inwardly extensible assurance runtime (seven crates + root CLI) and a **presence-only** health gate ([ADR 0009](../adr/0009-repository-health-gate.md)): `architecture/*.toml`, `docs/debt/register.toml`, and `cargo xtask guard` checks **01 / 02 / 03 / 13**. That gate does **not** yet treat architecture as law.

On the current tree:

1. `architecture/invariants.toml` is declared, but Guard **04** is `stub_check` → `skip(DEBT-GUARD-04)`. `INV-INVARIANTS-EVALUATED` explicitly says evaluation is `remaining_backlog`. File presence is not a claim that invariants hold.
2. Each implemented check walks the filesystem independently. There is no shared `RepositoryModel` (workspace, package graph, manifests, debt, ADR/spec metadata, packs, catalog sources). Later checks will fork into hand-written greps unless this increment introduces one evaluation plane.
3. Ownership rows have `crate` + `paths` only. They cannot say exclusive / facade / projection / adapter / shared-primitive (so `temporal_evidence_selection` cannot be declared exclusive while primitives still live in `weeping-angel-control-test`).
4. `forbidden-patterns.toml` already lists `kind = "package"|"path"` seeds (`weeping-angel-catalog`, `weeping-angel-assurance-cli`, `tests/sdd/`), but check **03** only requires file presence + schema. Kinds are not executed.
5. `GuardReport` is `{ checks }` plus `render()` text. No `violations`, `skipped`, `debt_exemptions`, `duration`. CLI is `guard` only — no `--json`, `--check NN`, `--explain INV-…`.
6. Without a freeze, later slices will invent new semantic SSoTs, framework interpretation engines, readiness implementations, temporal selection functions, catalog locations, baseline/target conventions, ADR numbering schemes, and source-grep “frameworks”.

**User-visible goal (program):** one pipeline, machine-enforced:

```text
Providers
  → Collectors
    → Canonical Evidence
      → Evidence Ledger (current() / as_of(t))
        → Canonical Tests
          → Control Assessments
            → Applicability + Risk/ISMS
              → immutable AssessmentRun lineage
                → Readiness / SoA / Explain
                  → Framework Projection
```

Enforcement plane (already started by ADR 0009, completed by this program):

```text
architecture/*.toml + docs/debt/register.toml + docs/adr + docs/specs
        ↓
cargo xtask guard   (ArchitectureCheck against RepositoryModel; no silent skips)
        ↓
CI
```

**User-visible goal (this increment):** a contributor runs `cargo xtask guard` and gets a structured report in which check **04 actually evaluates** every invariant against a loaded `RepositoryModel`; ownership rows carry kinds; forbidden pattern kinds execute; skips always cite a live debt ID; `--json` / `--check` / `--explain` work. Phases 2–28 remain documented backlog.

Definition of done for increment 1: *architecture is executable law for invariants, ownership kinds, and forbidden patterns, without expanding semantic SSoTs or implementing the rest of the pipeline.*

---

## 2. Compatibility / dependencies

Pinned to the tree as characterized in [§3](#3-current-behavior-baseline--green-on-current-code).

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Workspace members | root [`Cargo.toml`](../../Cargo.toml) | Keep seven `weeping-angel-*` crates + `xtask`. Do not add `weeping-angel-catalog` or `weeping-angel-assurance-cli`. |
| Health command | `xtask` + `.cargo/config.toml` alias | Keep `cargo xtask guard`. Extend flags. `publish = false`, `[package.metadata.dist] dist = false`. |
| Predecessor checks | ADR 0009 | Checks 01, 02, 13 stay real. Check 03 stays real and **gains kind execution**. Check 04 becomes real. 05–12 / 14–15 stay `stub_check` / skip-with-debt. |
| Debt | `docs/debt/register.toml` | `DEBT-GUARD-04` is **resolved** (`repository_guard = "04"`, `regression_tests = ["sdd_architectural_cleanup_target"]`). Other `DEBT-GUARD-*` stay open. |
| Neighbor RI-T13 | `tests/contracts/repository_integrity.target.rs` | **04 is pass/evaluated**; 05–12 / 14–15 stay skip-with-debt or fail closed. |
| Docs layout | `tests/contracts/documentation_layout.rs` | This spec path is in `CANONICAL_SPECS`. |
| ADR numbering | `docs/adr/` | Unique file is **0010**. Next unused unique number is **0011**. Do not add `0003-architecture-as-law.md`. |
| Dual-suite discovery | `xtask/tests/*.rs` | Auto-discovered by `cargo test -p xtask`. Do **not** add root `[[test]]` rows for this increment. Do **not** put suites in `tests/contracts/` for this increment’s executable law. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Untouched. |
| Product crates | `crates/**`, `src/**` | Untouched this increment (no catalog/ledger/temporal/lineage/readiness code). |

---

## 3. Current behavior (baseline — GREEN on pre-increment-1 code)

§3 is the **characterization** of the ADR 0009 presence-only gate (checks 01/02/03/13 real; 04 stub). After implement it is historical: increment-1 baselines are `#[ignore]`-superseded. Shipped law is [§4.5](#45-shipped-increment-1).

Executable characterization **must** live in `xtask/tests/` baseline tests that **PASS on CURRENT stub/skip/presence-only behavior before any product change**. Dedicated suite: [`xtask/tests/sdd_architectural_cleanup_baseline.rs`](../../xtask/tests/sdd_architectural_cleanup_baseline.rs) (ACP-B01–B06). Existing `xtask/tests/debt_register.rs` is part of that characterization:

- `guard_on_fixture_repo_runs_implemented_checks_and_skips_stubs` expects 01/02/03/13 pass and **`skip(DEBT-GUARD-04)`** (and 05–12, 14, 15).
- `stub_without_debt_finding_fails_closed` expects `not-yet-implemented: check 04` when `DEBT-GUARD-04` is missing.

### 3.1 Guard catalog

[`xtask/src/lib.rs`](../../xtask/src/lib.rs):

| ID | Name | Current implementation |
| --- | --- | --- |
| 01 | `architecture-manifest` | File present + `schema = weeping-angel/architecture/v1` |
| 02 | `canonical-ownership` | Seven concept keys; `crate` + non-empty `paths` that exist; live crate names; reject hypothetical packages |
| 03 | `forbidden-patterns` | File present + `schema = weeping-angel/forbidden-patterns/v1` only. **`[[pattern]]` kinds are not executed.** |
| 04 | `architecture-invariants` | `STUB_CHECKS` → `stub_check` → `skip(DEBT-GUARD-04)` or fail closed |
| 05–12, 14–15 | catalog-ssot … spec-lifecycle | Same stub path (`DEBT-GUARD-NN`) |
| 13 | `debt-register` | Schema, unique ids, resolved-without-proof rejected |

There is **no** function named `skip_guard_check`; the skip path is `stub_check`.

### 3.2 Types and CLI

```text
CheckStatus = Pass | Fail(String) | Skip { debt_id }
CheckResult { id, name, status }  // report_line, is_fail
GuardReport { checks }            // render(); failed() = any Fail
```

`GuardReport` has **no** `violations`, `skipped`, `debt_exemptions`, or `duration`.

[`xtask/src/lib.rs`](../../xtask/src/lib.rs) `main_with_args`:

- `guard` → `run_guard` → print `render()` → exit 0 if no Fail, else 1
- anything else → `usage: cargo xtask guard` → exit 2

No `--json`, `--check`, `--explain`. `xtask/Cargo.toml` depends on `toml` only (no `serde_json`).

### 3.3 No RepositoryModel / ArchitectureCheck

Checks 01/02/03/13 each `read_toml` independently. There is no:

- `RepositoryModel`
- `ArchitectureManifest` / `ArchitectureInvariant` / `InvariantResult`
- `trait ArchitectureCheck { fn check(&self, repo: &RepositoryModel) -> CheckResult }`

Invariant rows are **not** parsed for evaluation. Check 04 never opens `architecture/invariants.toml` except insofar as check 01’s sibling files exist on disk (04 does not read them).

### 3.4 Ownership rows: crate + paths only

[`architecture/architecture.toml`](../../architecture/architecture.toml) required concepts (hard-coded `REQUIRED_OWNERSHIP`):

| Concept | Package | Paths |
| --- | --- | --- |
| `catalog` | `weeping-angel-canonical-catalog` | `crates/weeping-angel-canonical-catalog` |
| `framework_compilation` | `weeping-angel-framework` | `crates/weeping-angel-framework` |
| `readiness_projection` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/readiness.rs` |
| `temporal_evidence_selection` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/temporal.rs` |
| `assessment_lineage` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/lineage.rs` |
| `evidence_persistence` | `weeping-angel-evidence` | `crates/weeping-angel-evidence` |
| `assurance_cli` | `weeping-angel` | `src/main.rs`, `src/cli.rs` |

No `kind` field. Comment on temporal: primitives also exist in `weeping-angel-control-test::temporal::select_latest_as_of`; ownership is declared on the assurance facade; code was not moved.

### 3.5 Forbidden patterns: declared, not executed

[`architecture/forbidden-patterns.toml`](../../architecture/forbidden-patterns.toml):

| id | kind | value |
| --- | --- | --- |
| `FORBID-HYPOTHETICAL-CATALOG` | `package` | `weeping-angel-catalog` |
| `FORBID-HYPOTHETICAL-ASSURANCE-CLI` | `package` | `weeping-angel-assurance-cli` |
| `FORBID-TESTS-SDD` | `path` | `tests/sdd/` |

File comment: “Enforcing grep/AST matches is remaining_backlog.” Check 03 ignores `[[pattern]]` entirely.

### 3.6 Invariants: evaluation is remaining_backlog

[`architecture/invariants.toml`](../../architecture/invariants.toml):

| id | guard_check | Notes |
| --- | --- | --- |
| `INV-OWNERSHIP-LIVE-CRATES` | 02 | Declared; 02 already checks live crates/paths |
| `INV-NO-HYPOTHETICAL-PACKAGES` | 02 | Declared; 02 already rejects those crate names in ownership |
| `INV-DEBT-RESOLVED-HAS-PROOF` | 13 | Declared; 13 already enforces proof law |
| `INV-INVARIANTS-EVALUATED` | 04 | **summary = "Evaluating this file against the tree is remaining_backlog"** |

### 3.7 Debt seeds

`docs/debt/register.toml` then had open `DEBT-GUARD-04` … `12`, `14`, `15` plus P0 and hygiene findings. `DEBT-GUARD-04` summary (characterization): “Architecture invariants.toml is declared but not evaluated. Check 04 is a stub this increment.” After implement, `DEBT-GUARD-04` is **resolved** (see [§4.5](#45-shipped-increment-1)).

### 3.8 Pipeline as-built (not this increment’s job to finish)

| Stage | Current home (law is tribal / split) |
| --- | --- |
| Providers / collectors | `weeping-angel-collector` (fixture + GitHub); collectors emit envelopes, not compliance |
| Canonical evidence | `weeping-angel-evidence` envelopes; schema `evidence/v1` |
| Evidence ledger | `EvidenceLedger` with `latest_as_of` — **no** `current()` API |
| Temporal selection | **Split:** `weeping-angel-control-test::temporal::select_latest_as_of` + assurance `temporal.rs` facade |
| Canonical tests | `weeping-angel-control-test` |
| Assessments / lineage / readiness / SoA / explain | `weeping-angel-assurance` (`lineage.rs`, `readiness.rs`, `soa.rs`, `snapshot.rs` `AssessmentRun`) |
| Catalog | `weeping-angel-canonical-catalog` + `catalog/canonical/v1` |
| Framework compile / packs | `weeping-angel-framework` + `frameworks/` |
| CLI | root package `weeping-angel` (`src/main.rs`, `src/cli.rs`) |

`EvidenceLedger::current()` does not exist. Temporal exclusive ownership is not machine-checked.

---

## 4. Desired behavior

### 4.1 Program end-state (all 29 phases — document now, implement later)

Normative pipeline (do not invert; do not skip stages):

```text
Providers → Collectors → Canonical Evidence
  → Evidence Ledger (current() / as_of(t))
  → Canonical Tests → Control Assessments
  → Applicability + Risk/ISMS
  → immutable AssessmentRun lineage
  → Readiness / SoA / Explain
  → Framework Projection
```

Governing rules (ADR 0001 still law):

- A finding is not a compliance result.
- A collector cannot declare compliance.
- A framework cannot perform network I/O.
- A control-test cannot know which provider produced its evidence.
- A crosswalk cannot manufacture equivalence through graph traversal.

Enforcement: `architecture/*.toml` + debt + ADRs + specs are evaluated by `cargo xtask guard` against one `RepositoryModel` and are mandatory in CI.

#### Phase catalog (0–28 = 29 phases)

| Phase | Name | Guard / debt | This increment |
| --- | --- | --- | --- |
| **0** | Architectural freeze | — | **In scope (spec/out-of-scope law)** |
| **1** | Architecture-as-law (`RepositoryModel`, Guard 04, ownership kinds, forbidden kinds, structured CLI) | 04; `DEBT-GUARD-04` resolved with proof | **Done** |
| 2 | Catalog SSOT (one catalog location; no dual sources) | 05 / `DEBT-P0-CATALOG-SSOT` | later |
| 3 | Framework pack parse fail-closed + digest redesign | 06, 07 / `DEBT-P0-PACK-PARSE`, `DEBT-P0-FRAMEWORK-DIGEST` | later |
| 4 | Evidence Ledger law: `current()` and `as_of(t)` as the only temporal read APIs | 11 / `DEBT-P0-EVIDENCE-LATEST` | later |
| 5 | Temporal selection exclusive owner `weeping-angel-evidence` (move off assurance/control-test split) | 09 | later |
| 6 | Immutable `AssessmentRun` lineage rebuild | 10 / `DEBT-P0-LINEAGE-REBUILD` | later |
| 7 | Readiness as projection (not a second SSOT) | 08 / `DEBT-P0-READINESS-SSOT` | later |
| 8 | SoA as projection + invariants | 12 / `DEBT-P0-SOA` | later |
| 9 | Explain as projection of lineage + assessments (no new interpretation engine) | — | later |
| 10 | Framework Projection (adapters only; no framework interpretation engines) | — | later |
| 11 | Collectors normalize-only (Providers → Collectors → facts) | INV-2 | later |
| 12 | Canonical Evidence envelope law (digest, validity, no conclusions) | `DEBT-P0-PERSISTENCE` (partial) | later |
| 13 | Canonical Tests ownership (EvidenceSet in, no provider I/O) | INV-4 | later |
| 14 | Control Assessments consume tests; no framework I/O | — | later |
| 15 | Applicability as law (consumes assessments + scope; not a catalog fork) | — | later |
| 16 | Risk / ISMS consume assessments; no parallel catalogs | — | later |
| 17 | ADR graph uniqueness (enforce unique IDs; **do not mass-renumber yet**) | 14 / `DEBT-DUP-ADR`, `DEBT-GUARD-14` | later |
| 18 | Spec lifecycle states | 15 / `DEBT-GUARD-15` | later |
| 19 | Crate dependency graph policy (ADR 0001 forbidden edges as architecture law) | 15 (partial) | later |
| 20 | Persistence invariants in `weeping-angel-evidence` | `DEBT-P0-PERSISTENCE` | later |
| 21 | Framework expression preservation | `DEBT-P0-FRAMEWORK-EXPRESSION` | later |
| 22 | CI `--workspace` bar; `cargo test -p xtask` in CI | — | later |
| 23 | Dual-suite hygiene / ignore-baseline deletion protocol | `DEBT-IGNORE` | later |
| 24 | Schema fixtures (IR / catalog / packs JSON Schema) | `DEBT-SCHEMA-DUP` (adjacent) | later |
| 25 | Test-support crate (not `tests/sdd/`) | — | later |
| 26 | Package install tests (xtask never distributed) | — | later |
| 27 | ADR mass-renumber **after** Guard 14 exists | 14 | later |
| 28 | Debt closure for remaining `DEBT-GUARD-*` / P0 rows with proof | 13 | later |

Phases 2–28 are specified here so later increments cannot invent a different end-state. They are **not** implemented in increment 1.

### 4.2 Phase 0 — freeze (this increment, spec law)

Until a later phase **explicitly** opens them, increment 1 and all concurrent work **must not** introduce:

1. New semantic SSoTs (second catalog, second debt register, second ownership table, second invariant file).
2. Framework interpretation engines (logic that “understands” ISO/GDPR/SOC2 outside `weeping-angel-framework` compile).
3. New readiness implementations (do not expand `readiness.rs` semantics).
4. New temporal selection functions (do not add `current()` / move `select_latest_as_of` this increment).
5. New catalog locations (do not add a parallel `catalog/` tree or crate).
6. New baseline/target conventions (this increment’s dual-suite stays under `xtask/tests/*.rs`; repo-wide contracts stay `tests/contracts/` + root `[[test]]`; never `tests/sdd/`).
7. New ADR numbering schemes (next unique id is **0011**, then 0012, …; do not mint `0003-*`).
8. Hand-written source-grep frameworks (guards evaluate `RepositoryModel` + data in `architecture/*.toml`; `kind = "source-pattern"` is data-driven from the forbidden file, not a new grep crate).

Phase 0 is **not** a new code module. It is out-of-scope law + review bar.

### 4.3 Phase 1 — architecture-as-law (this increment, **shipped**)

#### 4.3.1 Shared model and check trait (mandatory shape)

Introduce internal APIs **resembling** (names may match exactly; behavior must):

```rust
pub struct RepositoryModel { /* loaded once per run_guard */ }

pub struct ArchitectureManifest { /* architecture.toml including ownership kinds */ }

pub struct ArchitectureInvariant {
    pub id: String,       // e.g. INV-INVARIANTS-EVALUATED
    pub summary: String,
    pub guard_check: String, // "02" | "04" | "13" | …
}

pub struct InvariantResult { /* per-row pass/fail/skip with message */ }

pub trait ArchitectureCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult;
}
```

`RepositoryModel` is a snapshot of:

- Cargo workspace members (from root `Cargo.toml`)
- package graph (each member `Cargo.toml` dependencies)
- filesystem index (paths cited by ownership, forbidden patterns, specs, ADRs)
- architecture manifests (`architecture.toml`, `invariants.toml`, `forbidden-patterns.toml`)
- debt register (`docs/debt/register.toml`)
- ADR metadata (`docs/adr/*.md` filenames / ids — **read, do not renumber**)
- spec metadata (`docs/specs/*.md`, `CANONICAL_SPECS` if loaded)
- framework pack locations (`frameworks/**`)
- catalog sources (`catalog/canonical/**`, catalog crate)

`run_guard(root)` loads **one** `RepositoryModel`, then runs each `ArchitectureCheck`. **Do not** implement each guard as an independent filesystem grep.

#### 4.3.2 Phase 1.1 — Guard 04 evaluates every invariant

Check **04** (`architecture-invariants`):

1. Parse `architecture/invariants.toml` (`schema = weeping-angel/architecture-invariants/v1`).
2. Require a non-empty `[[invariant]]` array; each row has non-empty `id`, `summary`, `guard_check`.
3. **Evaluate every invariant** against `RepositoryModel`. Presence of the file is not a pass.
4. Map `guard_check` to already-computed check results and/or model predicates. At minimum:
   - `INV-OWNERSHIP-LIVE-CRATES`: ownership crates are workspace members; paths exist.
   - `INV-NO-HYPOTHETICAL-PACKAGES`: no workspace member named `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
   - `INV-DEBT-RESOLVED-HAS-PROOF`: check 13 pass (or equivalent model predicate).
   - `INV-INVARIANTS-EVALUATED`: this check (04) actually ran evaluations; **rewrite the row** so it no longer claims evaluation is `remaining_backlog`. A legal summary: every `[[invariant]]` is evaluated against `RepositoryModel`; skip is illegal without a live debt id.
5. Fail closed if any invariant fails. Report per-invariant outcomes (text and `--json`).
6. `--explain INV-…` prints that invariant’s id, summary, guard_check, evaluation evidence, and result.

Closing `DEBT-GUARD-04`:

- Allowed **only after** Guard 04 tests exist and evaluate invariants.
- Proof: non-empty `regression_tests` (xtask target tests) **or** `repository_guard = "04"`.
- Check 13 continues to reject resolved-without-proof.

#### 4.3.3 Phase 1.2 — Ownership kinds

Extend every `[ownership.*]` row:

```toml
[ownership.temporal_evidence_selection]
crate = "weeping-angel-assurance"
kind = "exclusive"   # exclusive | facade | projection | adapter | shared-primitive
paths = ["crates/weeping-angel-assurance/src/temporal.rs"]
```

Closed set of `kind`:

| kind | Meaning |
| --- | --- |
| `exclusive` | Named crate is the only legal owner of the concept. Other crates must not grow a competing SSOT this program (duplicates may remain as **debt** until the phase that moves code). |
| `facade` | Named crate orchestrates / re-exports; not the primitive store. |
| `projection` | Derived view of assessments/ledger (readiness, SoA, explain). |
| `adapter` | Maps provider/framework/external types into canonical types. |
| `shared-primitive` | Primitive may be referenced from more than one crate; the row still names the primary home. |

Check **02** must require `kind` on all seven mandatory concepts, validate the enum, and keep live crate + existing path rules. Increment 1 **does not move** `select_latest_as_of`. Declaring `ownership.temporal_evidence_selection.kind = "exclusive"` is metadata for Phase 5, not a code move.

Recommended kinds (implement may match this table):

| Concept | kind |
| --- | --- |
| `catalog` | `exclusive` |
| `framework_compilation` | `exclusive` |
| `readiness_projection` | `projection` |
| `temporal_evidence_selection` | `exclusive` |
| `assessment_lineage` | `exclusive` |
| `evidence_persistence` | `exclusive` |
| `assurance_cli` | `facade` |

#### 4.3.4 Phase 1.3 — Forbidden patterns executable

Check **03** parses **and executes** `[[pattern]]` rows against `RepositoryModel`.

Closed set of `kind`:

| kind | Evaluation |
| --- | --- |
| `package` | No workspace member / package name equals `value` |
| `path` | `value` must not exist as a file or directory under repo root |
| `dependency` | `value` names a forbidden edge (`from -> to` or documented equivalent); must not appear in the package graph |
| `symbol` | Named symbol must not appear in forbidden crates/paths recorded on the row (optional extra fields allowed; unknown required fields fail closed) |
| `source-pattern` | Pattern applied via the model’s source index (data from toml), **not** a new grep framework crate |

Seeds already in tree **must fail the check if violated**:

- `weeping-angel-catalog` (package)
- `weeping-angel-assurance-cli` (package)
- `tests/sdd/` (path)

Missing `kind` / unknown `kind` / empty `value` fail closed.

#### 4.3.5 Phase 1.4 — Structured guard output and CLI

```rust
pub struct GuardReport {
    pub checks: Vec<CheckResult>,
    pub violations: Vec</* fail details */>,
    pub skipped: Vec</* skip details */>,
    pub debt_exemptions: Vec</* live debt ids used for skips */>,
    pub duration: /* non-zero elapsed for a real run */,
}
```

CLI (still `cargo xtask guard` by default):

| Invocation | Behavior |
| --- | --- |
| `cargo xtask guard` | Run all checks; human report; exit 0 iff no Fail (skips are not Fail) |
| `cargo xtask guard --json` | Same evaluation; JSON object with the `GuardReport` fields |
| `cargo xtask guard --check 09` | Run the named check (and its model load / debt validation as required so skip-with-debt still works); do not silently skip others without listing them |
| `cargo xtask guard --explain INV-…` | Explain one invariant (load model, evaluate that id) |

No silent skips. Every skip **must** cite a live finding id from the loaded debt register (`skip(DEBT-GUARD-NN)` and `debt_exemptions`). If the id is missing, that check **fails closed** (`not-yet-implemented: check NN` or equivalent).

Human `render()` remains stable enough that existing 01/02/03/13 pass lines and `skip(DEBT-GUARD-NN)` still appear. JSON is additive.

Checks **05–12** and **14–15** remain stubs with live `DEBT-GUARD-*` ids. Check **04** must **not** skip after implement.

#### 4.3.6 Neighbor RI-T13

[`tests/contracts/repository_integrity.target.rs`](../../tests/contracts/repository_integrity.target.rs) `ri_t13_stub_checks_do_not_silently_pass` **shipped**:

- **04** is pass / evaluated (not skip-or-nyi)
- **05–12** and **14–15** still skip-with-debt or fail closed

Do not weaken the no-silent-skip law.

### 4.4 Dual-suite protocol (mandatory)

Protocol (this increment, not optional):

1. **Spec first** (this file + draft ADR 0010). No Guard 04 / kinds / kind-execution / JSON CLI product code in the spec-first commit.
2. **Baseline GREEN** on CURRENT skip/stub/presence-only behavior (`xtask/tests/` characterization, including `debt_register.rs` expecting `skip(DEBT-GUARD-04)`).
3. **Target RED** on CURRENT code for: Guard 04 evaluating invariants against `RepositoryModel`; ownership kinds; executable forbidden kinds; structured CLI/report (`GuardReport` fields + `--json` / `--check` / `--explain`).
4. **Implement** only xtask + `architecture/*.toml` (+ debt close, ADR Accepted, RI-T13, `debt_register.rs` fixture update) until target GREEN.
5. **Supersede** increment-1 baselines with `#[ignore = "superseded by …"]` **or** prove they FAIL on the new code. Do **not** this increment mass-delete ignored baselines (`DEBT-IGNORE` / Phase 23).
6. Keep this spec in `CANONICAL_SPECS`.
7. Set ADR 0010 to **Accepted**.

Target tests (must fail today; pass after implement). Suggested IDs (names may vary; assertions must):

| ID | Assertion |
| --- | --- |
| ACP-T01 | `RepositoryModel` loads workspace, package graph, filesystem, manifests, debt, ADR metadata, spec metadata, framework packs, catalog sources |
| ACP-T02 | Checks implement `ArchitectureCheck::check(&self, repo: &RepositoryModel)` (not independent greps for 04) |
| ACP-T03 | Guard 04 parses `invariants.toml` and **evaluates every** `[[invariant]]` |
| ACP-T04 | `INV-INVARIANTS-EVALUATED` no longer claims evaluation is `remaining_backlog`; 04 pass requires evaluation |
| ACP-T05 | Fixture missing an invariant evaluation fails check 04 |
| ACP-T06 | Every `[ownership.*]` mandatory row has `kind` ∈ exclusive\|facade\|projection\|adapter\|shared-primitive |
| ACP-T07 | `ownership.temporal_evidence_selection` may be `exclusive` without moving `select_latest_as_of` this increment |
| ACP-T08 | Check 03 executes `kind=package` and rejects `weeping-angel-catalog` / `weeping-angel-assurance-cli` if present as members |
| ACP-T09 | Check 03 executes `kind=path` and rejects existing `tests/sdd/` |
| ACP-T10 | Unknown forbidden `kind` fails closed |
| ACP-T11 | `GuardReport` has `checks`, `violations`, `skipped`, `debt_exemptions`, `duration` |
| ACP-T12 | `cargo xtask guard --json` emits those fields |
| ACP-T13 | `cargo xtask guard --check 09` runs check 09 (skip-with-debt or fail closed, never silent) |
| ACP-T14 | `cargo xtask guard --explain INV-…` explains an invariant |
| ACP-T15 | After implement, `cargo xtask guard` : 04 pass; 05–12 / 14–15 `skip(DEBT-GUARD-NN)` with live ids; exit 0 |
| ACP-T16 | `DEBT-GUARD-04` is `resolved` only with `regression_tests` or `repository_guard = "04"` |
| ACP-T17 | Dual-suite lives under `xtask/tests/*.rs`; no `tests/sdd/` |

Baseline tests (must pass **before** product changes):

| ID | Assertion |
| --- | --- |
| ACP-B01 | Check 04 currently skips with `DEBT-GUARD-04` when that finding exists (`debt_register.rs` fixture behavior) |
| ACP-B02 | Check 03 currently passes on schema presence even when `[[pattern]]` kinds are not executed |
| ACP-B03 | Ownership rows currently need only `crate` + `paths` (no `kind`) |
| ACP-B04 | `GuardReport` currently has only `checks` + `render`; CLI is `guard` only |
| ACP-B05 | `INV-INVARIANTS-EVALUATED` currently says evaluation is `remaining_backlog` |
| ACP-B06 | Stub without debt fails closed (`not-yet-implemented: check 04`) |

After implement, update `guard_on_fixture_repo_runs_implemented_checks_and_skips_stubs` so the fixture includes `invariants.toml` (and kinds / patterns as required) and expects **04 pass**, not `skip(DEBT-GUARD-04)`.

### 4.5 Shipped increment 1

Implemented in `xtask/src/lib.rs` + `architecture/*.toml` + `docs/debt/register.toml`:

| Surface | Law |
| --- | --- |
| Evaluation plane | `run_guard` loads one `RepositoryModel`; checks implement `ArchitectureCheck::check` |
| Check 04 | Evaluates every `[[invariant]]`; unknown id / remaining_backlog summary on `INV-INVARIANTS-EVALUATED` fails |
| Check 02 | Requires `kind` ∈ exclusive\|facade\|projection\|adapter\|shared-primitive |
| Check 03 | Executes package\|path\|dependency\|symbol\|source-pattern |
| Report | `GuardReport { checks, violations, skipped, debt_exemptions, duration }` |
| CLI | `cargo xtask guard [--json] [--check NN] [--explain INV-…]` |
| Debt | `DEBT-GUARD-04` resolved with proof; 05–12 / 14–15 still skip-with-live-id |

Shipped ownership kinds: `catalog` exclusive, `framework_compilation` exclusive, `readiness_projection` projection, `temporal_evidence_selection` exclusive (no code move), `assessment_lineage` exclusive, `evidence_persistence` exclusive, `assurance_cli` facade.

---

## 5. Acceptance criteria (testable) — increment 1

- [x] Dual-suite exists under `xtask/tests/*.rs` (not `tests/sdd/`). Baseline PASS on current skip/stub/presence-only behavior **before** product changes; target FAIL on current code for Guard 04 evaluation, ownership kinds, forbidden-kind execution, and structured CLI/report.
- [x] `run_guard` loads one `RepositoryModel` and runs `ArchitectureCheck` implementations. Guard 04 is not an independent grep and is not `stub_check`.
- [x] Guard 04 parses `architecture/invariants.toml` and evaluates **every** `[[invariant]]` against the model. `INV-INVARIANTS-EVALUATED` is rewritten so evaluation is no longer `remaining_backlog`.
- [x] Ownership rows include `kind` ∈ `exclusive|facade|projection|adapter|shared-primitive`; check 02 requires it. `ownership.temporal_evidence_selection` example kind is `exclusive` without moving temporal code.
- [x] Check 03 executes forbidden kinds `package|path|dependency|symbol|source-pattern` and rejects `weeping-angel-catalog`, `weeping-angel-assurance-cli`, and `tests/sdd/` when those exist.
- [x] `GuardReport` includes `checks`, `violations`, `skipped`, `debt_exemptions`, `duration`. CLI supports `guard`, `--json`, `--check NN`, `--explain INV-…`. No silent skips; every skip cites a live debt id.
- [x] `cargo xtask guard` exit 0: 01, 02, 03, **04**, 13 pass; 05–12 and 14–15 `skip(DEBT-GUARD-NN)` with live ids.
- [x] `DEBT-GUARD-04` closed only after Guard 04 tests evaluate invariants (`regression_tests` or `repository_guard = "04"`). Other `DEBT-GUARD-*` remain open.
- [x] RI-T13 updated: 04 pass/evaluated; 05–12 / 14–15 still skip-or-fail-closed.
- [x] After target GREEN, increment-1 baseline is `#[ignore]`-superseded or proven FAIL. ADR 0010 **Accepted**. This spec remains in `CANONICAL_SPECS`.
- [x] Phases 2–28, Phase 0 freeze list, and remaining_backlog are **not** implemented as product.

---

## 6. Risks

- Closing `DEBT-GUARD-04` before tests evaluate invariants would violate check 13 proof law and make 04 a paper pass.
- `INV-INVARIANTS-EVALUATED` currently *is* the remaining_backlog claim; forgetting to rewrite it would make Guard 04 pass while still asserting evaluation is future work.
- Independent greps per check will reappear unless `RepositoryModel` is the only input to `ArchitectureCheck`.
- Declaring `temporal_evidence_selection` `exclusive` while `select_latest_as_of` still lives in control-test can be misread as a required code move; increment 1 must treat kind as metadata and leave Phase 5 to move code.
- Executing `kind=path` / `source-pattern` too aggressively could fail CI on legitimate `docs/` mentions of `tests/sdd/`; evaluation is **path existence** / model source index, not “string appears in markdown”.
- `--check 09` must still load debt so skip-with-debt works; otherwise CI and local use diverge.
- Neighbor RI-T13 will go red if 04 starts passing without updating that suite.
- JSON/`duration` fields must not break the human report lines that RI-T11/T12/T13 already grep.
- Stub skip hatch: 05–12 / 14–15 must keep **per-check** `DEBT-GUARD-NN` ids (not one generic skip).
- xtask must stay `publish = false` / `dist = false` if `Cargo.toml` gains `serde_json`.

---

## 7. Dual-suite and verify commands

```text
cargo test -p xtask -- --nocapture
cargo xtask guard
cargo xtask guard --json
cargo test --test sdd_repository_integrity_target
cargo test --test sdd_documentation_layout
cargo fmt --all -- --check
```

After implement: `cargo xtask guard` prints 04 **pass**, remaining stubs `skip(DEBT-GUARD-NN)` with live ids, exit 0. Root `cargo test --all-targets` still does **not** run `cargo test -p xtask`; the guard CI step remains the health command (Phase 22 adds workspace CI).

---

## 8. Out of scope (phases 2–28 and Phase 0 freeze)

Do **not** this increment:

1. Catalog SSOT migration (Phase 2 / Guard 05)
2. Framework pack parse / digest / expression preservation (Phase 3 / 21 / Guards 06–07)
3. Evidence ledger `current()` / `as_of(t)` law (Phase 4 / Guard 11)
4. Temporal move to `weeping-angel-evidence` (Phase 5 / Guard 09)
5. `AssessmentRun` lineage rebuild (Phase 6 / Guard 10)
6. Readiness / SoA / Explain / Framework Projection implementations (Phases 7–10)
7. Remaining guards **05–12** and **14–15** as real implementations (keep debt-backed skips)
8. ADR mass-renumber (Phase 27) or Guard 14 uniqueness rewrite
9. Ignore-baseline deletion (Phase 23)
10. New semantic SSoTs, interpretation engines, catalog locations, ADR numbering schemes, or hand-written source-grep frameworks (Phase 0)
11. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`
12. Switching CI clippy/test to `--workspace` (Phase 22)
13. Test-support crate, package-install tests, schema-fixture program (Phases 24–26)
14. Changing `ASSURANCE_IR_SCHEMA`, catalog TOML identities, or scanner CLI

---

## 9. Related

- Decision (Accepted): [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md)
- Predecessor gate: [`docs/specs/repository-integrity.md`](repository-integrity.md), [ADR 0009](../adr/0009-repository-health-gate.md)
- Docs layout: [ADR 0004](../adr/0004-documentation-architecture.md)
- Crate graph / five invariants: [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md)
- SDD run report: [`docs/sdd/architectural-cleanup-program.md`](../sdd/architectural-cleanup-program.md)
- Debt: [`docs/debt/register.toml`](../debt/register.toml)
