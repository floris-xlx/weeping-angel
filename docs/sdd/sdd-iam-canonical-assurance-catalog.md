# SDD run: IAM Canonical Assurance Catalog v1

| Field | Value |
| --- | --- |
| Run id | `sdd-8b849c58bb26` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `8b849c58bb26e75c` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 04: 23 `control.identity.*` controls, `evidence.identity.*` facts, population `test.identity.*` declarations, eight fixtures. Consumes Prompts 01–03; no ISO remap, no IdP collectors. |
| Spec | [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| ADR | [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md) |
| Catalog infra ADR | [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-iam-canonical-assurance-catalog-telemetry.json`](sdd-iam-canonical-assurance-catalog-telemetry.json) |
| Dual-suite | `tests/sdd/iam_catalog.baseline.rs` (skip-retired / `#[ignore]`) · `tests/sdd/iam_catalog.target.rs` (active; iam_000…iam_016) |
| Characterization SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** IAM Canonical Assurance Catalog v1
- **Problem:** There is no provider-neutral IAM catalog. The only access/lifecycle controls live in the ISO 27001 pack as thin ids (`access.mfa.privileged` and siblings) wired to GitHub-shaped evidence and existence/hybrid checks, so a future identity provider cannot emit canonical facts and the runtime cannot assert population predicates such as all privileged identities have MFA.
- **Current behavior (planning SHA `5fa3a23`):** `catalog/canonical/v1` and `CanonicalCatalog::{load,validate,digest}` are absent (Prompts 01–03 have not landed on that SHA). ISO `metadata.toml` holds `access.mfa.privileged`, `access.least-privilege`, `access.periodic-review`, `personnel.access-termination` with tests requiring `source.admin.permissions`, `source.collaborator.permission`, `policy.access.reviewed`, `personnel.access.terminated`. `EvidenceObservation` facts are `BTreeMap<String,String>`; `EvidenceValue::parse_fact` coerces strings. `CoverageAtLeast` always returns `PartiallyEffective`; `TestExpr` has no AllSubjects/population index. Exception IR and `Effectiveness::ExceptionApproved` exist but evaluate never emits `ExceptionApproved`. `Identity`/`SubjectKind` are thin (no `ServiceAccount`). GitHub lists `source.admin.permissions` but `collaborators.rs` is a stub. No IAM fixtures or `sdd_iam_catalog_*` suites.
- **Desired behavior:** A Prompt-01 catalog family `control.identity.*` (~23 independently assessable controls), `evidence.identity.*` fact contracts, and population-based `test.identity.*` declarations that consume — not fork — the catalog loader, typed evidence, and population runtime. Hybrid/manual honesty for approval, SoD, and periodic review. Eight deterministic fixtures distinguishing missing, stale, fail, manual, and approved break-glass exception. ISO pack and collectors unchanged. Dual-suite `sdd_iam_catalog_baseline` (GREEN now) + `sdd_iam_catalog_target` (RED now); after target GREEN, retire baseline so absence-of-catalog is not required CI green. If `CoverageAtLeast` remains a stub, fail-closed and keep target RED.
- **ADR:** needed — accepted at [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_iam_catalog_baseline` + `sdd_iam_catalog_target` registered like existing root `[[test]]` SDD suites.
2. On current code, baseline GREEN characterizing catalog absence and the ISO IAM sliver; target RED for missing `control.identity.*` / `evidence.identity.*` / population fixtures, not unrelated compile noise.
3. After implement: target GREEN; baseline ignored or removed; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. 23 `control.identity.*` controls with stable ids, domains, evidence requirements, test refs, and honest Automated|Hybrid|Manual class (20–30, no micro-controls).
5. `evidence.identity.{inventory,authentication-state,mfa-status,privileged-membership,role-membership,last-active,account-status,account-owner,access-review,lifecycle-event,service-account,external-access}` declared as facts, not conclusions.
6. Required `test.identity.*` ids evaluate populations (all privileged identities have MFA), not existence of one envelope.
7. Eight fixtures distinguish missing vs stale vs fail vs manual vs approved exception; partial inventory cannot be Effective on all-subjects tests.
8. Access-approval, SoD, and periodic-review stay Hybrid or Manual and cannot auto-pass without attestation.
9. Catalog validator accepts the slice: no orphans/duplicates/dangling refs, no provider names, no ISO/SOC2/NIS2 in canonical IAM content.
10. ISO pack ids/mappings and `sdd_iso27001_assurance_target` remain unchanged/green; no Entra/Okta/Workspace collectors; no second loader, `EvidenceValue`, or local `CoverageAtLeast`/`AllSubjects` implementation.

### Out of scope

- Entra, Okta, Google Workspace, AD, Cognito, or GitHub-identity collectors
- ISO 27001 / SOC 2 / NIS2 remapping onto `control.identity.*` (Prompt 12)
- Redesign of `CanonicalCatalog` loader, validator, or digest (Prompt 01)
- Redesign of typed evidence or digest canonicalization (Prompt 02)
- Implementing real `CoverageAtLeast` / `AllSubjects` / population indexes (Prompt 03)
- Generic `TestExpr` semantic changes unless a documented Prompt-03 blocker exists
- Rewriting `frameworks/iso-27001/2022` `access.*` / `personnel.*` ids or mappings
- Adding `SubjectKind::ServiceAccount` or a third `SubjectSelector` in this slice
- HRIS, IGA, PAM, or ticketing product integrations
- Certification or compliant/audit-passed language
- SDLC, vulnerability, infrastructure, or governance catalog families
- Overwriting [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) (Prompt 01 SSOT)

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Prompts 01–03 missing; implementers invent a parallel loader, value type, or population evaluator | Consumed existing Prompt 01 loader and Prompt 03 coverage runtime; no second loader / `EvidenceValue` / local `CoverageAtLeast`/`AllSubjects`. |
| `CoverageAtLeast` stub makes all-subjects tests look `PartiallyEffective` or gets locally completed | Target `iam_011` fail-closed until coverage is real; implement used Prompt 03 population runtime rather than completing a local stub. |
| Existence checks shipped as IAM tests (a single `mfa-status` envelope passes privileged MFA) | `test.identity.*` ids evaluate populations (e.g. all privileged identities have MFA), not one envelope. |
| ISO pack rewritten or `sdd_iso27001_assurance_target` broken by new ids | ISO pack ids/mappings unchanged; `sdd_iso27001_assurance_target` 49 passed. |
| Provider or framework tokens leak into canonical IDs, fixtures, or narratives | Validator accepts the slice; no Entra/Okta/Workspace collectors; no ISO/SOC2/NIS2 in canonical IAM content. |
| Hybrid/manual controls auto-pass from one technical fact | Access-approval, SoD, and periodic-review stay Hybrid or Manual and cannot auto-pass without attestation. |
| `ExceptionApproved` never emitted so break-glass cannot green without a second exception engine | `iam_012` binds Exception IR to the break-glass subject; approved exception is not silent Effective. |
| Two `SubjectSelector` types plus a third invented here | No third selector; `SubjectKind::ServiceAccount` not added this slice. |
| Baseline remains required-green absence-of-catalog after target lands | Skip-retired with `#[ignore = "superseded by sdd_iam_catalog_target"]`; forced `--ignored` is not green. |
| Secrets or compliance narratives in identity fixtures | Eight deterministic fixtures; facts not conclusions; no certification language. |
| Prompt 01 SSOT overwritten by this slice | `docs/sdd/canonical-assurance-catalog-v1.md` updated as pointer only; not overwritten as SSOT. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| Baseline | PASS on old | `cargo test --workspace --features demo --test sdd_iam_catalog_baseline` → **ok. 17 passed; 0 failed**. Characterization SHA `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`: no `catalog/canonical/v1` or `CanonicalCatalog`; no `control.identity.*` / `evidence.identity.*` / `test.identity.*`; ISO sliver `access.mfa.privileged` + GitHub `source.admin.permissions` existence check; `CoverageAtLeast` stub; `ExceptionApproved` never emitted; `collaborators.rs` stub; no IAM fixtures. Excerpt: `catalog_canonical_v1_is_absent ... ok` … `no_canonical_identity_family_in_product_or_packs ... ok`. Suite: `tests/sdd/iam_catalog.baseline.rs`. |
| Target pre | FAIL on old | `cargo test --workspace --features demo --test sdd_iam_catalog_target` → exit 1. **FAILED. 6 passed; 13 failed**. Failures are missing catalog/fixtures/population semantics, not compile noise. Excerpt: `iam_001: catalog/canonical/v1 must exist so the IAM family can load`; `iam_010: fixtures/assurance/canonical/v1/identity must exist`; `iam_011: CoverageAtLeast must not stay a Prompt-03 stub` (`left/right: "subject coverage remains partial unless the threshold is met"`); `iam_012: Exception IR must bind the break-glass subject`. Suites: `tests/sdd/iam_catalog.target.rs`, `Cargo.toml`, spec. Baseline file untouched. |
| Implement | target PASS | `cargo test --features demo --test sdd_iam_catalog_target --offline` → **ok. 19 passed; 0 failed** (`iam_000`…`iam_013` and siblings). Prompt 04 IAM family (23 `control.identity.*`, `evidence.identity.*` facts, population `test.identity.*`, eight fixtures) on Prompt 01 loader and Prompt 03 coverage runtime. ISO pack and collectors unchanged. Workspace `cargo test --workspace --features demo --offline` exit 0; `sdd_iso27001_assurance_target` 49 passed; `sdd_canonical_assurance_catalog_target` 22 passed. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default: **ok. 0 passed; 0 failed; 17 ignored** (`#[ignore = "superseded by sdd_iam_catalog_target"]`). Forced `--ignored`: **FAILED. 8 passed; 9 failed** (`catalog_canonical_v1_is_absent`: current tree has no `catalog/` — inverted; `canonical_catalog_api_is_absent`: found struct `CanonicalCatalog`; `no_canonical_identity_family_in_product_or_packs`: `evidence.identity.inventory`; `no_identity_provider_collectors_or_iam_fixtures`; `test_expr_has_no_all_subjects_population_index`; `CoverageAtLeast` placeholder; `evaluate_compiled` attaches `TestExpr`). Not additive. Dual-suite registration kept (`iam_000`). |
| Supersede | target still PASS | After skip-supersede: `cargo test --workspace --features demo --test sdd_iam_catalog_target` → **ok. 19 passed; 0 failed; 0 ignored**. Workspace `cargo test --workspace --features demo` exit 0; target still 19 passed; baseline 0 passed / 17 ignored. Target remains the CI gate. |
| Docs/ADR | updated | [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`frameworks/README.md`](../../frameworks/README.md) |

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

Provider-neutral IAM catalog family on existing catalog/population infrastructure:

- 23 `control.identity.*` controls with stable ids, domains, evidence requirements, test refs, and honest Automated|Hybrid|Manual class (20–30 range; no micro-controls).
- `evidence.identity.{inventory,authentication-state,mfa-status,privileged-membership,role-membership,last-active,account-status,account-owner,access-review,lifecycle-event,service-account,external-access}` declared as facts, not conclusions.
- Population `test.identity.*` ids (including `mfa-enabled`, `privileged-mfa-enabled`, `no-inactive-privileged-accounts`, `no-terminated-active-accounts`, `all-service-accounts-have-owner`, `access-review-current`, `no-unapproved-guest-access`, plus non-orphan extras) evaluate populations, not existence of one envelope.
- Eight fixtures under `fixtures/assurance/canonical/v1/identity/` distinguishing missing vs stale vs fail vs manual vs approved exception (`healthy-org`, `privileged-without-mfa`, `inactive-admin-active`, `terminated-employee-active`, `service-account-without-owner`, `partial-inventory`, `stale-access-review`, `break-glass-approved-exception`). Partial inventory cannot be Effective on all-subjects tests.
- Access-approval, SoD, and periodic-review stay Hybrid or Manual and cannot auto-pass without attestation.
- Catalog validator accepts the slice: no orphans/duplicates/dangling refs, no provider names, no ISO/SOC2/NIS2 in canonical IAM content.
- Consumes Prompt 01 `CanonicalCatalog` loader and Prompt 03 coverage runtime. No second loader, `EvidenceValue`, or local `CoverageAtLeast`/`AllSubjects`. No Entra/Okta/Workspace collectors. ISO pack ids/mappings unchanged.

### Files changed (implement)

`catalog/canonical/v1/manifest.toml`, `catalog/canonical/v1/controls/identity.toml`, `catalog/canonical/v1/evidence/identity.toml`, `catalog/canonical/v1/tests/identity.toml`, `crates/weeping-angel-canonical-catalog/src/lib.rs`, `crates/weeping-angel-control-test/src/population.rs`, `docs/adr/0003-iam-canonical-assurance-catalog.md`, `docs/sdd/iam-canonical-assurance-catalog.md`, `fixtures/assurance/canonical/v1/identity/healthy-org/evidence.json`, `fixtures/assurance/canonical/v1/identity/privileged-without-mfa/evidence.json`, `fixtures/assurance/canonical/v1/identity/inactive-admin-active/evidence.json`, `fixtures/assurance/canonical/v1/identity/terminated-employee-active/evidence.json`, `fixtures/assurance/canonical/v1/identity/service-account-without-owner/evidence.json`, `fixtures/assurance/canonical/v1/identity/partial-inventory/evidence.json`, `fixtures/assurance/canonical/v1/identity/stale-access-review/evidence.json`, `fixtures/assurance/canonical/v1/identity/break-glass-approved-exception/evidence.json`, `tests/sdd/iam_catalog.baseline.rs`, `tests/sdd/iso27001_assurance.target.rs`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-8b849c58bb26` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 12 748 173 |
| `duration_ms_sum` | 7 611 098 (~126.9 min) |
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
| Scope | `sdd-scope` | ok | 818 522 | 378 524 |
| Spec | `sdd-spec` | ok | 1 096 017 | 568 643 |
| BaselineGreen | `sdd-baseline-green` | ok | 986 512 | 1 156 736 |
| TargetRed | `sdd-target-red` | ok | 1 207 370 | 1 348 357 |
| Implement | `sdd-implement` | ok | 2 768 406 | 7 169 644 |
| DocsAdr | `sdd-docs-adr` | ok | 492 278 | 1 507 490 |
| Iterate | `sdd-baseline-post-check` | ok | 117 351 | 200 170 |
| Supersede | `sdd-supersede` | ok | 124 642 | 418 609 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-iam-canonical-assurance-catalog-telemetry.json`](sdd-iam-canonical-assurance-catalog-telemetry.json).

---

## Remaining backlog (not this slice)

1. Entra, Okta, Google Workspace, AD, Cognito, or GitHub-identity collectors
2. ISO 27001 / SOC 2 / NIS2 remapping onto `control.identity.*` (Prompt 12)
3. HRIS, IGA, PAM, or ticketing product integrations
4. Adding `SubjectKind::ServiceAccount` or a third `SubjectSelector`
5. SDLC, vulnerability, infrastructure, or governance catalog families
6. Certification or compliant/audit-passed language (forbidden)
7. Rewriting ISO pack `access.*` / `personnel.*` ids or mappings (forbidden this slice)
8. Forking catalog loader, typed evidence, or population runtime (forbidden)

---

## Summary

IAM Canonical Assurance Catalog v1 landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `5fa3a23` (17 passed characterizing catalog absence and the ISO IAM sliver), target RED (13 failed) for missing `control.identity.*` / fixtures / population semantics, then target GREEN 19/19. Absence-of-catalog baseline skip-retired (`#[ignore]`; forced `--ignored` 9 fail). Prompt 01 loader and Prompt 03 coverage runtime consumed, not forked. ISO pack and collectors unchanged. Hybrid/manual honesty for approval, SoD, and periodic review. Eight fixtures distinguish missing vs stale vs fail vs manual vs approved exception.
