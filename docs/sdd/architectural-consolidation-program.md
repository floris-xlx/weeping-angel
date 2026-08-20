# Moved

Human SSOT: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)

Accepted decision: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)

Executable invariants: `xtask/tests/sdd_architectural_consolidation_target.rs` (`cargo test -p xtask`). The Phase 0 baseline suite is **deleted** (`INV-NO-SUPERSEDED-BASELINES`). Do not create `tests/sdd/`.

Generated SDD traces belong in `.sdd/runs/` and `.sdd/artifacts/` ([ADR 0004](../adr/0004-documentation-architecture.md)). This path is not a second specification.

Collision fence: architectural-cleanup Phase 0 is a different program (spec-law-only freeze). This pointer is for Architectural Consolidation Program **Phase 0** (machine-readable freeze + baseline + backlog schema).
