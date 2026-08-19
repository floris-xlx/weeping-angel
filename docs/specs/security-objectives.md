# SDD: Security Objectives Engine

| Field | Value |
| --- | --- |
| Status | **Implemented** — IR governance records + `evaluate_objective`; dual-suite registered; target GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — security objectives |
| Slice | First-class measurable security objectives + deterministic status projection from typed evidence |
| Dual-suite | `sdd_security_objectives_baseline` (skip-superseded) · `sdd_security_objectives_target` (GREEN) |
| Contract files | `tests/contracts/security_objectives.{baseline,target}.rs` — **not auto-discovered**; listed `[[test]]` in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0045-security-objectives.md`](../adr/0045-security-objectives.md). Cite by **path**. Concurrent `0008-*` siblings: [ISMS context](../adr/0008-isms-context.md), [scope engine](../adr/0044-scope-engine.md), [interested parties](../adr/0043-interested-parties-obligations.md). **Not** a `0003-*` sibling; 0004 is documentation architecture. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (security objectives section) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Typed evidence | [`docs/specs/typed-evidence.md`](typed-evidence.md) — **reuse** `EvidenceValue`; no second metric-value enum |
| Lineage | [`docs/specs/assessment-lineage.md`](assessment-lineage.md) — reuse `EvidenceSnapshot` / `LineageBundle` pins; do not fork digest law |
| Governance catalog (neighbor) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — `control.governance.security-objectives` remains an **attestation that a set exists**; it is not this engine |
| Neighbors (consume, do not rewrite) | ISMS context IR (`IsmsContext` + crate-root declaration `isms::SecurityObjective`), organizational scope engine (`ScopeResolution`), interested parties / obligations |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evaluation snapshot schema | `weeping-angel/objective-evaluation/v1` (`OBJECTIVE_EVALUATION_SCHEMA`) |
| Canonicalization | `canon/v1` via `weeping_angel_assurance_ir::canonical_digest` (compact serde JSON; no `f64`) |
| Workspace verify | `cargo test --test sdd_security_objectives_target`; `cargo test --test sdd_security_objectives_baseline`; `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for Operational ISMS v1 **security objectives**. It owns **objective records**, **typed metric/target/measurement**, **deterministic comparison**, **status projection**, and **measurement lineage**. It does **not** own dashboards, notifications, a formula VM, management-review workflows, ISMS context, the scope engine, obligations, Kleene applicability, catalog TOML, or collectors.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A security objective is a **governance record evaluated as a pure function over pinned evidence**, not a policy paragraph and not a collector conclusion.

### Landed surface

| Item | Home |
| --- | --- |
| `SecurityObjectiveId`, `ObjectiveMetricId`, `ObjectiveTargetId`, `ObjectiveMeasurementId` | `id.rs` `typed_id!` (crate-root) |
| `objectives::SecurityObjective`, `ObjectiveLifecycle`, `MetricKind`, `ComparisonOp`, `PopulationCompleteness`, `ObjectiveMeasurementSource` | [`crates/weeping-angel-assurance-ir/src/objectives.rs`](../../crates/weeping-angel-assurance-ir/src/objectives.rs) |
| Value-bearing `ObjectiveMetric` / `ObjectiveTarget` / `ObjectiveMeasurement`, `ObjectiveStatus`, `ObjectiveEvaluation`, `ObjectiveEvaluationSnapshot`, `ObjectiveError`, `evaluate_objective`, `evaluate_objective_with_resolution` | [`crates/weeping-angel-assurance/src/objectives.rs`](../../crates/weeping-angel-assurance/src/objectives.rs) |
| Context declaration `isms::SecurityObjective` (`ObjectiveId`, title/description/owner) | crate-root; **not** this engine |
| Continuity `ObjectiveStatus` | crate-root; **not** this engine |
| Dual-suite | `sdd_security_objectives_target` GREEN; baseline `#[ignore = "superseded by sdd_security_objectives_target"]` |
| Fixture | `tests/fixtures/assurance-ir/v1/security-objective-vuln-sla.json` |

IR crate-root `pub use`s only the enums/source types from `objectives`. Callers name `weeping_angel_assurance_ir::objectives::SecurityObjective` or `weeping_angel_assurance::objectives::{SecurityObjective, evaluate_objective, ObjectiveStatus}`.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) (ADR 0004). Do not write generated reports under `docs/sdd/`.

---

## 0. Collision fence (concurrent SDD)

This slice owns only security-objective types, the pure evaluator, dual-suite contracts, this spec, its ADR, `documentation_layout.rs` registration, and additive root `Cargo.toml` `[[test]]` entries.

