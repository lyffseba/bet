/**
 * Doc-gate cases for fence parsing, help tokens, launches, placeholders,
 * Claude project scope, tool type/enum tables, and STACK.md pin checks
 * against repo manifests (offline; no registries).
 * scripts/check-docs.mjs runs this file before the README gate.
 */
import { readFileSync } from "node:fs";
import {
  MCP_LIST_LAUNCHES,
  checkPlaceholders,
  classify,
  collectShell,
  documentedTools,
  expectedType,
  fences,
  helpMentions,
  takeFailures,
} from "./check-docs.mjs";
import {
  checkStackPins,
  firstSemver,
  markdownTables,
  mentionsName,
  readStackManifests,
} from "./check-stack.mjs";

const failures = [];
function assert(cond, msg) {
  if (!cond) failures.push(msg);
}

takeFailures();
collectShell("```\necho hidden-from-the-gate\n```\n", "fixture.md");
assert(
  takeFailures().some((m) => /untagged/.test(m)),
  "untagged ``` fence must fail the gate",
);

takeFailures();
collectShell("~~~\necho hidden-from-the-gate\n~~~\n", "fixture.md");
assert(
  takeFailures().some((m) => /untagged/.test(m)),
  "untagged ~~~ fence must fail the gate",
);

const tildeBash = fences("~~~bash\nmake verify\n~~~\n");
assert(
  tildeBash.length === 1 && tildeBash[0].info === "bash" && tildeBash[0].body.includes("make verify"),
  `~~~bash fence was not parsed (${JSON.stringify(tildeBash)})`,
);
const tildeJson = fences('~~~json\n{"ok":true}\n~~~\n');
assert(
  tildeJson.length === 1 && tildeJson[0].info === "json",
  `~~~json fence was not parsed (${JSON.stringify(tildeJson)})`,
);
const tickBash = fences("```bash\nmake verify\n```\n");
assert(
  tickBash.length === 1 && tickBash[0].info === "bash" && tickBash[0].body.includes("make verify"),
  "```bash fence regressed",
);

assert(helpMentions("see hostname later", "host") === false, "`host` matched inside hostname");
assert(helpMentions("flag --port-extra here", "--port") === false, "`--port` matched `--port-extra`");
assert(helpMentions("  host [--game ttt]", "host") === true, "`host` word was missed");
assert(helpMentions("  --port P", "--port") === true, "`--port` flag was missed");

const launches = MCP_LIST_LAUNCHES.map((launch) => launch.args.join(" "));
assert(
  launches.some((args) => args.includes("packages/bet-mcp/src/index.ts")),
  "tools/list must launch packages/bet-mcp/src/index.ts",
);
assert(
  launches.some((args) => args.includes("packages/bet-mcp/bin/bet-mcp.mjs")),
  "tools/list must launch packages/bet-mcp/bin/bet-mcp.mjs",
);

