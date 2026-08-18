# M-02 Gate evidence

| Gate | Command | Expected | Status |
| --- | --- | --- | --- |
| Spine | `cargo test --workspace --features demo` | ACT-001…015 COL-001…006 green | UNVERIFIED this authoring session |
| IR split | same after Phase 1 files exist | green, JSON unchanged | Open |
| IR-001 | `cargo test --test sdd_compliance_ir_target` | empty IDs fail | Open |
| Goldens | fixture tests | fail on key rename | Open |
| Dual-core | `rg "pub struct Assessment" crates` | one definition in IR | Open |
