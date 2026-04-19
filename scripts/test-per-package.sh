#!/usr/bin/env bash
# Isolated per-package test runner — works around bun's mock.module state
# leakage across files within a single bun process.
set -eu

# Packages deferred to maw-js SDK expansion (see maw-js#402 follow-up issue).
# These still import from host paths and will fail standalone until the SDK
# exports cmdBud, cmdOracle*, getTransportRouter + bud helpers.
SKIP="20-transport 50-bud 50-oracle 50-about"

fail=0
for d in packages/*/; do
  name=$(basename "$d")
  if echo " $SKIP " | grep -q " $name "; then
    echo "skip: $name (deferred — needs maw-js SDK expansion)"
    continue
  fi
  if ! ls "$d"*.test.ts >/dev/null 2>&1; then
    echo "skip: $name (no tests)"
    continue
  fi
  echo "── $name ──"
  if ! bun test "$d"; then
    fail=$((fail + 1))
  fi
done
if [ "$fail" -gt 0 ]; then echo "FAILED: $fail package(s)"; exit 1; fi
echo "all packages green"
