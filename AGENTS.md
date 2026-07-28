# AGENTS.md — BET monorepo

## Layout

```
crates/bet-core        Pure rules + virtual ledger (no I/O)
crates/bet-protocol    Multiplayer room authority
crates/bet-cli         ratatui + host/join binary `bet`
crates/bet-wasm        wasm-bindgen surface over bet-core
packages/bet-ts        TS7 loader + pkg/ (built WASM)
packages/bet-pi        pi extension (/b$t) — hangman/ttt via WASM
packages/bet-opencode  OpenCode plugin
docs/                  EPICS, QUALITY, NOSTR
```

## Commands

```bash
cargo test -p bet-core -p bet-protocol
cargo clippy -p bet-core -p bet-protocol -- -D warnings
cargo build -p bet-cli --release
make e2e                     # multiplayer smoke (BET_MOVES)
make ci                      # full local gate
make install                 # CLI → ~/.local/bin
make install-extension       # pi install monorepo root
npm run typecheck            # TS packages (after npm i)
```

## Multiplayer

- Host authority: `bet host` / `bet join` (TCP NDJSON)
- Ledger: `BET_CONFIG_DIR` or OS config; atomic write + merge on `MatchEnded`
- Automation: `BET_MOVES=0,1,2` (host) and `BET_MOVES=3,4` (guest)

## Rules for agents

1. **Do not reimplement hangman/ttt rules in TypeScript** — use `@lyffseba/bet-ts` / WASM.
2. **bet-core stays pure** — no fs/net/thread_rng.
3. **Virtual points only** for stakes.
4. Follow `docs/QUALITY.md` on core changes.
5. Prefer small vertical stories from `docs/EPICS.md`.
6. **CI must stay green**: core+protocol tests, clippy -D, e2e-mp, wasm goldens.
7. After engine changes: `make wasm && make goldens`.
