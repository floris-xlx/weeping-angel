# ADR 0049 — Architectural consolidation Phase 0 (program table, frozen baseline, duplication backlog schema)

| Field | Value |
| --- | --- |
| Status | **Accepted** — Phase 0 target GREEN (`sdd_architectural_consolidation_target`); baseline suite deleted (`INV-NO-SUPERSEDED-BASELINES`). |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Does not rewrite** [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), or [ADR 0048](0048-structural-reconciliation.md) bodies. **Amends** architecture policy: `architecture.toml` gains a parsed `[program.architectural_consolidation]` table; extra TOML tables are no longer ignored. |
| Extends | [ADR 0004](0004-documentation-architecture.md), [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), [ADR 0048](0048-structural-reconciliation.md) |
| Spec | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) |
| Tests | `xtask/tests/sdd_architectural_consolidation_target.rs` (auto-discovered via `cargo test -p xtask`). Phase 0 baseline suite deleted (`INV-NO-SUPERSEDED-BASELINES`). |

<!-- weeping-angel-adr-meta
id = "0049"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0010-architecture-as-law", "0011-repository-guard-governance", "0048-structural-reconciliation"]
-->

> Filename **`0049-architectural-consolidation-phase-0.md`**. Cite **this file by path**. Do **not** add a `0003-architectural-consolidation*.md` sibling. Next unused unique prefix after this file is **0050**.

## Context

Live `cargo xtask guard` already prints **pass** for checks **01–15**. `cargo xtask inventory` owns mechanical counts in `docs/debt/current.md` (`weeping-angel/inventory/v1`). `docs/debt/structural-duplication.toml` maps 17 duplication rows for Structural Reconciliation Phase 2 (`weeping-angel/structural-duplication/v1`).

That is not enough to start semantic consolidation:

1. Architectural-cleanup **Phase 0** is a review-bar freeze in [`docs/specs/architectural-cleanup-program.md`](../specs/architectural-cleanup-program.md) / [ADR 0010](0010-architecture-as-law.md). It is **not** a parsed program table. `load_architecture_manifest` reads `[policy]` + `[ownership.*]` and **ignores extra tables**, so a paper TOML section would not fail CI.
2. There is no frozen consolidation snapshot. `current.md` is **live** and must stay live. Later phases cannot prove “we did not add a second `[[test]]` / public struct / schema tree” against a Phase 0 line in the sand.
3. Duplication rows use statuses `migrating | resolved | false-positive` and omit `severity`, `canonical_symbol`, `migration_state`, `removal_blockers`, `public_api_impact`, `serialization_impact`, `tests`. `resolved` can be claimed while duplicate paths remain. No xtask parser loads the file.

Questions this decision answers:

1. Where does **consolidation mode** live, and when is it a gate rather than a comment?
2. What is the **single** count source for a frozen Phase 0 baseline vs live `current.md`?
3. What schema and close law apply to `structural-duplication.toml` as the program backlog?
4. Which guard check evaluates the freeze (without a 16th product-semantic check or a second health CLI)?

## Decision (Accepted — field-level law is the spec)

Field-level law is [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md). This ADR records the architecture-policy choices:

1. **Program table.** `architecture/architecture.toml` MUST contain `[program.architectural_consolidation]` with `status = "active"` and `feature_expansion = "restricted"` plus allowed/forbidden change-class arrays. `ArchitectureManifest` MUST parse the table. Missing/malformed → fail closed. Extra tables are no longer silently ignored for this key.
2. **Enforcement seat.** Fold into Guard **01** (manifest load) and Guard **04** invariants (`INV-CONSOLIDATION-MODE-ACTIVE`, `INV-CONSOLIDATION-BASELINE-PRESENT`, `INV-CONSOLIDATION-EXPANSION-RESTRICTED`, `INV-STRUCTURAL-DUPLICATION-BACKLOG` or equivalent names). Do **not** add Guard 16. Do **not** add a second health CLI. CI continues to run `cargo xtask guard`.
3. **Frozen baseline.** `docs/debt/consolidation-baseline.json` + `.md` (`weeping-angel/consolidation-baseline/v1`) are the Phase 0 snapshot. Counts come from extending `xtask/src/inventory.rs` (same walker as `current.md`). `docs/debt/current.md` remains the live mechanical snapshot. Print-only `cargo xtask inventory --consolidation-baseline` (and `--consolidation-baseline-markdown`) must not rewrite the committed freeze. While restricted, expansion metrics MUST NOT increase vs the frozen JSON.
4. **Backlog schema.** `docs/debt/structural-duplication.toml` becomes schema `weeping-angel/structural-duplication/v2`, `program = "architectural-consolidation"`, `phase = 0`. Required row fields and status set are in the spec. v1 `migrating` → `consumers-migrating`; `resolved` / `false-positive` MUST NOT auto-map to `verified` or `removed`. Close law: canonical owner exists, consumers migrated, old path removed (or `compatibility-only`), and a regression guard exists.
5. **Program law.** One concept → one semantic owner → one canonical representation → one computation path → multiple projections. Do not rewrite Weeping Angel; eliminate parallel truths. This Phase 0 is **not** architectural-cleanup Phase 0.
6. **Executable proof.** Dual-suite under `xtask/tests/` only (`FORBID-TESTS-SDD`). After target GREEN, the baseline suite was **deleted** (not `#[ignore]`).

## Consequences

- PRs that raise frozen expansion metrics (`[[test]]`, schema files, public structs/enums, `pub use`, duplicated helpers, workspace crate count) fail `cargo xtask guard` while the program is active/restricted.
- Contributors cannot treat a comment-only freeze as law; the loader is the gate.
- DUP rows stay open until close law holds; v1 “resolved” honesty is preserved as `canonicalized` / `consumers-migrating` / `compatibility-only`.
- Inventory remains the only counter (DUP-014: do not add a third walker).

## Rejected alternatives

- Leaving consolidation mode as markdown-only (architectural-cleanup Phase 0 style) — not machine-readable.
- A 16th `ProductLawCheck` or `cargo xtask consolidation` CLI — forks the health plane ([ADR 0011](0011-repository-guard-governance.md)).
- A second inventory/walker for the baseline — DUP-014.
- Mapping v1 `resolved` → `verified` — silently closes rows with duplicates still on disk.
- `#[ignore]`-superseding the baseline suite — violates `INV-NO-SUPERSEDED-BASELINES`.
- Minting `0003-*` / colliding `0011-*` ADR files.

## Status

**Accepted** after Phase 0 target GREEN (`cargo test -p xtask --test sdd_architectural_consolidation_target`) and deletion of `xtask/tests/sdd_architectural_consolidation_baseline.rs`. Proof: Guard 01 parses `[program.architectural_consolidation]`; Guard 04 evaluates `INV-CONSOLIDATION-MODE-ACTIVE`, `INV-CONSOLIDATION-BASELINE-PRESENT`, `INV-CONSOLIDATION-EXPANSION-RESTRICTED`, `INV-STRUCTURAL-DUPLICATION-BACKLOG`; frozen snapshot `docs/debt/consolidation-baseline.{json,md}`; backlog `docs/debt/structural-duplication.toml` schema v2.

## Related

- Spec: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)
- Pointer: [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md)
- Frozen snapshot: [`docs/debt/consolidation-baseline.md`](../debt/consolidation-baseline.md)
- Backlog: [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml)
- Successor (Phase 1 ownership law): [ADR 0050](0050-domain-ownership-model.md)
- Predecessor: [ADR 0048](0048-structural-reconciliation.md)
