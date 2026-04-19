// cmdBud in maw-js pulls in wake-target resolution, normalize-target,
// oracle-name validation, ensureBudRepo (git/gh plumbing), bud-init
// (vault + CLAUDE.md + fleet config), and bud-wake (tmux signal).
// None of that surface is currently exported from maw-js/sdk. Rather
// than copy a 600-LOC subtree into this plugin (and bloat past the
// 200-LOC/file budget), this standalone build ships a stub that
// errors at invoke time. The plugin still LOADS cleanly — critical
// for `maw plugin install` not to blow up at module-resolution.
//
// Follow-up: expose cmdBud via maw-js/sdk, then replace this file with
// a thin re-export. Tracking: Soul-Brews-Studio/maw-js#402.

export interface BudOpts {
  from?: string;
  repo?: string;
  org?: string;
  issue?: number;
  note?: string;
  fast?: boolean;
  root?: boolean;
  dryRun?: boolean;
}

export async function cmdBud(_name: string, _opts: BudOpts = {}): Promise<void> {
  throw new Error(
    "bud plugin is installed but cmdBud is not yet wired for standalone mode. " +
    "Tracking: Soul-Brews-Studio/maw-js#402 (SDK export for cmdBud). " +
    "For now, run bud from inside the maw-js monorepo.",
  );
}
