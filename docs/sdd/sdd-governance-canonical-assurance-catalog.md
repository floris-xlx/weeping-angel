# SDD run: Governance Canonical Assurance Catalog v1 (Prompt 08)

| Field | Value |
| --- | --- |
| Run id | `sdd-25d885be2883` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `25d885be2883ab58` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 08: 34 Hybrid/Manual `control.{governance,risk,personnel,vendor,incident,resilience}.*` controls, first-class `evidence.manual.attestation` plus domain types, population/freshness/manual-review tests, eight fixtures. Consumes Prompts 01–03; no ISO remap; no GRC product. |
| Spec | [`docs/sdd/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) |
| ADR | [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md) |
| Population ADR (pointer) | [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-governance-canonical-assurance-catalog-telemetry.json`](sdd-governance-canonical-assurance-catalog-telemetry.json) |
| Dual-suite | `tests/sdd/governance_catalog.baseline.rs` (skip-retired / `#[ignore]`) · `tests/sdd/governance_catalog.target.rs` (active; gov_000…gov_016) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Fixture clock | `2026-08-18T12:00:00Z` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Governance Canonical Assurance Catalog v1 (Prompt 08)
- **Problem:** Weeping Angel has no provider-neutral governance/risk/personnel/vendor/incident/continuity-governance catalog. Organizational controls exist only as a thin ISO-pack sliver of presence/hybrid checks, while `manual_attestation` is a capability/legacy type rather than first-class immutable evidence, so document existence can be mistaken for operational effectiveness.
- **Current behavior (SHA `e430980c`):** `catalog/canonical/v1` manifest lists only `fixture.example.toml` and `identity.toml`. `CanonicalCatalog::{load,validate,digest}` and `EvidenceValue::with_value` exist; IAM family and `fixtures/assurance/canonical/v1/identity/*` exist. There is no governance/risk/personnel/vendor/incident family TOML, no `fixtures/assurance/canonical/v1/governance/*`, and no `evidence.manual.attestation` catalog type. ISO pack still ships `incident.response-process`, `supplier.security-assessment`, `personnel.access-termination`, and `access.periodic-review` wired to `policy.security.reviewed` / `policy.supplier.assessed` / `personnel.access.terminated` / `policy.access.reviewed`. `TestExpr::ManualReview` always yields `ManualReviewRequired`; `ExceptionApproved` promotion in `evaluate_coverage` is identity break-glass-shaped.
- **Desired behavior:** A Prompt-01 catalog family of ~36 independently assessable Manual/Hybrid controls (`control.{governance,risk,personnel,vendor,incident,resilience}.*`; 30–45 band) with first-class `evidence.manual.attestation` plus domain evidence types, population/freshness/manual-review tests, and eight governance fixtures. Consume existing loader, `EvidenceValue::with_value`, Prompt-03 evaluator, and IR Exception/Risk. Missing evidence is `InsufficientEvidence`; partial training/vendor populations cannot be Effective; approved unexpired exceptions are `ExceptionApproved` never silent Effective; expired exceptions do not suppress fail. No second loader, no ISO remap, no GRC product, no Prompts 05–07 families.
- **ADR:** needed — accepted at [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md) (draft during spec: same path; Accepted after `sdd_governance_catalog_target` GREEN).

### Acceptance criteria (this slice)

1. Register `sdd_governance_catalog_baseline` and `sdd_governance_catalog_target` in root `Cargo.toml` (`tests/sdd` is not autodiscovered).
2. On current product tree, baseline GREEN characterizing absence of governance TOML/fixtures plus presence of IAM and the ISO organizational sliver; target RED for missing `control.governance.*` / `evidence.manual.attestation` / population fixtures, not compile noise.
3. After implement: target GREEN; baseline `#[ignore = "superseded by sdd_governance_catalog_target"]`; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. 36 independently assessable controls in the 30–45 band with honest Automated|Hybrid|Manual class; IDs only `control|evidence|test.{governance,risk,personnel,vendor,incident,resilience,manual}.*` (resilience = continuity/DR governance only).
5. Declare `evidence.manual.attestation` and the Prompt-08 domain types as facts with principal, timestamp, subject, artifact, freshness, and review state; store via `with_value`; no secrets or compliance narratives.
6. Tests include `policy-current`, `training-current-all`, `critical-risk-review-current`, `management-review-current`, `internal-audit-current`, and `exercise-current` as freshness/population/manual-review predicates, not Exists(one PDF).
7. Eight fixtures distinguish current/stale/missing documents, incomplete training, vendor-review gaps, approved vs expired exceptions, and manual-review-despite-evidence.
8. Partial training or vendor populations cannot be Effective on all-subjects tests; missing evidence is `InsufficientEvidence`.
9. Hybrid/manual operational-evidence controls stay Hybrid/Manual and cannot auto-pass from a document-present flag.
10. Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject (never silent Effective); expired exceptions do not suppress failing results.
11. Validator accepts the slice: no orphans/duplicates/dangling refs; target greps vanta/drata/servicenow/jira/iso/soc2/nis2 out of catalog TOML, never via a self-referential source-file substring assert.
12. ISO pack ids/mappings and IAM/fixture.example ids unchanged; no second loader, `EvidenceValue`, population resolver, or Exception/Risk engine; no `resilience.toml`, `vulnerability.toml`, or `evidence.secret.exposure`.

