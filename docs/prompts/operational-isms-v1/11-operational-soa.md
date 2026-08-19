# Grok 4.6 Prompt 11 — Operational Statement of Applicability

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 02, 06, 08, 10; existing ISO 27001 framework projection

## Mission

Upgrade Statement-of-Applicability output from a static assessment projection into a living, explainable operational record generated from framework applicability, risk treatment, canonical mappings, implementation state, exceptions, and evidence effectiveness.

## Required semantics

For every framework control/requirement in the ISO projection, expose applicability state, rationale, linked risks, treatment rationale, canonical controls, implementation references, owner, implementation status, effectiveness status, evidence lineage, exclusions/exceptions, review state, and approval metadata.

Do not let a missing implementation silently become `not applicable`. Applicability and implementation are separate dimensions.

Non-applicability requires explicit rationale and accountable approval/review semantics. Expired rationale/approval must surface a readiness gap.

The SoA must remain a projection: generic IR/control objects must not gain ISO-specific fields.

## History

Produce immutable SoA snapshots/digests so a prior audit-period SoA can be reconstructed. Support diff between snapshots with causes.

## Tests

Cover applicable+effective, applicable+not implemented, applicable+insufficient evidence, non-applicable approved, non-applicable expired, partial canonical mapping, risk-treatment-driven applicability, and snapshot diff.

## Non-goals

Do not store licensed ISO normative text. Do not create certification claims.

## Definition of done

The SoA is mostly generated from the operational ISMS graph and can explain every inclusion, exclusion, implementation and effectiveness state.