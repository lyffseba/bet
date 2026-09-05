# bet-opencode

OpenCode plugin for [BET](https://github.com/lyffseba/bet) — play terminal games while the coding agent thinks.

## Install

### From this monorepo (dev)

```json
// opencode.json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["file:./packages/bet-opencode"]
}
```

Or copy `src/index.ts` into `~/.config/opencode/plugins/bet.ts`.

### npm (when published)

```json
{
  "plugin": ["bet-opencode"]
}
```

## Tools

| Tool | Description |
|------|-------------|
| `bet_status` | WASM engine version + integrity via `@lyffseba/bet-ts/play`. Notes virtual points only. |
| `bet_play` | **ttt / hangman:** in-process WASM (`createTtt` / `createHangman`). Does **not** spawn the CLI. Hangman secret is omitted until game over. **menu / pong / chess / matrix:** optional CLI spawn — those games are not yet in WASM. |

`bet_play` WASM args: `game=ttt|hangman`, `action=new|move|status`, `move` (`0`–`8` or `A`–`Z`), optional `seed`.

## Notes

- Shared play facade: `@lyffseba/bet-ts/play` (same module as MCP / pi). No hangman/ttt rules in TypeScript.
- Multiplayer host/join is not exposed here yet.
- Not affiliated with the OpenCode team.
