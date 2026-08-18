# Weeping Angel framework packs

Versioned, deterministic, network-free **regime** packs (requirements + mappings).

They are **not** the reusable control library. Canonical controls, evidence, and tests live in `catalog/canonical/v1` (`weeping-angel/canonical-catalog/v1`; see [`docs/sdd/canonical-assurance-catalog-v1.md`](../docs/sdd/canonical-assurance-catalog-v1.md)). The IAM family (`control.identity.*`) is catalog content ([`docs/sdd/iam-canonical-assurance-catalog.md`](../docs/sdd/iam-canonical-assurance-catalog.md)); packs still map ISO onto pack-local stubs such as `source.branch-protection` and `access.mfa.privileged` until Prompt 12 remaps.

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
weeping-angel assurance catalog validate
```

Shipped: `iso-27001/2022` (StructuralOnly ISO 27001 readiness pack) and
`wa-baseline/1` (thin canonical baseline). Packs never contain GitHub/AWS
types or ISO normative clause text. Digest: `FrameworkPackDigest`.
