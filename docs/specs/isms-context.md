# SDD: ISMS Context IR

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_isms_context_target` GREEN (CTX-T01–T14); baseline skip-superseded |
| Program | Operational ISMS v1 — ISMS context IR |
| Slice | Provider-neutral, framework-neutral operational ISMS context as the **root object** for continuous management-system operation. Extend the existing assurance IR. Do **not** create a parallel GRC schema. |
| Dual-suite | `sdd_isms_context_baseline` · `sdd_isms_context_target` (registered in root [`Cargo.toml`](../../Cargo.toml); directory is **not** auto-discovered) |
| Contract files | `tests/contracts/isms_context.{baseline,target}.rs` |
| ADR | Accepted [`docs/adr/0008-isms-context.md`](../adr/0008-isms-context.md). Cite by **path**. Concurrent Operational ISMS drafts also use `0008-*`. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (landed ISMS context section: durable IR document, not an assessment result) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Docs layout | [ADR 0004](../adr/0004-documentation-architecture.md) — this file is the human SSOT; traces go to `.sdd/runs/`; do **not** write traces to `docs/sdd/` |
| Neighbors (consume / pin; do not implement here) | [`risk-methodology.md`](risk-methodology.md) (scoring engine), [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md) (when jobs run), [scope engine](scope-engine.md) (landed `ScopeResolution`; this slice stays the named handle), [interested parties / obligations](interested-parties-obligations.md) (registry; this slice keeps membership stubs), security-objectives engine |
| Neighbor baselines (skip-superseded) | `sdd_scope_engine_baseline` `scp_b09`; IPO baseline IsmsContext-absence comment; methodology/scheduler absence tests remain ignored on their own suites. Do **not** use those suites as this slice’s tests. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonicalization | `canon/v1` via `weeping_angel_assurance_ir::canonical_digest` / `typed_canonical_digest` (compact serde JSON; struct field order; `BTreeMap` / `BTreeSet`) |
| Workspace verify (after implement) | `cargo test --test sdd_isms_context_baseline`; `cargo test --test sdd_isms_context_target`; keep `sdd_compliance_ir_target` GREEN; `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable human SSOT for **ISMS context IR**. It owns the **canonical root `IsmsContext`**, **organization identity** (including business units), **management-system scope reference**, **internal/external issues**, **interested-party and obligation records sufficient for the context graph**, **security-objective definitions**, **risk-methodology reference**, **governance cadence**, and **lifecycle status** with fail-closed validation.

It does **not** own scope resolution (`ScopeResolution`), obligation mapping engines, objective measurement/status projection, risk scoring, persistence, CLI, framework packs, or UI.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

`IsmsContext` is a **durable management-system definition** that later scope, risk, governance, audit, and readiness work hang off. It is **not** a point-in-time `AssessmentDefinition`, not a compiled SoA, and not a collector document.

```text
IsmsContext  → Organization → ManagementSystemScope
             → InterestedParty → Obligation
             → SecurityObjective
             → RiskMethodologyId          (reference only)
             → AssetId / VendorId / IdentityId populations (existing IR ids)
```

### Landed surface

| Item | Home |
| --- | --- |
| `IsmsContext` + org/issue/party/obligation/objective/cadence/lifecycle | [`crates/weeping-angel-assurance-ir/src/isms.rs`](../../crates/weeping-angel-assurance-ir/src/isms.rs) |
| Typed ids (`IsmsContextId`, `OrganizationId`, `BusinessUnitId`, `ScopeId`, `IssueId`, `InterestedPartyId`, `ObligationId`, `ObjectiveId`) | `id.rs` via existing `typed_id!` |
| `RiskMethodologyId` | reused from the risk-methodology slice (not redefined) |
| `ValidateIr for IsmsContext`, `validate_assessment_against_context`, `explain_isms_context` | `isms.rs` |
| Optional assessment pointer | `AssessmentDefinition.isms_context_id: Option<IsmsContextId>` (`#[serde(default)]`, snake_case) |
| Golden fixture | [`tests/fixtures/assurance-ir/v1/isms-context.json`](../../tests/fixtures/assurance-ir/v1/isms-context.json) |

Crate-root `InterestedParty` / `Obligation` are the **membership-graph** records on `IsmsContext`. The obligation registry (`party.rs` / `obligation.rs`) is a distinct type family that shares the same id newtypes.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub pointer only.

---

## 0. Collision fence (concurrent SDD)

