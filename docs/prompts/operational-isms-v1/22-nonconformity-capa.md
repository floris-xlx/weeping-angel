# Grok 4.6 Prompt 22 — Nonconformity and CAPA Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 16, 21; existing `nonconformities` request semantics

## Mission

Implement nonconformity and corrective-action/preventive-improvement lifecycle with evidence-backed verification.

## Model

Add `Nonconformity`, source/audit/control/incident references, description, severity, affected scope, owner, detected time, containment actions, root-cause analysis, `CorrectiveAction`, target dates, implementation evidence, effectiveness criteria, review period, reviewer, closure decision, and history.

State flow:

`Open -> Contained -> RootCauseIdentified -> CorrectiveActionPlanned -> Implemented -> EffectivenessReview -> Closed`

Allow cancellation/supersession only with accountable rationale. A single green control test must not automatically close CAPA unless declared effectiveness criteria are satisfied over the required period.

## Integration

Control regressions, audit findings and incidents may propose nonconformities but must not silently create major/minor classification without the appropriate decision boundary.

## Tests

Cover complete CAPA, missing root cause, overdue action, failed effectiveness review, re-opened nonconformity, sustained verification window, audit linkage, incident linkage, and immutable closure.

## Non-goals

No generic issue tracker or AI root-cause engine.

## Definition of done

The system can prove how a detected nonconformity was contained, corrected, verified for effectiveness and formally closed.