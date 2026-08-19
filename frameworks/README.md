# Weeping Angel framework packs

Versioned, deterministic, network-free **regime** packs (requirements + mappings).

They are **not** the reusable control library. Canonical controls, evidence, and tests live in `catalog/canonical/v1` (`weeping-angel/canonical-catalog/v1`; see [`docs/sdd/canonical-assurance-catalog-v1.md`](../docs/sdd/canonical-assurance-catalog-v1.md)). ISO 27001:2022 mappings target catalog IDs (`control.identity.*`, landed `control.source.*`); pack-local slivers are retired ([`docs/sdd/iso-27001-canonical-remap.md`](../docs/sdd/iso-27001-canonical-remap.md) §13, [ADR](../docs/adr/0003-iso27001-canonical-remap.md)). Organizational / supplier / incident ISO clauses stay unmapped; the governance catalog family is separate ([`docs/sdd/governance-canonical-assurance-catalog.md`](../docs/sdd/governance-canonical-assurance-catalog.md)).

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
  metadata.toml        # pack annotations only (not a control library)
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
