/**
 * Thin CLI bridge for multiplayer MCP tools.
 *
 * Spawns the native `bet` binary (`bet host` / `bet join`). Does **not**
 * reimplement hangman/ttt rules or the TCP NDJSON protocol — those stay in
 * Rust (`bet-cli` / `bet-protocol`). Hangman `--word` is never copied into
 * tool JSON; the CLI itself keeps the secret off the wire until MatchEnded.
 */
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import path from "node:path";

export const DEFAULT_READY_TIMEOUT_MS = 10_000;
export const DEFAULT_END_TIMEOUT_MS = 60_000;
const MAX_PIPE = 256_000;

const liveChildren = new Set<ChildProcess>();

export type BetReady = {
  room: string;
  port: number;
  stake: number;
  proto: number;
  game: string;
  raw: string;
};

export type HostWait = "ready" | "end";

export type HostArgs = {
  game?: string;
  stake?: number;
  name?: string;
  port?: number;
  bind?: string;
  code?: string;
  /** Hangman host: pin the secret. Passed to the CLI only — omitted from JSON. */
  word?: string;
  seed?: number;
  /** Scripted turns (`BET_MOVES`). Empty / omitted ⇒ set so stdin is never read. */
  moves?: string;
  wait?: HostWait;
  timeout_ms?: number;
  /** Optional ledger dir (`BET_CONFIG_DIR`). */
  config_dir?: string;
};

export type JoinArgs = {
  /** `CODE` or `CODE@host:port`. Required unless `code` is set. */
  target?: string;
  code?: string;
  addr?: string;
  game?: string;
  stake?: number;
  name?: string;
  moves?: string;
  timeout_ms?: number;
  config_dir?: string;
};

export type CliResult = {
  ok: boolean;
  payload: Record<string, unknown>;
};

function missingBet(detail: string): Error {
  return new Error(
    `${detail}. Multiplayer tools spawn the native \`bet\` CLI (Rust host/join) — ` +
      "they do not implement the protocol in TypeScript. " +
      "Install: from the bet repo run `make install` (copies target/release/bet → ~/.local/bin/bet) " +
      "and ensure ~/.local/bin is on PATH. Or set BET_BIN to the binary.",
  );
}

function isFile(p: string): boolean {
  try {
    return existsSync(p) && statSync(p).isFile();
  } catch {
    return false;
  }
}

/** Resolve `bet` from `BET_BIN` or PATH. Throws if missing. */
export function resolveBetBinary(env: NodeJS.ProcessEnv = process.env): string {
  const pinned = env.BET_BIN?.trim();
  if (pinned) {
    if (isFile(pinned)) return pinned;
    throw missingBet(`BET_BIN=${pinned} is not an executable file`);
  }
  const pathEnv = env.PATH ?? "";
  for (const dir of pathEnv.split(path.delimiter)) {
    if (!dir) continue;
    const candidate = path.join(dir, "bet");
    if (isFile(candidate)) return candidate;
  }
  throw missingBet("`bet` is not on PATH");
}

export function cliGame(raw?: string): string | undefined {
  if (raw === undefined || raw === "") return undefined;
  const g = raw.trim().toLowerCase();
  if (g === "ttt" || g === "tictactoe" || g === "tic-tac-toe" || g === "tic_tac_toe") {
    return "ttt";
  }
  if (g === "hangman") return "hangman";
  throw new Error(`unsupported multiplayer game "${raw}" (CLI host/join: ttt | hangman)`);
}

export function parseBetReady(text: string): BetReady | null {
  const line = text.split(/\r?\n/).find((l) => l.startsWith("BET_READY "));
  if (!line) return null;
  const fields: Record<string, string> = {};
  for (const part of line.slice("BET_READY ".length).trim().split(/\s+/)) {
    const eq = part.indexOf("=");
    if (eq > 0) fields[part.slice(0, eq)] = part.slice(eq + 1);
  }
  const room = fields.room;
  const port = Number(fields.port);
  if (!room || !Number.isFinite(port)) return null;
  return {
    room,
    port,
    stake: Number(fields.stake) || 0,
    proto: Number(fields.proto) || 1,
    game: fields.game || "ttt",
    raw: line,
  };
}

