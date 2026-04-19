# maw-plugins

> Installable plugins for [maw-js](https://github.com/Soul-Brews-Studio/maw-js). Each plugin is a self-contained package with one handler, all surfaces.

## Setup (dev clones)

```bash
bun run setup   # creates node_modules/maw-js so `--link` installs resolve
```

Required so plugin imports like `import … from "maw-js/sdk"` resolve when a
package is installed via `maw plugin install ./packages/<name> --link`. The
script locates maw-js via `$MAW_JS_PATH`, sibling `../maw-js`, or the bun
global install, and symlinks it into `./node_modules/`.

## Install a plugin

```bash
maw plugin install <path-or-url>
```

## Plugin structure

```
packages/
├── 00-wake/          ← weight 0: core, loads first
├── 00-sleep/
├── 00-hey/
├── 10-fleet-doctor/  ← weight 10: infrastructure
├── 10-oracle-scan/
├── 20-ping/          ← weight 20: tools
├── 20-health/
├── 50-artifacts/     ← weight 50: features
├── 50-contacts/
└── 90-dashboard/     ← weight 90: customizations, runs last
```

Number prefix = weight = execution order. Lower fires first. `ls` shows the boot sequence.

## Plugin anatomy

```
20-ping/
├── plugin.json          # manifest — declares ALL surfaces
├── index.ts             # handler(ctx) → InvokeResult
└── ping.test.ts         # tests cover cli + api + peer
```

### plugin.json

```json
{
  "name": "ping",
  "version": "1.0.0",
  "weight": 20,
  "entry": "./index.ts",
  "sdk": "^1.0.0",
  "cli": { "command": "ping", "help": "maw ping <target>" },
  "api": { "path": "/api/ping", "methods": ["GET"] },
  "hooks": { "on": ["SessionStart"] },
  "transport": { "peer": true }
}
```

### handler

```ts
import type { InvokeContext, InvokeResult } from "maw/sdk";

export default async function(ctx: InvokeContext): Promise<InvokeResult> {
  // ctx.source: "cli" | "api" | "peer"
  // ctx.args: string[] (cli) or object (api/peer)
  return { ok: true, output: "pong" };
}
```

One function. Three surfaces. Manifest declares the routing.

## Surfaces

| Manifest field | Runtime wires | Invocation |
|----------------|---------------|------------|
| `cli` | command-registry | `maw <command>` |
| `api` | Elysia auto-mount | `GET/POST /api/<path>` |
| `hooks` | PluginSystem | GATE → FILTER → HANDLE → LATE |
| `transport` | peer routing | `maw hey plugin:<name>` |
| `cron` | scheduler | periodic execution |
| `module` | dependency resolver | cross-plugin imports |

## Weight

```
 0 — core (wake, sleep, hey, ls, serve)
10 — infrastructure (fleet, federation, pulse)
20 — tools (ping, health, costs, transport)
50 — features (artifacts, contacts, bud, oracle)
90 — customizations (dashboard, morning, quick)
```

## SDK

TS plugins: `import { loadConfig, createArtifact } from "maw/sdk"`
WASM plugins: host functions (`maw_identity`, `maw_send`, etc.)

## License

MIT
