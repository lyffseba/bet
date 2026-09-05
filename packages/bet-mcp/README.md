# @lyffseba/bet-mcp

MCP stdio server so BET is playable from **Claude Code**, **Codex**, and any other MCP host.

| Tool | Engine | What it does |
|------|--------|----------------|
| `bet_status` | WASM | Engine version + `assertEngineIntegrity` via `@lyffseba/bet-ts`. |
| `bet_play` | WASM | Play **ttt** (you are X vs seeded WASM AI) or **hangman** in-process. |
| `bet_host` | CLI bridge | Spawn native `bet host`. Returns `BET_READY` + `CODE@host:port`. |
| `bet_join` | CLI bridge | Spawn native `bet join CODE\|CODE@host:port`. Returns CLI outcome. |

Rules come from **Rust**:

- Solo play / status: **`@lyffseba/bet-ts` (WASM)** via the shared facade **`@lyffseba/bet-ts/play`**.
- Multiplayer: the **`bet` CLI** (`bet-protocol` room authority). This package does **not** reimplement hangman/ttt or the TCP NDJSON protocol.

Stakes are **virtual points only**.

## `bet` on PATH

`bet_host` / `bet_join` **spawn the native binary**. They fail loud if it is missing.

```bash
# from the bet monorepo
make install          # → ~/.local/bin/bet
export PATH="$HOME/.local/bin:$PATH"
# or
export BET_BIN=/abs/path/to/bet
```

Interactive ratatui cannot run under MCP stdio. The bridge always sets `BET_MOVES` (empty if you omit `moves`) so the CLI never blocks on stdin. Empty / exhausted script resigns on that side's turn.

## Tools

### `bet_play` (WASM, unchanged)

| Arg | Required | Notes |
|-----|----------|--------|
| `game` | yes | `ttt` (aliases: `tictactoe`) or `hangman` |
| `action` | no | `new` \| `move` \| `status`. Default: `move` if `move` is set, else `new` / `status` |
| `move` | for a turn | ttt: empty-cell index `0`–`8`. hangman: one letter `A`–`Z` |
| `seed` | no | Integer seed for a new game (same seed ⇒ same AI / word pick) |

Hangman secret is omitted until the engine reports the game is over.

### `bet_host` (CLI bridge)

Spawns `bet host --game … --stake N --name …` (plus `--port`, `--bind`, `--code`, `--word`, `--seed` when set).

| Arg | Required | Notes |
|-----|----------|--------|
| `game` | no | `ttt` (default) or `hangman` |
| `stake` | no | Virtual points (`--stake`, default 10) |
| `name` | no | `--name` (default `$BET_PLAYER` or `$USER`) |
| `port` | no | `--port` (CLI default 7733). `0` = ephemeral |
| `bind` | no | `--bind` (default `0.0.0.0`) |
| `code` | no | Fixed room code (`--code`, 4–8 alnum) |
| `word` | hangman tests | Pin the secret (`--word`). **Not copied into tool JSON.** |
| `seed` | hangman | `--seed` word-bank pick when `word` is omitted |
| `moves` | no | `BET_MOVES` (ttt: `0,1,2` — hangman: `B,E,T` — `q` resigns) |
| `wait` | no | `ready` (default): return after `BET_READY`, leave host running. `end`: wait until exit |
| `timeout_ms` | no | Cap wait (default 10s / 60s, max 5 min) |
| `config_dir` | no | `BET_CONFIG_DIR` ledger directory |

Default `wait=ready` returns JSON with `room`, `port`, `join` (`CODE@127.0.0.1:port`), and `guest_cmd`. The child stays up so a guest can connect.

### `bet_join` (CLI bridge)

Spawns `bet join CODE|CODE@host:port`.

| Arg | Required | Notes |
|-----|----------|--------|
| `target` | yes* | `CODE` or `CODE@host:port` (from `bet_host` `join`) |
| `code` | yes* | Room code if `target` is omitted |
| `addr` | no | `host:port` (`--addr`) when target is a bare `CODE` |
| `game` | no | Must match the host room (`ttt` or `hangman`) |
| `stake` | no | Must match the host (`--stake`) |
| `name` | no | `--name` |
| `moves` | no | Guest `BET_MOVES` |
| `timeout_ms` | no | Cap wait for match end (default 60s) |
| `config_dir` | no | `BET_CONFIG_DIR` |

\* `target` **or** `code`.

Hangman secret stays off the wire until `MatchEnded` (CLI already does this). The wrapper never adds `word` to JSON from the `--word` arg; `word=` in join/host stdout appears only after `MATCH ENDED`.

## Run locally (from the monorepo)

```bash
# once, from repo root
npm install

# stdio server (waits on stdin; log banner is on stderr)
node --experimental-strip-types packages/bet-mcp/src/index.ts

# or via the bin wrapper
node packages/bet-mcp/bin/bet-mcp.mjs
```

Requires **Node ≥ 22** (`--experimental-strip-types`) and a built WASM payload under `packages/bet-ts/pkg` (`make wasm` if missing).

Inspector (optional):

```bash
npx @modelcontextprotocol/inspector node --experimental-strip-types packages/bet-mcp/src/index.ts
```

## Claude Code

Project `.mcp.json` (or user `~/.claude.json`):

```json
{
  "mcpServers": {
    "bet": {
      "type": "stdio",
      "command": "node",
      "args": [
        "--experimental-strip-types",
        "${CLAUDE_PROJECT_DIR:-.}/packages/bet-mcp/src/index.ts"
      ]
    }
  }
}
```

CLI:

```bash
claude mcp add bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
```

After connect, ask the agent to call `bet_status`, then `bet_play` (WASM) or `bet_host` / `bet_join` (CLI).

## Codex

`~/.codex/config.toml` or project `.codex/config.toml` (trusted repo). The table name is `mcp_servers` (snake_case):

```toml
[mcp_servers.bet]
command = "node"
args = [
  "--experimental-strip-types",
  "/ABS/PATH/to/bet/packages/bet-mcp/src/index.ts",
]
```

CLI (if available in your Codex build):

```bash
codex mcp add bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
```

Then `/mcp` in a Codex session to confirm `bet_status`, `bet_play`, `bet_host`, and `bet_join` are listed.

## Notes

- One in-memory WASM session per game per server process (the host owns process lifetime).
- `bet_host` children stay alive after `wait=ready` so a guest can join; they die with the MCP process.
- OpenCode (`packages/bet-opencode`) imports the same `@lyffseba/bet-ts/play` facade for ttt/hangman (CLI spawn only for games not yet in WASM).
