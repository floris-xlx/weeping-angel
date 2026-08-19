# SDD run: Prompt 07 infrastructure canonical assurance catalog

| Field | Value |
| --- | --- |
| Run id | `sdd-d912c7981f2a` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `d912c7981f2a6c93` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 07: 43 provider-neutral `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` controls, 16 fact evidence contracts, population tests via Prompt 03, per-family TOML + fixtures. Dual-suite skip-retired. |
| Spec | [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md) |
| ADR | [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md) |
| Draft (retired) | [`docs/adr/0003-infrastructure-canonical-assurance-catalog-draft.md`](../adr/0003-infrastructure-canonical-assurance-catalog-draft.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-infrastructure-canonical-assurance-catalog-telemetry.json`](sdd-infrastructure-canonical-assurance-catalog-telemetry.json) |
| Dual-suite | `tests/sdd/infrastructure_catalog.baseline.rs` (skip-retired / `#[ignore]`) · `tests/sdd/infrastructure_catalog.target.rs` (active; infra_000…infra_020) |
| Characterization SHA | `e430980c` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Prompt 07: provider-neutral infrastructure canonical assurance catalog
- **Problem:** Organizations cannot assess network exposure, crypto/secret storage, data/database protection, logging, backup, or resilience as provider-neutral population tests. Today only an ISO-pack existence/hybrid sliver exists; future cloud/database/network collectors have no canonical emit contract.
- **Current behavior (planning SHA `e430980c`):** `catalog/canonical/v1/manifest.toml` lists only `fixture.example.toml` and `identity.toml`. Grep of product TOML/Rust finds zero `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` and zero required `evidence.network` / `data` / `crypto` / `secret.storage` / `database` / `logging` / `backup` / `resilience` contracts. Fixtures exist only under `fixtures/assurance/canonical/v1/identity/`. Root `Cargo.toml` does not auto-discover `tests/sdd/*.rs` and has no `sdd_infrastructure_catalog_*` rows. ISO pack still owns `logging.security-events`, `logging.audit-trail`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit` (existence/hybrid) and `security.tls` (`break_on` `security.tls.misconfiguration`). GitHubCollector emits `source.*` only. Prompt 03 `evaluate_coverage` is real; `resolve_population` special-cases identity inventory only; no `resolve_database_inventory`. `AllSubjects` classifies truthy/falsey fields, not integer `retention_days`. Public contract documents IAM, not infrastructure. IAM sibling is landed.
- **Desired behavior:** After implement: 43 independently assessable provider-neutral controls in `catalog/canonical/v1/{controls,evidence,tests}/{network,crypto,data,database,logging,backup,resilience}.toml` listed in manifest `[files]`; `control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`. Sixteen required evidence types are facts not conclusions. Eight required population tests (plus one test per remaining control) use `AllSubjects`/`NoneSubjects`/`CoverageAtLeast`/`ExceptionApproved`/`InsufficientEvidence` via Prompt 03. Thresholds come from `[test.expression]` or `AssessmentContext.max_age`, bound as `meets_threshold`/`meets_policy`/`approved_storage` bools. Fixtures under `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/` cover healthy/partial/stale/missing/failing/exception cases. Dual suites registered; target GREEN; baseline superseded; ADR accepted; `assurance-runtime.md` names the family. ISO pack and Prompts 05/06 untouched.
- **ADR:** needed — accepted at [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md)

### Acceptance criteria (this slice)

1. Register `sdd_infrastructure_catalog_baseline` and `sdd_infrastructure_catalog_target` in root `Cargo.toml` like IAM.
2. On current code, baseline GREEN (no infra family; ISO sliver present); target RED for missing `control.network.*` / `evidence.database.*` / population fixtures, not compile noise.
3. After implement: target GREEN; baseline `#[ignore = "superseded by sdd_infrastructure_catalog_target"]`; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. Exactly 43 controls in the 35–50 band with stable `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` ids, domains, evidence, tests, and honest automation class.
5. Declare the 16 required evidence contracts as facts; no `evidence.aws.*` / `evidence.cloudflare.*` / `evidence.secret.exposure`.
6. Eight Prompt-07 tests are population predicates, not `Exists(one envelope)`; unencrypted-critical-db fails and a lone encryption envelope does not pass.
7. Fixtures distinguish `InsufficientEvidence`, `StaleEvidence`, `Ineffective`, `ManualReviewRequired`, and `ExceptionApproved`.
8. DR exercise, recovery objectives, and network-segmentation rationale are hybrid/manual and cannot auto-pass from one technical flag.
9. `CanonicalCatalog::validate` accepts the slice: no orphans/duplicates, no provider/framework tokens in ids; `pci`/`pci-dss` rejected in file text even if not in `FRAMEWORK_SEGMENTS`.
10. ISO pack ids/mappings unchanged; `sdd_iso27001_assurance_target` and `sdd_iam_catalog_target` stay green.
11. No AWS/Azure/GCP/Cloudflare or remote-inventory collector; GitHub still emits `source.*` only.
12. No second loader, `EvidenceValue` fork, or `resolve_database_inventory`/`resolve_network_inventory`.
13. Retention/TLS/restore/approved-storage thresholds live in catalog/test config or `AssessmentContext`, not Rust ISO/PCI constants.
14. Fixtures/seal reject secret material and compliance narratives.
15. Do not overwrite Prompt 01/04/06 SSOTs, `identity.toml`, `fixture.example.toml` (including `control.source.protected-branch`), or Prompt 05/06 product paths.
16. `control.secret.*` lives in `crypto.toml`; no `secret.toml` or `vulnerability.toml`.
17. Target/baseline tests never self-read and assert absence of a substring that appears in the assertion (I4a).
18. Population tests bind Prompt-03-classifiable fields (`encrypted`, `meets_policy`, `meets_threshold`, `restricted`, `approved_storage`, `*_at`), not raw `retention_days` integers.

### Out of scope

- AWS/Azure/GCP/Cloudflare/on-prem collectors
- ISO/SOC2/NIS2/PCI remapping (Prompt 12) or growing the ISO pack
- Redesign of CanonicalCatalog loader/validator/digest
- Redesign of EvidenceValue or seal rules
- `resolve_database_inventory` / `resolve_network_inventory` or generic population semantic changes
- Prompt 05 SDLC (`control.source.*` / cicd / release)
- Prompt 06 `vulnerability.toml`, `evidence.secret.exposure`, vulnerability fixtures/tests
- Deleting `fixture.example` ids including `control.source.protected-branch`
- Remote inventory service or live cloud API evaluation
- Certification/compliant language
- Prompt 08 governance catalog
- Editing `identity.toml` or Prompt 01/04/06 SSOT files

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Prompt 06 file collision on `evidence/secret.toml` | Storage stays in `crypto.toml`; never created `evidence.secret.exposure`. |
| Prompt 05 landing `control.source.*` | SDLC paths and `fixture.example` (including `control.source.protected-branch`) untouched. |
| Existence checks masquerading as population tests | Eight required tests are AllSubjects/NoneSubjects/CoverageAtLeast predicates; unencrypted-critical-db fails; a lone encryption envelope does not pass. |
| Thresholds hardcoded as ISO/PCI constants in Rust | Thresholds live in `[test.expression]` or `AssessmentContext.max_age`, bound as `meets_threshold`/`meets_policy`/`approved_storage`. |
| ISO sliver rewritten or `sdd_iso27001_assurance_target` broken | ISO pack ids/mappings unchanged; ISO target 49 passed; IAM target 17 passed / 2 ignored. |
| Provider names leaking into canonical IDs or fixture types | No `evidence.aws.*` / `evidence.cloudflare.*`; no AWS/Azure/GCP/Cloudflare collectors; `pci`/`pci-dss` rejected in file text. |
| Hybrid DR/segmentation/objectives auto-passing from one flag | DR exercise, recovery objectives, and network-segmentation rationale stay hybrid/manual. |
| Adding `resolve_database_inventory` instead of generic `inventory.subject` | No `resolve_database_inventory` / `resolve_network_inventory`; no second loader or `EvidenceValue` fork. |
| Baseline remaining CI-green for catalog absence | Skip-retired with `#[ignore = "superseded by sdd_infrastructure_catalog_target"]`; forced `--ignored` FAILED 11 passed / 7 failed. |
| Secrets in fixtures | Fixtures/seal reject secret material and compliance narratives. |
| `AllSubjects` on `retention_days` classifying as Technical | Population tests bind Prompt-03-classifiable bools/`*_at`, not raw `retention_days` integers. |
| I4a self-referential target assertions | Target/baseline tests never self-read and assert absence of a substring that appears in the assertion. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md) |
| Baseline | PASS on old | `cargo test --workspace --features demo --test sdd_infrastructure_catalog_baseline --test sdd_infrastructure_catalog_target -- --nocapture` → **baseline ok. 18 passed; 0 failed**; target placeholder **0 tests**. Characterization SHA `e430980c`: no `control.network.*` / `evidence.database.*` family, no infrastructure fixtures, ISO logging/crypto/backup/TLS sliver still existence/hybrid, IAM sibling present. Excerpt: `required_infrastructure_evidence_and_population_tests_are_undeclared ... ok`. Suites: `tests/sdd/infrastructure_catalog.baseline.rs`, `tests/sdd/infrastructure_catalog.target.rs`. |
| Target pre | FAIL on old | Same dual-suite command → target **FAILED. 4 passed; 18 failed**. Compiled (no compile-noise RED). Failures: missing `control.network.*` family TOML, `evidence.database.*` / population tests, infrastructure fixtures. Excerpt: `INFRA-001: missing catalog family file .../catalog/canonical/v1/controls/network.toml`; `INFRA-010: fixture \`network/healthy\` is not shipped`; `INFRA-017: controls/crypto.toml must exist and host control.secret.*`; `INFRA-014: retention test must exist`. Four passes were already-true boundaries (dual-suite registration, ISO pack not retargeted, IAM/`fixture.example` remain, no cloud collectors). Combined `--workspace` also blocked by parallel GitHub-collector WIP and sibling SDLC/ISO-remap drift (3 baseline characterizations); baseline file not modified. Suite: `tests/sdd/infrastructure_catalog.target.rs`. |
| Implement | target PASS | Same dual-suite command → baseline **ok. 0 passed; 0 failed; 18 ignored** (`ignored, superseded by sdd_infrastructure_catalog_target`); target **ok. 22 passed; 0 failed** (`infra_000`…`infra_020`). Prompt 07 family landed: 43 controls, 16 fact contracts, population fixtures. INFRA-018 restored by naming `fixtures/assurance/canonical/v1/network` and `.../database` explicitly in `docs/contracts/assurance-runtime.md`. Additive catalog slice; baseline hold confirmed by re-run. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default dual run: baseline **ok. 0 passed; 0 failed; 18 ignored**. Forced `--ignored`: **FAILED. 11 passed; 7 failed**. Failures: `catalog_loader_validate_and_digest_remain_the_single_ssot`; `fixture_example_and_identity_are_the_only_catalog_families` (found `control.network.*`); `required_infrastructure_evidence_and_population_tests_are_undeclared` (`evidence.network.exposure` declared); `identity_fixtures_exist_and_infrastructure_fixtures_do_not` (network fixtures shipped); `iso_hybrid_kind_in_toml_loads_as_automated`; `iso_pack_holds_the_logging_crypto_backup_tls_sliver`; `public_contract_documents_iam_not_infrastructure`. Not additive. Dual-suite registration kept (`infra_000`). |
| Supersede | target still PASS | After skip-supersede: target **ok. 22 passed; 0 failed; 0 ignored**. Target remains the CI gate. Sibling hold: `sdd_iam_catalog_target` 17 passed / 2 ignored; `sdd_iso27001_assurance_target` 49 passed; `cargo fmt --all -- --check` exit 0. Full `cargo test --workspace --features demo` failed on sibling `sdd_assessment_lineage_target` (`lin_002`/`lin_004`), not this slice. `clippy -D warnings` still flags pre-existing scanner `collapsible_if` lints in `src/`. |
| Docs/ADR | updated | [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md) |

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