| Do not touch | Owner |
| --- | --- |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | GitHub collector |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps, `tests/contracts/iso27001_remap.*` | Catalog / ISO remap |
| Applicability Kleene evaluator (`weeping-angel-assurance::applicability`, `OrgContext`, `evaluate_org_context`) | Applicability engine |
| Collectors emitting objective status / `OnTrack` / `Achieved` as facts | Collectors stay observation-only |
| `IsmsContext`, issues, parties, cadence root object | ISMS context IR (landed; consume declarations) |
| `ScopeResolution` implementation / precedence engine | Organizational scope engine (landed; adapter only) |
| Obligation graph | Interested parties / obligations (landed; cite ids only) |
| Dashboards / notifications / formula VM / management-review workflow | Non-goals |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Typed ids (`SecurityObjectiveId`, `ObjectiveMetricId`, `ObjectiveTargetId`, `ObjectiveMeasurementId`) | `crates/weeping-angel-assurance-ir/src/id.rs` via existing `typed_id!` |
| Governance records (owner, scope, metric kind, comparison op, cadence, dates, lifecycle, evidence-requirement ids) | `crates/weeping-angel-assurance-ir/src/objectives.rs`; crate-root re-exports enums only (not `struct SecurityObjective`) |
| Value-bearing target / measurement / evaluation (embed `EvidenceValue`) + pure `evaluate_objective` | `crates/weeping-angel-assurance/src/objectives.rs` — same crate as lineage; **not** Kleene |
| Lineage snapshot `ObjectiveEvaluationSnapshot` | `weeping-angel-assurance` (attach to / beside `LineageBundle`; do not rewrite `AssessmentRun` identity) |
| Evidence crate | **Conclusion-free.** No objective status on envelopes. |
| Collectors | **No objective types.** They advertise evidence types and emit facts. |

Landed additively: IR module + ids; assurance evaluator module; serde camelCase + `serde(default)` on new structs; dual-suite registration; `CANONICAL_SPECS` row; public-contract pointer.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** replace `Control.objective: String`. Do **not** rewrite `control.governance.security-objectives`. Do **not** add `IR → evidence` crate edge.

---

## 1. Problem / user-visible goal

Information-security objectives exist only as **prose**.

On SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- `Control.objective` is an optional `String` (“Require a fresh scan…”) with `with_objective`.
- Canonical catalog TOML repeats the same prose field (`CatalogControl.objective`).
- `control.governance.security-objectives` is a **manual** control whose test is `manual-review` over `evidence.manual.attestation`. Passing it means “someone attested that an objective set exists,” not “critical vulnerabilities are remediated within seven days at ≥ 98%.”
- There is no `SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement`, or status projection in `crates/**`.
- `IsmsContext` is absent, so there is no root object that can even *reference* objectives.
- Control-test `Effectiveness` (`Effective` / `InsufficientEvidence` / …) is a **control** outcome, not an objective status.

That means management review cannot ask, as a reconstructed fact:

```text
which objective?
which metric and comparison?
what was the baseline and target?
which scoped population was measured?
which evidence digests produced the number?
was the measurement stale, partial, or missing?
what status did that yield at time T — and can I replay it?
```

Missing measurements can be misread as success if a later slice treats “no data” as OnTrack. Manual objectives can be “achieved” without immutable attestation. A collector could be tempted to emit `status: achieved` as if that were evidence.

**User-visible goal:** make information-security objectives **measurable first-class ISMS records**. Evaluate them reproducibly from canonical evidence so they can later feed management review and continual improvement — without a dashboard, notification bus, or scripting language.

```text
SecurityObjective
  + ObjectiveMetric (kind, domain, source)
  + ObjectiveTarget (EvidenceValue + ComparisonOp)
  + ObjectiveMeasurement (EvidenceValue + explicit scope + evidence refs)
  + pinned EvidenceSnapshot
  + as_of clock
  + canonical scope (IR AssessmentScope / SubjectSelector;
                     optional pinned ScopeResolution)
        → ObjectiveEvaluation { status, lineage }
```

Example (normative fixture, not a compiler constant):

```text
metric:  percentage of in-scope critical vulnerabilities remediated within 7 days
target:  >= 98%
source:  canonical vulnerability / remediation evidence
scope:   ISMS in-scope population (explicit selectors)
cadence: monthly (or declared seconds)
status:  OnTrack | AtRisk | Missed | Achieved | InsufficientEvidence
```

