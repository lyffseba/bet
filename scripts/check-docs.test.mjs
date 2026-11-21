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
  allActionRefs,
  caretSatisfies,
  checkStackPins,
  firstSemver,
  markdownTables,
  mentionsName,
  npmRequirementMatches,
  parseCi,
  parseCrateDeps,
  readStackManifests,
  wasmOptFromToml,
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


const fourTick = "````bash\necho hello\n```\nstill inside\n````\n";
const fourParsed = fences(fourTick);
assert(
  fourParsed.length === 1 &&
    fourParsed[0].info === "bash" &&
    fourParsed[0].body.includes("still inside") &&
    fourParsed[0].body.includes("```"),
  `4-backtick fence closed too early (${JSON.stringify(fourParsed)})`,
);
const threeStill = fences("```bash\nmake verify\n```\n");
assert(
  threeStill.length === 1 && threeStill[0].info === "bash" && threeStill[0].body.includes("make verify"),
  "3-backtick fence regressed after 4-backtick fix",
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
Edition 2024. npm lockfileVersion 3. Node 22 and bet-mcp engines.
wasm-opt off.

scripts/check-docs.mjs compares requirement and locked pins to repo manifests.

| Tool | Requirement | Where | Official docs | Role |
|------|-------------|-------|---------------|------|
| Node.js | 22 | actions/setup-node, bet-mcp engines | docs | typecheck |
| Resolver | "2" (explicit) | [workspace] | docs | feature unification |

| Field | Value |
|-------|--------|
| url | v1.0.0 tarball |
| head | main |

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

## CI

Two jobs:

- \`mcp-typecheck\` on \`ubuntu-latest\`
- \`verify-engine\` on \`ubuntu-latest\` and \`macos-latest\`

Windows is omitted.
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


assert(npmRequirementMatches("^7.0.2", ["^7.0.2"]) === true, "exact npm requirement should match");
assert(npmRequirementMatches("^7.0.20", ["^7.0.2"]) === false, "^7.0.20 must not match ^7.0.2");
assert(npmRequirementMatches("^4.0.0.1", ["^4.0.0"]) === false, "^4.0.0.1 must not match ^4.0.0");
assert(
  npmRequirementMatches("latest (dev + peer *)", ["latest", "*"]) === true,
  "latest (dev + peer *) should match mixed latest/* origins",
);
assert(
  npmRequirementMatches("latest (dev + peer *)", ["^7.0.2"]) === false,
  "latest wording must not satisfy a pinned origin",
);

const looseTs = goodStack.replaceAll("^7.0.2", "^7.0.20");
assert(
  checkStackPins(looseTs, baseManifests).some((m) => /typescript/.test(m) && /\^7\.0\.20/.test(m)),
  "^7.0.20 must fail against manifest ^7.0.2: " + checkStackPins(looseTs, baseManifests).join(" | "),
);
const looseZod = goodStack.replaceAll("^4.0.0", "^4.0.0.1");
assert(
  checkStackPins(looseZod, baseManifests).some((m) => /zod/.test(m) && /\^4\.0\.0\.1/.test(m)),
  "^4.0.0.1 must fail against manifest ^4.0.0",
);

const emptyOpenLock = goodStack.replace("| open | 5.1.2 | 5.3.3 |", "| open | 5.1.2 |  |");
// goodStack may not have open; add a dedicated fixture
const withOpen = goodStack.replace(
  "| wasm-bindgen | 0.2 | 0.2.114 | 0.2.130 |",
  "| wasm-bindgen | 0.2 | 0.2.114 | 0.2.130 |\n| open | 5.1.2 | 5.3.3 | 5.4.5 |",
);
const openManifests = structuredClone(baseManifests);
openManifests.workspace.dependencies.open = "5.1.2";
openManifests.cargoLock.open = ["5.3.3"];
assert(
  checkStackPins(withOpen, openManifests).length === 0,
  "open fixture should pass: " + checkStackPins(withOpen, openManifests).join(" | "),
);
assert(
  checkStackPins(withOpen.replace("| open | 5.1.2 | 5.3.3 |", "| open | 5.1.2 |  |"), openManifests).some((m) => /no Locked cell for crate open/.test(m)),
  "empty Locked cell for open must fail",
);
assert(
  checkStackPins(withOpen.replace(/\| open \| 5\.1\.2 \| 5\.3\.3 \| 5\.4\.5 \|\n/, ""), openManifests).some((m) => /missing a row for crate open|no Locked cell for crate open/.test(m)),
  "removing the open Ours-vs-latest row must fail",
);

const node24 = structuredClone(baseManifests);
node24.ci.nodeVersion = "24";
node24.npmByPackage["packages/bet-mcp/package.json"].engines.node = ">=24";
assert(
  checkStackPins(goodStack, node24).some((m) => /Node\.js requirement is 22/.test(m) && /node-version 24/.test(m)),
  "Toolchain Node.js 22 must fail when ci.yml is 24: " + checkStackPins(goodStack, node24).join(" | "),
);

const formulaDev = structuredClone(baseManifests);
formulaDev.formula.headBranch = "dev";
assert(
  checkStackPins(goodStack, formulaDev).some((m) => /Formula head is main/.test(m) && /dev/.test(m)),
  "Formula head main vs dev must fail: " + checkStackPins(goodStack, formulaDev).join(" | "),
);
const formulaUrl = structuredClone(baseManifests);
formulaUrl.formula.urlTag = "v1.0";
assert(
  checkStackPins(goodStack, formulaUrl).some((m) => /v1\.0\.0/.test(m) && /v1\.0(?!\.)/.test(m)),
  "Formula url v1.0 must not satisfy documented v1.0.0: " + checkStackPins(goodStack, formulaUrl).join(" | "),
);
const formulaRust = structuredClone(baseManifests);
formulaRust.formula.rustBuild = false;
assert(
  checkStackPins(goodStack, formulaRust).some((m) => /depends_on "rust"/.test(m)),
  "rustBuild false must fail",
);

const emptyCheckout = goodStack.replace("| actions/checkout | v4 |", "| actions/checkout |  |");
assert(
  checkStackPins(emptyCheckout, baseManifests).some((m) => /empty Ref we use for actions\/checkout/.test(m)),
  "empty checkout Ref we use must fail: " + checkStackPins(emptyCheckout, baseManifests).join(" | "),
);
const unstable = goodStack.replace("| dtolnay/rust-toolchain | stable |", "| dtolnay/rust-toolchain | unstable |");
assert(
  checkStackPins(unstable, baseManifests).some((m) => /unstable/.test(m) && /stable/.test(m)),
  "unstable must not pass as a suffix of stable: " + checkStackPins(unstable, baseManifests).join(" | "),
);
const extraAction = structuredClone(baseManifests);
extraAction.ci.actions["actions/cache"] = "v4";
assert(
  checkStackPins(goodStack, extraAction).some((m) => /actions\/cache/.test(m)),
  "a new ci.yml action with no STACK.md row must fail",
);
const mixedCheckout = structuredClone(baseManifests);
mixedCheckout.ci.actions["actions/checkout"] = ["v4", "v7"];
assert(
  checkStackPins(goodStack, mixedCheckout).some((m) => /actions\/checkout/.test(m) && /v7/.test(m)),
  "mixed checkout@v4 and @v7 must be recorded: " + checkStackPins(goodStack, mixedCheckout).join(" | "),
);
const ci = parseCi(`
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v4
`);
assert(
  JSON.stringify(allActionRefs(ci)["actions/checkout"]) === JSON.stringify(["v4", "v7"]),
  "parseCi must keep every ref for the same action, not only the last: " + JSON.stringify(ci),
);

const lintJobs = goodStack.replace("Two jobs:", "Three jobs: `lint` on ubuntu,");
assert(
  checkStackPins(lintJobs, baseManifests).some((m) => /lint/.test(m) && /job list/.test(m)),
  "a documented lint job not in ci.yml must fail: " + checkStackPins(lintJobs, baseManifests).join(" | "),
);
const windows = goodStack.replace("Windows is omitted", "Windows (`windows-latest`) is included");
assert(
  checkStackPins(windows, baseManifests).some((m) => /windows-latest/.test(m)),
  "claiming windows-latest must fail when ci.yml does not use it: " + checkStackPins(windows, baseManifests).join(" | "),
);

const resolver3 = structuredClone(baseManifests);
resolver3.workspace.resolver = "3";
assert(
  checkStackPins(goodStack, resolver3).some((m) => /Resolver requirement is "2"/.test(m) && /"3"/.test(m)),
  "resolver must use the Toolchain row, not the edition-default \"3\" prose: " + checkStackPins(goodStack, resolver3).join(" | "),
);

const future = goodStack.replace("Last checked: 2026-11-21", "Last checked: 2099-01-01");
assert(
  checkStackPins(future, baseManifests).some((m) => /2099-01-01 is in the future/.test(m)),
  "a future Last checked date must fail",
);

const xtermMd = goodStack.replace(
  "| crossterm | 0.28 | 0.28.1 | 0.29.0 |",
  "| crossterm | 0.28 | 0.28.1 (direct). ratatui-crossterm 0.1.0 also locks 0.29.0 | 0.29.0 |",
);
const xtermMan = structuredClone(baseManifests);
xtermMan.cargoLock.crossterm = ["0.28.1", "0.29.0"];
xtermMan.cargoLock["ratatui-crossterm"] = ["0.1.0"];
assert(
  checkStackPins(xtermMd, xtermMan).length === 0,
  "crossterm two-copy fixture should pass: " + checkStackPins(xtermMd, xtermMan).join(" | "),
);
const swapped = xtermMd.replace(
  "0.28.1 (direct). ratatui-crossterm 0.1.0 also locks 0.29.0",
  "0.29.0 (direct). ratatui-crossterm 0.1.0 also locks 0.28.1",
);
assert(
  checkStackPins(swapped, xtermMan).some((m) => /0\.29\.0 as \(direct\)/.test(m) && /0\.28\.1/.test(m)),
  "swapping which crossterm copy is direct must fail: " + checkStackPins(swapped, xtermMan).join(" | "),
);
assert(caretSatisfies("0.28", "0.28.1") === true, "0.28.1 should satisfy 0.28");
assert(caretSatisfies("0.28", "0.29.0") === false, "0.29.0 should not satisfy 0.28");

const targetToml = `
[dependencies]
wasm-bindgen = "0.2"

[dependencies.foo]
version = "1.2.3"

[target.'cfg(unix)'.dependencies]
nix = "0.29"

[target.'cfg(windows)'.dependencies.winapi]
version = "0.3"
`;
const parsedDeps = parseCrateDeps(targetToml).dependencies;
assert(parsedDeps["wasm-bindgen"] === "0.2", "plain [dependencies] missed wasm-bindgen");
assert(parsedDeps.foo === "1.2.3", "[dependencies.foo] must be parsed: " + JSON.stringify(parsedDeps));
assert(parsedDeps.nix === "0.29", "[target.cfg.dependencies] must be parsed: " + JSON.stringify(parsedDeps));
assert(parsedDeps.winapi === "0.3", "[target.cfg.dependencies.winapi] must be parsed: " + JSON.stringify(parsedDeps));

assert(wasmOptFromToml("wasm-opt = false\n") === false, "active wasm-opt = false");
assert(wasmOptFromToml("# wasm-opt = false\n") === null, "commented wasm-opt = false must not count as off");
assert(wasmOptFromToml("# wasm-opt = false\nwasm-opt = true\n") === true, "active line must win over a comment");
const commentedOpt = structuredClone(baseManifests);
commentedOpt.wasmOpt = null;
assert(
  checkStackPins(goodStack, commentedOpt).some((m) => /wasm-opt is off/.test(m) && /active line/.test(m)),
  "STACK.md wasm-opt off must fail when the Cargo.toml line is only a comment",
);

const manifests = readStackManifests();
assert(manifests.npmByPackage["package.json"], "readStackManifests must scan the root package.json");
const withRootDep = structuredClone(baseManifests);
withRootDep.npmByPackage["package.json"] = { dependencies: { "left-pad": "1.3.0" } };
withRootDep.npmLock["left-pad"] = "1.3.0";
assert(
  checkStackPins(goodStack, withRootDep).some((m) => /left-pad/.test(m)),
  "a root package.json dependency missing from STACK.md must fail",
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
