//! Target suite for continuous-assurance scheduler (CAS-001…016).
//!
//! Encodes DESIRED behavior in `docs/specs/continuous-assurance-scheduler.md`
//! §4 / §6. Must stay RED on current HEAD for missing scheduler behavior
//! (`Clock`, `JobSpec`, `tick`, slot-stable run identity, freshness reattach),
//! not for a missing `[[test]]` harness. Drive time with `FakeClock`. Do not
//! implement the product scheduler in this file.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use weeping_angel::cli::Cli;
use weeping_angel_assurance::scheduler::{
    AssuranceScheduler, Clock, FailureState, FakeClock, FreshnessPolicy, InMemorySchedulerStore,
    JobKind, JobSpec, PipelineStage, RetryPolicy, TickReport,
};
use weeping_angel_assurance::{AssessmentScope, compare, project_soa};
use weeping_angel_assurance_ir::{AssetId, FrameworkVersion};
use weeping_angel_collector::{
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector,
    FixtureCollector,
};
use weeping_angel_control_test::Effectiveness;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceLedger, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
};

const CADENCE: Duration = Duration::from_secs(3600);
const FRESH_MAX_AGE: Duration = Duration::from_secs(24 * 3600);
const INITIAL_BACKOFF: Duration = Duration::from_secs(60);
const MAX_BACKOFF: Duration = Duration::from_secs(240);
const TIMEOUT: Duration = Duration::from_secs(30);
const ASSET: &str = "repo:in-scope";
const MFA_TYPE: &str = "identity.privileged.mfa";
const MFA_CONTROL: &str = "control.identity.privileged-mfa";
const COLLECTOR_OK: &str = "fixture.cas-ok";
const COLLECTOR_FAIL: &str = "fixture.cas-fail";
const COLLECTOR_SLOW: &str = "fixture.cas-slow";
const COLLECTOR_A: &str = "fixture.cas-a";
const COLLECTOR_B: &str = "fixture.cas-b";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn scope() -> AssessmentScope {
    AssessmentScope::new().allow_asset(AssetId::new(ASSET))
}

fn mfa_observation(salt: &str) -> EvidenceObservation {
    EvidenceObservation::new(EvidenceType::new(MFA_TYPE))
        .with_fact("enabled", "true")
        .with_fact("salt", salt)
        .with_narrative("privileged MFA is enabled")
}

fn ok_collector(id: &str, salt: &str) -> FixtureCollector {
    FixtureCollector::new(id, "1")
        .with_evidence_types([EvidenceType::new(MFA_TYPE)])
        .with_planned(AssetId::new(ASSET), mfa_observation(salt))
}

fn seal_prior(collector_id: &str, collected_at: DateTime<Utc>, salt: &str) -> EvidenceEnvelope {
    let provenance = EvidenceProvenance {
        collector_id: collector_id.into(),
        collected_at,
        scope: ASSET.into(),
        asset: AssetId::new(ASSET),
    };
    EvidenceEnvelope::seal(mfa_observation(salt), provenance).expect("seal prior envelope")
}

struct CountingCollector {
    inner: FixtureCollector,
    collects: Arc<AtomicU32>,
}

impl CountingCollector {
    fn wrap(inner: FixtureCollector) -> (Self, Arc<AtomicU32>) {
        let collects = Arc::new(AtomicU32::new(0));
        (
            Self {
                inner,
                collects: collects.clone(),
            },
            collects,
        )
    }
}

impl EvidenceCollector for CountingCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        self.inner.descriptor()
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        self.collects.fetch_add(1, Ordering::SeqCst);
        self.inner.collect(scope)
    }
}

struct FailingCollector {
    id: String,
    collects: Arc<AtomicU32>,
}

impl FailingCollector {
    fn new(id: &str) -> (Self, Arc<AtomicU32>) {
        let collects = Arc::new(AtomicU32::new(0));
        (
            Self {
                id: id.into(),
                collects: collects.clone(),
            },
            collects,
        )
    }
}

