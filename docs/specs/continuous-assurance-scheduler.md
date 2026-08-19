# SDD: Continuous Assurance Scheduler

| Field | Value |
| --- | --- |
| Status | **Implemented** — library `AssuranceScheduler` in `weeping-angel-assurance::scheduler`; CAS-001…016 GREEN |
| Program | Operational ISMS v1 — continuous-assurance scheduler |
| Slice | Library-first scheduling runtime over the existing Collect → Evaluate → Snapshot spine |
| Dual-suite | `sdd_continuous_assurance_scheduler_target` GREEN · `sdd_continuous_assurance_scheduler_baseline` skip-superseded |
| Dual-suite paths | `tests/contracts/continuous_assurance_scheduler.{baseline,target}.rs` |
| ADR | Accepted [`docs/adr/0005-continuous-assurance-scheduler.md`](../adr/0005-continuous-assurance-scheduler.md) |
| Layout law | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — this file is the human SSOT; `docs/sdd/` is a stub pointer; traces go to `.sdd/` |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — facade `tick` composition |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Lineage / snapshots (consumed) | [`docs/specs/assessment-lineage.md`](assessment-lineage.md), ADR 0003-assessment-lineage |
| temporal assurance (do not land here) | Temporal validity windows — [`docs/specs/temporal-assurance.md`](temporal-assurance.md), [`docs/specs/evidence-validity-temporal-assurance.md`](evidence-validity-temporal-assurance.md); consume `control-test::FreshnessPolicy` / `AssessmentContext.now` |
| ISMS events/drift (landed; do not invent a second catalog) | Prompt 15 `detect_events` / `detect_isms_drift` — [`isms-events-drift.md`](isms-events-drift.md). Scheduler Drift still **calls** `compare`; it does not reimplement the event catalog. |
| ISMS IR slices 01–12 | **Not landed.** Do not invent `IsmsContext` / parallel GRC. Extend the existing assurance spine. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Workspace verify | `cargo test --workspace --features demo` |

This document is the durable SSOT for continuous-assurance scheduler. It owns **when** collection, evaluation, projection, and snapshot work run, **how** those runs identify and resume, and **how** a failed collector interacts with prior ledger evidence. It does **not** own catalog TOML, GitHub evidence mapping, Kleene applicability, ISO remaps, temporal `valid_from`/`valid_until`, or the ISMS events/drift event vocabulary.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

The scheduler is an **orchestrator** of that spine over time. It is not a second compiler and not a collector.

Operational pipeline (this slice):

```text
Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift
```

This is **not** the ADR 0001 compile pipeline (`normalize` → resolve applicability → … → integrity digest). Compile `normalize` stays inside `compile_framework`. Scheduler “Normalize” means preparing collector observations for `EvidenceEnvelope::seal`.

---

## 0. Collision fence (concurrent SDD)

This slice may add a scheduler module, a `Clock` seam, job/run contracts, and a thin CLI dispatch that **calls** the library. It may wire `assess`-equivalent steps through that library without changing collector blindness or test purity.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML | catalog family specs |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` mapping | GitHub collector / [`github-collector.md`](github-collector.md) |
| Kleene evaluator / `OrgContext` / `tests/contracts/applicability_engine.*` | applicability engine / [`applicability-engine.md`](applicability-engine.md) |
| ISO pack requirement/control IDs, pack `to =` remaps, `tests/contracts/iso27001_remap.*` | controlled documents / [`iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) |
| `IsmsContext` and ISMS IR slices 01–12 IR types | unlanded; do not fork a GRC graph here |
| `valid_from` / `valid_until` / revocation records | temporal assurance |
| `ControlRegressed` and the ISMS events/drift event catalog | ISMS events/drift |

Suggested **new** product modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `Clock`, `Schedule`, `JobSpec`, `JobState`, `Scheduler`, `tick` / `run_due` | `weeping-angel-assurance::scheduler` |
| Scheduler persistence (job state, in-flight run ids) | assurance-owned store; **not** evidence envelopes |
| Envelope append / query / collection-run record | existing `weeping-angel-evidence::ledger` |
| Evaluate | existing `weeping-angel-control-test` (`AssessmentContext.now` + `max_age`; `FreshnessPolicy` / as-of from temporal assurance) |
| Project / snapshot / compare | existing `readiness`, `soa`, `snapshot`, `lineage` |
| Thin `weeping-angel isms run` | `src/cli.rs` + a small dispatch module; **no** cadence/retry/backoff/jitter flags as clap source of truth |