### Out of scope

- Full GRC workflow/SaaS product or document editors
- ISO/SOC2/NIS2 remapping (Prompt 12) or growing the ISO pack sliver
- Second `CanonicalCatalog` loader, `EvidenceValue` fork, or coverage-math reimplementation
- Prompt 04 IAM MFA/privileged-membership/account-status content
- Prompt 05 SDLC/change technical evidence and secure-development policy
- Prompt 06 finding-level risk acceptance, `vulnerability.toml`, `evidence.secret.exposure`
- Prompt 07 operational backup/HA, `evidence.resilience.recovery-plan`, `resilience.toml`
- ServiceNow/Jira/Vanta/Drata or other GRC/ITSM collectors
- Certification or compliant/audit-passed language
- Deleting fixture.example IDs or overwriting Prompt 01/IAM/SDLC/vuln/infra SSOTs

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Excepted subjects leaving the population denominator silently become Effective | Generic `ExceptionApproved` promotion for the bound subject; never silent Effective (`gov_012` / approved-exception fixture). |
| PDF/document-present flags treated as operational effectiveness | Freshness/population/manual-review predicates (`policy-current`, `training-current-all`, …); Hybrid/Manual honesty. |
| Partial training or vendor inventory auto-passing all-subjects tests | Incomplete-training and vendor-review-gaps fixtures cannot be Effective on all-subjects. |
| GRC-product tokens leaking because validator omits vanta/drata/servicenow/jira | Target greps those tokens out of catalog TOML (`gov_014`), not via a self-referential source-file substring (no I4a). |
| Target test I4a trap: grepping its own source for a forbidden substring present in the assertion | Target asserts loaded catalog IDs and TOML content, not self-grep of the suite file. |
| Filename/slug collision with Prompt 07 `resilience.toml` and `control.resilience.*` operational ids | Continuity/DR *governance* lives in `governance.toml` (`evidence.resilience.continuity-plan`); no `resilience.toml` this slice. |
| Overlap with Prompt 05 IS-adjacent SDLC policy or Prompt 06 vuln exceptions | No overwrite of SDLC/vuln/infra SSOTs; no `vulnerability.toml` / `evidence.secret.exposure`. |
| ISO pack rewrite breaking `sdd_iso27001_assurance_target` | ISO pack ids/mappings unchanged (`gov_016`). |
| Inventing a second loader, `EvidenceValue`, population resolver, or exception engine | Consumed Prompt 01–03 APIs and IR Exception/Risk; no second copy. |
| Baseline remaining required-green absence-of-family after target lands | Skip-retired with `#[ignore = "superseded by sdd_governance_catalog_target"]`; forced `--ignored` FAILED 9 tests. |
| Parallel Prompt 05–07 product overwrites in the same session | Additive catalog/fixtures/attestation path; IAM/sibling gates asserted by target `gov_015`/`gov_016`. |
| Secrets or compliance narratives in governance fixtures | Facts via `with_value`; no secrets or certification language. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) |
| Baseline | PASS on old | `cargo test --test sdd_governance_catalog_baseline --test sdd_governance_catalog_target` → exit 0. Baseline: **ok. 28 passed; 0 failed**. Characterization of current tree only: no governance family TOML/fixtures/`evidence.manual.attestation`. Target file is a Cargo.toml harness so both `--test` names exist; desired-behavior target RED is a later step. Excerpt: `running 28 tests` … `test result: ok. 28 passed; 0 failed; 0 ignored` / `running 1 test` / `dual_suite_target_harness_is_registered ... ok` / `test result: ok. 1 passed`. Suites: `tests/sdd/governance_catalog.baseline.rs`, `tests/sdd/governance_catalog.target.rs`. |
| Target pre | FAIL on old | Same dual command → exit 1. Target encodes GOV-001–016 (36-control band, `evidence.manual.attestation` + domain types, six freshness/population tests, eight fixtures, ExceptionApproved vs expired, no GRC/framework tokens in catalog TOML, no I4a self-grep). **FAILED. 1 passed; 16 failed** on missing family; `gov_000` registration passes. Official cargo `--test` pair did not compile this workspace because sibling collector/assurance crates were mid-edit; the registered target file was `rustc --test`’d and executed with the same assertions (exit 1). No product TOML/fixtures implemented. Excerpt: `test result: FAILED. 1 passed; 16 failed; 0 ignored` / `GOV family missing: \`control.governance.information-security-policy\` is not loaded`. Suite: `tests/sdd/governance_catalog.target.rs`. |
| Implement | target PASS | `cargo test --test sdd_governance_catalog_baseline --test sdd_governance_catalog_target -- --test-threads=1` → exit 0. Target: **ok. 17 passed; 0 failed** (`gov_000`…`gov_016` all ok). Baseline default: **ok. 0 passed; 0 failed; 28 ignored** (`#[ignore = "superseded by sdd_governance_catalog_target"]`). Landed 34 Hybrid/Manual controls (25 Hybrid / 9 Manual; 30–45 band), first-class `evidence.manual.attestation`, eight fixtures, and generic ExceptionApproved (not silent Effective). `cargo fmt --all -- --check` exit 0. Workspace `cargo test --workspace --features demo`: governance/IAM/population targets pass; unrelated `sdd_assessment_lineage_target` LIN-002/004 fail on serialize purity. `clippy -D warnings` still fails in sibling applicability-engine files (not this slice). |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default: **ok. 0 passed; 0 failed; 28 ignored**. Forced `--ignored --nocapture`: **FAILED. 19 passed; 9 failed**. Failures: `catalog_manifest_lists_only_fixture_example_and_identity` (manifest lists `controls/governance.toml`); `loaded_catalog_has_no_governance_family` (found `control.governance.*`); `governance_fixtures_are_absent`; `evidence_manual_attestation_is_not_catalog_content`; `required_governance_tests_are_undeclared`; `exception_approved_promotion_is_identity_break_glass_shaped`; `public_contract_documents_iam_not_governance`; `spec_and_draft_adr_exist_as_spec_phase_artifacts`; `manual_attestation_is_capability_and_legacy_type`. Not additive. Dual-suite registration kept (`gov_000`). |
| Supersede | target still PASS | After skip-supersede: `cargo test --test sdd_governance_catalog_baseline --test sdd_governance_catalog_target` → baseline **0 passed / 28 ignored**; target **ok. 17 passed; 0 failed; 0 ignored** (`gov_000_dual_suite_is_registered` … `gov_016_iso_and_iam_gates_stay_intact`). Target remains the CI gate. |
| Docs/ADR | updated | [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md), [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md), [`docs/sdd/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`docs/sdd/population-runtime.md`](population-runtime.md), [`docs/sdd/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md), [`frameworks/README.md`](../../frameworks/README.md) |

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

