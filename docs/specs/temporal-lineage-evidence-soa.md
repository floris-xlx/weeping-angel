# SDD: Temporal Evidence, Lineage, Persistence, and SoA Integrity

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_temporal_lineage_evidence_soa_target` GREEN; baseline skip-superseded |
| Program | Concurrent cleanup Prompt 3 — close remaining trust-boundary debt after Prompt 14 / lineage / SoA / scheduler landed |
| Slice | Distinct `current` / `latest` / `valid-at` / `as-of` APIs; pinned-clock historical selection; append-only validity; fail-closed replay; collection-vs-erasure; persistence fail-closed; historical SoA bound to pinned assessment |
| Dual-suite (register at implement, **same commit** as the `.rs` files) | `sdd_temporal_lineage_evidence_soa_baseline` · `sdd_temporal_lineage_evidence_soa_target` (`tests/contracts/temporal_lineage_evidence_soa.{baseline,target}.rs`) |
| ADR | Accepted [`docs/adr/0011-temporal-lineage-evidence-soa-integrity.md`](../adr/0011-temporal-lineage-evidence-soa-integrity.md) |
| Extends (do not fork) | [`temporal-assurance.md`](temporal-assurance.md), [`evidence-validity-temporal-assurance.md`](evidence-validity-temporal-assurance.md), [`assessment-lineage.md`](assessment-lineage.md), [`operational-soa.md`](operational-soa.md), [`typed-evidence.md`](typed-evidence.md), [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md) |
| Public contract | [`assurance-runtime.md`](assurance-runtime.md) |
| Protocol report (generated, gitignored) | [`.sdd/runs/sdd-temporal-lineage-evidence-soa.md`](../../.sdd/runs/sdd-temporal-lineage-evidence-soa.md) |
| Layout | [ADR 0004](../adr/0004-documentation-architecture.md) — this file is the increment SSOT; `docs/sdd/` is a stub; traces stay under `.sdd/runs/` and `.sdd/artifacts/` |
| Spine / ISO law | [`assurance-runtime-spine.md`](assurance-runtime-spine.md), [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Architecture freeze (Prompt 1) | [`architectural-cleanup-program.md`](architectural-cleanup-program.md), [ADR 0010](../adr/0010-architecture-as-law.md) — do **not** move `select_latest_as_of`; expose typed APIs so Guard 09–12 can wire later |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `0015f6395e7ead042e3cfd3066fefde3d39aa36b` |
| Evidence schema | remains `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Validity-event schema | remains `evidence-validity/v1` (`EVIDENCE_VALIDITY_SCHEMA`) |
| Lineage schema | remains `weeping-angel/assessment-lineage/v1` (`LINEAGE_SNAPSHOT_SCHEMA`) |
| Workspace verify (after implement) | `cargo test --test sdd_temporal_lineage_evidence_soa_baseline`; `cargo test --test sdd_temporal_lineage_evidence_soa_target`; `cargo test --test sdd_temporal_assurance_target --test sdd_evidence_validity_temporal_assurance_target --test sdd_assessment_lineage_target --test sdd_operational_soa_target --test sdd_continuous_assurance_scheduler_target`; `cargo test -p weeping-angel-evidence`; `cargo test -p weeping-angel-assurance`; `cargo fmt --all -- --check`; `cargo check --workspace` |

This document is the durable human SSOT for **remaining** temporal / lineage / evidence-persistence / SoA integrity debt. Neighbor Prompt 14 dual-suites (`sdd_temporal_assurance_*`, `sdd_evidence_validity_temporal_assurance_*`), lineage (`sdd_assessment_lineage_*`), operational SoA (`sdd_operational_soa_*`), and scheduler (`sdd_continuous_assurance_scheduler_*`) are already **GREEN / skip-superseded**. Those suites remain neighbor law. **Do not reuse those superseded baselines as this increment’s GREEN characterization.**

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Time is an assurance dimension. A historical assessment is a **pinned execution artifact**. Current mutable pack, catalog, ledger “latest row”, or live `project_soa` must never silently rewrite what that artifact said.

## Implemented contract (what shipped)

Target `sdd_temporal_lineage_evidence_soa_target` GREEN. Baseline skip-superseded. Decision: [ADR 0011](../adr/0011-temporal-lineage-evidence-soa-integrity.md).

