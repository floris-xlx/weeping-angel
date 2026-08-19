# SDD: Risk Identification and Candidate Correlation (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_risk_identification_target` GREEN; characterization baseline skip-superseded |
| Program | Operational ISMS v1 — risk identification |
| Slice | Deterministic `RiskCandidate` discovery + clustering from existing evidence; explicit promotion / dismissal; claim-deny for scanner “risk accepted” / “ISO control failed” |
| Dual-suite | `sdd_risk_identification_baseline` · `sdd_risk_identification_target` (`tests/contracts/risk_identification.{baseline,target}.rs`) — listed in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0007-risk-identification-candidate-correlation.md`](../adr/0007-risk-identification-candidate-correlation.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Consumes | [`risk-methodology.md`](risk-methodology.md) (`RiskScoreInput` / `ScoredRisk` / `score_risk`); [`risk-register.md`](risk-register.md) (operational `Risk` slots on promote) |
| Neighbors (do not implement here) | risk treatment treatment engine; residual risk residual scoring; GitHub collector mapping; catalog TOML / ISO packs |
| Collision fence | Do not overwrite risk methodology methodology types or risk register `Risk` schema; no second `Risk` type; no second scoring model |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; keep `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_compliance_ir_target`, and any landed `sdd_risk_methodology_*` / `sdd_risk_register_*` GREEN |

This document is the durable human SSOT for risk identification. It owns **risk candidate discovery**, **deterministic correlation**, **explicit promotion and dismissal**, **resurfacing rules**, **stale-evidence gates**, and the **shared claim-deny extension** that scanners/collectors/evidence seal must honor. It does **not** own methodology scales/matrices (risk methodology), the operational `Risk` schema (risk register), treatment plans (risk treatment), residual scoring (residual risk), UI queues, LLM clients, or threat intelligence.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A scanner finding is **evidence**. A `RiskCandidate` is a **proposal**. A `Risk` is a **management-system record**. Only a named principal can cross the last boundary.

```text
observations / inventories
        ↓  deterministic identify + cluster
   RiskCandidate     ≠     Risk
        ↓  promote(principal, time, rationale, methodology inputs)
   Risk  (risk register record)
```

### Landed surface (this HEAD)

| Item | Where |
| --- | --- |
| Types | `weeping-angel-assurance-ir::{risk_candidate,risk_promotion}`; ids `RiskCandidateId` / `PromotionId` / `DismissalId` |
| Engine | `weeping-angel-assurance::risk_identification` — `identify_risk_candidates`, `correlate_candidates`, `promote_candidate`, `dismiss_candidate`, `should_resurface` |
| Claim-deny | `weeping_angel_evidence::looks_like_compliance_claim` needles include `risk accepted` / `ISO control failed` (and listed variants) |
| Promote insert | `promote_candidate(&mut AssessmentDefinition, …)` constructs Prompt 06 `Risk` (`scenario`, `RiskSource::Finding`, `finding_refs` from `oi:sha256:…`, `asset_ids`, `owner`, `discovered_at`) and pushes it. Identify never inserts. |
| Score suggestion | Omitted by identify. Optional reviewer `ScoreSuggestion` stored on `PromotionRecord`. No second matrix. Category cluster tie-break is `SuggestedRiskCategory::Other("mixed")`. |
| Resurface | Same `RiskCandidateId`; status `Dismissed` → `Resurfaced` when `I ⊈ D` on the same subject+scenario key. |
| Dual scenario | `canonical_type = security.vulnerability.present` on inventory `Service` also listed in a processing-activity `systems` vec → two keys (confidentiality / integrity-or-availability). |
| Mapper | `security_finding` only. Inventory-only / empty observations → `[]`. |

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

Parallel SDD runs landed risk methodology (`docs/specs/risk-methodology.md`, ADR `0005-risk-methodology.md`) and risk register (`docs/specs/risk-register.md`, ADR `0005-operational-risk-register.md`) in the same workspace. Those documents remain SSOT for scoring and the register. This slice **consumes** them. It must not rewrite, fork, or silently invert their contracts.

| Do not touch | Owner |
| --- | --- |
| `RiskMethodology`, scales, matrices, `score_risk`, `RiskScoreInput`, `ScoredRisk`, `DerivedRating` | risk methodology — **consume**; identify does not call `score_risk` |
| Operational `Risk` field list, status machine, `FindingRef`, history/supersession | risk register — **consume** on promotion (`Risk::new` + additive slots) |
| `tests/fixtures/assurance-ir/v1/risk.json` required keys | risk register / IR-019 — must keep decoding |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` rewrite | GitHub collector — **except** shared `looks_like_compliance_claim` needles if required |
| risk treatment treatment types / residual risk residual engine | Later slices |
| `tests/contracts/{compliance_ir,assurance_runtime,applicability_engine,governance_catalog}.*` rewrite | Existing suites — stay GREEN |
| `src/finding.rs` scanner `Finding` | Recon/scanner product; not IR |
| ADR filenames `0005-*` / `0006-*` | risk methodology, register, and treatment decisions |

