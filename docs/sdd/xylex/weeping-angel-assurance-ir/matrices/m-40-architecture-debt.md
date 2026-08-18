# M-40 Architecture debt

| ID | Title | Owner | Removal | Status | Enforcement |
| --- | --- | --- | --- | --- | --- |
| AD-001 | `Assessment` owned by framework | framework | Phase 5 | Open | `rg "pub struct Assessment"` |
| AD-002 | Facade collector-shaped scope | facade | Phase 5 | Open | IR-018 |
| AD-003 | `canonical_digest` over raw Serialize | IR | Phase 6 | Open | IR-015 IR-016 |
| AD-004 | empty typed IDs | IR | Phase 1 | Open | IR-001 |
| AD-005 | applicability identity | framework | later compile + Phase 2 IR | Open | IR-010 |
| AD-006 | IDs without records | IR | Phase 3 | Open | type exists |
| AD-007 | req-only ComplianceGraph | IR | Phase 4 | Open | IR-006 |
| AD-008 | no golden fixtures | IR | Phase 6 | Open | fixture tests |
| AD-009 | no control-test predicate AST | control-test | later program | Accepted | out of scope |
| AD-010 | empty framework catalogs | framework | spine Phases 9–14 | Accepted | stub_catalog |
