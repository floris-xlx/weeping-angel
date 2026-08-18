# SDD: IAM Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Implemented — target GREEN; baseline superseded** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 04 — identity / authentication / authorization / privileged access / account lifecycle |
| Source prompt | [`docs/prompts/canonical-assurance-v1/04-iam-catalog.md`](../prompts/canonical-assurance-v1/04-iam-catalog.md) |
| Planning baseline SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` (`main`, 2026-08-18) |
| Dual-suite | `sdd_iam_catalog_target` GREEN (IAM-001…016); `sdd_iam_catalog_baseline` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) — owned by Prompt 01 |
| Prompt-02 / 03 (consumed) | [`docs/sdd/typed-evidence.md`](typed-evidence.md), [`docs/sdd/population-runtime.md`](population-runtime.md) |
| Spine / ISO law | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **IAM catalog slice**. It does not replace the Prompt 01 catalog-infrastructure SSOT, the Prompt 02 typed-evidence contract, or the Prompt 03 population-runtime contract. Prompts 01–03 have landed; this slice consumes their loader, `EvidenceValue`, and population evaluator and **must not** invent a second copy.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

---

## 1. Problem / user-visible goal

Organizations need to assess identity, authentication, authorization, privileged access, and account lifecycle using **provider-neutral** canonical controls. On the planning SHA the only IAM-adjacent content lived inside the ISO 27001 framework pack as a handful of thin controls (`access.mfa.privileged`, `access.least-privilege`, `access.periodic-review`, `personnel.access-termination`) wired to **GitHub-shaped** evidence types (`source.admin.permissions`, `source.collaborator.permission`). Those tests are presence/hybrid/manual checks, not population assertions such as “all privileged identities have MFA.”

That SHA had no IAM family in `catalog/canonical/v1/`, no `control.identity.*` library, no `evidence.identity.*` fact contracts, and no deterministic identity-population fixtures. A future Entra, Okta, or Google Workspace collector therefore had nowhere canonical to emit facts. §13 records the family that shipped.

**User-visible goal:** a coherent IAM catalog (~20–30 independently assessable controls) that can evaluate realistic identity populations from **any** future identity provider’s canonical evidence, produce deterministic and explainable results (missing ≠ stale ≠ failure ≠ manual review ≠ approved exception), and pass the catalog validator plus full workspace verification.

This slice does **not** claim ISO/SOC 2/NIS2 coverage. Framework remapping is Prompt 12.

---

## 2. Dependencies and fail-closed blockers

| Prompt | Owns | Planning SHA `5fa3a23a…` | At implement | This slice may |
| --- | --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules, CLI validate/stats/inspect | Absent | **Landed.** Consume `weeping-angel-canonical-catalog`. | Add identity TOML + manifest lines. Do not invent a second loader/validator/digest. |
| 02 typed evidence | Typed `EvidenceValue`, canonical serialization, control-test typed comparisons | Landed | **Landed.** Consume `weeping-angel-evidence::EvidenceValue`. | Declare required fact *names* and semantic types. No second value enum. |
| 03 population runtime | Subject populations, `AllSubjects` / `CoverageAtLeast` real coverage, missing/stale/failing subject split | Stub `CoverageAtLeast` | **Landed.** Consume `weeping-angel-control-test` population arms / `PopulationEvaluation`. | Declare population-based tests. **Do not locally reimplement coverage math.** |

Rebase rule: if Prompts 01–03 land before or during implementation, rebase onto their file layout, ID validator, evidence value API, and population evaluator. Prefer adapting IAM content to those contracts over extending this slice’s scope.

---

## 3. Planning-SHA characterization (`5fa3a23a…`)

Historical. Product IAM content now exists (§13). Do not treat this section as current tree state.

Recorded against workspace HEAD `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` (merge of `agent/canonical-assurance-prompts-v1`). Prompt markdown exists under `docs/prompts/canonical-assurance-v1/`; product code for Prompts 01–03 does not.

### 3.1 Catalog infrastructure

- No `catalog/canonical/v1/` directory.
- No `CanonicalCatalog` type, `load` / `validate` / `digest` API, or schema `weeping-angel/canonical-catalog/v1`.
- Framework packs (`frameworks/iso-27001/2022/`, `frameworks/wa-baseline/1/`) remain the only on-disk control libraries. Schema: `weeping-angel/framework-pack/v1`.
- `weeping-angel-framework::pack` loads ISO `metadata.toml` `[[control]]` / `[[test]]` rows into IR `Control` / `PlannedControlTest`. Controls have `id`, `title`, `description` only (no domains, evidence-requirement refs, or test-expression AST on disk).

### 3.2 IAM-adjacent ISO pack content

`frameworks/iso-27001/2022/metadata.toml` holds four access/personnel controls:

| Pack control id | Automation | Required evidence (test) |
| --- | --- | --- |
| `access.mfa.privileged` | Automated (pack) / test `hybrid` | `source.admin.permissions` |
| `access.least-privilege` | Hybrid | `source.collaborator.permission` |
| `access.periodic-review` | Hybrid / test `manual` | `policy.access.reviewed` |
| `personnel.access-termination` | Hybrid | `personnel.access.terminated` |

IDs are **not** in the Prompt 01 `control.*` namespace. They contain no provider token, but they are GitHub-shaped in evidence and live inside a **framework pack**, not a canonical catalog.

ISO mappings (`mappings.toml`) point `iso27001:a.8.5` / `a.8.2` / `a.8.3` / `a.5.15` / `a.5.18` / `a.5.16` / `a.6.5` at those four IDs with `partial` completeness. This slice **must not** retarget those mappings (Prompt 12).

### 3.3 Evidence and evaluation (planning SHA `5fa3a23a…`; superseded by Prompt 02)

On the planning SHA:

- `EvidenceObservation.facts` was `BTreeMap<String, String>` (`crates/weeping-angel-evidence/src/lib.rs`).
- Control-test `EvidenceValue` (`Null`, `Boolean`, `Integer`, `Decimal`, `String`, `Timestamp`, `Duration`, `StringSet`, `Identifier`) was produced by `parse_fact` (string `"true"` → bool, integer parse, else string).

**Now (Prompt 02 landed):** facts are `BTreeMap<String, EvidenceValue>` in `weeping-angel-evidence`; control-test re-exports that enum. IAM fixtures declare typed names (`mfa_enabled = Bool`, counts as `Integer`, role lists as `StringList`) and must not fork a second value type. See [`docs/sdd/typed-evidence.md`](typed-evidence.md) and [ADR 0003 typed evidence](../adr/0003-typed-evidence-canonical-serialization.md).

Still on the planning SHA (population, not facts):

- `TestExpr` is a closed AST: `Exists`, `Missing`, comparisons, `Contains` / `NotContains`, `In`, `Count`, `FreshWithin`, `CoverageAtLeast`, `All` / `Any` / `None` / `Not`, `ManualReview`.
- `CoverageAtLeast` implementation:

```text
let _ = (selector, evidence, percentage);
effectiveness = PartiallyEffective
rationale = "subject coverage remains partial unless the threshold is met"
```

- Evaluator selects **first** matching envelope (`first_selector`). There is no per-subject index, no authoritative population, no failing/missing subject lists.
- `Effectiveness::ExceptionApproved` exists in the enum and rank table; **no `TestExpr` arm produces it**.
- Exception IR exists (`Exception` + `ExceptionStatus::{Proposed,Approved,Expired,Revoked}`) and attaches to `ControlImplementation.exception_ids`, not to catalog tests.

### 3.4 Identity / subject model already in IR (thin)

`Identity` = `{ id, kind: User|Service|Team|Role|Other, displayName? }`. Not an IAM platform.

IR `SubjectKind` includes `Identity`, `User`, `PrivilegedIdentity` — **not** `ServiceAccount`. `SubjectSelector` is `{ kind, ids, tags, scope }`.

A **second**, thinner `SubjectSelector` lives in `weeping-angel-control-test` (`kind: Option<String>`, `id: Option<String>`). Prompt 03 owns unifying population resolution; this slice must not create a third selector type.

### 3.5 Collectors

- `GitHubCollector` advertises `source.admin.permissions` and `source.collaborator.permission` in `descriptor.rs`. `collaborators.rs` is a module stub (`pub const MODULE`). Repo collection does not emit identity inventory, MFA status, last-active, account-status, or service-account facts.
- `FixtureCollector` / local / manual collectors have no IAM identity population fixtures.
- Fixtures on disk: `fixtures/assurance/iso27001/{repo-secure,repo-insecure}` only.

### 3.6 Tests and CLI

Root `Cargo.toml` now registers `sdd_iam_catalog_baseline` + `sdd_iam_catalog_target` alongside the other SDD suites. Product IAM catalog content is still absent.

ISO target suite (`tests/sdd/iso27001_assurance.target.rs`) freezes prefixes `access.`, `personnel.`, and expected ids including `access.mfa.privileged`. This slice must not break that suite by rewriting the ISO pack.

### 3.7 What “IAM assessment” means today

A caller can compile the ISO pack and run `test.access.mfa.privileged`, which requires **some** `source.admin.permissions` envelope to exist (hybrid). It cannot:

- require MFA on every privileged identity;
- distinguish a missing inventory from an inactive admin still marked active;
- evaluate terminated-but-active accounts, ownerless service accounts, stale access reviews, or an approved break-glass exception;
- accept Entra/Okta/Workspace-shaped facts without teaching tests about those providers.

The baseline suite for this slice therefore characterizes **absence of a canonical IAM catalog** and **presence of the ISO-pack IAM sliver**, not a working identity-population evaluator.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

IAM domain content lands in the Prompt 01 catalog tree (rebase if the exact filenames differ):

```text
catalog/canonical/v1/
  manifest.toml
  controls/          # includes identity family
  evidence/          # includes evidence.identity.*
  tests/             # includes test.identity.*