Missing, stale, or partial evidence is **never** `OnTrack` or `Achieved`.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Control.objective: String` | `assurance-ir::control` | **Keep.** Prose on a control is not a `SecurityObjective`. Do not retarget `with_objective`. |
| Catalog `objective = "…"` | `catalog/canonical/v1/**`, `CatalogControl.objective` | **Keep.** Collision fence: do not rewrite TOML. |
| `control.governance.security-objectives` | governance catalog | Attestation that a set exists. This engine does not replace that control or its `manual-review` test. |
| `EvidenceValue` / `typed_eq` / `cmp_numeric` | `weeping-angel-evidence::value` | **Reuse.** Percentage / count / duration / boolean / ratio / bounded numeric compare through this API. No `f64`. No formula AST. |
| `EvidenceSnapshot` / `LineageBundle` | `weeping-angel-assurance::lineage` | **Reuse.** Objective evaluation pins envelope digests; replay does not re-query live systems. |
| IR `AssessmentScope` / `ScopeExclusion` / `SubjectSelector` | `assessment.rs`, `subject.rs` | **Reuse now** as the explicit measurement scope. |
| Facade `weeping-angel-assurance::AssessmentScope` | asset allow-list on `assess()` | Different type. Do not conflate. Objective scope is the **IR** document. |
| `ScopeResolution` | `weeping-angel-assurance::scope` (landed) | Adapter: `evaluate_objective_with_resolution` pins the snapshot; `InScope` subjects are the population. Do not reimplement precedence. Default `evaluate_objective` binds IR `AssessmentScope.subjects` minus `exclusions`. |
| `IsmsContext` | ISMS context IR (landed) | Do **not** rebuild context. Crate-root `isms::SecurityObjective` is a declaration (`ObjectiveId`). This engine's record is `objectives::SecurityObjective` (`SecurityObjectiveId`). |
| `PrincipalRef` | `implementation.rs` | **Reuse** for owner and manual attestation principal. No `ObjectiveOwner` type. |
| `EvidenceRequirement` / `FreshnessRequirement` / `EvidenceCollectionKind` | `evidence.rs` | **Reuse** for source, freshness, automated vs manual. |
| `Effectiveness` | control-test | Collision fence. Objective status is **not** `Effectiveness`. Do not add `OnTrack` to that enum. |
| `canonical_digest` | `digest.rs` | **Reuse.** No second digest system. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Do **not** fork. Governance records inherit this schema. |
| Dual-suite neighbors | root `Cargo.toml` | `sdd_security_objectives_*` listed next to existing `sdd_*`. Directory is **not** auto-discovered. |
| Docs layout | ADR 0004 | Human SSOT is this file. Traces go to `.sdd/runs`. This path is in `CANONICAL_SPECS`. |

IR **must not** depend on `weeping-angel-evidence` (evidence already depends on IR; reversing the edge is forbidden by ADR 0001). Value-bearing evaluation types therefore live in `weeping-angel-assurance`, which already depends on both. See [ADR 0008](../adr/0045-security-objectives.md).

Landed additively: IR module + ids; assurance `objectives` module; golden fixture `security-objective-vuln-sla.json` **in addition to** existing IR JSON; dual-suite registration.

Do **not** redesign `AssessmentDefinition` inventories, collectors, catalog TOML, Kleene evaluation, or `Control.objective`.

---

## 3. Current behavior (baseline — characterization SHA; skip-superseded)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. After target GREEN the baseline suite is `#[ignore = "superseded by sdd_security_objectives_target"]`. This section remains the found-case record.

### 3.1 Zero objective engine types

Product crates (`crates/**/src/**/*.rs`) contain none of:

- `SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement`
- `SecurityObjectiveId`, `ObjectiveMetricId`, `ObjectiveTargetId`, `ObjectiveMeasurementId`
- `ObjectiveStatus`, `ObjectiveLifecycle`, `MetricKind`, `ComparisonOp`
- `OnTrack`, `AtRisk`, `Missed`, `Achieved` as objective-status variants (control-test has no such enum)
- `evaluate_objective`, `ObjectiveEvaluation`, `ObjectiveEvaluationSnapshot`

`weeping-angel-assurance-ir/src/lib.rs` has no `objectives` module. `id.rs` `typed_id!` list has no objective ids.

### 3.2 Objectives are prose on controls and catalog rows

[`crates/weeping-angel-assurance-ir/src/control.rs`](../../crates/weeping-angel-assurance-ir/src/control.rs):

```text
Control { … objective: String, … }   // skip empty
Control::new → objective = ""
Control::with_objective(self, impl Into<String>)
Control::objective(&self) -> &str
```

Canonical catalog loader copies the same string (`CatalogControl.objective`). Vulnerability / governance TOML fill it with English sentences. That string is never compared to a target.

### 3.3 Governance catalog attests existence, not measurement

[`catalog/canonical/v1/controls/governance.toml`](../../catalog/canonical/v1/controls/governance.toml):

```text
control.governance.security-objectives
  title       = "Security objectives"
  description = "Security objectives are recorded as an attestation, not inferred from a score."
  objective   = "Require an attested objective set for the in-scope organization."
  automation  = "manual"
  tests       = ["test.governance.objectives-attested"]