| Concern | Home |
| --- | --- |
| `latest` / `current` / `valid_at` / `as_of` | `EvidenceLedger` (`current` = `as_of(type, Utc::now())`; `latest_as_of` alias of `as_of`) |
| Membership vs leaf | `valid_at` = digest-sorted set; `as_of` = `select_leaf_as_of` among `project_validity` candidates |
| `select_latest_as_of` | **Unmoved** `weeping-angel-control-test::temporal` |
| Injected assessment clock | `AssessmentContext::as_of()` = `now` (`pinned_assessment_clock`); **not** ledger `current()` |
| JSON `asOf` | `AssessmentRun` serialize writes `self.as_of` (live `assess` may default it to `started_at`) |
| Fail-closed replay | `replay_assessment` → `verify_replay_bundle` then `reconstruct`. `ReplayFailure` maps to `AssuranceError::UnknownPack` |
| Collection vs erasure | `CollectionOutcome::{NoNewObservation,KnownAbsent,CollectionFailed,EvidenceNoLongerValid}`; `assess` on `Err`/empty uses `prior_valid_envelopes` (process-local) |
| Collection-run persist | Identical bytes no-op; completed different bytes → `LedgerError::Immutable` |
| Persistence integrity | `Corrupt` / `IncompatibleSchema` / `PersistenceIntegrity` → `LedgerError::Path` Display |
| Historical SoA | CLI `assurance soa` empty/`latest`/named: `replay_assessment` + `project_soa_from_snapshot`; never live `project_soa` as history |

`latest`, `current`, `valid_at`, and `as_of` disagree when the newest row is expired or revoked at `now`. Historical `as_of(t)` never includes envelopes with `collected_at`/`observed_at` > `t`, `valid_until <= t`, or revoked/invalidated at `t`. Period Instant conservatism is unchanged.

---

## 0. Collision fence (concurrent prompts)

Prompts 1, 2, and 4 run concurrently. This increment may edit only the surfaces it owns.

| Allowed | Home |
| --- | --- |
| Temporal / lineage / assessment-history modules | `crates/weeping-angel-assurance/**` especially `temporal.rs`, `lineage.rs`, replay / result-history, and directly related modules |
| Evidence persistence + validity | `crates/weeping-angel-evidence/**` |
| SoA-specific assurance + narrowly related CLI/service | `crates/weeping-angel-assurance/src/soa.rs`, `src/assurance_soa.rs` (and the smallest CLI dispatch already owned by SoA) |
| Increment dual-suite + required proof | `tests/contracts/temporal_lineage_evidence_soa.{baseline,target}.rs` and **only** additional temporal / evidence-validity / lineage / SoA / persistence tests needed to prove this increment |
| Spec / ADR reflections | this file; pointer sections in the six extended specs; accepted ADR 0011 |

| Forbidden | Owner |
| --- | --- |
| `xtask/**`, `architecture/**`, `docs/debt/register.toml` | Prompt 1 |
| Canonical catalog / framework parse-digest / readiness projection redesign | Prompt 2 |
| Broad ignored-test cleanup, panic-budget, schema dedup, README/artifact hygiene, `tests/contracts/documentation_layout.rs` | Prompt 4 |
| Repository-integrity metadata, broad documentation indexes, `tests/sdd/` | Prompt 1 / 4 / ADR 0004 |
| Inventing crates `weeping-angel-catalog` or `weeping-angel-assurance-cli` | architecture law |
| Moving `weeping-angel-control-test::temporal::select_latest_as_of` | `architecture.toml` freeze for Prompt 1 / Guard 09 |

Consume `select_latest_as_of` and framework pack pins through their stable interfaces. Do not redesign those subsystems.

Neighbor targets that **must stay GREEN**: `sdd_temporal_assurance_target`, `sdd_evidence_validity_temporal_assurance_target`, `sdd_assessment_lineage_target`, `sdd_operational_soa_target`, `sdd_continuous_assurance_scheduler_target`, plus `sdd_typed_evidence_target` and `sdd_assurance_runtime_target`. Do not weaken temporal leakage rules to preserve legacy fixtures. If a legacy test encodes temporal leakage or mutable-history behavior, replace it on **this prompt’s owned test surface** with a regression for the correct invariant. No new `#[ignore]`, broad allowlists, or new debt unless unavoidable, narrowly scoped, owned, expiring, and justified.

---

## 1. Problem / user-visible goal

Prompt 14 shipped validity windows, `latest_as_of`, and period projection. Lineage shipped persistable runs and `reconstruct`. Operational SoA shipped explainable rows and pinned snapshots. Those increments closed *presence* of the types. They did **not** close the trust boundary:

- Callers can still treat **latest recorded** evidence as **currently valid**.
- Historical assessment JSON still emits `asOf` from `startedAt`, so the evaluation clock is not independently pinned.
- `replay_assessment` is `Ok(reconstruct(bundle))` and never fail-closes on missing pins, digest mismatch, or incomplete lineage.
- `assess()` on collector `Err` evaluates an empty bag — a failed collection looks like “no evidence.”
- `record_collection_run` is `INSERT OR REPLACE`.
- The ledger has no typed schema / corruption / incompatible-version errors.
- CLI `assurance soa latest` (and empty assessment) calls live `project_soa`, which reloads today’s pack.

A reviewer still cannot trust that last quarter’s assessment, replay, or SoA is the same object it was when sealed.

