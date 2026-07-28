#!/usr/bin/env bash
# Multi-scenario multiplayer e2e (win + resign + stake mismatch).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BET="${BET_BIN:-$ROOT/target/release/bet}"
if [[ ! -x "$BET" ]]; then
  echo "building bet…"
  cargo build -p bet-cli --release
fi

pass=0
fail=0

run_case() {
  local name="$1"
  shift
  echo ""
  echo "======== CASE: $name ========"
  if "$@"; then
    echo "PASS: $name"
    pass=$((pass + 1))
  else
    echo "FAIL: $name" >&2
    fail=$((fail + 1))
  fi
}

case_host_wins() {
  local CFG PORT HPID ready BAL HE
  CFG="$(mktemp -d "${TMPDIR:-/tmp}/bet-e2e.XXXXXX")"
  PORT="${BET_E2E_PORT:-$((18000 + RANDOM % 2000))}"
  export BET_CONFIG_DIR="$CFG"

  BET_MOVES=0,1,2 "$BET" host --stake 10 --name alice --port "$PORT" --bind 127.0.0.1 --code WIN001 \
    >"$CFG/host.log" 2>&1 &
  HPID=$!

  ready=""
  for _ in $(seq 1 50); do
    ready=$(grep -E '^BET_READY ' "$CFG/host.log" 2>/dev/null | head -1 || true)
    [[ -n "$ready" ]] && break
    sleep 0.1
  done
  if [[ -z "$ready" ]]; then
    cat "$CFG/host.log" || true
    kill "$HPID" 2>/dev/null || true
    rm -rf "$CFG"
    return 1
  fi
  echo "  $ready"

  BET_MOVES=3,4 "$BET" join "WIN001@127.0.0.1:$PORT" --stake 10 --name bob \
    >"$CFG/guest.log" 2>&1
  wait "$HPID"
  HE=$?
  if [[ "$HE" -ne 0 ]]; then
    tail -30 "$CFG/host.log" || true
    rm -rf "$CFG"
    return 1
  fi

  BAL="$("$BET" balance)"
  echo "$BAL"
  if ! echo "$BAL" | grep -q 'alice: 1010'; then rm -rf "$CFG"; return 1; fi
  if ! echo "$BAL" | grep -q 'bob: 990'; then rm -rf "$CFG"; return 1; fi
  if ! grep -q 'MATCH ENDED' "$CFG/host.log"; then rm -rf "$CFG"; return 1; fi
  rm -rf "$CFG"
  return 0
}

case_guest_resigns() {
  local CFG PORT HPID BAL
  CFG="$(mktemp -d "${TMPDIR:-/tmp}/bet-e2e.XXXXXX")"
  PORT=$((19000 + RANDOM % 1000))
  export BET_CONFIG_DIR="$CFG"

  BET_MOVES=4 "$BET" host --stake 10 --name alice --port "$PORT" --bind 127.0.0.1 --code RES001 \
    >"$CFG/host.log" 2>&1 &
  HPID=$!

  for _ in $(seq 1 50); do
    grep -q '^BET_READY ' "$CFG/host.log" 2>/dev/null && break
    sleep 0.1
  done

  BET_MOVES=q "$BET" join "RES001@127.0.0.1:$PORT" --stake 10 --name bob \
    >"$CFG/guest.log" 2>&1 || true
  wait "$HPID" || true

  BAL="$("$BET" balance)"
  echo "$BAL"
  if ! echo "$BAL" | grep -q 'alice: 1010'; then
    tail -40 "$CFG/host.log" || true
    tail -20 "$CFG/guest.log" || true
    rm -rf "$CFG"
    return 1
  fi
  if ! echo "$BAL" | grep -q 'bob: 990'; then rm -rf "$CFG"; return 1; fi
  rm -rf "$CFG"
  return 0
}

case_stake_mismatch() {
  local CFG PORT HPID GE
  CFG="$(mktemp -d "${TMPDIR:-/tmp}/bet-e2e.XXXXXX")"
  PORT=$((20000 + RANDOM % 1000))
  export BET_CONFIG_DIR="$CFG"

  "$BET" host --stake 10 --name alice --port "$PORT" --bind 127.0.0.1 --code BAD001 \
    >"$CFG/host.log" 2>&1 &
  HPID=$!
  for _ in $(seq 1 50); do
    grep -q '^BET_READY ' "$CFG/host.log" 2>/dev/null && break
    sleep 0.1
  done

  set +e
  "$BET" join "BAD001@127.0.0.1:$PORT" --stake 5 --name bob >"$CFG/guest.log" 2>&1
  GE=$?
  set -e
  kill "$HPID" 2>/dev/null || true
  wait "$HPID" 2>/dev/null || true

  if [[ "$GE" -eq 0 ]]; then
    echo "expected join failure"
    rm -rf "$CFG"
    return 1
  fi
  if ! grep -qiE 'stake mismatch|join failed|Ledger' "$CFG/guest.log" "$CFG/host.log"; then
    cat "$CFG/guest.log" "$CFG/host.log" || true
    rm -rf "$CFG"
    return 1
  fi
  rm -rf "$CFG"
  return 0
}

run_case "host_wins_top_row" case_host_wins
run_case "guest_resigns" case_guest_resigns
run_case "stake_mismatch" case_stake_mismatch

echo ""
echo "======== SUMMARY: $pass passed, $fail failed ========"
[[ "$fail" -eq 0 ]]
echo "e2e-mp: OK"
