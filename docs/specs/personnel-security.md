# SDD: Personnel Security Lifecycle (Operational ISMS v1 Prompt 17)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_personnel_security_target` GREEN (17); baseline skip-superseded (12 ignored) |
| Program | Operational ISMS v1 — Prompt 17 personnel security |
| Prompt | [`docs/prompts/operational-isms-v1/17-personnel-security.md`](../prompts/operational-isms-v1/17-personnel-security.md) |
| Slice | Operationalize personnel-security **joiner / mover / leaver** lifecycle using provider-neutral identity/personnel evidence and population-aware control tests. Do **not** turn the generic identity model into an HRIS. |
| Dual-suite | `sdd_personnel_security_baseline` · `sdd_personnel_security_target` (`tests/contracts/personnel_security.{baseline,target}.rs`) — **not auto-discovered** (I3); registered in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md) (`0003-*` sibling; **0004** is documentation architecture). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — personnel lifecycle pointer |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Population (consumed) | [`docs/specs/population-runtime.md`](population-runtime.md), ADR [`0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md) |
| IAM sibling (consumed, do not rewrite) | [`docs/specs/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| Governance sibling (consumed, do not rewrite) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) |
| Prompt 12 (consume; do not implement) | [`docs/specs/controlled-documents.md`](controlled-documents.md) — policy ack uses `artifact_ref`. Sibling `DocumentRef` / `ControlledDocument` may exist; PER-007 greps `identity.rs` / `subject.rs` only. |
| Prompt 10 (consume; do not implement) | [`docs/specs/control-implementation-registry.md`](control-implementation-registry.md) — consume `ControlImplementation` as-is; no CIR expansion |
| Collision fence | Prompt 15 events/drift [`isms-events-drift.md`](isms-events-drift.md); Prompt 16 remediation [`remediation-engine.md`](remediation-engine.md); GitHub collector; ISO pack `personnel.access-termination`; IAM / governance dual-suites |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evidence schema | `evidence/v1` (`EVIDENCE_SCHEMA`) — facts, never conclusions |
| JSON | `#[serde(rename_all = "camelCase")]` |
| Digest | SHA-256 / `canon/v1`; **no** random v4 in persisted identity |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; keep `sdd_iam_catalog_target`, `sdd_governance_catalog_target`, `sdd_population_runtime_target` GREEN |

This document is the durable human SSOT for Operational ISMS v1 Prompt 17. It owns **personnel population membership**, **lifecycle evidence** (screening, commitments, training, acknowledgements, provisioning, review, role change, offboarding, disablement, asset-return **references**), and **population-honest control tests** across complete in-scope populations.

It does **not** own payroll, recruiting, employee profile UI, an HR system of record, IAM technical MFA/privileged-membership content, ISO remapping, Prompt 15 event emission, Prompt 16 remediations, Prompt 10 CIR expansion, or Prompt 12 document-registry types.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Personnel security is **not** an HRIS and **not** a second identity model:

```text
who the principal is                 = Identity { id, kind, displayName }  (thin IR; no Employee/Contractor kinds)
which population they belong to      = inventory.subject tags + evidence.personnel.population-membership
what happened in the lifecycle       = evidence.identity.lifecycle-event + personnel evidence
whether access is still live         = evidence.identity.account-status / role-membership (facts)
whether the control is effective     = ControlTestResult.effectiveness (tests only)
```

One trained user must **never** prove training coverage. A non-authoritative inventory must **never** yield `Effective` on an all-subjects test. Collectors **normalize facts**; they never emit personnel-compliance conclusions.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). [`docs/sdd/`](../sdd/README.md) is a pointer stub only.

---

## 0. Collision fence (concurrent SDD)

Sibling SDD runs for Prompt 15 (events/drift) and Prompt 16 (remediation) may still be in flight. **Do not fork those models.** Personnel lifecycle facts are **optional later inputs** to events/remediation; this slice does not emit `IsmsEvent` or create `Remediation` records.

This slice may add catalog personnel files, personnel fixtures, dual-suite contracts, and (if needed) additive boolean facts on **existing** `evidence.identity.*` / `evidence.personnel.*` types. It must not rewrite sibling suites or the GitHub collector.

| Do not touch | Owner |
| --- | --- |
| `docs/specs/isms-events-drift.md`, `tests/contracts/isms_events_drift.*`, `docs/adr/*isms-events*` | Prompt 15 events/drift |
| `docs/specs/remediation-engine.md`, `tests/contracts/remediation_engine.*`, `docs/adr/*remediation*` | Prompt 16 remediation |
| `tests/contracts/iam_catalog.*`, `catalog/canonical/v1/{controls,evidence,tests}/identity.toml` (except documented additive optional facts) | IAM catalog |
| `tests/contracts/governance_catalog.*`, existing five `control.personnel.*` rows in `governance.toml` | Governance catalog — **keep the five rows**; do not retarget their ids |
| `frameworks/iso-27001/2022/**`, `personnel.access-termination`, `tests/contracts/iso27001_*` | ISO pack / remap |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| `Identity` / `IdentityKind` / `SubjectKind` Employee or Contractor variants | Forbidden HRIS extension |
| `resolve_personnel_inventory` (or any second population resolver) | Population runtime owns resolution |
| Prompt 10 / Prompt 12 product types (`DocumentRef`, `ControlledDocument`, CIR field expansion) | Those prompts |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Additive personnel controls / evidence / tests | `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml` listed in `manifest.toml` |
| Deterministic fixtures | `fixtures/assurance/canonical/v1/personnel/<name>/` |
| Dual-suite | `tests/contracts/personnel_security.{baseline,target}.rs` + root `[[test]]` |
| Population evaluation | **Reuse** `weeping-angel-control-test` `AllSubjects` / `NoneSubjects` / `CoverageAtLeast` / `CountWhere` / `ManualReview` / `FreshWithin`. **No** `resolve_personnel_inventory`. |
| Identity / Exception / SubjectSelector | Reuse IR as shipped |
| Future HRIS / IdP / LMS / MDM normalize | `weeping-angel-collector` **only** if a fixture-normalizer is required; **no** live provider SDK; **no** dep on control-test or framework |
| Evidence crate | **Conclusion-free.** Seal still rejects `looks_like_compliance_claim`. |