**User-visible goal:** historical assessments remain reproducible and are never silently rewritten by current state. Evidence selection at a pinned clock cannot see future, expired-before, or revoked-before envelopes. Replay and historical SoA fail closed when pinned material is missing or inconsistent. Collection failure cannot erase or implicitly invalidate previously valid ledger evidence.

Definition of done: *`current`, `latest`, `valid-at`, and `as-of` have distinct tested APIs; replay and historical SoA cannot substitute live mutable state for missing pins; persistence fails closed on corrupt or incompatible bytes.*

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `0015f6395e7ead042e3cfd3066fefde3d39aa36b`.

| Surface | Location | Rule for this increment |
| --- | --- | --- |
| `EvidenceLedger::latest` | `ledger.rs` | Keep as **record-order** API (`collected_at DESC`). Must not become the current-assessment path. |
| `EvidenceLedger::latest_as_of` | `ledger.rs` | Today is validity-filtered leaf selection. After implement it is the **as-of evaluation leaf**, not “latest”. Prefer a new `as_of` name; `latest_as_of` may remain as a documented compatibility alias of **as-of**, never of **latest**. |
| `select_latest_as_of` | `weeping-angel-control-test::temporal` | **Consume. Do not move.** Prompt 1 / `architecture.toml` freeze. |
| `AssessmentContext::as_of()` | control-test | Today returns `now`. Must become a distinct pinned clock field or a typed wrapper so `now` (live) ≠ `as_of` (pinned assessment). Additive field with serde default is allowed if needed; do not silently keep the alias. |
| `AssessmentRun` serialize | `snapshot.rs` | Today always writes JSON `asOf` from `started_at`. Must persist the run’s `as_of` field. `startedAt` remains wall-clock start. |
| `replay_assessment` / `reconstruct` | `lineage.rs` | `reconstruct` may stay a pure clone helper. `replay_assessment` must **verify** then reconstruct, or return a typed failure. |
| `AssuranceError` | `lib.rs` | Grow typed replay / pin failures as `ReplayFailure` mapped into `AssuranceError::UnknownPack` (neighbor exhaustive matches stay exhaustive). Do not collapse to `Compile` / string `Path`. |
| `LedgerError` | `ledger.rs` | Grow typed `Corrupt`, `IncompatibleSchema` **names** (`PersistenceIntegrity`). Neighbor exhaustive matches keep `LedgerError` arms unchanged: those types map onto `LedgerError::Path` Display. Do not overload `Serialize` for those cases. |
| `record_collection_run` | `ledger.rs` | Stop silent `INSERT OR REPLACE` of a completed payload. Same immutability family as assessment-run persist. |
| `assess()` collector `Err` | `lib.rs` | Must not replace a previously valid evidence bag with `Vec::new()` as if the world were empty. |
| `project_soa` | `soa.rs` | Live convenience over **current** pack + empty graph. Must remain **unusable** as historical reconstruction. |
| `project_soa_from_snapshot` | `soa.rs` | Historical reconstruction from a pinned snapshot only. Historical CLI / API must use this (or replay of the bound assessment), never live `project_soa`. |
| CLI `assurance soa` | `src/assurance_soa.rs` | `latest` / empty must not mean “reload current pack.” Historical generation fail-closes if the selected assessment cannot be reconstructed exactly. |
| Scheduler collect | `scheduler.rs` | Already appends only on `Ok` and records a failed `CollectionRun` without deleting envelopes. Keep that. Tighten one-shot `assess()` and collection-run persist. |
| `PeriodEffectiveness` | control-test `temporal.rs` | Stay conservative. Instant / single positive observation must not become continuous effectiveness. Missing intervals and unknown population stay explicit. |
| Envelope `DigestBody` | evidence crate | Unchanged: `{ observation, provenance }`. Validity remains append-only events. |
| Catalog / framework loaders | Prompt 2 | Consume pins (`frameworkPackDigest`, `canonicalCatalogDigest`). Do not redesign parse/digest. |
| xtask guards 09–12 | Prompt 1 | Expose typed APIs + stable persisted metadata. Do not edit `xtask/**` or `architecture/**`. |

ISO Phase 7 clocks, `evidence-validity/v1`, and lineage snapshot types **stay**. This increment tightens the API boundary and fail-closed reconstruction; it does not invent a second validity model.

---

## 3. Current behavior (baseline — characterization of SHA `0015f63…`)

Characterized against `0015f6395e7ead042e3cfd3066fefde3d39aa36b` (working tree of the evidence ledger, assurance facade, lineage replay, SoA CLI, and snapshot serialize). Encode this as `tests/contracts/temporal_lineage_evidence_soa.baseline.rs`. After implement the baseline is skip-superseded (`#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]`). Do not treat this section as live product law.

Do **not** encode Prompt 14 “no validity events / no as-of types” absences. Those are already false. This baseline characterizes **remaining aliases and fail-open reconstruction**.

### 3.1 `latest` is collected_at-only; `latest_as_of` is validity-filtered

