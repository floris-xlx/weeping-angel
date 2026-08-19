# SDD: Interested Parties and Obligations

| Field | Value |
| --- | --- |
| Status | **Implemented** — standalone `ObligationRegistry` in `party.rs` + `obligation.rs`; dual-suite registered; target GREEN |
| Program | Operational ISMS v1 — foundation |
| Seed | [`docs/prompts/operational-isms-v1/03-interested-parties-obligations.md`](../prompts/operational-isms-v1/03-interested-parties-obligations.md) |
| Slice | Interested parties / obligations — canonical obligation layer between organizational context and controls |
| Dual-suite | `sdd_interested_parties_obligations_baseline` (skip-superseded) · `sdd_interested_parties_obligations_target` (GREEN) |
| Contract files | `tests/contracts/interested_parties_obligations.{baseline,target}.rs` — **not auto-discovered**; list `[[test]]` in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0043-interested-parties-obligations.md`](../adr/0043-interested-parties-obligations.md) (**0004** is documentation architecture — do not reuse that number). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (interested parties / obligations section; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Mapping honesty (reuse) | IR `MappingDirection` / `MappingRelation` / `MappingCompleteness`; [`docs/specs/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §4.2; `ComplianceGraph::equivalent` |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Depends on (consume, do not rewrite) | ISMS context IR (`IsmsContext` — landed; membership stubs share ids); organizational scope engine (`AssessmentScope` / `SubjectSelector` stored; `ScopeResolution` consumed when present) |
| Neighbors (keep GREEN) | `sdd_compliance_ir_target`, `sdd_assessment_lineage_target`, `sdd_applicability_engine_target` |
| Collision fence | ISMS context IR product, scope-engine product, controlled-documents (share `ObligationId` only), risk-*, operational SoA, catalog TOML, ISO remap, Kleene evaluator, collectors |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `canon/v1` (`serde_json` struct field order + `BTreeMap` / `BTreeSet`) |
| Workspace verify (after implement) | `cargo test --test sdd_interested_parties_obligations_baseline`; `cargo test --test sdd_interested_parties_obligations_target`; `cargo test --test sdd_compliance_ir_target`; `cargo test --test sdd_assessment_lineage_target`; `cargo test --test sdd_applicability_engine_target`; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for the interested parties / obligations slice. It owns **interested-party identity**, **requirement sources**, **organizational obligations**, **honest obligation mappings**, **lifecycle/supersession without deletion**, **scope-engine applicability**, **deterministic validation**, and **explainability of why a control or policy exists**. It does **not** own `IsmsContext`, the scope-resolution engine, controlled-document versioning, risk scoring, framework packs, collectors, legal advice, or ISO control libraries.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Obligations sit **beside** that spine as durable governance inputs, not as a second spine:

```text
InterestedParty → RequirementSource → Obligation
                                      ├─▶ Risk
                                      ├─▶ Canonical Control
                                      ├─▶ Policy / ControlledDocument
                                      └─▶ ExternalRequirementRef
```

An obligation answers **why a requirement exists for this organization**. A framework `Requirement` answers **what a named standard clause says**. A canonical `Control` answers **what the control means**. `Effectiveness` answers **whether a test passed**. Collectors emit **facts**. None of those layers may silently become another.

```text
why the org must care              = Obligation (this slice)
who cares                          = InterestedParty
where the duty comes from          = RequirementSource
what the control means             = Control (canonical catalog)
how this org implements it         = ControlImplementation
whether the control is effective   = ControlTestResult.effectiveness
which clause a pack projects       = Requirement + Mapping (framework pack)
```

### Landed surface

| Item | Home |
| --- | --- |
| Registry `InterestedParty` / `InterestedPartyKind` | [`crates/weeping-angel-assurance-ir/src/party.rs`](../../crates/weeping-angel-assurance-ir/src/party.rs) |
| `RequirementSource`, registry `Obligation`, `ObligationMapping`, `ObligationRegistry`, `obligation_applies`, `current_obligations_at`, `validate` | [`crates/weeping-angel-assurance-ir/src/obligation.rs`](../../crates/weeping-angel-assurance-ir/src/obligation.rs) |
| `ObligationId`, `InterestedPartyId`, `RequirementSourceId`, `ObligationMappingId`, `ControlledDocumentId` | `id.rs` `typed_id!` (crate-root; **shared** with ISMS context and controlled-documents) |
| Modules | `lib.rs` `mod party` + `mod obligation` (registry structs **not** crate-root `pub use` — crate-root names remain `isms::InterestedParty` / `isms::Obligation`) |
| Owner | Reuse `PrincipalRef` |
| Applicability | Stored IR `AssessmentScope` / `SubjectSelector`; `obligation_applies` matches caller/`implied_universe` subject ids (scope-engine `ScopeResolution` is `weeping-angel-assurance::scope`; no provider filters) |
| Explain | `weeping-angel-assurance::{explain_why_control_exists, explain_why_document_exists}` → `ObligationExplain`; additive `ControlExplanation.obligations` (default empty) |
| Dual-suite | `sdd_interested_parties_obligations_target` GREEN; baseline `#[ignore = "superseded by sdd_interested_parties_obligations_target"]` |
| Context refs | `ObligationRegistry::validate_against_isms_context` — context obligation ids must exist in the registry (bodies stay out of `IsmsContext`) |

Crate-root `InterestedParty` / `Obligation` remain **membership-graph** records on `IsmsContext`. This slice owns the **registry** type family and shares the id newtypes.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only. Do not write generated reports under `docs/sdd/`.

---

## 0. Collision fence (concurrent SDD)

This slice may add IR party/obligation types, dual-suite tests, and an additive explain helper. It must not rewrite in-flight neighbors and must not invent a parallel GRC graph.

