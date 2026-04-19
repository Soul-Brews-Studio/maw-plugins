#!/usr/bin/env bash
# Run each package's tests in its own bun process.
# Plugins mock the shared `maw-js/sdk` module; `mock.module` state leaks
# across files in a single bun run, so per-package isolation is required.
set -eu

fail=0
for d in packages/*/; do
  name=$(basename "$d")
  if ! ls "$d"*.test.ts >/dev/null 2>&1; then
    echo "skip: $name (no tests)"
    continue
  fi
  echo "── $name ──"
  if ! bun test "$d"; then
    fail=$((fail + 1))
  fi
done

if [ "$fail" -gt 0 ]; then
  echo "FAILED: $fail package(s)"
  exit 1
fi
echo "all packages green"