[`crates/weeping-angel-evidence/src/ledger.rs`](../../crates/weeping-angel-evidence/src/ledger.rs):

| API | Behavior on this HEAD |
| --- | --- |
| `latest(type)` | `ORDER BY collected_at DESC LIMIT 1`. No supersede walk. No validity window. No revoke filter. A future-dated or revoked envelope wins if its `collected_at` is max. |
| `latest_as_of(type, t)` | Filters with `project_validity`, then leaf-by-observed/collected/digest. This is **as-of evaluation**, named as if it were latest. |
| `within_window(start, end)` | Inclusive `collected_at` range. Not validity. |
| `valid_during([start, end))` | Validity-window overlap. Exists; not exposed as `valid_at`. |
| `current(...)` | **Absent** (`pub fn current(` does not exist). |
| `valid_at(...)` | **Absent**. |
| `as_of(...)` | **Absent** as a ledger method name. |

`AssessmentContext::as_of()` returns `self.now`. `FreshnessPolicy.into_context` copies `as_of` into `now`. The four terms are still **aliases in the public surface**.

### 3.2 `AssessmentRun` serialize always emits `asOf` from `started_at`

[`crates/weeping-angel-assurance/src/snapshot.rs`](../../crates/weeping-angel-assurance/src/snapshot.rs):

- Rust field `as_of: String` exists and is deserialized (default empty).
- Custom `Serialize` writes `state.serialize_field("asOf", &self.started_at)` and **ignores** `self.as_of`.
- Live `assess()` sets both `started_at` and `as_of` from the same `Utc::now()` stamp, so the field collision is invisible on the one-shot path.

A caller who pins `as_of` independently of `started_at` cannot persist that pin through JSON.

### 3.3 Replay is fail-open

[`crates/weeping-angel-assurance/src/lineage.rs`](../../crates/weeping-angel-assurance/src/lineage.rs):

```text
pub fn reconstruct(bundle: &LineageBundle) -> AssessmentReport { /* clone pins + results */ }
pub fn replay_assessment(bundle: &LineageBundle) -> Result<AssessmentReport, AssuranceError> {
    Ok(reconstruct(bundle))
}
```

No check that:

- `run.framework_pack_digest` equals `bundle.pack.digest`;
- `run.canonical_catalog_pin` equals `bundle.catalog.digest`;
- `run.assessment_definition_digest` equals `bundle.definition.digest`;
- `run.evidence_snapshot_digest` equals `bundle.evidence.digest`;
- `run.applicability_snapshot_id` equals `bundle.applicability.digest`;
- `run.result_digest` equals `assessment_result_digest(&bundle.results)`;
- `run.as_of` is present and is the evaluation clock;
- required snapshots exist and schema versions match.

Empty pins, stub payloads, or a bundle assembled from **current** files still return `Ok`. `detect_digest_mismatch` exists but is not called from `replay_assessment`.

`AssuranceError` variants on this HEAD: `Collector`, `Compile`, `MissingCollector`, `MissingFramework`, `UnknownPack`, `DigestMismatch`, `UnknownControl`. There is no `MissingPinnedMaterial`, `IncompleteLineage`, `InconsistentLineage`, or `CorruptPersistence`.

### 3.4 Collector `Err` replaces the bag with empty `Vec`

[`AssuranceEngineBuilder::assess`](../../crates/weeping-angel-assurance/src/lib.rs):

```text
Ok(envs)  → status completed, evaluate envs
Err(_err) → status failed/partial, envelopes = Vec::new(), evaluate empty EvidenceSet
```

Prior ledger rows are not consulted. A failed collector is observationally the same as “zero evidence in the universe” for evaluation. `CollectionRun` records `error_count = 1` but the sealed evidence snapshot is the empty digest of that empty bag.

The scheduler collect path (`scheduler.rs`) already **does not delete** prior envelopes on `Err` and only `append`s on `Ok`. One-shot `assess()` does not share that rule. There is no typed distinction among:

```text
no-new-observation
known-absent
evidence-no-longer-valid
collection-failed
```

### 3.5 Collection-run persist is replaceable

`EvidenceLedger::record_collection_run` executes `INSERT OR REPLACE INTO collection_runs`. A later write of the same `run_id` silently overwrites the previous payload, including a completed run. Assessment-run persist uses `persist_immutable` (conflict → `LedgerError::Immutable`); collection runs do not.

`persist_immutable` itself has no schema/version check: any UTF-8 JSON string is stored. `get` / `load_*` map malformed JSON to `LedgerError::Serialize`. There is no `Corrupt` (digest of payload ≠ key, truncated row, unknown schema) or `IncompatibleSchema` variant. Envelope `append` is `INSERT OR IGNORE` by digest (idempotent). Validity events are idempotent on identical bytes and `Immutable` on same `eventId` / different bytes. Those two append paths are already correct and must stay.

