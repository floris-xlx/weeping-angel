# Elite contract + architecture spec — deepen weeping-angel Compliance IR

| Field | Value |
| --- | --- |
| Product | `weeping-angel-assurance-ir` |
| Repo | [`floris-xlx/weeping-angel`](https://github.com/floris-xlx/weeping-angel) |
| Branch / SHA | `main` / `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7` |
| Date | 2026-08-18 |
| Recipe | contract + R13 architecture specification |
| Family | `FAM-CONTRACT` + `FAM-ARCH` + `FAM-PHASED` |
| Prior spine | [`docs/sdd/assurance-runtime-spine.md`](../../assurance-runtime-spine.md) (Phases 0–8 **Implemented**) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../../../contracts/assurance-runtime.md) |
| ADR | [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../../../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Self-score | **Forbidden.** Status = invariant count enforced · ACT green/red · open AD-* · surfaces closed Y/N |

## Goal

Promote the existing thin, architecturally correct Compliance IR into the **canonical semantic authority** of the assurance system — without redesigning the compiler, collectors, evidence, or control-test runtime.

The spine already solved the hard boundary problem. This program **deepens** typed identity, `Requirement` / `Control` / `Mapping` / evidence / test / applicability / subject / implementation / ownership / risk / exception / asset / identity / vendor / processing-activity models, moves `AssessmentDefinition` + `AssessmentScope` into the IR crate, and freezes compatibility with golden fixtures and IR-001…025.

## Repositories

| Role | Path | Canonical owner |
| --- | --- | --- |
| Target | `floris-xlx/weeping-angel` @ `8c0f36e` | this pack |
| IR crate | `crates/weeping-angel-assurance-ir/` | IR writes |
| Framework compiler | `crates/weeping-angel-framework/` | compile only |
| Facade | `crates/weeping-angel-assurance/` | composition |
| Control-test runtime | `crates/weeping-angel-control-test/` | evaluate only |
| Evidence / collector | `crates/weeping-angel-evidence/`, `crates/weeping-angel-collector/` | observations |
| Scanner | `src/**` | security documents only |
| Corpus | `~/Documents/spec-driven-development` | catalog IDs |

## Context

Re-evaluated against `floris-xlx/weeping-angel` `main` `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7`. Architectural correctness of the IR is high. Semantic completeness of a production-grade canonical assurance IR is **55–60%**.

Live evidence (HEAD):

- IR is two files: [`crates/weeping-angel-assurance-ir/src/lib.rs`](../../../../crates/weeping-angel-assurance-ir/src/lib.rs) (~296 lines) + [`crosswalk.rs`](../../../../crates/weeping-angel-assurance-ir/src/crosswalk.rs).
- `typed_id!` constructs `ControlId::new("")` with no validation (`lib.rs` L15–37).
- `Control` is `{ schema_version, id, title, description }` (L100–138).
- `Requirement` is `{ id, framework_id, framework_version, title, description }` (L142–188).
- `Mapping` is `{ from_requirement, to_control, direction, completeness }` (L190–231).
- `EvidenceRequirement` is `{ id, evidence_type }` (L233–257).
- `PlannedControlTest` is `{ id, control_id, kind, required_evidence, break_on }` (L259–281).
- `canonical_digest<T: Serialize>` hashes arbitrary serde JSON (L289–295).
- `ComplianceGraph` stores `(RequirementId, RequirementId) → MappingCompleteness` only (`crosswalk.rs` L8–10). Equivalence still requires full bidirectional edges (L36–44).
- `Assessment` lives in [`crates/weeping-angel-framework/src/lib.rs`](../../../../crates/weeping-angel-framework/src/lib.rs) L101–111.
- `resolve_applicability` returns `assessment.requirements.clone()` (L265–271) — the stage name exists; the IR has nothing to resolve.
- Facade `AssessmentScope` is an `AssetId` set in [`crates/weeping-angel-assurance/src/lib.rs`](../../../../crates/weeping-angel-assurance/src/lib.rs) L37–59.
- IDs exist without records: `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `RiskId`, `ExceptionId`, `ControlImplementationId`, `AuditProgramId`.
- Target suite `tests/sdd/assurance_runtime.target.rs` encodes ACT-001…015 + COL-001…006. That suite stays green. It does **not** cover IR semantic depth.

The user implementation sketch (slices A–F) is accepted as the **work breakdown**. This pack is the Level-3 specification those slices must satisfy.

## Why this architecture exists

Goals

- Reduce ambiguity: one place defines what a Control, Requirement, Mapping, and Assessment **are**.
- Reduce documentation burden: golden JSON + `ValidateIr` replace ad-hoc compiler comments.
- Reduce onboarding cost: module-per-entity instead of a growing `lib.rs`.
- Reduce migration complexity: N / N-1 schema policy before catalogs land.
- Reduce maintenance surface: framework adapters map onto canonical domains; they do not grow `Control`.
- Increase tooling leverage: structural validation before `compile_framework`.
- Increase consistency: `FrameworkRef`, digest domain, extension namespaces.
- Increase discoverability: `AssessmentDefinition` in the IR crate, not the compiler crate.

Non-goals (explicit)

- ISO 27001 / GDPR / SOC 2 / NIS2 / DORA catalogs.
- Real GitHub / AWS / Cloudflare collectors.
- Scheduler, continuous monitoring, persistent evidence ledger.
- Control-test expression evaluator / predicate AST.
- Policy management, vendor-management workflows, audit UI, trust center, questionnaires.
- Moving `EvidenceEnvelope`, `ControlTestResult`, `Effectiveness`, `CollectorDescriptor`, `FrameworkTarget`, or `CompiledFramework` into the IR.

## Working assumptions

- Phases 0–8 spine remains law. INV-1…5 and ACT-001…015 / COL-001…006 must stay green.
- Deepen, do not redesign. The chain `Requirement → Mapping → Canonical Control → Planned Control Test → Evidence Requirement` is frozen.
- Provider/network types stay out of `weeping-angel-assurance-ir`.
- Framework-specific semantics stay out of `Control`.
- Persistent identity is deterministic and non-random. No UUIDv4 in persisted IR.
- Mappings are explicitly directional. Equivalence is never inferred.
- `ImplementationStatus != ControlEffectiveness`.
- Slice A mechanical split is JSON-stable. Semantic additions start only after workspace tests stay green on the split.
- Catalog IDs in this pack come from `catalogs/05-ISSUE-CATALOG.md`. IR-001…025 are **requirement / ACT IDs**, not catalog findings.

## Out of scope

See Non-goals. This pass succeeds only if it makes those later surfaces easier — not if it builds them.

---

## Architecture invariants

The following are true at every commit after **Phase 6** (compatibility freeze). Doctor / ACT fail-closed.

| ID | Boolean claim | Enforcement | Owner |
| --- | --- | --- | --- |
| INV-IR-A | Requirement → Mapping → Control → PlannedControlTest → EvidenceRequirement is the only definition chain | IR-003…006, compile pipeline | `weeping-angel-assurance-ir` |
| INV-IR-B | IR crate has no provider/SDK/network types | ACT-006 path denylist + cargo deps | IR |
| INV-IR-C | `Control` has no framework-specific fields | IR-003, ACT-006 JSON key denylist | IR |
| INV-IR-D | `Requirement` carries `FrameworkRef`; `Control` does not | IR-004, IR-024 | IR |
| INV-IR-E | Control definition ≠ `ControlImplementation`; implementation status ≠ effectiveness | IR-008, IR-009 | IR |
| INV-IR-F | Persisted IDs validate (`try_new`); empty / overlong / illegal charset rejected | IR-001, IR-022 | IR |
| INV-IR-G | Equivalence only via explicit full bidirectional `Equivalent` + `Full` | IR-006, ACT-005 | IR |
| INV-IR-H | Generated mappings cannot silently equal curated authority | IR-007 | IR |
| INV-IR-I | `AssessmentDefinition` and semantic `AssessmentScope` live in the IR crate | ACT-IR-ASSESS, OWN-001 close | IR |
| INV-IR-J | Runtime results (`EvidenceEnvelope`, `ControlTestResult`, `CompiledFramework`) stay out of IR | cargo graph + ACT-013 | framework / evidence / control-test |
| INV-IR-K | Canonical digest uses domain-separated `CanonicalSerialize` | IR-015, IR-016 | IR |
| INV-IR-L | Doctor / target suite fails if any INV-1…5 or INV-IR-* becomes false | ACT-001…015 + IR-001…025 | tests |

Invariant H (doctor): **a PR that makes any row false after Phase 6 is a failed PR.**

## Ownership matrix

| Concern | Canonical owner | Must not own |
| --- | --- | --- |
| Typed IDs + `StableId` + `IdError` writes | `weeping-angel-assurance-ir` | framework, facade, scanner |
| `Requirement` / `Control` / `Mapping` / `ApplicabilityRule` writes | IR | framework adapters |
| `ControlImplementation` / `PrincipalRef` / `Asset` / `Identity` / `Risk` / `Exception` / `ProcessingActivity` writes | IR | collectors, scanner |
| `AssessmentDefinition` + semantic `AssessmentScope` writes | IR | framework, facade |
| `FrameworkTarget` / `compile_framework` / `CompiledFramework` writes | `weeping-angel-framework` | IR, collector |
| `EvidenceEnvelope` writes | `weeping-angel-evidence` | IR |
| `CollectorDescriptor` / collect writes | `weeping-angel-collector` | IR, framework |
| `evaluate` / `ControlTestResult` writes | `weeping-angel-control-test` | IR |
| `SemanticFinding` / `EngineHit` writes | root scanner `src/` | IR |
| Facade `assess` orchestration writes | `weeping-angel-assurance` | IR entity schemas |

## Ownership / dependency graph

```text
weeping-angel-assurance-ir
        ├── weeping-angel-framework          (compile only)
        └── weeping-angel-evidence
                └── weeping-angel-collector

weeping-angel-assurance-ir + weeping-angel-evidence
        └── weeping-angel-control-test

weeping-angel-framework
  + weeping-angel-collector
  + weeping-angel-control-test
  + weeping-angel (scanner types, bridge only)
        └── weeping-angel-assurance
```

Edges not shown are forbidden.

| From | To | Why forbidden |
| --- | --- | --- |
| IR | framework / collector / control-test / scanner | IR is the bottom of the graph |
| IR | reqwest / AWS / GitHub / Cloudflare SDKs | INV-IR-B |
| framework | collector / control-test / network | INV-3 / ACT-003 |
| collector | framework / ISO / GDPR / SOC2 crates | INV-2 / ACT-013 |
| control-test | collector / network clients | INV-4 |
| scanner `src/` | IR Control fields | INV-1 |
| `Control` type | `owner` / `implemented` fields | INV-IR-E |

## Package boundaries

### weeping-angel-assurance-ir owns

- typed IDs, `FrameworkRef`, schema/digest helpers
- definition entities and graph
- `ValidateIr` / `validate_assessment_ir`
- golden IR fixtures

### weeping-angel-framework must never

- own `AssessmentDefinition` after Phase 5
- perform network I/O
- invent applicability or subject models that are not IR types

### weeping-angel-assurance must never

- define a second semantic `Assessment` document
- treat collector `AssetId` sets as the only scope language after Phase 5 (it may **translate** IR scope → `CollectorScope`)

### weeping-angel-control-test must never

- take a provider id
- auto-pass on empty / stale / manual-without-attestation evidence

### scanner must never

- grow `iso_27001` / `gdpr` / `soc2` fields

## Runtime surface

Public IR crate after Phase 6. Additional exports require a plan amendment + schema note.

### `crates/weeping-angel-assurance-ir/src/lib.rs`

exports: module re-exports listed below. Nothing else at crate root.

| Module | Public exports | Nothing else |
| --- | --- | --- |
| `id` | typed IDs, `StableId`, `IdError`, `try_new` | no UUIDv4 helpers |
| `framework` | `FrameworkRef`, `ExternalRequirementRef` | no `FrameworkTarget` |
| `requirement` | `Requirement`, `RequirementKind` | no ISO annex types |
| `control` | `Control`, `ControlDomain`, `ControlExpectation` | no owner / implemented |
| `implementation` | `ControlImplementation`, `ImplementationStatus` | no `Effectiveness` |
| `mapping` | `Mapping`, `MappingId`, `MappingRelation`, `MappingConfidence`, `MappingProvenance`, `MappingVersionConstraint` | no inferred equivalence |
| `applicability` | `ApplicabilityRule`, `ApplicabilityPredicate` | no platform fact evaluators |
| `subject` | `SubjectKind`, `SubjectSelector`, `SelectorScope` | no `GithubRepositorySelector` |
| `evidence` | `EvidenceRequirement`, cardinality / freshness / collection / criticality | no envelope / collector |
| `test` | `PlannedControlTest`, `TestEvaluationRef`, `TestFailureSeverity` | no predicate AST |
| `assessment` | `AssessmentDefinition`, `AssessmentScope`, `ScopeExclusion` | no `CompiledFramework` |
| `asset` / `identity` / `vendor` / `risk` / `exception` / `privacy` | minimal records | no IAM / RoPA / risk engine |
| `crosswalk` | `ComplianceGraph`, `ComplianceNodeRef`, `ComplianceEdge` | no transitive `equivalent` |
| `validation` | `ValidateIr`, `IrValidationError`, `validate_assessment_ir` | no compile |
| `digest` | `CanonicalSerialize`, `CanonicalDigest`, domain prefix | no raw `Serialize` identity |
| `extension` | `ExtensionMap` + namespace rules | no canonical-field override |

### Downstream crates (unchanged allowlist)

- `weeping-angel-framework`: `FrameworkTarget`, `compile_framework`, `CompiledFramework`, `FrameworkCompileError`
- `weeping-angel-assurance`: `AssuranceEngine`, `AssessmentReport`; facade `AssessmentScope` becomes a **translator**, not a second SSOT

## Generated surface

This crate is **handwritten SSOT**, not a generator.

generator owns: nothing.

Allowlist for checked-in derived artifacts (tests, not runtime):

- `tests/fixtures/assurance-ir/v1/*.json` — golden serialized IR
- `docs/contracts/assurance-runtime.md` — human contract (hand-maintained, must match goldens)

Forbidden drift:

- `generated/helpers.rs`
- per-framework types inside IR (`IsoAnnexControl`, `GdprArticle`, `Soc2Criterion`)
- collector selectors inside IR

The long-term abstraction for fixture families is a **directory**: `tests/fixtures/assurance-ir/v1/` (not a single `models.rs`).

## Runtime guarantees

### IR documents

- deterministic serde `camelCase`
- `schema_version` present
- digest domain `wa:assurance-ir:<schema>:<type>:`
- unknown namespaced extensions survive round-trip
- no module-level RNG

### `id::try_new`

- fail-closed on empty, whitespace-only, overlong, illegal charset
- `new_unchecked` crate-private / test-only
- title changes do not mutate identity

### `ComplianceGraph::equivalent`

- false for identity (`a == a` is not a mapping)
- false for partial / related / one-way
- false for path `A → B → C`
- true only for explicit full bidirectional `Equivalent`

## Canonical definition

Canonical means all of the following are true for a type:

- single handwritten SSOT in `weeping-angel-assurance-ir` (no twin in framework/facade after Phase 5)
- documented in `docs/contracts/assurance-runtime.md`
- tested (IR-001…025 + goldens)
- ACT / compile-fail enforced
- migration target for framework/facade consumers
- examples (`examples/weeping-angel-demo.rs` later; fixture collector assessments now) use it

If any one is false, it is not canonical.

## Generated definition

Generated means: replaceable, reproducible, deterministic, never manually edited, regenerated from a documented source.

This program **does not** introduce a code generator. Golden JSON is **checked-in expected output**, reviewed as fixtures, not hand-crafted production logic.

## Compatibility policy (N and N-1)

| Rule | Policy |
| --- | --- |
| Current | `assurance-ir/v1` |
| N-1 | same major; optional field additions only |
| Breaking | field remove/rename or semantic change → `assurance-ir/v2` |
| IDs | never change because display title changed |
| Digest | `IrSchemaVersion` and `CanonicalizationVersion` are independent |
| Enums | external serialization must tolerate unknown / future variants where the document is persisted |
| Construction | after Phase 1, empty IDs fail `try_new`; previously accepted empty IDs are not in HEAD fixtures (no live N-1 documents) |
| Doctor | warn if consumer still constructs via removed `new("")` after Phase 1; fail if serialized document lacks `schemaVersion` after Phase 6 |
| Migration | framework `Assessment` type aliases / re-exports N-1 for one slice, then delete |

## Performance budgets

| Tool | Budget | Measure |
| --- | --- | --- |
| `cargo test --workspace --features demo` | no new flake; wall-clock not worse than 2× current local run | same command |
| IR validate of a 1k-node assessment | < 50 ms debug | unit bench or timed test later; Phase 6 records a one-shot `Instant` in IR-017/018 tests |
| `canonical_digest` of fixture `control.json` | < 5 ms | IR-015 |
| Mechanical `lib.rs` split | zero JSON key changes | `git diff` on goldens empty in Phase 1 |

## Architecture debt (AD-*)

See [`matrices/m-40-architecture-debt.md`](matrices/m-40-architecture-debt.md). Summary:

| ID | Title | Owner | Removal | Status |
| --- | --- | --- | --- | --- |
| AD-001 | `Assessment` owned by framework crate | framework → IR | Phase 5 | Open |
| AD-002 | Facade `AssessmentScope` is collector-shaped | facade → IR | Phase 5 | Open |
| AD-003 | `canonical_digest` over raw `Serialize` | IR | Phase 1 / 6 | Open |
| AD-004 | `typed_id!` accepts empty strings | IR | Phase 1 | Open |
| AD-005 | `resolve_applicability` is identity | framework + IR | Phase 2 + later catalog compile | Open |
| AD-006 | Entity IDs without records | IR | Phase 3 | Open |
| AD-007 | `ComplianceGraph` is requirement-only | IR | Phase 4 | Open |
| AD-008 | No golden IR fixtures | IR | Phase 6 | Open |
| AD-009 | Control-test predicate AST absent | control-test | later program | Accepted |
| AD-010 | Framework catalogs empty stubs | framework | Phases 9–14 of spine | Accepted |

## Reserved extension points

- `ExtensionMap` keys: `wa.*`, `iso27001.*`, `gdpr.*`, `soc2.*`, `nis2.*`, `dora.*`, `user.*`
- Canonical semantics must never depend solely on an extension
- Unknown extensions survive round-trips (IR-014)
- Extensions must not override canonical fields
- Provider-specific selectors live in collectors, not IR

## Architecture Conformance Suite (ACT-*)

Existing (must stay green): ACT-001…015, COL-001…006 in `tests/sdd/assurance_runtime.target.rs`.

New (Phase 6, `tests/sdd/compliance_ir.target.rs`):

| ID | Assertion |
| --- | --- |
| IR-001 | stable IDs reject empty values |
| IR-002 | title changes do not mutate identity |
| IR-003 | controls contain no framework-specific fields |
| IR-004 | requirements preserve framework identity |
| IR-005 | mappings reject dangling nodes |
| IR-006 | partial mapping never becomes equivalence |
| IR-007 | generated mapping does not gain curated authority |
| IR-008 | control implementation ≠ control definition |
| IR-009 | implementation status ≠ effectiveness |
| IR-010 | applicability round-trips deterministically |
| IR-011 | subject selectors are provider-neutral |
| IR-012 | evidence requirements contain no collector identity |
| IR-013 | control tests contain no provider identity |
| IR-014 | unknown extensions survive round-trip |
| IR-015 | canonical digest is deterministic |
| IR-016 | digest domain separation works |
| IR-017 | duplicate canonical IDs fail validation |
| IR-018 | assessment scope is deterministic |
| IR-019 | risk references must resolve |
| IR-020 | exception references must resolve |
| IR-021 | schema version mismatch fails closed |
| IR-022 | no random UUID identities appear in persisted IR |
| IR-023 | mapping version ranges are respected |
| IR-024 | requirement external IDs are not used as internal identity |
| IR-025 | framework-specific catalogs compile without extending `Control` |

## Canonical contract analysis

| Checklist | Today (`8c0f36e`) | After Phase 6 |
| --- | --- | --- |
| Single handwritten SSOT | Partial — entities in IR, `Assessment` in framework | Yes |
| Documented | `docs/contracts/assurance-runtime.md` thin | Updated contract |
| Tested | ACT-006 thin | IR-001…025 + goldens |
| Doctor-enforced | ACT-001…015 | + IR suite |
| Migration target | no N/N-1 policy | this pack |
| Examples use it | facade stub assessment | same stub via `AssessmentDefinition` |

Net reduction of concepts: **one** `AssessmentDefinition`, **one** semantic `AssessmentScope`, **one** `FrameworkRef`, **one** digest API, **one** graph node enum. Delete framework-owned `Assessment` fields that duplicate IR after the alias window.

## Public symbol budget

Add only types in the runtime-surface table. Do not add:

- `IsoAnnexControl`, `GdprArticle`, `Soc2Criterion`
- `GithubRepositorySelector` and siblings
- `getAssessment` / `getAssessmentDetailed` / `getAssessmentOrNull` families
- a second `equivalent_lenient`

Prefer extend (`MappingRelation` instead of a parallel graph type) over add.

## Error / outcome taxonomy

| Type | Layer | Meaning |
| --- | --- | --- |
| `IdError::{Empty, TooLong, InvalidCharacter, InvalidNamespace}` | IR construct | identity rejected |
| `IrValidationError` | IR validate | structural IR rejected |
| `CanonicalDigestError` | IR digest | serialize failed |
| `FrameworkCompileError::CapabilityViolation` | compile | requested unsupported capability |
| `FrameworkCompileError::Schema` / `Identity` / `MappingIntegrity` | compile | should become rare once `ValidateIr` runs first |
| `CollectorError::OutOfScope` | collect | not an IR error |
| `Effectiveness::{effective, ineffective, insufficientEvidence, inconclusive}` | evaluate | **not** IR; never stored on `Control` |

Infrastructure failure (digest serialize) must not be reported as “not implemented” or “ineffective.”

## Runtime ownership map

Request/assess path after Phase 5:

1. Caller builds `AssessmentDefinition` + `AssessmentScope` (IR).
2. Caller builds `FrameworkTarget` (framework).
3. `validate_assessment_ir` (IR) then `compile_framework` (framework).
4. Collectors translate `AssessmentScope` → `CollectorScope` (facade).
5. `evaluate` consumes `EvidenceSet` (control-test).
6. `AssessmentReport` is a runtime result (facade), not an IR document.

## Mutation and invalidation

- Sealed evidence stays immutable (existing envelope rule).
- IR documents are values; mutation is a new document + new digest.
- Changing a `Control` title does not invalidate `ControlId`.
- Changing mapping provenance or relation **does** change mapping digest.
- Cache key for compiled assessments: `canonical_digest(AssessmentDefinition) + digest(FrameworkTarget)`.

## Findings map

| Catalog | Sev | Title | Evidence at `8c0f36e` | Close in |
| --- | --- | --- | --- | --- |
| **OWN-001** | P0 | Dual core writes for assessment input | `Assessment` in framework L101–111; facade `AssessmentScope` L37–59 | Phase 5 |
| **DAT-002** | P1 | Unknown / empty identity accepted | `typed_id!` `new` L22–24 | Phase 1 |
| **DAT-013** | P1 | Unvalidated constructor APIs | same; no `try_new` | Phase 1 |
| **CON-006** | P1 | Digest/schema not independently versioned | `canonical_digest` L289–295; single `ASSURANCE_IR_SCHEMA` | Phase 1 + 6 |
| **DAT-022** | P2 | Advertised applicability stage inert | `resolve_applicability` L265–271 | Phase 2 (IR) + later compile |
| **DAT-005** | P1 | Orphan / dangling references unvalidated | no `ValidateIr` | Phase 6 |
| **CON-003** | P1 | Facade gap: scope not IR | facade-only `AssessmentScope` | Phase 5 |
| **CI-005** | P2 | No 1.0 freeze / compatibility policy | contract has schema string only | Phase 0 + 6 |

Full register: [`findings/finding-register.md`](findings/finding-register.md).

## Evidence → findings → phases → verification → acceptance → DoD

Phases below. Verification commands are the same bar unless a phase lists extras:

```powershell
cargo test --workspace --features demo
```

---

## Phase 0 — Freeze current IR and CI truth

### Objectives

Pin HEAD, record current JSON/API, seed AD-* / findings, prove ACT-001…015 still green **before** any IR deepen.

### In scope / out of scope

In: inventory, pins, this pack. Out: product code.

### Work

- Record SHA `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7`.
- Inventory public IR types in `lib.rs` + `crosswalk.rs`.
- Confirm CI truth command: `cargo test --workspace --features demo`.
- Seed [`findings/finding-register.json`](findings/finding-register.json) and AD-*.

### Catalog IDs

CI-005 (policy missing), CI-004 (do not leave contradictory “IR complete” claims).

### Verification

| Kind | Command / proof |
| --- | --- |
| Inventory | files and line refs in this document |
| Tests | `cargo test --workspace --features demo` at pinned SHA |

### Acceptance

- [x] SHA pinned in this pack
- [x] Current-state matrix recorded in ASSESS
- [x] Findings mapped to `catalogs/05`
- [x] AD-* seeded
- [ ] Live `cargo test --workspace --features demo` captured in implementer log (authoring session did not re-run the full suite)

### Exit evidence

This pack under `docs/sdd/xylex/weeping-angel-assurance-ir/`.

### Residual risks (phase-local)

Workspace tests not re-executed in the authoring session — label **UNVERIFIED** until implement mode runs them.

### Architecture debt delta

All AD-001…010 Open/Accepted as of this freeze.

### Invariants / ACT after freeze

INV-1…5 remain enforced. INV-IR-* not yet enforced.

---

## Phase 1 — Structure and identity (slice A)

### Objectives

Split `lib.rs` with **zero JSON/API changes**, then introduce `try_new` / `FrameworkRef` / digest abstraction without changing serialized field names of existing documents except where `frameworkId`+`frameworkVersion` become nested `framework` (that nesting is **deferred to Phase 2** if it breaks JSON). Phase 1 default: keep serialized `frameworkId` / `frameworkVersion` keys until goldens exist.

### In scope / out of scope

In: module split, `StableId`, `IdError`, `try_new`, `new_unchecked`, `FrameworkRef` type (may exist unused in constructors), `CanonicalSerialize` beside old `canonical_digest`. Out: Requirement/Control field growth.

### Work

- Split to `id.rs`, `framework.rs`, `requirement.rs`, `control.rs`, `mapping.rs`, `evidence.rs`, `test.rs`, `crosswalk.rs`, `digest.rs`, `lib.rs` re-exports.
- ID grammar (recommended, not forced identical per entity): `namespace.segment[-segment][:version][:external-id]`.
- Bounded length; normalized case rules documented in `id.rs`.
- Keep `canonical_digest` as a wrapper until Phase 6 goldens switch.

### Catalog IDs

DAT-002, DAT-013, CON-006 (digest abstraction starts).

### Verification

| Kind | Command |
| --- | --- |
| Unit | `cargo test --workspace --features demo` green **after split, before ID reject** and **after ID reject** |
| Compat | no new fields on existing structs in this phase |

### Dual-suite

| Suite | On current | On desired |
| --- | --- | --- |
| Baseline ACT-001…015 | PASS | PASS |
| IR-001 (empty ID) | FAIL | PASS |

### Acceptance

- [ ] Mechanical split merges with green workspace tests
- [ ] `try_new("")` errors; persisted non-empty IDs still construct
- [ ] No `Github` / `Aws` / `Cloudflare` identifiers in IR sources (ACT-006)

### Exit evidence

PR on IR crate files only + test log.

### Residual risks

`Requirement` constructor still takes loose `FrameworkId` + `FrameworkVersion` until Phase 2.

### Architecture debt delta

AD-004 Removed. AD-003 in progress.

---

## Phase 2 — Core semantic entities (slice B)

### Objectives

Expand `Requirement`, `Control`, `ControlDomain`, `SubjectSelector`, `ApplicabilityRule` to the target shapes. Keep `Control` framework-neutral.

### Work

Files: `requirement.rs`, `control.rs`, `applicability.rs`, `subject.rs`, `extension.rs`.

`RequirementKind` is generic (`Requirement`, `ControlObjective`, `Control`, `Clause`, `Article`, `Principle`, `Procedure`, `AuditRequirement`, `Guidance`). No `IsoAnnexControl`.

`ControlDomain` is semi-open (`Other(String)` or newtype). No organizational `owner` / `implemented` on `Control`.

### Catalog IDs

DAT-022 (applicability IR exists so compile can later resolve).

### Verification

Round-trip serde of expanded structs; ACT-006 still denies ISO fields on `Control`.

### Acceptance

- [ ] Target `Requirement` / `Control` fields present
- [ ] Applicability is declarative only (no GitHub/AWS fact eval)
- [ ] Subject selectors have no provider types
- [ ] ACT-001…015 green

### Residual risks

Compiler still clones all requirements until a later compile slice consumes the rules.

---

## Phase 3 — Implementation state (slice C)

### Objectives

Add `ControlImplementation`, `PrincipalRef`, `Asset`, `Identity`, `Vendor`, `Risk`, `Exception`, `ProcessingActivity` as **minimal** records.

### Work

`implementation.rs`, `asset.rs`, `identity.rs`, `vendor.rs`, `risk.rs`, `exception.rs`, `privacy.rs`.

Do not add lawful basis / retention / DPIA / DSR. Do not add a risk engine.

### Catalog IDs

AD-006 close (DAT-014 incomplete registry analogue).

### Acceptance

- [ ] Implementation status enum is not `Effectiveness`
- [ ] Risks/exceptions attach to implementations, not `Control`
- [ ] Processing activity has `id`, `name`, `systems`, `processors` only
- [ ] ACT-001…015 green

---

## Phase 4 — Relationship model (slice D)

### Objectives

Richer `Mapping` (id, relation, confidence, provenance, version constraint, generic endpoints) and generic `ComplianceGraph`.

### Work

`mapping.rs`, `crosswalk.rs`. Keep explicit-equivalence rule. No transitive upgrade.

### Catalog IDs

Closes the thin-mapping gap (CON-010 analogue). IR-006/007.

### Acceptance

- [ ] `MappingRelation::{Equivalent, Satisfies, PartiallySatisfies, Supports, EvidenceFor, SupersetOf, SubsetOf, Related}`
- [ ] Generated provenance ≠ curated authority
- [ ] ACT-005 still green
- [ ] Graph nodes include Requirement, Control, Test, EvidenceRequirement, Risk, Exception

---

## Phase 5 — Assurance-definition model (slice E)

### Objectives

Move framework-neutral `Assessment` to IR as `AssessmentDefinition`. Move semantic `AssessmentScope` to IR. Expand `EvidenceRequirement` and `PlannedControlTest`. Framework keeps compile types only.

### Work

- Add `assessment.rs`.
- Change `weeping-angel-framework` to consume IR `AssessmentDefinition` (alias `Assessment` for N-1 one slice).
- Facade translates IR scope → `CollectorScope`.
- Evidence requirement: subject, cardinality, freshness, collection kind, criticality.
- Planned test: subjects, evidence requirement IDs, `TestEvaluationRef` hook — **no** predicate AST.
- Update `docs/contracts/assurance-runtime.md` and ADR consequences (not INV-1…5).

### Catalog IDs

OWN-001, CON-003.

### Verification

`cargo test --workspace --features demo`. Facade `assess` still returns `AssessmentReport`.

### Acceptance

- [ ] `Assessment` document type lives in IR
- [ ] Framework crate has no second field set
- [ ] Tests reference evidence requirements, not providers
- [ ] Runtime results remain outside IR

### Architecture debt delta

AD-001 Removed. AD-002 Removed.

---

## Phase 6 — Validation and compatibility freeze (slice F)

### Objectives

`ValidateIr`, goldens, IR-001…025, contract + ADR update, compatibility policy live.

### Work

- `validation.rs` + `validate_assessment_ir`
- `tests/fixtures/assurance-ir/v1/{control,requirement,mapping,control-implementation,assessment,risk,exception,processing-activity}.json`
- `tests/sdd/compliance_ir.target.rs`
- Domain-separated digest default
- Update [`docs/contracts/assurance-runtime.md`](../../../contracts/assurance-runtime.md)
- Update ADR 0001 consequences (IR now semantic authority; Assessment moved)

### Catalog IDs

DAT-005, CON-006, CI-005.

### Verification

| Kind | Command |
| --- | --- |
| Workspace | `cargo test --workspace --features demo` |
| IR suite | `cargo test --test sdd_assurance_runtime_target --test sdd_compliance_ir_target --features demo` (name as listed in root `Cargo.toml`) |
| Goldens | fixture tests fail on accidental key rename |

### Dual-suite

| Suite | On current HEAD | After Phase 6 |
| --- | --- | --- |
| `sdd_assurance_runtime_target` | PASS | PASS |
| `sdd_compliance_ir.target` | absent / RED | GREEN |

### Acceptance

- [ ] All 15 DoD bullets in §Definition of done
- [ ] ACT-001…015 + COL-001…006 green
- [ ] IR-001…025 green
- [ ] Surfaces closed; AD-001…008 Removed or Accepted with removal phase

### Residual risks

AD-009, AD-010 remain Accepted (later programs).

### Invariants / ACT after freeze

INV-IR-A…L fail-closed. Doctor = the two target suites.

---

## Definition of done

A Compliance IR freeze is allowed only when all are true:

1. A canonical `Control` can represent ISO, GDPR, SOC2, NIS2, and DORA concepts without framework-specific fields.
2. `Requirement` remains framework-specific; `Control` remains framework-neutral.
3. Control definition is separated from organizational implementation state.
4. Applicability can be represented declaratively.
5. Assets / identities / vendors / processing activities can be referenced canonically.
6. Risks and exceptions attach to implementations without changing `Control`.
7. Tests reference evidence requirements rather than providers.
8. Mapping relationships carry explicit semantics, provenance, and versions.
9. `ComplianceGraph` cannot infer unjustified equivalence.
10. All persisted IDs are stable and validated.
11. `AssessmentDefinition` belongs to the IR layer.
12. All serialized IR has golden compatibility fixtures.
13. Canonical hashes have explicit version/domain semantics.
14. All IR references can be structurally validated before compilation.
15. Existing ACT-001…015 and COL-001…006 stay green.

Remaining 10–15% after this freeze is intentionally deferred to lessons from real ISO 27001 and GDPR catalogs.

## Production gates

| Gate | Command | Status at pack write |
| --- | --- | --- |
| Spine ACT | `cargo test --workspace --features demo` | **UNVERIFIED** this session; last documented GREEN on spine SDD |
| IR ACT | `tests/sdd/compliance_ir.target.rs` | Open (not written) |
| Goldens | `tests/fixtures/assurance-ir/v1/` | Open |
| Contract | `docs/contracts/assurance-runtime.md` matches goldens | Open |
| Dual-core closed | `AssessmentDefinition` only in IR | Open (OWN-001) |

Do not call the IR 85–90% until those gates are green with pasteable logs.

## Plan revision — R11

This is a public-contract consolidation **inside** the assurance workspace: one IR domain model, one assessment document, one digest, one graph. Overlapping result families (`Assessment` in framework + `AssessmentScope` in facade + future IR twins) are forbidden after Phase 5.

## Final-plan review (R11 + R13)

| # | Question | Answer |
| --- | --- | --- |
| 1 | Why section present? | Yes |
| 2 | Invariants boolean + doctor-enforced after named phase? | Yes — after Phase 6 |
| 3 | Dependency graph with edges not shown are forbidden? | Yes |
| 4 | Runtime export allowlist complete? | Yes — module table |
| 5 | Generated allowlist complete; models as directory? | Yes — fixtures dir; no generator |
| 6 | Runtime guarantees per public file? | Yes |
| 7 | Canonical + generated formal definitions? | Yes |
| 8 | Package must-never boundaries? | Yes |
| 9 | N / N-1 compatibility policy? | Yes |
| 10 | Performance budgets + measurement commands? | Yes |
| 11 | AD-* debt register with removal targets? | Yes |
| 12 | Reserved extension points? | Yes |
| 13 | ACT-* suite mapped to CI/doctor? | Yes — existing + IR-001…025 |
| 14 | Plan is Level 3 spec, not Level 2 only? | Yes |
| R11 | Net reduction + symbol budget + error taxonomy + ownership + mutation? | Yes |
| R11 | Final-plan review present? | This table |

## Handoff brief

See [`handoff-brief.md`](handoff-brief.md).

# Handoff brief

> **Repo / SHA:** `floris-xlx/weeping-angel` `main` `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7`  
> **Scores / P0:** Architecture boundaries strong; IR semantics thin. P0 = **OWN-001** (`Assessment` dual-owned). Do not self-score 9–10.  
> **Ownership:** IR owns definition writes; framework owns compile; facade owns orchestration; scanner owns security documents.  
> **Must-nots:** no provider types in IR; no framework fields on `Control`; no findings as results; no inferred equivalence; no catalogs/collectors/evaluators in this program.  
> **First order:** Phase 0 live test log, then Phase 1 mechanical split (zero JSON change) → ID validation RED/GREEN (IR-001).  
> **Verify:** `cargo test --workspace --features demo`  
> **DoD (next ship):** Phase 1 merge with workspace green + `try_new` fail-closed.
