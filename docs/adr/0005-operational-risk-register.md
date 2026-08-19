# ADR 0005 — Operational risk register in assurance IR

<!-- weeping-angel-adr-meta
id = "0005"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_risk_register_target` GREEN (RR-001–RR-015); found-case baseline skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The operational reading “`Risk` is a four-field inventory stub, not a GRC engine” from the governance catalog found-case (`risk_ir_is_a_minimal_record_not_a_grc_engine`). Does **not** supercede IR schema `assurance-ir/v1`, canonical digest `canon/v1`, ADR 0001 spine, or SDLC catalog methodology ownership (`score_risk`). |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/risk-register.md`](../specs/risk-register.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_risk_register_baseline` skip-superseded; `sdd_risk_register_target` GREEN (`tests/contracts/risk_register.{baseline,target}.rs`) |

> Filename **`0005-*`**. Operational ISMS siblings also use `0005-*` (methodology, scheduler, continuity). Cite **this file by path**.

## Context

On SHA `6e31bf1a…`, `weeping-angel-assurance-ir::Risk` was:

```text
id, title, description, status ∈ {Open, Accepted, Mitigated, Closed}
```

Module docs said *“Not a risk engine.”* `AssessmentDefinition.validate` used the risk vec only as an id bag for `ControlImplementation.risk_ids` (IR-019). Golden `tests/fixtures/assurance-ir/v1/risk.json` was that stub. Scanner `Finding` lives in `src/finding.rs` and is not IR.

Operational ISMS v1 needs a real register: scenario, threat, weakness refs, affected assets/services/processes, CIA inputs, likelihood/impact, inherent score/rating, residual placeholder, owner, source, review, status machine, treatment **reference**, control refs, evidence lineage, tags, history/supersession — without auto-generating risks, without a treatment workflow, and without control-derived residual math.

Prompt 05 methodology (`score_risk`) landed in parallel. catalog infrastructure–02 (`IsmsContext`, `ScopeResolution`) stay neighbor types.

Questions this decision answers:

1. Does the operational register live in existing IR `Risk`, a new crate, or a GRC sidecar?
2. How do we expand the record without breaking `Risk::new`, camelCase JSON, and the golden fixture?
3. What is the status machine, and is `Open` still the default?
4. How is history preserved (versions vs events vs new ids)?
5. How do findings relate to risks?
6. Who owns scoring, treatment, and residual calculation?
7. What fails closed when references dangle?

## Decision

This is what shipped. Field-level law is [`docs/specs/risk-register.md`](../specs/risk-register.md).

### 1. Same IR type, additive fields, same schema version

The register **is** `weeping-angel-assurance-ir::Risk` in [`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs). No parallel GRC schema or crate. `ASSURANCE_IR_SCHEMA` stays `assurance-ir/v1`. Additive fields use `serde(default)` and `skip_serializing_if`. `version` is a `VersionPin` that omits `1` unless the document pinned it, so `Risk::new(id, title, description)` JSON stays four keys + default status. JSON stays camelCase. Canonical digest stays serde field order + BTree maps.

Incorrect: `RiskV2`, `assurance-ir/v2` for this slice, renaming `description` to `scenario`.

### 2. Default status remains `Open`; new states are explicit

```text
Draft | Open | UnderTreatment | Accepted | Mitigated | Closed | Retired
```

`Open` is `#[default]`. `RiskStatus::can_transition` is the fail-closed table (spec §4.3). `Open → Mitigated` / `Open → Closed` / `Draft → Closed` / `Retired → *` are illegal. `Risk::transition` returns `RiskTransitionError::Illegal` and appends `StatusTransition` to `history`. Recorded illegal transitions fail `validate()`.

### 3. History is version + append-only events + optional id supersession

Stable `RiskId` + monotonic `version` (default 1) + `history: Vec<RiskEvent>` + optional `supersedes` / `supersededBy`.

```text
Risk::revise(title)      # increments version; event.previous keeps title/status/inherentScore
Risk::transition(to)     # fail-closed; appends StatusTransition
```

