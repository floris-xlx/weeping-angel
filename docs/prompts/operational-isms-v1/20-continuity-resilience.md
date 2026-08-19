# Grok 4.6 Prompt 20 — Business Continuity and Resilience Assurance

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: assets/services, Prompt 12, evidence/test runtime

## Mission

Model continuity and disaster-recovery governance as executable assurance, not `DR-plan.pdf exists`.

## Domain

Add provider-neutral concepts for business service, service criticality, dependency, recovery objective, RTO, RPO, backup expectation, recovery procedure/document reference, exercise/test, exercise result, observed recovery duration, observed data-loss window, issues, and remediation references.

Controls should distinguish plan existence, backup configuration evidence, successful restore evidence, exercise cadence, RTO/RPO achievement, unresolved exercise findings, and dependency coverage.

A plan document alone must never prove recovery capability.

## Tests

Fixtures: current plan but no exercise; successful exercise within RTO/RPO; failed restore; stale exercise; critical dependency not covered; backup evidence missing; manual tabletop vs technical recovery test; unresolved exercise remediation.

## Non-goals

Do not implement backup software, disaster orchestration, or business impact analysis UI. Keep generic enough for multiple providers.

## Definition of done

Weeping Angel can distinguish documented resilience intentions from demonstrated recovery effectiveness and surface gaps into risk/remediation.