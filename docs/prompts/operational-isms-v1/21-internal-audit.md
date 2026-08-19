# Grok 4.6 Prompt 21 — Internal Audit Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: operational ISMS graph, temporal assurance, existing `audit_program` request semantics

## Mission

Implement an internal-audit domain that uses machine evidence to prepare and support audits while preserving auditor independence and judgment.

## Model

Add `AuditProgram`, period, scope, objectives, criteria, scheduling, auditor/principal, independence declaration/evidence, and child audits. Each `Audit` should support scope/sample, selected controls/requirements, evidence snapshot, procedures, observations, findings, nonconformities, conclusion, sign-off, and immutable history.

Sampling must be explicit and reproducible. Machine-generated sample suggestions are proposals; audit conclusion remains a human decision.

Audit evidence should be pinned to immutable snapshot/digests so later system changes cannot rewrite what the auditor reviewed.

## Automation

Automatically prepare candidate scope, stale/failed controls, risk hotspots, evidence bundles, sample populations, prior findings, and remediation status. Never auto-sign an audit conclusion.

## Tests

Cover annual program, scoped audit, auditor independence metadata, deterministic sample, evidence snapshot pinning, finding creation, incomplete audit, signed audit, and historical reproducibility.

## Non-goals

No external certification workflow, auditor marketplace, or generic document editor.

## Definition of done

Internal audit becomes a first-class operational process backed by the same evidence graph rather than a disconnected manual folder.