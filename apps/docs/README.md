# weeping-angel docs

Fumadocs documentation for the weeping-angel CLI, auto-generated from the clap command tree.

```bash
# From repo root
pnpm --dir apps/docs install
pnpm --dir apps/docs dev

# Regenerate CLI JSON + MDX
pnpm --dir apps/docs run generate:docs
```

`generate:docs` runs:

1. `cargo run --bin weeping-angel-docs-export` → `generated/cli-root-reference.json`
2. `scripts/generate-docs.mjs` → `content/docs/**/*.mdx` + sidebar `meta.json`

CI / environments without Rust can set `WEEPING_ANGEL_DOCS_USE_CLI_SNAPSHOTS=1` to reuse the committed JSON snapshot.