| Do not touch | Owner |
| --- | --- |
| `IsmsContext` module (when it lands), its spec/tests/ADR | ISMS context IR |
| `ScopeResolution` engine, scope-engine spec/tests/ADR | Organizational scope engine |
| `docs/specs/controlled-documents.md`, `tests/contracts/controlled_documents.*`, `document.rs` | Controlled-documents slice (share `ObligationId` alias only) |
| `docs/specs/risk-*.md`, `tests/contracts/risk_*.rs`, methodology/register/treatment/residual | Risk slices |
| `docs/specs/operational-soa.md`, `soa.rs` | Operational SoA |
| `catalog/canonical/v1/**`, `frameworks/iso-27001/**`, `tests/contracts/iso27001_remap.*` | Catalog / ISO remap |
| Kleene evaluator (`weeping-angel-assurance::applicability`) | Applicability engine |
| Collector GitHub mapping / evidence types | GitHub collector |
| IR `Requirement` / `Mapping` field layout, `ComplianceGraph::equivalent` semantics | Compliance IR |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `InterestedParty` | `crates/weeping-angel-assurance-ir/src/party.rs` (new) |
| `RequirementSource`, `Obligation`, `ObligationMapping`, registry, validate, current-at-T, mapping projection | `crates/weeping-angel-assurance-ir/src/obligation.rs` (new) |
| `ObligationId`, `InterestedPartyId`, `RequirementSourceId`, `ObligationMappingId` | `crates/weeping-angel-assurance-ir/src/id.rs` (`typed_id!`) — **share** `ObligationId` with controlled-documents; do not fork a second alias |
| Re-exports | `crates/weeping-angel-assurance-ir/src/lib.rs` |
| Owner | Reuse `PrincipalRef` (`implementation.rs`) |
| Applicability | Reuse IR `AssessmentScope` / `SubjectSelector` / `ScopeExclusion` (`assessment.rs`, `subject.rs`). Consume `ScopeResolution` when the scope engine has landed. **No** provider-shaped filters. |
| Mapping honesty | Reuse `MappingDirection`, `MappingRelation`, `MappingCompleteness`, `MappingProvenance` (`mapping.rs`). Do not invent a parallel enum set. |
| External clause refs | Reuse `ExternalRequirementRef` (`framework.rs`) |
| Document mapping targets | Reuse `ControlledDocumentId` if present; otherwise add the same `typed_id!(ControlledDocumentId)` min alias (documents slice shares it) |
| Explain path | Additive helpers in `weeping-angel-assurance` lineage (`explain_why_control_exists` / obligation lineage on `ControlExplanation` with serde defaults). **Reuse** `canon/v1`. Do not add a second identity or digest system. |

Tiny allowed adjustments at implement: new `party.rs` + `obligation.rs`; additive `typed_id!`; `lib.rs` re-exports; serde camelCase; optional empty `obligations` field on `ControlExplanation`. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add obligation inventories onto `AssessmentDefinition` in this slice (collision with in-flight IR growth). The registry is a standalone IR document. `IsmsContext` (when present) **references** obligation and party ids; this slice does not implement context.

Shipped **after** ISMS context IR on the same worktree. Consumes `IsmsContext` membership ids (`validate_against_isms_context`), IR `AssessmentScope`, and `SubjectSelector`. `obligation_applies` matches selectors against `ObligationLinkUniverse`; `ScopeResolution` / `resolve_scope` stay the scope-engine product. Do not duplicate organization identity, issues, or a second selector type.

---

## 1. Problem / user-visible goal

Operators cannot answer **why a control or policy exists** for this organization. Weeping Angel can map a framework `Requirement` onto a canonical `Control` (honest `PartiallySatisfies` / `Supports`), can explain a pinned assessment run, and can attest that a document-control *process* exists. It cannot record:

- who the interested parties are;
- which contractual, legal, customer, employment, insurer, or supplier duties apply;
- which of those duties are current vs superseded vs expired;
- which canonical controls, risks, and policies those duties map to, with explicit relation strength.

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- `weeping-angel-assurance-ir` has **no** `InterestedParty`, `RequirementSource`, `Obligation`, or `ObligationMapping` types.
- `id.rs` `typed_id!` aliases stop at `MappingId`. **`ObligationId` is absent.**
- `IsmsContext` and `ScopeResolution` are **not** in this worktree.
- `Requirement` is a **framework-specific** clause (`FrameworkRef` + title/description). It is not an organizational duty.
- `Mapping` is only `fromRequirement → toControl`. `ComplianceGraph::equivalent` is fail-closed (full bidirectional only). Partial paths never upgrade.
- `explain_control` walks assessment mappings, tests, evidence, exceptions, and applicability. It does **not** walk organizational obligations.
- Collectors emit `EvidenceEnvelope` facts. There is no collector API that marks an obligation satisfied.

That means a reviewer cannot distinguish “ISO A.8.5 projects onto MFA” from “the customer contract requires MFA for production identities,” cannot keep a superseded DPA addressable, and cannot stop a later projection from treating `Supports` as equivalence.

**User-visible goal:** given a canonical control id or policy/document id and a time `T`, Weeping Angel can answer:

```text
which interested parties care?
which requirement sources created the duty?
which obligations are current at T?
which obligations are superseded / retired / expired, still get(id)?
how does each obligation map to this control / risk / document / external ref
  (direction, relation, completeness, rationale)?
does this mapping claim equivalence, satisfaction, support, or related-only?
is the duty in the ISMS scope (scope-engine selectors, not provider filters)?
```

and can prove the negatives:

```text
PartiallySatisfies / Supports / Related / EvidenceFor / SubsetOf
  → never project as Equivalent or full satisfaction
expired / superseded obligation → addressable, not in current()
duplicate stable ids → fail closed
dangling mapping target → fail closed
collector / framework pack → cannot set obligation satisfaction
protected normative text → not stored on generic IR
overlapping / conflicting current obligations → may coexist
```

