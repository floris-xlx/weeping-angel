//! Library-first continuous assurance scheduler.
//!
//! Orchestrates Collect → Normalize → Seal → Ledger → Evaluate → Project →
//! Snapshot → Drift over existing APIs. Time is injected via [`Clock`]; tests
//! use [`FakeClock`]. Collectors never set compliance results.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use thiserror::Error;

use weeping_angel_assurance_ir::{AssessmentId, canonical_digest};
use weeping_angel_collector::{CollectorScope, EvidenceCollector};
use weeping_angel_control_test::{AssessmentContext, ControlTestResult, EvidenceSet};
use weeping_angel_evidence::{CollectionRun, EvidenceLedger, LedgerError};
use weeping_angel_framework::{Assessment, CompiledFramework, FrameworkTarget, compile_framework};

use crate::lineage::{
    assessment_result_digest, assessment_summary, coverage_metrics, definition_snapshot,
    seal_evidence_snapshot, snapshot_applicability,
};
use crate::readiness::{FrameworkReadinessSnapshot, project_readiness};
use crate::snapshot::{AssessmentRun, compare};
use crate::{
    AssessmentScope, AssuranceError, assessment_for_target, evaluate_compiled, project_soa,
};

const ATTEMPT_POLICY_VERSION: &str = "v1";

/// Injected time seam. Framework and control-test crates do not take this trait.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Production UTC clock.
#[derive(Debug, Clone, Default)]
pub struct UtcClock;

impl Clock for UtcClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Deterministic clock for scheduler tests. Shared across clones.
#[derive(Debug, Clone)]
pub struct FakeClock {
    now: Arc<Mutex<DateTime<Utc>>>,
}

impl FakeClock {
    pub fn at(now: DateTime<Utc>) -> Self {
        Self {
            now: Arc::new(Mutex::new(now)),
        }
    }

