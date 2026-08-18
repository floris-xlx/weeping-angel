# Supersession banners

| Document | Status after this pack |
| --- | --- |
| [`docs/sdd/assurance-runtime-spine.md`](../../../assurance-runtime-spine.md) | **Still true** for Phases 0–8 invariants, crate graph, ACT-001…015. **Superseded** if read as “IR is production-complete.” |
| [`docs/sdd/sdd-assurance-runtime-spine.md`](../../../sdd-assurance-runtime-spine.md) | Same: spine done; IR deepen is this pack. |
| [`docs/contracts/assurance-runtime.md`](../../../../contracts/assurance-runtime.md) | Current thin contract. Phase 6 replaces field lists for IR documents. |
| [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../../../../adr/0001-inwardly-extensible-assurance-runtime.md) | Decision stands. Consequences update at Phase 5–6 (`AssessmentDefinition` moves to IR). |
| [`docs/adr/0002-iso-27001-assurance-vertical.md`](../../../../adr/0002-iso-27001-assurance-vertical.md) | **Accepted** ISO vertical. Consumes IR; does **not** close OWN-001 / IR deepen. |
| [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../../../../adr/0003-canonical-assurance-catalog-v1.md) | **Accepted** catalog infrastructure. Separate schema/crate; does **not** fork `assurance-ir/v1` or remap ISO packs. |
| [`docs/sdd/iso-27001-automated-assurance-mvp.md`](../../../iso-27001-automated-assurance-mvp.md) | Implemented ISO pack/ledger/TestExpr/CLI. IR still thin. |

ISO vertical (workflow `spec-driven-development`, 2026-08-18) extended `Mapping` with `relation` + `rationale` on the same type. IR Phase 4 must **extend that type**, not introduce a twin `Mapping`. `Assessment` remains in `weeping-angel-framework`.

Do not re-plan Phases 0–8 as if they were open. Do not re-plan the ISO MVP as if catalogs were still empty.
