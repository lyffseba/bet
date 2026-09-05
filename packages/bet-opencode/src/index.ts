/**
 * OpenCode plugin for BET.
 *
 * Install (project):
 *   opencode.json → { "plugin": ["file:./packages/bet-opencode"] }
 * or copy to ~/.config/opencode/plugins/bet.ts
 *
 * hangman / ttt: in-process WASM via `@lyffseba/bet-ts/play` (does not spawn CLI).
 * menu / pong / chess / matrix: optional CLI spawn — those games are not in WASM yet.
 */
import type { Plugin } from "@opencode-ai/plugin";
import { tool } from "@opencode-ai/plugin";
import {
  BetTable,
  engineStatus,
  normalizeGame,
  renderPayload,
} from "@lyffseba/bet-ts/play";

const BET_OPENCODE_VERSION = "2.0.0-alpha.0";

/** Games not yet in the Rust/WASM core — CLI spawn only, labeled as such. */
const CLI_ONLY_GAMES = ["menu", "pong", "chess", "matrix"] as const;
type CliOnlyGame = (typeof CLI_ONLY_GAMES)[number];

function isCliOnlyGame(game: string): game is CliOnlyGame {
  return (CLI_ONLY_GAMES as readonly string[]).includes(game);
}

export const BetPlugin: Plugin = async ({ client, $ }) => {
  const table = new BetTable();

  await client.app.log({
    body: {
      service: "bet-opencode",
      level: "info",
      message: "BET plugin initialized",
      extra: {
        version: BET_OPENCODE_VERSION,
        play: "wasm-facade",
        facade: "@lyffseba/bet-ts/play",
      },
    },
  });

  return {
    tool: {
      bet_status: tool({
        description:
          "BET engine version + WASM integrity. hangman/ttt play in-process " +
          "(no CLI). Virtual points only. CLI spawn is only for games not yet in WASM.",
        args: {},
        async execute(_args, _ctx) {
          let cli = "not found";
          try {
            const out = await $`command -v bet`.text();
            cli = out.trim() || "not found";
          } catch {
            cli = "not found";
          }
          return JSON.stringify(
            engineStatus({
              package: "bet-opencode",
              version: BET_OPENCODE_VERSION,
              transport: "opencode-plugin",
              play: "ttt/hangman in-process WASM via @lyffseba/bet-ts/play (does not spawn CLI)",
              games: ["ttt", "hangman"],
              planned: {
                multiplayer: "host/join not exposed in OpenCode yet",
              },
              extra: {
                cli,
                cli_only_games: [...CLI_ONLY_GAMES],
                cli_only_note:
                  "CLI spawn is only for games not yet in WASM (menu/pong/chess/matrix)",
              },
            }),
            null,
            2,
          );
        },
      }),
      bet_play: tool({
        description:
          "Play tic-tac-toe (you are X vs seeded WASM AI) or hangman in-process via " +
          "@lyffseba/bet-ts/play. Hangman secret is hidden until the game ends. " +
          "Does not spawn the bet CLI for ttt/hangman. Virtual points only. " +
          "Optional: game=menu|pong|chess|matrix spawns the native CLI (not yet in WASM).",
        args: {
          game: tool.schema
            .enum(["ttt", "tictactoe", "hangman", "menu", "pong", "chess", "matrix"])
            .optional()
            .describe("ttt/hangman = in-process WASM. menu/pong/chess/matrix = CLI (not yet WASM)."),
          action: tool.schema
            .string()
            .optional()
            .describe("WASM only: new | move | status. Default: move if `move` is set, else new/status"),
          move: tool.schema
            .string()
            .optional()
            .describe("WASM only. ttt: empty-cell index 0-8. hangman: one letter A-Z"),
          seed: tool.schema
            .number()
            .optional()
            .describe("WASM only. Optional u32 seed for a new game"),
        },
        async execute(args, _ctx) {
          const game = args.game ?? "ttt";
          const wasmGame = normalizeGame(game);
          if (wasmGame) {
            const result = table.play({
              game: wasmGame,
              action: args.action,
              move: args.move,
              seed: args.seed,
            });
            return renderPayload(result.payload);
          }

          if (!isCliOnlyGame(game)) {
            return `Unknown game "${game}". WASM: ttt, hangman. CLI-only (not yet in WASM): ${CLI_ONLY_GAMES.join(", ")}.`;
          }

          // CLI spawn — only for games not yet in the WASM core.
          try {
            if (game === "menu") {
              await $`bet`.nothrow();
            } else {
              await $`bet ${game}`.nothrow();
            }
            return (
              `Launched bet CLI for "${game}" (not yet in WASM). ` +
              `hangman/ttt do not use this path — they play in-process. ` +
              `If nothing appeared, install the CLI: cd <bet-repo> && make install`
            );
          } catch (e) {
            return `Failed to launch bet CLI for "${game}" (not yet in WASM): ${String(e)}. Install: make install`;
          }
        },
      }),
    },
    event: async ({ event }) => {
      if (event.type === "session.idle") {
        try {
          await client.tui.showToast({
            body: {
              title: "BET",
              message: "Agent idle — bet_play (WASM ttt/hangman) or /b$t in pi",
              variant: "info",
            },
          });
        } catch {
          // Toast API may vary by opencode version; ignore.
        }
      }
    },
  };
};

export default BetPlugin;
