# Grok 4.6 Prompt 04 — Security Objectives Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompt 01, typed evidence/runtime from Canonical Assurance v1

## Mission

Make information-security objectives measurable first-class ISMS records rather than prose in a policy.

## Model

Introduce `SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement`, and status projection. Objectives must support owner, scope, description, metric definition, baseline, target, comparison semantics, measurement source/evidence requirement, cadence, start date, deadline where applicable, review date, and lifecycle state.

Metrics must use typed values and deterministic comparison. Support percentage, count, duration, boolean, ratio, and bounded numeric values without introducing a general scripting language.

Example: critical vulnerabilities remediated within seven days, target >= 98%, measured from canonical vulnerability/remediation evidence.

## Evaluation

An objective can be `OnTrack`, `AtRisk`, `Missed`, `Achieved`, or `InsufficientEvidence`. Missing measurements must not be interpreted as success. Manual objectives may require immutable attestation/approval evidence.

Store measurement lineage so management review can reconstruct exactly how a status was produced at a point in time.

## Tests

Cover threshold boundaries, missing data, stale measurement, mixed manual/automated objectives, scoped populations, historical measurements, and deterministic status transitions.

## Non-goals

Do not create dashboards, notifications, arbitrary formula execution, or management-review workflows here.

## Definition of done

Security objectives can be evaluated reproducibly from evidence and later fed directly into management review and continual-improvement workflows.