Provider-neutral infrastructure catalog family on existing catalog/population infrastructure:

- 43 independently assessable `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` controls (35–50 band) with stable ids, domains, evidence, tests, and honest automation class.
- `control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml` (no `secret.toml`, no `vulnerability.toml`, no `evidence.secret.exposure`).
- Sixteen required evidence types declared as facts, not conclusions; no `evidence.aws.*` / `evidence.cloudflare.*`.
- Eight required population tests (plus one test per remaining control) use `AllSubjects`/`NoneSubjects`/`CoverageAtLeast`/`ExceptionApproved`/`InsufficientEvidence` via Prompt 03. Unencrypted-critical-db fails; a lone encryption envelope does not pass.
- Thresholds from `[test.expression]` or `AssessmentContext.max_age`, bound as `meets_threshold`/`meets_policy`/`approved_storage` bools — not raw `retention_days` and not Rust ISO/PCI constants.
- Fixtures under `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/` distinguish `InsufficientEvidence`, `StaleEvidence`, `Ineffective`, `ManualReviewRequired`, and `ExceptionApproved`.
- DR exercise, recovery objectives, and network-segmentation rationale are hybrid/manual and cannot auto-pass from one technical flag.
- `CanonicalCatalog::validate` accepts the slice: no orphans/duplicates; no provider/framework tokens in ids.
- Consumes Prompt 01 loader and Prompt 03 coverage runtime. No second loader, `EvidenceValue` fork, or `resolve_database_inventory`/`resolve_network_inventory`. GitHub still emits `source.*` only. ISO pack and Prompts 05/06 paths untouched.