The evidence crate has **no** in-crate `#[test]` modules. Persistence proof for this increment lives in the dual-suite and `cargo test -p weeping-angel-evidence` once crate tests are added on owned files.

### 3.6 Historical SoA CLI reloads the live pack

[`src/assurance_soa.rs`](../../src/assurance_soa.rs):

```text
if assessment.is_empty() || assessment.eq_ignore_ascii_case("latest") {
    project_soa("iso-27001", "2022")   // live pack + Utc::now() + empty graph
}
```

`project_soa` builds `OperationalSoaInput { as_of: Utc::now(), assessment: soa-live:{framework}:{version}, … }` and evaluates current pack applicability. `project_soa_from_snapshot` is used only when a **named** assessment id is found in `assurance-ledger.sqlite` and happens to carry a SoA object. Missing or incomplete historical material does not fail closed on the `latest` path — it always succeeds with today’s pack.

Scheduler `run_project` likewise calls live `project_soa(framework, version)` and does not bind SoA to the tick’s pinned assessment run.

### 3.7 Period effectiveness is already conservative on Instant — keep it

`project_period_effectiveness` hard-codes `TemporalSemantics::Instant` and therefore returns `InsufficientObservationCoverage` for a non-empty period even when a sample is `Effective`. That conservative default is **law** for this increment. Do not “fix” Instant so that one positive observation implies `ContinuouslyEffective`. Unknown population coverage must remain explicit.

### 3.8 Neighbor GREEN must not be rewritten as this increment’s baseline

The following characterize **older** shortcuts and are skip-superseded or already GREEN for their own contracts. They are **not** this increment’s characterization suite:

- `sdd_temporal_assurance_baseline` / `_target`
- `sdd_evidence_validity_temporal_assurance_baseline` / `_target`
- `sdd_assessment_lineage_baseline` / `_target`
- `sdd_operational_soa_baseline` / `_target`
- `sdd_continuous_assurance_scheduler_baseline` / `_target`

---

## 4. Desired behavior (target — now shipped; was RED on characterization HEAD)

### 4.1 Four distinct evidence-time APIs

Terms are **not** aliases. Encode them in types/method names **and** tests.

| Term | Question it answers | Result shape | Validity rules | Clock |
| --- | --- | --- | --- | --- |
| **latest** | What is the most recently **recorded** envelope/event? | Single envelope (or event) by recording time | **None.** May be expired, revoked, superseded, or not-yet-valid. | Record time (`collected_at` / event `at`) |
| **current** | What is valid for a **live** assessment right now? | Evaluation leaf (supersede + validity) | Window, revoke, future-exclusion, supersession | Declared live clock (`now`) — not “max collected_at” |
| **valid-at** | Which envelopes were **in force** at instant `T`? | Set (membership) | Half-open `[valid_from, valid_until)` plus revoke/invalidate at `T`; exclude collected/observed after `T` | Caller instant `T` |
| **as-of** | Which envelope would a **pinned assessment clock** have selected? | Evaluation leaf at `T` | Same candidate filter as valid-at, then latest-among-valid-leaves (existing `select_latest_as_of` / `latest_as_of` algorithm) | Pinned assessment `asOf` — never live `Utc::now()` unless the run’s pin *is* that instant |

Required public names (evidence crate, plus assurance re-exports where Guard 11 will look):

```text
EvidenceLedger::latest(type) -> Result<Option<Envelope>>
EvidenceLedger::current(type) -> Result<Option<Envelope>>          // valid leaf at ledger/live clock
EvidenceLedger::valid_at(type, t) -> Result<Vec<Envelope>>         // membership at t
EvidenceLedger::as_of(type, t) -> Result<Option<Envelope>>         // evaluation leaf at pinned t
```

`latest_as_of` may remain as an alias of `as_of` only. Tests must prove:

- `latest` ≠ `current` when the newest row is expired or revoked at `now`;
- `valid_at(t)` can contain more than the as-of leaf and never contains future/revoked-before-`t` rows;
- `as_of(t)` ≠ `current()` when `t` ≠ live `now`;
- `current()` never returns `latest()` when `latest()` fails `project_validity` at `now`.

Half-open window stays: `valid_from <= T` and (`valid_until` is none or `T < valid_until`). Evidence created after as-of, expired before as-of, or revoked before as-of **must not** leak into historical assessment results.

### 4.2 Pinned assessment clock

Every persisted `AssessmentRun` carries an `asOf` that is the evaluation clock. Serialize **that field**, not `startedAt`. Live `assess()` may default `as_of = started_at` when the caller does not pass a clock. Historical replay and SoA **must** use the persisted `asOf`.

`AssessmentContext` must stop treating `as_of()` as a pure alias of `now` in the public contract. Either:

- store `as_of: DateTime<Utc>` separately from `now`, or
- document `now` as the injected assessment clock and add a distinct live-`current` clock type for ledger `current()`.

