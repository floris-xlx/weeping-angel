# SDD run: Weeping Angel Structural Reconciliation Program — Phase 0+1

| Field | Value |
| --- | --- |
| Run id | `sdd-9cee200795c5` |
| Date | 2026-08-20 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `9cee200795c5cfab` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Phase 0 freeze + Phase 1 inventory / debt honesty / RI active-plane reconcile / active-spec drift. **Not** later Structural Reconciliation phases. |
| Spec (human SSOT) | [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md) |
| ADR | Accepted [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md) |
| Telemetry | [`sdd-structural-reconciliation-telemetry.json`](sdd-structural-reconciliation-telemetry.json) |
| Dual-suite | `xtask/tests/*.rs` via `cargo test -p xtask` — **not** `tests/sdd/` |
| Baseline | `xtask/tests/sdd_structural_reconciliation_baseline.rs` — **deleted** after target GREEN (`supersede_kind=delete`) |
| Target | [`xtask/tests/sdd_structural_reconciliation_target.rs`](../../xtask/tests/sdd_structural_reconciliation_target.rs) (SR-T01–T15; 15/15 pass) |
| Predecessor | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md), [ADR 0010](../adr/0010-architecture-as-law.md) |
| Collision fence | No new frameworks/collectors/ISMS/report scanners. No `weeping-angel-catalog` / `weeping-angel-assurance-cli`. No mass ADR renumber. No reopening resolved `DEBT-GUARD-05…12`. Inventory does not replace `cargo xtask guard`. |

Durable finalize artifact for telemetry run `sdd-9cee200795c5`. Product law lives in the linked spec; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)).

---

## Spec

- **Title:** Weeping Angel Structural Reconciliation Program — Phase 0+1
- **Problem:** Live `cargo xtask guard` already passes checks 01–15 (ProductLawCheck 05–12; `DEBT-GUARD-05…12` resolved), but active RI/debt prose still describes 05–12 skip-with-debt archaeology, `baseline-2026-08.md` still claims to be a live counts snapshot, and there is no `cargo xtask inventory` / `docs/debt/current.md` / active-spec drift guard—so contributors cannot trust active docs or regenerate debt counts.
- **Current behavior (pre-implement):** `cargo xtask` accepted only `guard [--json|--check|--explain]`; `xtask/src/inventory.rs` and `docs/debt/current.md` were missing; live guard printed pass for 01–15 while `docs/specs/repository-integrity.md` header/collision fence/current-plane and `docs/debt/README.md` stub section still claimed 05–12 (and historically 14–15) skip-with-debt; `baseline-2026-08.md` was titled as a live 2026-08 counts snapshot (exclusions `target/`, `target-*`, `node_modules/`).
- **Desired behavior (this slice):** Phase 0 freezes new frameworks/collectors/ISMS/report scanners with explicit exit criteria; Phase 1 adds `cargo xtask inventory` (`--json`/`--markdown`/`--check`) in `xtask/src/inventory.rs` with `weeping-angel/inventory/v1` JSON (required counts + exclusions + absences), marks `baseline-2026-08.md` Historical, mechanically generates/checks `docs/debt/current.md`, reconciles `repository-integrity.md` so active Guards 05–12/ADR/baseline language matches the live pass plane (archaeology under Historical), and fails closed on superseded-state phrases in active specs; dual-suite under `xtask/tests/`; guard/fmt/clippy/workspace tests stay green.
- **ADR:** needed — accepted at [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md)

### Acceptance criteria (this slice)

1. Phase 0 freeze is not violated (no new frameworks/collectors/ISMS/report scanners in the Phase 1 diff).
2. `xtask/src/inventory.rs` exists; `cargo xtask inventory` supports `--json`, `--markdown`, and `--check` with documented exit codes.
3. Inventory JSON includes schema `weeping-angel/inventory/v1`, required counts keys, and exclusions `target/`, `target-*`, `node_modules/`.
4. `docs/debt/current.md` is mechanical; `inventory --check` fails on drift.
5. `docs/debt/baseline-2026-08.md` is explicitly Historical; debt README points `current.md` as current.
6. Active `repository-integrity.md` and `docs/debt/README.md` match live Guards 01–15 pass / resolved `DEBT-GUARD-05…12`; stub archaeology is Historical only.
7. Active-spec drift guard fails closed on superseded-state phrases outside Historical/characterization fences.
8. Dual-suite `sdd_structural_reconciliation_{baseline,target}` under `xtask/tests/`; baseline superseded after target GREEN.
9. Spec registered in `CANONICAL_SPECS` and `architecture/spec-lifecycle.toml`.
10. ADR 0048 Accepted after target GREEN; `cargo fmt --all -- --check`, `cargo xtask guard`, `clippy --workspace --all-targets --features demo -- -D warnings`, and `cargo test --workspace --features demo --all-targets` stay green.

