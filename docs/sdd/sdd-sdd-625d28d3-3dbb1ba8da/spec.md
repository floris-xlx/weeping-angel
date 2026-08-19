# SDD: SDLC / Source-Control Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Specified — implement next (I1 frozen)** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 05 — source-control, change-management, CI/CD, secure-development, release-integrity, software-supply-chain |
| Source prompt | [`docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md`](../../prompts/canonical-assurance-v1/05-sdlc-catalog.md) |
| Run | `sdd-625d28d3-3dbb1ba8da` |
| Snapshot | head `f6eb344cacefe44f398730c7e963c98887427f1b` · tree `35de6aa30ef5b7fc0019a4f99841306ea3af406b` · commit `f6eb344cacefe44f398730c7e963c98887427f1b` |
| Dual-suite | `sdd_sdlc_catalog_baseline` · `sdd_sdlc_catalog_target` |
| Transition | **additive** (CI-004: baseline stays GREEN after implement) |
| Mode | `strict` · isolation `worktree` · transition hint `auto` |
| ADR draft | [`docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md`](../../adr/0003-sdlc-canonical-assurance-catalog-draft.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../../contracts/assurance-runtime.md) |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](../canonical-assurance-catalog-v1.md) |
| Prompt-02 / 03 (consumed) | [`docs/sdd/typed-evidence.md`](../typed-evidence.md), [`docs/sdd/population-runtime.md`](../population-runtime.md) |
| Prompt-04 pattern (do not overwrite) | [`docs/sdd/iam-canonical-assurance-catalog.md`](../iam-canonical-assurance-catalog.md) |
| Spine / ISO law | [`docs/sdd/assurance-runtime-spine.md`](../assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](../iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **SDLC catalog slice** on run `sdd-625d28d3-3dbb1ba8da`. It does not replace Prompt 01 catalog infrastructure, Prompt 02 typed evidence, Prompt 03 population runtime, or the Prompt 04 IAM family. Prompts 01–04 have landed on the snapshot head; this slice consumes their loader, `EvidenceValue`, and population evaluator and **must not** invent a second copy.

Prior Prompt 05 SDD runs (`sdd-088983da-389f66a4fd`, `sdd-59e9991c-2b5e88f63c`, xylex-sdd-v2/v3/v5) specified or even implemented this family in isolated worktrees; **none applied product content to this snapshot**. Do not treat those run directories as landed catalog. Prefer this spec’s IDs over the `59e9991c` suite (`repository-visibility` / `reviewer-count`).

Tool-state dirty paths excluded from the snapshot (not product state): `docs/sdd/sdd-625d28d3-8b811037fc/abort.json`, `docs/sdd/sdd-sdd-625d28d3-53ba1d39ae/{abort,manifest,state}.json`, `docs/sdd/sdd-sdd-625d28d3-8b811037fc/{manifest,state}.json`. Admitted dirty semantic paths: none.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

A GitHub, GitLab, or Bitbucket collector must be able to populate the same evidence contracts and receive the same control results. This slice is provider-neutral and framework-neutral.

---

## 1. Problem / user-visible goal

Organizations need to assess repository inventory, branch protection, review policy, CI/CD permissions, deployment authorization, release integrity, and software-supply-chain hygiene using **provider-neutral** canonical controls.

On snapshot head `f6eb344cacefe44f398730c7e963c98887427f1b` the only SDLC-adjacent product content is:

- a **thin ISO 27001 pack sliver** (`source.branch-protection`, `source.required-review`, `source.code-ownership`, `source.security-scanning`, `source.commit-signing`) wired to **GitHub-shaped** evidence types (`source.branch.protection`, `source.branch.required_reviews`, `source.codeowners.present`, `source.security.secret_scanning.enabled`, `source.commit.signing`) as presence/hybrid checks;
- a **catalog-infrastructure fixture** (`control.source.protected-branch` / `evidence.source.protected-branch` / `test.source.protected-branch`) whose test is `op = "exists"` — an existence check, not a population assertion;
- a landed **IAM family** (`control.identity.*`) that does not cover repositories, CI, or releases;
- a GitHub collector that advertises `source.*` types and must **not** be expanded here.

There is no `control.source.default-branch-protection` family, no `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*` contracts, and no multi-repository population fixtures. A future GitLab or Bitbucket collector therefore has nowhere canonical to emit SDLC facts.

**User-visible goal:** a coherent SDLC catalog (~20–30 independently assessable controls) that evaluates realistic **repository / branch / deployment** populations from any future SCM/CI collector’s canonical evidence, produces deterministic explainable results (missing ≠ stale ≠ failure ≠ manual review ≠ approved exception), and passes catalog validation plus full workspace verification.

This slice does **not** claim ISO/SOC 2 coverage. Framework remapping is Prompt 12. This slice does **not** expand the GitHub collector (Prompt 09). This slice does **not** implement the Prompt 06 vulnerability family.

---

## 2. Dependencies and fail-closed blockers

| Prompt | Owns | On snapshot head `f6eb344…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules | **Landed.** Identity + fixture.example listed in `manifest.toml`. | Add SDLC TOML + manifest lines. Do not invent a second loader/validator/digest. Do not delete fixture.example IDs. |
| 02 typed evidence | Typed `EvidenceValue`, seal rules | **Landed.** | Declare required fact *names* and semantic types. No second value enum. |
| 03 population runtime | Subject populations, `AllSubjects` / `CoverageAtLeast` / `NoneSubjects`, missing/stale/fail split | **Landed.** Identity inventory special-case + generic `inventory.subject` / `inventory.complete`. | Declare population-based tests. **Do not locally reimplement coverage math. Do not add `resolve_repository_inventory`. Do not change generic population semantics.** |
| 04 IAM family | `control.identity.*` | **Landed.** | Leave identity files and `sdd_iam_catalog_target` green. |
| 06 vulnerability | `control.vulnerability.*` | **Not landed** (sibling ADR draft only). | Do not implement finding/SLA/coverage family. Scanning-*enabled* belongs here; finding-as-evidence belongs in Prompt 06. |

Rebase rule: adapt SDLC content to the landed contracts. Prefer existing `CanonicalCatalog`, `EvidenceValue`, and `evaluate_coverage` over extending this slice’s scope.

---

## 3. Current behavior (characterization on snapshot head `f6eb344cacefe44f398730c7e963c98887427f1b`)

Inspected: `catalog/canonical/v1/`, `crates/weeping-angel-canonical-catalog`, `weeping-angel-control-test` (`population.rs`), `weeping-angel-collector/src/github/descriptor.rs`, `frameworks/iso-27001/2022/{metadata,mappings}.toml`, `tests/sdd/{iam,canonical,iso27001}_*`, `Cargo.toml` `[[test]]` table, Prompt 05, IAM SSOT, ADR draft at `docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md`.

### 3.1 Canonical catalog tree

`catalog/canonical/v1/manifest.toml` lists only:

```text
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests    = ["tests/fixture.example.toml", "tests/identity.toml"]
```

No `controls/source.toml`, `cicd.toml`, `release.toml`, or `supply-chain.toml`. No `evidence.repository.*` family. No `fixtures/assurance/canonical/v1/sdlc/`.

Pinned infrastructure fixture (CAT-015; **must survive**):

| Kind | Id | Expression |
| --- | --- | --- |
| control | `control.source.protected-branch` | domains `secureDevelopment` |
| evidence | `evidence.source.protected-branch` | declared envelope type `source.branch.protection` |
| test | `test.source.protected-branch` | `op = "exists"` on `evidence.source.protected-branch` |

IAM family (23 `control.identity.*`) is present and must remain. IAM tests already use `op = "all-subjects"` / `"coverage-at-least"` / `"none-subjects"` / `"manual-review"` with `[[test.subjects]] kind` values that parse via `SubjectKind::parse_name`.

Catalog validator already rejects reserved provider/framework **ID segments** (`github`, `gitlab`, `bitbucket`, `iso27001`, `soc2`, …). IDs must be lowercase dotted `kind.family.slug` with `-` allowed; `_` is malformed (`validate_id`). `azure-devops` is forbidden in SDLC content even if the validator only lists the `azure` segment. Extra files under `catalog/canonical/v1/` that are not listed in the manifest fail `CanonicalCatalog::validate` (`Unlisted`).

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

`sdd_iso27001_assurance_target` freezes prefixes `source.` and those pack ids. This slice **must not** retarget mappings or rename pack ids (Prompt 12).

Those tests are presence/hybrid checks, not “all non-archived in-scope repositories have a protected default branch.”

### 3.3 GitHub collector (do not expand)

`GITHUB_EVIDENCE_TYPES` in `crates/weeping-angel-collector/src/github/descriptor.rs` advertises GitHub-native `source.*` names (repository exists/visibility/archived, default branch, branch protection / reviews / status checks / force-push / deletion, CODEOWNERS, admin/collaborator, Dependabot / secret scanning / code scanning, workflow permissions, rulesets, commit signing). Subject types: `repository`, `branch`.

Catalog tests must **not** import or assert against `GITHUB_EVIDENCE_TYPES` as the SDLC contract. Scanner engines (`src/engines/*`, `src/depcheck/*`) are not evidence contracts. Scanner findings may later populate canonical evidence; tests depend only on those contracts.

### 3.4 Population runtime (consume, do not change)

`resolve_population` order (`crates/weeping-angel-control-test/src/population.rs`):

1. explicit `EvidenceSet` population;
2. selector with explicit IDs (authoritative);
3. identity inventory (`evidence.identity.inventory` + privileged / service-account);
4. generic `inventory.subject` + `inventory.complete` (`authoritative`);
5. otherwise infer subject ids from the observation type with **Unknown** completeness.

Strong all-subjects / 100% coverage **refuses** `Effective` when completeness is Partial (`InsufficientEvidence`) or Unknown (`Inconclusive`). Technical / missing / stale partitions are already distinct from `Ineffective`. Approved unexpired subject-scoped IR `Exception` records skip subjects (`ExceptionApproved` path via excepted partition / existing evaluator).

There is **no** repository-inventory special case. This slice must not add one. Repository populations use generic `inventory.subject` (`kind=repository` or `deployment`) plus `inventory.complete` and/or `EvidenceSet::set_population`.

### 3.5 Test harness topology (Rust)

Language: Rust. Package manager: Cargo. Test framework: `cargo test` (libtest). Root `Cargo.toml` registers SDD suites with explicit `[[test]]` entries. `tests/sdd/*.rs` is **not** auto-discovered and is **not** executable via `cargo test --test <name>` until registered.

On this snapshot, registered SDD targets include `sdd_{assurance_runtime,iso27001_assurance,population_runtime,typed_evidence,iam_catalog,canonical_assurance_catalog}_{baseline,target}` and `sdd_compliance_ir_target`. There is **no** `sdd_sdlc_catalog_*` target and **no** `tests/sdd/sdlc_catalog.*.rs` on the snapshot tree.

**Harness law:** BaselineAuthor / TargetAuthor write only the dedicated test files. They **must not** edit `Cargo.toml`. Registration of:

```toml
[[test]]
name = "sdd_sdlc_catalog_baseline"
path = "tests/sdd/sdlc_catalog.baseline.rs"

[[test]]
name = "sdd_sdlc_catalog_target"
path = "tests/sdd/sdlc_catalog.target.rs"
```

is an **orchestrator / implement-harness** step (same allowlist as existing SDD `[[test]]` rows). DiscoverSpec does not edit `Cargo.toml`.

### 3.6 What “SDLC assessment” means today

A caller can compile the ISO pack and run `test.source.branch-protection`, which requires **some** `source.branch.protection` envelope to exist. It cannot:

- require protection on every non-archived in-scope repository;
- distinguish missing inventory from one unprotected repo from stale scan evidence;
- evaluate force-push, deletion, reviewer count, review ownership, workflow write permissions, protected environments, provenance, or lockfile integrity as population predicates;
- accept GitLab/Bitbucket-shaped facts without teaching tests those providers.

The baseline suite therefore characterizes **invariants that remain true after this slice** (ISO sliver, fixture exists-check, IAM family, collector not remapped, no `resolve_repository_inventory`), not “absence of every `control.source.*` string” (that would break on the fixture and would fail after additive implement).

### 3.7 TargetAuthor hygiene (prior v3/v5 abort)

Prior Prompt 05 target suites aborted protocol gates:

- Do **not** read the target file’s own source and assert it does not contain `#[ignore` (self-referential; unsatisfiable).
- Do **not** pair a self-read of the test file with a negated `.contains("literal")` (I4a false-positive). Hyphen-id rules belong on **loaded catalog IDs** (`CanonicalCatalog` product state), not on the test source text.
- Do **not** write `#[ignore]` on target tests.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

SDLC domain content lands in the Prompt 01 catalog tree:

```text
catalog/canonical/v1/
  manifest.toml                         # list new files; keep fixture.example + identity
  controls/source.toml                  # control.source.*
  controls/cicd.toml                    # control.cicd.*
  controls/release.toml                 # control.release.*
  controls/supply-chain.toml            # control.supply-chain.*
  evidence/repository.toml              # evidence.repository.*
  evidence/cicd.toml                    # evidence.cicd.*
  evidence/deployment.toml              # evidence.deployment.*
  evidence/release.toml                 # evidence.release.*
  evidence/supply-chain.toml            # evidence.supply-chain.*
  tests/source.toml
  tests/cicd.toml
  tests/release.toml
  tests/supply-chain.toml
```

A single `sdlc.toml` per section is acceptable if every id below is present and listed in `[files]`. Do **not** add these controls to `frameworks/iso-27001/2022/metadata.toml`. Do **not** remove or rewrite `fixture.example.toml` / `identity.toml`.

Deterministic fixtures (IAM layout: `fixtures/assurance/canonical/v1/<family>/<name>/evidence.json` + optional exception):

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

Catalog TOML `[[evidence]]` rows may declare a short `evidence_type` (IAM pattern: `identity.mfa-status`). Fixture JSON and evaluator selectors must use the **catalog evidence id** (`evidence.repository.*`, same as IAM fixtures using `evidence.identity.*`).

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
| `evidence.repository.branch-protection` | `subject_id`, `protected` (bool), `force_push_allowed` (bool), `deletion_allowed` (bool), `admin_bypass_allowed` (bool) | “branch protection effective” |
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

**Generic population envelopes (not new catalog ids):** fixtures / the target harness MUST also establish Prompt 03 authoritative completeness using existing types `inventory.subject` (`kind=repository` or `deployment`) and `inventory.complete` (`authoritative=true`), and/or `EvidenceSet::set_population`. Do not treat inferred Unknown populations as `Effective`.

IAM fixtures store bools as strings (`"true"`). SDLC fixtures may do the same (`with_fact` string-compat) or use typed `EvidenceValue`; either is valid as long as evaluator comparisons succeed.

### 4.5 Tests (population-based, not existence checks)

Required reusable tests (Prompt 05 examples + extras so no control is untested):

```text
test.source.default-branches-protected
test.source.force-push-restricted
test.source.reviews-required
test.source.minimum-reviewer-count
test.source.secret-scanning-enabled
test.cicd.workflow-permissions-minimized
test.release.environments-protected
test.source.dependency-scanning-current
test.supply-chain.artifacts-have-integrity
```

Semantics (authoritative intent; exact `TestExpr` spelling follows Prompt 03 / IAM `all-subjects` / `coverage-at-least`):

| Test | Population | Pass | Fail | Missing | Stale |
| --- | --- | --- | --- | --- | --- |
| `default-branches-protected` | all **non-archived** in-scope repositories | every subject `protected=true` | ≥1 in-scope repo `protected=false` | inventory unknown **or** known repo lacks branch-protection | stale protection / inventory |
| `force-push-restricted` | protected default branches / their repos | none have `force_push_allowed=true` | unauthorized force-push still allowed | missing protection envelope | stale |
| `reviews-required` | production / in-scope repos (use `criticality=production` or `in_scope`) | `reviews_required=true` | review not required | missing review-policy | stale |
| `minimum-reviewer-count` | same as reviews | `meets_review_threshold=true` (threshold is a fixture/policy integer fact, not a hardcoded GitHub “2”) | count below threshold | missing count | stale |
| `secret-scanning-enabled` | in-scope repos where `applicable=true` (or all in-scope if applicability omitted) | `secret_scanning_enabled=true` | scanning disabled | missing scan evidence → **InsufficientEvidence**, never `Ineffective` | stale |
| `workflow-permissions-minimized` | in-scope repos with CI | `permissions_minimized=true` and not `default_write=true` | overbroad write | missing workflow-permissions | stale |
| `environments-protected` | production deployments | `authorization_required=true` / `protected=true` | prod env unprotected | missing environment-protection | stale |
| `dependency-scanning-current` | critical in-scope repos | enabled **and** `scanned_at` within freshness | enabled=false | missing scan envelope → InsufficientEvidence | `scanned_at` outside window → `StaleEvidence` |
| `artifacts-have-integrity` | repos/releases that produce artifacts (where required) | `integrity_evidence_present=true` (and provenance when bound) | integrity missing while required | missing artifact-integrity | stale |

**Forbidden encoding:** `Exists(evidence.repository.branch-protection)` as the body of `test.source.default-branches-protected`. Existence of some protection fact is not protection on the population. The infrastructure fixture may keep `exists` **only** on `test.source.protected-branch`.

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

- Loader / validate / digest: Prompt 01 `CanonicalCatalog`. SDLC files must pass `validate`.
- Typed facts: Prompt 02 `EvidenceValue`. Prefer `with_value`; `with_fact` remains string-compat (IAM fixtures still store `"true"` strings).
- Population evaluation: Prompt 03 `evaluate` / `evaluate_coverage`. SDLC tests are **declarations**. Do not implement `AllSubjects` here. Do not add repository-specific resolver functions.
- Exception: reuse IR `Exception` + `Effectiveness::ExceptionApproved`.
- Subject kinds: existing IR only.
- ISO pack, GitHub collector, scanner engines, framework compiler, generic `TestExpr` semantics: **untouched**.
- Catalog tests depend only on evidence contracts, **not** on `GITHUB_EVIDENCE_TYPES`, SARIF adapters, or `src/engines/*`.

### 4.9 Dual-suite protocol

Follow the existing root `[[test]]` + `tests/sdd/` pattern. **Additive:** baseline encodes invariants that remain true after implement (CI-004). Do **not** write baseline as “no `control.source.default-branch-protection` exists” — that would be replacement and would fail post-implement. Do **not** assert that `Cargo.toml` omits `sdd_sdlc_catalog_*` (harness registration would then fail the baseline).

| Suite | Path | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/sdlc_catalog.baseline.rs` · `sdd_sdlc_catalog_baseline` | GREEN on current tree **and** after implement: ISO sliver unchanged; fixture exists-check preserved; IAM family present; collector still GitHub `source.*`; no provider tokens in existing catalog IDs; population semantics unchanged (no `resolve_repository_inventory`); framework crate still catalog-blind. |
| Target | `tests/sdd/sdlc_catalog.target.rs` · `sdd_sdlc_catalog_target` | RED until SDLC content lands; then **GREEN** — CI gate (SDLC-001…016). Assert against loaded catalog/fixtures, not self-read negated substrings. |

Suggested target assertion clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| SDLC-001 | Catalog tree / loader loads SDLC files offline; `CanonicalCatalog::validate` succeeds |
| SDLC-002 | Digest remains deterministic after adding SDLC files |
| SDLC-003 | All 26 `control.{source,cicd,release,supply-chain}.*` ids present (20–30 independently assessable); fixture `control.source.protected-branch` still present |
| SDLC-004 | Required `evidence.repository.*` / cicd / deployment / release / supply-chain types declared; no orphans |
| SDLC-005 | Required `test.source.*` / cicd / release / supply-chain ids declared and referenced |
| SDLC-006 | Validator rejects provider tokens (`github`, `gitlab`, `bitbucket`) in new SDLC ids |
| SDLC-007 | Validator / file text rejects `iso27001` / `soc2` / `nis2` in SDLC catalog files |
| SDLC-008 | No SDLC control lives in the ISO pack as `control.source.*`; ISO pack ids and mappings unchanged |
| SDLC-009 | `test.source.default-branches-protected` is population-based (fails `unprotected-default-branch`; does not pass on a single protection envelope) |
| SDLC-010 | Missing vs stale vs fail vs manual vs exception distinguished on the seven fixtures |
| SDLC-011 | Partial coverage cannot yield Effective on all-subjects tests |
| SDLC-012 | Approved unexpired exception → `ExceptionApproved` / excepted for that subject |
| SDLC-013 | Authorization / authority-separation / security-review / secure-development-policy marked Hybrid or Manual |
| SDLC-014 | Catalog tests do not reference `GITHUB_EVIDENCE_TYPES`, scanner engines, or GitHub-native type ids |
| SDLC-015 | Identity family, CAT fixture IDs, and Prompt 03 population.rs identity/generic resolution remain; no `resolve_repository_inventory` |
| SDLC-016 | `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, `sdd_canonical_assurance_catalog_target` stay green |

### 4.10 Documentation after implement

Later docs pass (not this I1 write): durable SSOT copy under `docs/sdd/sdlc-canonical-assurance-catalog.md`, accept the ADR draft, pointer on `docs/contracts/assurance-runtime.md`. Do not overwrite Prompt 01 / 04 SSOTs. No GitHub collector expansion or ISO remap is claimed.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase. IDs `AC-1`…`AC-16` match this list in order.

1. **AC-1.** Dual-suite `sdd_sdlc_catalog_baseline` + `sdd_sdlc_catalog_target` is registered in root `Cargo.toml` (harness/implement, not BaselineAuthor) like existing SDD tests; paths `tests/sdd/sdlc_catalog.baseline.rs` and `tests/sdd/sdlc_catalog.target.rs`.
2. **AC-2.** After implement: target GREEN; baseline still GREEN (additive invariants); `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
3. **AC-3.** Twenty-six `control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*` controls exist with stable ids, domains, evidence requirements, test refs, and honest automation class; independently assessable count stays in 20–30; `control.source.protected-branch` fixture remains.
4. **AC-4.** Evidence types in §4.4 are declared as facts, not conclusions; IDs are provider-neutral (`evidence.repository.*` etc., not `evidence.github.*`).
5. **AC-5.** Tests include at least the nine Prompt-05 example ids and evaluate **populations** (all non-archived in-scope default branches protected), not existence of one envelope.
6. **AC-6.** Evaluator outcomes distinguish missing data, stale data, actual failure, manual review, and approved exception on the seven named fixtures; missing scan evidence is not a technical failure.
7. **AC-7.** Release authorization, authority-separation, security-review, and secure-development-policy are Hybrid or Manual; they cannot auto-pass without attestation.
8. **AC-8.** Catalog validator accepts the SDLC slice: no duplicate/orphan/dangling ids, no provider names, no ISO/SOC2/NIS2 references in canonical SDLC content.
9. **AC-9.** ISO pack control ids and mappings are unchanged; `sdd_iso27001_assurance_target` remains green.
10. **AC-10.** GitHub collector is not expanded; SDLC catalog tests do not couple to `GITHUB_EVIDENCE_TYPES` or scanner internals.
11. **AC-11.** No second `CanonicalCatalog` loader, no second `EvidenceValue` enum, no `resolve_repository_inventory` / `SdlcPopulation` fork. Prompt 03 coverage is consumed as-is (explicit population and/or `inventory.subject` + `inventory.complete`).
12. **AC-12.** Approved-exception fixture uses existing Exception IR; expired/revoked exceptions do not pass.
13. **AC-13.** IAM family and catalog-infrastructure fixture IDs remain; `sdd_iam_catalog_target` and `sdd_canonical_assurance_catalog_target` stay green.
14. **AC-14.** Prompt 01 SSOT `docs/sdd/canonical-assurance-catalog-v1.md` and Prompt 04 IAM SSOT are not overwritten by this slice.
15. **AC-15.** A GitLab or Bitbucket collector could populate the same evidence contracts without catalog changes (no GitHub-native object names in canonical IDs).
16. **AC-16.** `CanonicalCatalog::validate` and workspace checks pass with the SDLC files listed in `manifest.toml`.

---

## 6. Out of scope

- Expanding or remapping the GitHub collector (Prompt 09).
- Implementing GitLab, Bitbucket, Azure DevOps, or Gitea collectors.
- Remapping ISO 27001 (or SOC 2 / NIS2) onto `control.source.*` (Prompt 12).
- Redesign of `CanonicalCatalog` loader/validator/digest (Prompt 01).
- Redesign of typed evidence (Prompt 02).
- Changing generic population semantics or adding `resolve_repository_inventory` (Prompt 03).
- Rewriting ISO `metadata.toml` / `mappings.toml` (`source.branch-protection` stays until Prompt 12).
- Removing or changing `control.source.protected-branch` fixture IDs.
- Changing IAM catalog content.
- Implementing Prompt 06 vulnerability / finding / SLA catalog content.
- Scanner engine / depcheck / SARIF adapter changes.
- New `SubjectKind` variants.
- Certification, “compliant”, or audit-passed language.
- Infrastructure / governance catalog families (Prompts 07–08).
- BaselineAuthor editing `Cargo.toml`.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Fixture `control.source.protected-branch` collides with the new family | Distinct population id `control.source.default-branch-protection`; CAT-015 fixture stays exists-only. |
| Baseline asserts “no SDLC catalog” and fails after additive implement | Baseline encodes surviving invariants only (CI-004). |
| `tests/sdd/*.rs` unregistered → `cargo test --test sdd_sdlc_catalog_*` fails | AC-1: harness/implement adds `[[test]]`; BaselineAuthor does not touch `Cargo.toml`. |
| Implementer adds `resolve_repository_inventory` | AC-11 + SDLC-015; use `inventory.subject` / `inventory.complete` / explicit population. |
| Existence checks sneak in as SDLC tests | SDLC-009: unprotected-default-branch must fail; a lone protection envelope must not pass. |
| Missing scan evidence coded as Ineffective | SDLC-010 + AC-6: missing → InsufficientEvidence. |
| ISO pack rewritten | AC-9; do not touch `frameworks/iso-27001/2022` source rows. |
| Provider names leak into IDs or fixture types | Validator + SDLC-006/007/014. |
| Hybrid controls auto-pass from one technical fact | Honest automation class; AC-7. |
| Target suite self-reads `#[ignore` / `contains('_')` and trips I4a | Assert loaded catalog IDs; no self-read negated contains. |
| IAM / CAT target suites regress | AC-13; do not edit those files except dual-suite registration in `Cargo.toml`. |
| Unknown completeness makes healthy-org Inconclusive | Fixtures must mark authoritative completeness via existing Prompt 03 paths. |

---

## 8. Dual-suite and SDD protocol (implement phase)

```text
Spec (this file) → Baseline GREEN on CURRENT code → Target RED on CURRENT code
  → Implement SDLC catalog content only → Docs/ADR finalize if needed
  → Target GREEN → Baseline still GREEN (additive) → Target still GREEN
```

Fail-closed if: baseline cannot go green on current invariants; target cannot go red for the **right** reason (missing SDLC family / population fixtures); or target never greens within `max_iters=2`.

Do not modify production source in I1. Dedicated test paths only:

```text
tests/sdd/sdlc_catalog.baseline.rs
tests/sdd/sdlc_catalog.target.rs
```

Commands (after harness `[[test]]` registration):

```text
cargo test --test sdd_sdlc_catalog_baseline -- --nocapture
cargo test --test sdd_sdlc_catalog_target -- --nocapture
cargo test --workspace --features demo
```

`baseline_post_expected = pass` (additive / CI-004).

---

## 9. ADR

Architecture / public-contract decision: SDLC content is a **canonical catalog family** (`control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*`) consumed later by framework mappings, not an ISO-pack extension and not a GitHub-specific catalog.

Draft: [`docs/adr/0003-sdlc-canonical-assurance-catalog-draft.md`](../../adr/0003-sdlc-canonical-assurance-catalog-draft.md). Accept after implement. Do not grow the ISO pack `source.*` list, replace the exists-only fixture, add `resolve_repository_inventory`, or encode GitHub-native object names as catalog IDs.

---

## 10. Planning SHA record

```text
planning_sha = f6eb344cacefe44f398730c7e963c98887427f1b
tree         = 35de6aa30ef5b7fc0019a4f99841306ea3af406b
commit       = f6eb344cacefe44f398730c7e963c98887427f1b
branch       = main
run          = sdd-625d28d3-3dbb1ba8da
note         = prompts 01–04 landed (catalog fixture + IAM family + typed evidence +
               population runtime); Prompt 05 markdown present; no SDLC catalog files;
               ISO source sliver + exists-only fixture only; Cargo.toml has no
               sdd_sdlc_catalog_* [[test]] rows; prior Prompt 05 SDD runs did not
               land product content on this snapshot
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
- `tests/sdd/iam_catalog.target.rs`
- `tests/sdd/canonical_assurance_catalog.target.rs`
- `tests/sdd/iso27001_assurance.target.rs`
- `Cargo.toml`
- `docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md`

**Symbols:** `CanonicalCatalog::{load,validate,digest,stats}`, `CATALOG_SCHEMA`, `TestExpr::{AllSubjects,CoverageAtLeast,NoneSubjects,ManualReview,Exists}`, `evaluate` / `evaluate_coverage`, `Population` / `PopulationCompleteness` / `PopulationEvaluation`, `Effectiveness`, `SubjectKind::{Repository,Branch,Deployment,Organization}`, `Exception` / `ExceptionStatus`, `EvidenceValue`, `GITHUB_EVIDENCE_TYPES`.

**Live seams:** catalog TOML + manifest listing; Prompt 03 population completeness; ISO pack compile; GitHub collector advertisement (read-only); IAM and CAT dual-suites; root `[[test]]` harness.

**Tooling:** language Rust; package manager Cargo (pnpm only for `apps/docs`); test framework `cargo test`. Integration tests under `tests/sdd/` require explicit `[[test]]`.