```

[`catalog/canonical/v1/tests/governance.toml`](../../catalog/canonical/v1/tests/governance.toml): `kind = "manual"`, `required_evidence = ["evidence.manual.attestation"]`, `op = "manual-review"`.

There is no metric, baseline, target, cadence, or status projection.

### 3.4 Scope is assessment-shaped, not an objective engine

IR `AssessmentScope` is `{ organizations, subjects: Vec<SubjectSelector>, exclusions: Vec<ScopeExclusion> }`. `ScopeExclusion` is `{ subjects, rationale }` — no owner, approval, expiry (those belong to the scope engine). Facade `AssessmentScope` is an asset allow-list for collectors. `ScopeResolution` / `InScope|OutOfScope|Conditional|Unknown` **do not exist**.

No API scopes a *measurement* onto a population for an objective.

### 3.5 Evidence values and lineage exist — unused by objectives

`EvidenceValue` variants: `String`, `Bool`, `Integer`, `Decimal`, `Timestamp`, `DurationSeconds`, `StringList`, `Object`. Comparisons: `typed_eq`, `cmp_numeric` (Integer↔Decimal exact decimal; no `f64`). Type mismatch is an error string, not a silent coerce.

`EvidenceSnapshot { envelope_digests, collection_run_ids, digest }` and `LineageBundle` exist for **control tests**. Nothing stores how an objective status was produced.

`FreshnessRequirement { max_age_seconds }` exists on `EvidenceRequirement`. Stale control-tests become `Effectiveness::StaleEvidence`. There is no objective-level stale rule.

Manual attestation exists as catalog evidence `evidence.manual.attestation` and compile flag `supports_manual_attestation`. It is not wired to an objective measurement.

### 3.6 ISMS context / parties / obligations

| Neighbor | In tree on this SHA |
| --- | --- |
| `IsmsContext` + `ISMS → Objective` | No |
| Scope engine `ScopeResolution` | No |
| `InterestedParty` / `Obligation` | No |

`AssessmentDefinition` inventories: requirements, controls, mappings, evidence requirements, tests, implementations, scope, assets, identities, vendors, risks, exceptions, processing activities. **No objectives list.**

### 3.7 Collectors cannot (and must not) emit objective status

Collectors emit `EvidenceEnvelope` facts. Seal rejects compliance-shaped narratives. There is no path that writes `ObjectiveStatus` onto an envelope. Baseline must keep it that way; target must grep collectors for objective-status types.

### 3.8 What current tests lock

- `sdd_governance_catalog_target`: manual objectives-attested control remains `manual-review`.
- `sdd_typed_evidence_target`: one `EvidenceValue`; `cmp_numeric` / `typed_eq`.
- `sdd_assessment_lineage_target`: `EvidenceSnapshot` digest identity; replay from pins.
- `sdd_compliance_ir_target`: golden IR fixtures; `Control` round-trip including optional `objective` string.
- No workspace test constructs `SecurityObjective`, compares a 98% boundary, or forbids missing measurements from yielding success.

---

## 4. Desired behavior

### 4.1 Home and purity

```text
evaluate_objective(input) → ObjectiveEvaluation
```

Same objective bytes + same measurement bytes + same evidence snapshot digest + same `as_of` + same scope binding ⇒ same status, same reason codes, same `canonical_digest` of the evaluation snapshot.

- **No clock inside the function** (`as_of` is an argument).
- **No I/O**, no collector calls, no live GitHub, no `Utc::now()`.
- **No Kleene** applicability evaluation.
- **No `FrameworkProfile` / ISO clause** fields on generic types.
- **No formula language.** Comparison is `ComparisonOp` × `EvidenceValue` only.

### 4.2 Records

Serde: `camelCase`. Exhaustive enums; unknown JSON tags fail closed.

#### `SecurityObjective` (IR governance record)

| Field | Type / notes |
| --- | --- |
| `id` | `SecurityObjectiveId` (`typed_id!`) |
| `schemaVersion` | `assurance-ir/v1` (or inherit) |
| `title` | non-empty |
| `description` | non-empty |
| `owner` | `PrincipalRef` (required when `lifecycle = active`) |
| `scope` | IR `AssessmentScope` (required; empty subjects with empty organizations is invalid for Active) |
| `metricId` | `ObjectiveMetricId` |
| `targetId` | `ObjectiveTargetId` |
| `baseline` | optional encoded value (see §4.3) |
| `measurementSource` | `EvidenceType` + `EvidenceCollectionKind` + optional `EvidenceRequirementId` |
| `cadenceSeconds` | optional `u64`; if set, must be > 0 |
| `startAt` | `DateTime<Utc>` (required for Active) |
| `deadlineAt` | optional `DateTime<Utc>` (≥ `startAt` if both set) |
| `reviewAt` | optional review date |
| `lifecycle` | `ObjectiveLifecycle` |
| `logicalId` | stable family id (string; same charset as stable ids) |
| `revision` | monotonic `u32` starting at 1 |
| `supersedes` | optional previous `SecurityObjectiveId` |

`ObjectiveLifecycle = Draft | Active | Retired | Superseded` (`camelCase`).

Evaluating a non-`Active` objective is a **validation error** (fail closed), not `OnTrack`/`Achieved`. Historical snapshots remain queryable.

Do **not** store projected `ObjectiveStatus` on this record. Status is always a snapshot (§4.7).

#### `ObjectiveMetric`

| Field | Notes |
| --- | --- |
| `id` | `ObjectiveMetricId` |
| `kind` | `MetricKind` |
| `unit` | optional string (`percent`, `count`, `seconds`, …) — documentation; comparison uses `kind` |
| `domainMin` / `domainMax` | required for `boundedNumeric` and `percentage`; encoded as `EvidenceValue` integers/decimals |
| `evidenceType` | canonical type the measurement is drawn from (e.g. vulnerability remediation facts) |
| `valueField` | fact key on the observation (`EvidenceValue` at that key) |
| `freshness` | optional `FreshnessRequirement` |

```text
MetricKind =
  Percentage
  | Count
  | Duration
  | Boolean
  | Ratio
  | BoundedNumeric
