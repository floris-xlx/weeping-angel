# ADR 0002 — ISO 27001 automated-assurance vertical (first real framework pack)

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-18 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). |
| Spec | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](../sdd/iso-27001-automated-assurance-mvp.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Planning baseline | `8c0f36ed873c51a21aa3e6d377d2fdbc4bb458d7` |
| IR revision consumed | `assurance-ir/v1`. Canonical types were not forked. `Mapping` gained `relation` + `rationale` on the same type. |
| Tests | `sdd_iso27001_assurance_target` GREEN (ISO/EVD/CTL/GH + MVP assess); `sdd_iso27001_assurance_baseline` superseded. ACT-001…015 and COL-001…006 remain GREEN. |

## Context

ADR 0001 delivered an inwardly extensible assurance **spine**: six crates, fail-closed compile, fixture collector, presence/freshness control tests, and five frozen invariants. Profile catalogs were empty (`stub_catalog` → `[]`). Facade `assess` compiled a hard-coded stub assessment. There was no assurance CLI.

The product needed the **first genuinely useful automated ISO 27001:2022 readiness assessment** without becoming a Vanta-style pile of `github → ISO check` shortcuts, and without taking ownership of the canonical Compliance IR (a concurrent program).

Constraints that remain law from ADR 0001:

- INV-1…5 (finding ≠ result; collector ≠ compliance; framework/control-test offline and provider-blind; no inferred equivalence).
- ACT-001…015 and COL-001…006 stay green.
- `Control` stays framework-neutral. Collectors advertise evidence types, never frameworks.

Questions this decision answered:

1. Where does ISO content live, if not compiled into Rust structs and not copied from the standard?
2. Who owns evidence persistence, and what must it refuse to store?
3. How do control tests grow beyond type-presence without becoming a script host?
4. How does GitHub enter the system without teaching tests about GitHub?
5. What language is allowed on reports?

## Decision

Build a **deep ISO 27001 vertical** as a separate implementation program that consumes Compliance IR **through agreed contracts**. Do not redesign IR types except tiny compile compatibility. If concurrent IR work lands `AssessmentDefinition` or `SubjectSelector` as first-class IR documents, rebase onto those types rather than introducing a competing definition.

This is what shipped.

### 1. Versioned framework packs, not compiled-in catalogs

ISO 27001:2022 (and a thin `wa-baseline/1` pack) ship as immutable, deterministic, network-free packs:

```text
frameworks/iso-27001/2022/{manifest,requirements,mappings,applicability,metadata}.toml
```

Schema: `weeping-angel/framework-pack/v1`. Loader: `weeping-angel-framework::pack` (`load_framework_pack`, `validate_framework_pack`, `assessment_from_pack`). Compile still emits only the current internal `CompiledFramework`. `stub_catalog(profile)` remains `[]`; the ISO facade loads the on-disk pack instead of compiling Annex A into Rust structs.

`FrameworkPackDigest` is SHA-256 over canonical pack content and is recorded on readiness snapshots and serialized assessment reports.

### 2. Structural public content; no ISO normative text

The public pack is `content_mode = StructuralOnly`. It stores identifiers, mappings, classification, applicability, automation class, evidence expectations, and legally safe short titles.

Content modes exist as `FrameworkContentProvider`: `StructuralOnly` | `LicensedContent` | `UserSuppliedContent`. A structural pack compiles and assesses. Protected ISO clause / Annex A wording is not committed.

### 3. Canonical controls, ISO mappings — never ISO-prefixed controls

Reusable library (`source.branch-protection`, `access.mfa.privileged`, `security.tls`, … — twenty-plus controls in `metadata.toml`). ISO requirement ids (`iso27001:4.1`, `iso27001:a.8.25`, …) map **into** that library.

`Mapping` on `assurance-ir/v1` now carries:

```text
direction, completeness, relation, rationale
```

`MappingRelation`: `Equivalent` | `Satisfies` | `PartiallySatisfies` | `Supports` | `Related`. Default is derived from completeness. Non-trivial mappings require a rationale. A later SOC 2 pack should map onto the same controls. GitHub-specific or `iso27001.*` control/test ids are rejected.

### 4. Evidence ledger owns observations, never conclusions

`EvidenceEnvelope` remains immutable (`evidence/v1`). Seal still rejects credential keys and compliance narratives. The envelope now also records `evidenceId`, `artifactRef`, `collectionRunId`, `contentDigest`, `sensitivity`, `scope`, and `supersedes`.

Persistent append-only ledger (`weeping-angel-evidence::ledger::EvidenceLedger`): SQLite file or in-memory. Operations: `append`, `get`, `query`, `latest`, `for_subject`, `for_type`, `for_collection_run`, `within_window`, `supersede`, `record_collection_run`. Duplicate identical envelopes are not stored twice.

**Forbidden** ledger APIs: `set_compliant`, `set_control_status`.

Every envelope traces to a `CollectionRun`. Artifacts are digest-addressed (`EvidenceArtifactRef`) and sensitivity-tagged.

### 5. Bounded Control-Test AST, typed values, provider-blind selectors

`TestExpr` is a closed AST (exists/missing/compare/count/freshness/coverage/boolean combinators/`ManualReview`). No JS/Lua/Python/shell/Rhai/WASM.

Facts are typed `EvidenceValue` on `EvidenceObservation` ([ADR 0003](0003-typed-evidence-canonical-serialization.md)). String `with_fact` remains for collector compatibility and historical digest-stable JSON strings. The evaluator compares stored types (`Integer(2)` is not the string `"2"`). Selectors name evidence type + subject + field + freshness — never a collector id.

`Effectiveness` is specific: `Effective` | `Ineffective` | `PartiallyEffective` | `NotApplicable` | `NotTested` | `InsufficientEvidence` | `StaleEvidence` | `ManualReviewRequired` | `ExceptionApproved` | `Inconclusive`. Missing/stale/manual-without-attestation still cannot be `Effective`.

`ControlTestResult` adds `evidenceRefs`, `missingEvidence`, `evaluatedAt`, `testVersion`, `inputDigest`, and optional `duration` (duration is not part of semantic identity).

### 6. Collectors emit facts; GitHub is the first hosted collector

`CollectorDescriptor` grew `providerFamily`, `subjectTypes`, `capabilities`, and `requiredPermissions`. It must never grow `frameworks` / `iso_controls`.

`EvidenceCollector` remains synchronous (`collect(scope) → Vec<EvidenceEnvelope>`). `CollectionRequest` / `CollectionBatch` exist for run provenance. WASM-hostile `Send + Sync` bounds were not added.

Shipped collectors:

| Collector | Id | Role |
| --- | --- | --- |
| `FixtureCollector` | caller-supplied | Deterministic tests (COL-001…006) |
| `GitHubCollector` | `collector.github` | First production collector; provider types do not escape `crates/weeping-angel-collector/src/github/` |
| `LocalCollector` | `collector.local` | Structural local files (`CODEOWNERS`, policy, workflow presence) |
| `ManualEvidence` | `collector.manual` | Explicit attestation (`attested-by` required; never synthesized) |

GitHub evidence types are canonical (`source.branch.protection`, `source.branch.required_reviews`, …), not `github.*`. HTTP 403 is `PermissionDenied` (downstream `InsufficientEvidence`), not boolean false and not `Ineffective` unless the permission itself is under test. Tokens are redacted and never persisted.

Required GitHub permissions advertised by the descriptor: `contents:read`, `administration:read`, `metadata:read`.

Scanner bridge remains one-way (`security_finding` plus a `canonical_type` fact). Absence of findings is not positive evidence. `security.no_vulnerabilities` is not a passable evidence type.

### 7. Readiness and SoA are projections, not certificates

`FrameworkReadinessSnapshot` plus `project_soa` (JSON-serializable SoA with applicability rationale preserved from the pack). Aggregation is explicit: a partial mapping cannot fully satisfy a requirement (`partially covered`).

Reports and the CLI must say this is a **readiness assessment, not certification**, and must never emit `certified` / `compliant` / `audit passed` from automation. `AssessmentReport` serialization adds `disclaimer`, `banner`, pack digest, and coverage counts.

`AssessmentRun` is an immutable snapshot record. `compare(previous, next)` detects at least: control became effective/ineffective, stale evidence, plus reserved fields for subject/requirement/exception deltas.

CLI family (clap): `weeping-angel assurance {framework,collect,evidence,assess,result,compare,soa}` in this slice. Catalog validate/stats/inspect later landed under [ADR 0003](0003-canonical-assurance-catalog-v1.md). Non-catalog arms still print the non-certification banner (exit 0); their execution path is the library facade (`AssuranceEngine::assess`) plus `project_readiness` / `project_soa` / `compare`.

### 8. Dual-suite stays mandatory

Registered in root `Cargo.toml`:

```text
sdd_iso27001_assurance_baseline   # superseded / ignored after the vertical landed
sdd_iso27001_assurance_target     # normative ISO-001…010, EVD-001…010, CTL-001…012, GH-001…012 + MVP assess
```

ACT-001…015 and COL-001…006 remain the spine contract.

## Consequences

**Positive**

- ISO 27001 is the proving vertical for the Athena-shaped split: packs + mappings for regimes, collectors for providers, one engine in the middle.
- Copyright risk is bounded; structural packs remain useful.
- Evidence history is auditable without letting storage become a GRC conclusions database.
- Control tests can express real thresholds without becoming an execution sandbox.
- Concurrent IR work can continue; this program rebases rather than competing.

**Negative / cost**

- Structural packs will not read like the ISO standard; operators need licensed or user-supplied text for narrative clauses.
- SQLite + local artifact store is an operational surface (paths, permissions, size limits).
- `ControlTestResult` grew optional fields so ACT-012 serde remains honest (`deny_unknown_fields` is not on the expanded result).
- GitHub token/permission UX is easy to get wrong (403 → false); the collector must keep failing closed.
- Mapping quality (relation + rationale) is human work; review fixtures are generated, not source.
- The CLI command family is wired in clap; full subcommand dispatch beyond the banner is still library-first.

**Rejected alternatives**

- Compiling Annex A into Rust structs or copying ISO normative text into git.
- `iso27001.branch-protection` controls and `iso27001.a.x.y.github.*` tests.
- Letting the GitHub collector (or scanner) write ISO/control status.
- Storing compliance conclusions on the evidence ledger.
- Embedding Rhai/JS/WASM in control tests.
- One mega-PR that also redesigns Compliance IR.
- Shipping SOC 2 / GDPR / cloud collectors in this program.
- A single readiness percentage or “ISO 27001 compliant” banner.

## Access and security

- No credentials in evidence payloads, artifacts (unredacted), logs, or the SQLite file as stored tokens.
- GitHub token is read from the environment / config at collect time and never written back.
- Framework pack loader is local-filesystem only: no HTTP fetch of packs in the compiler (no SSRF).
- Framework and control-test crates remain without network/SDK dependencies.
- Artifact writes are size-bounded and path-traversal safe.
- Expression evaluation is a closed AST.

## Deferred (not this decision)

- Production packs for SOC 2, GDPR, NIS2, DORA, ISO 27701.
- Additional hosted collectors (AWS, Azure, GCP, Cloudflare, …).
- Certification-ready formal SoA / ISO 27007 audit product.
- PostgreSQL / remote object-storage ledger backends.
- HTML reports and dashboards.
- Full CLI dispatch for every non-catalog `assurance` subcommand (library assess is the MVP execution path). Catalog dispatch is ADR 0003.
- Concurrent Compliance IR redesign (`AssessmentDefinition` as an IR document, richer `SubjectSelector` ownership).
- Versioned canonical catalog outside the ISO pack (landed as [ADR 0003](0003-canonical-assurance-catalog-v1.md)).

## Related

- Spec SSOT: [`docs/sdd/iso-27001-automated-assurance-mvp.md`](../sdd/iso-27001-automated-assurance-mvp.md)
- Spine SDD: [`docs/sdd/assurance-runtime-spine.md`](../sdd/assurance-runtime-spine.md)
- ADR 0001: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](0001-inwardly-extensible-assurance-runtime.md)
- Canonical catalog (accepted): [`docs/adr/0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence (accepted; supersedes string-only facts): [`docs/adr/0003-typed-evidence-canonical-serialization.md`](0003-typed-evidence-canonical-serialization.md)
- Packs: [`frameworks/README.md`](../../frameworks/README.md)
- Concurrent IR (do not own): [`docs/sdd/xylex/weeping-angel-assurance-ir/`](../sdd/xylex/weeping-angel-assurance-ir/)
