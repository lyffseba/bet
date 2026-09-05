/**
 * Shared play facade: WASM rules only; hangman secret omitted until game over.
 */
import {
  BetTable,
  DEFAULT_HANGMAN_WORDS,
  normalizeGame,
  viewHangman,
  viewTtt,
} from "../src/play.ts";
import { createHangman, createTtt } from "../src/index.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

assert(normalizeGame("tictactoe") === "ttt", "tictactoe alias");
assert(normalizeGame("tic-tac-toe") === "ttt", "tic-tac-toe alias");
assert(normalizeGame("hangman") === "hangman", "hangman");
assert(normalizeGame("pong") === null, "pong is not WASM");

const hangman = createHangman(["SECRET"], 1n, 6);
const mid = viewHangman(hangman);
assert(mid.word === null, "secret must be omitted while the game is ongoing");
assert(mid.over === false, "fresh game is not over");
assert(mid.display.includes("_"), "masked display");

for (const ch of ["S", "E", "C", "R", "T"]) {
  hangman.guess(ch);
}
const ended = viewHangman(hangman);
assert(ended.over === true, "solved game is over");
assert(ended.won === true, "solved game is won");
assert(ended.word === "SECRET", "secret revealed only after game over");
hangman.free();

const ttt = createTtt(7n);
const tv = viewTtt(ttt);
assert(tv.board.length === 9, "board len");
assert(tv.empty_cells.length === 9, "all cells empty");
assert(tv.status === "ongoing", "fresh ttt");
ttt.free();

const table = new BetTable();
const started = table.play({ game: "hangman", action: "new", seed: 99 });
assert(started.ok, "start hangman");
assert(started.payload.word === null, "BetTable omits secret on new");
assert(started.payload.engine === "wasm", "engine tag");
assert(Array.isArray(DEFAULT_HANGMAN_WORDS) && DEFAULT_HANGMAN_WORDS.includes("CHARLIE"), "NATO bank");

const bad = table.play({ game: "pong", action: "new" });
assert(!bad.ok && bad.payload.error === "unknown_game", "non-WASM game rejected");

const tttNew = table.play({ game: "tictactoe", action: "new", seed: 7 });
assert(tttNew.ok, "ttt alias start");
const moved = table.play({ game: "ttt", move: "4" });
assert(moved.ok, "center vs AI");
assert(typeof moved.payload.board === "string", "board after move");
assert(moved.payload.word === undefined, "ttt payload has no hangman word");

console.log("play facade: OK (secret omitted until over; no TS rules)");
