# Dual-suite evidence log

| Date | Suite | Result | SHA | Notes |
| --- | --- | --- | --- | --- |
| 2026-08-18 | `sdd_assurance_runtime_target` | documented GREEN in spine SDD | `8c0f36e` | not re-run in authoring session — UNVERIFIED |
| 2026-08-18 | `sdd_compliance_ir.target` | absent | `8c0f36e` | expected RED once written on current IR |
| 2026-08-18 | `sdd_compliance_ir_target` | RED (compile) | working tree | missing `try_new` and IR types |
| 2026-08-18 | `sdd_compliance_ir_target` | GREEN 26 passed | working tree | IR-001…025 + goldens |
| 2026-08-18 | `sdd_assurance_runtime_target` | GREEN 21 passed | working tree | ACT/COL still green |
| 2026-08-18 | `sdd_iso27001_assurance_target` | GREEN 49 passed | working tree | ISO vertical still green |