Tiny allowed adjustments at implement: additive catalog TOML + manifest lines; additive optional boolean facts (`active`, `excessive`, `within_grace`, `member`) on declared types; new `evidence.personnel.*` types referenced by a control **and** a test (no orphans); dual-suite + `Cargo.toml` `[[test]]`; public-contract pointer. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add `Employee` / `Contractor` kinds. Do **not** add a collector dependency to `weeping-angel-control-test` or `weeping-angel-framework` (already collector-free — keep it).

GOV-003 counts every `control.personnel.*` in the loaded catalog and requires the governance-family slice stay in **30–45**. This slice may add **at most six** new `control.personnel.*` ids (34 + 6 = 40). Do not split micro-controls.

---

## 1. Problem / user-visible goal

Weeping Angel can already **name** personnel-security controls and run two kinds of check:

1. **Governance process** — `test.personnel.jml-process-attested` is `op = "manual-review"`. A process attestation (or its absence) is all it can say.
2. **All-subjects current flags** — awareness / role-specific training / confidentiality / policy acknowledgement require `current=true` on every in-scope `user`. That is real population math, but it only covers “is the flag current?”, not joiner grace, movers who kept excessive roles, leavers who still have access, screening, or asset return.

IAM owns the **technical** JML sliver (`control.identity.joiner-mover-leaver`, `terminated-user-removal`, `access-revocation-timeliness`) wired to `evidence.identity.lifecycle-event`. Those tests are hybrid/manual-review or a `none-subjects` check on the string field `status` — not a population-honest joiner / mover / leaver lifecycle. One `lifecycle-event` envelope does not prove the in-scope population completed join, move, or leave.

There is no way today to say, honestly:

```text
every required person has current training
this one user is overdue (current=false) — Ineffective, named
this new joiner is inside the grace window — not treated as overdue
this leaver still has active access — Ineffective
this mover still holds excessive privileges — Ineffective
this exception expired — fail is not suppressed
there is no authoritative personnel source — never Effective
screening was recorded as a fact, not as “cleared / compliant”
```

**User-visible goal:** Weeping Angel can continuously test personnel-security lifecycle controls **honestly across complete in-scope populations** — employees, contractors, privileged personnel, developers, finance / security / executive groups, and organization-defined populations — without becoming an HRIS.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Identity` | `weeping-angel-assurance-ir::identity` | **Keep thin.** `{ id, kind, displayName }`. `IdentityKind` = `User \| Service \| ServiceAccount \| Team \| Role \| Other`. Do **not** add `Employee` / `Contractor`. |
| `SubjectKind` | `subject.rs` | Already has `User`, `Identity`, `PrivilegedIdentity`. Org populations via `SubjectSelector.tags` + `inventory.subject` facts. Do **not** add HR kinds. |
| `SubjectSelector` | IR SSOT `{ kind, ids, tags, scope }` | Only selector. Control-test keeps the thin `{ kind, id }` adapter. No third type. |
| Population resolver | `weeping-angel-control-test::population` | Order: explicit `EvidenceSet` population → closed selector ids → `evidence.identity.inventory` → `inventory.subject` + `inventory.complete` → inferred (**Unknown**). **Do not** add `resolve_personnel_inventory`. |
| `Exception` | `exception.rs` | Already has `subjects` + `expires_at`. Empty `subjects` ≠ whole inventory. Reuse `ExceptionApproved` honesty (remaining-all-pass excepted sets). |
| `ControlImplementation` | `implementation.rs` | Thin (`status`, `applies_to`, exception/risk ids). Prompt 10 CIR expansion is **not** this slice. **Consume as-is.** |
| `DocumentRef` / `ControlledDocument` | Sibling Prompt 12 may define them (`implementation.rs` / `document.rs`) | Policy acknowledgement uses `artifact_ref` strings. Do not implement the document registry here. PER-007 greps **`identity.rs` / `subject.rs` only** so sibling types do not fail this slice. |
| IAM JML | `identity.toml` | Leave `control.identity.{joiner-mover-leaver,terminated-user-removal,access-revocation-timeliness}` and `test.identity.{jml-events-recorded,no-terminated-active-accounts,revocation-timely}` in place. Personnel tests **compose** those facts; they do not retarget IAM ids. |
| Governance personnel | `governance.toml` | Keep the five existing `control.personnel.*` rows and their five tests. Additive lifecycle ids go in `personnel.toml`. |
| Collectors | `weeping-angel-collector` | `github` + `local` only on this HEAD. Live HRIS/IdP/LMS/MDM adapters are **out of scope**. Emit contract is specified so later modules normalize only. |
| Control-test / framework | Cargo.toml | **Stay collector-free.** Framework already depends only on IR. |
| Seal | `looks_like_compliance_claim` | Collectors and `EvidenceEnvelope::seal` already reject compliance narratives. Personnel fixtures/narratives must not trip or weaken this. |

Serde / identity law:

- camelCase JSON
- SHA-256 digests (`canon/v1`)
- no random v4 in persisted catalog, fixture, or IR identity
- `#[serde(default)]` on any additive IR field (this slice should not need IR changes)

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Executable characterization lives in `sdd_personnel_security_baseline` (`#[ignore = "superseded by sdd_personnel_security_target"]` after target GREEN). This section remains the **pre-lifecycle** contract on SHA `6e31bf1`: five governance personnel rows, two evidence types, IAM technical JML, thin `Identity`, GitHub+local collectors, no `personnel/` fixtures.