Tests must be able to set `asOf` independently of wall-clock start.

### 4.3 Append-only validity history

Expiry, revocation, supersession, correction, and validity-window changes are **new** `evidence-validity/v1` events or new sealed envelopes. Never `UPDATE` a sealed envelope payload. `append` stays `INSERT OR IGNORE` by digest. `record_validity_event` stays idempotent on identical bytes and `Immutable` on same `eventId` / different bytes.

### 4.4 Reconstructable lineage

Every result must prove, from persisted data only:

```text
definition identity          = AssessmentDefinitionSnapshot.digest == run.assessmentDefinitionDigest
evidence snapshot identity   = EvidenceSnapshot.digest == run.evidenceSnapshotDigest
applicability identity       = ApplicabilitySnapshot.digest == run.applicabilitySnapshotId
catalog pin                  = CanonicalCatalogSnapshot.digest == run.canonicalCatalogPin
framework pack pin           = FrameworkPackSnapshot.digest == run.frameworkPackDigest
result identity              = assessment_result_digest(results) == run.resultDigest
asOf time                    = run.asOf (serialized from the as_of field)
```

Reconstruction from the same persisted bundle is deterministic (byte-stable report identity).

### 4.5 Replay fail-closed

`replay_assessment` returns a **typed** failure — never `Ok` plus current files — when any of:

| Failure | When |
| --- | --- |
| `MissingPinnedMaterial` | Required snapshot, pin string, or `asOf` absent / empty / `"unpinned"` where a pin is required |
| `DigestIdentity` | Any of the equalities in §4.4 fail, or `detect_digest_mismatch` would fail |
| `IncompleteLineage` | Bundle missing definition / evidence / applicability / results needed to prove the run |
| `InconsistentLineage` | Internal contradiction (e.g. envelope digest list vs snapshot digest, SoA pack pin ≠ run pack pin) |
| `IncompatibleSchema` | Snapshot `schema` ≠ expected lineage / evidence / validity schema |
| `CorruptPersistence` | Payload cannot be authenticated as the stored key / canonical bytes |

`reconstruct` may remain a trusted-clone helper for tests that already hold a verified bundle. Production / CLI replay goes through `replay_assessment`. Replay **must not** load current pack/catalog/evidence to fill gaps. Shipped mapping: `ReplayFailure` → `AssuranceError::UnknownPack` so neighbor exhaustive `AssuranceError` matches stay exhaustive.

### 4.6 Collection failure ≠ evidence erasure

A failed collector run:

- does **not** delete ledger envelopes;
- does **not** write revoke/invalidate events unless an explicit validity event is supplied;
- does **not** cause `assess()` / evaluate to treat the universe as empty when prior valid evidence exists for that scope.

Distinguish at the API / result surface:

| Kind | Meaning |
| --- | --- |
| `NoNewObservation` | Collector succeeded and produced zero new envelopes; prior valid evidence remains candidates |
| `KnownAbsent` | Collector succeeded and asserted the subject has no observation (explicit negative); still not a revoke |
| `CollectionFailed` | Transport / collector error; prior ledger evidence unchanged; assessment status `failed` or `partial` |
| `EvidenceNoLongerValid` | Only via an explicit validity event (revoke / invalidate / expiry) |

`CollectionRun` persist becomes idempotent-append / immutable-if-completed (same family as `persist_immutable`). Duplicate identical payloads are no-ops. A different payload for a completed `run_id` is `LedgerError::Immutable`.

### 4.7 Persistence invariants (`weeping-angel-evidence`)

| Invariant | Rule |
| --- | --- |
| Deterministic serialization | Envelope and validity-event payloads use existing canonical digest / camelCase serde. Re-read equals stored bytes for sealed documents. |
| Stable IDs | `evidence_id` / envelope `digest` / `event_id` / collection `run_id` (when not wall-clock one-shot) do not change on rewrite attempts. |
| Atomic / transactional writes | Multi-row append (envelope + asserted event; supersede + event) commits together or not at all. |
| Idempotent append | Same envelope digest or same validity-event bytes → no new row, `Ok`. |
| Duplicate-event handling | Same `eventId`, different bytes → `Immutable` (already). Same collection-run id, different completed payload → `Immutable` (new). |
| Corruption detection | `get` / load that cannot parse, fails schema, or fails digest-of-payload vs key → typed `Corrupt` (via `PersistenceIntegrity` → `LedgerError::Path`), not a silent default. |
| Schema / version validation | Unknown or newer-incompatible `schemaVersion` → typed `IncompatibleSchema` (via `Path`). Fail closed. |

### 4.8 Historical SoA

