# ADR 0003 — ISMS events and deterministic snapshot drift

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_isms_events_drift_target` GREEN; `sdd_isms_events_drift_baseline` skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** replace ADR 0003 assessment-lineage `SnapshotDiff` / `compare`, envelope immutability, scheduler cadence, or temporal validity tables. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (facade / projections), [ADR 0003 assessment lineage](0003-assessment-lineage.md) (immutable snapshots), [ADR 0005 continuous assurance scheduler](0005-continuous-assurance-scheduler.md) (Drift **stage** calls `compare`; `detect_events` is this contract, not cadence) |
| Spec | [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_isms_events_drift_target` GREEN; `sdd_isms_events_drift_baseline` skip-superseded. Neighbors `sdd_assessment_lineage_target`, `sdd_assurance_runtime_target`, `sdd_compliance_ir_target` stay GREEN. |

> Filename `0003-*` is shared with catalog-program siblings. **0004** is documentation architecture. **0005-*** is already used by scheduler and risk. Cite this decision by **path**. Sibling notes: [`0005-isms-events-drift.md`](0005-isms-events-drift.md) (same contract; not a second event model).

## Context

ADR 0003 lineage made assessment runs and pairwise `compare` real. What shipped as drift was a **readiness string bag**: `SnapshotDiff` fields such as `controlBecameIneffective: ["{id} became ineffective"]`. `subject_ids` are control IDs. `compare_runs` / `compare_lineage` only flip pack/catalog digest booleans. There was no `IsmsEvent`, no `EventId`, and no semantic inventory diff.

The continuous-assurance scheduler fences `ControlRegressed` and the ISMS event catalog to this slice. Temporal assurance owns validity windows as append-only evidence-validity documents, not management-system events. Prompt 16 remediation needs **observations to consume**, not a second ticket system inside drift.

Operational ISMS v1 Prompt 15 requires a canonical event stream of meaningful management-system state changes and deterministic drift detection between immutable snapshots — without Slack, a generic bus, or mutable workflow tickets.

Questions this decision answers:

1. Are ISMS events observations or tickets?
2. Do we replace `SnapshotDiff` or extend the Drift seam beside it?
3. How is event identity assigned (no random v4)?
4. How do we suppress order-only serialization noise?
5. How does a risk increase cite a control regression?
6. What do we do while temporal/residual/ISMS-context product code is unlanded?
7. Where does persist live, and what is explicitly not a bus?

## Decision (shipped)

### 1. Events are immutable observations

`IsmsEvent` records a state transition between two snapshots (or a snapshot clock vs a validity/exception deadline). No assignee, SLA, or open/closed machine. Prompt 16 remediations **reference** `eventId` as a cause. Collectors never emit events. Recovery is a new `ControlRecovered` observation, not a mutation of the regression.

Schema constant: `ISMS_EVENT_SCHEMA = "weeping-angel/isms-event/v1"`. Domain-separated digest type name remains `"isms-event"`.

### 2. Extend the existing Drift seam; do not fork compare

Keep `compare` / `compare_runs` / `compare_lineage` / `SnapshotDiff` as the readiness helper (lineage tests stay GREEN). `newSubjects` remain **control ids**.

Shipped APIs (existing crates only; no new workspace crate). Callers import `weeping-angel-assurance::drift` — there is **no** crate-root re-export of `detect_events`.

| Surface | Home |
| --- | --- |
| `EventId` | `typed_id!(EventId)` in `weeping-angel-assurance-ir::id`; re-exported from IR crate root and `weeping-angel-assurance::events` |
| `IsmsEvent`, `IsmsEventKind`, `EventSubjectRef`, `EventCauseRef`, `EventSeverity`, `ISMS_EVENT_SCHEMA` | `weeping-angel-assurance-ir::event` |
| `IsmsSnapshot`, `ControlPosture`, `RiskPosture`, `EvidenceValidityView`, `GovernanceRecord`, `IsmsDrift`, `detect_events`, `detect_isms_drift` | `weeping-angel-assurance::drift` |
| Re-exports | `weeping-angel-assurance::events` (types only; not a notification bus; no SQLite ledger) |

```text
detect_events(previous, next) → Vec<IsmsEvent>     # sorted by eventId
detect_isms_drift(previous, next) → IsmsDrift      # readiness SnapshotDiff + events
```

`ASSURANCE_IR_SCHEMA` stays `assurance-ir/v1`. Framework and control-test stay network-free and event-free. Scheduler `tick` still calls `compare`; it does **not** invoke `detect_events`.

Named kinds: `ControlRegressed`, `ControlRecovered`, `EvidenceExpired`, `EvidenceRevoked`, `RiskIncreased`, `RiskDecreased`, `RiskAccepted`, `ExceptionExpired`, `NewAssetDetected`, `AssetRemoved`, `VendorRiskChanged`, `ObjectiveMissed`, `PolicyExpired`, `AuditFindingOpened`, `NonconformityOpened`, `CorrectiveActionOverdue`, plus `Extensible { name }`. Unit kinds serialize as adjacent JSON strings (`"ControlRegressed"`); `Extensible` is externally tagged (`{"Extensible":{"name":…}}`). Subject/cause `kind` values are camelCase (`control`, `event`).

Shipped detection freeze (v1):

