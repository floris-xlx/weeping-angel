# Contract tests

Human assurance specs live in [`docs/specs/`](../specs/). Executable invariants live in [`tests/contracts/`](../../tests/contracts/).

Discovery is the root [`Cargo.toml`](../../Cargo.toml) `[[test]]` table (and `documentation_layout.rs`). Do not maintain a dual-suite inventory here.

```text
rg "^name = \"sdd_" Cargo.toml
```

Layout: [ADR 0004](../adr/0004-documentation-architecture.md). Hygiene: [ADR 0012](../adr/0012-repository-hygiene.md), spec [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md).