    pub fn advance(&self, by: Duration) {
        let mut guard = self.now.lock().expect("fake clock");
        *guard += chrono::Duration::from_std(by).unwrap_or_else(|_| chrono::Duration::zero());
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("fake clock")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    Collection,
    Test,
    Projection,
    Snapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Collect,
    Normalize,
    Seal,
    Ledger,
    Evaluate,
    Project,
    Snapshot,
    Drift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureState {
    None,
    Retrying,
    TimedOut,
    FailedExhausted,
    Crashed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessPolicy {
    pub max_age: Duration,
}

impl FreshnessPolicy {
    pub fn max_age(max_age: Duration) -> Self {
        Self { max_age }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial: Duration,
    pub multiplier: u32,
    pub ceiling: Duration,
}

impl RetryPolicy {
    pub fn exponential(
        max_attempts: u32,
        initial: Duration,
        multiplier: u32,
        ceiling: Duration,
    ) -> Self {
        Self {
            max_attempts,
            initial,
            multiplier: multiplier.max(1),
            ceiling,
        }
    }

    fn delay_after(&self, completed_attempts: u32) -> Duration {
        let mut delay = self.initial;
        for _ in 1..completed_attempts.max(1) {
            delay = delay.saturating_mul(self.multiplier);
            if delay > self.ceiling {
                return self.ceiling;
            }
        }
        delay.min(self.ceiling)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRef {
    pub id: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRef {
    pub id: String,
    pub at: DateTime<Utc>,
    timed_out: bool,
}

impl AttemptRef {
    pub fn timed_out(&self) -> bool {
        self.timed_out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    pub job_id: String,
    pub kind: JobKind,
    collector_id: Option<String>,
    cadence: Duration,
    freshness: FreshnessPolicy,
    retry: RetryPolicy,
    timeout: Duration,
    jitter: Duration,
    depends_on: Vec<String>,
    origin: DateTime<Utc>,
    due_at: DateTime<Utc>,
}

impl JobSpec {
    fn blank(job_id: impl Into<String>, kind: JobKind) -> Self {
        Self {
            job_id: job_id.into(),
            kind,
            collector_id: None,
            cadence: Duration::from_secs(3600),
            freshness: FreshnessPolicy::max_age(Duration::from_secs(24 * 3600)),
            retry: RetryPolicy::exponential(
                1,
                Duration::from_secs(60),
                2,
                Duration::from_secs(240),
            ),
            timeout: Duration::from_secs(30),
            jitter: Duration::ZERO,
            depends_on: Vec::new(),
            origin: DateTime::<Utc>::UNIX_EPOCH,
            due_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    pub fn collection(job_id: impl Into<String>, collector_id: impl Into<String>) -> Self {
        let mut spec = Self::blank(job_id, JobKind::Collection);
        spec.collector_id = Some(collector_id.into());
        spec
    }

    pub fn test(job_id: impl Into<String>) -> Self {
        Self::blank(job_id, JobKind::Test)
    }

    pub fn projection(job_id: impl Into<String>) -> Self {
        Self::blank(job_id, JobKind::Projection)
    }

    pub fn snapshot(job_id: impl Into<String>) -> Self {
        Self::blank(job_id, JobKind::Snapshot)
    }

    pub fn cadence(mut self, cadence: Duration) -> Self {
        self.cadence = cadence;
        self
    }

    pub fn freshness(mut self, freshness: FreshnessPolicy) -> Self {
        self.freshness = freshness;
        self
    }

    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn jitter(mut self, jitter: Duration) -> Self {
        self.jitter = jitter;
        self
    }

    pub fn depends_on(mut self, ids: impl IntoIterator<Item = String>) -> Self {
        self.depends_on = ids.into_iter().collect();
        self
    }

    pub fn due_at(mut self, due: DateTime<Utc>) -> Self {
        self.origin = due;
        self.due_at = due;
        self
    }

    pub fn configuration_digest(&self) -> String {
        let body = (
            self.job_id.as_str(),
            format!("{:?}", self.kind),
            self.collector_id.as_deref().unwrap_or(""),
            self.cadence.as_millis() as u64,
            self.freshness.max_age.as_millis() as u64,
            self.retry.max_attempts,
            self.retry.initial.as_millis() as u64,
            self.retry.multiplier,
            self.retry.ceiling.as_millis() as u64,
            self.timeout.as_millis() as u64,
            self.jitter.as_millis() as u64,
            &self.depends_on,
            ATTEMPT_POLICY_VERSION,
        );
        canonical_digest(&body).unwrap_or_else(|_| "0".repeat(16))
    }

    pub fn run_identity(
        &self,
        now: DateTime<Utc>,
        collector_id: impl AsRef<str>,
        configuration_digest: impl AsRef<str>,
    ) -> String {
        let collector_id = collector_id.as_ref();
        let configuration_digest = configuration_digest.as_ref();
        let slot = self.slot_for(now);
        let body = (
            self.job_id.as_str(),
            slot.to_rfc3339(),
            collector_id,
            configuration_digest,
            ATTEMPT_POLICY_VERSION,
        );
        let digest = canonical_digest(&body).unwrap_or_else(|_| "0".repeat(32));
        format!("run:{}", &digest[..32.min(digest.len())])
    }

    fn slot_for(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        let cadence_ms = self.cadence.as_millis() as i64;
        if cadence_ms <= 0 {
            return self.origin;
        }
        let elapsed = now.signed_duration_since(self.origin).num_milliseconds();
        if elapsed < 0 {
            return self.origin;
        }
        let slots = elapsed / cadence_ms;
        self.origin + chrono::Duration::milliseconds(slots * cadence_ms)
    }

    fn collector_id(&self) -> &str {
        self.collector_id.as_deref().unwrap_or("")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobState {
    pub last_successful_run: Option<RunRef>,
    pub last_attempt: Option<AttemptRef>,
    pub next_run: DateTime<Utc>,
    pub failure_state: FailureState,
    attempt_count: u32,
    current_slot: Option<String>,
}

impl JobState {
    fn fresh(next_run: DateTime<Utc>) -> Self {
        Self {
            last_successful_run: None,
            last_attempt: None,
            next_run,
            failure_state: FailureState::None,
            attempt_count: 0,
            current_slot: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotStatus {
    InFlight,
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone)]
struct SlotRecord {
    run_id: String,
    status: SlotStatus,
}

#[derive(Debug, Clone, Default)]
struct StoreInner {
    jobs: BTreeMap<String, JobState>,
    slots: BTreeMap<String, SlotRecord>,
    last_readiness: Option<FrameworkReadinessSnapshot>,
}

/// Shared in-memory persistence for job/run operational state (not envelopes).
#[derive(Debug, Clone, Default)]
pub struct InMemorySchedulerStore {
    inner: Arc<Mutex<StoreInner>>,
}

impl InMemorySchedulerStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_job(&self, job_id: &str) -> Option<JobState> {
        self.inner.lock().expect("store").jobs.get(job_id).cloned()
    }

    fn save_job(&self, job_id: &str, state: JobState) {
        self.inner
            .lock()
            .expect("store")
            .jobs
            .insert(job_id.to_string(), state);
    }

    fn load_slot(&self, run_id: &str) -> Option<SlotRecord> {
        self.inner.lock().expect("store").slots.get(run_id).cloned()
    }

    fn save_slot(&self, record: SlotRecord) {
        self.inner
            .lock()
            .expect("store")
            .slots
            .insert(record.run_id.clone(), record);
    }

    fn last_readiness(&self) -> Option<FrameworkReadinessSnapshot> {
        self.inner.lock().expect("store").last_readiness.clone()
    }

    fn save_readiness(&self, snap: FrameworkReadinessSnapshot) {
        self.inner.lock().expect("store").last_readiness = Some(snap);
    }
}

#[derive(Debug, Clone, Default)]
pub struct TickReport {
    pub ran_jobs: Vec<String>,
    pub run_ids: Vec<String>,
    pub timed_out_jobs: Vec<String>,
    pub stages: Vec<PipelineStage>,
    pub results: Vec<ControlTestResult>,
    pub collection_runs: Vec<CollectionRun>,
    pub evidence_count: usize,
}

impl TickReport {
    fn push_stage(&mut self, stage: PipelineStage) {
        if !self.stages.contains(&stage) {
            self.stages.push(stage);
        }
    }

    fn push_run_id(&mut self, run_id: String) {
        if !self.run_ids.contains(&run_id) {
            self.run_ids.push(run_id);
        }
    }
}

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("scheduler is missing a clock")]
    MissingClock,
    #[error("scheduler is missing a store")]
    MissingStore,
    #[error("scheduler is missing a ledger")]
    MissingLedger,
    #[error("scheduler is missing a framework target")]
    MissingFramework,
    #[error("unknown collector {0}")]
    UnknownCollector(String),
    #[error(transparent)]
    Assurance(#[from] AssuranceError),
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

#[derive(Default)]
pub struct AssuranceSchedulerBuilder {
    clock: Option<Arc<dyn Clock>>,
    store: Option<InMemorySchedulerStore>,
    ledger: Option<Arc<Mutex<EvidenceLedger>>>,
    collectors: Vec<Arc<dyn EvidenceCollector + Send + Sync>>,
    specs: Vec<JobSpec>,
    target: Option<FrameworkTarget>,
    scope: Option<AssessmentScope>,
}

impl AssuranceSchedulerBuilder {
    pub fn clock(mut self, clock: impl Clock + 'static) -> Self {
        self.clock = Some(Arc::new(clock));
        self
    }

    pub fn store(mut self, store: InMemorySchedulerStore) -> Self {
        self.store = Some(store);
        self
    }

    pub fn ledger(mut self, ledger: Arc<Mutex<EvidenceLedger>>) -> Self {
        self.ledger = Some(ledger);
        self
    }

    pub fn framework(mut self, target: FrameworkTarget) -> Self {
        self.target = Some(target);
        self
    }

    pub fn scope(mut self, scope: AssessmentScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn collector(mut self, collector: impl EvidenceCollector + Send + Sync + 'static) -> Self {
        self.collectors.push(Arc::new(collector));
        self
    }

    pub fn register(mut self, spec: JobSpec) -> Self {
        self.specs.push(spec);
        self
    }

    pub fn build(self) -> Result<AssuranceScheduler, SchedulerError> {
        let clock = self.clock.ok_or(SchedulerError::MissingClock)?;
        let store = self.store.ok_or(SchedulerError::MissingStore)?;
        let ledger = self.ledger.ok_or(SchedulerError::MissingLedger)?;
        let target = self.target.ok_or(SchedulerError::MissingFramework)?;
        let scope = self.scope.unwrap_or_default();
        let assessment = assessment_for_target(&target)?;
        let compiled = compile_framework(&assessment, &target).map_err(AssuranceError::Compile)?;

        let mut collectors = BTreeMap::new();
        for collector in self.collectors {
            let id = collector.descriptor().id.clone();
            collectors.insert(id, collector);
        }

        for spec in &self.specs {
            if store.load_job(&spec.job_id).is_none() {
                store.save_job(&spec.job_id, JobState::fresh(spec.due_at));
            }
        }

        Ok(AssuranceScheduler {
            clock,
            store,
            ledger,
            collectors,
            specs: self.specs,
            compiled,
            assessment,
            target,
            scope,
        })
    }
}

pub struct AssuranceScheduler {
    clock: Arc<dyn Clock>,
    store: InMemorySchedulerStore,
    ledger: Arc<Mutex<EvidenceLedger>>,
    collectors: BTreeMap<String, Arc<dyn EvidenceCollector + Send + Sync>>,
    specs: Vec<JobSpec>,
    compiled: CompiledFramework,
    assessment: Assessment,
    target: FrameworkTarget,
    scope: AssessmentScope,
}

impl AssuranceScheduler {
    pub fn builder() -> AssuranceSchedulerBuilder {
        AssuranceSchedulerBuilder::default()
    }

    pub fn job_state(&self, job_id: &str) -> Option<JobState> {
        self.store.load_job(job_id)
    }

    pub fn tick(&mut self) -> Result<TickReport, SchedulerError> {
        let now = self.clock.now();
        let mut report = TickReport::default();

        let collection_jobs: Vec<JobSpec> = self
            .specs
            .iter()
            .filter(|s| s.kind == JobKind::Collection)
            .cloned()
            .collect();
        let due_collections: Vec<JobSpec> = collection_jobs
            .iter()
            .filter(|spec| self.is_due(spec, now) && self.deps_ready(spec))
            .cloned()
            .collect();

        let outcomes = self.run_collections(&due_collections, now)?;
        for outcome in &outcomes {
            if !outcome.skipped {
                report.ran_jobs.push(outcome.job_id.clone());
            }
            report.push_run_id(outcome.run_id.clone());
            report.collection_runs.push(outcome.run.clone());
            if outcome.timed_out {
                report.timed_out_jobs.push(outcome.job_id.clone());
            }
            if outcome.did_collect_pipeline {
                report.push_stage(PipelineStage::Collect);
                report.push_stage(PipelineStage::Normalize);
                report.push_stage(PipelineStage::Seal);
                report.push_stage(PipelineStage::Ledger);
            } else if outcome.attempted {
                report.push_stage(PipelineStage::Collect);
            }
        }

        for spec in &collection_jobs {
            let run_id = spec.run_identity(now, spec.collector_id(), spec.configuration_digest());
            if let Some(slot) = self.store.load_slot(&run_id)
                && slot.status == SlotStatus::Succeeded
            {
                report.push_run_id(run_id);
            }
        }

        let mut evidence = self.evidence_for_slot(&collection_jobs)?;
        for exception in &self.assessment.exceptions {
            evidence.insert_exception(exception.clone());
        }
        report.evidence_count = evidence.len();

        let test_jobs: Vec<JobSpec> = self
            .specs
            .iter()
            .filter(|s| s.kind == JobKind::Test)
            .cloned()
            .collect();
        let mut evaluated = false;
        for spec in &test_jobs {
            if self.is_due(spec, now) && self.deps_ready(spec) {
                self.run_evaluate(spec, &evidence, now, &mut report)?;
                evaluated = true;
            }
        }

        if evaluated {
            for spec in self
                .specs
                .iter()
                .filter(|s| s.kind == JobKind::Projection)
                .cloned()
                .collect::<Vec<_>>()
            {
                if self.is_due(&spec, now) && self.deps_ready(&spec) {
                    self.run_project(&spec, now, &mut report)?;
                }
            }
            for spec in self
                .specs
                .iter()
                .filter(|s| s.kind == JobKind::Snapshot)
                .cloned()
                .collect::<Vec<_>>()
            {
                if self.is_due(&spec, now) && self.deps_ready(&spec) {
                    self.run_snapshot(&spec, now, &mut report)?;
                }
            }
        } else {
            // Projection/snapshot still respect dependsOn even if evaluate did not run.
            for spec in &self.specs {
                if !matches!(spec.kind, JobKind::Projection | JobKind::Snapshot) {
                    continue;
                }
                if self.is_due(spec, now) && self.deps_ready(spec) {
                    match spec.kind {
                        JobKind::Projection => self.run_project(spec, now, &mut report)?,
                        JobKind::Snapshot => self.run_snapshot(spec, now, &mut report)?,
                        _ => {}
                    }
                }
            }
        }

        Ok(report)
    }

    fn is_due(&self, spec: &JobSpec, now: DateTime<Utc>) -> bool {
        self.store
            .load_job(&spec.job_id)
            .map(|s| now >= s.next_run)
            .unwrap_or(now >= spec.due_at)
    }

    fn deps_ready(&self, spec: &JobSpec) -> bool {
        spec.depends_on.iter().all(|dep| self.dep_terminal(dep))
    }

    fn dep_terminal(&self, job_id: &str) -> bool {
        let Some(state) = self.store.load_job(job_id) else {
            return false;
        };
        state.last_attempt.is_some() || state.last_successful_run.is_some()
    }

    fn run_collections(
        &self,
        due: &[JobSpec],
        now: DateTime<Utc>,
    ) -> Result<Vec<CollectionOutcome>, SchedulerError> {
        if due.is_empty() {
            return Ok(Vec::new());
        }
        let mut outcomes = Vec::with_capacity(due.len());
        thread::scope(|scope| {
            let mut joins = Vec::new();
            for spec in due {
                let clock = Arc::clone(&self.clock);
                let store = self.store.clone();
                let ledger = Arc::clone(&self.ledger);
                let spec = spec.clone();
                let collector_id = spec.collector_id().to_string();
                let collector = self.collectors.get(&collector_id).cloned();
                let collector_scope = self.scope.to_collector_scope();
                joins.push(scope.spawn(move || {
                    collect_job(clock, store, ledger, spec, collector, collector_scope, now)
                }));
            }
            for join in joins {
                match join.join() {
                    Ok(result) => outcomes.push(result),
                    Err(_) => outcomes.push(Err(SchedulerError::Assurance(
                        AssuranceError::MissingCollector,
                    ))),
                }
            }
        });
        outcomes.into_iter().collect()
    }

    fn evidence_for_slot(
        &self,
        collection_jobs: &[JobSpec],
    ) -> Result<EvidenceSet, SchedulerError> {
        let ledger = self.ledger.lock().expect("ledger");
        let all = ledger.query()?;
        drop(ledger);
        let mut set = EvidenceSet::new();
        if collection_jobs.is_empty() {
            for env in all {
                set.insert(env);
            }
            return Ok(set);
        }
        let collector_ids: BTreeSet<&str> = collection_jobs
            .iter()
            .map(|s| s.collector_id())
            .filter(|id| !id.is_empty())
            .collect();
        for env in all {
            if collector_ids.is_empty()
                || collector_ids.contains(env.provenance().collector_id.as_str())
            {
                set.insert(env);
            }
        }
        Ok(set)
    }

    fn run_evaluate(
        &self,
        spec: &JobSpec,
        evidence: &EvidenceSet,
        now: DateTime<Utc>,
        report: &mut TickReport,
    ) -> Result<(), SchedulerError> {
        let ctx = AssessmentContext {
            now,
            max_age: spec.freshness.max_age,
        };
        let results = evaluate_compiled(&self.compiled, evidence, &ctx);
        report.results = results;
        report.evidence_count = evidence.len();
        report.ran_jobs.push(spec.job_id.clone());
        report.push_stage(PipelineStage::Evaluate);
        self.mark_success(spec, now);
        Ok(())
    }

    fn run_project(
        &self,
        spec: &JobSpec,
        now: DateTime<Utc>,
        report: &mut TickReport,
    ) -> Result<(), SchedulerError> {
        let framework = self.target.profile.as_selector();
        let version = self.target.version.as_str();
        let _soa = project_soa(framework, version);
        let pack_digest = self.compiled.framework_pack_digest.clone();
        let snap = project_readiness(
            &self.compiled,
            &report.results,
            framework,
            version,
            &pack_digest,
            self.assessment.id.clone(),
        );
        self.store.save_readiness(snap);
        report.ran_jobs.push(spec.job_id.clone());
        report.push_stage(PipelineStage::Project);
        self.mark_success(spec, now);
        Ok(())
    }

    fn run_snapshot(
        &self,
        spec: &JobSpec,
        now: DateTime<Utc>,
        report: &mut TickReport,
    ) -> Result<(), SchedulerError> {
        let framework = self.target.profile.as_selector();
        let pack_digest = self.compiled.framework_pack_digest.clone();
        let collector_runs: Vec<String> = report
            .collection_runs
            .iter()
            .map(|r| r.run_id.clone())
            .collect();
        let evidence_snapshot = seal_evidence_snapshot(
            self.ledger
                .lock()
                .expect("ledger")
                .query()
                .unwrap_or_default()
                .iter()
                .map(|e| e.digest().to_string()),
            collector_runs.clone(),
        );
        let run = AssessmentRun {
            id: self.assessment.id.clone(),
            framework: framework.into(),
            framework_pack_digest: pack_digest.clone(),
            assessment_definition_digest: definition_snapshot(&self.assessment).digest,
            started_at: now.to_rfc3339(),
            completed_at: now.to_rfc3339(),
            scope: self.scope.describe(),
            collector_runs,
            evidence_snapshot_digest: evidence_snapshot.digest,
            result_digest: assessment_result_digest(&report.results),
            status: "completed".into(),
            canonical_catalog_pin: self.compiled.catalog_digest.clone(),
            applicability_snapshot_id: snapshot_applicability(
                &self.assessment,
                &self.scope.describe(),
            )
            .digest,
            as_of: now.to_rfc3339(),
        };
        let payload = serde_json::to_string(&run).unwrap_or_else(|_| "{}".into());
        self.ledger
            .lock()
            .expect("ledger")
            .persist_assessment_run(run.id.as_str(), &payload)?;

        let current = self.store.last_readiness().unwrap_or_else(|| {
            project_readiness(
                &self.compiled,
                &report.results,
                framework,
                self.target.version.as_str(),
                &pack_digest,
                self.assessment.id.clone(),
            )
        });
        let previous = empty_readiness(&self.assessment.id, framework);
        let _diff = compare(&previous, &current);

        let _ = (assessment_summary, coverage_metrics);
        report.ran_jobs.push(spec.job_id.clone());
        report.push_stage(PipelineStage::Snapshot);
        report.push_stage(PipelineStage::Drift);
        self.mark_success(spec, now);
        Ok(())
    }

    fn mark_success(&self, spec: &JobSpec, now: DateTime<Utc>) {
        let run_id = spec.run_identity(now, spec.collector_id(), spec.configuration_digest());
        let mut state = self
            .store
            .load_job(&spec.job_id)
            .unwrap_or_else(|| JobState::fresh(spec.due_at));
        state.last_successful_run = Some(RunRef {
            id: run_id.clone(),
            at: now,
        });
        state.last_attempt = Some(AttemptRef {
            id: run_id,
            at: now,
            timed_out: false,
        });
        state.failure_state = FailureState::None;
        state.attempt_count = 0;
        if let Ok(cadence) = chrono::Duration::from_std(spec.cadence) {
            state.next_run = spec.slot_for(now) + cadence;
        }
        self.store.save_job(&spec.job_id, state);
    }
}

struct CollectionOutcome {
    job_id: String,
    run_id: String,
    run: CollectionRun,
    timed_out: bool,
    skipped: bool,
    attempted: bool,
    did_collect_pipeline: bool,
}

fn collect_job(
    clock: Arc<dyn Clock>,
    store: InMemorySchedulerStore,
    ledger: Arc<Mutex<EvidenceLedger>>,
    spec: JobSpec,
    collector: Option<Arc<dyn EvidenceCollector + Send + Sync>>,
    collector_scope: CollectorScope,
    now: DateTime<Utc>,
) -> Result<CollectionOutcome, SchedulerError> {
    let collector = collector
        .ok_or_else(|| SchedulerError::UnknownCollector(spec.collector_id().to_string()))?;
    let descriptor = collector.descriptor();
    let config = spec.configuration_digest();
    let run_id = spec.run_identity(now, spec.collector_id(), &config);

    if let Some(existing) = store.load_slot(&run_id) {
        match existing.status {
            SlotStatus::Succeeded | SlotStatus::InFlight => {
                let run = CollectionRun {
                    run_id: run_id.clone(),
                    collector_id: descriptor.id.clone(),
                    collector_version: descriptor.version.clone(),
                    started_at: now,
                    completed_at: Some(now),
                    scope: collector_scope.as_label(),
                    status: "completed".into(),
                    evidence_count: 0,
                    error_count: 0,
                    configuration_digest: config,
                };
                return Ok(CollectionOutcome {
                    job_id: spec.job_id.clone(),
                    run_id,
                    run,
                    timed_out: false,
                    skipped: true,
                    attempted: false,
                    did_collect_pipeline: false,
                });
            }
            SlotStatus::Failed | SlotStatus::TimedOut => {}
        }
    }

    store.save_slot(SlotRecord {
        run_id: run_id.clone(),
        status: SlotStatus::InFlight,
    });

    let started = clock.now();
    let collect_result = collector.collect(&collector_scope);
    let finished = clock.now();
    let elapsed = finished
        .signed_duration_since(started)
        .to_std()
        .unwrap_or(Duration::ZERO);
    let timed_out = elapsed >= spec.timeout && spec.timeout > Duration::ZERO;

    let mut run = CollectionRun {
        run_id: run_id.clone(),
        collector_id: descriptor.id.clone(),
        collector_version: descriptor.version.clone(),
        started_at: started,
        completed_at: Some(finished),
        scope: collector_scope.as_label(),
        status: "started".into(),
        evidence_count: 0,
        error_count: 0,
        configuration_digest: config,
    };

    let mut state = store
        .load_job(&spec.job_id)
        .unwrap_or_else(|| JobState::fresh(spec.due_at));
    let slot_key = spec.slot_for(now).to_rfc3339();
    if state.current_slot.as_deref() != Some(slot_key.as_str()) {
        state.attempt_count = 0;
        state.current_slot = Some(slot_key);
    }
    state.attempt_count = state.attempt_count.saturating_add(1);
    state.last_attempt = Some(AttemptRef {
        id: run_id.clone(),
        at: finished,
        timed_out,
    });

    let mut did_pipeline = false;
    match (timed_out, collect_result) {
        (true, _) => {
            run.status = "timed_out".into();
            run.error_count = 1;
            state.failure_state = FailureState::TimedOut;
            store.save_slot(SlotRecord {
                run_id: run_id.clone(),
                status: SlotStatus::TimedOut,
            });
        }
        (false, Ok(envelopes)) => {
            let mut sealed = Vec::new();
            for env in envelopes {
                sealed.push(env.with_collection_run(&run_id));
            }
            run.evidence_count = sealed.len() as u32;
            run.status = "completed".into();
            {
                let mut ledger = ledger.lock().expect("ledger");
                for env in &sealed {
                    ledger.append(env.clone())?;
                }
                ledger.record_collection_run(&run)?;
            }
            state.failure_state = FailureState::None;
            state.last_successful_run = Some(RunRef {
                id: run_id.clone(),
                at: finished,
            });
            state.attempt_count = 0;
            if let Ok(cadence) = chrono::Duration::from_std(spec.cadence) {
                state.next_run = spec.slot_for(now) + cadence;
            }
            store.save_slot(SlotRecord {
                run_id: run_id.clone(),
                status: SlotStatus::Succeeded,
            });
            did_pipeline = true;
        }
        (false, Err(_)) => {
            run.status = "failed".into();
            run.error_count = 1;
            {
                let mut ledger = ledger.lock().expect("ledger");
                ledger.record_collection_run(&run)?;
            }
            apply_retry_or_keep(&spec, &mut state, finished);
            store.save_slot(SlotRecord {
                run_id: run_id.clone(),
                status: SlotStatus::Failed,
            });
        }
    }

    store.save_job(&spec.job_id, state);
    Ok(CollectionOutcome {
        job_id: spec.job_id.clone(),
        run_id,
        run,
        timed_out,
        skipped: false,
        attempted: true,
        did_collect_pipeline: did_pipeline,
    })
}

fn apply_retry_or_keep(spec: &JobSpec, state: &mut JobState, now: DateTime<Utc>) {
    if state.attempt_count >= spec.retry.max_attempts {
        state.failure_state = FailureState::FailedExhausted;
        return;
    }
    state.failure_state = FailureState::Retrying;
    let delay = spec.retry.delay_after(state.attempt_count);
    if let Ok(delta) = chrono::Duration::from_std(delay) {
        state.next_run = now + delta;
    }
}

fn empty_readiness(id: &AssessmentId, framework: &str) -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot::empty(id.clone(), framework)
}
