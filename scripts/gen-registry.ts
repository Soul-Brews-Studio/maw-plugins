#!/usr/bin/env bun
/**
 * gen-registry.ts — regenerate registry.json deterministically from packages/*.
 *
 * registry.json is the default mawx resolution root (mawx WI-9). Nat's decision
 * 2 makes Soul-Brews-Studio/maw-plugins an AUTO-TRUST root: a pinned entry here
 * runs on a client with NO first-run prompt. So every entry's sha256 carries
 * real security weight and MUST be a pin that a fetched plugin.wasm is verified
 * against. This generator therefore refuses to invent pins — a package without a
 * resolvable wasm pin is skipped (and reported), never emitted with a null hash.
 *
 * Entry shape (flat map, keyed by cli.command verb):
 *   "<verb>": {
 *     "commit":       "<40-hex repo commit that anchors the immutable raw URL>",
 *     "sha256":       "sha256:<hex>",          // the manifest's artifact pin
 *     "path":         "packages/<dir>",
 *     "version":      "<manifest version>",
 *     "capabilities": [ ... ]                  // manifest order preserved
 *   }
 *
 * `commit` is the LAST commit that touched the package directory
 * (`git log -1 -- packages/<dir>`), not repo HEAD. That commit is guaranteed to
 * contain the package's current committed bytes, so
 * `raw.githubusercontent.com/<o>/<r>/<commit>/packages/<dir>/plugin.wasm` is
 * immutable and hashes to `sha256`. Using HEAD instead would make registry.json
 * reference a commit that does not yet exist at generation time, so the CI
 * staleness gate could never be green. Requires full git history — CI checks out
 * with fetch-depth: 0.
 *
 * Usage:
 *   bun run scripts/gen-registry.ts            # regenerate registry.json
 *   bun run scripts/gen-registry.ts --check    # verify freshness, no write (exit 1 on drift)
 */

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const repoRoot = resolve(dirname(new URL(import.meta.url).pathname), "..");
const packagesDir = join(repoRoot, "packages");
const registryPath = join(repoRoot, "registry.json");

interface Entry {
  commit: string;
  sha256: string;
  path: string;
  version: string;
  capabilities: string[];
}

interface Skip {
  dir: string;
  reason: string;
}

function readManifest(path: string): Record<string, unknown> | null {
  if (!existsSync(path)) return null;
  return JSON.parse(readFileSync(path, "utf8")) as Record<string, unknown>;
}

/** Pin sourcing mirrors the CI pin-integrity gate: plugin.json first, then a
 *  dev-tier-active package's plugin.source.json fallback. */
function resolvePin(dir: string, manifest: Record<string, unknown>): string | null {
  const fromJson = (manifest.artifact as { sha256?: string } | undefined)?.sha256;
  if (fromJson) return fromJson;
  const source = readManifest(join(dir, "plugin.source.json"));
  const fromSource = (source?.artifact as { sha256?: string } | undefined)?.sha256;
  return fromSource ?? null;
}

function lastCommitTouching(relPath: string): string {
  const out = execFileSync("git", ["-C", repoRoot, "log", "-1", "--format=%H", "--", relPath], {
    encoding: "utf8",
  }).trim();
  return out;
}

function build(): { registry: Record<string, Entry>; skipped: Skip[] } {
  const dirs = readdirSync(packagesDir)
    .filter((name) => statSync(join(packagesDir, name)).isDirectory())
    .sort();

  const registry: Record<string, Entry> = {};
  const skipped: Skip[] = [];
  const verbOrigin: Record<string, string> = {};

  for (const name of dirs) {
    const dir = join(packagesDir, name);
    const relPath = `packages/${name}`;
    const manifest = readManifest(join(dir, "plugin.json"));

    if (!manifest) {
      skipped.push({ dir: name, reason: "no plugin.json" });
      continue;
    }
    if (manifest.target !== "wasm") {
      skipped.push({ dir: name, reason: `target=${String(manifest.target)} (not wasm ship-tier; not mawx-resolvable)` });
      continue;
    }
    const verb = (manifest.cli as { command?: string } | undefined)?.command;
    if (!verb) {
      skipped.push({ dir: name, reason: "no cli.command (not a verb-resolvable plugin)" });
      continue;
    }
    if (!existsSync(join(dir, "plugin.wasm"))) {
      skipped.push({ dir: name, reason: "no plugin.wasm artifact" });
      continue;
    }
    const pin = resolvePin(dir, manifest);
    if (!pin) {
      skipped.push({ dir: name, reason: "no artifact.sha256 pin (would be an unpinnable auto-trust entry)" });
      continue;
    }
    if (verb in verbOrigin) {
      throw new Error(`duplicate verb '${verb}' from ${relPath} and ${verbOrigin[verb]}`);
    }
    const commit = lastCommitTouching(relPath);
    if (!/^[0-9a-f]{40}$/.test(commit)) {
      throw new Error(
        `could not resolve a commit for ${relPath} (got '${commit}'). Commit the package before generating, and ensure full git history (CI: fetch-depth: 0).`,
      );
    }

    verbOrigin[verb] = relPath;
    registry[verb] = {
      commit,
      sha256: pin,
      path: relPath,
      version: String(manifest.version ?? ""),
      capabilities: (manifest.capabilities as string[] | undefined) ?? [],
    };
  }

  // Re-insert keys in sorted order so JSON.stringify emits deterministic output.
  const sorted: Record<string, Entry> = {};
  for (const verb of Object.keys(registry).sort()) sorted[verb] = registry[verb];
  return { registry: sorted, skipped };
}

function serialize(registry: Record<string, Entry>): string {
  return `${JSON.stringify(registry, null, 2)}\n`;
}

const check = process.argv.includes("--check");
const { registry, skipped } = build();
const next = serialize(registry);

const included = Object.keys(registry);
console.error(`registry: ${included.length} entries — ${included.join(", ")}`);
for (const s of skipped) console.error(`  skip ${s.dir}: ${s.reason}`);

if (check) {
  const current = existsSync(registryPath) ? readFileSync(registryPath, "utf8") : "";
  if (current !== next) {
    console.error(
      "\nregistry.json is STALE. Run `bun run scripts/gen-registry.ts` and commit the result.",
    );
    process.exit(1);
  }
  console.error("registry.json is up to date.");
} else {
  writeFileSync(registryPath, next);
  console.error(`\nwrote ${registryPath}`);
}