### Out of scope

- New framework packs, catalog families, or collectors
- New ISMS/risk/remediation/audit product engines
- New SARIF/report formats or root scanner features
- Mass ADR renumber (`DEBT-DUP-ADR` remains)
- Mass deletion of ignore-superseded baselines
- Reopening resolved `DEBT-GUARD-05…12` as skip hatches
- pnpm / `apps/docs` changes
- Inventing `tests/sdd/` or `test/sdd/*.ts`
- Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`
- Forking `assurance-ir/v1`
- Replacing `cargo xtask guard` with inventory
- Changing ProductLawCheck 05–12 product semantics
- Later Structural Reconciliation phases beyond 0+1
- Broad unrelated README capability rewrites

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Hand-edited `current.md` without `--check` reintroduces count drift | `inventory --check` exit 0; mechanical regenerate after suite changes |
| Over-broad phrase bans may flag legitimate Historical sections | Drift guard scoped to active plane; Historical/characterization fences |
| RI section moves can break citation anchors if Historical text is deleted instead of fenced | Archaeology retained under Historical; active plane rewritten |
| Stale ACP/RI comments can look like product regressions during honesty edits | Neighbor ACP/RI targets and contracts updated for pass-plane language |
| Inventory walks that enter `target/` inflate counts and flake | Exclusions `target/`, `target-*`, `node_modules/` in v1 schema |
| Scope creep treating inventory as a new scanner product | Phase 0 freeze + out-of-scope; inventory is debt/honesty seat only |
| Accepting ADR 0048 before target GREEN yields Accepted-without-proof | ADR Accepted only after SR-T01–T15 GREEN |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md) |
| Baseline | PASS on old | `cargo test -p xtask --test sdd_structural_reconciliation_baseline --test sdd_structural_reconciliation_target` → **pass** (exit 0). SR-B01–B07 GREEN on pre-inventory tree, then skip-superseded after inventory/current.md/RI reconcile/drift guard + target GREEN. Guard 01–15 pass; inventory `--check` exit 0. Excerpt: `running 7 tests` / `test result: ok. 0 passed; 0 failed; 7 ignored` + `running 10 tests` / `test result: ok. 10 passed; 0 failed`. Suites: `xtask/tests/sdd_structural_reconciliation_baseline.rs`, `xtask/tests/sdd_structural_reconciliation_target.rs`. |
| Target pre | FAIL on old | `cargo test -p xtask --test sdd_structural_reconciliation_baseline --test sdd_structural_reconciliation_target` → **fail** (exit 1, expected). Existing SR-T01–T10 already matched implemented surfaces (GREEN); added SR-T11–T15 encoding remaining honesty gaps (SSOT ADR still Draft, unchecked §5 boxes, stale Current plane absences). Product code not modified for RED. Baseline left ignore-superseded. Excerpt: `test result: FAILED. 12 passed; 3 failed` — failures: `sr_t11_adr_0048_accepted_in_ssot_and_meta`, `sr_t12_acceptance_criteria_checkboxes_complete`, `sr_t13_ssot_current_plane_matches_live_inventory`; baseline: `0 passed; 0 failed; 7 ignored`. |
| Implement | target PASS | Dual suite: baseline **ok. 0 passed; 0 failed; 7 ignored** (superseded by `sdd_structural_reconciliation_target`); target **ok. 15 passed; 0 failed**. `cargo xtask guard` → 01–15 pass; `cargo xtask inventory --check` → INV=0; `cargo fmt --all -- --check` → FMT=0; `cargo clippy --workspace --all-targets --features demo -- -D warnings` → EXIT=0; `cargo test --workspace --features demo --all-targets` → TEST=0. Inventory v1, mechanical `docs/debt/current.md`, Historical baseline, RI/debt active-plane reconcile, Guard-15 active-spec drift, ADR 0048 Accepted. |
| Baseline post | FAIL or retired | **Retired by delete** (`supersede_kind=delete`). Before delete: baseline 7 ignored (superseded…); target 15 passed. After delete + sync: `cargo test -p xtask --test sdd_structural_reconciliation_target` → **ok. 15 passed; 0 failed; 0 ignored**. `cargo test -p xtask --test sdd_structural_reconciliation_baseline` → `error: no test target named sdd_structural_reconciliation_baseline` (`BASELINE_DELETED`). `baseline_retired=true`, `baseline_not_green=true`, `additive_baseline=false`. |
| Supersede | target still PASS | Target remains sole SSOT suite. SR-T10 asserts baseline absence; `docs/debt/current.md` regenerated after `ignored_test_attrs` dropped. `inventory --check` → 0. `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md), [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md), [`docs/README.md`](../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`README.md`](../../README.md) |

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

