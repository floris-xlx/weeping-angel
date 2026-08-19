# SDD run: Immutable Assessment Lineage, Explainability, and Report Cleanup

| Field | Value |
| --- | --- |
| Run id | `sdd-assessment-lineage` (protocol report; implement assigns telemetry id) |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` (dual-suite) |
| Status | **Specified** — durable SSOT + draft ADR; baseline GREEN on shortcuts; target LIN-001–008 and LIN-010–014 authored and RED on CURRENT; no product feature code |
| Slice | Prompt 11: persistable execution lineage, `ControlExplanation`, pure report serialization, generic framework facade, snapshot compare |
| Spec | [`docs/sdd/assessment-lineage.md`](assessment-lineage.md) |
| Draft ADR | [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/11-assessment-lineage.md`](../prompts/canonical-assurance-v1/11-assessment-lineage.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Dual-suite | `tests/sdd/assessment_lineage.baseline.rs` · `tests/sdd/assessment_lineage.target.rs` → `sdd_assessment_lineage_baseline` / `sdd_assessment_lineage_target` |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update on implement, not this phase |
| Collision fence | Prompt 09 GitHub collector · Prompt 10 evaluator reimplementation · Prompt 12 ISO remap / catalog domain TOML |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence and gates.

---

## Spec

- **Title:** Immutable Assessment Lineage, Explainability, and Report Cleanup
- **Problem:** An assessment is a current-state report, not a reproducible execution artifact. `AssessmentRun` is built and dropped; serialize re-loads ISO 27001:2022; non-ISO profiles compile a production stub; there is no explain CLI; compare only flips effectiveness/stale.
- **Current behavior (SHA `e430980c…`):** Encoded by `sdd_assessment_lineage_baseline`. Facade `assess` uses `let _run` with empty `collector_runs` and reused compile digest for definition / evidence-snapshot / result identity. `AssessmentReport::serialize` calls `load_framework_pack("iso-27001", "2022")` and invents `automationCoverage` / `evidenceCoverage` percent strings. `assessment_for_target` / `normalize` / `stub_catalog` special-case ISO. `compare` fills only effective/ineffective/stale. `project_soa` reads live `applicability.toml`. CLI has no `Explain`; non-catalog arms banner-and-exit-0. Ledger has lineage tables without persist/load APIs. No `ControlExplanation` / `CoverageMetrics` / snapshot persist types. Prompt 10 evaluator has since landed — persist its snapshot; do not re-evaluate.
- **Desired behavior:** Persist the immutable chain (pack, catalog, definition, applicability, collection runs, envelopes, evidence snapshot, control-test runs, assessment run, readiness, SoA). Return and persist `AssessmentRun`. Replay from pins; digest mismatch if current files are consulted. Generic `ControlExplanation` + `weeping-angel assurance explain --assessment <id> --control <id>`. Pure serialize. One pack loader path; no production stub. Separate `CoverageMetrics` families. Compare fills subjects, applicability, evidence, exceptions, digest changes. SHA-256 canonical JSON result/snapshot digests exclude wall-clock `duration` / `evaluatedAt`.
- **ADR:** needed — draft at [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md)

### Acceptance criteria (this slice)

1. Dual-suite remains registered; baseline GREEN on §3 shortcuts; author LIN-001–008 and LIN-010–014 so target is RED on CURRENT before product feature code; after implement target GREEN and baseline skip-superseded.
2. `AssessmentRun` is returned/persisted (never `let _run`) with start/completion, scope, `completed`/`partial`/`failed`, collector runs, and distinct pack / catalog / definition / evidence-snapshot / result / applicability pins.
3. Historical assessment reconstructs from pinned snapshots; current catalog/pack edits do not silently rewrite old results (LIN-001, LIN-002).
4. Historical evidence is append-only; partial/failed collection is distinguishable from a completed empty collection (LIN-005).
5. `ControlExplanation` + dispatched `assurance explain` cite exact evidence digests, population, missing evidence, failing/missing subjects, test id/version, exceptions, mappings (LIN-003, LIN-012).
6. `AssessmentReport` serialization is pure: no pack load, network, filesystem, or hidden current-state resolution (LIN-004).
7. Explicit `AssessmentSummary` / `FrameworkReadinessSnapshot` / `CoverageMetrics`; seven metric families stay separate; no single compliance percentage (LIN-013).
8. One registry/loader path; production stub `canonical:stub-1` / `assess-runtime-1` removed from production (LIN-010, LIN-011).
9. `compare` identifies applicability, subjects, evidence add/remove/supersession, test results, exceptions, framework/catalog digest changes (LIN-006, LIN-007).
10. Snapshot/result digests are deterministic SHA-256 of canonical JSON, domain-separated from compile-digest reuse, exclude `duration` / `evaluatedAt` (LIN-008).
11. Ledger persist/load for `AssessmentRun` and `ControlTestRun`; completed payloads are not silently replaced (LIN-014).
12. Neighbor targets `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target` and `cargo test --workspace --features demo` stay GREEN after implement (LIN-015).

### Out of scope

- Multi-tenant SaaS / UI
- New frameworks
- Domain catalog redesign / `catalog/canonical/v1` TOML rewrite
- Prompt 09 GitHub collector files
- Prompt 10 evaluator reimplementation (`OrgContext` / `evaluate_org_context`); persist the landed `ApplicabilitySnapshot` instead
- Prompt 12 ISO pack ID remap
- IR schema fork
- Certification claims
- Collector discovery / scanner-bridge redesign

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Spine ACT depends on production stub | Fail-closed production; test fixtures only |
| ISO `iso_007` pack-digest / serialize needles | Carry digest on snapshot; load at assess time |
| Concurrent Prompt 10 | Persist landed `ApplicabilitySnapshot`; do not reimplement Kleene |
| `INSERT OR REPLACE` on collection_runs | Lineage persist is append-only / digest-keyed |
| Wall-clock in result digest | Exclude `duration` / `evaluatedAt` |
| Neighbor suite red if stub removed early | Fixtures first; neighbor GREEN is a hard gate |
| Evidence crate concluding | Opaque JSON payloads only |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written before product feature code | [`docs/sdd/assessment-lineage.md`](assessment-lineage.md) — SSOT re-characterized against HEAD `e430980c…` + existing baseline |
| ADR draft | written (architecture/contract) | [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) |
| Dual-suite register | `[[test]]` in root `Cargo.toml` | **done** — `sdd_assessment_lineage_baseline` / `sdd_assessment_lineage_target` |
| Baseline | skip-supersede after target GREEN | `#[ignore = "superseded by sdd_assessment_lineage_target"]` — 14 ignored |
| Target | GREEN after implement | **GREEN** — 15 passed (LIN-001–015) |
| Implement | product feature code | landed: persistable `AssessmentRun`, snapshot types, `ControlExplanation` + `assurance explain`, pure serialize, generic pack loader, compare/diff, ledger persist/load |
| Neighbors | stay GREEN | Required after implement; this phase does not edit their suites |
| Contract | update on accept | Deferred to implement |

---

## Next (pre-product, then implement)

1. ~~Author LIN-001–008 and LIN-010–014~~ done in `tests/sdd/assessment_lineage.target.rs`.
2. Prove target RED on CURRENT (`cargo test --test sdd_assessment_lineage_baseline --test sdd_assessment_lineage_target`).
3. Implement lineage persist, explain, pure serialize, generic facade, compare, metrics — own paths only.
4. Target GREEN; skip-supersede baseline; workspace + neighbor targets GREEN.
5. Accept ADR; update `docs/contracts/assurance-runtime.md`.
