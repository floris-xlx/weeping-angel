# Grok 4.6 Prompt 01 — Canonical ISMS Context IR

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Canonical Assurance v1 complete or rebased to latest main

## Mission

Introduce a provider-neutral, framework-neutral operational ISMS context model that becomes the root object for continuous management-system operation. Extend the existing assurance IR instead of creating a parallel GRC schema. Preserve the current assurance spine and existing `AssessmentDefinition` compatibility.

## Required model

Add durable concepts for `IsmsContext`, organization identity, management-system scope reference, internal/external issues, interested-party references, obligation references, security objectives, risk methodology reference, governance cadence, and lifecycle status. Stable IDs must be explicit and serialization deterministic.

The model must be usable by ISO 27001 now and other frameworks later. No ISO clause numbers, Annex A semantics, cloud-provider fields, or vendor-specific objects belong in the generic IR.

The ISMS context must distinguish definition from point-in-time assessment input. Do not stuff mutable assessment results into the context record.

## Relationships

Support canonical relationships:

`ISMS -> Organization -> Scope`
`ISMS -> InterestedParty -> Obligation`
`ISMS -> Objective`
`ISMS -> RiskMethodology`
`ISMS -> Asset/Vendor/Identity populations through existing IR references`

Add validation for dangling IDs, duplicate IDs, empty required identity fields, and impossible lifecycle states.

## Compatibility

Existing `AssessmentDefinition::new` must continue to work. New fields should be optional/defaulted when required for backward compatibility. Avoid broad renames of `Asset`, `Vendor`, `Risk`, `Control`, `Requirement`, `Mapping`, `SubjectSelector`, or evidence types.

## Tests

Create dual SDD suites. Target tests must prove deterministic round-trip serialization, backward compatibility with current fixtures, rejection of duplicate/dangling references, provider/framework neutrality, and that the framework crate remains network-free.

Add representative fixtures for one organization with two business units, one external issue, one internal issue, interested parties, objectives, and a risk methodology reference.

## Non-goals

Do not build UI, persistence service, policy editor, workflow engine, ISO mapping, risk scoring, or auditor portal here.

## Definition of done

Operational ISMS has one canonical root model that can anchor later scope, risk, governance, audit, and readiness work without creating another compliance graph.