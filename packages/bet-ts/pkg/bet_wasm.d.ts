/* tslint:disable */
/* eslint-disable */

export class WasmHangman {
    free(): void;
    [Symbol.dispose](): void;
    attempts_left(): number;
    display_word(): string;
    /**
     * Guess a letter. Returns true if the letter is in the word.
     */
    guess(ch: string): boolean;
    guessed(): string;
    is_lost(): boolean;
    is_over(): boolean;
    is_won(): boolean;
    max_attempts(): number;
    /**
     * `words_json`: JSON array of strings, e.g. `["ALPHA","BRAVO"]`.
     */
    constructor(seed: bigint, words_json: string, max_attempts: number);
    state_hash(): string;
    /**
     * Secret word (use only after game over for UI).
     */
    word(): string;
}

export class WasmLedger {
    free(): void;
    [Symbol.dispose](): void;
    balance(id: string): number;
    /**
     * JSON object map of balances.
     */
    balances_json(): string;
    ensure_player(id: string): void;
    constructor(default_grant: number);
    open_pot(match_id: string): number;
    settle(match_id: string, winner?: string | null): number;
    stake(match_id: string, player: string, amount: number): void;
}

export class WasmTtt {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 9-char board: `.` empty, `X`, `O`.
     */
    board(): string;
    /**
     * `x` | `o`
     */
    current(): string;
    /**
     * Human (X) places at index 0..8; AI (O) replies if ongoing.
     * Returns false if the move was illegal.
     */
    make_move_vs_ai(index: number): boolean;
    constructor(seed: bigint);
    /**
     * Pure multiplayer place: "x" | "o" at index.
     */
    place(player: string, index: number): boolean;
    reset(): void;
    state_hash(): string;
    /**
     * `ongoing` | `win_x` | `win_o` | `draw`
     */
    status(): string;
}

export function engine_version(): string;

/**
 * Deterministic hangman hash path for cross-language goldens.
 */
export function golden_hangman_hash(seed: bigint, words_json: string): string;

export function golden_ttt_hash(seed: bigint): string;

export function wasm_start(): void;