This file and the ADR exist in-tree. Baseline tests **must not** assert that these markdown paths are missing.

### 3.1 Five `control.personnel.*` rows (governance family)

`catalog/canonical/v1/controls/governance.toml` ships exactly these personnel controls:

| Control | Automation | Evidence | Test | Expression |
| --- | --- | --- | --- | --- |
| `control.personnel.security-awareness` | hybrid | `evidence.personnel.training` | `test.personnel.awareness-current-all` | `all-subjects` / `current` / `kind = user` |
| `control.personnel.role-specific-training` | hybrid | `evidence.personnel.training` | `test.personnel.training-current-all` | `all-subjects` / `current` / `kind = user` |
| `control.personnel.onboarding-offboarding` | **manual** | `evidence.manual.attestation` | `test.personnel.jml-process-attested` | `op = "manual-review"` |
| `control.personnel.confidentiality-commitment` | hybrid | `evidence.personnel.acknowledgement` | `test.personnel.confidentiality-acknowledged-all` | `all-subjects` / `current` / `kind = user` |
| `control.personnel.policy-acknowledgement` | hybrid | `evidence.personnel.acknowledgement` | `test.personnel.policy-acknowledged-all` | `all-subjects` / `current` / `kind = user` |

There is **no** `control.personnel.{screening,joiner-grace,role-change,leaver-access,asset-return}` (or equivalent) in the loaded catalog.

### 3.2 Personnel evidence types

`catalog/canonical/v1/evidence/governance.toml` declares only:

```text
evidence.personnel.training
evidence.personnel.acknowledgement
```

Facts on those types follow the governance shared attestation shape plus `trained_at` / `training_kind` and `acknowledged_at` / `ack_kind`. There are **no** catalog types:

```text
evidence.personnel.screening
evidence.personnel.asset-return
evidence.personnel.joiner-grace
evidence.personnel.population-membership
```

### 3.3 IAM technical JML is not population-honest lifecycle

| Control | Test | Kind | What it actually evaluates |
| --- | --- | --- | --- |
| `control.identity.joiner-mover-leaver` | `test.identity.jml-events-recorded` | hybrid | `op = "manual-review"` — events recorded as a governance act, not all joiners/movers/leavers |
| `control.identity.terminated-user-removal` | `test.identity.no-terminated-active-accounts` | automated | `none-subjects` on `evidence.identity.account-status` field `status` (string `active`/`terminated` is **Technical**, not a boolean fail) |
| `control.identity.access-revocation-timeliness` | `test.identity.revocation-timely` | hybrid | `op = "manual-review"` |

`evidence.identity.lifecycle-event` exists (`event` ∈ `joined` \| `moved` \| `left` \| `role-changed` \| `access-approved` \| `access-revoked`, plus `occurred_at`, `approved?`). The IAM fixture `terminated-employee-active` proves a **single** `left` + still-`active` account, not a personnel-population leaver test.

### 3.4 Identity IR stays thin

```text
Identity        = { id, kind, displayName? }
IdentityKind    = User | Service | ServiceAccount | Team | Role | Other
SubjectKind     = … User | Identity | PrivilegedIdentity …   // no Employee, no Contractor
SubjectSelector = { kind, ids, tags, scope }
```

Organization-defined groups are **not** first-class kinds. Tags on `SubjectSelector` / `inventory.subject` are the intended extension point. This HEAD does not evaluate personnel groups as populations beyond `kind = user` and IAM privileged/service special-cases.

### 3.5 Collectors

`weeping-angel-collector` exports `github` and `local` only (`src/{github,local}`). There is no `hris`, `idp`, `lms`, or `mdm` module. Collectors already refuse `looks_like_compliance_claim` narratives and cannot set `Effectiveness`. `weeping-angel-control-test` and `weeping-angel-framework` have **no** collector dependency.

### 3.6 Prompt 12 / Prompt 10 (characterization SHA)

On SHA `6e31bf1`: `ControlImplementation` had no `document_refs` / review cadence; there was no `DocumentRef` / `ControlledDocument` Rust type; `Exception` already had `subjects` + `expires_at`.

This slice still consumes those types **as they exist** and does not implement CIR or the document registry. On later HEADs, PER-007 greps `identity.rs` / `subject.rs` so sibling Prompt 12 types in `document.rs` / `implementation.rs` do not fail this personnel slice.

### 3.7 Existing related fixtures (do not rewrite)

| Path | What it proves today |
| --- | --- |
| `fixtures/assurance/canonical/v1/governance/incomplete-training-population/` | Authoritative 2 users; one training envelope → `training-current-all` is `InsufficientEvidence` (GOV-009) |
| `fixtures/assurance/canonical/v1/governance/current-documents/` | Complete current training population can be `Effective` |
| `fixtures/assurance/canonical/v1/governance/{approved,expired}-exception/` | IR Exception honesty for governance |
| `fixtures/assurance/canonical/v1/identity/terminated-employee-active/` | IAM leaver+active fact pair |
| `fixtures/assurance/canonical/v1/identity/partial-inventory/` | Non-authoritative identity inventory |

There is **no** `fixtures/assurance/canonical/v1/personnel/`.

### 3.8 Dual-suite registration

Root `Cargo.toml` has **no** `sdd_personnel_security_baseline` / `sdd_personnel_security_target` `[[test]]` rows. `tests/contracts` is not auto-discovered.

### 3.9 What “personnel assessment” means today

A caller can load the canonical catalog and run `test.personnel.training-current-all` against a governance fixture. It **can** fail a missing trainee on an authoritative `inventory.subject` set. It **cannot**:

- honor a new-joiner grace window;
- fail a leaver who still has active access as a personnel-lifecycle result (without pretending IAM `status` strings are booleans);
- fail a mover who retained excessive privileges;
- require screening facts where screening applies;
- record asset-return references;
- treat a missing personnel source as anything other than whatever inventory the generic resolver inferred.

The baseline suite therefore characterizes **presence of the five governance personnel rows + two evidence types + IAM technical JML**, and **absence of lifecycle-honest personnel tests / evidence / fixtures**.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

Additive personnel lifecycle content lands beside the governance family, **without moving** the five existing rows:

```text
catalog/canonical/v1/
  manifest.toml                          # add personnel.toml listings only
  controls/personnel.toml                # new lifecycle controls only
  evidence/personnel.toml                # new evidence.personnel.* types only
  tests/personnel.toml                   # new tests only
  {controls,evidence,tests}/governance.toml   # UNCHANGED existing five personnel rows
  {controls,evidence,tests}/identity.toml     # UNCHANGED IAM JML rows
```

If a one-line manifest listing is the only `manifest.toml` edit required, that is allowed. Do **not** create a second catalog loader.

Deterministic fixtures (fixed `collectedAt`, preferred `2026-08-18T12:00:00Z` unless a grace/overdue case needs a documented second clock):

```text
fixtures/assurance/canonical/v1/personnel/
  complete-training-population/
  one-overdue-user/
  new-joiner-grace/
  leaver-with-active-access/
  mover-retaining-excessive-privileges/
  expired-exception/
  missing-personnel-source/
  manual-screening-evidence/
```

Each directory contains frozen `evidence.json` (+ optional `exceptions.json`). Fixtures emit canonical `evidence.personnel.*` / `evidence.identity.*` / `inventory.subject` / `inventory.complete` / `evidence.manual.attestation` only. No `source.*`. No HRIS/LMS product type ids. No secret material. No compliance narratives.

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.personnel.<slug>
evidence.personnel.<slug>
test.personnel.<slug>
```

Reuse (do not fork) existing:

```text
evidence.identity.{inventory,lifecycle-event,account-status,role-membership,access-review,privileged-membership}
inventory.subject
inventory.complete
evidence.manual.attestation
```

Reject in new personnel catalog content (target greps **catalog TOML**, never the test source — I4a):

- provider / HRIS / LMS / MDM / IdP tokens as the subject of an id (`workday`, `bamboohr`, `okta`, `entra`, `knowbe4`, `intune`, `jamf`, ` Rippling`, …);
- framework tokens (`iso27001`, `soc2`, `nis2`, `dora`, `gdpr`);
- GRC tokens (`vanta`, `drata`);
- orphaned evidence or tests;
- existence-only encodings of population tests;
- `Employee` / `Contractor` as `SubjectKind` or `IdentityKind` names in catalog TOML.

Correct: `control.personnel.screening`. Incorrect: `control.workday.screening`, `test.iso27001.a.6.3`.

### 4.3 Control family (keep 5 + add ≤ 6)

**Keep unchanged** in `governance.toml`:

| Control | Role after this slice |
| --- | --- |
| `control.personnel.security-awareness` | All required personnel have current **awareness** training |
| `control.personnel.role-specific-training` | All required personnel have current **role-specific** training |
| `control.personnel.onboarding-offboarding` | Process attestation remains **manual** (`jml-process-attested`) |
| `control.personnel.confidentiality-commitment` | All required personnel have current confidentiality acknowledgements |
| `control.personnel.policy-acknowledgement` | All required personnel have current policy acknowledgements |

**Add** in `personnel.toml` (titles framework-neutral; honest automation):

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.personnel.screening` | Required personnel screening | Hybrid / manual | user / tagged population | `evidence.personnel.screening` | `test.personnel.screening-recorded` |
| `control.personnel.joiner-grace` | New-joiner grace | Hybrid | user (joiners) | `evidence.personnel.joiner-grace`, `evidence.identity.lifecycle-event` | `test.personnel.joiner-grace-honored` |
| `control.personnel.access-provisioning` | Access provisioning | Hybrid | user (joiners) | `evidence.identity.lifecycle-event`, `evidence.identity.account-status` | `test.personnel.joiner-access-provisioned` |
| `control.personnel.role-change` | Role-change least privilege | Hybrid | user (movers) | `evidence.identity.lifecycle-event`, `evidence.identity.role-membership` | `test.personnel.mover-privileges-reduced` |
| `control.personnel.leaver-access` | Leaver access removal | Automated | user (leavers) | `evidence.identity.lifecycle-event`, `evidence.identity.account-status` | `test.personnel.no-leaver-active-access` |
| `control.personnel.asset-return` | Asset-return reference | Hybrid | user (leavers) | `evidence.personnel.asset-return` | `test.personnel.asset-return-recorded` |

If implement must stay leaner, **merge** `access-provisioning` into `joiner-grace` (one control, two tests) so the added control count stays ≤ 6 and GOV-003 remains in 30–45. Do **not** add `control.personnel.periodic-access-review` — IAM already owns `control.identity.periodic-access-review`. Personnel fixtures may **consume** `evidence.identity.access-review` for movers/privileged subsets.

Each new control: stable id, title, objective, domain `personnelSecurity` (plus `accessControl` where access is the subject), evidence refs, test refs, honest `hybrid` \| `automated` \| `manual`.

**Do not invent technical automation** for screening quality, training effectiveness, or legal effect of an acknowledgement. Screening stays Hybrid/Manual: a recorded fact is not “the person is cleared.”

Sibling boundary:

| Topic | This slice | Not this slice |
| --- | --- | --- |
| Training / ack **population** honesty + grace | here | — |
| JML **process** attestation | existing governance row (keep) | — |
| Technical MFA / privileged membership / unique identities | — | IAM |
| Terminated-user removal as IAM control id | — | IAM (leave id); personnel adds **population-honest** leaver test |
| Periodic access review control id | — | IAM |
| Policy document registry | — | Prompt 12 |
| How the org implements the control | — | Prompt 10 CIR |
| Events / remediations from a leaver fail | later consumers | Prompt 15 / 16 |

### 4.4 Canonical evidence (facts, not conclusions)

Reuse existing personnel/identity types. **Add** only the types below (each referenced by a control and a test).

Shared personnel facts (store via `EvidenceValue::with_value`; camelCase on the envelope, snake_case fact keys matching IAM/governance convention):

| Fact | Type | Notes |
| --- | --- | --- |
| `subject_id` | String | Durable principal id — **not** a v4 |
| `population_id` | String? | Org-defined group (`employees`, `contractors`, `developers`, `finance`, `security`, `executive`, or org slug) |
| `current` | Bool? | Inside the catalog freshness window when the collector can derive it; evaluator still applies stale/fresh rules |
| `attested_by` / `attested_at` | String / Timestamp | Where the record is an attestation |

| Evidence type | Additional facts | Not allowed |
| --- | --- | --- |
| `evidence.personnel.training` (existing) | `trained_at`, `training_kind` (`awareness` \| `role-specific`), `current` | exam scores, PII dumps, “training control passed” |
| `evidence.personnel.acknowledgement` (existing) | `acknowledged_at`, `ack_kind` (`confidentiality` \| `policy`), `artifact_ref?`, `current` | legal conclusions; do not require Prompt 12 `DocumentRef` |
| `evidence.personnel.screening` | `screened_at`, `required` (bool), `recorded` (bool), `kind?` (`background` \| `other`) | `cleared`, `compliant`, `pass` as a compliance sentence; no criminal-record payload |
| `evidence.personnel.joiner-grace` | `joined_at`, `grace_until`, `within_grace` (bool) | “joiner control effective” |
| `evidence.personnel.population-membership` | `population_id`, `member` (bool), `tags?` (string list) | HRIS employee-object dumps as type id |
| `evidence.personnel.asset-return` | `returned` (bool) or `recorded` (bool), `returned_at?`, `asset_ref?` | device serial dumps as secrets; “offboarding certified” |

Additive **optional** facts on existing IAM types (do not remove existing fields; do not rewrite IAM tests):

| Type | Additive fact | Why |
| --- | --- | --- |
| `evidence.identity.account-status` | `active` (bool, **defect flag**) | Population predicates cannot use string `status` (`classify_str` treats `active` as Technical). Truthy `active` is `failing` so `none-subjects` names still-live leavers. |
| `evidence.identity.role-membership` | `excessive` (bool, **defect flag**) | Mover test needs a boolean, not an org-specific role-name matrix. Truthy `excessive` is `failing`. |

Polarity is implemented in `classify_predicate` (`weeping-angel-control-test` `population.rs`). Only `active` and `excessive` invert. `current`, `recorded`, `within_grace`, `member`, and `returned` keep the default truthy-pass table.

`evidence.identity.lifecycle-event` remains the join/move/leave fact (`joined` / `moved` / `left` / `role-changed`). Personnel tests select those subjects via:

1. explicit fixture `EvidenceSet` population / selector `ids`, or
2. `inventory.subject` tags (`lifecycle=leaver`, `lifecycle=joiner`, `lifecycle=mover`, `population=contractor`, …), or
3. `evidence.personnel.population-membership` + generic inventory — **still** resolved by the existing `resolve_population` path.

Seal rules still apply. Collectors and fixtures must not emit `looks_like_compliance_claim` text (`iso 27001 compliant`, `audit passed`, `control test result`, …).

### 4.5 Tests (population-based; required scenarios)

Keep existing five tests. Add:

```text
test.personnel.screening-recorded
test.personnel.joiner-grace-honored
test.personnel.joiner-access-provisioned     # optional if merged into joiner-grace
test.personnel.mover-privileges-reduced
test.personnel.no-leaver-active-access
test.personnel.asset-return-recorded
```

**Forbidden encoding:** `Exists(evidence.personnel.training)` as the body of any training-coverage test. One trained user never proves coverage.

**Forbidden encoding:** `Exists(evidence.identity.lifecycle-event)` as the body of joiner / mover / leaver tests.

Semantics (authoritative intent; exact `TestExpr` spelling uses existing catalog ops `all-subjects` / `none-subjects` / `coverage-at-least` / `count-where` / `fresh-within` / `manual-review`):

| Test | Population | Pass | Fail | Missing | Stale | Manual / exception / grace |
| --- | --- | --- | --- | --- | --- | --- |
| `training-current-all` / `awareness-current-all` (existing) | all required personnel (authoritative user inventory) | every subject `current=true` | ≥1 subject `current=false` (**overdue**) | known person lacks envelope → `InsufficientEvidence` | stale `trained_at` → `StaleEvidence` | Partial/Unknown inventory → **never** `Effective` |
| `joiner-grace-honored` | subjects with `lifecycle-event=joined` (or tag `lifecycle=joiner`) | every such subject has `within_grace=true` **or** current training | joiner past `grace_until` without current training | joiner missing grace **and** training envelopes | stale `joined_at` | In-window joiner **must not** be classified as overdue on **this** test. Grace is **not** an IR `Exception`. |
| `no-leaver-active-access` | subjects with `lifecycle-event=left` (or tag `lifecycle=leaver`) | none have `active=true` | leaver + `active=true` → `Ineffective` naming the subject | leave event without account-status | stale status after leave | Unexpired subject-bound Exception → `ExceptionApproved` for that subject only |
| `mover-privileges-reduced` | subjects with `lifecycle-event=moved` / `role-changed` | none have `excessive=true` | mover + `excessive=true` → `Ineffective` | move event without role-membership | stale membership | SoD quality stays IAM/manual; this test is the boolean excess fact |
| `screening-recorded` | subjects for whom screening `required=true` (org-defined tagged population) | each has `recorded=true` (and dated `screened_at`) | `required=true` and `recorded=false` | required subject missing screening envelope | stale `screened_at` | Quality of the screen is `manual-review` / Hybrid — a recorded fact is not “cleared” |
| `asset-return-recorded` | leavers, **where an asset-return reference is available** | `recorded=true` or `returned=true` | leaver with `returned=false` | no asset-return envelope: `InsufficientEvidence` or skip-with-rationale — **never** fake `Effective` | stale | Absence of MDM data is missing evidence, not a pass |
| `jml-process-attested` (existing) | organization | — | — | — | — | stays `ManualReviewRequired` until attestation exists |

