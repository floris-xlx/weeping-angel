# weeping-angel

**v0.2.0** — authorized security toolchain and ISO 27001 *readiness* compiler, in Rust.

It does two jobs, on purpose, without mixing them:

1. **Scan** systems you own: live web recon/DAST, code SAST, and dependency-confusion detection.
2. **Assess** control readiness: collectors feed a canonical catalog; framework packs project onto that catalog; results are explainable, not a certificate.

Automated output is a **readiness assessment**. It is never ISO certification, never “audit passed,” never “compliant.”

> **Legal.** Only scan systems you **own** or have **written permission** to test. Web scans refuse to run without `--i-own-this` and a host allowlist.

Repository: [github.com/floris-xlx/weeping-angel](https://github.com/floris-xlx/weeping-angel) · License: MIT

---

## Install

```powershell
# Windows
.\scripts\setup.ps1
.\scripts\demo-scan.ps1
```

```bash
# Unix
chmod +x scripts/*.sh
./scripts/setup.sh
./scripts/demo-scan.sh
```

```bash
cargo build --release
# binary: target/release/weeping-angel
```

Installers (deb, msi, nsis, rpm, AppImage, pacman, dmg, app) via [`cargo-packager`](https://github.com/crabnebula-dev/cargo-packager):

```bash
pnpm run installer:msi   # or installer:deb, installer:nsis, …
pnpm run installer       # every format this host can build
```

Artifacts land in `target/packager/`.

---

## CLI

```text
weeping-angel scan         # live web recon / DAST
weeping-angel scan-code    # algorithmic SAST (tree)
weeping-angel scan-diff    # SAST on a git change-set
weeping-angel finalize     # seal a scan bundle (alias: seal)
weeping-angel workbench    # local SQLite register of sealed scans
weeping-angel depcheck     # dependency confusion (detection only)
weeping-angel assurance    # catalog / pack / assess / SoA / explain
weeping-angel completions  # shell completions
```

```bash
weeping-angel --help
weeping-angel --version
```

---

## Scan (authorized targets)

Consent is `--i-own-this` (or `--i-own-this=true|yes|1`) **and** `--allow-host` (CSV, `*.example.com`, or full URLs). `--allow-host-from-target` copies hosts from the scan target.

```bash
weeping-angel scan app.example.com \
  --i-own-this \
  --allow-host app.example.com \
  --profile standard \
  -o report \
  --format terminal,json,sarif,html
```

| Profile | What it runs |
| --- | --- |
| `recon` | discovery, headers, tls, cookies, secrets, exposures, tech, firebase |
| `standard` | recon + cors, auth-surface, rate-limits, wordlist, templates |
| `deep` | standard + openapi, auth-compare; all active probes if `--enable-active` |

Active probes (`xss`, `sqli`, `open-redirect`, `path-traversal`) never run without `--enable-active`. POST/PUT/PATCH/DELETE need `--allow-write-methods`. `robots.txt` is honored unless `--ignore-robots`.

| Format | File | Contents |
| --- | --- | --- |
| `terminal` / `json` / `sarif` / `html` | `report.*` | findings |
| `manifest` | `*.manifest.json` | route inventory, auth guesses, rate-limit map |
| `openapi` | `*.openapi.json` | synthesized OpenAPI 3.0 from discovered routes |
| `images` | `*.images.json` | harvested `img` / srcset / CSS / JS image paths |

YAML path templates live under `templates/` (`--templates-dir`). Copy `weeping-angel.example.toml` → `weeping-angel.toml` for defaults.

| Exit | Meaning |
| --- | --- |
| 0 | no findings ≥ `--fail-on` (default `medium`) |
| 1 | findings at or above threshold |
| 2 | usage / consent / tool error |

### Local lab

```bash
# terminal 1 — binds 127.0.0.1 only
cargo run --example weeping-angel-demo --features demo

# terminal 2
weeping-angel scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --profile deep --enable-active \
  --probe xss,sqli,open-redirect,path-traversal \
  --cookie "session=admin-session" --compare-auth --ignore-robots \
  -o report-lab --format terminal,json,html
```

### Code SAST and workbench

`scan-code` / `scan-diff` emit Codex Security–compatible sealed bundles (`report.md` + coverage). `finalize` validates and seals. `workbench` lists them in a local SQLite store.

```bash
weeping-angel scan-code . -o out/code --fail-on high
weeping-angel scan-diff --repo . -o out/diff --base main --head HEAD
weeping-angel workbench list
```

Set `WA_DEPCHECK_NETWORK=1` to add registry lookups during `scan-code`.

---

## Dependency confusion (`depcheck`)

Detection only. It does **not** publish packages, register domains, or generate exploit payloads.

```bash
weeping-angel depcheck package.json
weeping-angel depcheck -l npm -s '@mycompany/*' package.json
weeping-angel depcheck ./my-app/
weeping-angel depcheck --web --port 8443          # local UI, 127.0.0.1
```

A name **missing on the public registry** is a free namespace someone else could claim. `--inspect` (`-i`) finds the git commit that introduced each free-namespace dep. Remote host hunting (`--stdin` / `--hosts-file`) still requires `--i-own-this`.

Supported manifests include npm/yarn/pnpm, pip/Pipenv/pyproject, Composer, Bundler, Maven/Gradle, Go, Cargo, and NuGet. Theory and mitigation (no exploit recipes) live in `apps/docs`.

---

## Assurance (readiness, not certification)

The scanner emits **security findings**. The assurance runtime consumes **evidence** and produces **control-test results**. Those are different types. Collectors advertise evidence types, never frameworks. Empty findings are not `Effective`. A document on disk is not operational effectiveness.

```text
Providers → Collectors → Canonical evidence → Ledger (current / as-of)
        → Canonical tests → Control assessments
        → AssessmentRun (immutable lineage)
        → Readiness / SoA / Explain  →  framework projection
```

Canonical controls, tests, and evidence requirements live in [`catalog/canonical/v1`](catalog/canonical/v1) (`control.*` / `test.*` / `evidence.*`). Framework packs under [`frameworks/`](frameworks/) **map onto** that catalog; they do not redefine it. Shipped packs: `iso-27001/2022` (structural ISO 27001:2022, no normative clause text) and `wa-baseline/1`.

```bash
weeping-angel assurance catalog validate
weeping-angel assurance catalog stats
weeping-angel assurance catalog inspect control.identity.mfa

weeping-angel assurance framework validate frameworks/iso-27001/2022
weeping-angel assurance assess --framework iso-27001 --scope .
weeping-angel assurance soa
weeping-angel assurance explain --assessment <id> --control control.identity.privileged-mfa
```

Library entry:

```text
AssuranceEngine::builder().collector(…).framework(target).assess(scope)
AssuranceScheduler::builder().clock(…).store(…).ledger(…).register(JobSpec).tick()
```

Continuous scheduling is library `tick`, not a clap loop. Reports pin catalog and pack digests and serialize from those pins.

---

## Workspace

Root package `weeping-angel` is the CLI (scanner + assurance facade). Assurance libraries live under `crates/`. `xtask` is repository law, not a product crate.

```text
weeping-angel-assurance-ir          framework-neutral IR
        ├── weeping-angel-framework         pack parse + compile
        ├── weeping-angel-canonical-catalog offline catalog
        └── weeping-angel-evidence          ledger, validity, typed values
                └── weeping-angel-collector evidence types only (GitHub, local, …)

ir + evidence → weeping-angel-control-test     offline, provider-blind
framework + collector + control-test + root
              → weeping-angel-assurance        public facade
```

Ownership of catalog, framework compilation, readiness, temporal selection, lineage, persistence, and CLI is declared in [`architecture/architecture.toml`](architecture/architecture.toml) and enforced by `cargo xtask guard`.

Do not add `iso_27001` / `gdpr` / `soc2` fields to scanner findings. There is no `weeping-angel-catalog` or `weeping-angel-assurance-cli` package — those names are forbidden.

---

## Docs

Human map: [`docs/README.md`](docs/README.md).

| Path | Role |
| --- | --- |
| [`docs/specs/`](docs/specs/) | specifications |
| [`docs/adr/`](docs/adr/) | decisions |
| [`architecture/`](architecture/) | ownership, invariants, forbidden patterns |
| [`docs/debt/`](docs/debt/) | technical-debt register; mechanical [`current.md`](docs/debt/current.md) via `cargo xtask inventory` |
| [`tests/contracts/`](tests/contracts/) | executable dual-suite invariants |
| [`schemas/codex-security/`](schemas/codex-security/) | Codex Security JSON Schema SSOT |
| [`frameworks/`](frameworks/) | versioned regime packs |
| [`catalog/canonical/v1`](catalog/canonical/v1) | canonical controls / tests / evidence |
| [`apps/docs`](apps/docs/) | Fumadocs CLI site (generated from clap) |
| `.sdd/` | generated SDD traces (gitignored) |

Start here if you are reading architecture, not running scans:

- [Assurance runtime](docs/specs/assurance-runtime.md)
- [Canonical catalog](docs/specs/canonical-assurance-catalog-v1.md)
- [ISO 27001 vertical](docs/adr/0002-iso-27001-assurance-vertical.md)
- [Collector hexagonal modular monolith](docs/specs/collector-hexagonal.md) ([ADR 0013](docs/adr/0013-collector-hexagonal-modular-monolith.md))
- [GitHub collector evidence contract](docs/specs/github-collector.md)
- [Documentation layout](docs/adr/0004-documentation-architecture.md)
- [Repository health gate](docs/adr/0009-repository-health-gate.md)
- [Repository hygiene](docs/specs/repository-hygiene.md)
- [Structural reconciliation](docs/specs/structural-reconciliation.md) ([ADR 0048](docs/adr/0048-structural-reconciliation.md))

```bash
pnpm --dir apps/docs install
pnpm --dir apps/docs dev
```

---

## Development

```bash
cargo test --workspace --features demo
cargo xtask guard
cargo xtask guard --json
cargo xtask guard --check 04
cargo xtask guard --explain INV-INVARIANTS-EVALUATED
cargo xtask inventory --json
cargo xtask inventory --markdown
cargo xtask inventory --check
```

`cargo xtask guard` is the architecture gate: manifests, ownership, forbidden patterns, evaluated invariants, debt register, and active-spec drift (Guard 15). Silent skips are not allowed; a skip must cite a live finding in [`docs/debt/register.toml`](docs/debt/register.toml).

`cargo xtask inventory` is the mechanical count / debt-snapshot tool ([ADR 0048](docs/adr/0048-structural-reconciliation.md)): regenerate [`docs/debt/current.md`](docs/debt/current.md) with `--markdown`, verify with `--check`. [`docs/debt/baseline-2026-08.md`](docs/debt/baseline-2026-08.md) is Historical only.

---

## Safety defaults

- Web scans require `--i-own-this` **and** an allowlist
- Rate limit (`--rps`) and concurrency caps
- Response bodies capped at 2 MiB; redirects re-checked against the allowlist
- No write methods unless `--allow-write-methods`
- Active modules off unless `--enable-active`
- Demo server binds **127.0.0.1 only**
- `depcheck` never publishes or weaponizes a namespace

Misuse may be illegal. Use it on systems you are authorized to test.
