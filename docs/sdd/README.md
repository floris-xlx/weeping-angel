# Moved

Human canonical specifications live in [`docs/specs/`](../specs/).

Decisions stay in [`docs/adr/`](../adr/).

Executable invariants live in [`tests/contracts/`](../../tests/contracts/) (repo-wide dual-suites) and `xtask/tests/*.rs` (architecture-as-law increment).

This folder is a **stub**. Do not write generated SDD traces, hygiene count dumps, or raw `audit.txt` here. Historical notes in this folder are not a second SSOT: [`repository-integrity.md`](repository-integrity.md), [`architectural-cleanup-program.md`](architectural-cleanup-program.md), [`structural-reconciliation.md`](structural-reconciliation.md), [`architectural-consolidation-program.md`](architectural-consolidation-program.md). Per-increment consolidation run specs may live at `docs/sdd/<slug>-<run>/spec.md` (program §12); they are not a second program SSOT. DEBT-ENV P0: [`debt-env-p0-workspace-ssot-run/spec.md`](debt-env-p0-workspace-ssot-run/spec.md).

Generated SDD runs and snapshots are local-only under [`.sdd/`](../../.sdd/) and are not part of the source tree. Hygiene law: [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md), [ADR 0012](../adr/0012-repository-hygiene.md).

See [`docs/README.md`](../README.md) and [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md).
