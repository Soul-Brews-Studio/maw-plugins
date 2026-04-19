#!/usr/bin/env bash
# Create ./node_modules/maw-js so plugins in packages/*/ resolve
# `import ... from "maw-js/sdk"` when installed via `maw plugin install --link`.
#
# Without this link, bun walks up from each plugin's real path looking for
# node_modules/maw-js and finds nothing → plugin load fails with
# "Cannot find module 'maw-js/sdk'". Root cause of Soul-Brews-Studio/maw-js#402
# after the plugins-fixer#1 fix landed.
#
# Resolution order (first hit wins):
#   1. $MAW_JS_PATH env var
#   2. Sibling checkout: ../maw-js (same parent dir as maw-plugins)
#   3. Bun global install: ~/.bun/install/global/node_modules/maw-js
#   4. `bun pm ls --global` reports maw-js → use its resolved path
#
# Re-running is idempotent.

set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
nm_dir="$repo_root/node_modules"
link_dest="$nm_dir/maw-js"

find_mawjs() {
  if [ -n "${MAW_JS_PATH:-}" ] && [ -d "$MAW_JS_PATH" ]; then
    echo "$MAW_JS_PATH"; return 0
  fi
  local sibling
  sibling="$(cd "$repo_root/.." && pwd)/maw-js"
  if [ -d "$sibling" ] && [ -f "$sibling/package.json" ]; then
    echo "$sibling"; return 0
  fi
  local global="${BUN_INSTALL:-$HOME/.bun}/install/global/node_modules/maw-js"
  if [ -d "$global" ] && [ -f "$global/package.json" ]; then
    echo "$global"; return 0
  fi
  return 1
}

src="$(find_mawjs || true)"
if [ -z "$src" ]; then
  echo "error: could not locate maw-js" >&2
  echo "  tried: \$MAW_JS_PATH, ../maw-js, \$BUN_INSTALL/install/global/node_modules/maw-js" >&2
  echo "  fix: clone maw-js next to this repo, or run \`bun add --global maw-js\`, or set MAW_JS_PATH" >&2
  exit 1
fi

mkdir -p "$nm_dir"
ln -sfn "$src" "$link_dest"
echo "linked: $link_dest -> $src"
