# ADR 0048 — Structural reconciliation (inventory, mechanical debt snapshot, active-spec drift)

| Field | Value |
| --- | --- |
| Status | **Accepted** — Phase 0+1 target suite GREEN (`sdd_structural_reconciliation_target`). |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Does not rewrite** [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), or [ADR 0012](0012-repository-hygiene.md) bodies. **Amends** human active-plane honesty for Guards **05–12** (already real in code) and debt count evidence. |
| Extends | [ADR 0004](0004-documentation-architecture.md), [ADR 0009](0009-repository-health-gate.md), [ADR 0011](0011-repository-guard-governance.md) |
| Spec | [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md) |
| Tests | `xtask/tests/sdd_structural_reconciliation_{baseline,target}.rs` (auto-discovered via `cargo test -p xtask`) |

<!-- weeping-angel-adr-meta
id = "0048"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0009-repository-health-gate", "0011-repository-guard-governance"]
-->

> Filename **`0048-structural-reconciliation.md`**. Cite **this file by path**. Do **not** add a `0003-structural-reconciliation.md` sibling. Next unused unique prefix is **0050**.

## Context

Live `cargo xtask guard` already prints **pass** for checks **01–15**. `ProductLawCheck` implements **05–12**; `DEBT-GUARD-05`…`12` are `resolved` in `docs/debt/register.toml`.

Before Phase 0+1, active human surfaces still described stub / skip-with-debt archaeology as current law (`docs/specs/repository-integrity.md` header/collision fence/current plane; `docs/debt/README.md` stub section). `docs/debt/baseline-2026-08.md` still presented itself as a live counts snapshot. There was no `cargo xtask inventory`, no `xtask/src/inventory.rs`, and no mechanical `docs/debt/current.md`.

Questions this decision answers:

1. What is the **single** mechanical inventory command and JSON contract?
2. Where do **current** vs **historical** debt count snapshots live?
3. How must **active** repository-integrity / debt prose describe Guards 05–12?
4. How are **superseded-state phrases** kept out of active specs without new product scanners?

## Decision (shipped)

Field-level law is [`docs/specs/structural-reconciliation.md`](../specs/structural-reconciliation.md).

1. **Inventory CLI.** `cargo xtask inventory` with `--json` / `--markdown` / `--check` lives in `xtask/src/inventory.rs`. Schema `weeping-angel/inventory/v1`. Exclusions: `target/`, `target-*`, `node_modules/`. Mutually exclusive flags; unknown/conflicting flags exit **2**. Drift vs committed `docs/debt/current.md` exits **1**.
2. **Debt snapshots.** `docs/debt/current.md` is the mechanical **current** evidence (regenerate with `--markdown`, verify with `--check`). `docs/debt/baseline-2026-08.md` is **Historical** only. This amends the human evidence plane from [ADR 0009](0009-repository-health-gate.md) without rewriting that ADR’s decision body.
3. **RI reconcile.** Active RI (and debt README) must match live 01–15 pass / resolved DEBT-GUARD-05…12. Stub / skip-with-debt archaeology lives only under **Historical**.
4. **Active-spec drift — enforcement seat (shipped).** Folded into Guard **15** (`check_15` → `check_active_spec_drift_on_model` after lifecycle validation). The same `check_active_spec_drift` helper also runs from `cargo xtask inventory --check`. No new Guard id and no product scanner. Phrases describing Guards 05–12 as present-tense stubs / skip-with-debt fail closed outside Historical / characterization fences.
5. **Subtractive-only Phase 0+1.** No new frameworks, collectors, ISMS modules, report formats, or product scanners.

## Consequences

- Contributors regenerate or `--check` debt counts instead of hand-syncing baseline prose.
- Active docs stop contradicting the live guard report; Guard 15 + `inventory --check` keep that honesty fail-closed.
- Dual-suite under `xtask/tests/` owns executable proof; `tests/sdd/` remains forbidden.
- Status is **Accepted** with SR-T\* GREEN and meta `status = "accepted"`.

## Rejected alternatives

- Keeping baseline-2026-08 as the live snapshot (drifts silently).
- Reopening DEBT-GUARD-05…12 as skip hatches to match stale docs (code is ahead; docs must catch up).
- Building a new standalone inventory crate or pnpm tool (Cargo xtask only).
- Mass-deleting Historical RI sections (breaks characterization anchors).
- Minting a new Guard id solely for documentary drift (prefer Guard 15 fold-in).
