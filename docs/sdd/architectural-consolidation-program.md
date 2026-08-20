# Moved

Human SSOT: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) (Phase 0 implemented; **Phase 1 implemented** in §11).

Accepted decision: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)

Accepted decision: [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md)

Executable invariants: `xtask/tests/sdd_architectural_consolidation_target.rs` (`cargo test -p xtask`). Phase 0 and Phase 1 baseline suites are **deleted** (`INV-NO-SUPERSEDED-BASELINES` honesty-amended; do not `#[ignore]`). Do not create `tests/sdd/`.

Generated SDD traces belong in `.sdd/runs/` and `.sdd/artifacts/` ([ADR 0004](../adr/0004-documentation-architecture.md)). This path is not a second specification.

Collision fence: architectural-cleanup Phase 0 is a different program (spec-law-only freeze). This pointer is for Architectural Consolidation Program Phase 0 (machine-readable freeze) **and** Phase 1 (canonical domain ownership law in `architecture/domain-ownership.toml` — not consumer migration). C01 (DUP-002) increment spec: [`c01-contract-test-support-consolidation-run/spec.md`](c01-contract-test-support-consolidation-run/spec.md). DEBT-ENV P0 (Cargo workspace SSOT; not a second program SSOT): [`debt-env-p0-workspace-ssot-run/spec.md`](debt-env-p0-workspace-ssot-run/spec.md), [ADR 0051](../adr/0051-repository-environment.md).
