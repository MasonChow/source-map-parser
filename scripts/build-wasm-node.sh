#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CRATE_DIR="$ROOT_DIR/crates/node_sdk"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Error: wasm-pack not found. Install via: cargo install wasm-pack or official installer" >&2
  exit 1
fi

pushd "$CRATE_DIR" >/dev/null
wasm-pack build --target nodejs --release "$@"
popd >/dev/null

printf "\nDone. Output at crates/node_sdk/pkg\n"
