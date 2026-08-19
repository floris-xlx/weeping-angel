# ADR 0012 — Repository hygiene (panic budget, schema SSOT, generated artifacts, dual-suite collapse)

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_repository_hygiene_target` is fail-closed hygiene law (14/14 GREEN). Hygiene baseline is skip-superseded (`#[ignore = "superseded by sdd_repository_hygiene_target"]`); found-case debt no longer holds. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, collector, or repository-guard decisions. **Extends** [ADR 0004](0004-documentation-architecture.md) for generated *non-SDD* artifacts (`audit.txt`, Python bytecode). Does **not** amend [ADR 0009](0009-repository-health-gate.md) / [ADR 0010](0010-architecture-as-law.md). Does **not** mint another `0003-*` or a fourth `0011-*` (Prompts 1–3 already drafted `docs/adr/0011-*.md`). |
| Extends | [ADR 0004](0004-documentation-architecture.md) (specs / ADRs / contracts / `.sdd/`) |
| Spec | [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md) |
| Characterization | `0015f6395e7ead042e3cfd3066fefde3d39aa36b` |
| Tests | `sdd_repository_hygiene_target` GREEN (`tests/contracts/repository_hygiene.target.rs`). `sdd_repository_hygiene_baseline` skip-superseded. Both registered in root `Cargo.toml`. Neighbor `sdd_documentation_layout` indexes this spec in `CANONICAL_SPECS`. |

> Filename **`0012-*`**. Cite **this file by path**. Concurrent cleanup drafts occupy `0011-*`. Duplicate historical `0003-*` / `0005-*` / `0007-*` / `0008-*` prefixes remain `DEBT-DUP-ADR` (Prompt 1). Do **not** add `0003-repository-hygiene.md`.

<!-- weeping-angel-adr-meta
id = "0012"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture"]
-->

## Context

On SHA `0015f63…` the repository already has an assurance runtime and a health gate (`cargo xtask guard`), but accumulated **hygiene debt** that is not product semantics:

1. **632** line-starting `#[ignore…]` attributes, all “superseded by …” leftovers on dual-suite baselines (debt snapshot 661). Cargo still compiles those suites.
2. **38** `*.baseline.rs` + **39** `*.target.rs` plus 80 root `[[test]]` rows. Completed targets are law; ignored baselines remain compile-time tax.
3. **16** `fn require_needles` / **203** `require_needles(` matches, all in concurrent Prompt 2/3 `*.target.rs` files.
4. Root `src/`: **174** `.unwrap()` and **60** `.expect(` (≈88 / 52 outside tests). No Clippy `unwrap_used`. Network/HTTP already returns `Result`; regex literals panic only if the literal is wrong; some CLI/IO paths still need a typed budget.
5. Codex Security JSON Schemas are **byte-identical** in `schemas/codex-security/` and `codex-security/schemas/` (`DEBT-SCHEMA-DUP`).
6. Tracked `audit.txt` is raw `xbp audit` output (~2 710 findings). 21 tracked `*.pyc` files.
7. `.gitignore` misses `.env*` generically, `target-*`, `__pycache__`, `*.pem`/`*.key`, `*.sqlite`, `.idea`.
8. [`docs/contracts/README.md`](../contracts/README.md) is a hand-maintained dual-suite inventory. Root README is already capability-oriented.

Prompt 1 owns `xtask` and the debt register. This decision must be enforceable **without** editing the guard engine.

Questions this decision answers:

1. Where is the Codex Security JSON Schema SSOT, and how may a second path exist?
2. What generated artifacts may be git source?
3. What is the production panic budget, and how is it enforced without `xtask`?
4. When may an ignore-superseded dual-suite baseline be deleted?
5. Where are before/after hygiene counts recorded?

## Decision

Field-level law is [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md).

### 1. Schema SSOT

Authoritative files:

```text
schemas/codex-security/coverage.schema.json
schemas/codex-security/findings.schema.json
schemas/codex-security/scan-manifest.schema.json
```

`codex-security/schemas/` is **not** a second editorial source. The tree is a **generated packaging copy**, stamped by `codex-security/schemas/GENERATED_FROM_SSOT`, and must stay SHA-256 identical to the SSOT (hygiene contract test). Refresh from repo root: copy `schemas/codex-security/*.schema.json` onto `codex-security/schemas/`.

JSON Schema `$id` URLs (`https://openai.com/codex-security/schemas/…`) are identifiers, not filesystem homes.

### 2. Generated artifacts are not source

Extends ADR 0004:

| Path | Role | In git |
| --- | --- | --- |
| `docs/specs/`, `docs/adr/`, `tests/contracts/` | SSOT | Yes |
| `.sdd/runs/`, `.sdd/artifacts/` | SDD traces | No |
| `audit.txt` and raw `xbp` / scan logs | Execution output | No |
| `**/__pycache__/`, `*.pyc` | Interpreter cache | No |

