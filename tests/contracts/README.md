# Executable invariants

Dual-suite contract tests for the assurance runtime. Target suites are the current law. Baseline suites are superseded characterizations (`#[ignore]`).

These files are **not** Cargo auto-discovery. They run as modules of the one `[[test]]` harness on package `weeping-angel` (`apps/cli/tests/harness.rs`; [ADR 0004](../../docs/adr/0004-documentation-architecture.md) / [ADR 0051](../../docs/adr/0051-repository-environment.md)). Discover names with:

```text
rg "tests/contracts/" apps/cli/tests/harness.rs
```

Do not keep a parallel inventory, and do not list suites in root `Cargo.toml`.

Human specs: [`docs/specs/`](../../docs/specs/). Decisions: [`docs/adr/`](../../docs/adr/). Hygiene: [`docs/specs/repository-hygiene.md`](../../docs/specs/repository-hygiene.md), [ADR 0012](../../docs/adr/0012-repository-hygiene.md). Shared needle helper for contract `*.target.rs` files: [`tests/support/mod.rs`](../support/mod.rs) via `include!` (C01 / DUP-002). Do not add `tests/support.rs`.
