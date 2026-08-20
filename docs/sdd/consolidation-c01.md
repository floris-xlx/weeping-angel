# SDD run: C01 — Contract-test support consolidation (DUP-002 only)

| Field | Value |
| --- | --- |
| Run id | `wa-consolidation-c01` |
| Date | 2026-08-20 |
| Workflow | `xylex-sdd-consolidation` |
| Objective fingerprint | `6bedaac5f0637088` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | **C01 / DUP-002 only** — one crate-private `require_needles` owner, 17 contract consumers migrated, copies deleted, inventory uniqueness pin. **Not** C04–C09, DUP-003, Guard 16, or a new ADR. |
| Spec (run) | [`docs/sdd/c01-contract-test-support-consolidation-run/spec.md`](c01-contract-test-support-consolidation-run/spec.md) |
| Spec (master SSOT) | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) §12 (one-line C01 pointer only) |
| ADR | none this run — governing [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md); uniqueness is not new architecture law |
| Telemetry | [`consolidation-c01-telemetry.json`](consolidation-c01-telemetry.json) |
| Dual-suite | `xtask/tests/sdd_consolidation_c01_*.rs` via `cargo test -p xtask` — **not** `tests/sdd/`, `test/sdd/*.ts`, or a new root/`xtask` `[[test]]` |
| Baseline | `xtask/tests/sdd_consolidation_c01_baseline.rs` — **deleted** after target GREEN (`supersede_kind=delete`; `INV-NO-SUPERSEDED-BASELINES`) |
| Target | [`xtask/tests/sdd_consolidation_c01_target.rs`](../../xtask/tests/sdd_consolidation_c01_target.rs) (C01-T01–T06; 6/6 pass; uniqueness pin) |
| Classify | `disposition=CONSOLIDATE`; `capability_exists=true`; owner `tests/support/mod.rs::require_needles`; `debt_id=DUP-002` |
| Collision fence | No `tests/support.rs` (would raise `tests_rs_autodiscovered` 16→17). No `pub fn` (would raise frozen `public_symbols`). No Guard 16. No DUP-003 filesystem helpers. No second program spec. |

Durable finalize artifact for telemetry run `wa-consolidation-c01`. Product law lives in the linked run spec and master §12 pointer; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)). Generated traces belong under `.sdd/`, not here. The SDD pointer remains [`architectural-consolidation-program.md`](architectural-consolidation-program.md).

Complete = behavioral correctness + repository integrity + architectural ownership (`postcheck.ownership_ok`, `repo_integrity_ok`, no second way).

---

## Spec

- **Title:** C01 — Contract-test support consolidation (DUP-002 only)
- **Problem:** Seventeen contract dual-suites each define a private `fn require_needles`; inventory reports 18 files because `xtask/src/inventory.rs` substring-matches itself. There is no shared owner, so uniqueness is not a gate and copies drift.
- **Current behavior (pre-implement):** 17 competing crate-private `require_needles` copies in `tests/contracts/*.target.rs` (iso27001 uses `haystack` vs `src`; same types). Each `[[test]]` binary is the only consumer of its copy. `tests/support/` does not exist. Live inventory: `require_needles_fns=18`, `require_needles_calls=222`, `duplicate_helper_definitions=18`, `root_test_binaries=45`, `tests_rs_autodiscovered=16`, `public_symbols=2022` (freeze; live characterization noted 2035). DUP-002 `status=confirmed`. Guard 04 forbids increases vs freeze; matcher is `contains("fn require_needles")`.
- **Desired behavior:** Exactly one crate-private `fn require_needles(label, src, needles)` in `tests/support/` included via `include!` or `#[path]` by all 17 `sdd_*_target` binaries; per-file copies deleted with no aliases; inventory matcher `starts_with("fn require_needles")` so `require_needles_fns` and `duplicate_helper_definitions` equal 1; calls stay 222 or drop; dual-suite baseline GREEN on CURRENT then deleted after target GREEN; uniqueness pinned by staying C01 target plus Guard 04; DUP-002 closed only if close law holds.
- **ADR:** not needed (`classify.adr_action=none`; uniqueness is not promoted to architecture law).

### Classify (binding)

| Key | Value |
| --- | --- |
| `capability_exists` | `true` |
| `disposition` | `CONSOLIDATE` |
| `bounded_context` | `contract-test-support` |
| `canonical_owner` | `tests/support/mod.rs::require_needles` |
| `debt_id` | `DUP-002` |
| `adr_action` | `none` |
| `new_adr_required` | `false` |
| `new_public_surface` | `false` |
| `new_persistence` | `false` |
| `close_law` | verified/removed only when canonical owner exists, all consumers use it, old per-file copies are gone (not aliases), and a regression guard exists (inventory uniqueness / expansion freeze — **not** Guard 16) |