Tiny allowed adjustments at implement: new IR modules beside [`risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs) (`risk_candidate`, `risk_promotion` or equivalent); `RiskCandidateId` via `typed_id!`; `lib.rs` re-exports; `weeping-angel-assurance` identify/correlate module (applicability-style); extend [`looks_like_compliance_claim`](../../crates/weeping-angel-evidence/src/lib.rs) needles (collector already calls it). Optional serde-default fields **on candidate/promotion types only**.

Do **not**: expand `Risk` into a candidate; add `From<RiskCandidate> for Risk` that skips promotion; invent `RiskV2`; dump identification into `risk.rs`; put an LLM or probabilistic clusterer in core runtime.

---

## 1. Problem / user-visible goal

The IR already holds organizational inventories (`assets`, `identities`, `vendors`, `processing_activities`, `risks`) and collectors already emit `security_finding` observations. Nothing turns that evidence into **explainable risk proposals**, and nothing stops a future mapper from treating a finding as an accepted risk or a control failure.

Today that means:

- `Risk` is `{ id, title, description, status ∈ Open|Accepted|Mitigated|Closed }` — an inventory stub so `ControlImplementation.risk_ids` can resolve (IR-019).
- There is no `RiskCandidate`, correlation key, promotion record, or dismissal record in product crates.
- `AssessmentDefinition.risks` is only what authors put there. Observations never insert into it.
- `looks_like_compliance_claim` rejects ISO/SOC2/GDPR “compliant/certified” narratives and `control test result`, but **not** `risk accepted` or `ISO control failed`.
- Scanner bridge is one-way `security_finding` facts (`rule_id`, `path`, `category`, `canonical_type`) — correct as evidence, unused as identification input.
- risk register law (specified, not necessarily implemented): a finding is a contributor, never a risk. risk identification must enforce that boundary in runtime, not only in register docs.

**User-visible goal:** continuously surface explainable candidate risks from existing evidence while preserving the management decision boundary.

```text
N findings, same subject + same scenario
  → 1 RiskCandidate (clustered), 0 Risk until promote

1 finding, two distinct scenario keys
  → 2 RiskCandidates; each may promote to a distinct RiskId

identify() with no observations / no mappable facts
  → 0 candidates; AssessmentDefinition.risks unchanged

promote(candidate, principal, rationale, inputs)
  → new Risk (risk register record); candidate id + correlation key unchanged

dismiss(candidate, principal, rationale)
  → retained; re-identify on identical observation identities does not auto-promote
    and does not flip status back to Proposed

stale supporting evidence
  → candidate may be listed as Stale; promote() fails closed

scanner / collector / seal narrative "risk accepted" or "ISO control failed"
  → rejected (same claim-deny path as other compliance claims)
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Risk` / `RiskStatus` / `Risk::new` | `risk.rs` | **Do not redesign.** Promotion constructs a `Risk` via `Risk::new` plus register slots (`scenario`, `source`, `finding_refs`, `asset_ids`, `owner`, `discovered_at`, `methodology_version`). Never replace `Risk` with `RiskCandidate`. |
| `RiskId` | `id.rs` | Unchanged. Add sibling `RiskCandidateId` (and promotion/dismissal ids if needed). |
| `AssessmentDefinition.risks` | `assessment.rs` | Stay `Vec<Risk>`. Identification **must not** push into this vec. Only `promote` may insert a `Risk`. |
| `ValidateIr` / IR-019 | `validation.rs` | Keep dangling `ControlImplementation.risk_ids`. Candidate types validate separately; they are **not** required to live inside `AssessmentDefinition` in this slice. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for promotion and dismissal principals. Do not invent `Approver`. |
| `Asset` / `Identity` / `Vendor` / `ProcessingActivity` / `SubjectSelector` | IR | Inventory + impacted-subject SSOT. Do not invent a parallel org graph. |
| `security_finding` bridge | `weeping-angel-assurance::bridge` | **Consume** as observation input. Keep one-way; do not emit ratings or risk status. |
| Catalog vulnerability evidence | catalog + observations | Consume `canonical_type` facts (`security.vulnerability.present`, `security.secret.exposure`, …) as supporting observations. Do not rewrite catalog TOML. |
| `looks_like_compliance_claim` | `weeping-angel-evidence` | **Extend** needles. Collector collect-path and `EvidenceEnvelope::seal` already call it. |
| Collectors | `weeping-angel-collector` | Observation-only. No `Risk` / `RiskCandidate` / `RiskRating` imports. Shared claim-deny only. |
| risk methodology scoring | types present | Optional **score suggestion** on promotion only. Identify omits it. Never a second matrix. Derived ratings stay Prompt 05 `score_risk` output. |
| risk register | types present | Promotion produces a register `Risk` (`Risk::new` + additive slots). Extra decision data stays on `PromotionRecord`. Duplicate `RiskId` fails `validate` (`duplicate risk id`). |
| Golden `risk.json` | `tests/fixtures/assurance-ir/v1/risk.json` | Must keep decoding. This slice does not change required keys. |
| Applicability | `weeping-angel-assurance::applicability` | Pattern to copy (pure, inventory-sliced, network-free). Do not mix Kleene evaluation with identification. |
| Effectiveness | `weeping-angel-control-test` | **Remains control-test output.** Identification never writes `Effectiveness` or “ISO control failed”. |

Tiny allowed: new typed ids; new IR modules; assurance identify module; claim-deny needles.

Do **not** bump `ASSURANCE_IR_SCHEMA`.

---

## 3. Current behavior (baseline — characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Absence assertions are skip-superseded after target GREEN. Additive tests that still hold (`Risk::new` default JSON, golden `risk.json`, author-empty `AssessmentDefinition.risks`, observations not inserting risks) remain executable.

**HEAD delta vs §3.2 / §3.4:** Prompt 06 rejects duplicate `RiskId`s (`duplicate risk id`), so `p07_b06` (silent collapse) is skip-superseded. Claim-deny now matches `risk accepted` / `ISO control failed`; `p07_b13` is skip-superseded.

### 3.1 No `RiskCandidate` symbol

Product crate sources (`crates/**/src/**/*.rs`) contain none of:

- `RiskCandidate`, `RiskCandidateId`
- `CorrelationKey`, `identify_risk_candidates`, `correlate_candidates`
- `PromotionRecord`, `promote_candidate`
- `DismissalRecord`, `dismiss_candidate`
- `CandidateStatus`

`weeping-angel-assurance-ir` re-exports `risk::{Risk, RiskStatus}` only.

### 3.2 `Risk` is a four-field inventory stub

[`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs):

