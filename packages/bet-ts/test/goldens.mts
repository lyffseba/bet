/**
 * Cross-language smoke: WASM goldens must be stable and non-empty.
 */
import {
  engineVersion,
  goldenHangmanHash,
  goldenTttHash,
  createHangman,
  createTtt,
  isWasmLoaded,
} from "../src/index.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

assert(isWasmLoaded(), "wasm not loaded");
const ver = engineVersion();
console.log("engine_version", ver);
assert(ver.length > 0, "version empty");

const words = JSON.stringify(["ALPHA", "BRAVO", "CHARLIE"]);
const h1 = goldenHangmanHash(99n, words);
const h2 = goldenHangmanHash(99n, words);
assert(h1 === h2, "hangman golden not stable");
assert(h1.length === 16, `hangman hash len ${h1.length}`);
console.log("hangman_golden", h1);

const t1 = goldenTttHash(7n);
const t2 = goldenTttHash(7n);
assert(t1 === t2, "ttt golden not stable");
assert(t1.length === 16, `ttt hash len ${t1.length}`);
console.log("ttt_golden", t1);

const hm = createHangman(["BET"], 1n, 6);
assert(hm.guess("B") === true, "guess B");
assert(hm.display_word().includes("B"), "display");
assert(hm.state_hash().length === 16, "state hash");

const ttt = createTtt(42n);
assert(ttt.make_move_vs_ai(4) === true, "center move");
assert(ttt.board().length === 9, "board len");
assert(["ongoing", "win_x", "win_o", "draw"].includes(ttt.status()), "status");

console.log("goldens: OK");
