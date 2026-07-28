/**
 * BET TypeScript surface — loads the Rust `bet-wasm` engine (single source of truth).
 *
 * Build WASM: `bash scripts/build-wasm.sh`
 */

import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";

const require = createRequire(import.meta.url);

// wasm-pack nodejs target emits CommonJS (see packages/bet-ts/pkg/package.json type=commonjs).
// eslint-disable-next-line @typescript-eslint/no-require-imports
const wasm = require("../pkg/bet_wasm.js") as {
  engine_version: () => string;
  WasmHangman: new (
    seed: bigint,
    words_json: string,
    max_attempts: number,
  ) => HangmanHandle;
  WasmTtt: new (seed: bigint) => TttHandle;
  WasmLedger: new (default_grant: number) => LedgerHandle;
  golden_hangman_hash: (seed: bigint, wordsJson: string) => string;
  golden_ttt_hash: (seed: bigint) => string;
};

export const BET_TS_VERSION = "2.0.0-alpha.0";

export interface HangmanHandle {
  free(): void;
  guess(ch: string): boolean;
  display_word(): string;
  attempts_left(): number;
  max_attempts(): number;
  is_won(): boolean;
  is_lost(): boolean;
  is_over(): boolean;
  word(): string;
  guessed(): string;
  state_hash(): string;
}

export interface TttHandle {
  free(): void;
  make_move_vs_ai(index: number): boolean;
  place(player: string, index: number): boolean;
  reset(): void;
  board(): string;
  status(): string;
  current(): string;
  state_hash(): string;
}

export interface LedgerHandle {
  free(): void;
  ensure_player(id: string): void;
  balance(id: string): number;
  stake(match_id: string, player: string, amount: number): void;
  settle(match_id: string, winner?: string | null): number;
  open_pot(match_id: string): number;
  balances_json(): string;
}

export type WasmHangman = HangmanHandle;
export type WasmTtt = TttHandle;
export type WasmLedger = LedgerHandle;

export const engineVersion = (): string => wasm.engine_version();
export const goldenHangmanHash = (seed: bigint, wordsJson: string): string =>
  wasm.golden_hangman_hash(seed, wordsJson);
export const goldenTttHash = (seed: bigint): string => wasm.golden_ttt_hash(seed);

export function toSeed(n: number | bigint): bigint {
  return typeof n === "bigint" ? n : BigInt(n >>> 0);
}

export function timeSeed(): bigint {
  return BigInt(Date.now());
}

export function pkgDir(): string {
  return path.dirname(fileURLToPath(import.meta.url));
}

export function isWasmLoaded(): boolean {
  return typeof wasm.engine_version === "function";
}

export function createHangman(
  words: string[],
  seed: bigint = timeSeed(),
  maxAttempts = 6,
): HangmanHandle {
  return new wasm.WasmHangman(seed, JSON.stringify(words), maxAttempts);
}

export function createTtt(seed: bigint = timeSeed()): TttHandle {
  return new wasm.WasmTtt(seed);
}

export function createLedger(defaultGrant = 1000): LedgerHandle {
  return new wasm.WasmLedger(defaultGrant);
}

/** Board helpers for pi UI (3x3 from 9-char engine string). */
export function boardToGrid(board: string): (string | null)[][] {
  const g: (string | null)[][] = [
    [null, null, null],
    [null, null, null],
    [null, null, null],
  ];
  for (let i = 0; i < 9; i++) {
    const ch = board[i] ?? ".";
    const r = Math.floor(i / 3);
    const c = i % 3;
    g[r]![c] = ch === "." ? null : ch;
  }
  return g;
}

export function cellIndex(row: number, col: number): number {
  return row * 3 + col;
}