`test.personnel.confidentiality-acknowledged-all` / `policy-acknowledged-all` stay all-subjects on `current`. They do not become Prompt 12 document evaluation.

Result metadata (`PopulationEvaluation`) must explain: population size, evaluated, passing, failing, missing, coverage, failing/missing/stale/excepted subject ids.

Unknown / non-authoritative personnel source **must not** produce `Effective` for any all-subjects / none-subjects / 100% coverage personnel test.

### 4.6 Populations (not HRIS kinds)

Represent employees, contractors, privileged personnel, developers, finance / security / executive, and org-defined groups **without** new `SubjectKind` / `IdentityKind` variants:

```text
inventory.subject facts:  kind=user, id=user:ada, tags.population=contractor, tags.group=finance
SubjectSelector.tags:     { "population": "contractor" } or { "group": "executive" }
evidence.personnel.population-membership: subject_id, population_id=contractors, member=true
privileged subset:        existing evidence.identity.privileged-membership + SubjectKind::PrivilegedIdentity
```

Tag matching already exists on IR selectors. If implement discovers tags are not applied to `inventory.subject` members, **adapt the fixture/selector** (explicit ids or `EvidenceSet::set_population`) rather than adding `resolve_personnel_inventory`. Do not grow `Identity`.

### 4.7 Exceptions and grace

| Mechanism | Use |
| --- | --- |
| IR `Exception` (`Approved`, unexpired, `subjects` bound) | Named carve-out (e.g. one person skipped from screening). Overall result **must not** be silent `Effective` — prefer `ExceptionApproved`. |
| `expires_at` in the past or `status=Expired` / `Revoked` | Does **not** suppress fail/missing. |
| Empty `subjects` | Does **not** except the inventory. |
| `evidence.personnel.joiner-grace` | Policy window for new joiners. **Not** an Exception. |

Reuse the shipped `evaluate_coverage` ExceptionApproved promotion. Do not add `PersonnelException`.

### 4.8 Fixtures (required eight; target RED for their absence)

| Fixture | Intent | Expected highlights |
| --- | --- | --- |
| `complete-training-population` | Authoritative N users; every one has current awareness **and** role-specific training | `training-current-all` and `awareness-current-all` → `Effective`. A single envelope must **not** pass if N>1. |
| `one-overdue-user` | Authoritative N; N−1 `current=true`; one `current=false` | `training-current-all` → `Ineffective` naming that subject. Missing ≠ fail. |
| `new-joiner-grace` | Authoritative N including one `lifecycle-event=joined` inside `grace_until` without training | `joiner-grace-honored` is **not** `Ineffective` for that joiner. Strict `training-current-all` remains honest (missing/false current). Past-grace twin (or same fixture documented clock) is overdue. |
| `leaver-with-active-access` | `lifecycle-event=left` + `account-status.active=true` | `no-leaver-active-access` → `Ineffective` naming the leaver. |
| `mover-retaining-excessive-privileges` | `lifecycle-event=moved` or `role-changed` + `role-membership.excessive=true` | `mover-privileges-reduced` → `Ineffective`. |
| `expired-exception` | Same gap as an approved carve-out would hide; `Exception` expired or `expiresAt` in the past | Fail/missing **not** suppressed. Not `ExceptionApproved`. |
| `missing-personnel-source` | Inventory absent, `inventory.complete` missing, or `authoritative=false` / identity inventory without org-level authoritative mark | All-subjects / none-subjects personnel tests → `InsufficientEvidence` or `Inconclusive`. **Never** `Effective`. |
| `manual-screening-evidence` | Required screening population; envelopes carry `recorded` / `screened_at` facts only | Screening test evaluates facts. Fixture narrative must not contain compliance claims. Quality remains Hybrid/Manual (`ManualReviewRequired` if the control’s quality arm is `manual-review`). |

Optional extra (not required for RED): `approved-exception` bound to one overdue trainee → `ExceptionApproved`, not silent Effective.

Do **not** reuse governance fixture names as the personnel suite’s required set. Governance fixtures stay owned by `sdd_governance_catalog_target`.

### 4.9 Collector / integration contract

HRIS, IdP, LMS, and MDM **later** collectors normalize to the types in §4.4. Rules (enforce in target even if no module lands):

1. Collectors emit `EvidenceEnvelope` facts. They **never** set `Effectiveness`, `ControlTestResult`, or `looks_like_compliance_claim` narratives.
2. `weeping-angel-control-test` and `weeping-angel-framework` stay **collector-free**.
3. No collector crate depends on control-test or framework.
4. This slice does **not** implement live Workday / Okta / KnowBe4 / Intune adapters and does **not** rewrite the GitHub collector.
5. If implement adds a `personnel` or `local` fixture-normalizer, it only maps frozen records → canonical types advertised on its descriptor.

### 4.10 Dual-suite protocol

