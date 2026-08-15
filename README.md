# weeping-angel

current version: `0.1.3`
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
# unit + integration + e2e (shared lab router requires demo feature)
cargo test --features demo
```

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
cargo test
cargo run --bin weeping-angel -- --help
```

## Disclaimer

For defensive testing of **authorized** targets only. Misuse may be illegal.
