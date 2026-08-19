# ADR 0005 — Continuous assurance scheduler (library runtime seam)

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_continuous_assurance_scheduler_target` GREEN (CAS-001…016); baseline skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** replace ADR 0001 compile pipeline, ADR 0002 ISO vertical, or ADR 0003 assessment-lineage snapshot law. One-shot `assess` remains; failed one-shot collect still evaluates an empty set. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (facade orchestrates collectors + evaluate), [ADR 0003 assessment lineage](0003-assessment-lineage.md) (failed collection is representable; this ADR **reattaches** prior ledger evidence on scheduled ticks) |
| Spec | [`docs/specs/continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_continuous_assurance_scheduler_target` GREEN; `sdd_continuous_assurance_scheduler_baseline` `#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]` |

> Filename `0005-*` follows [ADR 0004](0004-documentation-architecture.md) (docs layout). Cite this decision by **path**. Do not reuse `0004-*` for the scheduler.

## Context

ADR 0001 shipped an inwardly extensible facade:

```text
AssuranceEngine::builder().collector(…).framework(target).assess(scope)
```

ADR 0003 lineage made `AssessmentRun` a real returned record and allowed `failed` / `partial` collection **without aborting** `assess`. On SHA `6e31bf1…` that still meant: one collector, one wall-clock `CollectionRun::new` (`run_id` from `Utc::now()`), and on `collect` `Err` an **empty** `EvidenceSet` even if the ledger held earlier envelopes. There was no `Clock` trait, no cadence/retry/backoff/jitter/next-run contract, and no scheduler module.

Operational ISMS continuous-assurance scheduler requires the **same** deterministic pipeline to run repeatedly as a local/offline engine:

```text
Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift
```

Questions this decision answers:

1. Where does scheduling live (library vs clap vs new crate vs cluster runtime)?
2. How is time injected so tests are deterministic?
3. What is a run identity, and how do duplicates/crashes collapse?
4. What happens to prior ledger evidence when a collector fails?
5. Who may set compliance results, and which crates stay network-free?
6. How does this relate to ISMS IR slices 01–12 / temporal validity / events-drift?

## Decision (as implemented)

### 1. Library runtime seam, not a clap DSL

Scheduling is a public module `weeping-angel-assurance::scheduler`. No new workspace crate. No Kubernetes, Temporal, OS cron, or cloud queue as the core.

`JobSpec` (cadence, freshness, `depends_on`, retry/backoff, timeout, jitter) is library data. `weeping-angel isms run` is **not** shipped. Clap must not become the source of truth for those fields (CAS-016: `src/cli.rs` has no `--cadence` / `--backoff` / `--jitter` / `--timeout`).

Daemon mode is an outer loop around `tick`. It is not required in this slice.

```text
AssuranceScheduler::builder()
    .clock(Clock)
    .store(InMemorySchedulerStore)
    .ledger(Arc<Mutex<EvidenceLedger>>)
    .framework(FrameworkTarget)
    .scope(AssessmentScope)
    .collector(C: EvidenceCollector)*
    .register(JobSpec)
    .build() → tick() → TickReport
```

### 2. `Clock` is the time seam

```text
trait Clock { fn now(&self) -> DateTime<Utc>; }
```

Tests use `FakeClock`. Production uses `UtcClock`. `weeping-angel-framework` and `weeping-angel-control-test` do **not** take `Clock`; the scheduler passes `AssessmentContext { now, max_age }` from `Clock::now()` and `JobSpec` freshness.

### 3. Scheduler orchestrates collectors

Crate edges stay ADR 0001: collectors do not depend on the facade; the facade calls `EvidenceCollector::collect`. Independent collection jobs (no `dependsOn` between them) run concurrently (`thread::scope`). Dependent jobs wait until the dependency has `last_attempt` or `last_successful_run` in the store.

Collectors emit envelopes only. They never set `Effectiveness` or other compliance results.

### 4. Four job kinds; eight pipeline stages

`JobKind`: `Collection` | `Test` | `Projection` | `Snapshot`. Drift is **not** a fifth kind. A successful slot records `PipelineStage` in order:

```text
Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift
```

| Kind | Stages |
| --- | --- |
| Collection | Collect → Normalize → Seal → Ledger (per collector) |
| Test | Evaluate (`evaluate_compiled` + existing `StaleEvidence` law) |
| Projection | `project_soa` / `project_readiness` |
| Snapshot | Persist `AssessmentRun`; Drift via existing `compare` (`SnapshotDiff`) |

Scheduler Drift does **not** invent ISMS event types. Typed observations are [ADR 0003 ISMS events/drift](0003-isms-events-drift.md) (`detect_events` / `detect_isms_drift`). `tick` may keep calling `compare` only.

### 5. Stable run identity and resume

Scheduled `run_id` is a canonical digest of **job id + cadence slot + collector id + configuration digest + attempt-policy version (`v1`)** — not `Utc::now()` uniqueness. Slot is origin-aligned truncation of `Clock::now()` by cadence. `CollectionRun.run_id` for scheduled collection **is** this identity. Envelope content digests remain observation+provenance; `with_collection_run` is attached outside digest law.

Duplicate `tick`s in the same slot: `InFlight` / `Succeeded` skip a second collect. Crash/restart with the same `InMemorySchedulerStore` + slot shares that identity. Envelope append remains digest `INSERT OR IGNORE`. Job/run operational state lives in the **assurance-owned** store, not inside evidence observation payloads.

### 6. Failure does not erase evidence

A failed or timed-out collect records the `CollectionRun` (`failed` / `timed_out`) and job `FailureState` (`Retrying` | `TimedOut` | `FailedExhausted`). It does **not** delete ledger envelopes.

Evaluate for the slot loads ledger envelopes for the scheduled collectors. Freshness (`max_age` vs `Clock::now()` and `collected_at`) decides usability: fresh prior evidence may yield `Effective`; stale prior evidence yields existing `StaleEvidence`. Temporal `valid_from` / `valid_until` windows are **not** invented here.

One-shot `assess` is unchanged: collector `Err` still evaluates an empty set. Reattach is scheduler-path law.

### 7. Retry, backoff, timeout

Job retry is exponential with ceiling, measured on `Clock` (not real sleep). Timeout is cooperative: if `Clock` advances ≥ `timeout` during `collect`, the attempt is `timed_out`. Jitter is an optional extra delay on collection next-run; tests set `0`. GitHub HTTP retry helpers stay collector-local.

### 8. Do not wait for ISMS IR

Jobs key off existing assessment/collector/scope identities. This slice does not fork a parallel GRC runtime (`IsmsContext` is not a scheduler key).

## Consequences

- Continuous operation is `tick` with a fake or UTC clock. One-shot `assess` stays a convenience and **does not** share reattach.
- GitHub HTTP retry helpers stay collector-local; job retry is scheduler-local.
- Neighbor suites (ACT, ISO, catalog, GitHub, Kleene, lineage) stay GREEN; framework and control-test remain network-free.

## Non-goals

- Cluster schedulers, hosted queues, clap crontab, `weeping-angel isms run` product, temporal `valid_from`/`valid_until` models, ISMS event catalog, catalog/ISO/Kleene/GitHub mapping edits.

## Related

- Spec: [`docs/specs/continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md)
- Stub: [`docs/sdd/continuous-assurance-scheduler.md`](../sdd/continuous-assurance-scheduler.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