```text
//! Minimal risk record. Not a risk engine.

RiskStatus = Open | Accepted | Mitigated | Closed   // camelCase JSON

Risk { id: RiskId, title: String, description: String, status: RiskStatus }

Risk::new(id, title, description) → status = Open
```

`AssessmentDefinition.risks: Vec<Risk>` defaults empty. Golden assessment fixture has `"risks": []`. Golden [`tests/fixtures/assurance-ir/v1/risk.json`](../../tests/fixtures/assurance-ir/v1/risk.json) is four keys. IR-019 is the only risk integrity check (implementation → `RiskId`). Duplicate risk ids silently collapsed in the membership set at characterization (HEAD: fail-closed `duplicate risk id`).

### 3.3 Observations never become register rows

- `bridge::from_engine_hit` / `from_semantic_finding` emit `EvidenceType("security_finding")` with string facts. They do not construct `Risk`.
- There is no assurance API that, given observations, mutates `AssessmentDefinition.risks`.
- Empty evidence and populated findings are equivalent from the register’s point of view: risks stay whatever the author listed.

### 3.4 Claim-deny does not cover risk/control verdicts

[`looks_like_compliance_claim`](../../crates/weeping-angel-evidence/src/lib.rs) is true for needles such as `iso 27001 compliant`, `gdpr compliant`, `soc 2 compliant`, `audit passed`, `control test result`, `certification guaranteed`, NIS2/DORA/PCI variants.

It is **false** today for:

- `risk accepted`
- `ISO control failed` / `iso control failed`

Collector collect-path and `EvidenceEnvelope::seal` both call this function. Extending the function is therefore enough for collector + seal **if** identification also refuses those narratives as supporting observations.

### 3.5 No incident / candidate store

There is no IR `Incident` type, no candidate ledger, no promotion principal, and no dismissal snapshot. Identity/vendor/asset/processing-activity records exist as **thin inventories**, not as an identification engine.

### 3.6 risk methodology and register in this tree

At characterization, product types for methodology and the operational register were **absent**. Baseline tests for *this* slice did not require those types to compile. On this HEAD both slices have landed; identification consumes them without forking `Risk` or `score_risk`.

### 3.7 What current tests already lock (must stay green)

- `sdd_compliance_ir_target`: golden `risk.json` decode; IR-019 dangling `RiskId`.
- `sdd_assurance_runtime_target` / ISO / vulnerability catalog: scanner hits are `security_finding`, not framework results; bridge is one-way.
- Governance found-case (already ignored): `Risk::new` JSON lacks `treatment` / `owner` / `residualScore`.

---

## 4. Desired behavior (target)

### 4.1 Product home

| Concern | Home |
| --- | --- |
| `RiskCandidate`, correlation key newtype, candidate status, source lineage types, validation | `weeping-angel-assurance-ir` — **new module(s) beside** `risk.rs`, not inside it |
| `PromotionRecord`, `DismissalRecord`, resurface snapshot | same IR crate |
| `RiskCandidateId` (+ optional `PromotionId` / `DismissalId`) | `id.rs` `typed_id!` |
| Deterministic `identify_risk_candidates` / `correlate_candidates` / `should_resurface` | `weeping-angel-assurance` — new module, **applicability-style** (pure, network-free, no `FrameworkProfile`) |
| `promote_candidate` / `dismiss_candidate` | assurance module calling IR constructors; `promote_candidate` takes `&mut AssessmentDefinition` and inserts `Risk` only on promote |
| Claim-deny needles | `weeping_angel_evidence::looks_like_compliance_claim` |
| Collectors | unchanged except inheriting claim-deny |
| Scoring | risk methodology APIs only |
| Register record | risk register `Risk` only |

No new crate. IR stays free of collector/SDK types. Assurance stays free of LLM clients.

### 4.2 `RiskCandidate` (not a `Risk`)

JSON camelCase. Schema `assurance-ir/v1` on the document (or inherit; do not fork).