This slice may add ISMS-context types, typed ids, `ValidateIr` for those types, one representative fixture, dual-suite registration, this spec, its ADR, and an additive `documentation_layout.rs` `CANONICAL_SPECS` entry.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/**`, `tests/contracts/github_collector.*` | Collectors |
| Kleene evaluator / `OrgContext` | [`applicability-engine.md`](applicability-engine.md) |
| `RiskMethodology` scales/matrices/`score_risk` | [`risk-methodology.md`](risk-methodology.md) — this slice stores a **typed id reference** only |
| `ScopeResolution`, inclusion/exclusion precedence | [Scope engine](scope-engine.md) (landed; consumes this root) |
| Obligation mapping / `RequirementSource` engine | [Interested parties / obligations](interested-parties-obligations.md) (landed `ObligationRegistry`) |
| `ObjectiveMetric` / measurement / `OnTrack` projection | Security-objectives engine (later) |
| Scheduler `Clock` / jobs / `weeping-angel isms run` | [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md) |
| Broad renames of `Asset`, `Vendor`, `Risk`, `Control`, `Requirement`, `Mapping`, `SubjectSelector`, evidence types | Frozen IR spine |
| Neighbor dual-suites as this slice’s tests | `sdd_risk_methodology_*`, `sdd_continuous_assurance_scheduler_*` |

Suggested **product** modules stay in **existing crates** (no new crate, no persistence crate, no CLI):

| Concern | Home |
| --- | --- |
| `IsmsContext` + org/issue/party/obligation/objective/cadence/lifecycle types | `crates/weeping-angel-assurance-ir/src/isms.rs` (name flexible; do not dump into `assessment.rs` or `identity.rs`) |
| Typed ids | `id.rs` via existing `typed_id!` |
| Validation | `validation.rs` + `ValidateIr` — fail closed, deterministic `IrValidationError` |
| Re-exports | `lib.rs` |
| Optional assessment **pointer** | `AssessmentDefinition.isms_context_id: Option<IsmsContextId>` with `#[serde(default)]` |
| Fixture | `tests/fixtures/assurance-ir/v1/isms-context.json` **in addition to** existing `assessment.json` |

Tiny allowed adjustments at implement: new IR module; `typed_id!` aliases; `lib.rs` re-exports; serde `camelCase` on new types; one optional/defaulted field on `AssessmentDefinition`; dual-suite `[[test]]` rows in root `Cargo.toml`; `CANONICAL_SPECS` entry.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add `reqwest` / provider SDKs to IR or framework. Do **not** implement `RiskMethodology` scoring types here.

If [`risk-methodology.md`](risk-methodology.md) has already landed `RiskMethodologyId`, **reuse it**. Do not define a second methodology identity type.

---

## 1. Problem / user-visible goal

The assurance IR today can describe a **point-in-time assessment** (`AssessmentDefinition`: requirements, controls, mappings, inventories) and compile it into control tests. It cannot describe the **management system that operates continuously**: who the organization is, which business units exist, which internal and external issues are in play, which interested parties and obligations the system answers to, which security objectives are declared, which risk methodology is in force, or how often governance repeats.

That gap forces later slices (scope, risk, audit, readiness) to invent a second compliance graph — a parallel GRC schema — or to stuff mutable assessment results into org notes and `AssessmentScope.organizations: Vec<String>`.

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- there is **no** `IsmsContext`, ISMS `Organization` record, `InterestedParty`, `Obligation`, `SecurityObjective`, or `RiskMethodology` type in product crates;
- `Identity` is an IAM principal (`User` / `Service` / …), not a legal entity;
- `AssetKind::Organization` / `SubjectKind::Organization` are inventory/selector kinds, not ISMS org identity;
- `AssessmentScope.organizations` is `Vec<String>`;
- `RequirementKind::ControlObjective` is a requirement classification, not a measurable ISMS objective.

**User-visible goal:** one canonical root model that ISO 27001 can use **now** and other frameworks can use **later**, without ISO clause numbers, Annex A semantics, or cloud-provider fields in the generic IR.

```text
construct → serialize → deserialize → validate → explain
one IsmsContext (one org, two business units, issues, parties, objectives, methodology ref)
```

A reviewer must be able to answer:

```text
which organization owns this ISMS?
which business units sit under it?
which management-system scope is referenced (not resolved)?
which issues are internal vs external?
which interested parties and obligations are in force?
which security objectives are declared (not scored)?
which risk methodology id is referenced (not computed)?
what is the governance cadence?
is the system Draft, Active, UnderReview, Retired, or Superseded?
```

Definition of done: Operational ISMS has **one** canonical root that later scope, risk, governance, audit, and readiness work can hang off **without another compliance graph**.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ASSURANCE_IR_SCHEMA` | `lib.rs` = `"assurance-ir/v1"` | **Do not fork.** Context documents carry this schema. No `grc-ir/v1`, no `isms-ir/v1`. |
| `AssessmentDefinition::new` | `assessment.rs` | **Must keep compiling and constructing.** New fields `#[serde(default)]` and initialized to empty/None. |
| Golden `tests/fixtures/assurance-ir/v1/assessment.json` | `sdd_compliance_ir_target` `ir_golden_fixtures_round_trip` | Must still decode + `validate()`. Do not add required keys. |
| `canonical_digest` / `typed_canonical_digest` / `typed_id!` / `validate_stable_id` | `digest.rs`, `id.rs` | **Reuse.** No second identity or digest system. No UUID v4. |
| `Identity` / `IdentityKind` | `identity.rs` | **Keep.** IAM principal. Do not reuse as the ISMS organization. |
| `Asset` / `Vendor` / `Risk` / `Control` / `Requirement` / `Mapping` / `SubjectSelector` / evidence types | existing modules | **No broad renames.** Population links are existing ids (`AssetId`, `VendorId`, `IdentityId`). |
| `AssessmentScope.organizations: Vec<String>` | `assessment.rs` | **Keep.** Scope engine may later bind strings to `OrganizationId`. This slice does not replace assessment scope. |
| IR-019 dangling `RiskId` | `validation.rs` | Unchanged. Context validation is additional, not a retarget. |
| `RiskMethodology` / scoring | [`risk-methodology.md`](risk-methodology.md) | **Absent on HEAD.** This slice may add `RiskMethodologyId` (`typed_id!`) for the reference. It must **not** add scales, matrices, or `score_risk`. |
| Framework crate | `weeping-angel-framework` | Stays network-free (no `reqwest`, octocrab, AWS/GitHub/Cloudflare SDKs). IR-only. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_isms_context_*` next to existing `sdd_*`. Contracts are **not** auto-discovered (`tests/contracts/` is not Cargo auto-discovery). |
| Docs layout | ADR 0004 | Human SSOT is this file. Implement may add this path to `sdd_documentation_layout` `CANONICAL_SPECS`. |

`AssessmentDefinition` remains the **assessment input**. `IsmsContext` is the **durable definition**. Optional pointer only:

```text
AssessmentDefinition.isms_context_id: Option<IsmsContextId>   // serde default None
```

Do **not** embed the full `IsmsContext` graph inside every assessment. Do **not** embed `Effectiveness`, residual scores, SoA rows, snapshot diffs, or control-test results inside `IsmsContext`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded later by `tests/contracts/isms_context.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is explicitly skip-superseded.

Neighbor suites already encode overlapping absences. Those found-cases are **not** this slice’s baseline. When this slice’s target is GREEN, implement **skip-supersedes only those neighbor found-cases** (`#[ignore]` + superseded comment) and leaves the rest of those suites intact.

### 3.1 No ISMS context types in product crates

`crates/**/src/**/*.rs` contain **none** of:

```text
struct IsmsContext
struct InterestedParty
struct Obligation          // IR type; English comments may still say "obligation"
struct SecurityObjective
struct ManagementSystemScope
struct ContextIssue
struct GovernanceCadence
struct BusinessUnit
RiskMethodology / RiskMethodologyId
IsmsLifecycleStatus
IsmsContextId / OrganizationId / InterestedPartyId / ObligationId / ObjectiveId / ScopeId
```

`Identity` exists and is an IAM principal. `AssetKind::Organization` and `SubjectKind::Organization` exist as **enum variants**, not as `struct Organization`.

Baseline must grep for `struct IsmsContext`, `struct InterestedParty`, `IsmsContextId`, `RiskMethodologyId` — not for the word `Organization` alone.

`lib.rs` re-exports assessment, asset, identity, vendor, risk, … It does **not** re-export an ISMS context module.

### 3.2 `AssessmentDefinition::new` is inventories-only

[`crates/weeping-angel-assurance-ir/src/assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs):

