# Documentation map

The source tree communicates architecture. Agent execution evidence does not belong next to the specs.

| Path | Role |
| --- | --- |
| [`docs/specs/`](specs/) | Human canonical specifications |
| [`docs/adr/`](adr/) | Decisions |
| [`architecture/`](../architecture/) | Concept ownership (with `kind`), evaluated invariants, executable forbidden patterns ([ADR 0009](adr/0009-repository-health-gate.md), [ADR 0010](adr/0010-architecture-as-law.md)) |
| [`docs/debt/`](debt/) | Technical-debt register (`register.toml`); mechanical current counts [`current.md`](debt/current.md) via `cargo xtask inventory`; [`baseline-2026-08.md`](debt/baseline-2026-08.md) is Historical only ([ADR 0048](adr/0048-structural-reconciliation.md)). Hygiene **counts** are not recorded here ([`repository-hygiene.md`](specs/repository-hygiene.md) §12). |
| [`tests/contracts/`](../tests/contracts/) | Executable invariants (dual-suite target + superseded baseline). Inventory: `rg "^name = \"sdd_" Cargo.toml` — not [`docs/contracts/README.md`](contracts/README.md). xtask architecture suites (incl. structural reconciliation) live under `xtask/tests/` (auto-discovered). |
| [`schemas/codex-security/`](../schemas/codex-security/) | Codex Security JSON Schema SSOT. `codex-security/schemas/` is a generated packaging copy ([ADR 0012](adr/0012-repository-hygiene.md)). |
| [`.sdd/runs/`](../.sdd/) | Generated execution history (gitignored) |
| [`.sdd/artifacts/`](../.sdd/) | Generated snapshots and elite packs (gitignored) |
| [`docs/sdd/`](sdd/) | Stub only. Not an execution dump. |
| Implementation seeds | Packs that produce specs; they are not the SSOT |

Do not add successful workflow traces, telemetry JSON, run directories (`sdd-*` / `sdd-sdd-*`), raw `audit.txt` / scan logs, or repository snapshots under `docs/`. Write those under `.sdd/runs/` or `.sdd/artifacts/` so a reader can learn the architecture without paging through tens of thousands of lines of agent evidence.

Layout decision: [`docs/adr/0004-documentation-architecture.md`](adr/0004-documentation-architecture.md). Repository health gate: [`docs/adr/0009-repository-health-gate.md`](adr/0009-repository-health-gate.md). Architecture-as-law (Guard 04 + `RepositoryModel`): [`docs/adr/0010-architecture-as-law.md`](adr/0010-architecture-as-law.md). Repository hygiene (panic budget, schema SSOT, generated artifacts, `.gitignore`): [`docs/adr/0012-repository-hygiene.md`](adr/0012-repository-hygiene.md). Structural reconciliation (inventory + mechanical debt snapshot + active-spec drift): [`docs/adr/0048-structural-reconciliation.md`](adr/0048-structural-reconciliation.md). Commands: `cargo xtask guard [--json] [--check NN] [--explain INV-…]`; `cargo xtask inventory [--json\|--markdown\|--check]`.

Start here:

- [Assurance runtime](specs/assurance-runtime.md)
- [Collector hexagonal modular monolith](specs/collector-hexagonal.md) ([ADR 0013](adr/0013-collector-hexagonal-modular-monolith.md)) — crate layout; adapters emit observations; `EnvelopeFactory` seals
- [GitHub collector](specs/github-collector.md) — evidence IDs, mappings, 403/404 (not crate layout)
- [Continuous assurance scheduler](specs/continuous-assurance-scheduler.md) — still consumes `EvidenceCollector`
- [Structural reconciliation](specs/structural-reconciliation.md) — Phase 0+1 inventory / debt honesty ([ADR 0048](adr/0048-structural-reconciliation.md))
