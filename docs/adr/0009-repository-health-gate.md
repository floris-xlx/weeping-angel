# ADR 0009 — Repository health gate (architecture ownership SSOT + `cargo xtask guard`)

| Field | Value |
| --- | --- |
| Status | **Accepted** — increment 1 health gate is law: architecture ownership SSOT + `cargo xtask guard` (checks 01, 02, 03, 13) is mandatory in CI. `sdd_repository_integrity_target` GREEN; baseline skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. Does **not** renumber existing `0003-*` / `0005-*` / `0007-*` / `0008-*` ADR files. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (crate graph as built), [ADR 0004](0004-documentation-architecture.md) (specs / ADRs / contracts / `.sdd/`) |
| Spec | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) |
| Characterization | `f560196c57e77df2573cfb9a4b384d3cf1c21e8a` |
| Tests | `sdd_repository_integrity_target` GREEN (`tests/contracts/repository_integrity.target.rs`, RI-T01–T16). `sdd_repository_integrity_baseline` `#[ignore = "superseded by sdd_repository_integrity_target"]`. Parser/fixture coverage in `cargo test -p xtask`. Dual-suite registered in root `Cargo.toml`. |

> Filename **`0009-*`**. This is the next unused **unique** ADR number. Cite **this file by path**. Do **not** add a `0003-repository-integrity.md` sibling. Duplicate `0003-*` IDs already exist and are debt (`DEBT-DUP-ADR`), not a license to mint another `0003`. Next unique number is **0010**.

## Context

On SHA `f560196c…` the repository has an inwardly extensible assurance runtime (seven workspace crates + root CLI) and many dual-suite contracts, but:

1. There was no `architecture/` tree, no concept-ownership table, no forbidden-pattern file, and no debt register.
2. There was no `xtask` package and no `.cargo/config.toml` alias. `cargo xtask` / `cargo run -p xtask` failed.
3. CI (`.github/workflows/ci.yml`) ran fmt, clippy (`--features demo`, not `--workspace`), and `cargo test --features demo --all-targets`. It did not run a repository health command.
4. Concept homes are tribal and easy to misname: the catalog crate is `weeping-angel-canonical-catalog` (not `weeping-angel-catalog`); the assurance CLI is root `src/main.rs` + `src/cli.rs` (not `weeping-angel-assurance-cli`).
5. ADR filenames reuse IDs (`0003` × 25, plus `0005`/`0007`/`0008` collisions). Later P0 remediations had no fail-closed place to record “resolved” with proof.

Without a gate, P0 remediations (pack parsing, digest, lineage, SoA, persistence) can land without regression controls, and ownership can drift into invented package names.

Questions this decision answers:

1. Where is canonical **concept ownership** recorded?
2. What is the **single** repository health command, and which checks actually run in increment 1?
3. When may a debt finding be `resolved`?
4. Which crate names are law (live tree vs hypothetical)?
5. How do stubs fail closed instead of silently passing?

## Decision (shipped)

This is what shipped. Field-level law is [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md).

### 1. Architecture manifests are SSOT for ownership

```text
architecture/architecture.toml     schema weeping-angel/architecture/v1
architecture/invariants.toml       schema weeping-angel/architecture-invariants/v1
architecture/forbidden-patterns.toml  schema weeping-angel/forbidden-patterns/v1
```

`architecture.toml` contains the canonical ownership table. Required concepts and **live** crates:

| Concept | Package | Canonical paths |
| --- | --- | --- |
| `catalog` | `weeping-angel-canonical-catalog` | `crates/weeping-angel-canonical-catalog` |
| `framework_compilation` | `weeping-angel-framework` | `crates/weeping-angel-framework` |
| `readiness_projection` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/readiness.rs` |
| `temporal_evidence_selection` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/temporal.rs` |
| `assessment_lineage` | `weeping-angel-assurance` | `crates/weeping-angel-assurance/src/lineage.rs` |
| `evidence_persistence` | `weeping-angel-evidence` | `crates/weeping-angel-evidence` |
| `assurance_cli` | `weeping-angel` | `src/main.rs`, `src/cli.rs` |

