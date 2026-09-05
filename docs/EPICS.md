# BET Epics & Stories

Canonical backlog for the monorepo rebuild. Full cycles: **spec → implement → test → ship**.

## Locked decisions

- Virtual points only (v1)
- Single engine: **Rust + WASM**; TS7 hosts only
- One monorepo (this repo)

## EPIC 0 — Monorepo foundation

| ID | Story | Status |
|----|-------|--------|
| E0-S1 | Cargo workspace + `bet-cli` | done (alpha) |
| E0-S2 | `bet-core` ttt + hangman + ledger | done (alpha) |
| E0-S3 | `packages/*` + pi migrate | done (alpha) |
| E0-S4 | TypeScript 7 toolchain | in progress |
| E0-S5 | CI matrix | **done** (ubuntu+macos stable / Node 22; Windows omitted — bash gate) |
| E0-S6 | Docs EPICS/QUALITY/AGENTS | done (alpha) |

## EPIC 1 — Core quality (Carmack × Bellard)

| ID | Story | Status |
|----|-------|--------|
| E1-S1 | Seeded RNG | done in core |
| E1-S2 | Golden fixtures files | next |
| E1-S3 | Pong integer physics in core | todo |
| E1-S4 | Chess pure wrapper | todo |
| E1-S5 | clippy -D on core | **done** (gate runs `--all-targets` on core+protocol+cli) |
| E1-S6 | PR quality checklist | todo |

## EPIC 2 — Multiplayer + virtual bets

| ID | Story | Status |
|----|-------|--------|
| E2-S1 | `bet-protocol` crate | **done** (TCP NDJSON; WS later if needed) |
| E2-S2 | Ledger persistence | **done** (`BET_CONFIG_DIR`, atomic write, merge) |
| E2-S3 | Loopback MP ttt | **done** (unit tests) |
| E2-S4 | `bet host` / `bet join` | **done** (TCP, `BET_READY`, `CODE@addr`) |
| E2-S5 | Stake UX | **done** (`--stake`, settle on end) |
| E2-S6 | Disconnect forfeit + resign | **done** |
| E2-S7 | Hangman MP (host/join, virtual stakes, word off-wire until end) | **done** |
| E2-S8 | Pong MP | todo |
| E2-CI | clippy + multi-case e2e | **done** |
| E2-UX | Numbered empty cells, proto v1 | **done** |

## EPIC 3 — WASM for TS hosts

| ID | Story | Status |
|----|-------|--------|
| E3-S1 | `bet-wasm` crate (hangman, ttt, ledger) | **done** |
| E3-S2 | `packages/bet-ts` loader + goldens | **done** |
| E3-S3 | Pi hangman/ttt use WASM | **done** |
| E3-S4 | CI wasm + goldens | **done** |
| E3-S5 | Pi multiplayer overlay | todo |
| E3-S6 | MCP stdio host (`packages/bet-mcp`) via bet-ts WASM | **done** (first cut) |
| E3-S7 | Shared WASM play facade (`@lyffseba/bet-ts/play`) for MCP / OpenCode / pi | **done** |

## EPIC 4 — OpenCode live

| ID | Story | Status |
|----|-------|--------|
| E4-S1–S2 | Plugin scaffold + tools | done (alpha stub) |
| E4-S3 | OpenCode ttt/hangman in-process via shared play facade | **done** |
| E4-S4–S6 | Toast polish, publish, MP args | todo |

MCP (E3-S6) and OpenCode (E4-S3) both call `@lyffseba/bet-ts/play`. CLI spawn remains only for games not yet in WASM.

## EPIC 5 — Nostr discovery

| ID | Story | Status |
|----|-------|--------|
| E5-S1–S4 | Invite events, not tick path | todo |

## EPIC 6 — Ship v2

| ID | Story | Status |
|----|-------|--------|
| E6-S1–S3 | Polish, release, demo | todo |
