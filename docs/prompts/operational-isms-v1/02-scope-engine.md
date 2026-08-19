# Grok 4.6 Prompt 02 — Organizational Scope Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Batch: 01/06 — Foundation (Prompts 01–04)
Execution: implement after Prompt 01 against the same branch/worktree
Dependencies: Prompt 01

## Mission

Turn ISMS scope from descriptive text into an executable, explainable boundary model. Reuse existing `AssessmentScope`, `ScopeExclusion`, `SubjectSelector`, assets, identities, vendors, and organization references where possible.

## Required capabilities

Model scopeable entities such as organizations, business units, locations, systems, services, repositories, cloud accounts, networks, data domains, personnel populations, vendors, and processing activities without embedding provider-specific schemas.

Every candidate subject must resolve to one of `InScope`, `OutOfScope`, `Conditional`, or `Unknown`. Resolution must return rationale and lineage, not only a boolean.

Exclusions must support rationale, owner/principal, approval reference, approved-at time, review/expiry time, and supporting evidence references. Silent exclusions are forbidden. Expired exclusions must no longer suppress scope unless explicitly renewed.

Support inclusion rules, explicit exclusions, nested subjects, inherited scope, and deterministic precedence. Fail closed on ambiguous conflicting rules.

## Outputs

Expose a deterministic `ScopeResolution` suitable for framework compilation and collector planning. A collector should be able to ask which subjects are in scope without learning ISO semantics.

Generate an explain trace such as:

`repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope`

## Implementation constraints

Scope resolution must be a pure deterministic operation over canonical inputs. Do not let collectors mutate scope state. Preserve explicit lineage for every rule that influenced a decision. Define precedence once and test it directly; do not rely on iteration order. Unknown or contradictory scope data must never become positive in-scope evidence implicitly.

## Tests

Cover nested inclusion, exclusion precedence, expired exclusion, unresolved subject, duplicate selectors, conflicting rules, organization-wide inclusion, and population selection. Prove that out-of-scope subjects cannot accidentally contribute positive assurance evidence to an in-scope assessment.

## Acceptance gates

- The same canonical inputs always produce the same resolution and explain trace.
- Expired exclusions fail closed and are visible in the trace.
- Conflicting equal-precedence rules produce `Unknown`/error rather than arbitrary selection.
- Collector planning can consume the resolved subject set without framework-specific logic.
- Existing `AssessmentScope` consumers remain compatible or receive an explicit migration adapter.

## Non-goals

Do not discover assets in this prompt. Do not add AWS/GitHub/Entra concepts. Do not generate a Statement of Applicability yet.

## Definition of done

The engine can deterministically answer what is inside the ISMS boundary, why, under whose approval, and for what period, with all decisions traceable to canonical records.