```text
AssessmentDefinition::new(id) →
  schema_version = ASSURANCE_IR_SCHEMA   // "assurance-ir/v1"
  requirements/controls/mappings/evidence_requirements/tests = []
  requests = AssessmentRequests::default()   // all false
  implementations = []
  scope = AssessmentScope { organizations: [], subjects: [], exclusions: [] }
  assets/identities/vendors/risks/exceptions/processing_activities = []
```

There is **no** `isms_context_id` field. `AssessmentDefinition` JSON uses **snake_case** keys (`schema_version`, `evidence_requirements`). Domain records such as `Asset` / `Identity` / `Risk` use `rename_all = "camelCase"`.

Golden fixture [`tests/fixtures/assurance-ir/v1/assessment.json`](../../tests/fixtures/assurance-ir/v1/assessment.json) decodes today. `sdd_compliance_ir_target` `ir_golden_fixtures_round_trip` asserts that.

### 3.3 Schema, ids, digests already exist (reuse, do not fork)

| Symbol | Value / behavior |
| --- | --- |
| `ASSURANCE_IR_SCHEMA` | `"assurance-ir/v1"` |
| `CanonicalizationVersion::CURRENT` | `"canon/v1"` |
| `canonical_digest` | SHA-256 hex of `serde_json::to_vec` |
| `typed_canonical_digest` | domain-separated `wa:assurance-ir:assurance-ir/v1:<type>:` + bytes |
| `typed_id!` | `try_new` via `validate_stable_id` (empty, too long, whitespace/control, illegal charset, UUID v4 → error) |
| `MAX_ID_LEN` | 256 |

Existing typed ids: `FrameworkId`, `RequirementId`, `ControlId`, `AssetId`, `IdentityId`, `VendorId`, `RiskId`, `ExceptionId`, `AssessmentId`, `MappingId`, … **No** `IsmsContextId` / `OrganizationId` / `RiskMethodologyId` on HEAD.

### 3.4 Validation today is assessment-graph only

`ValidateIr` is implemented for `AssessmentDefinition`. It checks schema version, duplicate requirement/control/evidence-requirement ids, dangling mappings/tests/implementations → controls, dangling implementation `RiskId` / `ExceptionId` (IR-019).

It does **not** reject duplicate `Risk` ids (membership `BTreeSet` only). This slice does **not** have to change that.