GitHub collector HTTP `backoff` / `Retry-After` helpers stay **collector-internal** and unused-or-caller-owned per the GitHub spec. This slice’s retry/backoff is **job-level** in the scheduler.

---

## 1. Problem / user-visible goal

Assurance today is a **one-shot** library call (`AssuranceEngineBuilder::assess`). Operators can collect and evaluate once; they cannot run the same deterministic pipeline repeatedly as a local/offline engine with cadence, freshness, retry, and resume.

That means:

- a collector failure on the one-shot path used to evaluate an **empty** `EvidenceSet` even when prior valid envelopes existed in-process (closed by [ADR 0011](../adr/0011-temporal-lineage-evidence-soa-integrity.md); tick reattach was already ledger-backed);
- `CollectionRun.run_id` is derived from `Utc::now()`, so crash/retry invents a new identity and cannot deduplicate;
- there is no due/not-due clock, no next-run, no job dependencies, no timeout/jitter contract;
- CLI has `weeping-angel assurance …` (mostly banner stubs) and **no** `weeping-angel isms` family;
- putting cron/backoff into clap would freeze scheduling semantics in the scanner CLI instead of the library.

**User-visible goal:** the same Collect → Evaluate → Snapshot pipeline can operate **continuously and safely over time** on a local/offline core: due jobs run, independent collectors may run concurrently, failed collection does not wipe prior evidence, duplicate work collapses to a stable run identity, and a process crash can resume.

Definition of done: the deterministic assurance pipeline is no longer only a one-shot assessment.

### Remaining increment (Prompt 3 — do not fork this spec)

CAS-001…016 and `sdd_continuous_assurance_scheduler_*` remain scheduler SSOT (GREEN / skip-superseded). Tick collect already avoids deleting envelopes on collector `Err`. One-shot `assess()` no longer evaluates an implicit empty world on collector `Err`: [`temporal-lineage-evidence-soa.md`](temporal-lineage-evidence-soa.md) (`sdd_temporal_lineage_evidence_soa_target` GREEN). This increment must not reimplement cadence / retry / daemon.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `AssuranceEngineBuilder::assess` | `weeping-angel-assurance::lib` | One-shot path may remain. Scheduler **reuses** collect/seal/evaluate/project/snapshot/compare; it must not bypass them with a parallel evaluator. Prefer extracting shared steps over duplicating evaluate. |
| `CollectionRun::new` | `weeping-angel-evidence` | Today `run_id = digest(collector_id, Utc::now())`. Scheduled runs **must not** use wall-clock uniqueness as identity. One-shot constructor may stay for non-scheduled callers until they opt in. |
| Envelope `collection_run_id` | `EvidenceEnvelope::seal` | Provenance digest (`collector_id`, `collected_at`, `scope`). Attaching a scheduler run id via `with_collection_run` is allowed if it does not change envelope **content digest** law. |
| `EvidenceLedger` | evidence crate | Append-only envelopes (`INSERT OR IGNORE`). Completed `record_collection_run` is lineage-immutable (identical retry no-op; different completed bytes → `Immutable`). Scheduler must not `DELETE` envelopes on collector failure. Do not store cadence/retry in envelope payloads. |
| `AssessmentContext` | control-test | `{ now, max_age }`. Scheduler supplies `now` from `Clock` and `max_age` from the job freshness policy. Control-test stays network-free and provider-blind. |
| `evaluate` / `Effectiveness` | control-test | Collectors **never** write these. Stale prior evidence already maps to `StaleEvidence` when age `> max_age`. |
| `compare` / `SnapshotDiff` | assurance snapshot | Scheduler Drift step **calls** existing compare on consecutive snapshots. Semantic observations are Prompt 15 `detect_events` / `detect_isms_drift` — do not invent a second catalog here. |
| `EvidenceCollector` | collector crate | `collect(&scope) -> Result<Vec<EvidenceEnvelope>, _>`. Scheduler calls this. Do not reverse the edge (collectors must not depend on scheduler). |
| `GitHubCollector::collect_batch` / `backoff` | github module | Collision fence. Scheduler treats GitHub as one `EvidenceCollector`. Do not rewrite mapping or wire HTTP retry as the scheduler contract. |
| CLI `Commands` / `AssuranceCommand` | `src/cli.rs` | No `Isms` command today. Implement may add `weeping-angel isms run` as a **thin** library call. Cadence, backoff, jitter, next-run **must not** be clap-defined. |
| Framework crate | `weeping-angel-framework` | Network-free; no scheduler, no collectors. |
| ISMS IR slices 01–12 | ISMS IR | Unlanded. Job keys use existing `AssessmentId`, collector id, scope, catalog/pack pins — not `IsmsContext`. |