### Files changed (implement)

`catalog/canonical/v1/manifest.toml`, `catalog/canonical/v1/controls/network.toml`, `catalog/canonical/v1/controls/crypto.toml`, `catalog/canonical/v1/controls/data.toml`, `catalog/canonical/v1/controls/database.toml`, `catalog/canonical/v1/controls/logging.toml`, `catalog/canonical/v1/controls/backup.toml`, `catalog/canonical/v1/controls/resilience.toml`, `catalog/canonical/v1/evidence/network.toml`, `catalog/canonical/v1/evidence/crypto.toml`, `catalog/canonical/v1/evidence/data.toml`, `catalog/canonical/v1/evidence/database.toml`, `catalog/canonical/v1/evidence/logging.toml`, `catalog/canonical/v1/evidence/backup.toml`, `catalog/canonical/v1/evidence/resilience.toml`, `catalog/canonical/v1/tests/network.toml`, `catalog/canonical/v1/tests/crypto.toml`, `catalog/canonical/v1/tests/data.toml`, `catalog/canonical/v1/tests/database.toml`, `catalog/canonical/v1/tests/logging.toml`, `catalog/canonical/v1/tests/backup.toml`, `catalog/canonical/v1/tests/resilience.toml`, `fixtures/assurance/canonical/v1/network`, `fixtures/assurance/canonical/v1/crypto`, `fixtures/assurance/canonical/v1/data`, `fixtures/assurance/canonical/v1/database`, `fixtures/assurance/canonical/v1/logging`, `fixtures/assurance/canonical/v1/backup`, `fixtures/assurance/canonical/v1/resilience`, `tests/sdd/infrastructure_catalog.baseline.rs`, `tests/sdd/infrastructure_catalog.target.rs`, `docs/sdd/infrastructure-canonical-assurance-catalog.md`, `docs/adr/0003-infrastructure-canonical-assurance-catalog.md`, `docs/contracts/assurance-runtime.md`, `crates/weeping-angel-evidence/src/lib.rs`, `crates/weeping-angel-assurance/Cargo.toml`, `crates/weeping-angel-assurance/src/bridge.rs`, `Cargo.toml`, `src/engines/mod.rs`, `src/contract/types.rs`.

