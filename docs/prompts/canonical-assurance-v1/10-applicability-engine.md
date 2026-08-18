# Grok 4.6 Prompt 10 — Organization Context and Applicability Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–08 and population runtime

## Mission

Make the existing `ApplicabilityRule`/`ApplicabilityPredicate` model operational by building a deterministic organization-context and applicability evaluator. The result must decide which canonical controls and framework requirements apply to a specific assessment scope without embedding provider or framework special cases into the generic evaluator.

## Context inputs

Build applicability context from existing IR entities and assessment scope, including where available:

```text
organizations
assets
identities
vendors
jurisdictions
technologies
data categories
processing activities
risk context
cloud usage
employee presence
personal-data processing
assessment inclusions/exclusions
```

Do not build a second competing inventory model if `Asset`, `Identity`, `Vendor`, `ProcessingActivity`, `SubjectSelector`, and `AssessmentScope` can express the needed context.

## Applicability result

Every evaluated rule should produce a deterministic result conceptually equivalent to:

```text
Applicable
NotApplicable
ManualDeterminationRequired
```

plus rationale and the facts/predicates that caused the result.

Unknown facts must not be treated as false. Example: if the system does not know whether personal data is processed, `ProcessesPersonalData(true)` should not evaluate to `NotApplicable`; it should remain unresolved/manual unless other logic establishes the result.

## Rule semantics

Implement correct three-state evaluation for:

```text
Always
Never
All
Any
Not
Predicate
```

Ensure `Not(Unknown)` remains unknown rather than becoming true.

Support existing predicates such as asset type, organization attribute, jurisdiction, processing category, technology, data category, vendor presence, employee presence, cloud-provider use, and personal-data processing.

## Scope and populations

Applicability should constrain downstream subject populations. A control may be applicable to only selected assets/subjects. Preserve the reason and selected scope.

Zero subjects should not automatically mean `NotApplicable` unless the rule/context justifies that conclusion.

## Persistence/explainability

Produce an applicability snapshot or explanation data that later assessment-lineage work can persist. It must be possible to explain:

```text
Why was control X applicable?
Why was control Y not applicable?
Which fact was unknown?
Which exclusion removed subject Z?
```

## Tests

Add cases for:

- static Always/Never;
- known true/false predicates;
- unknown predicates;
- nested All/Any/Not with unknown values;
- jurisdiction-specific context;
- organization with no cloud assets;
- cloud state unknown;
- personal-data processing known/unknown;
- explicit scope exclusions;
- vendor-dependent controls;
- deterministic rationale ordering.

## Non-goals

Do not add framework-specific applicability branches. Do not add provider API calls to the evaluator. Do not implement a generic ontology engine. Do not redesign the canonical catalog.

## Definition of done

Applicability is a real deterministic three-state evaluation layer, integrates with assessment scope/populations, preserves rationale and unknown facts, and can drive both canonical controls and framework projections without provider/framework coupling.