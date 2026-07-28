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
| E0-S5 | CI matrix | next |
| E0-S6 | Docs EPICS/QUALITY/AGENTS | done (alpha) |

## EPIC 1 — Core quality (Carmack × Bellard)

| ID | Story | Status |
|----|-------|--------|
| E1-S1 | Seeded RNG | done in core |
| E1-S2 | Golden fixtures files | next |
| E1-S3 | Pong integer physics in core | todo |
| E1-S4 | Chess pure wrapper | todo |
| E1-S5 | clippy -D on core | todo |
| E1-S6 | PR quality checklist | todo |

## EPIC 2 — Multiplayer + virtual bets

| ID | Story | Status |
|----|-------|--------|
| E2-S1 | `bet-protocol` crate | **done** (TCP NDJSON; WS later if needed) |
| E2-S2 | Ledger persistence | **done** (`BET_CONFIG_DIR`, atomic write, merge) |
| E2-S3 | Loopback MP ttt | **done** (unit tests) |
| E2-S4 | `bet host` / `bet join` | **done** (TCP + `BET_MOVES` scripting) |
| E2-S5 | Stake UX | **done** (`--stake`, settle on end) |
| E2-S6–S8 | Hangman/Pong MP + forfeit | todo (resign works) |
| E2-CI | clippy + e2e-mp.sh in CI | **done** |

## EPIC 3 — WASM for TS hosts

| ID | Story | Status |
|----|-------|--------|
| E3-S1–S5 | bet-wasm, bet-ts loader, pi consumes WASM | todo |

## EPIC 4 — OpenCode live

| ID | Story | Status |
|----|-------|--------|
| E4-S1–S2 | Plugin scaffold + tools | done (alpha stub) |
| E4-S3–S6 | Toast polish, publish, MP args | todo |

## EPIC 5 — Nostr discovery

| ID | Story | Status |
|----|-------|--------|
| E5-S1–S4 | Invite events, not tick path | todo |

## EPIC 6 — Ship v2

| ID | Story | Status |
|----|-------|--------|
| E6-S1–S3 | Polish, release, demo | todo |
