import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(appRoot, "..", "..");
const generatedDir = path.join(appRoot, "generated");
const rootOutput = path.join(generatedDir, "cli-root-reference.json");

fs.mkdirSync(generatedDir, { recursive: true });

function hasCargo() {
  const result = spawnSync("cargo", ["--version"], {
    cwd: repoRoot,
    stdio: "ignore",
    shell: true,
  });
  return result.status === 0;
}

if (process.env.WEEPING_ANGEL_DOCS_USE_CLI_SNAPSHOTS === "1" && fs.existsSync(rootOutput)) {
  console.log("Using committed CLI snapshot (WEEPING_ANGEL_DOCS_USE_CLI_SNAPSHOTS=1)");
  process.exit(0);
}

if (!hasCargo()) {
  if (fs.existsSync(rootOutput)) {
    console.warn("cargo not found; reusing existing cli-root-reference.json");
    process.exit(0);
  }
  console.error("cargo required to generate CLI reference (or commit generated/cli-root-reference.json)");
  process.exit(1);
}

const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--bin",
    "weeping-angel-docs-export",
    "--",
    "--output",
    rootOutput,
  ],
  {
    cwd: repoRoot,
    stdio: "inherit",
    shell: true,
  },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

console.log(`Wrote ${path.relative(repoRoot, rootOutput)}`);