Definition of done (program): Weeping Angel can answer `why does this control/policy exist?` through a stable obligation graph, later usable by management review and risk treatment.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | **Do not fork.** Obligation documents carry this schema version string. |
| `canonical_digest` / `typed_canonical_digest` / `canon/v1` | `digest.rs` | **Reuse.** No second digest algorithm, no UUID v4 identities. |
| `Requirement` / `RequirementKind` | `requirement.rs` | **Keep.** Framework clause ≠ organizational obligation. Do not overload `Requirement` as `Obligation`. |
| `Mapping` | `mapping.rs` | Keep for requirement → control. New `ObligationMapping` reuses direction/relation/completeness/provenance. |
| `ComplianceGraph::equivalent` | `crosswalk.rs` | **Keep fail-closed.** Obligation projection must use the same honesty: partial/support never equivalent. Do not retarget IR-006. |
| `ExternalRequirementRef` | `framework.rs` | Mapping target for pack/clause ids. Do not copy clause body text. |
| `AssessmentScope` / `ScopeExclusion` | `assessment.rs` | **Stored applicability.** Do not add a provider filter string type. |
| `SubjectSelector` / `SubjectKind` / `SelectorScope` | `subject.rs` | **SSOT selector.** Do not add a third selector type. |
| Facade `AssessmentScope` | `weeping-angel-assurance` | Different type (`BTreeSet<AssetId>` collector allow-set). Do not collapse names. |
| `ScopeResolution` | organizational scope engine | **Consume when landed.** Resolve obligation applicability through it. Do not reimplement nested inclusion/exclusion. |
| `IsmsContext` | ISMS context IR | **Consume when landed** for party/obligation *references*. Do not implement context, issues, objectives, cadence here. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for obligation owner. |
| `ControlId` / `RiskId` | `id.rs` | Mapping targets. Fail closed when the supplied universe does not contain them. |
| `ObligationId` | **share** `typed_id!` | Controlled-documents specifies a minimum versioned ref. This slice hangs `struct Obligation` off the **same** alias. Do not fork. |
| `ControlledDocumentId` | documents slice / this slice min alias | Document mapping target. Share; do not invent `PolicyId`. |
| `Effectiveness` | control-test | **Never** a field on `Obligation`. Mapping is not a test result. |
| `explain_control` / `ControlExplanation` | `weeping-angel-assurance` lineage | Extend additively. Keep `sdd_assessment_lineage_target` GREEN. |
| Kleene `ApplicabilityRule` | IR + assurance | Control/requirement static applicability. **Not** obligation boundary. Obligation uses scope-engine selectors. |
| Collectors / framework packs | collector + `weeping-angel-framework` | Must not mutate obligation satisfaction. Packs keep `Requirement` → `Control` mappings. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_interested_parties_obligations_*` next to existing `sdd_*`. |
| Docs layout | ADR 0004 | Human SSOT is this file. Traces go to `.sdd/runs`. Implement phase may add this path to `sdd_documentation_layout` `CANONICAL_SPECS`. |

JSON names are **camelCase**, matching IR. Empty collections/options use `serde(default)` / `skip_serializing_if`.

Existing `AssessmentDefinition::new` must continue to work. Do not rename `Asset`, `Vendor`, `Risk`, `Control`, `Requirement`, `Mapping`, or `SubjectSelector`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded later by `tests/contracts/interested_parties_obligations.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 No interested-party / obligation IR

[`crates/weeping-angel-assurance-ir/src/lib.rs`](../../crates/weeping-angel-assurance-ir/src/lib.rs) modules: applicability, assessment, asset, control, crosswalk, digest, evidence, exception, extension, framework, id, identity, implementation, mapping, privacy, requirement, risk, subject, test, validation, vendor.

There is **no** `mod party`, **no** `mod obligation`, no `struct InterestedParty`, `RequirementSource`, `Obligation`, or `ObligationMapping`.

[`id.rs`](../../crates/weeping-angel-assurance-ir/src/id.rs) `typed_id!` aliases: `FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `ExceptionId`, `AssessmentId`, `AuditProgramId`, `MappingId`. **No `ObligationId`.** No `InterestedPartyId`, `RequirementSourceId`, or `ObligationMappingId`.

### 3.2 `Requirement` is not an organizational duty

[`requirement.rs`](../../crates/weeping-angel-assurance-ir/src/requirement.rs): `Requirement` always carries `FrameworkRef` (`frameworkId` + `frameworkVersion`), a title, a description, and optional `externalId`. Kinds include `Requirement`, `ControlObjective`, `Clause`, `Article`, … That is pack/catalog language, not “customer Acme requires encryption in transit.”

Golden [`tests/fixtures/assurance-ir/v1/requirement.json`](../../tests/fixtures/assurance-ir/v1/requirement.json) remains a framework requirement. This slice must not retarget it.

### 3.3 Mapping honesty exists only for requirement → control

[`mapping.rs`](../../crates/weeping-angel-assurance-ir/src/mapping.rs): `Mapping { from_requirement, to_control, direction, completeness, relation, rationale, … }`. `MappingRelation::from_completeness` maps `Partial → PartiallySatisfies`. Golden [`mapping.json`](../../tests/fixtures/assurance-ir/v1/mapping.json) uses `"relation": "PartiallySatisfies"` and rationale `"partial mapping; PartiallySatisfies cannot fully satisfy"`.

[`crosswalk.rs`](../../crates/weeping-angel-assurance-ir/src/crosswalk.rs) `ComplianceGraph::equivalent` returns true only for **explicit full bidirectional** edges. IR-006 (`sdd_compliance_ir_target`) asserts a two-hop partial path is **not** equivalent. There is no obligation node in `ComplianceNodeRef`.

ISO remap uses `PartiallySatisfies` / `Supports` onto catalog IDs and **zero** convenience `Equivalent` rows for those identity remaps. Framework packs still cannot represent a customer contract.

### 3.4 Scope types exist; scope engine and ISMS context do not

IR `AssessmentScope { organizations, subjects, exclusions }` and `SubjectSelector { kind, ids, tags, scope }` exist. Facade `AssessmentScope` is a collector asset allow-set.

Product crates contain **no** `struct IsmsContext` and **no** `struct ScopeResolution` on this HEAD (confirmed by neighbor baselines: risk methodology, continuous-assurance scheduler). This slice must not invent replacements.

### 3.5 Explain does not answer organizational “why”

`weeping-angel-assurance::explain_control` builds `ControlExplanation` from pinned run + assessment: control, Kleene applicability, implementation, population, tests, evidence, exceptions, **requirement→control mappings**, effectiveness. No obligation lineage.

### 3.6 Validation has no obligation graph

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs) checks duplicate requirement/control/evidence ids, dangling requirement→control mappings, tests, implementations, risks, exceptions. It does not walk parties, sources, obligations, or obligation mappings.

### 3.7 Collectors and packs cannot (and must not) declare obligation satisfaction

`crates/weeping-angel-collector` has no obligation types. Evidence seal already rejects compliance-shaped narratives. Framework compile projects requirements onto controls; it does not write obligation lifecycle or “satisfied” flags.

### 3.8 Controlled-documents may share `ObligationId` later

[`docs/specs/controlled-documents.md`](controlled-documents.md) specifies a minimum `typed_id!(ObligationId)` so documents can **link** ids. On this HEAD that alias is **absent** (documents product also unlanded). This slice and that slice **share** the alias. Neither forks `ObligationId` into a different newtype.

### 3.9 Schema

`ASSURANCE_IR_SCHEMA == "assurance-ir/v1"`. Generic IR types do not carry ISO clause numbers, Annex A identifiers, or licensed normative text as schema fields.

### 3.10 Dual-suite for this slice is unregistered

Root `Cargo.toml` has no `sdd_interested_parties_obligations_*` entries. `tests/contracts/` is not Cargo auto-discovery.

---

## 4. Desired behavior (target)

### 4.1 Product home and purity

```text
weeping-angel-assurance-ir
  party.rs          # InterestedParty, InterestedPartyKind
  obligation.rs     # RequirementSource, Obligation, ObligationMapping,
                    # ObligationRegistry, validate, current-at-T, projection honesty
  id.rs             # ObligationId (shared), InterestedPartyId,
                    # RequirementSourceId, ObligationMappingId
                    # ControlledDocumentId min alias if still absent
  lib.rs            # mod party + mod obligation + re-exports
  mapping.rs        # consumed enums — unchanged
  assessment.rs     # AssessmentScope consumed — no new inventory field
  subject.rs        # SubjectSelector consumed
  digest.rs         # unchanged canon/v1
