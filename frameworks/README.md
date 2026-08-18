# Weeping Angel framework packs

Versioned, deterministic, network-free catalogs.

Schema: `weeping-angel/framework-pack/v1`

Packs are structural. They store identifiers, mappings, applicability,
automation class, and legally safe short titles. They do **not**
redistribute protected ISO/IEC normative wording.

```text
frameworks/<id>/<version>/
  manifest.toml
  requirements.toml
  mappings.toml
  applicability.toml   # optional
  metadata.toml        # canonical controls + tests
```

A structural pack must compile and assess. Licensed or user-supplied
narrative can be layered later via `FrameworkContentProvider`.

Validate:

```bash
weeping-angel assurance framework validate frameworks/iso-27001/2022
```

Shipped: `iso-27001/2022` (StructuralOnly ISO 27001 readiness pack) and
`wa-baseline/1` (thin canonical baseline). Packs never contain GitHub/AWS
types or ISO normative clause text. Digest: `FrameworkPackDigest`.
