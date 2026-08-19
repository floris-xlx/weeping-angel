# Documentation map

The source tree communicates architecture. Agent execution evidence does not belong next to the specs.

| Path | Role |
| --- | --- |
| [`docs/specs/`](specs/) | Human canonical specifications |
| [`docs/adr/`](adr/) | Decisions |
| [`tests/contracts/`](../tests/contracts/) | Executable invariants (dual-suite target + superseded baseline) |
| [`.sdd/runs/`](../.sdd/) | Generated execution history (gitignored) |
| [`.sdd/artifacts/`](../.sdd/) | Generated snapshots and elite packs (gitignored) |
| Implementation seeds | Packs that produce specs; they are not the SSOT |

Do not add successful workflow traces, telemetry JSON, run directories (`sdd-*` / `sdd-sdd-*`), or repository snapshots under `docs/`. Write those under `.sdd/runs/` or `.sdd/artifacts/` so a reader can learn the architecture without paging through tens of thousands of lines of agent evidence.

Layout decision: [`docs/adr/0004-documentation-architecture.md`](adr/0004-documentation-architecture.md).
