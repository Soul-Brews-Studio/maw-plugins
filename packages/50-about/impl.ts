// cmdOracleAbout in maw-js composes oracle-registry lookups, lineageOf
// resolution, and tmux probing — several internal modules not yet on
// maw-js/sdk. Standalone stub until SDK export lands. Tracking:
// Soul-Brews-Studio/maw-js#402.

export async function cmdOracleAbout(_name: string): Promise<void> {
  throw new Error(
    "about plugin is installed but cmdOracleAbout is not yet wired for " +
    "standalone mode. Tracking: Soul-Brews-Studio/maw-js#402 (SDK export for " +
    "cmdOracleAbout). For now, run this command from inside the maw-js monorepo.",
  );
}