weeping-angel-assurance
  lineage.rs        # additive explain_why_control_exists / optional obligations vec
```

Network-free IR. No ISO annex numbers, no provider SDK types, no GRC product vocabulary (`Vanta`, `Drata`, `ServiceNow GRC`) on generic IR types. No `f64`. No UUID v4 persisted ids.

Obligations are **durable governance inputs**, not assessment results. They do not store `Effectiveness`, residual score, or collector-derived “satisfied.”

### 4.2 Identifier aliases

```text
InterestedPartyId, RequirementSourceId, ObligationId, ObligationMappingId
  // typed_id! + StableId — empty / too-long / invalid charset / uuid-v4 still fail
```

`ObligationId` **is** the same type controlled-documents uses for `obligationIds`. If that slice lands first, this slice re-exports the existing alias. If this slice lands first, documents import it. Never `ObligationId(String)` as a second struct.

### 4.3 `InterestedParty`

Provider-neutral party the ISMS owes a duty to, or that imposes a duty.

```text
InterestedPartyKind =
  Internal | External | Customer | Regulator | Insurer | Supplier | Employee | Other(String)
```

JSON: camelCase (`"customer"`, `"regulator"`, …). `Other(String)` is the extensibility hatch.

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `schema_version` | `schemaVersion` | `assurance-ir/v1` |
| `id` | `id` | Stable `InterestedPartyId` |
| `name` | `name` | Human name (required, non-empty) |
| `kind` | `kind` | §4.3 |
| `notes` | `notes` | Optional org-owned note. **Not** licensed legal text. |

`InterestedParty::new(id, name, kind)`. Parties are not deleted; a party that no longer matters remains addressable (obligations retire instead).

### 4.4 `RequirementSource`

Where the duty comes from. Distinguishes categories **without** encoding the law or contract into a control.

```text
RequirementSourceKind =
  Contractual
  | LegalRegulatory
  | Customer
  | InternalPolicy
  | Insurer
  | Supplier
  | Employment
  | Other(String)
```

JSON: camelCase (`"legalRegulatory"`, `"internalPolicy"`, …). Extensible via `Other`.

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `schema_version` | `schemaVersion` | `assurance-ir/v1` |
| `id` | `id` | Stable `RequirementSourceId` |
| `kind` | `kind` | §4.4 |
| `title` | `title` | Short org-owned label (required) |
| `party_id` | `partyId` | Optional `InterestedPartyId` (the imposing / benefiting party) |
| `citation` | `citation` | Optional **short identifier** (contract id, “GDPR Art. 5(1)(e)”, policy code). **Not** quoted normative body text. |
| `external_ref` | `externalRef` | Optional `ExternalRequirementRef` when the source is also a framework clause |

Do **not** store protected ISO/IEC, statute, or contract body text. Citations are pointers. Framework validate-style protected-text markers (`the organization shall`, known ISO excerpts) fail closed on `title`, `citation`, `notes`, `description`, and `rationale`.

### 4.5 `Obligation`

One stable identity. Lifecycle changes **do not** mint a new id. A **successor** obligation (material rewrite) is a **new** `ObligationId` that `supersedes` the predecessor. The predecessor is never deleted.

```text
ObligationLifecycle =
  Draft | Active | Retired | Superseded
