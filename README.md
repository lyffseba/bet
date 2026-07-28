<div align="center">

# B$T (BET) — Terminal multiplayer bets + agent game hub

**Rust-core games. Virtual-point multiplayer stakes. Play in CLI, pi, or OpenCode while the LLM thinks.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![TypeScript 7](https://img.shields.io/badge/TypeScript-7-blue.svg)](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)

</div>

## Vision (v2 monorepo)

| Layer | Role |
|-------|------|
| **`bet-core` (Rust)** | Single rules engine + virtual ledger (Carmack/Bellard quality bar) |
| **`bet` CLI** | Full ratatui hub; upcoming `host` / `join` multiplayer |
| **pi extension** | `/b$t` overlay while coding |
| **OpenCode plugin** | `bet_play` / `bet_status` tools + idle toast |
| **WASM + TS7** | Same engine in agent hosts (Epic 3) |
| **Nostr** | Room discovery only (Epic 5) — not the tick path |

**Betting is virtual points only** in v1 (no real money).

See [docs/EPICS.md](docs/EPICS.md) and [docs/QUALITY.md](docs/QUALITY.md).

## Quick start

```bash
# CLI
cargo build -p bet-cli --release
./target/release/bet
# or
make install   # → ~/.local/bin/bet

# Tests (pure core)
cargo test -p bet-core

# pi extension (from monorepo root)
pi install .

# OpenCode (dev)
# opencode.json → { "plugin": ["file:./packages/bet-opencode"] }
```

## Monorepo layout

```
crates/bet-core        Pure hangman, tictactoe, ledger, seeded RNG
crates/bet-cli         Terminal UI binary
packages/bet-ts        TypeScript 7 types / WASM loader stub
packages/bet-pi        pi package (bet-pi-hub)
packages/bet-opencode  OpenCode plugin
docs/                  Epics, quality bar, Nostr notes
```

## Agent commands

| Host | How |
|------|-----|
| **pi** | `pi install .` then `/b$t` |
| **OpenCode** | plugin `bet-opencode` → tools `bet_status`, `bet_play` |
| **CLI** | `bet`, `bet hangman`, `bet tictactoe`, … |

## Multiplayer (virtual points)

```bash
# Terminal A
bet host --stake 10 --name alice
# prints: BET_READY room=XXXXXX port=7733 …

# Terminal B
bet join XXXXXX@127.0.0.1:7733 --stake 10 --name bob

bet balance
```

Empty cells show their index (`0`–`8`). Type the index to place; `q` resigns.  
Disconnect mid-match counts as a forfeit. Stakes are **virtual points only** (grant 1000).

| Env / flag | Purpose |
|------------|---------|
| `BET_CONFIG_DIR` | Ledger directory |
| `BET_PLAYER` | Default player id |
| `BET_MOVES` | Scripted moves (`0,3,1,4,2` or `q`) |
| `--code CODE` | Fixed room code (tests) |
| `CODE@host:port` | Join target shorthand |

```bash
make e2e   # win + resign + stake-mismatch smoke
make ci    # unit + clippy + e2e
```

## Status

- **Alpha** (`2.0.0-alpha.0`): monorepo + **tic-tac-toe multiplayer host/join with stakes**.
- **Next:** hangman/pong MP (E2-S6+), WASM for pi (E3), Nostr discovery (E5).

## License

MIT