impl EvidenceCollector for FailingCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: self.id.clone(),
            version: "1".into(),
            evidence_types: BTreeSet::from([EvidenceType::new(MFA_TYPE)]),
            provider_family: "fixture".into(),
            subject_types: BTreeSet::from(["repository".into()]),
            capabilities: CollectorCapabilities {
                offline: true,
                worker_safe: true,
                ..CollectorCapabilities::default()
            },
            required_permissions: Vec::new(),
        }
    }

    fn collect(&self, _scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        self.collects.fetch_add(1, Ordering::SeqCst);
        Err(CollectorError::InsufficientEvidence {
            detail: "forced collector failure for CAS target".into(),
        })
    }
}

/// Cooperative timeout fixture: advances the fake clock during collect.
struct ClockAdvancingCollector {
    id: String,
    clock: FakeClock,
    work: Duration,
    collects: Arc<AtomicU32>,
}

impl EvidenceCollector for ClockAdvancingCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: self.id.clone(),
            version: "1".into(),
            evidence_types: BTreeSet::from([EvidenceType::new(MFA_TYPE)]),
            provider_family: "fixture".into(),
            subject_types: BTreeSet::from(["repository".into()]),
            capabilities: CollectorCapabilities {
                offline: true,
                worker_safe: true,
                ..CollectorCapabilities::default()
            },
            required_permissions: Vec::new(),
        }
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        self.collects.fetch_add(1, Ordering::SeqCst);
        self.clock.advance(self.work);
        ok_collector(&self.id, "slow").collect(scope)
    }
}

fn collection_spec(job_id: &str, collector_id: &str, due: DateTime<Utc>) -> JobSpec {
    JobSpec::collection(job_id, collector_id)
        .cadence(CADENCE)
        .freshness(FreshnessPolicy::max_age(FRESH_MAX_AGE))
        .retry(RetryPolicy::exponential(3, INITIAL_BACKOFF, 2, MAX_BACKOFF))
        .timeout(TIMEOUT)
        .jitter(Duration::ZERO)
        .due_at(due)
}

fn test_spec(job_id: &str, depends_on: &[&str], due: DateTime<Utc>) -> JobSpec {
    JobSpec::test(job_id)
        .cadence(CADENCE)
        .freshness(FreshnessPolicy::max_age(FRESH_MAX_AGE))
        .retry(RetryPolicy::exponential(1, INITIAL_BACKOFF, 2, MAX_BACKOFF))
        .timeout(TIMEOUT)
        .jitter(Duration::ZERO)
        .depends_on(depends_on.iter().map(|s| (*s).to_string()))
        .due_at(due)
}

fn projection_spec(job_id: &str, depends_on: &[&str], due: DateTime<Utc>) -> JobSpec {
    JobSpec::projection(job_id)
        .cadence(CADENCE)
        .freshness(FreshnessPolicy::max_age(FRESH_MAX_AGE))
        .retry(RetryPolicy::exponential(1, INITIAL_BACKOFF, 2, MAX_BACKOFF))
        .timeout(TIMEOUT)
        .jitter(Duration::ZERO)
        .depends_on(depends_on.iter().map(|s| (*s).to_string()))
        .due_at(due)
}

fn snapshot_spec(job_id: &str, depends_on: &[&str], due: DateTime<Utc>) -> JobSpec {
    JobSpec::snapshot(job_id)
        .cadence(CADENCE)
        .freshness(FreshnessPolicy::max_age(FRESH_MAX_AGE))
        .retry(RetryPolicy::exponential(1, INITIAL_BACKOFF, 2, MAX_BACKOFF))
        .timeout(TIMEOUT)
        .jitter(Duration::ZERO)
        .depends_on(depends_on.iter().map(|s| (*s).to_string()))
        .due_at(due)
}

fn shared_ledger() -> Arc<Mutex<EvidenceLedger>> {
    Arc::new(Mutex::new(
        EvidenceLedger::open_in_memory().expect("open in-memory ledger"),
    ))
}

fn scheduler_builder(
    clock: FakeClock,
    store: InMemorySchedulerStore,
    ledger: Arc<Mutex<EvidenceLedger>>,
) -> weeping_angel_assurance::scheduler::AssuranceSchedulerBuilder {
    AssuranceScheduler::builder()
        .clock(clock)
        .store(store)
        .ledger(ledger)
        .framework(iso_target())
        .scope(scope())
}

fn mfa_effectiveness(report: &TickReport) -> Option<Effectiveness> {
    report
        .results
        .iter()
        .find(|r| r.control_id.as_str() == MFA_CONTROL)
        .map(|r| r.effectiveness)
}