- SoA derives from **pinned** assessment / application state.
- Preserve inclusion / exclusion / applicability rationale, implementation status vs effectiveness, exceptions, and evidence references (already operational-SoA law).
- Never infer certification or compliance from readiness / SoA output (disclaimer stays).
- Historical SoA generation **must not** reload current mutable framework / catalog / evidence state.
- CLI `assurance soa` with `latest` / empty / a named historical id uses the pinned assessment (or fail-closed). It must not call live `project_soa` as a stand-in for history.
- If the selected historical assessment cannot be reconstructed exactly (`replay_assessment` would fail), SoA generation fails explicitly with the same typed family.

### 4.9 Period effectiveness stays conservative

A single positive point observation must not imply continuous effectiveness over `[start, end)`. Missing sample intervals and unknown population coverage remain `InsufficientObservationCoverage` (or an equally explicit variant). Do not change Instant default semantics to close this increment.

### 4.10 Adversarial cases the target suite must encode

Clock boundaries; expiry at the exact instant (`T == valid_until` → not valid); evidence recorded after assessment `asOf`; revocation / supersession; duplicate events; stale snapshots; missing pins; corrupted persistence; collection failures; replay after repository / framework file changes; SoA from historical runs.

---

## 5. Acceptance criteria (testable)

1. `latest`, `current`, `valid_at`, and `as_of` are distinct public methods/types; tests show they disagree on a fixture where the newest row is expired or revoked.
2. Historical assessment / `as_of(t)` never includes envelopes with `collected_at`/`observed_at` > `t`, `valid_until <= t`, or revoked/invalidated at `t`.
3. Validity history is append-only: no API mutates a stored envelope payload; revoke/expiry/supersede add events or new rows.
4. `AssessmentRun` JSON `asOf` equals the run’s `as_of` field and can differ from `startedAt`.
5. `replay_assessment` on a bundle with missing pins, digest mismatch, incomplete snapshots, or inconsistent lineage returns a typed `Err` and does not load current pack/catalog files.
6. Replaying the same verified bundle twice yields the same report digest / result identity.
7. Collector `Err` does not delete ledger rows and does not evaluate as an implicit empty world when prior valid evidence exists.
8. `NoNewObservation`, `KnownAbsent`, `CollectionFailed`, and `EvidenceNoLongerValid` are distinguishable in types or recorded outcomes.
9. `record_collection_run` refuses to replace a completed payload (`Immutable` or equivalent); identical retry is idempotent.
10. Malformed payload, digest/key mismatch, and incompatible `schemaVersion` surface as typed `Corrupt` / `IncompatibleSchema` (mapped onto `LedgerError::Path`), not `Ok` defaults.
11. Historical SoA is bound to a reconstructed assessment; CLI `latest`/empty does not call live `project_soa` as history; reconstruction failure is explicit.
12. Period projection remains conservative: one `Effective` sample in Instant semantics is not `ContinuouslyEffective`.
13. Neighbor GREEN targets listed in §0 stay GREEN. This increment’s baseline is GREEN on CURRENT HEAD; target is RED on CURRENT HEAD for the reasons in §3, then GREEN after implement.
14. `select_latest_as_of` remains in `weeping-angel-control-test::temporal`. No new crate `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
15. `cargo fmt --all -- --check` and `cargo check --workspace` succeed after implement.

---

## 6. Dual-suite protocol

Mandatory: spec first (this file; no product feature code) → baseline GREEN on CURRENT code → target RED on CURRENT code for the **right** reasons in §3 → implement → docs/ADR if needed → target GREEN → prove baseline fails or is additive-documented → skip-supersede baseline → target still GREEN.

```text
# register in the implement commit that adds the .rs files (tests/contracts is not auto-discovered)
[[test]]
name = "sdd_temporal_lineage_evidence_soa_baseline"
path = "tests/contracts/temporal_lineage_evidence_soa.baseline.rs"

[[test]]
name = "sdd_temporal_lineage_evidence_soa_target"
path = "tests/contracts/temporal_lineage_evidence_soa.target.rs"
```

| Suite | Role |
| --- | --- |
| Baseline | Characterization of §3 on SHA `0015f63…` / current HEAD. **GREEN now.** After implement: `#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]`. |
| Target | Desired §4. **RED now** because `current`/`valid_at`/`as_of` are missing, serialize ignores `as_of`, `replay_assessment` is `Ok(reconstruct)`, `assess` empties on collector `Err`, collection-run replace, untyped persist errors, SoA `latest` → live `project_soa`. Not compile/harness noise. |

Suggested case ids (target): `TLE-001` four-term distinctness; `TLE-002` no future leakage; `TLE-003` expiry-at-instant; `TLE-004` revoke/supersede append-only; `TLE-005` `asOf` ≠ `startedAt` persist; `TLE-006` replay missing pin; `TLE-007` replay digest mismatch; `TLE-008` replay incomplete/inconsistent; `TLE-009` collection failure ≠ erasure; `TLE-010` collection-run immutable; `TLE-011` corrupt / incompatible schema fail-closed; `TLE-012` historical SoA not live `project_soa`; `TLE-013` period conservative; `TLE-014` duplicate validity event; `TLE-015` replay after pack/catalog file change.