```text
RiskCandidate {
  id: RiskCandidateId
  schemaVersion: "assurance-ir/v1"
  status: CandidateStatus
  correlationKey: CorrelationKey
  sourceLineage: Vec<SourceRef>          // collector/run/envelope/observation identities
  scenarioProposal: ScenarioProposal     // title + narrative + scenarioKey
  impactedSubjects: Vec<SubjectRef>      // kind + stable id, sorted
  supportingObservations: Vec<ObservationIdentity>
  confidence: CandidateConfidence        // deterministic band, not a probability
  duplicateCandidateIds: Vec<RiskCandidateId>
  suggestedRiskCategory: SuggestedRiskCategory
  scoreSuggestion?: ScoreSuggestion      // optional; see §4.8
  matchesExistingRiskIds: Vec<RiskId>    // advisory overlap, not auto-link
  resultingRiskId?: RiskId               // set only after promote; identity of *this* candidate unchanged
  firstSeenAt?: DateTime<Utc>
  lastSeenAt?: DateTime<Utc>
  stale: bool
}
```

```text
CandidateStatus =
  Proposed
  | ClusteredDuplicate
  | Promoted
  | Dismissed
  | Stale
  | Resurfaced
```

Invariants:

1. `RiskCandidate` is a distinct type. No `type Risk = RiskCandidate`. No serde alias that decodes a candidate as a register row.
2. Identification never sets `status = Promoted`. Only `promote_candidate` does.
3. Candidate `id` and `correlationKey` are stable across promote/dismiss. Promotion stores `resultingRiskId`; it does **not** rename the candidate to the risk id.
4. `duplicateCandidateIds` lists other candidate ids collapsed into this survivor for the same key in this run (or retained from prior runs). Survivors stay `Proposed` (or `Resurfaced` / `Stale`); duplicates are `ClusteredDuplicate` and are **not** independently promotable while clustered.
5. Empty `supportingObservations` after filtering claim-deny / undeclared types ⇒ that proposal is dropped (no candidate).
6. `matchesExistingRiskIds` is computed by overlap rules (§4.10). It does not insert or mutate `AssessmentDefinition.risks`.

`ScenarioProposal`:

```text
ScenarioProposal {
  scenarioKey: String      // normalized identity used in the correlation key
  title: String
  narrative: String        // explainable; must itself pass claim-deny
}
```

`SubjectRef`: `{ kind: SubjectKind, id: String }` with `id` a well-formed stable id. Prefer inventory ids (`AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`) when the observation’s asset/provenance matches the assessment inventory.

`SourceRef`: provider-neutral pointers (`evidenceType`, optional `envelopeDigest`, optional `collectionRunId`, optional `collectorId`). No GitHub/Jira types.

### 4.3 Observation identity (not envelope digest)

Envelope `canonical_digest` includes provenance `collected_at`, so a re-collection of the same finding would churn correlation and falsely resurface dismissals.

**Law:** clustering, dismissal snapshots, and resurfacing use `ObservationIdentity`, **excluding** collection time and run id.

```text
ObservationIdentity {
  evidenceType: String
  facts: BTreeMap<String, String>   // non-temporal facts only
  narrativeFingerprint: String      // normalize_scenario(narrative) or empty
}

observation_identity_digest = "oi:sha256:" || hex16(canonical_digest(ObservationIdentity))
```

For `security_finding`, facts **include** `rule_id`, `path` / `finding_id`, `category`, `canonical_type` when present. Facts **exclude** timestamps, collection-run ids, and any credential-shaped keys (those already fail seal).

Staleness uses envelope/provenance time (§4.7) and is **orthogonal** to identity.

### 4.4 Correlation (deterministic clustering)

Core runtime must not use embeddings, LLM classification, or probabilistic mixture models. If an AI-assisted adapter is added later, it may only emit a `ScenarioProposal` that:

1. passes deterministic validation (claim-deny, stable ids, non-empty scenario key, subjects well-formed);
2. is clustered with the same `correlation_key` function;
3. still requires human `promote_candidate`.

**Correlation key:**

```text
normalize_scenario(text):
  lowercase ASCII
  collapse whitespace
  drop characters other than [a-z0-9] and space
  trim

subject_key =
  join('\n', sort unique (kind as camelCase + ':' + id))

scenario_key =
  ScenarioProposal.scenarioKey   // already normalized; engine stores normalized form

CorrelationKey = "ck:sha256:" || hex32(canonical_digest({ subject_key, scenario_key }))
```

Two proposals share a key **iff** impacted subject sets (as `SubjectRef`s) and `scenarioKey` are identical. Category and confidence do **not** enter the key (otherwise the same scenario would split on band). Score suggestion does not enter the key.

`correlate_candidates(proposals) -> Vec<RiskCandidate>`:

1. Group by `CorrelationKey`.
2. Survivor = lowest `RiskCandidateId` if ids already assigned; else deterministic id derived from the key (`rc:` + key hex) so re-runs are stable **for the same key**.
3. Union `supportingObservations`, `sourceLineage`, `impactedSubjects`.
4. Confidence recomputed on the union (§4.9).
5. Non-survivors: `status = ClusteredDuplicate`, listed on survivor `duplicateCandidateIds`.
6. Suggested category: if all members agree, keep it; if they disagree, `Other("mixed")` or the first in sorted category order — **document the chosen tie-break in the type’s module docs** and lock it in a target test. Do not pick by model score.

### 4.5 Identification inputs and default mappers

```text
IdentificationContext {
  definition: &AssessmentDefinition     // inventories + existing risks
  observations: &[EvidenceObservation]
  envelopes: &[EvidenceEnvelope]        // collected_at / staleness
  priorCandidates: &[RiskCandidate]
  dismissals: &[DismissalRecord]
  promotions: &[PromotionRecord]
  policy: IdentificationPolicy
  asOf: DateTime<Utc>
}

IdentificationPolicy {
  maxEvidenceAgeSeconds?: u64           // fallback when no EvidenceRequirement.freshness
}

identify_risk_candidates(ctx) -> Vec<RiskCandidate>
```