Provider-neutral governance catalog family on existing catalog/population infrastructure:

- 34 independently assessable Hybrid/Manual controls (25 Hybrid / 9 Manual) in the 30–45 band: `control.{governance,risk,personnel,vendor,incident,resilience}.*`. Resilience ids are continuity/DR *governance* only (no operational `resilience.toml`).
- First-class `evidence.manual.attestation` plus Prompt-08 domain types as facts (principal, timestamp, subject, artifact, freshness, review state) stored via `with_value`. No secrets or compliance narratives.
- Required tests as freshness/population/manual-review predicates: `policy-current`, `training-current-all`, `critical-risk-review-current`, `management-review-current`, `internal-audit-current`, `exercise-current` — not Exists(one PDF).
- Eight fixtures under `fixtures/assurance/canonical/v1/governance/` (clock `2026-08-18T12:00:00Z`): `current-documents`, `stale-documents`, `missing-documents`, `incomplete-training-population`, `vendor-review-gaps`, `approved-exception`, `expired-exception`, `manual-review-despite-evidence`.
- Partial training or vendor populations cannot be Effective on all-subjects tests; missing evidence is `InsufficientEvidence`.
- Hybrid/manual operational-evidence controls stay Hybrid/Manual and cannot auto-pass from a document-present flag.
- Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject (never silent Effective); expired exceptions do not suppress failing results.
- Catalog validator accepts the slice: no orphans/duplicates/dangling refs; no vanta/drata/servicenow/jira/iso/soc2/nis2 in catalog TOML.
- Consumes Prompt 01 `CanonicalCatalog::{load,validate,digest}`, Prompt 02 `EvidenceValue::with_value`, Prompt 03 evaluator, and IR Exception/Risk. No second loader, no ISO remap, no GRC product. ISO pack ids/mappings and IAM/fixture.example ids unchanged (`gov_015`/`gov_016`).