Verify after implement:

```text
cargo test --test sdd_temporal_lineage_evidence_soa_baseline
cargo test --test sdd_temporal_lineage_evidence_soa_target
cargo test --test sdd_temporal_assurance_target --test sdd_evidence_validity_temporal_assurance_target --test sdd_assessment_lineage_target --test sdd_operational_soa_target --test sdd_continuous_assurance_scheduler_target
cargo test -p weeping-angel-evidence
cargo test -p weeping-angel-assurance
cargo fmt --all -- --check
cargo check --workspace
```

---

## 7. Guard handoff (Prompt 1 wires later)

Do not edit `xtask/**`. Expose stable symbols Prompt 1 can grep / call for Guards 09–12:

| Guard (architecture program) | This increment exposes |
| --- | --- |
| 09 temporal move | **Do not move** `select_latest_as_of`. Keep it in control-test. Ledger `as_of` / `valid_at` / `current` are the evidence-side law. |
| 10 `AssessmentRun` lineage rebuild | `replay_assessment` fail-closed + typed `Replay`/`AssuranceError` variants + proven pin equalities. |
| 11 ledger `current()` / `as_of(t)` | `EvidenceLedger::current` and `EvidenceLedger::as_of`. |
| 12 (neighbor) | Persist metadata: schema constants, typed `LedgerError::{Corrupt, IncompatibleSchema}`, SoA bound to run pins. |

---

## 8. Out of scope

- Moving `select_latest_as_of` into `weeping-angel-evidence` (Phase 5 / Guard 09).
- Catalog TOML, framework pack parse/digest engines, readiness projection redesign (Prompt 2).
- xtask guards, `architecture.toml`, `docs/debt/register.toml` (Prompt 1).
- Broad `#[ignore]` cleanup, panic-budget, schema-fixture dedup, README / `documentation_layout` index edits (Prompt 4).
- New crates `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
- New PostgreSQL / remote ledger / object store.
- UI charts / dashboards / certification claims.
- Weakening Instant period semantics so one sample becomes continuous effectiveness.
- Replacing neighbor dual-suites or writing product specs under `docs/sdd/` or tests under `tests/sdd/`.
- ISMS event-bus vocabulary (`ControlRegressed`, …).
- Licensed ISO normative text.

---

## 9. Risks

- Neighbor GREEN suites grep `latest_as_of` / `asOf` / `replay_assessment` / `project_soa` strings; additive aliases and serde defaults are required so those suites stay GREEN.
- Prompt 1 architectural-cleanup baseline currently asserts `!ledger.contains("pub fn current(")` — adding `current()` will go RED there. Prompt 1 owns that test; this increment still **must** add `current()` (scope notes). Coordinate by not editing xtask; Prompt 1 rewires Guard 11 against the new API.
- `AssessmentRun` custom serialize is grepped by `sdd_assessment_lineage_target` (LIN needles). Changing `asOf` to persist `as_of` must keep camelCase field set complete (`canonicalCatalogDigest`, `catalogDigest`, `applicabilitySnapshotId`).
- Treating scheduler `project_soa` live call as in-scope may collide with scheduler-owned files; prefer SoA helper + CLI + typed historical projector, and only touch `scheduler.rs` if required to stop historical SoA from using live pack on a historical tick.
- Atomic multi-row SQLite writes need an explicit transaction; a naive two-step append can leave envelope-without-event on crash.
- Overloading `valid_at` as a single leaf would re-alias as-of; tests must keep membership vs leaf distinct.
- Fail-closed replay will break any caller that passed a partial `LineageBundle` and expected `Ok`; those callers must use `reconstruct` only in tests or after verification.

---

## 10. Related

- [`temporal-assurance.md`](temporal-assurance.md) — Prompt 14 as-of / period (implemented). Four-clock ledger APIs extend that law; Instant period conservatism unchanged.
- [`evidence-validity-temporal-assurance.md`](evidence-validity-temporal-assurance.md) — validity events (implemented; still append-only `evidence-validity/v1`).
- [`assessment-lineage.md`](assessment-lineage.md) — persistable runs / `reconstruct` (implemented). Replay is now fail-closed (`replay_assessment` verifies pins). JSON `asOf` is the `as_of` field.
- [`operational-soa.md`](operational-soa.md) — operational projection (implemented). Historical CLI binds to reconstructed assessment; live `project_soa` is not history.
- [`typed-evidence.md`](typed-evidence.md) — envelope seal / digest law. Persistence integrity names live in the evidence crate without forking `DigestBody`.
- [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md) — tick collect-without-delete. One-shot `assess` no longer treats collector `Err` as an empty universe when process-local prior valid envelopes exist.
- Accepted decision: [`docs/adr/0011-temporal-lineage-evidence-soa-integrity.md`](../adr/0011-temporal-lineage-evidence-soa-integrity.md).
