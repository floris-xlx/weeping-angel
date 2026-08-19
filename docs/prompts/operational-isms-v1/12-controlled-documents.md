# Grok 4.6 Prompt 12 — Controlled Document and Policy Registry

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 01, 03; immutable artifact/evidence support

## Mission

Implement document-control governance without building a document editor. Treat policies, standards, procedures, plans, runbooks, guidelines, and records as immutable versioned artifacts with governance metadata.

## Model

Add `ControlledDocument`, document type, stable identifier, title, owner, version, artifact reference/digest, status, effective date, review date, approvers, approval evidence, supersedes reference, applicability scope, linked controls/obligations/risks, confidentiality/classification, acknowledgements where required, and retention metadata.

Distinguish draft metadata from an approved/effective version. An edited artifact must result in a new digest/version; do not mutate the evidence behind an approved document.

Document existence alone cannot make an operational control effective when the control requires execution evidence.

## Evaluation

Provide deterministic tests/helpers for current version present, approved, effective, within review window, acknowledgement coverage, and supersession consistency.

## Tests

Cover current policy, stale policy, draft-only policy, missing approval, superseded document, changed digest, required acknowledgement gaps, and retention metadata.

## Non-goals

No rich text editor, Google Drive clone, e-signature product, ISO text ingestion, or general DMS.

## Definition of done

Weeping Angel can prove which governed document version was effective at any time and connect it to obligations, controls and evidence.