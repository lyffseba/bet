/**
 * Shared WASM play facade for TS hosts (MCP, OpenCode, pi).
 *
 * Rules come only from the Rust engine via `./index` (`createTtt` / `createHangman`).
 * This module owns session + views; it does not decide wins, losses, or guesses.
 * Hangman `word` is omitted until the engine reports game over.
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

export type PlayResult = {
  ok: boolean;
  payload: Record<string, unknown>;
};

export type Integrity = { ok: true } | { ok: false; error: string };

export type TttView = {
  game: "ttt";
  engine: "wasm";
  board: string;
  grid: (string | null)[][];
  board_text: string;
  status: string;
  current: string;
  empty_cells: number[];
  state_hash: string;
  you: string;
  opponent: string;
};

export type HangmanView = {
  game: "hangman";
  engine: "wasm";
  display: string;
  guessed: string;
  attempts_left: number;
  max_attempts: number;
  won: boolean;
  lost: boolean;
  over: boolean;
  /** Secret word — `null` until the engine reports game over. */
  word: string | null;
  state_hash: string;
};

export type EngineStatusHost = {
  package: string;
  version: string;
  transport?: string;
  play?: string;
  games?: string[];
  planned?: Record<string, unknown>;
  extra?: Record<string, unknown>;
};

function checkIntegrity(): Integrity {
  try {
    assertEngineIntegrity();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

const integrity = checkIntegrity();

export function engineIntegrity(): Integrity {
  return integrity;
}

export function freeEngine(engine: { free(): void } | null): void {
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

/** Public ttt view — no rules, only engine fields + formatting. */
export function viewTtt(handle: TttHandle): TttView {
  const board = handle.board();
  return {
    game: "ttt",
    engine: "wasm",
    board,
    grid: boardToGrid(board),
    board_text: formatTttBoard(board),
    status: handle.status(),
    current: handle.current(),
    empty_cells: emptyTttCells(board),
    state_hash: handle.state_hash(),
    you: "X (human)",
    opponent: "O (seeded WASM AI)",
  };
}

/**
 * Public hangman view. `word` is omitted until `is_over()` is true
 * (same contract as MCP / protocol: secret stays hidden mid-game).
 */
export function viewHangman(handle: HangmanHandle): HangmanView {
  const over = handle.is_over();
  return {
    game: "hangman",
    engine: "wasm",
    display: handle.display_word(),
    guessed: handle.guessed(),
    attempts_left: handle.attempts_left(),
    max_attempts: handle.max_attempts(),
    won: handle.is_won(),
    lost: handle.is_lost(),
    over,
    word: over ? handle.word() : null,
    state_hash: handle.state_hash(),
  };
}

export function engineStatus(host: EngineStatusHost): Record<string, unknown> {
  return {
    package: host.package,
    version: host.version,
    bet_ts: BET_TS_VERSION,
    engine: {
      version: engineVersion(),
      loader: "@lyffseba/bet-ts",
      play_facade: "@lyffseba/bet-ts/play",
      wasm_loaded: isWasmLoaded(),
      integrity: integrity.ok ? "ok" : "failed",
      integrity_error: integrity.ok ? undefined : integrity.error,
    },
    play: host.play ?? "in-process WASM (does not spawn the bet CLI)",
    stakes: "virtual points only (no real money, no zaps)",
    games: host.games ?? ["ttt", "hangman"],
    transport: host.transport,
    planned: host.planned,
    ...(host.extra ?? {}),
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
    const status = this.ttt.status();
    return {
      ...viewTtt(this.ttt),
      seed: this.tttSeed,
      hint:
        status === "ongoing"
          ? "Call bet_play with move=0..8 (numbered empty cells)."
          : "Game over. Call bet_play action=new to start again.",
    };
  }

  snapshotHangman(): Record<string, unknown> | null {
    if (!this.hangman) return null;
    const view = viewHangman(this.hangman);
    return {
      ...view,
      seed: this.hangmanSeed,
      hint: view.over
        ? "Game over. Call bet_play action=new to start again."
        : "Call bet_play with move=A..Z to guess a letter.",
    };
  }

  play(args: PlayArgs): PlayResult {
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

  private moveTtt(move: string): PlayResult {
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

  private moveHangman(move: string): PlayResult {
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
