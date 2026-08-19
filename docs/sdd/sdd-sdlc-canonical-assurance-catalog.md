# SDD run: Prompt 05 SDLC canonical assurance catalog

| Field | Value |
| --- | --- |
| Run id | `sdd-e5b53f79bcd8` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `e5b53f79bcd836fa` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 05: 26 provider-neutral `control.source\|cicd\|release\|supply-chain.*` controls, 20 fact evidence types, 26 population tests, seven fixtures. Consumes Prompts 01–03; IAM sibling unchanged; no collector expansion; no Prompt 06/07/08 content. |
| Spec | [`docs/sdd/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md) |
| ADR | [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) (draft dropped after target GREEN) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md`](../prompts/canonical-assurance-v1/05-sdlc-catalog.md) |
| Telemetry | [`sdd-sdlc-canonical-assurance-catalog-telemetry.json`](sdd-sdlc-canonical-assurance-catalog-telemetry.json) |
| Dual-suite | `tests/sdd/sdlc_catalog.baseline.rs` (skip-retired / `#[ignore]`) · `tests/sdd/sdlc_catalog.target.rs` (active; SDLC-001…016) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Prompt 05 SDLC canonical assurance catalog (spec only at write; product catalog landed in Implement)
- **Problem:** Organizations cannot assess repository inventory, branch protection, review, CI/CD permissions, release authorization, or supply-chain integrity with provider-neutral population controls. A GitLab/Bitbucket collector has nowhere canonical to emit the same facts as GitHub.
- **Current behavior (SHA `e430980c`):** Canonical tree is `fixture.example.toml` + `identity.toml` only. The sole source-shaped canonical control is exists-only `control.source.protected-branch` (`op=exists` on `evidence.source.protected-branch`). ISO pack still ships `source.branch-protection`, `source.required-review`, `source.code-ownership`, `source.security-scanning`, `source.commit-signing` as presence checks on GitHub-shaped `source.*` evidence. IAM sibling is landed. GitHub collector still advertises `GITHUB_EVIDENCE_TYPES` `source.*` only. Prompt 03 has no `resolve_repository_inventory`. No `sdlc.toml`, no `fixtures/assurance/canonical/v1/sdlc`, no `sdd_sdlc_catalog_*` `[[test]]` rows until dual-suite registration.
- **Desired behavior:** Land 26 independently assessable provider-neutral controls (`control.source|cicd|release|supply-chain.*`), 20 fact evidence types (`evidence.repository|cicd|deployment|release|supply-chain.*`), 26 population tests, and 7 fixtures under `fixtures/assurance/canonical/v1/sdlc/{healthy-org,degraded-org,partial-coverage,unprotected-default-branch,missing-scan-evidence,stale-dependency-scan,approved-exception}`. Prefer `catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml`. Consume `CanonicalCatalog::{load,validate,digest}`, `EvidenceValue`, and Prompt 03 `AllSubjects`/`CoverageAtLeast`/`ExceptionApproved`/`InsufficientEvidence`. Missing evidence is `InsufficientEvidence`; partial/unknown populations cannot be `Effective`; hybrid release/authority/security-review/policy controls do not auto-pass from one flag. After target GREEN, ignore the baseline like IAM and accept the existing ADR draft.
- **ADR:** needed — accepted at [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) (draft path during spec: `docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md`).

### Acceptance criteria (this slice)

1. Register `[[test]]` `sdd_sdlc_catalog_baseline` and `sdd_sdlc_catalog_target` at `tests/sdd/sdlc_catalog.{baseline,target}.rs` (Cargo does not auto-discover `tests/sdd/*.rs`).
2. Baseline GREEN on current tree: no SDLC population family; ISO sliver + fixture `control.source.protected-branch` exist; IAM present; collector still `source.*`; no `resolve_repository_inventory`; do not assert absence of every `control.source.*`.
3. Target RED on current tree for missing family/population tests/fixtures, not compile noise; never self-read suite text to assert `!contains(#[ignore)` or other literals that appear in the assertion; assert loaded catalog IDs.
4. After implement: 26 controls, 20 evidence types, 26 tests, 7 fixtures; validator accepts; no provider/framework tokens in SDLC IDs; population default-branch id is `control.source.default-branch-protection`.
5. Required population examples evaluate all-subjects (protected default branches, force-push, reviews, scan enablement, workflow permissions, prod environments, dependency freshness, artifact integrity); missing scan evidence is `InsufficientEvidence`.
6. Release authorization, authority separation, security review, and secure-development policy stay Hybrid/Manual and cannot auto-pass from a single technical flag.
7. ISO pack `source.*` ids/mappings, IAM family, CAT fixture IDs, Prompt 03 semantics, and GitHub collector are unchanged; no Prompt 06/07/08 content.
8. Prefer `{controls,evidence,tests}/sdlc.toml` so `ghc_b028` stays green (no `evidence/repository.toml`).
9. Target GREEN then baseline FAIL-or-additive-documented then `#[ignore = "superseded by sdd_sdlc_catalog_target"]`; workspace `cargo test --workspace --features demo`, `fmt --check`, `clippy -D warnings` pass.
10. A GitHub, GitLab, or Bitbucket collector could populate the same evidence contracts and receive the same control results; accept ADR draft (drop `-draft`) and pointer-update `assurance-runtime.md` only after TOML lands.