- `ControlRegressed`: same `controlId`; `Effective` **or** `ExceptionApproved` → `Ineffective` / `PartiallyEffective`. Severity `material` / `notable`. Payload locks `fromEffectiveness` / `toEffectiveness` **and** `previousEffectiveness` / `nextEffectiveness` (same pair; target EVT-002 asserts the previous/next keys) plus `controlId` and `testIds`.
- `ControlRecovered`: inverse; new observation (not a mutation of the regression); severity `informational`.
- `EvidenceExpired`: `validUntil <= next.evaluatedAt` and the envelope was still inside the window on previous. Not `StaleEvidence`. Severity `notable`.
- `EvidenceRevoked`: `revokedAt` / `invalidatedAt` in `(previous.evaluatedAt, next.evaluatedAt]`. Severity `material`.
- `ExceptionExpired`: status becomes `Expired`, or `expiresAt` crossed while previous was `Approved` / `Proposed`. Keyed by `ExceptionId`.
- `NewAssetDetected` / `AssetRemoved`: asset-inventory membership. v1 sets `inScope: true` on add and `false` on remove; it does not apply a separate exclusion filter.
- `RiskIncreased` / `Decreased`: residual/inherent ordinal or documented status rank. Concurrent linked `ControlRegressed` is copied into `causeRefs` as `{kind:event,id:<eventId>}` and `{kind:control,id:C}`.
- `RiskAccepted`: `RiskStatus` becomes `Accepted`. v1 does not mint `acceptanceId`.
- `VendorRiskChanged`: one event per `RiskPosture.vendorIds` entry when that risk increased.
- Governance kinds fire only when those `GovernanceRecord` inventories are non-empty on at least one snapshot: terminal status / due clock (`ObjectiveMissed`, `PolicyExpired`, `CorrectiveActionOverdue`) or new id (`AuditFindingOpened`, `NonconformityOpened`). Empty both sides is a no-op. `CorrectiveActionOverdue` subject kind is `other`.
- `IsmsSnapshot.tests` is `Vec<ControlPosture>` and is **not** walked for events in v1.

### 3. Identity is SHA-256 typed digest of the event body

```text
eventId = "event:sha256:" + typed_canonical_digest("isms-event", EventIdentityBody)
EventIdentityBody = IsmsEvent without eventId
```

`EventId` is a `StableId`. `validate_stable_id` accepts the `event:sha256:` form. Identity body includes `schemaVersion`, `kind`, `occurredAt`, `sourceSnapshots`, `previousSnapshotDigest`, `nextSnapshotDigest`, sorted `subjects`, sorted `causeRefs`, optional `severity`, and `payload`. JSON is camelCase. No UUID v4.

`occurredAt` is the **next snapshot clock** (`IsmsSnapshot.evaluatedAt` as RFC 3339 Z), never emit-time `Utc::now()`. Snapshot pins on the event are the caller-supplied `snapshotId` pair (also copied into `sourceSnapshots`); this slice does not hash the inventory into a second digest.

Repeated `detect_events` on the same pair yields the same `eventId` set. `events.rs` does **not** ship a SQLite store; optional persist remains out of v1.

### 4. Semantic diff, not JSON-text diff

Callers assemble a normalized `IsmsSnapshot` (`snapshotId`, `evaluatedAt`, inventories keyed by stable id). Drift does not invent `IsmsContext`. Reordering assets, controls, exceptions, SoA entries, or evidence lists must not emit add/remove events. Semantically equal snapshots produce an empty event list.

`NewAssetDetected` is asset-inventory membership, **not** `SnapshotDiff.newSubjects`. Evidence expiry consumes `EvidenceValidityView.validUntil` vs the next clock; it is not `Effectiveness::StaleEvidence`. Exception expiry uses IR `Exception.id` / `expiresAt` / `ExceptionStatus::Expired`.

### 5. Causes are explicit

JSON field is `causeRefs` (serde alias `causes`). When a control regression and a linked risk increase are both detected in the same pair, `RiskIncreased.causeRefs` includes `{ kind: event, id: <ControlRegressed eventId> }` and `{ kind: control, id: C }`. Detection reads snapshot-local `RiskPosture` ordinals / `linkedControlIds`; this slice does not implement residual scoring.

### 6. Consume neighbors; do not implement them

- Temporal: `EvidenceValidityView` on the snapshot; no validity tables here.
- Scheduler: no cadence/retry/daemon. Scheduler Drift may still call `compare`; semantic events are this API.
- Residual/treatment/register: ordinals and ids if present on the view.
- Unlanded governance inventories: kinds exist; empty input is a no-op.

### 7. Not a notification bus

No Slack, email, webhooks, Kafka/NATS, or generic pub/sub. Events are library observations. Remediation, management review, and reporting consume `eventId`s later.

## Consequences

- Remediation, management review, and reporting have a stable observation vocabulary (`weeping-angel/isms-event/v1`).
- `SnapshotDiff` remains a readiness bag; docs must not claim it is the event stream.
- Two APIs coexist: `compare` (string bags, control-id subjects) and `detect_events` (typed observations, asset/risk/exception subjects).
- Neighbor dual-suites must stay GREEN; this slice must not rewrite lineage `compare`.

## Non-goals

- Notification transport / Slack / generic event bus.
- Scheduler product, temporal validity product, `IsmsContext`, remediation tickets.
- Replacing lineage `compare` or rewriting `newSubjects` to mean assets.
- A persisted event ledger in v1.

## Related

- Spec: [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md)
- Sibling notes: [`0005-isms-events-drift.md`](0005-isms-events-drift.md)
- Scheduler fence: [`docs/specs/continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md)
- Temporal: [`docs/specs/temporal-assurance.md`](../specs/temporal-assurance.md)
- Remediation consumer: [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
