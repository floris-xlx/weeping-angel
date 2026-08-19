# SDD: Continuity and Resilience Assurance (Operational ISMS v1 — Prompt 20)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `evaluate_continuity_resilience` and continuity IR landed; `sdd_continuity_resilience_target` GREEN; baseline characterizations that no longer hold are skip-superseded |
| Program | Operational ISMS v1 — continuity / disaster-recovery governance |
| Slice | Model business continuity and DR as **executable multi-dimension assurance**. Distinguish documented resilience *intentions* from demonstrated recovery *effectiveness*. Surface gaps as risk / remediation **references**. |
| Dual-suite | `sdd_continuity_resilience_baseline` · `sdd_continuity_resilience_target` (`tests/contracts/continuity_resilience.{baseline,target}.rs`) — **not auto-discovered**; listed in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0038-continuity-resilience.md`](../adr/0038-continuity-resilience.md) — `0005-*` Operational ISMS sibling; cite by **path** |
| Prompt | [`docs/prompts/operational-isms-v1/20-continuity-resilience.md`](../prompts/operational-isms-v1/20-continuity-resilience.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — Continuity / resilience; do not fork the spine |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Consumes | Existing `Asset` / `AssetKind::Service`; catalog `control.resilience.*` / `control.backup.*` + evidence/test runtime; CIR `DocumentRef` (opaque; Prompt 12 registry is standalone); Prompt 16 `ContinuityRemediationRef` (opaque id; not the remediation engine) |
| Neighbors (must stay green) | `sdd_infrastructure_catalog_target`, `sdd_governance_catalog_target`, `sdd_compliance_ir_target`, `sdd_documentation_layout` |
| Collision fence | Do **not** rewrite `sdd_{infrastructure,governance}_catalog_*`, ISO pack IDs / `to =` remaps, collectors, or `catalog/canonical/v1/{controls,evidence,tests}/backup.toml` product semantics |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify | `cargo test --test sdd_continuity_resilience_target`; `cargo test --test sdd_continuity_resilience_baseline`; `cargo test --test sdd_documentation_layout`; keep neighbor targets GREEN; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for Prompt 20. It owns **continuity / resilience IR**, **multi-dimension recovery-capability evaluation**, and **gap emission as risk/remediation references**. It does **not** own catalog family TOML (infrastructure / governance catalogs), backup-product collectors, disaster orchestration, a BIA UI, Prompt 12 document control, or Prompt 16 remediation workflow.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Continuity assurance is a **management-system projection over that graph**. A plan PDF, a `procedure_present=true` fact, or a current `continuity-plan` review timestamp is **intention evidence**. It is never by itself **demonstrated recovery**.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

This slice may add IR continuity types, an assurance evaluation module, dual-suite contracts, this spec, its ADR, `documentation_layout.rs` registration, and additive root `Cargo.toml` `[[test]]` entries. It may **consume** existing catalog facts and control-test results. It must **not** retarget existing catalog IDs so that a plan document becomes “recovery proven.”

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/{controls,evidence,tests}/{backup,resilience,governance}.toml` IDs, expressions, automation class | infrastructure / governance catalogs |
| `tests/contracts/{infrastructure,governance}_catalog.*` rewrite | Those dual-suites stay GREEN |
| `frameworks/iso-27001/2022/**` requirement/control IDs and `to =` remaps | ISO remap |
| `crates/weeping-angel-collector/src/**` | Collectors |
| `backup.toml` product semantics (`enabled`, run coverage, retention, `restore-test-fresh`) | infrastructure catalog |
| `src/workbench/remediation.rs` scanner patch engine | Recon product — not ISMS Prompt 16 |
| Kleene applicability evaluator | applicability engine |
| Full Prompt 12 `ControlledDocument` registry | controlled documents (CIR stores opaque `DocumentRef` only) |
| Full Prompt 16 `Remediation` state machine | remediation engine |
| BIA / PM / backup-software / orchestration UI | Non-goals |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Domain types, ids, serde, validation | `crates/weeping-angel-assurance-ir/src/continuity.rs` (+ `typed_id!` in `id.rs`; re-export `lib.rs`) |
| Inventory binding | Reuse `Asset` / `AssetKind::Service` / `AssetId` / `VendorId`. Do **not** invent a parallel `BusinessService` inventory |
| Document pointers | Reuse CIR `DocumentRef` **if landed**; otherwise the same opaque shape CIR specifies (`id`, optional `title`, optional `kind`) |
| Remediation / risk pointers | `RemediationRef` / `RiskId` only — no workflow |
| Evaluation | `crates/weeping-angel-assurance/src/continuity.rs` — `evaluate_continuity_resilience` |
| Evidence / control-test | **Consume** existing envelopes and `ControlTestResult`. Do not put capability conclusions on envelopes |

