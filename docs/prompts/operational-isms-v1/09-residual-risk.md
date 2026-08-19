# Grok 4.6 Prompt 09 — Residual Risk and Control Effectiveness

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 05, 06, 08; canonical control-test results

## Mission

Project control effectiveness into residual-risk assessment without pretending all risk can be reduced to a simplistic mathematical formula.

## Modes

Support `Calculated`, `Assessed`, and `Hybrid` residual-risk modes. Calculated methodologies must be deterministic and explicitly versioned. Assessed residual risk requires accountable manual evidence. Hybrid mode combines deterministic signals with an approved management assessment.

## Lineage

Every residual-risk result must identify inherent risk version, treatment plan version, relevant controls, control-test results/evidence snapshot, methodology, time, and any manual assessment/approval.

Historical changes must remain queryable. A control regression should be able to trigger a new residual-risk projection rather than mutating historical values.

Never map `Effective` directly to zero risk. Define methodology-specific reduction semantics and fail closed when required control-effectiveness evidence is missing.

## Tests

Cover effective/ineffective/missing controls, partial treatment, manual residual assessment, historical change, stale evidence, multiple controls, and no-reduction methodology. Ensure an approved exception does not silently mean risk is low.

## Non-goals

Do not build dashboards or risk acceptance workflows here.

## Definition of done

Residual risk becomes an explainable projection grounded in actual treatment/control state and can be reproduced for any finalized assessment snapshot.