There is no `ValidateIr` for an ISMS context because the type does not exist.

### 3.5 Organization-shaped strings are not ISMS identity

- `AssessmentScope.organizations: Vec<String>` — untyped labels.
- Applicability `FactKey::OrganizationAttribute` — Kleene facts, not a legal-entity record.
- `Identity { id, kind, display_name? }` — principal.
- `Vendor { id, name }` — supplier node.

None of these carry business units, interested parties, issues, objectives, cadence, or lifecycle of a management system.

### 3.6 Framework crate is already network-free

[`crates/weeping-angel-framework/Cargo.toml`](../../crates/weeping-angel-framework/Cargo.toml) depends on `serde`, `serde_json`, `thiserror`, `toml`, `weeping-angel-assurance-ir` only. `sdd_assurance_runtime_target` `act_003_framework_crate_has_no_network_or_sdk_deps` locks this. Baseline of **this** slice may re-assert it; target **must**.

### 3.7 Neighbor found-cases (do not clone as this suite)

| Suite | Found-case | After this slice lands |
| --- | --- | --- |
| `sdd_risk_methodology_baseline` | `p05_methodology_fixtures_and_isms_context_are_absent` asserts `!product.contains("IsmsContext")` | Skip-supersede **that assertion** (or the test if it cannot be split). Keep methodology-fixture absence until the methodology slice lands. |
| `sdd_continuous_assurance_scheduler_baseline` | `cas_b012_no_isms_context_ir_and_collectors_do_not_set_effectiveness` asserts no `IsmsContext` | Skip-supersede the IsmsContext half. **Keep** the collector-must-not-write-`Effectiveness` half. |

Do **not** register those files as `sdd_isms_context_*`.

### 3.8 What current tests lock (must stay green)

- `sdd_compliance_ir_target`: golden `assessment.json` decode; `AssessmentDefinition::new`; IR-001 empty ids; IR-003 control has no ISO fields; IR-013 IR has no `reqwest::` / `GitHubClient` / `octocrab` / `aws_sdk`.
- `sdd_assurance_runtime_target` ACT-003: framework crate has no network/SDK deps.
- Existing `sdd_*_target` suites remain GREEN.

---

## 4. Desired behavior

### 4.1 Home and purity

All context types live in **`weeping-angel-assurance-ir`**. Same layer as `canonical_digest`. No clock requirement for `new` / `validate` / `explain` / digest (optional `DateTime<Utc>` fields may exist; validation must not call `Utc::now()`). No network, no `FrameworkProfile` on the context record, no collector id, no ISO clause numbers on types or serde keys.

Incorrect: a parallel `weeping-angel-grc` crate, a context table in the evidence ledger, clap flags, or stuffing ISO 4.1/4.2/6.2 vocabulary into field names (`clause4`, `annexA`, `soa`).

Schema remains `assurance-ir/v1`.

### 4.2 Canonical root: `IsmsContext`

Serde: `camelCase`. `schema_version` serializes as `schemaVersion` on this document (domain-record convention), value always `assurance-ir/v1`. Unknown lifecycle enum tags fail closed.

```text
IsmsContext {
  schemaVersion: "assurance-ir/v1",
  id: IsmsContextId,                          // required, typed_id!
  organization: Organization,                 // exactly one in v1
  scope: ManagementSystemScope,               // reference record, not ScopeResolution
  issues: Vec<ContextIssue>,
  interestedParties: Vec<InterestedParty>,
  obligations: Vec<Obligation>,
  objectives: Vec<SecurityObjective>,
  riskMethodologyId: Option<RiskMethodologyId>,
  assetIds: BTreeSet<AssetId>,                // existing IR
  vendorIds: BTreeSet<VendorId>,
  identityIds: BTreeSet<IdentityId>,
  cadence: Option<GovernanceCadence>,
  lifecycle: IsmsLifecycleStatus,
  supersededBy: Option<IsmsContextId>,        // required iff lifecycle = superseded
}

IsmsContext::new(id, organization, scope) →
  empty collections, riskMethodologyId = None, cadence = None,
  lifecycle = Draft, supersededBy = None, schemaVersion = ASSURANCE_IR_SCHEMA
```

v1 is **one organization per context**. Do not ship `organizations: Vec<Organization>` in this slice.

`IsmsContext` **must not** contain: `Effectiveness`, control-test results, residual scores, SoA entries, snapshot diffs, collector payloads, AWS account structs, GitHub org structs, ISO clause numbers.

### 4.3 Organization identity (not `Identity`, not `AssetKind::Organization`)

```text
Organization {
  id: OrganizationId,
  legalName: String,                 // required; empty/whitespace fails validation
  displayName: Option<String>,
  businessUnits: Vec<BusinessUnit>,  // ids unique within the organization
  scopeId: ScopeId,                  // must equal IsmsContext.scope.id
}

BusinessUnit {
  id: BusinessUnitId,
  name: String,                      // required non-empty
  parentId: Option<BusinessUnitId>,  // if set, must resolve in the same organization
}
```

