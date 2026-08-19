# ADR 0005 — ISMS events and semantic snapshot drift (library observation stream)

| Field | Value |
| --- | --- |
| Status | **Accepted** — same shipped contract as [`0003-isms-events-drift.md`](0003-isms-events-drift.md); `sdd_isms_events_drift_target` GREEN; baseline skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** replace ADR 0001 compile pipeline, ADR 0002 ISO vertical, ADR 0003 assessment-lineage `compare` / `SnapshotDiff`, or [`0003-isms-events-drift.md`](0003-isms-events-drift.md). |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0003 assessment lineage](0003-assessment-lineage.md), [ADR 0003 ISMS events/drift](0003-isms-events-drift.md) (this file is **sibling notes**, not a second event model) |
| Spec | [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_isms_events_drift_{baseline,target}` |

> Filename `0005-*` is shared with scheduler / risk-register / methodology. Cite by **path**. SSOT decision: [`0003-isms-events-drift.md`](0003-isms-events-drift.md).

## Context

A Prompt 15 SDD run authored this `0005-*` sibling while catalog-program numbering also reserved `0003-isms-events-drift`. Both files describe **one** contract. Do not treat them as two event buses.

On SHA `6e31bf1…`, `compare` was readiness buckets only. Prompt 15 required a canonical observation stream and order-insensitive inventory drift.

## Decision (shipped; same as 0003)

1. **Facade modules, not a bus.** Import `weeping-angel-assurance::drift::{detect_events, detect_isms_drift, IsmsSnapshot}`. Types in `weeping-angel-assurance-ir::event` (and `EventId` from `id.rs`). `events` re-exports types only. No crate-root `detect_events`. No new crate. No Slack / Kafka / SQLite event ledger.

2. **Do not fork `SnapshotDiff`.** `detect_events(previous, next) → Vec<IsmsEvent>` is the catalog. `detect_isms_drift` wraps existing `compare` plus that catalog. Lineage `newSubjects` stay control ids. Scheduler `tick` still calls `compare`.

3. **Identity.** `eventId = event:sha256:{typed_canonical_digest("isms-event", body without id)}`. Time is next `evaluatedAt`. JSON camelCase; `causeRefs` is the persisted cause field (alias `causes`). Unit `kind` values are PascalCase strings. Repeated detect on the same pair dedupes by id. No UUID v4.

4. **Semantic equality.** Inventories keyed by stable id / envelope digest / SoA `reference`. Vec reorder emits nothing.

5. **Catalog and causes.** Prompt 15 kinds plus `Extensible`. Control regression includes `ExceptionApproved` → fail. Target-locked regression payload keys include `previousEffectiveness` / `nextEffectiveness`. Risk increase concurrent with a linked control regression carries `causeRefs` to that `ControlRegressed` `eventId`. `EvidenceExpired` is not `StaleEvidence`. `NewAssetDetected` is asset membership.

6. **Neighbors.** Temporal / residual / `IsmsContext` product is not this slice. Governance inventories may be empty; empty both sides is a no-op.

## Consequences

Readers should cite [`0003-isms-events-drift.md`](0003-isms-events-drift.md) as the decision. This file exists so older Prompt 15 links to `0005-*` keep resolving to the same Accepted law.

## Non-goals

Notification transport, scheduler/temporal product, `IsmsContext`, expanding IR `Risk`, forking `SnapshotDiff`, UI, new crate, UUID v4 identities.

## Related

- Decision SSOT: [`0003-isms-events-drift.md`](0003-isms-events-drift.md)
- Spec: [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md)
- Stub: [`docs/sdd/isms-events-drift.md`](../sdd/isms-events-drift.md)
- Scheduler Drift seam: [`0005-continuous-assurance-scheduler.md`](0005-continuous-assurance-scheduler.md)
