#!/usr/bin/env bash
# Build the Node package and run every Node example.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

wasm-pack build --target nodejs --quiet
for script in examples/node/*.mjs; do
  echo "=== $script ==="
  node "$script"
done
