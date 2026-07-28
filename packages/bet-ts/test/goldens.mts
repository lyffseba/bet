/**
 * Cross-language goldens: WASM must match protocols/fixtures/wasm_goldens.json
 * (same values asserted in bet-core Rust tests).
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  createHangman,
  createTtt,
  engineVersion,
  goldenHangmanHash,
  goldenTttHash,
  isWasmLoaded,
} from "../src/index.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

const here = dirname(fileURLToPath(import.meta.url));
const fixturePath = join(here, "../../../protocols/fixtures/wasm_goldens.json");
const fixture = JSON.parse(readFileSync(fixturePath, "utf8")) as {
  hangman: { seed: number; words: string[]; hash: string; word: string };
  ttt: { seed: number; hash: string };
};

assert(isWasmLoaded(), "wasm not loaded");
const ver = engineVersion();
console.log("engine_version", ver);
assert(ver.length > 0, "version empty");

const wordsJson = JSON.stringify(fixture.hangman.words);
const h1 = goldenHangmanHash(BigInt(fixture.hangman.seed), wordsJson);
const h2 = goldenHangmanHash(BigInt(fixture.hangman.seed), wordsJson);
assert(h1 === h2, "hangman golden not stable");
assert(
  h1 === fixture.hangman.hash,
  `hangman hash mismatch: got ${h1} want ${fixture.hangman.hash} (wasm32 RNG bug?)`,
);
console.log("hangman_golden", h1);

const hm = createHangman(
  fixture.hangman.words,
  BigInt(fixture.hangman.seed),
  6,
);
assert(hm.word() === fixture.hangman.word, `word ${hm.word()} != ${fixture.hangman.word}`);

const t1 = goldenTttHash(BigInt(fixture.ttt.seed));
const t2 = goldenTttHash(BigInt(fixture.ttt.seed));
assert(t1 === t2, "ttt golden not stable");
assert(
  t1 === fixture.ttt.hash,
  `ttt hash mismatch: got ${t1} want ${fixture.ttt.hash}`,
);
console.log("ttt_golden", t1);

const ttt = createTtt(42n);
assert(ttt.make_move_vs_ai(4) === true, "center move");
assert(ttt.board().length === 9, "board len");

console.log("goldens: OK (matches Rust fixture)");
