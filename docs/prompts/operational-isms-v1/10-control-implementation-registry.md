# Grok 4.6 Prompt 10 — Control Implementation Registry

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Canonical Assurance v1; Prompt 02

## Mission

Make the distinction between a canonical control and the organization's concrete implementation explicit and operational. Extend the existing `ControlImplementation` concept rather than creating a competing type.

## Required fields

A control implementation should support stable ID, canonical control ID, owner, implementation description, implementation state, scoped subjects/populations, systems/assets, effective date, review cadence, next review, evidence expectations, policy/document references, linked risks/treatments, exceptions, automation classification, and supersession/history.

States should distinguish planned, partially implemented, implemented, ineffective/disabled, retired, and unknown where required. `Implemented` must not imply `Effective`; effectiveness comes from control tests.

One canonical control may have several implementations over different populations or systems.

## Integrity

Validate no dangling control/subject/risk references. Ensure overlapping implementations do not accidentally double-count population coverage. Changes that materially alter implementation should create history/supersession rather than erase prior state.

## Tests

Cover split population implementations, partial rollout, retired implementation, missing evidence, one control with multiple systems, overlapping subject selectors, and compatibility with existing assessment fixtures.

## Non-goals

Do not encode ISO Annex A fields, provider APIs, or evidence conclusions in the implementation record.

## Definition of done

The assurance engine can distinguish `what the control means`, `how this organization implements it`, and `whether that implementation is actually effective`.