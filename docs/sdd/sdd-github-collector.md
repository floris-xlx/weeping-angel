# SDD run: Reference-Grade GitHub Assurance Collector

| Field | Value |
| --- | --- |
| Run id | `sdd-e4993ccb7da6` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `e4993ccb7da6e40d` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 09: first reference-grade provider collector. `GitHubCollector` emits only Prompt 03/04/05 canonical contracts, fails closed on 401/403, states inventory completeness honestly, and records a real `CollectionRun`. |
| Spec | [`docs/sdd/github-collector.md`](github-collector.md) |
| ADR | Accepted [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md) (draft filename dropped) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/09-github-collector.md`](../prompts/canonical-assurance-v1/09-github-collector.md) |
| Telemetry | [`sdd-github-collector-telemetry.json`](sdd-github-collector-telemetry.json) |
| Dual-suite | `tests/sdd/github_collector.baseline.rs` (skip-retired; `ghc_b001`–`ghc_b030`) · `tests/sdd/github_collector.target.rs` (active; `ghc_000`–`ghc_024` + `012b`/`013b`/`014b`/`018b`/`019b`) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

This report is the **finalize** artifact for telemetry run `sdd-e4993ccb7da6` against characterization SHA `e430980c…`. It supersedes the shorter implement-era stub previously stored at this path.

---

## Spec

- **Title:** Reference-Grade GitHub Assurance Collector
- **Problem:** The existing GitHub collector is an ISO-sliver prototype: it only walks `repo:owner/name` labels and emits GitHub-shaped `source.*` string facts, so 25–40 provider-neutral canonical controls cannot be exercised the same way a future GitLab/Bitbucket collector would.
- **Current behavior (SHA `e430980`):** `GitHubCollector` accepts only `repo:owner/name` (`org:` is `OutOfScope`). `normalize.rs` emits `source.repository.exists` / `visibility` / `archived` and `source.default_branch` as strings; `protection.rs` hits `/branches/main/protection` (hardcoded `main`) and emits `source.branch.*` string facts (404 ⇒ `enabled=false`; 403 aborts the whole collect). Six modules (`branches` / `collaborators` / `repositories` / `rulesets` / `security` / `workflows`) are MODULE stubs. Descriptor advertises 19 `source.*` types and `pagination=true` but collects only a subset and has no page walker; `CollectorDescriptor` has no `failure_behavior` field. `collect_batch` wraps `CollectionRun::new` leaving `status=started`, empty scope/digest, `evidence_count=0`. Client is fixture-only (401 without token; Transport without fixture; first-prefix-wins). Redact covers `ghp_` / `gho_` / `github_pat_` / `Bearer` / `token=` but not `ghs_`. Encoded by `ghc_b001`–`ghc_b030`.
- **Desired behavior:** Map GitHub-native API objects into Prompt 04/05 canonical contracts (`evidence.repository.*`, `evidence.cicd.*`, `evidence.deployment.*`, `evidence.identity.privileged-membership` / `external-access`) plus Prompt 03 `inventory.subject` / `inventory.complete`. Honest descriptor (only implemented types; true pagination/incremental; failure behavior documented in GitHub-owned sources without redesigning `CollectorDescriptor`). 401/403 yield `PermissionDenied` / insufficient-evidence diagnostics, never fabricated negatives, and do not abort other subjects. Authoritative `inventory.complete` only after complete pagination. Real `CollectionRun` (version, scope, secret-free digest, start/completion, counts, complete/partial/failed). Ten golden adapter fixtures. No tokens in facts/diagnostics/fixtures. No ISO/SOC2/NIS2/DORA logic and no Effective/Ineffective.
- **ADR:** needed — accepted at [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_github_collector_baseline` + `sdd_github_collector_target` remain registered.
2. Baseline GREEN on `e430980` characterizing §3; target RED after §4.11 authoring, then GREEN after implement.
3. Descriptor advertises only collected canonical types, honest pagination/incremental, real permissions, source-control family, and GitHub-owned failure behavior.
4. Emitted types are Prompt 04/05/03 contracts only; no `evidence.github.*` required by tests; no framework IDs in collector logic.
5. 401/403 produce explicit `PermissionDenied` / insufficient-evidence diagnostics; never negative booleans; no whole-batch abort.
6. Authoritative `inventory.complete` only after complete pagination with no list permission hole.
7. `collect_batch` records version, scope, secret-free configuration digest, start/completion, counts, and complete/partial/failed.
8. Ten golden fixtures exist and pass; `healthy-org` enables ≥25 canonical controls via type/fact coverage.
9. No `ghp_` / `gho_` / `github_pat_` / `ghs_` / `Bearer` material in envelopes, diagnostics, fixtures, or digest; ISO GH-007/GH-009 stay green.
10. `GITHUB_EVIDENCE_TYPES` stays free of `evidence.identity.*`; historical `source.*` strings remain as the GH-012 mapping table.
11. Another provider emitting the same contracts would receive the same test results; collector never computes Effective/Ineffective.