Phase 0+1 structural reconciliation is present and verified:

- `cargo xtask inventory` (`--json` / `--markdown` / `--check`) in `xtask/src/inventory.rs` with schema `weeping-angel/inventory/v1`, required counts, exclusions, and absences.
- Mechanical `docs/debt/current.md`; `inventory --check` fails closed on drift.
- `docs/debt/baseline-2026-08.md` marked Historical; debt README points at `current.md`.
- Active `repository-integrity.md` / debt README match live Guards 01–15 pass and resolved `DEBT-GUARD-05…12`; stub archaeology is Historical only.
- Guard 15 active-spec drift fails closed on superseded-state phrases outside Historical/characterization fences.
- ADR 0048 Accepted. Spec in `CANONICAL_SPECS` + `architecture/spec-lifecycle.toml`.
- Dual-suite target 15/15 GREEN; baseline file deleted after supersede.
- Verify bar green: fmt, guard, clippy (demo), workspace tests (demo). Pre-existing rustc 1.96 workspace clippy cleared so verify stays green.

### Files changed (implement)

`docs/specs/structural-reconciliation.md`, `docs/adr/0048-structural-reconciliation.md`, `xtask/src/inventory.rs`, `xtask/src/lib.rs`, `xtask/src/checks.rs`, `xtask/src/architecture.rs`, `xtask/src/model.rs`, `xtask/tests/sdd_structural_reconciliation_baseline.rs` (later deleted), `xtask/tests/sdd_structural_reconciliation_target.rs`, `docs/debt/current.md`, `docs/debt/baseline-2026-08.md`, `docs/debt/README.md`, `docs/specs/repository-integrity.md`, `docs/specs/architectural-cleanup-program.md`, `architecture/spec-lifecycle.toml`, `tests/contracts/documentation_layout.rs`, `tests/contracts/repository_integrity.target.rs`, `xtask/tests/sdd_architectural_cleanup_target.rs`, plus assurance/control-test clippy clearances under `src/lib.rs` and `crates/weeping-angel-assurance*`, `crates/weeping-angel-control-test*`.

### Docs/ADR (DocsAdr phase)

Accepted ADR 0048 documents the shipped inventory/debt/active-spec-drift seat (Guard 15 + `inventory --check`). READMEs and contracts map point at `cargo xtask inventory` and mechanical `docs/debt/current.md`; SSOT §4.6 records the same seat.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-9cee200795c5` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 11 160 524 |
| `duration_ms_sum` | 4 343 617 (~72.4 min) |
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
| Scope | `sdd-scope` | ok | 155 206 | 294 632 |
| Spec | `sdd-spec` | ok | 385 844 | 807 229 |
| BaselineGreen | `sdd-baseline-green` | ok | 889 702 | 3 011 983 |
| TargetRed | `sdd-target-red` | ok | 221 257 | 418 511 |
| Implement | `sdd-implement` | ok | 1 304 460 | 5 837 520 |
| DocsAdr | `sdd-docs-adr` | ok | 318 450 | 317 052 |
| Iterate | `sdd-baseline-post-check` | ok | 101 751 | 40 634 |
| Supersede | `sdd-supersede` | ok | 966 947 | 432 963 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement).

---

## remaining_backlog (not implemented)

1. Later Structural Reconciliation phases beyond 0+1
2. New framework packs, catalog families, or collectors (Phase 0 freeze)
3. New ISMS/risk/remediation/audit product engines
4. New SARIF/report formats or root scanner features
5. Mass ADR renumber (`DEBT-DUP-ADR` remains)
6. Mass deletion of ignore-superseded baselines (ACP Phase 23 style hygiene)
7. Reopening resolved `DEBT-GUARD-05…12` as skip hatches (forbidden)
8. pnpm / `apps/docs` changes
9. Inventing `tests/sdd/` or `test/sdd/*.ts` (forbidden)
10. Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli` (forbidden)
11. Forking `assurance-ir/v1`
12. Replacing `cargo xtask guard` with inventory (forbidden)
13. Changing ProductLawCheck 05–12 product semantics
14. Broad unrelated README capability rewrites

---

## Related

- Spec SSOT: [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md)
- Decision: [`docs/adr/0048-structural-reconciliation.md`](../adr/0048-structural-reconciliation.md)
- Predecessor: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md)
- Debt current: [`docs/debt/current.md`](../debt/current.md)
- Debt baseline (Historical): [`docs/debt/baseline-2026-08.md`](../debt/baseline-2026-08.md)
- Telemetry: [`sdd-structural-reconciliation-telemetry.json`](sdd-structural-reconciliation-telemetry.json)
