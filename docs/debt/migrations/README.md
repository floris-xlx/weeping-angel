# Consolidation migration contracts

Architect-owned work packages. Implementation agents **do not** invent architecture.

```text
architecture decision  →  implementation conforms  →  tests prove  →  debt verified
```

Never: implementation differs → agent edits `architecture.toml` → green.

- Index / lanes / waves: [`QUEUE.toml`](QUEUE.toml)
- One file per debt id: `DUP-NNN.toml`
- Executors consume the contract as the **only** allowed scope
- Close law stays in [`../structural-duplication.toml`](../structural-duplication.toml)

Pipeline: `/workflow xylex-sdd-consolidation-pipeline`

Hierarchy (max concurrent implementers: 6, default 3):

```text
Architect (this directory + architecture/*.toml + debt register)
    Inventory (measure)     Validation (disprove, no fixes)
                Work package queue
         Lane A / C / D / E  (limited parallel)
         Lane B serial only
                Integration (cherry-pick + cargo xtask guard)
```

Monotonic on every cleanup commit: duplicate types, public symbols, root test binaries, duplicate helpers, production unwrap/expect **must not increase**; architecture violations / new aliases / new SSOTs **= 0**.
