# Elite pack: weeping-angel-assurance-ir

| Field | Value |
| --- | --- |
| Product | `weeping-angel-assurance-ir` |
| Repo | `floris-xlx/weeping-angel` |
| Branch / SHA | `main` / `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7` |
| Recipe | **full** + **contract** + **R13 architecture specification** |
| Mode | Authoring only (no product-code changes in this pack) |
| Supersedes | Completeness claims that the Compliance IR is already a production-grade semantic authority. Does **not** supersede Phases 0–8 spine invariants. |

## Artifacts

- Assessment: [`ASSESS-weeping-angel-assurance-ir.md`](ASSESS-weeping-angel-assurance-ir.md)
- Plan SSOT: [`ELITE-contract-weeping-angel-assurance-ir.md`](ELITE-contract-weeping-angel-assurance-ir.md)
- Handoff: [`handoff-brief.md`](handoff-brief.md)
- Specs: [`specs/`](specs/)
- Findings: [`findings/`](findings/)
- Matrices: [`matrices/`](matrices/)
- Dual-suite: [`dual-suite/`](dual-suite/)
- Supersession: [`supersession/banner-list.md`](supersession/banner-list.md)

## Quality bar

```powershell
python $env:USERPROFILE\Documents\spec-driven-development\scripts\sdd_cli.py validate docs/sdd/xylex/weeping-angel-assurance-ir --require-filled
```

## Next command after this pack

```text
/xylex-sdd mode=implement target_path=crates/weeping-angel-assurance-ir
```

Implement only after Phase 0 acceptance is checked against live `main`.