```

JSON: camelCase. There is **no** `Deleted` / `Satisfied` variant.

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `schema_version` | `schemaVersion` | `assurance-ir/v1` |
| `id` | `id` | Stable `ObligationId` (survives retirement/supersession) |
| `source_id` | `sourceId` | `RequirementSourceId` (required) |
| `title` | `title` | Short title (required, non-empty) |
| `description` | `description` | Short org-owned paraphrase (required, non-empty). Not licensed text. |
| `applicability` | `applicability` | IR `AssessmentScope` (organizations + `SubjectSelector`s + `ScopeExclusion`s). **Not** a free-form provider filter. |
| `owner` | `owner` | `PrincipalRef` |
| `effective_from` | `effectiveFrom` | Optional start. Drafts may omit. |
| `effective_until` | `effectiveUntil` | Optional end (expiry). `None` = no scheduled expiry. |
| `review_by` | `reviewBy` | Optional review date. Unscheduled ≠ expired. |
| `lifecycle` | `lifecycle` | §4.5 |
| `supersedes` | `supersedes` | Optional predecessor `ObligationId` |
| `extensions` | `extensions` | `ExtensionMap` (same well-formedness rules as other IR) |

Constructor `Obligation::new(id, source_id, title, description, owner)` starts `Draft` with empty `AssessmentScope` and no dates.

**Conflicting or overlapping obligations may coexist.** Validation does **not** merge, reject overlap, or pick a winner. Explain lists all current mappings. Management review / risk treatment (later slices) may surface conflicts; this slice only preserves them.

### 4.6 Applicability uses the scope engine (not provider filters)

Stored shape is always IR `AssessmentScope`:

```text
AssessmentScope { organizations, subjects: Vec<SubjectSelector>, exclusions: Vec<ScopeExclusion> }
```

Forbidden on obligation IR:

- GitHub/Entra/AWS resource filter DSLs
- raw CEL/Rego blobs as the only selector
- collector allow-lists (`BTreeSet<AssetId>` facade scope)
- Kleene `ApplicabilityRule` as the obligation boundary

Resolution at time `T`:

```text
obligation_applies(obligation, scope_inputs, t) -> ObligationApplicability
  InScope | OutOfScope | Conditional | Unknown | Expired | NotCurrent
```

Rules:

1. If `lifecycle` is `Draft` / `Retired` / `Superseded` → `NotCurrent` (record still `get(id)`).
2. If `effective_from` is `Some(start)` and `t < start` → `NotCurrent`.
3. If `effective_until` is `Some(end)` and `t > end` → `Expired` (addressable; **not** in `current_at(t)`).
4. Otherwise resolve `obligation.applicability` through the **organizational scope engine** `ScopeResolution` when that type exists, using the same precedence as the scope-engine spec (inclusions, explicit exclusions, nested subjects, expired exclusions fail closed).
5. If `ScopeResolution` is not yet in tree at implement, resolve selectors against caller-supplied inventories (`Asset` / `Identity` / `Vendor` / `ProcessingActivity` ids) using IR `SubjectSelector` matching — still no provider filters. When the engine lands, switch the helper to call it; do not keep a second precedence table.
6. Ambiguous / unknown scope → `Unknown`, never implicit in-scope evidence.
7. Out-of-scope subjects must not contribute positive assurance that an obligation applies.

`current_at(t)` = lifecycle `Active` **and** applicability not `Expired` / `NotCurrent` / `OutOfScope`. `Unknown` is **not** treated as current satisfaction; it is visible in explain.

Empty `AssessmentScope` (default) means “whole ISMS / unspecified boundary”: treat as in-scope **only** when the caller’s ISMS context / scope engine says the assessment itself is in-scope. Do not interpret empty as “every GitHub repo on earth.”

### 4.7 `ObligationMapping`

Directed, explicit, serializable. Semantic strength is **never inferred**.

```text
ObligationMappingTarget =
  Risk(RiskId)
  | Control(ControlId)
  | Document(ControlledDocumentId)
  | ExternalRequirement(ExternalRequirementRef)
```

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `schema_version` | `schemaVersion` | `assurance-ir/v1` |
| `id` | `id` | `ObligationMappingId` |
| `from` | `from` | `ObligationId` |
| `to` | `to` | tagged target (`{ "risk": "…" }` / `{ "control": "…" }` / `{ "document": "…" }` / `{ "externalRequirement": {…} }`) |
| `direction` | `direction` | `MappingDirection` (`forward` / `reverse` / `bidirectional`) |
| `completeness` | `completeness` | `MappingCompleteness` (`full` / `partial` / `related`) |
| `relation` | `relation` | `MappingRelation` |
| `rationale` | `rationale` | Required, non-empty, org-owned. Not licensed text. |
| `provenance` | `provenance` | `MappingProvenance` |

Default constructor sets `relation` from completeness (`Full → Satisfies`, `Partial → PartiallySatisfies`, `Related → Related`) unless the caller sets an explicit relation **consistent** with completeness (§4.8).

JSON round-trip **must** preserve `direction`, `completeness`, and `relation` independently. A projection helper must not rewrite `PartiallySatisfies` to `Satisfies` because a test passed.

Do not reuse `struct Mapping` (that type is requirement→control). `ObligationMapping` is a sibling record with the same honesty vocabulary.

### 4.8 Projection honesty (must never silently become equivalence)

Reuse the ISO-remap honesty table for **obligation** edges:

| Relation | Completeness allowed | May project as equivalence? | May project as full satisfaction of the obligation? |
| --- | --- | --- | --- |
| `Equivalent` | `full` only, and only with `bidirectional` (or paired reverse full) | Yes | Yes |
| `Satisfies` | `full` | No (unless also equivalent by explicit reverse full) | Yes |
| `SupersetOf` | `full` | No | Yes |
| `PartiallySatisfies` | `partial` (or weaker) | **Never** | **Never** |
| `Supports` | `partial` / `related` | **Never** | **Never** |
| `EvidenceFor` | any | **Never** | **Never** |
| `SubsetOf` | `partial` | **Never** | **Never** |
| `Related` | `related` | **Never** | **Never** |

Rules:

1. `projects_as_equivalence(mapping)` is true only when relation is `Equivalent` **and** completeness is `Full` **and** direction is `Bidirectional` (or an explicit reverse full equivalent edge exists). Match `ComplianceGraph::equivalent` spirit.
2. `projects_as_full_satisfaction(mapping)` is true only for `Equivalent` / `Satisfies` / `SupersetOf` with completeness `Full`.
3. `PartiallySatisfies` and `Supports` **must not** become `Satisfies` or `Equivalent` through serde, graph walk, readiness projection, or explain summaries.
4. Transitive walks do not upgrade strength (A supports B, B satisfies C ⇏ A satisfies C).
5. Validate rejects illegal pairs: `Equivalent` + `partial`; `Satisfies` + `related`; `PartiallySatisfies` + `full` (full + partial-relation is a lie — use `Satisfies` or drop to `partial` completeness).
6. Empty rationale fails validate.
7. Self-mapping (`from` id equal to a control/risk/document id string coincidence is allowed; mapping `from` obligation onto itself is not a defined target — no-op forbidden if `to` were an obligation). Mapping an obligation to a missing target fails closed.

`sdd_compliance_ir_target` IR-006 remains the requirement-graph test. This slice adds **IPO-*** tests on `ObligationMapping` projection so packs/collectors cannot launder strength.

### 4.9 Registry, lifecycle, and addressability

```text
ObligationLinkUniverse {
  party_ids: BTreeSet<InterestedPartyId>,
  source_ids: BTreeSet<RequirementSourceId>,
  obligation_ids: BTreeSet<ObligationId>,
  control_ids: BTreeSet<ControlId>,
  risk_ids: BTreeSet<RiskId>,
  document_ids: BTreeSet<ControlledDocumentId>,
  subject_ids: BTreeSet<String>,          // ids appearing in SubjectSelector.ids
  external_requirement_ok: bool,         // if true, ExternalRequirementRef.external_id must be non-empty
}

