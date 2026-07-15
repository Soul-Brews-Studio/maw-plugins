# maw-plugins

> Extracted [maw](https://github.com/Soul-Brews-Studio/maw-rs) verb plugins — ship-tier WASM
> artifacts with sha256 pins, one package per verb under `packages/NN-verb/`.

These are the verbs moved out of the maw core (lean-core extraction) into
installable plugins. Each package ships a **committed `plugin.wasm`** built for
`wasm32-unknown-unknown`, pinned by sha256 in its manifest.

## Layout

```
packages/
├── 20-costs/            ← weight 20: tools
│   ├── Cargo.toml       # cdylib crate, extism-pdk = "1.4"
│   ├── src/lib.rs       # plugin implementation
│   ├── plugin.wasm      # committed ship artifact
│   └── plugin.json      # manifest — artifact.sha256 pin, cli surface, capabilities
├── 20-mega/
├── 50-incubate/         ← weight 50: features
└── ...
```

Number prefix = weight = execution order (lower fires first).

## Manifest (`plugin.json`)

```json
{
  "name": "costs",
  "schemaVersion": 1,
  "target": "wasm",
  "entry": { "kind": "wasm", "path": "plugin.wasm", "export": "handle" },
  "artifact": {
    "path": "./plugin.wasm",
    "sha256": "sha256:<hex digest of the committed plugin.wasm>"
  },
  "capabilities": ["fs:read:claude-projects"],
  "cli": { "command": "costs", "help": "maw costs [--json]" },
  "weight": 20
}
```

The `artifact.sha256` pin must always equal the sha256 of the committed
`plugin.wasm` — CI enforces this.

## Authoring a plugin

The pattern is **direct `extism-pdk`** — the old `maw-plugin-pdk` wrapper crate
is deleted and is NOT the pattern.

```toml
[package]
name = "maw_<verb>_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Build:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/maw_<verb>_plugin.wasm plugin.wasm
shasum -a 256 plugin.wasm   # update artifact.sha256 in plugin.json
```

Host functions (`maw_identity`, `maw_send`, …) are provided by the maw-rs
WASM host at runtime.

## Install

```bash
maw plugin install Soul-Brews-Studio/maw-plugins/packages/NN-verb --sha256 <pin>
```

> **Known issue**: the wasm git-install route is currently blocked by
> [maw-rs#521](https://github.com/Soul-Brews-Studio/maw-rs/issues/521) — until
> that lands, install from a local clone instead:
>
> ```bash
> git clone https://github.com/Soul-Brews-Studio/maw-plugins
> maw plugin install --path maw-plugins/packages/NN-verb
> ```

## CI

Every push/PR builds each `packages/*/` crate for `wasm32-unknown-unknown`,
runs its tests where present, and verifies **pin integrity**: the sha256 of
the committed `plugin.wasm` must equal the manifest's `artifact.sha256`.
(Rebuilt wasm is NOT compared against the pin — build determinism is
unproven.)

## License

[BUSL-1.1](./LICENSE) — Business Source License 1.1, converting to Apache-2.0
on the Change Date. See the LICENSE file for the Additional Use Grant.