Consume, when present:

| Input | Use |
| --- | --- |
| `definition.assets` / `identities` / `vendors` / `processing_activities` | Resolve impacted subjects; architecture via `Asset.parent`, `ProcessingActivity.systems` / `processors` |
| `security_finding` observations | Primary mapper (§4.5.1) |
| Catalog vulnerability `canonical_type` facts | Scenario family |
| Existing `definition.risks` | `matchesExistingRiskIds` |
| Identity / data-class / supplier / incident **facts** as observations | Additional mappers **if** evidence types already exist |
| Prior candidates + dismissals + promotions | Dedup, resurface, skip already-promoted keys |

Do **not** invent an `Incident` IR document in this slice. Operational incidents live in [`incident-governance.md`](incident-governance.md) and are not auto-promoted from candidates. If no incident evidence type is present, skip that mapper.

Inventory **alone** (assets sitting in the definition with no mappable observation/fact) yields **no** candidate.

#### 4.5.1 Built-in `security_finding` mapper

For each observation with `evidence_type == "security_finding"` that passes claim-deny:

- Subject: provenance `asset` if the envelope is supplied and that `AssetId` is in `definition.assets`; else a `SubjectRef` from fact `path` **only when** it matches an inventory id; else subject kind `Asset` with id from provenance/path if it is a valid stable id. Unresolvable subjects still produce a candidate at `confidence = Low` if the observation identity is well-formed.
- Default `scenarioKey` = `normalize_scenario(canonical_type || category || "security.finding")`.
- Title/narrative from observation narrative (must pass claim-deny).
- `suggestedRiskCategory` from `canonical_type` (§4.5.3).

#### 4.5.2 One finding → two candidates (mandatory target)

The engine must allow **multiple** `ScenarioProposal`s from one observation when deterministic rules emit distinct `scenarioKey`s.

Target fixture (found case to encode):

- One `security_finding` with `canonical_type = security.vulnerability.present` on asset `asset:payments-api`.
- `definition.processing_activities` lists an activity whose `systems` contains that asset (personal-data / confidentiality scenario) **and** the same asset is a production `Service`.
- Rules emit:
  1. `scenarioKey = "confidentiality exposure via known vulnerability"` (or the spec’s exact normalized form of that phrase);
  2. `scenarioKey = "integrity or availability failure via known vulnerability"`.
- Result: **two** candidates, two correlation keys, the **same** `ObservationIdentity` listed on both. Promoting both yields two `RiskId`s. risk register N:N `finding_refs` (when present) may both cite the same contributor id.

If risk register `FindingRef` exists, promotion copies a stable contributor id derived from `ObservationIdentity` (not from `src/finding.rs`).

#### 4.5.3 Suggested category (not ISO)

```text
SuggestedRiskCategory =
  Confidentiality
  | Integrity
  | Availability
  | Identity
  | Supplier
  | Vulnerability
  | Other(String)
```

No Annex A / clause numbers. Mapper examples: `security.secret.exposure` → Confidentiality; `security.authz.weakness` → Identity; `security.vulnerability.present` → Vulnerability (and may also emit Confidentiality/Integrity proposals per §4.5.2). `Other` is closed by validation: non-empty, not an `iso27001:` / `annex-a` prefix (fail closed).

### 4.6 Promotion and dismissal

```text
PromotionRecord {
  id: PromotionId
  candidateId: RiskCandidateId
  correlationKey: CorrelationKey
  riskId: RiskId
  principal: PrincipalRef
  at: DateTime<Utc>
  rationale: String            // non-empty, passes claim-deny
  methodologyInputs?: …        // risk methodology RiskScoreInput or opaque slot
  methodologyVersion?: …
}

DismissalRecord {
  id: DismissalId
  candidateId: RiskCandidateId
  correlationKey: CorrelationKey
  observationIdentities: BTreeSet<String>   // oi:sha256:… snapshots
  subjectKey: String
  scenarioKey: String
  principal: PrincipalRef
  at: DateTime<Utc>
  rationale: String            // non-empty, passes claim-deny
}
```

```text
promote_candidate(&mut definition, candidate, principal, at, rationale, methodology_inputs?)
  -> Result<(RiskCandidate, Risk, PromotionRecord), IdentificationError>

dismiss_candidate(candidate, principal, at, rationale)
  -> Result<(RiskCandidate, DismissalRecord), IdentificationError>
```

Promotion laws:

