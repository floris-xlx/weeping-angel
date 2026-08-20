# Executable invariants

Dual-suite contract tests for the assurance runtime. Target suites are the current law. Baseline suites are superseded characterizations (`#[ignore]`).

These files are **not** auto-discovered. Each suite must be listed as `[[test]]` in the root [`Cargo.toml`](../../Cargo.toml). Discover names with `rg "^name = \"sdd_" Cargo.toml` — do not keep a parallel inventory.

Human specs: [`docs/specs/`](../../docs/specs/). Decisions: [`docs/adr/`](../../docs/adr/). Hygiene: [`docs/specs/repository-hygiene.md`](../../docs/specs/repository-hygiene.md), [ADR 0012](../../docs/adr/0012-repository-hygiene.md). Shared needle helper for the 17 contract `*.target.rs` binaries: [`tests/support/mod.rs`](../support/mod.rs) via `include!` (C01 / DUP-002). Do not add `tests/support.rs`.
