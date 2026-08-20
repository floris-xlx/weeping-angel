# SDD run: Architectural Consolidation Phase 1 — canonical domain ownership model

| Field | Value |
| --- | --- |
| Run id | `wa-consolidation-phase-1` |
| Date | 2026-08-20 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `1197b85e2b0ba08a` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | **Phase 1 only** (concept-level five-role SSOT in `architecture/domain-ownership.toml`, fail-closed parse folded into Guard 01/04). **Not** Phase 2+ consumer migrations, duplicate deletes, or product-semantic rewrites. |
| Spec (human SSOT) | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) |
| ADR | Accepted [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md) |
| Telemetry | [`sdd-architectural-consolidation-phase-1-telemetry.json`](sdd-architectural-consolidation-phase-1-telemetry.json) |
| Dual-suite | `xtask/tests/*.rs` via `cargo test -p xtask` — **not** `tests/sdd/`, `test/sdd/*.ts`, or a new `[[test]]` |
| Baseline | `xtask/tests/sdd_architectural_consolidation_baseline.rs` — **deleted** after target GREEN (`supersede_kind=delete`; `INV-NO-SUPERSEDED-BASELINES` honesty-amended) |
| Target | [`xtask/tests/sdd_architectural_consolidation_target.rs`](../../xtask/tests/sdd_architectural_consolidation_target.rs) (CON-T01–T20; 20/20 pass) |
| Predecessor | Phase 0 [`sdd-architectural-consolidation-phase-0.md`](sdd-architectural-consolidation-phase-0.md), [ADR 0049](../adr/0049-architectural-consolidation-phase-0.md), [ADR 0010](../adr/0010-architecture-as-law.md) |
| Collision fence | Architectural-cleanup Phase 0 is a different program. Phase 0 freeze stays active. No Guard 16. No `cargo xtask consolidation`. No hypothetical `weeping-angel-catalog` / `weeping-angel-assurance-cli`. No `0003-*` / colliding `0011-*` ADR files. |

Durable finalize artifact for telemetry run `wa-consolidation-phase-1`. Product law lives in the linked spec; this file records protocol evidence, gates, and telemetry. It is not a second SSOT ([ADR 0004](../adr/0004-documentation-architecture.md)). Generated traces belong under `.sdd/`, not here. The SDD pointer remains [`architectural-consolidation-program.md`](architectural-consolidation-program.md).

---

## Spec

