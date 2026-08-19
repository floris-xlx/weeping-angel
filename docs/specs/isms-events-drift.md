# SDD: ISMS Events and Deterministic Drift

| Field | Value |
| --- | --- |
| Status | **Implemented** — `detect_events` / `detect_isms_drift` + `isms-event` observations; baseline GREEN for additive `compare`/`SnapshotDiff`; absence-of-events skip-superseded |
| Program | Operational ISMS v1 — ISMS events and drift engine |
| Slice | Canonical observation stream of management-system state changes + deterministic, order-insensitive snapshot drift |
| Dual-suite (registered) | `sdd_isms_events_drift_baseline` (GREEN additive compare; 3 absence/temporal found cases skip-superseded) · `sdd_isms_events_drift_target` (GREEN) |
| Dual-suite paths | `tests/contracts/isms_events_drift.{baseline,target}.rs` |
| ADR | **Accepted** [`docs/adr/0003-isms-events-drift.md`](../adr/0003-isms-events-drift.md) — public event/drift contract as shipped. Filename `0003-*` is a program sibling; cite by **path**. Sibling notes (same contract): [`docs/adr/0005-isms-events-drift.md`](../adr/0005-isms-events-drift.md). |
| Layout law | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — this file is the human SSOT; `docs/sdd/` is a stub pointer; traces go to `.sdd/runs/` |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — event stream + `detect_events` |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Lineage / snapshots (consumed, not redefined) | [`docs/specs/assessment-lineage.md`](assessment-lineage.md), ADR 0003-assessment-lineage |
| Scheduler (landed; do not implement cadence) | [`docs/specs/continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md), [ADR 0005](../adr/0005-continuous-assurance-scheduler.md) — Drift **stage** currently calls existing `compare`; this slice owns `detect_events` semantics, not cadence |
| Temporal (consumed; do not land windows here) | [`docs/specs/temporal-assurance.md`](temporal-assurance.md) |
| Risk / control / exception inventories | [`risk-register.md`](risk-register.md), [`risk-treatment.md`](risk-treatment.md), [`residual-risk.md`](residual-risk.md), [`control-implementation-registry.md`](control-implementation-registry.md) — consume views; do not implement those engines |
| Remediation (consumer; do not land tickets) | Landed [`remediation-engine.md`](remediation-engine.md) — events are inputs (`From<&IsmsEvent>`), not remediations |
| Incident governance (consumer; do not auto-promote) | [`incident-governance.md`](incident-governance.md) — `DetectionSource::AssuranceEvent(EventRef)` is an opaque id; events are not incidents until declare |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Event schema | `weeping-angel/isms-event/v1` (`ISMS_EVENT_SCHEMA`) — immutable observation document, not a mutable workflow ticket |
| Canonical digest | `canon/v1` via `canonical_digest` / `typed_canonical_digest`; SHA-256 hex; no random v4 identities |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; keep `sdd_assessment_lineage_target`, `sdd_assurance_runtime_target`, `sdd_compliance_ir_target` GREEN |

This document is the durable SSOT for Operational ISMS v1 **events and drift**. It owns **what a meaningful management-system change is**, **how two immutable snapshots produce a deterministic event list**, **event identity and deduplication**, and **the public observation types** later remediation, management review, and reporting consume.

It does **not** own scheduler cadence, temporal validity windows, residual-risk scoring, notification transport, Slack, a generic event bus, or remediation tickets.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Operational pipeline (scheduler owns cadence; this slice owns the last stage’s **semantics**):

```text
Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift
```

Events are **observations of state transition**, not mutable workflow tickets. Drift is a **pure function** of two snapshots. The same pair always yields the same events.

---

## 0. Collision fence (concurrent SDD)

Prompt 15 event types and Prompt 16 remediations are **landed**. **Do not fork a second event model.** Shipped names: `IsmsEvent` / `EventId` / `detect_events` / `detect_isms_drift` (`weeping-angel/isms-event/v1`). Remediation consumes these ids via `From<&IsmsEvent>`; it must not invent a second catalog.

This slice may add IR event types and an assurance events/drift module beside `snapshot.rs`. It may **call** existing `compare` / `compare_runs` / `compare_lineage`. It must not silently redefine `SnapshotDiff` field meaning (readiness string bags stay valid for lineage tests).

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| Kleene evaluator / `OrgContext` | applicability engine |
| Scheduler cadence, retry, backoff, jitter, daemon, `isms run` product | continuous-assurance scheduler |
| Envelope `DigestBody`, `valid_from` / `valid_until` product, revocation tables | temporal assurance — **consume a view** |
| `IsmsContext`, scope engine product, objectives/policy registries as new GRC | ISMS IR 01–12 — still unlanded; consume inventories that exist |
| Residual scoring engine, treatment state machine, register expansion | residual-risk / risk-treatment / risk-register — **consume views** |
| Remediation records, SLA, Jira/Linear adapters | Prompt 16 ([`remediation-engine.md`](remediation-engine.md), landed) |
| Slack, email, webhooks, Kafka/NATS/generic bus | non-goal |
| `tests/sdd/` | ADR 0004 forbids this path |

Suggested **new** product modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `EventId`, `IsmsEvent`, `IsmsEventKind`, payloads, `EventSubjectRef`, `EventCauseRef`, `EventSeverity` | `weeping-angel-assurance-ir::event` (`event.rs`); `typed_id!(EventId)` in `id.rs`; re-export from `lib.rs` |
| Semantic snapshot view, `detect_isms_drift`, order-insensitive inventory diff | `weeping-angel-assurance::drift` beside [`snapshot.rs`](../../crates/weeping-angel-assurance/src/snapshot.rs) |
| Event stream helpers (re-exports; no ledger in v1) | `weeping-angel-assurance::events` |
| Existing pairwise readiness compare | **keep** `snapshot::compare` / `SnapshotDiff` |
| Persist | **not shipped in v1**; any later store is keyed by `eventId` and is **not** a notification bus |

Tiny allowed adjustments: additive serde-default fields on `SnapshotDiff` **only if** existing needles still decode; `lib.rs` re-exports; `typed_id!` alias. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** change envelope digest law. Do **not** turn `new_subjects` into assets by rewriting lineage tests.

Neighbor targets that **must stay GREEN**: `sdd_assessment_lineage_target`, `sdd_assurance_runtime_target`, `sdd_compliance_ir_target`.

---

## 1. Problem / user-visible goal

Operators can run an assessment and pairwise-compare two **readiness** snapshots. They cannot treat operational change as a first-class, explainable observation.

On characterization SHA `6e31bf1a…`:

- there is no `IsmsEvent`, `EventId`, event stream, or drift module;
- `SnapshotDiff` is a bag of **formatted strings** (`"{id} became ineffective"`) plus digest booleans;
- `compare` walks `FrameworkReadinessSnapshot.controls` / `requirements` in **Vec order**;
- `subject_ids()` returns **control IDs**, not assets — `newSubjects` is not “new asset in scope”;
- `compare_runs` / `compare_lineage` only flip `frameworkPackDigestChanged` / `canonicalCatalogDigestChanged`;
- exception “expiry” is inferred from `Effectiveness::ExceptionApproved` transitions, not `Exception.expires_at`;
- evidence “stale” is `Effectiveness::StaleEvidence` on a **control**, not envelope `valid_until` / revocation;
- risks are a four-field stub (`id`, `title`, `description`, `status`); no residual projection, no cause graph;
- scheduler spec **explicitly fences** `ControlRegressed` and the event catalog to this slice.

That means later remediation, management review, and reporting have nothing canonical to consume. A control that flips `Effective → Ineffective` is a prose string. Re-running compare on the same pair is not a documented identity. Reordering a serialized asset list would look like change if a naive JSON diff were used.

**User-visible goal:** given two immutable snapshots of the management system, produce a **canonical, deduplicated event stream** of meaningful state changes (control regression/recovery, evidence expiry/revocation, risk movement/acceptance, asset enter/leave, exception expiry, and extensible governance kinds) with stable IDs, time, source snapshots, subjects, causes, optional severity, and a deterministic payload — and **emit nothing** when only serialization order changed.

Definition of done: *operational changes are first-class, explainable events that later remediation, management review, and reporting can consume.*

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `SnapshotDiff` / `compare` / `compare_runs` / `compare_lineage` | `weeping-angel-assurance::snapshot` | **Keep.** Readiness string bags remain valid. Drift **extends** this seam: after snapshot, call semantic detect **in addition to** (or wrapping) compare. Do not make `new_subjects` mean assets. Do not replace `control_became_ineffective` strings with events in a way that breaks `sdd_assessment_lineage_target`. |
| `FrameworkReadinessSnapshot` | `readiness.rs` | Input for control-effectiveness transitions. |
| `AssessmentRun` | `snapshot.rs` | Source snapshot identity, clock (`completed_at` / `started_at`), pack/catalog pins. |
| `AssessmentDefinition` inventories | IR `assessment.rs` | Scope, assets, vendors, risks, exceptions, implementations, tests, controls — the management-system graph. |
| `Exception` / `ExceptionStatus` / `expires_at` | IR `exception.rs` | SSOT for `ExceptionExpired` (not only readiness `ExceptionApproved`). |
| `Asset` / `AssetId` | IR `asset.rs` | SSOT for `NewAssetDetected` / `AssetRemoved`. |
| `Vendor` / `VendorId` | IR `vendor.rs` | Inventory for `VendorRiskChanged` when a vendor-linked risk posture changes. Today `Vendor` is `{ id, name }` — consume additive fields if they land; otherwise posture comes from linked `Risk` records. |
| `Risk` / `RiskStatus` | IR `risk.rs` | Still a four-field stub on HEAD. Drift reads a **normalized `RiskPosture` view** (status + optional ordinal scores + linked control ids) so this slice does not implement scoring. |
| Residual / treatment / acceptance | specs only on HEAD | Consume if types exist at implement. `RiskAccepted` requires a sealed acceptance **or** an explicit `RiskStatus::Accepted` transition **plus** whatever 08 types are present — do not invent `RiskAcceptance` here. |
| `ControlImplementation` | IR `implementation.rs` | Implementation inventory drift; reverse link `risk_ids` for cause wiring. |
| `Effectiveness` | control-test | Regression = leaving `Effective` toward `Ineffective` / `PartiallyEffective` / fail-closed defects. Recovery is the inverse. Do not add residual ratings to this enum. |
| `TestFailureSeverity` | IR `test.rs` | Optional event severity source. |
| `EvidenceSnapshot` | lineage | Envelope digest set. Add/remove/supersede remain lineage fields; **expiry/revocation** consume a temporal **view**. |
| Validity events / `valid_until` / revoke | temporal spec; **absent in product** | Drift accepts `EvidenceValidityView` (envelope digest, window, revoked/invalidated-at). Do not implement ledger validity tables here. |
| `StatementOfApplicability` | `soa.rs` | SoA entry semantic diff (applicability / implementation_state / exceptions), order-insensitive by `reference`. |
| `StableId` / `typed_id!` / `validate_stable_id` | IR `id.rs` | `EventId` uses this. Reject UUID v4. |
| `canonical_digest` / `typed_canonical_digest` | IR `digest.rs` | Event identity. Domain-separated type name `"isms-event"`. |
| Scheduler Drift stage | scheduler spec | Will call this detect API. This slice does not schedule. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_isms_events_drift_*` only in the implement commit that adds the `.rs` files. |

Serde law: camelCase JSON (`rename_all = "camelCase"`). `BTreeMap` / `BTreeSet` / sorted subject/cause vectors for canonical bytes.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded by `tests/contracts/isms_events_drift.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 No event types, no drift module, no EventId

Workspace product crates have **no** `IsmsEvent`, `EventId`, `IsmsEventKind`, `ControlRegressed`, `detect_isms_drift`, or `isms-event/v1`.

[`crates/weeping-angel-assurance/src/`](../../crates/weeping-angel-assurance/src/) is `applicability/`, `bridge.rs`, `lib.rs`, `lineage.rs`, `readiness.rs`, `snapshot.rs`, `soa.rs`. There is no `events.rs` / `drift.rs`.

IR `id.rs` has no `EventId`. `lib.rs` does not export an event module.

### 3.2 `SnapshotDiff` is a readiness string bag

[`snapshot.rs`](../../crates/weeping-angel-assurance/src/snapshot.rs):

```text
SnapshotDiff {
  controlBecameEffective: Vec<String>      // "{id} became effective"
  controlBecameIneffective: Vec<String>    // "{id} became ineffective"
  evidenceBecameStale: Vec<String>         // control id
  newSubjects / disappearedSubjects        // control ids (see §3.4)
  requirementBecameApplicable / NotApplicable
  manualReviewResolved
  newExceptions / expiredExceptions        // "{id} exception approved|expired"
  evidenceAdded / Removed / Superseded     // default empty; compare() does not fill
  frameworkPackDigestChanged
  canonicalCatalogDigestChanged
}
```

These are **not** typed events. There is no stable event id, no cause reference, no severity, no snapshot pair pin on the diff document itself.

### 3.3 `compare` only walks readiness Vecs

`compare(previous, next)`:

- matches controls by `id` in **iteration order of `next.controls`**;
- `Effective → Ineffective` pushes a formatted string (not `ControlRegressed`);
- `Ineffective → Effective` pushes “became effective”;
- **new** controls (no prior) also push “became effective”;
- `StaleEvidence` pushes the **control id** into `evidenceBecameStale`;
- `ExceptionApproved` enter/leave fill exception string bags;
- requirements walk similarly;
- pack/assessment digest inequality sets **both** digest-changed flags when `assessment_digest` differs.

`compare_runs` / `compare_lineage` only set digest booleans. They do not emit control, asset, risk, or exception observations.

### 3.4 “Subjects” are control IDs

```text
fn subject_ids(snapshot) = snapshot.controls.iter().map(|c| c.id)
```

`newSubjects` / `disappearedSubjects` therefore mean **controls appearing/disappearing on the readiness projection**, not `Asset` inventory membership, not `AssessmentScope` subjects. A new in-scope asset with no new control row produces **no** `newSubjects` entry.

### 3.5 Inventories exist but are unused by compare

`AssessmentDefinition` already holds `scope`, `assets`, `vendors`, `risks`, `exceptions`, `implementations`, `tests`, `controls`. `Exception` has `expires_at` and `ExceptionStatus::Expired`. `compare` never reads them.

`Risk` is `{ id, title, description, status }`. `Vendor` is `{ id, name }`. No residual ordinal, no vendor risk rating, no `IsmsContext`, no objectives/policy documents in product IR.

### 3.6 Evidence expiry vs stale vs revoke

Stale on HEAD = `Effectiveness::StaleEvidence` (age of `collected_at` vs `max_age`) attributed to a **control**. There is no `EvidenceExpired` (validity window) and no `EvidenceRevoked` (validity event). Envelope digest add/remove fields on `SnapshotDiff` are unused by `compare`.

### 3.7 Order sensitivity / no-op

Two readiness snapshots that are semantically equal except control **Vec order** still walk in that order. String bags are not sorted by a documented law. There is no “no-op snapshots ⇒ empty event list” API. Identical compare inputs yield the same `SnapshotDiff` **values**, but there is no event identity to dedupe across repeated calls into a stream.

### 3.8 Scheduler / temporal / remediation fences (must remain true of this slice’s non-goals)

- Scheduler module still absent; this slice must not add `Clock` cadence/retry.
- Temporal validity events were absent on the characterization SHA; Prompt 14 later landed `evidence-validity/v1`. This slice still must not own those tables or validity-event kinds.
- No Slack, no notification transport.
- No `Remediation` type.

### 3.9 What current tests lock (do not break)

- `sdd_assessment_lineage_target` serializes `SnapshotDiff` and requires `compare` / `compare_runs` / `compare_lineage`.
- `sdd_assurance_runtime_target` facade contract: compare remains a readiness projection helper.
- `sdd_compliance_ir_target` golden IR fixtures (`risk.json`, `control-implementation.json`, …) still decode; do not require event fields on those documents.

---

## 4. Desired behavior (after implement)

### 4.1 Events are observations, not tickets

An `IsmsEvent` records that **between two immutable snapshots** (or at a snapshot clock vs a validity/exception deadline) a management-system fact changed. It has no assignee, no SLA, no open/closed state machine, and cannot be “updated in place.”

Prompt 16 remediations **reference** event ids as causes. They are a different type.

Forbidden:

- mutating an event row except identical-byte idempotent append;
- using events as a notification bus;
- treating `ControlRegressed` as a work item that an operator closes.

### 4.2 Public types (names normative; rust fields snake_case, JSON camelCase)

Shipped freeze (target GREEN):

```text
ISMS_EVENT_SCHEMA = "weeping-angel/isms-event/v1"   # digest type name remains "isms-event"
eventId             = "event:sha256:" + typed_canonical_digest("isms-event", body without id)
occurredAt          = next.evaluatedAt as RFC 3339 Z
sourceSnapshots     = [previous.snapshotId, next.snapshotId]
previousSnapshotDigest / nextSnapshotDigest = those snapshotId strings
subjects            # JSON; sorted (kind, id)
causeRefs           # JSON (serde alias "causes"); sorted
kind                # unit variants as strings ("ControlRegressed"); Extensible externally tagged
```

`events.rs` re-exports types only; v1 does **not** ship `append_isms_events` / a SQLite event ledger.

```text
ISMS_EVENT_SCHEMA = "weeping-angel/isms-event/v1"

EventId                 # typed StableId; form event:sha256:{hex}; not UUID v4
EventSeverity           # informational | notable | material | critical
EventSubjectRef { kind, id }     # kind: control|asset|risk|exception|evidence|vendor|implementation|test|requirement|objective|policy|finding|nonconformity|other
EventCauseRef { kind, id }       # kind: event|control|risk|evidence|exception|snapshot|other

IsmsEventKind =
    ControlRegressed
  | ControlRecovered
  | EvidenceExpired
  | EvidenceRevoked
  | RiskIncreased
  | RiskDecreased
  | RiskAccepted
  | ExceptionExpired
  | NewAssetDetected
  | AssetRemoved
  | VendorRiskChanged
  | ObjectiveMissed
  | PolicyExpired
  | AuditFindingOpened
  | NonconformityOpened
  | CorrectiveActionOverdue
  | Extensible { name }          # equivalent variants; name is a StableId-shaped token

IsmsEvent {
  schemaVersion: "weeping-angel/isms-event/v1"
  eventId: EventId
  kind: IsmsEventKind            # adjacent string for unit variants
  occurredAt: RFC3339            # next snapshot clock, never emit-time wall clock
  sourceSnapshots: [String]      # [previous.snapshotId, next.snapshotId]
  previousSnapshotDigest: String
  nextSnapshotDigest: String
  subjects: [EventSubjectRef]    # sorted (kind, id)
  causeRefs: [EventCauseRef]     # sorted; may be empty; alias "causes"
  severity: EventSeverity?       # required for regression / risk-increase / overdue kinds
  payload: <kind-specific object, canonical JSON>
}
```

`eventId` **excludes itself** from the hashed body:

```text
eventId = "event:sha256:" + typed_canonical_digest("isms-event", EventIdentityBody)
EventIdentityBody = IsmsEvent without eventId
```

`validate_stable_id` accepts `event:sha256:` plus hex.

Payloads (shipped keys; extra keys allowed if `skip_serializing_if` empty so identity stays stable):

| Kind | Payload |
| --- | --- |
| `ControlRegressed` | `controlId`, `fromEffectiveness`, `toEffectiveness`, `previousEffectiveness`, `nextEffectiveness` (same pair; **EVT-002 locks previous/next**), `testIds` |
| `ControlRecovered` | `controlId`, `fromEffectiveness`, `toEffectiveness`, `previousEffectiveness`, `nextEffectiveness` |
| `EvidenceExpired` | `envelopeDigest`, `validUntil` |
| `EvidenceRevoked` | `envelopeDigest`, `revokedAt` |
| `RiskIncreased` / `RiskDecreased` | `riskId`, `fromStatus`, `toStatus`, `fromOrdinal`, `toOrdinal` (integer ranks; **no** crate-wide `RiskRating` enum) |
| `RiskAccepted` | `riskId` (v1 does not mint `acceptanceId`) |
| `ExceptionExpired` | `exceptionId`, `controlId`, `expiresAt` |
| `NewAssetDetected` | `assetId`, `kind`, `inScope: true` |
| `AssetRemoved` | `assetId`, `inScope: false` |
| `VendorRiskChanged` | `vendorId`, `riskId`, `fromOrdinal`, `toOrdinal` — one event per linked vendor on `RiskIncreased` |
| governance kinds | `{ id, status? }`; omit the event entirely if **both** snapshots have empty inventories |

### 4.3 Snapshot input to drift (normalized view, not a second GRC graph)

Do **not** invent `IsmsContext`. Drift reads a **normalized** pair:

```text
IsmsSnapshot {
  snapshotId: String              # caller pin; copied onto events (not auto-hashed inventory)
  evaluatedAt: DateTime<Utc>      # snapshot clock / injected as_of; becomes event occurredAt
  runId?: AssessmentId / run pin
  assets: [Asset]                 # keyed by AssetId; membership is the in-scope signal
  vendors: [Vendor]
  risks: [RiskPosture]            # see below
  controls: [{ id, effectiveness, implementationIds, testIds }]
  implementations: [ControlImplementation]
  exceptions: [Exception]
  evidence: [EvidenceValidityView]    # envelopeDigest + validity window / revoke clocks (no nested wrapper)
  tests: [ControlPosture]             # present on the view; **not** walked for events in v1
  soa: StatementOfApplicability?
  objectives / policies / findings / nonconformities / corrective_actions: [GovernanceRecord]
}

RiskPosture {
  id: RiskId
  status: RiskStatus
  linkedControlIds: [ControlId]   # from Risk.control_ids if present, else reverse ControlImplementation.risk_ids
  residualOrdinal?: i32           # consume residual-risk projection when present
  inherentOrdinal?: i32
  vendorIds?: [VendorId]
}

EvidenceValidityView {
  envelopeDigest: String
  validFrom?: DateTime<Utc>
  validUntil?: DateTime<Utc>
  revokedAt?: DateTime<Utc>
  invalidatedAt?: DateTime<Utc>
}
```

Callers **assemble** this view from lineage snapshots + definition inventories + optional temporal/residual modules. Target tests may construct `IsmsSnapshot` directly (fixtures) so this slice is not blocked on Prompts 01–14 product code.

Helpers:

```text
detect_isms_drift(previous: &IsmsSnapshot, next: &IsmsSnapshot) -> IsmsDrift
IsmsDrift {
  readiness: SnapshotDiff      # existing compare on embedded/derived readiness, if available
  events: Vec<IsmsEvent>       # sorted by eventId
}
```

If a caller only has two `FrameworkReadinessSnapshot`s, a documented adapter may emit **control** events (regression/recovery/exception-from-effectiveness) but **must not** claim `NewAssetDetected` from control-id `newSubjects`.

### 4.4 Semantic equality — suppress order-only noise

Two snapshots are **semantically equal** (no-op) iff, after normalizing unordered collections by `StableId` (and SoA by `reference`, evidence by envelope digest):

- every inventory membership matches;
- every compared field used by event rules matches;
- Vec/map **order** and equivalent JSON key order are ignored.

Rules:

1. No events when previous and next are semantically equal (required test: no-op snapshots).
2. Reordering `assets`, `controls`, `exceptions`, SoA `entries`, or evidence digest lists **must not** emit add/remove events.
3. `detect_isms_drift` output `events` is sorted by `eventId` (then kind, then first subject id) so callers never depend on walk order.
4. Do **not** emit events for whitespace-identical JSON that serde would canonicalize to the same typed values.
5. Digest-only churn of a document that is **not** in the compared semantic fields is out of scope unless a kind exists for that document.

Existing `compare` may remain order-sensitive in its **string bag order**; the **event** list must not be.

### 4.5 Detection rules (normative for required tests)

Clock `T = next.evaluatedAt`.

| Event | When |
| --- | --- |
| `ControlRegressed` | Same `controlId` present in both; `from` is `Effective` **or** `ExceptionApproved`; `to` is `Ineffective` or `PartiallyEffective`. One event per control per snapshot pair. Severity `material` / `notable`. |
| `ControlRecovered` | Inverse: `Ineffective`/`PartiallyEffective` → `Effective`. New observation; severity `informational`. |
| `EvidenceExpired` | An envelope in previous (or still listed) has `validUntil <= T` and was inside its window on previous (`validUntil` absent on previous or `previous.evaluatedAt < validUntil`). Not the same as `StaleEvidence`. Severity `notable`. |
| `EvidenceRevoked` | `revokedAt`/`invalidatedAt` in `(previous.evaluatedAt, T]`. Severity `material`. |
| `RiskIncreased` | Same `riskId`; `residualOrdinal` or `inherentOrdinal` increased, **or** `status` worsened by rank (`Closed`/`Retired` < `Mitigated`/`Accepted` < `Draft`/`UnderTreatment` < `Open`; title-only edits do not). |
| `RiskDecreased` | Inverse ordinal/status improvement. |
| `RiskAccepted` | `status` becomes `Accepted`. v1 does not emit from a sealed `RiskAcceptance` document. |
| `ExceptionExpired` | `Exception.status` becomes `Expired`, **or** `expiresAt <= T` while previous was still `Approved`/`Proposed` and `previous.evaluatedAt < expiresAt`. Subject is IR `Exception.id`. |
| `NewAssetDetected` | `AssetId` present in next `assets` and absent from previous. Payload `inScope: true` (v1 does not apply a separate exclusion filter). |
| `AssetRemoved` | Inverse; payload `inScope: false`. |
| `VendorRiskChanged` | Emitted for each `RiskPosture.vendorIds` entry when that risk increased. |
| governance kinds | Only when those inventories are non-empty on at least one snapshot. Terminal status / due clock (`ObjectiveMissed`, `PolicyExpired`, `CorrectiveActionOverdue`) or new id (`AuditFindingOpened`, `NonconformityOpened`). Empty both sides ⇒ no event, not an error. `CorrectiveActionOverdue` subject kind is `other`. |

**Cause wiring (required):** if a `ControlRegressed` event is produced for control `C` and a `RiskIncreased` event is produced for a risk that lists `C` in `linkedControlIds`, the risk event’s `causeRefs` **must** include `{ kind: event, id: <ControlRegressed eventId> }` (and may also include `{ kind: control, id: C }`). This is the “risk increase **caused by** a control regression” found case. Do not emit a cause edge when the risk does not cite the control.

Severity:

- `ControlRegressed`: map `TestFailureSeverity` of the failing test if present, else `material` for `Ineffective`, `notable` for `PartiallyEffective`.
- `RiskIncreased`: `material` if ordinal jump > 1 or status → `Open` from treated; else `notable`.
- `EvidenceExpired` / `ExceptionExpired`: `notable` default.
- Informational kinds (`NewAssetDetected` without risk change): `informational` unless specified.

### 4.6 Deduplication

`detect_isms_drift(A, B)` is a pure function. Two calls with equal snapshots return **byte-equal** `events` (same `eventId`s).

v1 does **not** ship `append_isms_events`. Dedup is the pure function: two calls with equal snapshots return the same `eventId` set. A later persist, if added, must be idempotent by `eventId` (`INSERT OR IGNORE`); conflicting bytes for an id fail closed.

Required test: run detect twice on the same regression pair; the event id set is identical.

### 4.7 Time and identity

- `occurredAt` is `next.evaluatedAt` (snapshot completion / injected `as_of`). Tests inject a fixed clock on the snapshot. **Never** `Utc::now()` inside detect.
- Snapshot digests on the event pin the pair. Changing an unrelated field that **is** part of `IsmsSnapshot` canonical body changes `snapshotDigest` and therefore event identity — keep the snapshot body to compared inventories so whitespace-only serialization of unrelated reports does not churn ids.
- No random v4. No `CollectionRun::new`-style wall-clock uniqueness.

### 4.8 Extensibility

Unknown future kinds use `IsmsEventKind::Extensible { name }` with a deterministic payload object. Detectors for unlanded inventories (objectives, policies, audit findings, nonconformity, CAPA overdue) **may** be implemented as empty-input no-ops. The enum **must** name the Prompt 15 kinds so serde/API consumers can match them without waiting for Prompt 16–22 product registries.

### 4.9 Crate and pipeline placement

```text
weeping-angel-assurance-ir   # types only
weeping-angel-assurance      # detect_isms_drift + optional persist
weeping-angel-evidence       # optional table only if persist reuses the ledger; no bus
weeping-angel-control-test   # unchanged Effectiveness
weeping-angel-framework      # unchanged; network-free
weeping-angel-collector      # unchanged; never emits IsmsEvent
```

Scheduler (later) Drift stage:

```text
Snapshot → detect_isms_drift(previous_snapshot, next_snapshot) → append_isms_events
```

until then, library callers invoke detect directly.

### 4.10 Public contract / docs (implementation phase)

Done after target GREEN:

1. [`docs/adr/0003-isms-events-drift.md`](../adr/0003-isms-events-drift.md) **Accepted** (sibling [`0005-isms-events-drift.md`](../adr/0005-isms-events-drift.md) restates the same contract).
2. [`docs/specs/assurance-runtime.md`](assurance-runtime.md) records `detect_events` / `detect_isms_drift` and that `compare` / `SnapshotDiff` stay readiness helpers.
3. This path is in `CANONICAL_SPECS` (`tests/contracts/documentation_layout.rs`).
4. Status **Implemented**; baseline skip-superseded.

---

## 5. Dual-suite law

Root `Cargo.toml` does **not** auto-discover `tests/contracts/*.rs`. Implement **must** add, in the **same commit** as the `.rs` files:

```toml
[[test]]
name = "sdd_isms_events_drift_baseline"
path = "tests/contracts/isms_events_drift.baseline.rs"

[[test]]
name = "sdd_isms_events_drift_target"
path = "tests/contracts/isms_events_drift.target.rs"
```

Without those stanzas, `cargo test --test sdd_isms_events_drift_{baseline,target}` fails with `no test target named …` before any `#[test]` runs (I3 HARD FAIL).

| Suite | File | Bar |
| --- | --- | --- |
| Baseline | `isms_events_drift.baseline.rs` | GREEN on `6e31bf1…`: no `IsmsEvent`/`EventId`; `SnapshotDiff` string bags; `subject_ids` are control ids; `compare` does not produce `ControlRegressed`. After target GREEN: skip-supersede. |
| Target | `isms_events_drift.target.rs` | **RED on current HEAD for missing event/drift contract**, not missing `[[test]]` harness. GREEN after implement. Command: `cargo test --test sdd_isms_events_drift_target --offline -- --nocapture`. |

I4a: the target suite must **never** read its own source and assert it lacks a substring. Assert product types and runtime behavior.

Transition: **replacement**. Absence-of-events baseline cannot stay CI-required after the module exists.

Regression after GREEN: `cargo test --workspace --features demo`. Neighbors listed in the header stay green.

Protocol: write tests first (target encodes original found cases) → **RED** → fix → **GREEN**. Abort if baseline cannot go GREEN on current compare/absence characterization, or target cannot go RED for the missing event/drift contract (not harness noise).

### 5.1 Baseline suite contents (GREEN on CURRENT)

- Product crates do not define `IsmsEvent`, `EventId`, `detect_isms_drift`, or schema `isms-event/v1`.
- `SnapshotDiff` serializes the existing camelCase string-bag fields.
- `compare` on Effective → Ineffective fills `controlBecameIneffective` with a formatted string containing the control id, and does **not** return a typed `ControlRegressed` event.
- `newSubjects` for a readiness snapshot whose **controls** differ uses control ids; adding an `Asset` to a definition is out of band for `compare`.
- `compare_runs` only flips digest booleans.
- No notification / Slack / bus types in assurance.

### 5.2 Target suite (GREEN after implement)

Landed target tests (`tests/contracts/isms_events_drift.target.rs`). Found-case titles are the `#[test]` names:

| ID | Test | Found case → shipped |
| --- | --- | --- |
| — | `dual_suite_is_registered` | Root `Cargo.toml` lists both `[[test]]` paths. |
| EVT-001 | `evt_001_noop_permuted_equal_snapshots` | Identical or Vec-permuted inventories → empty `events`. |
| EVT-002 | `evt_002_one_control_regressed` | One control `Effective → Ineffective` → exactly one `ControlRegressed` (`previousEffectiveness` / `nextEffectiveness`); `compare` string bags unchanged. |
| EVT-003 | `evt_003_evidence_expired` | Envelope `validUntil` crossed by next clock → `EvidenceExpired`; not `StaleEvidence`. |
| EVT-004 | `evt_004_risk_increased_caused_by_control_regression` | Linked risk ordinal/status worsens **and** the control regresses → `RiskIncreased.causeRefs` includes the `ControlRegressed` `eventId`. |
| EVT-005 | `evt_005_new_asset_detected` | Asset appears in next inventory → `NewAssetDetected` (not a control-id `newSubjects` string). |
| EVT-006 | `evt_006_exception_expired` | Status `Expired` → `ExceptionExpired` keyed by `ExceptionId`. |
| EVT-007 | `evt_007_detect_events_dedupes` | Second `detect_events` on the same pair yields the same `eventId` set (SHA-256, no UUID v4). |
| — | `p15_control_recovered_is_a_new_event` | Recovery is a new `ControlRecovered`, not a mutated regression. |
| — | `p15_compare_snapshot_diff_not_forked` | `SnapshotDiff` remains the readiness string bag. |

---

## 6. Acceptance criteria (testable)

1. Dual-suite files exist at `tests/contracts/isms_events_drift.{baseline,target}.rs` and are registered in root `Cargo.toml` in the **same implement commit**.
2. Baseline is GREEN on characterization SHA / current pre-implement HEAD (absence + current `compare` characterization); target is RED there for the **missing event/drift contract**, not harness noise.
3. After implement: target GREEN (EVT-001…007 plus recovery / `SnapshotDiff` not-forked); baseline skip-superseded; `sdd_assessment_lineage_target`, `sdd_assurance_runtime_target`, `sdd_compliance_ir_target` stay GREEN.
4. Public types live in existing crates only (`assurance-ir` + `assurance` events/drift beside `snapshot.rs`). No new workspace crate. No notification bus.
5. Each required event kind used by tests (`ControlRegressed`, `EvidenceExpired`, `RiskIncreased`, `NewAssetDetected`, `ExceptionExpired`, plus catalog of named kinds) carries stable `eventId`, `occurredAt`, source snapshot digests, subjects, causes (where applicable), optional severity, and a deterministic camelCase payload.
6. Events are immutable observations: no ticket state machine; identical bytes append is idempotent; conflicting bytes for an id fail closed.
7. Semantically equal snapshots, including order-only inventory reshuffles, emit **no** events.
8. One control `Effective → Ineffective` emits one `ControlRegressed`.
9. Evidence validity crossing `validUntil` emits `EvidenceExpired` (temporal view consumed; windows not reimplemented).
10. A risk increase concurrent with a linked control regression carries a cause reference to that `ControlRegressed` event.
11. A new in-scope `Asset` emits `NewAssetDetected` (distinct from readiness control-id subjects).
12. An exception past `expiresAt` / status `Expired` emits `ExceptionExpired`.
13. Repeated diff of the same snapshot pair deduplicates by `eventId`.
14. `eventId` uses `StableId` + SHA-256 `typed_canonical_digest`; no random v4.
15. Existing `compare` / `SnapshotDiff` remain valid readiness helpers; this slice extends the scheduler Drift seam rather than replacing lineage compare.

---

## 7. Out of scope

- Notification transport, Slack, email, webhooks, Kafka/NATS/Redis streams, or a generic application event bus.
- Scheduler cadence, retry, backoff, jitter, daemon, or `weeping-angel isms run` product.
- Implementing temporal `valid_from`/`valid_until` product tables (consume `EvidenceValidityView` only).
- Implementing `IsmsContext`, scope engine, objectives/policy registries, residual scoring, treatment state machines, or register expansion.
- Remediation tickets, CAPA workflow, Jira/Linear/GitHub issue adapters (Prompt 16).
- Rewriting `SnapshotDiff` so `newSubjects` means assets, or breaking `sdd_assessment_lineage_target`.
- New workspace crate or long-term database product.
- Collectors emitting `IsmsEvent` or `Effectiveness`.
- UI dashboards / management-review reports (they **consume** events later).
- Persisted event ledger / `append_isms_events` (v1 is the pure `detect_events` function).

---

## 8. Risks

- Forking a second event model while a sibling Prompt 15/16 run lands different names — **mitigation:** continue existing types if present; this spec is SSOT for names `IsmsEvent` / `EventId` / `detect_isms_drift`.
- Silently redefining `SnapshotDiff` and breaking lineage target tests — **mitigation:** wrap/extend; keep string bags.
- Blocking on unlanded residual/temporal/ISMS-context engines — **mitigation:** normalized `IsmsSnapshot` views and fixtures.
- Using `Utc::now()` in `eventId` / `occurredAt`, making dedup tests flake — **mitigation:** snapshot clock only.
- Order-sensitive Vec walks emitting noisy add/remove — **mitigation:** id-keyed maps; sort events by id.
- Treating events as tickets (owner, status updates) and colliding with Prompt 16.
- Target RED only because `[[test]]` is missing (I3) instead of missing contract.
- Putting validity windows or scheduler cadence into this crate surface.
- Inventing a crate-wide `RiskRating` enum against risk-treatment/methodology law — use ordinals on `RiskPosture`.
- Confusing `StaleEvidence` with `EvidenceExpired`.

---

## 9. ADR

**Required and Accepted.** Public contract: `IsmsEvent`, `EventId`, `detect_events` / `detect_isms_drift`. Decision: [`docs/adr/0003-isms-events-drift.md`](../adr/0003-isms-events-drift.md). Sibling notes: [`docs/adr/0005-isms-events-drift.md`](../adr/0005-isms-events-drift.md) (same contract; cite `0003-*` as SSOT). Filename `0003-*` follows catalog-program sibling numbering.

---

## 10. Implement sequence

Completed: IR types + `drift::detect_events` / `detect_isms_drift`; target GREEN; baseline skip-superseded; ADRs Accepted; `assurance-runtime.md` and `CANONICAL_SPECS` record the contract.

---

## 11. Definition of done

Operational changes are first-class, explainable events. Snapshot pairs produce a deterministic, deduplicated stream. Order-only serialization is silent. Required found cases (no-op, one regression, evidence expiry, risk increase caused by regression, new asset, expired exception, repeated-diff dedup) are GREEN. Neighbor lineage/runtime/IR targets remain GREEN. No notification bus. No remediation tickets.