export function joinTargetFromReady(ready: BetReady, bind?: string): string {
  const host = !bind || bind === "0.0.0.0" || bind === "::" || bind === "[::]"
    ? "127.0.0.1"
    : bind;
  return `${ready.room}@${host}:${ready.port}`;
}

/** Redact `--word` so wrappers never echo the hangman secret. */
export function redactArgv(argv: string[]): string[] {
  const out = [...argv];
  for (let i = 0; i < out.length; i++) {
    if (out[i] === "--word" && out[i + 1] !== undefined) {
      out[i + 1] = "***";
    } else if (out[i]?.startsWith("--word=")) {
      out[i] = "--word=***";
    }
  }
  return out;
}

export function summarizeMatch(stdout: string): Record<string, unknown> | undefined {
  if (!/MATCH ENDED/.test(stdout)) return undefined;
  const winner = /winner=([^\s]+)/.exec(stdout)?.[1];
  const pot = /pot=(\d+)/.exec(stdout)?.[1];
  const word = /^word=(.+)$/m.exec(stdout)?.[1];
  const out: Record<string, unknown> = { ended: true };
  if (winner) out.winner = winner;
  if (pot) out.pot = Number(pot);
  if (word) out.word = word;
  return out;
}

function track(child: ChildProcess): void {
  liveChildren.add(child);
  child.on("exit", () => liveChildren.delete(child));
}

export function killTracked(signal: NodeJS.Signals = "SIGTERM"): void {
  for (const child of liveChildren) {
    try {
      child.kill(signal);
    } catch {
      // already gone
    }
  }
}

type PipeBuf = { stdout: string; stderr: string };

function attachPipes(child: ChildProcess, buf: PipeBuf): void {
  const append = (key: keyof PipeBuf) => (chunk: Buffer | string) => {
    buf[key] += typeof chunk === "string" ? chunk : chunk.toString("utf8");
    if (buf[key].length > MAX_PIPE) {
      buf[key] = buf[key].slice(-MAX_PIPE);
    }
  };
  child.stdout?.setEncoding("utf8");
  child.stderr?.setEncoding("utf8");
  child.stdout?.on("data", append("stdout"));
  child.stderr?.on("data", append("stderr"));
}

function spawnBet(
  bin: string,
  argv: string[],
  extraEnv: NodeJS.ProcessEnv,
): ChildProcess {
  const child = spawn(bin, argv, {
    env: { ...process.env, ...extraEnv },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  track(child);
  return child;
}

function failSpawn(bin: string, err: unknown): CliResult {
  const message = err instanceof Error ? err.message : String(err);
  return {
    ok: false,
    payload: {
      error: "spawn_failed",
      message,
      hint: `Could not launch \`${bin}\`. Install with \`make install\` or set BET_BIN.`,
    },
  };
}

function waitForReady(
  child: ChildProcess,
  buf: PipeBuf,
  timeoutMs: number,
): Promise<BetReady> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (fn: () => void) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      child.stdout?.off("data", onData);
      child.stderr?.off("data", onData);
      child.off("error", onError);
      child.off("exit", onExit);
      fn();
    };
    const tryParse = (): boolean => {
      const ready = parseBetReady(buf.stdout) ?? parseBetReady(buf.stderr);
      if (ready) {
        finish(() => resolve(ready));
        return true;
      }
      return false;
    };
    const onData = () => {
      tryParse();
    };
    const onError = (err: Error) => {
      finish(() => reject(err));
    };
    const onExit = (code: number | null, signal: NodeJS.Signals | null) => {
      if (tryParse()) return;
      finish(() =>
        reject(
          new Error(
            `bet host exited before BET_READY (code=${code} signal=${signal}). ` +
              `stdout:\n${buf.stdout}\nstderr:\n${buf.stderr}`,
          ),
        ),
      );
    };
    const timer = setTimeout(() => {
      finish(() => {
        try {
          child.kill("SIGTERM");
        } catch {
          // ignore
        }
        reject(
          new Error(
            `timed out waiting for BET_READY after ${timeoutMs}ms. ` +
              `stdout:\n${buf.stdout}\nstderr:\n${buf.stderr}`,
          ),
        );
      });
    }, timeoutMs);
    child.stdout?.on("data", onData);
    child.stderr?.on("data", onData);
    child.on("error", onError);
    child.on("exit", onExit);
    tryParse();
  });
}