A compact audit **manifest** (generator, schema version, source commit, digest, finding **counts**) may live untracked under `.sdd/artifacts/` or, if a committed pointer is required, as a small structured file — never a megabyte log.

### 3. Production panic budget

Runtime failure that can come from **input, IO, network, auth, reporting, or workbench state** in root `src/**` (excluding `#[cfg(test)]` and the demo lab) returns a typed `Result` / error with context.

`.unwrap()` / `.expect()` remain only for:

- statically closed programmer errors (invalid regex **literals**, exhaustive-branch `unwrap`)
- tests, examples, build scripts

Exceptions are explicit (`// panic-ok: …` or Clippy `#[expect]`). Implemented marks are `// panic-ok: regex literal` on statically closed `Regex::new` / `OnceLock` compiles in `src/discovery/` and `src/depcheck/parsers/`. Runtime conversions on this increment: `scan-diff` missing `--base` returns `anyhow` instead of `unwrap`; Pipfile `split_once` is a `continue` on malformed lines.

Enforcement is the **hygiene contract test** (budgeted prefixes under `src/{parse,http,authz,report,workbench,cli,lib,main,contract,discovery,depcheck/parsers}`). Root `[lints.clippy] unwrap_used` was **not** enabled — workspace `--all-targets` would force mass allows on tests and Prompt 2/3 crates. Do not add it here.

This slice does not convert unwraps in Prompt 2/3 crates.

### 4. Dual-suite collapse and `#[ignore]`

- Target suites that are already authoritative **may** drop their ignore-superseded baseline file and `[[test]]` row **only** when the files are not owned by Prompts 1–3 and no skipped suite asserts the baseline’s presence.
- **This increment collapsed none of those pairs.** Nearly every GREEN `*.target.rs` still asserts its `*.baseline.rs` path in `Cargo.toml`. Deleting them would fail Prompt 1–3 targets. Hygiene reports the skip list; it does not delete foreign baselines to move a count.
- Do not delete coverage to improve counts.
- Do not add `#[ignore]` to hide a red test. The only new ignore allowed on hygiene-owned files is the skip-supersede attr on `repository_hygiene.baseline.rs`.
- `tests/contracts/` remains explicitly listed in root `Cargo.toml` (ADR 0004). Auto-discovery continues to own `tests/*.rs`. `e2e_demo` / `e2e_recon` stay explicit because of `required-features = ["demo"]`.
- Source-grep `require_needles` is forbidden in hygiene-owned tests. Existing needles in Prompt 2/3 targets are left in place during concurrent work.

### 5. Admission hygiene

Root `.gitignore` must fail-closed for secrets (`.env` / `.env.*`), `node_modules/`, Rust `target/` and `target-*/`, `.sdd/runs/` + `.sdd/artifacts/`, local sqlite, private keys, Python bytecode, editor caches, and raw audit/scan dumps — without ignoring `tests/fixtures/**` or schema examples.

### 6. Documentation indexes

Root README explains capabilities, architecture, and canonical commands. Dual-suite inventories are **not** hand-synchronized in `docs/contracts/README.md`; point at `docs/specs/`, `tests/contracts/`, and `Cargo.toml`.

Hygiene metrics are recorded in the spec and/or `.sdd/runs/`, **not** in `docs/debt/register.toml` (Prompt 1).

### 7. Forbidden names and paths

Unchanged: do not invent `weeping-angel-catalog` or `weeping-angel-assurance-cli`; do not create `tests/sdd/`.

## Consequences

- Contributors stop treating ignored baselines, audit logs, and duplicate schemas as source of truth.
- Scanner runtime panics on plausible input become typed errors over time, gated by the hygiene target.
- Prompt 1 can later close `DEBT-IGNORE` / `DEBT-UNWRAP` / `DEBT-SCHEMA-DUP` using this slice’s proof without this slice editing the register.
- Concurrent Prompts 2/3 keep their `require_needles` targets and dual-suite baselines until a non-colliding pass.
- Before/after counts live in the spec §12 (and optionally `.sdd/runs/repository-hygiene-counts.md`), never `docs/debt/register.toml`.
- Root README stays capability + CLI. `docs/contracts/README.md` is a pointer (`rg "^name = \"sdd_" Cargo.toml`), not a hand-maintained suite inventory.

## Non-goals

- Implementing or rewriting `cargo xtask guard`.
- Catalog / framework / temporal / lineage / SoA product semantics.
- Mass-renumbering historical ADRs.

## Related

- Spec: [`docs/specs/repository-hygiene.md`](../specs/repository-hygiene.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Health gate (do not edit): [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md)
- Schema SSOT: [`schemas/codex-security/`](../../schemas/codex-security/); generated copy stamp: [`codex-security/schemas/GENERATED_FROM_SSOT`](../../codex-security/schemas/GENERATED_FROM_SSOT)
- Index: [`docs/contracts/README.md`](../contracts/README.md)