`legalName` is the required identity field. `displayName` is optional. Do not add tax IDs, DUNS, AWS account, Entra tenant, or GitHub login as first-class fields. Those belong in collector facts or `ExtensionMap` if a later slice needs them — **not** in this generic IR.

Do **not** rename `Identity`. Do **not** add `IdentityKind::Organization`.

### 4.4 Management-system scope **reference** (not the scope engine)

```text
ManagementSystemScope {
  id: ScopeId,
  title: String,                     // required non-empty
  summary: Option<String>,
}
```

This is a **named boundary handle**. It does not resolve subjects to `InScope` / `OutOfScope` / `Conditional` / `Unknown`. It does not encode inclusion/exclusion precedence, expiry, or explain traces of the form `repo:payments -> …`. That is the scope engine.

`IsmsContext.scope.id` **is** `Organization.scopeId`. Mismatch is a dangling/inconsistent reference.

### 4.5 Internal / external issues

```text
IssueKind = Internal | External          // serde camelCase: internal, external

ContextIssue {
  id: IssueId,
  kind: IssueKind,
  title: String,                         // required non-empty
  description: String,
}
```

Issues are **context inputs**, not findings and not risks. Do not put `RiskRating`, CVE ids, or GitHub issue numbers on this type. Duplicate `IssueId` values fail closed. Unknown `IssueKind` tags fail closed.

Representative fixture: **one internal** and **one external** issue.

### 4.6 Interested parties and obligations (graph records, not the mapping engine)

```text
InterestedPartyKind =
  Customer | Regulator | Employee | Supplier | Insurer | Internal | Other

InterestedParty {
  id: InterestedPartyId,
  name: String,                          // required non-empty
  kind: InterestedPartyKind,
  obligationIds: Vec<ObligationId>,      // must resolve in IsmsContext.obligations
}

Obligation {
  id: ObligationId,
  title: String,                         // required non-empty
  description: Option<String>,
  interestedPartyId: InterestedPartyId,  // must resolve; must be listed on that party
}
```

Canonical edge: **InterestedParty → Obligation** (party lists ids) **and** Obligation carries `interestedPartyId` so dangling checks fail closed in both directions. The two must agree.

This slice does **not** add `ObligationMapping`, `RequirementSource`, supersession history, effective/review dates engines, or “why does this control exist?” lineage beyond id graph integrity. Those records live on the landed obligation registry (`party.rs` / `obligation.rs`); context keeps membership stubs and shared ids.

Do not copy protected normative text from ISO/IEC or contracts into fixtures.

### 4.7 Security objectives (definitions, not measurements)

```text
SecurityObjective {
  id: ObjectiveId,
  title: String,                         // required non-empty
  description: String,
  owner: Option<PrincipalRef>,           // reuse implementation::PrincipalRef
}
```

This is a **declared objective**. It is **not** `RequirementKind::ControlObjective`. It must **not** store `OnTrack` / `AtRisk` / `Achieved` / `Missed` / `InsufficientEvidence` — those are point-in-time projections owned by the security-objectives engine (`objectives::SecurityObjective` + `evaluate_objective`; [`security-objectives.md`](security-objectives.md)).

Do not add metric formula languages in this slice.

### 4.8 Risk methodology **reference**

```text
riskMethodologyId: Option<RiskMethodologyId>
```

`RiskMethodologyId` uses existing `typed_id!` (same empty/too-long/UUID-v4 rules). If the risk-methodology slice already defined it, import/re-export that type.

This slice:

- **does** store and validate the id (well-formed; required when lifecycle is `Active` or `UnderReview`);
- **does not** embed scales, matrices, appetite, or scores;
- **does not** implement `score_risk`.

A well-formed id is not a scoring result. Collectors still cannot emit ratings as evidence.

### 4.9 Populations through existing IR references

```text
assetIds:    BTreeSet<AssetId>
vendorIds:   BTreeSet<VendorId>
identityIds: BTreeSet<IdentityId>
```

These are **references**, not copies of `Asset` / `Vendor` / `Identity` records. `IsmsContext::validate()` checks id well-formedness (enforced by typed ids) and uniqueness (sets).

When an `AssessmentDefinition` carries `isms_context_id` **and** the caller asks to validate the pair (see §4.12), population ids that do not appear in that assessment’s `assets` / `vendors` / `identities` fail closed as dangling. Standalone `IsmsContext::validate()` does **not** require those inventories to be present — the context document must be constructable before an assessment exists.

Do not introduce `AwsAccount` or `GitHubOrganization` population types.

### 4.10 Governance cadence

```text
CadenceUnit = Day | Week | Month | Quarter | Year

CadenceInterval {
  count: u32,          // must be ≥ 1
  unit: CadenceUnit,
}

GovernanceCadence {
  managementReview: CadenceInterval,
  internalAudit: CadenceInterval,
  riskAssessment: CadenceInterval,
}
```

Field names are operational, **not** clause numbers (`clause93` is forbidden). `count == 0` is an impossible cadence and fails closed. Unknown units fail closed.

Required when lifecycle is `Active` or `UnderReview`. Optional on `Draft`. Retired/Superseded may retain the last cadence.

### 4.11 Lifecycle (exhaustive, serializable, centrally validated)

