# SDD run: Repository Integrity increment 1 — executable architecture health gate

| Field | Value |
| --- | --- |
| Run id | `sdd-f0d2a357dd63` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `f0d2a357dd63c911` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Increment 1: `architecture/*.toml` + `docs/debt/register.toml` SSOT; `cargo xtask guard` checks 01/02/03/13; fail-closed stubs 04–12/14–15; CI-mandatory guard. **Not** P0 remediations. |
| Spec | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) |
| ADR | Accepted [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md) |
| Pointer stub | [`docs/sdd/repository-integrity.md`](repository-integrity.md) |
| Telemetry | [`.sdd/runs/repository-integrity-health-gate-telemetry.json`](../../.sdd/runs/repository-integrity-health-gate-telemetry.json) |
| Dual-suite | `tests/contracts/repository_integrity.baseline.rs` (skip-retired; 12 ignored) · `tests/contracts/repository_integrity.target.rs` (active; RI-T01–T17 / 18 tests) |
| Characterization SHA | `f560196c57e77df2573cfb9a4b384d3cf1c21e8a` |
| Collision fence | Do not implement remaining_backlog. Do not invent `weeping-angel-catalog` or `weeping-angel-assurance-cli`. Do not mint another `0003-*` ADR. `xtask` stays `publish = false`. Successor increment: [ADR 0010](../adr/0010-architecture-as-law.md). |

Durable finalize artifact for telemetry run `sdd-f0d2a357dd63`. Product law lives in the linked spec; this file records protocol evidence, gates, and telemetry.

Spec-first increment 1: `docs/specs/repository-integrity.md` (SSOT), `docs/sdd` pointer stub, draft then Accepted ADR 0009. The gate is `cargo xtask guard` checks 01/02/03/13 plus fail-closed stubs; P0 remediations remain remaining_backlog.

---

## Spec

- **Title:** Repository Integrity increment 1 — executable architecture health gate
- **Problem:** The assurance workspace has no machine-checked concept ownership, debt register, or fail-closed health command, so later P0 remediations can land without regression proof and crate names can drift from the live tree.
- **Current behavior (SHA `f560196c`):** On SHA `f560196c` there is no `architecture/`, `docs/debt/`, xtask crate, or `.cargo/config.toml`. Workspace members are only the seven `weeping-angel-*` crates; the CLI is root `src/main.rs` + `src/cli.rs`. `cargo xtask` and `cargo run -p xtask` fail. CI runs fmt/clippy/test with `--features demo --all-targets` and does not contain `xtask guard`. ADR files reuse IDs (`0003`×25, `0005`×5, `0007`×2, `0008`×4). Packages `weeping-angel-catalog` and `weeping-angel-assurance-cli` do not exist.
- **Desired behavior:** `architecture/*.toml` plus `docs/debt/register.toml` become SSOT. `cargo xtask guard` (alias + xtask member) runs checks 01, 02, 03, 13 for real: parse `architecture.toml`, require the seven-concept ownership table mapped to live crates (`canonical-catalog`, `framework`, `assurance`, `evidence`, root `weeping-angel`), require `forbidden-patterns.toml`, and reject duplicate debt IDs or `status=resolved` without `regression_tests`/`repository_guard`. Checks 04–12 and 14–15 stub fail-closed or skip only with a registered debt finding. CI must run `cargo xtask guard`. Dual-suite `sdd_repository_integrity_{baseline,target}` then goes RED→GREEN and baseline is ignore-superseded.
- **ADR:** needed — accepted at [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md)

### Acceptance criteria (this slice)

