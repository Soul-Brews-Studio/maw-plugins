import { execSync } from "child_process";
import { loadConfig, cfgTimeout, curlFetch, tmux } from "maw-js/sdk";

export async function cmdHealth() {
  const checks: { name: string; status: string; detail: string }[] = [];

  try {
    const sessions = await tmux.listSessions();
    checks.push({ name: "tmux server", status: "ok", detail: `running (${sessions.length} sessions)` });
  } catch {
    checks.push({ name: "tmux server", status: "fail", detail: "not running" });
  }

  try {
    const port = loadConfig().port;
    const res = await fetch(`http://localhost:${port}/api/sessions`, { signal: AbortSignal.timeout(cfgTimeout("health")) });
    if (res.ok) {
      const data = await res.json();
      const count = Array.isArray(data) ? data.length : (data.sessions?.length || 0);
      checks.push({ name: "maw server", status: "ok", detail: `online (:${port}, ${count} sessions)` });
    } else {
      checks.push({ name: "maw server", status: "warn", detail: `HTTP ${res.status}` });
    }
  } catch {
    checks.push({ name: "maw server", status: "fail", detail: "offline" });
  }

  try {
    const df = execSync("df -h /tmp | tail -1", { encoding: "utf-8" }).trim();
    const parts = df.split(/\s+/);
    const avail = parts[3] || "?";
    const pct = parseInt(parts[4] || "0");
    checks.push({ name: "disk /tmp", status: pct > 90 ? "warn" : "ok", detail: `${avail} free` });
  } catch {
    checks.push({ name: "disk /tmp", status: "warn", detail: "unknown" });
  }

  try {
    const mem = execSync("free -m | grep Mem", { encoding: "utf-8" }).trim();
    const parts = mem.split(/\s+/);
    const avail = parseInt(parts[6] || "0");
    checks.push({ name: "memory", status: avail < 500 ? "warn" : "ok", detail: `${avail}MB available` });
  } catch {
    checks.push({ name: "memory", status: "warn", detail: "unknown" });
  }

  try {
    const pm2 = execSync("pm2 jlist 2>/dev/null", { encoding: "utf-8" });
    const procs = JSON.parse(pm2);
    const maw = procs.find((p: any) => p.name === "maw");
    if (maw) {
      checks.push({ name: "pm2 maw", status: maw.pm2_env?.status === "online" ? "ok" : "warn", detail: `${maw.pm2_env?.status} (pid ${maw.pid})` });
    } else {
      checks.push({ name: "pm2 maw", status: "fail", detail: "not found" });
    }
  } catch {
    checks.push({ name: "pm2 maw", status: "warn", detail: "pm2 not available" });
  }

  const config = loadConfig();
  const rawPeers: string[] = config.peers || [];
  const namedPeers: { name: string; url: string }[] = config.namedPeers || [];
  const allPeers: { label: string; url: string }[] = [
    ...rawPeers.map(url => ({ label: url, url })),
    ...namedPeers.map(p => ({ label: `${p.name} (${p.url})`, url: p.url })),
  ];
  if (allPeers.length === 0) {
    checks.push({ name: "peers", status: "none", detail: "none configured" });
  } else {
    for (const peer of allPeers) {
      try {
        const r = await curlFetch(`${peer.url}/api/federation/status`, { timeout: cfgTimeout("health") });
        checks.push({ name: `peer ${peer.label}`, status: r.ok ? "ok" : "warn", detail: r.ok ? "online" : `HTTP ${r.status}` });
      } catch {
        checks.push({ name: `peer ${peer.label}`, status: "fail", detail: "unreachable" });
      }
    }
  }

  console.log("\nmaw health\n");
  for (const c of checks) {
    const icon = c.status === "ok" ? "\x1b[32m●\x1b[0m" : c.status === "warn" ? "\x1b[33m●\x1b[0m" : c.status === "fail" ? "\x1b[31m●\x1b[0m" : "\x1b[90m○\x1b[0m";
    console.log(`  ${icon} ${c.name.padEnd(18)} ${c.detail}`);
  }
  console.log();
}
