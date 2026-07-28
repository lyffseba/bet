#!/usr/bin/env bash
# Build bet-wasm for Node and place artifacts in packages/bet-ts/pkg
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT="$ROOT/packages/bet-ts/pkg"
mkdir -p "$OUT"

if command -v wasm-pack >/dev/null 2>&1; then
  echo "building with wasm-pack…"
  wasm-pack build crates/bet-wasm \
    --target nodejs \
    --out-dir "$OUT" \
    --out-name bet_wasm \
    --release
else
  echo "wasm-pack missing; trying cargo + wasm-bindgen…"
  rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
  if ! command -v wasm-bindgen >/dev/null 2>&1; then
    cargo install wasm-bindgen-cli --locked
  fi
  cargo build -p bet-wasm --target wasm32-unknown-unknown --release
  wasm-bindgen \
    --target nodejs \
    --out-dir "$OUT" \
    --out-name bet_wasm \
    target/wasm32-unknown-unknown/release/bet_wasm.wasm
fi

# Drop wasm-pack boilerplate; force CJS for Node under ESM monorepo packages.
rm -f "$OUT"/.gitignore "$OUT"/README.md 2>/dev/null || true
cat >"$OUT/package.json" <<'EOF'
{
  "type": "commonjs",
  "name": "bet-wasm-pkg",
  "private": true
}
EOF

echo "WASM ready: $OUT"
ls -la "$OUT"