fn network_sdk_needles() -> &'static [&'static str] {
    &[
        "reqwest",
        "octocrab",
        "hyper",
        "aws-sdk",
        "tokio-tungstenite",
        "ureq",
    ]
}

fn crate_toml(name: &str) -> String {
    read_repo_file(&format!("crates/{name}/Cargo.toml"))
}

/// CAS-001 — Dual-suite registered (harness + behavior binary).
#[test]
fn cas_001_dual_suite_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("name = \"sdd_continuous_assurance_scheduler_baseline\"")
            && toml
                .contains("path = \"tests/contracts/continuous_assurance_scheduler.baseline.rs\""),
        "baseline suite must be listed in root Cargo.toml"
    );
    assert!(
        toml.contains("name = \"sdd_continuous_assurance_scheduler_target\"")
            && toml.contains("path = \"tests/contracts/continuous_assurance_scheduler.target.rs\""),
        "target suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/continuous_assurance_scheduler.baseline.rs")
            .is_file()
            && manifest_dir()
                .join("tests/contracts/continuous_assurance_scheduler.target.rs")
                .is_file(),
        "both dual-suite files must exist"
    );
}

/// CAS-002 — Fake clock at/after nextRun → collection/test job runs once.
#[test]
fn cas_002_due_job_runs_once() {
    let clock = FakeClock::at(t0());
    assert_eq!(Clock::now(&clock), t0(), "FakeClock must implement Clock");
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "due"));
    let mut scheduler = scheduler_builder(clock.clone(), store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, t0()))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick due jobs");
    assert!(
        report.ran_jobs.iter().any(|id| id == "job.collect"),
        "due collection job must run; ran {:?}",
        report.ran_jobs
    );
    assert_eq!(
        collects.load(Ordering::SeqCst),
        1,
        "due collection must collect exactly once on the first due tick"
    );

    let state = scheduler
        .job_state("job.collect")
        .expect("collection job state");
    assert!(
        state.last_successful_run.is_some(),
        "successful due run records lastSuccessfulRun"
    );
    assert_eq!(state.failure_state, FailureState::None);
}

/// CAS-003 — Clock before nextRun → no collect, no evaluate.
#[test]
fn cas_003_not_due_does_not_collect_or_evaluate() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "not-due"));
    let due_later = t0() + chrono::Duration::from_std(CADENCE).unwrap();
    let mut scheduler = scheduler_builder(clock, store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, due_later))
        .register(test_spec("job.test", &["job.collect"], due_later))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick while not due");
    assert!(
        report.ran_jobs.is_empty(),
        "not-due tick must run no jobs; ran {:?}",
        report.ran_jobs
    );
    assert_eq!(
        collects.load(Ordering::SeqCst),
        0,
        "not-due must not collect"
    );
    assert!(
        report.results.is_empty(),
        "not-due must not evaluate control tests"
    );
    assert_eq!(
        scheduler.job_state("job.collect").expect("state").next_run,
        due_later
    );
}

/// CAS-004 — Failed collect with remaining attempts schedules another try.
#[test]
fn cas_004_retry_schedules_another_attempt() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = FailingCollector::new(COLLECTOR_FAIL);
    let mut scheduler = scheduler_builder(clock.clone(), store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_FAIL, t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick failed collect");
    assert_eq!(collects.load(Ordering::SeqCst), 1);
    let state = scheduler.job_state("job.collect").expect("state");
    assert_eq!(state.failure_state, FailureState::Retrying);
    assert!(
        state.last_attempt.is_some(),
        "failed try records lastAttempt"
    );
    assert!(
        state.last_successful_run.is_none(),
        "no successful run after the first failure"
    );
    assert_eq!(
        state.next_run,
        t0() + chrono::Duration::from_std(INITIAL_BACKOFF).unwrap(),
        "remaining attempts must schedule a retry"
    );
    assert!(
        report.ran_jobs.iter().any(|id| id == "job.collect"),
        "the failed attempt still counts as a ran job"
    );
}

