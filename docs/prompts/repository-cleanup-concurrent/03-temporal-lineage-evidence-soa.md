# Prompt 3 — Temporal Evidence, Lineage, Persistence and SoA Integrity

Work in `floris-xlx/weeping-angel` on temporal assurance and evidence-history correctness. This prompt is designed to run concurrently with Prompts 1, 2 and 4. Stay within the ownership boundary below.

## Objective

Close the trust-boundary debt around temporal evidence selection, assessment lineage reconstruction, evidence `latest` versus `current` semantics, persistence invariants, replayability, and Statement of Applicability correctness. Historical assessments must remain reproducible and must never be silently rewritten by current state.

## Exclusive ownership boundary

You may modify:

- temporal/lineage/assessment-history modules under `crates/weeping-angel-assurance/**`, especially `temporal.rs`, `lineage.rs`, replay/result-history code, and directly related modules
- `crates/weeping-angel-evidence/**`
- SoA-specific assurance implementation and narrowly related CLI/service code where required
- temporal/evidence/lineage/SoA-specific tests under `tests/contracts/**`
- directly relevant specs/ADRs only to reflect implemented semantics; do not modify repository-integrity metadata or broad documentation indexes

Do not modify `xtask/**`, `architecture/**`, `docs/debt/register.toml`, canonical catalog/framework/readiness code owned by Prompt 2, or broad test/schema/README hygiene owned by Prompt 4.

## Required work

1. Define and enforce precise `current`, `latest`, `valid-at`, and `as-of` evidence semantics. These terms must not be aliases. Document them in code-level types/APIs and encode them in tests.

2. Ensure temporal evidence selection uses a pinned assessment clock. Evidence created after `asOf`, expired before `asOf`, revoked before `asOf`, or otherwise invalid for that point in time must never leak into historical assessment results.

3. Preserve append-only evidence validity history. Expiry, revocation, supersession, correction, or validity changes must be represented as immutable events or new records rather than mutation of sealed evidence envelopes.

4. Rebuild assessment lineage so every result can prove its definition identity, evidence snapshot identity, applicability identity, catalog/framework pins supplied by the execution layer, result identity, and `asOf` time. Reconstruction from persisted data must be deterministic.

5. Make replay fail closed. If required pinned material is missing, digest/identity verification fails, history is incomplete, or persisted lineage is inconsistent, replay must return a typed failure rather than silently using current state.

6. Separate collection failure from evidence erasure. A failed collector run must not delete or invalidate previously valid ledger evidence unless an explicit validity event says so. Distinguish "no new observation" from "known absent" and from "evidence no longer valid".

7. Harden persistence invariants in `weeping-angel-evidence`: deterministic serialization, stable IDs, atomic/transactional writes where applicable, idempotent append behavior, duplicate-event handling, corruption detection, and explicit schema/version validation.

8. Make `latest` versus `current` queries explicit at the API boundary. "Latest recorded event" may refer to an expired/revoked item; "current valid evidence" must apply validity rules. Prevent callers from accidentally using latest-record semantics for current assessment.

9. Strengthen Statement of Applicability invariants. The SoA must derive from pinned assessment/application state, preserve inclusion/exclusion/applicability rationale, implementation status versus effectiveness distinction, exceptions, and evidence references. It must never infer certification/compliance from readiness output.

10. Ensure historical SoA generation does not reload current mutable framework/catalog/evidence state. If the selected historical assessment cannot be reconstructed exactly, fail explicitly.

11. Add adversarial tests for clock boundaries, expiry at exact instants, evidence recorded after assessment time, revocation, supersession, duplicate events, stale snapshots, missing pins, corrupted persistence, collection failures, replay after repository/framework changes, and SoA generation from historical runs.

12. Keep period effectiveness conservative. A single positive observation must not imply continuous effectiveness over a period. Missing intervals or unknown population coverage must remain explicit.

## Concurrency contract

Prompt 1 owns all repository guard implementation. Do not modify `xtask`; expose typed invariant APIs or stable persisted metadata where useful, and let Prompt 1 wire Guard 09–12.

Prompt 2 owns catalog/framework parsing, digesting and readiness projection. Consume catalog/framework pins through stable interfaces; do not redesign those subsystems here.

Prompt 4 owns broad ignored-test cleanup, panic-budget policy, schema deduplication, README and generated-artifact hygiene. Only touch tests directly required to prove temporal/evidence/lineage/SoA behavior.

## Acceptance criteria

- `current`, `latest`, `valid-at`, and `as-of` have distinct tested semantics.
- Historical assessments cannot see future evidence.
- Validity history is append-only.
- Assessment lineage is reconstructable and deterministic.
- Replay cannot substitute current mutable state for missing pinned state.
- Failed collection cannot erase previous evidence implicitly.
- Persistence detects malformed/corrupt/incompatible state and fails closed.
- SoA output is bound to an exact assessment and preserves applicability rationale and evidence lineage.
- Period effectiveness remains conservative under incomplete evidence.
- `cargo fmt --all -- --check` passes.
- Relevant temporal, evidence-validity, assessment-lineage, continuous-assurance, SoA and persistence tests pass.
- Full workspace compile succeeds.

Do not weaken temporal rules to preserve legacy fixtures. If a legacy test encodes temporal leakage or mutable-history behavior, replace it with a regression test for the correct invariant within this prompt's owned test surface.