Tiny allowed adjustments at implement: additive IR types; `AssessmentDefinition.continuity_profiles: Vec<…>` with `serde(default)`; validation messages; re-exports. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add `effectiveness` to plan documents.

Law (non-negotiable):

```text
A plan document alone MUST NEVER prove recovery capability.
procedure_present = true MUST NEVER imply demonstrated_recovery.
continuity-plan-current MUST NEVER imply RTO/RPO achievement.
A tabletop MUST NEVER satisfy technical RTO/RPO.
```

Existing catalog tests that pass on `procedure_present` / `fresh-within reviewed_at` remain **plan-existence / freshness** tests. This slice adds a **capability** projection beside them.

---

## 1. Problem / user-visible goal

Weeping Angel can already say “a recovery procedure is attested” and “a BCP record was reviewed this year.” Operators cannot say whether a **business service** is recoverable within its **RTO/RPO**, whether **backups are configured**, whether a **restore actually succeeded**, whether **exercises are current**, whether **critical dependencies** were in scope, or whether **exercise findings** are still open.

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` that means:

- `AssetKind::Service` exists, but IR has no service criticality, dependency graph, RTO/RPO, backup expectation, exercise result, observed recovery duration, observed data-loss window, or remediations.
- `Risk` is still a four-field stub (`id`, `title`, `description`, `status`). Prompt 12 `ControlledDocument` and Prompt 16 `Remediation` are **not landed**. CIR only specifies an opaque `DocumentRef`.
- The catalog already has plan-presence / freshness IDs:

  ```text
  control.resilience.recovery-procedure
  control.resilience.disaster-recovery-exercise
  control.resilience.recovery-objectives
  control.resilience.recovery-evidence-freshness
  control.resilience.business-continuity-plan
  control.resilience.disaster-recovery-governance
  ```

- `test.resilience.dr-exercise-recorded` and `test.resilience.recovery-objectives-documented` are `manual-review`.
- `test.resilience.recovery-procedure-present` is `all-subjects` on `procedure_present` — it can be **Effective** with no restore test.
- `test.resilience.continuity-plan-current` is `fresh-within` on `reviewed_at` (`365d`) — it can pass with a document-reference fact and no demonstrated restore.

**User-visible goal:** given assets/services, catalog evidence, and (when present) controlled-document / remediation refs, Weeping Angel evaluates **seven independent dimensions** and a derived **demonstrated recovery** flag:

```text
business service (AssetKind::Service) + criticality
  → dependencies (coverage)
  → recovery objectives (RTO / RPO)
  → backup expectation vs evidence.backup.configuration
  → procedure / plan DocumentRef (existence only)
  → exercise + result + observed duration / data-loss
  → open issues + RemediationRef / RiskId
        → ContinuityResilienceVerdict
```

A reviewer must be able to answer:

```text
is there a current plan?                         → plan existence
are backups configured for in-scope stores?      → backup configuration
did a restore actually succeed?                  → successful restore
is the exercise cadence met?                     → exercise cadence
did observed recovery meet RTO?                  → RTO achievement
did observed data-loss meet RPO?                 → RPO achievement
are exercise findings still open?                → unresolved findings
are critical dependencies in the exercise scope? → dependency coverage
did we demonstrate recovery?                     → NEVER from the plan PDF
```

Example distinctions:

```text
current BCP + procedure_present=true + no exercise
  → plan existence Satisfied
  → demonstrated_recovery = false

technical restore inside RTO and RPO, no open findings, deps covered
  → demonstrated_recovery = true

restore success=false
  → successful_restore = Failed; demonstrated_recovery = false

last exercise older than cadence
  → exercise_cadence = Stale; demonstrated_recovery = false

critical payment-gateway dependency omitted from the exercise
  → dependency_coverage = Gap; demonstrated_recovery = false

no evidence.backup.configuration for a required store
  → backup_configuration = Missing; demonstrated_recovery = false

tabletop only, even with a spoken “we would meet RTO”
  → RTO/RPO = NotMeasured (not Met); demonstrated_recovery = false