### Out of scope

- ISO/SOC2/NIS2/DORA remapping and ISO pack rewrite
- Calculating Effective/Ineffective, readiness, or SoA
- Redesigning catalog IDs or landing Prompt 05–08 catalog TOML
- SaaS credential store, OAuth app, or secret manager
- Entra/Okta/GitLab/Bitbucket collectors
- Mandatory live HTTP in unit tests
- Changing `EvidenceValue` or population-evaluator semantics / `resolve_repository_inventory`
- Rewriting IAM, population, typed-evidence, or ISO SDD suites
- Adding `failure_behavior` to shared `CollectorDescriptor` unless strictly required
- Scanner engines and the one-way bridge
- Concurrent Prompt 06/07/08 file trees

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Dual-emitting `source.*` plus canonical types would couple tests to GitHub | New observations are Prompt 03/04/05 contracts only (`ghc_005`). Historical `source.*` remain mapping-table needles, not emitted facts. |
| Putting `evidence.identity.*` on `GITHUB_EVIDENCE_TYPES` breaks IAM-015 | `GITHUB_EVIDENCE_TYPES` stays the ADR 0002 `source.*` mapping table; descriptor advertises `GITHUB_CANONICAL_EVIDENCE_TYPES`. |
| Prompt 05 AC10 snapshots pinning an untouched collector | Collector emit path is the change; catalog TOML / snapshot pins were not rewritten. |
| Advertised-vs-collected gap returning after implement | Descriptor lists only implemented canonical types; target `ghc_001` locks the set. |
| 403 abort dropping the rest of an org inventory | Per-subject `PermissionDenied` diagnostics; batch continues (`ghc_012b`). |
| Partial pagination claimed as `inventory.complete` authoritative | Authoritative complete only after full page walk with no list-permission hole. |
| Hardcoded `main` missing real default-branch protection | Protection uses the repo default branch, not a hardcoded `main`. |
| Token leak via 403 bodies, `ghs_`, or fixtures | `ghs_` folded by GitHub-owned `sanitize_diagnostic`; `ghc_021` scans envelopes/diagnostics/fixtures/digest. |
| Inventing `evidence.github.*` required by tests | Target suite forbids provider-shaped required types. |
| Changing `EvidenceProvenance` and breaking typed-evidence digests | Provenance / value model untouched. |
| Redesigning shared `CollectorDescriptor` for one GitHub field | Failure behavior stays GitHub-owned documentation; shared descriptor not extended. |
| Retry logic duplicating envelopes after 429→200 | Rate-limit golden asserts a single emit after retry. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/github-collector.md`](github-collector.md) |
| Baseline | PASS on old | `cargo test --test sdd_github_collector_baseline --test sdd_github_collector_target -- --nocapture` → exit 0. Characterization SHA `e430980`. Baseline **30 passed; 0 failed** (`ghc_b001`–`ghc_b030`: `source.*` strings, hardcoded `main`, stub modules, advertised-vs-collected gap, abort-on-403, empty `CollectionRun`, fixture-only client, `ghs_` not redacted). Target binary is assertion-empty by protocol (**0 passed**). No product feature code. Excerpt: `test result: ok. 30 passed; 0 failed; 0 ignored` / `running 0 tests` / `test result: ok. 0 passed`. Suites: `tests/sdd/github_collector.baseline.rs`, `tests/sdd/github_collector.target.rs`. |
| Target pre | FAIL on old | Same dual command → exit 1. Target authored §4.11 assertions. **FAILED. 1 passed; 29 failed**. `ghc_000` keep-alive passed (suites already registered). Remaining assertions failed on current `source.*` emit, abort-on-403, empty `CollectionRun`, missing goldens, and advertised-vs-collected descriptor. Baseline unmodified and GREEN (**30 passed**). No `crates/weeping-angel-collector` product edits. Excerpt: `ghc_001 descriptor must advertise implemented canonical type evidence.repository.inventory`; `ghc_005 new observations must be canonical contracts, got source.repository.exists`; `ghc_010 golden healthy-org must exist`; `ghc_012b protection 403 is a per-subject diagnostic, not a batch Err: permission denied: 403 reading branch protection`. Suite: `tests/sdd/github_collector.target.rs`. |
| Implement | target PASS | Same dual command after shipping the reference-grade collector. Target **30 passed; 0 failed** (`ghc_000`–`ghc_024` + `012b`/`013b`/`014b`/`018b`/`019b`). Baseline default: **0 passed; 0 failed; 30 ignored** (`superseded by sdd_github_collector_target`). Not an additive baseline hold; ignore-supersede is the documented retire path. Files: collector modules under `crates/weeping-angel-collector/src/github/`, ten goldens under `fixtures/assurance/canonical/v1/github/`, `tests/sdd/github_collector.baseline.rs`, spec/ADR. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default dual run: baseline **ok. 0 passed; 0 failed; 30 ignored**. Forced `cargo test --test sdd_github_collector_baseline -- --ignored --nocapture` → **FAILED. 7 passed; 23 failed**. Failures include `ghc_b001` (org inventory now emitted), `ghc_b010` (got `evidence.repository.default-branch`), `ghc_b014`/`b015` (403 no longer aborts), `ghc_b022` (`CollectionRun.completed_at` filled), `ghc_b028` (github adapter goldens exist). Not additive. Characterization of the ISO-sliver is no longer CI-required. |
| Supersede | target still PASS | After skip-supersede: target **ok. 30 passed; 0 failed; 0 ignored**. Dual-suite registration kept (`ghc_000`). `target_still_green=true`. Baseline file stays registered because `ghc_000` requires both `Cargo.toml` test registrations. |
| Docs/ADR | updated | [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md), [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`docs/sdd/github-collector.md`](github-collector.md), [`docs/sdd/sdd-github-collector.md`](sdd-github-collector.md) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `skip` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

Reference-grade GitHub collector in `weeping-angel-collector` (Prompt 09):

- Descriptor advertises `GITHUB_CANONICAL_EVIDENCE_TYPES` only (implemented Prompt 03/04/05 contracts). `GITHUB_EVIDENCE_TYPES` remains the ADR 0002 `source.*` mapping-table list and stays free of `evidence.identity.*`.
- `org:` inventory via `/orgs/{org}/repos` with `Link` / `per_page` walker; `exclude_archived` selector; authoritative `inventory.complete` only after complete pagination with no list-permission hole.
- Protection uses the repo default branch (not hardcoded `main`).
- 401/403 → `PermissionDenied` / insufficient-evidence diagnostics, never `protected=false` / fabricated negatives; batch continues for other subjects.
- `collect_batch` fills a real `CollectionRun` (version, scope, secret-free configuration digest, start/completion, counts, complete/partial/failed).
- Ten goldens under `fixtures/assurance/canonical/v1/github/` (`healthy-org`, `unprotected-repo`, `missing-branch-protection-permission`, `paginated-inventory`, `paginated-inventory-truncated`, `archived-excluded-by-selector`, `disabled-security-scanning`, `protected-environment-absent`, `privileged-membership-population`, `api-partial-failure`, `rate-limit-retry`).
- `healthy-org` enables ≥25 canonical controls via type/fact coverage.
- `ghs_` folded by GitHub-owned `sanitize_diagnostic`; no `ghp_` / `gho_` / `github_pat_` / `ghs_` / `Bearer` material in envelopes, diagnostics, fixtures, or digest.
- No ISO/SOC2/NIS2/DORA IDs and no Effective/Ineffective in collector logic. Another provider emitting the same contracts would receive the same test results.

### Files changed (implement)

`crates/weeping-angel-collector/src/github/mod.rs`, `crates/weeping-angel-collector/src/github/client.rs`, `crates/weeping-angel-collector/src/github/descriptor.rs`, `crates/weeping-angel-collector/src/github/error.rs`, `crates/weeping-angel-collector/src/github/normalize.rs`, `crates/weeping-angel-collector/src/github/protection.rs`, `crates/weeping-angel-collector/src/github/branches.rs`, `crates/weeping-angel-collector/src/github/repositories.rs`, `crates/weeping-angel-collector/src/github/rulesets.rs`, `crates/weeping-angel-collector/src/github/security.rs`, `crates/weeping-angel-collector/src/github/workflows.rs`, `crates/weeping-angel-collector/src/github/collaborators.rs`, `fixtures/assurance/canonical/v1/github/healthy-org/http.json`, `fixtures/assurance/canonical/v1/github/unprotected-repo/http.json`, `fixtures/assurance/canonical/v1/github/missing-branch-protection-permission/http.json`, `fixtures/assurance/canonical/v1/github/paginated-inventory/http.json`, `fixtures/assurance/canonical/v1/github/paginated-inventory-truncated/http.json`, `fixtures/assurance/canonical/v1/github/archived-excluded-by-selector/http.json`, `fixtures/assurance/canonical/v1/github/disabled-security-scanning/http.json`, `fixtures/assurance/canonical/v1/github/protected-environment-absent/http.json`, `fixtures/assurance/canonical/v1/github/privileged-membership-population/http.json`, `fixtures/assurance/canonical/v1/github/api-partial-failure/http.json`, `fixtures/assurance/canonical/v1/github/rate-limit-retry/http.json`, `tests/sdd/github_collector.baseline.rs`, `docs/sdd/github-collector.md`, `docs/sdd/sdd-github-collector.md`, `docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-e4993ccb7da6` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 12 178 860 |
| `duration_ms_sum` | 5 586 117 (~93.1 min) |
| `budget.total` | 48 |
| `budget.spent` | 8 |
| `budget.remaining` | 40 |
| `event_count` | 29 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |

### Gates (final snapshot)

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agents

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 135 071 | 318 725 |
| Spec | `sdd-spec` | ok | 455 225 | 911 988 |
| BaselineGreen | `sdd-baseline-green` | ok | 76 392 | 228 125 |
| TargetRed | `sdd-target-red` | ok | 878 232 | 1 235 104 |
| Implement | `sdd-implement` | ok | 2 634 593 | 7 529 873 |
| DocsAdr | `sdd-docs-adr` | ok | 1 040 196 | 1 610 003 |
| Iterate | `sdd-baseline-post-check` | ok | 268 819 | 139 837 |
| Supersede | `sdd-supersede` | ok | 97 589 | 205 205 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-github-collector-telemetry.json`](sdd-github-collector-telemetry.json).

