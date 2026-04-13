import { describe, it, expect, mock } from "bun:test";
import { join } from "path";
import type { InvokeContext } from "../../../plugin/types";

const root = join(import.meta.dir, "../../..");

mock.module(join(root, "config"), () => ({
  loadConfig: () => ({
    namedPeers: [{ name: "white", url: "http://white.local:3456" }],
    peers: [],
  }),
  cfgTimeout: () => 2000,
}));

mock.module(join(root, "curl-fetch"), () => ({
  curlFetch: async (_url: string) => ({
    ok: true,
    status: 200,
    data: { enabled: false, tokenPreview: "" },
  }),
}));

const { default: handler } = await import("./index");

describe("ping plugin", () => {
  it("CLI surface — ping all peers returns ok", async () => {
    const ctx: InvokeContext = { source: "cli", args: [] };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("white");
  });

  it("CLI surface — ping specific node returns ok", async () => {
    const ctx: InvokeContext = { source: "cli", args: ["white"] };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("white");
  });

  it("API surface — ping all peers returns ok", async () => {
    const ctx: InvokeContext = { source: "api", args: {} };
    const result = await handler(ctx);
    expect(result.ok).toBe(true);
    expect(result.output).toContain("white");
  });

  it("CLI surface — unknown node returns ok:false", async () => {
    const origExit = process.exit;
    (process as any).exit = (code: number) => { throw new Error(`exit:${code}`); };
    try {
      const ctx: InvokeContext = { source: "cli", args: ["nonexistent-xyz"] };
      const result = await handler(ctx);
      expect(result.ok).toBe(false);
    } finally {
      (process as any).exit = origExit;
    }
  });
});
