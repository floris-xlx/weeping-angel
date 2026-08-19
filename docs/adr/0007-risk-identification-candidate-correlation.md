# ADR 0007 — Risk identification via candidates and deterministic correlation

<!-- weeping-angel-adr-meta
id = "0007"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_risk_identification_target` GREEN (RI-001…010) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine. Does **not** supercede risk methodology scoring ownership, risk register `Risk` schema, IR-019, collector blindness, or Kleene applicability. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md); consumes [0005-risk-methodology](0005-risk-methodology.md) and [0005-operational-risk-register](0005-operational-risk-register.md) |
| Spec | [`docs/specs/risk-identification.md`](../specs/risk-identification.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_risk_identification_target` GREEN; `sdd_risk_identification_baseline` skip-superseded (additive `Risk::new` / golden `risk.json` / author-empty `risks` tests remain executable) |

> Filename **`0007-*`**. Methodology/register drafts already use `0005-*`; treatment uses `0006-*`. Do **not** steal those numbers. Cite this decision by **path**.

## Context

On SHA `6e31bf1a…`:

1. `Risk` was `{ id, title, description, status ∈ Open|Accepted|Mitigated|Closed }` — *“Minimal risk record. Not a risk engine.”*
2. `AssessmentDefinition.risks` was an author-supplied inventory. IR-019 only checked `ControlImplementation.risk_ids`; duplicate `RiskId`s silently collapsed into that membership set.
3. Scanner bridge emitted one-way `security_finding` observations. Nothing identified risks from them.
4. There was no `RiskCandidate`, correlation key, promotion record, or dismissal record.
5. `looks_like_compliance_claim` did not reject `risk accepted` or `ISO control failed`.

Operational ISMS v1 risk identification must automate **candidate discovery** without allowing machine observations to become organizationally accepted risks without review.

Questions this decision answers:

1. Is a candidate the same type as a register `Risk`?
2. Where do types vs identify/correlate live?
3. How are identical subjects/scenarios clustered without a probabilistic model?
4. What is recorded on promotion and dismissal, and when may a rejected candidate resurface?
5. How can one finding become two risks, and many findings one candidate?
6. How are risk methodology scoring and risk register `Risk` consumed without a second model?
7. How do scanners get denied `risk accepted` / `ISO control failed`?

## Decision

Accepted and landed. Field-level law is [`docs/specs/risk-identification.md`](../specs/risk-identification.md). Schema stays `assurance-ir/v1`.

### 1. `RiskCandidate != Risk`; promotion is the only insert

Identification emits `RiskCandidate` (`weeping-angel-assurance-ir::risk_candidate`). The register remains the Prompt 06 `Risk` record. There is no `From<RiskCandidate> for Risk`. `identify_risk_candidates` never pushes into `AssessmentDefinition.risks`.

`promote_candidate` takes `&mut AssessmentDefinition`, constructs `Risk` via `Risk::new` plus register slots (`scenario`, `source = Finding` when observations exist, `finding_refs` from `ObservationIdentity` digests, `asset_ids`, `owner`, `discovered_at`, optional `methodology_version`), then inserts that row. Candidate `id` / `correlationKey` stay unchanged; `resultingRiskId` is a distinct `risk:` id.

Incorrect: expanding `Risk` into a proposal; a second `RiskV2`; auto-accepting scanner hits as `RiskStatus::Accepted`.

### 2. Types in IR; engine in assurance (applicability-style)

| Concern | Home |
| --- | --- |
| `RiskCandidate`, `CorrelationKey`, `ObservationIdentity`, status/confidence/category | `weeping-angel-assurance-ir::risk_candidate` (beside `risk.rs`) |
| `PromotionRecord`, `DismissalRecord` | `weeping-angel-assurance-ir::risk_promotion` |
| `RiskCandidateId`, `PromotionId`, `DismissalId` | `id.rs` `typed_id!` |
| `identify_risk_candidates` / `correlate_candidates` / `promote_candidate` / `dismiss_candidate` / `should_resurface` | `weeping-angel-assurance::risk_identification` |

Engine is pure: no network, no `FrameworkProfile`, no ISO annex branches, no LLM client. An AI adapter, if added later, may only emit a `ScenarioProposal` that passes deterministic validation and still requires `promote_candidate`.

Landed `IdentificationContext` is inventories + observations/envelopes + prior candidates + dismissal/promotion logs + `IdentificationPolicy` + `asOf`. It does **not** take a methodology document. Identify omits `scoreSuggestion`. Promotion records the reviewer-selected `ScoreSuggestion` on `PromotionRecord` (optional). Identification never hardcodes a 5×5 or `RiskRating::High`; derived ratings, if present on a suggestion, remain Prompt 05 `score_risk` output.

### 3. Correlation is identical subject + scenario keys

```text
normalize_scenario(text) → lowercase ASCII, collapse whitespace, keep [a-z0-9] and space
subject_key = join('\n', sort unique (SubjectKind camelCase + ':' + id))
CorrelationKey = "ck:sha256:" || hex32(canonical_digest({ subject_key, scenario_key }))
survivor id = "rc:" || key hex
duplicate id = "rc:dup:" || hex16(digest(key, observation identities))
```

