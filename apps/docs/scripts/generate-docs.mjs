import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(scriptDir, "..");
const contentDocsDir = path.join(appRoot, "content", "docs");
const generatedDir = path.join(appRoot, "generated");
const rootReferencePath = path.join(generatedDir, "cli-root-reference.json");

const GENERATED_HEADER =
  "{/* Generated from clap export by scripts/generate-docs.mjs */}";

function writeFile(relativePath, content) {
  const absolutePath = path.join(appRoot, relativePath);
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true });
  fs.writeFileSync(absolutePath, content, "utf8");
}

function renderPage({ title, description, body }) {
  return [
    "---",
    `title: ${JSON.stringify(title)}`,
    `description: ${JSON.stringify(description)}`,
    "---",
    "",
    GENERATED_HEADER,
    "",
    body.trim(),
    "",
  ].join("\n");
}

function readRootReference() {
  if (!fs.existsSync(rootReferencePath)) {
    throw new Error(
      `Missing ${rootReferencePath}. Run generate:cli-reference first.`,
    );
  }
  return JSON.parse(fs.readFileSync(rootReferencePath, "utf8"));
}

function argTable(args) {
  if (!args?.length) return "_No arguments._";
  const rows = args.map((a) => {
    const name = a.display || a.long || a.short || a.id;
    const help = (a.help || "").replace(/\|/g, "\\|").replace(/\n/g, " ");
    const def = a.default_values?.length
      ? a.default_values.join(", ")
      : "—";
    return `| \`${name}\` | ${help || "—"} | \`${def}\` |`;
  });
  return [
    "| Flag / arg | Description | Default |",
    "|---|---|---|",
    ...rows,
  ].join("\n");
}

function commandBody(node) {
  const parts = [
    node.about ? `> ${node.about}` : "",
    "",
    "## Usage",
    "",
    "```bash",
    node.usage || node.display_name,
    "```",
    "",
  ];
  if (node.long_about) {
    parts.push("## Details", "", node.long_about, "");
  }
  parts.push("## Arguments", "", argTable(node.arguments), "");
  if (node.subcommands?.length) {
    parts.push("## Subcommands", "");
    for (const sc of node.subcommands) {
      parts.push(
        `- [\`${sc.name}\`](/docs/commands/${sc.name}) — ${sc.about || ""}`,
      );
    }
    parts.push("");
  }
  return parts.join("\n");
}

const root = readRootReference();

