# Grok 4.6 Prompt 24 — Certification Readiness and Auditor Evidence Pack

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 01-23; ISO 27001 framework pack and canonical assurance program

## Mission

Integrate Operational ISMS v1 into an end-to-end ISO 27001 certification-readiness projection and immutable auditor evidence package. This is readiness support, never a certification claim.

## Readiness command/API

Provide library API and CLI surface conceptually equivalent to:

`weeping-angel isms readiness --framework iso-27001 --period <range>`

It must project context, scope, interested parties, obligations, objectives, assets, risk methodology/register, risk treatment, operational SoA, control implementations, policy/document register, evidence index, control assurance history, personnel assurance, supplier register, incidents, continuity, internal audits, nonconformities/CAPA, management reviews, exceptions, and unresolved readiness gaps.

Statuses must use honest language such as ready, effective, ineffective, insufficient evidence, requires manual review, not applicable, and coverage. Forbidden claims include `certified`, `audit passed`, `compliant`, or `certification guaranteed` unless representing an externally supplied certification artifact as a fact.

## Audit pack

Implement deterministic export with a manifest and digests, structured folders/records for context, scope, policies, risks, SoA, controls, evidence, personnel, suppliers, incidents, continuity, audits, CAPA, management reviews, exceptions, and readiness output. Every referenced artifact/evidence item must be addressable by immutable digest or stable reference.

Support point-in-time and audit-period export. Pin all projections to a finalized assessment/ISMS snapshot.

## Explainability

For every readiness gap, provide lineage such as:

`ISO requirement -> canonical mapping -> risk/control -> implementation -> test -> evidence -> result`

For every readiness delta between two snapshots, return causal changes such as control regression, evidence expiry, scope change, new risk, expired exception or missing review.

## End-to-end tests

Build a representative organization fixture with mixed automated/manual controls, risks/treatments, current/stale evidence, personnel/vendor populations, one incident, continuity exercise, internal audit, CAPA and management review. Prove deterministic export/digest, period-aware assurance, SoA consistency, no certification language, gap explainability, and clean workspace regression tests.

## Non-goals

Do not impersonate a certification body, replace external auditor judgment, or bundle copyrighted ISO normative text.

## Definition of done

Weeping Angel can operate and package a largely automated ISO 27001 ISMS lifecycle from scope and risk through continuous evidence, audit, corrective action, management review and certification-readiness evidence.