---

## Remaining backlog (not this slice)

1. ISO/SOC2/NIS2/DORA remapping and ISO pack rewrite
2. Calculating Effective/Ineffective, readiness, or SoA
3. Redesigning catalog IDs or landing Prompt 05–08 catalog TOML
4. SaaS credential store, OAuth app, or secret manager
5. Entra/Okta/GitLab/Bitbucket collectors
6. Mandatory live HTTP in unit tests
7. Changing `EvidenceValue` or population-evaluator semantics / `resolve_repository_inventory`
8. Rewriting IAM, population, typed-evidence, or ISO SDD suites
9. Adding `failure_behavior` to shared `CollectorDescriptor`
10. Scanner engines and the one-way bridge
11. Concurrent Prompt 06/07/08 file trees

---

## Summary

Prompt 09 reference-grade GitHub collector landed under dual-suite SDD: spec + accepted ADR 0003 (canonical-evidence mapping; draft filename dropped), baseline GREEN on SHA `e430980` (30 passed characterizing the ISO-sliver `source.*` emit), target RED (1 passed / 29 failed on missing canonical types, abort-on-403, empty `CollectionRun`, missing goldens), then implement until target GREEN 30/30. Baseline characterization is skip-retired (30 ignored; forced `--ignored` 23 FAIL). Descriptor advertises only collected canonical types; 401/403 are per-subject diagnostics; `inventory.complete` is authoritative only after full pagination; `CollectionRun` is real and secret-free; ten goldens pass; no tokens and no Effective/Ineffective.