```

JSON tags: `percentage`, `count`, `duration`, `boolean`, `ratio`, `boundedNumeric`.

#### `ObjectiveTarget`

| Field | Notes |
| --- | --- |
| `id` | `ObjectiveTargetId` |
| `comparison` | `ComparisonOp` |
| `value` | `EvidenceValue` (assurance layer) / evidence-value/v1 encoding on the IR document |

```text
ComparisonOp = Eq | Neq | Gt | Gte | Lt | Lte
```

Boolean metrics allow `Eq` / `Neq` only. Ordered kinds use `cmp_numeric`. `Neq` on numeric kinds is allowed but must not be used to encode a band (use two objectives or `boundedNumeric` domain).

#### `ObjectiveMeasurement`

| Field | Notes |
| --- | --- |
| `id` | `ObjectiveMeasurementId` |
| `objectiveId` | `SecurityObjectiveId` |
| `observedAt` | `DateTime<Utc>` (must be ≤ `as_of` to be eligible) |
| `value` | `EvidenceValue` |
| `scope` | IR `AssessmentScope` **required** — unscoped measurements are invalid |
| `evidenceRefs` | one or more envelope digests (`ev:sha256:…`) |
| `attestationRef` | required when `measurementSource.collection = manual` |
| `populationCompleteness` | `authoritative` / `partial` / `unknown` (align names with population runtime; unknown/partial degrade) |

A measurement whose `scope` is not a subset of the objective’s scope, or that includes out-of-scope subjects, is **not** silently trimmed: evaluation degrades to `InsufficientEvidence` with reason `scopeMismatch`.

### 4.3 `EvidenceValue` mapping (no second metric type)

| `MetricKind` | Allowed `EvidenceValue` | Comparison |
| --- | --- | --- |
| `percentage` | `Integer` 0..=100 or `Decimal` in that range | `cmp_numeric` vs target of the same kind |
| `count` | `Integer` ≥ 0 | `cmp_numeric` |
| `duration` | `DurationSeconds` | `cmp_numeric` |
| `boolean` | `Bool` | `typed_eq` |
| `ratio` | `Object { "numerator": Integer, "denominator": Integer }` both ≥ 0; denominator **must not** be 0 | exact decimal `n/d` vs target `Decimal` or another ratio object via `cmp_numeric` on the reduced decimal text — **no `f64`** |
| `boundedNumeric` | `Integer` or `Decimal` inside declared `[min, max]` inclusive | `cmp_numeric`; out of domain → `InsufficientEvidence` (**no clamp**) |

Type mismatch, illegal decimal text, denominator 0, percentage outside 0..=100, or out-of-domain bounded value ⇒ `InsufficientEvidence` (`typeMismatch` / `outOfDomain`). Never success.

**Crate split (normative):** IR documents persist target/baseline/measurement payloads as **evidence-value/v1 JSON** (the existing codec). They do **not** define `enum EvidenceValue`. `weeping-angel-assurance` is the only constructor that accepts `EvidenceValue` and writes that encoding. Evaluation **must** deserialize through `EvidenceValue` before compare. Tests grep IR sources for a second `enum EvidenceValue` / `enum MetricValue` / `enum ObjectiveValue` and fail if found.

### 4.4 Scope every measurement

Default binding (`evaluate_objective`):

1. Objective `scope` is IR `AssessmentScope`.
2. Measurement `scope` is IR `AssessmentScope`.
3. A subject contributes only if it matches objective inclusion selectors and is not in `exclusions`.
4. Ambiguous / empty active scope fails closed (`InsufficientEvidence` / validate error on Active records).

Pinned resolution (`evaluate_objective_with_resolution`):

- Evaluator accepts an optional `ScopeResolution` snapshot (digest-pinned).
- Population = subjects with outcome `InScope` (not `Conditional` / `Unknown` / `OutOfScope`).
- `Conditional` / `Unknown` subjects **do not** count as in-scope successes; if the metric needs a complete population and any required subject is not `InScope`, degrade to `InsufficientEvidence`.
- Do not reimplement precedence here.

Facade `AssessmentScope` (asset allow-list) is **not** an objective scope.

Out-of-scope subjects **must not** contribute positive measurements (same spirit as the scope engine’s “out-of-scope cannot contribute positive assurance”).

### 4.5 Manual vs automated

| Source | Required evidence | Missing piece |
| --- | --- | --- |
| `automated` | One or more sealed envelopes of the declared `EvidenceType`, facts typed, scoped | `InsufficientEvidence` |
| `manual` | Immutable attestation/approval envelope (`evidence.manual.attestation` or the declared type) with `PrincipalRef` + timestamp + subject + artifact; **sealed** (ledger-append semantics) | `InsufficientEvidence` — a boolean fact without attestation is not achievement |
| mixed (two objectives, one auto + one manual) | Each evaluated independently; a bundle projection must not let the automated `OnTrack` promote the manual one | Target tests a mixed pair |

Collectors never write `ObjectiveStatus`. A string fact `"status": "achieved"` is not an objective status.

### 4.6 Status projection

```text
ObjectiveStatus = OnTrack | AtRisk | Missed | Achieved | InsufficientEvidence
```

JSON: `onTrack`, `atRisk`, `missed`, `achieved`, `insufficientEvidence`.

`as_of` is the evaluation clock. Eligible measurements: `observedAt ≤ as_of`, in cadence window if cadence is set:

```text
window_start = max(startAt, as_of - cadenceSeconds)   // if cadence present
             | startAt                                 // else
candidate    = latest eligible measurement in [window_start, as_of]
               (tie: lexicographically greater measurement id)