### Files changed (implement)

`catalog/canonical/v1/manifest.toml`, `catalog/canonical/v1/controls/governance.toml`, `catalog/canonical/v1/evidence/governance.toml`, `catalog/canonical/v1/tests/governance.toml`, `crates/weeping-angel-control-test/src/population.rs`, `crates/weeping-angel-assurance/src/lib.rs`, `docs/sdd/governance-canonical-assurance-catalog.md`, `docs/adr/0003-governance-canonical-assurance-catalog.md`, `docs/contracts/assurance-runtime.md`, `tests/sdd/governance_catalog.baseline.rs`, `tests/sdd/governance_catalog.target.rs`, `fixtures/assurance/canonical/v1/governance/current-documents/evidence.json`, `fixtures/assurance/canonical/v1/governance/stale-documents/evidence.json`, `fixtures/assurance/canonical/v1/governance/missing-documents/evidence.json`, `fixtures/assurance/canonical/v1/governance/incomplete-training-population/evidence.json`, `fixtures/assurance/canonical/v1/governance/vendor-review-gaps/evidence.json`, `fixtures/assurance/canonical/v1/governance/approved-exception/evidence.json`, `fixtures/assurance/canonical/v1/governance/expired-exception/evidence.json`, `fixtures/assurance/canonical/v1/governance/manual-review-despite-evidence/evidence.json`, `crates/weeping-angel-collector/src/github/client.rs`, `crates/weeping-angel-collector/src/github/protection.rs`, `crates/weeping-angel-collector/src/github/security.rs`, `crates/weeping-angel-collector/src/github/mod.rs`.

Collector GitHub files in that list are sibling Prompt-09 dirt recorded on the implement worktree, not governance catalog content.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-25d885be2883` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 17 881 599 |
| `duration_ms_sum` | 6 210 254 (~103.5 min) |
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
| Scope | `sdd-scope` | ok | 214 662 | 485 882 |
| Spec | `sdd-spec` | ok | 961 989 | 1 128 960 |
| BaselineGreen | `sdd-baseline-green` | ok | 602 963 | 2 414 151 |
| TargetRed | `sdd-target-red` | ok | 1 067 633 | 3 830 983 |
| Implement | `sdd-implement` | ok | 1 914 135 | 7 598 175 |
| DocsAdr | `sdd-docs-adr` | ok | 1 215 252 | 2 097 340 |
| Iterate | `sdd-baseline-post-check` | ok | 147 691 | 109 436 |
| Supersede | `sdd-supersede` | ok | 85 929 | 216 672 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-governance-canonical-assurance-catalog-telemetry.json`](sdd-governance-canonical-assurance-catalog-telemetry.json).

---

## Remaining backlog (not this slice)

1. Full GRC workflow/SaaS product or document editors
2. ISO/SOC2/NIS2 remapping onto `control.governance.*` (Prompt 12) or growing the ISO pack sliver
3. ServiceNow/Jira/Vanta/Drata or other GRC/ITSM collectors
4. Prompt 04 IAM MFA/privileged-membership/account-status content (owned by IAM family)
5. Prompt 05 SDLC/change technical evidence and secure-development policy
6. Prompt 06 finding-level risk acceptance, `vulnerability.toml`, `evidence.secret.exposure`
7. Prompt 07 operational backup/HA, `evidence.resilience.recovery-plan`, `resilience.toml`
8. Certification or compliant/audit-passed language (forbidden)
9. Sibling workspace `clippy -D warnings` failures in applicability-engine files (not this slice)
10. Unrelated `sdd_assessment_lineage_target` LIN-002/004 serialize-purity failures on `cargo test --workspace --features demo`
11. Forking catalog loader, `EvidenceValue`, population resolver, or Exception/Risk engine (forbidden)

---

## Summary

Governance Canonical Assurance Catalog v1 (Prompt 08) landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `e430980c` (28 passed characterizing absence of governance family plus IAM/ISO sliver), target RED (16 failed) for missing `control.governance.information-security-policy` / `evidence.manual.attestation` / fixtures, then target GREEN 17/17. Absence-characterization baseline skip-retired (`#[ignore]`; forced `--ignored` 9 fail). 34 Hybrid/Manual controls (30–45 band), first-class `evidence.manual.attestation`, eight fixtures, ExceptionApproved not silent Effective. Prompt 01–03 APIs consumed, not forked. ISO pack and IAM ids unchanged.