### Acceptance criteria (this slice)

1. Baseline GREEN on CURRENT: 17 tests/contracts copies and live `require_needles_fns==18` with `contains` matcher.
2. Target RED on CURRENT then GREEN: one `starts_with("fn require_needles")` definition, 17 consumers migrated, copies gone.
3. Canonical signature `fn require_needles(label: &str, src: &str, needles: &[&str])`; iso27001 haystack call sites keep the same types; existing needles unchanged.
4. Owner is `tests/support/` directory (`mod.rs` or `require_needles.rs`) via `include!`/`#[path]`; not `tests/support.rs`, `tests/support/main.rs`, `tests/sdd/`, or a new `contracts/*.rs`.
5. `root_test_binaries` stays 45; `tests_rs_autodiscovered` stays 16; `tests_contracts_rs` stays 43; `KNOWN_CHECK_IDS` len stays 15; no Guard 16; no `pub fn` helper.
6. `require_needles_fns` and `duplicate_helper_definitions` become 1; `require_needles_calls` ≤ 222; `public_symbols` does not increase vs 2022 freeze.
7. Extract, migrate, delete copies, and tighten inventory in one change so `require_needles_fns` never increases.
8. `cargo test -p xtask` green after target GREEN and baseline delete; all 17 `sdd_*_target` binaries green.
9. `cargo xtask guard` (01–15) green; `cargo xtask inventory` refreshes `docs/debt/current.md`.
10. DUP-002 updated from evidence; verified only if owner exists, consumers migrated, copies gone (not aliases), and uniqueness guard exists.
11. C01 baseline file deleted after GREEN (`INV-NO-SUPERSEDED-BASELINES`), not `#[ignore]`; target kept as uniqueness pin.
12. Master program spec unchanged except the one-line C01 pointer.

### Out of scope

