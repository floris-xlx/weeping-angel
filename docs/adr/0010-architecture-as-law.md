# ADR 0010 — Architecture-as-law (`RepositoryModel` + Guard 04 + ownership kinds + executable forbidden patterns + structured `cargo xtask guard`)

<!-- weeping-angel-adr-meta
id = "0010"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0001-inwardly-extensible-assurance-runtime", "0004-documentation-architecture", "0009-repository-health-gate"]
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — increment 1 (Phase 0 freeze + Phase 1 architecture-as-law) implemented. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Amends** [ADR 0009](0009-repository-health-gate.md): check **04** is no longer a stub; check **03** executes `[[pattern]]` kinds; `[ownership.*]` rows require `kind`; `GuardReport`/CLI are structured. Does **not** renumber existing `0003-*` / `0005-*` / `0007-*` / `0008-*` ADR files. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md), [ADR 0009](0009-repository-health-gate.md) |
| Spec | [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md) |
| Tests | `cargo test -p xtask` dual-suite GREEN (`xtask/tests/sdd_architectural_cleanup_target.rs` ACP-T01–T17). Increment-1 baseline `#[ignore = "superseded by sdd_architectural_cleanup_target"]` (ACP-B01–B06). Neighbor `ri_t13` treats check **04** as pass/evaluated. |

> Filename **`0010-*`**. Next unused unique ADR number after 0009. Cite **this file by path**. Do **not** add a `0003-architecture-as-law.md` sibling. Duplicate `0003-*` IDs remain debt (`DEBT-DUP-ADR`). Next unique number is **0011**.

## Context

ADR 0009 shipped a presence-only health gate: `architecture/*.toml`, `docs/debt/register.toml`, and `cargo xtask guard` checks **01, 02, 03, 13**. Checks **04–12** and **14–15** skipped with `DEBT-GUARD-NN` or failed closed. That is not enough for the architectural-cleanup **program**:

1. `architecture/invariants.toml` was declared but not evaluated. `INV-INVARIANTS-EVALUATED` said evaluation was `remaining_backlog`. Check 04 was `stub_check`.
2. Each check grepped/read files independently. There was no `RepositoryModel`. Later guards would fork into hand-written source-grep frameworks unless evaluation was centralized.
3. Ownership rows had `crate` + `paths` only. They could not express exclusive / facade / projection / adapter / shared-primitive (needed before Phase 5 moves temporal selection).
4. `forbidden-patterns.toml` already listed `weeping-angel-catalog`, `weeping-angel-assurance-cli`, and `tests/sdd/`, but check 03 did not execute `kind`.
5. `GuardReport` was `{ checks }` only. CLI was `guard` with no `--json`, `--check`, or `--explain`.
6. Concurrent slices would otherwise invent new SSoTs, interpretation engines, catalog locations, ADR numbering schemes, and `tests/sdd/` suites.

Questions this decision answers:

1. What is the single evaluation plane for `cargo xtask guard`?
2. When is an invariant “held” vs merely declared?
3. How is concept ownership qualified (kind) without moving product code this increment?
4. Which forbidden-pattern kinds are executable law?
5. What is the structured guard report / CLI?
6. What is frozen (Phase 0) vs implemented (Phase 1) vs later (Phases 2–28)?

## Decision (shipped)

Field-level law is [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md). What follows is the increment-1 contract as implemented in `xtask/src/lib.rs`.

### 1. One program, 29 phases (0–28)

Target pipeline (do not invert; Phases 2–28 remain remaining_backlog):

```text
Providers → Collectors → Canonical Evidence
  → Evidence Ledger (current() / as_of(t))
  → Canonical Tests → Control Assessments
  → Applicability + Risk/ISMS
  → immutable AssessmentRun lineage
  → Readiness / SoA / Explain
  → Framework Projection
```

Increment 1 implements **Phase 0** (freeze) + **Phase 1** only.

### 2. Phase 0 freeze

Do not introduce: new semantic SSoTs; framework interpretation engines; new readiness implementations; new temporal selection functions; new catalog locations; new baseline/target path conventions; new ADR numbering schemes; hand-written source-grep frameworks. Dual-suite for this increment lives in `xtask/tests/*.rs`. `tests/sdd/` remains forbidden (ADR 0004 / `FORBID-TESTS-SDD`).

### 3. `RepositoryModel` + `ArchitectureCheck` are the evaluation plane

`run_guard` / `run_guard_with_options` load **one** `RepositoryModel` (Cargo workspace members, package graph, filesystem index, architecture manifests, debt register, ADR/spec filenames, framework packs, catalog sources) and run checks via:

```rust
trait ArchitectureCheck {
    fn check(&self, repo: &RepositoryModel) -> CheckResult;
}
```

Shipped types: `ArchitectureManifest`, `OwnershipRow`, `ArchitectureInvariant`, `InvariantResult`, `ForbiddenPattern`, `GuardViolation`, `GuardSkip`, `GuardReport`. Implemented checks (`ArchitectureManifestCheck`, `CanonicalOwnershipCheck`, `ForbiddenPatternsCheck`, `ArchitectureInvariantsCheck`, `DebtRegisterCheck`) and remaining stubs (`StubArchitectureCheck`) all take the loaded model. Guards **must not** be independent filesystem greps.