Crate graph (frozen ADR 0001): framework ↛ collector; collector ↛ framework; control-test ↛ collector/network; scheduler lives in the **facade** and may depend on collector + control-test + evidence + framework the same way `assess` already does.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded later by `tests/contracts/continuous_assurance_scheduler.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 No scheduler module, no `Clock`, no job contracts

`crates/weeping-angel-assurance/src/` is `applicability/`, `bridge.rs`, `lib.rs`, `lineage.rs`, `readiness.rs`, `snapshot.rs`, `soa.rs`. There is no `scheduler` module.

Workspace search: no `trait Clock`, no `cadence` / `jitter` / `next_run` types, no public `JobSpec` / `Schedule` / `BackoffPolicy`.

`AssessmentContext.now` is filled with `Utc::now()` inside `assess`. Control-test timestamps use `Utc::now()` as `checked_at` default.

### 3.2 `assess` is a single-shot library call

[`crates/weeping-angel-assurance/src/lib.rs`](../../crates/weeping-angel-assurance/src/lib.rs) `assess`:

1. compile the framework;
2. `CollectionRun::new(descriptor.id, descriptor.version)` (wall clock);
3. `collector.collect` once;
4. on `Ok`, evaluate those envelopes;
5. on `Err`, set run `failed`/`partial` and evaluate **`Vec::new()`**;
6. build `AssessmentRun` and return `AssessmentReport`.

There is no loop, no due-time check, no dependency graph, no timeout around `collect`, no job-level retry, no ledger load, no persist of the report into the ledger from this function (`load_lineage` exists; `assess` does not call `EvidenceLedger`).

Hard-coded freshness: `max_age: Duration::from_secs(24 * 3600)`.

### 3.3 Collector failure discards usable history for that call

On `collector.collect` `Err`:

- `collection_run.evidence_count` is still `0` (never updated before the match);
- envelopes become `Vec::new()`;
- `EvidenceSet` contains only exceptions from the assessment definition;
- prior `evidence_envelopes` rows are not queried.

`EvidenceLedger` can `query` / `latest` / `for_subject` / `for_type` / `within_window` / `supersede`. The one-shot path does not reattach those envelopes.

### 3.4 `CollectionRun` identity is not idempotent

```text
started_at = Utc::now()
run_id = "run:" ++ canonical_digest(collector_id, started_at)[0..16]
```

Two attempts of the same collector + scope + configuration produce two run ids. Duplicate work cannot collapse.

`record_collection_run` replaces by `run_id` (`INSERT OR REPLACE`). Envelope append is digest-keyed `INSERT OR IGNORE` — content-identical envelopes dedupe, **runs** do not.

### 3.5 Independent collectors are not a scheduler concern

`EvidenceCollector` is one object on the builder. GitHub has `collect_batch` for a single collector. There is no fan-out of independent collectors, no isolation of per-collector `CollectionRun` state, and no concurrency contract at the facade.

### 3.6 CLI has no ISMS runtime

`Commands`: `Scan`, `Finalize`, `ScanCode`, `ScanDiff`, `Workbench`, `Depcheck`, `Assurance`, `Version`, `Completions`.

`AssuranceCommand`: `Framework`, `Collect`, `Evidence`, `Assess`, `Result`, `Compare`, `Soa`, `Catalog`, `Explain`.

`src/main.rs` dispatches `Catalog` and `Explain`; every other assurance arm prints the not-certification banner and exits 0. There is no `isms` subcommand and no `weeping-angel isms run`.

### 3.7 Drift is on-demand compare, not a scheduled job

`compare` / `compare_runs` / `compare_lineage` exist. Nothing schedules them after a snapshot. ISMS events/drift events do not exist — and must not be invented here.

### 3.8 Network-free crates (must remain true)

`weeping-angel-framework` depends on serde/toml/IR only. `weeping-angel-control-test` depends on chrono/serde/IR/evidence only (chrono `clock` feature for `DateTime`, not HTTP). Neither talks to collectors.

---

## 4. Desired behavior (after implement)

### 4.1 Library runtime first

Public composition (as implemented; camelCase JSON on persisted documents):

```text
Clock::now() -> DateTime<Utc>
AssuranceScheduler { clock, store, collectors, framework target, ledger }
  .register(JobSpec)
  .tick() -> TickReport
