# Technical-debt register

Canonical machine register: [`register.toml`](register.toml) (`schema = "weeping-angel/debt-register/v1"`). Decisions: [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md), [`docs/adr/0010-architecture-as-law.md`](../adr/0010-architecture-as-law.md), [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md), [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md) (concept-level ownership lives in `architecture/domain-ownership.toml`, not this register).

This README is not a second register. **Current** mechanical counts: [`current.md`](current.md) (regenerate / verify with `cargo xtask inventory --markdown` / `--check`). [`baseline-2026-08.md`](baseline-2026-08.md) is **Historical** evidence only, not live status. Frozen Phase 0 consolidation snapshot (not live; Guard 04 expansion reference): [`consolidation-baseline.json`](consolidation-baseline.json) / [`consolidation-baseline.md`](consolidation-baseline.md) (`schema = "weeping-angel/consolidation-baseline/v1"`). Print with `cargo xtask inventory --consolidation-baseline`; do not treat a reprint as a rewrite of the committed freeze. Structural duplication **program backlog**: [`structural-duplication.toml`](structural-duplication.toml) (`schema = "weeping-angel/structural-duplication/v2"`, close law: no `verified`/`removed` until canonical owner, consumers, old-path removal or `compatibility-only`, and a regression guard).

Repository-hygiene before/after counts live in [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md) §12 (and optionally `.sdd/runs/`), not in `register.toml`. [ADR 0012](../adr/0012-repository-hygiene.md) does not close `DEBT-IGNORE` / `DEBT-UNWRAP` / `DEBT-SCHEMA-DUP` from this slice.

## Status machine

Every `[[finding]]` has a required `id`, `title`, `status`, and `summary`.

Closed set of `status` values:

| status | Meaning |
| --- | --- |
| `open` | Known, not yet triaged to a workstream |
| `confirmed` | Accepted as real debt |
| `in-progress` | Active remediation |
| `resolved` | Fixed **and** guarded (see proof law) |
| `rejected` | Not debt (wontfix / false positive) |
| `superseded` | Replaced by another finding or design |

## Proof law (`resolved`)

A finding may be `status = "resolved"` **only if** it lists a non-empty `regression_tests` array **or** a non-empty `repository_guard` value (check id string, or boolean `true` meaning the live guard covers it).

`rejected` and `superseded` do not require `regression_tests` or `repository_guard`.

Guard check **13** (`cargo xtask guard`) rejects:

- missing required fields
- illegal status
- duplicate `finding.id`
- `resolved` without proof

## Guard checks (current plane)

Checks **01–15** are implemented `ArchitectureCheck`s. `DEBT-GUARD-04`…`15` are `resolved` with `repository_guard` and/or `regression_tests`. There is no live debt-backed skip hatch for 05–12 or 14–15 on the healthy tree. Silent pass remains forbidden for any future exemption.

## Historical — former stub skip policy

Prior increments allowed checks **05–12** and **14–15** to skip only by citing a registered `DEBT-GUARD-NN` finding. That archaeology is retained here for characterization; it is **not** current gate behavior.
