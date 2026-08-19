# SDD: Operational Risk Register (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_risk_register_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — vulnerability catalog |
| Slice | Expand IR `Risk` from a four-field inventory stub into an operational information-security risk record; keep `assurance-ir/v1` and existing callers |
| Dual-suite | `sdd_risk_register_baseline` (skip-superseded) · `sdd_risk_register_target` GREEN (`tests/contracts/risk_register.{baseline,target}.rs`) — **not auto-discovered**; listed in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0040-operational-risk-register.md`](../adr/0040-operational-risk-register.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (landed Risk register section; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Consumes | ISMS context IR; scope engine; [`docs/specs/risk-methodology.md`](risk-methodology.md) (`score_risk`; register adapter `score_inherent`) |
| Neighbors (do not implement here) | infrastructure catalog risk identification; governance catalog treatment workflow; residual-risk projection ([`residual-risk.md`](residual-risk.md)) |
| Collision fence | Catalog TOML, ISO packs, GitHub collector, existing `sdd_*` suites except additive `Cargo.toml` / `documentation_layout.rs` registration |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; keep `sdd_compliance_ir_target` and `sdd_assurance_runtime_target` GREEN |

This document is the durable human SSOT for the vulnerability catalog. It owns the **canonical operational `Risk` record**, **status transition validation**, **reference integrity against the existing assets/controls/evidence graph**, **finding contributors (N:N, no auto-promotion)**, **review overdue semantics**, and **history/supersession**. It does **not** own risk methodology scales/matrices (SDLC catalog), `RiskCandidate` generation (infrastructure catalog), treatment plan state machines (governance catalog), or control-effectiveness residual calculation ([`residual-risk.md`](residual-risk.md)).

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A risk is a **management-system record** over that graph. A scanner finding is **not** a risk. A collector must not emit `RiskRating` as compliance evidence.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

### Landed surface

| Item | Home |
| --- | --- |
| `Risk`, `RiskStatus`, `RiskSource`, `RiskEvent`, `CiaImpactInputs`, `ReviewCadence`, `transition` / `revise` / `review_overdue` | [`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs) |
| `score_inherent`, `MethodologyValue` | [`crates/weeping-angel-assurance-ir/src/risk_scoring.rs`](../../crates/weeping-angel-assurance-ir/src/risk_scoring.rs) |
| `FindingRef`, `RiskTreatmentId` | `id.rs` `typed_id!` |
| Graph + overdue reviews | `validation.rs` (`validate_risk_reviews_at`; per-risk refs from `validate_assessment_ir`) |
| `Some(treatmentId)` resolution | `risk_treatment::validate_treatment_inventory` (invoked from IR validate) |
| Re-exports | `weeping-angel-assurance-ir` `lib.rs` |
| Golden (still four keys) | `tests/fixtures/assurance-ir/v1/risk.json` |

`score_risk` remains methodology SSOT ([`risk-methodology.md`](risk-methodology.md)). `score_inherent` is the register adapter over opaque `MethodologyValue` snapshots. Residual placeholders on the row are not `ResidualRiskProjection`.

---

## 0. Collision fence (concurrent SDD)

SDLC catalog methodology (`score_risk`, scales, matrices) is **landed** — consume it; do not fork a second matrix in `risk.rs`. catalog infrastructure (`IsmsContext`) and typed evidence (`ScopeResolution`) remain neighbor types: consume them if present; do not re-implement them here.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| SDLC catalog methodology modules, scales, matrices, `score_risk`, methodology version types | SDLC catalog — **consume, do not fork** |
| catalog infrastructure `IsmsContext` / typed evidence `ScopeResolution` modules if they land | catalog infrastructure–02 |
| `tests/contracts/{compliance_ir,assurance_runtime,applicability_engine,governance_catalog}.*` rewrite | Existing suites — stay GREEN; do not convert their characterizations |
| `src/finding.rs` scanner `Finding` | Recon/scanner product; not IR |

