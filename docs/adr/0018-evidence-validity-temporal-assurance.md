# ADR 0018 — Evidence validity and temporal assurance

<!-- weeping-angel-adr-meta
id = "0018"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_temporal_assurance_target` GREEN (17/17); baseline skip-superseded. Sibling dual-suite `sdd_evidence_validity_temporal_assurance_*` encodes the same product contract. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “time is `provenance.collected_at` plus `max_age`” *operational* reading of [ADR 0002](0002-iso-27001-assurance-vertical.md) Phase 7–8 **as implemented** (window = collection time; no as-of). Does **not** supercede envelope immutability, `DigestBody = observation + provenance`, ledger ownership of observations, or INV-1…5. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [typed evidence](0036-typed-evidence-canonical-serialization.md), [population](0034-subject-population-runtime-and-coverage-semantics.md), [assessment lineage](0015-assessment-lineage.md) |
| Spec | [`docs/specs/temporal-assurance.md`](../specs/temporal-assurance.md) (SSOT). Sibling Prompt 14 spec: [`docs/specs/evidence-validity-temporal-assurance.md`](../specs/evidence-validity-temporal-assurance.md). |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_temporal_assurance_target` GREEN; `sdd_temporal_assurance_baseline` skip-superseded. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Product types (`evidence-validity/v1`, `PeriodEffectiveness`, `select_latest_as_of`) exist **once**.

## Context

ADR 0002 required immutable envelopes and a SQLite ledger, and named `observedAt` / `validFrom` / `validUntil` as *target* envelope fields. What shipped hashed `DigestBody { observation, provenance }` only. `collected_at` was the sole clock. `supersedes` is an out-of-digest pointer. Staleness was `now - collected_at > max_age`. Facade `assess` used `Utc::now()` and a 24h window. `Exists` could conclude `Effective` from one in-set observation. `compare` diffs two readiness snapshots.

Operational ISMS v1 temporal assurance requires Weeping Angel to prove **historical operating effectiveness**, not only current posture: deterministic as-of and period evaluation, no future-into-past leakage, no expired-into-window leakage, and timeline/diff primitives — without mutating sealed evidence and without a new long-term database.

The continuous-assurance scheduler owns cadence. This decision leaves a `FreshnessPolicy` / evaluation-clock seam and does not implement retry or a daemon.

Questions this decision answers:

1. Are validity windows part of the sealed digest, or append-only events?
2. How do `observed_at`, `collected_at`, `valid_from`, `valid_until`, supersession, and revocation compose?
3. What may satisfy an assessment at time `T` or range `[A, B)`?
4. When may a single observation imply continuous effectiveness?
5. How do timeline/diff relate to existing `SnapshotDiff`?
6. What does the scheduler consume later?

## Decision

### 1. Sealed envelopes stay observation+provenance; validity is a new event

`DigestBody` remains `{ observation, provenance }`. Historical digests do not churn.

Temporal usability is an append-only document `evidence-validity/v1` (`EvidenceValidityEvent`) stored beside envelopes on the existing SQLite `EvidenceLedger`. Kinds: `asserted` | `superseded` | `revoked` | `invalidated`. `eventId` is SHA-256 of canonical JSON of the event **excluding** `eventId`.

Forbidden: editing a sealed payload, rewriting `valid_until` in place, resealing to change a window.

Optional envelope fields `observedAt` / `validFrom` / `validUntil` / `sourceRevision` may exist **outside** `DigestBody` (serde default, omitted when none). Accessors default `observed_at → collected_at`, `valid_from → observed_at`. `project_validity` starts from those accessors and overlays events with `at <= T`. ISO Phase 7’s named fields are therefore **projected**, not mixed into the digest.

### 2. First-class clocks

| Field | Role |
| --- | --- |
| `collected_at` | Seal/collection instant (`EvidenceProvenance`) |
| `observed_at` | World-fact instant (default `collected_at`) |
| `valid_from` / `valid_until` | Half-open assurance window `[from, until)` — `T == valid_from` in; `T == valid_until` out |
| `supersedes` | Prior envelope digest (existing field) |
| revocation / invalidation | Later event; envelope remains readable |
| `source_revision` | Source/collector revision string (optional) |
| artifact digest | `EvidenceArtifactRef.digest` when present |

### 3. Selection is as-of, fail-closed on leakage

An envelope may satisfy point-in-time `T` only if it is a **candidate** at `T`: collected and observed at or before `T`, inside `[valid_from, valid_until)`, not revoked/invalidated at or before `T` (unless a later `asserted` event restores the window), and a supersession leaf among candidates. Ties: `observed_at`, then `collected_at`, then digest.

Future observations never satisfy a past clock. Expired windows never satisfy outside themselves. Evaluation selectors (`first_selector` / `EvidenceIndex` via `build_index_as_of`) use this filter. Digest-order first-hit over the unbounded bag is not an evaluation selector.

