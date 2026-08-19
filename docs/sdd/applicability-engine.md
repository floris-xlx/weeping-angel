# SDD: Organization Context and Applicability Engine

| Field | Value |
| --- | --- |
| Status | **Specified** — product implementation must not start until dual-suite registration and this spec are the SSOT |
| Program | Canonical Assurance Catalog v1 — Prompt 10 |
| Source prompt | [`docs/prompts/canonical-assurance-v1/10-applicability-engine.md`](../prompts/canonical-assurance-v1/10-applicability-engine.md) |
| Slice | Deterministic organization-context + Kleene three-state evaluator over existing IR `ApplicabilityRule` / `ApplicabilityPredicate`; applicability snapshot for lineage; population scope constraint |
| Dual-suite (register at implement) | `sdd_applicability_engine_baseline` · `sdd_applicability_engine_target` |
| ADR | Draft [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md) — finalize when the evaluator + snapshot types land |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update in implement, not this spec-only phase |
| Spine (still law) | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Catalog infra | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Typed evidence | [`docs/sdd/typed-evidence.md`](typed-evidence.md) |
| Population (consumed) | [`docs/sdd/population-runtime.md`](population-runtime.md), ADR [`0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md) |
| Lineage (neighbor, do not implement) | [`docs/sdd/assessment-lineage.md`](assessment-lineage.md) — persist/explain/ledger stay Prompt 11 |
| GitHub collector (collision fence) | [`docs/sdd/github-collector.md`](github-collector.md) — do not touch |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Catalog schema | `weeping-angel/canonical-catalog/v1` |
| Pack schema | `weeping-angel/framework-pack/v1` |
| Snapshot schema (new, not IR) | `weeping-angel/applicability-snapshot/v1` |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for Prompt 10. It owns **organization-context construction**, **three-state rule evaluation**, **selected-subject / exclusion traces**, and the **in-memory `ApplicabilitySnapshot`** that Prompt 11 will persist. It does **not** own catalog TOML, ISO pack `applicability.toml` as an evaluator, provider APIs, explain/ledger, or a generic ontology engine.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Applicability is a **generic, provider-blind, framework-blind** decision over IR rules + assessment scope. Framework projections consume the result; they do not implement a second evaluator.

---

## 1. Problem / user-visible goal

The IR already has a rich `ApplicabilityRule` / `ApplicabilityPredicate` tree on every `Requirement` and `Control`. Nothing evaluates it against an organization.

Today:

- `ApplicabilityRule::statically_applicable()` is a **compile-time** `Option<bool>` fold: `Always`/`Never` and boolean combinations resolve; every `Predicate` is `None`.
- Framework compile `resolve_applicability` keeps a requirement unless that fold is `Some(false)`. Unknown predicates therefore stay in the compiled set, but **no rationale, no facts, no selected subjects**.
- SoA (`project_soa`) rereads pack `applicability.toml` booleans from disk. That is a **second**, ISO-shaped applicability path — not the IR rule tree.
- Control-test populations resolve subjects for **coverage**, not for **whether the control applies**.
- There is no `ApplicabilitySnapshot`. Prompt 11 has reserved the persist shape and currently characterizes Prompt 10 as **absent**.

Unknown facts are accidentally safe only because predicates never become `false`. They are not operational: a reviewer cannot ask why control X applied, why Y did not, which fact was unknown, or which exclusion removed subject Z.

**User-visible goal:** given an assessment definition (IR inventories + `AssessmentScope`) and a rule tree, decide deterministically:

```text
Applicable
NotApplicable
ManualDeterminationRequired
```

with an ordered rationale, the predicates/facts that caused the result, unknown facts named rather than coerced to false, and the selected subject set (plus exclusion reasons) that downstream population evaluation must honor.

Example the engine must distinguish:

```text
ProcessesPersonalData(true) + personal-data fact unknown
  → ManualDeterminationRequired   (never NotApplicable)

UsesCloudProvider(true) + authoritative assets with no CloudAccount/CloudResource
  → NotApplicable

Always + zero selected subjects after exclusions
  → Applicable, selected_subjects = []
    (zero subjects ≠ NotApplicable)
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `e430980c0d27a8138a153d49b62ddf3c57827891`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ApplicabilityRule` / `ApplicabilityPredicate` | `weeping-angel-assurance-ir::applicability` | **SSOT rule AST.** Do not add variants unless a listed predicate cannot be expressed. Keep `statically_applicable` as the no-context fold. **IR stays declarative** — no platform fact evaluator in the IR crate. |
| `AssessmentDefinition` inventories | `assessment.rs` | Context source: `assets`, `identities`, `vendors`, `processing_activities`, `risks`, `scope`. |
| `AssessmentScope` / `ScopeExclusion` | IR | `{ organizations, subjects, exclusions }`. Do not collapse with facade `AssessmentScope` (`BTreeSet<AssetId>` collector allow-set). |
| `Asset` / `Identity` / `Vendor` / `ProcessingActivity` / `Risk` | IR | **The inventory.** Do not invent a parallel org graph. Thin records stay thin; use `Asset.tags` and explicit tri-state facts for attributes IR does not store. |
| `SubjectSelector` | IR | Population + exclusion SSOT. Reuse Prompt 03 resolution semantics. |
| `Control.subjects` / `PlannedControlTest.subjects` | IR | Constrain selected subjects. Tiny allowed: public `Control::subjects()` getter (field exists, no accessor today). |
| `Population` / `resolve_population` / `EvidenceSet::set_population` | control-test | **Consume.** Applicability selected-scope is injected as an explicit population; do not fork a second resolver. |
| `resolve_applicability` | `weeping-angel-framework` | May **consume** generic decisions (drop only `NotApplicable`). Must not grow ISO/provider branches. Without a context, keep today’s `statically_applicable != Some(false)` filter. |
| `project_soa` / `frameworks/iso-27001/2022/applicability.toml` | assurance + pack | **Not a second evaluator.** Do not rewrite pack TOML or teach the engine ISO booleans. |
| `ApplicabilitySnapshot` persist shape | Prompt 11 spec §4.3 | **Fill this shape** from the generic evaluator. Do **not** implement ledger persist, `ControlExplanation`, or `assurance explain`. |
| Catalog TOML | `catalog/canonical/v1/` | Do not edit. |
| Collectors / `GITHUB_EVIDENCE_TYPES` | collector crate | **Collision fence.** Do not touch. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_applicability_engine_*` next to existing `sdd_*`. Do not rewrite github/lineage/population suites. |

Tiny allowed adjustments: new module(s) under `weeping-angel-assurance` and optionally a thin re-export/helper in `weeping-angel-control-test`; optional `Control::subjects()`; optional compile hook that calls the generic evaluator when a context is supplied; serde-default fields on the new snapshot types.

Do **not** redesign `ApplicabilityRule`, catalog schema, IR `AssessmentDefinition` core fields, collectors, or ISO pack IDs.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `e430980c0d27a8138a153d49b62ddf3c57827891`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 IR is declarative

[`crates/weeping-angel-assurance-ir/src/applicability.rs`](../../crates/weeping-angel-assurance-ir/src/applicability.rs) module docs: *“Declarative applicability. The IR does not evaluate platform facts.”*

```text
ApplicabilityRule = Always | Never | All | Any | Not | Predicate
ApplicabilityPredicate =
  AssetType(String)
  | OrganizationAttribute { key, value }
  | Jurisdiction(String)
  | ProcessingCategory(String)
  | Technology(String)
  | DataCategory(String)
  | RiskLevel(String)
  | HasVendor(bool)
  | HasEmployees(bool)
  | UsesCloudProvider(bool)
  | ProcessesPersonalData(bool)
```

Helpers: `ApplicabilityRule::jurisdiction`, `processes_personal_data`. Round-trip + digest already covered by `ir_010` in `sdd_compliance_ir_target`.

### 3.2 `statically_applicable` is `Option<bool>` without facts

| Tree | Result today |
| --- | --- |
| `Always` | `Some(true)` |
| `Never` | `Some(false)` |
| `Predicate(_)` | `None` |
| `All` | `Some(false)` if any child is `Some(false)`; `Some(true)` iff every child is `Some(true)`; else `None` |
| `Any` | `Some(true)` if any child is `Some(true)`; `Some(false)` iff every child is `Some(false)`; else `None` |
| `Not(inner)` | `inner.map(!)` — `Not(None)` stays `None` |

Empty `All` → `Some(true)`. Empty `Any` → `Some(false)`. This fold is **correct Kleene logic over static constants** and must remain. It does **not** consult inventories, scope, or tags.

### 3.3 Compile keeps unknown predicates

[`crates/weeping-angel-framework/src/lib.rs`](../../crates/weeping-angel-framework/src/lib.rs) `resolve_applicability`:

```text
filter(|req| req.applicability().statically_applicable() != Some(false))
```

`FrameworkTarget` is unused (`let _ = target`). Controls are **not** filtered by `Control.applicability`. There is no rationale, no three-state decision, no selected subjects.

### 3.4 SoA is a boolean pack projection

[`crates/weeping-angel-assurance/src/soa.rs`](../../crates/weeping-angel-assurance/src/soa.rs) `project_soa` reads `applicability.toml` `[[entry]]` and copies `applicable: bool`. `SoaEntry.applicable` is `bool`, not a three-state. ISO pack entries are all `applicable = true`.

### 3.5 No evaluator / snapshot module

Product crates have **no**:

- `evaluate_applicability` / organization-context builder
- `ApplicabilityDecision` / `ManualDeterminationRequired`
- `struct ApplicabilitySnapshot`
- module path `applicability` under `weeping-angel-assurance` or `weeping-angel-control-test` that evaluates predicates against facts

Prompt 11 baseline (`applicability_rule_is_static_only_prompt_10_absent`, `product_crates_lack_explanation_and_snapshot_types`) encodes this absence. That is **their** characterization of pre-Prompt-10 HEAD, not a ban on landing these types in this slice.

### 3.6 Inventories exist; nothing reads them for applicability

`AssessmentDefinition` already carries `scope`, `assets`, `identities`, `vendors`, `risks`, `processing_activities`.

- `Asset` = `{ id, kind, name, parent?, tags }` — kinds include `CloudAccount`, `CloudResource`, `Organization`, …
- `Identity` = `{ id, kind, displayName? }` — kinds include `User`
- `Vendor` = `{ id, name }`
- `ProcessingActivity` = `{ id, name, systems, processors }` — **no** category / personal-data field
- `Risk` = `{ id, title, description, status }` — **no** level field
- `AssessmentScope` = `{ organizations, subjects, exclusions }`
- `Control.subjects: Vec<SubjectSelector>` is stored/serialized; **no public getter**
- Facade `AssessmentScope` is an asset allow-set for collectors — different type

Population runtime (Prompt 03) resolves subjects **inside control-test** from explicit `Population`, selector ids, or inventory evidence. It does **not** walk IR `AssessmentDefinition` inventories. Callers inject via `EvidenceSet::set_population`.

### 3.7 Effectiveness `NotApplicable` is unrelated

`weeping-angel-control-test::Effectiveness::NotApplicable` exists as a test outcome. Nothing maps IR applicability onto it. Authoritative empty populations evaluate to `InsufficientEvidence` (never `Effective`); they do **not** auto-mark the control `NotApplicable`.

---

## 4. Desired behavior

### 4.1 Product home

New **applicability engine** module owned by `weeping-angel-assurance` (preferred split):

```text
crates/weeping-angel-assurance/src/applicability/
  mod.rs          # re-exports
  context.rs      # ApplicabilityContext builder (derived view)
  evaluator.rs    # Kleene evaluate_applicability
  snapshot.rs     # ApplicabilitySnapshot + decisions + digest
```

A single file `applicability.rs` is acceptable if it stays readable.

`weeping-angel-control-test` may:

- accept a selected-subject list / `Population` already produced by the engine;
- expose a tiny helper that intersects `SubjectSelector`s using existing population code.

It must **not** become an org-fact engine and must **not** grow `TestExpr` arms for `ApplicabilityPredicate`.

`weeping-angel-assurance-ir` stays declarative. Do not add `evaluate` to `ApplicabilityRule`. Do not add a competing inventory type.

Network I/O is forbidden in the evaluator (same law as control-test).

### 4.2 Three-state decision

Public enum (names may be camelCase in JSON):

```text
ApplicabilityDecision {
  Applicable,
  NotApplicable,
  ManualDeterminationRequired,
}
```

Map from Kleene `FactValue`:

| FactValue | Decision |
| --- | --- |
| `True` | `Applicable` |
| `False` | `NotApplicable` |
| `Unknown` | `ManualDeterminationRequired` |

`Unresolved` may be a serde alias of `ManualDeterminationRequired` if Prompt 12/SoA language prefers it; the stored variant name in this slice is `ManualDeterminationRequired`.

### 4.3 Kleene semantics (normative)

Evaluate `ApplicabilityRule` as `FactValue`:

| Node | Rule |
| --- | --- |
| `Always` | `True` |
| `Never` | `False` |
| `Predicate(p)` | `eval_predicate(p, context)` — never coerce `Unknown` to `False` |
| `All(rs)` | `False` if any child is `False`; `True` iff every child is `True`; else `Unknown`. Empty `All` = `True`. |
| `Any(rs)` | `True` if any child is `True`; `False` iff every child is `False`; else `Unknown`. Empty `Any` = `False`. |
| `Not(inner)` | `True`↔`False` swap; **`Not(Unknown) = Unknown`** |

This matches `statically_applicable` when every predicate is treated as `Unknown`. After this slice, predicates become `True`/`False` only when the context **knows**.

**Unknown-as-not-false (normative):** if the system does not know whether personal data is processed, `ProcessesPersonalData(true)` is `Unknown` → `ManualDeterminationRequired`. It is **never** `NotApplicable` solely because the fact is missing.

### 4.4 ApplicabilityContext is a derived view, not a second inventory

```text
ApplicabilityContext {
  assessment_id,
  scope: IR AssessmentScope,          # copy of definition.scope (after any caller overlay)
  organizations: [org id…],           # from scope.organizations
  assets, identities, vendors,        # in-scope slices of definition inventories
  processing_activities, risks,
  completeness: {                     # per family
    assets, identities, vendors,
    processing_activities, risks,
    organization_attributes,
    jurisdictions,
    technologies,
    data_categories,
    processing_categories,
    personal_data,
    cloud_usage,
    employees,
    risk_level,
  },                                  # Authoritative | Partial | Unknown
  facts: ordered explicit FactValue   # optional overrides / attributes IR cannot store
}
```

**Builder order** (deterministic):

1. Start from `AssessmentDefinition` inventories + `definition.scope`.
2. Restrict members by `scope.subjects` (`SubjectSelector` include) when non-empty.
3. Remove members matching `scope.exclusions[].subjects`. Record each removal as `{ subject_id, exclusion_index, rationale }`.
4. Apply caller-supplied explicit facts (`KnownTrue` / `KnownFalse` / omit = unknown). Explicit facts **win** over inferred presence.
5. Default completeness: a family with **no records and no explicit completeness** is `Unknown`, not authoritative-empty. A caller (or fixture) may mark a family `Authoritative`.
6. Do not call collectors. Do not read pack `applicability.toml` to populate facts.

Inferred presence (only when no explicit fact for that predicate family):

| Family | True when | False when | Else |
| --- | --- | --- | --- |
| vendors | ≥1 in-scope vendor | vendor completeness `Authoritative` and 0 vendors | `Unknown` |
| employees | ≥1 in-scope `IdentityKind::User` (or `SubjectKind::User`) | identity completeness `Authoritative` and 0 users | `Unknown` |
| cloud | ≥1 in-scope `AssetKind::CloudAccount` or `CloudResource` | asset completeness `Authoritative` and 0 such assets | `Unknown` |
| personal data | explicit fact or a processing-activity / org tag `personalData=true` / `processesPersonalData=true` | explicit `false`, or personal-data completeness `Authoritative` and no supporting tag/activity | **empty `processing_activities` alone is Unknown** |
| asset type `T` | ≥1 in-scope asset whose `AssetKind` matches `T` (parse like `SubjectKind::parse_name`: alnum, case-insensitive) | asset completeness `Authoritative` and none match | `Unknown` |
| organization attribute `{k,v}` | an `AssetKind::Organization` (or scope org id) has `tags[k] == v` | org-attribute completeness `Authoritative` and none match | `Unknown` |
| jurisdiction `C` | org/asset tag `jurisdiction` / `jurisdictionCode` equals `C` (case-insensitive ISO-ish string), or explicit fact | jurisdiction completeness `Authoritative` and `C` absent | `Unknown` |
| processing category / technology / data category | matching tag on in-scope processing activity or asset (`processingCategory`, `technology`, `dataCategory`, or the predicate’s key) | that family’s completeness `Authoritative` and no match | `Unknown` |
| risk level `L` | explicit fact, or a `Risk` title/tag/extension carries `level=L` if present | risk-level completeness `Authoritative` and no match | `Unknown` (IR `Risk` has **no** level field — do not invent one) |

Boolean predicates `HasVendor(expected)`, `HasEmployees(expected)`, `UsesCloudProvider(expected)`, `ProcessesPersonalData(expected)`:

- let `presence = eval_presence(family)` ∈ {T, F, U}
- result is `True` iff `presence` is known and equals `expected`; `False` iff known and differs; `Unknown` if `presence` is `Unknown`.

`Not(HasVendor(true))` with unknown vendors stays `Unknown`.

### 4.5 Outcome + rationale

```text
ApplicabilityOutcome {
  decision: ApplicabilityDecision,
  rationale: [RationaleEntry…],     # deterministic order
  predicates: [PredicateTrace…],    # each leaf + FactValue + source
  unknown_facts: [UnknownFact…],    # named; empty iff no Unknown leaves contributed
  selected_subjects: [id…],         # lex-sorted unique
  excluded_subjects: [{ id, reason }],
}
```

Rationale / predicate traces are ordered by:

1. preorder walk of the rule tree (All/Any children in vec order);
2. then lexicographic `unknown_facts` / `excluded_subjects` by id;
3. `selected_subjects` sorted lexicographically (same law as `Population.subject_ids`).

Same `(rule, context)` always yields the same JSON (ignore wall-clock). Digest the snapshot with existing `canonical_digest` / `typed_canonical_digest`.

Every outcome must be able to answer:

```text
Why was control X applicable?
Why was control Y not applicable?
Which fact was unknown?
Which exclusion removed subject Z?
```

### 4.6 Scope, populations, zero subjects

After the rule decision:

1. Start from in-scope inventory ids of the kinds named by `Control.subjects` / `Requirement` (if a requirement has no subjects, selected subjects may be empty even when `Applicable`).
2. Intersect with `Control.subjects` selectors (IR `SubjectSelector` + Prompt 03 include/exclude/tag rules).
3. Subtract assessment `exclusions` (already applied in context) and record reasons.
4. The selected set is attached to the decision and **handed to population evaluation** via `EvidenceSet::set_population` (completeness = context family completeness for that kind).

**Zero selected subjects does not change the rule decision** unless a predicate in the tree is false. `Always` + empty inventory → `Applicable` + `selected_subjects = []`. Downstream tests remain fail-closed (Prompt 03: empty authoritative population is never `Effective`).

Do **not** map `selected_subjects.is_empty()` → `NotApplicable`.

When the control decision **is** `NotApplicable`, callers may skip tests or record `Effectiveness::NotApplicable`. When it is `ManualDeterminationRequired`, tests must not treat missing org facts as a pass.

### 4.7 ApplicabilitySnapshot (fill Prompt 11’s reserved shape)

```text
ApplicabilitySnapshot {
  schema,                    # "weeping-angel/applicability-snapshot/v1"
  assessment_id,
  scope,                     # IR AssessmentScope
  requirement_decisions[],   # id, rule (or digest), decision, rationale, predicates, subjects
  control_decisions[],       # same
  pack_entries[],            # optional copy of pack rows if a caller supplies them; NOT evaluated
  digest
}
```

`evaluate_assessment_applicability(definition, context) -> ApplicabilitySnapshot` walks `definition.requirements` and `definition.controls` in **id lexicographic order**, evaluates each rule, and seals `digest` over the snapshot body (exclude the digest field itself).

Prompt 11 persists this document. This slice **returns it**. No `persist_assessment_run`, no explain CLI, no `ControlExplanation`.

`pack_entries` exists so lineage can attach ISO `applicability.toml` rows **as artifacts**. The engine must not interpret `applicable = true/false` as Kleene facts.

### 4.8 Compile / projection integration

When a context is available, `resolve_applicability` (or the facade immediately after compile) **consumes** generic decisions:

- drop requirements whose decision is `NotApplicable`;
- **keep** `Applicable` and `ManualDeterminationRequired` in `applicable_requirements` (unresolved stays in-scope for review);
- do not special-case `FrameworkProfile` or collector ids.

Without a context, preserve today’s filter (`statically_applicable != Some(false)`).

Do not change `project_soa` in this slice (Prompt 11/12 own SoA purity / remap). The snapshot is what those slices will consume.

### 4.9 Public functions (normative names; module path may nest)

```text
fn build_applicability_context(definition: &AssessmentDefinition, extras: ContextExtras)
  -> ApplicabilityContext

fn evaluate_applicability(rule: &ApplicabilityRule, context: &ApplicabilityContext)
  -> ApplicabilityOutcome

fn evaluate_assessment_applicability(definition: &AssessmentDefinition, context: &ApplicabilityContext)
  -> ApplicabilitySnapshot
```

Prefer these names over `OrgContext` / `evaluate_org_context` so concurrent Prompt 11 **absence** string-scans stay meaningful until this slice actually lands the engine. Once landed, `ManualDeterminationRequired` and `struct ApplicabilitySnapshot` **will** appear — that is this slice’s job. Prompt 11 implement then persists the snapshot instead of asserting absence.

---

## 5. Acceptance criteria

Testable. Dual-suite target encodes these; titles should stay stable (`P10: …`).

1. Dual-suite `sdd_applicability_engine_baseline` / `sdd_applicability_engine_target` registered in root `Cargo.toml` (same `[[test]]` pattern as population / github / lineage).
2. Baseline GREEN on current HEAD: IR declarative + `statically_applicable` only; compile filter `!= Some(false)`; no evaluator module; SoA still boolean pack TOML; no product `ApplicabilitySnapshot`.
3. Target RED on current code for the cases in §6, then GREEN after implement.
4. `Always` → `Applicable`; `Never` → `NotApplicable`; no facts consulted.
5. Known-true predicate → `Applicable`; known-false → `NotApplicable`.
6. Unknown predicate (including `ProcessesPersonalData(true)` with no personal-data fact) → `ManualDeterminationRequired`, **not** `NotApplicable`.
7. Nested `All` / `Any` / `Not` implement §4.3. `Not(Unknown)` stays `Unknown`. `All(true, unknown)` is unknown; `All(false, unknown)` is false; `Any(true, unknown)` is true; `Any(false, unknown)` is unknown.
8. Jurisdiction-specific context: `Jurisdiction("EU")` true/false/unknown as §4.4.
9. Organization with **authoritative** assets and no `CloudAccount`/`CloudResource`: `UsesCloudProvider(true)` → `NotApplicable`.
10. Cloud state **unknown** (no authoritative asset inventory): `UsesCloudProvider(true)` → `ManualDeterminationRequired`.
11. Personal-data processing known true/false vs unknown — unknown never becomes `NotApplicable`.
12. Explicit `AssessmentScope` exclusions remove subject Z and record the exclusion rationale; the control may still be `Applicable`.
13. Vendor-dependent `HasVendor(true)` is true when in-scope vendors exist; false only when vendor inventory is authoritative and empty; unknown otherwise.
14. Rationale / predicate traces / selected and excluded subject ids are deterministically ordered; two evaluations of the same inputs yield the same digest.
15. Zero selected subjects + `Always` → `Applicable` with empty `selected_subjects`.
16. Same engine evaluates `Requirement.applicability` and `Control.applicability`. No `FrameworkProfile` / collector_id / ISO annex branch in the evaluator.
17. `evaluate_assessment_applicability` produces `ApplicabilitySnapshot` with Prompt 11’s reserved fields (`schema`, `assessment_id`, `scope`, `requirement_decisions`, `control_decisions`, `pack_entries`, `digest`).
18. Selected scope can be applied as a `Population` (via existing `EvidenceSet::set_population`) without a second inventory model.
19. IR crate still does not evaluate platform facts (`statically_applicable` unchanged in meaning). Catalog TOML and ISO `applicability.toml` are unmodified.
20. After implement: `cargo test --workspace --features demo`, `cargo fmt --all -- --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings` hold for files this slice touches. Neighbor targets (`sdd_population_runtime_target`, `sdd_canonical_assurance_catalog_target`, `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`) stay GREEN.

---

## 6. Dual-suite protocol

```text
Spec first (this file; no product feature code)
  → Register dual-suite at implement
  → Baseline GREEN on CURRENT code
  → Target RED on CURRENT code (right reasons)
  → Implement (applicability engine / evaluator / snapshot paths only)
  → Docs/ADR finalize if needed
  → Target GREEN
  → Prove baseline FAILS or skip-supersede
  → Target still GREEN
```

Register at implement (do **not** register in this spec-only phase unless a later implement commit does):

```toml
[[test]]
name = "sdd_applicability_engine_baseline"
path = "tests/sdd/applicability_engine.baseline.rs"

[[test]]
name = "sdd_applicability_engine_target"
path = "tests/sdd/applicability_engine.target.rs"
```

### 6.1 Baseline suite (GREEN on CURRENT)

Source/API characterization of §3, for example:

| ID | Assertion |
| --- | --- |
| P10-B01 | IR `applicability.rs` still says it does not evaluate platform facts; `fn statically_applicable` exists. |
| P10-B02 | `Always` → `Some(true)`; `Never` → `Some(false)`; `Predicate(Jurisdiction(_))` → `None`. |
| P10-B03 | `Not(Predicate(_))` → `None` (static fold already treats unknown as unknown). |
| P10-B04 | `resolve_applicability` filters with `statically_applicable() != Some(false)` and does not name `evaluate_applicability`. |
| P10-B05 | `project_soa` reads `applicability.toml`; `SoaEntry.applicable` is `bool`. |
| P10-B06 | Product crates have no `struct ApplicabilitySnapshot` and no `fn evaluate_applicability`. |
| P10-B07 | No `weeping-angel-assurance/src/applicability` module (or equivalent evaluator path). |
| P10-B08 | Compile of a requirement with `ProcessesPersonalData(true)` **keeps** the requirement. |
| P10-B09 | `Control` has no public `subjects()` getter (field is private). |
| P10-B10 | Collision fence: this suite does not import collector GitHub types. |

### 6.2 Target suite (RED on CURRENT, GREEN after)

Stable titles. Encode the **original found case** from the prompt:

| ID | Title / assertion |
| --- | --- |
| P10-T01 | `P10: static Always/Never` |
| P10-T02 | `P10: known true/false predicates` |
| P10-T03 | `P10: unknown predicates` |
| P10-T04 | `P10: nested All/Any/Not with unknown values` |
| P10-T05 | `P10: jurisdiction-specific context` |
| P10-T06 | `P10: organization with no cloud assets` |
| P10-T07 | `P10: cloud state unknown` |
| P10-T08 | `P10: personal-data processing known/unknown` |
| P10-T09 | `P10: explicit scope exclusions` |
| P10-T10 | `P10: vendor-dependent controls` |
| P10-T11 | `P10: deterministic rationale ordering` |
| P10-T12 | `P10: zero selected subjects is not NotApplicable` |
| P10-T13 | `P10: Not(Unknown) remains unknown` |
| P10-T14 | `P10: snapshot fills lineage persist shape` |
| P10-T15 | `P10: same engine for controls and requirements` |
| P10-T16 | `P10: evaluator has no framework/provider branches` |

Protocol: write the failing target test first (RED) → implement → GREEN. One regression test per comment/case titled as above.

---

## 7. Out of scope

- Framework-specific applicability branches (ISO Annex A, GDPR territorial scope engines, SOC 2 TSC trees)
- Teaching the evaluator to parse pack `applicability.toml` as Kleene truth
- Provider API calls / collector changes (`crates/weeping-angel-collector/**`)
- `tests/sdd/github_collector.*`, `docs/sdd/github-collector.md`, `GITHUB_EVIDENCE_TYPES`
- Generic ontology / description-logic engine
- Canonical catalog TOML redesign or new catalog families
- Prompt 11 explain CLI, ledger persist/load, `ControlExplanation`, pure report serialization
- Collapsing facade `AssessmentScope` with IR `AssessmentScope`
- Growing IR `Risk` / `ProcessingActivity` / `Vendor` into full RoPA / risk / vendor-management models
- Rewriting `statically_applicable` to consult inventories
- Certification / compliant / audit-passed language
- Prompt 12 ISO remap content

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Unknown facts treated as false (classic two-valued bug) | Normative Kleene tables; goldens T03/T07/T08/T13 |
| `Not(Unknown)` flipped to true | Explicit law + target test |
| Second inventory / org-graph crate | Derived `ApplicabilityContext` over existing IR types only |
| IR becomes a fact engine | Evaluator lives in assurance; IR comment + `statically_applicable` stay |
| Zero subjects auto-NA | T12; decision independent of selected-set cardinality |
| Collision with Prompt 09 collector SDD | Hard fence: no collector / github-collector / `GITHUB_EVIDENCE_TYPES` edits |
| Collision with Prompt 11 lineage (`struct ApplicabilitySnapshot` absence asserts) | Prompt 10 **fills** the reserved shape; Prompt 11 persist/explain stays out. After this implement, Prompt 11 baseline absence tests are expected to fail for the right reason and must be skip-superseded by the lineage run — not avoided by renaming the snapshot |
| SoA boolean path remains a silent second evaluator | This slice does not change `project_soa`; snapshot is the generic SSOT those slices must consume |
| Completeness defaulted to authoritative-empty | Empty list without an explicit completeness flag is `Unknown` |
| Facade vs IR `AssessmentScope` confusion | Keep both; context uses IR scope only |
| Prompt 11 needles `OrgContext` / `evaluate_org_context` | Use `ApplicabilityContext` / `evaluate_applicability` |
| Workspace fmt/clippy already red on unrelated crates | Do not mix unrelated rustfmt; new files must be clean |

---

## 9. ADR

This is an architecture/contract decision (Kleene law, crate home, derived context, snapshot contract). Draft: [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md). Accept when the public types land.

---

## 10. Handoff / done

**Done:** applicability is a real deterministic three-state evaluation layer; it integrates with assessment scope / populations; it preserves rationale and unknown facts; the same engine can drive canonical controls and framework projections without provider/framework coupling; `ApplicabilitySnapshot` is producible for Prompt 11; catalog/workspace validation holds for this slice’s files.

Prior session attempt for this prompt never started (4-run cap). Treat implement as a **fresh start** against this spec.

---

## 11. Implemented (what shipped)

*Empty — spec-only phase. Fill after target GREEN.*