ObligationRegistry {
  parties: Vec<InterestedParty>,
  sources: Vec<RequirementSource>,
  obligations: Vec<Obligation>,
  mappings: Vec<ObligationMapping>,
}
```

Queries (names flexible if tests can call them):

| Helper | Behavior |
| --- | --- |
| `get_party` / `get_source` / `get_obligation` / `get_mapping` | By stable id; **including** retired/superseded/expired |
| `current_obligations_at(t)` | `Active` and in-window and not expired |
| `supersession_chain(id)` | Predecessor/successor walk; no deletion |
| `mappings_from(obligation_id)` | All mappings, current or historical |
| `why_control_exists(control_id, t)` | Current mappings to that control + party/source lineage + relation/completeness |
| `why_document_exists(document_id, t)` | Same for document targets |
| `validate(&universe)` | Fail closed (§4.10) |
| `canonical_digest(registry)` | `canon/v1` byte-stable for equivalent BTree ordering |

Lifecycle transitions (pure functions; no clock inside the record except the `t` argument on queries):

| From | To | Rule |
| --- | --- | --- |
| `Draft` | `Active` | Requires non-empty title/description, `source_id`, `owner`, and `effective_from` |
| `Active` | `Retired` | Record remains `get`; excluded from `current_at` |
| `Active` | `Superseded` | Successor exists with `supersedes = this id`; both remain `get` |
| any | delete | **Forbidden.** No API removes the row from the registry |

A superseded obligation **must not** contribute to `current_obligations_at` or to “current why does this control exist?” It **must** appear in lineage/replay (`supersession_chain`, historical `why_*` with `include_historical = true`).

### 4.10 Validation (deterministic, fail closed)

`ObligationRegistry::validate(&ObligationLinkUniverse)` (and/or `ValidateIr`) errors on:

| Case | Error (matchable class or message needle) |
| --- | --- |
| Duplicate `InterestedPartyId` / `RequirementSourceId` / `ObligationId` / `ObligationMappingId` | duplicate stable id |
| `source.party_id` not in registry parties (when `Some`) | dangling party |
| `obligation.source_id` not in registry sources | dangling source |
| `obligation.supersedes` not in registry obligations | dangling predecessor |
| Supersession cycles | cycle |
| `Superseded` without a successor that points at it **or** without its own `supersedes` filled per implementer convention — require: successor has `supersedes = predecessor` and predecessor `lifecycle = Superseded` | inconsistent supersession |
| `Active` + `lifecycle` conflict with `superseded` successor already published as current replacement | predecessor must not stay `Active` |
| Mapping `from` not in registry | dangling obligation |
| Mapping `to` Control/Risk/Document not in **universe** | dangling mapping |
| Empty `title` / `description` / `rationale` / party `name` | empty required field |
| Illegal relation/completeness pair (§4.8) | mapping honesty |
| Protected-text markers in title/description/citation/rationale/notes | protected text |
| `SubjectSelector.ids` not in `universe.subject_ids` when non-empty | dangling scope id |
| Unknown `schemaVersion` | schema mismatch |
| UUID v4 as any typed id | rejected by `typed_id!` |

Conflicting **content** (two Active obligations that disagree) is **not** an error.

### 4.11 Explain path (assurance lineage, not a second digest)

Add a pure helper in `weeping-angel-assurance` (lineage module or thin wrapper):

```text
explain_why_control_exists(control_id, registry, t) -> ObligationExplain
explain_why_document_exists(document_id, registry, t) -> ObligationExplain
```

```text
ObligationExplain {
  target: ControlId | ControlledDocumentId,
  at: DateTime<Utc>,
  current: Vec<ObligationExplainEdge>,      // current_at(t) only
  historical: Vec<ObligationExplainEdge>,   // superseded/expired/retired that still map
}