1. `architecture.toml`, `invariants.toml`, and `forbidden-patterns.toml` exist and parse with the specified schema strings.
2. Ownership table lists `catalog`, `framework_compilation`, `readiness_projection`, `temporal_evidence_selection`, `assessment_lineage`, `evidence_persistence`, `assurance_cli` mapped to live crates/paths (not hypothetical catalog/assurance-cli packages).
3. `docs/debt/register.toml` requires `id`+`status`, unique finding IDs, and rejects resolved without `regression_tests` or `repository_guard`.
4. `docs/debt/README.md` and `baseline-2026-08.md` exist with re-measured live counts.
5. xtask workspace member and `.cargo` alias make `cargo xtask guard` run checks 01, 02, 03, 13.
6. Stub checks 04–12 and 14–15 never silently pass (fail closed or skip with a registered `DEBT-GUARD-NN` finding).
7. CI contains a mandatory `cargo xtask guard` step.
8. Dual-suite `sdd_repository_integrity_{baseline,target}` registered in root `Cargo.toml`; baseline skip-superseded after target GREEN.
9. `sdd_documentation_layout` stays GREEN with this spec in `CANONICAL_SPECS` after implement.
10. ADR 0009 Accepted at implement; remaining_backlog not implemented.

### Out of scope

- P0 framework expression preservation
- Fail-closed pack parsing
- Catalog SSOT migration
- Framework digest redesign
- Readiness SSOT
- Lineage rebuild
- Evidence latest vs current
- Statement of Applicability invariants
- Persistence invariants
- Package install tests
- Crate dependency graph policy
- Schema fixtures and ADR graph uniqueness rewrite
- Spec lifecycle states; deleting obsolete baseline suites; test-support crate
- Implementing guard checks 04–12 and 14–15 beyond stubs
- Switching CI clippy/test to `--workspace`
- Inventing `weeping-angel-catalog` or `weeping-angel-assurance-cli`

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Generic skip-with-debt hatch if stubs do not each cite a dedicated `DEBT-GUARD-NN` id | Stubs 04–12 and 14–15 skip only as `skip(DEBT-GUARD-NN)` with matching seed findings; `ri_t07b` / `ri_t13` encode that |
| Fail-closed stubs would keep CI red; exit-0 must apply only after implemented checks pass | Implemented 01/02/03/13 pass; stubs skip with registered debt; `cargo xtask guard` exit 0 |
| Minting another `0003-*` ADR would worsen duplicate-ID debt | ADR is `0009-repository-health-gate.md`; next unique number is 0010 |
| `temporal_evidence_selection` also lives in `weeping-angel-control-test`; ownership declared on assurance without moving code | Ownership table maps the concept to `weeping-angel-assurance`; no code move |
| xtask must stay `publish=false` so cargo-dist/packager does not ship it | `xtask/Cargo.toml` is workspace member with `publish = false` |
| Root `cargo test --all-targets` does not run xtask tests; CI must invoke `cargo xtask guard` as its own step | `.github/workflows/ci.yml` has a mandatory `cargo xtask guard` step |
| unwrap/ignore counts include tests and can inflate debt if the inclusion rule is omitted from the baseline snapshot | `docs/debt/baseline-2026-08.md` is a re-measured live snapshot with the inclusion rule documented |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) |
| Baseline | PASS on old | `cargo test --test sdd_repository_integrity_baseline` → **pass** (exit 0). Characterization of SHA `f560196c`: architecture manifests, `docs/debt`, xtask member/alias, and CI `xtask guard` are absent; `cargo xtask` / `cargo run -p xtask` fail; live packages are `weeping-angel-canonical-catalog` + root `src/main.rs`+`src/cli.rs` (not `weeping-angel-catalog` / `weeping-angel-assurance-cli`). Combined `--test sdd_repository_integrity_target` is expected RED until implement; this baseline run is GREEN. No product gate implemented. Excerpt: `running 12 tests` / `ri_b01_architecture_toml_is_absent` … `ri_b10_hypothetical_packages_do_not_exist` / `test result: ok. 12 passed; 0 failed; 0 ignored`. Suites: `tests/contracts/repository_integrity.baseline.rs`, `tests/contracts/repository_integrity.target.rs`. |
| Target pre | FAIL on old | `cargo test --test sdd_repository_integrity_baseline --test sdd_repository_integrity_target` → **fail** (exit 1, expected). Target suite rewritten to encode increment-1 acceptance criteria. No product code (`architecture.toml`, debt register, xtask crate, CI guard) was implemented. Baseline characterization remains GREEN on current absences. Dual-suite `[[test]]` rows already exist; `CANONICAL_SPECS` and ADR 0009 Accepted are still implement-phase. Excerpt: `test result: FAILED. 1 passed; 17 failed` / `ri_t01_architecture_toml_exists_and_parses` (`architecture/architecture.toml` must exist); `ri_t07_debt_register_has_unique_ids_and_status` (`docs/debt/register.toml` missing); `ri_t10_xtask_member_and_alias` (workspace.members has no xtask); `ri_t11_cargo_xtask_guard_runs_implemented_checks` (`error: no such command: xtask`); `ri_t14_ci_runs_xtask_guard`; baseline still GREEN (12 passed; 0 failed). Suites: `tests/contracts/repository_integrity.target.rs`, `tests/contracts/repository_integrity.baseline.rs`. |
| Implement | target PASS | `cargo test --test sdd_repository_integrity_baseline --test sdd_repository_integrity_target`: baseline **ok. 0 passed; 0 failed; 12 ignored** (`superseded by sdd_repository_integrity_target`); target **ok. 18 passed; 0 failed; 0 ignored**. `cargo test -p xtask`: lib 6 passed; `tests/debt_register.rs` 5 passed. `cargo xtask guard`: 01 architecture-manifest pass; 02 canonical-ownership pass; 03 forbidden-patterns pass; 13 debt-register pass; 04–12, 14, 15 `skip(DEBT-GUARD-*)`; exit 0. Notes: baseline cargo-ok is skip-superseded (`#[ignore]`), not additive characterization still holding. Un-ignored RI-B01–B10 would fail because `architecture/`, `docs/debt/`, xtask, and the cargo alias now exist. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Baseline **ok. 0 passed; 0 failed; 12 ignored**. Not additive: absence characterization is now false (`architecture/`, `docs/debt/`, xtask, CI guard exist). File kept so dual-suite registration stays GREEN. `baseline_retired=true`, `baseline_not_green=true`. |
| Supersede | target still PASS | After skip-supersede: `cargo test --test sdd_repository_integrity_baseline --test sdd_repository_integrity_target -- --nocapture` → baseline 12 ignored; target **ok. 18 passed; 0 failed; 0 ignored** (0.31s) including `ri_t01`–`ri_t17` (`ri_t11` `cargo xtask guard` implemented checks, `ri_t13` stubs do not silently pass, `ri_t14` CI guard, `ri_t15b` baseline ignore-superseded, `ri_t16` spec in `CANONICAL_SPECS`, `ri_t17` ADR 0009 Accepted and backlog not shipped as product). `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md), [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md), [`docs/sdd/repository-integrity.md`](repository-integrity.md), [`docs/README.md`](../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`docs/debt/README.md`](../debt/README.md), [`README.md`](../../README.md), [`tests/contracts/documentation_layout.rs`](../../tests/contracts/documentation_layout.rs) |

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

Increment 1 repository health gate is present and verified:

- `architecture/architecture.toml` (`weeping-angel/architecture/v1`) with the seven-concept ownership table mapped to live crates (`weeping-angel-canonical-catalog`, `weeping-angel-framework`, `weeping-angel-assurance`, `weeping-angel-evidence`, root `weeping-angel`).
- `architecture/invariants.toml` (`weeping-angel/architecture-invariants/v1`) declared; evaluation is stub check 04.
- `architecture/forbidden-patterns.toml` (`weeping-angel/forbidden-patterns/v1`) seeds hypothetical packages and `tests/sdd/`.
- `docs/debt/register.toml` (`weeping-angel/debt-register/v1`): unique IDs; `status=resolved` rejected without `regression_tests` or `repository_guard`; seed `DEBT-GUARD-NN` rows for stubs.
- `docs/debt/README.md` and `docs/debt/baseline-2026-08.md` with re-measured live counts.
- Workspace member `xtask/` (`publish = false`) + `.cargo/config.toml` alias `xtask = "run --package xtask --"`.
- `cargo xtask guard` implements 01/02/03/13; stubs 04–12 and 14–15 never silently pass.
- CI `.github/workflows/ci.yml` mandatory `cargo xtask guard` step.
- Dual-suite registered in root `Cargo.toml`; target GREEN (18/18); baseline 12 ignored (`superseded by sdd_repository_integrity_target`).
- ADR 0009 Accepted. remaining_backlog not implemented as product.

### Files changed (implement)

`.cargo/config.toml`, `.github/workflows/ci.yml`, `Cargo.lock`, `Cargo.toml`, `README.md`, `architecture/architecture.toml`, `architecture/forbidden-patterns.toml`, `architecture/invariants.toml`, `docs/README.md`, `docs/adr/0009-repository-health-gate.md`, `docs/contracts/README.md`, `docs/debt/README.md`, `docs/debt/baseline-2026-08.md`, `docs/debt/register.toml`, `docs/sdd/repository-integrity.md`, `docs/specs/repository-integrity.md`, `tests/contracts/documentation_layout.rs`, `tests/contracts/repository_integrity.baseline.rs`, `tests/contracts/repository_integrity.target.rs`, `xtask/Cargo.toml`, `xtask/src/lib.rs`, `xtask/src/main.rs`, `xtask/tests/debt_register.rs`.

### Docs/ADR (DocsAdr phase)

ADR 0009 Accepted to match the shipped health gate: architecture `*.toml` ownership SSOT, debt register proof law, `cargo xtask guard` (01/02/03/13 + `skip(DEBT-GUARD-NN)` stubs), CI-mandatory. Spec, README, docs map, and contracts pointer updated; remaining_backlog not implemented.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-f0d2a357dd63` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 4 568 838 |
| `duration_ms_sum` | 2 448 452 (~40.8 min) |
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
| Scope | `sdd-scope` | ok | 342 803 | 232 906 |
| Spec | `sdd-spec` | ok | 477 246 | 815 153 |
| BaselineGreen | `sdd-baseline-green` | ok | 426 682 | 766 225 |
| TargetRed | `sdd-target-red` | ok | 279 690 | 702 059 |
| Implement | `sdd-implement` | ok | 476 738 | 1 070 272 |
| DocsAdr | `sdd-docs-adr` | ok | 298 849 | 751 719 |
| Iterate | `sdd-baseline-post-check` | ok | 86 024 | 117 369 |
| Supersede | `sdd-supersede` | ok | 60 420 | 113 135 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement).

---

## remaining_backlog (not implemented)

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
11. Crate dependency graph policy (check 15) and switching CI clippy/test to `--workspace`
12. Schema fixtures (JSON Schema for IR/catalog/packs)
13. ADR graph validator / unique ADR IDs (check 14) — do not renumber existing `0003-*` files
14. Spec lifecycle states; deleting obsolete baseline suites; test-support crate
15. Remaining guard checks 04–12 and 14–15 beyond stubs (including evaluating `invariants.toml`)
16. Inventing packages `weeping-angel-catalog` or `weeping-angel-assurance-cli` (still forbidden)

---

## Related

- Spec SSOT: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md)
- Decision: [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md)
- Pointer stub: [`docs/sdd/repository-integrity.md`](repository-integrity.md)
- Debt register: [`docs/debt/register.toml`](../debt/register.toml)
- Telemetry: [`.sdd/runs/repository-integrity-health-gate-telemetry.json`](../../.sdd/runs/repository-integrity-health-gate-telemetry.json)