`tests/contracts` is **not** autodiscovered. Implement **must** add, in the **same commit** as the `.rs` files (I3):

```toml
[[test]]
name = "sdd_personnel_security_baseline"
path = "tests/contracts/personnel_security.baseline.rs"

[[test]]
name = "sdd_personnel_security_target"
path = "tests/contracts/personnel_security.target.rs"
```

| Suite | Role |
| --- | --- |
| Baseline | GREEN on **current** tree characterizing §3. After target GREEN: `#[ignore = "superseded by sdd_personnel_security_target"]` then **re-prove** target GREEN. |
| Target | RED on current tree for **missing lifecycle honesty**, not harness noise; then the CI gate. |

Suggested target clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| PER-001 | Catalog loads additive `personnel.toml` (or equivalent listed files) offline via `CanonicalCatalog::load` |
| PER-002 | Digest remains deterministic after personnel files are listed |
| PER-003 | Existing five `control.personnel.*` ids still present; new lifecycle ids present; governance-family count still 30–45 |
| PER-004 | New evidence types declared as facts (screening, joiner-grace, population-membership, asset-return); no orphans; no conclusion phrases |
| PER-005 | Required tests declared; none are existence-only |
| PER-006 | Catalog TOML has no provider/HRIS/LMS/MDM/framework tokens (grep catalog files, **not** the test source) |
| PER-007 | `Identity` / `SubjectKind` still have no Employee/Contractor; no `resolve_personnel_inventory` symbol |
| PER-008 | IAM JML ids and `test.identity.jml-events-recorded` unchanged; ISO pack not rewritten |
| PER-009 | Complete training population → Effective; a single training envelope on N>1 is not Effective |
| PER-010 | One overdue user (`current=false`) → Ineffective naming the subject |
| PER-011 | New joiner inside grace is not overdue on `joiner-grace-honored` |
| PER-012 | Leaver with `active=true` → Ineffective |
| PER-013 | Mover with `excessive=true` → Ineffective |
| PER-014 | Expired exception does not suppress fail; approved unexpired bound exception is not silent Effective |
| PER-015 | Missing / non-authoritative personnel source → never Effective |
| PER-016 | Manual screening envelopes are facts; seal rejects compliance claims; control-test/framework have no collector dep |

Baseline clusters (current tree):

- five `control.personnel.*` only; tests as in §3.1
- only `evidence.personnel.{training,acknowledgement}`
- no `fixtures/assurance/canonical/v1/personnel/`
- IAM JML hybrid/manual-review + `none-subjects`/`status` as in §3.3
- Identity / SubjectKind lack Employee/Contractor
- collectors are github+local
- no `sdd_personnel_security_*` rows until implement adds them (do not assert Cargo.toml absence after those rows land)

**I4a:** never grep the target source for a token that also appears in the assertion string.

### 4.11 Documentation after implement

- This file: **Implemented**; dual-suite registered; baseline skip-superseded; target 17/17 GREEN.
- ADR: Accepted at [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md).
- [`docs/specs/assurance-runtime.md`](assurance-runtime.md): personnel lifecycle pointer (do not rewrite IAM/governance sections).
- Population runtime: document `active` / `excessive` defect-flag polarity only; do not fork coverage math.
- Spec registered in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS`.
- Do not overwrite IAM / governance / catalog-infrastructure SSOTs.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase.

1. Dual-suite `sdd_personnel_security_baseline` + `sdd_personnel_security_target` is registered in root `Cargo.toml` in the same commit as the `.rs` files.
2. On SHA `6e31bf1` (pre-lifecycle content): baseline GREEN characterizing §3; target RED for missing lifecycle honesty / personnel fixtures / new evidence types — not for unrelated compile or harness errors.
3. After implement: target GREEN; baseline skip-superseded with `#[ignore = "superseded by sdd_personnel_security_target"]`; target re-proven GREEN; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. Existing five `control.personnel.*` rows and their tests remain; additive lifecycle controls/tests exist; governance-family control count stays in 30–45.
5. Evidence types include existing `evidence.personnel.{training,acknowledgement}` plus `screening`, `joiner-grace`, `population-membership`, `asset-return`, declared as facts. IAM `evidence.identity.*` is reused. No `resolve_personnel_inventory`.
6. Required fixtures evaluate: complete training population; one overdue user; new-joiner grace; leaver with active access; mover retaining excessive privileges; expired exception; missing personnel source (never Effective); manual screening evidence (facts, not conclusions).
7. One trained user never proves training coverage. Partial/Unknown inventory never yields `Effective` on all-subjects / none-subjects / 100% personnel tests.
8. Exceptions stay subject-scoped with validity periods; expired/revoked do not pass; approved unexpired bound exceptions are `ExceptionApproved`, not silent `Effective`. Joiner grace is not an Exception.
9. `Identity` / `SubjectKind` gain no Employee/Contractor variants. Populations use tags / `population-membership` / existing privileged inventory.
10. Control-test and framework stay collector-free. No collector emits compliance conclusions. GitHub collector is not rewritten. No live HRIS/IdP/LMS/MDM adapter is required in this slice.
11. Prompt 10 CIR and Prompt 12 controlled documents are not implemented here. Prompt 15/16 models are not forked.
12. IAM / governance / population target suites stay GREEN. ISO pack `personnel.access-termination` / remap suites are not rewritten.
13. camelCase JSON, SHA-256 / `canon/v1` digests, no random v4 in persisted identity. Credential-shaped keys and compliance narratives remain rejected.

---

## 6. Out of scope

