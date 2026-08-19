# SDD: Organizational Scope Engine

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_scope_engine_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — scope engine |
| Slice | Turn ISMS scope from descriptive text into an **executable, explainable boundary**. Every candidate subject resolves to `InScope` \| `OutOfScope` \| `Conditional` \| `Unknown` with rationale and lineage. |
| Dual-suite | `sdd_scope_engine_baseline` (skip-superseded) · `sdd_scope_engine_target` GREEN (`tests/contracts/scope_engine.{baseline,target}.rs`) — **not** auto-discovered; listed in root [`Cargo.toml`](../../Cargo.toml). `tests/sdd/` is forbidden ([ADR 0004](../adr/0004-documentation-architecture.md)) |
| ADR | Accepted [`docs/adr/0044-scope-engine.md`](../adr/0044-scope-engine.md). Cite by **path**. Concurrent sibling: [`0008-isms-context.md`](../adr/0008-isms-context.md). |
| Depends on | [`isms-context.md`](isms-context.md) (ISMS context IR — landed). Reuse `IsmsContext` / `Organization` / `BusinessUnit` / `ManagementSystemScope` / existing `AssessmentScope`. Do **not** invent a parallel GRC graph. |
| Seed (not SSOT) | [`docs/prompts/operational-isms-v1/02-scope-engine.md`](../prompts/operational-isms-v1/02-scope-engine.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (Organizational scope engine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Applicability (consume, do not fork) | [`applicability-engine.md`](applicability-engine.md) — Kleene stays; silent `excluded by assessment scope[{index}]` synthesis removed |
| Population (consume, extend consult) | [`population-runtime.md`](population-runtime.md) — `in_scope_population` injects in-scope ids via `EvidenceSet::set_population` |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) |
| Neighbors (must stay GREEN after implement) | `sdd_applicability_engine_target`, `sdd_population_runtime_target`, `sdd_assessment_lineage_target`, `sdd_compliance_ir_target`, `sdd_documentation_layout` |
| Collision fence | Facade `weeping_angel_assurance::AssessmentScope` and collector `CollectorScope` remain `AssetId` allow-sets — **adapt, do not collapse**. `src/engine/scope.rs` is **crawl URL scope** — do not reuse that module for ISMS. Collectors must not mutate scope. Statement of Applicability is **not** this slice. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Resolution snapshot schema (new, not IR) | `weeping-angel/scope-resolution/v1` (`SCOPE_RESOLUTION_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify (after implement) | `cargo test --test sdd_scope_engine_baseline`; `cargo test --test sdd_scope_engine_target`; `cargo test --test sdd_documentation_layout`; keep neighbor targets GREEN; `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable human SSOT for the **organizational scope engine**. It owns **scopeable-entity kinds**, **inclusion / exclusion / inheritance / precedence**, **exclusion governance (rationale, owner, approval, times, evidence)**, **`ScopeResolution`**, and **explain traces**.

It does **not** own asset discovery, AWS/GitHub/Entra types, Statement of Applicability, Kleene applicability evaluation, crawl URL scope, or a second GRC inventory.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Scope is a **pure deterministic operation over canonical inputs**. Collectors ask which subjects are in the boundary; they do not learn ISO semantics and they do not write scope state.

```text
IsmsContext → Organization → ManagementSystemScope     (named handle; ISMS context IR)
AssessmentDefinition.scope + inventories               (executable boundary input)
        → ScopeResolution (InScope | OutOfScope | Conditional | Unknown)
        → collector planning / compile / population / applicability filter
```

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only. Do **not** write reports under `docs/sdd/`.

---

## 0. Collision fence (concurrent SDD)

This slice may add IR fields on `ScopeExclusion` (serde defaults), additive `SubjectKind` variants that are **generic** (not provider schemas), a `scope` engine module in `weeping-angel-assurance`, a resolution snapshot type, dual-suite registration, this spec, its ADR, and a `CANONICAL_SPECS` entry.

| Do not touch | Owner |
| --- | --- |
| `src/engine/scope.rs` (`in_scope` for crawl URLs) | Recon product — **name collision only** |
| `weeping_angel_assurance::AssessmentScope` (`BTreeSet<AssetId>`) | Facade collector allow-set — **adapter only** |
| `weeping_angel_collector::CollectorScope` | Collector crate — consume via adapter; collectors **must not** mutate IR scope |
| Kleene evaluator (`applicability/evaluator.rs`) truth table | Applicability engine |
| `catalog/canonical/v1/**` IDs, ISO pack `to =` remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**`, Entra/AWS SDKs | Collectors / non-goals |
| `project_soa` / pack `applicability.toml` | Operational SoA / ISO remap — **not** this slice |
| Full `IsmsContext` construction, issues, parties, objectives, cadence | ISMS context IR — **reuse; do not fork** |
| Obligation mapping, risk scoring, residual, treatment | Later operational slices |
| Neighbor dual-suite bodies except listed skip-supersedes | Neighbors stay GREEN |
| `tests/sdd/` | ADR 0004 forbids this path |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Additive exclusion governance fields; optional inclusion-rule vec | `crates/weeping-angel-assurance-ir/src/assessment.rs` (`ScopeExclusion`, `AssessmentScope`) |
| Additive generic `SubjectKind` variants | `subject.rs` (`parse_name` + enum) |
| `ScopeDecision`, `ScopeResolution`, explain trace types | Prefer `weeping-angel-assurance/src/scope/` (engine). Thin IR DTOs only if compile/validation must name them. |
| Precedence + `resolve_scope` | `weeping-angel-assurance/src/scope/engine.rs` (name flexible) |
| Facade → collector adapter | `weeping-angel-assurance/src/lib.rs` `AssessmentScope::to_collector_scope` already exists — feed it **resolved** `AssetId`s |
| Validation of silent/expired/dangling exclusions | `validation.rs` (`ValidateIr`) |
| Fixture | `tests/fixtures/assurance-ir/v1/scope-engine.json` **in addition to** existing `assessment.json` |

Tiny allowed adjustments at implement: serde-default fields on `ScopeExclusion` / `AssessmentScope`; additive `SubjectKind` variants; `AssessmentDefinition::new` remains compatible; optional `as_of: DateTime<Utc>` on the **engine call**, not a required IR field; dual-suite `[[test]]` rows; `CANONICAL_SPECS` entry.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** collapse facade vs IR `AssessmentScope`. Do **not** put `reqwest` or provider SDKs in IR or the engine.

The engine resolves `AssessmentDefinition.scope` + inventories alone, and optionally binds `&IsmsContext` (`AssessmentScope.organizations` / `OrganizationId` / `BusinessUnit`) **without** copying those types into a second module.

---

## 1. Problem / user-visible goal

An ISMS is only as honest as its **boundary**. Today the boundary is a bag of selectors plus optional exclusion text. Reviewers cannot ask, as a machine:

```text
is repo:payments inside the ISMS?
why?
who approved the carve-out?
until when?
```

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- IR `AssessmentScope` is `{ organizations: Vec<String>, subjects: Vec<SubjectSelector>, exclusions: Vec<ScopeExclusion> }`.
- `ScopeExclusion` is `{ subjects, rationale: Option<String> }` — **no** owner/principal, approval reference, approved-at, review/expiry, or evidence refs.
- Applicability [`context.rs`](../../crates/weeping-angel-assurance/src/applicability/context.rs) synthesizes `excluded by assessment scope[{index}]` when rationale is missing, so **silent exclusions work**.
- There is **no** `ScopeResolution`, no `InScope` \| `OutOfScope` \| `Conditional` \| `Unknown` quad, no explain traces of the form `repo:payments -> business-unit:finance -> …`.
- Facade `weeping_angel_assurance::AssessmentScope` and `CollectorScope` are `AssetId` allow-sets — a different type with the same English name.
- `src/engine/scope.rs` is crawl URL membership (`authz.url_in_scope`).
- `SubjectKind` already has Organization / Repository / Service / Vendor / Network / CloudAccount / Dataset / ProcessingActivity (and more). It does **not** have generic **business unit**, **location**, **data domain**, or **personnel population**.
- Nested structure can reuse `Asset.parent`; nothing walks it for ISMS membership.
- `Exception` expiry (`status != Approved` or `expires_at <= now` → do not suppress) is the **pattern** for exclusion review/expiry; `ScopeExclusion` does not use it.
- Population `resolve_population` does **not** consult IR `AssessmentScope` inventories.
- ISMS context IR is a **spec** ([`isms-context.md`](isms-context.md)); product crates have **no** `struct IsmsContext`.

Silent, expired, and unordered exclusions are an audit defect: they look like a smaller ISMS without accountable records.

**User-visible goal:** given canonical inventories + `AssessmentScope` (+ `IsmsContext` when present) and an `as_of` instant, the engine deterministically answers **what is inside the ISMS boundary, why, under whose approval, and for what period**, with decisions traceable to canonical records.

```text
repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope
```

Definition of done: framework compilation and collector planning consume `ScopeResolution` without ISO semantics; out-of-scope subjects **cannot** contribute positive assurance evidence to an in-scope assessment; unknown/contradictory data **never** becomes positive in-scope evidence.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ASSURANCE_IR_SCHEMA` | `lib.rs` = `"assurance-ir/v1"` | **Do not fork.** |
| `AssessmentDefinition::new` | `assessment.rs` | **Must keep compiling.** New fields `#[serde(default)]`. |
| IR `AssessmentScope` / `ScopeExclusion` | `assessment.rs` | **Extend, do not replace.** Keep `{ organizations, subjects, exclusions }` as the document shape. |
| Facade `AssessmentScope` | `weeping-angel-assurance` | **Different type.** Adapter from `ScopeResolution` in-scope `AssetId`s. Do not collapse names. |
| `CollectorScope` | collector crate | Allow-set. Collectors **read** resolved ids; they **must not** mutate `AssessmentDefinition.scope`. |
| `SubjectSelector` / `SubjectKind` / `SelectorScope` | `subject.rs` | **SSOT selector.** Additive generic kinds allowed. No AWS/GitHub/Entra variants. |
| `Asset.parent` | `asset.rs` | Nested inheritance graph for inventory assets. Cycle → fail closed (`Unknown` + error). |
| `Identity` / `Vendor` / `ProcessingActivity` | existing modules | Inventory families. Do not invent parallel nodes. |
| `PrincipalRef` | `implementation.rs` | Exclusion **owner**. Reuse. |
| `Exception` / `ExceptionStatus` / `expires_at` | `exception.rs` + `subject_is_excepted` | **Pattern** for “expired does not suppress.” Do not overload `Exception` as ISMS exclusion. |
| Applicability `build_applicability_context` | `applicability/context.rs` | After this engine: filter inventories from `ScopeResolution`. Stop defaulting empty rationale. Do **not** change Kleene `All`/`Any`/`Not`. |
| `resolve_population` | control-test `population.rs` | Still injected via `EvidenceSet::set_population`. This engine **produces** the in-scope id list; population still does not grow a second selector type. |
| `compile_framework` / `resolve_applicability` | framework crate | May **consume** resolved in-scope subjects. Must not grow ISO/provider branches. `FrameworkTarget` unused in applicability filter stays unused for ISO. |
| `IsmsContext` / `Organization` / `BusinessUnit` / `ManagementSystemScope` | ISMS context IR (`isms.rs`) | Reuse. `ManagementSystemScope` remains a **named handle**; this engine **resolves**. |
| Golden `tests/fixtures/assurance-ir/v1/assessment.json` | `sdd_compliance_ir_target` | Must still decode. Do not add required keys. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_scope_engine_*` next to existing `sdd_*`. |

Do **not** redesign `AssessmentDefinition` core inventories, catalog schema, collectors, or ISO pack IDs.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded later by `tests/contracts/scope_engine.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is explicitly skip-superseded (`#[ignore]` + superseded comment).

### 3.1 IR scope is a descriptive bag

[`crates/weeping-angel-assurance-ir/src/assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs):

```text
ScopeExclusion {
  subjects: Vec<SubjectSelector>,
  rationale: Option<String>,     // optional; skip_serializing_if None
}

AssessmentScope {
  organizations: Vec<String>,    // not OrganizationId
  subjects: Vec<SubjectSelector>,
  exclusions: Vec<ScopeExclusion>,
}
```

No owner, no `PrincipalRef`, no approval reference, no `approved_at`, no `review_by` / `expires_at`, no evidence refs. Empty `rationale` is valid JSON.

`AssessmentDefinition::new` sets `scope` to that default (all vecs empty). Schema remains `assurance-ir/v1`.

`validate_assessment_ir` does **not** walk scope (no dangling selector ids, no silent-exclusion check).

### 3.2 Silent exclusions are operational

[`crates/weeping-angel-assurance/src/applicability/context.rs`](../../crates/weeping-angel-assurance/src/applicability/context.rs) `apply_exclusions`:

```text
reason = exclusion.rationale.clone()
         .unwrap_or_else(|| format!("excluded by assessment scope[{index}]"))
drop matching inventory members
```

- Missing rationale still **removes** subjects.
- There is no `as_of`; exclusions never expire.
- Inclusion is “retain if any `scope.subjects` matches” when that vec is non-empty; otherwise the full inventory is kept, then exclusions drop members.
- Order is **enumeration order** of `scope.exclusions` (and retain uses `any`). There is no declared precedence table and no conflict → `Unknown` path.
- Nested `Asset.parent` is **not** walked.
- `scope.organizations` is copied onto `ApplicabilityContext.organizations` and is **not** itself an inclusion expander.

### 3.3 No resolution quad or explain traces

Product crates have **none** of:

```text
enum ScopeDecision / InScope / OutOfScope / Conditional   (as a four-state scope result)
struct ScopeResolution
fn resolve_scope / resolve_subject
explain trace "repo:… -> business-unit:… -> ISMS scope"
SCOPE_RESOLUTION_SCHEMA / weeping-angel/scope-resolution/v1
```

Collector `CollectorError::OutOfScope` is an **asset allow-set miss**, not this quad. GitHub collector `OutOfScope` strings are provider planning errors.

### 3.4 Two other “scope” types (must remain)

| Type | Meaning on HEAD |
| --- | --- |
| Facade `weeping_angel_assurance::AssessmentScope` | `BTreeSet<AssetId>`; `to_collector_scope()` |
| `CollectorScope` | `BTreeSet<AssetId>`; `allows(&AssetId)` |
| `src/engine/scope.rs` | `in_scope(authz, url)` for crawl |

Baseline must grep for **ISMS** resolution types (`struct ScopeResolution`, `ScopeDecision::InScope` in assurance/IR), not for the word `scope` or crawl `fn in_scope`.

### 3.5 Subject kinds and nesting

`SubjectKind` (HEAD): Organization, Asset, Repository, Service, Identity, User, PrivilegedIdentity, Device, Vendor, Dataset, ProcessingActivity, Branch, Application, Database, CloudAccount, CloudResource, ServiceAccount, Endpoint, DataStore, Network, Deployment.

**Absent** generic kinds: BusinessUnit, Location, DataDomain, PersonnelPopulation (and `parse_name` aliases).

`Asset` has `parent: Option<AssetId>`. Nothing in applicability or population walks it for membership.

### 3.6 Exception expiry exists; exclusions ignore it

[`subject_is_excepted`](../../crates/weeping-angel-control-test/src/population.rs): only `ExceptionStatus::Approved` **and** `expires_at` not `<= now` suppress. Empty `Exception.subjects` does **not** mean the whole inventory.

`ScopeExclusion` has no parallel clock.

### 3.7 Population does not consult IR `AssessmentScope`

`resolve_population` uses explicit `EvidenceSet` population, selector ids, `inventory.subject` / `inventory.complete` envelopes, or inferred observation subjects. It does **not** read `AssessmentDefinition.scope` or `AssessmentDefinition` inventories.

### 3.8 ISMS context IR not in product

No `struct IsmsContext` in `crates/**/src/**/*.rs`. Concurrent spec [`isms-context.md`](isms-context.md) forbids this slice from implementing that root. `ManagementSystemScope` is specified as a **named handle**, not a resolver.

### 3.9 Compile ignores assessment scope

Framework `resolve_applicability` keeps requirements unless `statically_applicable() == Some(false)`. `FrameworkTarget` is unused. Controls are not filtered by ISMS boundary.

---

## 4. Desired behavior

### 4.1 Product home

New **scope engine** owned by `weeping-angel-assurance` (preferred split):

```text
crates/weeping-angel-assurance/src/scope/
  mod.rs       # re-exports
  engine.rs    # resolve_scope / resolve_subject
  snapshot.rs  # ScopeResolution + digest
  adapter.rs   # → facade AssessmentScope / CollectorScope (AssetId allow-set)
```

A single `scope.rs` is acceptable if it stays readable. **Do not** add this module under `src/engine/` (crawl).

IR stays **data**. The engine is **pure**:

```text
resolve_scope(input: ScopeInputs, as_of: DateTime<Utc>) -> Result<ScopeResolution, ScopeError>
resolve_subject(subject: SubjectRef, input, as_of) -> SubjectScopeDecision
```

`ScopeInputs` is a derived view of:

- `AssessmentDefinition.scope` + inventories (`assets`, `identities`, `vendors`, `processing_activities`);
- optional `&IsmsContext` when the type exists (organization, business units, `ManagementSystemScope.id` must match / bind);
- **no** collector handles, **no** wall-clock besides the caller-supplied `as_of`.

Network I/O is forbidden. Collectors must not call setters on `AssessmentScope`.

Same `(input, as_of)` always yields the same JSON resolution and the same explain string (ignore host clock).

### 4.2 Four-state decision (normative)

```text
ScopeDecision = InScope | OutOfScope | Conditional | Unknown
# serde camelCase: inScope, outOfScope, conditional, unknown
```

| Decision | Meaning | Positive assurance evidence? | Collector planning |
| --- | --- | --- | --- |
| `InScope` | A winning **valid** inclusion (direct or inherited) and no winning valid exclusion | **Yes** (may count as in-scope) | Include subject |
| `OutOfScope` | A winning **valid** exclusion, or an explicit `NoneOf` / out-rule, after expiry filtering | **No** | Do not collect as in-scope |
| `Conditional` | Winning rule carries a documented **condition** that this slice does not evaluate as a fact | **No** (not automatic positive) | May plan collection; effectiveness must not treat as proven in-scope membership |
| `Unknown` | Unresolved subject, missing inventory, invalid/silent exclusion that would have mattered, **equal-precedence conflict**, cycle, or contradictory rules | **No** | Do not treat as in-scope |

**Fail closed:** `Unknown` and `Conditional` **never** become `InScope` by default. `Unknown` / contradictory data **never** becomes positive in-scope evidence.

Boolean helpers (if provided) must not be the stored result:

```text
is_definitely_in_scope(d) == (d == InScope)
```

Callers that need a bool for allow-sets use **only** `InScope` (not Conditional, not Unknown).

### 4.3 Scopeable entities (no provider schemas)

A **candidate subject** is `{ kind: SubjectKind, id: String }` plus optional tags, resolved against inventories.

Kinds the engine must be able to name (existing **or** additive generic):

| Entity | Representation |
| --- | --- |
| Organization | `SubjectKind::Organization`; bind to `IsmsContext.organization.id` when present; else `AssessmentScope.organizations` strings / `AssetKind::Organization` |
| Business unit | Additive `SubjectKind::BusinessUnit` **or** `IsmsContext` `BusinessUnitId` + `Asset.tags["businessUnit"]`. Prefer additive kind + reuse `BusinessUnit.parentId` when context landed |
| Location | Additive `SubjectKind::Location` (generic; not a cloud region type) |
| System / application / service | Existing `Application` / `Service` / `Asset` |
| Repository | Existing `Repository` |
| Cloud account | Existing `CloudAccount` (no AWS account struct) |
| Network | Existing `Network` |
| Data domain | Additive `SubjectKind::DataDomain` (generic; not S3/GCS) |
| Personnel population | Additive `SubjectKind::PersonnelPopulation` **or** `Identity` + selector tags; engine returns member identity ids that are in scope |
| Vendor | Existing `Vendor` |
| Processing activity | Existing `ProcessingActivity` |

`parse_name` aliases (alnum, case-insensitive, same law as today): `businessunit`, `location`, `datadomain`, `personnelpopulation` / `population`.

**Forbidden** first-class kinds in this slice: `AwsAccount`, `GitHubOrganization`, `EntraTenant`, `GcpProject`. Those remain collector facts / tags.

Do not invent a parallel org graph. Inventories stay `Asset` / `Identity` / `Vendor` / `ProcessingActivity`. Nested **asset** membership walks `Asset.parent`. Nested **business unit** membership walks `BusinessUnit.parentId` when `IsmsContext` is supplied.

### 4.4 Exclusion governance (silent exclusions forbidden)

Extend IR `ScopeExclusion` additively (`#[serde(default)]`, `skip_serializing_if` so old `{ subjects, rationale? }` still **deserializes**). Validation of **new** documents fail-closed:

```text
ScopeExclusion {
  subjects: Vec<SubjectSelector>,          // required non-empty
  rationale: String,                       // required non-empty (trim); Option remains on the wire via default "" → invalid
  owner: Option<PrincipalRef>,             // required for a valid suppressing exclusion
  approvalRef: Option<String>,             // required; well-formed stable id (no UUID v4 mandate — same stable-id law as IR)
  approvedAt: Option<DateTime<Utc>>,       // required
  reviewBy: Option<DateTime<Utc>>,         // at least one of reviewBy, expiresAt required
  expiresAt: Option<DateTime<Utc>>,
  evidenceRefs: Vec<String>,               // supporting evidence / document ids; may be empty only if approvalRef present? required non-empty OR approvalRef — pick: require ≥1 evidenceRefs
}
```

Normative validation for an exclusion that is allowed to **suppress**:

1. `subjects` non-empty;
2. `rationale` non-empty after trim;
3. `owner` present (`PrincipalRef::Identity` / `Team` / `Role`);
4. `approvalRef` non-empty after trim;
5. `approvedAt` present;
6. `reviewBy` and/or `expiresAt` present (`reviewBy`/`expiresAt` must be `>= approvedAt` when both set);
7. `evidenceRefs` non-empty (stable ids).

Old JSON with only `subjects` + optional rationale **deserializes** (compat) but `validate()` **rejects** it as a silent exclusion. Engine: an invalid exclusion **must not suppress**. If applying it would have been the only OutOfScope path, the subject is **not** silently InScope either when an inclusion is **ambiguous**; if a valid inclusion independently wins, result is InScope and the invalid exclusion is recorded in the trace as `invalid; did not suppress`. If the invalid exclusion **conflicts** with an inclusion at equal specificity, result is `Unknown` (contradictory data).

**Expired / overdue (normative, Exception pattern):** at `as_of`:

```text
expired  = expiresAt.is_some_and(|t| t <= as_of)
overdue  = reviewBy.is_some_and(|t| t <= as_of)
active   = valid_record && !expired && !overdue
```

Only **active** exclusions may suppress. Expired/overdue exclusions **must not** suppress unless renewed (new `approvedAt` / later `expiresAt` / later `reviewBy` on a canonical record). They **remain visible** in the explain trace (`expired; did not suppress` / `review overdue; did not suppress`).

There is no wall-clock inside the engine. Tests pass a frozen `as_of`.

### 4.5 Inclusion rules

`AssessmentScope.subjects` remains the inclusion selector list (no approval required on inclusions).

**Organization-wide inclusion:** if `organizations` is non-empty (or `IsmsContext.organization` is supplied and bound), every candidate that **belongs** to those organizations is included at the organization rank (§4.6) unless a more specific rule wins.

Belonging:

- `AssetKind::Organization` id equals an listed org; or
- `IsmsContext.organization.id` / `legalName` / listed string matches; or
- walk `Asset.parent` until an organization asset / listed org; or
- `BusinessUnit` under that organization (when context present) and the subject’s `businessUnit` tag or kind/id matches; or
- identities/vendors/activities tagged or inventoried under that org (id prefix is **not** a membership rule — only explicit parent/tag/inventory links).

Empty `subjects` **and** empty `organizations` **and** no `IsmsContext` org: there is **no** implicit “everything in inventory is in scope.” Candidates resolve `Unknown` unless an explicit inclusion selector matches (`SelectorScope::All` on a kind with matching inventory members **is** explicit).

`SelectorScope`:

| Scope | Inclusion meaning |
| --- | --- |
| `AnyOf` | ids empty → no members (do not treat empty AnyOf as All); ids non-empty → those ids |
| `All` | all inventory members of that kind (and matching tags) |
| `NoneOf` | inclusion of the complement is **not** a back-door org-wide include; treat as an **exclude-shaped** selector at kind rank — prefer encoding true carve-outs as `ScopeExclusion`. If present on `subjects`, it competes as an exclusion-class rule |

Duplicate selectors: **idempotent**. Deduplicate by `canonical_digest` of the selector (or kind+ids+tags+scope tuple). Lineage lists unique rule ids in lexicographic order. Duplicates must not change the decision or double-count.

### 4.6 Precedence (defined once — not iteration order)

Decisions **must not** depend on `Vec` order of `subjects` or `exclusions`. Sort and rank.

**Specificity rank** (higher wins). Shipped table: include and exclude **share** the class value so equal-specificity include vs exclude is a conflict (`Unknown`, SCP-T06), not “exclude always wins.” Exact-id exclusion still beats org-wide / inherited inclusion (100 > 40/30, SCP-T02). Tests pin this.

| Rank | Rule class |
| --- | --- |
| 100 | Exact-id match (`AnyOf`/`ids` contains id) — include **or** active exclude |
| 80 | Tag match (non-empty non-condition tags, not id-only) |
| 60 | Kind-only (`SelectorScope::All` or kind-wide `NoneOf`) |
| 40 | **Inherited** decision from the nearest ancestor (asset `parent` / business-unit `parentId` / `businessUnit` tag). Closer ancestor wins over farther |
| 30 | Organization-wide inclusion (§4.5) |
| 0 | No matching rule |

Tie-break when two rules have the **same rank**:

1. If they **agree** (all include or all exclude) → that decision; lineage includes every unique contributing rule, sorted by canonical selector digest then exclusion/inclusion index **after** sort, never as a truth source.
2. If they **conflict** (include vs active exclude at the same rank) → `Unknown` + `ScopeError::Conflict` recorded on the subject. **Fail closed.** Do **not** pick the first in iteration order.
3. Conditional vs include at the same rank → `Conditional` (condition is stricter than bare InScope, not a conflict).
4. Conditional vs active exclude at the same rank → conflict → `Unknown`.

Inheritance: compute the parent’s **full** decision first (same function, memoized by subject id). Cycles in `parent` / `parentId` → that chain’s subjects are `Unknown` with `cycle` in the trace; do not infinite-loop.

Expired exclusions are **removed from the competing set** before ranking (they still appear in the trace).

### 4.7 Conditional

An inclusion (direct or inherited) may carry `condition: Option<String>` if implement adds an optional field on a wrapper; until then, a **tag** `scopeCondition` on the selector **or** a dedicated optional `conditions: Vec<String>` on `AssessmentScope` entries is allowed.

If the winning rule has a non-empty condition string, the decision is `Conditional`. The condition text is copied into rationale. This slice does **not** evaluate Kleene `ApplicabilityPredicate` (that remains the applicability engine).

`Conditional` is **not** `InScope` for positive assurance or collector allow-set adapters.

### 4.8 `ScopeResolution` output

Schema `weeping-angel/scope-resolution/v1` (`SCOPE_RESOLUTION_SCHEMA`):

```text
ScopeResolution {
  schema: String,
  assessmentId: AssessmentId,
  asOf: DateTime<Utc>,
  scopeId: Option<ScopeId>,          // IsmsContext.scope.id when bound
  subjects: Vec<SubjectScopeDecision>,  // lex-sorted by (kind, id)
  digest: String,                    // canonical_digest of body excluding digest
}

SubjectScopeDecision {
  kind: SubjectKind,
  id: String,
  decision: ScopeDecision,
  rationale: String,                 // deterministic; never empty
  lineage: Vec<LineageHop>,          // ordered leaf → root
  explain: String,                   // e.g. "repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope"
  influencingRules: Vec<InfluencingRule>,  # every rule that competed or was skipped for expiry/invalid
}

LineageHop { kind, id }

InfluencingRule {
  class: inclusion | exclusion | inheritance | organization | invalidExclusion | expiredExclusion | conflict,
  rank: u16,
  selectorDigest: String,
  exclusionIndex: Option<u32>,       // index after stable sort, for debug only
  owner: Option<PrincipalRef>,
  approvalRef: Option<String>,
  approvedAt: Option<DateTime<Utc>>,
  expiresAt: Option<DateTime<Utc>>,
  reviewBy: Option<DateTime<Utc>>,
  applied: bool,                     // false if expired/invalid/overdue
}
```

Walk inventories in a **fixed family order**: organizations, business units, locations, assets (including repos/services/networks/cloud accounts/datasets), identities (personnel), vendors, processing activities. Within a family, sort by id.

Every candidate the caller asks about **and** every inventory member appears unless the caller passes an explicit candidate list. Unresolved ids (selector names a subject not in inventory and not constructible) still emit a row with `Unknown` and rationale `unresolved subject`.

Digest: existing `canonical_digest` / `typed_canonical_digest` over the struct **excluding** the digest field. Same inputs → same digest.

### 4.9 Explain traces

Normative format (ASCII, ` -> ` separators, no ISO clause numbers):

```text
{kind}:{id} -> {parentKind}:{parentId} -> … -> ISMS scope -> {Decision}
```

Kind tokens are `parse_name` keys (`repo` may be an alias of `repository` in the **string**; pick one and test it). Recommended tokens: `repo`, `business-unit`, `service`, `org`, `identity`, `vendor`, `processing-activity`, `location`, `data-domain`, `population`, `network`, `cloud-account`.

Example:

```text
repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope
```

Expired exclusion example (must remain visible):

```text
repo:payments -> ISMS scope -> InScope
# influencingRules contains expiredExclusion applied=false
```

Conflict example:

```text
repo:payments -> ISMS scope -> Unknown
```

Same `(subject, input, as_of)` → byte-identical `explain` string.

### 4.10 Collector and facade adapter

```text
ScopeResolution::in_scope_asset_ids() -> BTreeSet<AssetId>
  # only decision == InScope and kind maps to an AssetId in inventory

to_facade_assessment_scope() -> weeping_angel_assurance::AssessmentScope
to_collector_scope() -> CollectorScope
```

`Conditional` / `Unknown` / `OutOfScope` ids are **omitted** from the allow-set.

Collectors receive `CollectorScope` and collect. They **must not** insert into `AssessmentDefinition.scope`. Tests assert engine modules have no `&mut AssessmentScope` / no collector trait bounds.

Framework compile **may** read `ScopeResolution` to know which subjects exist for planning; it must not import ISO types. Collectors never import `ScopeDecision` ISO mappings (there are none).

### 4.11 Population selection

The engine answers personnel/population selectors against **IR inventories ∩ scope**, not against observed envelopes:

```text
in_scope_population(selector, resolution, inventories) -> { ids, completeness }
```

- `PersonnelPopulation` / `Identity` / `User` / `PrivilegedIdentity` members that are `InScope` are returned, lex-sorted.
- Completeness is `Authoritative` when the identity (or named population) family was an explicit inclusion or org-wide include over an inventoried set; `Unknown` if any member is `Unknown`; never treat `Unknown` members as in-scope coverage successes.
- Callers inject this list via existing `EvidenceSet::set_population`. This slice does **not** rewrite `resolve_population`’s envelope inference except as needed to **refuse** out-of-scope ids.

### 4.12 Out-of-scope evidence must not contribute positive assurance

Normative (target test):

Given an in-scope assessment and a control test over in-scope subjects, an evidence envelope whose `provenance.asset` (or subject id) resolves `OutOfScope` **must not** increase `passing` / produce `Effectiveness::Effective` for that assessment.

`Unknown` and `Conditional` envelopes likewise **must not** count as positive in-scope evidence.

Implementation options (pick one, test the law):

1. Population injected into evaluate is already the in-scope id set; extra envelopes for other assets are ignored by existing subject matching; **and** a guard rejects using them as population members;
2. Explicit filter in control-test / assurance evaluate path: skip envelopes whose subject is not `InScope`.

Do **not** mark the control `NotApplicable` merely because some out-of-scope evidence exists.

Applicability context: **replace** silent rationale synthesis with resolution-driven drops. Expired exclusions must **not** remove members there either. Do not change Kleene `Not(Unknown)`.

### 4.13 Binding to ISMS context IR

When `&IsmsContext` is provided:

- `Organization.scopeId` must equal `IsmsContext.scope.id` (already validated by context IR);
- `AssessmentScope.organizations` entries bind to `OrganizationId` or `legalName`; unbound org strings → `Unknown` for membership that depended on them (fail closed), not a guessed include;
- `BusinessUnit` tree is the inheritance graph for `SubjectKind::BusinessUnit`;
- `ManagementSystemScope` title/summary are **not** parsed as selectors.

When context is absent, the engine still runs on `AssessmentDefinition` alone.

### 4.14 Determinism and errors

```text
ScopeError = Conflict | Cycle | InvalidExclusion | Unresolved | Schema
```

`resolve_scope` returns `Ok(ScopeResolution)` with per-subject `Unknown` for recoverable fail-closed cases (unresolved, conflict, cycle). Use `Err` only for input schema that cannot be interpreted (wrong IR schema version). Per-subject conflicts **must not** pick a winner.

Do not hash `HashMap`. Use `BTreeMap` / `BTreeSet`. Sort before rank. Do not iterate `exclusions` as the precedence source.

---

## 5. Tests (dual SDD)

Register at implement:

```text
[[test]]
name = "sdd_scope_engine_baseline"
path = "tests/contracts/scope_engine.baseline.rs"

[[test]]
name = "sdd_scope_engine_target"
path = "tests/contracts/scope_engine.target.rs"
```

`tests/contracts/` is **not** Cargo auto-discovery. Comments in those files name the slice **scope engine** and **ISMS context IR** — never “Prompt N”.

Protocol: write tests first → baseline GREEN on characterization (descriptive/silent scope) → target RED until engine exists → implement → target GREEN → skip-supersede baseline (`#[ignore]`). Traces only under `.sdd/runs/`.

### 5.1 Baseline (`sdd_scope_engine_baseline`) — GREEN on HEAD, skip-supersede after target GREEN

Titles `P?: <exact subject>` are for the later found-case protocol; dual-suite ids below are the spec names.

| Id | Found case (current HEAD) |
| --- | --- |
| SCP-B01 | IR `AssessmentScope` fields are `organizations`, `subjects`, `exclusions` only |
| SCP-B02 | `ScopeExclusion` has `subjects` + optional `rationale`; source has no `approvalRef` / `approvedAt` / `expiresAt` / `reviewBy` / `owner` on that struct |
| SCP-B03 | `applicability/context.rs` contains `excluded by assessment scope[` (silent default rationale) |
| SCP-B04 | Product crates have no `struct ScopeResolution`, no `SCOPE_RESOLUTION_SCHEMA`, no `enum ScopeDecision` with four ISMS states |
| SCP-B05 | Facade `weeping_angel_assurance::AssessmentScope` is an `AssetId` set (`allow_asset`); `CollectorScope` likewise |
| SCP-B06 | `src/engine/scope.rs` exports crawl `in_scope` taking `Authorization` + `Url` |
| SCP-B07 | `SubjectKind::parse_name("businessunit")`, `"location"`, `"datadomain"`, `"personnelpopulation"` are `None` |
| SCP-B08 | `resolve_population` source does not mention `AssessmentScope` |
| SCP-B09 | No `struct IsmsContext` in product crate sources (ISMS context IR not landed) |
| SCP-B10 | Dual-suite names `sdd_scope_engine_*` need not exist yet; after registration this row skip-supersedes |

### 5.2 Target (`sdd_scope_engine_target`) — RED on HEAD, GREEN after implement

| Id | Desired case |
| --- | --- |
| SCP-T01 | **Nested inclusion:** `repo:payments` parented under `service:payments` under `business-unit:finance` under listed org → `InScope` with explain `repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope` (hop order may follow parent walk; string is pinned in the test). Same inputs twice → identical JSON + explain |
| SCP-T02 | **Exclusion precedence:** exact-id active exclusion beats org-wide and parent inclusion → `OutOfScope`; lineage cites owner, `approvalRef`, `approvedAt`, `expiresAt`/`reviewBy`, evidence refs |
| SCP-T03 | **Expired exclusion:** same as T02 but `expiresAt <= as_of` (or `reviewBy <= as_of`) → exclusion `applied=false`, subject **InScope** (given remaining inclusion); trace still names the expired exclusion. Renewed later `expiresAt` restores suppression |
| SCP-T04 | **Unresolved subject:** selector/candidate id not in inventories → `Unknown`, not `InScope` |
| SCP-T05 | **Duplicate selectors:** two identical inclusions → one decision `InScope`, lineage deduped, digest stable regardless of vec order |
| SCP-T06 | **Conflicting rules:** exact-id inclusion and exact-id active exclusion → `Unknown` / conflict; swapping vec order does **not** change the decision; never `InScope` |
| SCP-T07 | **Organization-wide inclusion:** `organizations = ["org:acme"]` includes nested inventory members; unrelated org’s asset stays `Unknown`/`OutOfScope` per rules, not silently in |
| SCP-T08 | **Population selection:** personnel/identity selector ∩ inventories ∩ scope returns only `InScope` member ids; `resolve` path used by the engine consults IR `AssessmentScope` (not envelope inference alone) |
| SCP-T09 | **Out-of-scope evidence:** envelope for an `OutOfScope` subject does not create `Effectiveness::Effective` / passing count on an in-scope population test |
| SCP-T10 | Silent exclusion (empty rationale / missing owner/approval/times/evidence) fails `validate()`; engine does not suppress with a synthesized reason |
| SCP-T11 | `AssessmentDefinition::new` still works; golden `assessment.json` still decodes; schema remains `assurance-ir/v1` |
| SCP-T12 | Facade + `CollectorScope` still `AssetId` allow-sets; adapter maps **only** `InScope` assets; collectors have no `&mut` IR scope |
| SCP-T13 | `src/engine/scope.rs` still has only crawl URL helpers (collision fence) |
| SCP-T14 | Additive kinds parse: `businessunit`, `location`, `datadomain`, `personnelpopulation`; no `AwsAccount` / `GitHubOrg` / `EntraTenant` variants |
| SCP-T15 | Dual-suite `[[test]]` names listed in root `Cargo.toml`; this spec path in `CANONICAL_SPECS` |

---

## 6. Non-goals

- Asset / account / repository **discovery**
- AWS, GitHub, Entra, GCP **first-class types** or SDKs
- Statement of Applicability (`project_soa`, pack `applicability.toml`)
- Kleene applicability truth table rewrite
- Crawl URL scope (`src/engine/scope.rs`)
- Collapsing facade `AssessmentScope` with IR `AssessmentScope`
- Implementing `IsmsContext` in this slice (ISMS context IR owns it)
- Persistence, UI, policy editor, auditor portal
- Parallel GRC schema / new crate / `assurance-ir/v2`
- Treating `Unknown` or `Conditional` as in-scope for positive evidence

---

## 7. Risks

- **Name collision** with crawl `scope.rs`, facade `AssessmentScope`, collector `OutOfScope`. Mitigate: engine lives in `weeping-angel-assurance::scope`; tests pin crawl module unchanged; adapters only.
- **ISMS context IR not in tree at implement.** Mitigate: engine runs on `AssessmentDefinition` alone; binds context types only when present; do not fork `Organization` / `BusinessUnit`.
- **Applicability silent exclusions** keep dropping subjects if context.rs is not switched over. Mitigate: T03/T10 and replace synthesized rationale in the same implement commit; do not retarget Kleene tests.
- **Iteration-order bugs** (HashMap / vec `any`). Mitigate: BTree collections, explicit rank table, T05/T06 order-swap tests.
- **Expired exclusion still suppresses** if `as_of` is ignored. Mitigate: T03 frozen clock; copy Exception pattern.
- **Out-of-scope evidence leaks** into Effective. Mitigate: T09 end-to-end on evaluate/population.
- **Additive SubjectKind** breaks catalogs that reject unknown kinds — additive is OK; do not reuse provider names. Mitigate: T14 denylist.
- **Serde required fields** break golden `assessment.json`. Mitigate: defaults + T11; validate() fail-closed on **new** silent exclusions without requiring keys on empty default scope.

---

## 8. Acceptance criteria (testable)

- Dual suites `sdd_scope_engine_baseline` and `sdd_scope_engine_target` are explicitly listed in root `Cargo.toml` (not auto-discovered).
- Baseline is GREEN on characterization SHA until skip-superseded; target is RED before implement and GREEN after.
- Every candidate subject resolves to `InScope` \| `OutOfScope` \| `Conditional` \| `Unknown` with non-empty rationale and explicit lineage (not a boolean).
- Exclusions that suppress have rationale, owner/`PrincipalRef`, approval reference, approved-at, review and/or expiry, and evidence refs; silent exclusions fail validation and do not suppress.
- Expired or review-overdue exclusions do not suppress; they remain in the explain trace; renewal restores suppression.
- Nested inclusion, exclusion precedence, duplicate selectors, conflicting equal-rank rules, organization-wide inclusion, unresolved subjects, and population selection match §5.2.
- Equal-precedence include vs exclude is `Unknown` (fail closed), independent of vec order.
- `Unknown` / `Conditional` / `OutOfScope` never count as positive in-scope evidence; T09 holds.
- `ScopeResolution` is deterministic (`canon/v1` digest) and consumable by collector planning without ISO types.
- Facade `AssessmentScope` and `CollectorScope` remain `AssetId` allow-sets; crawl `src/engine/scope.rs` is untouched.
- `ASSURANCE_IR_SCHEMA` stays `assurance-ir/v1`; `AssessmentDefinition::new` and golden assessment JSON remain valid.
- Engine is pure over canonical inputs; collectors do not mutate scope.
- Neighbor targets `sdd_applicability_engine_target`, `sdd_population_runtime_target`, `sdd_assessment_lineage_target`, `sdd_compliance_ir_target` stay GREEN.
- Human SSOT is this file; traces go to `.sdd/runs/`; no generated reports under `docs/sdd/`.

---

## 9. Shipped (implement complete)

Product:

```text
crates/weeping-angel-assurance/src/scope/
  mod.rs       # re-exports
  engine.rs    # resolve_scope / resolve_subject / in_scope_population
  snapshot.rs  # ScopeResolution + SCOPE_RESOLUTION_SCHEMA
  adapter.rs   # InScope-only facade AssessmentScope / CollectorScope
```

IR: additive `ScopeExclusion` governance fields; `SubjectKind::{BusinessUnit,Location,DataDomain,PersonnelPopulation}`; `validate_scope_exclusions` on `AssessmentDefinition`. Schema remains `assurance-ir/v1`.

Dual suites registered. Target GREEN. Baseline skip-superseded. ADR 0008 **Accepted**. Public contract: [`assurance-runtime.md`](assurance-runtime.md) Organizational scope engine.

Do not mention “Prompt N” in the ADR, this spec, or test comments. Slice names: **scope engine**, **ISMS context IR**.
