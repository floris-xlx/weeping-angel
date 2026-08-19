# SDD run: Architectural-cleanup PROGRAM increment 1 — Phase 0 freeze + Phase 1 architecture-as-law

| Field | Value |
| --- | --- |
| Run id | `sdd-8d8d2d102fc3` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `8d8d2d102fc31244` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Increment 1 of the 29-phase (0–28) architectural-cleanup PROGRAM: Phase 0 freeze + Phase 1 architecture-as-law only. **Not** phases 2–28. |
| Spec (human SSOT) | [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md) |
| ADR | Accepted [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md) |
| Telemetry | [`.sdd/runs/architectural-cleanup-program-telemetry.json`](../../.sdd/runs/architectural-cleanup-program-telemetry.json) |
| Dual-suite | `xtask/tests/*.rs` via `cargo test -p xtask` — **not** `tests/sdd/` |
| Baseline | [`xtask/tests/sdd_architectural_cleanup_baseline.rs`](../../xtask/tests/sdd_architectural_cleanup_baseline.rs) (ACP-B01–B06; skip-retired, 11 ignored) |
| Target | [`xtask/tests/sdd_architectural_cleanup_target.rs`](../../xtask/tests/sdd_architectural_cleanup_target.rs) (ACP-T01–T17; 17/17 pass) |
| Predecessor | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [ADR 0009](../adr/0009-repository-health-gate.md) |
| Collision fence | Phases 2–28 not implemented. No `weeping-angel-catalog` / `weeping-angel-assurance-cli`. No ADR mass-renumber. No ignore-baseline deletion. |

Durable finalize artifact for telemetry run `sdd-8d8d2d102fc3`. Product law lives in the linked spec; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)). Generated traces belong under `.sdd/`, not here.

---

## Spec

- **Title:** Architectural-cleanup PROGRAM increment 1 (Phase 0 freeze + Phase 1 architecture-as-law)
- **Problem:** ADR 0009 is a presence-only health gate: `invariants.toml` is declared but Guard 04 stubs with `skip(DEBT-GUARD-04)`, ownership has no exclusive/facade/projection/adapter/shared-primitive kinds, forbidden-pattern kinds are not executed, and `GuardReport`/CLI cannot JSON-export, select, or explain checks. Without a freeze and one `RepositoryModel` evaluation plane, later pipeline work will invent new SSoTs, interpretation engines, catalog locations, ADR numbering, and source-grep frameworks instead of Providers→Collectors→Canonical Evidence→Ledger(`current()`/`as_of(t)`)→Tests→Assessments→Applicability+Risk/ISMS→immutable AssessmentRun→Readiness/SoA/Explain→Framework Projection enforced by `cargo xtask guard` + CI.
- **Current behavior (pre-implement):** `cargo xtask guard` implemented checks 01/02/03/13 only. Check 03 required `forbidden-patterns.toml` presence+schema and ignored `[[pattern]]` kinds. Checks 04–12 and 14–15 were `stub_check`: `skip(DEBT-GUARD-NN)` if the live debt id existed, else fail not-yet-implemented. `GuardReport` was `{ checks }` with `render()`; no `violations`/`skipped`/`debt_exemptions`/`duration`. CLI was `guard` only (no `--json`/`--check`/`--explain`). Ownership rows were crate+paths (`REQUIRED_OWNERSHIP`); no `kind`. `INV-INVARIANTS-EVALUATED` said evaluation was `remaining_backlog`. `xtask/tests/debt_register.rs` expected `skip(DEBT-GUARD-04)`. There was no `RepositoryModel` or `ArchitectureCheck` trait; each check read TOML independently. `EvidenceLedger` had `latest_as_of`, not `current()`; temporal selection was split across assurance and control-test.
- **Desired behavior (this increment):** Phase 0 freezes new semantic SSoTs, framework interpretation engines, readiness/temporal implementations, catalog locations, dual-suite path conventions, ADR numbering schemes, and hand-written grep frameworks. Phase 1 loads one `RepositoryModel` and runs `ArchitectureCheck::check`. Guard 04 parses `invariants.toml` and evaluates every `[[invariant]]` (rewrite `INV-INVARIANTS-EVALUATED`). Ownership rows require `kind` (e.g. `temporal_evidence_selection` exclusive) without moving code. Check 03 executes `kind=package|path|dependency|symbol|source-pattern` and rejects `weeping-angel-catalog`, `weeping-angel-assurance-cli`, and `tests/sdd/`. `GuardReport { checks, violations, skipped, debt_exemptions, duration }`; CLI `guard`/`--json`/`--check`/`--explain`. No silent skips. After implement: 04 pass; 05–12/14–15 skip with live `DEBT-GUARD-*`; exit 0. Close `DEBT-GUARD-04` only after Guard 04 tests evaluate invariants. Update RI-T13 so 04 is pass/evaluated.
- **ADR:** needed — accepted at [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md)