```

If Prompt 01 uses a different split (single TOML vs per-file), follow that contract. Do **not** add IAM controls to `frameworks/iso-27001/2022/metadata.toml`.

Optional deterministic fixtures (preferred path; rebase to Prompt 01 fixture convention if it lands):

```text
fixtures/assurance/canonical/v1/identity/
  healthy-org/
  privileged-without-mfa/
  inactive-admin-active/
  terminated-employee-active/
  service-account-without-owner/
  partial-inventory/
  stale-access-review/
  break-glass-approved-exception/
```

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.identity.<slug>
evidence.identity.<slug>
test.identity.<slug>
```

Reject in canonical IAM content (validator + target suite):

- provider tokens in IDs or as the subject of a control (`okta`, `entra`, `azure-ad`, `google-workspace`, `workspace`, `github`, `aws`, ` cognito`);
- framework tokens in IDs or narrative (`iso27001`, `iso-27001`, `soc2`, `soc-2`, `nis2`, `dora`, `gdpr`);
- orphaned evidence types or tests (every `evidence.identity.*` / `test.identity.*` referenced by at least one control; every control test id resolves);
- duplicate IDs;
- existence-only tests masquerading as population tests (see §4.5).

Correct: `control.identity.mfa`. Incorrect: `control.okta.mfa`, `control.entra.admin-mfa`, `test.iso27001.a.8.5`.

