/**
 * In-process BET play — rules come only from @lyffseba/bet-ts (Rust WASM).
 * This module formats engine state for agents; it does not decide wins/losses.
 */
import {
  BET_TS_VERSION,
  assertEngineIntegrity,
  boardToGrid,
  createHangman,
  createTtt,
  engineVersion,
  isWasmLoaded,
  timeSeed,
  toSeed,
  type HangmanHandle,
  type TttHandle,
} from "@lyffseba/bet-ts";

export const BET_MCP_VERSION = "2.0.0-alpha.0";

/** Same NATO bank as `bet-protocol` when the CLI does not pin a word. */
export const DEFAULT_HANGMAN_WORDS = [
  "ALPHA",
  "BRAVO",
  "CHARLIE",
  "DELTA",
  "ECHO",
  "FOXTROT",
  "GOLF",
  "HOTEL",
];

export type GameKind = "ttt" | "hangman";
export type PlayAction = "new" | "move" | "status";

export type PlayArgs = {
  game: string;
  action?: string;
  move?: string;
  seed?: number;
};

type Integrity = { ok: true } | { ok: false; error: string };

function checkIntegrity(): Integrity {
  try {
    assertEngineIntegrity();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

const integrity = checkIntegrity();

function freeEngine(engine: { free(): void } | null): void {
  if (!engine) return;
  try {
    engine.free();
  } catch {
    // already freed
  }
}

export function normalizeGame(raw: string): GameKind | null {
  const g = raw.trim().toLowerCase();
  if (g === "ttt" || g === "tictactoe" || g === "tic-tac-toe" || g === "tic_tac_toe") {
    return "ttt";
  }
  if (g === "hangman") return "hangman";
  return null;
}

export function normalizeAction(
  raw: string | undefined,
  hasMove: boolean,
  hasSession: boolean,
): PlayAction {
  const a = (raw ?? "").trim().toLowerCase();
  if (a === "new" || a === "reset" || a === "start") return "new";
  if (a === "move" || a === "play" || a === "guess") return "move";
  if (a === "status" || a === "state") return "status";
  if (hasMove) return "move";
  return hasSession ? "status" : "new";
}

function parseTttIndex(move: string): number | null {
  const t = move.trim();
  if (!/^[0-8]$/.test(t)) return null;
  return Number(t);
}

function parseHangmanLetter(move: string): string | null {
  const t = move.trim();
  if (!/^[a-zA-Z]$/.test(t)) return null;
  return t.toUpperCase();
}

function formatTttBoard(board: string): string {
  const cells = board.padEnd(9, ".").slice(0, 9).split("");
  const cell = (i: number): string => {
    const ch = cells[i] ?? ".";
    return ch === "." ? String(i) : ch;
  };
  const row = (i: number): string => ` ${cell(i)} | ${cell(i + 1)} | ${cell(i + 2)}`;
  return [row(0), "---+---+---", row(3), "---+---+---", row(6)].join("\n");
}

function emptyTttCells(board: string): number[] {
  const out: number[] = [];
  const b = board.padEnd(9, ".").slice(0, 9);
  for (let i = 0; i < 9; i++) {
    if ((b[i] ?? ".") === ".") out.push(i);
  }
  return out;
}

export function engineStatus(): Record<string, unknown> {
  return {
    package: "@lyffseba/bet-mcp",
    version: BET_MCP_VERSION,
    bet_ts: BET_TS_VERSION,
    engine: {
      version: engineVersion(),
      loader: "@lyffseba/bet-ts",
      wasm_loaded: isWasmLoaded(),
      integrity: integrity.ok ? "ok" : "failed",
      integrity_error: integrity.ok ? undefined : integrity.error,
    },
    play: "in-process WASM (does not spawn the bet CLI)",
    stakes: "virtual points only (no real money, no zaps)",
    games: ["ttt", "hangman"],
    transport: "stdio",
    planned: {
      multiplayer: "host/join not exposed in MCP yet",
      opencode: "separate CLI-spawn stub; not expanded here",
    },
  };
}

export class BetTable {
  private ttt: TttHandle | null = null;
  private hangman: HangmanHandle | null = null;
  private tttSeed: string | null = null;
  private hangmanSeed: string | null = null;

  has(game: GameKind): boolean {
    return game === "ttt" ? this.ttt !== null : this.hangman !== null;
  }

  snapshotTtt(): Record<string, unknown> | null {
    if (!this.ttt) return null;
    const board = this.ttt.board();
    const status = this.ttt.status();
    return {
      game: "ttt",
      engine: "wasm",
      seed: this.tttSeed,
      board,
      grid: boardToGrid(board),
      board_text: formatTttBoard(board),
      status,
      current: this.ttt.current(),
      empty_cells: emptyTttCells(board),
      state_hash: this.ttt.state_hash(),
      you: "X (human)",
      opponent: "O (seeded WASM AI)",
      hint:
        status === "ongoing"
          ? "Call bet_play with move=0..8 (numbered empty cells)."
          : "Game over. Call bet_play action=new to start again.",
    };
  }

  snapshotHangman(): Record<string, unknown> | null {
    if (!this.hangman) return null;
    const over = this.hangman.is_over();
    return {
      game: "hangman",
      engine: "wasm",
      seed: this.hangmanSeed,
      display: this.hangman.display_word(),
      guessed: this.hangman.guessed(),
      attempts_left: this.hangman.attempts_left(),
      max_attempts: this.hangman.max_attempts(),
      won: this.hangman.is_won(),
      lost: this.hangman.is_lost(),
      over,
      word: over ? this.hangman.word() : null,
      state_hash: this.hangman.state_hash(),
      hint: over
        ? "Game over. Call bet_play action=new to start again."
        : "Call bet_play with move=A..Z to guess a letter.",
    };
  }

  play(args: PlayArgs): { ok: boolean; payload: Record<string, unknown> } {
    if (!integrity.ok) {
      return {
        ok: false,
        payload: {
          error: "engine_integrity_failed",
          message: integrity.error,
        },
      };
    }

    const game = normalizeGame(args.game);
    if (!game) {
      return {
        ok: false,
        payload: {
          error: "unknown_game",
          message: `Unknown game "${args.game}". Use ttt or hangman.`,
        },
      };
    }

    const action = normalizeAction(args.action, Boolean(args.move?.trim()), this.has(game));

    if (action === "new" || (action === "move" && !this.has(game))) {
      const started = this.start(game, args.seed);
      if (action === "new" && !args.move?.trim()) {
        return { ok: true, payload: { accepted: true, action: "new", ...started } };
      }
    }

    if (action === "status") {
      const snap = game === "ttt" ? this.snapshotTtt() : this.snapshotHangman();
      if (!snap) {
        return {
          ok: false,
          payload: {
            error: "no_session",
            message: `No ${game} session. Call bet_play with action=new.`,
          },
        };
      }
      return { ok: true, payload: { accepted: true, action: "status", ...snap } };
    }

    const move = args.move?.trim() ?? "";
    if (!move) {
      return {
        ok: false,
        payload: {
          error: "missing_move",
          message:
            game === "ttt"
              ? "Provide move as an empty-cell index 0..8."
              : "Provide move as a single letter A-Z.",
        },
      };
    }

    if (game === "ttt") {
      return this.moveTtt(move);
    }
    return this.moveHangman(move);
  }

  private start(game: GameKind, seedArg?: number): Record<string, unknown> {
    const seed = seedArg === undefined ? timeSeed() : toSeed(seedArg);
    const seedStr = seed.toString();
    if (game === "ttt") {
      freeEngine(this.ttt);
      this.ttt = createTtt(seed);
      this.tttSeed = seedStr;
      return this.snapshotTtt() as Record<string, unknown>;
    }
    freeEngine(this.hangman);
    this.hangman = createHangman(DEFAULT_HANGMAN_WORDS, seed, 6);
    this.hangmanSeed = seedStr;
    return this.snapshotHangman() as Record<string, unknown>;
  }

  private moveTtt(move: string): { ok: boolean; payload: Record<string, unknown> } {
    const idx = parseTttIndex(move);
    if (idx === null) {
      return {
        ok: false,
        payload: {
          error: "bad_move",
          message: `Tic-tac-toe move must be a single index 0..8 (got "${move}").`,
        },
      };
    }
    if (!this.ttt) {
      return { ok: false, payload: { error: "no_session", message: "No ttt session." } };
    }
    const accepted = this.ttt.make_move_vs_ai(idx);
    return {
      ok: accepted,
      payload: {
        accepted,
        action: "move",
        move: idx,
        ...(accepted
          ? {}
          : { error: "illegal_move", message: "Engine rejected that cell (occupied or game over)." }),
        ...(this.snapshotTtt() ?? {}),
      },
    };
  }

  private moveHangman(move: string): { ok: boolean; payload: Record<string, unknown> } {
    const letter = parseHangmanLetter(move);
    if (letter === null) {
      return {
        ok: false,
        payload: {
          error: "bad_move",
          message: `Hangman move must be a single letter A-Z (got "${move}").`,
        },
      };
    }
    if (!this.hangman) {
      return { ok: false, payload: { error: "no_session", message: "No hangman session." } };
    }
    try {
      const hit = this.hangman.guess(letter);
      return {
        ok: true,
        payload: {
          accepted: true,
          action: "move",
          move: letter,
          hit,
          ...(this.snapshotHangman() ?? {}),
        },
      };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      return {
        ok: false,
        payload: {
          accepted: false,
          action: "move",
          move: letter,
          error: message,
          ...(this.snapshotHangman() ?? {}),
        },
      };
    }
  }
}

export function renderPayload(payload: Record<string, unknown>): string {
  const lines: string[] = [];
  if (typeof payload.board_text === "string") {
    lines.push(payload.board_text);
    lines.push("");
  }
  if (typeof payload.display === "string") {
    lines.push(`word: ${payload.display}`);
    lines.push(`guessed: ${String(payload.guessed ?? "")}`);
    lines.push(`attempts: ${String(payload.attempts_left)}/${String(payload.max_attempts)}`);
    lines.push("");
  }
  lines.push(JSON.stringify(payload, null, 2));
  return lines.join("\n");
}