### Acceptance criteria (this slice)

1. Dual-suite lives in `xtask/tests/*.rs` (not `tests/sdd/`); baseline PASS on current skip/stub/presence-only behavior before product edits; target FAIL today on Guard 04 evaluation, ownership kinds, executable forbidden kinds, and structured CLI/report.
2. `run_guard` loads one `RepositoryModel` (workspace, package graph, filesystem, manifests, debt, ADR/spec metadata, framework packs, catalog sources) and runs `ArchitectureCheck`; Guard 04 is not `stub_check` and not an independent grep.
3. Guard 04 evaluates every `[[invariant]]` against the model; `INV-INVARIANTS-EVALUATED` no longer claims `remaining_backlog`.
4. Ownership rows require `kind` in `exclusive|facade|projection|adapter|shared-primitive`; `temporal_evidence_selection` may be exclusive without moving `select_latest_as_of`.
5. Check 03 executes `package|path|dependency|symbol|source-pattern` and rejects `weeping-angel-catalog`, `weeping-angel-assurance-cli`, and `tests/sdd/` when present.
6. `GuardReport` includes `checks`, `violations`, `skipped`, `debt_exemptions`, `duration`; CLI supports `guard`, `--json`, `--check NN`, `--explain INV-…`; every skip cites a live debt id.
7. `cargo xtask guard` exit 0: 01/02/03/04/13 pass; 05–12 and 14–15 `skip(DEBT-GUARD-NN)` with live ids.
8. `DEBT-GUARD-04` resolved only with `regression_tests` or `repository_guard=04` after Guard 04 tests evaluate invariants.
9. RI-T13 updated so 04 is pass/evaluated and 05–12/14–15 stay skip-or-fail-closed.
10. After target GREEN, increment-1 baseline is `#[ignore]`-superseded or proven FAIL; ADR 0010 Accepted; this spec stays in `CANONICAL_SPECS`.

### Out of scope

- Catalog SSOT (Phase 2 / Guard 05)
- Framework pack parse/digest and expression preservation (Phase 3 / 21 / Guards 06–07)
- Evidence ledger `current()`/`as_of(t)` law (Phase 4 / Guard 11)
- Temporal move to `weeping-angel-evidence` (Phase 5 / Guard 09)
- AssessmentRun lineage rebuild (Phase 6 / Guard 10)
- Readiness/SoA/Explain/Framework Projection implementations (Phases 7–10)
- Guards 05–12 and 14–15 as real implementations (keep `DEBT-GUARD-*` skips)
- ADR mass-renumber and Guard 14 uniqueness rewrite (Phases 17/27)
- Ignore-baseline deletion (Phase 23)
- New semantic SSoTs, interpretation engines, catalog locations, ADR numbering schemes, or hand-written source-grep frameworks (Phase 0)
- Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`
- CI `--workspace` / `cargo test -p xtask` in CI (Phase 22)
- Test-support crate, package-install tests, schema-fixture program (Phases 24–26)
- Changing `ASSURANCE_IR_SCHEMA`, catalog identities, or scanner CLI

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Closing `DEBT-GUARD-04` before tests evaluate invariants violates check 13 proof law | Resolved only after Guard 04 tests evaluate every `[[invariant]]`; `repository_guard = "04"` + `regression_tests` |
| Leaving `INV-INVARIANTS-EVALUATED` as `remaining_backlog` makes Guard 04 a paper pass | Invariant rewritten; ACP-T suite asserts evaluation is not remaining_backlog |
| Independent greps will return unless `RepositoryModel` is the only `ArchitectureCheck` input | `run_guard` loads one model; checks implement `ArchitectureCheck::check` |
| Exclusive temporal kind can be misread as a required code move this increment | Kind declared exclusive without moving `select_latest_as_of` (Phase 5 remains backlog) |
| Over-broad path/source-pattern matching could fail on markdown mentions of `tests/sdd/` | Path/source-pattern execution scoped so docs mentions do not fail the gate |
| `--check 09` must still load debt so skip-with-debt works | Debt is part of `RepositoryModel`; stubs still `skip(DEBT-GUARD-NN)` |
| RI-T13 goes red if 04 starts passing without updating that neighbor | `tests/contracts/repository_integrity.target.rs` updated; 18/18 pass |
| JSON/duration must not break existing human report greps | Human `render()` retained; `--json` is opt-in |
| Generic skip hatch if 05–12/14–15 stop citing per-check `DEBT-GUARD-NN` ids | Remaining stubs still cite live per-check ids |
| xtask must stay `publish=false`/`dist=false` if `serde_json` is added | `xtask/Cargo.toml` remains unpublished / not dist |

---

## Program end-state (29 phases, 0–28)

Normative pipeline (do not invert; later phases remain remaining_backlog):

```text
Providers → Collectors → Canonical Evidence
  → Evidence Ledger (current() / as_of(t))
  → Canonical Tests → Control Assessments
  → Applicability + Risk/ISMS
  → immutable AssessmentRun lineage
  → Readiness / SoA / Explain
  → Framework Projection
