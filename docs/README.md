# Documentation map

The source tree communicates architecture. Agent execution evidence does not belong next to the specs.

| Path | Role |
| --- | --- |
| [`docs/specs/`](specs/) | Human canonical specifications |
| [`docs/adr/`](adr/) | Decisions |
| [`architecture/`](../architecture/) | Crate-level concept ownership (`architecture.toml` `[ownership.*]` with `kind`) **and** concept-level five-role SSOT [`domain-ownership.toml`](../architecture/domain-ownership.toml) ([ADR 0050](adr/0050-domain-ownership-model.md)); evaluated invariants; executable forbidden patterns ([ADR 0009](adr/0009-repository-health-gate.md), [ADR 0010](adr/0010-architecture-as-law.md)) |
| [`docs/debt/`](debt/) | Technical-debt register (`register.toml`); live mechanical counts [`current.md`](debt/current.md) via `cargo xtask inventory`; frozen Phase 0 snapshot [`consolidation-baseline.md`](debt/consolidation-baseline.md) / [`.json`](debt/consolidation-baseline.json); duplication backlog [`structural-duplication.toml`](debt/structural-duplication.toml) v2 ([ADR 0049](adr/0049-architectural-consolidation-phase-0.md)). [`baseline-2026-08.md`](debt/baseline-2026-08.md) is Historical only ([ADR 0048](adr/0048-structural-reconciliation.md)). Hygiene **counts** are not recorded here ([`repository-hygiene.md`](specs/repository-hygiene.md) §12). |
| [`tests/contracts/`](../tests/contracts/) | Executable invariants (dual-suite target + superseded baseline). Discovery: one harness [`apps/cli/tests/harness.rs`](../apps/cli/tests/harness.rs) ([ADR 0051](adr/0051-repository-environment.md)) — not [`docs/contracts/README.md`](contracts/README.md) and not a root `Cargo.toml` catalog. xtask architecture suites (incl. structural reconciliation) live under `xtask/tests/` (auto-discovered). |
| [`schemas/codex-security/`](../schemas/codex-security/) | Codex Security JSON Schema SSOT (only tracked schema tree; `DEBT-SCHEMA-DUP` resolved). |
| [`.sdd/runs/`](../.sdd/) | Generated execution history (gitignored) |
| [`.sdd/artifacts/`](../.sdd/) | Generated snapshots and elite packs (gitignored) |
| [`docs/sdd/`](sdd/) | Stub only. Not an execution dump. |
| Implementation seeds | Packs that produce specs; they are not the SSOT |

Do not add successful workflow traces, telemetry JSON, run directories (`sdd-*` / `sdd-sdd-*`), raw `audit.txt` / scan logs, or repository snapshots under `docs/`. Write those under `.sdd/runs/` or `.sdd/artifacts/` so a reader can learn the architecture without paging through tens of thousands of lines of agent evidence.

Layout decision: [`docs/adr/0004-documentation-architecture.md`](adr/0004-documentation-architecture.md). Repository health gate: [`docs/adr/0009-repository-health-gate.md`](adr/0009-repository-health-gate.md). Architecture-as-law (Guard 04 + `RepositoryModel`): [`docs/adr/0010-architecture-as-law.md`](adr/0010-architecture-as-law.md). Repository hygiene (panic budget, schema SSOT, generated artifacts, `.gitignore`): [`docs/adr/0012-repository-hygiene.md`](adr/0012-repository-hygiene.md). Structural reconciliation (inventory + mechanical debt snapshot + active-spec drift): [`docs/adr/0048-structural-reconciliation.md`](adr/0048-structural-reconciliation.md). Architectural consolidation Phase 0 (program table, frozen baseline, duplication backlog): [`docs/adr/0049-architectural-consolidation-phase-0.md`](adr/0049-architectural-consolidation-phase-0.md). Phase 1 domain-ownership law (five roles, fail-closed sibling SSOT): [`docs/adr/0050-domain-ownership-model.md`](adr/0050-domain-ownership-model.md). Repository environment (virtual Cargo workspace, CLI path, toolchain pin, one harness): [`docs/adr/0051-repository-environment.md`](adr/0051-repository-environment.md). Commands: `cargo xtask guard [--json] [--check NN] [--explain INV-…]`; `cargo xtask inventory [--json\|--markdown\|--check\|--consolidation-baseline]`.

Start here:

- [Assurance runtime](specs/assurance-runtime.md)
- [Collector hexagonal modular monolith](specs/collector-hexagonal.md) ([ADR 0013](adr/0013-collector-hexagonal-modular-monolith.md)) — crate layout; adapters emit observations; `EnvelopeFactory` seals
- [GitHub collector](specs/github-collector.md) — evidence IDs, mappings, 403/404 (not crate layout)
- [Continuous assurance scheduler](specs/continuous-assurance-scheduler.md) — still consumes `EvidenceCollector`
- [Structural reconciliation](specs/structural-reconciliation.md) — Phase 0+1 inventory / debt honesty ([ADR 0048](adr/0048-structural-reconciliation.md))
- [Architectural consolidation](specs/architectural-consolidation-program.md) — Phase 0 freeze + baseline + backlog schema ([ADR 0049](adr/0049-architectural-consolidation-phase-0.md)); Phase 1 concept-level ownership law ([ADR 0050](adr/0050-domain-ownership-model.md))