```

**Degradation (evaluated first; wins over comparison):**

| Condition | Status | Reason code (stable) |
| --- | --- | --- |
| Objective not `Active` | **error**, not a status | `notActive` |
| No candidate measurement | `InsufficientEvidence` | `missingMeasurement` |
| Candidate `observedAt` older than `freshness.max_age_seconds` relative to `as_of` | `InsufficientEvidence` | `staleMeasurement` |
| Cadence set and no measurement in the current window | `InsufficientEvidence` | `staleMeasurement` |
| `populationCompleteness` is `partial` or `unknown` | `InsufficientEvidence` | `partialEvidence` |
| Evidence cardinality / required refs missing | `InsufficientEvidence` | `partialEvidence` |
| Unscoped measurement or `scopeMismatch` | `InsufficientEvidence` | `scopeMismatch` |
| Manual without `attestationRef` resolving in the pinned snapshot | `InsufficientEvidence` | `missingAttestation` |
| Type / domain / ratio-denominator failure | `InsufficientEvidence` | `typeMismatch` / `outOfDomain` |
| Pinned envelope digest not in `EvidenceSnapshot` | `InsufficientEvidence` | `missingEvidence` |

**Never** map those rows to `OnTrack` or `Achieved`.

**Comparison (only after a valid candidate):**

Let `holds = compare(candidate.value, target.comparison, target.value)` using `typed_eq` / `cmp_numeric`.

| `holds` | Deadline | Status |
| --- | --- | --- |
| true | absent, or `as_of ≤ deadlineAt` | `OnTrack` |
| true | `deadlineAt` present and `as_of > deadlineAt` | `Achieved` |
| false | absent, or `as_of ≤ deadlineAt` | `AtRisk` |
| false | `deadlineAt` present and `as_of > deadlineAt` | `Missed` |

Ongoing objectives (no deadline) **cannot** become `Achieved` or `Missed` from time alone; they stay `OnTrack` / `AtRisk` / `InsufficientEvidence`. That is intentional: “≥ 98% remediated within seven days” is a standing objective.

`startAt > as_of` → `InsufficientEvidence` (`notStarted`) — not success.

Historical windows: each evaluation snapshot is immutable. A later measurement produces a **new** snapshot; it does not rewrite the previous digest.

### 4.7 Lineage (management-review reconstructability)

`ObjectiveEvaluationSnapshot` (`weeping-angel/objective-evaluation/v1`):

| Pin | Purpose |
| --- | --- |
| `objectiveId` + objective document digest | which record |
| `metricId` + metric digest | which definition |
| `targetId` + target encoding digest | which threshold |
| `measurementId` + measurement digest (or explicit missing) | which number |
| `evidenceSnapshotDigest` + envelope digests | which facts |
| `scopeDigest` (IR `AssessmentScope` or later `ScopeResolution`) | which population |
| `asOf` | which clock |
| `status` + `reasonCodes` | result |
| `comparison` + operands as evidence-value/v1 | how it was decided |

Replay: `evaluate_objective` over the pinned bundle must byte-equal the stored status and reason codes. Consumers **must not** need live collectors or current catalog files.

Attach snapshots to lineage storage the same way other projections do (ledger opaque JSON / bundle field). Do not collapse them into `ControlTestResult`.

### 4.8 Example: critical vulnerability remediation (fixture law)

```text
kind:        percentage
field:       remediated_within_sla_percent   // EvidenceValue Decimal or Integer
evidence:    canonical vulnerability / remediation types (consume vulnerability catalog facts; do not edit that catalog)
comparison:  gte
target:      98
population:  in-scope assets with open-or-closed critical findings in the window
SLA window:  7 days (encoded as duration fact on each finding, not as a script)
cadence:     2592000 seconds (30d) or a test-declared period
```

Boundary tests (normative):

| Measured | `as_of` vs deadline | Expected |
| --- | --- | --- |
| `98` | before deadline | `OnTrack` |
| `97` | before deadline | `AtRisk` |
| `98` | after deadline | `Achieved` |
| `97` | after deadline | `Missed` |
| missing | any | `InsufficientEvidence` |
| `98` but stale vs freshness | any | `InsufficientEvidence` |
| `100` with `completeness=unknown` | any | `InsufficientEvidence` |
| `100` including an out-of-scope repo | any | `InsufficientEvidence` (`scopeMismatch`) |

`98.0` vs `98` must compare equal via `cmp_numeric` Integer↔Decimal. `97.999` (canonical decimal text) is `AtRisk`/`Missed`, not rounded up.

### 4.9 Validation

`ObjectiveError` (or equivalent) has stable `Display` needles:

| Class | Examples |
| --- | --- |
| Identity | empty title; invalid `SecurityObjectiveId`; uuid-v4; self-`supersedes` |
| Lifecycle | Active without owner/scope/start; evaluate on Draft/Retired/Superseded |
| Time | `deadlineAt < startAt`; zero cadence |
| Metric | percentage without domain; bounded without min/max; min > max |
| Value | evidence-value/v1 that will not decode; `f64` JSON number with fraction for Integer |
| Scope | Active with empty scope; measurement missing scope |
| Manual | `collection=manual` without attestation slot on the measurement type |
| Locked/supersede | optional: once an evaluation snapshot pins a revision, the objective document is immutable; evolution is `supersede()` → new id, `revision+1` |

Do not stringify as opaque `"invalid"`.

### 4.10 Collision fences (product)

| Do not | Why |
| --- | --- |
| Add `OnTrack` to `Effectiveness` | Different domain |
| Let GitHub collector emit objective status | Collectors emit facts |
| Rewrite catalog TOML / ISO remaps | Concurrent owners |
| Call Kleene `evaluate_org_context` from the objective evaluator | Applicability is not a metric |
| Implement `IsmsContext` / `ScopeResolution` / obligations | Neighbors own those; this slice consumes |
| Build a formula VM (`measured * 0.98 + baseline`) | Non-goal |
| Dashboard / notifications / `isms objectives` CLI in this slice | Non-goal |
| Management-review workflow / CAPA linkage | Later slices consume snapshots |

---

## 5. Dual-suite protocol (HARD SDD)

`tests/contracts` is **not** auto-discovered. Dual-suite is **registered** in root `Cargo.toml`:

```toml
[[test]]
name = "sdd_security_objectives_baseline"
path = "tests/contracts/security_objectives.baseline.rs"

