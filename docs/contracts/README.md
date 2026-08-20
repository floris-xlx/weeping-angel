# Contract tests

Human assurance specs live in [`docs/specs/`](../specs/). Executable invariants live in [`tests/contracts/`](../../tests/contracts/).

Discovery is the root [`Cargo.toml`](../../Cargo.toml) `[[test]]` table (and `documentation_layout.rs`). Do not maintain a dual-suite inventory here.

```text
rg "^name = \"sdd_" Cargo.toml
```

Architecture / repository-law dual-suites under `xtask/tests/` (including `sdd_structural_reconciliation_{baseline,target}`) are auto-discovered via `cargo test -p xtask` — not listed in this folder. Spec: [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md); decision: [ADR 0048](../adr/0048-structural-reconciliation.md).

Layout: [ADR 0004](../adr/0004-documentation-architecture.md). Hygiene: [ADR 0012](../adr/0012-repository-hygiene.md), spec [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md).
