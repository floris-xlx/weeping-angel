# Prompt 4 — Test Surface, Panic Budget, Schema and Repository Hygiene

Work in `floris-xlx/weeping-angel` on cleanup and regression containment that does not require changing the core assurance semantics owned by Prompts 2 and 3 or the repository guard engine owned by Prompt 1. This prompt is designed to run concurrently with all three.

## Objective

Reduce the repository's accumulated maintenance burden: retire obsolete dual-suite scaffolding and ignored tests, replace brittle source-grep contracts where possible, contain production panic paths, deduplicate schemas, and remove generated/log/documentation artifacts from source-of-truth roles.

## Exclusive ownership boundary

You may modify:

- broad test infrastructure under `tests/**` except repository-integrity/architectural-cleanup suites owned by Prompt 1 and semantic target suites actively owned by Prompts 2/3
- `schemas/**` and `codex-security/schemas/**`
- `README.md` and documentation indexes that merely enumerate generated/current state
- `.gitignore` and repository artifact hygiene configuration
- `audit.txt` and its replacement location/format if it is generated evidence
- production `src/**` only for narrowly scoped panic/error-handling cleanup that does not change assurance semantics
- test/support utilities used to replace `require_needles`-style source greps

Do not modify `xtask/**`, `architecture/**`, `docs/debt/register.toml`, canonical catalog/framework/readiness product code, temporal/lineage/evidence/SoA product code, or their actively owned semantic contract suites.

## Required work

1. Inventory the 661 ignored tests and classify them into: obsolete migration baseline, intentionally characterization-only, temporarily blocked, or still valuable regression coverage. Remove obsolete baseline suites rather than preserving permanent `#[ignore]` debt.

2. Collapse completed `baseline + target` dual suites into durable contract/regression tests where the target behavior is already authoritative. Preserve useful red characterization only when it documents a known unresolved product defect. Do not delete coverage merely to reduce counts.

3. Replace brittle `require_needles` and source-spelling assertions with semantic tests wherever feasible. Prefer public API behavior, typed metadata, serialized schema contracts, compile-time boundaries, or AST-aware checks. Keep source-grep checks only when exact source structure is itself the invariant and document why.

4. Establish a production panic budget. Separate test/example/build-script usage from runtime paths. Remove or convert unsafe `.unwrap()`/`.expect()` usage in scanner, parser, network, auth, reporting, workbench and other production paths where external/input/runtime failure is possible. Use typed errors with context. Do not perform mechanical churn on statically impossible/test-only unwraps.

5. Add regression protection against new panic paths in production code using the repository's existing lint/check infrastructure without editing Prompt 1's `xtask` implementation. If a lint config or test can enforce this independently, use it. Keep narrowly justified exceptions explicit.

6. Deduplicate Codex Security JSON schemas. Select one authoritative schema location. Convert any required second location into generated packaging/output or remove it. Add a test/build assertion proving distributed copies, if still required, are generated from and byte/semantic-equivalent to the SSOT.

7. Clean generated artifact policy. Determine whether `audit.txt` is source material or generated execution output. If generated, remove it from normal tracked source and replace it with a compact structured manifest or CI artifact policy containing generator/schema version, source commit, digest and relevant finding summary. Prevent raw logs and execution caches from being reintroduced.

8. Harden `.gitignore`/admission hygiene for `.env*` secrets, `node_modules`, Rust `target*`, generated SDD runtime state, local databases, raw scan/audit output, private keys/cert material, editor caches and other non-source artifacts. Do not ignore legitimate fixtures or schema examples required by tests.

9. Simplify README and documentation indexes. The README should explain capabilities, architecture and canonical commands, not manually enumerate dozens of contract suites or copy rapidly changing architecture state. Generate inventories where they are useful, or link to canonical metadata/tests.

10. Remove stale compatibility comments, dead fixtures and duplicated helper functions discovered during this cleanup, but stay within this prompt's ownership boundary and avoid broad product refactors.

11. Keep test discovery clear and predictable. Reduce redundant explicit `[[test]]` declarations where Cargo auto-discovery can safely own them, but do not change names relied on by CI/scripts without updating those consumers inside this ownership boundary.

12. Measure before/after cleanup. Record counts for ignored tests, baseline/target suites, `require_needles` occurrences, production unwrap/expect usage, duplicate schemas and tracked generated artifacts. Do not optimize the metric at the expense of coverage.

## Concurrency contract

Prompt 1 owns `xtask`, architecture metadata, repository-integrity suites and debt register. Do not edit them.

Prompt 2 owns catalog/framework/readiness semantics and associated active contract tests. If one of those tests appears obsolete, leave it and report it rather than colliding.

Prompt 3 owns temporal/evidence/lineage/SoA semantics and associated active contract tests. Apply the same no-collision rule.

If a broad test cleanup encounters a file being actively owned by another prompt, skip that file and continue with the rest of the cleanup. Do not use cross-cutting formatting or mass renames that create unnecessary merge conflicts.

## Acceptance criteria

- Obsolete ignored baseline suites are removed, not merely left ignored.
- Valuable characterization/regression coverage remains.
- `require_needles` usage is materially reduced and remaining uses are justified.
- Production panic-prone paths return typed errors rather than process panics where runtime failure is plausible.
- Codex Security schemas have one SSOT.
- Raw generated audit/execution artifacts are not treated as hand-maintained source.
- `.gitignore` prevents recurrence of `.env`, `node_modules`, build/cache and sensitive local artifact commits without hiding legitimate fixtures.
- README no longer serves as a manually synchronized test inventory.
- Before/after cleanup counts are recorded in an appropriate cleanup report outside the debt register.
- `cargo fmt --all -- --check` passes.
- `cargo check --workspace --all-targets` passes.
- Relevant retained tests pass.
- No new `#[ignore]` is introduced as a shortcut for failures.

Prefer deletion and simplification over replacing one form of scaffolding with another. The end state should have fewer moving pieces and stronger regression coverage.