[[test]]
name = "sdd_security_objectives_target"
path = "tests/contracts/security_objectives.target.rs"
```

Protocol (completed):

```text
Spec first
  → Dual-suite registered
  → Baseline GREEN on characterization absence / prose-only governance
  → Target RED (missing types/APIs)
  → Implement (IR ids + governance records + assurance evaluator + lineage snapshot)
  → ADR 0008 Accepted
  → Target GREEN
  → Baseline skip-supersede (`#[ignore = "superseded by sdd_security_objectives_target"]`)
  → Target still GREEN
```

Write tests **before** product evaluator (RED → fix → GREEN). One regression test per case, titled `SO: <exact subject>` (slice name; **never** “Prompt N” in specs, ADRs, or test comments).

Absence/characterization baseline is a **replacement** transition: after target GREEN, baseline must fail or be `#[ignore]` superseded. Keep dual-suite registration.

---

## 6. Acceptance criteria (testable)

### 6.1 Baseline suite (skip-superseded)

Encode **current** HEAD. Titles `SO: …` for the found case.

| ID | Assertion |
| --- | --- |
| SO-B01 | Product crate sources contain no `struct SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement` |
| SO-B02 | `id.rs` / IR `lib.rs` do not define `SecurityObjectiveId` or an `objectives` module |
| SO-B03 | `Control.objective` is still `String`; `with_objective` exists; empty default |
| SO-B04 | `control.governance.security-objectives` remains manual + `test.governance.objectives-attested` `manual-review` |
| SO-B05 | No `evaluate_objective` / `ObjectiveStatus` / `OnTrack` objective projection in `crates/**` |
| SO-B06 | Collectors have no objective-status types |
| SO-B07 | `IsmsContext` and `ScopeResolution` remain absent (do not fail the suite if a concurrent slice lands first — skip those needles only if types exist **and** this suite still asserts prose-only objectives) |
| SO-B08 | This spec path exists under `docs/specs/security-objectives.md` |

### 6.2 Target suite (GREEN)

| ID | Assertion |
| --- | --- |
| SO-T01 | Types exist: `SecurityObjective`, `ObjectiveMetric`, `ObjectiveTarget`, `ObjectiveMeasurement`, `ObjectiveStatus`, `evaluate_objective` |
| SO-T02 | Percentage boundary: `98` vs target `gte 98` → success path; `97` → not success (`AtRisk` or `Missed` per deadline) |
| SO-T03 | Integer↔Decimal `98` vs `98.0` compares equal via `EvidenceValue::cmp_numeric`; no `f64` |
| SO-T04 | Count / duration / boolean / ratio / bounded-numeric each have a boundary test (ratio denominator 0 → `InsufficientEvidence`) |
| SO-T05 | Missing measurement → `InsufficientEvidence`, never `OnTrack`/`Achieved` |
| SO-T06 | Stale measurement (freshness or cadence window) → `InsufficientEvidence`, never success |
| SO-T07 | Partial / unknown population completeness → `InsufficientEvidence` even if the number meets target |
| SO-T08 | Mixed pair: automated objective `OnTrack` does not promote a manual objective lacking attestation |
| SO-T09 | Manual with sealed attestation + meeting target → `OnTrack`/`Achieved` per deadline; manual boolean without attestation → `InsufficientEvidence` |
| SO-T10 | Measurement scope required; unscoped or out-of-scope mix → `InsufficientEvidence` (`scopeMismatch`) |
| SO-T11 | Historical: two `as_of` clocks on the same pinned snapshots yield two immutable evaluations; later clock does not mutate the earlier digest |
| SO-T12 | Deterministic transitions: same inputs → byte-equal snapshot; `OnTrack`→`Achieved` after deadline if still meeting; `AtRisk`→`Missed` after deadline if not |
| SO-T13 | Replay of a pinned `EvidenceSnapshot` reproduces status without collectors |
| SO-T14 | Collectors / GitHub normalize contain no `ObjectiveStatus`; seal still rejects compliance narratives |
| SO-T15 | IR sources do not define a second `EvidenceValue` / `MetricValue` enum; evaluator calls `typed_eq` / `cmp_numeric` |
| SO-T16 | Kleene applicability module is not imported by the objective evaluator |
| SO-T17 | `Control.objective` string API unchanged; governance catalog TOML for `security-objectives` unchanged |
| SO-T18 | Active objective with empty scope fails validate; Draft may omit owner |
| SO-T19 | Schema remains `assurance-ir/v1` for governance records; evaluation snapshot uses `weeping-angel/objective-evaluation/v1` |
| SO-T20 | Dual-suite names registered in root `Cargo.toml`; this spec listed in `CANONICAL_SPECS` after implement |

