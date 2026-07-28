/**
 * BET TypeScript surface — loads the Rust `bet-wasm` engine (single source of truth).
 *
 * Build WASM: `bash scripts/build-wasm.sh`
 * Verify: `bash scripts/verify-engine.sh`
 */

import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";

const require = createRequire(import.meta.url);

/** Canonical goldens — keep in sync with protocols/fixtures/wasm_goldens.json */
export const ENGINE_GOLDENS = {
  hangman: {
    seed: 99,
    words: ["ALPHA", "BRAVO", "CHARLIE"] as string[],
    word: "CHARLIE",
    hash: "1324f2e7c253daa3",
  },
  ttt: {
    seed: 7,
    hash: "4328a9625e30b2de",
  },
} as const;

type WasmModule = {
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

function loadWasm(): WasmModule {
  try {
    // wasm-pack nodejs target is CommonJS (pkg/package.json type=commonjs).
    return require("../pkg/bet_wasm.js") as WasmModule;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    throw new Error(
      `Failed to load BET WASM engine (packages/bet-ts/pkg). Run: bash scripts/build-wasm.sh\n${msg}`,
    );
  }
}

const wasm = loadWasm();

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

/** Fail fast: WASM must match ENGINE_GOLDENS (same as Rust fixture). */
export function assertEngineIntegrity(): void {
  const { hangman, ttt } = ENGINE_GOLDENS;
  const wordsJson = JSON.stringify(hangman.words);
  const hh = goldenHangmanHash(BigInt(hangman.seed), wordsJson);
  if (hh !== hangman.hash) {
    throw new Error(
      `BET engine integrity failed: hangman golden ${hh} != ${hangman.hash}`,
    );
  }
  const th = goldenTttHash(BigInt(ttt.seed));
  if (th !== ttt.hash) {
    throw new Error(
      `BET engine integrity failed: ttt golden ${th} != ${ttt.hash}`,
    );
  }
  const probe = createHangman([...hangman.words], BigInt(hangman.seed), 6);
  if (probe.word() !== hangman.word) {
    throw new Error(
      `BET engine integrity failed: word ${probe.word()} != ${hangman.word}`,
    );
  }
  probe.free();
}

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