Envelope timestamps are **not** part of `ObservationIdentity` (facts exclude temporal/credential keys; `security_finding` keeps `rule_id` / `path` / `finding_id` / `category` / `canonical_type`). Re-collection must not churn clusters or falsely resurface dismissals.

```text
observation_identity_digest = "oi:sha256:" || hex16(canonical_digest(ObservationIdentity))
```

Category, confidence, and score suggestion are not part of the key. Clustered members that disagree on `SuggestedRiskCategory` use `Other("mixed")` (not a model score). N findings with the same key collapse to one `Proposed` survivor; others are `ClusteredDuplicate` listed on `duplicateCandidateIds` and are not independently promotable.

One `security_finding` with `canonical_type = security.vulnerability.present` on a production `Service` that also appears in a processing-activity `systems` list emits two scenario keys:

- `confidentiality exposure via known vulnerability`
- `integrity or availability failure via known vulnerability`

Promoting both yields two `RiskId`s. Inventory alone (no mappable observation) yields no candidate. Built-in mapper is `security_finding` only; no `Incident` document is invented here.

### 4. Promotion and dismissal are auditable decisions

Promotion records `PrincipalRef` (reuse implementation type), time, non-empty claim-deny-clean rationale, selected methodology inputs, and the resulting `RiskId` (`risk:` + hex20 of candidate id, correlation key, and `at`). Identity principals must exist in `definition.identities`. Promotable statuses are `Proposed` and `Resurfaced` only. `Stale` / `stale = true` fails closed (`stale evidence`). Already `Promoted` / `ClusteredDuplicate` / `Dismissed` fail (`not promotable`).

Dismissal is retained (`DismissalRecord` snapshots `oi:sha256:…` identities). Resurface uses **same candidate id**, status `Dismissed` → `Resurfaced`, when a **new** `ObservationIdentity` appears on the same subject+scenario key. Same identities, clock-only refresh, empty evidence, or a previously promoted key do not resurface and never auto-promote. A new `Risk` on a promoted key requires a distinct scenario key or an out-of-scope register revise.

Cluster `stale` is true iff every supporting observation is stale against `EvidenceRequirement.freshness` for `security_finding` (else `IdentificationPolicy.maxEvidenceAgeSeconds`). Unconfigured freshness is not stale. Missing `collected_at` is stale when a limit is configured.

### 5. Consume 05/06; optional score suggestion only

Prompt 05 types (`RiskScoreInput`, `ScoredRisk`, `score_risk`) are present. Identification does not call `score_risk` and does not author derived ratings. Collectors must not construct `ScoreSuggestion` / `RiskCandidate` / `promote_candidate`.

Prompt 06 operational fields are present. Promotion fills those slots as in §1. Overlap (`matchesExistingRiskIds`) is advisory: `finding_refs` vs observation-identity contributor ids, or normalized `scenario`/`description` plus equal asset-id sets. Overlap never copies `RiskStatus::Accepted` onto a candidate.

`validate_assessment_ir` now fail-closes duplicate `RiskId` (`duplicate risk id`). Characterization baseline `p07_b06` (silent collapse) is skip-superseded for that reason; IR-019 dangling `risk_ids` remains.

### 6. Shared claim-deny, not a collector rewrite

`looks_like_compliance_claim` additionally matches (ASCII case-insensitive):

- `risk accepted`
- `risk is accepted`
- `iso control failed`
- `iso 27001 control failed`
- `iso27001 control failed`

Collector collect-path and evidence seal already call it. Identify drops matching narratives. Promotion/dismissal rationale and scenario narratives fail closed (`compliance claim`). Effectiveness remains control-test output. Do not retarget GitHub collector mappings in this slice.

## Consequences

- Reviewers can see *why* a candidate exists (lineage, subjects, observations, key) without the system claiming the risk is accepted.
- Risk treatment and residual risk still have a clean `RiskId` boundary; they do not consume candidates.
- Dual-suite `sdd_risk_identification_{baseline,target}` is registered in root `Cargo.toml`; contracts are not auto-discovered.
- After target GREEN, absence baseline tests are `#[ignore = "superseded by sdd_risk_identification_target"]`. Additive characterization (`Risk::new` default JSON, golden `risk.json`, author-empty `AssessmentDefinition.risks`, observations not inserting risks) remains executable.

## Non-goals

LLM client, embedding/probabilistic clusterer, threat intel, UI queue, treatment engine, residual scoring, catalog TOML / ISO pack rewrites, GitHub evidence-type remapping (except shared claim-deny), second Risk type, second scoring model, `Incident` IR, `AssessmentDefinition.candidates`.

## Related

- Spec: [`docs/specs/risk-identification.md`](../specs/risk-identification.md)
- Risk methodology: [`docs/specs/risk-methodology.md`](../specs/risk-methodology.md), [ADR 0005 methodology](0005-risk-methodology.md)
- Risk register: [`docs/specs/risk-register.md`](../specs/risk-register.md), [ADR 0005 register](0005-operational-risk-register.md)
- Typed evidence (facts ≠ conclusions): [ADR 0003 typed evidence](0003-typed-evidence-canonical-serialization.md)
- Docs layout: [ADR 0004](0004-documentation-architecture.md)