- C04–C09 readiness/applicability/lineage/SoA/temporal work
- DUP-003 filesystem helpers (`manifest_dir`, `read_repo_file`, `crate_sources_joined`, `forbid_needles`)
- `tests/sdd/`, new root or xtask `[[test]]`, Guard 16
- Product types, persistence, public API, new crates, pnpm/`apps/docs`
- Second giant consolidation/cleanup program spec
- New ADR (uniqueness not promoted to architecture law)
- Rebasing `docs/debt/consolidation-baseline.json`
- Weakening needles or replacing them with semantic tests (C16)

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| `tests/support.rs` would raise `tests_rs_autodiscovered` 16→17 | Owner is `tests/support/mod.rs` included by existing binaries; not a new autodiscovered `tests/*.rs` |
| `pub fn` would raise frozen `public_symbols` 2022 | Helper is crate-private `fn require_needles` |
| `starts_with` would miss `pub(crate) fn` and count 0 | Signature is `fn require_needles(`; matcher `trimmed.starts_with("fn require_needles")`; live `fns=1` |
| Extract-without-delete would raise `require_needles_fns` and fail Guard 04 | Extract, migrate, and delete copies in one change |
| Expansion freeze still allows 1→18 unless C01 target stays | `sdd_consolidation_c01_target` kept as uniqueness pin; Guard 04 still forbids increase vs freeze |
| Target RED on unrelated product needles instead of helper uniqueness | RED was C01-T01/T02/T03/T04/T06 on fns=18, missing `tests/support/`, 17 copies, `contains` matcher |
| `#[ignore]` leftover baseline violates `INV-NO-SUPERSEDED-BASELINES` | Baseline **deleted**; `cargo test --test sdd_consolidation_c01_baseline` has no such target |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/c01-contract-test-support-consolidation-run/spec.md`](c01-contract-test-support-consolidation-run/spec.md) |
| Baseline | PASS on old | `cargo test -p xtask -- --nocapture` → **pass** (exit 0). Characterization only: 17 contract copies + inventory `contains` matcher file (`fns=18`, `calls=222`). `tests/support` absent; DUP-002 `status=confirmed`. Live `public_symbols=2035` vs freeze snapshot 2022. No extract/migration. Excerpt: `running 5 tests` / `c01_b04_inventory_matcher_is_contains_and_freeze_is_eighteen` … `c01_b05_expansion_counts_and_dup002_still_confirmed` / `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s`. Suite: `xtask/tests/sdd_consolidation_c01_baseline.rs`. |
| Target pre | FAIL on old | `cargo test -p xtask -- --nocapture` → **fail** (exit 1, expected). Target encodes DUP-002 close law (one crate-private helper in `tests/support` via `include!`/`#[path]`, 17 consumers migrated, copies gone, `starts_with` matcher, `fns==1`, `calls<=222`). RED on CURRENT as required; baseline untouched and GREEN. No product implementation. Excerpt: `test result: FAILED. 1 passed; 5 failed`; `c01_t01 left: 18 right: 1`; `c01_t02 canonical owner home is tests/support/ directory`; `c01_t03 per-file copies ... still in [17 tests/contracts/*.target.rs]`; `c01_t04 matcher must be trimmed.starts_with`; `c01_t06 close law: old copies gone, still [17 files]`; baseline: `test result: ok. 5 passed`. Suite: `xtask/tests/sdd_consolidation_c01_target.rs`. |
| Implement | target PASS | `cargo test -p xtask --test sdd_consolidation_c01_target -- --nocapture` → **ok. 6 passed; 0 failed**. CONSOLIDATE DUP-002: one crate-private `fn require_needles` in `tests/support/mod.rs`, 17 contract binaries `include!` it, copies deleted, inventory matcher `starts_with` so `require_needles_fns==1` (`calls=206`). All 17 `sdd_*_target` binaries green (15+20+16+14+12+18+10+49+12+12+36+13+15+15+17+16+15). `cargo test -p xtask -- --nocapture`: lib 9, `debt_register` 5, cleanup 17, consolidation 20, c01_target 6, SR 15. `cargo xtask guard` 01–15 all pass. `cargo xtask inventory --check` exit 0. Inventory: `require_needles_fns=1` `require_needles_calls=206` `root_test_binaries=45` `tests_rs_autodiscovered=16` `tests_contracts_rs=43`. DUP-002 verified. |
| Baseline post | FAIL or retired | **Retired by delete** (`supersede_kind=delete`). After target GREEN the characterization file is gone (`INV-NO-SUPERSEDED-BASELINES`), not `#[ignore]`. `cargo test -p xtask --test sdd_consolidation_c01_baseline -- --nocapture` → `error: no test target named sdd_consolidation_c01_baseline in xtask package` (exit 101). `Test-Path xtask/tests/sdd_consolidation_c01_baseline.rs` → `False` (`BASELINE_ABSENT`). `baseline_retired=true`, `baseline_not_green=true`, `additive_baseline=false`. |
| Supersede | target still PASS | `cargo test -p xtask --test sdd_consolidation_c01_target -- --nocapture` → **ok. 6 passed; 0 failed; 0 ignored** (C01-T01–T06). Full `cargo test -p xtask -- --nocapture` stays green (no `sdd_consolidation_c01_baseline` harness). DUP-002 remains pinned by `sdd_consolidation_c01_target`. `target_still_green=true`. |
| Docs/ADR | updated | No new ADR (`classify.adr_action=none`). [`docs/sdd/c01-contract-test-support-consolidation-run/spec.md`](c01-contract-test-support-consolidation-run/spec.md), [`docs/contracts/README.md`](../contracts/README.md), [`tests/contracts/README.md`](../../tests/contracts/README.md), [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) (one-line C01 pointer), [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md), [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md), [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml), [`docs/debt/migrations/DUP-002.toml`](../debt/migrations/DUP-002.toml), [`docs/debt/migrations/DUP-003.toml`](../debt/migrations/DUP-003.toml), [`docs/debt/migrations/QUEUE.toml`](../debt/migrations/QUEUE.toml), [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md), [`architecture/maintainability.toml`](../../architecture/maintainability.toml) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `delete` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |
| `arch_postcheck` | `true` |

`verify_ok` = `target_still_green` ∧ `arch_postcheck` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ constrained additive) = **true**.

---

## What landed

CONSOLIDATE C01 / DUP-002 is present and verified:

- One crate-private `fn require_needles(label: &str, src: &str, needles: &[&str])` in [`tests/support/mod.rs`](../../tests/support/mod.rs).
- All 17 `sdd_*_target` contract binaries `include!` that owner; iso27001 haystack call sites keep the same types; existing needles unchanged.
- Per-file copies deleted (not aliases). Inventory matcher is `trimmed.starts_with("fn require_needles")` so `require_needles_fns=1` and `duplicate_helper_definitions=1`; `require_needles_calls=206` (≤ 222).
- Freeze neighbors hold: `root_test_binaries=45`, `tests_rs_autodiscovered=16`, `tests_contracts_rs=43`, `KNOWN_CHECK_IDS` len 15, no Guard 16, no `pub fn` helper, `public_symbols` not increased vs freeze 2022.
- DUP-002 verified from evidence: owner exists, consumers migrated, copies gone, uniqueness guard exists (C01 target + Guard 04 + inventory matcher).
- Dual-suite under `xtask/tests` only. Target C01-T01–T06 GREEN. Characterization baseline deleted (`INV-NO-SUPERSEDED-BASELINES`).
- `cargo xtask guard` 01–15 green. `cargo xtask inventory --check` exit 0. `docs/debt/current.md` refreshed.
- No new ADR. Master program spec unchanged except the one-line C01 pointer. Frozen `consolidation-baseline.json` not rebased. DUP-003 filesystem helpers untouched.