/// CAS-005 — Next attempt time is exponential with ceiling on the fake clock.
#[test]
fn cas_005_backoff_is_exponential_with_ceiling() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = FailingCollector::new(COLLECTOR_FAIL);
    let mut scheduler = scheduler_builder(clock.clone(), store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_FAIL, t0()))
        .build()
        .expect("build scheduler");

    scheduler.tick().expect("attempt 1");
    let after_first = scheduler.job_state("job.collect").unwrap().next_run;
    assert_eq!(
        after_first,
        t0() + chrono::Duration::from_std(INITIAL_BACKOFF).unwrap(),
        "attempt 1 backoff is the initial interval"
    );

    clock.advance(INITIAL_BACKOFF);
    scheduler.tick().expect("attempt 2");
    let after_second = scheduler.job_state("job.collect").unwrap().next_run;
    let expected_second = INITIAL_BACKOFF * 2;
    assert_eq!(
        after_second,
        Clock::now(&clock) + chrono::Duration::from_std(expected_second).unwrap(),
        "attempt 2 backoff doubles"
    );

    clock.advance(expected_second);
    scheduler.tick().expect("attempt 3");
    let exhausted = scheduler.job_state("job.collect").unwrap();
    assert_eq!(exhausted.failure_state, FailureState::FailedExhausted);
    assert_eq!(collects.load(Ordering::SeqCst), 3);
    assert_eq!(
        Clock::now(&clock).signed_duration_since(t0()),
        chrono::Duration::seconds(180),
        "backoff is measured on FakeClock, not wall-clock sleep"
    );
}

/// CAS-006 — Attempt that exceeds timeout on the clock is timed_out.
#[test]
fn cas_006_timeout_marks_attempt_without_hanging_tick() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let collects = Arc::new(AtomicU32::new(0));
    let collector = ClockAdvancingCollector {
        id: COLLECTOR_SLOW.into(),
        clock: clock.clone(),
        work: Duration::from_secs(90),
        collects: collects.clone(),
    };
    let prior = seal_prior(COLLECTOR_SLOW, t0() - chrono::Duration::hours(1), "prior");
    let prior_digest = prior.digest().to_string();
    ledger.lock().unwrap().append(prior).expect("seed prior");

    let mut scheduler = scheduler_builder(clock.clone(), store, ledger.clone())
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_SLOW, t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick must return on timeout");
    assert_eq!(collects.load(Ordering::SeqCst), 1);
    let state = scheduler.job_state("job.collect").expect("state");
    assert_eq!(state.failure_state, FailureState::TimedOut);
    assert!(
        report.timed_out_jobs.iter().any(|id| id == "job.collect")
            || state.last_attempt.as_ref().is_some_and(|a| a.timed_out()),
        "timeout must be visible on the tick report or lastAttempt"
    );
    let still = ledger.lock().unwrap().query().expect("query");
    assert!(
        still.iter().any(|e| e.digest() == prior_digest),
        "timed-out collect must not delete prior envelopes"
    );
}

/// CAS-007 — Persist in-flight identity; restart with same store + slot does not double-apply.
#[test]
fn cas_007_crash_restart_does_not_double_apply() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "crash"));
    let mut scheduler = scheduler_builder(clock.clone(), store.clone(), ledger.clone())
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, t0()))
        .build()
        .expect("build scheduler");

    let first = scheduler.tick().expect("first tick");
    let run_id = first
        .run_ids
        .first()
        .cloned()
        .expect("successful slot assigns a run identity");
    let envelopes_after_first = ledger.lock().unwrap().query().expect("query").len();
    assert_eq!(collects.load(Ordering::SeqCst), 1);
    drop(scheduler);

    let (collector2, collects2) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "crash"));
    let mut restarted = scheduler_builder(clock, store, ledger.clone())
        .collector(collector2)
        .register(collection_spec("job.collect", COLLECTOR_OK, t0()))
        .build()
        .expect("restore scheduler from the same store");
    let second = restarted.tick().expect("restart tick same slot");
    assert_eq!(
        collects2.load(Ordering::SeqCst),
        0,
        "restart in the same slot must not collect again"
    );
    assert!(
        second.run_ids.is_empty() || second.run_ids.iter().all(|id| id == &run_id),
        "restart returns the existing run identity, not a new wall-clock id"
    );
    assert_eq!(
        ledger.lock().unwrap().query().expect("query").len(),
        envelopes_after_first,
        "restart must not double-append envelopes"
    );
}