Cover explicitly: threshold boundaries, missing data, stale measurement, mixed manual/automated, scoped populations, historical measurements, deterministic status transitions.

---

## 7. Out of scope

- Dashboards, charts, UI, or operator consoles
- Notifications, alerting, email, chatops
- Arbitrary formula execution / scripting / expression VM
- Management-review workflows, meeting minutes, CAPA linkage (consumers of snapshots later)
- Implementing `IsmsContext`, organizational scope-engine precedence, or the obligation graph
- Rewriting `Control.objective` prose or governance catalog attestation control
- Catalog TOML / ISO remaps / GitHub collector mapping
- Kleene applicability changes
- New crate, CLI `objectives` command, persistence service, or remote ledger
- Collectors emitting objective status

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Missing data treated as OnTrack | SO-T05; degradation table is first in the evaluator |
| Second metric-value enum in IR | SO-T15; ADR 0008 crate split; values are `EvidenceValue` |
| IR → evidence crate edge | Forbidden; value-bearing types in `weeping-angel-assurance` |
| Collectors emit `Achieved` | SO-T14; evidence crate conclusion-free |
| Conflating `Effectiveness` with `ObjectiveStatus` | Separate enum; T16; do not extend control-test |
| Facade `AssessmentScope` used as ISMS scope | Spec names IR `AssessmentScope` / future `ScopeResolution` |
| Scope engine / context land in different shapes | Adapter pins `ScopeResolution.digest`; fail closed when a measurement subject is not `InScope` |
| Stale evidence still “green” | SO-T06; cadence + freshness |
| Partial population + 100% of *observed* treated as Achieved | SO-T07 |
| Formula VM creep | `ComparisonOp` exhaustive; tests forbid `eval`/`script` APIs |
| Baseline blocks CI after implement | Skip-supersede like other replacement suites |
| “Prompt N” leaking into docs/tests | This spec and ADR use slice name **security objectives** only |

---

## 9. ADR

This is an architecture/contract decision (crate home for value-bearing types, reuse of `EvidenceValue`, status algebra, lineage snapshot schema, scope binding, crate-root name split). Accepted: [`docs/adr/0045-security-objectives.md`](../adr/0045-security-objectives.md).

Filename **`0008-*`**. Do **not** add `0003-security-objectives.md`. Catalog-program ADRs share `0003-*`; documentation architecture is `0004`; concurrent Operational ISMS siblings also use `0008-*`.

---

## 10. Implementation notes (landed)

Owned crates: `weeping-angel-assurance-ir` (ids + governance records) and `weeping-angel-assurance` (evaluator + snapshots). No new crate.

Public exports:

```text
SecurityObjectiveId, ObjectiveMetricId, ObjectiveTargetId, ObjectiveMeasurementId
weeping_angel_assurance_ir::objectives::SecurityObjective
ObjectiveMetric, ObjectiveTarget, ObjectiveMeasurement
ObjectiveLifecycle, MetricKind, ComparisonOp
weeping_angel_assurance::objectives::{ObjectiveStatus, ObjectiveEvaluation, ObjectiveEvaluationSnapshot}
ObjectiveError, evaluate_objective, evaluate_objective_with_resolution
SecurityObjective::{try_new, supersede}
```

Keep `Control` / `with_objective` untouched.

Fixture: `tests/fixtures/assurance-ir/v1/security-objective-vuln-sla.json` plus sealed evidence envelopes for the 98% case.

Public contract: [`docs/specs/assurance-runtime.md`](assurance-runtime.md). Spec path is in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS`. ADR 0008 is Accepted.

Traces: `.sdd/runs/` only (ADR 0004). `docs/sdd` remains a stub.

---

## 11. Handoff contract (downstream slices)

```text
ISMS context IR     crate-root isms::SecurityObjective (ObjectiveId declaration); this engine is objectives::SecurityObjective
scope engine        evaluate_objective_with_resolution accepts pinned ScopeResolution
interested parties  obligations may *cite* an objective id; they do not compute status
management review   reads ObjectiveEvaluationSnapshot; does not re-query collectors
continual improvement / CAPA   may open from Missed / AtRisk snapshots later
vulnerability catalog          facts remain findings/remediation; this slice measures them
GitHub collector               still emits repository/vuln facts, never ObjectiveStatus
```

Downstream must **not** teach tests that an attested governance control is a measured SLA. Downstream must **not** hardcode 98% in the evaluator (it is fixture data).

---

## 12. Definition of done

Security objectives evaluate reproducibly from evidence. Replaying a pinned snapshot yields the same status. Missing, stale, or partial evidence never yields `OnTrack` or `Achieved`. Typed comparisons have boundary tests. Objective scope reuses canonical IR `AssessmentScope` / `SubjectSelector` and a pinned `ScopeResolution` adapter. Management-review consumers can reconstruct metric, target, measurement, evidence lineage, and status without live systems.

Dual-suite complete: spec → baseline GREEN → target RED → implement → target GREEN → baseline superseded → target still GREEN. Workspace verify: `cargo test --workspace --features demo`.
