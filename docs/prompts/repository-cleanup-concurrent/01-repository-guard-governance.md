# Prompt 1 — Repository Guard and Governance Hardening

Work in `floris-xlx/weeping-angel` on the repository-governance plane only. This prompt is designed to run concurrently with Prompts 2–4. Do not make opportunistic changes outside the ownership boundary below.

## Objective

Turn the repository health gate into a durable architectural enforcement system rather than a collection of source greps and debt-backed stubs. Focus on the guard engine, architecture metadata, ADR/spec lifecycle enforcement, and debt-expiry mechanics. Do not implement assurance product semantics owned by Prompts 2 or 3.

## Exclusive ownership boundary

You may modify:

- `xtask/**`
- `architecture/**`
- `.cargo/**` only where required for `cargo xtask`
- `.github/workflows/**` only for repository-health-gate enforcement
- `docs/adr/**` only for metadata/identity normalization required by Guard 14; do not rewrite architectural decisions
- `docs/specs/repository-integrity.md`
- `docs/debt/register.toml`
- repository-integrity-specific tests under `tests/contracts/repository_integrity*` and `tests/contracts/architectural_cleanup*`

Do not modify product implementation under `src/**` or `crates/**`. Do not modify unrelated contract suites, schemas, README, or generated audit artifacts.

## Required work

1. Decompose `xtask/src/lib.rs` into a maintainable structure before adding more policy. Introduce clear modules for repository model loading, architecture metadata, debt metadata, individual checks, and report rendering. Preserve the public `cargo xtask guard` behavior and current JSON/human output contracts.

2. Make `RepositoryModel` a single-load evaluation plane. Avoid rereading the same Rust source file for every check. Cache normalized source text or a lightweight indexed representation during model construction. Keep ordering deterministic.

3. Move repository-specific policy out of hard-coded Rust constants wherever practical. Architecture ownership kinds, forbidden patterns, required concepts, and similar policy should come from versioned files under `architecture/`. The Rust code should validate and interpret policy, not duplicate it.

4. Implement Guard 14 as a real ADR graph/identity check. Enforce repository-wide ADR identity uniqueness for all new ADRs, parse machine-readable lifecycle metadata, validate references, detect dangling `supersedes`/`superseded_by`/`depends_on` edges, and reject cycles where the relationship is required to be acyclic. Existing duplicate historical IDs must be represented explicitly as grandfathered debt; do not silently renumber historical ADRs unless a safe migration with redirects/aliases is proven.

5. Implement Guard 15 as a real spec-lifecycle and dependency-policy check. Specs must have explicit lifecycle state and valid transitions. Validate that active specs reference existing architectural ownership and that superseded/retired specs cannot masquerade as active requirements. Keep this repository-bound, deterministic, and offline.

6. Harden the debt exemption model. Every live guard exemption must have at minimum an owner, introduced date, severity, remediation statement, associated guard/check, and expiry/review date. An expired exemption must fail CI. A resolved debt item must prove closure through either a live repository guard or named regression tests. Reject malformed, duplicate, or orphaned debt IDs.

7. Preserve fail-closed behavior. A missing/malformed architecture manifest, invariants file, forbidden-pattern file, debt register, ADR metadata set, or spec lifecycle file must never degrade to a silent pass.

8. Add machine-readable guard output suitable for CI consumers without breaking current JSON. Include schema/version, aggregate counts, failed/skipped checks, debt exemptions, and deterministic check IDs. Avoid embedding unstable wall-clock values in equality-sensitive fixtures.

9. Ensure CI requires `cargo xtask guard` and cannot bypass it through a path filter when architecture, ADRs, specs, debt, workspace manifests, framework/catalog locations, or Rust source change.

## Concurrency contract

Prompts 2 and 3 will change assurance product code. Do not edit their files. Build Guard 05–12 plumbing/interfaces only if it can be done without encoding incomplete product semantics. It is acceptable for those guards to call dedicated invariant evaluators or metadata contracts that Prompts 2/3 will satisfy, but do not duplicate their business logic inside `xtask`.

Prompt 4 owns broad test-suite retirement, schemas, README/audit hygiene, and general panic-budget work. Do not touch those surfaces except the two repository-integrity contract suites named above.

## Acceptance criteria

- `xtask` is modular rather than another monolith.
- Repository model loading is deterministic and avoids repeated whole-repo reads per check.
- Guard 14 and Guard 15 are real checks, not debt-backed stubs.
- Expired guard debt fails closed.
- Existing historical ADR duplication is explicitly contained and no new duplicate ADR ID can land.
- Architecture/debt/spec metadata parse failures fail CI.
- Existing implemented checks 01–04 and 13 retain behavior unless deliberately strengthened with regression coverage.
- `cargo fmt --all -- --check` passes.
- `cargo test -p xtask` passes.
- repository-integrity/architectural-cleanup target suites pass.
- `cargo xtask guard` passes on the final integrated tree, or any temporarily failing product-semantic checks are clearly attributable to concurrently executing Prompts 2/3 rather than weakened/skipped enforcement.

Do not paper over failures with new `#[ignore]`, broad allowlists, or new debt records unless the debt is unavoidable, narrowly scoped, owned, expiring, and justified.