```

`tick` is the unit tests drive with `FakeClock`. Production uses `UtcClock`. A future daemon is `loop { scheduler.tick(); sleep_until(next_wakeup) }` **outside** clap.

CLI may expose:

```text
weeping-angel isms run
```

That command is **not shipped**. Cadence, backoff, jitter, and timeout live on `JobSpec`, not clap (`src/cli.rs` has no those flags).

Daemon mode is out of this slice except that `tick` is daemon-safe (idempotent, no global collector mutation).

### 4.2 Scheduling contracts

Every job carries:

| Field | Meaning |
| --- | --- |
| `jobId` | Stable identity (not a run id) |
| `kind` | `collection` \| `test` \| `projection` \| `snapshot` (Drift is a snapshot-adjacent step; may be `kind=snapshot` with a drift flag **or** an explicit `drift` kind — pick one and freeze in the target suite) |
| `cadence` | Fixed interval on the fake/real clock (local; not OS crontab, not Kubernetes CronJob) |
| `freshness` | Max age / reuse policy for prior evidence (feeds `AssessmentContext.max_age`) |
| `dependsOn` | Other `jobId`s that must have a **terminal attempt** (`lastAttempt` or `lastSuccessfulRun`) before this job starts. Failed collect still unblocks Evaluate so freshness reattach can run. |
| `retry` | `maxAttempts`, exponential backoff with ceiling |
| `timeout` | Wall time on `Clock` after which the attempt is `timed_out` (tests advance the fake clock; no real `thread::sleep` required) |
| `jitter` | Optional extra delay on **collection** next-run only; tests set `0` |
| `lastSuccessfulRun` | Identity + timestamp of last success (absent if never) |
| `lastAttempt` | Identity + timestamp + outcome of last try |
| `nextRun` | Clock time when the job becomes due |
| `failureState` | `none` \| `retrying` \| `timed_out` \| `failed_exhausted` \| `crashed` (exact enum frozen by target tests) |

`JobKind` coverage:

| Kind | Runs |
| --- | --- |
| Collection | Collect → Normalize → Seal → Ledger (per collector) |
| Test | Evaluate compiled tests against the evidence set selected for this slot |
| Projection | `project_soa` / `project_readiness` from the evaluation |
| Snapshot | Persist `AssessmentRun` + lineage snapshots; then Drift via existing `compare` (semantic events: Prompt 15 `detect_events`, not invented here) |

**As implemented:** a DAG of the four kinds above. Drift is a `PipelineStage` on snapshot, not a fifth `JobKind`. Successful slots record the eight stage names in order. Independent collection jobs run concurrently.

### 4.3 Idempotent run identity

A scheduled **run identity** is a canonical digest of at least:

```text
jobId, slot (cadence-aligned clock truncation), collectorId? , configurationDigest, attemptPolicyVersion
```

Not `Utc::now()` uniqueness. Re-invoking `tick` at the same slot with the same spec **returns the existing run** (same id, no second collect, no second envelope set) if that run already succeeded or is in-flight.

`CollectionRun.run_id` for scheduled collection **is** this identity (or a documented 1:1 derivative). Envelope content digests remain observation+provenance; scheduler must not require rewriting digest law.

### 4.4 Failed collector does not erase previous evidence

If collect errors, times out, or returns no usable envelopes:

1. Record the attempt on the `CollectionRun` (`failed` / `partial`) and job `failureState`.
2. **Do not** delete or truncate ledger envelopes.
3. Reattach prior envelopes that still pass the job **freshness** policy (`collected_at` vs `Clock::now()` and `max_age`).
4. Evaluate that set. Fresh prior evidence may still yield `Effective` / `Ineffective` / etc. Stale prior evidence yields existing `StaleEvidence` (control-test), not a silent empty fail that pretends history never existed.
5. If no prior envelope is still fresh, results are insufficient/stale as today — **with** lineage pointing at the failed collection run **and** the unused/stale prior digests.

This is the opposite of current `Err => Vec::new()`.

### 4.5 Resume, concurrency, isolation

- Persist job state + in-flight run identity **before** side effects when practical; always persist after attempt completion.
- Crash mid-collect: restart with the same clock/slot **resumes or skips** rather than corrupting the ledger (append-only + identity dedupe).
- Independent collection jobs (no `dependsOn` edge between them) **may run concurrently**. Shared ledger must remain correct (digest primary key). Per-run structs are not globals.
- Dependent jobs wait: Test/Evaluate does not start for a slot until required collection attempts for that slot have terminal state (success, or failure that still allows freshness reattach per §4.4).

### 4.6 Clock seam

```text
trait Clock { fn now(&self) -> DateTime<Utc>; }
```

Fake clock: tests set `now`, advance by cadence/backoff/timeout, and assert due vs not-due. Production clock is UTC.

Control-test and framework crates **do not** take a `Clock` trait. The scheduler injects `now` into `AssessmentContext`.

### 4.7 Safety

1. Framework and control-test crates remain network-free (no reqwest/octocrab/AWS; no scheduler→network shortcut through those crates).
2. Scheduler orchestrates collectors; collectors do not call the scheduler and do not depend on `weeping-angel-assurance`.
3. No collector sets `Effectiveness` or other compliance results. Collectors emit envelopes only.
4. Scheduler output is still a **readiness** assessment: never `ISO 27001 certified` / `compliant` / `audit passed`.

### 4.8 Pipeline mapping onto existing code

| Stage | Existing code | Scheduler duty |
| --- | --- | --- |
| Collect | `EvidenceCollector::collect` | Due collection jobs; timeout; retry |
| Normalize | collector observation assembly (not compile `normalize`) | Keep provider-blind facts |
| Seal | `EvidenceEnvelope::seal` | Reject claims/credentials as today |
| Ledger | `EvidenceLedger::append` + `record_collection_run` | Never erase on failure |
| Evaluate | `evaluate` + compiled tests | After collect DAG; freshness context |
| Project | `project_soa`, readiness snapshot | After evaluate |
| Snapshot | `AssessmentRun`, lineage persist APIs | Pin digests; append-only completed runs |
| Drift | `compare` / `compare_runs` | After snapshot. Readiness bags stay here. Typed `IsmsEvent`s are Prompt 15 `detect_isms_drift` — call, do not reimplement. |

`assess` remains a separate one-shot path (`CollectionRun::new` wall-clock identity; collector `Err` still evaluates `Vec::new()`). Reattach, slot identity, retry, and freshness are **scheduler-path** law. Do not assume `assess` and `tick` share evidence selection.

---

## 5. Dual-suite law

Root `Cargo.toml` does **not** auto-discover `tests/contracts/*.rs`. Implement **must** add, in the **same commit** as the `.rs` files:

```toml
[[test]]
name = "sdd_continuous_assurance_scheduler_baseline"
path = "tests/contracts/continuous_assurance_scheduler.baseline.rs"

[[test]]
name = "sdd_continuous_assurance_scheduler_target"
path = "tests/contracts/continuous_assurance_scheduler.target.rs"
```

Without those stanzas, `cargo test --test sdd_continuous_assurance_scheduler_{baseline,target}` fails with `no test target named …` before any `#[test]` runs (I3 HARD FAIL).

| Suite | File | Bar |
| --- | --- | --- |
| Baseline | `continuous_assurance_scheduler.baseline.rs` | Characterization of SHA `6e31bf1…`. **Skip-superseded:** `#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]`. |
| Target | `continuous_assurance_scheduler.target.rs` | **GREEN** on CAS-001…016. Drive time with `FakeClock`. Command: `cargo test --test sdd_continuous_assurance_scheduler_target --offline -- --nocapture`. |

I4a: the target suite must **never** read its own source and assert it lacks a substring. Assert product types and runtime behavior.

Transition: **replacement**. Absence-of-scheduler baseline cannot stay CI-required after the module exists.

Regression after GREEN: `cargo test --workspace --features demo`. Neighbor targets (`sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_iso27001_remap_target`, `sdd_github_collector_target`, `sdd_applicability_engine_target`, `sdd_assessment_lineage_target`, catalog family targets, `sdd_documentation_layout`) stay green.

At implement, add this spec path to `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` and the README spec list if those files would otherwise omit a landed SSOT.

---

## 6. Target tests (CAS)

Encode the **original found cases** on current HEAD. Titles may use `CAS-00N` and/or `P?: <subject>` as below.

| Id | Subject | Assert |
| --- | --- | --- |
| CAS-001 | Dual-suite registered | `[[test]]` names exist after implement (harness + behavior). |
| CAS-002 | Due | Fake clock at/after `nextRun` → collection/test job runs once. |
| CAS-003 | Not due | Clock before `nextRun` → no collect, no evaluate. |
| CAS-004 | Retry | Failed collect with remaining attempts schedules another try. |
| CAS-005 | Backoff | Next attempt time is exponential with ceiling, measured on the fake clock (not real sleep). |
| CAS-006 | Timeout | Attempt that exceeds `timeout` on the clock is `timed_out`; collector is not allowed to hang the tick. |
| CAS-007 | Crash / restart | Persist in-flight identity; new scheduler instance with same store + slot does not double-apply side effects. |
| CAS-008 | Duplicate run | Two `tick`s for the same identity return one run id; ledger envelope count does not double. |
| CAS-009 | Dependency ordering | Evaluate/project/snapshot do not run before required collection terminal state. |
| CAS-010 | Concurrent independent collectors | Two collectors without a dependency both complete; envelopes isolated by run id; no crossed `CollectionRun` fields. |
| CAS-011 | Stale previous evidence | Failed collect + prior envelope older than freshness → `StaleEvidence` (or insufficient), prior digest still in ledger. |
| CAS-012 | Fresh previous evidence | Failed collect + prior envelope within freshness → evaluate **uses** that envelope; ledger still has it. |
| CAS-013 | Pipeline order | Successful slot records Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift. |
| CAS-014 | Network-free crates | Framework and control-test `Cargo.toml` still have no HTTP/SDK deps; scheduler is not imported from those crates. |
| CAS-015 | Collector blindness | Fixture collector cannot set `Effectiveness`; results come from evaluate. |
| CAS-016 | CLI seam | If `isms run` exists, clap source does not define cadence/retry/backoff/jitter; library does. |

CAS-002…012 **must** fail on current HEAD once the target harness is registered (missing API / missing reattach / wall-clock run ids). A target that only checks `Cargo.toml` for `[[test]]` is insufficient.

---

## 7. Acceptance criteria (testable)

1. **AC-1.** Dual-suite files exist at `tests/contracts/continuous_assurance_scheduler.{baseline,target}.rs` and are registered in root `Cargo.toml` in the **same implement commit**.
2. **AC-2.** Baseline GREEN on characterization SHA / current pre-implement HEAD; target RED there for **missing scheduler behavior**.
3. **AC-3.** After implement: target GREEN (CAS-001…016); baseline skip-superseded; `cargo test --workspace --features demo` green on files this slice touches.
4. **AC-4.** Public scheduler types live in `weeping-angel-assurance` (library). No new workspace crate required.
5. **AC-5.** `Clock` trait + fake clock drive due/not-due, retry, backoff, and timeout without wall-clock sleeps.
6. **AC-6.** Job contracts include cadence, freshness, dependencies, retry/backoff, timeout, optional jitter, last successful run, last attempt, next run, failure state, and idempotent run identity.
7. **AC-7.** A successful repeated slot executes Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift against the existing spine.
8. **AC-8.** Collector `Err` / timeout does not delete ledger envelopes; freshness decides whether prior evidence is evaluated.
9. **AC-9.** Duplicate `tick` for the same run identity deduplicates; crash/restart is resumable.
10. **AC-10.** Independent collectors may run concurrently without cross-run state corruption; dependent jobs respect `dependsOn`.
11. **AC-11.** Framework and control-test crates remain network-free; scheduler orchestrates collectors; collectors never set compliance results.
12. **AC-12.** `weeping-angel isms run` is optional/thin; scheduling semantics are not clap-defined. No Kubernetes, Temporal, OS cron daemon, or cloud queue is required.

---

## 8. Out of scope

- Kubernetes CronJobs, Temporal/Cadence workflows, systemd/OS cron as the core scheduler, cloud queues (SQS, Pub/Sub).
- Landing ISMS IR slices 01–12 (`IsmsContext`, scope engine, risk register, operational SoA IR, controlled documents).
- temporal assurance temporal fields (`valid_from`, `valid_until`, revocation, period effectiveness).
- ISMS events/drift event catalog and notification transport.
- Rewriting GitHub collector mapping, catalog TOML, Kleene evaluator, or ISO remap suites.
- Embedding schedule DSL in clap (`--every`, `--backoff`, crontab strings as CLI law).
- Multi-tenant SaaS control plane, UI, or hosted worker fleet.
- Changing envelope content-digest law or allowing collectors to emit `Effectiveness`.
- Product implementation in this spec-only phase (no `scheduler` module, no dual-suite `.rs`, no `[[test]]` until implement).

---

## 9. Risks

- Routing around `assess` and duplicating evaluate, producing two spines.
- Using `Utc::now()` in identity after “adding a scheduler,” which would keep duplicate-run tests red or flake.
- Treating empty `EvidenceSet` on collect failure as acceptable because lineage already records `failed` (assessment lineage) — continuous-assurance scheduler additionally requires **reattach**.
- Putting job state in evidence envelopes (ledger would own conclusions/operations).
- Reversing crate edges (collector → assurance, framework → collector).
- Target suite going RED only because `[[test]]` is missing (I3) instead of asserting behavior.
- Touching GitHub mapping / catalog TOML / remap / Kleene files and breaking neighbor suites.
- Real-thread timeouts that flake CI; fake clock + cooperative timeout flags are required.
- Inventing `IsmsContext` to “satisfy dependencies” that are not in tree.
- Clap flags becoming the schedule SSOT, blocking library/daemon reuse.

---

## 10. ADR

**Accepted.** Public library runtime seam (`Clock`, `JobSpec`, `JobState`, `InMemorySchedulerStore`, `AssuranceScheduler::tick`) and scheduled collection-failure reattach: [`docs/adr/0005-continuous-assurance-scheduler.md`](../adr/0005-continuous-assurance-scheduler.md).

Filename `0005-*` follows ADR 0004 (documentation architecture). Cite by **path**.

[`docs/specs/assurance-runtime.md`](assurance-runtime.md) records the `tick` composition.

---

## 11. Implement sequence

Landed: dual-suite + `[[test]]`; `Clock` / `JobSpec` / `JobState` / `InMemorySchedulerStore` / `tick`; Collect → Drift; ledger reattach; slot-stable run identity; retry/backoff/timeout; concurrent independent collectors; no clap schedule DSL; target GREEN; baseline skip-superseded; ADR 0005 accepted.

`weeping-angel isms run` remains unshipped (allowed; CAS-016 asserts clap is not the schedule SSOT).

---

## 12. Definition of done

The same deterministic assurance pipeline can operate **repeatedly and safely over time**, not only as a one-shot `assess`. Target suite GREEN on fake-clock CAS cases. Neighbor assurance suites remain GREEN. Framework and control-test stay network-free. Collectors still do not set compliance results.
