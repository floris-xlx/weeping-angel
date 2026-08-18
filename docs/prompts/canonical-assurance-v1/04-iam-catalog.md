# Grok 4.6 Prompt 04 — IAM Canonical Assurance Catalog

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompt 01 catalog contract, Prompt 02 typed evidence, Prompt 03 population runtime

## Mission

Implement the identity, authentication, authorization, privileged-access, and account-lifecycle portion of the canonical assurance catalog. This prompt owns IAM domain content only: canonical controls, evidence requirements/types if not already declared by the central catalog contract, deterministic provider-blind tests, fixtures, and documentation.

Do not redesign the catalog loader, evidence value system, population runtime, framework mappings, or provider collectors.

## Architecture

Every control must be provider-neutral. Correct:

```text
control.identity.mfa
control.identity.privileged-mfa
control.identity.inactive-account-lifecycle
```

Incorrect:

```text
control.okta.mfa
control.entra.admin-mfa
control.google-workspace.users
```

Provider-specific details belong in collector implementations that emit canonical evidence.

## Required control families

Implement a coherent IAM catalog targeting roughly 20–30 independently assessable controls, including where appropriate:

- unique user identities;
- MFA;
- privileged MFA;
- strong authentication policy;
- privileged identity inventory;
- least privilege;
- privileged-access minimization;
- access approval/authorization;
- periodic access review;
- inactive account lifecycle;
- terminated-user removal;
- joiner/mover/leaver lifecycle;
- service-account inventory;
- service-account ownership;
- service-account credential governance;
- emergency/break-glass access governance;
- shared-account restriction;
- authentication credential management;
- privileged-role changes monitored;
- external/guest access governance;
- stale privileged membership;
- access revocation timeliness;
- segregation of duties where expressible canonically.

Do not create artificial micro-controls purely to increase count.

## Canonical evidence

Define/reuse evidence contracts such as:

```text
evidence.identity.inventory
evidence.identity.authentication-state
evidence.identity.mfa-status
evidence.identity.privileged-membership
evidence.identity.role-membership
evidence.identity.last-active
evidence.identity.account-status
evidence.identity.account-owner
evidence.identity.access-review
evidence.identity.lifecycle-event
evidence.identity.service-account
evidence.identity.external-access
```

Evidence contracts describe observed facts, not assessment conclusions.

## Tests

Implement reusable declarative tests where possible, including:

```text
test.identity.mfa-enabled
test.identity.privileged-mfa-enabled
test.identity.no-inactive-privileged-accounts
test.identity.no-terminated-active-accounts
test.identity.all-service-accounts-have-owner
test.identity.access-review-current
test.identity.no-unapproved-guest-access
```

Use subject populations. Example: `all privileged identities have MFA`, not `some mfa_status evidence exists`.

Tests must distinguish missing data, stale data, actual failure, and manual review.

## Manual/hybrid semantics

Controls such as access approval, segregation of duties, or periodic review may require manual/hybrid evidence. Mark them honestly. Do not invent technical automation where the canonical control fundamentally depends on governance.

## Fixtures

Create deterministic fixtures for at least:

- healthy organization;
- one privileged user without MFA;
- inactive admin still active;
- terminated employee account active;
- service account without owner;
- partial identity inventory;
- stale access-review evidence;
- approved exception for a break-glass account.

## Validation

All controls must have stable IDs, domains, evidence requirements, and test references as appropriate. No orphaned evidence/tests. No provider names in IDs. No ISO/SOC2/NIS2 references in canonical content.

## Non-goals

Do not implement Entra/Okta/Google collectors. Do not map ISO requirements here. Do not modify generic rule semantics unless a documented blocker exists and the population-runtime owner must be involved.

## Definition of done

The IAM catalog can evaluate realistic identity populations using canonical evidence from any future identity provider, provides deterministic and explainable results, and passes the catalog validator and full workspace verification.