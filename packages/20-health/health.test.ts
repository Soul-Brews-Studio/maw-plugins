import { describe, it, expect, mock } from "bun:test";
import type { InvokeContext } from "maw-js/sdk";

mock.module("maw-js/sdk", () => ({
  loadConfig: () => ({ host: "localhost", port: 3456, peers: [] }),
  cfgTimeout: () => 2000,
  curlFetch: async (_url: string) => ({ ok: true, status: 200, data: {} }),
  tmux: {
    listSessions: async () => [{ name: "neo" }, { name: "white" }],
  },
}));

mock.module("child_process", () => ({
  execSync: (cmd: string) => {
    if (cmd.includes("df -h")) return "tmpfs   1G  500M  500M  50% /tmp\n";
    if (cmd.includes("free -m")) return "Mem:   8192   4096   2048   512   1024   2048\n";
    if (cmd.includes("pm2 jlist")) return JSON.stringify([{ name: "maw", pid: 1234, pm2_env: { status: "online" } }]);
    return "";
  },
}));

(global as any).fetch = mock(async (_url: string) => ({
  ok: true,
  json: async () => [{ name: "neo" }],
}));

const { default: handler } = await import("./index");

describe("health plugin", () => {
  it("CLI surface — returns ok with health output", async () => {
    const ctx: InvokeContext = { source: "cli", args: [] };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("maw health");
  });

  it("API surface — returns ok with health output", async () => {
    const ctx: InvokeContext = { source: "api", args: {} };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("maw health");
  });

  it("reports tmux server in output", async () => {
    const ctx: InvokeContext = { source: "cli", args: [] };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("tmux server");
  });

  it("accepts extra args gracefully (no-op)", async () => {
    const ctx: InvokeContext = { source: "cli", args: ["--verbose"] };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
  });
});
