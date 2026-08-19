# 05 Architecture spec extract (R13)

SSOT remains [`../ELITE-contract-weeping-angel-assurance-ir.md`](../ELITE-contract-weeping-angel-assurance-ir.md).

This extract is for PR reviewers who need invariants without the phase cards.

## Why

Make the IR the semantic authority so framework/facade/test crates stop inventing domain types.

## Invariants after Phase 6

INV-IR-A…L plus spine INV-1…5.

## Graph

IR → framework, evidence, control-test. Edges not shown are forbidden.

## Surfaces

Handwritten modules listed in the elite plan. Fixtures directory `tests/fixtures/assurance-ir/v1/` is the serialized allowlist.

## Debt

AD-001…010 in `matrices/m-40-architecture-debt.md`.

## ACT

ACT-001…015 + IR-001…025.
