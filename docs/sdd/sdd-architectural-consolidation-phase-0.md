# SDD run: Architectural Consolidation Program Phase 0 — freeze, baseline snapshot, backlog schema

| Field | Value |
| --- | --- |
| Run id | `wa-consolidation-phase-0` |
| Date | 2026-08-20 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `1ea77782b7274964` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | **Phase 0 only** (0.1 consolidation mode, 0.2 frozen baseline snapshot, 0.3 structural-duplication backlog schema). **Not** Phases 1+ consumer migrations or duplicate deletes. |
| Spec (human SSOT) | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) |
| ADR | Accepted [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md) |
| Telemetry | [`sdd-architectural-consolidation-phase-0-telemetry.json`](sdd-architectural-consolidation-phase-0-telemetry.json) |
| Dual-suite | `xtask/tests/*.rs` via `cargo test -p xtask` — **not** `tests/sdd/`, `test/sdd/*.ts`, or a new root `[[test]]` |
| Baseline | `xtask/tests/sdd_architectural_consolidation_baseline.rs` — **deleted** after target GREEN (`supersede_kind=delete`; `INV-NO-SUPERSEDED-BASELINES`) |
| Target | [`xtask/tests/sdd_architectural_consolidation_target.rs`](../../xtask/tests/sdd_architectural_consolidation_target.rs) (CON-T01–T10; 10/10 pass) |
| Predecessor | [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md), [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md), [ADR 0048](../adr/0048-structural-reconciliation.md), [ADR 0010](../adr/0010-architecture-as-law.md) |
| Collision fence | Architectural-cleanup Phase 0 is a different program (spec-law-only freeze). This slice is the **machine-readable** freeze. No Guard 16. No second health CLI. No `weeping-angel-catalog` / `weeping-angel-assurance-cli`. |

Durable finalize artifact for telemetry run `wa-consolidation-phase-0`. Product law lives in the linked spec; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)). Generated traces belong under `.sdd/`, not here. The SDD pointer remains [`architectural-consolidation-program.md`](architectural-consolidation-program.md).

---

## Spec

- **Title:** Architectural Consolidation Program Phase 0 — freeze, baseline snapshot, backlog schema
- **Problem:** Feature expansion is not machine-frozen: `architecture.toml` has no parsed consolidation-mode table (extra TOML is ignored), there is no frozen consolidation-baseline against live inventory, and `structural-duplication.toml` v1 cannot serve as a close-law backlog. Parallel truths can still land before any semantic migration.
- **Current behavior (pre-implement):** `architecture.toml` is `[policy]` + `[ownership.*]` only; `load_architecture_manifest` ignores extra tables so Guard 01 cannot see a paper `[program.architectural_consolidation]`. Guards 01–15 pass without consolidation invariants. `docs/debt/current.md` is the live `weeping-angel/inventory/v1` snapshot and does not project crates/modules/public symbols/`pub use`/structs/enums/duplicate types/ownership/debt rows/spec count/schema locations. No `docs/debt/consolidation-baseline.json|md`. `structural-duplication.toml` is v1 `program=structural-reconciliation` phase=2 with 17 rows (DUP-001..017); statuses `candidate|confirmed|migrating|resolved|false-positive`; missing severity, canonical_symbol, migration_state, removal_blockers, public_api_impact, serialization_impact, tests; no Rust parser loads it.
- **Desired behavior (this slice):** Phase 0.1: `[program.architectural_consolidation]` `status=active` `feature_expansion=restricted` with allowed/forbidden change classes is parsed on `ArchitectureManifest` and fail-closed via Guard 01 and/or Guard 04 (not Guard 16, not a second health CLI); restricted mode rejects increases vs the frozen baseline in `[[test]]`, schema trees, workspace crates, public structs/enums, `pub use`, duplicated helpers, and second SSOTs while allowing bug/security, consolidation, non-semantic collectors, and consolidation docs. Phase 0.2: `docs/debt/consolidation-baseline.json|md` (`weeping-angel/consolidation-baseline/v1`) generated from the existing inventory walker; `current.md` stays live. Phase 0.3: `structural-duplication.toml` v2 is the program backlog with required row fields and statuses `candidate|confirmed|canonicalized|consumers-migrating|compatibility-only|removed|verified`; v1 migrating/resolved/false-positive map without silent verified/removed; close law requires canonical owner, migrated consumers, old path gone or compatibility-only, and a regression guard. Dual-suite under `xtask/tests` only; delete baseline after target GREEN.
- **ADR:** needed — accepted at [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)

### Acceptance criteria (this slice)

