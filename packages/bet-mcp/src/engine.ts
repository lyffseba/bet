/**
 * MCP adapter over the shared WASM play facade (`@lyffseba/bet-ts/play`).
 * Rules stay in Rust; this file only names the MCP host.
 */
import { engineStatus as sharedEngineStatus } from "@lyffseba/bet-ts/play";

export const BET_MCP_VERSION = "2.0.0-alpha.0";

export { BetTable, renderPayload } from "@lyffseba/bet-ts/play";

export function engineStatus(): Record<string, unknown> {
  return sharedEngineStatus({
    package: "@lyffseba/bet-mcp",
    version: BET_MCP_VERSION,
    transport: "stdio",
    planned: {
      multiplayer: "host/join not exposed in MCP yet",
    },
  });
}
