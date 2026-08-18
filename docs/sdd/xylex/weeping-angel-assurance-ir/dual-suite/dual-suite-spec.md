# Dual-suite spec — Compliance IR deepen

## Goal

Characterize the thin IR, then drive RED→GREEN on IR-001…025 without regressing ACT-001…015.

## Repositories

`floris-xlx/weeping-angel` crate `weeping-angel-assurance-ir`.

## Context

Baseline suite `sdd_assurance_runtime_baseline` is already superseded. Do not revive it.

Characterization of current IR is the existing target suite (PASS on HEAD) plus this document’s current-behavior list.

## Working assumptions / Non-goals

Target suite `tests/sdd/compliance_ir.target.rs` must be RED on current HEAD for new assertions. Do not implement product code in authoring mode.

## Characterization (current HEAD — must keep passing)

| Fact | Proof |
| --- | --- |
| Empty `ControlId::new("")` succeeds | `lib.rs` L22–24 |
| `Control` JSON keys are schemaVersion, id, title, description | ACT-006 |
| Partial path not equivalent | ACT-005 |
| `resolve_applicability` returns all requirements | framework L265–271 |
| `Assessment` is in the framework crate | framework L101 |

## Target suite (must FAIL on current HEAD)

Path: `tests/sdd/compliance_ir.target.rs` (add to root `Cargo.toml` `[[test]]` like the existing SDD tests).

Titles must include the IR-### id:

| ID | Current | Desired |
| --- | --- | --- |
| IR-001 | empty ID constructs | reject |
| IR-002 | n/a | title change does not change id |
| IR-003 | thin Control | no framework fields after expand |
| IR-004 | framework id loose | `FrameworkRef` preserved |
| IR-005 | no validator | dangling mapping fails |
| IR-006 | ACT-005 | still no inferred equivalence on richer graph |
| IR-007 | n/a | generated provenance ≠ curated |
| IR-008 | no implementation type | definition ≠ implementation |
| IR-009 | n/a | status ≠ Effectiveness |
| IR-010 | no rule type | applicability round-trip |
| IR-011 | n/a | no provider selectors |
| IR-012 | evidence has type only | no collector id |
| IR-013 | tests provider-blind already | still no provider id after expand |
| IR-014 | no extensions | unknown keys survive |
| IR-015 | digest exists | still deterministic |
| IR-016 | no domain prefix | domain separation |
| IR-017 | no validator | duplicate ids fail |
| IR-018 | facade scope | IR scope deterministic |
| IR-019 | no Risk record | dangling risk fails |
| IR-020 | no Exception record | dangling exception fails |
| IR-021 | unknown schema not checked on all docs | fail closed |
| IR-022 | no UUID scan | persisted IR has no v4 |
| IR-023 | no version constraint | ranges respected |
| IR-024 | no external_id | external ≠ internal id |
| IR-025 | Control has no ISO fields | catalogs (stubs) still cannot extend Control |

Protocol: write test first → RED → fix → GREEN. One logical assertion cluster per IR-###.

## Handoff brief

Implement mode starts at IR-001 after the mechanical split.
