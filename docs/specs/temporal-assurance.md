# SDD: Evidence Validity and Temporal Assurance

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_temporal_assurance_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — temporal assurance |
| Slice | First-class evidence validity windows, deterministic as-of / period selection, temporal control projection, timeline/diff primitives |
| Dual-suite (register at implement, same commit as `.rs`) | `sdd_temporal_assurance_baseline` · `sdd_temporal_assurance_target` (`tests/contracts/temporal_assurance.{baseline,target}.rs`) |
| ADR | Accepted [`docs/adr/0003-evidence-validity-temporal-assurance.md`](../adr/0003-evidence-validity-temporal-assurance.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) |
| Consumes | Immutable ledger ([`typed-evidence.md`](typed-evidence.md), ISO Phase 7–8), population latest/supersede ([`population-runtime.md`](population-runtime.md)), lineage snapshots ([`assessment-lineage.md`](assessment-lineage.md)) |
| continuous-assurance scheduler (landed) | [`docs/specs/continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md), [ADR 0005](../adr/0005-continuous-assurance-scheduler.md) — cadence / retry / `tick`; this slice still must not implement them |
| Spine / ISO law | [`assurance-runtime-spine.md`](assurance-runtime-spine.md), [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Evidence schema | remains `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Validity-event schema | `evidence-validity/v1` (`EVIDENCE_VALIDITY_SCHEMA`) — append-only document, not an IR type |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for temporal assurance. It owns **evidence temporal fields**, **append-only validity / revocation events**, **deterministic selection at an assessment clock or range**, **point-in-time and period control projection**, and **timeline/diff primitives** for readiness and audit exports.

It does **not** own catalog TOML, the GitHub collector, ISO remapping, continuous-assurance scheduler scheduling, UI charts, or a new long-term database.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Time is an **assurance dimension**, not a UI concern. A finalized assessment must evaluate against the evidence that was **valid at that exact time** or **throughout a declared audit period**, with no temporal leakage.

## Implemented contract (what shipped)

Target `sdd_temporal_assurance_target` GREEN. Types exist once (sibling dual-suite `sdd_evidence_validity_temporal_assurance_*` asserts the same APIs).

| Concern | Home |
| --- | --- |
| `EvidenceValidityEvent`, `project_validity`, `window_contains` | `weeping-angel-evidence::validity` |
| Out-of-digest clocks | `EvidenceEnvelope` `observedAt?` / `validFrom?` / `validUntil?` / `sourceRevision?` (accessors default `observed_at → collected_at`, `valid_from → observed_at`) |
| Ledger | `append` records `asserted`; `supersede` records `superseded` without mutating the prior row; `record_validity_event` (idempotent `eventId`; conflict → `Immutable`); `validity_events` / `validity_events_for`; `valid_during([start,end))`; `latest_as_of`. `within_window` / `latest` stay collection-time. |
| Selection | `select_latest_as_of`, `select_evidence(TemporalQuery)`, `build_index_as_of`; evaluate `first_selector` uses the as-of leaf |
| Clock | `AssessmentContext { now, max_age }`; `as_of()` = `now`; `FreshnessPolicy { max_age, as_of, period }` |
| Period | `PeriodEffectiveness` on `ControlTestResult.period`; default `TemporalSemantics::instant`; sampling = validity-event / envelope boundaries plus `[start, end)` |
| Timeline | `weeping-angel-assurance::{project_timeline, compare_temporal, diff_period}` |
| Replay pin | `AssessmentRun` JSON `asOf` (live `assess` serializes from `startedAt`) |

Half-open window: `valid_from <= T` and (`valid_until` is none or `T < valid_until`). Future / expired / revoked-at-T envelopes are not candidates. One `Exists` hit is point-in-time `Effective` and period `insufficientObservationCoverage` unless semantics explicitly allow continuity.

---

## 0. Collision fence (concurrent SDD)

This slice may edit only temporal-validity / as-of selection / period projection / timeline-diff / ledger validity-event paths.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML | catalog families / controlled documents |
| `frameworks/iso-27001/2022/**` IDs and `to =` mappings | controlled documents |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | residual risk |
| continuous-assurance scheduler cadence, retry, backoff, jitter, daemon, `isms run` product | continuous-assurance scheduler ([ADR 0005](../adr/0005-continuous-assurance-scheduler.md)) |
| New PostgreSQL / remote ledger / object store | non-goal |
| UI charts / dashboards | non-goal |
| `tests/sdd/` | [ADR 0004](../adr/0004-documentation-architecture.md) forbids this path |

Suggested **new** product modules stay in **existing crates** (no new crate, no new long-term DB):

| Concern | Home |
| --- | --- |
| Envelope temporal view, validity events, `select_as_of` / window query | `weeping-angel-evidence` (+ `ledger`) |
| As-of index, stale vs expired vs future, period projection | `weeping-angel-control-test` |
| Facade clock / `as_of` / period, timeline + temporal diff, contract text | `weeping-angel-assurance` |

Tiny allowed adjustments: optional serde-default fields on envelopes (must not rewrite `DigestBody`); new validity-event type + ledger table; `AssessmentContext` clock fields; `Effectiveness` / period-result enums; `SnapshotDiff` sibling for period coverage. Do **not** redesign catalog TOML, IR core fields, collector discovery, or sealed-envelope digest law.

Neighbor targets that **must stay GREEN**: `sdd_typed_evidence_target`, `sdd_assessment_lineage_target`, `sdd_population_runtime_target`, `sdd_assurance_runtime_target`.

---

## 1. Problem / user-visible goal

Weeping Angel can describe **current** posture from whatever envelopes sit in an `EvidenceSet`. It cannot prove **historical operating effectiveness**.

On characterization SHA `6e31bf1a…`:

- the only envelope clock is `provenance.collected_at`;
- there is no `observed_at`, `valid_from`, `valid_until`, source revision, or revocation record;
- `DigestBody` is observation + provenance; `supersedes` / `collectionRunId` / `artifactRef` are outside the digest;
- ledger `within_window` and `latest` order by `collected_at` only;
- population `select_latest` walks `supersedes` then latest `collected_at` + digest;
- stale means `now - collected_at > AssessmentContext.max_age` (or selector freshness);
- facade `assess()` stamps `Utc::now()` and a hard-coded 24h `max_age`;
- `TestExpr::Exists` may conclude `Effective` from a single in-set observation;
- there is no as-of or period selector, so a bag that already contains later envelopes can leak into a past clock;
- `compare` / `SnapshotDiff` is pairwise readiness, not period coverage.

That means a reviewer cannot ask:

```text
what evidence was valid at 2026-08-01T00:00:00Z?
was control X continuously effective throughout Q2?
did we have observation coverage, or a single lucky snapshot?
did a later collection or a revocation change the historical answer?
```

**User-visible goal:** given a finalized assessment identity and either an instant or a declared audit period, reconstruct the evidence that was valid **then**, evaluate controls against **only** that evidence, and export a timeline/diff that distinguishes continuous effectiveness from intermittent regression, sparse observation, and expiry — without mutating sealed envelopes.

Definition of done: *a finalized assessment can be evaluated against the evidence that was valid at that exact time or throughout a declared audit period, with no temporal leakage.*

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `EvidenceEnvelope` / `seal` | `weeping-angel-evidence` | Keep `DigestBody { observation, provenance }`. Do not put validity events inside the sealed digest. Additive optional fields / accessors only. |
| `EvidenceProvenance.collected_at` | same | Remains the **collection** instant. Do not overload it as observed-at or valid-until. |
| `supersedes` / `with_supersedes` | envelope + `EvidenceLedger::supersede` | Keep append-only pointer. Validity change is a **new** envelope or validity event, never an UPDATE. |
| `EvidenceArtifactRef.digest` | evidence crate | This is the artifact digest. Surface it; do not invent a second blob store. |
| `EvidenceLedger` | `ledger.rs` | Reuse SQLite. Add validity-event persist/query. No new DB product. `append` stays idempotent by envelope digest. |
| `AssessmentContext` | control-test | Today `{ now, max_age }`. Grow a declared clock: `as_of` / optional `period`. Must be injectable (tests + historical replay). |
| `select_latest` / `EvidenceIndex` | `population.rs` | Today latest-by-`collected_at` after supersede walk. Must become **as-of latest** (no future, no expired, revoked excluded). |
| `first_selector` | `control-test/lib.rs` | Digest-order first match. Point-in-time / Exists must use the same temporal selector as the index (not an unbounded bag). |
| `is_stale` / `envelope_stale` / `FreshWithin` | control-test | Today age vs `collected_at`. Must distinguish **stale** (policy freshness) vs **expired** (outside `valid_until`) vs **future** (observed/collected after the clock). |
| `TestExpr::Exists` | control-test | May stay a point-in-time predicate. Must **not** imply period-continuous effectiveness. |
| `Effectiveness` | control-test | Keep existing variants. Period projection adds a **distinct** period-result type (do not overload `Effective` to mean “continuously effective”). |
| `assess` | `weeping-angel-assurance` | Today wall-clock `Utc::now()` + 24h. Must accept a declared `as_of` / period / clock. Default may remain “now + 24h” for live assess. |
| `SnapshotDiff` / `compare` | `snapshot.rs` | Pairwise readiness stays. Add timeline / period-diff primitives; do not silently redefine `compare`. |
| `AssessmentRun` / lineage | lineage + snapshot | Replay must pin the evaluation clock (`as_of` / period) so historical assessment is reproducible. Prefer carried fields with serde defaults. |
| continuous-assurance scheduler | **landed** ([ADR 0005](../adr/0005-continuous-assurance-scheduler.md)) | This slice exposes `FreshnessPolicy` / evaluation-clock types only. Do not reimplement cadence, retry, or daemon here. |
| Dual-suite neighbors | root `Cargo.toml` | Do not disturb green targets listed in the header. Register `sdd_temporal_assurance_*` only in the implement commit that adds the `.rs` files. |

ISO Phase 7 already *named* `observedAt` / `validFrom` / `validUntil` as target envelope fields ([`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md) §Phase 7) and required the sealed core `{ observation, provenance, digest }` to remain. This slice **implements that intent** without breaking ACT digest law.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded later by `tests/contracts/temporal_assurance.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 Provenance clock only

[`crates/weeping-angel-evidence/src/lib.rs`](../../crates/weeping-angel-evidence/src/lib.rs):

```text
EvidenceProvenance { collector_id, collected_at, scope, asset }
EvidenceEnvelope {
  evidence_id, schema_version, artifact_ref?, collection_run_id,
  content_digest, sensitivity, scope, supersedes?,
  observation, provenance, digest
}
```

There are **no** envelope fields named `observed_at`, `valid_from`, `valid_until`, `revoked_at`, `invalidated_at`, or `source_revision`.

`content_digest` is cloned from the observation+provenance digest at `seal`. Artifact identity lives on `EvidenceArtifactRef.digest` when `with_artifact_ref` is used; it is **not** mixed into `DigestBody`.

### 3.2 Seal / digest / immutability

```text
DigestBody { observation, provenance }
digest = SHA-256 hex(serde_json of DigestBody)   // IR canonical_digest
evidence_id = "ev:sha256:{digest}"
content_digest = digest
schema_version = "evidence/v1"
```

Outside `DigestBody` (changing them after seal does not rewrite `digest`): `evidenceId`, `artifactRef`, `collectionRunId`, `sensitivity`, `scope`, `supersedes`.

`seal` always sets `supersedes = None`. Callers attach history via `with_supersedes` / `EvidenceLedger::supersede` (new row; previous payload untouched).

### 3.3 Ledger windows are collection-time only

[`crates/weeping-angel-evidence/src/ledger.rs`](../../crates/weeping-angel-evidence/src/ledger.rs):

| API | Behavior on this HEAD |
| --- | --- |
| `append` | `INSERT OR IGNORE` by envelope `digest`; stores `collected_at` column from provenance |
| `latest(type)` | `ORDER BY collected_at DESC LIMIT 1` — no supersede walk, no as-of |
| `within_window(start, end)` | `collected_at >= start AND collected_at <= end` (inclusive), order `collected_at` |
| `for_subject` / `for_type` / `for_collection_run` | order `collected_at` |
| `supersede` | `get` previous + `with_supersedes` + `append` |
| validity events | **none** — no table, no revoke API |

`latest` can return a superseded envelope if its `collected_at` is newer than the leaf, or a future-dated `collected_at` if that is the max row.

### 3.4 Population latest vs Exists first-hit

[`select_latest`](../../crates/weeping-angel-control-test/src/population.rs):

1. Collect every `supersedes` pointer in the `(type, subject)` group.
2. Keep envelopes whose **own digest** is not in that set (leaves).
3. Sort leaves by `provenance.collected_at` then `digest`; take the last.

No filter on evaluation clock. A leaf collected **after** `AssessmentContext.now` still wins.

[`first_selector`](../../crates/weeping-angel-control-test/src/lib.rs): first envelope in `EvidenceSet` digest (`BTreeMap`) order matching `evidence_type` and optional subject id. **Not** latest, **not** as-of.

`TestExpr::Exists` on a hit that is not `is_stale` → `Effective`. One in-set observation is enough.

### 3.5 Stale is age of `collected_at`

```text
is_stale(env, ctx) =
  (ctx.now - env.provenance.collected_at) > ctx.max_age
  or conversion failure (including collected_at > now) → true
```

`envelope_stale` also applies optional `EvidenceSelector.freshness`. `FreshWithin` uses the same `collected_at` age.

Consequences:

- **Future `collected_at`** (`collected_at > now`) becomes `StaleEvidence`, not “out of candidate set”. `first_selector` may pick that future row by digest order and **shadow** an older still-valid envelope.
- There is no **expired** state: nothing can leave the window except by age-from-collection.
- There is no **observed_at**: a fact about a later world-state sealed with an earlier `collected_at` looks current.

### 3.6 Facade clock is wall time

[`AssuranceEngineBuilder::assess`](../../crates/weeping-angel-assurance/src/lib.rs):

```text
AssessmentContext { now: Utc::now(), max_age: Duration::from_secs(24 * 3600) }
```

No `as_of`, no period, no injected clock. Historical replay of `evaluate_compiled` against a stored `EvidenceSet` uses **today’s** wall clock unless a test builds its own context. `AssessmentRun` does not pin an evaluation `as_of`.

### 3.7 Compare is pairwise readiness

[`compare`](../../crates/weeping-angel-assurance/src/snapshot.rs) diffs two `FrameworkReadinessSnapshot`s: effectiveness transitions, stale ids, subjects, applicability, exceptions, digest-changed flags. `compare_runs` / `compare_lineage` only flip pack/catalog digest booleans.

There is **no** period coverage, observation-gap, intermittent-regression, or validity-event timeline.

### 3.8 continuous-assurance scheduler (characterization SHA)

At SHA `6e31bf1…` there was no scheduler crate/module, no `FreshnessPolicy` type, no cadence/retry/daemon. Collectors ran inside one-shot `assess`. **This slice still must not implement** cadence. Scheduler product now lives in [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md) / [ADR 0005](../adr/0005-continuous-assurance-scheduler.md).

### 3.9 What current tests lock (do not break)

- Envelope digest = observation + provenance (`sdd_typed_evidence_target`, ACT/EVD needles).
- `supersedes` / `collectionRunId` / artifact refs outside `DigestBody`.
- Ledger append-only + `supersede` keeps previous row.
- Population latest/supersede + stale-from-`collected_at` (`sdd_population_runtime_target`).
- Lineage run pins and pure serialize (`sdd_assessment_lineage_target`).
- Catalog families that treat `max_age` / `reviewed_at` / `valid_until` **as observation facts**, not envelope validity.

---

## 4. Desired behavior

### 4.1 Time vocabulary (normative)

Every sealed observation participates in a **validity record**. Clocks are UTC instants (`DateTime<Utc>`). Lexical export is RFC 3339.

| Field | Meaning | Source of truth |
| --- | --- | --- |
| `observed_at` | Instant the **world fact** was true (source system / attestation time) | Validity record; default `collected_at` if omitted at first assertion |
| `collected_at` | Instant the collector **sealed** the envelope | `EvidenceProvenance.collected_at` (unchanged encoding) |
| `valid_from` | Inclusive start of the assurance-usable window | Validity record; default `observed_at` |
| `valid_until` | Exclusive end of the assurance-usable window (`None` = open until superseded, revoked, or policy stale) | Validity record |
| `supersedes` | Prior **envelope digest** replaced by this observation | Existing envelope field; history kept |
| `revocation` / invalidation | Later event that withdraws usability without rewriting the envelope | Append-only validity event |
| `source_revision` | Collector/source revision of the observed system (git SHA, API ETag, config digest) — **not** the envelope digest | Validity record (optional string) |
| `artifact_digest` | Digest of the raw artifact, when present | `EvidenceArtifactRef.digest`; else omit (do not invent) |

**Half-open window:** an instant `T` is inside the record iff

```text
valid_from <= T  &&  (valid_until.is_none() || T < valid_until)
```

Exact-boundary tests are required: `T == valid_from` is in; `T == valid_until` is out.

### 4.2 Immutability: validity is a new record, never an edit

Sealed envelopes stay immutable. Forbidden: `UPDATE` of payload, rewriting `valid_until` on a stored envelope, “unseal and reseal”, or `set_valid_until`.

```text
EvidenceEnvelope          # sealed observation; DigestBody unchanged
    └─ EvidenceValidityEvent[]   # append-only; schema evidence-validity/v1
```

Suggested event kinds (camelCase JSON):

```text
asserted | superseded | revoked | invalidated
```

| Kind | Meaning |
| --- | --- |
| `asserted` | First (or replacement) usable window for this envelope digest |
| `superseded` | A newer envelope digest replaced this one (`supersedes` pointer on the new envelope) |
| `revoked` | Collector/operator withdrew the observation (fraud, bad extract, source delete) |
| `invalidated` | Window closed because a later fact proved it no longer holds, without a full replacement observation |

Event identity is **not** the envelope digest. Suggested body (names may follow crate conventions):

```text
schemaVersion = "evidence-validity/v1"
eventId                       # stable, digest-derived; no random v4
envelopeDigest                # sealed observation
kind
at                            # event time (UTC)
observedAt?
validFrom?
validUntil?
sourceRevision?
artifactDigest?
supersedesEventId?            # prior validity event, if any
reason?                       # non-compliance narrative; still not a ControlTestResult
```

`eventId` = SHA-256 hex of canonical JSON of the event **excluding** `eventId` (same IR `canonical_digest` law). Ledger persist is idempotent by `eventId`. A second write of **different** bytes for the same `eventId` is rejected (`LedgerError::Immutable` or a typed sibling). Identical bytes are a no-op.

`EvidenceLedger::supersede` continues to append a **new envelope**. It must also append a `superseded` event for the previous digest and an `asserted` event for the next (or a single documented pairing). It must not mutate the previous envelope row.

Revocation of a still-stored envelope: `record_validity_event(revoked)` only. The observation remains gettable for audit.

### 4.3 Envelope surface (additive, digest-stable)

Do **not** add `observed_at` / `valid_*` to `DigestBody`. Historical string and typed envelopes keep the same digest when resealed from the same observation+provenance.

Allow:

- optional accessors that **project** the latest asserted validity event for an envelope;
- optional builder helpers that stage an initial assertion applied at `append` time (stored as an event, not mixed into `digest`);
- serde-default optional envelope fields **only if** they remain outside `DigestBody` and default-absent JSON matches today’s payloads.

Two distinct validity windows for the **same** observation+provenance remain one envelope (same digest PK) plus two events. That is required: validity is not a second observation.

`artifact_digest` on the event copies `artifact_ref.digest` when the envelope has an artifact; it is documentary, not a new store.

### 4.4 Evaluation clock and continuous-assurance scheduler seam

```text
TimeRange { start: DateTime<Utc>, end: DateTime<Utc> }   # half-open [start, end)

AssessmentContext {
  now: DateTime<Utc>,           # evaluation clock (injected)
  max_age: Duration,            # policy freshness default
  as_of: DateTime<Utc>,         # point-in-time; default = now
  period: Option<TimeRange>,    # when set, period projection
}

FreshnessPolicy {               # seam for continuous-assurance scheduler — types only
  max_age: Duration,
  as_of: DateTime<Utc>,
  period: Option<TimeRange>,
}
```

Rules:

- `as_of` is the instant used for point-in-time selection. Live `assess` may set `now = as_of = Utc::now()`.
- Historical / target tests **must** inject a fixed clock. Facade `assess` must accept an explicit clock (`as_of` / context) so replay does not depend on wall time.
- `period` present ⇒ produce a **period result** per control (see §4.7). Point-in-time `Effectiveness` is still computed at `as_of` (convention: `as_of = period.end` unless the caller sets otherwise) and must not be labeled “continuously effective”.
- continuous-assurance scheduler later supplies cadence and “last successful run”. This slice only consumes `FreshnessPolicy` / `AssessmentContext`. No scheduler loop, retry, jitter, or daemon.

Pin the declared clock on lineage (`AssessmentRun` or the evidence/result snapshot) so two processes with the same pins + same ledger prefix reproduce the same results.

### 4.5 Deterministic selection (no temporal leakage)

Public selector (names may follow crate style):

```text
select_evidence(set_or_ledger, query: TemporalQuery) → Vec<EvidenceEnvelope>  # stable order
select_latest_as_of(group, as_of) → Option<&EvidenceEnvelope>

TemporalQuery {
  evidence_type?, subject?,
  as_of?: DateTime<Utc>,
  range?: TimeRange,
  include_revoked: bool,        # default false for evaluation; true for audit export
}
```

For **point-in-time** `as_of = T`, an envelope is a **candidate** iff all of:

1. `collected_at <= T` — collection cannot satisfy a past clock;
2. `observed_at <= T` — a future world-observation cannot satisfy a past clock;
3. `valid_from <= T` and (`valid_until` is `None` or `T < valid_until`);
4. no `revoked` / `invalidated` event with `at <= T`;
5. if a `supersedes` chain exists among candidates, the envelope is a **leaf at T** (its digest is not superseded by another **candidate** whose own assertion is valid at T).

Then:

6. Among remaining leaves for `(evidence_type, subject)`, take latest `observed_at`, then latest `collected_at`, then lexicographic `digest` (stable).
7. Digest-identical duplicates count once (`EvidenceSet` already keys by digest).

`EvidenceIndex` / `select_latest` / `first_selector` used by evaluation **must** use this candidate filter. Digest-order first-hit over the unbounded bag is forbidden for control evaluation.

Ledger:

- `within_window` gains a documented **validity** mode (or a sibling `valid_during`) that filters on the validity window, not only `collected_at`. Keep the existing `collected_at` filter as an explicit collection-time query so current callers do not silently change meaning — rename or split rather than overload.
- `latest` at an `as_of` must walk supersession + validity, not `ORDER BY collected_at DESC LIMIT 1`.

**No leakage:**

| Forbidden | Required instead |
| --- | --- |
| Envelope with `observed_at > T` or `collected_at > T` counted as passing at `T` | Excluded from candidates (not `Effective`, not “stale of the future”) |
| Envelope with `T >= valid_until` counted as passing | Expired — see §4.6 |
| Revoked-at-or-before-`T` envelope counted as passing | Excluded |
| Later leaf shadowing an older still-valid envelope via digest order | As-of leaf selection |
| Evaluating Monday with Tuesday’s envelopes because they are “in the set” | Candidate rule 1–2 |

### 4.6 Stale vs expired vs future vs missing

These are **disjoint** evaluation defects:

| Defect | Condition | Typical effectiveness |
| --- | --- | --- |
| Missing | no candidate at `T` | `InsufficientEvidence` |
| Future | observation/collection after `T` (present in the bag, not a candidate) | not used; must not become `Effective` |
| Expired | was a candidate only for `T' < valid_until <= T` | not `Effective`; prefer a distinct rationale (`expired` / outside window). Do **not** call this `StaleEvidence` if the only issue is `valid_until`. |
| Stale | candidate at `T` but `T - collected_at > max_age` (or selector `freshness` / `FreshWithin` fail) **and** the test’s freshness policy applies | `StaleEvidence` |
| Fail | candidate fresh and predicate false | `Ineffective` |

`FreshWithin` and catalog fact timestamps (`reviewed_at`, `completed_at`, …) stay **predicate** freshness. They do not replace envelope validity. An envelope can be inside `valid_*` and still fail a field-level freshness test.

Conversion of a negative duration (`collected_at > now`) must **not** be the mechanism that implements “future”. Future is excluded in selection; `is_stale` runs only on candidates.

### 4.7 Temporal control projection

#### Point-in-time

`evaluate(..., AssessmentContext { as_of: T, period: None, ... })` uses `select_latest_as_of(..., T)` and existing `TestExpr` / population arithmetic. Results remain `Effectiveness`.

`Exists` at `T` means: a **candidate** envelope exists at `T`. It does **not** mean the control was continuously effective on any interval.

#### Period

When `period = [A, B)` is set, the runtime also produces a **period projection** per control test:

```text
PeriodEffectiveness =
  continuouslyEffective
  | intermittentRegression
  | insufficientObservationCoverage
  | ineffective
  | manualReviewRequired
```

Map onto exports without collapsing into a fake `Effective`:

| Period result | When |
| --- | --- |
| `continuouslyEffective` | Every required sampling instant (or the whole interval under explicit continuous semantics) has a passing candidate; no fail; no expiry hole; observation coverage meets the test’s semantics |
| `intermittentRegression` | At least one sub-interval is failing and at least one is passing (control flipped) |
| `insufficientObservationCoverage` | Gaps where no candidate covers the interval **and** those gaps are large enough that continuous effectiveness cannot be claimed |
| `ineffective` | The period is dominated by failing observations (no passing sub-interval, or a documented fail-closed rule) |
| `manualReviewRequired` | Test/catalog semantics are hybrid/manual, or coverage/validity is ambiguous in a way automation must not paper over |

Do **not** add these five as silent aliases of `Effectiveness::Effective`. Persist them on a nested object (e.g. `ControlTestResult.period` / `TemporalProjection`) so existing point-in-time ACT needles stay valid.

#### Do not infer continuity from one observation

Default **temporal semantics** (fail-closed):

```text
TemporalSemantics = instant | interval | continuousUntilSuperseded
```

| Semantics | Who may set it | Period rule |
| --- | --- | --- |
| `instant` | **default** for `Exists` / one-shot facts | A single observation covers **its instant** (or a documented sampling grain). It does **not** fill `[A, B)`. One fresh `Exists` ⇒ point-in-time `Effective` and period `insufficientObservationCoverage` unless another rule applies. |
| `interval` | explicit `valid_from`/`valid_until` on the asserted event | Covers exactly that window. Gaps between intervals are insufficient coverage, not passes. |
| `continuousUntilSuperseded` | only when the **evidence type or test** declares it (catalog/test expression / selector flag). This slice may add an optional field on `EvidenceSelector` or a documented test-expr wrapper; it must **not** rewrite catalog TOML in this commit set. | One assertion covers `[valid_from, next_supersede_or_revoke_or_valid_until)`. |

Unless semantics are `continuousUntilSuperseded` or an explicit interval covers the gap, **sparse observations cannot yield `continuouslyEffective`**.

Sampling for period evaluation must be deterministic: either (a) every validity-event boundary plus `A` and `B`, or (b) a documented fixed grain. Same inputs ⇒ same partition. Event-boundary sampling is the default (no hidden daily cron).

Overlapping windows for the same `(type, subject)`: apply §4.5 leaf selection **per instant** in the partition. Overlap is not “both pass”; the as-of leaf wins.

### 4.8 Timeline / diff primitives

Usable by readiness and audit exports (library first; no charts).

```text
EvidenceTimeline {
  subject, evidenceType,
  intervals: [{ envelopeDigest, eventId, validFrom, validUntil?, kind, observedAt, collectedAt }]
}

TemporalDiff {
  # existing SnapshotDiff classes remain valid for pairwise snapshots
  plus:
  observationGaps[],            # [start, end) with no candidate
  expiredAt[],                  # envelopes that left the window
  revoked[],                    # revocation events in range
  superseded[],                 # supersede advances in range
  intermittentControls[],       # control ids with PeriodEffectiveness::intermittentRegression
  coverageInsufficient[],       # control ids with insufficientObservationCoverage
}
```

- `project_timeline(set_or_ledger, range, selectors?)` → ordered intervals (sort `validFrom`, then digest).
- `diff_period(range, previous_projection?, next_projection?)` or `compare_temporal(...)` fills `TemporalDiff`.
- Existing `compare(readiness, readiness)` **stays pairwise**. Do not reuse `control_became_effective` to mean “continuously effective in Q2”.

Audit export may include revoked envelopes when `include_revoked = true`; evaluation defaults to excluding them.

### 4.9 Reproducible historical assessment

Given:

- pinned `AssessmentRun` (definition, catalog, pack, evidence-snapshot, result digest, **evaluation clock**);
- the ledger prefix of envelopes + validity events whose `collected_at` / `at` are `<= as_of` (or within the declared period and visible under §4.5);

then `evaluate` / `replay_assessment` returns the **same semantic results** (result digest law unchanged: still excludes wall-clock `duration` / `evaluatedAt`).

Adding an envelope with `collected_at > as_of` or a validity event with `at > as_of` after the fact must **not** change the replay at that `as_of`.

### 4.10 Collectors and catalog (this slice)

- Do **not** rewrite GitHub / local / manual collector semantics or catalog TOML.
- Collectors may keep setting only `collected_at`. The first `asserted` event defaults `observed_at = collected_at`, `valid_from = observed_at`, `valid_until = None`.
- Tests/fixtures **may** emit explicit validity events to prove overlap, revoke, and expiry.
- Governance/IAM fact fields named `valid_until` / `reviewed_at` remain **observation facts**. Envelope validity is a separate layer; a fact `valid_until` does not automatically close the envelope window unless a later adapter copies it into an event (out of scope unless a target test requires a documented helper).

### 4.11 Public contract / docs (done)

1. ADR [`docs/adr/0003-evidence-validity-temporal-assurance.md`](../adr/0003-evidence-validity-temporal-assurance.md) is **Accepted**.
2. [`docs/specs/assurance-runtime.md`](assurance-runtime.md) records envelope clocks, validity events, as-of/`within_window` split, period results, timeline/diff, scheduler seam.
3. This file is in `CANONICAL_SPECS`.
4. Status is **Implemented**.

---

## 5. Dual-suite protocol (HARD SDD)

`tests/contracts` is **not** auto-discovered. Register in root [`Cargo.toml`](../../Cargo.toml) **in the same commit as the `.rs` files** (do not invent `tests/sdd/`):

```toml
[[test]]
name = "sdd_temporal_assurance_baseline"
path = "tests/contracts/temporal_assurance.baseline.rs"

[[test]]
name = "sdd_temporal_assurance_target"
path = "tests/contracts/temporal_assurance.target.rs"
```

| Gate | Suite | Expected |
| --- | --- | --- |
| Spec | this file | written **before** product feature code |
| Baseline on CURRENT HEAD | `sdd_temporal_assurance_baseline` | **GREEN** — characterizes §3 |
| Target on CURRENT HEAD | `sdd_temporal_assurance_target` | **RED** for **missing temporal contract** (as-of, validity events, period results, no leakage) — not compile/harness noise |
| Implement | evidence + control-test + facade + ledger events | — |
| Target after | same target suite | **GREEN** |
| Baseline after | baseline | skip-supersede (`#[ignore = "superseded by sdd_temporal_assurance_target"]`) |
| Neighbors | `sdd_typed_evidence_target`, `sdd_assessment_lineage_target`, `sdd_population_runtime_target`, `sdd_assurance_runtime_target` | stay GREEN |
| Workspace | `cargo test --workspace --features demo` | GREEN after implement |

Protocol: write tests first (target encodes original found cases) → **RED** → fix → **GREEN**.

### 5.1 Baseline suite contents (GREEN on CURRENT)

Assert **today’s** behavior, titled so they characterize HEAD rather than the wish list:

- Envelope type / serde has `provenance.collected_at` and does **not** expose `observed_at` / `valid_from` / `valid_until` / revocation / `source_revision` as envelope fields.
- `DigestBody` / `seal` hashes observation + provenance only; `supersedes`, `collectionRunId`, `artifactRef` stay outside the digest.
- `EvidenceLedger::within_window` filters `collected_at`; `latest` is `ORDER BY collected_at DESC`.
- `select_latest` walks `supersedes` then `collected_at` + digest; no as-of filter.
- `is_stale` / `envelope_stale` = `now - collected_at > max_age` (or selector freshness).
- `assess` builds `AssessmentContext { now: Utc::now(), max_age: 24h }`.
- `TestExpr::Exists` → `Effective` from a single non-stale in-set observation.
- No public as-of / period selector type; a future-dated leaf can be selected; a past `now` still sees every envelope in the set.
- `SnapshotDiff` / `compare` is pairwise readiness (no period coverage / observation-gap fields).

### 5.2 Target suite contents (RED on CURRENT, GREEN after)

One regression test per comment-style subject, encoding the **original found case**. Suggested IDs `TMP-001`… (titles are normative for review):

| ID | Title (use as test name subject) | Found case on HEAD → required |
| --- | --- | --- |
| TMP-001 | overlapping evidence | Two overlapping windows for one `(type, subject)`; as-of leaf is deterministic; evaluation never double-counts |
| TMP-002 | supersession | Older fail + newer pass with `supersedes`; at `T` after the new assertion only the leaf is used; previous row unchanged |
| TMP-003 | revocation | Revoke event at `T_r`; as-of `T < T_r` still uses the envelope; as-of `T >= T_r` does not; payload still `get`able |
| TMP-004 | clock boundaries | `T == valid_from` in; `T == valid_until` out; inclusive/exclusive half-open |
| TMP-005 | stale evidence | Candidate inside `valid_*` but `T - collected_at > max_age` → `StaleEvidence`, not missing, not expired |
| TMP-006 | future observation | Envelope with `observed_at` or `collected_at` after `as_of` cannot make a past assessment `Effective`; must not shadow an older candidate |
| TMP-007 | intermittent control failure | Period contains pass then fail (or fail then pass) → `intermittentRegression`, not `continuouslyEffective` |
| TMP-008 | sparse observations | Single `Exists` / instant observation in a wide period → `insufficientObservationCoverage` unless semantics explicitly allow continuity |
| TMP-009 | reproducible historical assessment | Same pins + same ledger prefix + same `as_of` ⇒ same result digest; appending a later envelope does not change the historical result |
| TMP-010 | expired evidence | `valid_until <= as_of` cannot satisfy the control (not `Effective`) |
| TMP-011 | sealed envelope untouched | Validity change is a new event/envelope; previous digest/payload bytes identical |
| TMP-012 | timeline/diff primitives | `project_timeline` / temporal diff reports gaps, revoke, supersede, intermittent ids |

Target failures on this HEAD must mention the **missing contract** (no `valid_from`, no as-of selector, Exists-implies-effective, compare-is-pairwise, etc.), not a missing test harness.

---

## 6. Acceptance criteria

1. Dual-suite `sdd_temporal_assurance_baseline` + `sdd_temporal_assurance_target` is registered in root `Cargo.toml` in the **same commit** as `tests/contracts/temporal_assurance.{baseline,target}.rs`.
2. Baseline is GREEN on characterization SHA / current HEAD **before** product feature code, locking §3.
3. Target is RED on that HEAD for the missing temporal contract (TMP-001–012), not compile noise; after implement, target GREEN and baseline skip-superseded.
4. Formal temporal fields exist as specified: `observed_at`, `collected_at`, `valid_from`, `valid_until`, `supersedes`, revocation/invalidation events, `source_revision`, artifact digest.
5. Sealed envelopes are immutable; validity changes are new records/events; `DigestBody` remains observation + provenance.
6. Selection at `as_of` / range is deterministic (sort law §4.5) and excludes future, expired, and revoked-at-T evidence.
7. Future evidence cannot satisfy a past assessment; expired evidence cannot satisfy a control outside its window.
8. Point-in-time evaluation and period projection both exist; period results distinguish continuously effective, intermittent regression, insufficient observation coverage, ineffective, and manual review.
9. Continuous effectiveness is **not** inferred from one observation unless evidence/test semantics explicitly allow `continuousUntilSuperseded` or an explicit interval covers the gap.
10. Timeline/diff primitives are callable from readiness/audit library paths (no UI).
11. A finalized assessment replayed at its pinned clock uses only evidence valid at that time / period (TMP-009).
12. This slice does not implement the scheduler; `FreshnessPolicy` / context is the handoff. Cadence lives in [ADR 0005](../adr/0005-continuous-assurance-scheduler.md).
13. SQLite `EvidenceLedger` is reused; no new long-term database; no UI charts.
14. `sdd_typed_evidence_target`, `sdd_assessment_lineage_target`, `sdd_population_runtime_target`, `sdd_assurance_runtime_target` stay GREEN; catalog TOML, GitHub collector, and ISO remap are not rewritten.
15. Draft ADR is finalized and `docs/specs/assurance-runtime.md` is updated so the public contract does not lie.

---

## 7. Out of scope

- UI charts, dashboards, or calendar visualizations.
- New long-term database backend (PostgreSQL, remote ledger-as-a-service, object-store product).
- continuous-assurance scheduler continuous scheduler (cadence, retry, backoff, jitter, daemon, `isms run` product).
- Rewriting canonical catalog TOML, ISO pack mappings, or the GitHub collector.
- Changing sealed `DigestBody` law or requiring historical envelopes to be resealed.
- Inferring compliance or certification from temporal coverage.
- Multi-tenant SaaS clocks / NTP service.
- Copying every catalog fact `valid_until` into envelope events automatically.

---

## 8. Risks

- **Digest breakage:** putting `valid_*` inside `DigestBody` would churn every fixture digest and fail typed-evidence / ACT suites. Mitigation: events outside the sealed core.
- **Silent meaning change of `within_window` / `latest`:** callers today mean `collected_at`. Split APIs instead of overloading.
- **Exists → continuous:** catalog tests still use `Exists` / freshness facts. Period projection must not relabel those as `continuouslyEffective`.
- **first_selector vs select_latest split:** leaving Exists on digest-order first-hit reintroduces leakage even if the index is as-of-correct.
- **Future-as-stale:** today’s `to_std()` failure treats future `collected_at` as stale and can shadow a good older envelope.
- **continuous-assurance scheduler collision:** implementing cadence here would fork the scheduler. Keep types only.
- **Fact `valid_until` vs envelope `valid_until`:** governance/IAM facts already use the name. Document the two layers; do not auto-merge.
- **Clock injection forgotten on facade:** if `assess` keeps `Utc::now()` only, TMP-009 cannot hold in production replay.

---

## 9. Implementation notes

Landed. Dual-suite registered; target GREEN; baseline skip-superseded. Product modules stayed in existing crates (no new crate, no new DB).

Generated SDD traces belong in [`.sdd/`](../../.sdd/) (gitignored). [`docs/sdd`](../sdd/) is the report_dir stub, not a second SSOT.
