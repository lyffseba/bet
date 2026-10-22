# @lyffseba/bet-mcp

MCP stdio server for BET. Claude Code, Codex, and any other stdio host get the same four tools. Hangman and tic-tac-toe rules stay in Rust. Stakes are virtual points only.

| Tool | Engine | What it does |
|------|--------|----------------|
| `bet_status` | WASM | Engine version and integrity check. No arguments. |
| `bet_play` | WASM | Tic-tac-toe (you are X vs a seeded AI) or hangman, in-process. |
| `bet_host` | `bet` CLI | Spawn `bet host`. Returns `BET_READY` and `CODE@host:port`. |
| `bet_join` | `bet` CLI | Spawn `bet join`. Returns the CLI outcome. |

`bet_host` and `bet_join` spawn the native `bet` binary. `make install` installs it to `~/.local/bin`; that directory has to be on `PATH`, or set `BET_BIN` to the binary.

`bet_status` and `bet_play` are in-process WASM. They do not need the `bet` binary.

```bash
make install
export PATH="$HOME/.local/bin:$PATH"
export BET_BIN=/ABS/PATH/to/bet
```

Interactive ratatui cannot run under MCP stdio. The bridge always sets `BET_MOVES` (empty when `moves` is omitted) so the CLI never reads stdin. An empty or exhausted script resigns on that side's turn.

Requires Node ≥ 22.

## Tools

### `bet_status`

No arguments. Returns the engine version and the WASM integrity check.

### `bet_play`

| Arg | Required | Type | Notes |
|-----|----------|------|--------|
| `game` | yes | `string` | `ttt` (alias `tictactoe`) or `hangman` |
| `action` | no | `string` | `new`, `move`, or `status`. Default: `move` when `move` is set, otherwise `new` / `status` |
| `move` | no | `string` | ttt: empty-cell index `0`–`8`. hangman: one letter `A`–`Z` |
| `seed` | no | `integer` | Integer seed for a new game. The same seed repeats the AI or the word pick |

The hangman secret is omitted until the engine reports the game is over.

### `bet_host`

Spawns `bet host`. Default `wait` is `ready`: the tool returns after `BET_READY` and leaves the host running.

| Arg | Required | Type | Notes |
|-----|----------|------|--------|
| `game` | no | `string` | `ttt` (default) or `hangman` |
| `stake` | no | `integer` | Virtual points (`--stake`, default 10) |
| `name` | no | `string` | `--name` (default `$BET_PLAYER` or `$USER`) |
| `port` | no | `integer` | `--port` (default 7733). `0` picks an ephemeral port |
| `bind` | no | `string` | `--bind` (default `0.0.0.0`) |
| `code` | no | `string` | Fixed room code (`--code`, 4–8 alnum) |
| `word` | no | `string` | Hangman only: pin the secret (`--word`). Not copied into tool JSON |
| `seed` | no | `integer` | Hangman: `--seed` picks from the word bank when `word` is omitted |
| `moves` | no | `string` | `BET_MOVES` (`0,1,2`, `B,E,T`, or `q`) |
| `wait` | no | `enum ready\|end` | `ready` (default) returns after `BET_READY`. `end` waits until exit |
| `timeout_ms` | no | `integer` | Cap the wait (default 10s for `ready`, 60s for `end`, max 5 min) |
| `config_dir` | no | `string` | `BET_CONFIG_DIR` ledger directory |

The `ready` payload includes `room`, `port`, `join` (`CODE@127.0.0.1:port`), and `guest_cmd`.

### `bet_join`

Spawns `bet join`. Pass `target` or `code`.

| Arg | Required | Type | Notes |
|-----|----------|------|--------|
| `target` | yes* | `string` | `CODE` or `CODE@host:port` (the host tool's `join` field) |
| `code` | yes* | `string` | Room code when `target` is omitted |
| `addr` | no | `string` | `host:port` (`--addr`) when `target` is a bare code |
| `game` | no | `string` | Must match the host room (`ttt` or `hangman`) |
| `stake` | no | `integer` | Must match the host (`--stake`) |
| `name` | no | `string` | `--name` |
| `moves` | no | `string` | Guest `BET_MOVES` |
| `timeout_ms` | no | `integer` | Cap the wait for match end (default 60s, max 5 min) |
| `config_dir` | no | `string` | `BET_CONFIG_DIR` |

\* One of `target` or `code`.

The hangman secret stays off the wire until `MatchEnded`. `word` from the host is not added to tool JSON; `word=` in stdout appears only after `MATCH ENDED`.

## Run locally

```bash
npm install
node --experimental-strip-types packages/bet-mcp/src/index.ts
node packages/bet-mcp/bin/bet-mcp.mjs
```

The TypeScript entry waits on stdin. Its banner goes to stderr. The bin wrapper adds `--experimental-strip-types`. WASM must already be built (`make wasm` from the repo root).

```bash
npx @modelcontextprotocol/inspector node --experimental-strip-types packages/bet-mcp/src/index.ts
```

## Claude Code

Docs: [Connect Claude Code to tools via MCP](https://code.claude.com/docs/en/mcp) and [Connect to MCP servers](https://code.claude.com/docs/en/mcp-quickstart).

| Scope | Flag | File |
|-------|------|------|
| local (default) | | `~/.claude.json`, under this project |
| user | `--scope user` | `~/.claude.json`, top-level `mcpServers` |
| project | `--scope project` | `.mcp.json` in the project root |

Stdio is the default transport. Everything after `--` is the server command. In `.mcp.json`, `${CLAUDE_PROJECT_DIR:-.}` expands to the project root; the `:-.` default is required for that expansion.

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

```bash
claude mcp add bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
claude mcp add --scope user bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
claude mcp list
```

For a server committed with the repo, use the `.mcp.json` block above. `${CLAUDE_PROJECT_DIR:-.}` stays tied to the project root. A project entry stays pending until you approve it in `claude`. Inside a session, `/mcp` manages servers.

## Codex

Docs: [Model Context Protocol](https://learn.chatgpt.com/docs/extend/mcp) and the [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference).

Codex reads `~/.codex/config.toml`. A trusted project may also use `.codex/config.toml`. Each server is a `[mcp_servers.<name>]` table. `command` starts a stdio server; `args` is the argument array.

```toml
[mcp_servers.bet]
command = "node"
args = [
  "--experimental-strip-types",
  "/ABS/PATH/to/bet/packages/bet-mcp/src/index.ts",
]
```

```bash
codex mcp add bet -- node --experimental-strip-types /ABS/PATH/to/bet/packages/bet-mcp/src/index.ts
codex mcp list
```

`codex mcp add <name> -- <command>` writes the user config. In the Codex TUI, `/mcp` lists connected servers.

## Other MCP hosts

Same stdio process. Point `command` at Node and `args` at the wrapper (it adds the type-strip flag):

```json
{
  "mcpServers": {
    "bet": {
      "command": "node",
      "args": ["/ABS/PATH/to/bet/packages/bet-mcp/bin/bet-mcp.mjs"]
    }
  }
}
```

Add `"type": "stdio"` when the host requires a transport field. Claude Code treats a missing `type` as stdio.

## Notes

- One in-memory WASM session per game per server process.
- `bet_host` with `wait=ready` keeps the child alive so a guest can join. The child exits with the MCP process.
- OpenCode (`packages/bet-opencode`) imports the same `@lyffseba/bet-ts/play` facade for tic-tac-toe and hangman.
