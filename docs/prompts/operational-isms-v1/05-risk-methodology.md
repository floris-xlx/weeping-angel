# Grok 4.6 Prompt 05 — Risk Methodology IR and Scoring

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompt 01

## Mission

Replace implicit/hardcoded risk scoring assumptions with an explicit canonical methodology. Preserve compatibility with the current minimal `Risk` record while establishing the primitives needed for a real risk engine.

## Required primitives

Implement stable types for likelihood scales, impact scales, risk matrices, risk scores, risk ratings, appetite, tolerance, acceptance thresholds, and scoring mode. Support qualitative, semi-quantitative, quantitative, and custom bounded methodologies.

Do not hardcode a 5x5 matrix. An organization must be able to use 1-3, 1-5, low/medium/high, or a quantitative expected-loss model without modifying control logic.

Methodologies must be versioned and immutable once used in a finalized risk assessment. New methodology revisions supersede old ones.

## Evaluation

Provide deterministic scoring APIs and validation. Reject malformed matrices, duplicate ordinal positions, unreachable ratings, invalid boundaries, and scores outside declared domains.

The API must separate raw input values from derived rating. Do not let a collector directly emit `RiskRating::High` as compliance evidence.

## Tests

Include 3x3 and 5x5 fixtures, custom thresholds, invalid matrices, deterministic serialization, methodology supersession, and boundary calculations.

## Non-goals

Do not identify risks, apply controls, calculate residual risk, or accept risks in this prompt.

## Definition of done

Risk scoring is explicit, organization-configurable, versioned, reproducible, and no longer an ad-hoc field on a risk record.