- Payroll, recruiting, employee profile UI, or an HR system of record.
- Live HRIS / IdP / LMS / MDM API adapters (Workday, Okta, KnowBe4, Intune, …).
- `Employee` / `Contractor` `SubjectKind` or `IdentityKind` variants.
- `resolve_personnel_inventory` or any second population evaluator.
- Rewriting IAM catalog TOML / `sdd_iam_catalog_*` or governance catalog TOML existing rows / `sdd_governance_catalog_*`.
- Rewriting ISO pack `personnel.access-termination` or ISO remap mappings.
- Rewriting the GitHub collector.
- Implementing Prompt 10 CIR or Prompt 12 `ControlledDocument` / `DocumentRef`.
- Forking Prompt 15 events/drift or Prompt 16 remediations (they may later consume personnel facts).
- Certification / “compliant” / “cleared” language on evidence.
- New crates; collector deps on control-test or framework.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Target goes red for missing `[[test]]` / compile noise | I3: register dual-suite in the same commit as `.rs`; RED must cite missing lifecycle honesty |
| GOV-003 count exceeds 45 | Cap additive `control.personnel.*` at six; merge provisioning into joiner-grace if needed |
| Governance/IAM suites break | Do not move existing five personnel rows; do not edit identity.toml tests; keep neighbor targets GREEN |
| String `status=active` mis-encoded as population fail | Additive boolean `active`; never rely on `classify_str` for lifecycle status |
| Grace modeled as Exception | §4.7; PER-011 uses `within_grace` facts |
| Silent Effective on excepted remainder | Reuse shipped ExceptionApproved promotion; expired fixture must still fail |
| Missing source treated as empty pass | PER-015; consume population Partial/Unknown refusal |
| HRIS kinds sneak into IR | PER-007; ADR forbids Employee/Contractor |
| Second population resolver | Explicit ban; reuse `resolve_population` only |
| Collectors declare compliance | Seal + collector isolation; no control-test dep |
| Prompt 15/16 fork | Collision fence; personnel facts only |
| I4a self-grep of provider tokens | Grep catalog TOML only |
| Baseline remains required-green absence after landing | skip-supersede then re-prove target GREEN |

---

## 8. Dual-suite and SDD protocol (implement phase)

Hard protocol (do not skip):

```text
Spec + draft ADR (this phase; no product feature code)
  → Register [[test]] rows + write suites (I3)
  → Baseline GREEN on CURRENT product tree
  → Target RED for missing lifecycle honesty (right reasons)
  → Implement catalog + fixtures (+ optional fixture-normalizer)
  → Docs/ADR accept + public-contract pointer if needed
  → Target GREEN
  → #[ignore = "superseded by sdd_personnel_security_target"]
  → Target still GREEN
  → cargo test --workspace --features demo
  → cargo fmt --all -- --check
  → cargo clippy --workspace --all-targets --all-features -- -D warnings
  → sdd_iam_catalog_target / sdd_governance_catalog_target / sdd_population_runtime_target GREEN
```

Fail-closed if: baseline cannot go green on current characterization; target cannot go red for the **right** reason; or target never greens within max_iters.

---

## 9. ADR

Architecture / public-contract decision: personnel lifecycle is **canonical `control.personnel.*` content plus population tests**, not an HRIS identity extension, not a second population resolver, and not IAM JML retargeting.

Accepted: [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md).

---

## 10. Characterization SHA record

```text
characterization_sha = 6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a
branch               = main
note                 = five control.personnel.* in governance.toml;
                       evidence.personnel.{training,acknowledgement} only;
                       IAM JML hybrid/manual-review + none-subjects/status;
                       Identity thin; collectors github+local;
                       Prompt 10/12 not implemented by this slice;
                       Exception already has subjects+expiresAt
```

---

## 11. Baseline suite record

| Field | Value |
| --- | --- |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Suite | `sdd_personnel_security_baseline` · `tests/contracts/personnel_security.baseline.rs` |
| After target GREEN | `#[ignore = "superseded by sdd_personnel_security_target"]` (12 tests) |
| Command | `cargo test --workspace --features demo --test sdd_personnel_security_baseline` |
| Landed | skip-superseded after target GREEN |

---

## 12. Target suite record

| Field | Value |
| --- | --- |
| Suite | `sdd_personnel_security_target` · `tests/contracts/personnel_security.target.rs` |
| Expected after implement | **GREEN** (CI gate; PER-001…016 / 17 tests) |
| Command | `cargo test --workspace --features demo --test sdd_personnel_security_target` |
| Landed | additive `personnel.toml` (6 controls / 4 evidence types / 6 tests) + eight fixtures + defect-flag polarity for `active` / `excessive` |

---

## 13. Landed record

| Surface | Location |
| --- | --- |
| Additive controls / evidence / tests | `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml` listed in `manifest.toml` |
| Controls | `control.personnel.{screening,joiner-grace,access-provisioning,role-change,leaver-access,asset-return}` |
| Evidence | `evidence.personnel.{screening,joiner-grace,population-membership,asset-return}` (plus existing training / acknowledgement) |
| Tests | `test.personnel.{screening-recorded,joiner-grace-honored,joiner-access-provisioned,mover-privileges-reduced,no-leaver-active-access,asset-return-recorded}` |
| Existing five personnel controls | `governance.toml` (unchanged) |
| Fixtures | `fixtures/assurance/canonical/v1/personnel/{complete-training-population,one-overdue-user,new-joiner-grace,leaver-with-active-access,mover-retaining-excessive-privileges,expired-exception,missing-personnel-source,manual-screening-evidence}/` |
| Evaluator polarity | `classify_predicate` in `crates/weeping-angel-control-test/src/population.rs` — `active` / `excessive` invert |
| Target suite | `tests/contracts/personnel_security.target.rs` (17 GREEN) |
| Baseline suite | `#[ignore = "superseded by sdd_personnel_security_target"]` (12 ignored) |
| ADR | Accepted at [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md) |
| PER-007 | `DocumentRef` / `ControlledDocument` grep scoped to `identity.rs` / `subject.rs` |