ObligationExplainEdge {
  party: InterestedParty,
  source: RequirementSource,
  obligation: Obligation,
  mapping: ObligationMapping,
  applicability: ObligationApplicability,
  projects_as_equivalence: bool,
  projects_as_full_satisfaction: bool,
}
```

Deterministic ordering: sort edges by obligation id, then mapping id. Digest of `ObligationExplain` uses `canon/v1`.

Optional additive field on `ControlExplanation`:

```text
obligations: Vec<ObligationExplainEdge>  // default empty; skip_serializing_if empty
```

Must not break `sdd_assessment_lineage_target`. Do **not** put obligation satisfaction or effectiveness on the explain struct.

CLI `assurance explain` may later print these edges; this slice does not require a CLI rewrite. Library path is enough for target tests.

### 4.12 Collectors and framework packs cannot satisfy obligations

Invariants:

1. No field `satisfied` / `obligationStatus` / `Effectiveness` on `Obligation` or `ObligationMapping`.
2. `weeping-angel-collector` must not construct, mutate, or persist obligation lifecycle.
3. Framework pack load/compile must not write `Obligation.lifecycle` or mapping relation upgrades.
4. Evidence envelopes remain facts. Presence of evidence does not mark an obligation satisfied.
5. A `Supports` mapping plus `Effectiveness::Effective` on the control **still** explains as support, not equivalence.

Target tests grep collector + framework sources for mutation APIs and assert projection helpers refuse to upgrade strength.

### 4.13 Fixtures (target suite data)

Author under `tests/fixtures/assurance-ir/v1/` **or** construct in-test. Four required found cases:

| Fixture | Party | Source kind | Obligation id (stable) | Mapping (honest) |
| --- | --- | --- | --- | --- |
| Customer security commitment | `party.customer.acme` (`Customer`) | `Customer` | `obl.customer.security-commitment` | → `control.identity.mfa` `PartiallySatisfies` / `partial` + optional policy `Supports` |
| Employment confidentiality | `party.workforce` (`Employee`) | `Employment` | `obl.employment.confidentiality` | → policy/document `Satisfies` / `full` (org policy **is** the duty vehicle) **and/or** control `Supports` — never silent equivalent to an ISO clause |
| Regulatory retention | `party.regulator.dpa` (`Regulator`) | `LegalRegulatory` | `obl.regulatory.retention` | → `RiskId` `Related`/`Supports` + control `PartiallySatisfies`; citation pointer only |
| Supplier contractual | `party.vendor.payroll` (`Supplier`) | `Contractual` (or `Supplier`) | `obl.supplier.dpa-security` | → document + optional `RiskId`; `PartiallySatisfies` |

Also construct (same suite, not necessarily extra files):

- **Supersession:** `obl.customer.security-commitment` superseded by `obl.customer.security-commitment.2026` (`lifecycle = Superseded` / `Active`). `get` both; `current_at` only successor; explain historical includes predecessor.
- **Expired applicability:** `obl.regulatory.retention` with `effective_until` before `T` → `Expired`; still `get`; not in `current_at(T)`.
- **Dangling mapping:** map to `control.missing` with empty universe → validate error.
- **Duplicate stable ids:** two obligations with the same `ObligationId` → validate error.
- **Partial mapping semantics:** `PartiallySatisfies` + `partial` to `control.identity.mfa` → `projects_as_equivalence == false` and `projects_as_full_satisfaction == false` even if a control test is `Effective`.

Protected text: a description containing a known ISO “the organization shall” excerpt fails validate.

### 4.14 Interaction with `IsmsContext`

When ISMS context IR is present:

```text
IsmsContext.interested_party_ids: Vec<InterestedPartyId>   // references only
IsmsContext.obligation_ids: Vec<ObligationId>             // references only
```

This slice owns the **records**. Context owns the **root membership list**. Dangling ids fail closed on context validate (context slice) and on this registry validate (this slice). Do not embed full obligation bodies inside `IsmsContext`.

### 4.15 Target suite contents (RED on CURRENT, GREEN after)

Stable titles. Author **before** product feature code. Prefer compile-safe source needles plus later public helpers so RED is a named assertion (missing `struct Obligation` / helpers), not unrelated compile noise. When types land, keep the same ids and assert behavior through the public API.

| ID | Assertion |
| --- | --- |
| IPO-001 | Customer security commitment fixture constructs, serializes (`assurance-ir/v1`, camelCase), deserializes, validates, and explains `why_control_exists` for the mapped control. |
| IPO-002 | Employment confidentiality fixture maps to a policy/document with explicit relation; explain_why_document_exists returns the party/source/obligation edge. |
| IPO-003 | Regulatory retention fixture uses `LegalRegulatory` + citation pointer (no normative body); maps to risk and control without copying protected text. |
| IPO-004 | Supplier contractual fixture uses contractual/supplier source and remains provider-neutral (no AWS/GitHub SDK types). |
| IPO-005 | Supersession: predecessor remains `get_obligation`; `current_obligations_at(T)` excludes it; successor is current; chain is replayable. |
| IPO-006 | Expired applicability: `effective_until < T` → not current, still addressable; explain historical can include it. |
| IPO-007 | Dangling mapping target (control/risk/document) fails closed. |
| IPO-008 | Duplicate `ObligationId` (and party/source/mapping ids) fails closed. |
| IPO-009 | `PartiallySatisfies` / `Supports` never `projects_as_equivalence` or `projects_as_full_satisfaction`; serde round-trip preserves relation + completeness + direction. |
| IPO-010 | Illegal pair `Equivalent` + `partial` fails validate. |
| IPO-011 | Applicability is IR `AssessmentScope` / `SubjectSelector`; product sources contain no obligation provider-filter type (`githubOrg`, `awsAccountFilter`, facade allow-set as stored applicability). When `ScopeResolution` exists, obligation resolution calls it. |
| IPO-012 | Conflicting overlapping Active obligations both remain current; validate succeeds. |
| IPO-013 | Collectors and framework crate sources do not set obligation satisfaction / lifecycle. |
| IPO-014 | `explain_why_control_exists` is deterministic (`canon/v1` digest stable under reorder-then-sort). |
| IPO-015 | Dual-suite names registered in root `Cargo.toml`. Schema remains `assurance-ir/v1`. No ISO Annex A fields on `Obligation`. |
| IPO-016 | `ObligationId` is a single `typed_id!` alias (shared with controlled-documents). |
| IPO-017 | `Requirement` type is unchanged as a framework clause; obligations are a distinct type (not `RequirementKind::Obligation` bolted onto framework requirements). |
| IPO-018 | Retired/superseded obligations have no delete API; registry still returns them by id. |

One regression test per later review comment must be titled `P?: <exact subject>` and encode the original found case (test first → RED → fix → GREEN).

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | On this HEAD |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/interested_parties_obligations.baseline.rs` | `sdd_interested_parties_obligations_baseline` | skip-superseded (`#[ignore = "superseded by sdd_interested_parties_obligations_target"]`); §3 is characterization of `6e31bf1a…` |
| Target | `tests/contracts/interested_parties_obligations.target.rs` | `sdd_interested_parties_obligations_target` | **GREEN** (IPO-001–018) |