```text
IsmsLifecycleStatus = Draft | Active | UnderReview | Retired | Superseded
# serde camelCase: draft, active, underReview, retired, superseded
```

Default for `IsmsContext::new`: `Draft`.

| Status | Meaning | Extra constraints |
| --- | --- | --- |
| `Draft` | Definition in progress | Identity fields still required if records are present (empty `legalName` never ok). Methodology and cadence **may** be absent. `supersededBy` **must** be absent. |
| `Active` | In operation | Scope title, org `legalName`, `riskMethodologyId`, `cadence` required. `supersededBy` absent. |
| `UnderReview` | Still the current system, under governance review | Same completeness as `Active`. `supersededBy` absent. |
| `Retired` | No longer in force | `supersededBy` absent (retirement is not replacement). |
| `Superseded` | Replaced by another context | `supersededBy` required, well-formed, **not equal** to `id`. |

Impossible combinations (non-exhaustive but required):

- `superseded` without `supersededBy`;
- `supersededBy` present when status ≠ `superseded`;
- `supersededBy == id` (self-successor);
- `active` / `underReview` missing methodology id or cadence;
- `active` / `underReview` / any status with empty `legalName`, empty scope title, empty party/obligation/objective/issue titles when those records exist;
- `count == 0` cadence;
- unknown enum tag on the wire.

There is **no** `Utc::now()` transition API in this slice. Validation is **record consistency**, not a workflow engine.

### 4.12 Validation (fail closed, deterministic)

Implement `ValidateIr for IsmsContext`. Reuse `IrValidationError` (message strings must be stable enough for tests to match `dangling`, `duplicate`, `empty`, `lifecycle`).

Checks (all required):

1. `schemaVersion == ASSURANCE_IR_SCHEMA`;
2. duplicate ids within each collection (`IssueId`, `InterestedPartyId`, `ObligationId`, `ObjectiveId`, `BusinessUnitId`);
3. dangling: `Organization.scopeId` ↔ `scope.id`; `BusinessUnit.parentId`; `InterestedParty.obligationIds`; `Obligation.interestedPartyId` (must exist **and** list this obligation); `supersededBy` rules;
4. empty required identity / title fields (trim-aware: whitespace-only is empty);
5. impossible lifecycle (§4.11);
6. unknown serde enum tags fail at deserialize (do not coerce);
7. `riskMethodologyId` well-formed when present; required for Active/UnderReview.

Optional **pair** validator (same crate, not a new schema):

```text
validate_assessment_against_context(&AssessmentDefinition, &IsmsContext)
```

- if `assessment.isms_context_id` is `Some(id)`, it must equal `context.id` or fail (`dangling`);
- if `None`, the pair validator is a no-op for the pointer (assessment remains valid standalone — backward compatible);
- population ids on the context that are absent from the assessment inventories fail as dangling.

`AssessmentDefinition::validate()` **must not** start requiring an `IsmsContext`. Golden `assessment.json` stays valid.

### 4.13 Serialization and digest

- New types: serde `camelCase`, exhaustive enums, `skip_serializing_if` for empty `BTreeSet`s / `None` options as elsewhere.
- `canonical_digest(&isms_context)` and `typed_canonical_digest("IsmsContext", &isms_context)` reuse existing functions.
- Round-trip: `to_vec` → `from_slice` → `to_vec` is **byte-identical**.
- Equivalent `BTreeSet` / `BTreeMap` contents yield the same digest regardless of insertion order.
- `Vec` order is **authorial** (issues, parties, objectives). Duplicate-id validation still applies. Tests that prove digest stability for sets must use the `BTreeSet` fields, not shuffle `Vec`s and expect equality.
- No `f64` on context types.

### 4.14 Explain (definition, not assessment results)

```text
explain_isms_context(&IsmsContext) -> String
```

Pure, deterministic, no I/O. Must include: context id, org id + legal name, both business-unit names, scope id, issue kinds, party names, objective titles, methodology id if present, lifecycle.

Example shape (fixture):

```text
isms:acme -> org:acme (Acme Corp) -> scope:acme-ms
  bu:finance, bu:engineering
  issue:internal:staffing ; issue:external:regulation
  party:customers -> obligation:customer-security
  objective:reduce-incidents
  methodology:risk-method:acme-v1
  lifecycle:active
```

Exact wording is implement-defined; tests lock a **stable** string for the golden fixture (byte-stable across runs).

### 4.15 `AssessmentDefinition` compatibility

```text
AssessmentDefinition {
  …
  #[serde(default, skip_serializing_if = "Option::is_none")]
  isms_context_id: Option<IsmsContextId>,   // snake_case JSON key: isms_context_id
}
```

`AssessmentDefinition::new` sets `None`. Existing fixtures without the key deserialize. Do not reorder or rename existing fields. Do not switch `AssessmentDefinition` to `camelCase` (would break `assessment.json`).

### 4.16 Provider / framework neutrality

Generic IR modules introduced here must not contain first-class fields or serde keys whose folded names include:

```text
annex, soa, clause, iso27001, iso-27001, gdpr, soc2, nis2, dora,
aws, amazon, github, entra, okta, cloudflare, gcp, azure
```