Provider-specific field names (`okta_factor_type`, `entra_directory_role_template_id`) must not appear in evidence **type** ids. They may appear only inside a collector’s private normalize step that **emits** canonical facts.

### 4.3 Control family (23 independently assessable controls)

Do not split these into micro-controls to inflate count. Titles and objectives are framework-neutral.

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.identity.unique-user-identities` | Unique user identities | Automated | identity / user | `inventory` | `test.identity.unique-user-identities` |
| `control.identity.mfa` | Multi-factor authentication | Automated | user | `inventory`, `mfa-status` | `test.identity.mfa-enabled` |
| `control.identity.privileged-mfa` | Privileged multi-factor authentication | Automated | privileged identity | `privileged-membership`, `mfa-status` | `test.identity.privileged-mfa-enabled` |
| `control.identity.strong-authentication-policy` | Strong authentication policy | Hybrid | organization | `authentication-state` (+ manual policy attestation allowed) | `test.identity.strong-authentication-policy` |
| `control.identity.privileged-inventory` | Privileged identity inventory | Automated | privileged identity | `inventory`, `privileged-membership` | `test.identity.privileged-inventory-complete` |
| `control.identity.least-privilege` | Least privilege | Hybrid | identity | `role-membership`, `privileged-membership` | `test.identity.least-privilege` |
| `control.identity.privileged-access-minimization` | Privileged-access minimization | Hybrid | privileged identity | `privileged-membership`, `role-membership` | `test.identity.privileged-access-minimized` |
| `control.identity.access-approval` | Access approval / authorization | Hybrid / manual | identity | `lifecycle-event` and/or manual approval record | `test.identity.access-approval-recorded` |
| `control.identity.periodic-access-review` | Periodic access review | Hybrid / manual | identity / privileged identity | `access-review` | `test.identity.access-review-current` |
| `control.identity.inactive-account-lifecycle` | Inactive account lifecycle | Automated | identity | `account-status`, `last-active` | `test.identity.no-inactive-privileged-accounts` (privileged subset) + general inactive lifecycle test |
| `control.identity.terminated-user-removal` | Terminated-user removal | Automated | user | `account-status`, `lifecycle-event` | `test.identity.no-terminated-active-accounts` |
| `control.identity.joiner-mover-leaver` | Joiner / mover / leaver lifecycle | Hybrid | user | `lifecycle-event`, `account-status`, `role-membership` | `test.identity.jml-events-recorded` |
| `control.identity.service-account-inventory` | Service-account inventory | Automated | service account | `service-account`, `inventory` | `test.identity.service-accounts-inventoried` |
| `control.identity.service-account-ownership` | Service-account ownership | Automated | service account | `service-account`, `account-owner` | `test.identity.all-service-accounts-have-owner` |
| `control.identity.service-account-credential-governance` | Service-account credential governance | Hybrid | service account | `service-account` (+ credential-state facts; no secret material) | `test.identity.service-account-credentials-governed` |
| `control.identity.break-glass-access` | Emergency / break-glass access governance | Hybrid | privileged identity | `privileged-membership`, `account-status`, Exception IR | `test.identity.break-glass-governed` |
| `control.identity.shared-account-restriction` | Shared-account restriction | Automated | identity | `inventory`, `account-status` | `test.identity.no-ungoverned-shared-accounts` |
| `control.identity.credential-management` | Authentication credential management | Hybrid | identity | `authentication-state` | `test.identity.credentials-managed` |
| `control.identity.privileged-role-change-monitoring` | Privileged-role changes monitored | Hybrid | privileged identity | `privileged-membership`, `lifecycle-event` | `test.identity.privileged-role-changes-monitored` |
| `control.identity.external-guest-access` | External / guest access governance | Automated + hybrid gate | identity (guest) | `external-access`, `account-status` | `test.identity.no-unapproved-guest-access` |
| `control.identity.stale-privileged-membership` | Stale privileged membership | Automated | privileged identity | `privileged-membership`, `last-active` / `access-review` | `test.identity.no-stale-privileged-membership` |
| `control.identity.access-revocation-timeliness` | Access revocation timeliness | Hybrid | user | `lifecycle-event`, `account-status` | `test.identity.revocation-timely` |
| `control.identity.segregation-of-duties` | Segregation of duties | Manual / hybrid | identity / role | `role-membership` + manual SoD matrix / attestation | `test.identity.sod-review` |

Each control record (once Prompt 01 schema exists) must carry: stable id, title, description/objective, domain(s) from existing `ControlDomain` (`Authentication`, `Authorization`, `AccessControl`, `PersonnelSecurity` as appropriate), evidence-requirement refs, test refs, and an honest automation class (`Automated` | `Hybrid` | `Manual`).

**Do not invent technical automation** for access approval, SoD, or periodic review. Those controls must remain Hybrid or Manual even if a single technical signal exists.

### 4.4 Canonical evidence (facts, not conclusions)

Reuse Prompt 01/02 evidence declarations when present. This slice **defines** the IAM family if the central contract has not already reserved the ids.

| Evidence type | Observed facts (canonical names; types are semantic — store via Prompt 02 `EvidenceValue` when available) | Not allowed |
| --- | --- | --- |
| `evidence.identity.inventory` | `subject_id`, `account_kind` (`user` \| `service` \| `shared` \| `guest` \| `break-glass`), `display_name?`, `unique_key` (login/UPN hash or durable id — **not** a secret) | `compliant`, provider user-object dumps as type id |
| `evidence.identity.authentication-state` | `subject_id`, `auth_methods` (string list), `password_age_days?`, `phish_resistant?` (bool) | password / token / cookie values |
| `evidence.identity.mfa-status` | `subject_id`, `mfa_enabled` (bool), `methods?` (string list) | “MFA control passed” |
| `evidence.identity.privileged-membership` | `subject_id`, `privileged` (bool), `roles` (string list), `membership_observed_at` | “least privilege effective” |
| `evidence.identity.role-membership` | `subject_id`, `roles` (string list), `high_privilege_count?` (integer) | SoD pass/fail |
| `evidence.identity.last-active` | `subject_id`, `last_active_at` (timestamp), `inactive` (bool, derived by collector from policy window **or** raw timestamp only — prefer raw timestamp) | “inactive lifecycle effective” |
| `evidence.identity.account-status` | `subject_id`, `status` (`active` \| `inactive` \| `disabled` \| `terminated` \| `pending`) | HR legal conclusions |
| `evidence.identity.account-owner` | `subject_id`, `owner_subject_id`, `owner_assigned` (bool) | — |
| `evidence.identity.access-review` | `subject_id` or `population_id`, `reviewed_at` (timestamp), `reviewer_id?`, `result` (`approved` \| `revoked` \| `exception`) | “periodic review effective” |
| `evidence.identity.lifecycle-event` | `subject_id`, `event` (`joined` \| `moved` \| `left` \| `role-changed` \| `access-approved` \| `access-revoked`), `occurred_at`, `approved?` (bool) | — |
| `evidence.identity.service-account` | `subject_id`, `is_service_account` (true), `credential_rotated_at?`, `interactive_login?` (bool) | raw keys |
| `evidence.identity.external-access` | `subject_id`, `external` (bool), `sponsor_id?`, `approved` (bool), `expires_at?` | — |

Seal rules already in force still apply: no credential-shaped keys (`token`, `password`, `secret`, …); no compliance narratives.

Additional supporting evidence types may be added only if referenced by a control and a test (no orphans). Prefer extending facts on the types above.

### 4.5 Tests (population-based, not existence checks)

Required reusable tests (Prompt 04 list + the minimum extras so no control is untested):

```text
test.identity.mfa-enabled
test.identity.privileged-mfa-enabled
test.identity.no-inactive-privileged-accounts
test.identity.no-terminated-active-accounts
test.identity.all-service-accounts-have-owner
test.identity.access-review-current
test.identity.no-unapproved-guest-access
```

Semantics (authoritative intent; exact `TestExpr` spelling follows Prompt 03 once landed):

| Test | Population | Pass | Fail | Missing | Stale | Manual / exception |
| --- | --- | --- | --- | --- | --- | --- |
| `mfa-enabled` | all in-scope **user** identities from authoritative inventory | every subject has `mfa_enabled=true` | ≥1 user with `mfa_enabled=false` | inventory unknown **or** a known user lacks `mfa-status` | `mfa-status` / inventory older than catalog freshness | n/a |
| `privileged-mfa-enabled` | all **privileged** identities | every privileged subject MFA-enabled | privileged + MFA false | privileged population incomplete or MFA envelope missing for a known privileged subject | stale privileged-membership or mfa-status | approved break-glass exception may yield `ExceptionApproved` **for that subject only** |
| `no-inactive-privileged-accounts` | privileged identities | none are `inactive`/`disabled` **and** still privileged/active | inactive/disabled admin still `status=active` or still privileged | missing last-active or account-status for a known privileged subject | stale last-active | — |
| `no-terminated-active-accounts` | users with `lifecycle-event=left` or `status=terminated` | none remain `status=active` | terminated + active | termination event without matching account-status (or inverse) | stale account-status after leave event | — |
| `all-service-accounts-have-owner` | service accounts | every SA has `owner_assigned=true` and non-empty `owner_subject_id` | SA with no owner | partial SA inventory | stale owner evidence | — |
| `access-review-current` | in-scope identities (or privileged subset per control binding) | each has `reviewed_at` within freshness window | review present but expired / failed | no review envelope for a known subject | `reviewed_at` outside window → `StaleEvidence` (not Ineffective-as-missing) | Hybrid: missing **manual** attestation → `ManualReviewRequired` / `InsufficientEvidence`, never Effective |
| `no-unapproved-guest-access` | identities with `external=true` or `account_kind=guest` | every guest has `approved=true` (and unexpired if `expires_at` set) | guest `approved=false` | guest inventory incomplete | stale external-access | — |

**Forbidden encoding:** `Exists(evidence.identity.mfa-status)` as the body of `test.identity.mfa-enabled`. Existence of some MFA fact is not MFA on the population.

Result metadata (from Prompt 03 evaluation detail, not invented here) must be sufficient to explain: population size, evaluated, passing, failing, missing, coverage, failing subject ids, missing subject ids.

Unknown / non-authoritative population **must not** produce `Effective` for an all-subjects test.

### 4.6 Manual / hybrid honesty

| Control | Why not fully automated |
| --- | --- |
| `access-approval` | Approval is a governance act. Technical `lifecycle-event.approved=true` is supporting, not sufficient, unless the catalog marks a specific org policy as fully encoded. Default: Hybrid; absence of attestation → `ManualReviewRequired` or `InsufficientEvidence`. |
| `periodic-access-review` | Cadence, reviewer independence, and recertification quality are organizational. `access-review` freshness is the automatable slice. |
| `segregation-of-duties` | Conflicting-role matrices are org-specific. Canonical test is “SoD review attested / matrix present for in-scope roles,” not a hardcoded pair of role names. |
| `strong-authentication-policy` | Policy text and risk acceptance are manual; method inventory is automated. |
| `joiner-mover-leaver` | HR source-of-truth and ticket completeness are hybrid. |
| `break-glass-access` | Inventory + MFA/monitoring can be automated; approval, time-box, and post-use review use Exception IR + manual attestation. |

Do not add a synthetic collector that auto-passes these controls.

### 4.7 Fixtures (deterministic)

Each fixture is a frozen evidence set (+ optional Exception records) with a fixed `collectedAt`. Expected effectiveness is part of the target suite.

| Fixture | Intent | Expected highlights |
| --- | --- | --- |
| `healthy-org` | Authoritative inventory; all users MFA; privileged MFA; no inactive privileged; no terminated-active; every SA owned; current reviews; guests approved or absent | Automated IAM tests `Effective`. Hybrid/manual tests `Effective` only if attestations present; otherwise `ManualReviewRequired` / `InsufficientEvidence` — document the fixture’s attestation choice and keep it deterministic. |
| `privileged-without-mfa` | One privileged user `mfa_enabled=false`; rest healthy | `privileged-mfa-enabled` → `Ineffective` naming that subject. `mfa-enabled` fails if that subject is also in the user population. Missing ≠ fail. |
| `inactive-admin-active` | Privileged identity `inactive=true` / stale `last_active_at` but `account-status=active` and still privileged | `no-inactive-privileged-accounts` → `Ineffective`. |
| `terminated-employee-active` | `lifecycle-event=left` (or `status=terminated` on HR fact) but account still `active` | `no-terminated-active-accounts` → `Ineffective`. |
| `service-account-without-owner` | SA inventory row with `owner_assigned=false` | `all-service-accounts-have-owner` → `Ineffective`. |
| `partial-inventory` | Population marked **non-authoritative** or known subjects without envelopes | All-subjects tests → `InsufficientEvidence` (not Effective, not Ineffective-as-if-empty). |
| `stale-access-review` | Review envelopes exist but `reviewed_at` outside freshness | `access-review-current` → `StaleEvidence`. |
| `break-glass-approved-exception` | Named break-glass account lacks MFA or is shared; `Exception` `status=Approved` bound to `control.identity.break-glass-access` / `privileged-mfa` for that subject, unexpired | That subject contributes `ExceptionApproved`, not silent Effective and not Ineffective. Expired/revoked exception must not pass. |

Fixtures emit **canonical** `evidence.identity.*` only. No `source.admin.permissions`. No collector id in evidence type.

### 4.8 Integration rules (consume, do not redesign)

- Loader / validate / digest: Prompt 01 `CanonicalCatalog`. IAM files must pass `validate` (no orphans, no provider/framework tokens, deterministic digest).
- Typed facts: Prompt 02 landed. Store via `weeping-angel-evidence::EvidenceValue` (`with_value`). `with_fact` remains string-compat only; do not rely on `parse_fact` (deleted from the evaluate path).
- Population evaluation: Prompt 03 (`evaluate_coverage`, identity inventory resolution). IAM tests are **declarations**. Do not implement `AllSubjects` in this slice.
- Exception: reuse IR `Exception` + existing `Effectiveness::ExceptionApproved`. If the evaluator still never emits that state, record a Prompt 03 / runtime blocker rather than adding a parallel exception engine.
- Subject kinds: consume Prompt 03 IR kinds (`Identity`, `User`, `PrivilegedIdentity`, `ServiceAccount`). Select service accounts by evidence type `evidence.identity.service-account` and/or `account_kind=service` on inventory. Do not add a third `SubjectSelector` type.
- ISO pack, GitHub collector, framework compiler, generic `TestExpr` semantics: **untouched** unless a documented compile blocker requires a one-line compatibility fix, which must be called out in the implement-phase SDD log.

### 4.9 Dual-suite protocol

Follow the existing root `[[test]]` pattern.

| Suite | Path (planned) | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/iam_catalog.baseline.rs` · `sdd_iam_catalog_baseline` | Historical: GREEN on planning SHA (no IAM family). Now `#[ignore]` so absence-of-catalog is not CI green. |
| Target | `tests/sdd/iam_catalog.target.rs` · `sdd_iam_catalog_target` | **GREEN** — CI gate (IAM-001…016). |

