import { homedir } from "os";
import { basename, resolve as resolvePath } from "path";
import { existsSync, readFileSync } from "fs";
import { detectActiveToken } from "./lib";

// Ported from maw-rs core_impl/token.rs token_cmd_resolve. Order matters:
// the oracle assignment wins over whatever .envrc happens to say, so a
// worktree inherits its oracle's token rather than the last one used here.
//
// Not ported: the fleet-config membership lookup that sits between the oracle
// and the .envrc fallback. It needs fleet session members, which this plugin
// does not read; when a fleet assignment exists maw-rs may answer differently.
export function resolveOracleFromCwd(cwd = process.cwd()): string | null {
  const registry = `${process.env.MAW_HOME || `${homedir()}/.maw`}/oracles.json`;
  try {
    const parsed = JSON.parse(readFileSync(registry, "utf8"));
    const here = resolvePath(cwd);
    const matches = (parsed.oracles ?? [])
      .filter((oracle: any) => typeof oracle?.local_path === "string" && oracle.local_path.trim())
      .map((oracle: any) => ({ path: resolvePath(oracle.local_path), name: oracle.name }))
      // Longest matching prefix wins, so a worktree inside an oracle resolves
      // to that oracle and not to a shorter parent path.
      .filter((oracle: any) => here === oracle.path || here.startsWith(`${oracle.path}/`))
      .sort((a: any, b: any) => b.path.split("/").length - a.path.split("/").length);
    if (matches.length && matches[0].name) return matches[0].name;
  } catch { /* no registry, fall through to the directory name */ }
  const name = basename(resolvePath(cwd)).replace(/-oracle$/, "");
  return name || null;
}

export function cmdResolve(cwd = process.cwd()): { ok: boolean; name?: string; error?: string } {
  const envrc = resolvePath(cwd, ".envrc");
  if (existsSync(envrc)) {
    const active = detectActiveToken(readFileSync(envrc, "utf8"));
    if (active) return { ok: true, name: active };
  }
  return { ok: false, error: "maw token resolve: no token assignment found" };
}
