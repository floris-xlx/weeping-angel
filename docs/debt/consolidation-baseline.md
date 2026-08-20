# Frozen Phase 0 consolidation baseline

This file is the **frozen Phase 0** consolidation snapshot, **not** the live [`current.md`](current.md). `docs/debt/current.md` remains the live mechanical inventory projection (`weeping-angel/inventory/v1`). Counts come from the same `xtask/src/inventory.rs` walker (`walk_included`). Public-symbol / type-name metrics are a **line-based heuristic** (not a uniqueness ban).

- schema: `weeping-angel/consolidation-baseline/v1`
- program: `architectural-consolidation`
- phase: `0`
- source: `weeping-angel/inventory/v1`
- exclusions: `target/`, `target-*`, `node_modules/`

## Inventory counts

| Metric | Count |
| --- | --- |
| root_test_binaries | 45 |
| tests_rs_autodiscovered | 16 |
| tests_contracts_rs | 43 |
| ignored_test_attrs | 5 |
| unwrap_calls | 1239 |
| expect_calls | 635 |
| unwrap_plus_expect | 1874 |
| require_needles_fns | 18 |
| require_needles_calls | 222 |
| adr_markdown_files | 49 |
| catalog_test_toml | 13 |
| framework_packs | 2 |
| schema_json_files | 3 |

## Extended

| Metric | Count |
| --- | --- |
| workspace_crates | 9 |
| rust_modules | 290 |
| public_symbols | 2022 |
| pub_use_count | 110 |
| public_structs | 523 |
| public_enums | 221 |
| duplicate_helper_definitions | 18 |
| duplicate_type_names | 23 |
| adr_count | 49 |
| spec_count | 46 |
| debt_rows | 24 |

## Schema locations

- `schemas/codex-security/coverage.schema.json`
- `schemas/codex-security/findings.schema.json`
- `schemas/codex-security/scan-manifest.schema.json`

## Architecture ownership

| Concept | Crate | Kind | Paths |
| --- | --- | --- | --- |
| assessment_lineage | weeping-angel-assurance | exclusive | crates/weeping-angel-assurance/src/lineage.rs |
| assurance_cli | weeping-angel | facade | src/main.rs, src/cli.rs |
| catalog | weeping-angel-canonical-catalog | exclusive | crates/weeping-angel-canonical-catalog |
| evidence_persistence | weeping-angel-evidence | exclusive | crates/weeping-angel-evidence |
| framework_compilation | weeping-angel-framework | exclusive | crates/weeping-angel-framework |
| readiness_projection | weeping-angel-assurance | projection | crates/weeping-angel-assurance/src/readiness.rs |
| repository_guard | xtask | exclusive | xtask |
| temporal_evidence_selection | weeping-angel-assurance | exclusive | crates/weeping-angel-assurance/src/temporal.rs |

## Stable marker

<!-- weeping-angel-consolidation-baseline-stable -->