1. Fails if `candidate.status` ∈ `{Promoted, ClusteredDuplicate, Dismissed}` unless the candidate is `Resurfaced` (then promotion is allowed as a new decision). `Stale` always fails (§4.7).
2. Fails if rationale empty or claim-deny positive.
3. Fails if principal is `PrincipalRef::Identity(id)` and `id` ∉ `definition.identities`. Team/Role require non-empty strings.
4. Constructs `Risk` **only** via the register surface: `Risk::new(risk_id, title, narrative)` then `scenario` = proposal narrative, `source = Finding` when observations exist, `finding_refs` from `observation_identity_digest`, `asset_ids` from asset-kind subjects, `owner` = principal, `discovered_at` = `at`, optional `methodology_version` from the selected suggestion. Extra decision data (full `ScoreSuggestion`) stays on `PromotionRecord`. Identification does not call `score_risk`.
5. Inserts the `Risk` into the caller-owned assessment `risks` vec (`&mut AssessmentDefinition`). Identification (`identify_risk_candidates`) **never** inserts.
6. Assigned `RiskId` is `risk:` + hex20 of `(candidate id, correlation key, at)` — not uuid-v4 and **not** equal to `RiskCandidateId`.
7. Updates candidate: `status = Promoted`, `resultingRiskId = Some(risk_id)`. `id` / `correlationKey` unchanged.
8. Re-promote of the same candidate id fails closed (already promoted). A resurfaced **new** candidate id (or same id flipped to `Resurfaced` after dismissal) is a new decision.

Dismissal laws:

1. Fails if already `Promoted` (close or treat the **Risk** via risk register/08; do not dismiss a promoted candidate as a substitute for risk acceptance).
2. Records the observation-identity snapshot used for resurfacing.
3. Sets `status = Dismissed`. Retained in `priorCandidates` / dismissal log for audit and dedup.
4. Does **not** create or mutate `Risk`. Does **not** set any risk status to `Accepted`.

### 4.7 Stale evidence

A supporting observation is **stale** at `asOf` when:

```text
age = asOf - collected_at
age > freshness.max_age_seconds
```

Freshness source, first match wins:

1. `EvidenceRequirement` in the assessment whose `evidence_type` matches the observation, if `freshness` is `Some`.
2. Else `IdentificationPolicy.maxEvidenceAgeSeconds`.
3. Else **not stale** (unconfigured freshness ≠ stale). Missing `collected_at` (bare observation without envelope) is **stale** if a freshness limit *is* configured; otherwise treated as fresh for identification but **cannot promote** (promotion requires a dated envelope when policy/requirement freshness is set).

Cluster stale flag: `true` iff **every** supporting observation is stale, **or** (stricter promote gate) promotion requires **at least one non-stale** supporting observation.

Laws:

- `identify` may return candidates with `status = Stale` or `stale = true` (do not hide them).
- `promote_candidate` **fails closed** if the candidate is stale or has zero non-stale supporting observations.
- Re-collection of the **same** `ObservationIdentity` with a new `collected_at` inside the freshness window clears staleness; it does **not** by itself resurface a dismissal (§4.8).

### 4.8 Resurfacing (explicit)

`should_resurface(cluster, dismissal) -> bool` is a pure function.

Let `I` = new cluster’s set of `observation_identity_digest`s; `D` = dismissal snapshot set; `S` / `C` = subject_key / scenario_key.

| Condition | Resurface? |
| --- | --- |
| `I ⊆ D` and `S == dismissal.subjectKey` and `C == dismissal.scenarioKey` | **No** — identical (or weaker) evidence |
| `I` is empty | **No** — no-finding does not resurrect |
| `S` or `C` differs (would be a different correlation key anyway) | N/A — new key is a new candidate, not a resurface of this dismissal |
| `I ⊈ D` (at least one **new** observation identity) and same key | **Yes** |
| Only `collected_at` / run id changed for identities already in `D` | **No** |
| Previously stale identities in `D` become fresh, no new identity | **No** |
| Candidate for this key is already `Promoted` and `I ⊈` the observations known at promotion | **No auto-promote.** Identify may attach new identities onto the promoted candidate’s `supportingObservations` for review; creating another `Risk` requires a **new** candidate with a **distinct** `scenarioKey` **or** an explicit second promotion API that is **out of scope** (risk register `revise` / finding_refs). Target test: extra finding on an already-promoted key does not insert a second `Risk` by itself. |

On resurface:

1. New or updated candidate `status = Resurfaced` (still not a `Risk`).
2. Link to the dismissed candidate (`resurfaces: RiskCandidateId` field **or** retain the same `id` with status flip — **choose same-id status flip** so correlation stays auditable: `id` stable, `status` Dismissed → Resurfaced, dismissal records remain append-only).
3. Still requires `promote_candidate` with a new rationale. Dismissal records are never deleted.

Rejected/dismissed candidates **never** auto-promote.

### 4.9 Confidence (deterministic)

```text
CandidateConfidence = Low | Medium | High
```

No floats. No calibrated probabilities.

| Band | Rule (evaluate in order, first match) |
| --- | --- |
| High | ≥ 2 distinct observation identities **and** every impacted subject id resolves in the assessment inventory |
| Medium | 1 observation identity **and** ≥ 1 subject resolves in inventory |
| Low | otherwise (including unresolved subjects, or only a single weak mapper hit) |

Recompute after clustering (union of observations). Tests lock the table; do not substitute a model score.

### 4.10 Existing risks (overlap, not merge)

`matchesExistingRiskIds` includes a `RiskId` if:

- any `finding_refs` equal a contributor id (`oi:sha256:…`) derived from this cluster’s observation identities; **or**
- `normalize_scenario(risk.scenario || risk.description) == scenarioKey` **and** the risk’s `asset_ids` equal the cluster’s asset-type subject ids; **or**
- four-field stub overlap: empty `asset_ids` and empty `finding_refs` and `normalize_scenario(risk.description) == scenarioKey`.

Overlap never copies `RiskStatus::Accepted` onto a candidate and never skips promotion.

