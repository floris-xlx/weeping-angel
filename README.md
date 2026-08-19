# weeping-angel

current version: `0.2.0`
Authorized **web recon + security scanning** CLI (Rust). Discover routes (including SPA/JS surfaces), flag misconfigurations and exposed secrets, map auth, run YAML path templates, compare authenticated vs anonymous access, and optionally fire **gated** active probes.

> **Legal:** Only scan systems you **own** or have **written permission** to test. The tool refuses to run without `--i-own-this` and a host allowlist.

## Quick setup (Windows)

```powershell
.\scripts\setup.ps1
# full lab demo + scan:
.\scripts\demo-scan.ps1
```

## Quick setup (Unix)

```bash
chmod +x scripts/*.sh
./scripts/setup.sh
./scripts/demo-scan.sh
```

## Installers (deb, msi, …)

Requires [`cargo-packager`](https://github.com/crabnebula-dev/cargo-packager) (`cargo install cargo-packager`).

```bash
pnpm run installer:deb
pnpm run installer:msi
pnpm run installer:nsis
pnpm run installer:rpm
pnpm run installer:appimage
pnpm run installer:pacman
pnpm run installer:dmg
pnpm run installer:app
pnpm run installer          # all formats cargo-packager can build on this host
```

Artifacts land in `target/packager/`.

## Manual install

```bash
cargo build --release
# scanner: target/release/weeping-angel

# optional local lab server (example, not a dist install artifact):
cargo build --release --example weeping-angel-demo --features demo
# lab: target/release/examples/weeping-angel-demo.exe   (127.0.0.1 only)
```

## Tests

```bash
# scanner + assurance workspace + e2e (lab router requires demo)
cargo test --workspace --features demo
```

Layout: [`docs/README.md`](docs/README.md) (`docs/specs/` specs, `docs/adr/` decisions, `tests/contracts/` executable invariants). Generated SDD traces stay out of git under `.sdd/`.

Assurance contract tests (`tests/contracts/`): `sdd_assurance_runtime_target` (ACT-001…015), `sdd_iso27001_assurance_target` (ISO/EVD/CTL/GH), `sdd_iso27001_remap_target` (ISO-R catalog projection), `sdd_canonical_assurance_catalog_target` (CAT-001…016), `sdd_typed_evidence_target` (typed facts + `evidence-value/v1`), `sdd_population_runtime_target`, `sdd_iam_catalog_target` (IAM-001…016), `sdd_sdlc_catalog_target` (SDLC-001…016), `sdd_vulnerability_catalog_target` (VULN-001…016), `sdd_infrastructure_catalog_target` (INFRA-001…016), `sdd_governance_catalog_target` (GOV-001…016), `sdd_github_collector_target` (canonical GitHub collector, `ghc_000`–`ghc_024`), `sdd_applicability_engine_target` (P10-T01…T16 Kleene three-state), `sdd_assessment_lineage_target` (LIN-001…015 immutable run / explain / pure serialize), `sdd_temporal_assurance_target` (TMP-001…012 as-of / period / no temporal leakage), `sdd_incident_governance_target` (IG-001…012 explicit declare / timeline / PIR; observations are not incidents), `sdd_nonconformity_capa_target` (NC-001…012 contain → RCA → sustained effectiveness → explicit close; one green is not CAPA done), `sdd_continuity_resilience_target` (P20-T01…T16 plan existence is not demonstrated recovery), `sdd_isms_context_target` (CTX-T01…T14 durable `IsmsContext` root; `AssessmentDefinition::new` still valid), `sdd_interested_parties_obligations_target` (IPO-001…018 standalone `ObligationRegistry`; partial mappings never equivalence), `sdd_security_objectives_target` (SO-T01…T20 measurable `SecurityObjective` + `evaluate_objective`; missing/stale/partial never success), `sdd_risk_methodology_target` (P05-T01…T17 versioned `score_risk`; 3×3/5×5 fixtures, not a crate-wide 5×5), `sdd_risk_register_target` (RR-001…015 operational `Risk`; findings are not auto-promoted), `sdd_risk_identification_target` (RI-001…010 candidate cluster / explicit promote; scanners cannot declare `risk accepted` or `ISO control failed`), `sdd_risk_treatment_target` (P08-T01…T16 Mitigate/Accept/Avoid/Transfer; expired acceptance never suppresses), `sdd_control_implementation_registry_target` (CIR-001…015 organizational implementation ≠ effectiveness), `sdd_continuous_assurance_scheduler_target` (CAS-001…016 library `tick` / fake clock; failed collect does not erase ledger evidence), `sdd_isms_events_drift_target` (`P15:` no-op / ControlRegressed / EvidenceExpired / risk-increase-caused-by-regression / NewAssetDetected / ExceptionExpired / dedupe), `sdd_personnel_security_target` (PER-001…016 population-honest joiner/mover/leaver; not an HRIS), and `sdd_evidence_validity_temporal_assurance_target` (EVT-001…012 validity events / historical assessment). Layout invariant: `sdd_documentation_layout`. Matching baseline suites are superseded / ignored except characterization suites that remain RED on the product (e.g. `sdd_incident_governance_baseline`).

## Assurance runtime (ISO 27001 readiness)

Weeping Angel is also an **inwardly extensible** assurance compiler: capabilities + observations in, control-test results out. Findings stay security-only. Collectors advertise evidence types, not frameworks. Automated output is a **readiness assessment**, never ISO certification.

```text
AssuranceEngine::builder().collector(…).framework(target).assess(scope)
AssuranceScheduler::builder().clock(…).store(…).ledger(…).register(JobSpec).tick()
```

ISO 27001:2022 ships as a versioned structural pack (`frameworks/iso-27001/2022`) that **projects** onto the canonical catalog. Pack mappings target `control.identity.*` and landed `control.source.*` IDs (`PartiallySatisfies` / `Supports`; never convenience `Equivalent`). Pack-local slivers (`access.mfa.privileged`, `source.branch-protection`, …) are retired. The reusable library is the offline canonical catalog (`catalog/canonical/v1`, schema `weeping-angel/canonical-catalog/v1`, IDs `control.*` / `evidence.*` / `test.*`). IAM tests are provider-neutral population predicates, not Entra/Okta/GitHub checks. SDLC tests (`control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*`) assess repository / CI / release / supply-chain populations from `evidence.repository.*` (and cicd / deployment / release / supply-chain) facts; default-branch protection is `control.source.default-branch-protection` (the exists-only fixture `control.source.protected-branch` remains); missing scan evidence is `InsufficientEvidence`. Vulnerability tests (`control.vulnerability.*`) treat scanner findings as evidence, not compliance results; accepted-risk is not remediation; empty findings plus unknown coverage are never Effective. Infrastructure tests (`control.network.*` / `control.crypto.*` / `control.secret.*` / `control.data.*` / `control.database.*` / `control.logging.*` / `control.backup.*` / operational `control.resilience.*`) are provider-neutral population predicates on `evidence.network.*` / `evidence.data.*` / `evidence.crypto.key-state` / `evidence.secret.storage-configuration` / `evidence.database.*` / `evidence.logging.*` / `evidence.backup.*` / `evidence.resilience.recovery-plan`; missing evidence is `InsufficientEvidence`; DR exercise, recovery objectives, and segmentation rationale stay hybrid/manual. Governance tests (`control.governance.*` / `control.risk.*` / `control.personnel.*` / `control.vendor.*` / `control.incident.*` plus continuity `control.resilience.business-continuity-plan` / `disaster-recovery-governance`) treat manual evidence as first-class immutable facts (`evidence.manual.attestation` and domain types); a document-present flag is not operational effectiveness; missing evidence is `InsufficientEvidence`; approved unexpired IR exceptions are `ExceptionApproved`, never silent `Effective`. Continuity capability (`evaluate_continuity_resilience` over `AssetKind::Service` profiles) is a separate IR projection: a current BCP or `procedure_present` is not demonstrated recovery; tabletop cannot satisfy RTO/RPO. Personnel lifecycle tests (additive `control.personnel.*` in `personnel.toml`) evaluate complete in-scope populations for screening, joiner grace, provisioning, role-change least privilege, leaver access removal, and asset-return references; one trained user never proves coverage; `active`/`excessive` are defect flags; identity stays thin (no Employee/Contractor kinds). Organizational ISO clauses stay unmapped. Observation facts are typed `EvidenceValue` (`evidence-value/v1`); `with_fact` remains string-compatible. `GitHubCollector` emits canonical `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / observable `evidence.identity.*` plus inventory envelopes — never `source.*` observations, never `evidence.github.*`, never framework status. Local / manual / scanner evidence also never writes framework status. Reports pin both pack and catalog digests, serialize from those pins (no pack load at serialize time), and never say certified / compliant / audit passed. An `AssessmentRun` is a returned execution record (`completed` / `partial` / `failed`) with distinct definition / evidence-snapshot / result / applicability identities and a pinned `asOf` clock. Replay uses pinned snapshots and only evidence that was valid at that clock; a current-file digest mismatch is detected, not rewritten. Validity changes are append-only `evidence-validity/v1` events (never edits to a sealed envelope). Period results (`PeriodEffectiveness`) do not infer continuous effectiveness from one `Exists` hit.

```bash
weeping-angel assurance assess --framework iso-27001 --scope .
weeping-angel assurance framework validate frameworks/iso-27001/2022
weeping-angel assurance catalog validate
weeping-angel assurance catalog stats
weeping-angel assurance catalog inspect control.source.protected-branch
weeping-angel assurance catalog inspect control.source.default-branch-protection
weeping-angel assurance catalog inspect control.identity.mfa
weeping-angel assurance catalog inspect control.vulnerability.periodic-scanning
weeping-angel assurance catalog inspect control.database.encryption
weeping-angel assurance catalog inspect control.network.tls-sensitive-traffic
weeping-angel assurance catalog inspect control.governance.information-security-policy
weeping-angel assurance soa
weeping-angel assurance explain --assessment <id> --control control.identity.privileged-mfa
```

The clap family is `assurance {framework,collect,evidence,assess,result,compare,soa,catalog,explain}`. `assurance catalog`, `assurance explain`, and `assurance soa` are dispatched (validate / stats / inspect; explain from a pinned ledger run; SoA prints the not-certification banner then operational JSON — live `iso-27001`/`2022` for `latest`, unknown pinned assessment exits non-zero). Other `assurance` subcommands print the non-certification banner; library `assess` / `project_readiness` / `project_soa` / `project_operational_soa` / `compare` / `explain_control` is their execution path. Continuous operation is library `AssuranceScheduler::tick` (`JobSpec` owns cadence/retry/backoff/jitter, not clap).

Workspace crates: `weeping-angel-assurance-ir` → `framework` / `evidence` → `collector`; `ir` + `evidence` → `control-test`; `ir` → `canonical-catalog` (offline); facade `weeping-angel-assurance` composes the runtime. Framework and collector do not depend on the catalog crate.

- Map: [`docs/README.md`](docs/README.md)
- Decisions: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](docs/adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](docs/adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](docs/adr/0003-canonical-assurance-catalog-v1.md), [`docs/adr/0003-typed-evidence-canonical-serialization.md`](docs/adr/0003-typed-evidence-canonical-serialization.md), [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](docs/adr/0003-subject-population-runtime-and-coverage-semantics.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](docs/adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](docs/adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](docs/adr/0003-vulnerability-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](docs/adr/0003-infrastructure-canonical-assurance-catalog.md), [`docs/adr/0003-governance-canonical-assurance-catalog.md`](docs/adr/0003-governance-canonical-assurance-catalog.md), [`docs/adr/0003-personnel-security-lifecycle.md`](docs/adr/0003-personnel-security-lifecycle.md), [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](docs/adr/0003-github-collector-canonical-evidence-mapping.md), [`docs/adr/0003-applicability-engine.md`](docs/adr/0003-applicability-engine.md), [`docs/adr/0003-assessment-lineage.md`](docs/adr/0003-assessment-lineage.md), [`docs/adr/0003-evidence-validity-temporal-assurance.md`](docs/adr/0003-evidence-validity-temporal-assurance.md), [`docs/adr/0003-operational-soa.md`](docs/adr/0003-operational-soa.md), [`docs/adr/0003-incident-governance.md`](docs/adr/0003-incident-governance.md), [`docs/adr/0003-internal-audit.md`](docs/adr/0003-internal-audit.md), [`docs/adr/0003-nonconformity-capa.md`](docs/adr/0003-nonconformity-capa.md), [`docs/adr/0003-remediation-engine.md`](docs/adr/0003-remediation-engine.md), [`docs/adr/0003-control-implementation-registry.md`](docs/adr/0003-control-implementation-registry.md), [`docs/adr/0003-iso27001-canonical-remap.md`](docs/adr/0003-iso27001-canonical-remap.md), [`docs/adr/0003-temporal-assurance.md`](docs/adr/0003-temporal-assurance.md), [`docs/adr/0004-documentation-architecture.md`](docs/adr/0004-documentation-architecture.md), [`docs/adr/0005-continuous-assurance-scheduler.md`](docs/adr/0005-continuous-assurance-scheduler.md), [`docs/adr/0005-continuity-resilience.md`](docs/adr/0005-continuity-resilience.md), [`docs/adr/0003-isms-events-drift.md`](docs/adr/0003-isms-events-drift.md), [`docs/adr/0007-risk-identification-candidate-correlation.md`](docs/adr/0007-risk-identification-candidate-correlation.md), [`docs/adr/0008-isms-context.md`](docs/adr/0008-isms-context.md), [`docs/adr/0008-scope-engine.md`](docs/adr/0008-scope-engine.md), [`docs/adr/0008-interested-parties-obligations.md`](docs/adr/0008-interested-parties-obligations.md), [`docs/adr/0008-security-objectives.md`](docs/adr/0008-security-objectives.md)
- Contract: [`docs/specs/assurance-runtime.md`](docs/specs/assurance-runtime.md)
- Specs: [`docs/specs/assurance-runtime-spine.md`](docs/specs/assurance-runtime-spine.md), [`docs/specs/iso-27001-automated-assurance-mvp.md`](docs/specs/iso-27001-automated-assurance-mvp.md), [`docs/specs/iso-27001-canonical-remap.md`](docs/specs/iso-27001-canonical-remap.md), [`docs/specs/canonical-assurance-catalog-v1.md`](docs/specs/canonical-assurance-catalog-v1.md), [`docs/specs/typed-evidence.md`](docs/specs/typed-evidence.md), [`docs/specs/population-runtime.md`](docs/specs/population-runtime.md), [`docs/specs/iam-canonical-assurance-catalog.md`](docs/specs/iam-canonical-assurance-catalog.md), [`docs/specs/sdlc-canonical-assurance-catalog.md`](docs/specs/sdlc-canonical-assurance-catalog.md), [`docs/specs/vulnerability-canonical-assurance-catalog.md`](docs/specs/vulnerability-canonical-assurance-catalog.md), [`docs/specs/infrastructure-canonical-assurance-catalog.md`](docs/specs/infrastructure-canonical-assurance-catalog.md), [`docs/specs/governance-canonical-assurance-catalog.md`](docs/specs/governance-canonical-assurance-catalog.md), [`docs/specs/personnel-security.md`](docs/specs/personnel-security.md), [`docs/specs/github-collector.md`](docs/specs/github-collector.md), [`docs/specs/applicability-engine.md`](docs/specs/applicability-engine.md), [`docs/specs/assessment-lineage.md`](docs/specs/assessment-lineage.md), [`docs/specs/operational-soa.md`](docs/specs/operational-soa.md), [`docs/specs/incident-governance.md`](docs/specs/incident-governance.md), [`docs/specs/continuity-resilience.md`](docs/specs/continuity-resilience.md), [`docs/specs/internal-audit.md`](docs/specs/internal-audit.md), [`docs/specs/nonconformity-capa.md`](docs/specs/nonconformity-capa.md), [`docs/specs/remediation-engine.md`](docs/specs/remediation-engine.md), [`docs/specs/isms-context.md`](docs/specs/isms-context.md), [`docs/specs/scope-engine.md`](docs/specs/scope-engine.md), [`docs/specs/interested-parties-obligations.md`](docs/specs/interested-parties-obligations.md), [`docs/specs/security-objectives.md`](docs/specs/security-objectives.md), [`docs/specs/risk-identification.md`](docs/specs/risk-identification.md), [`docs/specs/control-implementation-registry.md`](docs/specs/control-implementation-registry.md), [`docs/specs/continuous-assurance-scheduler.md`](docs/specs/continuous-assurance-scheduler.md), [`docs/specs/isms-events-drift.md`](docs/specs/isms-events-drift.md), [`docs/specs/evidence-validity-temporal-assurance.md`](docs/specs/evidence-validity-temporal-assurance.md), [`docs/specs/temporal-assurance.md`](docs/specs/temporal-assurance.md)
- Packs: [`frameworks/README.md`](frameworks/README.md)

## Scan a target you control

```bash
# Bare host (defaults to https://), or http(s):// / //host
cargo run --bin weeping-angel -- scan app.example.com \
  --i-own-this \
  --allow-host app.example.com \
  --profile standard \
  -o report \
  --format terminal,json,sarif,html
```

Consent accepts bare `--i-own-this` **or** `--i-own-this=true|yes|1` (value requires `=`).  
`--allow-host` accepts CSV, wildcards (`*.example.com`), and full URLs.  
Optional: `--allow-host-from-target`, `--fast` (higher rps/concurrency), `--log-http full|compact|summary|off`.

### Docs site

```bash
pnpm --dir apps/docs install
pnpm --dir apps/docs dev
```

Auto-generated from clap via `weeping-angel-docs-export` (see `apps/docs/README.md`).

### Lab demo (local, intentionally weak)

```bash
# terminal 1
cargo run --example weeping-angel-demo --features demo
# listens on http://127.0.0.1:8787

# terminal 2
cargo run --bin weeping-angel -- scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --profile deep \
  --enable-active \
  --probe xss,sqli,open-redirect,path-traversal \
  --cookie "session=admin-session" \
  --compare-auth \
  --ignore-robots \
  -o report-lab \
  --format terminal,json,html
```

## Dependency confusion scanner (`depcheck`)

Detection-only multi-format dependency confusion scanner (DepCheck-compatible parsers).
Does **not** publish packages or generate exploit payloads.

```bash
# Scan a single manifest (auto-detects format)
weeping-angel depcheck package.json
weeping-angel depcheck Cargo.toml
weeping-angel depcheck requirements.txt

# confused-compatible language + known-secure namespaces (npm scopes, etc.)
weeping-angel depcheck -l npm package.json
weeping-angel depcheck -l npm -s '@mycompany/*' package.json
weeping-angel depcheck -l pip requirements.txt
weeping-angel depcheck -l mvn pom.xml
weeping-angel depcheck -l rubygems Gemfile.lock
weeping-angel depcheck -l composer composer.json

# DepFuzzer-compatible provider / path / single dependency / email heuristics
weeping-angel depcheck --provider pypi --path ~/Projects/MyApp
weeping-angel depcheck --provider npm --dependency left-pad:1.3.0 --check-email
weeping-angel depcheck --provider npm --dependency acme-private --print-takeover --output-file takeover.txt
weeping-angel depcheck --provider all --path ./my-app/

# Loki-style inspector + npm hardening recon (no attack / publish / reverse-shell)
weeping-angel depcheck -d ./app --inspect --entrypoint index.js
weeping-angel depcheck ./app -i   # git commit that introduced each free-namespace dep

# DepenFusion-style remote hunt: probe hosts for exposed package.json / package-lock.json
cat subdomains.txt | weeping-angel depcheck --stdin --i-own-this --threads 20
weeping-angel depcheck --hosts-file hosts.txt --i-own-this --link --silent
weeping-angel depcheck --hosts-file hosts.txt --i-own-this --append "?token=foo" --strip-path

# Scan a project tree (finds known manifests recursively)
weeping-angel depcheck ./my-app/

# List packages only / convert / export
weeping-angel depcheck --list package-lock.json
weeping-angel depcheck --convert pnpm-lock.yaml
weeping-angel depcheck --export results.json composer.lock

# Fetch a remote manifest (requires consent)
weeping-angel depcheck --url https://example.com/package.json --i-own-this

# Quiet: only vulnerable (free-namespace) names
weeping-angel depcheck -q package.json

# Local scan-only Web UI (default http://127.0.0.1:8443)
weeping-angel depcheck --web
weeping-angel depcheck --web --port 9090
```

**Interpreting results (confused / DepFuzzer / Loki recon / Alex Birsan):**
- **Missing on public registry** → free namespace an attacker could claim (dependency confusion).
- **`--inspect`**: git commit that first introduced each free-namespace dependency (requires git).
- **Hardening recon** (npm, on by default): `.npmrc` hybrid/public fallback, floating `^`/`~`
  ranges on private candidates, missing `@scope:registry`, entrypoint presence.
- **`--check-email`** (packages that *exist*): disposable / possibly purchasable maintainer
  domains (heuristic). Detection only — does not register domains, publish packages, or open shells.
- npm scopes: use `-s '@org/*'` for scopes you already own.

**Remote hunt:** for each host/URL, probe common dependency paths (npm/yarn/pnpm, pip, Composer,
RubyGems, Maven/Gradle, Go, Cargo, NuGet), check public registries, print missing names (and
`=>` source URLs unless `--silent`). Requires `--i-own-this`. Detection only.

Docs: `apps/docs` → **Dependency confusion** (theory, recon commands, mitigation — no exploit recipes).

**Not implemented (offensive surfaces from Loki / marketing copy):** npm publish of PoC packages,
reverse-shell payloads, `--attack` / payload injection, automated exploitation.

Supported formats: `package.json`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`,
`requirements.txt`, `Pipfile`, `Pipfile.lock`, `pyproject.toml`, `composer.json`/`.lock`,
`Gemfile`/`.lock`, `pom.xml`, `build.gradle`/`.kts`, `go.mod`, `Cargo.toml`/`.lock`,
`packages.config`, `*.csproj`.

`scan-code` can optionally run the same registry checks when `WA_DEPCHECK_NETWORK=1` is set.

## Modules & profiles

| Profile | Modules |
|---------|---------|
| **recon** | discovery, headers, tls, cookies, secrets, exposures, tech, **firebase** |
| **standard** | recon + cors, auth-surface, **rate-limits**, wordlist, **templates** |
| **deep** | standard + openapi, auth-compare + all active probes if `--enable-active` |

| Module | Role |
|--------|------|
| `discovery` | HTML crawl, robots/sitemap, JS endpoints, **SPA**, **image hosting patterns** (`/assets/images/{section}/{name}.{ext}`) |
| `wordlist` | Common path probing |
| `templates` | YAML path templates in `templates/` (Nuclei-lite) |
| `headers` / `tls` / `cookies` | Hardening signals |
| `secrets` | Client-visible credential patterns |
| `exposures` | `.env`, `.git`, phpinfo, actuator, dir listing, traces |
| `cors` | Wildcard / reflected Origin |
| `auth-surface` | Login/signup forms, **guarded vs unauthenticated** summary, admin paths, session cookies |
| `auth-compare` | Anon vs `--cookie` / `Authorization` (enable with `--compare-auth`) |
| `firebase` | **Firestore / Firebase** client config, project IDs, Auth/RTDB surfaces, weakness checklist |
| `rate-limits` | Per-route **429 / RateLimit headers**; optional light burst on auth paths with `--enable-active` |
| `tech` | Light fingerprinting (includes Firebase/Firestore signatures) |
| Active: `xss`, `sqli`, `open-redirect`, `path-traversal` | Opt-in via `--enable-active` |

### Artifacts (report formats)

```bash
cargo run --bin weeping-angel -- scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --profile deep \
  -o report-lab \
  --format terminal,json,html,manifest,openapi
```

| Format | Output | Contents |
|--------|--------|----------|
| `manifest` | `*.manifest.json` | Route inventory, auth guesses, Firebase signals, rate-limit map, embedded image harvest |
| `openapi` | `*.openapi.json` | **Synthesized** OpenAPI 3.0 from discovered routes + findings |
| `images` | `*.images.json` | **Full image harvest**: every `img`/srcset/CSS/JS path + **OPTIONS preflight** + **HEAD** status/type/length |

## Templates

Drop YAML files under `templates/`:

```yaml
id: my-check
name: Example
severity: high
paths:
  - /.env
matchers:
  - type: status
    status: [200]
  - type: body
    regex:
      - "(?i)secret\\s*="
```

```bash
cargo run --bin weeping-angel -- scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --templates-dir templates
```

## Safety defaults

- Requires `--i-own-this` **and** `--allow-host` (supports `*.example.com`)
- Rate limit (`--rps`) + concurrency caps
- Response body cap 2 MiB; redirects re-validated against allowlist
- No POST/PUT/PATCH/DELETE unless `--allow-write-methods`
- Active modules never run without `--enable-active`
- robots.txt honored unless `--ignore-robots`
- Demo binds **127.0.0.1 only**

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | No findings ≥ `--fail-on` (default `medium`) |
| 1 | Findings at or above threshold |
| 2 | Tool / usage / consent error |

## Config file

Copy `weeping-angel.example.toml` → `weeping-angel.toml`:

```toml
[authorization]
i_own_this = true
allow_hosts = ["127.0.0.1", "localhost", "app.example.com"]

[scan]
profile = "standard"
fail_on = "medium"
```

## Development

```bash
cargo test --workspace --features demo
cargo run --bin weeping-angel -- --help
```

Scanner package stays at the repo root. Assurance crates live under `crates/`. Do not add `iso_27001` / `gdpr` / `soc2` fields to `SemanticFinding`.

## Disclaimer

For defensive testing of **authorized** targets only. Misuse may be illegal.
