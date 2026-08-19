# Generated SDD output

This directory is **not** the architecture SSOT.

| Path | Contents |
| --- | --- |
| `runs/` | Workflow traces, dual-suite reports, telemetry, abort records, failure notes |
| `artifacts/` | Snapshots and generated packs (including elite xylex packs) |

Both subtrees are gitignored (see [`.gitignore`](../.gitignore)). Keep them locally if a run needs replay; do not commit routine successful traces.

Canonical surfaces:

- Specs: [`docs/specs/`](../docs/specs/)
- Decisions: [`docs/adr/`](../docs/adr/)
- Executable invariants: [`tests/contracts/`](../tests/contracts/)