function waitForExit(
  child: ChildProcess,
  timeoutMs: number,
): Promise<{ code: number | null; signal: NodeJS.Signals | null }> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (fn: () => void) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      fn();
    };
    const timer = setTimeout(() => {
      finish(() => {
        try {
          child.kill("SIGTERM");
        } catch {
          // ignore
        }
        reject(new Error(`timed out waiting for bet to exit after ${timeoutMs}ms`));
      });
    }, timeoutMs);
    child.once("error", (err) => finish(() => reject(err)));
    child.once("exit", (code, signal) => finish(() => resolve({ code, signal })));
  });
}

function clampTimeout(ms: number | undefined, fallback: number): number {
  if (ms === undefined || !Number.isFinite(ms)) return fallback;
  return Math.min(Math.max(Math.trunc(ms), 250), 5 * 60_000);
}

function hostArgv(args: HostArgs): string[] {
  const argv = ["host"];
  const game = cliGame(args.game);
  if (game) argv.push("--game", game);
  if (args.stake !== undefined) argv.push("--stake", String(args.stake));
  if (args.name) argv.push("--name", args.name);
  if (args.port !== undefined) argv.push("--port", String(args.port));
  if (args.bind) argv.push("--bind", args.bind);
  if (args.code) argv.push("--code", args.code);
  if (game === "hangman" && args.word) argv.push("--word", args.word);
  if (game === "hangman" && args.seed !== undefined) {
    argv.push("--seed", String(args.seed));
  }
  return argv;
}

function joinArgv(args: JoinArgs): string[] {
  const target = args.target?.trim() || args.code?.trim();
  if (!target) {
    throw new Error("bet_join requires `target` (CODE or CODE@host:port) or `code`");
  }
  const argv = ["join", target];
  const game = cliGame(args.game);
  if (game) argv.push("--game", game);
  if (args.stake !== undefined) argv.push("--stake", String(args.stake));
  if (args.name) argv.push("--name", args.name);
  if (args.addr) argv.push("--addr", args.addr);
  return argv;
}

function extraEnv(args: { moves?: string; config_dir?: string }): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = {
    // Always set so the CLI never blocks on stdin under MCP stdio.
    BET_MOVES: args.moves ?? "",
  };
  if (args.config_dir) env.BET_CONFIG_DIR = args.config_dir;
  return env;
}

