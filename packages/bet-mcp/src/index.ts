/**
 * BET MCP stdio server.
 *
 * bet_status / bet_play: in-process @lyffseba/bet-ts (Rust WASM).
 * bet_host / bet_join: thin spawn of the native `bet` CLI (Rust protocol).
 * Does not reimplement hangman/ttt rules. Stakes are virtual points only.
 */
import { McpServer } from "@modelcontextprotocol/server";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import * as z from "zod";
import { asCliResult, runHost, runJoin } from "./cli.ts";
import {
  BET_MCP_VERSION,
  BetTable,
  engineStatus,
  renderPayload,
} from "./engine.ts";

const playInput = z.object({
  game: z
    .string()
    .describe("Game to play: ttt (aliases: tictactoe) or hangman"),
  action: z
    .string()
    .optional()
    .describe("new | move | status. Default: move if `move` is set, else new/status"),
  move: z
    .string()
    .optional()
    .describe("ttt: empty-cell index 0-8. hangman: one letter A-Z"),
  seed: z
    .number()
    .int()
    .optional()
    .describe("Optional u32 seed for a new game (deterministic vs AI / word pick)"),
});

const hostInput = z.object({
  game: z
    .string()
    .optional()
    .describe("ttt (default, aliases: tictactoe) or hangman — passed to `bet host --game`"),
  stake: z
    .number()
    .int()
    .optional()
    .describe("Virtual points to wager (`--stake`, default 10). Not real money."),
  name: z.string().optional().describe("Player id (`--name`, default $BET_PLAYER or $USER)"),
  port: z
    .number()
    .int()
    .optional()
    .describe("Listen port (`--port`, default 7733). Use 0 for an ephemeral port."),
  bind: z.string().optional().describe("Bind address (`--bind`, default 0.0.0.0)"),
  code: z
    .string()
    .optional()
    .describe("Fixed room code (`--code`, 4–8 alnum). Useful for scripted joins."),
  word: z
    .string()
    .optional()
    .describe("Hangman host only: pin the secret (`--word`). Not returned in tool JSON."),
  seed: z
    .number()
    .int()
    .optional()
    .describe("Hangman host: pick word from the CLI bank (`--seed`) when `word` is omitted"),
  moves: z
    .string()
    .optional()
    .describe(
      "Scripted host turns via BET_MOVES (ttt: 0,1,2 — hangman: B,E,T — q resigns). " +
        "Always set (empty if omitted) so the CLI never reads stdin under MCP.",
    ),
  wait: z
    .enum(["ready", "end"])
    .optional()
    .describe("ready (default): return after BET_READY, leave host running. end: wait until exit."),
  timeout_ms: z.number().int().optional().describe("Cap wait (default 10s ready / 60s end, max 5min)"),
  config_dir: z.string().optional().describe("Ledger directory (BET_CONFIG_DIR)"),
});

const joinInput = z.object({
  target: z
    .string()
    .optional()
    .describe("Join target: CODE or CODE@host:port (from bet_host `join` / BET_READY)"),
  code: z.string().optional().describe("Room code if `target` is omitted"),
  addr: z.string().optional().describe("host:port (`--addr`) when target is a bare CODE"),
  game: z.string().optional().describe("ttt (default) or hangman — must match the host room"),
  stake: z
    .number()
    .int()
    .optional()
    .describe("Virtual points (`--stake`, must match the host). Not real money."),
  name: z.string().optional().describe("Player id (`--name`)"),
  moves: z
    .string()
    .optional()
    .describe(
      "Scripted guest turns via BET_MOVES. Empty if omitted (resigns when it is the guest's turn).",
    ),
  timeout_ms: z.number().int().optional().describe("Cap wait for match end (default 60s, max 5min)"),
  config_dir: z.string().optional().describe("Ledger directory (BET_CONFIG_DIR)"),
});

function jsonResult(result: { ok: boolean; payload: Record<string, unknown> }) {
  return {
    content: [{ type: "text" as const, text: JSON.stringify(result.payload, null, 2) }],
    isError: !result.ok,
  };
}

function createServer(): McpServer {
  const table = new BetTable();
  const server = new McpServer({
    name: "bet",
    version: BET_MCP_VERSION,
  });

  server.registerTool(
    "bet_status",
    {
      description:
        "BET engine version and WASM integrity check. Virtual points only. " +
        "bet_play is in-process WASM. bet_host / bet_join spawn the native `bet` CLI " +
        "(must be on PATH, or set BET_BIN; `make install`).",
    },
    async () => {
      const payload = engineStatus();
      return {
        content: [{ type: "text" as const, text: JSON.stringify(payload, null, 2) }],
      };
    },
  );

  server.registerTool(
    "bet_play",
    {
      description:
        "Play tic-tac-toe (vs seeded WASM AI) or hangman in-process via @lyffseba/bet-ts. " +
        "You are X on ttt (move=0..8). Hangman: move=A-Z. Secret word is hidden until the game ends. " +
        "Does not spawn the bet CLI. Virtual points only.",
      inputSchema: playInput,
    },
    async (args) => {
      const result = table.play(args);
      return {
        content: [{ type: "text" as const, text: renderPayload(result.payload) }],
        isError: !result.ok,
      };
    },
  );

  server.registerTool(
    "bet_host",
    {
      description:
        "Thin CLI bridge: spawn `bet host` (native binary on PATH / BET_BIN). " +
        "Does not reimplement multiplayer rules — Rust CLI/protocol owns the room. " +
        "Default wait=ready returns the BET_READY line + CODE@host:port and leaves the host running. " +
        "Uses BET_MOVES (empty if omitted) so MCP stdio never hangs on the TUI. " +
        "Hangman --word is passed to the CLI only and is omitted from this tool's JSON. " +
        "Virtual points only. Fails loud if `bet` is missing (`make install`).",
      inputSchema: hostInput,
    },
    async (args) => jsonResult(await asCliResult(runHost(args))),
  );

  server.registerTool(
    "bet_join",
    {
      description:
        "Thin CLI bridge: spawn `bet join CODE|CODE@host:port` (native binary on PATH / BET_BIN). " +
        "Does not reimplement multiplayer rules — Rust CLI/protocol owns the room. " +
        "Uses BET_MOVES (empty if omitted) so MCP stdio never hangs on the TUI. " +
        "Returns CLI stdout/stderr and MATCH ENDED summary. " +
        "Hangman secret appears in stdout only after MATCH ENDED (CLI never puts it on the wire earlier). " +
        "Virtual points only. Fails loud if `bet` is missing (`make install`).",
      inputSchema: joinInput,
    },
    async (args) => jsonResult(await asCliResult(runJoin(args))),
  );

  return server;
}

void serveStdio(createServer);
console.error(
  `bet-mcp ${BET_MCP_VERSION} stdio — WASM play + CLI host/join, virtual points only`,
);