ISO 27001 remains a **pack** that can **use** this model. The model does not belong to ISO. A future SOC 2 or NIS2 pack must be able to point at the same `IsmsContext`.

### 4.17 Representative fixture

Path: `tests/fixtures/assurance-ir/v1/isms-context.json`

Must decode as `IsmsContext` and `validate()` ok:

| Element | Content |
| --- | --- |
| Organization | one (`org:acme`), `legalName` non-empty |
| Business units | **two** (`bu:finance`, `bu:engineering`) |
| Issues | **one internal**, **one external** |
| Interested parties | ≥ 1 with ≥ 1 obligation |
| Objectives | ≥ 1 definition |
| Risk methodology | present well-formed `RiskMethodologyId` (no methodology document required) |
| Lifecycle | `active` (so cadence + methodology are present) |
| Populations | may be empty sets on the standalone fixture |

Do **not** change required fields of existing `assessment.json`.

---

## 5. Dual-suite plan

Contracts are **not** auto-discovered. Implement **must** add to root `Cargo.toml`:

```toml
[[test]]
name = "sdd_isms_context_baseline"
path = "tests/contracts/isms_context.baseline.rs"

[[test]]
name = "sdd_isms_context_target"
path = "tests/contracts/isms_context.target.rs"
```

Do not mention “Prompt N” in those files. Name the slice **ISMS context IR**.

Protocol: write tests first → baseline GREEN on characterization (absence) → target RED on characterization → implement → target GREEN → skip-supersede this baseline.

### 5.1 Baseline (`sdd_isms_context_baseline`) — GREEN on HEAD, skip-supersede after target GREEN

| Id | Found case |
| --- | --- |
| CTX-B01 | Product crate sources have no `struct IsmsContext`, `IsmsContextId`, `struct InterestedParty`, `struct Obligation`, `struct SecurityObjective`, `RiskMethodologyId` |
| CTX-B02 | `AssessmentDefinition::new` succeeds; `schema_version == "assurance-ir/v1"`; no `isms_context_id` required |
| CTX-B03 | Golden `tests/fixtures/assurance-ir/v1/assessment.json` decodes as `AssessmentDefinition` and `validate()` ok |
| CTX-B04 | `ASSURANCE_IR_SCHEMA == "assurance-ir/v1"` |
| CTX-B05 | `canonical_digest` / `typed_canonical_digest` / `typed_id!` (`AssessmentId::try_new("")` → `IdError::Empty`) still exist — reuse surface |
| CTX-B06 | `AssessmentScope.organizations` is `Vec<String>` (empty default); `IdentityKind` has no ISMS org variant requirement |
| CTX-B07 | No fixture `tests/fixtures/assurance-ir/v1/isms-context.json` |
| CTX-B08 | `weeping-angel-framework` `Cargo.toml` has no `reqwest` / `octocrab` / `aws-sdk` / `cloudflare` |

### 5.2 Target (`sdd_isms_context_target`) — RED on HEAD, GREEN after implement

| Id | Desired case |
| --- | --- |
| CTX-T01 | Golden `isms-context.json` constructs, `validate()` ok, serde round-trip **byte-identical**, `canonical_digest` stable across two loads |
| CTX-T02 | `AssessmentDefinition::new` still works; golden `assessment.json` still decodes; missing `isms_context_id` defaults to `None` |
| CTX-T03 | Duplicate ids (party, obligation, issue, objective, business unit, context id used as successor of itself) fail `validate()` |
| CTX-T04 | Dangling `obligationIds`, `interestedPartyId`, `parentId`, `scopeId` mismatch, population id vs assessment inventory (pair validator) fail closed |
| CTX-T05 | Empty/whitespace `legalName`, empty issue/party/obligation/objective/scope titles fail closed |
| CTX-T06 | Impossible lifecycle: `superseded` without `supersededBy`; `supersededBy` on `active`; `active` without methodology id or cadence; cadence `count == 0` |
| CTX-T07 | Serialized generic context JSON keys contain none of the forbidden ISO/provider tokens in §4.16; IR sources for the new module contain no `reqwest::` / `GitHubClient` / `aws_sdk` |
| CTX-T08 | `weeping-angel-framework` remains network-free (Cargo.toml + no SDK types in crate src) |
| CTX-T09 | `explain_isms_context` on the golden fixture is deterministic (same string twice) and mentions org, both BUs, both issue kinds, a party, an objective, methodology id, lifecycle |
| CTX-T10 | `assetIds` insertion order does not change `canonical_digest` (`BTreeSet`) |
| CTX-T11 | Context `schemaVersion` is `assurance-ir/v1`; no second schema constant forked |
| CTX-T12 | `IsmsContext` serde value has no `effectiveness`, `residualScore`, `statementOfApplicability`, `controlTestResults` keys |
| CTX-T13 | Existing type names `Asset`, `Vendor`, `Risk`, `Control`, `Requirement`, `Mapping`, `SubjectSelector` still resolve from `weeping_angel_assurance_ir` |
| CTX-T14 | Dual-suite `[[test]]` names `sdd_isms_context_baseline` / `sdd_isms_context_target` are listed in root `Cargo.toml` |

