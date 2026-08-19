# SDD: Vulnerability and Scanner Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Specified — not implemented** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 06 — vulnerability / exposure / remediation / dependency-risk / secret-exposure / scanner-derived assurance |
| Source prompt | [`docs/prompts/canonical-assurance-v1/06-vulnerability-catalog.md`](../../prompts/canonical-assurance-v1/06-vulnerability-catalog.md) |
| Planning / characterization SHA | `e2def07ee4c3ec265a6b5fee116931f0b2c9ce94` (`main`, 2026-08-19) |
| Dual-suite (register at implement) | `sdd_vulnerability_catalog_baseline` · `sdd_vulnerability_catalog_target` |
| ADR | Draft [`docs/adr/0003-vulnerability-canonical-assurance-catalog-draft.md`](../../../adr/0003-vulnerability-canonical-assurance-catalog-draft.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../../contracts/assurance-runtime.md) |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](../canonical-assurance-catalog-v1.md) |
| Prompt-02 / 03 (consumed) | [`docs/sdd/typed-evidence.md`](../typed-evidence.md), [`docs/sdd/population-runtime.md`](../population-runtime.md) |
| Prompt-04 sibling (do not overwrite) | [`docs/sdd/iam-canonical-assurance-catalog.md`](../iam-canonical-assurance-catalog.md) |
| Spine / ISO law | [`docs/sdd/assurance-runtime-spine.md`](../assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](../iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Isolation | worktree |
| Mode | balanced (SDD run is `strict`) |
| Transition | `auto` → **replacement** (absence of canonical vuln family superseded; ISO sliver kept) |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **vulnerability catalog slice** (Prompt 06). It does not replace Prompt 01 catalog infrastructure, Prompt 02 typed evidence, Prompt 03 population runtime, or Prompt 04 IAM content. Prompts 01–04 have landed; this slice consumes their loader, `EvidenceValue`, population evaluator, and catalog tree and **must not** invent a second copy.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

Core law for this slice: **a scanner finding is evidence, not a compliance result.** Empty finding lists are not positive assurance unless scan coverage is authoritative and current.

---

## 1. Problem / user-visible goal

Organizations need to assess vulnerability management, exposure handling, remediation SLAs, secret exposure, dependency risk, and scan coverage using **provider-neutral** canonical controls. On SHA `e2def07…` the only vulnerability-adjacent catalog content is:

- a **thin ISO-pack hybrid** control `vulnerability.remediation` whose test requires some `security.vulnerability.present` envelope (existence / hybrid), not “no critical finding exceeds SLA”;
- a one-way scanner bridge (`weeping-angel-assurance::bridge`) that emits `security_finding` observations with a `canonical_type` of `security.vulnerability.present` / `security.secret.exposure` / `security.dependency_confusion_risk` — **facts about a hit**, not remediation state, owner, exception, coverage, or freshness;
- Weeping Angel / Codex Security scan contracts (`findings.json`, `coverage.json`, `Finding` / `ScanReport`) that must remain scanner documents, not catalog IDs.

The canonical catalog at `catalog/canonical/v1/` lists only `fixture.example.toml` and the IAM family (`identity.toml`). There is no `control.vulnerability.*` library, no `evidence.vulnerability.*` / `evidence.secret.*` / `evidence.dependency.*` contracts, and no deterministic vulnerability-population fixtures. Prompt 03 already names the sentence “no critical vulnerability exceeds SLA” as a handoff example, but evaluates it today as `NoneSubjects` on **branch-protection** fixtures — not findings.

A future Weeping Angel / Snyk / Dependabot / Trivy collector therefore has nowhere canonical to emit coverage, SLA age, owner, exception, or secret-exposure facts. ISO remapping (Prompt 12) has nothing stable to map `vulnerability.remediation` onto.

**User-visible goal:** a coherent vulnerability catalog (~15–25 independently assessable controls) that can consume evidence from Weeping Angel or any future scanner, evaluate remediation and coverage deterministically, distinguish failure from missing evidence and exceptions, and pass catalog plus workspace validation.

This slice does **not** claim ISO/SOC 2/NIS2 coverage. Framework remapping is Prompt 12. It does **not** implement provider collectors or redesign scanner engines.

---

## 2. Dependencies and fail-closed blockers

| Prompt | Owns | On `e2def07…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules, CLI validate/stats/inspect | **Landed.** | Add vulnerability TOML + manifest lines. Do not invent a second loader/validator/digest. |
| 02 typed evidence | Typed `EvidenceValue`, canonical serialization, seal rules | **Landed.** | Declare required fact *names* and semantic types. No second value enum. No credential/secret material in facts. |
| 03 population runtime | Subject populations, `AllSubjects` / `NoneSubjects` / `CoverageAtLeast`, missing/stale/fail split, `ExceptionApproved` for subject-scoped exceptions | **Landed.** | Declare population-based tests. **Do not locally reimplement coverage math.** Prefer fact-driven finding status; reuse IR `Exception` when binding approved exceptions. |
| 04 IAM | `control.identity.*` family + identity fixtures | **Landed.** | Do not edit identity TOML except if a shared fixture convention needs a documented one-line pointer. Do not overwrite IAM SSOT. |
| 05 SDLC | `control.source.*` / CI / supply-chain (includes “scanning enabled”) | **Not landed.** | Do **not** implement SDLC. Vulnerability slice owns finding/SLA/coverage/secret-*exposure* / dependency-*risk*, not “repo has branch protection / required reviews.” If both later exist, IDs stay in different namespaces. |
| Scanner engines / Codex Security contract | `src/finding.rs`, `src/engines/*`, `src/depcheck/*`, `src/checks/secrets.rs`, `codex-security/schemas/*` | Live | Design fixtures **compatible** with those outputs. Do not modify scanner engines except a narrowly required compile/test adapter. A later prompt expands the scanner bridge. |
| ISO pack | `vulnerability.remediation` + `test.vulnerability.remediation` + `security.vulnerability.present` | Frozen | **Do not retarget mappings or rename pack IDs.** |

Rebase rule: follow the landed Prompt 01 file layout (`controls/*.toml`, `evidence/*.toml`, `tests/*.toml`, manifest `[files]`). Prefer adapting vulnerability content to those contracts over extending this slice’s scope.

---

## 3. Current behavior (characterization on `e2def07ee4c3ec265a6b5fee116931f0b2c9ce94`)

Inspected: `catalog/canonical/v1/`, `crates/weeping-angel-canonical-catalog`, `weeping-angel-control-test`, `weeping-angel-assurance-ir`, `weeping-angel-assurance/src/bridge.rs`, `frameworks/iso-27001/2022/`, `src/finding.rs`, `codex-security/schemas/`, `tests/sdd/*`, `docs/prompts/canonical-assurance-v1/06-vulnerability-catalog.md`.

### 3.1 Canonical catalog has no vulnerability family

`catalog/canonical/v1/manifest.toml` lists only:

```text
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests    = ["tests/fixture.example.toml", "tests/identity.toml"]
```

No `controls/vulnerability.toml` (or `secret.toml` / `dependency.toml`). Grep over `catalog/` finds no `control.vulnerability`, `evidence.vulnerability`, or `test.vulnerability` ids.

`CanonicalCatalog::{load,validate,digest}` is provider-blind and will load additional listed files without loader changes.

### 3.2 ISO-pack vulnerability sliver (must stay)

`frameworks/iso-27001/2022/metadata.toml`:

| Pack control id | Automation | Test | Required evidence |
| --- | --- | --- | --- |
| `vulnerability.remediation` | Hybrid | `test.vulnerability.remediation` (`hybrid`) | `security.vulnerability.present` |

IDs are **not** in the Prompt 01 `control.*` namespace. `sdd_iso27001_assurance_target` freezes prefix `vulnerability.` and expected id `vulnerability.remediation`. This slice must not break that suite by rewriting the ISO pack.

ISO mappings point A.8.8-adjacent clauses at `vulnerability.remediation` with partial completeness. Prompt 12 remaps; this slice does not.

### 3.3 Scanner bridge is not a vulnerability catalog

`weeping-angel-assurance::bridge` classifies `EngineHit` / `SemanticFinding` into `security_finding` observations. `canonical_type` may be `security.vulnerability.present`, `security.secret.exposure`, or `security.dependency_confusion_risk`. Facts are `rule_id`, `path`, `category`, `finding_id` — no severity age, owner, status, exception, coverage completeness, or SLA clock.

Contract law already documented: empty scan ≠ Effective control; do not emit `security.no_vulnerabilities` as a passable fact. This slice **reuses** that law and does not collapse scanner types into catalog IDs.

### 3.4 Scanner documents remain scanner documents

- CLI `Finding` (`src/finding.rs`): `id`, `title`, `severity` (info…critical), `url`, `module`, `description`, `remediation?`, `cwe?`, `evidence[]`, `found_at`. No lifecycle status, owner, exception, or population coverage.
- Codex Security completed-scan bundle: `findings.json` + `coverage.json` + `scan-manifest.json`. Coverage has `completeness` ∈ {complete, partial, unknown}. Findings are **not** a workflow-state database.
- Depcheck / secrets engines produce scanner findings, not `evidence.vulnerability.*`.

Fixtures for this slice must be **mappable** from those fields (severity, found_at, rule/module, coverage completeness) into canonical facts. The mapping adapter is out of scope except as comments / fixture documentation.

### 3.5 Population runtime can express the tests, but has no finding population

`TestExpr` already has `AllSubjects`, `NoneSubjects`, `CoverageAtLeast`, `CountWhere`, `FreshWithin`. `Effectiveness` includes `InsufficientEvidence`, `StaleEvidence`, `ExceptionApproved`, `ManualReviewRequired`. `Exception` binds `subjects` + `status` + `expires_at`.

`SubjectKind` has Asset, Repository, Application, Endpoint — **not** `Finding`. Finding populations are modeled as **Asset** subjects with stable `finding:<id>` identifiers, selected by evidence type `evidence.vulnerability.finding` (or secret/dependency types). Do **not** add `SubjectKind::Finding` unless a compile blocker makes Asset selectors unusable; prefer zero IR change.

`evaluate_coverage` special-cases approved break-glass privileged MFA. Vulnerability exceptions should be **fact-driven** (`evidence.vulnerability.exception` + finding `status`) and may additionally attach IR `Exception` records so `ExceptionApproved` can surface. Do not fork a second exception engine. If the evaluator cannot emit `ExceptionApproved` for finding subjects without a one-line adapter, document the adapter; otherwise evaluate via `status=exception-approved` vs `status=open` so SLA tests exclude excepted findings without calling them `resolved`.

### 3.6 What “vulnerability assessment” means today

A caller can compile the ISO pack and run `test.vulnerability.remediation`, which requires **some** `security.vulnerability.present` envelope (hybrid). It cannot:

- require current, complete scan coverage across an in-scope asset population;
- fail a critical finding older than SLA while passing one still inside SLA;
- distinguish `resolved` from `accepted-risk`, `false-positive`, and `exception-approved`;
- treat expired exceptions as open again;
- fail active secret exposure independently of “a secret scanner ran”;
- treat zero findings + unknown coverage as missing evidence.

The baseline suite therefore characterizes **absence of a canonical vulnerability catalog** and **presence of the ISO-pack sliver + scanner-bridge taxonomy**, not a working finding-population evaluator.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

Vulnerability domain content lands in the Prompt 01 catalog tree:

```text
catalog/canonical/v1/
  manifest.toml                 # add vulnerability (and optional secret/dependency) files
  controls/vulnerability.toml
  evidence/vulnerability.toml   # may include secret + dependency + software-inventory rows
  tests/vulnerability.toml
```

Split files (`evidence/secret.toml`, `evidence/dependency.toml`) are allowed if listed in the manifest. Do **not** add `control.vulnerability.*` to `frameworks/iso-27001/2022/metadata.toml`.

Deterministic fixtures:

```text
fixtures/assurance/canonical/v1/vulnerability/
  complete-clean-scan/
  critical-inside-sla/
  critical-overdue/
  critical-approved-exception/
  critical-expired-exception/
  incomplete-scan-coverage/
  stale-scan/
  unresolved-secret-exposure/
  duplicate-superseded/
  zero-findings-unknown-coverage/
```

Each fixture is a frozen evidence JSON set (IAM `healthy-org` shape: `fixture`, `collectedAt`, `authoritative`, `evidence[]`) plus optional IR `Exception` records. `collectedAt` is fixed (recommend `2026-08-19T12:00:00Z`) so SLA age is deterministic.

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.vulnerability.<slug>
evidence.vulnerability.<slug>
evidence.secret.<slug>
evidence.dependency.<slug>
evidence.asset.software-inventory
test.vulnerability.<slug>
test.secret.<slug>
test.dependency.<slug>
```

Reject in canonical vulnerability content (validator already rejects listed provider/framework segments; target suite **also** greps file text):

- scanner / provider tokens as ID segments or as the subject of a control: `snyk`, `dependabot`, `trivy`, `grype`, `aqua`, `nessus`, `qualys`, `tenable`, `wiz`, `prisma`, `sonarqube`, `checkmarx`, `semgrep`, `gitleaks`, `trufflehog`, `github`, `gitlab`, `azure-devops`, `osv`, `nvd` (NVD may appear only as a *fact* `advisory_source`, never as an ID segment);
- framework tokens in IDs or narrative (`iso27001`, `iso-27001`, `soc2`, `soc-2`, `nis2`, `dora`, `gdpr`);
- orphaned evidence types or tests;
- duplicate IDs;
- existence-only tests masquerading as population tests (see §4.5).

Correct: `control.vulnerability.critical-remediation-sla`. Incorrect: `control.snyk.critical-sla`, `test.iso27001.a.8.8`, `control.trivy.image-scan`.

Provider field names (`snyk_issue_url`, `dependabot_alert_number`) must not appear in evidence **type** ids. They may appear only inside a future collector’s private normalize step that **emits** canonical facts.

### 4.3 Control family (20 independently assessable controls)

Do not split these into micro-controls to inflate count. Titles and objectives are framework-neutral. Domain is primarily `vulnerabilityManagement`; secret-exposure may also use `secureDevelopment`; unsupported software may also use `assetManagement`.

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.vulnerability.periodic-scanning` | Periodic vulnerability scanning | Automated | asset / repository / application | `scan-run`, `scan-coverage` | `test.vulnerability.scan-current` |
| `control.vulnerability.source-code-scanning` | Source-code security scanning | Automated | repository | `scan-run` (class `sast`), `scan-coverage` | `test.vulnerability.source-scan-current` |
| `control.vulnerability.dependency-scanning` | Dependency vulnerability scanning | Automated | repository / application | `scan-run` (class `sca`), `dependency.vulnerability` | `test.dependency.no-critical-over-sla` (plus current SCA scan) |
| `control.vulnerability.container-scanning` | Container / image scanning | Hybrid | application / asset | `scan-run` (class `container`), `scan-coverage` | `test.vulnerability.container-scan-current` |
| `control.vulnerability.secret-exposure` | Secret exposure detection | Automated | repository / asset | `secret.exposure`, `scan-run` (class `secret`) | `test.secret.no-active-exposure` |
| `control.vulnerability.dependency-confusion` | Dependency-confusion / supply-chain exposure monitoring | Automated | repository / application | `dependency.confusion-risk` | `test.dependency.confusion-monitored` |
| `control.vulnerability.critical-remediation-sla` | Critical vulnerability remediation SLA | Automated | finding (`asset` / `finding:*`) | `finding`, `remediation-state` | `test.vulnerability.no-critical-over-sla` |
| `control.vulnerability.high-remediation-sla` | High vulnerability remediation SLA | Automated | finding | `finding`, `remediation-state` | `test.vulnerability.no-high-over-sla` |
| `control.vulnerability.finding-ownership` | Vulnerability ownership | Automated | finding | `finding`, `owner` | `test.vulnerability.findings-have-owner` |
| `control.vulnerability.remediation-tracking` | Remediation tracking | Automated | finding | `finding`, `remediation-state` | `test.vulnerability.remediation-state-present` |
| `control.vulnerability.unsupported-software` | Unsupported software / dependency handling | Hybrid | asset | `software-inventory` | `test.vulnerability.unsupported-software-handled` |
| `control.vulnerability.exposure-review` | Recurring exposure review | Hybrid | asset / organization | `exposure-review` (or reuse `scan-run` + review facts) | `test.vulnerability.exposure-review-current` |
| `control.vulnerability.internet-exposed-critical` | Internet-exposed critical finding handling | Automated | finding | `finding` | `test.vulnerability.no-open-internet-exposed-critical` |
| `control.vulnerability.risk-acceptance` | Risk acceptance for unresolved findings | Hybrid | finding | `finding`, `exception` | `test.vulnerability.risk-acceptance-recorded` |
| `control.vulnerability.exception-expiry` | Exception expiry | Automated | finding | `exception` | `test.vulnerability.exceptions-unexpired` |
| `control.vulnerability.scan-coverage` | Scan coverage across in-scope assets | Automated | asset / repository | `scan-coverage` | `test.vulnerability.scan-coverage` |
| `control.vulnerability.scan-freshness` | Scan freshness | Automated | asset / repository | `scan-run` | `test.vulnerability.scan-current` (shared) |
| `control.vulnerability.duplicate-superseded` | Duplicate / superseded finding handling | Automated | finding | `finding` | `test.vulnerability.duplicates-superseded` |
| `control.vulnerability.false-positive-separation` | False-positive / accepted-risk separation from resolved | Automated | finding | `finding`, `remediation-state` | `test.vulnerability.resolved-not-accepted-risk` |
| `control.vulnerability.scan-population-authority` | Authoritative scan population | Automated | organization / asset | `scan-coverage` | `test.vulnerability.scan-coverage` (unknown completeness cannot pass) |

`control.vulnerability.scan-freshness` and `periodic-scanning` may share `test.vulnerability.scan-current` if both are independently assessable (distinct objectives, same predicate). Do not invent a 21st micro-control.

Default SLA windows (catalog/test constants, not runtime magic): **critical = 7 days**, **high = 30 days**, measured from `discovered_at` against fixture `collectedAt` / assessment `now`. Record the constants in catalog test expressions or fixture docs so they are deterministic.

Container scanning is Hybrid because image inventory completeness is often organizational; the automatable slice is “covered images have a current container-class scan.” Absence of image inventory → `InsufficientEvidence` or `ManualReviewRequired`, never Effective.

Exposure review and risk-acceptance are Hybrid: attestation quality is organizational. Technical `reviewed_at` / `exception_status=approved` is supporting.

### 4.4 Canonical evidence (facts, not conclusions)

Reuse Prompt 01/02 evidence declarations. This slice **defines** the vulnerability family.

| Evidence type | Observed facts (store via `EvidenceValue`) | Not allowed |
| --- | --- | --- |
| `evidence.vulnerability.finding` | `subject_id` (finding id), `asset_subject_id`, `severity` (`info`\|`low`\|`medium`\|`high`\|`critical`), `status` (see §4.5), `discovered_at`, `age_days` (integer), `class` (`sast`\|`sca`\|`container`\|`secret`\|`dast`\|`generic`), `internet_exposed` (bool), `duplicate_of?`, `superseded_by?`, `cve_id?` | `compliant`, `iso_passed`, raw secret values |
| `evidence.vulnerability.scan-run` | `subject_id` (asset), `scan_id`, `completed_at`, `scan_class`, `finding_count` (integer), `fresh` (bool *or* omit and let `FreshWithin` decide) | vendor product name as type id; “scan effective” |
| `evidence.vulnerability.scan-coverage` | `population_id`, `in_scope_count`, `covered_count`, `completeness` (`authoritative`\|`partial`\|`unknown`), `authoritative` (bool) | empty-findings ⇒ clean |
| `evidence.vulnerability.remediation-state` | `subject_id` (finding), `status` (same enum as finding), `remediated_at?`, `tracked` (bool) | treating accepted-risk as remediated |
| `evidence.vulnerability.owner` | `subject_id` (finding), `owner_assigned` (bool), `owner_subject_id?` | — |
| `evidence.vulnerability.exception` | `subject_id` (finding), `exception_id`, `exception_kind` (`accepted-risk`\|`exception-approved`\|`false-positive`), `exception_status` (`proposed`\|`approved`\|`expired`\|`revoked`), `expires_at?` | silent Effective |
| `evidence.vulnerability.exposure-review` | `subject_id` (asset or org), `reviewed_at`, `reviewer_id?`, `result` (`open`\|`accepted`\|`closed`) | “review effective” |
| `evidence.secret.exposure` | `subject_id`, `asset_subject_id`, `active` (bool), `revoked` (bool), `secret_class` (`credential`\|`key`\|`token`\|`other` — **never the secret**) | password/token/key material |
| `evidence.dependency.vulnerability` | `subject_id` (finding or purl), `severity`, `status`, `discovered_at`, `age_days`, `package_name`, `package_version` | — |
| `evidence.dependency.confusion-risk` | `subject_id` (package or repo), `risk_present` (bool), `monitored` (bool), `package_name` | — |
| `evidence.asset.software-inventory` | `subject_id` (component), `package_name`, `package_version`, `support_status` (`supported`\|`unsupported`\|`unknown`), `handled` (bool — observed treatment, not a control pass) | EOL “compliant” |

Seal rules still apply: no credential-shaped keys (`token`, `password`, `secret`, `api_key`, …); no compliance narratives. `secret_class` is an enum label, not a recovered secret.

Finding `status` is **the** state field. Duplicate it on `remediation-state` when that envelope exists; they must not contradict in golden fixtures.

### 4.5 State semantics (normative)

Explicit finding / remediation states:

```text
open
resolved
accepted-risk
false-positive
exception-approved
unknown
```

Rules:

1. **`resolved`** means a remediation outcome was observed (`remediated_at` set, or status=`resolved`). It is the only state that counts as remediating an SLA clock to zero.
2. **`accepted-risk`** and **`exception-approved`** are **not** remediation. SLA tests **exclude** them from the open-overdue population; they must **not** satisfy `test.vulnerability.resolved-not-accepted-risk` as `resolved`. Risk-acceptance control requires a recorded, unexpired exception fact (Hybrid).
3. **`false-positive`** is not `resolved` and not `open`. It is excluded from SLA open populations. Separation test fails if `status=resolved` and `exception_kind=false-positive` (or a `false_positive=true` flag) on the same subject.
4. **`unknown`** is insufficient for SLA pass; treat as missing / `InsufficientEvidence` for all-subjects predicates that require a known status.
5. **`open`** (and missing status treated as open when a finding envelope exists without status) participates in SLA, ownership, and internet-exposure tests.
6. Expired or revoked exceptions (`exception_status=expired`\|`revoked`, or `expires_at` before assessment `now`) return the finding to **open** for SLA purposes. They must not yield `ExceptionApproved` or Effective.
7. Superseded / duplicate findings (`superseded_by` or `duplicate_of` set) are excluded from open SLA and ownership populations; `test.vulnerability.duplicates-superseded` fails if two open, non-superseding duplicates remain.

Do not treat an empty finding list as `Effective` unless `evidence.vulnerability.scan-coverage.completeness=authoritative` **and** `scan-run` is fresh. Completeness `unknown` or `partial` + zero findings → `InsufficientEvidence` (unknown) or coverage failure (partial below 100% for all-subjects coverage tests).

### 4.6 Tests (population-based, not existence checks)

Required reusable tests (Prompt 06 list + extras so no control is untested):

```text
test.vulnerability.scan-current
test.vulnerability.scan-coverage
test.vulnerability.no-critical-over-sla
test.vulnerability.no-high-over-sla
test.vulnerability.findings-have-owner
test.secret.no-active-exposure
test.dependency.no-critical-over-sla
test.vulnerability.source-scan-current
test.vulnerability.container-scan-current
test.dependency.confusion-monitored
test.vulnerability.unsupported-software-handled
test.vulnerability.exposure-review-current
test.vulnerability.no-open-internet-exposed-critical
test.vulnerability.risk-acceptance-recorded
test.vulnerability.exceptions-unexpired
test.vulnerability.duplicates-superseded
test.vulnerability.resolved-not-accepted-risk
test.vulnerability.remediation-state-present
```

Semantics (authoritative intent; encode with Prompt 03 arms — typically `all-subjects`, `none-subjects`, `coverage-at-least`, `fresh-within`, `count-where`):

| Test | Population | Pass | Fail | Missing / unknown | Stale / exception |
| --- | --- | --- | --- | --- | --- |
| `scan-current` | in-scope assets from authoritative coverage / inventory | each has `scan-run` within freshness | scan present but `fresh=false` or outside window | no scan-run for a known in-scope asset | stale → `StaleEvidence`; unknown coverage → `InsufficientEvidence` |
| `scan-coverage` | in-scope assets | `completeness=authoritative` and `covered_count == in_scope_count` (or 100% `coverage-at-least`) | `partial` below threshold | `completeness=unknown` | — |
| `no-critical-over-sla` | open (or unknown-as-open) **critical** findings that are not FP / accepted-risk / exception-approved / superseded | none have `age_days` > 7 (or `discovered_at` older than SLA) | ≥1 critical open overdue | finding without `discovered_at` / `age_days` | approved unexpired exception → exclude from fail set and do **not** call it resolved; expired exception → fail as open |
| `no-high-over-sla` | same for `severity=high`, window 30 days | none overdue | ≥1 high open overdue | missing age | same exception rules |
| `findings-have-owner` | open findings (exclude superseded/FP) | every subject `owner_assigned=true` | owner false / empty | missing owner envelope | — |
| `no-active-exposure` | secret-exposure subjects | none with `active=true` | ≥1 active | secret scan coverage unknown | stale secret scan → `StaleEvidence` |
| `dependency.no-critical-over-sla` | open critical dependency findings | none overdue | ≥1 overdue | missing SCA coverage | same as critical SLA |
| `no-open-internet-exposed-critical` | findings with `internet_exposed=true` and `severity=critical` | none `status=open` | ≥1 open internet-exposed critical | missing internet_exposed on a known critical | excepted ≠ resolved |
| `resolved-not-accepted-risk` | findings marked resolved or accepted-risk / FP | no subject is both `resolved` and (`accepted-risk` or `false-positive`) | mixed/collapsed states | missing status → not a pass for this test’s “resolved” set | — |
| `exceptions-unexpired` | findings with an exception envelope | every approved exception has `exception_status=approved` and `expires_at` in the future (or no expiry only if documented as explicit non-expiring + Hybrid attestation) | expired/revoked still treated as covering | missing `expires_at` on time-boxed kinds | — |

**Forbidden encoding:** `Exists(evidence.vulnerability.finding)` as the body of `test.vulnerability.no-critical-over-sla`. Existence of some finding (or some scan-run) is not “population within SLA” or “coverage current.”

**Forbidden encoding:** `Exists(evidence.vulnerability.scan-run)` as proof that all in-scope assets are scanned.

Unknown / non-authoritative coverage **must not** produce `Effective` for `scan-coverage` or for SLA tests that infer cleanliness from an empty finding list.

Result metadata must include Prompt 03 population detail: population, evaluated, passing, failing, missing, coverage, failing/missing subject ids.

### 4.7 Golden fixtures (deterministic)

| Fixture | Intent | Expected highlights |
| --- | --- | --- |
| `complete-clean-scan` | Authoritative coverage; fresh scans (sast/sca/secret at minimum); zero open findings; no active secrets; no unsupported unhandled software | Automated vuln tests `Effective`. Hybrid tests Effective only with attestations; otherwise `ManualReviewRequired` / `InsufficientEvidence` — document the choice. |
| `critical-inside-sla` | One open critical finding with `age_days` < 7; coverage authoritative; owner assigned | `no-critical-over-sla` → `Effective`. Finding exists; SLA has not failed. |
| `critical-overdue` | One open critical finding with `age_days` > 7 | `no-critical-over-sla` → `Ineffective` naming that subject. Not missing. |
| `critical-approved-exception` | Same overdue critical, plus approved unexpired exception (`exception-approved` or IR `Exception` Approved) | SLA test does **not** treat as `resolved`. Outcome `ExceptionApproved` **or** exclusion from open population with risk-acceptance test Effective and `resolved-not-accepted-risk` still Effective. Must not be silent Effective on “all criticals remediated.” |
| `critical-expired-exception` | Overdue critical with `exception_status=expired` | Treated as **open**; `no-critical-over-sla` → `Ineffective`; `exceptions-unexpired` → `Ineffective`. |
| `incomplete-scan-coverage` | `in_scope_count=10`, `covered_count=8`, `completeness=partial` | `scan-coverage` → `Ineffective` or `InsufficientEvidence` (not Effective). Zero-or-few findings cannot prove the two unscanned assets clean. |
| `stale-scan` | Coverage authoritative but `scan-run.completed_at` outside freshness / `fresh=false` | `scan-current` → `StaleEvidence`. |
| `unresolved-secret-exposure` | `evidence.secret.exposure` with `active=true` | `no-active-exposure` → `Ineffective`. Other SLA tests unaffected unless the same subject is also a finding. |
| `duplicate-superseded` | Two observations of one issue; one `superseded_by` / envelope `supersedes` the other; survivor open inside SLA | `duplicates-superseded` → `Effective`. Counting both as open SLA subjects is a fail. |
| `zero-findings-unknown-coverage` | No finding envelopes; `completeness=unknown` (or missing coverage) | `scan-coverage` and cleanliness-from-empty **must not** be `Effective`. `InsufficientEvidence` or `Inconclusive`. This is the Prompt 06 “scanner returned zero findings but population coverage is unknown” case. |

Fixtures emit **canonical** `evidence.vulnerability.*` / `evidence.secret.*` / `evidence.dependency.*` / `evidence.asset.software-inventory` only. No `security.vulnerability.present` as the catalog evidence type. Optional fixture README/comment may show a field mapping from `Finding.severity` / `found_at` / Codex `coverage.completeness`.

### 4.8 Integration rules (consume, do not redesign)

- Loader / validate / digest: Prompt 01 `CanonicalCatalog`. New files must pass `validate` (no orphans, no reserved provider/framework segments, deterministic digest).
- Typed facts: Prompt 02 `EvidenceValue` (`with_value`). Do not rely on string `parse_fact`.
- Population evaluation: Prompt 03. Vulnerability tests are **declarations**. No `VulnPopulation` / `vuln_all_subjects` fork.
- Exception: reuse IR `Exception` + `Effectiveness::ExceptionApproved` when binding subject-scoped approved exceptions. Finding `status` remains the catalog fact. Expired/revoked must not pass.
- Subject kinds: Asset (findings as `finding:<id>`), Repository, Application, Organization. Do not add a third `SubjectSelector` type. Do not add `SubjectKind::Finding` unless Asset selection is a proven compile blocker.
- Scanner engines, Codex schemas, `Finding` / `ScanReport`, `bridge.rs` classify function: **untouched** unless a documented compile blocker requires a one-line compatibility fix.
- ISO pack, IAM catalog, GitHub collector, framework compiler: **untouched**.
- Credential-shaped keys and compliance / certification phrases remain rejected.

### 4.9 Dual-suite protocol

Follow the existing root `[[test]]` pattern (IAM / catalog infrastructure).

| Suite | Path | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/vulnerability_catalog.baseline.rs` · `sdd_vulnerability_catalog_baseline` | GREEN on `e2def07…`: no `control.vulnerability.*` in canonical catalog; ISO sliver `vulnerability.remediation` still present; scanner bridge still `security_*` types. After target GREEN: `#[ignore]` superseded (`supersede_kind=skip`). |
| Target | `tests/sdd/vulnerability_catalog.target.rs` · `sdd_vulnerability_catalog_target` | RED on `e2def07…`; **GREEN** after implement — CI gate (AC-1…AC-n). |

Suggested target assertion clusters (test titles include the id):

| ID | Asserts |
| --- | --- |
| VULN-001 | Catalog loads vulnerability files offline via Prompt 01 API |
| VULN-002 | Digest of catalog including vuln slice is deterministic |
| VULN-003 | All 20 `control.vulnerability.*` ids present, stable, prefixed |
| VULN-004 | Required evidence types declared; no orphans |
| VULN-005 | Required tests declared and referenced; population ops not mere `exists` for SLA/coverage |
| VULN-006 | Validator / suite rejects provider and scanner-product tokens in vuln IDs |
| VULN-007 | No ISO/SOC2/NIS2/DORA/GDPR tokens in vuln catalog files |
| VULN-008 | ISO pack `vulnerability.remediation` unchanged; no `control.vulnerability.*` inside the pack |
| VULN-009 | `no-critical-over-sla` fails `critical-overdue` and passes `critical-inside-sla` |
| VULN-010 | Ten fixtures distinguish missing vs stale vs fail vs exception vs unknown coverage |
| VULN-011 | `zero-findings-unknown-coverage` is not Effective |
| VULN-012 | Approved unexpired exception is not `resolved`; expired exception reopens SLA |
| VULN-013 | `resolved-not-accepted-risk` and FP separation |
| VULN-014 | Active secret exposure fails `test.secret.no-active-exposure` |
| VULN-015 | Incomplete coverage (8/10) cannot prove all 10 clean |
| VULN-016 | Existing `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, `sdd_canonical_assurance_catalog_target` stay green; scanner crates unedited except documented adapter |

### 4.10 Scanner-bridge compatibility (non-goals for engines)

Fixtures **should** be constructible from Weeping Angel / Codex fields:

| Scanner field | Canonical fact |
| --- | --- |
| `Finding.severity` / Codex severity | `severity` |
| `Finding.found_at` / occurrence time | `discovered_at` → `age_days` vs fixture now |
| `Finding.id` / `findingId` | `subject_id` |
| `Finding.module` / `ruleId` / category | `class` (mapped to sast/sca/container/secret/generic) |
| Codex `coverage.completeness` | `scan-coverage.completeness` (`complete`→`authoritative`) |
| Codex `scanId` | `scan-run.scan_id` |
| `security.secret.exposure` bridge type | emit `evidence.secret.exposure` in a **future** adapter |
| `security.dependency_confusion_risk` | emit `evidence.dependency.confusion-risk` in a **future** adapter |

This slice does **not** implement that adapter in `bridge.rs`. Golden fixtures are already-canonical JSON.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase. IDs `AC-1`…`AC-18` match the target suite.

1. **AC-1.** Dual-suite `sdd_vulnerability_catalog_baseline` + `sdd_vulnerability_catalog_target` is registered in root `Cargo.toml` like existing SDD tests.
2. **AC-2.** On SHA `e2def07…` (pre-vuln catalog content): baseline GREEN; target RED for missing `control.vulnerability.*` / fixtures — not for unrelated compile errors.
3. **AC-3.** After implement: target GREEN; baseline ignored so absence-of-catalog is not a CI requirement; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green on files this slice touches.
4. **AC-4.** Twenty `control.vulnerability.*` controls exist with stable ids, domains, evidence requirements, test refs, and honest automation class; count stays in 15–25 with no artificial micro-controls.
5. **AC-5.** Evidence types listed in §4.4 are declared as facts (severity, age, status, subject, discovery time, remediation time, exception state) without depending on a specific scanner.
6. **AC-6.** Tests include at least the seven Prompt-06 ids and evaluate **populations** (no critical over SLA; coverage of in-scope assets), not existence of one envelope.
7. **AC-7.** State enum `open` / `resolved` / `accepted-risk` / `false-positive` / `exception-approved` / `unknown` is used in fixtures and tests; accepted-risk and approved exception are not remediation.
8. **AC-8.** Empty finding list + unknown coverage cannot yield `Effective` on coverage or “clean scan” tests (`zero-findings-unknown-coverage`).
9. **AC-9.** `critical-inside-sla` passes critical SLA; `critical-overdue` fails it naming the subject.
10. **AC-10.** `critical-approved-exception` is not treated as resolved; `critical-expired-exception` reopens SLA (`Ineffective`).
11. **AC-11.** `incomplete-scan-coverage` (8 of 10) cannot prove all 10 clean; `stale-scan` is `StaleEvidence`.
12. **AC-12.** `unresolved-secret-exposure` fails `test.secret.no-active-exposure`; dependency-confusion monitoring has a declared test and evidence type.
13. **AC-13.** Duplicate/superseded fixture counts one open subject; `resolved-not-accepted-risk` fails collapsed FP/accepted-risk-as-resolved encodings.
14. **AC-14.** Catalog validator accepts the slice: no duplicate/orphan/dangling ids; no provider/scanner-product/framework tokens in canonical vuln IDs or vuln TOML narrative.
15. **AC-15.** ISO pack control ids and mappings are unchanged; `sdd_iso27001_assurance_target` remains green. IAM catalog files are not rewritten as the vuln library.
16. **AC-16.** No second `CanonicalCatalog` loader, no second `EvidenceValue`, no local population-math fork. Prompt 03 coverage is consumed as-is.
17. **AC-17.** Scanner engines, Codex Security schemas, and `bridge.rs` are not redesigned; no provider collector is added; no ISO remap; no certification/readiness language in catalog content or CLI.
18. **AC-18.** Prompt 01 / 02 / 03 / 04 SSOT paths are not overwritten by this slice.

---

## 6. Out of scope

- Provider collectors (Snyk, Dependabot, Trivy, Qualys, GitHub Advanced Security, Weeping Angel live collect → catalog).
- Expanding `weeping-angel-assurance::bridge` into a full catalog emitter (later execution prompt).
- Redesigning scanner engines, `Finding`, Codex Security schemas, or SARIF adapters.
- Mapping ISO/SOC 2/NIS2 clauses (Prompt 12).
- Inferring certification, readiness, or “audit passed” from findings.
- SDLC catalog (Prompt 05), infrastructure catalog (Prompt 07), governance (Prompt 08).
- Rewriting IAM identity TOML or ISO pack `vulnerability.remediation`.
- Adding `SubjectKind::Finding` unless a documented compile blocker requires it.
- Implementing a second exception or coverage engine.
- Storing secret material, exploit PoCs, or live attack payloads in fixtures.

---

## 7. Risks

- Overlap with unlanded Prompt 05 (secret scanning / dependency scanning as *enablement* vs this slice’s *exposure/SLA*). Mitigate with namespaces (`control.vulnerability.*` vs future `control.source.secret-scanning`).
- SLA encoded as existence of a scan-run instead of finding age — target suite must pin overdue vs inside-SLA fixtures.
- Treating accepted-risk / FP / exception as `resolved` — dedicated fixture + `resolved-not-accepted-risk`.
- Empty findings + unknown coverage silently passing — dedicated fixture.
- Accidental ISO pack edit breaking `sdd_iso27001_assurance_target`.
- Accidental scanner-engine edits expanding scope.
- `ExceptionApproved` only implemented for IAM break-glass — finding exceptions must still be distinguishable via facts even if effectiveness stays `Ineffective`/`ExceptionApproved`.
- Catalog validator provider-segment list does not include `snyk`/`trivy`; target suite must grep those tokens in vuln files.

---

## 8. ADR

A draft ADR is required (same pattern as IAM): vulnerability family lives in the canonical catalog, not the ISO pack and not the scanner contract; findings are evidence; coverage and state semantics are normative.

---

## 9. Transition

| Field | Value |
| --- | --- |
| `transition_kind` | `replacement` |
| `baseline_post_expected` | `retired` |
| Rationale | On `e2def07…` there is no canonical vulnerability family. The dual-suite baseline characterizes that **absence** (plus the still-true ISO sliver). After implement, absence asserts are false; the baseline is `#[ignore]` superseded like IAM. The ISO-pack control is **not** deleted (pack-layer additive), but it is not the canonical library. Hint `auto` is therefore **replacement** of the empty canonical family, not additive-keep-baseline-green. CI-004 (baseline need not fail when additive) does not apply to this absence-characterization suite. |

| Suite path | Command |
| --- | --- |
| `tests/sdd/vulnerability_catalog.baseline.rs` | `cargo test --test sdd_vulnerability_catalog_baseline -- --nocapture` |
| `tests/sdd/vulnerability_catalog.target.rs` | `cargo test --test sdd_vulnerability_catalog_target -- --nocapture` |
| Regression | `cargo test --workspace --features demo` |

---

## 10. Live seams (do not fork)

- `weeping_angel_canonical_catalog::CanonicalCatalog::{load,validate,digest}`
- `weeping_angel_evidence::{EvidenceValue,EvidenceEnvelope,EvidenceObservation}`
- `weeping_angel_control_test::{TestExpr,evaluate,Effectiveness,EvidenceSet,PopulationEvaluation}`
- `weeping_angel_assurance_ir::{Exception,ExceptionStatus,ControlDomain::VulnerabilityManagement,SubjectKind}`
- ISO `vulnerability.remediation` / `security.vulnerability.present`
- `weeping_angel_assurance::bridge` (`security_finding`, `canonical_type`)
- CLI `Finding` / Codex `findings.json` + `coverage.json` (compatibility only)