After target GREEN: `#[ignore]` or delete/move the baseline so CI does **not** keep “absence of IAM catalog” as required green (`supersede_kind=skip` preferred, matching ISO). Target remains the gate.

Suggested target assertion clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| IAM-001 | Catalog tree / loader (Prompt 01 API) loads IAM content offline |
| IAM-002 | Digest of IAM slice is deterministic |
| IAM-003 | All 23 `control.identity.*` ids present, stable, prefixed `control.identity.` |
| IAM-004 | Required `evidence.identity.*` types declared; no orphans |
| IAM-005 | Required `test.identity.*` ids declared and referenced |
| IAM-006 | Validator rejects provider tokens in IAM ids |
| IAM-007 | Validator rejects `iso27001` / `soc2` / `nis2` in IAM ids and IAM file text |
| IAM-008 | No IAM control lives in `frameworks/iso-27001/2022` as `control.identity.*`; ISO pack ids unchanged |
| IAM-009 | `test.identity.privileged-mfa-enabled` is population-based (fails privileged-without-mfa; does not pass on a single MFA envelope) |
| IAM-010 | Missing vs stale vs fail vs manual vs exception distinguished on the eight fixtures |
| IAM-011 | Partial inventory cannot yield Effective on all-subjects tests |
| IAM-012 | Approved unexpired break-glass exception → `ExceptionApproved` for that subject |
| IAM-013 | Access-approval / SoD / periodic-review marked Hybrid or Manual |
| IAM-014 | Credential-shaped facts still rejected |
| IAM-015 | Framework crate still has no collector/provider SDK dependency; collector still has no IAM-framework mapping |
| IAM-016 | Existing `sdd_iso27001_assurance_target` and `sdd_assurance_runtime_target` stay green (ISO sliver not rewritten) |

