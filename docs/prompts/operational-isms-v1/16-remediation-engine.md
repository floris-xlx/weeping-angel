# Grok 4.6 Prompt 16 — Remediation Workflow Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompt 15; risks/control tests/exceptions

## Mission

Implement canonical remediation records that connect assurance failures and risk treatment to accountable work without turning Weeping Angel into a generic project-management system.

## Model

Add `Remediation`, source type/reference, affected risks/controls/subjects, owner, priority, severity, SLA policy/reference, due time, state, external ticket references, planned actions, evidence-of-fix requirements, verification state, closure principal/time, and history.

State machine should include proposed/open, in progress, awaiting verification, verified, closed, accepted/waived where governed by valid exception/risk acceptance, cancelled, and superseded as appropriate.

A test returning green once must not automatically close a remediation unless the remediation verification policy explicitly permits it. Support sustained-effectiveness windows.

External Jira/Linear/GitHub issues are adapters/references only; canonical remediation identity stays inside Weeping Angel.

## Tests

Cover creation from control regression, risk treatment action linkage, SLA overdue, external ticket reference, verification failure, sustained success, expired waiver, and immutable closure history.

## Non-goals

No kanban UI, assignment notifications, or external ticket implementation.

## Definition of done

Every material assurance/risk gap can be tracked from cause through corrective evidence and independent verification with an auditable lifecycle.