# SDD run: Reference-Grade GitHub Assurance Collector

| Field | Value |
| --- | --- |
| Run id | `sdd-github-collector` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` (dual-suite) |
| Status | **Target GREEN** — baseline superseded |
| Slice | Prompt 09: `GitHubCollector` emits canonical evidence only |
| Spec | [`docs/sdd/github-collector.md`](github-collector.md) |
| ADR | [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md) (accepted) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/09-github-collector.md`](../prompts/canonical-assurance-v1/09-github-collector.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Dual-suite | `tests/sdd/github_collector.baseline.rs` (ignored) · `tests/sdd/github_collector.target.rs` (30 pass) |

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/github-collector.md`](github-collector.md) |
| ADR | accept after GREEN | accepted |
| Baseline | PASS on old | Encoded §3, then superseded |
| Target pre | FAIL on old | RED before implement |
| Implement | target PASS | **PASS** — 30 `sdd_github_collector_target` tests |
| Baseline post | FAIL or retired | Retired: `#[ignore = "superseded by sdd_github_collector_target"]` |
| Supersede | target still PASS | Yes |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | ignore-baseline |
| `baseline_retired` | true |
| `additive_baseline` | false |
| `baseline_not_green` | n/a (ignored) |
| `target_still_green` | true |

## Implement notes

- Descriptor advertises `GITHUB_CANONICAL_EVIDENCE_TYPES` only; `GITHUB_EVIDENCE_TYPES` remains the ADR 0002 `source.*` mapping-table list.
- `org:` inventory via `/orgs/{org}/repos` with `Link` / `per_page` walker; `exclude_archived` selector; authoritative `inventory.complete` only after complete pagination.
- 401/403 → `PermissionDenied` diagnostics, never `protected=false`; batch continues.
- `collect_batch` fills `CollectionRun` (version, scope, secret-free digest, start/completion, counts, complete/partial/failed).
- Ten goldens under `fixtures/assurance/canonical/v1/github/`.
- `ghs_` folded by GitHub-owned `sanitize_diagnostic`.