failed restore with an open exercise issue and no RemediationRef
  → fail closed (gap must carry a remediation ref or explicit untracked marker)
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Asset` / `AssetKind::Service` / `AssetId` | `asset.rs` | **Business service = an Asset of kind Service.** Optional `parent`. Do not add `BusinessService`. Criticality lives on the continuity profile, not by overloading `tags` as SSOT |
| `SubjectKind::Service` / `DataStore` | `subject.rs` | Population kinds already exist; consume them |
| `Vendor` / `VendorId` | `vendor.rs` | Allowed dependency target |
| `Risk` / `RiskId` | `risk.rs` | Four-field stub. Gaps **cite** `RiskId` when the assessment has a matching risk; do not expand the register here |
| `ControlImplementation` / CIR `DocumentRef` | CIR spec; **not landed** | Store opaque `DocumentRef { id, title?, kind? }`. If CIR lands first, **reuse** that type — do not fork a second document pointer |
| Prompt 12 `ControlledDocument` | not landed | Do not implement the document registry. A `DocumentRef` is not proof the document is approved/effective |
| Prompt 16 `Remediation` | not landed | Store `RemediationRef { id: String }` (stable id). Do not implement the workflow |
| `src/workbench/remediation.rs` | scanner diffs | **Not** Prompt 16. Do not promote `RemediationRequest` into IR |
| Catalog resilience / backup / governance IDs | `catalog/canonical/v1/**` | **Consume.** Do not change expressions so plan-presence becomes capability |
| `evidence.resilience.recovery-plan` | facts: `procedure_present`, `objectives_documented`, `exercise_at?`, `redundant?`, `reviewed_at?` | Intention / freshness facts |
| `evidence.resilience.continuity-plan` | facts: `reviewed_at`, `plan_kind` (`bcp` \| `dr-governance`) | Governance freshness |
| `evidence.backup.{configuration,run,restore-test}` | restore-test: `tested_at`, `success` | Configuration + demonstrated restore **inputs** |
| Control-test runtime | `weeping-angel-control-test` | Reuse `Effectiveness`. Do not add `DemonstratedRecovery` to that enum |
| `AssessmentDefinition` | `assessment.rs` | Additive `continuity_profiles` (name flexible) with `serde(default)` so golden assessments stay valid |
| `validate_assessment_ir` | `validation.rs` | Add fail-closed graph checks for new refs |
| Golden IR fixtures | `tests/fixtures/assurance-ir/v1/**` | Must keep decoding. No schema bump |
| `sdd_compliance_ir_target` | IR-008/009/019 + goldens | Stay GREEN |
| `sdd_infrastructure_catalog_target` / `sdd_governance_catalog_target` | catalog law | Stay GREEN — this slice must not flip their plan-presence semantics |

Serde: new types `rename_all = "camelCase"`. Empty/None skip-serialize. `ASSURANCE_IR_SCHEMA` stays `assurance-ir/v1`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 Service assets have no continuity fields

[`crates/weeping-angel-assurance-ir/src/asset.rs`](../../crates/weeping-angel-assurance-ir/src/asset.rs):

```text
AssetKind includes Service
Asset { schema_version, id, kind, name, parent?, tags }
Asset::new(id, kind, name)
```

No `criticality`, no dependency list, no RTO/RPO, no backup expectation, no exercise binding.

### 3.2 IR has no continuity / exercise / remediation types

Grep of `crates/weeping-angel-assurance-ir/src/**` on this SHA finds **no**:

```text
BusinessService, ServiceCriticality, ServiceDependency,
RecoveryObjective, RecoveryObjectiveId, rto, rpo,
BackupExpectation, ContinuityExercise, ExerciseResult,
observed_recovery_duration, observed_data_loss,
DocumentRef, ControlledDocument, Remediation, RemediationRef,
ContinuityResilienceProfile, ContinuityResilienceVerdict
```

`id.rs` has no `RecoveryObjectiveId` / `ContinuityExerciseId`.

`AssessmentDefinition` inventories: requirements, controls, mappings, evidence_requirements, tests, implementations, scope, assets, identities, vendors, risks, exceptions, processing_activities. **No** continuity collection.

### 3.3 `Risk` is a four-field stub

[`risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs): *“Minimal risk record. Not a risk engine.”*

```text
Risk { id, title, description, status ∈ {Open, Accepted, Mitigated, Closed} }
```

No continuity-sourced risks, no auto-emission from exercise gaps.

### 3.4 Prompt 12 / 16 are specified, not landed

- CIR (`docs/specs/control-implementation-registry.md`) specifies opaque `DocumentRef`; product `implementation.rs` has **no** `document_refs`.
- Prompt 12 / 16 product types are absent from IR.
- `src/workbench/remediation.rs` is a **scanner unified-diff** helper (`RemediationRequest` / `RemediationResult`), not an ISMS remediation record.

### 3.5 Catalog already encodes plan presence and freshness

Landed IDs (do not rewrite):

| Control | Test | What it actually proves today |
| --- | --- | --- |
| `control.resilience.recovery-procedure` | `test.resilience.recovery-procedure-present` (`all-subjects` / `procedure_present`) | Procedure **flag** present |
| `control.resilience.disaster-recovery-exercise` | `test.resilience.dr-exercise-recorded` (`manual-review`) | Manual review required |
| `control.resilience.recovery-objectives` | `test.resilience.recovery-objectives-documented` (`manual-review`) | Manual review required |
| `control.resilience.recovery-evidence-freshness` | `test.resilience.recovery-evidence-fresh` (`reviewed_at` window `24h`) | Plan-fact freshness |
| `control.resilience.redundancy` | `test.resilience.redundancy-where-required` (`redundant`) | Redundancy **flag** |
| `control.resilience.business-continuity-plan` | `test.resilience.continuity-plan-current` (`fresh-within` `reviewed_at` `365d`) | Plan-record freshness |
| `control.resilience.disaster-recovery-governance` | `test.resilience.dr-governance-attested` (`manual-review`) | Attestation + plan record |
| `control.backup.restore-testing` | `test.backup.restore-test-fresh` | Store-level restore `success` + freshness (infrastructure product) |

Fixtures already in tree (catalog, **not** this slice’s capability fixtures):

- `fixtures/assurance/canonical/v1/resilience/{healthy,missing-dr-exercise,stale-recovery-plan,exception-approved-rto}/`
- `fixtures/assurance/canonical/v1/backup/{healthy,missing-backup,stale-restore-test,failing-restore}/`
- `fixtures/assurance/canonical/v1/governance/current-documents/` includes `evidence.resilience.continuity-plan` with `plan_kind: "bcp"`

`resilience/healthy` can combine `procedure_present=true` with restore-test facts, but **no IR evaluation** composes those into RTO/RPO achievement, dependency coverage, or open-finding tracking.

### 3.6 Plan-presence can pass without demonstrated restore

Found case (must remain true of **catalog** tests; this is why a new projection is required):

1. `test.resilience.recovery-procedure-present` evaluates only `procedure_present`. A recovery-plan envelope with `procedure_present=true` and **no** `evidence.backup.restore-test` can yield `Effectiveness::Effective`.
2. `test.resilience.continuity-plan-current` evaluates only `reviewed_at` freshness on `evidence.resilience.continuity-plan`. A `kind: document-reference` BCP fact inside 365 days can pass with **no** restore and **no** exercise.
3. DR exercise and RTO/RPO documentation tests are `manual-review` — they never auto-conclude achievement.

### 3.7 No continuity evaluation API

`weeping-angel-assurance` has applicability, lineage, readiness, snapshot, SoA. **No** `continuity` module, **no** `evaluate_continuity_resilience`.

### 3.8 Baseline suite obligations (must PASS on current code)

| Id | Characterization |
| --- | --- |
| P20-B01 | `AssetKind::Service` exists; `Asset` JSON from `new` has no `criticality` / `rto` / `rpo` / `dependencies` |
| P20-B02 | IR sources have no continuity domain types listed in §3.2 |
| P20-B03 | `Risk` remains the four-field stub; module docs still say *Not a risk engine* |
| P20-B04 | IR has no `DocumentRef` / `ControlledDocument` / Prompt 16 `Remediation` type |
| P20-B05 | Catalog IDs in §3.5 exist with the listed test ops (`procedure_present`, `manual-review`, `fresh-within`) |
| P20-B06 | `test.resilience.dr-exercise-recorded` and `test.resilience.recovery-objectives-documented` use `op = "manual-review"` |
| P20-B07 | `test.resilience.recovery-procedure-present` expression field is `procedure_present` only (no restore / RTO field) |
| P20-B08 | `test.resilience.continuity-plan-current` is `fresh-within` on `reviewed_at` — no restore predicate |
| P20-B09 | `AssessmentDefinition` source has no `continuity_profiles` / `recovery_objectives` inventory |
| P20-B10 | `src/workbench/remediation.rs` remains a scanner patch helper; not imported by IR |
| P20-B11 | Dual-suite names are listed in root `Cargo.toml` |
| P20-B12 | Collision fence: this suite does not require edits to backup/resilience/governance catalog IDs |

After the target is GREEN, skip-supersede: `#[ignore = "superseded by sdd_continuity_resilience_target"]` on characterizations that no longer hold. Landed: P20-B02, P20-B04, P20-B09, and the “no continuity module” found-case are skip-superseded. Catalog plan-presence found cases (P20-B05–B08, B10–B12, B03, B01 Asset JSON) remain GREEN.

---

## 4. Desired behavior (target)

### 4.1 Product home

```text
weeping-angel-assurance-ir
  continuity.rs     # profiles, objectives, exercises, verdicts, refs
  asset.rs          # consumed; Service stays an AssetKind
  id.rs             # RecoveryObjectiveId, ContinuityExerciseId, ContinuityProfileId
  assessment.rs     # additive vec
  validation.rs     # dangling refs + unique ids
  implementation.rs # DocumentRef reuse if CIR landed

weeping-angel-assurance
  continuity.rs     # evaluate_continuity_resilience
```

Network-free. Provider-neutral (no AWS Backup, Azure Site Recovery, Veeam, Terraform DR, or Chaos-engineering product types in generic IR).

### 4.2 Domain model (provider-neutral)

JSON names camelCase. Suggested types (names stable enough for tests):

```text
ServiceCriticality = MissionCritical | High | Medium | Low

DependencyKind = Runtime | Data | Identity | Network | Supplier | Other

ServiceDependency {
  from: AssetId,                 // must resolve to AssetKind::Service
  to: AssetRef,                  // AssetId or VendorId
  kind: DependencyKind,
  critical: bool,
}

AssetRef = Asset(AssetId) | Vendor(VendorId)

RecoveryObjective {
  id: RecoveryObjectiveId,
  subject: AssetId,              // service or supporting store
  rto_seconds: u64,              // > 0
  rpo_seconds: u64,              // ≥ 0
}

BackupExpectation {
  subject: AssetId,              // typically DataStore / Database / Dataset
  required: bool,
  evidence_type: "evidence.backup.configuration",  // consume catalog id; do not fork
}

DocumentKind = Policy | Standard | Procedure | Record | Plan | Runbook

DocumentRef {                     // identical to CIR if that type exists
  id: String,
  title?: String,
  kind?: DocumentKind,
}

RecoveryProcedureRef {
  document: DocumentRef,
  role: BusinessContinuityPlan | DisasterRecoveryPlan | RecoveryRunbook | Other,
}

ExerciseKind = Tabletop | Walkthrough | TechnicalRecovery | RestoreTest | Other

ContinuityExercise {
  id: ContinuityExerciseId,
  subject: AssetId,
  kind: ExerciseKind,
  conducted_at: DateTime<Utc>,
  procedure?: RecoveryProcedureRef,
  in_scope_dependencies: [AssetRef],
}

ExerciseOutcome = Passed | Failed | Partial | NotExecuted

ExerciseIssue {
  id: String,                    // stable, non-empty
  summary: String,
  open: bool,
  remediation_refs: [RemediationRef],
}

RemediationRef { id: String }    // Prompt 16 identity; opaque here
RiskRef { id: RiskId }           // optional link into assessment.risks

ExerciseResult {
  exercise_id: ContinuityExerciseId,
  outcome: ExerciseOutcome,
  observed_recovery_duration_seconds?: u64,
  observed_data_loss_window_seconds?: u64,
  issues: [ExerciseIssue],
  remediation_refs: [RemediationRef],
  risk_refs: [RiskRef],
}

ContinuityResilienceProfile {
  id: ContinuityProfileId,
  service: AssetId,              // AssetKind::Service
  criticality: ServiceCriticality,
  dependencies: [ServiceDependency],
  objectives: [RecoveryObjectiveId],   // or inline RecoveryObjective
  backup_expectations: [BackupExpectation],
  procedures: [RecoveryProcedureRef],
  exercise_cadence_seconds?: u64,      // required for MissionCritical | High
  exercises: [ContinuityExerciseId],   // or inline
  results: [ExerciseResult],
}
```

Durations are **integer seconds** (no `f64`). Zero RTO is illegal. Zero RPO means “no data loss tolerated.”

**Landed shape (do not treat the id-only sketch above as SSOT):**

- `ContinuityResilienceProfile.objectives` is `Vec<RecoveryObjective>` (inline), not objective ids.
- `ContinuityResilienceProfile.exercises` is `Vec<ContinuityExercise>` (inline), not exercise ids.
- Opaque Prompt 16 pointer is `ContinuityRemediationRef { id: String }` so it does not collide with typed-id `crate::RemediationRef`.
- CIR `DocumentRef` is reused. `DocumentKind` includes `Plan` and `Runbook` in addition to Policy / Standard / Procedure / Record.

### 4.3 Evaluation API

```text
evaluate_continuity_resilience(
  assessment: &AssessmentDefinition,
  profile: &ContinuityResilienceProfile,
  evidence: &EvidenceSet,            // existing sealed envelopes
  as_of: DateTime<Utc>,
) -> Result<ContinuityResilienceVerdict, ContinuityResilienceError>

ContinuityResilienceVerdict {
  profile_id: ContinuityProfileId,
  service: AssetId,
  as_of: DateTime<Utc>,
  plan_existence: DimensionStatus,          // Satisfied | Missing | Stale | NotApplicable
  backup_configuration: DimensionStatus,    // Satisfied | Missing | Insufficient | NotApplicable
  successful_restore: RestoreStatus,        // Demonstrated | Failed | Missing | Stale | NotApplicable
  exercise_cadence: CadenceStatus,          // Current | Stale | Missing
  rto_achievement: ObjectiveStatus,         // Met | Missed | NotMeasured
  rpo_achievement: ObjectiveStatus,         // Met | Missed | NotMeasured
  unresolved_exercise_findings: FindingStatus, // None | Open
  dependency_coverage: CoverageStatus,      // Covered | Gap
  demonstrated_recovery: bool,
  gaps: [ContinuityGap],
}

ContinuityGap {
  dimension: ContinuityDimension,
  summary: String,
  risk_refs: [RiskRef],
  remediation_refs: [ContinuityRemediationRef],
}
```

Rules:

1. **`demonstrated_recovery` is derived.** It is true only when **all** of:
   - `successful_restore == Demonstrated`
   - `rto_achievement == Met`
   - `rpo_achievement == Met`
   - `unresolved_exercise_findings == None`
   - `dependency_coverage == Covered`
   - the satisfying exercise `kind` is `TechnicalRecovery` or `RestoreTest`
   - `backup_configuration` is `Satisfied` or `NotApplicable` (required backup expectation with evidence, or none required)
   - `exercise_cadence == Current`
2. **`plan_existence` is not an input to `demonstrated_recovery`.** `demonstrated_recovery` is derived and **excludes** plan existence. Satisfied plan + everything else failing ⇒ `demonstrated_recovery = false`.
3. **Tabletop / Walkthrough** may satisfy `exercise_cadence` and may populate issues. They **cannot** set RTO/RPO to `Met` and **cannot** set `successful_restore = Demonstrated`.
4. **Failed restore** (`ExerciseOutcome::Failed` or `evidence.backup.restore-test` `success=false` used as the technical result) ⇒ `successful_restore = Failed`, `demonstrated_recovery = false`.
5. **Stale exercise:** last exercise (any kind) older than `exercise_cadence_seconds` relative to `as_of` ⇒ `exercise_cadence = Stale`. MissionCritical / High with no cadence configured fails closed.
6. **Missing backup evidence** for a `required` expectation ⇒ `backup_configuration = Missing` and a gap.
7. **Critical dependency** (`critical: true`) absent from the satisfying exercise’s `in_scope_dependencies` ⇒ `dependency_coverage = Gap`.
8. **Open issue** (`issues[].open == true`) ⇒ `unresolved_exercise_findings = Open`. Each open issue **must** carry at least one `ContinuityRemediationRef` **or** the verdict fails closed (`untracked exercise finding`). Prompt 16 is landed: when `assessment.remediations` is non-empty, dangling remediation ids fail validation.
9. **RTO/RPO:** compare `observed_recovery_duration_seconds` to `rto_seconds` and `observed_data_loss_window_seconds` to `rpo_seconds`. Missing observations on a technical exercise that `Passed` ⇒ `NotMeasured` (not `Met`). `observed > objective` ⇒ `Missed`.
10. **Catalog facts are inputs, not conclusions.** `procedure_present=true` may support `plan_existence = Satisfied` together with a `RecoveryProcedureRef`. It must not flip `demonstrated_recovery`.
11. **Gaps always surface.** Every failing dimension emits a `ContinuityGap`. When the assessment contains a `Risk` whose id is cited, keep the ref; when none exists, still emit the gap (do not invent a `Risk` row in this slice). Remediation refs are required for open exercise issues (rule 8); other dimensions may emit refs when the author supplied them.

### 4.4 Validation (fail closed)

On `AssessmentDefinition::validate()` (additive):

| Check | Error needle (stable enough) |
| --- | --- |
| Duplicate `ContinuityProfileId` / exercise / objective ids | `duplicate continuity` |
| `profile.service` not in `assets` or not `AssetKind::Service` | `continuity service` |
| Dependency `from` ≠ profile service | `dependency from` |
| Dependency / objective / backup / exercise subject dangling | `dangling` |
| `rto_seconds == 0` | `rto` |
| MissionCritical / High missing cadence | `exercise cadence` |
| `ExerciseResult.exercise_id` unknown | `dangling exercise` |
| `RiskRef` not in `assessment.risks` | `dangling risk` |
| CIR/12 document registry present **and** `DocumentRef.id` cannot resolve | `dangling document` (only when that inventory exists) |

Clockless `validate()` does **not** evaluate staleness. Staleness is `evaluate_continuity_resilience(..., as_of)`.

### 4.5 Relationship to existing catalog tests

| Catalog test | Stays | This slice |
| --- | --- | --- |
| `recovery-procedure-present` | procedure flag | plan_existence input only |
| `continuity-plan-current` | 365d review freshness | plan_existence freshness input only |
| `dr-exercise-recorded` | manual-review | **not** a substitute for ExerciseResult |
| `recovery-objectives-documented` | manual-review | objectives must be **typed IR**, not a documented-policy boolean |
| `recovery-evidence-fresh` | 24h `reviewed_at` | not RTO/RPO achievement |
| `backup.restore-test-fresh` | store restore freshness + success | may feed `successful_restore` when bound as the technical result |

Do not change those catalog expressions in this slice. If implement needs an additional catalog test id, add it in a **new** file or additive records only after a separate catalog SDD — default is **IR + assurance evaluation**, no TOML rewrite.

### 4.6 Acceptance fixtures (target must encode)

Construct in tests (IR + evidence), not by mutating infrastructure catalog fixture meaning:

| Id | Fixture | Expected |
| --- | --- | --- |
| P20-T01 | Current plan / procedure ref; no exercise | `plan_existence = Satisfied`; `demonstrated_recovery = false`; gap on exercise/restore |
| P20-T02 | Technical restore/exercise within RTO and RPO; deps covered; no open issues; required backup evidence present | `demonstrated_recovery = true`; RTO/RPO `Met`; restore `Demonstrated` |
| P20-T03 | Failed restore (`success=false` or `ExerciseOutcome::Failed`) | `successful_restore = Failed`; `demonstrated_recovery = false` |
| P20-T04 | Last exercise older than cadence | `exercise_cadence = Stale`; `demonstrated_recovery = false` |
| P20-T05 | Critical dependency not in exercise scope | `dependency_coverage = Gap`; `demonstrated_recovery = false` |
| P20-T06 | Required backup expectation, no `evidence.backup.configuration` | `backup_configuration = Missing`; `demonstrated_recovery = false` |
| P20-T07 | Manual tabletop only vs technical recovery test | Tabletop: cadence may be `Current`; RTO/RPO `NotMeasured`; `demonstrated_recovery = false`. Technical path remains T02 |
| P20-T08 | Passed technical exercise with an open issue | `unresolved_exercise_findings = Open`; `demonstrated_recovery = false`; issue carries `ContinuityRemediationRef` |

Plus locks:

| Id | Assertion |
| --- | --- |
| P20-T09 | Plan document / `procedure_present` / current BCP **never** sets `demonstrated_recovery` |
| P20-T10 | Existing catalog IDs and `sdd_infrastructure_catalog_target` / `sdd_governance_catalog_target` semantics unchanged |
| P20-T11 | Dual-suite registered in root `Cargo.toml` |
| P20-T12 | Business service is `AssetKind::Service`; no parallel inventory type |
| P20-T13 | Every capability gap is a `ContinuityGap` with optional `RiskRef` and required `ContinuityRemediationRef`s on open findings |
| P20-T14 | `DocumentRef` is opaque; document existence ≠ capability |
| P20-T15 | Schema remains `assurance-ir/v1`; old assessments deserialize |
| P20-T16 | Collectors / evidence crate stay conclusion-free (no `demonstratedRecovery` on envelopes) |

### 4.7 Dual-suite protocol

`tests/contracts/` is **not** Cargo auto-discovery.

```toml
[[test]]
name = "sdd_continuity_resilience_baseline"
path = "tests/contracts/continuity_resilience.baseline.rs"

[[test]]
name = "sdd_continuity_resilience_target"
path = "tests/contracts/continuity_resilience.target.rs"
```

| Gate | Suite | Status |
| --- | --- | --- |
| Spec + ADR | this file + `docs/adr/0038-continuity-resilience.md` | Accepted |
| Baseline on CURRENT | `sdd_continuity_resilience_baseline` | GREEN — §3 found cases that still hold; skip-superseded where they do not |
| Target | `sdd_continuity_resilience_target` | **GREEN** — P20-T01…T16 call `evaluate_continuity_resilience` |
| Implement | IR + `evaluate_continuity_resilience` | landed; **no** catalog ID rewrite |
| Neighbors | infra / governance / compliance IR / documentation_layout | stay GREEN |

---

## 5. Acceptance criteria (testable)

1. Dual-suite `sdd_continuity_resilience_baseline` / `sdd_continuity_resilience_target` is registered in root `Cargo.toml`; this spec is in `CANONICAL_SPECS`; target GREEN; baseline skip-superseded where §3 no longer holds.
2. Continuity IR models service criticality, dependencies, recovery objectives (RTO/RPO seconds), backup expectations, procedure/document refs, exercises, results, observed recovery duration, observed data-loss window, issues, and risk/remediation refs — provider-neutral, `assurance-ir/v1` additive.
3. Business services are `Asset` + `AssetKind::Service`. No second service inventory.
4. Evaluation exposes the seven dimensions in §4.3 plus derived `demonstrated_recovery`.
5. A current plan, `procedure_present=true`, or `continuity-plan-current` **never** yields `demonstrated_recovery = true` by itself (T01, T09).
6. Successful technical exercise/restore within RTO and RPO, covered critical dependencies, required backup evidence, and no open findings yields `demonstrated_recovery = true` (T02).
7. Failed restore, stale exercise, uncovered critical dependency, missing backup evidence, tabletop-only, and unresolved exercise remediation each fail closed as specified (T03–T08).
8. Tabletop / walkthrough cannot satisfy RTO/RPO or successful restore.
9. Open exercise findings require a `ContinuityRemediationRef` (or fail closed as untracked). Gaps are first-class `ContinuityGap` records.
10. Prompt 12 / 16 engines are **not** implemented here; refs stay opaque. Scanner `workbench/remediation.rs` is untouched.
11. Catalog backup/resilience/governance IDs, ISO packs, and collectors are not rewritten; neighbor target suites stay GREEN.
12. No backup software, disaster orchestration, or BIA/PM UI ships in this slice.

---

## 6. Out of scope

- Backup software, snapshot agents, or cloud DR product integrations
- Disaster-recovery **orchestration** (failover runtimes, runbooks-as-code execution)
- Business impact analysis **UI**, questionnaires, or financial-loss calculators
- Rewriting `backup.toml` / `resilience.toml` / `governance.toml` control expressions
- Rewriting `sdd_infrastructure_catalog_*` or `sdd_governance_catalog_*`
- ISO pack ID changes or `to =` remaps
- Collector implementations
- Full Prompt 12 controlled-document registry
- Full Prompt 16 remediation state machine / tickets / kanban
- Expanding `Risk` beyond citing `RiskId` (risk register owns that)
- Treating `Effectiveness::Effective` on plan-presence tests as demonstrated recovery
- Hardcoding AWS/Azure/GCP recovery APIs in IR
- Multi-tenant SaaS control plane

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Implementers treat `procedure_present` or BCP freshness as capability | T01/T09; `demonstrated_recovery` formula excludes `plan_existence` |
| Catalog tests are rewritten so plan-presence fails (breaks infra/gov suites) | Collision fence; T10; consume facts only |
| Tabletop marked as RTO Met | T07; ExerciseKind gate |
| Parallel `BusinessService` type forks inventory | T12; `AssetKind::Service` only |
| Prompt 12/16 scope creep | Opaque refs; fail closed only when those inventories exist |
| Scanner `RemediationRequest` confused with ISMS remediation | B10; do not import workbench into IR |
| Half-written stubs make target GREEN for the wrong reason | Target calls `evaluate_continuity_resilience`; P20-T01…T16 encode the original found cases |
| Residual/risk register rebase conflicts | Cite `RiskId` only; do not add score fields |
| Duration drift (`f64` hours) | Integer seconds only |
| Neighbor suites regress | Verify infra / governance / compliance IR / documentation_layout |

---

## 8. ADR

This is an architecture/contract decision (plan ≠ capability; AssetKind::Service; multi-dimension verdict; opaque Prompt 12/16 refs; no catalog rewrite). Accepted: [`docs/adr/0038-continuity-resilience.md`](../adr/0038-continuity-resilience.md).

Filename **`0005-*`**. Operational ISMS siblings already use `0041-risk-methodology.md`, `0040-operational-risk-register.md`, `0005-continuous-assurance-scheduler.md`. Cite this file by **path**.

---

## 9. Landed product

Owned crates: `weeping-angel-assurance-ir` + `weeping-angel-assurance` (evaluation). Dual-suite registered. Schema remains `assurance-ir/v1`. Catalog TOML was not rewritten.

```text
weeping-angel-assurance-ir::continuity
  ContinuityProfileId, RecoveryObjectiveId, ContinuityExerciseId
  ServiceCriticality, ServiceDependency, DependencyKind, AssetRef
  RecoveryObjective, BackupExpectation
  DocumentRef (CIR), RecoveryProcedureRef, DocumentKind::{Plan,Runbook}
  ContinuityExercise, ExerciseKind, ExerciseResult, ExerciseOutcome, ExerciseIssue
  ContinuityRemediationRef, RiskRef
  ContinuityResilienceProfile on AssessmentDefinition.continuity_profiles
  ContinuityResilienceVerdict, ContinuityGap, ContinuityDimension
  ContinuityResilienceError, validate_continuity_profiles

weeping-angel-assurance::evaluate_continuity_resilience   # crate-root re-export
```

Public-contract pointer: [`docs/specs/assurance-runtime.md`](assurance-runtime.md). Traces: `.sdd/runs/` and `.sdd/artifacts/` only (ADR 0004).
