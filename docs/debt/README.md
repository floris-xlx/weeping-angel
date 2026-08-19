# Technical-debt register

Canonical machine register: [`register.toml`](register.toml) (`schema = "weeping-angel/debt-register/v1"`). Decision: [`docs/adr/0009-repository-health-gate.md`](../adr/0009-repository-health-gate.md).

This README is not a second register. Dated snapshots such as [`baseline-2026-08.md`](baseline-2026-08.md) are evidence, not status.

## Status machine

Every `[[finding]]` has a required `id`, `title`, `status`, and `summary`.

Closed set of `status` values:

| status | Meaning |
| --- | --- |
| `open` | Known, not yet triaged to a workstream |
| `confirmed` | Accepted as real debt |
| `in-progress` | Active remediation |
| `resolved` | Fixed **and** guarded (see proof law) |
| `rejected` | Not debt (wontfix / false positive) |
| `superseded` | Replaced by another finding or design |

## Proof law (`resolved`)

A finding may be `status = "resolved"` **only if** it lists a non-empty `regression_tests` array **or** a non-empty `repository_guard` value (check id string, or boolean `true` meaning the live guard covers it).

`rejected` and `superseded` do not require `regression_tests` or `repository_guard`.

Guard check **13** (`cargo xtask guard`) rejects:

- missing required fields
- illegal status
- duplicate `finding.id`
- `resolved` without proof

## Stubbed guard checks

Checks 04–12 and 14–15 may skip only by citing a registered `DEBT-GUARD-NN` finding. Silent pass is forbidden.