### 4.10 Documentation after implement

Done in the docs pass: this file’s §13, accepted [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), IAM pointer on [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md). Prompt 01 SSOT is not overwritten. No Entra/Okta collection or ISO remap is claimed.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase.

1. Dual-suite `sdd_iam_catalog_baseline` + `sdd_iam_catalog_target` is registered in root `Cargo.toml` like existing SDD tests.
2. On SHA `5fa3a23a…` (pre-IAM content): baseline GREEN; target RED for missing `control.identity.*` / catalog / population fixtures — not for unrelated compile errors.
3. After implement (**met**): target GREEN; baseline ignored so absence-of-catalog is not a CI requirement; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. Twenty-three `control.identity.*` controls exist with stable ids, domains, evidence requirements, test refs, and honest automation class; count stays in 20–30 with no artificial micro-controls.
5. Evidence types `evidence.identity.{inventory,authentication-state,mfa-status,privileged-membership,role-membership,last-active,account-status,account-owner,access-review,lifecycle-event,service-account,external-access}` are declared as facts, not conclusions.
6. Tests include at least the seven Prompt-04 ids and evaluate **populations** (all privileged identities have MFA), not existence of one envelope.
7. Evaluator outcomes distinguish missing data, stale data, actual failure, manual review, and approved exception on the eight named fixtures.
8. Access-approval, segregation-of-duties, and periodic-access-review are Hybrid or Manual; they cannot auto-pass without attestation.
9. Catalog validator (Prompt 01) accepts the IAM slice: no duplicate/orphan/dangling ids, no provider names, no ISO/SOC2/NIS2 references in canonical IAM content.
10. ISO pack control ids and mappings are unchanged; `sdd_iso27001_assurance_target` remains green.
11. No Entra / Okta / Google Workspace / GitHub IAM collector is added; GitHub continues to emit `source.*` only.
12. No second `CanonicalCatalog` loader, no second `EvidenceValue` enum, no local `IamPopulation` / `iam_all_subjects` fork. Prompt 03 coverage is consumed as-is.
13. Break-glass approved-exception fixture uses existing Exception IR and `ExceptionApproved`; expired/revoked exceptions do not pass.
14. Credential keys and compliance narratives remain rejected on IAM evidence.
15. Prompt 01 SSOT path `docs/sdd/canonical-assurance-catalog-v1.md` is not overwritten by this slice.

