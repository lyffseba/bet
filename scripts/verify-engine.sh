#!/usr/bin/env bash
# Single integrity gate: native tests + wasm rebuild + shared goldens + typecheck.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.cargo/bin:${PATH}"

echo "==> cargo test bet-core + bet-protocol"
cargo test -p bet-core -p bet-protocol

echo "==> clippy (native + wasm32)"
cargo clippy -p bet-core -p bet-protocol -- -D warnings
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo clippy -p bet-core -p bet-wasm --target wasm32-unknown-unknown -- -D warnings

echo "==> rebuild wasm"
bash scripts/build-wasm.sh

echo "==> JS goldens (must match protocols/fixtures/wasm_goldens.json)"
node --experimental-strip-types packages/bet-ts/test/goldens.mts

echo "==> Rust goldens example must match fixture"
FIX="$ROOT/protocols/fixtures/wasm_goldens.json"
H_WANT=$(python3 -c "import json;print(json.load(open('$FIX'))['hangman']['hash'])")
T_WANT=$(python3 -c "import json;print(json.load(open('$FIX'))['ttt']['hash'])")
OUT=$(cargo run -q -p bet-core --example goldens 2>/dev/null | tail -20)
echo "$OUT"
echo "$OUT" | grep -q "hashAX=${H_WANT}" || echo "$OUT" | grep -q "${H_WANT}" || {
  # example prints hashAX=...
  echo "$OUT" | grep -F "$H_WANT" >/dev/null || {
    echo "FAIL: rust example missing hangman hash $H_WANT" >&2
    exit 1
  }
}
echo "$OUT" | grep -F "$T_WANT" >/dev/null || {
  echo "FAIL: rust example missing ttt hash $T_WANT" >&2
  exit 1
}

echo "==> npm typecheck"
npm run typecheck

echo "==> multiplayer e2e"
cargo build -q -p bet-cli --release
bash scripts/e2e-mp.sh

echo ""
echo "verify-engine: OK"
