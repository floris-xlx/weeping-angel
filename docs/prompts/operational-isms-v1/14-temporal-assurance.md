# Grok 4.6 Prompt 14 — Evidence Validity and Temporal Assurance

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: immutable evidence ledger; Prompt 13

## Mission

Make time a first-class assurance dimension so Weeping Angel can prove not only current posture but historical operating effectiveness.

## Evidence semantics

Formalize `observed_at`, `collected_at`, `valid_from`, `valid_until`, `supersedes`, revocation/invalidation where needed, source revision, and artifact digest. Preserve immutable envelopes; validity changes should be represented through new records/events rather than editing sealed evidence.

Define deterministic selection of evidence for assessment time/range. Prevent future evidence from satisfying past assessments. Prevent expired evidence from satisfying controls outside its validity window.

## Temporal control projection

Support point-in-time and period assessment. Period results should distinguish continuously effective, intermittent regression, insufficient observation coverage, ineffective, and manual review where appropriate. Do not infer continuous effectiveness from one observation unless the evidence/test semantics explicitly allow it.

## History

Provide timeline/diff primitives usable by readiness and audit exports.

## Tests

Cover overlapping evidence, supersession, revocation, clock boundaries, stale evidence, future observation, intermittent control failure, sparse observations, and reproducible historical assessment.

## Non-goals

Do not build UI charts or long-term database backend here.

## Definition of done

A finalized assessment can be evaluated against the evidence that was valid at that exact time or throughout a declared audit period, with no temporal leakage.