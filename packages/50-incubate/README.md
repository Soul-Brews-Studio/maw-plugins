# incubate

Rust WASM implementation of `maw incubate`.

The plugin preserves the native structural `bud` sub-dispatch through the capability-scoped
`maw.cli.run` ABI, then resolves and sends the incubation trigger through `maw.tmux.*`.
It requests only `cli:run:bud`, `tmux:read`, and `tmux:send`.
