# Assessment — weeping-angel Compliance IR at `8c0f36e`

| Field | Value |
| --- | --- |
| Product | `weeping-angel-assurance-ir` |
| Repo | `floris-xlx/weeping-angel` |
| SHA | `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7` |
| Date | 2026-08-18 |
| Verdict | **Architecturally correct, semantically thin.** Next program is deepen-IR, not redesign. |

## Goal

Score the current Compliance IR against a production-grade canonical assurance IR. Do not implement.

## Repositories

| Role | Path |
| --- | --- |
| Target | `floris-xlx/weeping-angel` @ `8c0f36e` |
| IR | `crates/weeping-angel-assurance-ir/src/{lib.rs,crosswalk.rs}` |

## Context

Phases 0–8 spine is implemented. ACT-001…015 and COL-001…006 exist. IR types are framework-neutral newtypes plus five thin documents. Eight entity kinds exist only as IDs.

## Working assumptions

Evidence is file+line at this SHA. Scores are **dimension ratings**, not a plan band. Self-score of this assessment as “elite 9/10” is forbidden.

## Ownership

| Concern | Canonical owner | Must not own |
| --- | --- | --- |
| IR definition writes | `weeping-angel-assurance-ir` | framework, facade, scanner |
| Assessment document writes | IR (target) / framework (today) | scanner |
| compile writes | `weeping-angel-framework` | IR |
| SemanticFinding writes | root `src/` | IR |

## Overall assessment

The hardest boundary problem is solved. The IR is not yet the semantic authority downstream crates can compile against without inventing domain meaning (`resolve_applicability` is identity; `Assessment` lives in the compiler crate).

## Scorecard

| Dimension | Score | Evidence |
| --- | --- | --- |
| Typed identities | 9/10 | newtypes exist; construction unrestricted (`lib.rs` L15–37) |
| Schema versioning | 8/10 | `ASSURANCE_IR_SCHEMA` on documents; no independent canonicalization version |
| Deterministic digest | 8/10 | SHA-256 of serde JSON; no domain prefix |
| Requirement model | 5/10 | framework id+version+title+description only |
| Canonical Control | 3.5/10 | id+title+description only |
| Mapping model | 4.5/10 | direction+completeness; no relation/provenance/id |
| Evidence requirement | 3.5/10 | id+type only |
| Planned test | 3.5/10 | id+control+kind+required+break_on |
| Applicability | 1/10 | compile stage clones all requirements |
| Subject / resource | 0.5/10 | no `SubjectSelector` |
| Control implementation | 0.5/10 | `ControlImplementationId` only |
| Ownership | 0.5/10 | no `PrincipalRef` |
| Risk / exception / asset / identity / vendor / processing | ID only | typed IDs, no records |
| Crosswalk metadata | 3/10 | req–req graph; ACT-005 correct |
| Extension / versioning strategy | 2.5/10 | no `ExtensionMap`; no N/N-1 policy |
| Validation / invariants | 4.5/10 | ACT-001…015; no `ValidateIr` |
| Boundary architecture | 9/10 | crate graph + INV-1…5 hold |

Composite semantic completeness: **55–60%** of a freeze-ready IR. Boundary architecture is higher.

## Production verdict

**Do not freeze `assurance-ir/v1` as stable.** Freeze after Phases 1–6 of the elite plan. Catalogs must not start until DoD 1–15 are true.

## Findings → catalog IDs

| Catalog | Sev | Status | Evidence |
| --- | --- | --- | --- |
| OWN-001 | P0 | Open | `Assessment` in `weeping-angel-framework` L101–111 |
| DAT-002 | P1 | Open | `typed_id!` accepts `""` |
| DAT-013 | P1 | Open | no `try_new` |
| CON-006 | P1 | Open | digest = raw serde; one schema string |
| DAT-005 | P1 | Open | no dangling-ref validator |
| CON-003 | P1 | Open | facade `AssessmentScope` is the only scope type |
| DAT-022 | P2 | Open | `resolve_applicability` L265–271 |
| CI-005 | P2 | Open | no compatibility policy on IR |

## Handoff brief

See [`handoff-brief.md`](handoff-brief.md).
