# Grok 4.6 Prompt 07 — Risk Identification and Candidate Correlation

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompt 06; canonical findings/assets/populations available

## Mission

Automate risk candidate discovery without allowing machine observations to become organizationally accepted risks without review.

## Inputs

Consume canonical assets, security findings, vulnerability evidence, incidents, supplier dependencies, identity posture, data classifications, architecture relationships, existing risks, and other provider-neutral evidence.

## Output

Introduce `RiskCandidate` with source lineage, scenario proposal, impacted subjects, supporting observations, confidence, correlation key, duplicate candidates, suggested risk category, and optional score suggestion.

Candidate promotion must be explicit. `RiskCandidate != Risk`. Promotion records principal, time, rationale, selected methodology inputs, and resulting risk ID. Rejection/dismissal should also be retained for deduplication and auditability.

## Correlation

Implement deterministic candidate clustering for clearly identical subjects/scenarios. Avoid probabilistic black-box behavior in core runtime. If an AI-assisted adapter is later used, it must emit a proposal that passes deterministic validation and human approval.

## Tests

Include multiple findings collapsing into one candidate, same finding contributing to two distinct risks, rejected candidate resurfacing rules, candidate promotion, stale evidence, and no-finding cases. Prove scanners cannot declare `risk accepted` or `ISO control failed`.

## Non-goals

Do not build an LLM client, threat intelligence service, UI queue, or risk treatment engine.

## Definition of done

The system can continuously surface explainable candidate risks from existing evidence while preserving the management decision boundary.