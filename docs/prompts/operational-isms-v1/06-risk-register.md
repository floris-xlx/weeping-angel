# Grok 4.6 Prompt 06 — Operational Risk Register

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 01, 02, 05

## Mission

Expand the current intentionally minimal `Risk` model into an operational information-security risk record while maintaining compatibility for existing fixtures and callers.

## Required model

Risk must support title, scenario, threat, vulnerability/weakness references, affected assets/services/processes, CIA impact dimensions where configured, likelihood, impact, inherent score/rating, residual score/rating placeholder, owner, source, discovered time, review cadence, next review, status, treatment reference, canonical control references, evidence lineage, tags/classification, and history/supersession.

A security finding is not automatically a risk. One risk may aggregate multiple findings; one finding may contribute to multiple risk candidates.

Statuses should cover at least draft/open, under treatment, accepted, mitigated, closed, and retired with explicit transition validation.

## Integrity

All asset/control/treatment references must validate. Risk history must not be destroyed by edits: use version/supersession semantics or immutable events where appropriate.

## Tests

Cover backward-compatible decoding of old minimal risk fixtures, complete risk round-trip, invalid transitions, dangling asset/control IDs, overdue reviews, and deterministic scoring through the selected methodology.

## Non-goals

Do not auto-generate risks, implement treatment workflow, or calculate control-derived residual risk yet.

## Definition of done

Weeping Angel has a real canonical risk register suitable for operational ISMS use and linked to the same assets/controls/evidence graph used by assurance.