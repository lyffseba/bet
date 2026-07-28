# AGENTS.md — BET monorepo

## Layout

```
crates/bet-core     Pure rules + virtual ledger (no I/O)
crates/bet-cli      ratatui binary `bet`
packages/bet-ts     TS7 types / future WASM loader
packages/bet-pi     pi coding-agent extension (/b$t)
packages/bet-opencode  OpenCode plugin
docs/               EPICS, QUALITY, NOSTR
```

## Commands

```bash
cargo test -p bet-core
cargo build -p bet-cli --release
make install                 # CLI → ~/.local/bin
make install-extension       # pi install monorepo root
npm run typecheck            # TS packages (after npm i)
```

## Rules for agents

1. **Do not reimplement game rules in TypeScript** once WASM exists; until then, keep pi logic temporary and mark TODO(E3).
2. **bet-core stays pure** — no fs/net/thread_rng.
3. **Virtual points only** for stakes.
4. Follow `docs/QUALITY.md` on core changes.
5. Prefer small vertical stories from `docs/EPICS.md`.
