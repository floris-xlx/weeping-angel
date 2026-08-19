# Grok 4.6 Prompt 18 — Supplier Risk Management Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: existing `Vendor`, Prompts 03, 06, 08, 12

## Mission

Turn the current vendor concept into an operational supplier-security lifecycle linked to services, assets, obligations and risks.

## Required capabilities

Model supplier/vendor classification, criticality, supplied services, data/system access, owner, onboarding review, security requirements, risk assessment, approval, contract/document evidence, reassessment cadence, monitoring status, issues, termination/offboarding, and linked organizational risks.

Do not assume every vendor needs identical review. Support risk-tiered requirements and population tests such as `all critical suppliers have current security review`.

Supplier evidence may come from questionnaires/manual review or automated posture sources, but evidence presence alone must not imply risk acceptance.

## Lifecycle

Candidate -> under review -> approved -> active -> restricted/suspended -> terminating -> terminated, with explicit review/approval history. Expired assessments must create gaps/events.

## Tests

Cover critical vendor current review, stale review, low-risk reduced requirements, vendor with privileged access, vendor termination with lingering access, missing contract security requirement, expired exception, and supplier-related risk linkage.

## Non-goals

No questionnaire SaaS, procurement suite, contract authoring, or external trust-center scraping.

## Definition of done

Critical suppliers are continuously represented as dependencies with accountable risk, evidence, review cadence and control impact.