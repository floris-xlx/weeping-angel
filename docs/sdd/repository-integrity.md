# Moved

Canonical specification: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md)

Accepted decision: [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md)

Successor increment (Guard 04 / `RepositoryModel`): [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md), [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md).

Executable invariants: `sdd_repository_integrity_{baseline,target}` in `tests/contracts/repository_integrity.{baseline,target}.rs`. Authoritative health command: `cargo xtask guard`.

Generated traces belong under `.sdd/`, not here ([ADR 0004](../adr/0004-documentation-architecture.md)). This path is not a second specification.
