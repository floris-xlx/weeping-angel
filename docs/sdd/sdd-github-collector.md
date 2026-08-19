# SDD run: Reference-Grade GitHub Assurance Collector

| Field | Value |
| --- | --- |
| Run id | `sdd-github-collector` (protocol report; implement assigns telemetry id) |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` (dual-suite) |
| Status | **Target RED** — §4.11 authored; baseline GREEN (30); target 1 pass / 29 fail; no product feature code |
| Slice | Prompt 09: turn `GitHubCollector` into the first reference-grade provider collector; emit canonical evidence only |
| Spec | [`docs/sdd/github-collector.md`](github-collector.md) |
| Draft ADR | [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/09-github-collector.md`](../prompts/canonical-assurance-v1/09-github-collector.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Dual-suite | `tests/sdd/github_collector.baseline.rs` · `tests/sdd/github_collector.target.rs` → `sdd_github_collector_baseline` / `sdd_github_collector_target` |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and (later) telemetry.

Treat prior interrupted Scope (1/10) as abandoned. This report is a **fresh start** against HEAD `e430980c…` and the already-uncommitted baseline characterization.

---

## Spec

- **Title:** Reference-Grade GitHub Assurance Collector
- **Problem:** The existing GitHub collector emits ISO-sliver `source.*` string facts for named `repo:owner/name` labels only. It cannot populate provider-neutral SDLC/IAM evidence, so 25–40 canonical controls cannot be exercised from GitHub the same way they would from another SCM.
- **Current behavior (SHA `e430980c…`):** `GitHubCollector` collects scoped `repo:owner/name` only; `normalize.rs` + `protection.rs` emit `source.repository.*` / `source.branch.*` string facts; protection path hardcodes `main`; `branches` / `collaborators` / `repositories` / `rulesets` / `security` / `workflows` are `MODULE` stubs; descriptor advertises pagination plus admin/collaborator/scanning/workflow/ruleset/signing types it does not collect; `CollectorDescriptor` has no `failure_behavior` field; 403 on repo or branch protection aborts the whole `collect` (second-repo 403 discards first-repo envelopes); `collect_batch` wraps `CollectionRun::new` without `completed_at`, scope, `configuration_digest`, evidence/error counts, or partial/complete status; `GitHubClient` is fixture-only (401 without token, `Transport` without fixture; first-prefix-wins match). `redact` covers `ghp_` / `gho_` / `github_pat_` / `Bearer ` / `token=` but not `ghs_`. ISO GH-007/GH-009 needle tests exist. Encoded by `ghc_b001`–`ghc_b030`.
- **Desired behavior:** Map GitHub-native API objects to Prompt 04/05 canonical contracts (`evidence.repository.*`, `evidence.cicd.*`, `evidence.deployment.*`, `evidence.identity.privileged-membership` / `external-access`) plus Prompt 03 `inventory.subject` / `inventory.complete`. Honest descriptor; advertise failure behavior **without** redesigning shared `CollectorDescriptor`; 403 → diagnostic not fabricated negative; complete pagination or explicit partial; real `CollectionRun`; ten golden adapter fixtures; no credential leaks; no ISO logic; no `Effective`/`Ineffective`.
- **ADR:** needed — draft at [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md)

### Acceptance criteria (this slice)

1. Dual-suite remains registered like existing root `[[test]]` SDD suites.
2. Baseline GREEN on current tree characterizing §3 of the spec; target suite encodes §4.11 and is RED until implement.
3. After implement: target GREEN; baseline superseded; `cargo test --workspace --features demo`, `fmt --check`, `clippy -D warnings` stay green.
4. Descriptor honesty: types, permissions, subjects, pagination/incremental, provider family, failure behavior (GitHub-owned const/docs; no shared-type redesign unless strictly required).
5. Canonical evidence only; no `evidence.github.*` required by tests; no framework IDs in the collector.
6. Permission denial → explicit insufficient-evidence diagnostics; no fabricated negatives; no whole-batch abort.
7. Authoritative `inventory.complete` only after complete pagination.
8. `CollectionRun` records version, scope, configuration digest, start/completion, counts, complete/partial/failed.
9. Ten required goldens (healthy org; unprotected repo; missing branch-protection permission; paginated inventory; archived excluded; disabled scanning; absent protected environment; privileged membership; API partial failure; rate-limit/retry).
10. No token/header/cookie material in facts, diagnostics, or fixtures; ISO GH-007/GH-009 remain green without rewriting those suites.
11. `GITHUB_EVIDENCE_TYPES` stays free of `evidence.identity.*`; ISO `source.*` strings remain as the mapping table.
12. ≥25 canonical controls exercisable through emitted contracts; another provider could emit the same types and get the same test results.

### Out of scope

- ISO/SOC2/NIS2 remapping; ISO pack rewrite
- Effectiveness / readiness / SoA
- Catalog ID redesign; Prompt 05–08 catalog TOML
- SaaS credential store
- Other provider collectors
- Mandatory live HTTP in unit tests
- EvidenceValue / population-evaluator redesign
- Shared `CollectorDescriptor.failure_behavior` unless strictly required
- IAM / population / typed-evidence / ISO suite rewrites
- Scanner engines
- Concurrent Prompt 06/07/08 trees

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Dual-emit `source.*` | Spec forbids; mapping table only |
| IAM-015 / ISO GH-012 | Separate identity advertisement; keep `source.*` strings |
| Prompt 05 “collector untouched” snapshots | Prompt 09 owns collector files; do not edit Prompt 05 trees |
| Partial pagination claimed complete | Target golden |
| Token leak / `ghs_` gap | Reuse `redact`; GitHub-owned sanitizer if shared redact cannot grow |
| Provenance digest break | Optional observation `extensions`, not typed-evidence law |
| Shared collector type redesign | Failure behavior documented in `github/descriptor.rs` |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/github-collector.md`](github-collector.md) — fresh-start SSOT, re-characterized against HEAD + baseline suite |
| ADR draft | written if mapping is a contract decision | [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md) |
| Baseline | PASS on old | **PASS** — `sdd_github_collector_baseline` 30 tests (`ghc_b001`–`ghc_b030`) encode §3. Target binary registered, 0 assertions. |
| Target pre | FAIL on old | **FAIL** — `sdd_github_collector_target` `ghc_000`–`ghc_024` (plus inline 012b/013b/014b/018b/019b). 1 keep-alive pass (dual-suite registration); 29 failed on missing canonical emit, goldens, org inventory, 403-continue, filled `CollectionRun`. |
| Implement | target PASS | **Not started** — no production collector/feature code in this spec slice. |
| Baseline post | FAIL or retired | Pending implement + supersede. |
| Supersede | target still PASS | Pending. |
| Docs/ADR | updated | Spec + draft ADR + this report. Accept ADR after target GREEN. Contract/`assurance-runtime.md` GitHub type list updates at implement (not this slice). |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | unset (pre-implement) |
| `baseline_retired` | false |
| `additive_baseline` | false |
| `baseline_not_green` | n/a |
| `target_still_green` | n/a |

`verify_ok` is **not** claimed. Spec-first gate only.

---

## What this spec slice wrote / updated

- [`docs/sdd/github-collector.md`](github-collector.md) — durable SSOT (current behavior, mapping table, goldens, 25-control matrix, §4.11 RED catalog)
- [`docs/sdd/sdd-github-collector.md`](sdd-github-collector.md) — this protocol report
- [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md) — draft mapping ADR

No `crates/weeping-angel-collector/src/github/**` product edits. No catalog TOML. Dual-suite registered; target now encodes §4.11 and is RED.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `characterization_sha` | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| `spec_first` | true |
| `product_code_changed` | false |
| `dual_suite_registered` | true (baseline GREEN 30; target RED 29 fail) |

### Gates (this snapshot)

| Gate | Value |
| --- | --- |
| `spec_written` | true |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | false |
| `baseline_superseded` | false |