export function runHost(args: HostArgs = {}): CliResult | Promise<CliResult> {
  let bin: string;
  try {
    bin = resolveBetBinary();
  } catch (e) {
    return {
      ok: false,
      payload: {
        error: "bet_missing",
        message: e instanceof Error ? e.message : String(e),
      },
    };
  }

  let argv: string[];
  try {
    argv = hostArgv(args);
  } catch (e) {
    return {
      ok: false,
      payload: {
        error: "bad_args",
        message: e instanceof Error ? e.message : String(e),
      },
    };
  }

  const wait: HostWait = args.wait === "end" ? "end" : "ready";
  const timeoutMs = clampTimeout(
    args.timeout_ms,
    wait === "end" ? DEFAULT_END_TIMEOUT_MS : DEFAULT_READY_TIMEOUT_MS,
  );
  const env = extraEnv(args);
  const child = spawnBet(bin, argv, env);
  const buf: PipeBuf = { stdout: "", stderr: "" };
  attachPipes(child, buf);

  return new Promise((resolve) => {
    let settled = false;
    const done = (result: CliResult) => {
      if (settled) return;
      settled = true;
      resolve(result);
    };
    child.once("error", (err) => done(failSpawn(bin, err)));

    const base = () => ({
      tool: "bet_host",
      via: "cli",
      binary: bin,
      argv: redactArgv(argv),
      pid: child.pid ?? null,
      wait,
      stakes: "virtual points only",
      note:
        "Spawned `bet host`. Rules/protocol stay in Rust. " +
        "Hangman secret is not returned here; CLI prints word= only after MATCH ENDED.",
    });

    if (wait === "ready") {
      waitForReady(child, buf, timeoutMs)
        .then((ready) => {
          const join = joinTargetFromReady(ready, args.bind);
          done({
            ok: true,
            payload: {
              ...base(),
              ready: ready.raw,
              room: ready.room,
              port: ready.port,
              stake: ready.stake,
              proto: ready.proto,
              game: ready.game,
              join,
              guest_cmd: `bet join ${join} --stake ${ready.stake} --game ${ready.game}`,
              running: true,
              stdout: buf.stdout,
              stderr: buf.stderr,
              hint:
                "Host is still running (waiting for a guest). Call bet_join with `target` = join. " +
                "Pass `moves` on host/join for scripted turns; empty BET_MOVES resigns on that side's turn.",
            },
          });
        })
        .catch((err: unknown) => {
          done({
            ok: false,
            payload: {
              ...base(),
              error: "host_ready_failed",
              message: err instanceof Error ? err.message : String(err),
              stdout: buf.stdout,
              stderr: buf.stderr,
            },
          });
        });
      return;
    }

    waitForExit(child, timeoutMs)
      .then(({ code, signal }) => {
        const match = summarizeMatch(buf.stdout);
        done({
          ok: code === 0,
          payload: {
            ...base(),
            exit_code: code,
            signal,
            match,
            stdout: buf.stdout,
            stderr: buf.stderr,
          },
        });
      })
      .catch((err: unknown) => {
        done({
          ok: false,
          payload: {
            ...base(),
            error: "host_wait_failed",
            message: err instanceof Error ? err.message : String(err),
            stdout: buf.stdout,
            stderr: buf.stderr,
          },
        });
      });
  });
}

export function runJoin(args: JoinArgs): CliResult | Promise<CliResult> {
  let bin: string;
  try {
    bin = resolveBetBinary();
  } catch (e) {
    return {
      ok: false,
      payload: {
        error: "bet_missing",
        message: e instanceof Error ? e.message : String(e),
      },
    };
  }

  let argv: string[];
  try {
    argv = joinArgv(args);
  } catch (e) {
    return {
      ok: false,
      payload: {
        error: "bad_args",
        message: e instanceof Error ? e.message : String(e),
      },
    };
  }

  const timeoutMs = clampTimeout(args.timeout_ms, DEFAULT_END_TIMEOUT_MS);
  const env = extraEnv(args);
  const child = spawnBet(bin, argv, env);
  const buf: PipeBuf = { stdout: "", stderr: "" };
  attachPipes(child, buf);

  return new Promise((resolve) => {
    let settled = false;
    const done = (result: CliResult) => {
      if (settled) return;
      settled = true;
      resolve(result);
    };
    child.once("error", (err) => done(failSpawn(bin, err)));
    const base = () => ({
      tool: "bet_join",
      via: "cli",
      binary: bin,
      argv: redactArgv(argv),
      pid: child.pid ?? null,
      stakes: "virtual points only",
      note:
        "Spawned `bet join`. Rules/protocol stay in Rust. " +
        "word= appears in CLI stdout only after MATCH ENDED.",
    });
    waitForExit(child, timeoutMs)
      .then(({ code, signal }) => {
        const match = summarizeMatch(buf.stdout);
        done({
          ok: code === 0,
          payload: {
            ...base(),
            exit_code: code,
            signal,
            match,
            stdout: buf.stdout,
            stderr: buf.stderr,
          },
        });
      })
      .catch((err: unknown) => {
        done({
          ok: false,
          payload: {
            ...base(),
            error: "join_wait_failed",
            message: err instanceof Error ? err.message : String(err),
            stdout: buf.stdout,
            stderr: buf.stderr,
          },
        });
      });
  });
}

export async function asCliResult(
  result: CliResult | Promise<CliResult>,
): Promise<CliResult> {
  return result;
}
