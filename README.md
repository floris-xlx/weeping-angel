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

Assurance contract tests: `sdd_assurance_runtime_target` (ACT-001…015), `sdd_iso27001_assurance_target` (ISO/EVD/CTL/GH), `sdd_iso27001_remap_target` (ISO-R catalog projection), `sdd_canonical_assurance_catalog_target` (CAT-001…016), `sdd_typed_evidence_target` (typed facts + `evidence-value/v1`), `sdd_population_runtime_target`, `sdd_iam_catalog_target` (IAM-001…016), `sdd_sdlc_catalog_target` (SDLC-001…016), `sdd_vulnerability_catalog_target` (VULN-001…016), `sdd_infrastructure_catalog_target` (INFRA-001…016), `sdd_governance_catalog_target` (GOV-001…016), `sdd_github_collector_target` (canonical GitHub collector, `ghc_000`–`ghc_024`), `sdd_applicability_engine_target` (P10-T01…T16 Kleene three-state), and `sdd_assessment_lineage_target` (LIN-001…015 immutable run / explain / pure serialize). Matching baseline suites are superseded / ignored.

## Assurance runtime (ISO 27001 readiness)

Weeping Angel is also an **inwardly extensible** assurance compiler: capabilities + observations in, control-test results out. Findings stay security-only. Collectors advertise evidence types, not frameworks. Automated output is a **readiness assessment**, never ISO certification.

```text
AssuranceEngine::builder().collector(…).framework(target).assess(scope)
```

ISO 27001:2022 ships as a versioned structural pack (`frameworks/iso-27001/2022`) that **projects** onto the canonical catalog. Pack mappings target `control.identity.*` and landed `control.source.*` IDs (`PartiallySatisfies` / `Supports`; never convenience `Equivalent`). Pack-local slivers (`access.mfa.privileged`, `source.branch-protection`, …) are retired. The reusable library is the offline canonical catalog (`catalog/canonical/v1`, schema `weeping-angel/canonical-catalog/v1`, IDs `control.*` / `evidence.*` / `test.*`). IAM tests are provider-neutral population predicates, not Entra/Okta/GitHub checks. SDLC tests (`control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*`) assess repository / CI / release / supply-chain populations from `evidence.repository.*` (and cicd / deployment / release / supply-chain) facts; default-branch protection is `control.source.default-branch-protection` (the exists-only fixture `control.source.protected-branch` remains); missing scan evidence is `InsufficientEvidence`. Vulnerability tests (`control.vulnerability.*`) treat scanner findings as evidence, not compliance results; accepted-risk is not remediation; empty findings plus unknown coverage are never Effective. Infrastructure tests (`control.network.*` / `control.crypto.*` / `control.secret.*` / `control.data.*` / `control.database.*` / `control.logging.*` / `control.backup.*` / operational `control.resilience.*`) are provider-neutral population predicates on `evidence.network.*` / `evidence.data.*` / `evidence.crypto.key-state` / `evidence.secret.storage-configuration` / `evidence.database.*` / `evidence.logging.*` / `evidence.backup.*` / `evidence.resilience.recovery-plan`; missing evidence is `InsufficientEvidence`; DR exercise, recovery objectives, and segmentation rationale stay hybrid/manual. Governance tests (`control.governance.*` / `control.risk.*` / `control.personnel.*` / `control.vendor.*` / `control.incident.*` plus continuity `control.resilience.business-continuity-plan` / `disaster-recovery-governance`) treat manual evidence as first-class immutable facts (`evidence.manual.attestation` and domain types); a document-present flag is not operational effectiveness; missing evidence is `InsufficientEvidence`; approved unexpired IR exceptions are `ExceptionApproved`, never silent `Effective`. Organizational ISO clauses stay unmapped. Observation facts are typed `EvidenceValue` (`evidence-value/v1`); `with_fact` remains string-compatible. `GitHubCollector` emits canonical `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / observable `evidence.identity.*` plus inventory envelopes — never `source.*` observations, never `evidence.github.*`, never framework status. Local / manual / scanner evidence also never writes framework status. Reports pin both pack and catalog digests, serialize from those pins (no pack load at serialize time), and never say certified / compliant / audit passed. An `AssessmentRun` is a returned execution record (`completed` / `partial` / `failed`) with distinct definition / evidence-snapshot / result / applicability identities. Replay uses pinned snapshots; a current-file digest mismatch is detected, not rewritten.

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

The clap family is `assurance {framework,collect,evidence,assess,result,compare,soa,catalog,explain}`. `assurance catalog` and `assurance explain` are dispatched (validate / stats / inspect; explain from a pinned ledger run). Other `assurance` subcommands print the non-certification banner; library `assess` / `project_readiness` / `project_soa` / `compare` / `explain_control` is their execution path.

Workspace crates: `weeping-angel-assurance-ir` → `framework` / `evidence` → `collector`; `ir` + `evidence` → `control-test`; `ir` → `canonical-catalog` (offline); facade `weeping-angel-assurance` composes the runtime. Framework and collector do not depend on the catalog crate.

- Decisions: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](docs/adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](docs/adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](docs/adr/0003-canonical-assurance-catalog-v1.md), [`docs/adr/0003-typed-evidence-canonical-serialization.md`](docs/adr/0003-typed-evidence-canonical-serialization.md), [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](docs/adr/0003-subject-population-runtime-and-coverage-semantics.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](docs/adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](docs/adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](docs/adr/0003-vulnerability-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](docs/adr/0003-infrastructure-canonical-assurance-catalog.md), [`docs/adr/0003-governance-canonical-assurance-catalog.md`](docs/adr/0003-governance-canonical-assurance-catalog.md), [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](docs/adr/0003-github-collector-canonical-evidence-mapping.md), [`docs/adr/0003-applicability-engine.md`](docs/adr/0003-applicability-engine.md), [`docs/adr/0003-assessment-lineage.md`](docs/adr/0003-assessment-lineage.md), [`docs/adr/0003-iso27001-canonical-remap.md`](docs/adr/0003-iso27001-canonical-remap.md)
- Contract: [`docs/contracts/assurance-runtime.md`](docs/contracts/assurance-runtime.md)
- Specs: [`docs/sdd/assurance-runtime-spine.md`](docs/sdd/assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](docs/sdd/iso-27001-automated-assurance-mvp.md), [`docs/sdd/iso-27001-canonical-remap.md`](docs/sdd/iso-27001-canonical-remap.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](docs/sdd/canonical-assurance-catalog-v1.md), [`docs/sdd/typed-evidence.md`](docs/sdd/typed-evidence.md), [`docs/sdd/population-runtime.md`](docs/sdd/population-runtime.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](docs/sdd/iam-canonical-assurance-catalog.md), [`docs/sdd/sdlc-canonical-assurance-catalog.md`](docs/sdd/sdlc-canonical-assurance-catalog.md), [`docs/sdd/vulnerability-canonical-assurance-catalog.md`](docs/sdd/vulnerability-canonical-assurance-catalog.md), [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](docs/sdd/infrastructure-canonical-assurance-catalog.md), [`docs/sdd/governance-canonical-assurance-catalog.md`](docs/sdd/governance-canonical-assurance-catalog.md), [`docs/sdd/github-collector.md`](docs/sdd/github-collector.md), [`docs/sdd/applicability-engine.md`](docs/sdd/applicability-engine.md), [`docs/sdd/assessment-lineage.md`](docs/sdd/assessment-lineage.md)
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