1. Dual-suite is `xtask/tests/sdd_architectural_consolidation_{baseline,target}.rs` via `cargo test -p xtask`; never `tests/sdd/`, `test/sdd/*.ts`, or a new root `[[test]]`.
2. Baseline CON-B01–B06 PASS on CURRENT: no program table, extra TOML ignored, no consolidation-baseline artifacts, duplication v1/old statuses/missing fields, inventory without extended projection.
3. Target CON-T01–T10 FAIL on CURRENT because those three artifacts/schema/enforcement are absent, not because of unrelated product code; then PASS after implement.
4. `architecture.toml` `[program.architectural_consolidation]` `status=active` `feature_expansion=restricted` with required allowed/forbidden classes; loader fails closed if missing or malformed.
5. Guard 01 and/or 04 evaluate the table and expansion monotonicity vs frozen baseline; no Guard 16; no second health CLI.
6. `docs/debt/consolidation-baseline.json` and `.md` exist with schema `weeping-angel/consolidation-baseline/v1` and §5.2 coverage; shared counts come from inventory; `current.md` remains live.
7. `structural-duplication.toml` is v2; every row has `id`, `concept`, `severity`, `canonical_owner`, `canonical_symbol`, `duplicates`, `migration_state`, `removal_blockers`, `public_api_impact`, `serialization_impact`, `tests`, `guard`, `status` in the new closed set.
8. v1 `migrating`→`consumers-migrating`; `resolved`/`false-positive` never auto-map to `verified` or `removed`; close law blocks `verified`/`removed` until owner, consumers, removal, and guard all hold.
9. After target GREEN the baseline suite is deleted (`INV-NO-SUPERSEDED-BASELINES`), not `#[ignore]`.
10. Neighbors stay green: `CANONICAL_SPECS`, `spec-lifecycle.toml`, ACP target, SR target, `cargo xtask guard` 01–15; ADR 0049 Accepted only after target GREEN.

### Out of scope

