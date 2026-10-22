<div align="center">

# BET

**One Rust engine. Virtual points. Play from the CLI, pi, OpenCode, Claude Code, Codex, or any MCP host.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

Hangman and tic-tac-toe rules live in `bet-core`. Agent hosts call `@lyffseba/bet-ts` (WASM) and do not reimplement them. Multiplayer is `bet host` / `bet join` on `bet-protocol` (TCP NDJSON). Stakes are virtual points only.

`bet_host` and `bet_join` spawn the native `bet` binary. `make install` installs it to `~/.local/bin`; that directory has to be on `PATH`, or set `BET_BIN` to the binary.

`bet_status` and `bet_play` are in-process WASM. They do not need the `bet` binary.

`BET_CONFIG_DIR` is the ledger directory. `BET_PLAYER` is the default player id. `BET_MOVES` scripts a side (`0,1,2` or `B,E,T`); an exhausted script resigns, and a set value means stdin is never read.

## CLI

```bash
cargo build -p bet-cli --release
./target/release/bet --help
make install
```

`bet` opens the hub on a TTY. Single-player commands include `hangman` and `tictactoe`.

```bash
bet host --stake 10 --name alice
bet join CODE@127.0.0.1:7733 --stake 10 --name bob
bet host --game hangman --word BET --stake 10 --name alice
bet join CODE@127.0.0.1:7733 --game hangman --stake 10 --name bob
bet balance
```

The host prints `BET_READY` with the room code. Tic-tac-toe cells are `0`–`8`. Hangman is one letter; a hit keeps the turn. `q` resigns. A disconnect forfeits. The hangman secret stays off the wire until the match ends.

## pi

From this checkout:

```bash
make install-extension
```

That builds WASM, installs workspace dependencies, then runs the local-package command from the [pi package docs](https://pi.dev/docs/latest/packages):

```bash
pi install .
```

In pi, `/b$t` opens the game.

## OpenCode

The OpenCode plugin implements the 1.x plugin API (`@opencode-ai/plugin`). The config field is `plugin`. Point it at the entry in this checkout so `@lyffseba/bet-ts` resolves. OpenCode also loads `.opencode/plugins/` and `~/.config/opencode/plugins/`.

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["file:///ABS/PATH/to/bet/packages/bet-opencode/src/index.ts"]
}
```

`bet_status` and `bet_play` run hangman and tic-tac-toe in-process. Host and join are not in this plugin.

## Claude Code

A shared project server belongs in `.mcp.json`, with `${CLAUDE_PROJECT_DIR:-.}` so the path is the project root rather than this machine. Private setup uses `claude mcp add` and `~/.claude.json`. Scopes, the JSON block, and every tool argument: [packages/bet-mcp/README.md](packages/bet-mcp/README.md).

```bash
claude mcp add --scope user bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
claude mcp list
```

## Codex

User config is `~/.codex/config.toml`. A trusted repo may use `.codex/config.toml`. The table name is `mcp_servers`.

```bash
codex mcp add bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
codex mcp list
```

In the Codex TUI, `/mcp` lists the server. The TOML block is in [packages/bet-mcp/README.md](packages/bet-mcp/README.md).

## MCP

Any stdio host can launch `packages/bet-mcp/bin/bet-mcp.mjs` (Node ≥ 22). The tools are `bet_status`, `bet_play`, `bet_host`, and `bet_join`. Setup for that host is in [packages/bet-mcp/README.md](packages/bet-mcp/README.md).

## Layout

```text
crates/bet-core        Rules, virtual ledger, seeded RNG. No I/O.
crates/bet-protocol    Room authority. TCP NDJSON. Join, stake, settle.
crates/bet-cli         bet binary. TUI, host, join.
crates/bet-wasm        WASM build of bet-core.
packages/bet-ts        Loader and @lyffseba/bet-ts/play.
packages/bet-pi        pi extension, command /b$t.
packages/bet-opencode  OpenCode plugin. WASM ttt and hangman.
packages/bet-mcp       MCP stdio. WASM play, CLI host/join.
docs/                  Epics, quality bar, stack pins.
```

## Verify

```bash
make verify
make e2e
make wasm
```

`make verify` is the integrity gate: Rust tests, clippy, WASM goldens, typecheck, multiplayer e2e, and these doc checks.

[docs/EPICS.md](docs/EPICS.md) · [docs/QUALITY.md](docs/QUALITY.md) · [docs/STACK.md](docs/STACK.md)

## License

MIT
