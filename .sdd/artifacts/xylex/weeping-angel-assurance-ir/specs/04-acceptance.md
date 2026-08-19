# 04 Acceptance

Copy of elite DoD 1–15. Gate command:

```powershell
cargo test --workspace --features demo
```

Plus after Phase 6:

```powershell
cargo test --features demo --test sdd_assurance_runtime_target --test sdd_compliance_ir_target
```

DoD is not “compiles.”