/// CAS-008 — Two ticks for the same identity return one run id; ledger does not double.
#[test]
fn cas_008_duplicate_tick_dedupes_run_identity() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, collects) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "dup"));
    let spec = collection_spec("job.collect", COLLECTOR_OK, t0());
    let mut scheduler = scheduler_builder(clock.clone(), store, ledger.clone())
        .collector(collector)
        .register(spec.clone())
        .build()
        .expect("build scheduler");

    let first = scheduler.tick().expect("tick 1");
    let second = scheduler.tick().expect("tick 2 same slot");
    assert_eq!(
        collects.load(Ordering::SeqCst),
        1,
        "duplicate tick must not collect"
    );
    assert_eq!(first.run_ids.len(), 1);
    assert_eq!(
        first.run_ids, second.run_ids,
        "duplicate tick returns the same slot-stable run identity"
    );
    let expected = spec.run_identity(t0(), COLLECTOR_OK, spec.configuration_digest());
    assert_eq!(
        first.run_ids[0], expected,
        "run identity is a canonical digest of job+slot+config, not Utc::now uniqueness"
    );
    assert_eq!(
        ledger.lock().unwrap().query().expect("query").len(),
        1,
        "duplicate tick must not double ledger envelopes"
    );
}

/// CAS-009 — Evaluate/project/snapshot do not run before required collection terminal state.
#[test]
fn cas_009_depends_on_waits_for_collection_terminal_state() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let due_later = t0() + chrono::Duration::from_std(CADENCE).unwrap();
    let (collector, collects) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "dep"));
    let mut scheduler = scheduler_builder(clock, store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, due_later))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .register(projection_spec("job.project", &["job.test"], t0()))
        .register(snapshot_spec("job.snapshot", &["job.project"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick with blocked dependents");
    assert_eq!(collects.load(Ordering::SeqCst), 0);
    for blocked in ["job.test", "job.project", "job.snapshot"] {
        assert!(
            !report.ran_jobs.iter().any(|id| id == blocked),
            "{blocked} must wait on collection terminal state; ran {:?}",
            report.ran_jobs
        );
    }
    assert!(
        !report.stages.iter().any(|s| matches!(
            s,
            PipelineStage::Evaluate
                | PipelineStage::Project
                | PipelineStage::Snapshot
                | PipelineStage::Drift
        )),
        "evaluate/project/snapshot/drift must not run before collection; stages {:?}",
        report.stages
    );
}

/// CAS-010 — Independent collectors both complete; envelopes isolated by run id.
#[test]
fn cas_010_concurrent_independent_collectors_do_not_cross_runs() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (a, collects_a) = CountingCollector::wrap(ok_collector(COLLECTOR_A, "a"));
    let (b, collects_b) = CountingCollector::wrap(ok_collector(COLLECTOR_B, "b"));
    let mut scheduler = scheduler_builder(clock, store, ledger.clone())
        .collector(a)
        .collector(b)
        .register(collection_spec("job.a", COLLECTOR_A, t0()))
        .register(collection_spec("job.b", COLLECTOR_B, t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick independent collectors");
    assert_eq!(collects_a.load(Ordering::SeqCst), 1);
    assert_eq!(collects_b.load(Ordering::SeqCst), 1);
    assert!(report.ran_jobs.iter().any(|id| id == "job.a"));
    assert!(report.ran_jobs.iter().any(|id| id == "job.b"));
    assert_eq!(report.run_ids.len(), 2);
    assert_ne!(report.run_ids[0], report.run_ids[1]);

    let envelopes = ledger.lock().unwrap().query().expect("query");
    assert_eq!(envelopes.len(), 2);
    let run_a = report
        .collection_runs
        .iter()
        .find(|r| r.collector_id == COLLECTOR_A)
        .expect("collection run for A");
    let run_b = report
        .collection_runs
        .iter()
        .find(|r| r.collector_id == COLLECTOR_B)
        .expect("collection run for B");
    assert_ne!(run_a.run_id, run_b.run_id);
    assert_eq!(run_a.collector_id, COLLECTOR_A);
    assert_eq!(run_b.collector_id, COLLECTOR_B);
    for env in &envelopes {
        let cid = env.provenance().collector_id.as_str();
        let rid = env.collection_run_id();
        if cid == COLLECTOR_A {
            assert_eq!(
                rid,
                run_a.run_id.as_str(),
                "A envelopes must not carry B's run id"
            );
        } else if cid == COLLECTOR_B {
            assert_eq!(
                rid,
                run_b.run_id.as_str(),
                "B envelopes must not carry A's run id"
            );
        } else {
            panic!("unexpected collector {cid}");
        }
    }
}

/// CAS-011 — Failed collect + prior envelope older than freshness → StaleEvidence; digest remains.
#[test]
fn cas_011_stale_previous_evidence_is_not_erased() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let stale_at = t0() - chrono::Duration::hours(48);
    let prior = seal_prior(COLLECTOR_FAIL, stale_at, "stale");
    let digest = prior.digest().to_string();
    ledger
        .lock()
        .unwrap()
        .append(prior)
        .expect("seed stale prior");

    let (collector, _) = FailingCollector::new(COLLECTOR_FAIL);
    let mut scheduler = scheduler_builder(clock, store, ledger.clone())
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_FAIL, t0()))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick with stale prior");
    assert_eq!(
        mfa_effectiveness(&report),
        Some(Effectiveness::StaleEvidence),
        "stale prior evidence must evaluate as StaleEvidence, not an empty-set miss"
    );
    let still = ledger.lock().unwrap().query().expect("query");
    assert!(
        still.iter().any(|e| e.digest() == digest),
        "failed collect must not delete the stale prior envelope"
    );
}

