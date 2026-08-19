# SDD: Reference-Grade GitHub Assurance Collector

| Field | Value |
| --- | --- |
| Status | **Target RED** — durable SSOT; baseline GREEN (30); `sdd_github_collector_target` §4.11 authored and RED on current collector; **no product feature code** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 09 — first reference-grade provider collector |
| Source prompt | [`docs/prompts/canonical-assurance-v1/09-github-collector.md`](../prompts/canonical-assurance-v1/09-github-collector.md) |
| Planning / characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`, 2026-08-19) |
| Dual-suite | **Registered** in root `Cargo.toml`: `sdd_github_collector_baseline` → `tests/sdd/github_collector.baseline.rs` (30 GREEN); `sdd_github_collector_target` → `tests/sdd/github_collector.target.rs` (`ghc_000`–`ghc_024` RED). Implement owned GitHub paths next — do not weaken these tests. |
| Draft ADR | [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Consumes | Prompts 01–08 contracts, especially typed evidence, population completeness, IAM `evidence.identity.*`, SDLC I1 freeze `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Prompt-02 / 03 / 04 (consumed) | [`typed-evidence.md`](typed-evidence.md), [`population-runtime.md`](population-runtime.md), [`iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| Prompt-05 contracts (not landed as catalog TOML) | Prompt [`05-sdlc-catalog.md`](../prompts/canonical-assurance-v1/05-sdlc-catalog.md), draft ADR [`0003-sdlc-canonical-assurance-catalog-draft.md`](../adr/0003-sdlc-canonical-assurance-catalog-draft.md), I1 freeze [`sdd-sdd-088983da-389f66a4fd/spec.md`](sdd-sdd-088983da-389f66a4fd/spec.md) |
| Spine / ISO law | [`assurance-runtime-spine.md`](assurance-runtime-spine.md), [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **GitHub collector slice**. It does not replace catalog-infrastructure, typed-evidence, population, or IAM SSOTs. Prompt 05 SDLC catalog TOML is **not** landed; this slice **emits** the evidence contracts those prompts already named. It must **not** invent catalog IDs, write `catalog/canonical/v1` Prompt 05–08 TOML, or calculate control effectiveness.

This file supersedes the interrupted Scope-only planning attempt. Characterization was re-read against workspace HEAD `e430980c…` and the uncommitted baseline suite (`ghc_b001`–`ghc_b030`).

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

Never `GitHub -> ISO check`. The collector never knows ISO/SOC2/NIS2/DORA requirement IDs.

---

## 1. Problem / user-visible goal

Organizations need a GitHub collector that can populate the **same provider-neutral evidence contracts** a future GitLab or Bitbucket collector would populate, so 25–40 canonical controls evaluate identically regardless of SCM.

On SHA `e430980c…` the existing `GitHubCollector` is an ISO-sliver prototype: it walks `repo:owner/name` scope labels, emits GitHub-shaped `source.repository.*` / `source.branch.*` **string** facts, advertises types and pagination it does not collect, aborts the whole run on HTTP 403, and records an empty `CollectionRun`. Six feature modules are stubs. A 403 on branch protection is not a boolean “unprotected”; a missing page is not a complete inventory; a token must never appear in facts or diagnostics.

**User-visible goal:** GitHub is the first reference-grade provider collector for the canonical assurance runtime. It emits **canonical evidence only**, fails closed on permission holes, states population completeness honestly, redacts credential material, and records a real collection run. Another provider emitting the same contracts receives the same test results.

---

## 2. Dependencies and fail-closed blockers

| Prompt / contract | Owns | This slice may | Must not |
| --- | --- | --- | --- |
| 01 catalog | `catalog/canonical/v1/`, loader, IDs | Consume existing `fixture.example` + `identity.toml` | Invent catalog IDs; write Prompt 05/06/07/08 TOML; redesign the loader |
| 02 typed evidence | `EvidenceValue`, seal, redact | Emit typed facts via `with_value`; reuse `redact` / credential-key reject | Fork a second value enum; persist tokens; change digest law / `EvidenceProvenance` |
| 03 population | `inventory.subject` + `inventory.complete`, `AllSubjects` | Emit those generic envelopes when inventory pagination is complete | Add `resolve_repository_inventory`; claim `authoritative` on partial pages |
| 04 IAM | `evidence.identity.*` | Emit `privileged-membership` and `external-access` (and inventory/role/service-account facts **only** when GitHub can observe them) | Become an IdP; emit MFA/last-active/termination unless the API actually yields them |
| 05 SDLC (not landed) | `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` | Emit those **type strings and fact names** from the Prompt 05 / I1 freeze | Wait for SDLC TOML; invent `evidence.github.*` required by tests |
| 06 / 07 / 08 | vuln / infra / governance catalogs | Ignore | Touch their files or SDD suites |
| ADR 0002 ISO sliver | `source.*` types, GH-007 / GH-009 / GH-012 | Keep ISO **needles** in collector sources; do not retarget pack mappings | Rewrite ISO pack, `sdd_iso27001_assurance_*`, or Prompt 12 remap |
| IAM-015 | `GITHUB_EVIDENCE_TYPES` has no `evidence.identity.*` | Advertise identity types on the **descriptor** without putting them on that const | Edit `tests/sdd/iam_catalog.*` |
| Shared collector types | `CollectorDescriptor` has **no** `failure_behavior` field | Document / advertise failure behavior in GitHub-owned sources (const, comments, descriptor docs) | Redesign `CollectorDescriptor` / `CollectorCapabilities` unless a later implement proves it is strictly required |

Rebase rule: if Prompt 05 lands `catalog/canonical/v1/evidence/repository.toml` (etc.) before implement, emit the **landed** catalog evidence ids and fact names. Prefer adapting the collector to that file over inventing a parallel mapping.

---

## 3. Current behavior (baseline on `e430980c…`)

Characterized against workspace HEAD `e430980c0d27a8138a153d49b62ddf3c57827891`. Encoded by `sdd_github_collector_baseline` (`ghc_b001`–`ghc_b030`). The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is explicitly superseded.

### 3.1 Scope and collect path

[`crates/weeping-angel-collector/src/github/mod.rs`](../../crates/weeping-angel-collector/src/github/mod.rs):

- `GitHubCollector::collect` splits `CollectorScope::as_label()` on commas.
- Each label must parse as `repo:owner/name` (or `owner/name`). Anything else (`org:acme`, topic, selector) is `OutOfScope` (`ghc_b001`).
- A bare `owner/name` allow-list entry is rewritten to asset `repo:owner/name` and then fails `scope.allows` (`ghc_b002`).
- There is **no** org / user / topic / archived-exclusion selector. Archived repos in scope are collected like any other repo (`ghc_b006`).
- There is **no** repository inventory listing (`GET /orgs/{org}/repos` or equivalent). Sources contain none of `/orgs/`, `exclude_archived`, `inventory.subject`, `inventory.complete` (`ghc_b003`).

### 3.2 What is actually collected

For each in-scope repo:

1. `GET /repos/{owner}/{name}` → [`normalize.rs`](../../crates/weeping-angel-collector/src/github/normalize.rs) emits four envelopes (`ghc_b004`):
   - `source.repository.exists` `{exists: "true"}`
   - `source.repository.visibility` `{visibility: public|private|internal|unknown}` — falls back to `private` flag, then `"unknown"` (`ghc_b005`)
   - `source.default_branch` `{name}` when present (omitted when absent)
   - `source.repository.archived` `{archived: "true"|"false"}`
2. `GET /repos/{owner}/{name}/branches/main/protection` — **hardcoded `main`**, not the repo’s `default_branch` (`ghc_b007`). A `develop` default branch with protection only on `develop` still reads `main`.
   - **403 → abort the entire `collect`** with `PermissionDenied` (not a diagnostic envelope) (`ghc_b015`).
   - **404 →** `source.branch.protection` `{enabled: "false"}` (“default branch has no protection rule”) (`ghc_b008`).
   - **200 →** [`protection.rs`](../../crates/weeping-angel-collector/src/github/protection.rs) emits string facts (`ghc_b009`):
     - `source.branch.protection` `{enabled: "true"}`
     - `source.branch.required_reviews` `{count}` (string; default `"0"`)
     - `source.branch.force_push_protection` `{enabled}` — inverted from `allow_force_pushes.enabled` (missing pointer ⇒ treated as force-push **not** allowed ⇒ `"true"`)
     - `source.branch.deletion_protection` `{enabled}` — inverted from `allow_deletions.enabled`
     - `source.branch.required_status_checks` `{configured}` — `true` if the JSON pointer exists

Facts are stored with `with_fact` (`EvidenceValue::String`). Typed bools/integers are not used (`ghc_b004`, `ghc_b009`). Emitted types are `source.*` only — no `evidence.repository.*` / `evidence.identity.*` (`ghc_b010`).

Repo-level **403 also aborts the whole collect** (`ghc_b014`). A 403 on the second repo discards the first repo’s envelopes (`ghc_b016`). Repo 404 / 429 / 500 become `InsufficientEvidence` and also abort (`ghc_b017`). There is no per-subject continue.

Repo 403 / protection 403 use **fixed detail strings** (they do not copy the response body). Protection 401 goes through `handle_status` and also uses a static phrase (`ghc_b025`). `handle_status` itself **does** `redact(body)` for generic 403.

### 3.3 Stub modules

These files export only `pub const MODULE: &str` and are referenced solely so the compiler keeps the modules (`ghc_b011`):

| File | Const |
| --- | --- |
| `branches.rs` | `"branches"` |
| `collaborators.rs` | `"collaborators"` |
| `repositories.rs` | `"repositories"` |
| `rulesets.rs` | `"rulesets"` |
| `security.rs` | `"security"` |
| `workflows.rs` | `"workflows"` |

Nothing collects CODEOWNERS, admins, outside collaborators, deploy keys, webhooks, Dependabot, secret/code scanning, workflow permissions, rulesets, environments, or commit-signing.

### 3.4 Descriptor vs collected (advertised-vs-collected gap)

[`descriptor.rs`](../../crates/weeping-angel-collector/src/github/descriptor.rs) `GITHUB_EVIDENCE_TYPES` lists 19 `source.*` types, including types **never emitted** (`ghc_b012`):

`source.codeowners.present`, `source.admin.permissions`, `source.collaborator.permission`, `source.security.dependabot.enabled`, `source.security.secret_scanning.enabled`, `source.security.code_scanning.configured`, `source.workflow.permissions`, `source.workflow.review_requirement`, `source.ruleset.present`, `source.commit.signing`.

Capabilities: `pagination: true`, `point_in_time: true`, `worker_safe: true`. **`incremental` is false (default).** There is no pagination implementation (`per_page` / `Link` / `page=` absent from `client.rs`) and no incremental cursor. `provider_family` is `"source-control"`. Subject types: `repository`, `branch` only. Required permissions: `contents:read`, `administration:read`, `metadata:read`.

`CollectorDescriptor` (shared `lib.rs`) has **no** `failure_behavior` field. Serialized descriptor has neither `failureBehavior` nor `failure_behavior` (`ghc_b013`). Do **not** add that field to the shared type unless implement later proves it is strictly required — advertise failure behavior in GitHub-owned sources instead.

ISO GH-012 ([`tests/sdd/iso27001_assurance.target.rs`](../../tests/sdd/iso27001_assurance.target.rs)) keeps a **local** copy of that `source.*` list and asserts each string appears in collector crate sources. IAM-015 imports crate `GITHUB_EVIDENCE_TYPES` and asserts **none** start with `evidence.identity.`.

### 3.5 Client / transport

[`client.rs`](../../crates/weeping-angel-collector/src/github/client.rs): fixture map `(path, status, body, retry_after)`. Match is `path.starts_with(fixture)` **or** exact; **first match wins** (`ghc_b020` — a `/repos/acme/app` fixture steals `/repos/acme/app/branches/main/protection`). No live HTTP. No token → **401** JSON. Token present but no fixture → `ClientError::Transport("no fixture and no live transport for {path}")` (`ghc_b019`). `authorization_header()` returns `"Bearer [redacted]"` and never the raw token (`ghc_b021`). `get` calls `redact` on that header. 429 bodies are returned; **`collect` does not retry** (`ghc_b018`). `backoff` / `sleep_retry_after` exist as unused helpers (cap 32s). Collector crate has **no** `reqwest` / `octocrab` dependency.

No token + no fixture → `PermissionDenied` unauthorized (`ghc_b029`). Token + no fixture → `InsufficientEvidence` transport, token not in the diagnostic (`ghc_b030`).

### 3.6 CollectionRun

`collect_batch` wraps `CollectionRun::new("collector.github", &self.version)` and returns `errors: Vec::new()`. `CollectionRun::new` leaves (`ghc_b022`):

- `completed_at = None`
- `scope = ""`
- `status = "started"`
- `evidence_count = 0`
- `error_count = 0`
- `configuration_digest = ""`

Envelope `collection_run_id` is a **deterministic digest of provenance**, not this run’s `run_id`. `collect_batch` on a collect error returns `Err` and produces **no** run (`ghc_b023`).

### 3.7 Security already present (keep)

[`weeping_angel_evidence::redact`](../../crates/weeping-angel-evidence/src/lib.rs) folds `Bearer `, `token=`, `ghp_`, `gho_`, `github_pat_`. It does **not** currently fold `ghs_` (`ghc_b024`). Seal rejects credential-shaped **fact keys**. ISO GH-007: 403 is `PermissionDenied` / `InsufficientEvidence`, not `403 => false`. ISO GH-009: collector sources must not contain `ghp_` or `GITHUB_TOKEN=` (`ghc_b026`). GitHub module sources do not mention ISO/SOC2/NIS2/DORA, `Effective`/`Ineffective`, or `evidence.github.` (`ghc_b027`).

### 3.8 Catalog / fixtures today

`catalog/canonical/v1` has `fixture.example` + IAM `identity.toml` only. No `evidence.repository.*` catalog rows. No `fixtures/assurance/canonical/v1/github/` and no `fixtures/collectors/github/` (`ghc_b028`). Identity goldens live under `fixtures/assurance/canonical/v1/identity/` and are **owned by Prompt 04** — do not rewrite them.

### 3.9 What “GitHub assessment” means today

A caller can collect one or more explicitly named repos and get ISO-sliver `source.*` string facts for existence/visibility/default-branch/archive plus protection on **`main`**. They cannot:

- inventory an org authoritatively (or say the inventory is partial);
- protect the **default** branch when it is not `main`;
- observe scanning, workflows, environments, admins, outside collaborators, deploy keys, or CODEOWNERS;
- continue after a permission hole;
- feed Prompt 03/04/05 population tests with `evidence.repository.*` / `evidence.identity.privileged-membership`;
- distinguish “protection 403” from “protection absent”.

The baseline suite therefore characterizes **source.\* emit, stubs, advertised-vs-collected gap, abort-on-403, empty CollectionRun, and existing redact/ghp_ guards** — not a working canonical provider collector.

---

## 4. Desired behavior (after implement)

### 4.1 Placement and ownership

Own **only**:

```text
crates/weeping-angel-collector/src/github/**
fixtures/assurance/canonical/v1/github/**     # preferred
  # or fixtures/collectors/github/** if a later convention lands first
tests/sdd/github_collector.baseline.rs
tests/sdd/github_collector.target.rs
docs/sdd/github-collector.md
docs/sdd/sdd-github-collector.md
docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md
```

Dual-suite `[[test]]` entries already exist. Do **not** rewrite them into other crates.

Do **not** write `catalog/canonical/v1/{controls,evidence,tests}/{source,cicd,release,supply-chain,vulnerability,infrastructure,governance}*.toml`. Do not rewrite ISO pack, scanner engines, or IAM/population/typed-evidence suites. Do not add a field to shared `CollectorDescriptor` unless implement cannot advertise failure behavior any other way.

### 4.2 Emit canonical evidence only

Envelope `evidence_type` for **new** observations is the Prompt 04/05 catalog evidence id (IAM fixture convention), **not** `source.*` and **not** `evidence.github.*`.

Canonical tests and golden adapter fixtures must not require `evidence.github.*`. Provider-native ids belong in an optional `extensions` object (see ADR), never as the type id.

ISO GH-012 still requires the historical `source.*` **strings** to appear in collector crate sources. Keep them as a documented **compatibility / mapping table** (keys), not as emitted types. Do **not** put `evidence.identity.*` on the exported `GITHUB_EVIDENCE_TYPES` const (IAM-015). Advertise identity types on `CollectorDescriptor.evidence_types` via a separate list that the descriptor unions.

Do **not** dual-emit `source.*` and canonical types as first-class observations. Dual-emit would make “another provider, same results” false (ISO existence tests would still pass on GitHub-only types).

### 4.3 Mapping: GitHub API → canonical contracts

Fact names and types follow Prompt 05 I1 freeze §4.4 and Prompt 04 §4.4. Use `EvidenceValue` (`Bool`, `Integer`, `String`, `StringList`, `Timestamp`) via `with_value`. Do not stringify booleans/integers for new envelopes. Threshold / policy comparison is the **catalog test’s** job: emit observed counts and booleans; do not invent `Effective`.

| GitHub observation | Canonical type | Required facts (min) | Notes |
| --- | --- | --- | --- |
| Repo list / get | `evidence.repository.inventory` | `subject_id`, `archived`, `in_scope` | `owner_id?`, `criticality?` when observable |
| `visibility` / `private` | `evidence.repository.visibility` | `subject_id`, `visibility` | `public` \| `internal` \| `private`. Emit `visibility_allowed` **only** if a configured allowed-set is in collector config; otherwise omit (do not guess policy) |
| `default_branch` | `evidence.repository.default-branch` | `subject_id`, `default_branch` | Protection calls **this** name, never hardcoded `main`. `default_branch_named` only if a naming policy is configured |
| Branch protection **or** equivalent ruleset on the default branch | `evidence.repository.branch-protection` | `subject_id`, `protected`, `force_push_allowed`, `deletion_allowed`, `admin_bypass_allowed?` | 404 / empty ruleset → `protected=false` (observed). **403 → diagnostic, not `protected=false`.** |
| Required reviews | `evidence.repository.review-policy` | `subject_id`, `reviews_required`, `required_reviewer_count` | Emit `meets_review_threshold` **only** when a policy integer is in collector config; otherwise omit. Never hardcode GitHub “2” as the control threshold |
| CODEOWNERS / review-ownership API | `evidence.repository.review-ownership` | `subject_id`, `ownership_defined` | Presence of CODEOWNERS or required-reviewer owners; not effectiveness |
| Required status checks | `evidence.cicd.status-checks` | `subject_id`, `status_checks_required` | |
| Secret + code scanning alerts/settings | `evidence.repository.security-scanning` | `subject_id`, `secret_scanning_enabled`, `code_scanning_enabled`, `applicable` | Disabled is a **true observation**. Missing permission is **not** `enabled=false`. |
| Dependabot / vulnerability alerts / security updates | `evidence.repository.dependency-scanning` | `subject_id`, `dependency_scanning_enabled`, `updates_monitored` | `scanned_at` only if the API supplies a time |
| Actions default workflow permissions / `permissions` block where collected | `evidence.cicd.workflow-permissions` | `subject_id`, `default_write`, `permissions_minimized` | |
| Environment protection + reviewers | `evidence.deployment.environment-protection` | `subject_id` (environment), `production?`, `authorization_required`, `protected` | Absent protected env is an observation or an explicit empty population — never a fabricated `protected=true` |
| Repo admins / owners / admin teams | `evidence.identity.privileged-membership` | `subject_id`, `privileged`, `roles` | Plus `inventory.subject` kind=identity/user when building a privileged population |
| Outside collaborators | `evidence.identity.external-access` | `subject_id`, `external`, `approved?` | |
| Deploy keys | map to `evidence.identity.privileged-membership` and/or `evidence.identity.service-account` + `external-access` if the key is write-capable / not user-bound | Never store the key material |
| Security policy file (`SECURITY.md`) | only if a landed canonical type supports presence; otherwise optional fact on inventory / omit | Do not invent `evidence.github.security-policy`. Do not emit `evidence.repository.secure-development-policy` `attested=true` from file presence alone (that control is Manual) |
| Webhooks / integrations | optional `extensions` only unless a landed contract exists | Never required by canonical tests |
| Rulesets / commit signing | fold into branch-protection / `evidence.repository.commit-signing` `{signing_required}` when observable | GitHub-native ruleset ids stay in `extensions` |
| Lockfile / provenance / artifact integrity / change-trace / security-review | emit **only** if a GitHub API (or checked-in file) actually yields the I1 facts | Do not fabricate `lockfile_present` / `provenance_present` / `traceable` |

**Generic population envelopes** (Prompt 03, not catalog IDs):

- One `inventory.subject` per in-scope repository (`kind=repository`, `id=repo:owner/name`) and per privileged/external identity when those populations are collected.
- `inventory.complete` with `authoritative=true` **only** when pagination finished for that kind and no page/permission hole remains.
- Partial list, truncated pages, or 403 mid-list → omit `inventory.complete` **or** emit `authoritative=false` / completeness `partial`. Never claim complete coverage.

**Scope / selector (GitHub-owned, do not invent `GithubRepositorySelector` on the IR):**

- Accept `org:{login}` and `repo:owner/name` (and comma-lists) on `CollectorScope` labels.
- Archived exclusion is a GitHub collection-config / request flag (e.g. `exclude_archived=true`), **not** a new Prompt 03 `SubjectSelector`.
- Archived repos: collect inventory+archived facts when they appear in the provider list; **selector “exclude archived”** must drop them from the in-scope protection/scanning populations and from `inventory.subject` used for those tests. Golden: archived repo excluded by selector.

Do not change shared `CollectorScope` / IR subject types unless a label/config approach cannot express org + exclude-archived.

### 4.4 Descriptor contract

`CollectorDescriptor` must **accurately** advertise:

| Field | Required value after implement |
| --- | --- |
| `id` | `collector.github` |
| `version` | crate / collector version (`CARGO_PKG_VERSION` or explicit collector semver) |
| `evidence_types` | **exactly** the canonical types this build can emit (union of implemented modules). No advertised-but-uncollected types. |
| `provider_family` | `source-control` |
| `subject_types` | at least `repository`, `branch`, plus `organization` / `identity` / `deployment` when those modules emit |
| `capabilities.pagination` | `true` **iff** list endpoints walk all pages |
| `capabilities.incremental` | `true` **iff** a cursor/etag path exists; else `false` (do not lie) |
| `required_permissions` | the GitHub permission/scope names actually needed for advertised types (expand beyond the three ISO-era scopes when scanning/actions/members are collected) |
| failure behavior | **documented in GitHub-owned sources** (e.g. `GITHUB_FAILURE_BEHAVIOR` const + mapping-table comments). Do **not** add `failure_behavior` to shared `CollectorDescriptor` unless implement cannot advertise otherwise. Semantics: 401/403 → `PermissionDenied` diagnostic (downstream insufficient evidence); 404 on a *protection* resource → observed absent; 404 on a *repo* → insufficient / not visible; 429 → retry-or-partial, never a boolean observation; never fabricate negatives |

COL-001 still holds: emit only declared types. COL-002: no framework results. COL-006: out-of-scope → `OutOfScope`.

`GITHUB_EVIDENCE_TYPES` remains the ADR 0002 `source.*` list (ISO GH-012 + IAM-015 compatibility). `CollectorDescriptor.evidence_types` is a **different** set: `GITHUB_CANONICAL_EVIDENCE_TYPES` (or equivalent) unioned at `descriptor()`.

### 4.5 Permission denial and partial failure

- **Never** turn 401/403 into `protected=false`, `enabled=false`, empty collaborator list-as-complete, or `secret_scanning_enabled=false`.
- Produce an explicit collector diagnostic (`CollectorError::PermissionDenied` and/or a sealed diagnostic observation that tests treat as insufficient evidence — **not** a negative control fact).
- **Do not abort the whole batch** because one subject or one sub-resource is 403. Continue other subjects; record the error; mark the run **partial**.
- API partial failure golden: one endpoint 500/502 after some success → envelopes for successes + errors + `status=partial`.
- If a required *population* list is permission-denied, do not emit `inventory.complete` authoritative.
- Redact any body that flows into diagnostics (including `handle_status` 403).

### 4.6 Pagination

- Authoritative repo / collaborator / admin / environment populations must exhaust GitHub pagination (`Link` / `page` / GraphQL cursors as implemented).
- Incomplete pagination ⇒ incomplete population. Partial page fixtures must not emit `inventory.complete` `authoritative=true`.
- Descriptor `pagination` must match the implementation.
- Fixture matching must not let a shorter path steal a longer one (today first-prefix-wins). Goldens must use exact or longest-prefix-safe keys.

### 4.7 Collection runs

`collect_batch` (and any public run API) must record a real `CollectionRun`:

| Field | Rule |
| --- | --- |
| `collector_id` | `collector.github` |
| `collector_version` | collector version |
| `scope` | canonical scope label (org/repos/selector), never a token |
| `configuration_digest` | digest of non-secret config (scope, advertised types, permission set, fixture/transport mode). **Never** include the access token. |
| `started_at` / `completed_at` | both set when the batch returns |
| `evidence_count` | `envelopes.len()` |
| `error_count` | diagnostic / hard errors |
| `status` | `complete` if every requested subject/resource finished without permission/transport holes; `partial` if any subject skipped or page missing; `failed` if nothing usable was collected |

Prefer attaching the batch `run_id` onto envelopes via existing `EvidenceEnvelope::with_collection_run` **if** that does not change typed-evidence digest law (digest body is observation+provenance only today). If attaching the run id is unsafe, document that envelopes keep the provenance digest and the batch `CollectionRun` is the authoritative run record.

Retry: the crate already owns `backoff` / `Retry-After`. If implement wires retry into `get`/`collect`, add the rate-limit golden (429 then 200, one envelope, no duplicate digest). If retry stays caller-owned, descriptor/docs must say so and the golden asserts **no silent drop** of 429 (error + partial, not a fake negative). Do not retry 401 or permission 403.

### 4.8 Security

- Never persist access tokens, `Authorization` headers, cookies, or credential material in facts, narratives, diagnostics, fixtures, or `configuration_digest`.
- Reuse `weeping_angel_evidence::redact` and seal `CREDENTIAL_KEYS`.
- Add **GitHub collector / fixture tests** for `ghp_`, `gho_`, `github_pat_`, `ghs_`, fine-grained PAT prefixes, and `Bearer ` in bodies that leak into diagnostics.
- Extending shared `redact` to cover `ghs_` is allowed **only** if it does not break typed-evidence / ISO suites; otherwise apply a GitHub-owned diagnostic sanitizer that folds `ghs_` before strings leave the collector. Do not put live-shaped tokens in fixtures.
- Fixtures may use obviously fake placeholders that **do not** match live token regexes (ISO GH-009 forbids literal `ghp_` in collector **sources**; fixture JSON under `fixtures/` must also redact).
- `authorization_header` remains redacted. Live transport, if added, is optional and out of “SaaS credential store”.

### 4.9 Golden adapter fixtures

Deterministic HTTP fixtures + expected envelopes under `fixtures/assurance/canonical/v1/github/` (or `fixtures/collectors/github/`). Target suite loads them through `GitHubClient::with_fixture` (or a thin adapter helper in `github/`). Required scenarios:

| Fixture id | Intent |
| --- | --- |
| `healthy-org` | Fully protected healthy org: authoritative inventory; default branches protected; reviews/status checks/scanning/workflow perms/prod env protection; privileged membership present |
| `unprotected-repo` | One in-scope repo `protected=false` (404 or empty ruleset), rest healthy |
| `missing-branch-protection-permission` | 403 on protection/rulesets → diagnostic / insufficient, **not** `protected=false`; run partial or failed-closed for that subject |
| `paginated-inventory` | Multi-page repo list; complete pages → `inventory.complete` authoritative; a sibling truncated fixture must **not** |
| `archived-excluded-by-selector` | Archived repo in the org list, excluded by selector; not in protection population |
| `disabled-security-scanning` | Scanning settings explicitly off → `secret_scanning_enabled=false` / `code_scanning_enabled=false` (observed) |
| `protected-environment-absent` | No protected production environment → not `protected=true`; population honest |
| `privileged-membership-population` | Admins / owners / outside collaborators mapped to identity contracts |
| `api-partial-failure` | Mid-run 5xx / transport → partial run, no fabricated facts |
| `rate-limit-retry` | 429 + `Retry-After` then success **or** explicit partial if retry not owned |

### 4.10 Controls exercisable through emitted contracts (25–40 DoD)

Count **independently assessable canonical controls enabled by emitted types**, not GitHub-native object names. The collector still must not emit `Effective`. Hybrid/manual controls count only as “supporting facts present.”

**Must-emit types that unlock ≥18 Prompt-05 automated/hybrid source+CI+release controls** (I1 freeze ids):

| Control | Enabling evidence |
| --- | --- |
| `control.source.repository-inventory` | `evidence.repository.inventory` + `inventory.complete` |
| `control.source.visibility-governance` | `evidence.repository.visibility` |
| `control.source.default-branch-protection` | `evidence.repository.default-branch` + `branch-protection` |
| `control.source.force-push-restricted` | `branch-protection.force_push_allowed` |
| `control.source.branch-deletion-restricted` | `branch-protection.deletion_allowed` |
| `control.source.required-review` | `evidence.repository.review-policy` |
| `control.source.minimum-reviewer-count` | `review-policy.required_reviewer_count` |
| `control.source.review-ownership` | `evidence.repository.review-ownership` |
| `control.source.required-status-checks` | `evidence.cicd.status-checks` |
| `control.source.admin-bypass-governance` | `admin_bypass_allowed` (hybrid supporting) |
| `control.source.signed-commits` | `evidence.repository.commit-signing` when rulesets/protection observe it |
| `control.source.secret-scanning` | `security-scanning.secret_scanning_enabled` |
| `control.source.code-scanning` | `security-scanning.code_scanning_enabled` |
| `control.source.dependency-scanning` | `evidence.repository.dependency-scanning` |
| `control.source.dependency-update-monitoring` | `updates_monitored` |
| `control.cicd.workflow-permissions` | `evidence.cicd.workflow-permissions` |
| `control.release.protected-environment` | `evidence.deployment.environment-protection` |
| `control.release.authorization` | environment reviewers as supporting facts only (hybrid) |

**Plus Prompt-04 IAM controls GitHub can actually observe** (do not emit MFA/last-active/termination unless the API yields them):

| Control | Enabling evidence |
| --- | --- |
| `control.identity.privileged-membership` (and inventory of privileged subjects) | `evidence.identity.privileged-membership` + `inventory.subject` |
| `control.identity.external-access` | outside collaborators |
| `control.identity.service-account` (supporting) | deploy keys mapped without key material |

Healthy-org evidence must be sufficient for a later catalog (or a target-suite harness that binds Prompt 05/04 test ids when present) to exercise **at least 25** of the above. Target suite enumerates the enabled control ids / type-fact pairs when Prompt 05 TOML is still absent. Do **not** claim lockfile / provenance / artifact-integrity / change-trace / security-review / secure-development-policy as GitHub-exercised unless the collector truly observes those I1 facts.

### 4.11 Target suite RED catalog (author before product code)

`tests/sdd/github_collector.target.rs` is currently assertion-empty so the registered binary stays GREEN on HEAD. The **next** slice must add these tests **first** (RED on current code, failures = missing behavior, not compile noise), then implement owned GitHub paths until GREEN.

Suggested names (stable; may add siblings, do not reuse baseline `ghc_b*` ids):

| Id | Asserts (must FAIL on `e430980c…`) |
| --- | --- |
| `ghc_000` | Dual-suite registration still present |
| `ghc_001` | Descriptor `evidence_types` equals implemented canonical types; no advertised-uncollected `source.codeowners.present` etc. as emitted types |
| `ghc_002` | `GITHUB_EVIDENCE_TYPES` still has no `evidence.identity.*`; identity types advertised via a second const |
| `ghc_003` | Failure behavior documented in GitHub-owned sources without requiring a new `CollectorDescriptor` field |
| `ghc_004` | `capabilities.pagination` matches a real page walker; `incremental` is honest |
| `ghc_005` | New envelopes use Prompt 04/05 type ids (`evidence.repository.branch-protection`, …) not `source.*` |
| `ghc_006` | Facts use typed `EvidenceValue` (bool/int) via `with_value` |
| `ghc_007` | Protection/ruleset path uses `default_branch`, never hardcoded `main` |
| `ghc_008` | Mapping table keeps historical `source.*` strings in crate sources (GH-012) |
| `ghc_009` | `org:` scope inventories repos; `inventory.subject` + honest `inventory.complete` |
| `ghc_010` | Golden `healthy-org` |
| `ghc_011` | Golden `unprotected-repo` → `protected=false` observed |
| `ghc_012` | Golden `missing-branch-protection-permission` → diagnostic, not `protected=false`; other subjects continue |
| `ghc_013` | Golden `paginated-inventory` authoritative; truncated sibling not authoritative |
| `ghc_014` | Golden `archived-excluded-by-selector` |
| `ghc_015` | Golden `disabled-security-scanning` observed false |
| `ghc_016` | Golden `protected-environment-absent` |
| `ghc_017` | Golden `privileged-membership-population` |
| `ghc_018` | Golden `api-partial-failure` → `status=partial` |
| `ghc_019` | Golden `rate-limit-retry` (retry **or** explicit partial; no silent drop) |
| `ghc_020` | `CollectionRun` filled: version, scope, configuration digest (no secrets), start/completion, counts, complete/partial/failed |
| `ghc_021` | No `ghp_` / `gho_` / `github_pat_` / `ghs_` / `Bearer ` credential material in envelopes, diagnostics, fixtures, or digest |
| `ghc_022` | ≥25 canonical controls exercisable via emitted type/fact coverage (enumerated list) |
| `ghc_023` | No `evidence.github.*` required by goldens or target assertions |
| `ghc_024` | Collector sources still lack ISO/SOC2/NIS2/DORA ids and `Effective`/`Ineffective` |

### 4.12 Dual-suite protocol

1. **Spec** (this file) — no product feature code. **This slice.**
2. **Baseline GREEN** on current HEAD: characterize §3 (`ghc_b001`–`ghc_b030`). **Already GREEN.**
3. **Target RED** on current HEAD: author §4.11 assertions. Failures must be missing behavior, not compile noise. **Authored** (`ghc_000` keep-alive; `ghc_001`–`ghc_024` plus inline siblings RED).
4. **Implement** only owned paths → target GREEN.
5. **Supersede** baseline (`#[ignore = "superseded by sdd_github_collector_target"]`).
6. Regression: workspace `cargo test --features demo`, `fmt --check`, `clippy -D warnings`. Existing `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, `sdd_typed_evidence_target`, `sdd_population_runtime_target` stay green **without rewriting those suites**.

---

## 5. Acceptance criteria

1. Dual-suite `sdd_github_collector_baseline` + `sdd_github_collector_target` remain registered like IAM/population/typed-evidence.
2. On SHA `e430980c…` / current pre-implement tree: baseline GREEN characterizing §3; after the target-suite slice, target RED on canonical mapping, goldens, pagination, 403-as-diagnostic, filled `CollectionRun`, and credential guards — not unrelated compile noise.
3. After implement: target GREEN; baseline ignored/superseded; workspace verify stays green.
4. Descriptor advertises only collected canonical types, true pagination/incremental flags, real permissions, subject types, provider family `source-control`, and failure behavior (GitHub-owned documentation/const; no shared-type redesign unless strictly required).
5. Emitted envelope types are Prompt 04/05 contracts (`evidence.repository.*`, `evidence.cicd.*`, `evidence.deployment.*`, `evidence.identity.privileged-membership` / `external-access`, plus Prompt 03 `inventory.subject` / `inventory.complete`). No `evidence.github.*` required by canonical tests. No ISO/SOC2/NIS2/DORA ids in collector logic.
6. Permission 403/401 produce explicit insufficient-evidence / `PermissionDenied` diagnostics; they never become negative boolean observations; the batch does not abort other subjects.
7. Pagination for authoritative populations is complete; partial pages never claim `inventory.complete` authoritative.
8. `collect_batch` records collector version, scope, configuration digest (no secrets), start/completion, evidence count, errors, and `complete`/`partial`/`failed`.
9. All ten golden adapter fixtures exist and pass; healthy-org can exercise ≥25 canonical controls through evidence contracts (catalog tests if landed; otherwise type/fact coverage enumerated in the target suite).
10. Tokens (`ghp_`, `gho_`, `github_pat_`, `ghs_`, `Bearer`) never appear in envelopes, diagnostics, fixtures, or configuration digest; `redact` reused; ISO GH-007/GH-009 needles remain satisfied.
11. `GITHUB_EVIDENCE_TYPES` remains free of `evidence.identity.*` (IAM-015); ISO GH-012 `source.*` strings remain present in collector sources as the mapping table.
12. Another provider could emit the same contracts and receive the same test results; the collector never computes `Effective`/`Ineffective`.

---

## 6. Out of scope

- ISO 27001 / SOC 2 / NIS2 / DORA remapping (Prompt 12) and any rewrite of `frameworks/iso-27001/2022`
- Calculating `Effective` / `Ineffective` / readiness / SoA
- Redesigning catalog IDs or landing Prompt 05/06/07/08 catalog TOML
- SaaS credential store, OAuth app, or secret manager
- Entra / Okta / GitLab / Bitbucket collectors
- Live mandatory HTTP in unit tests (fixtures remain the golden path)
- Changing `EvidenceValue`, population evaluator semantics, or adding `resolve_repository_inventory`
- Rewriting IAM / population / typed-evidence / ISO SDD suites
- Adding `failure_behavior` to shared `CollectorDescriptor` unless a later implement cannot advertise otherwise
- Scanner engines and the one-way bridge
- Concurrent Prompt 06/07/08 file trees

---

## 7. Risks

| Risk | Disposition |
| --- | --- |
| Dual-emit `source.*` + canonical types couples tests to GitHub | Emit canonical only; keep `source.*` as mapping-table strings for ISO GH-012 |
| Changing `GITHUB_EVIDENCE_TYPES` to `evidence.identity.*` breaks IAM-015 | Identity types on descriptor via a second const; do not edit IAM suite |
| Prompt 05 AC10 snapshots assert “collector untouched” | This slice owns Prompt 09; Prompt 05 must not land a permanent “never emit canonical” pin. Do not edit Prompt 05/06/07/08 trees here |
| Advertised-vs-collected gap returns | Descriptor is generated from implemented modules; target asserts set equality |
| 403 abort loses the rest of the org | Per-subject diagnostics + `partial` status |
| Partial pagination marked authoritative | `inventory.complete` only after last page and no list 403 |
| Hardcoded `main` misses real default branches | Protection/ruleset path uses `default_branch` |
| Token leak via 403 body, `ghs_`, or fixture | Redact diagnostics; GitHub-pattern tests; no live-shaped tokens in sources/fixtures |
| Inventing `evidence.github.*` required by tests | Forbidden; extensions optional and ignored by canonical tests |
| Expanding evidence crate provenance (digest break) | Prefer optional observation `extensions` object; do not change typed-evidence digest law |
| Shared `CollectorDescriptor` redesigned for one field | Advertise failure behavior in `github/descriptor.rs`; do not add `failure_behavior` unless strictly required |
| Implementing retry incorrectly duplicates envelopes | Ledger/set keyed by digest; golden asserts one envelope after 429→200 |

---

## 8. ADR

**Needed.** ADR 0002 called GitHub `source.*` types “canonical.” Prompts 04–05 replaced that with provider-neutral `evidence.*` contracts. Mapping, ISO/IAM needle coexistence, “no `evidence.github.*` in tests”, and “do not redesign shared collector types for failure behavior” are architecture/contract decisions.

Draft: [`docs/adr/0003-github-collector-canonical-evidence-mapping-draft.md`](../adr/0003-github-collector-canonical-evidence-mapping-draft.md). Accept after target GREEN.

---

## 9. Implement notes (later phase — not this spec slice)

Suggested module fill (names may follow crate style):

- `repositories.rs` — org/user inventory, visibility, archive, selector
- `branches.rs` / `protection.rs` / `rulesets.rs` — default-branch protection + reviews + force-push/deletion + status checks + signing
- `security.rs` — secret/code/dependency scanning
- `workflows.rs` — Actions default permissions + environment protection
- `collaborators.rs` — admins, outside collaborators, deploy keys (no key material)
- `normalize.rs` — GitHub JSON → canonical `EvidenceValue` facts
- `descriptor.rs` — honest advertisement + `source.*` → canonical mapping table + failure-behavior const
- `client.rs` — fixture transport; longest-prefix-safe match; optional pagination helpers; retry if owned

Do not add `octocrab` if ISO scanner-bridge tests forbid it in **other** crates; collector crate may grow a small HTTP client later, but goldens stay fixture-based.

---

## 10. Definition of done

GitHub exercises at least 25–40 canonical controls through canonical evidence; population completeness is explicit; permissions fail safely; no credential material leaks; another provider could emit the same contracts and receive the same test results. Dual-suite protocol closed; workspace verify green; this spec and the run report updated with proof.
