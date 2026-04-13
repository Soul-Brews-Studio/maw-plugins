import { describe, it, expect, mock, beforeEach } from "bun:test";
import type { InvokeContext } from "../../../plugin/types";

mock.module("../../bud", () => ({
  cmdBud: async (name: string, _opts: any) => {
    console.log(`budding ${name}`);
  },
}));

describe("bud plugin", () => {
  let handler: (ctx: InvokeContext) => Promise<any>;

  beforeEach(async () => {
    const mod = await import("./index");
    handler = mod.default;
  });

  it("cli: basic bud", async () => {
    const result = await handler({ source: "cli", args: ["myoracle"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("budding myoracle");
  });

  it("cli: bud with flags", async () => {
    const result = await handler({ source: "cli", args: ["newbud", "--from", "neo", "--dry-run"] });
    expect(result.ok).toBe(true);
    expect(result.output).toContain("budding newbud");
  });

  it("cli: name starts with dash returns error", async () => {
    const result = await handler({ source: "cli", args: ["--unknown-flag"] });
    expect(result.ok).toBe(false);
    expect(result.error).toContain("looks like a flag");
  });
});