### 4.11 Optional score suggestion

```text
ScoreSuggestion {
  methodologyId?: RiskMethodologyId    // if risk methodology landed
  methodologyVersion?: String          // opaque slot otherwise
  input: …                             // RiskScoreInput if 05 landed, else typed raw wrapper
  derived?: ScoredRisk                 // MUST be absent unless computed by score_risk
}
```

Laws:

1. Collectors must not construct `ScoreSuggestion`. Target tests grep `weeping-angel-collector` for `RiskCandidate`, `ScoreSuggestion`, `promote_candidate`, `RiskRating`, `DerivedRating`.
2. If `derived` is present, risk methodology `score_risk` must reproduce it; mismatch fails validation. No public `RiskRating::High` unit variant (risk methodology law).
3. Identification **omits** score suggestion. Promotion may pass `methodology_inputs` the reviewer selected. Those inputs are recorded on `PromotionRecord`; `methodology_version` may copy onto the `Risk`.
4. Do not hardcode a 5×5 or Low/Medium/High matrix in the identification module.

### 4.12 Claim-deny extension

Extend `looks_like_compliance_claim` so the following are **true** (ASCII case-insensitive, like existing needles):

- `risk accepted`
- `risk is accepted`
- `iso control failed`
- `iso 27001 control failed`
- `iso27001 control failed`

Keep existing needles. Collector and seal automatically inherit.

Additional gates:

- `identify_risk_candidates` drops observations whose narrative fails claim-deny (do not cluster them).
- `ScenarioProposal.narrative`, promotion/dismissal `rationale` fail closed on claim-deny.
- Identification, promotion, bridge, and collectors **must not** emit `Effectiveness`, `ControlTestResult`, `RiskStatus::Accepted`, or ISO clause verdicts.
- Effectiveness remains **only** control-test output (`weeping-angel-control-test`). Target test: a `security_finding` cannot be the sole author of `Effectiveness::Ineffective` or a SoA “failed” row.

GitHub collector mapping files are a collision fence; **do not** retarget GitHub evidence types in this slice. Shared claim-deny is the only allowed collector-adjacent change.

### 4.13 Serialization

- camelCase serde.
- `canonical_digest` unchanged algorithm.
- Empty vecs / None skip-serialize where that matches sibling IR style.
- `BTreeMap` / `BTreeSet` for maps/sets.
- Candidate documents are not required to appear inside `AssessmentDefinition` JSON in this slice (avoids expanding the assessment inventory contract risk register also touches). Store them as their own IR documents or as arguments to the engine. If an optional `AssessmentDefinition` field is tempting, **do not add it here** — collision with ISMS context IR/06.

### 4.14 Errors (deterministic `Display`)

Suggested needles (names flexible):

| Situation | `IdentificationError` Display fragment |
| --- | --- |
| Promote stale | `stale evidence` |
| Promote dismissed / duplicate / already promoted | `not promotable` |
| Promote without principal / empty team/role | `principal` |
| Empty rationale | `rationale` |
| Claim-deny | `compliance claim` |
| Dangling identity principal | `dangling` |
| Second scoring model detected in this module (test grep) | n/a (compile/grep) |

No panic on library paths.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | This HEAD |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/risk_identification.baseline.rs` | `sdd_risk_identification_baseline` | skip-superseded + additive GREEN (`Risk::new` / golden `risk.json` / author-empty `risks` / observations do not insert) |
| Target | `tests/contracts/risk_identification.target.rs` | `sdd_risk_identification_target` | **GREEN** (RI-001…010) |

Protocol (complete):

1. Spec (this file) + ADR 0007.
2. Baseline characterized found case on SHA `6e31bf1a…`.
3. Target encoded RI-001…010 (never `#[ignore]`).
4. Product implemented (IR modules + assurance engine + claim-deny needles).
5. Target GREEN; neighbor `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_compliance_ir_target`, and landed `sdd_risk_methodology_*` / `sdd_risk_register_*` stay GREEN.
6. Absence baseline `#[ignore = "superseded by sdd_risk_identification_target"]`. `p07_b06` superseded because the register now rejects duplicate `RiskId`s (`duplicate risk id`) instead of silently collapsing.
7. Target still GREEN.

