# ADR 0011 — Temporal evidence, lineage, persistence, and SoA integrity

<!-- weeping-angel-adr-meta
id = "0011"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_temporal_lineage_evidence_soa_target` GREEN; baseline skip-superseded |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The *operational* reading that `latest` ≈ `latest_as_of` ≈ `current` ≈ live wall-clock, that `replay_assessment` may `Ok(reconstruct)` without pin checks, that collector `Err` is an empty evidence world, that completed `collection_runs` may `INSERT OR REPLACE`, and that CLI `assurance soa latest` may call live `project_soa`. Does **not** supercede envelope `DigestBody`, `evidence-validity/v1`, lineage snapshot schema, operational-SoA row law, or `LedgerError` / `AssuranceError` variant sets used by neighbor exhaustive matches. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0003 assessment lineage](0003-assessment-lineage.md), [ADR 0003 temporal assurance](0003-temporal-assurance.md), [ADR 0003 evidence-validity temporal](0003-evidence-validity-temporal-assurance.md), [ADR 0003 operational SoA](0003-operational-soa.md), [ADR 0003 typed evidence](0003-typed-evidence-canonical-serialization.md), [ADR 0005 scheduler](0005-continuous-assurance-scheduler.md), [ADR 0004 docs](0004-documentation-architecture.md), [ADR 0010 architecture-as-law](0010-architecture-as-law.md) |
| Spec | [`docs/specs/temporal-lineage-evidence-soa.md`](../specs/temporal-lineage-evidence-soa.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `0015f6395e7ead042e3cfd3066fefde3d39aa36b` |
| Tests | `sdd_temporal_lineage_evidence_soa_target` GREEN; `sdd_temporal_lineage_evidence_soa_baseline` skip-superseded. Neighbor `sdd_temporal_assurance_target`, `sdd_evidence_validity_temporal_assurance_target`, `sdd_assessment_lineage_target`, `sdd_operational_soa_target`, `sdd_continuous_assurance_scheduler_target` stay GREEN. |

> Cite this decision by **path** (`docs/adr/0011-temporal-lineage-evidence-soa-integrity.md`). Concurrent cleanup also uses `0011-*` filenames for Prompt 1/2 ADRs; they are different decisions. Do not invent `0003-*` forks of Prompt 14 ADRs.

## Context

Prompt 14 and assessment-lineage shipped validity windows, `latest_as_of`, persistable `AssessmentRun`, and `reconstruct`. Operational SoA shipped explainable rows and `project_soa_from_snapshot`. On characterization SHA `0015f63…` the **trust boundary** was still open:

1. `EvidenceLedger::latest` is `collected_at DESC` with no validity filter; `latest_as_of` is validity-filtered; `current` / `valid_at` / `as_of` methods did not exist.
2. `AssessmentRun` serialize always emitted JSON `asOf` from `started_at`.
3. `replay_assessment` was `Ok(reconstruct(bundle))` with no pin / schema / identity checks.
4. One-shot `assess()` mapped collector `Err` to `Vec::new()` and evaluated an empty bag.
5. `record_collection_run` was `INSERT OR REPLACE`. Load mapped malformed JSON to `LedgerError::Serialize`.
6. CLI `assurance soa` with empty or `"latest"` called live `project_soa`.

Historical assessments could be silently rewritten by current pack, catalog, ledger-latest, or live SoA projection.

Concurrent Prompt 1 owns xtask / architecture guards and freezes `select_latest_as_of` in `weeping-angel-control-test`. This decision exposes typed APIs for Guards 09–12; it does not move that function or edit `xtask/**`. Neighbor exhaustive matches on `LedgerError` and `AssuranceError` stay exhaustive; new integrity names are **named types**, not new enum arms on those two enums.

## Decision

### 1. Four clocks are four APIs

`latest`, `current`, `valid-at`, and `as-of` are not aliases.

| Term | API | Question | Validity |
| --- | --- | --- | --- |
| **latest** | `EvidenceLedger::latest` | Most recently **recorded** envelope | None. Expired / revoked / future rows may win. |
| **current** | `EvidenceLedger::current` | Valid evaluation leaf at **live wall-clock** | `as_of(type, Utc::now())`. Not `latest`. Not `AssessmentContext.now`. |
| **valid-at** | `EvidenceLedger::valid_at` | Membership **set** in force at instant `T` | Half-open window via `project_validity`; digest-sorted. |
| **as-of** | `EvidenceLedger::as_of` | Evaluation **leaf** at pinned clock `T` | Candidate filter then latest-among-valid-leaves (`select_leaf_as_of`). |

`latest_as_of` remains a documented compatibility **alias of `as_of`**, never of `latest`.

`select_latest_as_of` stays in `weeping-angel-control-test::temporal`. `AssessmentContext::as_of()` remains the injected assessment clock (`now` via `pinned_assessment_clock`). That injected clock is **not** ledger `current()`. Live `assess` still builds `AssessmentContext { now: Utc::now(), max_age: 24h }`.

### 2. Pinned `asOf` is independent of `startedAt`

`AssessmentRun` JSON `asOf` serializes the `as_of` field. Live `assess` may default `as_of = started_at`. Replay and historical SoA use the persisted pin. CamelCase field set stays complete (`canonicalCatalogDigest`, `catalogDigest`, `applicabilitySnapshotId`).

### 3. Validity history is append-only

Sealed envelopes are never updated in place. `append` is `INSERT OR IGNORE` by digest and commits envelope + `asserted` event in one SQLite transaction. `supersede` appends a new envelope plus a `superseded` event. `record_validity_event` is idempotent on identical bytes and `LedgerError::Immutable` on same `eventId` / different bytes.

### 4. Replay fail-closed

`replay_assessment` calls `verify_replay_bundle` then `reconstruct`. It does **not** load current pack/catalog/evidence files to fill gaps. Checks:

```text
definition / evidence / applicability / catalog / pack pin equalities
resultDigest == assessment_result_digest(results)
asOf present and non-empty
lineage snapshot schema == weeping-angel/assessment-lineage/v1
envelope digest list vs evidence snapshot identity
SoA pack pin vs run pack pin when SoA pin is present
```

Typed names live on `ReplayFailure`: `MissingPinnedMaterial`, `IncompleteLineage`, `InconsistentLineage`, `CorruptPersistence`, `IncompatibleSchema`. `From<ReplayFailure>` maps into existing `AssuranceError::UnknownPack` so neighbor exhaustive matches stay exhaustive. `reconstruct` remains a clone helper for already-verified bundles.

Empty / `"unpinned"` pins fail closed. Digest mismatch uses existing `detect_digest_mismatch`.

### 5. Collection failure is not erasure

`CollectionOutcome` (`weeping-angel-evidence`): `NoNewObservation` | `KnownAbsent` | `CollectionFailed` | `EvidenceNoLongerValid`.

A failed collector run does not delete envelopes and does not write revoke/invalidate events. One-shot `assess` on `Err` (or successful empty collect) evaluates `prior_valid_envelopes(Utc::now())` filtered by assessment scope — process-local remembered envelopes from prior `append` / load in this process. `assess` still does not open a SQLite ledger. `EvidenceNoLongerValid` is only an explicit validity event, never implied by collector `Err`.

`record_collection_run`: identical payload is a no-op; a **completed** run with different bytes is `LedgerError::Immutable`; in-flight (non-completed) rows may still update.

### 6. Persistence fail-closed

Named integrity types: `Corrupt`, `IncompatibleSchema`, enum `PersistenceIntegrity`. `From<PersistenceIntegrity>` maps onto `LedgerError::Path` with Display `corrupt: …` / `incompatible schema: found …, expected …` so HEAD `LedgerError` matches remain exhaustive. Guard 12 SSOT is the **type names**, not new `LedgerError` arms.

- Envelope `get` / decode: malformed JSON or digest/key mismatch → `Corrupt` (via `Path`).
- Unknown / mismatched `schemaVersion` → `IncompatibleSchema` (via `Path`).
- Lineage persist validates JSON / known `weeping-angel/*` or `evidence/v1` schema before `persist_immutable`.

### 7. Historical SoA is bound to a reconstructed assessment

CLI `assurance soa` with empty / `"latest"` / a named id loads a pinned assessment from `assurance-ledger.sqlite`, runs `replay_assessment`, then `project_soa_from_snapshot`. Missing ledger, missing run, or replay failure **bails** — it must not call live `project_soa` as history. Live `project_soa(framework, version)` remains a current-pack convenience (scheduler `run_project` still uses it for tick projection; that path is not historical reconstruction). Never infer certification.

### 8. Period effectiveness stays conservative

Default `TemporalSemantics::Instant`. One positive Instant observation is `InsufficientObservationCoverage` over a non-empty period, not `ContinuouslyEffective`.

## Consequences

- Callers that used `latest` for current posture must switch to `current` / `as_of`.
- Partial `LineageBundle`s that previously replayed as `Ok` become `AssuranceError::UnknownPack` wrapping `ReplayFailure`.
- Prompt 1 architectural-cleanup characterization that asserts absence of `pub fn current(` goes RED on this API; Prompt 1 rewires Guard 11. This increment does not edit `xtask/**`.
- Neighbor dual-suites stay GREEN via additive APIs, serde defaults, and mapping new integrity types onto existing error enums — not by weakening leakage rules.
- One-shot `assess` reattach is process-local; a fresh process without prior `append` in that process still sees an empty bag unless the caller supplies envelopes. Scheduler tick reattach remains ledger-backed ([ADR 0005](0005-continuous-assurance-scheduler.md)).

## Non-goals

Moving `select_latest_as_of`; catalog/framework/readiness redesign; xtask / `architecture.toml` / debt register; Prompt 4 hygiene; new crates `weeping-angel-catalog` / `weeping-angel-assurance-cli`; UI; certification claims; new databases; making scheduler tick SoA historical; adding `LedgerError::{Corrupt,IncompatibleSchema}` or `AssuranceError` replay arms (would break neighbor exhaustive matches).

## Related

- Increment spec: [`docs/specs/temporal-lineage-evidence-soa.md`](../specs/temporal-lineage-evidence-soa.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Extended human specs: [`temporal-assurance.md`](../specs/temporal-assurance.md), [`evidence-validity-temporal-assurance.md`](../specs/evidence-validity-temporal-assurance.md), [`assessment-lineage.md`](../specs/assessment-lineage.md), [`operational-soa.md`](../specs/operational-soa.md), [`typed-evidence.md`](../specs/typed-evidence.md), [`continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md)
- Dual-suite: `tests/contracts/temporal_lineage_evidence_soa.{baseline,target}.rs`