Arch postcheck: `ownership_ok=true`, `repo_integrity_ok=true`, `second_way=false`, `inventory_increased=false`, `added_public_type=false`, `suspicious=false`. `architecture.toml` still crate-level SSOT (forbidden for C01 edits except concurrent ADR 0050 comments outside this increment).

### Files changed (implement)

`tests/support/mod.rs`, `xtask/src/inventory.rs`, `docs/debt/structural-duplication.toml`, `docs/debt/current.md`, `tests/contracts/assessment_lineage.target.rs`, `tests/contracts/control_implementation_registry.target.rs`, `tests/contracts/controlled_documents.target.rs`, `tests/contracts/continuity_resilience.target.rs`, `tests/contracts/iso27001_assurance.target.rs`, `tests/contracts/interested_parties_obligations.target.rs`, `tests/contracts/incident_governance.target.rs`, `tests/contracts/nonconformity_capa.target.rs`, `tests/contracts/internal_audit.target.rs`, `tests/contracts/operational_soa.target.rs`, `tests/contracts/population_runtime.target.rs`, `tests/contracts/remediation_engine.target.rs`, `tests/contracts/risk_register.target.rs`, `tests/contracts/supplier_risk.target.rs`, `tests/contracts/temporal_lineage_evidence_soa.target.rs`, `tests/contracts/temporal_assurance.target.rs`, `tests/contracts/typed_evidence.target.rs`, `xtask/tests/sdd_consolidation_c01_baseline.rs` (deleted after target GREEN).

### Docs/ADR (DocsAdr phase)

No new ADR (`classify.adr_action=none`; governing ADR 0049). C01 run spec marked implemented (baseline deleted, target pin). Master §12 C01 pointer notes DUP-002 verified at `require_needles_fns=1`. Contract READMEs document `include!` of `tests/support/mod.rs`. DUP-002 evidence records `calls=206`; migration contract/QUEUE note close. Hygiene/SR current-plane updated so `tests/support` is the C01 owner, not an absence.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `wa-consolidation-c01` |
| `agents_ok` | 9 |
| `agents_fail` | 0 |
| `agents_total` | 9 |
| `tokens_used_sum` | 7 242 260 |
| `duration_ms_sum` | 2 688 434 (~44.8 min) |
| `budget.total` | 48 |
| `budget.spent` | 9 |
| `budget.remaining` | 39 |
| `event_count` | 35 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |
| `reason` | `pre_finalize` |
| `workflow` | `xylex-sdd-consolidation` |

### Gates

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `arch_postcheck` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agent phases

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 227 656 | 455 114 |
| Classify | `sdd-classify` | ok | 199 548 | 333 999 |
| Spec | `sdd-spec` | ok | 389 276 | 786 021 |
| BaselineGreen | `sdd-baseline-green` | ok | 272 341 | 673 923 |
| TargetRed | `sdd-target-red` | ok | 390 043 | 867 917 |
| Implement | `sdd-implement` | ok | 419 325 | 2 040 433 |
| DocsAdr | `sdd-docs-adr` | ok | 465 451 | 1 290 741 |
| ArchPostcheck | `sdd-arch-postcheck` | ok | 228 081 | 554 725 |
| Supersede | `sdd-supersede` | ok | 96 713 | 239 387 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement; no iterate agent).

---

## remaining_backlog (not implemented)

1. C04–C09 readiness/applicability/lineage/SoA/temporal work
2. DUP-003 filesystem helpers (`manifest_dir`, `read_repo_file`, `crate_sources_joined`, `forbid_needles`)
3. `tests/sdd/`, new root or xtask `[[test]]`, Guard 16 (still forbidden)
4. Product types, persistence, public API, new crates, pnpm/`apps/docs`
5. Second giant consolidation/cleanup program spec (still forbidden)
6. New ADR promoting uniqueness to architecture law (not this increment)
7. Rebasing `docs/debt/consolidation-baseline.json` (still forbidden)
8. Weakening needles or replacing them with semantic tests (C16)

---

## Related

- Run spec: [`docs/sdd/c01-contract-test-support-consolidation-run/spec.md`](c01-contract-test-support-consolidation-run/spec.md)
- Master program SSOT: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)
- SDD pointer: [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md)
- Governing ADR: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)
- Owner: [`tests/support/mod.rs`](../../tests/support/mod.rs)
- Debt: [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml), [`docs/debt/migrations/DUP-002.toml`](../debt/migrations/DUP-002.toml)
- Live inventory: [`docs/debt/current.md`](../debt/current.md)
- Target suite: [`xtask/tests/sdd_consolidation_c01_target.rs`](../../xtask/tests/sdd_consolidation_c01_target.rs)
- Telemetry: [`consolidation-c01-telemetry.json`](consolidation-c01-telemetry.json)
