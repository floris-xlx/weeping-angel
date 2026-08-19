# Grok 4.6 Prompt 23 — Management Review Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 01-22, especially objectives, risk, audit, CAPA and temporal snapshots

## Mission

Automate preparation of management-review inputs while keeping management decisions and approvals explicitly human/accountable.

## Review package

Generate an immutable review snapshot containing at least previous review actions, changes in internal/external issues, interested-party/obligation changes, security-objective performance, risk/register changes, residual risk, control-effectiveness trends, incidents, audit results, nonconformities/CAPA, supplier posture, personnel/training posture, continuity exercises, resource/capacity concerns, exceptions, improvement opportunities, and other configured ISMS metrics.

The generator must identify data gaps instead of omitting them.

## Decisions

Add `ManagementReview`, participants/principals, review period, snapshot digest, agenda/input references, decisions, action items/remediation references, resource decisions, risk decisions, improvement decisions, approvals/sign-off, and next review date.

Machine output cannot sign or approve the review.

## Reproducibility

A management review must be reconstructable later from its pinned snapshot even if current state has changed.

## Tests

Cover complete package, missing objective metrics, open high risks, overdue CAPA, supplier regression, previous-action carryover, signed review, unsigned draft, and historical reconstruction.

## Non-goals

No meeting/video software or generative minutes system required.

## Definition of done

Most management-review preparation is generated directly from operational ISMS state, leaving management to make and approve the decisions rather than assemble evidence manually.