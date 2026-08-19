# Grok 4.6 Prompt 13 — Continuous Assurance Scheduler

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: existing collectors/evidence/control tests; Prompts 01-12 interfaces

## Mission

Move assurance from an on-demand CLI assessment into a continuously operable engine while preserving deterministic collector/test boundaries.

## Runtime

Introduce scheduling contracts for collection jobs, test jobs, projection jobs, and snapshot jobs. Support cadence, freshness policy, dependencies, retry/backoff, timeout, jitter where appropriate, last successful run, last attempt, next run, failure state, and idempotent run identity.

Required pipeline:

`Collect -> Normalize -> Seal -> Ledger -> Evaluate -> Project -> Snapshot -> Drift`

A failed collector must not erase previous evidence; freshness/validity rules determine whether prior evidence remains usable.

Runs must be resumable/idempotent. Duplicate work should deduplicate by stable run/evidence identities where possible. Independent collectors should execute concurrently without cross-run state corruption.

Expose a library runtime first; CLI/daemon wiring may expose `weeping-angel isms run` / future daemon mode without embedding scheduling semantics in clap.

## Safety

Framework and control-test crates remain network-free. Scheduler orchestrates collectors, not the reverse. No collector may directly set compliance results.

## Tests

Use deterministic/fake clock tests for schedule due/not-due, retry, backoff, timeout, crash/restart, duplicate run, dependency ordering, concurrent independent collectors, and stale previous evidence.

## Non-goals

Do not require Kubernetes, Temporal, cron service, or cloud queue infrastructure. Keep a local/offline-capable core.

## Definition of done

The same deterministic assurance pipeline can operate repeatedly and safely over time rather than only as a one-shot assessment.