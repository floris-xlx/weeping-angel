# ADR 0011 — Repository guard governance (modular engine, architecture policy SSOT, Guards 14–15, debt expiry)

| Field | Value |
| --- | --- |
| Status | **Accepted** — increment 2 implemented: modular guard engine, architecture policy SSOT, real Guards 14–15, fail-closed debt expiry. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Does not rewrite** [ADR 0009](0009-repository-health-gate.md) or [ADR 0010](0010-architecture-as-law.md) bodies. **Amends** their remaining-backlog: Guards **14** and **15** become real; policy constants move into `architecture/`; debt exemptions expire. Does **not** mass-renumber existing `0003-*` / `0005-*` / `0007-*` / `0008-*` files or the concurrent `0011-*` siblings. |
| Extends | [ADR 0004](0004-documentation-architecture.md), [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md) |
| Spec | [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) §10–§18 |
| Tests | `sdd_repository_integrity_target` increment-2 IDs (RI-T18–T31) GREEN; increment-2 baseline RI-B11–B18 `#[ignore]`-superseded. `cargo test -p xtask` ACP-T01–T03 discover the modular tree. |

<!-- weeping-angel-adr-meta
id = "0011"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0009-repository-health-gate", "0010-architecture-as-law"]
-->

> Filename **`0011-repository-guard-governance.md`**. Cite **this file by path**. Concurrent Prompts 2/3 also minted `0011-*` files; those collisions are pinned in `architecture/adr-identity.toml` (`DEBT-DUP-ADR`). Prompt 4 used [0012](0012-repository-hygiene.md). Do **not** add a fourth `0011-*` or a `0003-repository-guard-governance.md` sibling. Next unused unique prefix is **0013**.

## Context

ADR 0009 shipped presence-only checks **01, 02, 03, 13**. ADR 0010 made `RepositoryModel` + `ArchitectureCheck` the evaluation plane, implemented Guard **04**, required ownership `kind`, executed forbidden-pattern kinds, and structured `GuardReport` / CLI (`--json` / `--check` / `--explain`).

That is still not a durable governance system:

1. `xtask/src/lib.rs` is a ~1434-line monolith. ACP-T01/T02 grep that single file for `RepositoryModel` / `ArchitectureCheck`.
2. `source_files` is a path list; `source_contains` and `symbol`/`in_crate` reread disk per check.
3. `REQUIRED_OWNERSHIP`, `OWNERSHIP_KINDS`, `FORBIDDEN_PACKAGES`, and `REMAINING_STUBS` duplicate `architecture/` TOML.
4. Guards **14** and **15** skip with `DEBT-GUARD-14` / `DEBT-GUARD-15`. A new duplicate ADR prefix can land. Specs have no machine lifecycle.
5. Live skip exemptions lack owner, dates, severity, remediation, and expiry — expired debt cannot fail CI.
6. JSON has no schema/version/counts; `duration` is wall-clock.
7. CI already always runs `cargo xtask guard`; that property must not be lost to path filters.

Questions this decision answers:

1. Where does repository **policy** live (Rust vs `architecture/`)?
2. What is the **module** boundary of the guard engine?
3. How are **ADR identities** unique for new files without silent historical renumber?
4. What is the **spec lifecycle** machine and where is it stored?
5. When does a **debt exemption** expire?
6. How is **JSON** extended without breaking current keys?

## Decision (shipped)

Field-level law is [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) §13. What follows is the increment-2 contract as implemented.

### 1. Modular evaluation plane

`xtask` is `model`, `architecture`, `debt`, `checks`, and `report` plus a thin `lib.rs` / `main.rs`. `run_guard` / `run_guard_with_options` load **one** `RepositoryModel`. Normalized Rust source is cached at construction (`BTreeMap` by repo-relative path). Walk order and report order are deterministic. Public CLI (`guard`, `--json`, `--check NN`, `--explain INV-…`) and human `NN  name  pass|fail|skip(DEBT-…)` lines are preserved. Public wrappers `check_01_*`…`check_04_*` may still reload the model.

### 2. Policy is versioned under `architecture/`

Rust validates and interprets. It is not the SSOT for ownership kinds, required concept keys, or forbidden package names.