Public wrappers `check_01_*` … `check_04_*` still exist for tests; each reloads the model and delegates to the same `ArchitectureCheck`.

### 4. Guard 04 evaluates every invariant

Check **04** (`architecture-invariants`) parses `architecture/invariants.toml` (`schema = weeping-angel/architecture-invariants/v1`) and evaluates every `[[invariant]]`. File presence is not a pass. Unknown invariant ids fail closed (no evaluation predicate). Empty `id` / `summary` / `guard_check` fail closed.

Shipped predicates:

| id | Evaluation |
| --- | --- |
| `INV-OWNERSHIP-LIVE-CRATES` | Mandatory ownership crates are workspace members; required paths exist |
| `INV-NO-HYPOTHETICAL-PACKAGES` | No workspace member named `weeping-angel-catalog` or `weeping-angel-assurance-cli` |
| `INV-DEBT-RESOLVED-HAS-PROOF` | Debt register validates (check 13 / `debt_error` is none) |
| `INV-INVARIANTS-EVALUATED` | Every `[[invariant]]` was evaluated; **summary must not contain `remaining_backlog`** |

`--explain INV-…` prints `id`, `summary`, `guard_check`, `result`, `evidence`.

`DEBT-GUARD-04` is **resolved** with `repository_guard = "04"` and `regression_tests = ["sdd_architectural_cleanup_target"]`. Checks **05–12** and **14–15** remain skip-with-live-`DEBT-GUARD-NN` or fail closed.

Neighbor RI-T13: **04** pass/evaluated; **05–12 / 14–15** still skip-or-fail-closed.

### 5. Ownership `kind`

Every `[ownership.*]` row (mandatory seven + any extra) requires:

```text
kind ∈ exclusive | facade | projection | adapter | shared-primitive
```

Shipped rows:

| Concept | Package | kind |
| --- | --- | --- |
| `catalog` | `weeping-angel-canonical-catalog` | `exclusive` |
| `framework_compilation` | `weeping-angel-framework` | `exclusive` |
| `readiness_projection` | `weeping-angel-assurance` | `projection` |
| `temporal_evidence_selection` | `weeping-angel-assurance` | `exclusive` |
| `assessment_lineage` | `weeping-angel-assurance` | `exclusive` |
| `evidence_persistence` | `weeping-angel-evidence` | `exclusive` |
| `assurance_cli` | `weeping-angel` | `facade` |

`ownership.temporal_evidence_selection.kind = "exclusive"` is **metadata for Phase 5**. Increment 1 does **not** move `weeping-angel-control-test::temporal::select_latest_as_of`.

### 6. Forbidden patterns are executable

Check **03** executes `kind ∈ package | path | dependency | symbol | source-pattern` against `RepositoryModel`. Missing/unknown `kind` or empty `value` fail closed.

- `package`: no workspace package name equals `value`
- `path`: `value` must not exist on disk (path existence, not markdown mention)
- `dependency`: `value` is `from -> to`; that edge must not appear in the package graph
- `symbol`: named symbol must not appear in the model’s source index (optional `in_crate` extra)
- `source-pattern`: data-driven from the toml via the source index — **not** a new grep crate

Seeds `weeping-angel-catalog`, `weeping-angel-assurance-cli`, and `tests/sdd/` fail the check if present.

### 7. Structured report and CLI

```text
GuardReport { checks, violations, skipped, debt_exemptions, duration }
cargo xtask guard
cargo xtask guard --json
cargo xtask guard --check 09
cargo xtask guard --explain INV-…
```

`--check NN` loads the model once, runs all checks, then retains the selected id (unknown id fails closed). Human `render()` still prints `NN  <name>  pass|fail|skip(DEBT-…)`. JSON is additive (`to_json()`). Zero-duration runs are recorded as 1 ns so `duration` is never silent-zero.

No silent skips. Every skip cites a live debt id (`skip(DEBT-GUARD-NN)` and `debt_exemptions`). Missing id → fail closed (`not-yet-implemented: check NN`).

### 8. Dual-suite home

Executable law for this increment: `cargo test -p xtask` (`xtask/tests/*.rs`). Do not register a new root `[[test]]` dual-suite and do not create `tests/sdd/`.

## Non-goals (remaining_backlog)

Catalog SSOT; framework parse/digest; evidence ledger `current()`/`as_of(t)`; temporal move; AssessmentRun rebuild; readiness/SoA/explain/framework projection implementations; guards **05–12** / **14–15** as real; ADR mass-renumber; ignore-baseline deletion; CI `--workspace`; inventing hypothetical packages.

## Consequences

- Architecture manifests are executable law for invariants, ownership kinds, and forbidden patterns.
- Later phases cannot add a second evaluation plane or a `tests/sdd/` suite without failing the gate.
- `DEBT-GUARD-04` is closed with proof; remaining `DEBT-GUARD-*` stay open.
- Temporal `exclusive` kind does not by itself move code; Phase 5 still must.

## Related

- Spec: [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md)
- SDD run: [`docs/sdd/architectural-cleanup-program.md`](../sdd/architectural-cleanup-program.md)
- Predecessor: [ADR 0009](0009-repository-health-gate.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Spine: [ADR 0001](0001-inwardly-extensible-assurance-runtime.md)
- Debt: [`docs/debt/register.toml`](../debt/register.toml) (`DEBT-GUARD-04` resolved)