const gate = readFileSync(new URL("./check-docs.mjs", import.meta.url), "utf8");
assert(
  !/existsSync\(\s*pathToFileURL\(/.test(gate),
  "existsSync(pathToFileURL(...)) does not check a filesystem path string",
);

const root = readFileSync(new URL("../README.md", import.meta.url), "utf8");
const mcp = readFileSync(new URL("../packages/bet-mcp/README.md", import.meta.url), "utf8");
takeFailures();
checkPlaceholders(root, "README.md");
checkPlaceholders(mcp, "packages/bet-mcp/README.md");
const placeholders = takeFailures();
assert(placeholders.length === 0, `path placeholder: ${placeholders.join(" | ")}`);

takeFailures();
const commands = [...collectShell(root, "README.md"), ...collectShell(mcp, "packages/bet-mcp/README.md")];
const parsed = takeFailures();
assert(parsed.length === 0, `fence parse: ${parsed.join(" | ")}`);
for (const cmd of commands) classify(cmd);
const projectAdds = takeFailures().filter((m) => /scope project/.test(m));
assert(
  projectAdds.length === 0,
  `drop \`claude mcp add --scope project\` with a machine path: ${projectAdds.join(" | ")}`,
);

assert(
  expectedType({ type: "string", enum: ["ready", "end"] }) === "enum ready|end",
  "wait schema should document as enum ready|end",
);
const wait = documentedTools(mcp).get("bet_host")?.table?.find((row) => row.arg === "wait");
assert(wait?.type === "enum ready|end", `bet_host.wait type is \`${wait?.type}\` (schema enum ready|end)`);


assert(firstSemver("0.28.1 (direct). ratatui-crossterm 0.1.0 also locks 0.29.0") === "0.28.1", "firstSemver should take the first version token");
assert(mentionsName("serde / serde_json", "serde_json") === true, "mentionsName missed serde_json");
assert(mentionsName("ratatui 0.30.0", "ratatui") === true, "mentionsName missed ratatui");
assert(mentionsName("ratatui-crossterm 0.1.0", "ratatui") === false, "mentionsName matched a prefix");
assert(mentionsName("serde-wasm-bindgen 0.6.5", "wasm-bindgen") === false, "mentionsName matched a suffix");

const pinTables = markdownTables(`| Crate | Requirement | Locked |
|-------|-------------|--------|
| ratatui | 0.30.0 | 0.30.0 |
`);
assert(pinTables.length === 1 && pinTables[0].rows[0].requirement === "0.30.0", `markdownTables missed requirement (${JSON.stringify(pinTables)})`);

const baseManifests = {
  workspace: {
    members: ["crates/bet-core", "crates/bet-protocol", "crates/bet-cli", "crates/bet-wasm"],
    dependencies: { ratatui: "0.30.0", crossterm: "0.28" },
    edition: "2024",
    version: "2.0.0-alpha.0",
    resolver: "2",
  },
  crateDeps: { "crates/bet-wasm": { "wasm-bindgen": "0.2" } },
  cargoLock: { ratatui: ["0.30.0"], crossterm: ["0.28.1"], "wasm-bindgen": ["0.2.114"] },
  npmByPackage: {
    "packages/bet-ts/package.json": { devDependencies: { typescript: "^7.0.2" } },
    "packages/bet-mcp/package.json": {
      devDependencies: { typescript: "^7.0.2" },
      dependencies: { zod: "^4.0.0" },
      engines: { node: ">=22" },
    },
  },
  npmLock: { typescript: "7.0.2", zod: "4.5.4" },
  lockfileVersion: 3,
  ci: {
    actions: {
      "actions/checkout": "v4",
      "actions/setup-node": "v4",
      "dtolnay/rust-toolchain": "stable",
      "Swatinem/rust-cache": "v2",
    },
    nodeVersion: "22",
    jobs: ["mcp-typecheck", "verify-engine"],
    os: ["ubuntu-latest", "macos-latest"],
  },
  formula: { urlTag: "v1.0.0", headBranch: "main", rustBuild: true },
  tsconfigs: {},
  wasmOpt: false,
};

const goodStack = `# Stack
**Last checked: 2026-11-21.**

Pins for 2.0.0-alpha.0. Workspace members: bet-core, bet-protocol, bet-cli, bet-wasm.
Edition 2024. Resolver "2". npm lockfileVersion 3. Node 22 and bet-mcp engines.
wasm-opt off. Formula url v1.0.0, head main.

scripts/check-docs.mjs compares requirement and locked pins to repo manifests.

| Crate | Requirement | Locked | Latest stable |
|-------|-------------|--------|---------------|
| ratatui | 0.30.0 | 0.30.0 | 9.9.9 |
| crossterm | 0.28 | 0.28.1 | 0.29.0 |
| wasm-bindgen | 0.2 | 0.2.114 | 0.2.130 |

| Package | Requirement | Locked |
|---------|-------------|--------|
| typescript | ^7.0.2 | 7.0.2 |
| zod | ^4.0.0 | 4.5.4 |

| Action | Ref we use |
|--------|------------|
| actions/checkout | v4 |
| actions/setup-node | v4 |
| dtolnay/rust-toolchain | stable |
| Swatinem/rust-cache | v2 |

Jobs: mcp-typecheck, verify-engine on ubuntu-latest and macos-latest.
`;

const goodFails = checkStackPins(goodStack, baseManifests);
assert(goodFails.length === 0, `correct fixture failed: ${goodFails.join(" | ")}`);

const staleLatest = goodStack.replace("9.9.9", "0.0.1");
assert(
  checkStackPins(staleLatest, baseManifests).length === 0,
  "Latest stable is a manual column and must not fail the offline gate",
);

const wrongLocked = goodStack.replace("| 0.30.0 | 0.30.0 |", "| 0.30.0 | 0.29.9 |");
assert(
  checkStackPins(wrongLocked, baseManifests).some((m) => /locked ratatui/.test(m)),
  "wrong Cargo.lock version in STACK.md must fail",
);

const wrongReq = goodStack.replace("| ratatui | 0.30.0 |", "| ratatui | 0.29.0 |");
assert(
  checkStackPins(wrongReq, baseManifests).some((m) => /requirement for ratatui/.test(m)),
  "wrong Cargo.toml requirement in STACK.md must fail",
);

const missingRow = goodStack.replace(/\| ratatui \| 0\.30\.0 \| 0\.30\.0 \| 9\.9\.9 \|\n/, "");
assert(
  checkStackPins(missingRow, baseManifests).some((m) => /missing a row for crate ratatui/.test(m)),
  "a workspace crate missing from STACK.md must fail",
);

const wrongAction = goodStack.replace("| actions/checkout | v4 |", "| actions/checkout | v3 |");
assert(
  checkStackPins(wrongAction, baseManifests).some((m) => /actions\/checkout/.test(m)),
  "wrong CI action ref in STACK.md must fail",
);

const staleMaintenance = goodStack.replace(
  "scripts/check-docs.mjs compares requirement and locked pins to repo manifests.",
  "`scripts/verify-engine.sh` only checks that `docs/STACK.md` is non-empty and that `README.md` links to it.",
);
assert(
  checkStackPins(staleMaintenance, baseManifests).some((m) => /only checks/.test(m) || /pin gate/.test(m)),
  "stale 'only checks that STACK.md is non-empty' maintenance claim must fail",
);

const gateSrc = readFileSync(new URL("./check-docs.mjs", import.meta.url), "utf8");
assert(
  /checkStackPins\(/.test(gateSrc) && /readStackManifests\(/.test(gateSrc),
  "check-docs.mjs must run checkStackPins against repo manifests",
);

const realFails = checkStackPins(
  readFileSync(new URL("../docs/STACK.md", import.meta.url), "utf8"),
  readStackManifests(),
);
assert(realFails.length === 0, `docs/STACK.md pins drifted from the repo: ${realFails.join(" | ")}`);


if (failures.length) {
  console.error(`check-docs.test: ${failures.length} failure(s)`);
  for (const msg of failures) console.error(`- ${msg}`);
  process.exit(1);
}
console.log("check-docs.test: OK");
