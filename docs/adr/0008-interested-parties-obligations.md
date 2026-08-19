# ADR 0008 — Interested parties and obligations (governance inputs ≠ controls)

<!-- weeping-angel-adr-meta
id = "0008"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_interested_parties_obligations_target` GREEN (IPO-001–018); `sdd_interested_parties_obligations_baseline` skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Adds a public IR obligation **registry** contract. Does **not** supercede crate-root `isms::InterestedParty` / `isms::Obligation` (context membership graph), framework `Requirement` / `Mapping`, `ComplianceGraph::equivalent`, `canon/v1`, Kleene applicability, or collector evidence-as-fact. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [ISO remap honesty](0003-iso27001-canonical-remap.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [assessment lineage](0003-assessment-lineage.md), [ADR 0004](0004-documentation-architecture.md), [ISMS context](0008-isms-context.md) |
| Spec | [`docs/specs/interested-parties-obligations.md`](../specs/interested-parties-obligations.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Seed | [`docs/prompts/operational-isms-v1/03-interested-parties-obligations.md`](../prompts/operational-isms-v1/03-interested-parties-obligations.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | Dual-suite `sdd_interested_parties_obligations_baseline` / `sdd_interested_parties_obligations_target` at `tests/contracts/interested_parties_obligations.{baseline,target}.rs` (root `Cargo.toml`). Neighbors `sdd_compliance_ir_target`, `sdd_assessment_lineage_target`, `sdd_applicability_engine_target` stay GREEN. |

> Filename **`0008-*`**. Cite **this file by path**. **0004** is documentation architecture. Concurrent Operational ISMS siblings also use `0008-*` ([ISMS context](0008-isms-context.md), [scope engine](0008-scope-engine.md), [security objectives](0008-security-objectives.md)).

## Context

On SHA `6e31bf1a…`, Weeping Angel could project a framework `Requirement` onto a canonical `Control` with honest `MappingRelation` (`PartiallySatisfies` / `Supports` never inferred as `Equivalent` — IR-006 / ISO remap). It could explain a pinned assessment run. It could not record **why this organization** has a duty: customer commitments, employment confidentiality, regulatory retention, supplier contracts.

Missing at characterization:

1. No registry `InterestedParty`, `RequirementSource`, `Obligation`, or `ObligationMapping` in `weeping-angel-assurance-ir`.
2. No `ObligationId` (`typed_id!` stopped at `MappingId`). Controlled-documents specified a *minimum* `ObligationId` ref; that alias was unlanded.
3. Concurrent ISMS context IR and organizational scope engine were unlanded on that SHA.
4. `explain_control` did not walk organizational duties.
5. Risk that a later slice would encode every law/contract as a `Control`, or that `Supports` mappings would be projected as equivalence.

Operational ISMS v1 needed a **canonical obligation layer** between organizational context and controls without a legal-advice engine, regulation scrape, contract NLP, hardcoded ISO controls, or collectors declaring duties satisfied.

Questions this decision answers:

1. Are organizational duties `Requirement`s or a distinct type?
2. Where do records live (IR vs a new crate vs stuffed into `IsmsContext`)?
3. How do mappings stay honest (`PartiallySatisfies` / `Supports` ≠ equivalence)?
4. How is applicability scoped without provider filters?
5. How do we share `ObligationId` with controlled-documents and the context graph?
6. May collectors or packs mark an obligation satisfied?
7. What happens to superseded/expired duties?

## Decision

This is what shipped. Field-level law is [`docs/specs/interested-parties-obligations.md`](../specs/interested-parties-obligations.md).

### 1. Distinct registry types, same schema, no new crate, no crate-root collision

Registry types live in existing `weeping-angel-assurance-ir`:

| Type | Home |
| --- | --- |
| `party::InterestedParty` / `InterestedPartyKind` | [`party.rs`](../../crates/weeping-angel-assurance-ir/src/party.rs) |
| `obligation::RequirementSource`, `Obligation`, `ObligationMapping`, `ObligationRegistry` | [`obligation.rs`](../../crates/weeping-angel-assurance-ir/src/obligation.rs) |
| `ObligationId`, `InterestedPartyId`, `RequirementSourceId`, `ObligationMappingId` | `id.rs` `typed_id!` (crate-root) |

Schema remains `assurance-ir/v1`. Modules are public (`mod party`, `mod obligation`). Registry structs are **not** crate-root `pub use` — crate-root `InterestedParty` / `Obligation` stay the **ISMS membership-graph** records (`isms.rs`). Callers name `weeping_angel_assurance_ir::party::InterestedParty` and `…::obligation::{Obligation, ObligationRegistry, …}`. Shared **id** newtypes are crate-root.

A framework `Requirement` stays a pack/catalog clause. A registry `Obligation` is an **organization-specific duty**. There is no `RequirementKind::Obligation`. Document-control and ISO Annex A are not obligation subtypes.

The registry is a **standalone IR document**. This slice does **not** add `AssessmentDefinition.obligations`. `IsmsContext` membership rows remain graph stubs; `ObligationRegistry::validate_against_isms_context` fail-closes when a context obligation id is missing from the registry. Bodies are not embedded in context.

### 2. Share `ObligationId`; reuse identity and digest law

`ObligationId`, `InterestedPartyId`, `RequirementSourceId`, and `ObligationMappingId` are `typed_id!` aliases. Controlled-documents and the context graph use the **same** `ObligationId` / `InterestedPartyId` — no forked newtype. `ControlledDocumentId` is the document mapping target.

Reuse `canon/v1` (`canonical_digest` / `typed_canonical_digest`). No UUID v4 persisted ids. Explain path is additive in `weeping-angel-assurance` lineage, not a second identity system.

### 3. Honest mappings reuse existing vocabulary

`ObligationMapping` is a sibling of framework `Mapping`, not a reuse of the requirement→control struct. It uses `MappingDirection`, `MappingRelation`, `MappingCompleteness`, and `MappingProvenance`. JSON round-trip preserves all three independently.

Targets: `Risk(RiskId)`, `Control(ControlId)`, `Document(ControlledDocumentId)`, `ExternalRequirement(ExternalRequirementRef)`. Direction and semantic strength are explicit in serialization.

Projection (shipped helpers):

- `projects_as_equivalence` is true only for `Equivalent` + `Full` + `Bidirectional`.
- `projects_as_full_satisfaction` is true only for `Equivalent` / `Satisfies` / `SupersetOf` with `Full`.
- `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` never project as equivalence or full satisfaction.
- Transitive walks do not upgrade strength. Validate rejects illegal relation/completeness pairs (including `Equivalent` + `partial`).

### 4. Applicability is IR scope selectors, not provider filters

Stored shape is IR `AssessmentScope` (`organizations` + `SubjectSelector` + `ScopeExclusion`). No GitHub/Entra/AWS filter DSLs, no facade collector allow-sets as stored applicability, no parallel org graph.

Shipped `obligation_applies` matches selectors against `ObligationLinkUniverse.subject_ids`. The scope-engine `ScopeResolution` type lives in `weeping-angel-assurance::scope`; this slice does not reimplement inclusion/exclusion precedence and does not store provider filters.

- Empty `AssessmentScope` → `InScope` (ISMS/unspecified boundary, not “every provider resource”).
- `Draft` / `Retired` / `Superseded` / before `effective_from` → `NotCurrent`.
- After `effective_until` → `Expired`.
- Ambiguous selector ids → `Unknown`.
- `current_obligations_at(T)` keeps `Active` rows that are in-scope; it excludes `Expired`, `NotCurrent`, `OutOfScope`, and `Unknown`.

### 5. Lifecycle: retire/supersede, never delete

`Draft | Active | Retired | Superseded`. No `Deleted` / `Satisfied`. Historical and superseded obligations remain `get_obligation` / `get_mapping` addressable. `supersession_chain` walks predecessor/successor. A `Superseded` row without a successor, or an `Active` predecessor with a published successor, fails validate. Conflicting/overlapping Active obligations may coexist.

### 6. Governance inputs, not assessment results

No `Effectiveness`, no collector-written `satisfied` flag, no pack mutation of obligation lifecycle. Evidence remains facts. A green control test plus a `Supports` mapping is still support (`projects_as_full_satisfaction == false`).

### 7. No protected normative text

Titles, descriptions, citations, notes, and rationales are org-owned short text and identifiers. Validate fails closed on protected-text markers (`the organization shall`, concatenated annex needle, `iso/iec 27001`). Do not scrape statutes or copy ISO/IEC bodies.

### 8. Owner reuse

`PrincipalRef` is the obligation owner. There is no `ObligationOwner`.

### 9. Additive explain (lineage, not a second digest)

`weeping-angel-assurance::explain_why_control_exists` / `explain_why_document_exists` return `ObligationExplain` (`current` + `historical` edges, deterministic sort by obligation id then mapping id, `canon/v1` digest). `ControlExplanation.obligations` is additive (`serde default`, skip if empty). `explain_control` does not populate it; the dedicated helpers do. `sdd_assessment_lineage_target` stays GREEN.

## Non-goals

Legal advice; regulation/contract NLP or scrape; hardcoded ISO controls; collector satisfaction; implementing `IsmsContext` or `ScopeResolution` in this slice (consume only); document registry product; risk scoring / treatment / SoA; UI; forking `assurance-ir/v1`.

## Consequences

- Reviewers can answer `why does this control/policy exist?` through party → source → obligation → honest mapping.
- Management review and risk treatment can consume `ObligationId` without inventing a second GRC graph.
- Controlled-documents and supplier-risk link the shared `ObligationId` without owning registry `struct Obligation`.
- Crate-root `isms::Obligation` remains a membership stub; registry records live under `obligation::`.
- Scope-engine product still owns `ScopeResolution` / `resolve_scope`; obligation stored shape remains IR `AssessmentScope`.
- Collision fence remains: do not retarget `Requirement` / `Mapping` / `ComplianceGraph::equivalent`, catalog TOML, ISO remap, collectors, or the Kleene evaluator.

## Related

- Spec: [`docs/specs/interested-parties-obligations.md`](../specs/interested-parties-obligations.md)
- Tests: `tests/contracts/interested_parties_obligations.{baseline,target}.rs`
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Context membership graph: [ADR 0008 ISMS context](0008-isms-context.md)
- Controlled-documents `ObligationId`: [`docs/adr/0003-controlled-documents.md`](0003-controlled-documents.md)
