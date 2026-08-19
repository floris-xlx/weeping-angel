# Grok 4.6 Prompt 02 — Organizational Scope Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
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

## Tests

Cover nested inclusion, exclusion precedence, expired exclusion, unresolved subject, duplicate selectors, conflicting rules, organization-wide inclusion, and population selection. Prove that out-of-scope subjects cannot accidentally contribute positive assurance evidence to an in-scope assessment.

## Non-goals

Do not discover assets in this prompt. Do not add AWS/GitHub/Entra concepts. Do not generate a Statement of Applicability yet.

## Definition of done

The engine can deterministically answer what is inside the ISMS boundary, why, under whose approval, and for what period, with all decisions traceable to canonical records.