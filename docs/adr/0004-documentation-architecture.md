# ADR 0004 — Documentation architecture (specs, ADRs, contracts, generated SDD)

<!-- weeping-angel-adr-meta
id = "0004"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0001-inwardly-extensible-assurance-runtime"]
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Relocates documentation that previously mixed SSOT and agent traces under `docs/sdd/`. Does **not** change assurance runtime, catalog, or collector decisions. Dual-suite home for architecture-as-law increment: [ADR 0010](0010-architecture-as-law.md) (`xtask/tests/*.rs`). |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (where the spine spec and public contract live) |

## Context

Assurance work produced durable specs, ADRs, and dual-suite tests **and** large generated traces in the same tree: `docs/sdd/<spec>.md` next to `sdd-*.md` reports, `*-telemetry.json`, `sdd-sdd-*` run directories, abort records, and an elite xylex pack.

A reader should be able to learn the architecture from the source repository without navigating tens of thousands of lines of agent execution evidence. Routine successful workflow traces are not architecture.

## Decision

Own each class of document in one place:

| Path | Owner | In git |
| --- | --- | --- |
| [`docs/specs/`](../specs/) | Human canonical specifications | Yes |
| [`docs/adr/`](./) | Decisions | Yes |
| [`tests/contracts/`](../../tests/contracts/) | Executable invariants | Yes |
| [`.sdd/runs/`](../../.sdd/) | Generated execution history | No (gitignored) |
| [`.sdd/artifacts/`](../../.sdd/) | Snapshots and generated packs | No (gitignored) |

Rules:

1. **Specs** are the human SSOT. Dual-suite reports, telemetry, and run folders MUST NOT live beside them.
2. **ADRs** stay under `docs/adr/`. Cite specs and tests by those canonical paths.
3. **Executable invariants** are `tests/contracts/*.target.rs` (and superseded `*.baseline.rs`). They remain explicitly listed in root [`Cargo.toml`](../../Cargo.toml); the directory is not Cargo auto-discovery. [ADR 0010](0010-architecture-as-law.md) increment-1 law additionally lives in `xtask/tests/*.rs` (`cargo test -p xtask`); still never `tests/sdd/`.
4. **Generated SDD output** writes to `.sdd/runs/` (history) and `.sdd/artifacts/` (snapshots). Successful traces MUST NOT be added to the primary source tree. [`docs/sdd/`](../sdd/) is a stub, not an execution dump.
5. Stubs at [`docs/sdd/README.md`](../sdd/README.md) and [`docs/contracts/README.md`](../contracts/README.md) point at the new locations. They are not a second SSOT.
6. **Non-SDD generated artifacts** (raw `audit.txt`, Python `__pycache__` / `*.pyc`, local scan dumps) follow the same rule: not git source. [ADR 0012](0012-repository-hygiene.md) is the hygiene extension.

The public assurance contract previously at `docs/contracts/assurance-runtime.md` is [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md). It remains a human document; the executable contract is `tests/contracts/`.

## Non-goals

- Splitting existing ADRs into technical/stakeholder pairs.
- Changing product behavior, catalog IDs, or dual-suite names (`sdd_*` test names stay).
- Committing `.sdd/runs/` or `.sdd/artifacts/` “for completeness.”

## Consequences

- Links that said `docs/sdd/<spec>.md` now say `docs/specs/<spec>.md`.
- Links that said `tests/sdd/` now say `tests/contracts/`.
- Historical freeze reports remain on disk under `.sdd/` when a developer has run SDD locally; they are not required to clone the architecture.
- New SDD workflows MUST write specs to `docs/specs/`, tests to `tests/contracts/`, and traces to `.sdd/`.

## Related

- Map: [`docs/README.md`](../README.md)
- Generated-output note: [`.sdd/README.md`](../../.sdd/README.md)
- Layout invariant: `sdd_documentation_layout` (`tests/contracts/documentation_layout.rs`)
- Hygiene (schemas, panic budget, admission, generated non-SDD artifacts): [ADR 0012](0012-repository-hygiene.md)