---

## 6. Out of scope

- Entra ID, Okta, Google Workspace, AD, Cognito, or GitHub-identity collector implementations.
- Remapping ISO 27001 (or SOC 2 / NIS2) onto `control.identity.*` (Prompt 12).
- Redesign of `CanonicalCatalog` loader/validator/digest (Prompt 01).
- Redesign of typed evidence / digest canonicalization (Prompt 02).
- Reimplementing `CoverageAtLeast` / `AllSubjects` / population indexes (Prompt 03 owns them).
- Changing generic `TestExpr` semantics unless Prompt 03 owner agrees a documented blocker exists.
- Rewriting ISO `metadata.toml` / `mappings.toml` control ids (`access.mfa.privileged` stays until Prompt 12).
- Adding further `SubjectKind` variants or a third `SubjectSelector` type in this slice (`ServiceAccount` already exists from Prompt 03).
- HRIS, IGA, PAM, or ticket-system product integrations.
- Certification, “compliant”, or audit-passed language.
- SDLC / vulnerability / infrastructure / governance catalog families (Prompts 05–08).

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Prompts 01–03 have not landed; implementers invent a parallel catalog/runtime | Hard rebase rule + AC 12; fail-closed if loader/population missing. |
| `CoverageAtLeast` stub makes “all privileged have MFA” look PartiallyEffective always | Target suite must assert real population outcomes; stay RED until Prompt 03. Do not locally finish the runtime. |
| Existence checks sneak in as IAM tests | IAM-009: privileged-without-mfa fixture must fail; a lone MFA envelope must not pass. |
| ISO pack rewritten or broken by new ids | AC 10; do not touch `frameworks/iso-27001/2022` IAM rows. |
| Provider names leak into IDs or fixture type strings | Validator + IAM-006/007. |
| Hybrid controls auto-pass from one technical fact | Honest automation class; approval/SoD/review cannot Effective without attestation. |
| ExceptionApproved never emitted; break-glass fixture can’t green | Reuse IR Exception; if evaluator lacks the arm, document runtime blocker — do not invent a second exception type. |
| Two SubjectSelector types confuse IAM tests | Consume Prompt 03 population API; do not add a third. |
| Typed vs string facts double system | Declare semantic types; store via Prompt 02 when present; no second enum. |
| Prompt 01 SSOT overwritten | This file is the IAM slice SSOT; `canonical-assurance-catalog-v1.md` is off-limits. |
| Baseline remains a CI green that asserts catalog absence | After target GREEN, ignore/delete/move baseline. |
| Secrets in identity fixtures (tokens, passwords) | Seal + IAM-014; fixtures use booleans/timestamps/ids only. |

