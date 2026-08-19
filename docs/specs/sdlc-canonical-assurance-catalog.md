# SDD: SDLC / Source-Control Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Implemented — target GREEN; baseline superseded** |
| Program | Canonical Assurance Catalog v1 |
| Slice | SDLC catalog — source-control, change-management, CI/CD, secure-development, release-integrity, software-supply-chain |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`, 2026-08-19; HEAD still has no SDLC population family) |
| Prior I1 freeze (reuse IDs, not as SSOT) | [`.sdd/runs/sdd-sdd-625d28d3-3dbb1ba8da/spec.md`](../../.sdd/runs/sdd-sdd-625d28d3-3dbb1ba8da/spec.md) — 26 controls, 20 evidence types, 26 tests, 7 fixtures |
| Prior abort notes | [`.sdd/runs/xylex-sdd-v3-v5-sdlc-catalog-failure.md`](../../.sdd/runs/xylex-sdd-v3-v5-sdlc-catalog-failure.md) |
| Dual-suite | `sdd_sdlc_catalog_target` GREEN (SDLC-001…016); `sdd_sdlc_catalog_baseline` superseded (`#[ignore]`) |
| Transition | **replacement** (IAM pattern): baseline `#[ignore = "superseded by sdd_sdlc_catalog_target"]` |
| ADR | Accepted [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) |
| Catalog-infrastructure SSOT (do not overwrite) | [`docs/specs/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) — pointer-only |
| Typed evidence / population runtime (consumed) | [`docs/specs/typed-evidence.md`](typed-evidence.md), [`docs/specs/population-runtime.md`](population-runtime.md) |
| IAM pattern (do not overwrite) | [`docs/specs/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) + [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md) + `tests/contracts/iam_catalog.{baseline,target}.rs` |
| Spine / ISO law | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Product manager | Cargo (`pnpm` is `apps/docs` only) |

This document is the durable SSOT for the **SDLC catalog slice**. It does not replace catalog infrastructure catalog infrastructure, typed evidence typed evidence, population runtime population runtime, or the IAM catalog IAM family. catalog infrastructure through IAM have landed; this slice consumes their loader, `EvidenceValue`, and population evaluator and **must not** invent a second copy.

This spec is the durable SSOT. Product TOML, fixtures, contract pointer, and ADR accept have landed.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

A GitHub, GitLab, or Bitbucket collector must be able to populate the same evidence contracts and receive the same control results. This slice is provider-neutral and framework-neutral.

---

## 1. Problem / user-visible goal

Organizations need to assess repository inventory, visibility, branch protection, review policy, CI/CD permissions, deployment authorization, release integrity, and software-supply-chain hygiene using **provider-neutral** canonical controls.

On SHA `e430980c0d27a8138a153d49b62ddf3c57827891` the only SDLC-adjacent product content is:

- a **thin ISO 27001 pack sliver** (`source.branch-protection`, `source.required-review`, `source.code-ownership`, `source.security-scanning`, `source.commit-signing`) wired to **GitHub-shaped** evidence types (`source.branch.protection`, `source.branch.required_reviews`, `source.codeowners.present`, `source.security.secret_scanning.enabled`, `source.commit.signing`) as presence/hybrid checks;
- a **catalog-infrastructure fixture** (`control.source.protected-branch` / `evidence.source.protected-branch` / `test.source.protected-branch`) whose test is `op = "exists"` — an existence check, not a population assertion;
- a landed **IAM family** (`catalog/canonical/v1/{controls,evidence,tests}/identity.toml`) that does not cover repositories, CI, or releases;
- a GitHub collector that advertises `source.*` types (`GITHUB_EVIDENCE_TYPES`) and must **not** be expanded here.

There is no `control.source.default-branch-protection` population family, no `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*` contracts, no `catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml`, and no `fixtures/assurance/canonical/v1/sdlc/`. A future GitLab or Bitbucket collector therefore has nowhere canonical to emit SDLC facts.

**User-visible goal:** a coherent SDLC catalog (20–30 independently assessable controls; this slice specifies **26**) that evaluates realistic **repository / branch / deployment** populations from any future SCM/CI collector’s canonical evidence, produces deterministic explainable results (missing ≠ stale ≠ failure ≠ manual review ≠ approved exception), and passes catalog validation plus full workspace verification.