Protocol (completed): spec + ADR Accepted; target GREEN; baseline skip-superseded. Neighbors `sdd_compliance_ir_target`, `sdd_assessment_lineage_target`, `sdd_applicability_engine_target` stay GREEN.

Traces only under `.sdd/runs/` and `.sdd/artifacts/`.

Test comments and spec/ADR prose name the slice **interested parties / obligations**, never a seed index.

---

## 6. Acceptance criteria (testable)

- **IPO-001** Customer security commitment fixture round-trips and explains why the mapped control exists.
- **IPO-002** Employment confidentiality fixture explains why the linked policy/document exists.
- **IPO-003** Regulatory retention fixture cites a legal/regulatory source without storing protected normative text.
- **IPO-004** Supplier contractual fixture is provider-neutral and maps with explicit relation/rationale.
- **IPO-005** Superseded obligations remain replayable and are excluded from `current_obligations_at`.
- **IPO-006** Expired applicability is addressable and not current.
- **IPO-007** Dangling mappings fail closed.
- **IPO-008** Duplicate stable ids fail closed.
- **IPO-009** Partial/supporting mappings cannot be promoted to equivalence or full satisfaction through projection or serde.
- **IPO-010** Illegal `Equivalent` + `partial` (and listed honesty pairs) fail validate.
- **IPO-011** Applicability uses canonical scope-engine / IR `AssessmentScope` + `SubjectSelector`, not free-form provider filters.
- **IPO-012** Conflicting/overlapping current obligations may coexist.
- **IPO-013** Collectors and framework packs cannot mutate obligation satisfaction.
- **IPO-014** Explain is deterministic via `canon/v1`.
- Dual-suite registered; schema remains `assurance-ir/v1`; `ObligationId` is shared, not forked.
- Neighbor targets listed in the header stay GREEN after implement.

---

## 7. Out of scope

- Legal advice engine, statute interpretation, or “are we compliant with GDPR?” conclusions.
- Scraping or ingesting regulation / ISO / contract body text (NLP, crawlers, licensed corpus).
- Hardcoding ISO Annex A controls or packing ISO clause numbers onto `Obligation`.
- Collectors declaring obligations satisfied; framework packs writing obligation lifecycle.
- Implementing `IsmsContext` (ISMS context IR) or `ScopeResolution` (organizational scope engine) in this slice — consume only.
- Controlled-document versioning, e-sign, or document-control registry (share `ObligationId` / document mapping targets only).
- Risk scoring, treatment plans, residual math, operational SoA projection, Kleene evaluator rewrite.
- UI, persistence service, workflow inbox, auditor portal.
- Forking `assurance-ir/v1` or inventing a second digest (`canon/v1` only).
- Parallel org/GRC graph (second `SubjectSelector`, second party-as-vendor type, stuffing results into context).

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Controlled-documents lands a different `ObligationId` newtype | One `typed_id!(ObligationId)` alias; share; this slice owns `struct Obligation`. |
| ISMS context IR / scope engine not landed at implement | Registry standalone; store `AssessmentScope`; switch resolution to `ScopeResolution` when present; context holds refs only. |
| Overloading `Requirement` as org duty | IPO-017: distinct types; do not add `RequirementKind::Obligation`. |
| `Supports` laundered to `Equivalent` in explain/readiness | IPO-009/010 + reuse `MappingRelation` + no transitive upgrade. |
| Provider filters sneak in as “scope” | IPO-011; stored type is IR `AssessmentScope` only. |
| Collectors mark duties satisfied | IPO-013; no satisfaction field; grep collector/framework. |
| Protected ISO/contract text in descriptions | Validate protected-text markers; fixtures use pointers. |
| Deleting superseded rows for “cleanup” | No delete API; IPO-005/018; `get` always. |
| Parallel GRC graph / second selector | Collision fence; reuse `SubjectSelector`, `PrincipalRef`, `Vendor` for suppliers. |
| Lineage suite breaks from `ControlExplanation` shape change | Additive default-empty field or sibling explain API. |
| Neighbor suites break | Do not edit their files; verify listed targets stay GREEN. |
| AssessmentDefinition inventory collision | Standalone `ObligationRegistry`; no new assessment field in this slice. |

---

## 9. ADR

Accepted: [`docs/adr/0043-interested-parties-obligations.md`](../adr/0043-interested-parties-obligations.md). **0004** remains documentation architecture. Cite by path.

---

## 10. Landed files

Product:

- `crates/weeping-angel-assurance-ir/src/party.rs`
- `crates/weeping-angel-assurance-ir/src/obligation.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (shared `typed_id!` aliases)
- `crates/weeping-angel-assurance-ir/src/lib.rs` (`mod party`, `mod obligation`)
- `crates/weeping-angel-assurance/src/lineage.rs` (`explain_why_control_exists` / `explain_why_document_exists`; additive `ControlExplanation.obligations`)

Tests/docs:

- `tests/contracts/interested_parties_obligations.{baseline,target}.rs`
- root `Cargo.toml` `[[test]]` rows
- this spec; [`docs/adr/0043-interested-parties-obligations.md`](../adr/0043-interested-parties-obligations.md)
- `sdd_documentation_layout` `CANONICAL_SPECS` entry `docs/specs/interested-parties-obligations.md`
