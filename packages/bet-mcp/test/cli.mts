/**
 * MCP CLI bridge: parse/redact/resolve + optional live `bet host`/`bet join`.
 * Does not implement protocol — it only checks the spawn wrapper.
 */
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  cliGame,
  joinTargetFromReady,
  parseBetReady,
  redactArgv,
  resolveBetBinary,
  runHost,
  runJoin,
} from "../src/cli.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

const readyLine =
  "BET multiplayer host\nBET_READY room=WIN001 port=18001 stake=10 proto=1 game=ttt\nWaiting\n";
const parsed = parseBetReady(readyLine);
assert(parsed !== null, "parse BET_READY");
assert(parsed.room === "WIN001", "room");
assert(parsed.port === 18001, "port");
assert(parsed.stake === 10, "stake");
assert(parsed.proto === 1, "proto");
assert(parsed.game === "ttt", "game");
assert(joinTargetFromReady(parsed, "0.0.0.0") === "WIN001@127.0.0.1:18001", "loopback join");
assert(joinTargetFromReady(parsed, "10.0.0.2") === "WIN001@10.0.0.2:18001", "bind join");

const hang = parseBetReady("BET_READY room=HNG001 port=9 stake=10 proto=1 game=hangman");
assert(hang?.game === "hangman", "hangman ready");

assert(parseBetReady("nope") === null, "missing ready");
assert(cliGame("tictactoe") === "ttt", "ttt alias");
assert(cliGame("hangman") === "hangman", "hangman");
try {
  cliGame("pong");
  throw new Error("pong should be rejected");
} catch (e) {
  assert(e instanceof Error && e.message.includes("unsupported"), "pong rejected");
}

const redacted = redactArgv(["host", "--game", "hangman", "--word", "SECRET", "--seed", "1"]);
assert(!redacted.includes("SECRET"), "redact --word value");
assert(redactArgv(["host", "--word=SECRET"])[1] === "--word=***", "redact --word=");

try {
  resolveBetBinary({ PATH: "", BET_BIN: "/no/such/bet-binary" });
  throw new Error("missing BET_BIN should throw");
} catch (e) {
  assert(e instanceof Error && e.message.includes("make install"), "loud missing-binary hint");
}

try {
  resolveBetBinary({ PATH: "/empty/path/does/not/exist" });
  throw new Error("empty PATH should throw");
} catch (e) {
  assert(e instanceof Error && /not on PATH/.test(e.message), "PATH miss is loud");
}

const prevBin = process.env.BET_BIN;
process.env.BET_BIN = "/no/such/bet-binary";
const hostMissing = await runHost({ game: "ttt" });
if (prevBin === undefined) delete process.env.BET_BIN;
else process.env.BET_BIN = prevBin;
assert(!hostMissing.ok && hostMissing.payload.error === "bet_missing", "runHost fails if bet missing");
assert(String(hostMissing.payload.message).includes("make install"), "runHost fails loud");

console.log("mcp cli unit: OK");

const bin = process.env.BET_BIN?.trim() || (() => {
  try {
    return resolveBetBinary();
  } catch {
    return "";
  }
})();

if (!bin) {
  console.log("mcp cli smoke: SKIP (no bet on PATH; make install or set BET_BIN)");
  process.exit(0);
}

const cfg = mkdtempSync(path.join(tmpdir(), "bet-mcp-"));
const port = 23000 + Math.floor(Math.random() * 1000);
const code = "MCP001";
writeFileSync(path.join(cfg, ".keep"), "");

const host = await runHost({
  game: "ttt",
  stake: 10,
  name: "alice",
  port,
  bind: "127.0.0.1",
  code,
  moves: "0,1,2",
  wait: "ready",
  config_dir: cfg,
  timeout_ms: 15_000,
});
assert(host.ok, `host ready: ${JSON.stringify(host.payload)}`);
assert(host.payload.room === code, "host room");
assert(host.payload.join === `${code}@127.0.0.1:${host.payload.port}`, "host join");
assert(host.payload.word === undefined, "host JSON must not echo a hangman word");
assert(typeof host.payload.ready === "string" && String(host.payload.ready).startsWith("BET_READY "), "ready line");

const guest = await runJoin({
  target: String(host.payload.join),
  game: "ttt",
  stake: 10,
  name: "bob",
  moves: "3,4",
  config_dir: cfg,
  timeout_ms: 20_000,
});
assert(guest.ok, `join: ${JSON.stringify(guest.payload)}`);
assert(guest.payload.match && typeof guest.payload.match === "object", "match summary");
assert(String(guest.payload.stdout).includes("MATCH ENDED"), "guest saw MATCH ENDED");

const hport = 24000 + Math.floor(Math.random() * 1000);
const hcfg = mkdtempSync(path.join(tmpdir(), "bet-mcp-h-"));
const hhost = await runHost({
  game: "hangman",
  stake: 10,
  name: "alice",
  port: hport,
  bind: "127.0.0.1",
  code: "MCPH01",
  word: "BET",
  moves: "B,E,T",
  wait: "ready",
  config_dir: hcfg,
  timeout_ms: 15_000,
});
assert(hhost.ok, `hangman host: ${JSON.stringify(hhost.payload)}`);
assert(hhost.payload.word === undefined, "wrapper must not echo --word");
assert(Array.isArray(hhost.payload.argv) && !(hhost.payload.argv as string[]).includes("BET"), "argv redacts word");
assert(!/word=BET/.test(String(hhost.payload.stdout)), "host stdout has no word= before match");

const hguest = await runJoin({
  target: String(hhost.payload.join),
  game: "hangman",
  stake: 10,
  name: "bob",
  moves: "",
  config_dir: hcfg,
  timeout_ms: 20_000,
});
assert(hguest.ok, `hangman join: ${JSON.stringify(hguest.payload)}`);
const gout = String(hguest.payload.stdout);
const leakBeforeEnd = (() => {
  const end = gout.indexOf("MATCH ENDED");
  const slice = end === -1 ? gout : gout.slice(0, end);
  return /word=BET/.test(slice);
})();
assert(!leakBeforeEnd, "guest stdout must not show word=BET before MATCH ENDED");

console.log("mcp cli smoke: OK (spawned bet host/join; no TS protocol)");
