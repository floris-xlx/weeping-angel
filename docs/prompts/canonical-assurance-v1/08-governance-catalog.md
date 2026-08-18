# Grok 4.6 Prompt 08 — Governance, Vendor, Personnel, and Manual Assurance Catalog

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–03

## Mission

Implement the governance-heavy canonical assurance catalog that technical collectors cannot honestly automate: policy, risk governance, personnel security, supplier management, incident governance, awareness/training, continuity governance, internal review, and manual evidence semantics.

This prompt owns canonical content and manual/hybrid evidence patterns only.

## Required control families

Build approximately 30–45 meaningful controls across:

- information-security policy;
- policy review cadence;
- security roles and responsibilities;
- risk assessment;
- risk treatment;
- risk ownership;
- security objectives;
- documented scope;
- internal audit;
- management review;
- corrective action/nonconformity handling;
- continual improvement;
- incident-response plan;
- incident exercises/tabletops;
- incident postmortem/review;
- security awareness;
- role-specific training;
- personnel onboarding/offboarding governance;
- confidentiality commitments where applicable;
- supplier inventory;
- supplier risk review;
- supplier security requirements;
- supplier reassessment;
- cloud/vendor governance;
- business continuity plan;
- disaster-recovery governance;
- data classification policy;
- acceptable-use policy;
- asset ownership;
- document-control governance;
- evidence collection/retention governance.

## Manual evidence subsystem expectations

Represent manual evidence as first-class immutable evidence, not as a boolean bypass. Support concepts such as:

```text
attestation
document reference
approval
meeting/review record
auditor observation
training record
exercise record
risk acceptance
policy acknowledgement
```

Evidence should retain principal/author, timestamp, subject, artifact reference where relevant, freshness/validity, and review state.

## Canonical evidence examples

```text
evidence.governance.policy
evidence.governance.policy-review
evidence.governance.management-review
evidence.governance.internal-audit
evidence.risk.assessment
evidence.risk.treatment
evidence.personnel.training
evidence.personnel.acknowledgement
evidence.vendor.inventory
evidence.vendor.risk-review
evidence.incident.exercise
evidence.resilience.continuity-plan
evidence.manual.attestation
```

## Tests

Use deterministic tests for freshness, presence, required approvals, population coverage where possible, and manual-review requirements. Do not pretend document existence proves implementation effectiveness when the control requires operational evidence.

Examples:

- required policy exists and is within review window;
- all required personnel have current training evidence;
- all critical vendors have current risk reviews;
- management review exists within policy period;
- internal audit evidence is current;
- incident exercise occurred within required window.

## Exceptions and risks

Integrate existing IR `Risk` and `Exception` concepts where appropriate. Approved exceptions must never be silently converted into `Effective`. Expired exceptions must not suppress failing results.

## Fixtures

Include current/stale/missing documents, incomplete training populations, vendor review gaps, approved and expired exceptions, and controls requiring manual review despite supporting evidence.

## Non-goals

Do not implement a full GRC workflow/SaaS product. Do not build document editors. Do not map ISO requirements here. Do not change generic catalog/runtime contracts unless unavoidable.

## Definition of done

The governance catalog gives Weeping Angel an honest manual/hybrid control layer, with immutable and explainable evidence, rather than forcing technical automation onto inherently organizational controls.