Register this path in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` (this spec-first phase).

One regression test per invariant titled `P07: <exact subject>` (and baseline `P07-B…` found cases).

---

## 6. Acceptance criteria (testable)

Target suite must encode at least:

- **RI-001** `P07: N findings collapse to one candidate` — two `security_finding` observations with the same resolved subject and same `scenarioKey` produce exactly one `Proposed` survivor; the other id is `ClusteredDuplicate` on `duplicateCandidateIds`. `AssessmentDefinition.risks` remains unchanged until promote.
- **RI-002** `P07: one finding contributes to two distinct candidates` — fixture in §4.5.2 yields two correlation keys and two candidates sharing one `ObservationIdentity`. Promoting both creates two distinct `RiskId`s. Candidate ids are not reused as risk ids.
- **RI-003** `P07: candidate promotion is explicit` — `promote_candidate` requires `PrincipalRef`, non-empty rationale, timestamp; writes `PromotionRecord`; constructs `Risk` via risk register record when present else `Risk::new`; sets `resultingRiskId`; candidate `id` + `correlationKey` unchanged. `identify_risk_candidates` never inserts into `definition.risks`.
- **RI-004** `P07: dismissed candidates do not auto-promote and follow resurfacing rules` — after `dismiss_candidate`, re-identify on the same observation identities leaves status `Dismissed` and creates no `Risk`. A **new** observation identity on the same key sets `Resurfaced` and still requires promote. Clock-only / `collected_at`-only refresh does not resurface.
- **RI-005** `P07: stale evidence cannot promote` — supporting observations older than configured freshness identify as stale; `promote_candidate` fails; replacing with a fresh envelope of the same identity allows promote once non-stale.
- **RI-006** `P07: no-finding yields no candidate` — empty observation list (and inventory-only definition) ⇒ `identify_risk_candidates` returns `[]`.
- **RI-007** `P07: scanners cannot declare risk accepted or ISO control failed` — `looks_like_compliance_claim` is true for those phrases; collector collect-path and `EvidenceEnvelope::seal` reject them; identify drops such narratives; no path sets `RiskStatus::Accepted` or control-test `Effectiveness` from a scanner hit.
- **RI-008** `P07: RiskCandidate is not Risk` — types are distinct; no auto `From`; promoted risk lives in `definition.risks`; candidate documents do not decode as `Risk`.
- **RI-009** `P07: score suggestion is optional and validated` — omit is valid; if derived rating present, it matches risk methodology `score_risk` (or the typed slot is raw-only when 05 is absent). Identification module has no hardcoded 5×5 / `RiskRating::High`.
- **RI-010** Dual-suite names `sdd_risk_identification_baseline` / `sdd_risk_identification_target` are listed in root `Cargo.toml`. Golden `risk.json` still decodes. Neighbor target suites in §header stay GREEN.

Baseline suite encoded the found case in §3 (no `RiskCandidate` symbol; observations do not insert `definition.risks`; empty evidence has no identify API / no candidates; `looks_like_compliance_claim("risk accepted")` and `looks_like_compliance_claim("ISO control failed")` are false). Those absence tests are skip-superseded. Additive characterization that still holds remains executable.

---

## 7. Out of scope

- LLM client, embedding index, or probabilistic clusterer in core runtime.
- Threat-intelligence service or CVE enrichment network calls.
- UI review queue / workbench.
- risk treatment treatment engine (`Mitigate`/`Accept`/`Avoid`/`Transfer` plans, immutable acceptance as treatment).
- residual risk residual scoring / control-effectiveness→residual functions.
- Catalog TOML rewrites and ISO pack remaps.
- GitHub collector evidence-type mapping (except shared claim-deny needles).
- Inventing a second `Risk` type or expanding `Risk` into a candidate.
- A second scoring model / hardcoded 5×5.
- Moving scanner `Finding` (`src/finding.rs`) into IR.
- `Incident` IR document, ISMS context, scope engine, SoA rewrite.
- Auto-advancing reviews, ticketing, auditor portal.
- Bumping `assurance-ir/v1`.
- Adding `AssessmentDefinition.candidates` (inventory collision).

---

## 8. Risks

- Parallel risk methodology and register landing: forking `Risk` or `score_risk` would split the ISMS graph. Mitigation: consume those specs; type-gate; extra fields on promotion/candidate only.
- Using envelope digests for dismissal snapshots would resurface on every re-collection. Mitigation: `ObservationIdentity` excludes `collected_at`.
- Over-clustering (all vulns on one host → one risk) hides distinct scenarios. Mitigation: scenario key is part of the correlation key; RI-002 locks the split.
- Under-clustering (path typo ⇒ duplicate candidates). Mitigation: normalize + inventory subject ids preferred over raw paths.
- Claim-deny false positives on legitimate narratives (“user accepted the TOS”). Mitigation: needles are the listed phrases, not the bare word `accepted`.
- Promotion inserting risks without IR-019-compatible ids. Mitigation: `RiskId` via `typed_id!`; validate assessment after insert.
- Stale policy defaulting to “never stale” could promote ancient evidence. Mitigation: when freshness is configured, missing timestamps cannot promote.
- AI adapter later bypassing promote. Mitigation: adapter output is a proposal; promote remains the only insert path; tests grep for identify→risks mutation.

---

## 9. Landed files

- `crates/weeping-angel-assurance-ir/src/risk_candidate.rs`
- `crates/weeping-angel-assurance-ir/src/risk_promotion.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (`RiskCandidateId`, `PromotionId`, `DismissalId`)
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports
- `crates/weeping-angel-assurance/src/risk_identification/mod.rs`
- `crates/weeping-angel-evidence/src/lib.rs` claim-deny needles
- `tests/contracts/risk_identification.{baseline,target}.rs` + root `Cargo.toml` `[[test]]` rows
- [`docs/adr/0007-risk-identification-candidate-correlation.md`](../adr/0007-risk-identification-candidate-correlation.md) Accepted

---

## 10. Definition of done

The system can continuously surface explainable candidate risks from existing evidence while preserving the management decision boundary: `RiskCandidate != Risk`; promotion is explicit and auditable; dismissal is retained; correlation is deterministic; scanners cannot declare `risk accepted` or `ISO control failed`.

Dual-suite SDD protocol is mandatory: spec first (this document), baseline GREEN on current code, target RED on current code, implement, docs+ADR, iterate until target GREEN, prove baseline fails or is additive-documented, supersede baseline, target still GREEN.
