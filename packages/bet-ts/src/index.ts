/**
 * BET TypeScript surface (TypeScript 7).
 *
 * Rules engine lives in Rust (`bet-core`). This package will load WASM once
 * `bet-wasm` ships (Epic 3). Until then, types mirror the pure protocol.
 */

export type PlayerId = string;

export type Cell = "empty" | "x" | "o";
export type Player = "x" | "o";

export type GameStatus =
  | { kind: "ongoing" }
  | { kind: "win"; player: Player }
  | { kind: "draw" };

export interface LedgerSnapshot {
  balances: Record<PlayerId, number>;
  openPots: Record<string, number>;
}

export interface BetEngine {
  /** Placeholder until WASM is wired (E3-S1). */
  readonly version: string;
  /** Fingerprint of last applied state when WASM is available. */
  stateHash?: string;
}

export const BET_TS_VERSION = "2.0.0-alpha.0";

/** Create a stub engine host. Real implementation loads `bet-wasm`. */
export function createEngineStub(): BetEngine {
  return { version: BET_TS_VERSION };
}
