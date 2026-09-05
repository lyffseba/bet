/**
 * BET MCP stdio server.
 *
 * Play hangman / tic-tac-toe in-process through @lyffseba/bet-ts (Rust WASM).
 * Does not spawn the `bet` CLI. Stakes are virtual points only.
 */
import { McpServer } from "@modelcontextprotocol/server";
import { serveStdio } from "@modelcontextprotocol/server/stdio";
import * as z from "zod";
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
        "BET engine version and WASM integrity check. Virtual points only; " +
        "play is in-process WASM (not the bet CLI). Multiplayer host/join is planned.",
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

  return server;
}

void serveStdio(createServer);
console.error(
  `bet-mcp ${BET_MCP_VERSION} stdio — in-process WASM, virtual points only`,
);
