# @lyffseba/bet-mcp

First-cut [MCP](https://modelcontextprotocol.io) stdio server so BET is playable from **Claude Code**, **Codex**, and any other MCP host.

Rules come from **`@lyffseba/bet-ts` (Rust WASM)**. This package does **not** reimplement hangman/ttt and does **not** spawn the `bet` CLI. Stakes are **virtual points only**.

## Tools

| Tool | What it does |
|------|----------------|
| `bet_status` | Engine version + `assertEngineIntegrity` via `@lyffseba/bet-ts`. Notes virtual points only. Multiplayer host/join is listed as planned. |
| `bet_play` | Play **ttt** (you are X vs seeded WASM AI) or **hangman** in-process. |

`bet_play` args:

| Arg | Required | Notes |
|-----|----------|--------|
| `game` | yes | `ttt` (aliases: `tictactoe`) or `hangman` |
| `action` | no | `new` \| `move` \| `status`. Default: `move` if `move` is set, else `new` / `status` |
| `move` | for a turn | ttt: empty-cell index `0`–`8`. hangman: one letter `A`–`Z` |
| `seed` | no | Integer seed for a new game (same seed ⇒ same AI / word pick) |

Hangman secret is omitted until the engine reports the game is over.

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

After connect, ask the agent to call `bet_status`, then `bet_play` with `game=ttt` / `game=hangman`.

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

Then `/mcp` in a Codex session to confirm `bet_status` and `bet_play` are listed.

## Notes

- One in-memory session per game per server process (the host owns process lifetime).
- Multiplayer `bet host` / `bet join` is **not** wired here yet.
- OpenCode remains a separate CLI-spawn stub (`packages/bet-opencode`); this package is the WASM host for MCP.
