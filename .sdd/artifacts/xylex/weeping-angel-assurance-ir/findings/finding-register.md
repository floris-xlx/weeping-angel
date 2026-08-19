# Finding register — weeping-angel-assurance-ir

As of `main` `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7`. Catalog IDs from `catalogs/05-ISSUE-CATALOG.md`.

| ID | Sev | Title | Status | Owner | Close phase |
| --- | --- | --- | --- | --- | --- |
| OWN-001 | P0 | Assessment input dual write cores | Closed | IR | Phase 5 — `AssessmentDefinition` is the SSOT; framework re-exports |
| DAT-002 | P1 | Empty persisted IDs accepted | Closed | IR | Phase 1 — `try_new` / IR-001 |
| DAT-013 | P1 | Unvalidated ID constructors | Closed | IR | Phase 1 |
| CON-006 | P1 | Digest/schema not independently versioned | Closed | IR | `CanonicalizationVersion` + `typed_canonical_digest` |
| DAT-005 | P1 | No structural IR validation | Closed | IR | `ValidateIr` / IR-005…021 |
| CON-003 | P1 | Assessment scope is facade-only | Partial | facade | IR `AssessmentScope` exists; facade still translates assets |
| DAT-022 | P2 | Applicability stage inert | Partial | framework + IR | Static `Never` filtered; predicates still unknown |
| CI-005 | P2 | No IR compatibility freeze | Closed | docs | Goldens + N/N-1 policy in elite pack |

SSOT JSON: [`finding-register.json`](finding-register.json).