- Phases 1+ consumer migrations and deleting DUP duplicate source trees
- Rewriting assurance, collector, catalog, or IR
- New frameworks, collectors, ISMS engines, report formats, product scanners
- Hypothetical packages `weeping-angel-catalog` and `weeping-angel-assurance-cli`
- `tests/sdd/`, `test/sdd/*.ts`, new root `[[test]]` for this program
- Mass ADR renumber and ignore-baseline mass delete
- pnpm / `apps/docs`
- A 16th ProductLawCheck or second health CLI
- A second inventory walker
- Changing `ASSURANCE_IR_SCHEMA` or catalog identities
- Treating architectural-cleanup Phase 0 as this freeze
- Reopening resolved `DEBT-GUARD-05…15` skip hatches
- Silently marking DUP rows `verified` because v1 said `resolved`
- Feature expansion (new public domain types, second persistence, second projection path) under the freeze

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| A paper TOML table without loader changes looks like a freeze but is not a gate | `ArchitectureManifest` parses `[program.architectural_consolidation]`; missing/malformed fails closed (CON-T01/T02) |
| Public-symbol heuristics can flake on comments/strings if the scan is undocumented | Frozen baseline projected from the existing inventory walker; shared counts reuse inventory keys |
| Monotonic expansion checks can fail legitimate bug-fixes that add helpers or types | Restricted mode allows bug/security, consolidation, non-semantic collectors, and consolidation docs |
| Mapping v1 `resolved`→`verified` silently closes rows with duplicates still on disk | Close law blocks `verified`/`removed`; v1 `resolved`/`false-positive` do not auto-map (CON-T06) |
| Adding Guard 16 or a new xtask health command forks the health plane | Guard 01 and/or 04 only; no Guard 16; no second health CLI |
| `#[ignore]` on the baseline suite after GREEN violates `INV-NO-SUPERSEDED-BASELINES` | Baseline file deleted; CON-T07 fails closed if it returns |
| Confusing this freeze with architectural-cleanup Phase 0 (review-bar only) | Collision fence in spec/ADR/SDD pointer; different program |
| Target RED for unrelated product needles instead of the three missing artifacts | Target RED: T01–T06/T09 failed for missing table/baseline/v2/close-law/INV; T07/T08/T10 already ok |
| A different exclusion set than inventory desyncs `current.md` from the frozen baseline | Shared counts come from inventory; `current.md` remains live |
| Guard 15 fails if this spec is on disk without a spec-lifecycle row | Spec indexed; CON-T08 and live Guard 01–15 green |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) |
| Baseline | PASS on old | `cargo test -p xtask -- --nocapture` → **pass**. Characterization only (CON-B01–B06). No Phase 0.1–0.3 product implement. `docs/debt/current.md` ADR count resynced 48→49 to match live `collect()` after draft ADR 0049; unwrap/expect needles avoided in the new suite so inventory counts otherwise stayed put. Excerpt: `running 6 tests` / `con_b01_architecture_toml_has_no_program_table` … `con_b06_live_guard_01_through_15_pass` / `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s`. Suite: `xtask/tests/sdd_architectural_consolidation_baseline.rs`. |
| Target pre | FAIL on old | `cargo test -p xtask -- --nocapture` → **fail** (exit 1, expected). Target RED on CURRENT because Phase 0.1–0.3 artifacts/schema/enforcement are absent, not because of unrelated product regressions. T07/T08/T10 pass: dual-suite lives under `xtask/tests` with no new root `[[test]]`, spec/ADR 0049 already indexed, neighbors and guard 01–15 stay green. No product code was changed. Excerpt: `con_t01`…`con_t06`/`con_t09` FAILED; `con_t07`/`con_t08`/`con_t10` ok; `architecture.toml must declare [program.architectural_consolidation]`; `missing [program.architectural_consolidation] must fail closed`; `docs/debt/consolidation-baseline.json must exist`; schema v1 vs v2; `DUP-001 v1 resolved maps to canonicalized\|consumers-migrating\|compatibility-only, got resolved`; `architecture/invariants.toml must declare INV-CONSOLIDATION-MODE-ACTIVE`; `test result: FAILED. 3 passed; 7 failed`. Baseline CON-B01–B06 still 6 passed. Suite: `xtask/tests/sdd_architectural_consolidation_target.rs`. |
| Implement | target PASS | `cargo test -p xtask --test sdd_architectural_consolidation_target -- --nocapture` → **ok. 10 passed; 0 failed**. Phase 0 freeze+baseline+backlog: parsed `[program.architectural_consolidation]` (Guard 01/04), frozen `consolidation-baseline.json\|md` from inventory, `structural-duplication.toml` v2 with close law. `cargo test -p xtask -- --nocapture`: consolidation target 10 passed; ACP 17 passed; SR 15 passed; `debt_register` 5 passed. `cargo xtask guard` 01–15 pass. `cargo xtask inventory --check` exit 0. ADR 0049 Accepted. |
| Baseline post | FAIL or retired | **Retired by delete** (`supersede_kind=delete`). After implement the leftover characterization file failed to compile (`error[E0027]: pattern does not mention field consolidation` at `sdd_architectural_consolidation_baseline.rs:163`; exit 1). Suite then deleted (`INV-NO-SUPERSEDED-BASELINES`). `Test-Path xtask/tests/sdd_architectural_consolidation_baseline.rs` → `False`. `cargo test -p xtask --test sdd_architectural_consolidation_baseline` → `error: no test target named sdd_architectural_consolidation_baseline in xtask package`. `baseline_retired=true`, `baseline_not_green=true`, `additive_baseline=false`. |
| Supersede | target still PASS | `cargo test -p xtask -- --nocapture` / `--test sdd_architectural_consolidation_target`: **ok. 10 passed; 0 failed; 0 ignored** (CON-T01–T10). CON-T07 now asserts the baseline file must stay deleted. xtask package: debt 9, `debt_register` 5, cleanup 17, consolidation 10, reconciliation 15 — all ok. `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md), [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md), [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md), [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md), [`docs/README.md`](../README.md), [`README.md`](../../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`docs/debt/README.md`](../debt/README.md), [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md), [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `delete` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

Phase 0 freeze + baseline snapshot + backlog schema is present and verified:

- `[program.architectural_consolidation]` `status=active` `feature_expansion=restricted` with required allowed/forbidden classes, parsed on `ArchitectureManifest` and fail-closed via Guard 01/04 (`INV-CONSOLIDATION-MODE-ACTIVE`).
- Restricted mode rejects increases vs the frozen baseline in `[[test]]`, schema trees, workspace crates, public structs/enums, `pub use`, duplicated helpers, and second SSOTs.
- `docs/debt/consolidation-baseline.json` and `.md` exist with schema `weeping-angel/consolidation-baseline/v1`; shared counts come from inventory; `current.md` remains live.
- `structural-duplication.toml` is v2 with required row fields and closed status set; v1 mapping does not silently close rows as `verified`/`removed`; close law requires canonical owner, migrated consumers, old path gone or compatibility-only, and a regression guard.
- Dual-suite under `xtask/tests` only. Target CON-T01–T10 GREEN. Baseline characterization suite deleted (`INV-NO-SUPERSEDED-BASELINES`).
- No Guard 16. No second health CLI. Neighbors stay green: `CANONICAL_SPECS`, `spec-lifecycle.toml`, ACP target, SR target, `cargo xtask guard` 01–15.
- ADR 0049 Accepted after target GREEN.

### Files changed (implement)

`architecture/architecture.toml`, `architecture/invariants.toml`, `docs/adr/0049-architectural-consolidation-phase-0.md`, `docs/debt/README.md`, `docs/debt/consolidation-baseline.json`, `docs/debt/consolidation-baseline.md`, `docs/debt/structural-duplication.toml`, `docs/sdd/architectural-consolidation-program.md`, `docs/specs/architectural-consolidation-program.md`, `xtask/src/architecture.rs`, `xtask/src/checks.rs`, `xtask/src/duplication.rs`, `xtask/src/inventory.rs`, `xtask/src/lib.rs`, `xtask/tests/debt_register.rs`, `xtask/tests/sdd_architectural_cleanup_target.rs`, `xtask/tests/sdd_architectural_consolidation_baseline.rs` (later deleted), `xtask/tests/sdd_architectural_consolidation_target.rs`, `xtask/tests/sdd_structural_reconciliation_target.rs`.

### Docs/ADR (DocsAdr phase)

Finalized ADR 0049 as Accepted Phase 0 law and aligned spec/README/debt/contracts with the shipped freeze, frozen baseline, v2 backlog close law, and deleted baseline suite. Neighbor ADR 0048 / structural-reconciliation / architectural-cleanup specs note the collision fence.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `wa-consolidation-phase-0` |
| `agents_ok` | 7 |
| `agents_fail` | 0 |
| `agents_total` | 7 |
| `tokens_used_sum` | 10 368 014 |
| `duration_ms_sum` | 3 159 462 (~52.7 min) |
| `budget.total` | 48 |
| `budget.spent` | 7 |
| `budget.remaining` | 41 |
| `event_count` | 28 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |
| `reason` | `pre_finalize` |

### Gates

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agent phases

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 170 952 | 332 487 |
| Spec | `sdd-spec` | ok | 571 833 | 747 554 |
| BaselineGreen | `sdd-baseline-green` | ok | 394 692 | 1 181 830 |
| TargetRed | `sdd-target-red` | ok | 565 684 | 1 769 570 |
| Implement | `sdd-implement` | ok | 934 560 | 4 811 210 |
| DocsAdr | `sdd-docs-adr` | ok | 397 081 | 1 269 908 |
| Supersede | `sdd-supersede` | ok | 124 660 | 255 455 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement; no iterate agent).

---

## remaining_backlog (not implemented)

1. Phases 1+ consumer migrations and deleting DUP duplicate source trees
2. Rewriting assurance, collector, catalog, or IR
3. New frameworks, collectors, ISMS engines, report formats, product scanners
4. Hypothetical packages `weeping-angel-catalog` and `weeping-angel-assurance-cli` (still forbidden)
5. `tests/sdd/`, `test/sdd/*.ts`, new root `[[test]]` (still forbidden)
6. Mass ADR renumber and ignore-baseline mass delete
7. pnpm / `apps/docs`
8. A 16th ProductLawCheck or second health CLI (still forbidden)
9. A second inventory walker (still forbidden; extend existing projection)
10. Changing `ASSURANCE_IR_SCHEMA` or catalog identities
11. Treating architectural-cleanup Phase 0 as this freeze (collision fence remains)
12. Reopening resolved `DEBT-GUARD-05…15` skip hatches
13. Silently marking DUP rows `verified` because v1 said `resolved` (close law remains)
14. Feature expansion (new public domain types, second persistence, second projection path) under the freeze

---

## Related

- Spec SSOT: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)
- SDD pointer: [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md)
- Decision: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)
- Neighbor: [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md)
- Frozen baseline: [`docs/debt/consolidation-baseline.md`](../debt/consolidation-baseline.md), [`docs/debt/consolidation-baseline.json`](../debt/consolidation-baseline.json)
- Live inventory: [`docs/debt/current.md`](../debt/current.md)
- Backlog: [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml)
- Target suite: [`xtask/tests/sdd_architectural_consolidation_target.rs`](../../xtask/tests/sdd_architectural_consolidation_target.rs)
- Telemetry: [`sdd-architectural-consolidation-phase-0-telemetry.json`](sdd-architectural-consolidation-phase-0-telemetry.json)
