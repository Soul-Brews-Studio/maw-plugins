import { describe, it, expect, mock, beforeEach } from "bun:test";
import type { InvokeContext } from "../../../plugin/types";

mock.module("../../oracle", () => ({
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