Stale (`max_age` / `FreshWithin`) is distinct from expired, future, and missing. `is_stale` runs only on candidates.

`EvidenceLedger::within_window` remains inclusive `collected_at`. Validity-window query is `valid_during` (`[start, end)`). Collection-time `latest` is unchanged. As-of evaluation leaf is `as_of` (`latest_as_of` is an alias). Live valid leaf is `current`; membership at `T` is `valid_at` ([ADR 0011](0047-temporal-lineage-evidence-soa-integrity.md)).

`append` of a new envelope also records an `asserted` event. `supersede` appends the new envelope and a `superseded` event for the previous digest; the previous payload is untouched. `record_validity_event` is idempotent by `eventId`; different bytes for the same id are `LedgerError::Immutable`.

### 4. Period projection is a separate result

Point-in-time keeps `Effectiveness`. Period assessment emits `PeriodEffectiveness` on `ControlTestResult.period`: `continuouslyEffective` | `intermittentRegression` | `insufficientObservationCoverage` | `ineffective` | `manualReviewRequired`.

Default temporal semantics are `instant`. Continuous fill of a gap requires explicit `continuousUntilSuperseded` or an asserted interval. One `Exists` hit must not imply continuous operating effectiveness.

`AssessmentContext` remains `{ now, max_age }`. `as_of()` is `now` (injected clock). `period()` is unset on the struct; `project_period_effectiveness` uses `FreshnessPolicy.period` when present, otherwise implicit `[now - max_age, now)`. Sampling is event-boundary plus period endpoints (deterministic; no hidden cron).

### 5. Timeline/diff are library primitives

`project_timeline` and `compare_temporal` / `diff_period` (`EvidenceTimeline`, `TemporalDiff`: gaps, expiry, revoke, supersede, intermittent / insufficient-coverage control ids) serve readiness and audit exports. Existing pairwise `compare` / `SnapshotDiff` is not redefined.

### 6. Scheduler seam only

`FreshnessPolicy { max_age, as_of, period }` is the handoff. Live `assess` still defaults `now = Utc::now()` and 24h `max_age`. Historical replay injects a fixed `AssessmentContext.now`. `AssessmentRun` JSON `asOf` is the pinned evaluation clock (serialized from the `as_of` field; live `assess` may default it to `startedAt`).

### 7. Same ledger, no new product DB, no UI

Reuse `EvidenceLedger`. No charts.

## Implemented surfaces

| Concern | Home |
| --- | --- |
| `EvidenceValidityEvent`, `project_validity`, `window_contains` | `weeping-angel-evidence::validity` (`EVIDENCE_VALIDITY_SCHEMA`) |
| Envelope clocks (out-of-digest) | `EvidenceEnvelope::{with_observed_at,with_valid_from,with_valid_until,with_source_revision}` |
| Ledger events / `valid_during` / `as_of` / `latest_as_of` / `current` / `valid_at` | `weeping-angel-evidence::ledger` |
| `TimeRange`, `FreshnessPolicy`, `TemporalQuery`, `PeriodEffectiveness`, `select_latest_as_of`, `select_evidence`, `project_period_effectiveness` | `weeping-angel-control-test::temporal` |
| As-of index | `build_index_as_of` / evaluate `first_selector` |
| `ControlTestResult.period` | control-test result document |
| `project_timeline`, `compare_temporal`, `diff_period` | `weeping-angel-assurance::temporal` |
| Pinned `asOf` | `AssessmentRun` |

## Consequences

- Public contract documents the temporal-evidence API in [`assurance-runtime.md`](../specs/assurance-runtime.md).
- Neighbor SDD targets that lock digest law stay GREEN: validity clocks are outside `DigestBody`; as-of filtering is additive to evaluation, not a digest change.
- Catalog fact fields already named `valid_until` remain observation facts; they are not automatically envelope events.
- Historical `assess` replay must inject `AssessmentContext.now`; live `assess` still uses wall-clock unless the caller builds its own context.
- Two Prompt 14 dual-suites may exist; they share one `evidence-validity/v1` type.

## Non-goals

UI charts; new long-term database; scheduler product (cadence/retry/daemon); catalog/GitHub/ISO remap rewrites; certification claims.

## Related

- Spec: [`docs/specs/temporal-assurance.md`](../specs/temporal-assurance.md)
- Sibling spec: [`docs/specs/evidence-validity-temporal-assurance.md`](../specs/evidence-validity-temporal-assurance.md)
- Ledger / envelope: [`docs/specs/typed-evidence.md`](../specs/typed-evidence.md), ISO Phase 7–8
- Lineage replay: [`docs/specs/assessment-lineage.md`](../specs/assessment-lineage.md)
- Scheduler seam: [`docs/specs/continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md)
- Sibling ADR: [`0035-temporal-assurance.md`](0035-temporal-assurance.md)
- Four-clock / fail-closed replay / historical SoA: [ADR 0011](0047-temporal-lineage-evidence-soa-integrity.md)
