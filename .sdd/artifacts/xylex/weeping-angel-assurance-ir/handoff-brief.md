# Handoff brief

> **Repo / SHA:** `floris-xlx/weeping-angel` `main` `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7`  
> **Scores / P0:** IR boundaries ~9/10; semantic completeness 55–60%. P0 **OWN-001**. Do not self-label this pack 9–10.  
> **Ownership:** `weeping-angel-assurance-ir` writes definitions. Framework writes compile. Facade writes orchestration. Scanner writes security documents.  
> **Must-nots:** provider types in IR; framework fields on `Control`; findings as compliance results; inferred equivalence; catalogs / hosted collectors / test DSL in this program.  
> **First order:** implement Phase 1 — split `lib.rs` (zero JSON change) then `try_new` (IR-001 RED → GREEN).  
> **Verify:** `cargo test --workspace --features demo`  
> **DoD (next ship):** workspace green after split; empty IDs rejected; ACT-001…015 still green.