Landed adjustments: expanded [`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs); sibling [`risk_scoring.rs`](../../crates/weeping-angel-assurance-ir/src/risk_scoring.rs); `FindingRef` / `RiskTreatmentId`; `validation.rs` graph + `validate_risk_reviews_at`; `lib.rs` re-exports; serde defaults on additive fields; `PrincipalRef` reuse from [`implementation.rs`](../../crates/weeping-angel-assurance-ir/src/implementation.rs). The scoring adapter stores register JSON and does not copy Prompt 05 matrices.

Do **not** redesign `AssessmentDefinition` core inventories, catalog schema, collectors, or ISO pack IDs. `AssessmentDefinition.risks` remains `Vec<Risk>`.

---

## 1. Problem / user-visible goal

Weeping Angel stores risks as a **minimal inventory stub** so `ControlImplementation.risk_ids` can resolve. The module is explicitly documented *“Minimal risk record. Not a risk engine.”* Operators cannot record an information-security scenario, name the threat and weaknesses, bind affected assets/processes, score inherent risk under a versioned methodology, assign an owner, schedule review, or keep history when the record is edited.

Today that means:

- a golden fixture is three strings plus `status: "open"`;
- `RiskStatus` has no draft, treatment, or retired states and no transition table;
- dangling `AssetId` / `ControlId` / treatment ids on a risk are not validated (only implementation→risk ids are);
- `Risk::new` JSON has no `owner`, `treatment`, or `residualScore` (asserted absent by the superseded governance baseline);
- scanner `Finding` in `src/finding.rs` is a recon artifact, not an IR type — and nothing in IR even references findings as contributors;
- there is no methodology version on the record, so any future rating would be an ad-hoc field.

**User-visible goal:** given an `AssessmentDefinition` that already holds assets, controls, evidence requirements, and implementations, an organization can construct, serialize, validate, and revise an operational risk such that:

```text
title + scenario + threat
  → affected assets / services / processes
  → raw likelihood/impact inputs + methodology version
  → derived inherent score/rating (SDLC catalog)
  → residual score/rating placeholder only
  → owner, source, review cadence, next review, status
  → treatment ref (id only) + canonical control refs + evidence lineage
  → finding refs as N:N contributors (never auto-promotion)
  → version/history so edits do not destroy prior state
```

Example the register must distinguish:

```text
Finding "unprotected-branch" exists
  → not a Risk

Two findings contribute to one Risk (same scenario)
  → one RiskId, two FindingRef entries

One finding contributes to two Risk candidates/records
  → same FindingRef on two RiskIds

nextReview < as_of and status is Open
  → overdue (fail-closed query / validation-at-time)

Risk.control_ids contains an id absent from assessment.controls
  → validate() error (fail closed)

Edit title after scoring
  → previous version retained in history; version increments
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Risk` / `RiskStatus` | `weeping-angel-assurance-ir::risk` | **SSOT record.** Expand in place. Keep `Risk::new(id, title, description)` and public `id` / `title` / `description` / `status`. Keep `#[serde(rename_all = "camelCase")]`. |
| `RiskId` | `id.rs` `typed_id!(RiskId)` | Unchanged. Optional new typed ids: `RiskTreatmentId`, `FindingRef` (reference newtype, **not** a `Finding` struct). |
| `AssessmentDefinition.risks` | `assessment.rs` | Stay `Vec<Risk>`. Do not introduce a parallel register crate. |
| `ValidateIr` | `validation.rs` | Today checks implementation→`RiskId` only (IR-019). **Add** per-risk graph checks. Keep IR-019. |
| `ControlImplementation.risk_ids` | `implementation.rs` | Unchanged direction (implementation cites risks). Risks may also cite `ControlId`s. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for `owner`. Do not invent `RiskOwner`. |
| `Asset` / `AssetId` / `AssetKind::Service` | `asset.rs` | Affected assets and services. Services are assets, not a new type. |
| `ProcessingActivity` / `ProcessingActivityId` | `privacy.rs` | Affected processes. |
| `Vendor` / `VendorId` | `vendor.rs` | Optional affected-supplier refs (same inventory). Reverse edge is `Vendor.risk_ids` ([`supplier-risk.md`](supplier-risk.md)); dangling ids fail closed; reverse listing is not required. |
| `Control` / `ControlId` | `control.rs` | Canonical control references on the risk. |
| `EvidenceRequirement` / `EvidenceRequirementId` | `evidence.rs` | Evidence-requirement lineage. Envelope/observation digests are opaque strings, not a second evidence graph. |
| Scanner `Finding` | `src/finding.rs` | **Not IR.** Do not move it into `weeping-angel-assurance-ir`. |
| Applicability | `weeping-angel-assurance::applicability` | Clones `definition.risks`; `risk_matches` uses `id` + `SubjectKind::Asset`. Keep `Risk: Clone + Serialize`. Do not require applicability to understand new fields. |
| Golden fixture | `tests/fixtures/assurance-ir/v1/risk.json` | Must keep **decoding**. Additive fields use `serde(default)` and `skip_serializing_if` so `Risk::new` JSON stays minimal. |
| `sdd_compliance_ir_target` | `ir_golden_fixtures_round_trip`, `ir_019_risk_references_must_resolve` | Must stay GREEN. |
| Governance baseline | `risk_ir_is_a_minimal_record_not_a_grc_engine` | Already `#[ignore = "superseded by sdd_governance_catalog_target"]`. Do not revive it as a product invariant. After this slice lands, `sdd_risk_register_baseline` holds the *found* characterization and is skip-superseded in the usual way. |
| SDLC catalog scoring | `risk_methodology.rs` `score_risk` (landed) | **Consume.** Separate raw inputs from derived rating. **Do not hardcode 5×5.** Register JSON uses `MethodologyValue` + `score_inherent`; do not copy scales/matrices into `risk.rs`. |
| catalog infrastructure / 02 | spec-only at characterization | Risks may later hang off `IsmsContext` / `ScopeResolution`. This slice does not require those types. Affected-subject ids still validate against **this** assessment’s inventories. |
| Canonical digest | `digest.rs` | Unchanged algorithm. Field order of the expanded struct is the new canonical bytes for a fully populated risk; empty optional fields omit via `skip_serializing_if` so old `Risk::new` digests stay stable. |

Tiny allowed: new `typed_id!` aliases; `Risk` serde defaults; validation messages; re-exports.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Additive optional fields keep `assurance-ir/v1`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 IR `Risk` is four public fields

[`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs) module docs: *“Minimal risk record. Not a risk engine.”*

```text
RiskStatus = Open | Accepted | Mitigated | Closed
  #[default] Open
  serde rename_all = "camelCase"   →  "open" | "accepted" | "mitigated" | "closed"

Risk {
  id: RiskId,
  title: String,
  description: String,
  #[serde(default)] status: RiskStatus,
}

Risk::new(id, title, description) → status = Open
```

No `Draft`, `UnderTreatment`, or `Retired`. No transition function. No owner, scenario, threat, assets, controls, treatment, scores, review clock, evidence, findings, tags, version, or history.

### 3.2 Callers construct with `Risk::new`

| Caller | Use |
| --- | --- |
| `sdd_governance_catalog_baseline` `risk_ir_is_a_minimal_record_not_a_grc_engine` (ignored) | Constructs via `new`; asserts JSON lacks `treatment`, `owner`, `residualScore`; names `Accepted`/`Mitigated`/`Closed` |
| `sdd_applicability_engine.baseline` | `Risk::new` compiles; `AssessmentDefinition.risks` default empty |
| `ApplicabilityContext` | `definition.risks.clone()`; match by `risk.id` only |

### 3.3 Assessment validation is implementation→risk only

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs):

- Builds `risk_ids: BTreeSet` from `assessment.risks` (duplicates **silently collapse**; no duplicate-id error).
- For each `ControlImplementation`, each `risk_ids` entry must exist (`dangling risk reference` — IR-019).
- **Does not** walk `Risk` for `AssetId`, `ControlId`, treatment, evidence, or owner identity.
- `AssessmentDefinition.risks` is an unvalidated `Vec` aside from being an id bag for implementations.

### 3.4 Golden fixture

[`tests/fixtures/assurance-ir/v1/risk.json`](../../tests/fixtures/assurance-ir/v1/risk.json):

```json
{
  "id": "risk:source-tamper",
  "title": "Source tampering",
  "description": "Unauthorized change to the source of record.",
  "status": "open"
}
```

`ir_golden_fixtures_round_trip` decodes it and asserts `id == "risk:source-tamper"`. Assessment golden fixture has `"risks": []`.

### 3.5 Findings are not IR

- `weeping-angel-assurance-ir` has **no** `Finding` type.
- `src/finding.rs` `Finding` is scanner output (`severity`, `url`, `module`, `cwe`, file evidence). It is not in the assurance graph and is not referenced by `Risk`.

### 3.6 Scoring / methodology

No likelihood scale, impact scale, matrix, score, rating, or methodology version types exist in this tree at characterization (SDLC catalog is a spec file only). `Risk` has no score fields.

### 3.7 Crosswalk

`ComplianceNodeRef::Risk(RiskId)` exists. No risk→control edges are built from `Risk` fields.

---

## 4. Desired behavior (target)

### 4.1 Product home

The record stays in `weeping-angel-assurance-ir`. Landed layout:

```text
weeping-angel-assurance-ir
  risk.rs            # Risk, RiskStatus, RiskSource, ReviewCadence, CiaImpactInputs, RiskEvent, transition/revise
  risk_scoring.rs    # MethodologyValue + score_inherent (adapter; not score_risk)
  id.rs              # RiskId; RiskTreatmentId; FindingRef newtype
  validation.rs      # graph integrity + duplicate RiskId + recorded illegal transitions + validate_risk_reviews_at
  implementation.rs  # PrincipalRef (consumed)
  risk_methodology.rs  # Prompt 05 score_risk — consumed, not copied
```

Network-free. No ISO annex numbers, no provider SDK types, no GRC product vocabulary (`Jira`, `ServiceNow`, `RiskCloud`) in generic IR.

### 4.2 Operational record (additive fields)

JSON names are **camelCase**, matching existing `status`. Keep `id`, `title`, `description`, `status`. New fields **default on deserialize** and **omit when empty** on serialize so old fixtures and `Risk::new` remain valid.

| Field (Rust) | JSON | Required on `new` | Semantics |
| --- | --- | --- | --- |
| `id` | `id` | yes | Stable `RiskId` |
| `title` | `title` | yes | Short name |
| `description` | `description` | yes | Existing narrative body (compat). Not renamed to `scenario`. |
| `scenario` | `scenario` | no | Threat scenario / how the risk materializes |
| `threat` | `threat` | no | Threat actor or event description (string; not a threat-intel object) |
| `weakness_refs` | `weaknessRefs` | no | Vulnerability/weakness identifiers (CWE, CVE, catalog ids). `Vec` of stable-id strings or a thin `WeaknessRef` newtype. Not a CVE engine. |
| `asset_ids` | `assetIds` | no | Affected assets (includes `AssetKind::Service` for services) |
| `processing_activity_ids` | `processingActivityIds` | no | Affected processes |
| `vendor_ids` | `vendorIds` | no | Affected suppliers, when the scenario is supplier-bound |
| `cia` | `cia` | no | Optional CIA **raw impact inputs** (see §4.5). Omitted if methodology has no CIA dimensions. |
| `likelihood` | `likelihood` | no | **Raw** methodology input, not a rating |
| `impact` | `impact` | no | **Raw** methodology input, not a rating |
| `inherent_score` | `inherentScore` | no | **Derived** snapshot via SDLC catalog |
| `inherent_rating` | `inherentRating` | no | **Derived** snapshot via SDLC catalog |
| `residual_score` | `residualScore` | no | **Placeholder only** (authoritative projection is `ResidualRiskProjection`; [`residual-risk.md`](residual-risk.md)) |
| `residual_rating` | `residualRating` | no | **Placeholder only** |
| `methodology_version` | `methodologyVersion` | no | Version pin of the methodology used for derived inherent fields |
| `owner` | `owner` | no | `PrincipalRef` (`Identity` / `Team` / `Role`) |
| `source` | `source` | no | `RiskSource` (see §4.4) |
| `discovered_at` | `discoveredAt` | no | `DateTime<Utc>` |
| `review_cadence` | `reviewCadence` | no | Crate-root `ReviewCadence` `{ "intervalSeconds": u64 }` (not `implementation::ReviewCadence` `intervalDays`) |
| `next_review` | `nextReview` | no | `DateTime<Utc>` |
| `status` | `status` | default `open` | §4.3 |
| `treatment_id` | `treatmentId` | no | `RiskTreatmentId` — **reference**, not a treatment plan (governance catalog) |
| `control_ids` | `controlIds` | no | Canonical `ControlId`s |
| `evidence_refs` | `evidenceRefs` | no | Evidence lineage (§4.7) |
| `finding_refs` | `findingRefs` | no | Contributor ids, N:N (§4.4) |
| `tags` | `tags` | no | `BTreeSet<String>` |
| `classification` | `classification` | no | Optional classification label (not an ISO clause number) |
| `version` | `version` | default `1` | Monotonic revision of this `RiskId` |
| `supersedes` | `supersedes` | no | Prior `RiskId` this record replaces |
| `superseded_by` | `supersededBy` | no | Successor `RiskId` when this record is retired by replacement |
| `history` | `history` | no | Append-only `RiskEvent` list (§4.8) |

`Risk::new` sets `status = Open`, `version = 1`, and empty/None for every additive field. It must **not** start emitting `owner` / `treatment` / `residualScore` keys (governance found-case remains true for the constructor). Prefer JSON key `treatmentId` over `treatment` so a later governance catalog object named `treatment` does not collide; `skip_serializing_if` keeps both absent on `new`.

Public fields may remain public for serde and existing `risk.id` / `risk.status` access (`ApplicabilityContext`, tests). **State changes that are semantically transitions must go through** `transition` / `revise` (or equivalent) so history is appended. Assessment validation rejects a `history` that contains an illegal transition even if someone mutated `status` directly.

### 4.3 Status and transitions

```text
RiskStatus =
  Draft
  | Open                 // default; existing "open"
  | UnderTreatment       // JSON "underTreatment"
  | Accepted
  | Mitigated
  | Closed
  | Retired
```

Keep existing camelCase encoding so `"open"` / `"accepted"` / `"mitigated"` / `"closed"` still decode.

Fail-closed table (from → allowed to). Any other pair is an error.

| From | Allowed targets | Notes |
| --- | --- | --- |
| `Draft` | `Open`, `Retired` | Not yet an accepted ISMS risk |
| `Open` | `UnderTreatment`, `Accepted`, `Retired` | Cannot skip to `Mitigated` or `Closed` |
| `UnderTreatment` | `Open`, `Accepted`, `Mitigated`, `Retired` | Treatment workflow itself is governance catalog; this is register status only |
| `Accepted` | `Open`, `UnderTreatment`, `Closed`, `Retired` | Acceptance expiry/evidence is governance catalog |
| `Mitigated` | `Open`, `UnderTreatment`, `Closed`, `Retired` | Reopen if the scenario returns |
| `Closed` | `Open`, `Retired` | Reopen or retire |
| `Retired` | ∅ | Terminal for this `RiskId`. Replacement is a **new** record with `supersedes` |

`fn RiskStatus::can_transition(from, to) -> bool` and `Risk::transition(to) -> Result<Self, …>` (or `&mut self`) are mandatory. Invalid transitions return a deterministic error (no panic in library paths). Recorded `history` status steps must satisfy the same table.

Default `Open` (not `Draft`) preserves `Risk::new` and old fixtures.

### 4.4 Findings are contributors, not risks

```text
FindingRef          // typed stable id, not a Finding document
Risk.finding_refs   // Vec<FindingRef>, default empty
```

Invariants:

1. Deserializing or constructing a scanner `Finding` does **not** create a `Risk`.
2. There is no `From<Finding> for Risk` and no IR function that promotes a finding.
3. One risk may list many `FindingRef`s; the same `FindingRef` may appear on many risks.
4. Empty `finding_refs` is valid (manually identified risk).
5. `FindingRef` values must be well-formed stable ids (`validate_stable_id`). There is **no** finding inventory in `AssessmentDefinition` in this slice; dangling finding documents are infrastructure catalog’s problem. Do not fail validation solely because a scanner finding file is absent.
6. Do **not** add `struct Finding` to `weeping-angel-assurance-ir`.

`RiskSource` (camelCase JSON) is a provenance tag, not promotion:

```text
RiskSource = Manual | Finding | Incident | Assessment | Supplier | Other(String)
```

`source = Finding` without `finding_refs` is allowed only as incomplete draft data; target tests should prefer the pair (source + refs) for the N:N case. Source never implies `RiskStatus::Accepted` or a framework fail.

### 4.5 Scoring: consume SDLC catalog; do not invent a second model

**Raw vs derived (SDLC catalog law):** collectors and authors store likelihood/impact **inputs**. `inherentRating` is derived. Never accept a collector-emitted `RiskRating::High` as the only scoring evidence.

Prompt 05 owns `score_risk(&RiskMethodology, &RiskScoreInput)`. The register stores opaque `MethodologyValue` snapshots (`levelId` / `cellId` / `methodologyId` / `revision` / `ratingId`) and derives inherent fields through the adapter:

```text
score_inherent(methodology_version, likelihood, impact, cia?)
  → Result<(MethodologyValue /* score */, MethodologyValue /* rating */), RiskScoringError>
```

Rules:

1. Do **not** duplicate `LikelihoodScale` / `ImpactScale` / `RiskMatrix` / `RiskScore` under `risk.rs`. Call `score_risk` when a methodology document is in hand; otherwise `score_inherent` hashes the authored level pair into a `cellId` (`{likelihoodId}-{impactId}`) plus a version-pinned `ratingId`. That is **not** a hardcoded 5×5, 1–5, or Low/Medium/High matrix.
2. Same methodology version + same raw inputs ⇒ same score and rating (byte-stable for equal ordering).
3. If `inherent_score` / `inherent_rating` are present, `methodology_version` is required **and** raw `likelihood`/`impact` must carry a level id. Derived rating as the only authoring input fails closed. Clockless validate does not re-run `score_risk` against an in-scope methodology document; it refuses incomplete derived fields. Do not invent ratings when the methodology document is absent.
4. CIA fields are **optional raw impact dimensions** (`confidentiality`, `integrity`, `availability` as `u32`). They serialize when present and omit when unset. They are not a second hardcoded matrix and do not substitute for methodology ratings.
5. **Residual** `residualScore` / `residualRating` may be present as placeholders (manual/unspecified). This slice **must not** compute them from control tests, effectiveness, or treatment completion ([`residual-risk.md`](residual-risk.md)). Tests assert placeholder semantics: setting residual does not require controls to be `Effective`; omitting residual is valid.

Do not treat `ApplicabilityPredicate::RiskLevel(String)` as this register’s rating SSOT.

### 4.6 Review cadence and overdue

```text
Risk::review_overdue(as_of: DateTime<Utc>) -> bool
```

| `nextReview` | Result |
| --- | --- |
| `None` | not overdue (unscheduled ≠ overdue) |
| `Some(t)` and `t >= as_of` | not overdue |
| `Some(t)` and `t < as_of` | **overdue** |

`Closed` and `Retired` still **report** overdue if `nextReview` is in the past (data fact), but assessment `validate()` without a clock does not fail on it. Provide `validate_risk_reviews_at(assessment, as_of)` (name flexible) that fails closed for overdue risks whose status is **not** `Closed` or `Retired`. Existing `AssessmentDefinition::validate()` stays clockless so IR-021 and golden empty assessments keep working.

`reviewCadence` is documentary/schedule metadata; this slice does not auto-advance `nextReview`.

### 4.7 Reference integrity (fail closed)

On `AssessmentDefinition::validate()` (in addition to IR-019):

| Reference | Rule |
| --- | --- |
| Duplicate `Risk.id` | error (today duplicates collapse) |
| `asset_ids` | every id ∈ `assessment.assets` |
| `processing_activity_ids` | every id ∈ `assessment.processing_activities` |
| `vendor_ids` | every id ∈ `assessment.vendors` |
| `control_ids` | every id ∈ `assessment.controls` |
| `treatment_id` | if `Some`, must resolve in `AssessmentDefinition.risk_treatments` via `validate_treatment_inventory` ([`risk-treatment.md`](risk-treatment.md)). **This slice does not own treatment plans.** Empty/`None` is valid. |
| `evidence_refs` that name `EvidenceRequirementId` | id ∈ `assessment.evidence_requirements` |
| `evidence_refs` that name an envelope/observation digest | well-formed non-empty string; IR does not open the ledger |
| `owner = PrincipalRef::Identity(id)` | id ∈ `assessment.identities` |
| `owner = Team(_) \| Role(_)` | non-empty string; no team inventory in IR |
| `history` transitions | each consecutive status pair obeys §4.3 |
| `supersedes` / `superseded_by` | if set, those `RiskId`s exist in `assessment.risks` (or the pair is consistent inside the vec) |
| `version` | `>= 1`; default 1 on old JSON |

Dangling implementation→risk remains IR-019.

### 4.8 History and supersession

Edits must not destroy prior state.

**Chosen model (ADR 0005):** stable `RiskId` + monotonic `version` + append-only `history: Vec<RiskEvent>` + optional identity-level supersession (`supersedes` / `supersededBy`).

```text
RiskEvent {
  version: u32,
  at: DateTime<Utc>,           // required on recorded events; construction may omit until first revise
  principal: Option<PrincipalRef>,
  kind: Created
      | FieldsRevised
      | StatusTransition { from: RiskStatus, to: RiskStatus }
      | Superseded { successor: RiskId },
}
```

Rules:

1. `revise(...)` increments `version` and appends `FieldsRevised` (or a typed field-change event). Previous field values remain recoverable from history **or** from a prior serialized snapshot of the same id+version. Minimum bar for target tests: after revise, `history` is non-empty and the previous `title`/`status`/`inherent_score` is still represented (event payload or retained prior revision struct). Do not overwrite `history`.
2. `transition(to)` appends `StatusTransition` and fails if §4.3 forbids it.
3. Replacing identity: successor `Risk` has `supersedes = Some(old_id)`; old risk `superseded_by = Some(new_id)` and `status = Retired` via a legal transition path (`Open|… → Retired` or via `Closed → Retired`).
4. `Risk::new` may have empty `history`. First explicit revise/transition seeds it. Old fixtures with no `history` key decode as empty.
5. Do not require an event-sourcing database. In-memory IR + serde is enough.

### 4.9 Serialization and digest

- Schema stays `assurance-ir/v1`.
- `canonical_digest` remains SHA-256 of `serde_json::to_vec` (struct field order + BTree maps).
- Empty vectors / None skip serialize so `Risk::new` and `risk.json` do not grow required keys.
- Maps/sets that this slice adds use `BTreeMap`/`BTreeSet`.
- `version` default `1` on missing key; skip-serialize if implementers can do so **only when** it would change `Risk::new` bytes — prefer `skip_serializing_if = "is_one"` for version so constructor JSON stays four fields + default status.

### 4.10 Constructor and builders

Keep:

```text
Risk::new(id, title, description) -> Risk
```

Additive builders (names flexible): `with_scenario`, `with_owner`, `with_assets`, `with_controls`, `with_findings`, `with_methodology_inputs`, `with_review`, `with_treatment`, etc., each compatible with serde defaults.

Applicability and governance tests that only call `new` must keep compiling.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | After implement |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/risk_register.baseline.rs` | `sdd_risk_register_baseline` | skip-superseded (`#[ignore = "superseded by sdd_risk_register_target"]`) |
| Target | `tests/contracts/risk_register.target.rs` | `sdd_risk_register_target` | **GREEN** (RR-001–RR-015) |

Protocol (completed): spec first, baseline GREEN on characterization, target RED, implement, docs+ADR, target GREEN, baseline skip-superseded, target still GREEN.

One regression test per comment/invariant titled from the spec ids below, encoding the **original found case** in baseline.

Register the spec path in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` (done when this file exists).

---

## 6. Acceptance criteria (testable)

Target suite must encode at least:

- **RR-001** Old minimal fixture `tests/fixtures/assurance-ir/v1/risk.json` decodes; `id == "risk:source-tamper"`; `status == Open`; missing additive keys default empty.
- **RR-002** `Risk::new` still exists; default `status == Open`; serialized JSON omits `owner`, `treatment`/`treatmentId`, `residualScore` when unset.
- **RR-003** Fully populated operational risk round-trips through serde; `canonical_digest` is stable for equivalent `BTree` ordering.
- **RR-004** `RiskStatus` includes `Draft`, `Open`, `UnderTreatment`, `Accepted`, `Mitigated`, `Closed`, `Retired`; JSON `"open"` still decodes; `"underTreatment"` round-trips.
- **RR-005** Illegal transitions fail (`Open → Mitigated`, `Draft → Closed`, `Retired → Open`, `Open → Closed`). Legal transitions succeed and append history.
- **RR-006** Dangling `AssetId` on a risk fails `validate()`. Dangling `ControlId` fails. `Some(treatmentId)` with no resolving treatment record fails.
- **RR-007** IR-019 still fails on dangling `ControlImplementation.risk_ids`. Duplicate `RiskId` in `assessment.risks` fails.
- **RR-008** `review_overdue(as_of)` is true iff `nextReview < as_of`; unscheduled is not overdue; clocked validation fails for overdue non-terminal risks.
- **RR-009** Inherent score/rating is derived from raw likelihood/impact + methodology version via SDLC catalog APIs (or the replaceable adapter). Equal inputs ⇒ equal outputs. Derived rating is not accepted as the only authoring input. **No hardcoded 5×5** in this crate’s control logic.
- **RR-010** `residualScore` / `residualRating` may be set as placeholders without control-effectiveness calculation; omitting them is valid; this slice does not map `Effectiveness::Effective` to residual zero.
- **RR-011** Finding refs are N:N: one risk aggregates two refs; one ref appears on two risks. No API auto-promotes `src/finding.rs` `Finding` to `Risk`. No `Finding` struct in the IR crate.
- **RR-012** `revise` / equivalent preserves prior state in `history` (or retained revision); `version` increments; history is not cleared by a title/status edit.
- **RR-013** CIA raw inputs serialize when present and omit when unset; they do not substitute for methodology ratings.
- **RR-014** Owner uses `PrincipalRef`; `Identity` owner dangling vs `assessment.identities` fails closed.
- **RR-015** Dual-suite names `sdd_risk_register_baseline` / `sdd_risk_register_target` are listed in root `Cargo.toml`.

Baseline suite must encode the found case in §3 (minimal fields, four statuses, no transition validator, no per-risk asset/control/treatment validation, no residual/owner on `new`, scanner finding not IR).

---

## 7. Out of scope

- Auto-generating risks or `RiskCandidate` correlation (infrastructure catalog).
- Treatment plan workflow, acceptance immutability, Mitigate/Accept/Avoid/Transfer engine (governance catalog).
- Control-derived residual calculation, effectiveness→residual functions ([`residual-risk.md`](residual-risk.md)).
- Hardcoding a 5×5 (or any) matrix; replacing SDLC catalog scoring.
- Implementing `IsmsContext` or `ScopeResolution` (catalog infrastructure–02) except consuming them if already landed.
- Moving scanner `Finding` into IR or treating recon severity as ISMS rating.
- UI, persistence service, ticketing, auditor portal, GRC SaaS sync.
- Rewriting catalog TOML, ISO packs, GitHub collector mapping.
- Bumping `assurance-ir/v1`.
- Auto-advancing `nextReview` from cadence.
- Claiming ISO 27001 clause satisfaction from a risk record.

---

## 8. Risks

- SDLC catalog landing in parallel: a second scoring model in `risk.rs` would fork the methodology SSOT. Mitigation: consume 05 types; adapter only if 05 is absent; never hardcode matrices.
- Expanding public `Risk` fields can break exhaustive struct literals in-tree. Mitigation: keep `Risk::new`; search/fix literals in this slice only; serde defaults for fixtures.
- Making `status` default `Draft` would break `Risk::new` and `risk.json`. Mitigation: default remains `Open`.
- Fail-closed `treatmentId` with no governance catalog inventory means authors cannot persist a treatment key until 08 lands unless they omit it. That is intended.
- Applicability `risk_matches` treats risks as `SubjectKind::Asset`. Changing kind matching is out of scope; do not “fix” that here.
- History-in-document vs event store: large `history` vectors change digests. Tests pin explicit fixtures; do not log unbounded debug events.
- Residual placeholders can be mistaken for the Prompt 09 projection. Tests and field docs must say placeholder; see [`residual-risk.md`](residual-risk.md).
- `skip_serializing_if` on new fields is required for golden/constructor compatibility; forgetting it changes `canonical_digest` of `Risk::new`.

---

## 9. Landed files

Product:

- `crates/weeping-angel-assurance-ir/src/risk.rs`
- `crates/weeping-angel-assurance-ir/src/risk_scoring.rs` (adapter)
- `crates/weeping-angel-assurance-ir/src/id.rs` (`FindingRef`, `RiskTreatmentId`)
- `crates/weeping-angel-assurance-ir/src/validation.rs`
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports

Tests/docs:

- `tests/contracts/risk_register.baseline.rs` (skip-superseded)
- `tests/contracts/risk_register.target.rs` (GREEN)
- root `Cargo.toml` `[[test]]` rows `sdd_risk_register_{baseline,target}`
- Accepted ADR [`docs/adr/0040-operational-risk-register.md`](../adr/0040-operational-risk-register.md)
- Public contract section in [`assurance-runtime.md`](assurance-runtime.md)

---

## 10. Definition of done

Weeping Angel has a canonical operational risk register in `assurance-ir/v1`, linked to the same assets/controls/evidence graph used by assurance, with explicit status transitions, fail-closed references, finding contributors that are not auto-promoted, inherent scoring via SDLC catalog (or a replaceable adapter), residual as placeholder, and history/supersession that survives edits.

Dual-suite SDD protocol is complete for this slice: spec first (this document), baseline GREEN on characterization, target RED, implement, docs+ADR, target GREEN, found-case baseline skip-superseded, target still GREEN.
