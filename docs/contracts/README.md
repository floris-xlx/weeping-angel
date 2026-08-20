# Contract tests

Human assurance specs live in [`docs/specs/`](../specs/). Executable invariants live in [`tests/contracts/`](../../tests/contracts/).

Discovery is the root [`Cargo.toml`](../../Cargo.toml) `[[test]]` table (and `documentation_layout.rs`). Do not maintain a dual-suite inventory here.

```text
rg "^name = \"sdd_" Cargo.toml
```

Architecture / repository-law dual-suites under `xtask/tests/` (including `sdd_structural_reconciliation_target` and `sdd_architectural_consolidation_target`) are auto-discovered via `cargo test -p xtask` — not listed in this folder. Specs: [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md), [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md); decisions: [ADR 0048](../adr/0048-structural-reconciliation.md), [ADR 0049](../adr/0049-architectural-consolidation-phase-0.md), [ADR 0050](../adr/0050-domain-ownership-model.md). Phase 1 characterization baseline is **deleted** (`INV-NO-SUPERSEDED-BASELINES`); do not recreate `tests/sdd/`.

Contract dual-suites that still source-grep product surface share one crate-private `fn require_needles` in [`tests/support/mod.rs`](../../tests/support/mod.rs) (`include!`; not `tests/support.rs` / `main.rs`). That is C01 / DUP-002, not a hygiene helper. Hygiene-owned suites must not call it. Uniqueness pin: `xtask/tests/sdd_consolidation_c01_target.rs`.

Layout: [ADR 0004](../adr/0004-documentation-architecture.md). Hygiene: [ADR 0012](../adr/0012-repository-hygiene.md), spec [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md).
