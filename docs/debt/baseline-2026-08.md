# Live repository counts — 2026-08 implement snapshot

Re-measured on the increment-1 implement tree (Windows). Inclusion rule: all matching files under the repo **excluding** `target/`, `target-*`, and `node_modules/`.

Spec-first characterization (SHA `f560196c57e77df2573cfb9a4b384d3cf1c21e8a`) is preserved in `docs/specs/repository-integrity.md` §3.6. This file is evidence, not the debt register.

## Test binaries and suites

| Metric | Count | Notes |
| --- | --- | --- |
| Root `[[test]]` binaries | **80** | Dual-suites + `e2e_demo` + `e2e_recon`, including `sdd_repository_integrity_{baseline,target}` (characterization was 78 before this pair). |
| `tests/*.rs` (root of `tests/`, auto-discovered) | **16** | Includes `e2e_demo.rs`, `e2e_recon.rs`, `contract_spine.rs`, … |
| `tests/contracts/*.rs` | **78** | 38 `*.baseline.rs`, 39 `*.target.rs`, 1 `documentation_layout.rs` (characterization: 76). |
| ignored tests (`#[ignore` attributes in `*.rs`) | **661** | Mostly superseded baseline suites (characterization: 659). |

## unwrap / expect

| Metric | Count | Notes |
| --- | --- | --- |
| `.unwrap()` in `*.rs` | **1710** | Includes tests (characterization: 1726). |
| `.expect(` in `*.rs` | **796** | Includes tests (characterization: 776). |
| unwrap + expect | **2506** | Combined (characterization: 2502). |

## Source-grep contract tests

| Metric | Count | Notes |
| --- | --- | --- |
| Files defining `fn require_needles` | **16** | Dual-suite needle greps, not a dedicated source-grep crate. |
| `require_needles(` occurrences | **203** | Same as characterization. |

## ADR IDs

| Metric | Count / status | Notes |
| --- | --- | --- |
| ADR markdown files | **41** | `docs/adr/*.md` (characterization: 40; + ADR 0009). |
| ADR ID prefixes | `0001`(1), `0002`(1), `0003`(25), `0004`(1), `0005`(5), `0006`(1), `0007`(2), `0008`(4), `0009`(1) | Unique new number: **0009**. |
| Duplicate ADR ID prefixes | **4** prefixes (`0003`,`0005`,`0007`,`0008`); **36** files under those prefixes | Recorded as `DEBT-DUP-ADR`. Not renumbered this slice. |

## Catalog, frameworks, schemas

| Metric | Count | Notes |
| --- | --- | --- |
| Catalog test TOML | **13** | `catalog/canonical/v1/tests/*.toml` |
| Framework packs | **2** | `frameworks/iso-27001/2022`, `frameworks/wa-baseline/1` (`manifest.toml`) |
| `*.schema.json` files | **6** | 3 under `schemas/codex-security/` duplicated in `codex-security/schemas/` (`DEBT-SCHEMA-DUP`) |

## Workspace cargo status (if runnable)

| Command | Status | Notes |
| --- | --- | --- |
| `cargo fmt --all -- --check` | **pass** (exit 0) | Same command CI runs; re-run after product files landed. |
| `cargo check --workspace --offline` | **pass** (exit 0) | Runnable at implement. |
| `cargo test --features demo --all-targets` | **CI command; not re-run as a full workspace job at implement** | CI is **not** `--workspace`. Dual-suite + `cargo test -p xtask` are the increment-1 verify commands. |
| `cargo clippy --all-targets --features demo -- -D warnings` | **CI command; not re-run as a full workspace job at implement** | CI is **not** `--workspace`. |