This slice does **not** claim ISO/SOC 2 coverage. Framework remapping is ISO remap — landed for A.8.25 / A.8.26 onto `control.source.*`; see [`iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §13. This slice does **not** expand the GitHub collector. This slice does **not** implement vulnerability, infrastructure, and governance catalogs families.

---

## 2. Dependencies and fail-closed blockers

| Slice | Owns | On characterization SHA `e430980c…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules | **Landed.** Identity + fixture.example listed in `manifest.toml`. | Add SDLC TOML + manifest lines. Do not invent a second loader/validator/digest. Do not delete fixture.example IDs. |
| 02 typed evidence | Typed `EvidenceValue`, seal rules | **Landed.** | Declare required fact *names* and semantic types. No second value enum. |
| 03 population runtime | Subject populations, `AllSubjects` / `CoverageAtLeast` / `NoneSubjects` / `ExceptionApproved` / `InsufficientEvidence` | **Landed.** Identity inventory special-case + generic `inventory.subject` / `inventory.complete`. | Declare population-based tests. **Do not locally reimplement coverage math. Do not add `resolve_repository_inventory`. Do not change generic population semantics.** |
| 04 IAM family | `control.identity.*` | **Landed.** | Leave identity files and `sdd_iam_catalog_target` green. |
| 06 vulnerability | `control.vulnerability.*` | **Landed** (`vulnerability.toml`; SSOT [`vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md)). | Do not implement finding/SLA/coverage family. Scanning-*enabled* belongs here; finding-as-evidence belongs in vulnerability catalog. |
| 07 / 08 | infrastructure / governance | Spec / draft only. | Do not implement those families. |
| 09 GitHub collector | emit canonical facts from GitHub | Sibling spec exists; collector still emits `source.*` only. | Do not expand the collector. |
| 12 ISO remap | retarget pack mappings onto `control.*` | Spec / suite sibling; pack still maps to `source.branch-protection`. | Do not retarget ISO mappings. |

Rebase rule: adapt SDLC content to the landed contracts. Prefer existing `CanonicalCatalog`, `EvidenceValue`, and `evaluate_coverage` over extending this slice’s scope.

---

## 3. Current behavior (characterization on `e430980c0d27a8138a153d49b62ddf3c57827891`)

Inspected: `catalog/canonical/v1/`, `crates/weeping-angel-canonical-catalog`, `weeping-angel-control-test` (`population.rs`), `weeping-angel-collector/src/github/descriptor.rs`, `frameworks/iso-27001/2022/{metadata,mappings}.toml`, `tests/contracts/{iam,canonical,iso27001}_*`, root `Cargo.toml` `[[test]]` table, SDLC catalog, IAM SSOT, ADR draft, `.sdd/runs/xylex-sdd-v3-v5-sdlc-catalog-failure.md`.

### 3.1 Canonical catalog tree

`catalog/canonical/v1/manifest.toml` lists only:

```text
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests    = ["tests/fixture.example.toml", "tests/identity.toml"]
```

No `controls/sdlc.toml`, `evidence/sdlc.toml`, or `tests/sdlc.toml`. No split `source.toml` / `repository.toml` / `cicd.toml`. No `fixtures/assurance/canonical/v1/sdlc/`.

Pinned infrastructure fixture (CAT-015; **must survive**):

| Kind | Id | Expression |
| --- | --- | --- |
| control | `control.source.protected-branch` | domains `secureDevelopment` |
| evidence | `evidence.source.protected-branch` | declared envelope type `source.branch.protection` |
| test | `test.source.protected-branch` | `op = "exists"` on `evidence.source.protected-branch` |

IAM family (23 `control.identity.*`, 12 `evidence.identity.*`, 23 `test.identity.*`) is present and must remain. IAM tests already use `op = "all-subjects"` / `"coverage-at-least"` / `"none-subjects"` / `"manual-review"` with `[[test.subjects]] kind` values that parse via `SubjectKind::parse_name`.

Catalog validator already rejects reserved provider/framework **ID segments** (`github`, `gitlab`, `bitbucket`, `azure`, `iso27001`, `soc2`, …). IDs must be lowercase dotted `kind.family.slug` with `-` allowed; `_` is malformed (`validate_id`). `azure-devops` is forbidden in SDLC content even if the validator only lists the `azure` segment. Extra files under `catalog/canonical/v1/` that are not listed in the manifest fail `CanonicalCatalog::validate` (`Unlisted`).

### 3.2 ISO pack SDLC sliver (frozen for this slice)

`frameworks/iso-27001/2022/metadata.toml`:

| Pack control id | Test | Required evidence (GitHub-shaped) |
| --- | --- | --- |
| `source.branch-protection` | `test.source.branch-protection` | `source.branch.protection` |
| `source.required-review` | `test.source.required-review` | `source.branch.required_reviews` |
| `source.code-ownership` | `test.source.code-ownership` | `source.codeowners.present` |
| `source.security-scanning` | `test.source.security-scanning` | `source.security.secret_scanning.enabled` |
| `source.commit-signing` | `test.source.commit-signing` | `source.commit.signing` |

Mappings (unchanged): `iso27001:a.8.25` → `source.branch-protection` / `source.required-review` / `source.code-ownership`; `iso27001:a.8.26` → `source.security-scanning`. Completeness remains `partial`.

`sdd_iso27001_assurance_target` froze prefixes `source.` and those pack ids. This slice **must not** retarget mappings or rename pack ids. ISO remap remapped A.8.25 / A.8.26 onto `control.source.default-branch-protection` / `required-review` / `secure-development-policy` / `secret-scanning` / `security-review` and retired the slivers ([remap §13](iso-27001-canonical-remap.md#13-implement-log)).

Those tests are presence/hybrid checks, not “all non-archived in-scope repositories have a protected default branch.”

### 3.3 GitHub collector (do not expand)

`GITHUB_EVIDENCE_TYPES` in `crates/weeping-angel-collector/src/github/descriptor.rs` advertises GitHub-native `source.*` names (repository exists/visibility/archived, default branch, branch protection / reviews / status checks / force-push / deletion, CODEOWNERS, admin/collaborator, Dependabot / secret scanning / code scanning, workflow permissions, rulesets, commit signing). Subject types: `repository`, `branch`.

Catalog tests must **not** import or assert against `GITHUB_EVIDENCE_TYPES` as the SDLC contract. Scanner engines (`src/engines/*`, `src/depcheck/*`) are not evidence contracts.

Sibling pin: `tests/contracts/github_collector.baseline.rs` `ghc_b028` asserts `catalog/canonical/v1/evidence/repository.toml` **does not exist**. Prefer a single `sdlc.toml` per section so that pin stays green.

### 3.4 Population runtime (consume, do not change)

`resolve_population` order (`crates/weeping-angel-control-test/src/population.rs`):

1. explicit `EvidenceSet` population;
2. selector with explicit IDs (authoritative);
3. identity inventory (`evidence.identity.inventory` + privileged / service-account);
4. generic `inventory.subject` + `inventory.complete` (`authoritative`);
5. otherwise infer subject ids from the observation type with **Unknown** completeness.

Strong all-subjects / 100% coverage **refuses** `Effective` when completeness is Partial (`InsufficientEvidence`) or Unknown (`Inconclusive`). Technical / missing / stale partitions are already distinct from `Ineffective`. Approved unexpired subject-scoped IR `Exception` records skip subjects (`ExceptionApproved`).

There is **no** `resolve_repository_inventory`. This slice must not add one. Repository populations use generic `inventory.subject` (`kind=repository` or `deployment`) plus `inventory.complete` and/or `EvidenceSet::set_population`.

### 3.5 Test harness topology (Rust)

Language: Rust. Package manager: Cargo. Test framework: `cargo test` (libtest). Root `Cargo.toml` registers SDD suites with explicit `[[test]]` entries. `tests/contracts/*.rs` is **not** auto-discovered.

On this SHA, registered SDD targets include `sdd_{assurance_runtime,iso27001_assurance,iso27001_remap,population_runtime,typed_evidence,iam_catalog,canonical_assurance_catalog,github_collector,assessment_lineage}_{baseline,target}` and `sdd_compliance_ir_target`. There is **no** `sdd_sdlc_catalog_*` target and **no** `tests/contracts/sdlc_catalog.{baseline,target}.rs` on the primary tree (scratch copies live only under `.sdd/runs/sdd-sdd-*`).

**Harness law:** implement must register:

```toml
[[test]]
name = "sdd_sdlc_catalog_baseline"
path = "tests/contracts/sdlc_catalog.baseline.rs"

[[test]]
name = "sdd_sdlc_catalog_target"
path = "tests/contracts/sdlc_catalog.target.rs"
```

Without those rows, `cargo test --test sdd_sdlc_catalog_*` cannot run.

### 3.6 What “SDLC assessment” means today

A caller can compile the ISO pack and run `test.source.branch-protection`, which requires **some** `source.branch.protection` envelope to exist. It cannot:

- require protection on every non-archived in-scope repository;
- distinguish missing inventory from one unprotected repo from stale scan evidence;
- evaluate force-push, deletion, reviewer count, review ownership, workflow write permissions, protected environments, provenance, or lockfile integrity as population predicates;
- accept GitLab/Bitbucket-shaped facts without teaching tests those providers.

The baseline suite therefore characterizes **absence of the SDLC population family** plus **presence of the ISO sliver, exists-only fixture, and IAM sibling** — not “absence of every `control.source.*` string” (that would collide with `control.source.protected-branch`).

### 3.7 TargetAuthor hygiene (prior v3/v5 abort)

Prior SDLC catalog target suites aborted protocol gates (`.sdd/runs/xylex-sdd-v3-v5-sdlc-catalog-failure.md`):

- Do **not** read the target file’s own source and assert it does not contain `#[ignore` (self-referential; unsatisfiable — the assertion quotes the needle).
- Do **not** pair a self-read of the test file with a negated `.contains("literal")` (I4a false-positive). Hyphen-id rules belong on **loaded catalog IDs** (`CanonicalCatalog` product state), not on the test source text.
- Do **not** write `#[ignore]` on target tests.
- Do **not** treat prior isolated-worktree implementations as landed product.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

**Preferred layout** (keeps `ghc_b028` green; IAM-shaped single family file):

```text
catalog/canonical/v1/
  manifest.toml                         # list new files; keep fixture.example + identity
  controls/sdlc.toml                    # all 26 control.{source,cicd,release,supply-chain}.*
  evidence/sdlc.toml                    # all 20 evidence.{repository,cicd,deployment,release,supply-chain}.*
  tests/sdlc.toml                       # all 26 test.{source,cicd,release,supply-chain}.*
```

`manifest.toml` `[files]` must list the three `sdlc.toml` paths. Do **not** add `evidence/repository.toml` (sibling collector baseline `ghc_b028` asserts that path is absent). Split `source.toml` / `cicd.toml` / `release.toml` / `supply-chain.toml` is acceptable **only** if no `evidence/repository.toml` is created and every id below is listed in `[files]`.

Do **not** add these controls to `frameworks/iso-27001/2022/metadata.toml`. Do **not** remove or rewrite `fixture.example.toml` / `identity.toml`.

Deterministic fixtures (IAM layout):

```text
fixtures/assurance/canonical/v1/sdlc/
  healthy-org/
  degraded-org/
  partial-coverage/
  unprotected-default-branch/
  missing-scan-evidence/
  stale-dependency-scan/
  approved-exception/
```

Each fixture is a frozen evidence set (+ optional Exception) with fixed `collectedAt`.

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.source.<slug>
control.cicd.<slug>
control.release.<slug>
control.supply-chain.<slug>
evidence.repository.<slug>
evidence.cicd.<slug>
evidence.deployment.<slug>
evidence.release.<slug>
evidence.supply-chain.<slug>
test.source.<slug>
test.cicd.<slug>
test.release.<slug>
test.supply-chain.<slug>
```

Never put `github`, `gitlab`, `azure-devops`, `bitbucket`, `iso`, `iso27001`, `soc2`, or similar provider/framework names in canonical IDs.

Reject in canonical SDLC content (validator + target suite):

- provider tokens in IDs or as the subject of a control (`github`, `gitlab`, `bitbucket`, `azure-devops`, `gitea`);
- framework tokens in IDs or narrative (`iso27001`, `iso-27001`, `soc2`, `soc-2`, `nis2`, `dora`, `gdpr`);
- GitHub-native object names as **catalog** ids (`CODEOWNERS` as a type id, `rulesets`, `dependabot` as a control id);
- orphaned evidence types or tests;
- duplicate IDs;
- existence-only tests masquerading as the required population tests (see §4.5).

Correct: `control.source.default-branch-protection`. Incorrect: `control.github.branch-protection`, `control.gitlab.protected-branch`, `test.iso27001.a.8.25`.

Provider-specific field names (`github_ruleset_id`, `gitlab_approval_rule_id`) must not appear in evidence **type** ids. They may appear only inside a collector’s private normalize step that **emits** canonical facts.

**Collision with the infrastructure fixture:** keep `control.source.protected-branch` (exists-only fixture). The population control is `control.source.default-branch-protection`. Do not reuse `test.source.protected-branch` as a population test.

Catalog TOML `[[evidence]]` rows may declare a short `evidence_type` (IAM pattern: `identity.mfa-status` → SDLC `repository.branch-protection`). Fixture JSON and evaluator selectors must use the **catalog evidence id** (`evidence.repository.*`, same as IAM fixtures using `evidence.identity.*`).

Test `[[test.subjects]]` `kind` values must parse via `SubjectKind::parse_name` (`repository`, `branch`, `deployment`, `organization`). Do not invent new `SubjectKind` variants.

### 4.3 Control family (26 independently assessable controls)

Do not split these into micro-controls to inflate count. Titles and objectives are framework-neutral. Count must stay in **20–30**.

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.source.repository-inventory` | Repository inventory and ownership | Automated | repository | `inventory` | `test.source.repository-inventory-complete` |
| `control.source.visibility-governance` | Repository visibility governance | Automated | repository | `visibility` | `test.source.visibility-governed` |
| `control.source.default-branch-protection` | Protected default branch | Automated | repository | `default-branch`, `branch-protection` | `test.source.default-branches-protected` |
| `control.source.force-push-restricted` | Force-push restriction | Automated | repository / branch | `branch-protection` | `test.source.force-push-restricted` |
| `control.source.branch-deletion-restricted` | Branch deletion restriction | Automated | repository / branch | `branch-protection` | `test.source.branch-deletion-restricted` |
| `control.source.required-review` | Required pull-request review | Automated | repository | `review-policy` | `test.source.reviews-required` |
| `control.source.minimum-reviewer-count` | Minimum reviewer count | Automated | repository | `review-policy` | `test.source.minimum-reviewer-count` |
| `control.source.review-ownership` | Review ownership | Automated | repository | `review-ownership` | `test.source.review-ownership-present` |
| `control.source.required-status-checks` | Required status checks | Automated | repository | `status-checks` | `test.source.required-status-checks` |
| `control.source.admin-bypass-governance` | Administrator bypass governance | Hybrid | repository | `branch-protection`, `review-policy` | `test.source.admin-bypass-governed` |
| `control.source.signed-commits` | Signed commits / artifacts | Automated | repository | `commit-signing` | `test.source.signed-commits-required` |
| `control.source.secret-scanning` | Secret scanning | Automated | repository | `security-scanning` | `test.source.secret-scanning-enabled` |
| `control.source.code-scanning` | Code scanning / SAST | Automated | repository | `security-scanning` | `test.source.code-scanning-enabled` |
| `control.source.dependency-scanning` | Dependency vulnerability scanning | Automated | repository | `dependency-scanning` | `test.source.dependency-scanning-current` |
| `control.source.dependency-update-monitoring` | Dependency update monitoring | Automated | repository | `dependency-scanning` | `test.source.dependency-updates-monitored` |
| `control.supply-chain.dependency-integrity` | Dependency pinning / lockfile integrity | Automated | repository | `lockfile-state` | `test.supply-chain.lockfile-integrity` |
| `control.cicd.workflow-permissions` | CI workflow permission minimization | Automated | repository | `workflow-permissions` | `test.cicd.workflow-permissions-minimized` |
| `control.release.protected-environment` | Protected deployment environments | Automated | deployment | `environment-protection` | `test.release.environments-protected` |
| `control.release.authorization` | Release authorization | Hybrid | repository / deployment | `authorization` | `test.release.authorization-recorded` |
| `control.release.authority-separation` | Separation of development / release authority | Hybrid | organization | `authorization` | `test.release.authority-separated` |
| `control.supply-chain.build-provenance` | Build provenance | Automated | repository | `build-provenance` | `test.supply-chain.provenance-present` |
| `control.supply-chain.artifact-integrity` | Artifact integrity | Automated | repository | `artifact-integrity` | `test.supply-chain.artifacts-have-integrity` |
| `control.source.change-traceability` | Change traceability | Hybrid | repository | `change-trace` | `test.source.changes-traceable` |
| `control.source.security-review` | Security review for material changes | Hybrid | repository | `security-review` | `test.source.security-review-recorded` |
| `control.source.secure-development-policy` | Secure-development policy evidence | Manual | organization | `secure-development-policy` | `test.source.secure-development-policy-attested` |
| `control.supply-chain.unsupported-components` | Unsupported / deprecated component handling | Hybrid | repository | `component-support`, `dependency-scanning` | `test.supply-chain.unsupported-components-handled` |

Each control record must carry: stable id, title, description/objective, domain(s) from existing `ControlDomain` (`SecureDevelopment`, `ChangeManagement`, `VulnerabilityManagement`, `Governance` as appropriate; TOML tokens follow IAM/fixture camelCase such as `secureDevelopment`), evidence-requirement refs, test refs, and an honest automation class (`Automated` | `Hybrid` | `Manual`; IAM uses lowercase `automation = "automated"`).

**Do not invent technical automation** for release authorization, authority separation, security review of material changes, or secure-development policy. Those remain Hybrid or Manual.

### 4.4 Canonical evidence (facts, not conclusions)

Envelope `type` in fixtures should equal the **catalog evidence id** (IAM fixture convention). Collectors later normalize provider payloads into these types.

| Evidence type | Observed facts (canonical names; store via `EvidenceValue`) | Not allowed |
| --- | --- | --- |
| `evidence.repository.inventory` | `subject_id`, `archived` (bool), `owner_id?`, `criticality?` (`production` \| `non-production` \| `unknown`), `in_scope` (bool) | `compliant`, provider repo-object dumps as type id |
| `evidence.repository.visibility` | `subject_id`, `visibility` (`public` \| `internal` \| `private`), `visibility_allowed` (bool) | “visibility control passed” |
| `evidence.repository.default-branch` | `subject_id`, `default_branch`, `default_branch_named` (bool) | — |
| `evidence.repository.branch-protection` | `subject_id`, `protected` (bool), `force_push_restricted` (bool), `deletion_restricted` (bool); fixtures also store inverse `force_push_allowed` / `deletion_allowed` and `admin_bypass_allowed` / `admin_bypass_restricted` | “branch protection effective” |
| `evidence.repository.review-policy` | `subject_id`, `reviews_required` (bool), `required_reviewer_count` (integer), `meets_review_threshold` (bool) | “review control passed” |
| `evidence.repository.review-ownership` | `subject_id`, `ownership_defined` (bool) | `codeowners` as a type id |
| `evidence.repository.security-scanning` | `subject_id`, `secret_scanning_enabled` (bool), `code_scanning_enabled` (bool), `applicable` (bool) | scanner finding dumps |
| `evidence.repository.dependency-scanning` | `subject_id`, `dependency_scanning_enabled` (bool), `scanned_at` (timestamp), `updates_monitored` (bool), `critical` (bool) | “no vulns” / engine internals |
| `evidence.repository.commit-signing` | `subject_id`, `signing_required` (bool) | raw signatures |
| `evidence.repository.change-trace` | `subject_id` or change id, `traceable` (bool), `change_ref?` | ticket-system dumps as type id |
| `evidence.repository.security-review` | `subject_id` or change id, `material` (bool), `reviewed` (bool), `reviewed_at?` | “security review effective” |
| `evidence.repository.secure-development-policy` | `population_id` / org id, `attested` (bool), `attested_at?` | policy PDF as type id |
| `evidence.cicd.workflow-permissions` | `subject_id`, `default_write` (bool), `permissions_minimized` (bool) | workflow YAML as type id |
| `evidence.cicd.status-checks` | `subject_id`, `status_checks_required` (bool) | — |
| `evidence.deployment.environment-protection` | `subject_id` (environment), `production` (bool), `authorization_required` (bool), `protected` (bool) | provider environment object dumps |
| `evidence.release.authorization` | `subject_id`, `authorized` (bool), `authorizer_id?`, `dev_release_separated` (bool) | “release approved by compliance” |
| `evidence.supply-chain.build-provenance` | `subject_id`, `provenance_present` (bool) | SLSA level as a control conclusion |
| `evidence.supply-chain.artifact-integrity` | `subject_id`, `integrity_evidence_present` (bool) | raw checksum files as secrets |
| `evidence.supply-chain.lockfile-state` | `subject_id`, `lockfile_present` (bool), `pins_direct_deps` (bool) | — |
| `evidence.supply-chain.component-support` | `subject_id`, `unsupported_present` (bool), `unsupported_handled` (bool) | — |

Seal rules still apply: no credential-shaped keys; no compliance narratives (`certified`, `compliant`, `audit passed`).

Additional supporting types may be added only if referenced by a control and a test (no orphans). Prefer extending facts on the types above.

**Generic population envelopes (not new catalog ids):** fixtures / the target harness MUST also establish population runtime authoritative completeness using existing types `inventory.subject` (`kind=repository` or `deployment`) and `inventory.complete` (`authoritative=true`), and/or `EvidenceSet::set_population`. Do not treat inferred Unknown populations as `Effective`.

IAM fixtures store bools as strings (`"true"`). SDLC fixtures may do the same (`with_fact` string-compat) or use typed `EvidenceValue`; either is valid as long as evaluator comparisons succeed.

### 4.5 Tests (population-based, not existence checks)

Required reusable tests (SDLC catalog examples + extras so no control is untested) — **26** tests, one per control:

```text
test.source.repository-inventory-complete
test.source.visibility-governed
test.source.default-branches-protected
test.source.force-push-restricted
test.source.branch-deletion-restricted
test.source.reviews-required
test.source.minimum-reviewer-count
test.source.review-ownership-present
test.source.required-status-checks
test.source.admin-bypass-governed
test.source.signed-commits-required
test.source.secret-scanning-enabled
test.source.code-scanning-enabled
test.source.dependency-scanning-current
test.source.dependency-updates-monitored
test.supply-chain.lockfile-integrity
test.cicd.workflow-permissions-minimized
test.release.environments-protected
test.release.authorization-recorded
test.release.authority-separated
test.supply-chain.provenance-present
test.supply-chain.artifacts-have-integrity
test.source.changes-traceable
test.source.security-review-recorded
test.source.secure-development-policy-attested
test.supply-chain.unsupported-components-handled
```

Semantics (authoritative intent; exact `TestExpr` spelling follows population runtime / IAM `all-subjects` / `coverage-at-least`):

| Test | Population | Pass | Fail | Missing | Stale |
| --- | --- | --- | --- | --- | --- |
| `default-branches-protected` | all **non-archived** in-scope repositories | every subject `protected=true` | ≥1 in-scope repo `protected=false` | inventory unknown **or** known repo lacks branch-protection | stale protection / inventory |
| `force-push-restricted` | protected default branches / their repos | `force_push_restricted=true` (equivalently none have unauthorized `force_push_allowed=true`) | unauthorized force-push still allowed | missing protection envelope | stale |
| `reviews-required` | production / in-scope repos (use `criticality=production` or `in_scope`) | `reviews_required=true` | review not required | missing review-policy | stale |
| `minimum-reviewer-count` | same as reviews | `meets_review_threshold=true` (threshold is a fixture/policy integer fact, not a hardcoded GitHub “2”) | count below threshold | missing count | stale |
| `secret-scanning-enabled` | in-scope repos where `applicable=true` (or all in-scope if applicability omitted) | `secret_scanning_enabled=true` | scanning disabled | missing scan evidence → **InsufficientEvidence**, never `Ineffective` | stale |
| `workflow-permissions-minimized` | in-scope repos with CI | `permissions_minimized=true` and not `default_write=true` | overbroad write | missing workflow-permissions | stale |
| `environments-protected` | production deployments | `authorization_required=true` / `protected=true` | prod env unprotected | missing environment-protection | stale |
| `dependency-scanning-current` | critical in-scope repos | enabled **and** `scanned_at` within freshness | enabled=false | missing scan envelope → InsufficientEvidence | `scanned_at` outside window → `StaleEvidence` |
| `artifacts-have-integrity` | repos/releases that produce artifacts (where required) | `integrity_evidence_present=true` (and provenance when bound) | integrity missing while required | missing artifact-integrity | stale |

**Forbidden encoding:** `Exists(evidence.repository.branch-protection)` as the body of `test.source.default-branches-protected`. Existence of some protection fact is not protection on the population. The infrastructure fixture may keep `exists` **only** on `test.source.protected-branch`.

**Shipped `TestExpr` bindings** (population runtime `all-subjects` / `coverage-at-least` 100%; hybrid/manual use `manual-review`):

| Test | `op` | Field |
| --- | --- | --- |
| `test.source.repository-inventory-complete` | `all-subjects` | `in_scope` |
| `test.source.visibility-governed` | `coverage-at-least` | `visibility_allowed` |
| `test.source.default-branches-protected` | `all-subjects` | `protected` |
| `test.source.force-push-restricted` | `coverage-at-least` | `force_push_restricted` |
| `test.source.branch-deletion-restricted` | `coverage-at-least` | `deletion_restricted` |
| `test.source.reviews-required` | `coverage-at-least` | `reviews_required` |
| `test.source.minimum-reviewer-count` | `coverage-at-least` | `meets_review_threshold` |
| `test.source.review-ownership-present` | `all-subjects` | `ownership_defined` |
| `test.source.required-status-checks` | `coverage-at-least` | `status_checks_required` |
| `test.source.admin-bypass-governed` | `manual-review` | — |
| `test.source.signed-commits-required` | `coverage-at-least` | `signing_required` |
| `test.source.secret-scanning-enabled` | `coverage-at-least` | `secret_scanning_enabled` |
| `test.source.code-scanning-enabled` | `coverage-at-least` | `code_scanning_enabled` |
| `test.source.dependency-scanning-current` | `coverage-at-least` | `scanned_at` |
| `test.source.dependency-updates-monitored` | `coverage-at-least` | `updates_monitored` |
| `test.supply-chain.lockfile-integrity` | `coverage-at-least` | `pins_direct_deps` |
| `test.cicd.workflow-permissions-minimized` | `coverage-at-least` | `permissions_minimized` |
| `test.release.environments-protected` | `coverage-at-least` | `authorization_required` (`kind=deployment`) |
| `test.release.authorization-recorded` | `manual-review` | — |
| `test.release.authority-separated` | `manual-review` | — |
| `test.supply-chain.provenance-present` | `coverage-at-least` | `provenance_present` |
| `test.supply-chain.artifacts-have-integrity` | `coverage-at-least` | `integrity_evidence_present` |
| `test.source.changes-traceable` | `coverage-at-least` | `traceable` |
| `test.source.security-review-recorded` | `manual-review` | — |
| `test.source.secure-development-policy-attested` | `manual-review` | — |
| `test.supply-chain.unsupported-components-handled` | `manual-review` | — |

Missing evidence must **not** be converted into technical failure (`Effectiveness` technical / type-mismatch). Unknown / partial inventory must not produce `Effective` on all-subjects tests.

Result metadata must explain: population size, evaluated, passing, failing, missing, coverage, failing subject ids, missing subject ids.

Applicability: archived repositories are out of the default-branch-protection population. Secret/code scanning may use `applicable=false` to exclude subjects without treating them as failures. Do not change generic population semantics to encode this — filter via inventory facts / selector tags / fixture population membership.

Subject kinds: consume existing IR `Repository`, `Branch`, `Deployment`, `Organization`. Do not add new `SubjectKind` variants.

### 4.6 Manual / hybrid honesty

| Control | Why not fully automated |
| --- | --- |
| `admin-bypass-governance` | Whether admin override is *allowed by policy* is organizational; the boolean `admin_bypass_allowed` is supporting. Default Hybrid. |
| `release.authorization` | Authorization is a governance act. Technical `authorized=true` is supporting. |
| `release.authority-separation` | Org role design; not a single SCM flag. |
| `change-traceability` | Ticket completeness and linkage quality are hybrid. |
| `security-review` | Materiality and reviewer independence are organizational. |
| `secure-development-policy` | Policy text and attestation are Manual. |
| `unsupported-components` | Risk acceptance vs upgrade is hybrid. |

A single technical signal must not auto-pass Hybrid/Manual controls. Absence of attestation → `ManualReviewRequired` or `InsufficientEvidence`, never `Effective`.

### 4.7 Fixtures (deterministic)

| Fixture | Intent | Expected highlights |
| --- | --- | --- |
| `healthy-org` | Authoritative multi-repo inventory; all in-scope non-archived defaults protected; force-push/deletion restricted; reviews + reviewer count; ownership; status checks; scanning enabled and current; workflow permissions minimized; prod envs protected; lockfile + provenance + integrity present | Automated SDLC tests `Effective`. Hybrid/manual `Effective` only if attestations present; otherwise `ManualReviewRequired` / `InsufficientEvidence` — document the choice and keep it deterministic. |
| `degraded-org` | Multi-repo population with several independent defects (unprotected + overbroad workflow + unprotected prod env) | Corresponding tests `Ineffective` naming failing subjects. Other healthy subjects still pass. |
| `partial-coverage` | Population marked **non-authoritative** / Partial | All-subjects tests → `InsufficientEvidence` or `Inconclusive` (not Effective, not Ineffective-as-if-empty). |
| `unprotected-default-branch` | Authoritative inventory; exactly one in-scope non-archived repo `protected=false`; rest healthy | `default-branches-protected` → `Ineffective` naming that repo. Missing ≠ fail. |
| `missing-scan-evidence` | Authoritative inventory; one in-scope repo has no `security-scanning` / `dependency-scanning` envelope | Scan tests → `InsufficientEvidence` (not Ineffective, not technical failure). |
| `stale-dependency-scan` | Scan envelopes exist but `scanned_at` / collectedAt outside freshness | `dependency-scanning-current` → `StaleEvidence`. |
| `approved-exception` | Named repo lacks protection (or scanning); IR `Exception` `status=Approved`, unexpired, subject-scoped to that repo and the bound control | That subject contributes `ExceptionApproved` / excepted, not silent Effective and not Ineffective. Expired/revoked must not pass. |

Fixtures emit **canonical** `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*` (plus generic `inventory.subject` / `inventory.complete` as required for completeness). No `source.branch.protection` in SDLC fixtures. No collector id in evidence type. No GitHub/GitLab/Bitbucket tokens in fixture type strings.

Follow IAM fixture JSON shape (`fixture`, `collectedAt`, `evidence[]` with `type` / `subject_id` / `facts`, optional `exception` object).

### 4.8 Integration rules (consume, do not redesign)

- Loader / validate / digest: catalog infrastructure `CanonicalCatalog`. SDLC files must pass `validate`.
- Typed facts: typed evidence `EvidenceValue`. Prefer `with_value`; `with_fact` remains string-compat (IAM fixtures still store `"true"` strings).
- Population evaluation: population runtime `evaluate` / `evaluate_coverage`. SDLC tests are **declarations**. Do not implement `AllSubjects` here. Do not add `resolve_repository_inventory` or an `SdlcPopulation` fork.
- Exception: reuse IR `Exception` + `Effectiveness::ExceptionApproved`.
- Subject kinds: existing IR only.
- ISO pack, GitHub collector, scanner engines, framework compiler, generic `TestExpr` semantics: **untouched**.
- Catalog tests depend only on evidence contracts, **not** on `GITHUB_EVIDENCE_TYPES`, SARIF adapters, or `src/engines/*`.

### 4.9 Dual-suite protocol

Follow `tests/contracts/iam_catalog.{baseline,target}.rs`. Root `Cargo.toml` does **not** auto-discover `tests/contracts/*.rs`.

| Suite | Path | Role |
| --- | --- | --- |
| Baseline | `tests/contracts/sdlc_catalog.baseline.rs` · `sdd_sdlc_catalog_baseline` | GREEN on **current** tree: no SDLC population family (`control.source.default-branch-protection` absent); ISO sliver present; fixture `control.source.protected-branch` exists-only; IAM sibling present; collector still `source.*`; no `resolve_repository_inventory`. **Do not** assert absence of every `control.source.*` (fixture collision). After target GREEN: `#[ignore = "superseded by sdd_sdlc_catalog_target"]`. |
| Target | `tests/contracts/sdlc_catalog.target.rs` · `sdd_sdlc_catalog_target` | RED on current tree for **missing SDLC family / population tests / fixtures** — not compile noise. Then **GREEN** — CI gate (SDLC-001…016). Assert **loaded catalog IDs** and fixture evaluation, never self-read suite text to assert absence of a substring that appears in the assertion. |

Suggested target assertion clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| SDLC-001 | Catalog tree / loader loads `*/sdlc.toml` offline; `CanonicalCatalog::validate` succeeds |
| SDLC-002 | Digest remains deterministic after adding SDLC files |
| SDLC-003 | All 26 `control.{source,cicd,release,supply-chain}.*` ids present (20–30 independently assessable); fixture `control.source.protected-branch` still present |
| SDLC-004 | Required `evidence.repository.*` / cicd / deployment / release / supply-chain types declared; no orphans |
| SDLC-005 | Required `test.source.*` / cicd / release / supply-chain ids declared and referenced |
| SDLC-006 | Validator rejects provider tokens (`github`, `gitlab`, `bitbucket`) in new SDLC ids |
| SDLC-007 | Validator / SDLC file text rejects `iso27001` / `soc2` / `nis2` in SDLC catalog files |
| SDLC-008 | No SDLC control lives in the ISO pack as `control.source.*`; ISO pack ids and mappings unchanged |
| SDLC-009 | `test.source.default-branches-protected` is population-based (fails `unprotected-default-branch`; does not pass on a single protection envelope) |
| SDLC-010 | Missing vs stale vs fail vs manual vs exception distinguished on the seven fixtures |
| SDLC-011 | Partial coverage cannot yield Effective on all-subjects tests |
| SDLC-012 | Approved unexpired exception → `ExceptionApproved` / excepted for that subject |
| SDLC-013 | Authorization / authority-separation / security-review / secure-development-policy marked Hybrid or Manual |
| SDLC-014 | Catalog tests do not reference `GITHUB_EVIDENCE_TYPES`, scanner engines, or GitHub-native type ids |
| SDLC-015 | Identity family, CAT fixture IDs, and population runtime `population.rs` identity/generic resolution remain; no `resolve_repository_inventory` |
| SDLC-016 | `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, `sdd_canonical_assurance_catalog_target` stay green; `ghc_b028` stays green (no `evidence/repository.toml`) |

### 4.10 Documentation after implement

- This file’s landed-record section (§12).
- Accepted [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) (draft filename retired).
- Pointer on [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (family exists after TOML lands).
- Pointer-only mention on catalog infrastructure SSOT. Do not overwrite catalog infrastructure / 04 SSOTs.

No GitHub collector expansion or ISO remap is claimed by this slice.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase.

1. Dual-suite `sdd_sdlc_catalog_baseline` + `sdd_sdlc_catalog_target` is registered in root `Cargo.toml` at paths `tests/contracts/sdlc_catalog.baseline.rs` and `tests/contracts/sdlc_catalog.target.rs`.
2. On current tree: baseline GREEN characterizing **no SDLC population family**, ISO sliver + `control.source.protected-branch` exists-only fixture, IAM sibling present, collector still `source.*`, no `resolve_repository_inventory` — and **not** “no `control.source.*` at all.”
3. On current tree: target RED for missing SDLC family / population tests / fixtures, not compile noise; suite never self-reads its source to assert `!contains("#[ignore")` or other literals that appear in the assertion.
4. After implement: target GREEN; baseline proven FAIL or additive-documented then `#[ignore = "superseded by sdd_sdlc_catalog_target"]`; target still GREEN; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
5. Twenty-six `control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*` controls exist with stable ids, domains, evidence requirements, test refs, and honest automation class; independently assessable count stays in 20–30; `control.source.protected-branch` fixture remains; population default-branch id is `control.source.default-branch-protection`.
6. The twenty evidence types in §4.4 are declared as facts, not conclusions; IDs are provider-neutral (`evidence.repository.*` etc., not `evidence.github.*`).
7. Tests include at least the nine SDLC catalog example ids and evaluate **populations** (all non-archived in-scope default branches protected), not existence of one envelope.
8. Evaluator outcomes distinguish missing data, stale data, actual failure, manual review, and approved exception on the seven named fixtures; missing scan evidence is `InsufficientEvidence`, not technical failure; partial/unknown population cannot be `Effective` on all-subjects tests.
9. Release authorization, authority-separation, security-review, and secure-development-policy are Hybrid or Manual; they cannot auto-pass from a single technical flag.
10. Catalog validator accepts the SDLC slice: no duplicate/orphan/dangling ids, no provider names, no ISO/SOC2/NIS2 references in canonical SDLC content; files listed in `manifest.toml`; preferred paths are `{controls,evidence,tests}/sdlc.toml` (no `evidence/repository.toml`).
11. ISO pack control ids and mappings are unchanged; `sdd_iso27001_assurance_target` remains green.
12. GitHub collector is not expanded; SDLC catalog tests do not couple to `GITHUB_EVIDENCE_TYPES` or scanner internals.
13. No second `CanonicalCatalog` loader, no second `EvidenceValue` enum, no `resolve_repository_inventory` / `SdlcPopulation` fork. population runtime coverage is consumed as-is.
14. Approved-exception fixture uses existing Exception IR; expired/revoked exceptions do not pass.
15. IAM family and catalog-infrastructure fixture IDs remain; `sdd_iam_catalog_target` and `sdd_canonical_assurance_catalog_target` stay green.
16. catalog infrastructure SSOT `docs/specs/canonical-assurance-catalog-v1.md` is pointer-only (not overwritten as domain SSOT); this file is the SDLC slice SSOT.
17. A GitHub, GitLab, or Bitbucket collector could independently populate the same evidence contracts and receive the same control results (no GitHub-native object names in canonical IDs).
18. After implement, public contract `docs/specs/assurance-runtime.md` names the landed SDLC family; ADR is accepted at [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) (no `-draft`).

---

## 6. Out of scope

- Expanding or remapping the GitHub collector.
- Implementing GitLab, Bitbucket, Azure DevOps, or Gitea collectors.
- Remapping ISO 27001 (or SOC 2 / NIS2) onto `control.source.*` (ISO remap — A.8.25 / A.8.26 remapped; see [`iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §13).
- Redesign of `CanonicalCatalog` loader/validator/digest (catalog infrastructure).
- Redesign of typed evidence.
- Changing generic population semantics or adding `resolve_repository_inventory` (population runtime).
- Rewriting ISO `metadata.toml` / `mappings.toml` (`source.branch-protection` stayed until ISO remap remapped A.8.25 / A.8.26; [remap §13](iso-27001-canonical-remap.md#13-implement-log)).
- Removing or changing `control.source.protected-branch` fixture IDs.
- Changing IAM catalog content.
- Implementing vulnerability catalog / 07 / 08 families.
- Scanner engine / depcheck / SARIF adapter changes.
- New `SubjectKind` variants.
- Certification, “compliant”, or audit-passed language.
- Inventing a parallel ADR or overwriting catalog infrastructure SSOT as the domain SSOT.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Fixture `control.source.protected-branch` collides with the new family | Distinct population id `control.source.default-branch-protection`; CAT-015 fixture stays exists-only. Baseline must not assert “no `control.source.*`”. |
| `tests/contracts/*.rs` unregistered → `cargo test --test sdd_sdlc_catalog_*` fails | AC-1: implement adds `[[test]]` rows. |
| Implementer adds `resolve_repository_inventory` | AC-13 + SDLC-015; use `inventory.subject` / `inventory.complete` / explicit population. |
| Existence checks sneak in as SDLC tests | SDLC-009: unprotected-default-branch must fail; a lone protection envelope must not pass. |
| Missing scan evidence coded as Ineffective | SDLC-010 + AC-8: missing → InsufficientEvidence. |
| ISO pack rewritten | AC-11; do not touch `frameworks/iso-27001/2022` source rows. |
| Provider names leak into IDs or fixture types | Validator + SDLC-006/007/014. |
| Hybrid controls auto-pass from one technical fact | Honest automation class; AC-9. |
| Target suite self-reads `#[ignore` / `contains('_')` and trips I4a / v3 AC-2 | Assert loaded catalog IDs; no self-read negated contains. |
| `evidence/repository.toml` breaks `ghc_b028` | Prefer `{controls,evidence,tests}/sdlc.toml`. |
| Baseline remains a CI green that asserts catalog absence | After target GREEN, `#[ignore]` like IAM. |
| Unknown completeness makes healthy-org Inconclusive | Fixtures must mark authoritative completeness via existing population runtime paths. |

---

## 8. Dual-suite and SDD protocol (implement phase)

Hard protocol (do not skip):

```text
1. Spec (this file) — no product feature code
2. Register [[test]] + write baseline/target suites
3. Baseline GREEN on CURRENT code (characterization in §3)
4. Target RED on CURRENT code for the RIGHT reason
     (missing SDLC family / population tests / fixtures — not compile noise)
5. Implement catalog TOML + fixtures + contract pointer + ADR accept
     (consume loader / EvidenceValue / population math; no second evaluator)
6. Target GREEN; prove baseline FAILS or is additive-documented;
     #[ignore = "superseded by sdd_sdlc_catalog_target"]; target still GREEN
7. Workspace verify: cargo test --workspace --features demo;
     cargo fmt --all -- --check;
     cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Fail-closed if: baseline cannot go green on current characterization; target cannot go red for the **right** reason; or target never greens.

Commands (after `[[test]]` registration):

```text
cargo test --test sdd_sdlc_catalog_baseline -- --nocapture
cargo test --test sdd_sdlc_catalog_target -- --nocapture
cargo test --workspace --features demo
```

---

## 9. ADR

Architecture / public-contract decision: SDLC content is a **canonical catalog family** (`control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*`) consumed later by framework mappings, not an ISO-pack extension and not a GitHub-specific catalog.

Accepted: [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md).

---

## 10. Planning SHA record

```text
planning_sha = e430980c0d27a8138a153d49b62ddf3c57827891
branch       = main
note         = catalog infrastructure through IAM landed (catalog fixture + IAM family + typed evidence +
               population runtime); SDLC catalog markdown present; no SDLC catalog files;
               ISO source sliver + exists-only fixture only; Cargo.toml has no
               sdd_sdlc_catalog_* [[test]] rows; prior SDLC catalog SDD runs did not
               land product content on this tree
```

---

## 11. Relevant files and symbols (discovery)

**Files (do not all change):**

- `catalog/canonical/v1/manifest.toml`
- `catalog/canonical/v1/controls/fixture.example.toml`
- `catalog/canonical/v1/controls/identity.toml`
- `catalog/canonical/v1/evidence/fixture.example.toml`
- `catalog/canonical/v1/tests/fixture.example.toml`
- `crates/weeping-angel-canonical-catalog/src/lib.rs`
- `crates/weeping-angel-control-test/src/{lib,expr,population}.rs`
- `crates/weeping-angel-collector/src/github/descriptor.rs`
- `crates/weeping-angel-assurance-ir/src/{subject,control,exception}.rs`
- `frameworks/iso-27001/2022/metadata.toml`
- `frameworks/iso-27001/2022/mappings.toml`
- `tests/contracts/iam_catalog.{baseline,target}.rs`
- `tests/contracts/canonical_assurance_catalog.target.rs`
- `tests/contracts/iso27001_assurance.target.rs`
- `tests/contracts/github_collector.baseline.rs` (`ghc_b028`)
- `Cargo.toml`

**Symbols:** `CanonicalCatalog::{load,validate,digest,stats}`, `CATALOG_SCHEMA`, `TestExpr::{AllSubjects,CoverageAtLeast,NoneSubjects,ManualReview,Exists}`, `evaluate` / `evaluate_coverage`, `Population` / `PopulationCompleteness` / `PopulationEvaluation`, `Effectiveness::{Effective,Ineffective,InsufficientEvidence,StaleEvidence,ManualReviewRequired,ExceptionApproved}`, `SubjectKind::{Repository,Branch,Deployment,Organization}`, `Exception` / `ExceptionStatus`, `EvidenceValue`, `GITHUB_EVIDENCE_TYPES`.

**Live seams:** catalog TOML + manifest listing; population runtime population completeness; ISO pack compile; GitHub collector advertisement (read-only); IAM and CAT dual-suites; root `[[test]]` harness.

**Tooling:** language Rust; package manager Cargo (pnpm only for `apps/docs`); test framework `cargo test`. Integration tests under `tests/contracts/` require explicit `[[test]]`.

---

## 12. Landed record

| Surface | Location |
| --- | --- |
| Controls (26) | `catalog/canonical/v1/controls/sdlc.toml` |
| Evidence (20) | `catalog/canonical/v1/evidence/sdlc.toml` |
| Tests (26) | `catalog/canonical/v1/tests/sdlc.toml` |
| Manifest listing | `catalog/canonical/v1/manifest.toml` `[files]` |
| Fixtures (7) | `fixtures/assurance/canonical/v1/sdlc/{healthy-org,degraded-org,partial-coverage,unprotected-default-branch,missing-scan-evidence,stale-dependency-scan,approved-exception}/` |
| Loader / digest | catalog infrastructure crate; no SDLC-specific load path |
| Target suite | `tests/contracts/sdlc_catalog.target.rs` (`sdd_sdlc_catalog_target`) GREEN SDLC-001…016 |
| Baseline suite | `tests/contracts/sdlc_catalog.baseline.rs` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md) (draft filename retired) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) names the family and evidence types |
| ISO pack | This slice did not rewrite pack-local `source.*` rows. ISO remap remapped A.8.25 / A.8.26 onto catalog `control.source.*` ([remap §13](iso-27001-canonical-remap.md#13-implement-log)). |
| Collectors | This slice did not expand GitHub/GitLab/Bitbucket. GitHub collector later emits these contracts ([`github-collector.md`](github-collector.md)). |
| Test-bound facts | Force-push / deletion predicates bind `force_push_restricted` / `deletion_restricted` (fixtures also store inverse `*_allowed`). Hybrid/manual tests use `op = "manual-review"`. |

Workspace `assurance catalog stats` after this family (sibling families may also be listed in the same manifest):

```text
schema: weeping-angel/canonical-catalog/v1
catalog: canonical
version: 1
```

This slice contributes 26 controls, 20 evidence types, and 26 tests. Digest is catalog infrastructure `CatalogDigest` over parsed documents and changes if any catalog TOML changes.

---

## 13. Definition of done

A GitHub, GitLab, or Bitbucket collector could independently populate the same evidence contracts and receive the same control results. Catalog validation, deterministic tests, and full workspace checks pass. No certification language. No second loader, value type, or population evaluator.