/// CAS-012 — Failed collect + prior envelope within freshness → evaluate uses that envelope.
#[test]
fn cas_012_fresh_previous_evidence_is_reattached() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let fresh_at = t0() - chrono::Duration::hours(1);
    let prior = seal_prior(COLLECTOR_FAIL, fresh_at, "fresh");
    let digest = prior.digest().to_string();
    ledger
        .lock()
        .unwrap()
        .append(prior)
        .expect("seed fresh prior");

    let (collector, _) = FailingCollector::new(COLLECTOR_FAIL);
    let mut scheduler = scheduler_builder(clock, store, ledger.clone())
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_FAIL, t0()))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick with fresh prior");
    assert_eq!(
        mfa_effectiveness(&report),
        Some(Effectiveness::Effective),
        "fresh prior evidence must be reattached and evaluated; got {:?}",
        mfa_effectiveness(&report)
    );
    assert!(
        report.evidence_count >= 1,
        "reattached evidence must be visible on the tick"
    );
    let still = ledger.lock().unwrap().query().expect("query");
    assert!(
        still.iter().any(|e| e.digest() == digest),
        "failed collect must not delete the fresh prior envelope"
    );
}

/// CAS-013 — Successful slot records Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift.
#[test]
fn cas_013_successful_slot_records_pipeline_order() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, _) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "pipe"));
    let mut scheduler = scheduler_builder(clock, store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, t0()))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .register(projection_spec("job.project", &["job.test"], t0()))
        .register(snapshot_spec("job.snapshot", &["job.project"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("successful slot");
    let expected = [
        PipelineStage::Collect,
        PipelineStage::Normalize,
        PipelineStage::Seal,
        PipelineStage::Ledger,
        PipelineStage::Evaluate,
        PipelineStage::Project,
        PipelineStage::Snapshot,
        PipelineStage::Drift,
    ];
    assert_eq!(
        report.stages, expected,
        "successful slot must record the eight spine stages in order"
    );
    assert!(
        report.ran_jobs.iter().any(|id| id == "job.collect")
            && report.ran_jobs.iter().any(|id| id == "job.test")
            && report.ran_jobs.iter().any(|id| id == "job.project")
            && report.ran_jobs.iter().any(|id| id == "job.snapshot"),
        "DAG kinds collection/test/projection/snapshot must all run in the slot; ran {:?}",
        report.ran_jobs
    );
    let _existing_spine = (project_soa, compare);
    let _kinds = [
        JobKind::Collection,
        JobKind::Test,
        JobKind::Projection,
        JobKind::Snapshot,
    ];
}

/// CAS-014 — Framework and control-test crates remain network-free; no scheduler import.
#[test]
fn cas_014_framework_and_control_test_remain_network_free() {
    let framework_toml = crate_toml("weeping-angel-framework");
    let control_toml = crate_toml("weeping-angel-control-test");
    let collector_toml = crate_toml("weeping-angel-collector");
    for needle in network_sdk_needles() {
        assert!(
            !framework_toml.contains(needle),
            "framework Cargo.toml must stay network-free; found `{needle}`"
        );
        assert!(
            !control_toml.contains(needle),
            "control-test Cargo.toml must stay network-free; found `{needle}`"
        );
    }
    assert!(
        !framework_toml.contains("weeping-angel-collector"),
        "framework must not depend on collectors"
    );
    assert!(
        !control_toml.contains("weeping-angel-collector"),
        "control-test must not depend on collectors"
    );
    for toml in [&framework_toml, &control_toml, &collector_toml] {
        assert!(
            !toml.lines().any(|line| {
                let t = line.trim();
                t.contains("weeping-angel-assurance") && !t.contains("weeping-angel-assurance-ir")
            }),
            "scheduler must orchestrate collectors; collectors/framework/control-test must not depend on weeping-angel-assurance"
        );
    }
}

/// CAS-015 — Fixture collector cannot set Effectiveness; results come from evaluate.
#[test]
fn cas_015_collectors_never_set_effectiveness() {
    let clock = FakeClock::at(t0());
    let store = InMemorySchedulerStore::new();
    let ledger = shared_ledger();
    let (collector, _) = CountingCollector::wrap(ok_collector(COLLECTOR_OK, "blind"));
    let mut scheduler = scheduler_builder(clock, store, ledger)
        .collector(collector)
        .register(collection_spec("job.collect", COLLECTOR_OK, t0()))
        .register(test_spec("job.test", &["job.collect"], t0()))
        .build()
        .expect("build scheduler");

    let report = scheduler.tick().expect("tick");
    let mfa = report
        .results
        .iter()
        .find(|r| r.control_id.as_str() == MFA_CONTROL)
        .expect("evaluate must emit the privileged-MFA result");
    assert_eq!(mfa.effectiveness, Effectiveness::Effective);
    assert!(
        !report
            .collection_runs
            .iter()
            .any(|run| run.status == "effective" || run.status == "compliant"),
        "collectors record collection status, never compliance Effectiveness"
    );

    let collector_src = {
        let mut out = String::new();
        fn walk(dir: &std::path::Path, out: &mut String) {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if entry.file_type().unwrap().is_dir() {
                    walk(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    out.push_str(&fs::read_to_string(&path).unwrap());
                    out.push('\n');
                }
            }
        }
        walk(
            &manifest_dir().join("crates/weeping-angel-collector/src"),
            &mut out,
        );
        out
    };
    assert!(
        !collector_src.contains("Effectiveness::") && !collector_src.contains("set_compliant"),
        "collector crate must not write compliance Effectiveness"
    );
}

/// CAS-016 — If `isms run` exists, clap is not the schedule SSOT.
#[test]
fn cas_016_cli_is_thin_and_does_not_define_schedule_semantics() {
    let cli_src = read_repo_file("src/cli.rs");
    for flag in ["--cadence", "--backoff", "--jitter", "--timeout"] {
        assert!(
            !cli_src.contains(flag),
            "cadence/retry/backoff/jitter/timeout must not be clap-defined; found {flag}"
        );
    }

    let names: Vec<String> = Cli::clap_command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    if names.iter().any(|n| n == "isms") {
        let parsed = Cli::try_parse_from(["weeping-angel", "isms", "run"]);
        assert!(
            parsed.is_ok(),
            "when isms exists it must expose a thin `run` that calls the library"
        );
        let command = Cli::clap_command();
        if let Some(run) = command
            .find_subcommand("isms")
            .and_then(|c| c.find_subcommand("run"))
        {
            for flag in ["cadence", "backoff", "jitter", "timeout"] {
                assert!(
                    !run.get_arguments().any(|a| a.get_id() == flag
                        || a.get_long() == Some(flag)
                        || a.get_all_aliases()
                            .into_iter()
                            .flatten()
                            .any(|al| al == flag)),
                    "isms run must not own `{flag}` as a clap argument"
                );
            }
        }
    }
}
