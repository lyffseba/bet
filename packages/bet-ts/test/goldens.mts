/**
 * Cross-language goldens: WASM + ENGINE_GOLDENS + protocols fixture must agree.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  ENGINE_GOLDENS,
  assertEngineIntegrity,
  createTtt,
  engineVersion,
  isWasmLoaded,
} from "../src/index.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

const here = dirname(fileURLToPath(import.meta.url));
const fixturePath = join(here, "../../../protocols/fixtures/wasm_goldens.json");
const fileFix = JSON.parse(readFileSync(fixturePath, "utf8")) as {
  hangman: { seed: number; words: string[]; hash: string; word: string };
  ttt: { seed: number; hash: string };
};

assert(isWasmLoaded(), "wasm not loaded");
console.log("engine_version", engineVersion());

// Package constants ↔ repo fixture
assert(
  ENGINE_GOLDENS.hangman.hash === fileFix.hangman.hash,
  "ENGINE_GOLDENS hangman.hash drifted from protocols/fixtures",
);
assert(
  ENGINE_GOLDENS.ttt.hash === fileFix.ttt.hash,
  "ENGINE_GOLDENS ttt.hash drifted from protocols/fixtures",
);
assert(
  ENGINE_GOLDENS.hangman.word === fileFix.hangman.word,
  "ENGINE_GOLDENS hangman.word drifted",
);

assertEngineIntegrity();
console.log("hangman_golden", ENGINE_GOLDENS.hangman.hash);
console.log("ttt_golden", ENGINE_GOLDENS.ttt.hash);

const ttt = createTtt(42n);
assert(ttt.make_move_vs_ai(4) === true, "center move");
assert(ttt.board().length === 9, "board len");
ttt.free();

console.log("goldens: OK (TS constants + file fixture + WASM)");
