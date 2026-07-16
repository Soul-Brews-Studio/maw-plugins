# registry.json — the default mawx resolution root

`registry.json` (repo root) is the index mawx uses to resolve a **bare verb**
(`maw x costs`) to an exact, pinned wasm artifact. It is generated from
`packages/*` by `scripts/gen-registry.ts` and enforced fresh by CI. This
implements **WI-9** of the mawx V1 spec
(`maw-rs:ψ/design/mawx-spec.md`, §2.2 resolution, §4 WI-9).

## Why this file carries real security weight

Per Nat's decision 2 (spec §6), `Soul-Brews-Studio/maw-plugins` is an
**auto-trust root**: a client resolving a pinned entry from *this* registry runs
it with **no first-run trust prompt**. The `sha256` in each entry is therefore
the trust anchor the client verifies a freshly fetched `plugin.wasm` against — a
wrong or missing pin here is a security hole, not a cosmetic bug. The generator
never invents a pin, and CI fails on any drift.

## Schema

A flat JSON object keyed by the plugin's `cli.command` verb. Each entry:

```json
{
  "costs": {
    "commit": "4c34f2a99e0cc4c1dd8ccabf1a0dff25a3fde69f",
    "sha256": "sha256:d6fecca891f8cbb4208dcc2469be763e41013957044041adfc15423f753f553f",
    "path": "packages/20-costs",
    "version": "1.0.0",
    "capabilities": ["fs:read:claude-projects"]
  }
}
```

| Field | Source | Meaning |
|---|---|---|
| *key* (verb) | `plugin.json` → `cli.command` | what `maw x <verb>` resolves |
| `commit` | `git log -1 -- packages/<dir>` | the commit that anchors the immutable raw URL (see below) |
| `sha256` | `plugin.json` → `artifact.sha256` (fallback `plugin.source.json`) | pin the fetched `plugin.wasm` is verified against — same value the CI pin-integrity gate proves against the committed bytes |
| `path` | package directory | `packages/<dir>` |
| `version` | `plugin.json` → `version` | display / provenance |
| `capabilities` | `plugin.json` → `capabilities` (manifest order) | fs/tmux/net/... grants, surfaced in the client trust card |

The registry's owner/repo are **not** stored per entry — they are implicit in
the URL the client fetched `registry.json` from
(`raw.githubusercontent.com/<owner>/<repo>/HEAD/registry.json`).

### `commit` is the package's last commit, not repo HEAD

Each entry pins the commit that **last touched its package directory**, so
`raw.githubusercontent.com/<owner>/<repo>/<commit>/packages/<dir>/plugin.wasm`
is immutable and hashes to `sha256`. Using repo HEAD instead would make
`registry.json` reference a commit that does not yet exist at generation time,
so the CI staleness gate could never be green. Per-package-last-commit is also
stable across unrelated commits, so touching one package does not churn every
other entry.

## Which packages are indexed

Only **wasm ship-tier** packages: those with `target: "wasm"`, a `cli.command`,
a committed `plugin.wasm`, and a resolvable `artifact.sha256`. All 21 current
wasm packages qualify.

**JS-target packages are deliberately excluded** (currently `share`,
`p2p-share`, `maw-menubar`). They have no `plugin.wasm` and no wasm pin, so they
are not mawx-resolvable and cannot be a valid auto-trust wasm entry — emitting
one with a null hash would create an unpinnable entry in an auto-trust root. The
generator reports each skip with its reason.

## Regenerating

```bash
bun run scripts/gen-registry.ts          # or: bun run registry
```

Deterministic: two runs on the same checkout produce byte-identical output.

## Publishing a wasm change (two commits)

Because `commit` must reference bytes that are already committed, update a
package's wasm in **two steps**:

1. Rebuild + re-pin the package (`plugin.wasm` + `plugin.json` `artifact.sha256`)
   and **commit** it. The pin-integrity gate proves the committed bytes match.
2. Run `bun run scripts/gen-registry.ts` and **commit** the updated
   `registry.json`. Its entry now anchors to the commit from step 1.

Bundling the wasm change and the registry regeneration into a single commit will
fail the freshness gate: at generation time the new bytes are not yet committed,
so the anchor cannot point at them.

## CI enforcement

The `registry-freshness` job (`.github/workflows/ci.yml`) checks out with full
history (`fetch-depth: 0`) and runs `bun run scripts/gen-registry.ts --check`,
which regenerates in memory and fails if the committed `registry.json` differs.
Combined with the `build-and-verify` pin-integrity gate
(`registry.sha256 == manifest pin == committed plugin.wasm`), the registry can
never silently drift from the artifacts it pins.
