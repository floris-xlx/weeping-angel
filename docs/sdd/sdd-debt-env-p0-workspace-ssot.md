# SDD run: DEBT-ENV P0 — Cargo workspace SSOT (virtual workspace, apps/cli, toolchain pin, one test harness)

| Field | Value |
| --- | --- |
| Run id | `sdd-cf5759e37523` |
| Date | 2026-08-20 |
| Workflow | `xylex-sdd-consolidation` |
| Objective fingerprint | `07240381acca80eb` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | **DEBT-ENV P0 only** — virtual Cargo workspace, `apps/cli` package name `weeping-angel`, workspace tables, pinned `rust-toolchain.toml` consumed by CI, one `[[test]]` harness (`root_test_binaries` 45→1), consumers migrated. **Not** P1–P3, Guard 16, Turbo, or `weeping-angel-assurance-cli`. |
| Spec (run) | [`docs/sdd/debt-env-p0-workspace-ssot-run/spec.md`](debt-env-p0-workspace-ssot-run/spec.md) |
| Spec (master SSOT) | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) — unchanged except a one-line DEBT-ENV pointer |
| ADR | Accepted [`docs/adr/0051-repository-environment.md`](../adr/0051-repository-environment.md); path-fact amends on [`0009`](../adr/0009-repository-health-gate.md) / [`0001`](../adr/0001-inwardly-extensible-assurance-runtime.md); consumer amends on [`0004`](../adr/0004-documentation-architecture.md) / [`0012`](../adr/0012-repository-hygiene.md) |
| Telemetry | [`sdd-debt-env-p0-workspace-ssot-telemetry.json`](sdd-debt-env-p0-workspace-ssot-telemetry.json) |
| Dual-suite | `xtask/tests/sdd_consolidation_debt_env_*.rs` via `cargo test -p xtask` — **not** `tests/sdd/`, `test/sdd/*.ts`, or a new root/`xtask` `[[test]]` |
| Baseline | `xtask/tests/sdd_consolidation_debt_env_baseline.rs` — **deleted** after target GREEN (`supersede_kind=delete`; `INV-NO-SUPERSEDED-BASELINES`) |
| Target | [`xtask/tests/sdd_consolidation_debt_env_target.rs`](../../xtask/tests/sdd_consolidation_debt_env_target.rs) (env_t01–t11; 11/11 pass) |
| Classify | `disposition=CONSOLIDATE`; `capability_exists=true`; owner `xtask` + `architecture/`; `debt_id=DEBT-ENV`; `bounded_context=repository-guard` |
| Collision fence | Package name stays `weeping-angel` (never `weeping-angel-assurance-cli`). `workspace_crates` stays 9 (no 10th crate). No Guard 16, Turbo, `tests/sdd/`, or new root/xtask `[[test]]`. `INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays pass. Frozen `docs/debt/consolidation-baseline.json` not rebased. |

Durable finalize artifact for telemetry run `sdd-cf5759e37523`. Product law lives in the linked run spec, ADR 0051, and master program pointer; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)). Generated traces belong under `.sdd/`, not here.

Complete = behavioral correctness + repository integrity + architectural ownership (`postcheck.ownership_ok`, `repo_integrity_ok`, no second way).

---

## Spec

- **Title:** DEBT-ENV P0 — Cargo workspace SSOT (virtual workspace, apps/cli, toolchain pin, one test harness)
- **Problem:** Repository-guard already exists (xtask + architecture), but the live tree still fuses root `Cargo.toml` as workspace+CLI+release+`[[test]]` catalog, copies crate metadata, uses relative internal paths, and lets CI float on `dtolnay/rust-toolchain@stable`. Environment consolidation is the prerequisite before more ISO/ISMS work.
- **Current behavior (pre-implement):** Root `Cargo.toml` is workspace plus package `weeping-angel` at `src/main.rs` with 45 `[[test]]` rows, dist/packager metadata, and no `[workspace.package]`/`[workspace.dependencies]`. Seven internal crates copy version/edition/license and `path = "../weeping-angel-*"`. `rust-toolchain.toml` is absent; four GitHub workflows use `dtolnay/rust-toolchain@stable`. Inventory `workspace_crates=9`, `root_test_binaries=45`. Nested `apps/docs` pnpm lock/workspace and postinstall `generate:docs` remain (P1, not this slice).
- **Desired behavior:** Virtual root workspace with `[workspace.package]` and `[workspace.dependencies]`; CLI package name remains `weeping-angel` at `apps/cli` (never `weeping-angel-assurance-cli`); internal deps `workspace=true`; `rust-toolchain.toml` pins a versioned channel and CI consumes it; one `[[test]]` harness on the CLI package (`root_test_binaries` 45→1) with consumers migrated; `workspace_crates` stays 9. Product CLI/scanner/assurance behavior unchanged. P1–P3 stay later.
- **ADR:** needed — [`docs/adr/0051-repository-environment.md`](../adr/0051-repository-environment.md) Accepted after target GREEN.

### Classify (binding)

| Key | Value |
| --- | --- |
| `capability_exists` | `true` |
| `disposition` | `CONSOLIDATE` |
| `bounded_context` | `repository-guard` |
| `canonical_owner` | `xtask` |
| `debt_id` | `DEBT-ENV` |
| `adr_action` | `new` |
| `new_adr_required` | `true` |
| `governing_adr` | [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md) |
| `new_public_surface` | `false` |
| `new_persistence` | `false` |
| `next_workflow` | `xylex-sdd-consolidation` |
| `close_law` | Canonical environment policy is architecture + xtask (no second health CLI, no Guard 16, no Turbo). One Cargo workspace SSOT (`[workspace.package]`/`[workspace.dependencies]`, workspace-owned internal paths); CLI package name remains `weeping-angel` (path may move to `apps/cli`, never `weeping-angel-assurance-cli`); consumers migrated; `INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays green. P1 one pnpm workspace/lockfile and P2/P3 topology/release remain later. |