Docs/ADR also finalized [`README.md`](../../README.md) (draft ADR retired; accepted ADR 0003-infrastructure).

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-d912c7981f2a` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 21 964 971 |
| `duration_ms_sum` | 6 750 527 (~112.5 min) |
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
| Scope | `sdd-scope` | ok | 287 608 | 281 233 |
| Spec | `sdd-spec` | ok | 430 838 | 1 478 175 |
| BaselineGreen | `sdd-baseline-green` | ok | 520 491 | 2 028 084 |
| TargetRed | `sdd-target-red` | ok | 1 403 797 | 5 001 461 |
| Implement | `sdd-implement` | ok | 3 131 298 | 11 215 408 |
| DocsAdr | `sdd-docs-adr` | ok | 682 093 | 1 482 999 |
| Iterate | `sdd-baseline-post-check` | ok | 160 139 | 230 421 |
| Supersede | `sdd-supersede` | ok | 134 263 | 247 190 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-infrastructure-canonical-assurance-catalog-telemetry.json`](sdd-infrastructure-canonical-assurance-catalog-telemetry.json).

---

## Remaining backlog (not this slice)

1. AWS/Azure/GCP/Cloudflare/on-prem collectors and remote inventory / live cloud API evaluation
2. ISO/SOC2/NIS2/PCI remapping onto infrastructure ids (Prompt 12) or growing the ISO pack
3. Prompt 05 SDLC (`control.source.*` / cicd / release) — do not delete `fixture.example` / `control.source.protected-branch`
4. Prompt 06 `vulnerability.toml`, `evidence.secret.exposure`, vulnerability fixtures/tests
5. Prompt 08 governance catalog
6. `resolve_database_inventory` / `resolve_network_inventory` or generic population semantic changes
7. Redesign of CanonicalCatalog loader/validator/digest or EvidenceValue/seal rules
8. Certification/compliant language (forbidden)
9. Sibling hold: `sdd_assessment_lineage_target` `lin_002`/`lin_004` still fail full `--workspace`; pre-existing `clippy -D warnings` `collapsible_if` in `src/` scanner (not this slice)

---

## Summary

Prompt 07 infrastructure catalog landed under dual-suite SDD: completed SSOT + accepted ADR 0003, baseline GREEN on SHA `e430980c` (18 passed characterizing no infra family; ISO sliver + IAM sibling present), target RED (18 failed) for missing `control.network.*` / `evidence.database.*` / population fixtures (not compile noise), then target GREEN 22/22. Absence-of-catalog baseline skip-retired (`#[ignore]`; forced `--ignored` 7 fail). 43 controls, 16 fact contracts, population tests via Prompt 03; `control.secret.*` in `crypto.toml`; no cloud collectors or inventory resolvers. ISO and IAM targets remain green. Hybrid/manual honesty for DR, recovery objectives, and network-segmentation rationale.
