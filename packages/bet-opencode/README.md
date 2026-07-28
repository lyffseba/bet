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
| `bet_status` | CLI path + engine notes |
| `bet_play` | Spawn native `bet` binary (`menu`, `hangman`, …) |

## Notes

- OpenCode plugins are event/tool based; full TUI is the Rust `bet` CLI.
- Multiplayer + virtual stakes land in Epic 2; WASM shared engine in Epic 3.
- Not affiliated with the OpenCode team.