### Acceptance criteria (this slice)

1. Baseline GREEN on CURRENT: fused root workspace+package, no `workspace.package`/`dependencies`, relative internal paths, no `rust-toolchain.toml`, four workflows use `@stable`, `root_test_binaries=45`, `workspace_crates=9`, CLI at `src/main.rs`, `KNOWN_CHECK_IDS.len()==15`.
2. Target RED on CURRENT then GREEN after P0: virtual workspace, `apps/cli` named `weeping-angel`, workspace tables present, `workspace=true` internals, pinned `rust-toolchain.toml` consumed by CI, one `[[test]]` harness, architecture paths follow `apps/cli`.
3. `root_test_binaries` decreases 45→1; inventory counts the `weeping-angel` package `[[test]]` tables once root is virtual.
4. `workspace_crates` remains 9 (7 libraries + xtask + `apps/cli`); no 10th crate.
5. Package name stays `weeping-angel`; `FORBID-HYPOTHETICAL-ASSURANCE-CLI` holds; no Guard 16, Turbo, `tests/sdd/`, or new root/xtask `[[test]]`.
6. `INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays pass (no frozen metric increase).
7. Consumers migrated in the implement PR: architecture/domain-ownership paths, dist/packager/CI, `CARGO_MANIFEST_DIR` helpers, contract `Cargo.toml` `[[test]]` greps, ADR 0004/0012 wording.
8. After target GREEN, delete `sdd_consolidation_debt_env_baseline.rs` (never `#[ignore]`); keep the target as the pin; accept ADR 0051.

### Out of scope

