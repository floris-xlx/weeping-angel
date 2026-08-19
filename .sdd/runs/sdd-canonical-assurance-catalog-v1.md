# SDD run: Canonical Assurance Catalog v1 infrastructure

| Field | Value |
| --- | --- |
| Run id | `sdd-5334381c09f2` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `5334381c09f2f3db` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Catalog infrastructure only (dedicated crate, `catalog/canonical/v1` fixture, offline load/validate/digest, catalog CLI). No ISO remapping, no production regime content. |
| Spec | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| ADR | [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-canonical-assurance-catalog-v1-telemetry.json`](sdd-canonical-assurance-catalog-v1-telemetry.json) |
| Dual-suite | `tests/sdd/canonical_assurance_catalog.baseline.rs` (absence asserts skip-superseded) · `tests/sdd/canonical_assurance_catalog.target.rs` (active; CAT-001..016) |
| Base SHA (characterization) | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Canonical Assurance Catalog v1 infrastructure
- **Problem:** Downstream catalog, collector, framework, and test work has no versioned, framework-neutral, provider-neutral catalog contract. Canonical controls/tests currently live as thin stubs inside ISO/wa-baseline framework packs with pack-local IDs, so there is no offline `CanonicalCatalog` load/validate/digest, no public `control.*` / `evidence.*` / `test.*` API, and no assurance catalog CLI.
- **Current behavior (pre-catalog, SHA 5fa3a23):** no `catalog/` tree and no `CanonicalCatalog` API. Framework packs (`weeping-angel/framework-pack/v1`) own thin canonical stubs (e.g. `source.branch-protection`, `test.source.branch-protection`, synthesized `ev.<type>`). IR `ControlId` / `EvidenceRequirementId` / `ControlTestId` accept those IDs and also `control.github.*` / `control.iso27001.*`. `IdError::InvalidNamespace` is never returned. IR schema remains `assurance-ir/v1` and must not be forked. clap `AssuranceCommand` is Framework/Collect/Evidence/Assess/Result/Compare/Soa only; `main.rs` Assurance arm prints the not-certification banner and exits 0. Framework crate has no collector/SDK deps; collector has no framework crate. Workspace tests are green; rustfmt `--check` and clippy `-D warnings` are already red on pre-existing hygiene.
- **Desired behavior:** A dedicated `weeping-angel-canonical-catalog` crate (IR + toml/fs/digest only) loads `catalog/canonical/v1` offline with schema `weeping-angel/canonical-catalog/v1`, validates fail-closed, and emits a deterministic domain-separated digest. Catalog IDs must be `control.*` / `evidence.*` / `test.*` and must reject provider/framework segments, duplicates, dangling refs, orphaned tests, malformed selectors/expressions, unsupported schemas, and unlisted files. CLI `weeping-angel assurance catalog {validate,stats,inspect <control-id>}` parses in `src/cli.rs` with execution kept separate. Dual-suite `sdd_canonical_assurance_catalog_baseline` (GREEN on current) / `_target` (RED then GREEN). ISO pack IDs are not remapped; framework stays catalog-free; collector stays framework- and catalog-blind.
- **ADR:** needed — accepted at [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_canonical_assurance_catalog_baseline` / `_target` registered; baseline GREEN on current product behavior; target RED on current tree for missing catalog surfaces; after implement target GREEN and baseline fail-or-skip-superseded.
2. `catalog/canonical/v1/{manifest.toml,controls/,evidence/,tests/}` exists with schema `weeping-angel/canonical-catalog/v1`.
3. `CanonicalCatalog::{load,validate,digest}` exist on `weeping-angel-canonical-catalog`; load/validate perform zero network I/O.
4. Catalog digest is deterministic across reload and insert-order and is domain-separated from `assurance-ir/v1`.
5. Validator rejects duplicate IDs, unknown refs, orphaned tests, malformed selectors/expressions, unsupported schema, provider/framework ID segments, extra/unlisted section files, and path escape.
6. Shipped catalog content is only a minimal provider- and framework-neutral fixture.
7. CLI `weeping-angel assurance catalog validate|stats|inspect <control-id>` parses in `src/cli.rs`; execution is not inlined in the clap enum; inspect shows the named control and linked evidence/tests.
8. `weeping-angel-framework` remains collector/SDK-free and must not depend on the catalog crate; collector remains framework-blind and catalog-blind.
9. `assurance-ir/v1` is not forked; `Control` / `Requirement` / `Mapping` / `EvidenceRequirement` / `PlannedControlTest` / `AssessmentDefinition` are not redesigned; ISO/wa-baseline pack IDs are not remapped.
10. `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_compliance_ir_target`, and `cargo test --workspace --features demo` stay GREEN.
11. Downstream can add control/evidence/test TOML plus a manifest file entry without editing loader source.
12. No SOC 2 / NIS2 / DORA / ISO normative content in `catalog/canonical/v1`.

### Out of scope

- Full IAM / SDLC / vulnerability / infrastructure / governance catalogs
- Typed evidence value model and population / `CoverageAtLeast` runtime
- GitHub collector, applicability engine, assessment lineage, ISO remapping
- Redesign of `AssessmentDefinition` / `Control` / `Requirement` / `Mapping` / `EvidenceRequirement` / `PlannedControlTest`
- Enforcing `control.*` on IR `ControlId` constructors
- SOC 2 / NIS2 / DORA / GDPR / ISO 27701 production content
- ISO clause / Annex text in the canonical catalog
- Teaching `compile_framework` to load `catalog/canonical/v1`
- Implementing non-catalog assurance subcommand execution
- Fixing pre-existing workspace rustfmt / clippy failures
- Certification claims or licensed ISO narrative

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Tightening IR `ControlId` would break ISO packs | Catalog-only ID law; IR constructors stay permissive. Two ID layers remain until ISO remap. |
| Loader in `weeping-angel-framework` would couple packs to catalog | Dedicated `weeping-angel-canonical-catalog` crate; framework stays catalog-free. |
| Catalog crate depending on collector or control-test | Catalog crate is IR + toml/fs/digest only. |
| Hashing raw TOML bytes would make digests differ on Windows CRLF | Digest is over canonicalized structure, domain-separated from `assurance-ir/v1`. |
| `main.rs` Assurance stub could swallow catalog CLI | Dispatch tested; execution not inlined in the clap enum. |
| Unlisted TOML omitted from digest if extra-file validation is skipped | Validator rejects extra/unlisted section files and path escape. |
| Incomplete reserved-segment denylist | Provider/framework ID segments rejected (`github`, `iso27001`, etc.). |
| Leaving baseline “no catalog” asserts required-green in CI after ship | Absence asserts skip-superseded (`#[ignore]`); forced `--ignored` fails. |
| Fixture IDs colliding with later IAM/source catalogs | Shipped content is a minimal `fixture.example` only. |
| Pre-existing fmt/clippy RED mistaken for catalog-infra failure | Pre-existing hygiene stays out of scope; dual-suite + workspace tests are the gate. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Baseline | PASS on old | `cargo test --workspace --features demo --test sdd_canonical_assurance_catalog_baseline --test sdd_canonical_assurance_catalog_target` → exit 0. Characterization at SHA `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`. Baseline: **14 passed**. Target binary is a registration-only harness (**1 passed**) so the assigned dual `--test` command exits 0; CAT-001..016 are not asserted yet. Excerpt: `test result: ok. 14 passed; 0 failed` / `test result: ok. 1 passed`. |
| Target pre | FAIL on old | `cargo test --workspace --features demo --test sdd_canonical_assurance_catalog_baseline --test sdd_canonical_assurance_catalog_target -- --test-threads=1` → exit 1. Target: **FAILED. 4 passed; 18 failed**. Fail-closed tests abort before vacuous CLI-missing successes. CAT-012/013/016 and IR/ISO non-remap stay green as already-true compatibility locks. Baseline unmodified and GREEN (**14 passed**). Excerpts: `cat_001: catalog/canonical/v1/manifest.toml must exist`; `cat_002: crates/weeping-angel-canonical-catalog must exist`; `cat_011: AssuranceCommand must grow catalog`; have `["framework", "collect", "evidence", "assess", "result", "compare", "soa"]`. |
| Implement | target PASS | Same dual command after crate/fixture/CLI. Target: **22 passed; 0 failed** (CAT-001..016 plus extra_unlisted_section_files, path_escape, malformed_catalog_ids, shipped_catalog_is_minimal_and_regime_free, loader_reads_manifest_file_list, ir_schema_and_iso_pack_ids_are_not_remapped). Baseline default: **8 passed; 6 ignored** (superseded by `sdd_canonical_assurance_catalog_target`). Passed remain dual-suite registration, IR/ISO/crate-graph compatibility. |
| Baseline post | FAIL or retired | Skip-supersede (`supersede_kind=skip`). Default: **8 passed; 0 failed; 6 ignored**. Forced `--ignored`: **FAILED. 0 passed; 6 failed; 8 filtered out**. Examples: `catalog_canonical_v1_tree_does_not_exist` found `catalog/`; `no_canonical_catalog_crate_or_workspace_member`; `assurance_command_lists_only_current_family` have `[..., "catalog"]` vs Soa-only; `assurance_catalog_cli_does_not_parse` clap now accepts `assurance catalog validate`. Not additive. Remaining 8 IR/ISO/crate-graph tests stay registered and pass. |
| Supersede | target still PASS | After skip-supersede: target **22/22** still GREEN. Baseline absence characterization is not the CI gate. Dual-suite registration and IR/ISO/crate-graph checks kept. |
| Docs/ADR | updated | [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`frameworks/README.md`](../../frameworks/README.md), [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/xylex/weeping-angel-assurance-ir/supersession/banner-list.md`](xylex/weeping-angel-assurance-ir/supersession/banner-list.md) |

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

Dedicated catalog infrastructure, not a production control library:

- `weeping-angel-canonical-catalog` (IR + toml/fs/digest only) with `CanonicalCatalog::{load,validate,digest}`.
- On-disk tree `catalog/canonical/v1/{manifest.toml,controls/,evidence/,tests/}` schema `weeping-angel/canonical-catalog/v1`.
- Fail-closed offline validator: duplicate IDs, unknown refs, orphaned tests, malformed selectors/expressions, unsupported schema, provider/framework ID segments, extra/unlisted section files, path escape.
- Deterministic digest, insert-order independent, domain-separated from `assurance-ir/v1`.
- Shipped content is a minimal provider- and framework-neutral fixture only.
- CLI `weeping-angel assurance catalog {validate,stats,inspect <control-id>}` parsed in `src/cli.rs`; execution in `src/assurance_catalog.rs` (not inlined in the clap enum).
- Framework remains collector/SDK-free and catalog-free; collector remains framework-blind and catalog-blind.
- IR schema and ISO/wa-baseline pack IDs are not remapped. Two ID layers remain until a later ISO remap slice.

### Files changed (implement)

`Cargo.toml`, `Cargo.lock`, `src/cli.rs`, `src/lib.rs`, `src/main.rs`, `src/assurance_catalog.rs`, `crates/weeping-angel-canonical-catalog/Cargo.toml`, `crates/weeping-angel-canonical-catalog/src/lib.rs`, `catalog/canonical/v1/manifest.toml`, `catalog/canonical/v1/controls/fixture.example.toml`, `catalog/canonical/v1/evidence/fixture.example.toml`, `catalog/canonical/v1/tests/fixture.example.toml`, `docs/sdd/canonical-assurance-catalog-v1.md`, `docs/adr/0003-canonical-assurance-catalog-v1.md`, `docs/contracts/assurance-runtime.md`, `tests/sdd/canonical_assurance_catalog.baseline.rs`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-5334381c09f2` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 7 885 884 |
| `duration_ms_sum` | 7 132 492 (~118.9 min) |
| `budget.total` | 48 |
| `budget.spent` | 8 |
| `budget.remaining` | 40 |
| `event_count` | 29 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |

### Gates (final snapshot)

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agents

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 792 363 | 293 294 |
| Spec | `sdd-spec` | ok | 1 299 069 | 1 058 547 |
| BaselineGreen | `sdd-baseline-green` | ok | 952 351 | 997 989 |
| TargetRed | `sdd-target-red` | ok | 823 612 | 552 296 |
| Implement | `sdd-implement` | ok | 2 244 550 | 3 553 892 |
| DocsAdr | `sdd-docs-adr` | ok | 854 747 | 1 067 895 |
| Iterate | `sdd-baseline-post-check` | ok | 74 651 | 182 267 |
| Supersede | `sdd-supersede` | ok | 91 149 | 179 704 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-canonical-assurance-catalog-v1-telemetry.json`](sdd-canonical-assurance-catalog-v1-telemetry.json).

---

## Remaining backlog (not this slice)

1. Full IAM / SDLC / vulnerability / infrastructure / governance catalogs
2. Teaching `compile_framework` to load `catalog/canonical/v1`
3. Remapping ISO / wa-baseline pack IDs onto `control.*` / `evidence.*` / `test.*` (two ID layers remain)
4. Enforcing `control.*` on IR `ControlId` constructors
5. Typed evidence value model and population / `CoverageAtLeast` runtime
6. SOC 2 / NIS2 / DORA / GDPR / ISO 27701 production content
7. ISO clause / Annex text in the canonical catalog (forbidden this slice)
8. Implementing non-catalog assurance subcommand execution
9. Fixing pre-existing workspace rustfmt / clippy failures
10. Certification claims or licensed ISO narrative (forbidden)

---

## Summary

Canonical Assurance Catalog v1 infrastructure landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `5fa3a23` (14 passed / 1 target harness pass), target RED (18 failed) for missing catalog tree/crate/CLI, then target GREEN 22/22. Absence-of-catalog baseline asserts skip-superseded (`#[ignore]`; forced `--ignored` 6/6 FAIL). Remaining 8 IR/ISO/crate-graph baseline checks stay registered and pass. Framework stays catalog-free; collector stays framework- and catalog-blind; IR is not forked; ISO pack IDs are not remapped. Shipped catalog is a minimal provider- and framework-neutral fixture only.
