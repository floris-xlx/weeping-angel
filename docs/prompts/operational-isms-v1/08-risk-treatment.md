# Grok 4.6 Prompt 08 — Risk Treatment Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 06, 10 interface if available; otherwise define treatment references without owning control implementation schema

## Mission

Implement first-class risk treatment decisions and plans for `Mitigate`, `Accept`, `Avoid`, and `Transfer`.

## Model

Add `RiskTreatmentDecision`, `TreatmentPlan`, `TreatmentAction`, owner, decision principal, rationale, target date, target residual risk, linked canonical controls, linked remediation references, evidence expectations, approval, review/expiry, and lifecycle state.

Risk acceptance must be immutable governance evidence with accountable principal and validity/review semantics. An expired acceptance must never continue suppressing treatment requirements.

Mitigation plans may reference multiple control implementations and actions. Avoidance/transfer must still require evidence showing the organizational action happened; selecting an enum is not sufficient.

## State machine

Support proposed -> approved -> executing -> verification -> completed, plus cancelled/superseded. Invalid transitions fail closed.

## Tests

Cover all four strategies, expired risk acceptance, partially complete mitigation, transferred risk with missing contract evidence, superseded treatment, target residual risk mismatch, and dangling control references.

## Non-goals

Do not calculate residual effectiveness here, create external tickets, or interpret framework applicability directly.

## Definition of done

Every open risk can have an explicit accountable treatment path whose state, evidence and approvals are reproducible and auditable.