- P1 root pnpm workspace, nested `apps/docs` lock/workspace, postinstall generation, JS CI, install/generate/verify/build command split
- P2 `cargo xtask doctor|ci|docs` and topology/dep-direction guards as new check IDs
- P3 release SSOT, short crate directories, empty `demo=[]` removal, `.gitignore` class policy
- New crate, Turbo, Guard 16, second health CLI, `weeping-angel-assurance-cli`
- `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]`
- Product ISO/ISMS features, public API, persistence, catalog/IR semantics
- Rewriting `docs/specs/architectural-consolidation-program.md` into a 32-phase env program
- Rebasing `docs/debt/consolidation-baseline.json`

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Virtual workspace plus `apps/cli` counted as a 10th crate | Inventory counts 7 libraries + xtask + `apps/cli` = 9; virtual root is not a package |
| `CARGO_MANIFEST_DIR` becoming `apps/cli` breaks `tests/support` and fixtures | Helpers/consumers migrated in implement; env_t09 pins dist/CI/`CARGO_MANIFEST_DIR` |
| One harness collides on `#[test]` names across former binaries | Single `apps/cli/tests/harness.rs`; env_t05 inventory `root_test_binaries=1` |
| `[[test]]` collapse without migrating contract `Cargo.toml` greps | env_t10 catalog greps and ADR wording migrated |
| Inventory still counting virtual root `[[test]]` → metric 0 instead of 1 | Counter retargeted to `apps/cli` `Cargo.toml`; live `root_test_binaries=1` |
| One workflow left on `dtolnay/rust-toolchain@stable` | Four workflows consume the pin; env_t04 |
| Hypothetical `weeping-angel-assurance-cli` or Guard 16 for topology | `FORBID-HYPOTHETICAL-ASSURANCE-CLI`; `KNOWN_CHECK_IDS` still 15; env_t07 close-law |
| `#[ignore]` leftover baseline after GREEN | Baseline **deleted**; `cargo test --test sdd_consolidation_debt_env_baseline` has no such target |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/debt-env-p0-workspace-ssot-run/spec.md`](debt-env-p0-workspace-ssot-run/spec.md) |
| Baseline | PASS on old | `cargo test -p xtask --test sdd_consolidation_debt_env_baseline -- --nocapture` → **pass** (exit 0). Characterization of CURRENT fused Cargo environment only. No workspace SSOT, `rust-toolchain.toml`, `apps/cli` move, or `[[test]]` collapse implemented. Target suite remains RED by design and was not part of this GREEN run. Excerpt: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s`. Suite: `xtask/tests/sdd_consolidation_debt_env_baseline.rs`. |
| Target pre | FAIL on old | `cargo test -p xtask --test sdd_consolidation_debt_env_baseline --test sdd_consolidation_debt_env_target -- --nocapture` → **fail** (exit 1, expected). Baseline GREEN on CURRENT (fused root, 45 `[[test]]` rows, `workspace_crates=9`). Target RED on CURRENT; no product code changed. Passing target tests are stay-green close-law pins (Guard 01–15, no Turbo/health CLI, `INV-CONSOLIDATION-EXPANSION-RESTRICTED`). P1–P3 (nested pnpm/postinstall) not asserted. Excerpt: `baseline: test result: ok. 11 passed; 0 failed` / `target: test result: FAILED. 2 passed; 9 failed`; failures: `env_t01`…`env_t06`, `env_t08`…`env_t10`; `env_t05: left: 45 right: 1`; `env_t07_close_law_no_new_plane ... ok`; `env_t11_expansion_restricted_stays_pass ... ok`. Suite: `xtask/tests/sdd_consolidation_debt_env_target.rs`. |
| Implement | target PASS | `cargo test -p xtask --test sdd_consolidation_debt_env_target -- --nocapture` → **ok. 11 passed; 0 failed** (finished in 0.62s). P0 CONSOLIDATE: virtual Cargo workspace + `apps/cli` `weeping-angel`, workspace tables, pinned toolchain, one harness (45→1), inventory/architecture/CI/ADR consumers migrated. Combined dual-suite after delete: `error: no test target named sdd_consolidation_debt_env_baseline in xtask package`. `cargo xtask guard`: 01–15 pass; inventory `root_test_binaries=1` `workspace_crates=9`. |
| Baseline post | FAIL or retired | **Retired by delete** (`supersede_kind=delete`). Characterization baseline deleted after target GREEN (`INV-NO-SUPERSEDED-BASELINES`), never `#[ignore]`. `cargo test -p xtask --test sdd_consolidation_debt_env_baseline --test sdd_consolidation_debt_env_target -- --nocapture` → `error: no test target named sdd_consolidation_debt_env_baseline in xtask package` (exit 101). `Test-Path xtask/tests/sdd_consolidation_debt_env_baseline.rs` → `False` (`BASELINE_ABSENT`). `baseline_retired=true`, `baseline_not_green=true`, `additive_baseline=false`. |
| Supersede | target still PASS | `cargo test -p xtask --test sdd_consolidation_debt_env_target -- --nocapture` → **ok. 11 passed; 0 failed; 0 ignored** (env_t01–t11; finished in 0.50s). Target remains the SSOT pin. `target_still_green=true`. skip/additive forbidden. |
| Docs/ADR | updated | [`docs/adr/0051-repository-environment.md`](../adr/0051-repository-environment.md), [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md), [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md), [`docs/adr/0012-repository-hygiene.md`](../adr/0012-repository-hygiene.md), [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md), [`README.md`](../../README.md), [`docs/README.md`](../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`tests/contracts/README.md`](../../tests/contracts/README.md), [`docs/sdd/debt-env-p0-workspace-ssot-run/spec.md`](debt-env-p0-workspace-ssot-run/spec.md), [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md), [`docs/specs/assurance-runtime-spine.md`](../specs/assurance-runtime-spine.md), [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md), [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [`docs/debt/README.md`](../debt/README.md), [`docs/debt/register.toml`](../debt/register.toml), [`docs/debt/current.md`](../debt/current.md) |

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

CONSOLIDATE DEBT-ENV P0 is present and verified:

- Virtual root Cargo workspace with `[workspace.package]` and `[workspace.dependencies]`; CLI package name remains `weeping-angel` at `apps/cli` (never `weeping-angel-assurance-cli`).
- Internal crates inherit metadata and use `workspace=true` (no copied version/edition/license; no `path = "../weeping-angel-*"`).
- `rust-toolchain.toml` pins a versioned channel; four GitHub workflows consume it (not `dtolnay/rust-toolchain@stable`).
- One `[[test]]` harness on the CLI package: inventory `root_test_binaries` 45→1; `workspace_crates` stays 9 (7 libraries + xtask + `apps/cli`); no 10th crate.
- Consumers migrated: architecture/domain-ownership/`forbidden-patterns` paths, dist/packager/CI, `CARGO_MANIFEST_DIR` helpers, contract `Cargo.toml` greps, ADR 0004/0012 wording.
- Dual-suite under `xtask/tests` only. Target env_t01–t11 GREEN. Characterization baseline deleted (`INV-NO-SUPERSEDED-BASELINES`).
- `cargo xtask guard` 01–15 green. `KNOWN_CHECK_IDS` length 15. No Guard 16, Turbo, `tests/sdd/`, or second health CLI.
- `INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays pass. Frozen `docs/debt/consolidation-baseline.json` not rebased.
- ADR 0051 Accepted as the environment SSOT (architecture + xtask). Product CLI/scanner/assurance behavior unchanged. P1–P3 remain in-progress.

Arch postcheck: `ownership_ok=true`, `repo_integrity_ok=true`, `second_way=false`, `inventory_increased=false`, `added_public_type=false`, `added_parser_projection_dto=false`, `new_cross_crate_dep=false`, `architecture_toml_matches=true`, `suspicious=false`. Inventory freeze neighbors: `workspace_crates=9`, `root_test_binaries=1`, `public_structs=397<=523`, `public_enums=193<=221`, `pub_use=104<=110`, `duplicate_helper_definitions=1<=18`.

### Files changed (implement)

`Cargo.toml`, `apps/cli/Cargo.toml`, `apps/cli/tests/harness.rs`, `rust-toolchain.toml`, `xtask/Cargo.toml`, `xtask/src/inventory.rs`, `xtask/src/architecture.rs`, `architecture/architecture.toml`, `architecture/domain-ownership.toml`, `architecture/forbidden-patterns.toml`, `dist-workspace.toml`, `docs/adr/0004-documentation-architecture.md`, `docs/adr/0012-repository-hygiene.md`, `docs/adr/0051-repository-environment.md`, `docs/contracts/README.md`, `docs/debt/current.md`, `tests/support/mod.rs`, `.github/workflows/ci.yml`, `.github/workflows/compliance-regression.yml`, `.github/workflows/security-diff.yml`, `.github/workflows/release-provenance.yml`, `crates/weeping-angel-assurance/Cargo.toml`, `crates/weeping-angel-canonical-catalog/src/lib.rs`, `crates/weeping-angel-framework/src/pack.rs`, `xtask/tests/sdd_consolidation_c01_target.rs`. Baseline `xtask/tests/sdd_consolidation_debt_env_baseline.rs` deleted after target GREEN.

### Docs/ADR (DocsAdr phase)

ADR 0051 Accepted as the environment SSOT (architecture + xtask). Path-fact amends on 0009/0001; consumer amends on 0004/0012. README/contracts/specs and DEBT-ENV register now match the virtual workspace, `apps/cli` `weeping-angel`, one harness, and pinned toolchain. P1–P3 remain in-progress.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-cf5759e37523` |
| `agents_ok` | 9 |
| `agents_fail` | 0 |
| `agents_total` | 9 |
| `tokens_used_sum` | 23 183 955 |
| `duration_ms_sum` | 4 720 622 (~78.7 min) |
| `budget.total` | 128 |
| `budget.spent` | 9 |
| `budget.remaining` | 119 |
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
| Scope | `sdd-scope` | ok | 220 044 | 485 114 |
| Classify | `sdd-classify` | ok | 189 523 | 395 050 |
| Spec | `sdd-spec` | ok | 642 095 | 2 198 609 |
| BaselineGreen | `sdd-baseline-green` | ok | 175 559 | 481 830 |
| TargetRed | `sdd-target-red` | ok | 437 067 | 1 065 820 |
| Implement | `sdd-implement` | ok | 2 042 352 | 15 566 786 |
| DocsAdr | `sdd-docs-adr` | ok | 702 269 | 2 320 600 |
| ArchPostcheck | `sdd-arch-postcheck` | ok | 226 770 | 454 474 |
| Supersede | `sdd-supersede` | ok | 84 943 | 215 672 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement; no iterate agent).

---

## remaining_backlog (not implemented)

1. P1 root pnpm workspace, nested `apps/docs` lock/workspace, postinstall generation, JS CI, install/generate/verify/build command split
2. P2 `cargo xtask doctor|ci|docs` and topology/dep-direction guards as new check IDs
3. P3 release SSOT, short crate directories, empty `demo=[]` removal, `.gitignore` class policy
4. New crate, Turbo, Guard 16, second health CLI, `weeping-angel-assurance-cli` (still forbidden)
5. `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]` (still forbidden)
6. Product ISO/ISMS features, public API, persistence, catalog/IR semantics
7. Rewriting `docs/specs/architectural-consolidation-program.md` into a 32-phase env program (still forbidden)
8. Rebasing `docs/debt/consolidation-baseline.json` (adr_count 51 and debt_rows 25 vs freeze 49/24 remain untracked by `INV-CONSOLIDATION-EXPANSION-RESTRICTED`)