---

## 6. Non-goals

- UI, dashboards, auditor portal
- Persistence service, database, evidence-ledger tables for context
- Policy editor, workflow engine, management-review runtime
- ISO mapping / Annex A / clause-numbered fields
- Risk scoring, treatment, residual projection, identification
- Scope resolution engine (`InScope` / exclusions / expired exclusions)
- Obligation mapping engine / legal NLP
- Objective metrics, measurements, `OnTrack` projection
- CLI (`weeping-angel isms …`)
- Provider SDKs, network clients, framework packs in this slice
- Parallel GRC schema or new crate
- Rewriting neighbor dual-suites (only skip-supersede listed found-cases)

---

## 7. Risks

- **Name collision:** `Organization` vs `AssetKind::Organization` vs `SubjectKind::Organization` vs `Identity`. Mitigate with a dedicated `isms` module, explicit docs, and baseline greps for `struct Organization` only inside that module’s type — prefer exporting `Organization` as the ISMS legal entity and never aliasing it to `Identity`.
- **Slice overlap:** risk-methodology also wants `RiskMethodologyId`. Duplicate types would break the crate. Mitigate: “reuse if present; add `typed_id!` only if absent; never add scoring types here.”
- **Neighbor baselines go red** when `IsmsContext` appears (`p05_…_isms_context_are_absent`, `cas_b012_…`). Mitigate: skip-supersede those found-cases in the **same implement commit**.
- **Assessment JSON camelCase trap:** adding `ismsContextId` on `AssessmentDefinition` without matching its snake_case document would break golden decode **or** surprise callers. Mitigate: keep assessment document snake_case; `isms_context_id` + `serde(default)`.
- **Stuffing results into context:** later slices may be tempted to write effectiveness onto `IsmsContext`. Mitigate: CTX-T12 and ADR decision “definition ≠ assessment input.”
- **ISO vocabulary leak:** field names like `clause4Context` would block SOC 2/NIS2 reuse. Mitigate: CTX-T07 denylist.
- **Over-building parties/objectives:** implementing the later engines here would fork those slices. Mitigate: collision fence + additive serde defaults for future fields.

---

## 8. Acceptance criteria (testable)

- Dual suites `sdd_isms_context_baseline` and `sdd_isms_context_target` are explicitly listed in root `Cargo.toml` (not auto-discovered).
- Baseline is GREEN on characterization SHA until skip-superseded; target is RED before implement and GREEN after.
- `IsmsContext` lives in `weeping-angel-assurance-ir`, schema `assurance-ir/v1`, digested with existing `canonical_digest` / `typed_canonical_digest`.
- Representative fixture (one org, two BUs, one internal + one external issue, parties, objectives, methodology reference) round-trips byte-stably, validates, and explains.
- `AssessmentDefinition::new` and golden `assessment.json` remain valid; new assessment fields optional/defaulted.
- Duplicate ids, dangling internal refs, empty required identity fields, and impossible lifecycle states fail closed with deterministic errors.
- Generic IR introduces no ISO clause/Annex A/SoA field names and no AWS/GitHub/Entra objects.
- `weeping-angel-framework` stays network-free.
- `Asset`, `Vendor`, `Risk`, `Control`, `Requirement`, `Mapping`, `SubjectSelector`, evidence types are not broadly renamed.
- Neighbor IsmsContext-absence found-cases are skip-superseded; those suites are not reused as this slice.
- No persistence, CLI, UI, scoring engine, scope engine, or parallel GRC schema ships in this slice.

---

## 9. Landed checklist

1. Dual-suite `[[test]]` rows registered; baseline skip-superseded; target GREEN (CTX-T01–T14).
2. `typed_id!` aliases and `isms.rs` re-exported from `lib.rs`.
3. `ValidateIr`, `explain_isms_context`, `validate_assessment_against_context` shipped.
4. `AssessmentDefinition.isms_context_id` with `serde(default)` — `AssessmentDefinition::new` and golden `assessment.json` remain valid.
5. Golden [`tests/fixtures/assurance-ir/v1/isms-context.json`](../../tests/fixtures/assurance-ir/v1/isms-context.json).
6. Neighbor IsmsContext-absence found-cases skip-superseded (scope-engine `scp_b09`; IPO baseline comment). Those suites are not this slice.
7. This path is in `sdd_documentation_layout` `CANONICAL_SPECS`.
8. [`docs/adr/0008-isms-context.md`](../adr/0008-isms-context.md) **Accepted**.
9. Public-contract pointer and ISMS context section in [`docs/specs/assurance-runtime.md`](assurance-runtime.md).
10. Traces only under `.sdd/runs/` / `.sdd/artifacts/`.

---

## 10. Related

- Accepted ADR: [`docs/adr/0008-isms-context.md`](../adr/0008-isms-context.md)
- Spine: [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md)
- Docs layout: [ADR 0004](../adr/0004-documentation-architecture.md)
- Risk methodology (reference target): [`risk-methodology.md`](risk-methodology.md)
- Scheduler (must not invent a second root): [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md)
- Typed evidence (facts ≠ conclusions): [`typed-evidence.md`](typed-evidence.md)