Packages `weeping-angel-catalog` and `weeping-angel-assurance-cli` **must not** be created. Mapping them in the ownership table is a check-02 failure.

`temporal_evidence_selection` primitives also exist in `weeping-angel-control-test::temporal::select_latest_as_of`. Ownership is declared on the assurance facade; this slice does not move that code.

`invariants.toml` is declared this increment; **evaluating** invariants is guard check 04 (stub). File presence is not a claim that all invariants hold.

`forbidden-patterns.toml` seeds (check 03 is presence + schema parse; grep/AST enforcement is remaining_backlog):

- package `weeping-angel-catalog`
- package `weeping-angel-assurance-cli`
- path `tests/sdd/` (ADR 0004)

### 2. Debt register with proof-of-resolution

`docs/debt/register.toml` (`weeping-angel/debt-register/v1`) is the only debt SSOT. Status ∈ `open|confirmed|in-progress|resolved|rejected|superseded`.

Required fields per `[[finding]]`: `id`, `title`, `status`, `summary`. IDs must be unique.

A finding may be `status = "resolved"` **only if** it lists non-empty `regression_tests` or `repository_guard`. Check 13 rejects resolved-without-proof and duplicate ids. `rejected` and `superseded` do not require proof.

`docs/debt/README.md` is the status-machine / proof-law note, not a second register. `docs/debt/baseline-2026-08.md` is a dated live-count snapshot, not the register.

### 3. `cargo xtask guard` is the health command

Workspace member `xtask` (`publish = false`, `[package.metadata.dist] dist = false`) + `.cargo/config.toml`:

```toml
[alias]
xtask = "run --package xtask --"
```

Implemented checks this increment: **01** (architecture manifest), **02** (ownership table), **03** (forbidden-patterns file), **13** (debt register).

Checks **04–12** and **14–15** are stubs. Shipped policy:

- skip with report line `NN  <name>  skip(DEBT-GUARD-NN)` when that finding id exists in the register
- otherwise fail closed: `not-yet-implemented: check NN (no registered DEBT-GUARD-NN finding)`
- process exit **0** if no implemented check failed (skips are not failures); silent pass is forbidden

CI job `test` runs `cargo xtask guard` as a mandatory step (`repository health gate`) after rustfmt and before clippy. Existing fmt / clippy / test / demo-example steps are unchanged. Clippy/test remain `--features demo --all-targets`, not `--workspace`. Root `cargo test --all-targets` does not run `cargo test -p xtask`; the guard step is the CI health command.

### 4. Dual-suite is explicit, not auto-discovered

`[[test]]` names `sdd_repository_integrity_{baseline,target}` are registered in root `Cargo.toml`. Human spec remains [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md); `docs/sdd/repository-integrity.md` is a pointer stub (ADR 0004). That spec path is in `CANONICAL_SPECS`.

## Non-goals (remaining_backlog)

P0 framework expression preservation; fail-closed pack parsing; catalog SSOT migration; framework digest redesign; readiness SSOT; lineage rebuild; evidence latest vs current; SoA; persistence invariants; package install tests; crate dependency graph policy; schema fixtures; ADR graph uniqueness rewrite; spec lifecycle states; deleting obsolete baseline suites; test-support crate; implementing guard checks 04–12 / 14–15 beyond stubs; switching CI to `--workspace`.

## Consequences

- Contributors run `cargo xtask guard` locally and in CI; missing manifests or dishonest `resolved` debt fail the build.
- Later remediations must add regression tests or a `repository_guard` citation before closing debt.
- Ownership mistakes (wrong crate names) fail check 02 instead of silently growing a parallel package.
- Duplicate ADR IDs remain until check 14; new decisions use unused unique numbers (0010, …).
- Stub skips are attributable (`DEBT-GUARD-NN`); they are not a generic hatch to skip the gate.

## Related

- Spec: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md)
- Debt register: [`docs/debt/register.toml`](../debt/register.toml), [`docs/debt/README.md`](../debt/README.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Crate graph: [ADR 0001](0001-inwardly-extensible-assurance-runtime.md)
