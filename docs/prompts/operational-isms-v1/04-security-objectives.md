# Grok 4.6 Prompt 04 — Security Objectives Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Batch: 01/06 — Foundation (Prompts 01–04)
Execution: implement after Prompts 01–03 against the same branch/worktree
Dependencies: Prompt 01, Prompt 02 scope engine, typed evidence/runtime from Canonical Assurance v1

## Mission

Make information-security objectives measurable first-class ISMS records rather than prose in a policy.

## Model

Introduce `SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement`, and status projection. Objectives must support owner, scope, description, metric definition, baseline, target, comparison semantics, measurement source/evidence requirement, cadence, start date, deadline where applicable, review date, and lifecycle state.

Metrics must use typed values and deterministic comparison. Support percentage, count, duration, boolean, ratio, and bounded numeric values without introducing a general scripting language.

Example: critical vulnerabilities remediated within seven days, target >= 98%, measured from canonical vulnerability/remediation evidence.

## Evaluation

An objective can be `OnTrack`, `AtRisk`, `Missed`, `Achieved`, or `InsufficientEvidence`. Missing measurements must not be interpreted as success. Manual objectives may require immutable attestation/approval evidence.

Store measurement lineage so management review can reconstruct exactly how a status was produced at a point in time.

## Implementation constraints

Objective evaluation must remain deterministic and side-effect free. Reuse `EvidenceValue` and existing evidence snapshot/lineage primitives instead of introducing a second metric-value representation. Scope every measurement explicitly. Stale, partial, or missing evidence must degrade to `InsufficientEvidence` or another non-success state according to explicit rules.

## Tests

Cover threshold boundaries, missing data, stale measurement, mixed manual/automated objectives, scoped populations, historical measurements, and deterministic status transitions.

## Acceptance gates

- Replaying a pinned evidence snapshot reproduces the same objective status.
- Missing or stale measurement data never yields `OnTrack` or `Achieved`.
- Typed percentage/count/duration/boolean/ratio comparisons have boundary tests.
- Objective scope reuses canonical scope resolution from Prompt 02.
- Management-review consumers can reconstruct metric, target, measurement, evidence lineage, and resulting status without re-querying live systems.

## Non-goals

Do not create dashboards, notifications, arbitrary formula execution, or management-review workflows here.

## Definition of done

Security objectives can be evaluated reproducibly from evidence and later fed directly into management review and continual-improvement workflows.