---

## 8. Dual-suite and SDD protocol (implement phase)

Hard protocol (do not skip):

```text
Spec (this file) → Baseline GREEN on CURRENT code → Target RED on CURRENT code
  → Implement IAM catalog content only → Docs/ADR finalize if needed
  → Target GREEN → Prove Baseline FAILS or is additive-documented
  → Supersede Baseline → Target still GREEN
```

Fail-closed if: baseline cannot go green on current characterization; target cannot go red for the **right** reason (missing IAM catalog / population semantics); or target never greens within max_iters.

Workspace verify command is unchanged. Record the implement-phase HEAD SHA in this document when product work starts (may differ from planning SHA if 01–03 merge first).

---

## 9. ADR

Architecture / public-contract decision: IAM content is a **canonical catalog family** (`control.identity.*`) consumed later by framework mappings, not an ISO-pack extension and not provider-prefixed checks.

Accepted: [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md).

---

## 10. Planning SHA record

```text
planning_sha = 5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b
branch       = main
note         = prompts 01–03 markdown present; catalog/canonical/v1 absent;
               CoverageAtLeast stub; ISO IAM sliver only
```

Implementers re-ran characterization against in-tree Prompt 01–03 outputs on this planning SHA (catalog fixture, typed `EvidenceValue`, `population.rs`) before adding IAM content.