- `architecture/architecture.toml` `[policy]` lists `ownership_kinds` and `required_concepts`. Optional ownership row `repository_guard` → package `xtask`.
- Forbidden packages stay `architecture/forbidden-patterns.toml` `kind = "package"`.
- `architecture/adr-identity.toml` (`weeping-angel/adr-identity/v1`) lists `grandfathered_debt`, `grandfathered_prefixes`, and the pinned `grandfathered_files` set.
- `architecture/spec-lifecycle.toml` (`weeping-angel/spec-lifecycle/v1`) lists every `docs/specs/*.md`.

Missing or malformed manifests fail closed (never skip).

### 3. Guard 14 is a real ADR identity/graph check

- Filename `NNNN-slug.md`. Prefix uniqueness for **new** files.
- Prefix collisions are legal **only** when the prefix is in `grandfathered_prefixes`, the file is in `grandfathered_files`, and `DEBT-DUP-ADR` is live. Pinned prefixes: `0003`, `0005`, `0007`, `0008`, and concurrent increment-2 `0011`. A new path that reuses a pinned prefix fails. No silent renumber.
- Each ADR has one `<!-- weeping-angel-adr-meta ... -->` block (`id`, `status`, `supersedes`, `superseded_by`, `depends_on`). Graph nodes are **filename stems**. Ambiguous four-digit prefix references fail when the prefix is not unique.
- Dangling edges and cycles on `supersedes` / `depends_on` fail. Inverse `supersedes`/`superseded_by` must agree.
- `docs/adr/**` edits in this increment are metadata/identity only (plus this file). Do not rewrite 0009/0010 decisions.

`DEBT-GUARD-14` is `resolved` with `repository_guard = "14"` and `regression_tests = ["sdd_repository_integrity_target", "sdd_architectural_cleanup_target"]`.

### 4. Guard 15 is a real spec-lifecycle / spec-dependency check

States: `draft | active | superseded | retired` with the transitions in the spec. Every on-disk `docs/specs/*.md` has a row. Active specs reference existing `[ownership]` keys. Superseded rows require a successor path; superseded/retired rows cannot use `state = "active"`. Spec `depends_on` is acyclic and repository-bound. Crate-graph product law stays check 03 `kind = dependency` / later Phase 19 — not incomplete Prompt 2/3 semantics inside xtask.

`DEBT-GUARD-15` is `resolved` with `repository_guard = "15"` and the same named regression tests.

### 5. Debt exemptions expire

Live guard exemptions require `owner`, `introduced`, `severity`, `remediation`, associated `repository_guard`, and `expires` or `review_by`. Expired exemptions fail check 13 (`expired debt <id>`). Resolved still needs live guard or named tests. Malformed, duplicate, and orphaned IDs fail. Evaluation date is UTC, overridable with `WEEPING_ANGEL_GUARD_AS_OF` for fixtures.

Checks **05–12** remain skip-with-debt plumbing until Prompts 2/3 implement product semantics. Those skip findings are owned, dated, and expire.

### 6. Additive JSON

`schema = "weeping-angel/guard-report/v1"`, `version`, `counts`, `failed` are added (`failed` is the same list as `violations`). Existing keys including `duration` remain. Equality-sensitive fixtures must not compare `duration`.

### 7. CI

`.github/workflows/ci.yml` keeps a mandatory `cargo xtask guard` step. Path filters must not bypass architecture, ADRs, specs, debt, workspace manifests, frameworks/catalog, or Rust source.

## Non-goals

- Guards 05–12 product semantics; moving product code; ADR mass-renumber; Prompt 4 hygiene/schemas/panic budget; `--workspace` CI; hypothetical crates; a second spec SSOT; rewriting ADR 0009/0010.

## Consequences

- Architecture + ADR metadata + spec lifecycle + debt expiry are executable law.
- New duplicate ADR prefixes cannot land; historical collisions stay explicit debt.
- Stub skips cannot live forever without an owner and a date.
- Later phases still cannot add a second evaluation plane or `tests/sdd/`.

## Related

- Spec: [`docs/specs/repository-integrity.md`](../specs/repository-integrity.md) increment 2
- Predecessor: [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Debt: [`docs/debt/register.toml`](../debt/register.toml)
