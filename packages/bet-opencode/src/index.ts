/**
 * OpenCode plugin for BET.
 *
 * Install (project):
 *   opencode.json → { "plugin": ["file:./packages/bet-opencode"] }
 * or copy to ~/.config/opencode/plugins/bet.ts
 *
 * v0: custom tools + session.idle toast. Full TUI via spawning the `bet` binary.
 */

import type { Plugin } from "@opencode-ai/plugin";
import { tool } from "@opencode-ai/plugin";

export const BetPlugin: Plugin = async ({ client, $ }) => {
  await client.app.log({
    body: {
      service: "bet-opencode",
      level: "info",
      message: "BET plugin initialized",
      extra: { version: "2.0.0-alpha.0" },
    },
  });

  return {
    tool: {
      bet_status: tool({
        description:
          "Report BET virtual ledger path and whether the `bet` CLI is on PATH",
        args: {},
        async execute(_args, _ctx) {
          let which = "not found";
          try {
            const out = await $`command -v bet`.text();
            which = out.trim() || "not found";
          } catch {
            which = "not found";
          }
          return JSON.stringify(
            {
              engine: "rust-bet-core (via CLI/WASM)",
              cli: which,
              ledger: "~/.config/bet/ledger.json (planned E2)",
              multiplayer: "planned E2 (host/join + virtual stakes)",
              tip: "Run `bet` in a terminal, or `make install` from the monorepo",
            },
            null,
            2,
          );
        },
      }),
      bet_play: tool({
        description:
          "Launch a BET game in the terminal (spawns the native `bet` binary)",
        args: {
          game: tool.schema
            .enum(["menu", "hangman", "tictactoe", "pong", "chess", "matrix"])
            .optional()
            .describe("Game to open; menu if omitted"),
        },
        async execute(args, _ctx) {
          const game = args.game ?? "menu";
          // OpenCode plugins do not embed a full TUI; spawn CLI when available.
          try {
            if (game === "menu") {
              await $`bet`.nothrow();
            } else {
              await $`bet ${game}`.nothrow();
            }
            return `Launched bet (${game}). If nothing appeared, install the CLI with: cd <bet-repo> && make install`;
          } catch (e) {
            return `Failed to launch bet: ${String(e)}. Install: make install`;
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
              message: "Agent idle — play /b$t in pi or run tool bet_play",
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