- **Title:** Architectural Consolidation Phase 1 — canonical domain ownership model
- **Problem:** Phase 0 froze expansion, but ownership is still crate-level kinds. There is no concept-level SSOT, so later phases cannot name who owns a concept versus who stores, projects, evaluates, or adapts it. `architecture.toml` claims exclusive `temporal_evidence_selection` on `weeping-angel-assurance` while `select_latest_as_of` still lives in `weeping-angel-control-test`. A paper `domain-ownership` file would not be a gate because `load_architecture_manifest` only parsed `architecture.toml`.
- **Current behavior (pre-implement):** `architecture/domain-ownership.toml` did not exist. `architecture.toml` `[ownership.*]` mapped catalog, framework compilation, readiness projection, temporal evidence selection, assessment lineage, evidence persistence, assurance CLI, and repository guard to crate + `kind` `exclusive|facade|projection|adapter|shared-primitive` + paths. `load_architecture_manifest` parsed schema, `[policy]`, `[ownership]`, and `[program.architectural_consolidation]` from `architecture.toml` only. Guard IDs remained 01–15; no `INV-DOMAIN-OWNERSHIP*`. `temporal_evidence_selection` was `kind=exclusive` on `weeping-angel-assurance/src/temporal.rs` while `pub fn select_latest_as_of` lived in `control-test/src/temporal.rs`. `INV-NO-SUPERSEDED-BASELINES` fail-closed any indexed path ending in `.baseline.rs` or `_baseline.rs`; `RepositoryModel.filesystem` did not index `xtask/`; CON-T07 asserted the Phase 0 baseline file was deleted. Phase 0 CON-T01–T10 were GREEN. Workspace members did not include `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
- **Desired behavior (this slice):** `architecture/domain-ownership.toml` (schema `weeping-angel/domain-ownership/v1`) is the concept-level SSOT expanding crate ownership. Every concept table has `semantic_owner`, `storage_owner`, `projection_owner`, `evaluation_primitive_owner`, and `adapter_owner` (`persistence_owner` maps to `storage_owner`; no sixth role). xtask parses the file fail-closed if missing/malformed; a paper file without parse is not a pass. Fold into Guard 01 and/or Guard 04 `INV-DOMAIN-OWNERSHIP-PRESENT` / `INV-DOMAIN-OWNERSHIP-ROLES`; no Guard 16. Seed live evidence only. `temporal_evaluation` and `control_status` are `split=divided`. `INV-NO-SUPERSEDED-BASELINES` means superseded leftovers: a live non-ignored `xtask/tests/sdd_*_baseline.rs` is allowed during the dual-suite window then deleted after GREEN. Phase 1 is ownership law, not consumer migration. Phase 0 freeze stays active.
- **ADR:** needed — accepted at [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md) only after target GREEN.

### Acceptance criteria (this slice)

1. Dual-suite only at `xtask/tests/sdd_architectural_consolidation_{baseline,target}.rs`; no `tests/sdd/`, `test/sdd/*.ts`, or new `[[test]]`; baseline deleted after GREEN (not `#[ignore]`).
2. CON-B11–B16 PASS on CURRENT: missing `domain-ownership.toml`, no parser, crate-level kinds without five roles, temporal exclusive vs `select_latest_as_of`, no `INV-DOMAIN-OWNERSHIP*`, no hypothetical crates.
3. CON-T11–T20 FAIL on CURRENT because file/parser/roles/seeds/split/invariants are absent, then PASS after implement.
4. Phase 0 CON-T01–T06/T08–T09 stay GREEN; CON-T07 allows the live non-ignored baseline during the window and reasserts deletion after GREEN.
5. `domain-ownership.toml` schema `weeping-angel/domain-ownership/v1` with required five role keys; `persistence_owner` is not a sixth role.
6. Seeded concepts cite live symbols in live workspace crates only; no `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
7. `temporal_evaluation` and `control_status` are `split=divided`; domain-ownership must not copy `architecture.toml` `kind=exclusive` as fake exclusivity.
8. Missing/malformed/unparsed paper file fails Guard 01 and/or 04; no Guard 16; no `cargo xtask consolidation`.
9. `INV-NO-SUPERSEDED-BASELINES` honesty-amended: allow live `xtask/tests/sdd_*_baseline.rs` during the window; fail `#[ignore]`, `tests/sdd` leftovers, leftover after GREEN.
10. Neighbors stay green: `CANONICAL_SPECS`, Guard 15 existing consolidation spec row, ACP target, SR target, `cargo xtask guard` 01–15.
11. ADR 0050 Accepted only after target GREEN; no `0003-*` / colliding `0011-*`.
12. No consumer migration, duplicate deletes, or applicability/readiness/lineage product semantic rewrites; freeze expansion metrics must not rise.

### Out of scope

- Phase 2+ consumer migrations and deleting DUP duplicate source trees
- Rewriting applicability, readiness, lineage, catalog, collector, or IR product semantics
- New frameworks, collectors, ISMS engines, report formats, or product scanners
- Hypothetical packages `weeping-angel-catalog` and `weeping-angel-assurance-cli`
- `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]`
- Guard 16, `cargo xtask consolidation`, or a second health CLI
- A second program spec or second Guard 15 consolidation row
- Mass ADR renumber or `#[ignore]`-superseding the baseline
- pnpm / `apps/docs`
- Moving `select_latest_as_of` into assurance (Phase 1 only records the split)
- Collapsing five roles into crate `kind=exclusive`
- Flipping `feature_expansion` to unrestricted

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Paper `domain-ownership.toml` without a parser looks like law but is not a gate | xtask parses the sibling file fail-closed into `ArchitectureManifest`; missing/malformed fails Guard 01/04 (CON-T11/T12) |
| Copying `kind=exclusive` onto `temporal_evaluation` or `control_status` hides the live split | Both concepts are `split=divided`; CON-T15 asserts honest divided ownership vs `select_latest_as_of` in control-test |
| Recreating the baseline without amending CON-T07 REDS Phase 0 CON-T07 | CON-T07 honesty-amended for the dual-suite window; after GREEN CON-T07/T17 reassert deletion |
| Indexing `xtask/` in `RepositoryModel` without the honesty-amended invariant REDS Guard 04 during the window | `INV-NO-SUPERSEDED-BASELINES` allows live non-ignored `xtask/tests/sdd_*_baseline.rs` during the window only |
| `#[ignore]` after GREEN violates `INV-NO-SUPERSEDED-BASELINES` | Baseline deleted (`supersede_kind=delete`); CON-T07/T17 fail closed if it returns |
| `pub struct` parser types in xtask can raise frozen `public_structs` under the Phase 0 freeze | Parse types stay crate-private; freeze expansion metrics must not rise |
| Inventing hypothetical crate owner seats trips `FORBID-HYPOTHETICAL-*` | Seeds cite live workspace crates only (CON-T14 / CON-B16) |
| A second `docs/specs` consolidation file forks the program SSOT | Single program spec; CON-T18; Guard 15 existing consolidation row only |
| Dual-suite needles that rewrite product modules make target RED for the wrong reason | Target RED was CON-T11–T20 for absent file/parser/roles/seeds/split/invariants; Phase 0 CON-T01–T10 stayed GREEN |
| Adding Guard 16 or a second health CLI forks the health plane | Folded into Guard 01/04 only; no Guard 16; no `cargo xtask consolidation` |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) |
| Baseline | PASS on old | `cargo test -p xtask -- --nocapture` → **pass** (exit 0). Phase 1 characterization only (CON-B11–B16). `architecture/domain-ownership.toml` still absent; `load_architecture_manifest` still ignored sibling files. CON-T07 honesty-amended for the dual-suite window (live non-ignored CON-B11–B16). `current.md` ADR count resynced 49→50 for live inventory after draft ADR 0050. No Guard 16, no new `[[test]]`, no hypothetical crates, no product semantic changes. Excerpt: `running 6 tests` / `con_b11_domain_ownership_toml_is_not_a_file` … `con_b16_no_hypothetical_workspace_crates` / `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`. Phase 0 target still 10 passed; Guard 01–15 still 15 passed. Suites: `xtask/tests/sdd_architectural_consolidation_baseline.rs`, `xtask/tests/sdd_architectural_consolidation_target.rs`, `docs/debt/current.md`. |
| Target pre | FAIL on old | `cargo test -p xtask` → **fail** (exit 1, expected). Target-only dual-suite extension. CON-T11–T20 RED on CURRENT because `domain-ownership.toml`, parser, five roles, seeded concepts, divided temporal/`control_status` split, and `INV-DOMAIN-OWNERSHIP*` are absent. Phase 0 CON-T01–T10 remain GREEN; CON-T07 allows the live non-ignored baseline window file. No product implementation. Excerpt: `test result: FAILED. 10 passed; 10 failed`; failures: `con_t11`…`con_t20`; `con_t11: architecture/domain-ownership.toml must exist as the concept-level SSOT`; `con_t12: paper architecture/domain-ownership.toml without a parser is not a pass`; `con_t16: architecture/invariants.toml must declare INV-DOMAIN-OWNERSHIP-PRESENT`. Phase 0 CON-T01–T10: 10 passed; baseline CON-B11–B16: 6 passed. Suite: `xtask/tests/sdd_architectural_consolidation_target.rs`. |
| Implement | target PASS | `cargo test -p xtask --test sdd_architectural_consolidation_target` → **ok. 20 passed; 0 failed**. Phase 1 domain-ownership law: `architecture/domain-ownership.toml` (v1, five roles) parsed fail-closed into `ArchitectureManifest`; Guard 01/04 invariants `INV-DOMAIN-OWNERSHIP-PRESENT` / `INV-DOMAIN-OWNERSHIP-ROLES`; `temporal_evaluation` and `control_status` are `split=divided`; `INV-NO-SUPERSEDED-BASELINES` honesty-amended. `cargo test -p xtask --test sdd_architectural_consolidation_baseline`: compile fail (`pattern requires \`..\`` due to inaccessible `ArchitectureManifest` fields) then deleted. `cargo test -p xtask`: `debt_register` 5 passed; `sdd_architectural_cleanup_target` 17 passed; `sdd_architectural_consolidation_target` 20 passed; `sdd_structural_reconciliation_target` 15 passed. `cargo xtask guard` 01–15 all pass. ADR 0050 Accepted. |
| Baseline post | FAIL or retired | **Retired by delete** (`supersede_kind=delete`). After implement the leftover characterization file failed to compile (`error: pattern requires \`..\`` due to inaccessible fields (`ArchitectureManifest`); `error: could not compile xtask (test sdd_architectural_consolidation_baseline)`). Suite then deleted (`INV-NO-SUPERSEDED-BASELINES`). `Test-Path xtask/tests/sdd_architectural_consolidation_baseline.rs` → `False` (`BASELINE_ABSENT`). `cargo test -p xtask --test sdd_architectural_consolidation_baseline -- --nocapture` → `error: no test target named sdd_architectural_consolidation_baseline in xtask package`. Available test targets: `debt_register`, `sdd_architectural_cleanup_target`, `sdd_architectural_consolidation_target`, `sdd_consolidation_c01_baseline`, `sdd_structural_reconciliation_target`. `baseline_retired=true`, `baseline_not_green=true`, `additive_baseline=false`. `current.md` is the live inventory file, not retired. |
| Supersede | target still PASS | `cargo test -p xtask -- --nocapture` / `sdd_architectural_consolidation_target`: **ok. 20 passed; 0 failed; 0 ignored** (CON-T01–T20). CON-T07/T17 require the baseline file stay deleted. Also: xtask lib 9 passed; `debt_register` 5; cleanup 17; c01 baseline 5; SR 15 — all ok. `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md), [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md), [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md), [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md), [`docs/README.md`](../README.md), [`docs/contracts/README.md`](../contracts/README.md), [`docs/debt/README.md`](../debt/README.md), [`architecture/architecture.toml`](../../architecture/architecture.toml) |

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

Phase 1 canonical domain ownership law is present and verified:

- `architecture/domain-ownership.toml` schema `weeping-angel/domain-ownership/v1` is the concept-level SSOT expanding crate-level `architecture.toml` ownership (not replacing it).
- Every concept table has the five required roles: `semantic_owner`, `storage_owner`, `projection_owner`, `evaluation_primitive_owner`, `adapter_owner`. `persistence_owner` maps to `storage_owner`; no sixth role.
- xtask parses the sibling file fail-closed into `ArchitectureManifest`; a paper file without parse is not a pass. Folded into Guard 01/04 as `INV-DOMAIN-OWNERSHIP-PRESENT` / `INV-DOMAIN-OWNERSHIP-ROLES`. No Guard 16. No `cargo xtask consolidation`.
- Seeded concepts cite live symbols in live workspace crates only (ApplicabilitySnapshot + lineage LineageApplicabilitySnapshot, `project_readiness`, `CanonicalCatalog::load`, `compile_framework`, evidence ledger current/as_of/latest, `select_latest_as_of`, `replay_assessment`, `project_soa_from_snapshot`, ImplementationStatus vs Effectiveness vs SoA, evaluate in `run.inc`, `project_validity`, CollectorAdapter, CLI facade `src/main.rs`+`src/cli.rs`). No hypothetical crates.
- `temporal_evaluation` and `control_status` are `split=divided`; domain-ownership does not copy `architecture.toml` `kind=exclusive` as fake exclusivity. Phase 1 records the split; it does not move `select_latest_as_of` into assurance.
- `INV-NO-SUPERSEDED-BASELINES` honesty-amended: live non-ignored `xtask/tests/sdd_*_baseline.rs` allowed during the dual-suite window, then deleted after GREEN (not `#[ignore]`).
- Dual-suite under `xtask/tests` only. Target CON-T01–T20 GREEN. Characterization baseline deleted.
- Phase 0 freeze stays active. No consumer migration, duplicate deletes, or product-semantic rewrites. Freeze expansion metrics must not rise.
- Neighbors stay green: `CANONICAL_SPECS`, Guard 15 existing consolidation spec row, ACP target, SR target, `cargo xtask guard` 01–15.
- ADR 0050 Accepted after target GREEN.

### Files changed (implement)

`architecture/domain-ownership.toml`, `architecture/invariants.toml`, `xtask/src/architecture.rs`, `xtask/src/checks.rs`, `xtask/tests/debt_register.rs`, `xtask/tests/sdd_architectural_cleanup_target.rs`, `docs/adr/0050-domain-ownership-model.md`, `docs/specs/architectural-consolidation-program.md`, `docs/sdd/architectural-consolidation-program.md`, `docs/debt/current.md`, `xtask/tests/sdd_architectural_consolidation_baseline.rs` (later deleted).

### Docs/ADR (DocsAdr phase)

Finalized ADR 0050 Accepted to the implemented Phase 1 law: sibling `domain-ownership.toml` (v1, five roles, crate-private parse, Guard 01/04 `INV-DOMAIN-OWNERSHIP-PRESENT`/`ROLES`), honest divided `temporal_evaluation` and `control_status`, facade CLI, no hypothetical crates, `INV-NO-SUPERSEDED-BASELINES` window-then-delete. Updated the single program spec, SDD pointer, docs map, contracts README, debt README, and `architecture.toml` comments so crate-kind exclusive is not read as whole-concept ownership. Neighbor ADR 0049 notes the Phase 1 successor.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `wa-consolidation-phase-1` |
| `agents_ok` | 7 |
| `agents_fail` | 0 |
| `agents_total` | 7 |
| `tokens_used_sum` | 11 327 024 |
| `duration_ms_sum` | 2 917 571 (~48.6 min) |
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
| Scope | `sdd-scope` | ok | 297 602 | 542 155 |
| Spec | `sdd-spec` | ok | 579 109 | 1 593 269 |
| BaselineGreen | `sdd-baseline-green` | ok | 316 722 | 1 332 248 |
| TargetRed | `sdd-target-red` | ok | 503 296 | 1 710 337 |
| Implement | `sdd-implement` | ok | 798 890 | 4 934 647 |
| DocsAdr | `sdd-docs-adr` | ok | 274 221 | 777 156 |
| Supersede | `sdd-supersede` | ok | 147 731 | 437 212 |

Iterate used 0 of `max_iters` 3 (target already GREEN after implement; no iterate agent).

---

## remaining_backlog (not implemented)

1. Phase 2+ consumer migrations and deleting DUP duplicate source trees
2. Rewriting applicability, readiness, lineage, catalog, collector, or IR product semantics
3. New frameworks, collectors, ISMS engines, report formats, or product scanners
4. Hypothetical packages `weeping-angel-catalog` and `weeping-angel-assurance-cli` (still forbidden)
5. `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]` (still forbidden)
6. Guard 16, `cargo xtask consolidation`, or a second health CLI (still forbidden)
7. A second program spec or second Guard 15 consolidation row (still forbidden)
8. Mass ADR renumber or `#[ignore]`-superseding the baseline
9. pnpm / `apps/docs`
10. Moving `select_latest_as_of` into assurance (Phase 1 only recorded the split)
11. Collapsing five roles into crate `kind=exclusive`
12. Flipping `feature_expansion` to unrestricted (Phase 0 freeze stays active)

---

## Related

- Spec SSOT: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)
- SDD pointer: [`docs/sdd/architectural-consolidation-program.md`](architectural-consolidation-program.md)
- Decision: [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md)
- Predecessor: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md), [`sdd-architectural-consolidation-phase-0.md`](sdd-architectural-consolidation-phase-0.md)
- Domain ownership SSOT: [`architecture/domain-ownership.toml`](../../architecture/domain-ownership.toml)
- Live inventory: [`docs/debt/current.md`](../debt/current.md)
- Target suite: [`xtask/tests/sdd_architectural_consolidation_target.rs`](../../xtask/tests/sdd_architectural_consolidation_target.rs)
- Telemetry: [`sdd-architectural-consolidation-phase-1-telemetry.json`](sdd-architectural-consolidation-phase-1-telemetry.json)