---

## 11. Baseline suite record (superseded)

| Field | Value |
| --- | --- |
| Planning SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| Suite | `sdd_iam_catalog_baseline` · [`tests/sdd/iam_catalog.baseline.rs`](../../tests/sdd/iam_catalog.baseline.rs) |
| Expected now | **ignored** (`#[ignore = "superseded by sdd_iam_catalog_target"]`) so absence-of-catalog is not CI green |
| Command | `cargo test --workspace --features demo --test sdd_iam_catalog_baseline` |

Baseline asserts (found case):

- no `catalog/canonical/v1`, no `CanonicalCatalog` type/crate
- no `control.identity.*` / `evidence.identity.*` / `test.identity.*` in product or ISO pack
- ISO pack still ships `access.mfa.privileged`, `access.least-privilege`, `access.periodic-review`, `personnel.access-termination` wired to GitHub/policy evidence
- `test.access.mfa.privileged` is an existence check on `source.admin.permissions` (one envelope → Effective even if MFA-false identity facts exist)
- pack TOML `kind = "hybrid"` loads as `PlannedTestKind::Automated`
- `CoverageAtLeast` always `PartiallyEffective`; no `AllSubjects`
- `Effectiveness::ExceptionApproved` exists; `evaluate` never emits it
- `Identity` / `SubjectKind` remain thin (`ServiceAccount` absent)
- `EvidenceObservation.facts` is `BTreeMap<String,String>`; `parse_fact` coerces
- GitHub advertises `source.admin.permissions` but `collaborators.rs` is a stub; no Entra/Okta/Workspace collectors; no IAM fixtures

---

## 12. Target suite record (GREEN)

| Field | Value |
| --- | --- |
| Suite | `sdd_iam_catalog_target` · [`tests/sdd/iam_catalog.target.rs`](../../tests/sdd/iam_catalog.target.rs) |
| Expected | **GREEN** (CI gate) |
| Command | `cargo test --workspace --features demo --test sdd_iam_catalog_target` |
| Landed catalog | `catalog/canonical/v1/{controls,evidence,tests}/identity.toml` listed in `manifest.toml` |
| Landed fixtures | `fixtures/assurance/canonical/v1/identity/{healthy-org,privileged-without-mfa,inactive-admin-active,terminated-employee-active,service-account-without-owner,partial-inventory,stale-access-review,break-glass-approved-exception}/` |
| Loader | Prompt 01 `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — no second loader |
| Population | Prompt 03 `evaluate_coverage` / identity inventory resolution — no `IamPopulation` fork |

Implement-phase note: workspace HEAD at implement was still `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` plus in-tree Prompt 01–03 outputs (`catalog/` fixture, typed `EvidenceValue`, `population.rs`). IAM content consumes those contracts.

---

## 13. Landed record

| Surface | Location |
| --- | --- |
| Controls (23) | `catalog/canonical/v1/controls/identity.toml` |
| Evidence (12) | `catalog/canonical/v1/evidence/identity.toml` |
| Tests (23) | `catalog/canonical/v1/tests/identity.toml` |
| Manifest listing | `catalog/canonical/v1/manifest.toml` `[files]` |
| Fixtures (8) | `fixtures/assurance/canonical/v1/identity/<name>/evidence.json` |
| Loader / digest | Prompt 01 crate; no IAM-specific load path |
| Target suite | `tests/sdd/iam_catalog.target.rs` (`sdd_iam_catalog_target`) GREEN IAM-001…016 |
| Baseline suite | `tests/sdd/iam_catalog.baseline.rs` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md) |
| ISO pack | Unchanged `access.*` / `personnel.access-termination` ids and mappings |
| Collectors | No Entra/Okta/Workspace/GitHub-identity collector |

`assurance catalog stats` after this family (including the Prompt 01 protected-branch fixture):

```text
schema: weeping-angel/canonical-catalog/v1
catalog: canonical
version: 1
controls: 24
evidence: 13
tests: 24
digest: wa:canonical-catalog:weeping-angel/canonical-catalog/v1:232dfad3868fb66db5775fd9e174d2198824e8254bd1f7a66a448d611c29d2dc
```

Digest is Prompt 01 `CatalogDigest` over parsed documents. It changes if catalog TOML changes; it is not mixed with the `wa:canonical-catalog:…` prefix in the hash input.

Protocol:

```text
Spec (this file) → Baseline GREEN on planning characterization
  → Target RED for missing IAM family / population fixtures
  → Implement identity TOML + fixtures + catalog-crate validator denylist only as needed
  → ADR / contract / README finalized
  → Target GREEN → Baseline skip-superseded → Target still GREEN
```

Fail-closed gates that were met: baseline characterized absence; target went red for missing `control.identity.*` / fixtures / population semantics; target greened on the Prompt 01 loader and Prompt 03 evaluator without an IAM fork.

