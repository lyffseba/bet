/**
 * Failing cases for the doc gate. Run by scripts/check-docs.mjs.
 * Each assertion is the behavior BetQA asked for; this file stays red until
 * the gate and the READMEs implement it.
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

if (failures.length) {
  console.error(`check-docs.test: ${failures.length} failure(s)`);
  for (const msg of failures) console.error(`- ${msg}`);
  process.exit(1);
}
console.log("check-docs.test: OK");
