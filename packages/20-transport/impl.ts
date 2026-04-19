import { loadConfig } from "maw-js/sdk";

// Standalone transport status — config view only.
// The upstream impl also reads live transport router state via
// getTransportRouter(), which is not yet on the SDK surface. When that
// export lands, restore the live-router section. Tracking: maw-js#402.
export async function cmdTransportStatus() {
  const config = loadConfig() as any;
  const node = config.node ?? "local";
  console.log(`\n\x1b[36;1mTransport Status\x1b[0m  \x1b[90m(node: ${node})\x1b[0m\n`);

  const peers = config.peers ?? [];
  const namedPeers = config.namedPeers ?? [];
  const peerCount = peers.length + namedPeers.length;

  const rows = [
    { name: "tmux", note: "local" },
    { name: "http-federation", note: peerCount ? `${peerCount} peer(s)` : "no peers" },
  ];

  for (let i = 0; i < rows.length; i++) {
    const r = rows[i];
    const dot = "\x1b[90m○\x1b[0m";
    console.log(`  ${i + 1}. ${dot}  ${r.name.padEnd(18)}  \x1b[90m(${r.note})\x1b[0m`);
  }
  console.log(`\n  \x1b[90mnote: live router introspection requires a maw-js SDK export (tracked).\x1b[0m`);

  if (config.agents && Object.keys(config.agents).length > 0) {
    console.log(`\n  \x1b[36mAgent Registry:\x1b[0m`);
    for (const [agent, agentNode] of Object.entries(config.agents)) {
      const local = agentNode === node;
      const dot = local ? "\x1b[32m●\x1b[0m" : "\x1b[34m●\x1b[0m";
      console.log(`    ${dot} ${agent} → ${agentNode}${local ? " (local)" : ""}`);
    }
  }

  const hints: string[] = [];
  if (!peerCount) hints.push(`peers: "peers": ["http://host:3456"]`);
  if (!config.agents) hints.push(`agents: "agents": { "neo": "white" }`);
  if (hints.length > 0) {
    console.log(`\n  \x1b[90mConfigure in maw.config.json:\x1b[0m`);
    for (const h of hints) console.log(`    \x1b[90m${h}\x1b[0m`);
  }

  console.log();
}
