# Grok 4.6 Prompt 01 — Canonical ISMS Context IR

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Batch: 01/06 — Foundation (Prompts 01–04)
Execution: implement, test, document, and leave the tree ready for Prompt 02
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

## Implementation constraints

Keep the canonical model in the assurance IR layer and keep persistence, network clients, provider SDKs, framework packs, and CLI concerns outside it. Reuse existing stable-ID and canonical serialization conventions rather than introducing a second identity or digest system. Any new lifecycle enum must be exhaustive, serializable, and validated centrally.

## Tests

Create dual SDD suites. Target tests must prove deterministic round-trip serialization, backward compatibility with current fixtures, rejection of duplicate/dangling references, provider/framework neutrality, and that the framework crate remains network-free.

Add representative fixtures for one organization with two business units, one external issue, one internal issue, interested parties, objectives, and a risk methodology reference.

## Acceptance gates

- Existing assurance tests remain green.
- Canonical serialization is byte-stable for equivalent input ordering.
- No ISO/provider vocabulary is introduced into generic IR types.
- Invalid references fail closed with deterministic errors.
- A fixture can construct, serialize, deserialize, validate, and explain one complete ISMS context.

## Non-goals

Do not build UI, persistence service, policy editor, workflow engine, ISO mapping, risk scoring, or auditor portal here.

## Definition of done

Operational ISMS has one canonical root model that can anchor later scope, risk, governance, audit, and readiness work without creating another compliance graph.