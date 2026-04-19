// cmdOracleList/Scan/Fleet/About in maw-js span a multi-file subtree
// (impl-list, impl-about, impl-scan, impl-stale, impl-prune,
// impl-register, impl-helpers) with deep deps on oracle-registry,
// fleet.worktrees, lineageOf, and the registry cache — several thousand
// LOC total. That cannot be inlined under the 200 LOC/file budget.
//
// This standalone build ships stubs that throw at invoke time so the
// plugin LOADS cleanly (no module-resolution errors on `maw plugin
// install`). Replace with thin re-exports once the SDK exposes the
// cmdOracle* surface. Tracking: Soul-Brews-Studio/maw-js#402.

function stub(verb: string): never {
  throw new Error(
    `oracle ${verb} is installed but not yet wired for standalone mode. ` +
    "Tracking: Soul-Brews-Studio/maw-js#402 (SDK export for cmdOracle*). " +
    "For now, run this command from inside the maw-js monorepo.",
  );
}

export async function cmdOracleList(): Promise<void> { stub("ls"); }
export async function cmdOracleScan(_opts?: any): Promise<void> { stub("scan"); }
export async function cmdOracleFleet(_opts?: any): Promise<void> { stub("fleet"); }
export async function cmdOracleAbout(_name: string): Promise<void> { stub("about"); }