// Static hand-authored pages
const staticPages = [
  {
    slug: "index",
    title: "weeping-angel",
    description: "Authorized web recon and security scanning CLI",
    body: `# weeping-angel

Authorized **web recon + security scanning** CLI (Rust). Discover routes (including SPA/JS surfaces), flag misconfigurations and exposed secrets, map auth, run YAML path templates, compare authenticated vs anonymous access, and optionally fire **gated** active probes.

> **Legal:** Only scan systems you **own** or have written permission to test. The tool refuses to run without \`--i-own-this\` and a host allowlist.

## Quick start

\`\`\`bash
weeping-angel scan example.com \\
  --i-own-this \\
  --allow-host example.com \\
  --profile standard \\
  -o report \\
  --format terminal,json,html
\`\`\`

Targets accept bare hosts, \`//host\`, \`http://\`, or \`https://\`. Consent accepts bare \`--i-own-this\` or \`--i-own-this=true|yes|1\`.

## Docs map

- [Installation](/docs/installation)
- [Safety & authorization](/docs/safety)
- [CLI reference](/docs/cli)
- [Modules & profiles](/docs/modules)
- [Report formats](/docs/reports)
- [Lab demo](/docs/lab-demo)
`,
  },
  {
    slug: "installation",
    title: "Installation",
    description: "Build and install weeping-angel",
    body: `# Installation

\`\`\`bash
cargo build --release
# binary: target/release/weeping-angel
\`\`\`

Windows:

\`\`\`powershell
.\\scripts\\setup.ps1
\`\`\`

Unix:

\`\`\`bash
chmod +x scripts/*.sh
./scripts/setup.sh
\`\`\`

Optional lab demo:

\`\`\`bash
cargo build --release --example weeping-angel-demo --features demo
\`\`\`
`,
  },
  {
    slug: "safety",
    title: "Safety & authorization",
    description: "Consent gates and allowlists",
    body: `# Safety & authorization

weeping-angel will not scan without explicit ownership consent.

| Control | Flag | Notes |
|---|---|---|
| Consent | \`--i-own-this\` / \`--i-own-this=true\` | Required. \`=false\` is rejected loudly |
| Allowlist | \`--allow-host\` | Repeatable; CSV ok; supports \`*.example.com\` and full URLs |
| From target | \`--allow-host-from-target\` | Copies hosts from targets (still needs consent) |
| Active probes | \`--enable-active\` | Second gate for intrusive checks |
| Writes | \`--allow-write-methods\` | Blocks POST/PUT/PATCH/DELETE by default |

Never auto-consent. Config file \`i_own_this = true\` is allowed only when you intentionally set it.
`,
  },
  {
    slug: "cli",
    title: "CLI overview",
    description: "Root command tree",
    body: commandBody(root.command),
  },
  {
    slug: "modules",
    title: "Modules & profiles",
    description: "recon, standard, deep",
    body: `# Modules & profiles

| Profile | Modules |
|---|---|
| **recon** / quick | discovery, headers, tls, cookies, secrets, exposures, tech, firebase |
| **standard** | recon + cors, auth-surface, rate-limits, wordlist, templates |
| **deep** / full | standard + openapi, auth-compare + active probes if \`--enable-active\` |

Use \`--modules\` to override (comma or space separated). Active probes: \`--probe xss,sqli,open-redirect,path-traversal\` with \`--enable-active\`.
`,
  },
  {
    slug: "reports",
    title: "Report formats",
    description: "terminal, json, html, sarif, manifest, openapi, images",
    body: `# Report formats

\`\`\`bash
weeping-angel scan 127.0.0.1:8787 --i-own-this --allow-host 127.0.0.1 \\
  -o report --format terminal,json,html,manifest,openapi,images
\`\`\`

| Format | Output | Contents |
|---|---|---|
| terminal | stderr | Wide multi-section colored report |
| json | \`*.json\` | Full findings + phases + surface inventory |
| html | \`*.html\` | Dashboard with filters |
| sarif | \`*.sarif.json\` | SARIF 2.1 for CI |
| manifest | \`*.manifest.json\` | Route inventory |
| openapi | \`*.openapi.json\` | Synthesized OpenAPI 3 |
| images | \`*.images.json\` | Image harvest HEAD/OPTIONS |

Terminal width: \`--report-width\`, route cap: \`--max-terminal-routes\`.
`,
  },
  {
    slug: "lab-demo",
    title: "Lab demo",
    description: "Local intentionally weak target",
    body: `# Lab demo

\`\`\`bash
# terminal 1
cargo run --example weeping-angel-demo --features demo

# terminal 2
weeping-angel scan 127.0.0.1:8787 \\
  --i-own-this --allow-host 127.0.0.1 \\
  --profile deep --enable-active \\
  --probe xss,sqli,open-redirect,path-traversal \\
  --cookie "session=admin-session" \\
  --compare-auth --ignore-robots \\
  --fast \\
  -o report-lab --format terminal,json,html
\`\`\`

Loopback hosts default to **http** when the scheme is omitted.
`,
  },
];

for (const page of staticPages) {
  writeFile(
    path.join("content", "docs", `${page.slug}.mdx`),
    renderPage(page),
  );
}

// Command pages from clap tree
const commandsDir = path.join(contentDocsDir, "commands");
if (fs.existsSync(commandsDir)) {
  fs.rmSync(commandsDir, { recursive: true, force: true });
}

const commandMeta = [];
for (const sc of root.command.subcommands || []) {
  const slug = `commands/${sc.name}`;
  writeFile(
    path.join("content", "docs", `${slug}.mdx`),
    renderPage({
      title: sc.display_name || sc.name,
      description: sc.about || `weeping-angel ${sc.name}`,
      body: commandBody(sc),
    }),
  );
  commandMeta.push(sc.name);
}

writeFile(
  path.join("content", "docs", "commands", "meta.json"),
  JSON.stringify({ title: "Commands", pages: commandMeta }, null, 2) + "\n",
);

writeFile(
  path.join("content", "docs", "meta.json"),
  JSON.stringify(
    {
      title: "weeping-angel",
      pages: [
        "index",
        "installation",
        "safety",
        "cli",
        "modules",
        "reports",
        "lab-demo",
        "---",
        "commands",
      ],
    },
    null,
    2,
  ) + "\n",
);

console.log(
  `Generated ${staticPages.length} docs pages + ${commandMeta.length} command pages`,
);
