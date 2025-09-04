#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CRATE_DIR="$ROOT_DIR/crates/node_sdk"
OUT_DIR="$CRATE_DIR/pkg"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Error: wasm-pack not found. Install via: cargo install wasm-pack" >&2
  exit 1
fi

rm -rf "$OUT_DIR"
pushd "$CRATE_DIR" >/dev/null
echo "[build] wasm-pack (bundler target) -> $OUT_DIR"
wasm-pack build --target bundler --release --out-dir "$OUT_DIR" "$@"
popd >/dev/null

echo "\nDone. Output at crates/node_sdk/pkg (pure ESM bundler target)"
