import { describe, it, expect, mock, beforeEach } from "bun:test";
import type { InvokeContext } from "maw-js/sdk";

mock.module("maw-js/sdk", () => ({
  parseFlags: (args: string[], spec: Record<string, any>, positional = 0) => {
    const out: Record<string, any> = { _: [] as string[] };
    for (const k of Object.keys(spec)) out[k] = undefined;
    let i = 0;
    while (i < args.length) {
      const a = args[i];
      if (a in spec) {
        const kind = spec[a];
        if (kind === Boolean) { out[a] = true; i++; continue; }
        out[a] = kind === Number ? Number(args[i + 1]) : args[i + 1];
        i += 2;
        continue;
      }
      out._.push(a);
      i++;
    }
    void positional;
    return out;
  },
}));

mock.module("./impl", () => ({
  cmdOracleList: async () => {
    console.log("Oracle Fleet  (1/2 awake)");
  },
  cmdOracleScan: async (_opts: any) => {
    console.log("Scanned 5 oracles locally");
  },
  cmdOracleFleet: async (_opts: any) => {
    console.log("Oracle Fleet  (5 oracles)");
  },
  cmdOracleAbout: async (name: string) => {
    console.log(`Oracle — ${name}`);
  },
}));

describe("oracle plugin", () => {
  let handler: (ctx: InvokeContext) => Promise<any>;

  beforeEach(async () => {
    const mod = await import("./index");
    handler = mod.default;
  });

  it("cli: ls lists oracles", async () => {
    const result = await handler({ source: "cli", args: ["ls"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("Oracle Fleet");
  });

  it("cli: scan runs oracle scan", async () => {
    const result = await handler({ source: "cli", args: ["scan"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("Scanned");
  });

  it("cli: fleet shows fleet", async () => {
    const result = await handler({ source: "cli", args: ["fleet"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("Oracle Fleet");
  });

  it("cli: about <name> shows oracle details", async () => {
    const result = await handler({ source: "cli", args: ["about", "neo"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("Oracle — neo");
  });
});