`Retired` is terminal for an id; replacement is a new id with `supersedes`. Empty `history` on `Risk::new` and old fixtures. `supersedes` / `supersededBy` must resolve in `assessment.risks`.

### 4. Findings contribute; they do not become risks

`FindingRef` is a typed stable id (`id.rs`). `Risk.finding_refs` is N:N. No `Finding` struct in the IR crate. No `From<Finding> for Risk`. `RiskSource` is a provenance tag (`manual` / `finding` / `incident` / `assessment` / `supplier` / `other`). infrastructure catalog owns `RiskCandidate`.

### 5. Scoring is Prompt 05; the register adapter is not a second engine

Inherent score/rating are **derived** from raw likelihood/impact + a methodology version. Prompt 05 owns scales, matrices, and `score_risk`. The register stores `MethodologyValue` snapshots (`levelId` / `cellId` / `methodologyId` / `revision` / `ratingId`) and exposes:

```text
score_inherent(methodology_version, likelihood, impact, cia?)
  → Result<(MethodologyValue, MethodologyValue), RiskScoringError>
```

in [`risk_scoring.rs`](../../crates/weeping-angel-assurance-ir/src/risk_scoring.rs). Cell identity is the authored level pair (`{likelihoodId}-{impactId}`). **No hardcoded 5×5** and no crate-wide `enum RiskRating`. Clockless validate requires a version pin and raw level ids when inherent fields are present; it rejects rating-only authoring. It does not invent ratings when the methodology document is absent.

Residual `residualScore` / `residualRating` on the row are **placeholders**. Control-derived residual is owned by [ADR 0003 residual risk](0003-residual-risk.md) (`ResidualRiskProjection` in assurance-IR + `weeping-angel-assurance::residual`). Collectors must not author ratings as compliance evidence.

`PrincipalRef` from `implementation.rs` is the owner type.

### 6. Graph integrity is fail-closed on the existing inventories

`validate_assessment_ir` rejects duplicate `RiskId`, dangling `AssetId` / `ProcessingActivityId` / `VendorId` / `ControlId` / identity `owner` / `EvidenceRequirementId` / supersession ids, malformed evidence digests, and illegal recorded transitions. IR-019 remains. `Some(treatmentId)` is resolved by `validate_treatment_inventory` (risk treatment owns the inventory). Overdue `nextReview` is `validate_risk_reviews_at(assessment, as_of)` only; terminal `Closed` / `Retired` are spared; clockless `validate()` stays valid for old assessments. Unscheduled (`nextReview` absent) is not overdue.

Crate-root `ReviewCadence` is `{ intervalSeconds }`. Implementation review uses `implementation::ReviewCadence` (`intervalDays`) and is not re-exported at crate root.

## Non-goals

- Auto-identification (`RiskCandidate`), treatment state machine, residual-risk projection engine ([ADR 0003 residual risk](0003-residual-risk.md)).
- Rewriting catalog TOML, ISO packs, collectors, or existing dual-suites except additive registration.
- UI, persistence, ticketing.
- Bumping `assurance-ir/v1` or auto-advancing `nextReview` from cadence.

## Consequences

- `Risk` is the operational register SSOT; governance catalog’s “minimal record” characterization is historical (already ignored) and restated as `sdd_risk_register_baseline` found-case (now skip-superseded).
- Treatment (Prompt 08) attaches plans to `RiskTreatmentId` without renaming this record.
- Methodology can replace `score_inherent` without changing risk graph fields.
- Applicability continues to clone `Vec<Risk>` by id; new fields are ignored there.
- `sdd_compliance_ir_target` and `sdd_assurance_runtime_target` stay green.

## Related

- Spec: [`docs/specs/risk-register.md`](../specs/risk-register.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Risk methodology: [`docs/specs/risk-methodology.md`](../specs/risk-methodology.md), [ADR 0005 risk methodology](0005-risk-methodology.md)
- Residual (projection, not placeholders): [ADR 0003 residual risk](0003-residual-risk.md)
- Treatment (`RiskTreatmentId` inventory): [ADR 0006 risk treatment](0006-risk-treatment-engine.md)
- Identification (candidates, not register rows): [ADR 0007 risk identification](0007-risk-identification-candidate-correlation.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
