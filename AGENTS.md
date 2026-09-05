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
cargo clippy -p bet-core -p bet-protocol --all-targets -- -D warnings
cargo clippy -p bet-cli --all-targets -- -D warnings
cargo build -p bet-cli --release
make e2e                     # multiplayer smoke (BET_MOVES)
make ci                      # full local gate
make install                 # CLI → ~/.local/bin
make install-extension       # pi install monorepo root
npm run typecheck            # TS packages (after npm i)
```

## Multiplayer

- Host authority: `bet host` / `bet join` (TCP NDJSON)
- Games: `--game ttt` (default) or `--game hangman`
- Hangman: `--word WORD` pins the secret (tests); otherwise `--seed` / wall-clock pick
- Secret word is never on the wire until `MatchEnded`
- Join / stake / settle live in `bet-protocol::Table` (rooms own rules only)
- `HostRoom` is the protocol dispatch (`Ttt` | `Hangman`); CLI owns sockets only
- Ledger: `BET_CONFIG_DIR` or OS config; atomic write + merge on `MatchEnded`
- Automation: `BET_MOVES=0,1,2` (ttt) or `BET_MOVES=B,E,T` (hangman).
  If `BET_MOVES` is set (even empty), stdin is never read — exhausted script resigns.

## Rules for agents

1. **Do not reimplement hangman/ttt rules in TypeScript** — use `@lyffseba/bet-ts` / WASM.
2. **bet-core stays pure** — no fs/net/thread_rng.
3. **Virtual points only** for stakes.
4. Follow `docs/QUALITY.md` on core changes.
5. Prefer small vertical stories from `docs/EPICS.md`.
6. **CI is `bash scripts/verify-engine.sh`** (or `make verify`) — must stay green.
7. After engine changes: update `protocols/fixtures/wasm_goldens.json` **and**
   `ENGINE_GOLDENS` in `packages/bet-ts/src/index.ts`, then `make verify`.
8. Pi imports `@lyffseba/bet-ts` only (never reimplement hangman/ttt).
