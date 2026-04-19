import { listSessions, hostExec, tmux, buildCommandInDir } from "maw-js/sdk";

void buildCommandInDir;

export async function cmdTake(source: string, targetSession?: string) {
  const [srcSession, srcWindow] = source.includes(":") ? source.split(":", 2) : [source, ""];

  if (!srcWindow) {
    console.error("usage: maw take <session>:<window> [target-session]");
    console.error("  e.g. maw take neo:neo-skills pulse");
    throw new Error("usage: maw take <session>:<window> [target-session]");
  }

  let target = targetSession;
  const split = !target;

  if (split) {
    target = srcWindow;
    try {
      await hostExec(`tmux new-session -d -s '${target}'`);
    } catch (e: any) {
      if (!e.message?.includes("duplicate")) {
        throw new Error(`could not create session '${target}': ${e.message}`);
      }
    }
  }

  if (target === srcSession) {
    console.log("  \x1b[33m⚠\x1b[0m source and target are the same session");
    return;
  }

  const sessions = await listSessions();
  const srcSess = sessions.find(s => s.name.toLowerCase() === srcSession.toLowerCase());
  if (!srcSess) {
    throw new Error(`session '${srcSession}' not found`);
  }

  const srcWin = srcSess.windows.find(w =>
    w.name.toLowerCase() === srcWindow.toLowerCase() || String(w.index) === srcWindow
  );
  if (!srcWin) {
    throw new Error(`window '${srcWindow}' not found in session '${srcSession}'`);
  }

  let paneCwd = "";
  try {
    paneCwd = (await hostExec(`tmux display-message -t '${srcSess.name}:${srcWin.name}' -p '#{pane_current_path}'`)).trim();
  } catch { /* ok */ }

  try {
    await hostExec(`tmux move-window -s '${srcSess.name}:${srcWin.name}' -t '${target}:'`);
    if (split) {
      try { await hostExec(`tmux kill-window -t '${target}:1' 2>/dev/null`); } catch {}
    }
    void tmux;
    console.log(`  \x1b[32m✓\x1b[0m ${srcSess.name}:${srcWin.name} → ${target}${split ? " (new session)" : ""}`);
    if (paneCwd) {
      console.log(`  \x1b[90m  cwd: ${paneCwd}\x1b[0m`);
    }
  } catch (e: any) {
    throw new Error(`move failed: ${e.message || e}`);
  }
}
