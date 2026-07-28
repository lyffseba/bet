#!/usr/bin/env bash
# End-to-end multiplayer smoke: host alice wins vs bob, stakes settle.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BET="${BET_BIN:-$ROOT/target/release/bet}"
if [[ ! -x "$BET" ]]; then
  echo "building bet…"
  cargo build -p bet-cli --release
fi

CFG="$(mktemp -d "${TMPDIR:-/tmp}/bet-e2e.XXXXXX")"
PORT="${BET_E2E_PORT:-$((18000 + RANDOM % 1000))}"
export BET_CONFIG_DIR="$CFG"

cleanup() { rm -rf "$CFG"; }
trap cleanup EXIT

echo "e2e: config=$CFG port=$PORT"

# Host X plays 0,1,2; guest O plays 3,4 → X wins top row
BET_MOVES=0,1,2 "$BET" host --stake 10 --name alice --port "$PORT" --bind 127.0.0.1 \
  >"$CFG/host.log" 2>&1 &
HPID=$!

CODE=""
for _ in $(seq 1 50); do
  if CODE=$(grep -E 'room:' "$CFG/host.log" 2>/dev/null | awk '{print $2}' | head -1); then
    [[ -n "$CODE" ]] && break
  fi
  sleep 0.1
done
if [[ -z "$CODE" ]]; then
  echo "FAIL: no room code" >&2
  cat "$CFG/host.log" >&2 || true
  kill "$HPID" 2>/dev/null || true
  exit 1
fi
echo "e2e: room=$CODE"

BET_MOVES=3,4 "$BET" join "$CODE" --stake 10 --name bob --addr "127.0.0.1:$PORT" \
  >"$CFG/guest.log" 2>&1
wait "$HPID"
HE=$?

echo "=== host log (tail) ==="
tail -20 "$CFG/host.log"
echo "=== guest log (tail) ==="
tail -20 "$CFG/guest.log"

if [[ "$HE" -ne 0 ]]; then
  echo "FAIL: host exit $HE" >&2
  exit 1
fi

BAL="$("$BET" balance)"
echo "=== balance ==="
echo "$BAL"

echo "$BAL" | grep -q 'alice: 1010' || { echo "FAIL: expected alice: 1010" >&2; exit 1; }
echo "$BAL" | grep -q 'bob: 990' || { echo "FAIL: expected bob: 990" >&2; exit 1; }
grep -q 'MATCH ENDED' "$CFG/host.log" || { echo "FAIL: no MATCH ENDED" >&2; exit 1; }

echo "e2e-mp: OK"