### Out of scope

- Expanding or remapping the GitHub collector
- GitLab/Bitbucket/Azure DevOps/Gitea collectors
- ISO/SOC2/NIS2 remapping onto `control.source.*` (Prompt 12)
- Second `CanonicalCatalog` loader, validator, or digest
- Second `EvidenceValue` enum
- Changing generic population semantics or adding `resolve_repository_inventory`
- Rewriting ISO pack `source.*` rows or mappings
- Removing `control.source.protected-branch` fixture IDs
- Changing IAM catalog content
- Prompt 06/07/08 families
- Scanner engine / depcheck / SARIF changes
- New `SubjectKind` variants
- Certification or audit-passed language
- Inventing a parallel ADR or overwriting Prompt 01 SSOT as domain SSOT

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| `control.source.protected-branch` collides if baseline asserts no `control.source.*` | Baseline characterizes absence of the **population** family and keeps CAT exists-only fixture; population default-branch id is `control.source.default-branch-protection`. |
| Unregistered `tests/sdd/*.rs` so `cargo test --test sdd_sdlc_catalog_*` never runs | Dual-suite registered as `[[test]]` `sdd_sdlc_catalog_baseline` / `sdd_sdlc_catalog_target`. |
| Implementer adds `resolve_repository_inventory` or a second evaluator | Consumed Prompt 03 AllSubjects/CoverageAtLeast/ExceptionApproved/InsufficientEvidence; no inventory resolver, no second evaluator. |
| Existence checks used as population tests | Population tests evaluate all-subjects; CAT `control.source.protected-branch` stays exists-only. |
| Missing scan evidence coded as Ineffective | Missing scan evidence is `InsufficientEvidence` (`missing-scan-evidence` fixture). |
| ISO pack rewritten or remapped here | ISO `source.*` ids/mappings unchanged this slice. |
| Provider/framework tokens leak into IDs or fixtures | Validator accepts; no GitHub/GitLab/Bitbucket/ISO/SOC2/NIS2 tokens in SDLC IDs. |
| Hybrid/manual controls auto-pass from one technical flag | Release authorization, authority separation, security review, and secure-development policy stay Hybrid/Manual. |
| Target suite self-reads `#[ignore` and repeats xylex-sdd-v3 AC-2 / v5 I4a | Target asserts loaded catalog IDs (`control.source.default-branch-protection`), not suite-text literals. |
| `evidence/repository.toml` breaks `ghc_b028` | Family lives at `{controls,evidence,tests}/sdlc.toml`. |
| Baseline stays CI-green asserting catalog absence | Skip-retired with `#[ignore = "superseded by sdd_sdlc_catalog_target"]`; forced `--ignored` is not green. |
| Unknown/partial completeness makes healthy-org Effective or Inconclusive incorrectly | Partial-coverage fixture cannot be Effective on all-subjects; unknown/partial populations cannot be Effective. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md) |
| Baseline | PASS on old | `cargo test --test sdd_sdlc_catalog_baseline -- --nocapture` → exit 0. **ok. 18 passed; 0 failed; 0 ignored**. Characterization-only GREEN on current tree: no SDLC population family, exists-only fixture `control.source.protected-branch`, ISO source sliver, IAM sibling, collector `source.*`, no `resolve_repository_inventory`. Dual-suite registered. Target compiles and is RED for missing `sdlc.toml` / family / fixtures (6 failed, 2 passed at this characterization). No product catalog TOML implemented. Excerpt: `running 18 tests` … `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`. Suites: `tests/sdd/sdlc_catalog.baseline.rs`, `tests/sdd/sdlc_catalog.target.rs`. |
| Target pre | FAIL on old | `cargo test --test sdd_sdlc_catalog_baseline -- --nocapture; cargo test --test sdd_sdlc_catalog_target -- --nocapture` → exit 1. Baseline GREEN (18/18). Target compiled and **FAILED. 0 passed; 16 failed** on missing loaded catalog ID `control.source.default-branch-protection` (not compile noise, not a self-matching `#[ignore` contains). Product TOML/fixtures untouched. Excerpt: `test result: FAILED. 0 passed; 16 failed; 0 ignored; 0 measured; 0 filtered out` / `SDLC family missing: \`control.source.default-branch-protection\` is not loaded (unknown control control.source.default-branch-protection). Current tree still has only \`control.source.protected-branch\``. Suite: `tests/sdd/sdlc_catalog.target.rs`. |
| Implement | target PASS | PRE-IMPLEMENT: baseline 18 passed; target 0 passed / 16 failed (SDLC family missing: `control.source.default-branch-protection`). POST-IMPLEMENT: `cargo test --test sdd_sdlc_catalog_target` → **ok. 16 passed; 0 failed**. Landed `catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml` (26 controls, 20 evidence types, 26 population tests) plus seven fixtures. Population default-branch id is `control.source.default-branch-protection` (CAT fixture `control.source.protected-branch` stays exists-only). Hybrid/manual honesty holds. Missing scan evidence is `InsufficientEvidence`. `cargo fmt --all -- --check` passed. Workspace `cargo test --workspace --features demo` failed on `sdd_canonical_assurance_catalog_baseline::iso_metadata_owns_thin_canonical_stubs` (concurrent Prompt 12 remapped pack). `clippy -D warnings` failed on weeping-angel-collector Prompt 09 edits (`double_comparisons`/`vec_init_then_push`/`collapsible_if`), not `sdlc.toml`. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). POST-IMPLEMENT proof before ignore: `catalog_has_no_sdlc_population_family` FAILED (`current tree has no Prompt 05 catalog file \`controls/sdlc.toml\``); `sdlc_population_fixtures_are_absent` FAILED. After `#[ignore]`: `cargo test --test sdd_sdlc_catalog_baseline` → **ok. 0 passed; 0 failed; 18 ignored** (`superseded by sdd_sdlc_catalog_target`). Forced `--ignored`: **FAILED. 13 passed; 5 failed** (absence characterization inverted). Not additive. Dual-suite registration kept (target still requires both Cargo.toml rows). |
| Supersede | target still PASS | After skip-supersede: `cargo test --test sdd_sdlc_catalog_target -- --nocapture` → **ok. 16 passed; 0 failed; 0 ignored** (`sdlc_001_catalog_tree_lists_and_loads_sdlc_files` … `sdlc_016_no_repository_toml_and_provider_neutral_collectors_share_contracts`). Baseline remains 0 passed / 18 ignored. Target remains the CI gate. |
| Docs/ADR | updated | [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/sdd/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`docs/sdd/github-collector.md`](github-collector.md), [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |

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

Provider-neutral SDLC catalog family on existing catalog/population infrastructure:

- 26 independently assessable `control.source|cicd|release|supply-chain.*` controls with stable ids and honest Automated|Hybrid|Manual class.
- 20 fact evidence types `evidence.repository|cicd|deployment|release|supply-chain.*` (facts, not conclusions).
- 26 population tests; default-branch population id is `control.source.default-branch-protection`. CAT fixture `control.source.protected-branch` stays exists-only.
- Seven fixtures under `fixtures/assurance/canonical/v1/sdlc/`: `healthy-org`, `degraded-org`, `partial-coverage`, `unprotected-default-branch`, `missing-scan-evidence`, `stale-dependency-scan`, `approved-exception`.
- Population examples evaluate all-subjects (protected default branches, force-push, reviews, scan enablement, workflow permissions, prod environments, dependency freshness, artifact integrity). Missing scan evidence is `InsufficientEvidence`. Partial/unknown completeness cannot be `Effective`.
- Release authorization, authority separation, security review, and secure-development policy stay Hybrid/Manual and cannot auto-pass from a single technical flag.
- Layout `{controls,evidence,tests}/sdlc.toml` (no `evidence/repository.toml`) so `ghc_b028` stays green.
- Consumes Prompt 01 `CanonicalCatalog::{load,validate,digest}`, Prompt 02 `EvidenceValue`, and Prompt 03 population runtime. No `resolve_repository_inventory`, no second loader/evaluator, no Prompt 06/07/08 families. ISO pack `source.*`, IAM family, CAT fixture IDs, and GitHub collector `source.*` advertisement unchanged.
- ADR accepted (dropped `-draft`); `assurance-runtime.md` and Prompt 01 SSOT pointer updated only after TOML landed.

### Files changed (implement)

`catalog/canonical/v1/controls/sdlc.toml`, `catalog/canonical/v1/evidence/sdlc.toml`, `catalog/canonical/v1/tests/sdlc.toml`, `catalog/canonical/v1/manifest.toml`, `fixtures/assurance/canonical/v1/sdlc/healthy-org/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/degraded-org/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/partial-coverage/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/unprotected-default-branch/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/missing-scan-evidence/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/stale-dependency-scan/evidence.json`, `fixtures/assurance/canonical/v1/sdlc/approved-exception/evidence.json`, `tests/sdd/sdlc_catalog.baseline.rs`, `tests/sdd/sdlc_catalog.target.rs`, `docs/sdd/sdlc-canonical-assurance-catalog.md`, `docs/adr/0003-sdlc-canonical-assurance-catalog.md`, `docs/contracts/assurance-runtime.md`, `docs/sdd/canonical-assurance-catalog-v1.md`.

### Docs/ADR (finalize)

[`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/sdd/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`docs/sdd/github-collector.md`](github-collector.md), [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md).

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-e5b53f79bcd8` |
| `agents_ok` | 7 |
| `agents_fail` | 0 |
| `agents_total` | 7 |
| `tokens_used_sum` | 15 506 509 |
| `duration_ms_sum` | 5 240 934 (~87.3 min) |
| `budget.total` | 48 |
| `budget.spent` | 7 |
| `budget.remaining` | 41 |
| `event_count` | 28 |
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
| Scope | `sdd-scope` | ok | 165 432 | 379 216 |
| Spec | `sdd-spec` | ok | 697 360 | 790 613 |
| BaselineGreen | `sdd-baseline-green` | ok | 539 547 | 1 444 886 |
| TargetRed | `sdd-target-red` | ok | 416 503 | 1 951 249 |
| Implement | `sdd-implement` | ok | 2 493 010 | 9 456 279 |
| DocsAdr | `sdd-docs-adr` | ok | 715 677 | 1 166 427 |
| Supersede | `sdd-supersede` | ok | 213 405 | 317 839 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-sdlc-canonical-assurance-catalog-telemetry.json`](sdd-sdlc-canonical-assurance-catalog-telemetry.json).

---

## Remaining backlog (not this slice)

1. Expanding or remapping the GitHub collector (Prompt 09; concurrent clippy dirt is collector-owned)
2. GitLab/Bitbucket/Azure DevOps/Gitea collectors
3. ISO/SOC2/NIS2 remapping onto `control.source.*` (Prompt 12; concurrent workspace baseline failure is remap-owned)
4. Prompt 06/07/08 families as this slice’s work (siblings may land separately)
5. Scanner engine / depcheck / SARIF changes
6. New `SubjectKind` variants
7. Certification or audit-passed language (forbidden)
8. Second catalog loader, `EvidenceValue`, or `resolve_repository_inventory` (forbidden)
9. Rewriting ISO pack `source.*` rows, CAT fixture IDs, or IAM catalog content (forbidden this slice)
10. Workspace `cargo test --workspace --features demo` / clippy hygiene owned by concurrent Prompt 09/12 (not SDLC family files)

---

## Summary

Prompt 05 SDLC canonical assurance catalog landed under dual-suite SDD: durable SSOT + accepted ADR 0003 (draft dropped), baseline GREEN (18 passed characterizing no population family + CAT exists-only `control.source.protected-branch` + ISO sliver + IAM + collector `source.*`), target RED (16 failed) on missing loaded ID `control.source.default-branch-protection`, then target GREEN 16/16. Absence-characterization baseline skip-retired (`#[ignore]`; forced `--ignored` 5 fail). Twenty-six controls, 20 evidence types, 26 population tests, seven fixtures in `{controls,evidence,tests}/sdlc.toml`. Hybrid/manual honesty holds; missing scan evidence is InsufficientEvidence. Prompt 01–03 consumed, not forked. Workspace cargo test/clippy remain dirty from concurrent Prompt 09/12, not from this family.
