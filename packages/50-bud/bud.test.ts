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
