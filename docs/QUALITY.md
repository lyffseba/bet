# Quality bar — Carmack × Bellard

Every change touching `bet-core` or multiplayer protocol must pass this checklist in the PR.

## Carmack

- [ ] Simulation separated from presentation (no ratatui/crossterm in core)
- [ ] Deterministic: same seed + inputs ⇒ same `state_hash`
- [ ] Realtime games use fixed timestep / integer-friendly math
- [ ] Input is data; no hidden global RNG

## Bellard

- [ ] Small public API; no framework soup in core
- [ ] Minimal deps in `bet-core` (serde optional only)
- [ ] Readable modules; prefer simple structs
- [ ] `cargo test -p bet-core` green
- [ ] No `unsafe` in core (`forbid(unsafe_code)`)

## Betting

- [ ] Virtual points only in v1 (no real money, no zaps)
- [ ] Ledger never mixes into pixel code
- [ ] Overdraw refused; draw refunds stakes
- [ ] Join / stake / settle live in one `Table` (rooms own rules only)
- [ ] Hangman secret never on the wire until `MatchEnded`

## Hosts

- [ ] Pi / OpenCode / MCP do not reimplement hangman/ttt rules (`@lyffseba/bet-ts/play`). MCP `bet_host` / `bet_join` only spawn the native `bet` CLI (Rust protocol owns MP). Other CLI spawn is only for games not yet in WASM.
- [ ] TypeScript 7 typecheck green for packages
- [ ] `protocols/fixtures/wasm_goldens.json` == `ENGINE_GOLDENS` == Rust tests == WASM
- [ ] RNG: full u64 samples + rejection sampling; never `(next_u64() as usize) % n`
- [ ] `make verify` green before merge
