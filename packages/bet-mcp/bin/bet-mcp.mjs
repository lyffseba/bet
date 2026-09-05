#!/usr/bin/env node
/**
 * Launch the TypeScript stdio server with Node's type stripper.
 * Hosts should talk to this process; stdio is inherited by the TS entry.
 */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const entry = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../src/index.ts",
);

const child = spawn(
  process.execPath,
  ["--experimental-strip-types", entry, ...process.argv.slice(2)],
  { stdio: "inherit" },
);

child.on("error", (err) => {
  console.error(err);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
