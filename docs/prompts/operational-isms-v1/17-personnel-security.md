# Grok 4.6 Prompt 17 — Personnel Security Lifecycle

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: canonical identity/population runtime, Prompt 12, Prompt 10

## Mission

Operationalize personnel security across joiner, mover, and leaver lifecycle using provider-neutral identity/personnel evidence and population-aware control tests.

## Domain

Represent personnel population membership and lifecycle events without turning the generic identity model into an HRIS. Support employees, contractors, privileged personnel, developers, finance/security/executive groups, and organization-defined populations.

Evidence/control families should cover required screening where applicable, employment/confidentiality commitments, awareness training, role-specific training, policy acknowledgement, access provisioning, periodic access review, role change, offboarding, account disablement/removal, and asset return references where available.

Tests must evaluate the full required population. One trained user must never prove training coverage. Exceptions must be scoped to individuals/populations and validity periods.

## Integrations boundary

HRIS, IdP, LMS, and MDM collectors normalize to canonical evidence; they do not emit personnel-compliance conclusions.

## Tests

Include complete training population, one overdue user, new joiner grace period, leaver with active access, mover retaining excessive privileges, expired exception, missing personnel source, and manual screening evidence.

## Non-goals

No payroll, recruiting, employee profile UI, or HR system of record.

## Definition of done

Weeping Angel can continuously test personnel-security lifecycle controls honestly across complete in-scope populations.