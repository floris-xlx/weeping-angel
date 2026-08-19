# ADR 0005 — Continuity and resilience as executable assurance

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_continuity_resilience_target` GREEN; plan existence is not demonstrated recovery. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The operational reading “a current DR/BCP document (or `procedure_present`) is sufficient resilience evidence.” Does **not** supercede infrastructure catalog backup/restore tests, governance catalog BCP/DR **governance** IDs, IR schema `assurance-ir/v1`, ADR 0001 spine, or ADR 0004 documentation layout. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/continuity-resilience.md`](../specs/continuity-resilience.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) — Continuity / resilience |
| Prompt | [`docs/prompts/operational-isms-v1/20-continuity-resilience.md`](../prompts/operational-isms-v1/20-continuity-resilience.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_continuity_resilience_baseline` GREEN then skip-superseded on characterizations that no longer hold; `sdd_continuity_resilience_target` RED then GREEN (P20-T01…T16) |

> Filename `0005-*` is the Operational ISMS register. Siblings include [`0005-operational-risk-register.md`](0005-operational-risk-register.md), [`0005-risk-methodology.md`](0005-risk-methodology.md), [`0005-continuous-assurance-scheduler.md`](0005-continuous-assurance-scheduler.md). Cite **this file by path**.

## Context

On SHA `6e31bf1a…` the catalog could already attest:

```text
procedure_present                  → test.resilience.recovery-procedure-present
reviewed_at within 365d (BCP)      → test.resilience.continuity-plan-current
manual-review DR exercise          → test.resilience.dr-exercise-recorded
manual-review RTO/RPO documented   → test.resilience.recovery-objectives-documented
store restore success + freshness  → test.backup.restore-test-fresh
```

IR `AssetKind::Service` existed, but there was no service criticality, dependency coverage, typed RTO/RPO, backup expectation, exercise result, observed recovery duration / data-loss window, or remediation/risk gap projection. That left a false assurance path: `DR-plan.pdf exists` (or `procedure_present=true`) reads as recoverable. Prompt 20 forbids that.

Questions this decision answers:

1. Where does continuity capability live — catalog TOML rewrite, IR+evaluation, or a GRC sidecar?
2. What is a “business service” if `AssetKind::Service` already exists?
3. Which evidence proves **intention** vs **demonstrated recovery**?
4. Can a tabletop satisfy RTO/RPO?
5. How do Prompt 12 documents and Prompt 16 remediations attach without landing those engines here?
6. How are gaps surfaced without a BIA/PM UI?

## Decision

This is what shipped. Field-level law is [`docs/specs/continuity-resilience.md`](../specs/continuity-resilience.md).

Product home: `weeping-angel-assurance-ir::continuity` (`crates/weeping-angel-assurance-ir/src/continuity.rs`) plus `evaluate_continuity_resilience` in `weeping-angel-assurance::continuity` (crate-root re-export). Schema stays `assurance-ir/v1`. JSON camelCase. No new crate.

### 1. Executable projection beside the catalog; do not retarget plan-presence tests

Continuity **capability** is `evaluate_continuity_resilience` over IR profiles + existing sealed evidence. Catalog IDs keep their landed meaning (procedure flag, plan freshness, manual-review, store restore). Incorrect: rewriting `test.resilience.recovery-procedure-present` so it requires a restore, or introducing `assurance-ir/v2`.

`AssessmentDefinition.continuity_profiles` is additive (`serde(default)`, `skip_serializing_if` empty, JSON `continuityProfiles`). Golden assessments without the field still decode.

### 2. Business service is `AssetKind::Service`

There is no `BusinessService` inventory. Criticality, dependencies, objectives, backup expectations, procedures, and exercises hang off a `ContinuityResilienceProfile` keyed by `AssetId`. `profile.service` must resolve to `AssetKind::Service` or `validate()` fails (`continuity service`).

Typed ids: `ContinuityProfileId`, `RecoveryObjectiveId`, `ContinuityExerciseId`.

Objectives and exercises are **inline** on the profile (`Vec<RecoveryObjective>`, `Vec<ContinuityExercise>`), not id-only inventories.

### 3. Plan existence is a dimension; it is never capability

Eight first-class dimensions: plan existence, backup configuration evidence, successful restore evidence, exercise cadence, RTO achievement, RPO achievement, unresolved exercise findings, dependency coverage.

`demonstrated_recovery` is derived and **excludes** plan existence. It is true only when all of:

- `successful_restore == Demonstrated`
- `rto_achievement == Met`
- `rpo_achievement == Met`
- `unresolved_exercise_findings == None`
- `dependency_coverage == Covered`
- `backup_configuration` is `Satisfied` or `NotApplicable`
- `exercise_cadence == Current`
- the satisfying exercise `kind` is `TechnicalRecovery` or `RestoreTest`

A Satisfied plan with no technical restore is **not** recovered. Catalog `procedure_present` / current BCP may set `plan_existence = Satisfied` and must not flip `demonstrated_recovery`.

### 4. Tabletop ≠ technical recovery

`ExerciseKind::Tabletop` / `Walkthrough` may satisfy cadence and record issues. Only `TechnicalRecovery` / `RestoreTest` may set `successful_restore = Demonstrated` and RTO/RPO `Met`. Tabletop-only profiles yield `successful_restore = NotApplicable` and RTO/RPO `NotMeasured`.

Failed restore (`ExerciseOutcome::Failed` or `evidence.backup.restore-test` `success=false`) ⇒ `successful_restore = Failed`.

RTO/RPO compare integer seconds (`observed_recovery_duration_seconds` vs `rto_seconds`, `observed_data_loss_window_seconds` vs `rpo_seconds`). Missing observations on a passed technical exercise ⇒ `NotMeasured`, not `Met`. Zero RTO fails validation.

### 5. Opaque document and remediation refs

Plan/procedure pointers reuse CIR `DocumentRef` (`id`, optional `title`, optional `kind`). `DocumentKind` includes `Plan` and `Runbook` in addition to CIR’s Policy/Standard/Procedure/Record. A `DocumentRef` is not proof the document is approved or effective. Prompt 12 `ControlledDocument` registry is **not** resolved from `AssessmentDefinition` (standalone registry); dangling-document checks fire only when a document inventory is present on the assessment.

Prompt 16 work records stay the remediation engine. Continuity stores opaque `ContinuityRemediationRef { id: String }` so it does not collide with typed-id `RemediationRef`. When `assessment.remediations` is non-empty, dangling remediation ids fail `validate()`. Open exercise issues without a remediation ref fail evaluation (`untracked exercise finding`). Scanner `src/workbench/remediation.rs` is not the ISMS record.

Gaps are `ContinuityGap` rows that may cite `RiskId` (`RiskRef`). This slice does not invent `Risk` bodies or a ticket UI.

### 6. Provider-neutral; integer seconds; fail closed

No backup-vendor types in IR. Durations are `u64` seconds. MissionCritical / High require a non-zero `exercise_cadence_seconds`. Clockless `validate()` does not evaluate staleness; staleness is `evaluate_continuity_resilience(..., as_of)`. Duplicate profile/objective/exercise ids and dangling service/dependency/objective/exercise/risk refs fail `validate()`.

Backup configuration consumes `evidence.backup.configuration` (catalog type, not a fork). Required expectation with no envelope ⇒ `backup_configuration = Missing`. Critical dependency (`critical: true`) absent from the latest exercise’s `in_scope_dependencies` ⇒ `dependency_coverage = Gap`.

## Non-goals

- Backup software, DR orchestration, BIA UI.
- Rewriting infrastructure/governance catalog suites, ISO packs, collectors, or `backup.toml` semantics.
- Landing Prompt 12 / 16 engines in this slice.

## Consequences

- Operators can distinguish “we have a plan” from “we restored inside RTO/RPO with covered dependencies, current cadence, required backup evidence, and closed findings.”
- Existing `procedure_present` / `continuity-plan-current` Effective results remain valid **plan** evidence and stay insufficient for capability.
- Dual-suite `sdd_continuity_resilience_{baseline,target}` is registered under `tests/contracts/`; spec path is in `CANONICAL_SPECS`. Neighbor suites `sdd_infrastructure_catalog_target`, `sdd_governance_catalog_target`, `sdd_compliance_ir_target`, and `sdd_documentation_layout` stay green.
- Traces go to `.sdd/runs` and `.sdd/artifacts`. `docs/sdd/` remains a stub. No `tests/sdd/`.

## Related

- Spec: [`docs/specs/continuity-resilience.md`](../specs/continuity-resilience.md)
- Infrastructure catalog: [`docs/specs/infrastructure-canonical-assurance-catalog.md`](../specs/infrastructure-canonical-assurance-catalog.md)
- Governance catalog: [`docs/specs/governance-canonical-assurance-catalog.md`](../specs/governance-canonical-assurance-catalog.md)
- CIR `DocumentRef`: [`docs/specs/control-implementation-registry.md`](../specs/control-implementation-registry.md)
- Remediation engine (Prompt 16): [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
