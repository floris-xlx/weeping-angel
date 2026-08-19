# Prompt 2 — Canonical Catalog, Framework and Readiness Trust Boundary

Work in `floris-xlx/weeping-angel` on the canonical assurance trust boundary. This prompt is designed to run concurrently with Prompts 1, 3 and 4. Stay strictly within the ownership boundary below.

## Objective

Eliminate the P0 ambiguity around catalog SSOT, framework expression preservation, fail-closed pack parsing, framework digest integrity, and readiness projection ownership. The result must make it impossible for two representations of the same assurance semantics to drift silently.

## Exclusive ownership boundary

You may modify:

- `crates/weeping-angel-canonical-catalog/**`
- `crates/weeping-angel-framework/**`
- readiness-specific code in `crates/weeping-angel-assurance/**`, especially `readiness.rs` and directly related modules
- `catalog/**`
- `frameworks/**`
- catalog/framework/readiness-specific target tests under `tests/contracts/**`
- relevant ADR/spec text only when necessary to accurately describe the implemented product semantics; avoid repository-integrity metadata owned by Prompt 1

Do not modify `xtask/**`, `architecture/**`, `docs/debt/register.toml`, temporal/lineage/evidence persistence/SoA implementation owned by Prompt 3, or broad test/schema/README hygiene owned by Prompt 4.

## Required work

1. Establish one canonical catalog SSOT. Inventory every path that can define or reinterpret control, evidence, test, relation, applicability, or catalog identity. Remove or convert secondary representations into generated/projection forms. There must be exactly one authoritative semantic source for a catalog item.

2. Make canonical catalog loading deterministic and fail closed. Duplicate IDs, malformed records, unknown relation kinds, invalid references, unsupported schema versions, impossible mappings, and partial parse failures must be explicit errors. Do not silently drop records.

3. Preserve framework expressions losslessly. Framework-pack parse/serialize/reload must preserve the complete expression tree and semantic distinctions. Avoid normalization that changes `all`/`any`/negation/threshold/partial-support meaning. Add round-trip and adversarial fixtures.

4. Make framework pack parsing fail closed. A malformed manifest, mapping, expression, control reference, evidence/test reference, version, or digest field must prevent assessment with a typed error. No best-effort fallback to a weaker interpretation.

5. Redesign framework digesting around canonical semantic content rather than incidental filesystem ordering or formatting. Digest results must be deterministic across whitespace/comments/path enumeration differences where semantics are identical, and must change for any semantic change that could affect assessment output.

6. Bind assessment/readiness execution to pinned catalog and framework identities. The result must carry enough identity to prove exactly which canonical catalog and framework pack semantics produced it. Avoid reloading mutable current files during serialization/reporting.

7. Make readiness projection a single semantic owner. Locate all code that computes, translates, labels, or infers readiness/effectiveness status. Consolidate the authoritative projection logic into the intended assurance layer. Callers should invoke it rather than reimplement status rules.

8. Preserve the critical semantic distinctions already established by the project: evidence is not framework status; scanner findings are not compliance results; accepted risk is not remediation; document existence is not operational effectiveness; missing coverage is not success; `Equivalent`, `PartiallySatisfies`, and `Supports` must remain distinct; framework packs project onto canonical controls rather than creating hidden alternative controls.

9. Add strong mutation/negative tests for catalog duplication, malformed packs, unknown IDs, altered expression trees, digest instability, digest collision-by-normalization mistakes, stale pin usage, and duplicated readiness calculations.

10. Keep provider/framework-specific naming out of canonical evidence where the architecture requires provider-neutral facts. A collector-facing or catalog-facing API must not leak ISO-specific status into evidence.

## Concurrency contract

Prompt 1 owns `xtask` and will implement repository-level guards. Do not edit the guard implementation. Expose clean APIs/metadata that Guard 05–08 can consume if needed, but leave repository-policy wiring to Prompt 1.

Prompt 3 owns temporal selection, lineage reconstruction, evidence current/latest semantics, persistence, and SoA. Do not refactor those modules as part of readiness cleanup unless a tiny interface adjustment is unavoidable; keep changes minimal and backwards compatible.

Prompt 4 owns broad baseline retirement and repository hygiene. Only touch contract tests directly tied to catalog/framework/readiness semantics.

## Acceptance criteria

- There is one authoritative canonical catalog semantic source.
- Catalog loading rejects duplicates, invalid references and unsupported/malformed input.
- Framework parse/serialize is expression-lossless.
- Framework parsing fails closed.
- Framework digest is semantic, deterministic and pinned into assessment identity.
- Readiness/effectiveness projection has one authoritative implementation path.
- Serialization/reporting cannot silently substitute current pack/catalog files for pinned execution semantics.
- Negative tests prove malformed or ambiguous inputs fail rather than degrade.
- Existing public APIs remain compatible unless a breaking change is required to close a correctness hole and is explicitly documented.
- `cargo fmt --all -- --check` passes.
- Relevant catalog/framework/readiness tests pass.
- Full workspace compile succeeds.

Do not solve drift by duplicating types or adding compatibility branches that create a second semantic implementation. Prefer adapters into one authoritative model.