```

Enforcement: `architecture/*.toml` + debt + ADRs + specs, evaluated by `cargo xtask guard` + CI.

| Phase | Name | This increment |
| --- | --- | --- |
| **0** | Architectural freeze (no new SSoTs / interpretation engines / temporal APIs / catalog locations / ADR numbering / grep frameworks) | **Done (spec/out-of-scope law)** |
| **1** | Architecture-as-law (`RepositoryModel`, Guard 04, ownership kinds, forbidden kinds, structured CLI) | **Done** |
| 2 | Catalog SSOT | later |
| 3 | Framework pack parse fail-closed + digest | later |
| 4 | Evidence Ledger `current()` / `as_of(t)` | later |
| 5 | Temporal selection exclusive owner `weeping-angel-evidence` | later |
| 6 | Immutable `AssessmentRun` lineage rebuild | later |
| 7 | Readiness as projection | later |
| 8 | SoA as projection + invariants | later |
| 9 | Explain as projection | later |
| 10 | Framework Projection (adapters only) | later |
| 11 | Collectors normalize-only | later |
| 12 | Canonical Evidence envelope law | later |
| 13 | Canonical Tests ownership | later |
| 14 | Control Assessments consume tests | later |
| 15 | Applicability as law | later |
| 16 | Risk / ISMS consume assessments | later |
| 17 | ADR graph uniqueness (no mass-renumber yet) | later |
| 18 | Spec lifecycle states | later |
| 19 | Crate dependency graph policy | later |
| 20 | Persistence invariants | later |
| 21 | Framework expression preservation | later |
| 22 | CI `--workspace` bar | later |
| 23 | Dual-suite hygiene / ignore-baseline deletion | later |
| 24 | Schema fixtures | later |
| 25 | Test-support crate (not `tests/sdd/`) | later |
| 26 | Package install tests | later |
| 27 | ADR mass-renumber after Guard 14 | later |
| 28 | Debt closure for remaining `DEBT-GUARD-*` / P0 with proof | later |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md) |
| Baseline | PASS on old | `cargo test -p xtask -- --nocapture` → **pass** (exit 0). Characterization of CURRENT stub/skip/presence-only `cargo xtask guard` (01/02/03/13 real; 04–12 and 14–15 `skip(DEBT-GUARD-NN)`). No Phase 1 product implement. Full 29-phase program remains spec/SDD law only. Excerpt: `running 11 tests` / `live_debt_guard_04_is_open_stub` … `acp_b02_check_03_ignores_pattern_kinds` / `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`. Suites: `xtask/tests/sdd_architectural_cleanup_baseline.rs`, `docs/specs/architectural-cleanup-program.md`, `docs/sdd/architectural-cleanup-program.md`. |
| Target pre | FAIL on old | `cargo test -p xtask -- --nocapture` → **fail** (exit 1, expected). Target encodes ACP-T01–T17 (Guard 04 evaluation, ownership kinds, executable forbidden kinds, structured CLI/report). Product code untouched. `acp_t17` passed because dual-suite already lives in `xtask/tests/`. Baseline ACP-B01–B06 remained GREEN. Excerpt: `test result: FAILED. 1 passed; 16 failed (acp_t01..t16). acp_t15: check 04 must be evaluated pass, not skip(DEBT-GUARD-04) left: Skip { debt_id: "DEBT-GUARD-04" } right: Pass. acp_t08/t09: check 03 got Pass. Baseline: 11 passed.` Suite: `xtask/tests/sdd_architectural_cleanup_target.rs`. |
| Implement | target PASS | `cargo test -p xtask -- --nocapture`: target **ok. 17 passed; 0 failed; 0 ignored** (`acp_t01` … `acp_t15_live_guard_04_passes_remaining_stubs_skip`). Baseline default run **ok. 0 passed; 0 failed; 11 ignored** (`superseded by sdd_architectural_cleanup_target`). `cargo xtask guard`: 01 architecture-manifest pass; 02 canonical-ownership pass; 03 forbidden-patterns pass; 04 architecture-invariants pass; 13 debt-register pass; 05–12, 14, 15 `skip(DEBT-GUARD-*)`; exit 0. Neighbor `cargo test --test sdd_repository_integrity_target`: **ok. 18 passed; 0 failed**. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default `cargo test -p xtask`: baseline **ok. 0 passed; 0 failed; 11 ignored**. Not additive: `cargo test -p xtask --test sdd_architectural_cleanup_baseline -- --ignored --nocapture` → **FAILED. 2 passed; 9 failed**. Failures include `acp_b01_check_04_skips_with_debt_guard_04`, `acp_b01_live_repo_check_04_is_stub_skip`, `acp_b02_check_03_ignores_pattern_kinds`, `acp_b03_ownership_is_crate_and_paths_without_kind`, `acp_b04_guard_report_is_checks_plus_render_cli_is_guard_only`, `acp_b05_inv_invariants_evaluated_is_remaining_backlog`, `acp_b06_stub_without_debt_fails_closed`, `current_tree_has_no_repository_model_or_architecture_check`, `live_debt_guard_04_is_open_stub` (left: Pass vs Skip{DEBT-GUARD-04}; ownership.catalog.kind is required; kinds now executed). Spec/SDD SSOT files kept (delete/move would fail ACP-T17 / Phase 23). `baseline_retired=true`, `baseline_not_green=true`. |
| Supersede | target still PASS | After skip-supersede ACP-B01–B06 with `#[ignore = "superseded by sdd_architectural_cleanup_target"]`: target **ok. 17 passed; 0 failed; 0 ignored** (`acp_t01_repository_model_loads_workspace_graph_and_manifests` … `acp_t15_live_guard_04_passes_remaining_stubs_skip`). `target_still_green=true`. Dual-suite files retained. |
| Docs/ADR | updated | [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md), [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md), [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md), [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md), [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [`docs/README.md`](../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`docs/debt/README.md`](../debt/README.md), [`docs/sdd/README.md`](README.md), [`docs/sdd/architectural-cleanup-program.md`](architectural-cleanup-program.md), [`docs/sdd/repository-integrity.md`](repository-integrity.md), [`docs/sdd/repository-integrity-health-gate.md`](repository-integrity-health-gate.md), [`README.md`](../../README.md) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `skip` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

Phase 1 architecture-as-law is present and verified:

- `run_guard` loads one `RepositoryModel` and runs `ArchitectureCheck::check` (not independent greps).
- Guard 04 evaluates every `[[invariant]]`; `INV-INVARIANTS-EVALUATED` rewritten (no longer `remaining_backlog`).
- Ownership rows require `kind` ∈ exclusive \| facade \| projection \| adapter \| shared-primitive (`architecture/architecture.toml`); `temporal_evidence_selection` exclusive without a code move.
- Check 03 executes `package|path|dependency|symbol|source-pattern` and rejects hypothetical packages and `tests/sdd/`.
- `GuardReport { checks, violations, skipped, debt_exemptions, duration }`; CLI `cargo xtask guard [--json] [--check NN] [--explain INV-…]`.
- `DEBT-GUARD-04` resolved with `repository_guard = "04"` and `regression_tests` after Guard 04 tests evaluate invariants.
- RI-T13 treats 04 as pass/evaluated; 05–12 / 14–15 stay skip-or-fail-closed.
- ADR 0010 Accepted. Dual-suite under `xtask/tests/`. Target 17/17 pass; baseline 11 ignored-superseded.
- `cargo xtask guard` exit 0: 01/02/03/04/13 pass; remaining stubs skip with live `DEBT-GUARD-*`.

### Files changed (implement)

`xtask/src/lib.rs`, `xtask/Cargo.toml`, `Cargo.lock`, `architecture/architecture.toml`, `architecture/invariants.toml`, `architecture/forbidden-patterns.toml`, `docs/debt/register.toml`, `docs/adr/0010-architecture-as-law.md`, `docs/specs/architectural-cleanup-program.md`, `docs/sdd/architectural-cleanup-program.md`, `tests/contracts/repository_integrity.target.rs`, `tests/contracts/documentation_layout.rs`, `xtask/tests/debt_register.rs`, `xtask/tests/sdd_architectural_cleanup_baseline.rs`.

### Docs/ADR (DocsAdr phase)

Finalized ADR 0010 against the shipped Phase 1 contract (`RepositoryModel` + `ArchitectureCheck`, Guard 04 evaluation, ownership kinds, executable forbidden kinds, structured `GuardReport`/CLI, `DEBT-GUARD-04` resolved). Amended ADR 0009/0004 and neighbor README/spec/SDD pointers so check 04 is no longer documented as a stub.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-8d8d2d102fc3` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 6 261 965 |
| `duration_ms_sum` | 2 726 215 (~45.4 min) |
| `budget.total` | 48 |
| `budget.spent` | 8 |
| `budget.remaining` | 40 |
| `event_count` | 29 |
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
| Scope | `sdd-scope` | ok | 144 654 | 155 838 |
| Spec | `sdd-spec` | ok | 486 995 | 793 103 |
| BaselineGreen | `sdd-baseline-green` | ok | 288 534 | 569 105 |
| TargetRed | `sdd-target-red` | ok | 412 110 | 784 761 |
| Implement | `sdd-implement` | ok | 702 608 | 1 926 019 |
| DocsAdr | `sdd-docs-adr` | ok | 491 685 | 1 591 955 |
| Iterate | `sdd-baseline-post-check` | ok | 77 978 | 139 239 |
| Supersede | `sdd-supersede` | ok | 121 651 | 301 945 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement).

---

## remaining_backlog (not implemented)

1. Catalog SSOT (Phase 2 / Guard 05)
2. Framework pack parse fail-closed + digest; expression preservation (Phases 3 / 21 / Guards 06–07)
3. Evidence Ledger `current()` / `as_of(t)` (Phase 4 / Guard 11)
4. Temporal selection exclusive owner `weeping-angel-evidence` (Phase 5 / Guard 09)
5. Immutable `AssessmentRun` lineage rebuild (Phase 6 / Guard 10)
6. Readiness / SoA / Explain / Framework Projection implementations (Phases 7–10)
7. Remaining guards 05–12 and 14–15 as real implementations (keep `DEBT-GUARD-*` skips)
8. ADR graph uniqueness without mass-renumber (Phase 17 / Guard 14); ADR mass-renumber (Phase 27)
9. Spec lifecycle states (Phase 18)
10. Crate dependency graph policy; CI `--workspace` / `cargo test -p xtask` in CI (Phases 19 / 22)
11. Persistence invariants (Phase 20)
12. Dual-suite hygiene / ignore-baseline deletion (Phase 23)
13. Schema fixtures; test-support crate; package-install tests (Phases 24–26)
14. Debt closure for remaining `DEBT-GUARD-*` / P0 with proof (Phase 28)
15. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli` (still forbidden)
16. New semantic SSoTs, interpretation engines, catalog locations, ADR numbering schemes, or hand-written source-grep frameworks (Phase 0 freeze)

---

## Related

- Spec SSOT: [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md)
- Decision: [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md)
- Predecessor: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [ADR 0009](../adr/0009-repository-health-gate.md)
- Debt register: [`docs/debt/register.toml`](../debt/register.toml)
- Telemetry: [`.sdd/runs/architectural-cleanup-program-telemetry.json`](../../.sdd/runs/architectural-cleanup-program-telemetry.json)
