#!/usr/bin/env node
/**
 * Doc-claim gate for README.md and packages/bet-mcp/README.md.
 *
 * Checks prose claims, parses JSON/TOML config snippets, runs the shell
 * commands in those files that are safe in CI, and compares documented MCP
 * tool arguments to a live tools/list from the stdio server.
 *
 * Official pages this file's expectations were checked against (2026-10-22):
 * - Claude Code MCP: https://code.claude.com/docs/en/mcp
 *   and https://code.claude.com/docs/en/mcp-quickstart
 *   (.mcp.json project scope, ~/.claude.json local/user, `claude mcp add
 *   [options] <name> -- <command>`, default stdio, `${CLAUDE_PROJECT_DIR:-.}`)
 * - Codex MCP: https://learn.chatgpt.com/docs/extend/mcp
 *   and https://learn.chatgpt.com/docs/config-file/config-reference
 *   (`[mcp_servers.*]` in ~/.codex/config.toml or trusted .codex/config.toml,
 *   `codex mcp add <name> -- <command>`)
 * - OpenCode plugins (1.x, the API this repo implements):
 *   https://opencode.ai/docs/plugins/
 *   (`plugin` array, file:// URL, .opencode/plugins/, ~/.config/opencode/plugins/)
 * - pi packages: https://pi.dev/docs/latest/packages (`pi install ./local-package`)
 */
import { spawn, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const isMain = path.resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url);

/** tools/list processes. The bin wrapper is the "Other MCP hosts" entry. */
export const MCP_LIST_LAUNCHES = [
  { command: "node", args: ["--experimental-strip-types", "packages/bet-mcp/src/index.ts"] },
];

export function helpMentions(help, token) {
  return help.includes(token);
}

export function expectedType(schema) {
  if (Array.isArray(schema?.enum) && schema.enum.length > 0) return `enum ${schema.enum.join("|")}`;
  if (typeof schema?.type === "string") return schema.type;
  return "unknown";
}

function normalizeType(value) {
  return String(value ?? "").trim().replace(/`/g, "").replace(/\s+/g, " ");
}
process.env.PATH = `${process.env.HOME}/.cargo/bin:${process.env.PATH ?? ""}`;

const PATH_CLI =
  "`bet_host` and `bet_join` spawn the native `bet` binary. `make install` installs it to `~/.local/bin`; that directory has to be on `PATH`, or set `BET_BIN` to the binary.";
const PATH_WASM =
  "`bet_status` and `bet_play` are in-process WASM. They do not need the `bet` binary.";

const failures = [];
function fail(msg) {
  failures.push(msg);
}

function read(rel) {
  return readFileSync(path.join(ROOT, rel), "utf8");
}

export function fences(md) {
  const out = [];
  const re = /```([^\n`]*)\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(md))) out.push({ info: m[1].trim().split(/\s+/)[0] ?? "", body: m[2] });
  return out;
}

function stripComment(s) {
  let q = null;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (q) {
      if (c === q) q = null;
      continue;
    }
    if (c === '"' || c === "'") {
      q = c;
      continue;
    }
    if (c === "#") return s.slice(0, i).trim();
  }
  return s.trim();
}

function shellLines(body) {
  const lines = [];
  let cur = "";
  for (const raw of body.split("\n")) {
    const line = raw.trim();
    if (line.endsWith("\\")) {
      cur += `${line.slice(0, -1).trim()} `;
      continue;
    }
    cur += line;
    const joined = stripComment(cur.trim());
    cur = "";
    if (joined) lines.push(joined);
  }
  if (cur.trim()) fail(`dangling shell continuation: ${cur}`);
  return lines;
}

function subst(token) {
  return token
    .replaceAll("${CLAUDE_PROJECT_DIR:-.}", ".")
    .replaceAll("/ABS/PATH/to/bet", ROOT);
}

export function takeFailures() {
  return failures.splice(0, failures.length);
}

export function checkPlaceholders(md, label) {
  const re = /[^\s`'"]*path\/to\/bet[^\s`'"]*/g;
  for (const match of md.matchAll(re)) {
    const token = match[0].replace(/[),.;]+$/, "");
    if (!token.includes("/ABS/PATH/to/bet")) {
      fail(`${label}: use the placeholder /ABS/PATH/to/bet (found ${token})`);
    }
  }
}

function mustExist(label, abs) {
  if (!existsSync(abs)) fail(`${label} does not exist: ${abs}`);
}

function headings(md) {
  return [...md.matchAll(/^#{2,3} +(.+)$/gm)].map((m) => m[1].trim());
}

function sectionAfter(md, headingRe) {
  const re = /^### +(.+)$/gm;
  const marks = [...md.matchAll(re)];
  const hit = marks.find((m) => headingRe.test(m[1].trim()));
  if (!hit) return null;
  const start = hit.index + hit[0].length;
  const next = marks.find((m) => m.index > hit.index);
  return md.slice(start, next ? next.index : md.length);
}

function parseTable(section) {
  const raw = section.split("\n");
  const start = raw.findIndex((l) => l.trim().startsWith("|"));
  if (start < 0) return null;
  const lines = [];
  for (const line of raw.slice(start)) {
    if (!line.trim().startsWith("|")) break;
    lines.push(line);
  }
  if (lines.length < 2) return null;
  const cells = (line) =>
    line
      .trim()
      .replace(/^\|/, "")
      .replace(/\|$/, "")
      .split("|")
      .map((c) => c.trim().replace(/`/g, ""));
  const header = cells(lines[0]).map((h) => h.toLowerCase());
  const argIdx = header.indexOf("arg");
  const reqIdx = header.findIndex((h) => h.startsWith("required"));
  if (argIdx < 0 || reqIdx < 0) return null;
  const rows = [];
  for (const line of lines.slice(1)) {
    const row = cells(line);
    if (/^[-: ]+$/.test(row[0] ?? "")) continue;
    rows.push({ arg: row[argIdx], required: (row[reqIdx] ?? "").toLowerCase() });
  }
  return rows;
}

function logicalToml(body) {
  const lines = [];
  let buf = "";
  let depth = 0;
  for (const raw of body.split("\n")) {
    const code = stripComment(raw).trim();
    if (!code && depth === 0) continue;
    buf = buf ? `${buf} ${code}` : code;
    depth += (code.match(/\[/g) ?? []).length;
    depth -= (code.match(/\]/g) ?? []).length;
    if (depth <= 0) {
      if (buf.trim()) lines.push(buf.trim());
      buf = "";
      depth = 0;
    }
  }
  if (buf.trim()) throw new Error(`unclosed TOML value: ${buf}`);
  return lines;
}

function parseTomlValue(v) {
  if (v.startsWith('"') && v.endsWith('"')) return JSON.parse(v);
  if (v.startsWith("[")) {
    const inner = v.slice(1, v.lastIndexOf("]")).trim();
    if (!inner) return [];
    return inner
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean)
      .map((s) => {
        if (s.startsWith('"')) return JSON.parse(s);
        throw new Error(`unsupported TOML array item: ${s}`);
      });
  }
  throw new Error(`unsupported TOML value: ${v}`);
}

function parseToml(body) {
  const root = {};
  let current = root;
  for (const line of logicalToml(body)) {
    const header = line.match(/^\[([^\]]+)\]$/);
    if (header) {
      current = root;
      for (const part of header[1].trim().split(".")) {
        if (current[part] === undefined || typeof current[part] !== "object") current[part] = {};
        current = current[part];
      }
      continue;
    }
    const kv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.*)$/);
    if (!kv) throw new Error(`bad TOML line: ${line}`);
    current[kv[1]] = parseTomlValue(kv[2].trim());
  }
  return root;
}

function repoPathFromToken(token) {
  const replaced = subst(token);
  if (replaced.startsWith("file://")) {
    const file = fileURLToPath(replaced);
    return file.startsWith(ROOT) ? file : null;
  }
  if (replaced.startsWith("~/") || replaced.startsWith("$")) return null;
  const abs = path.isAbsolute(replaced) ? replaced : path.resolve(ROOT, replaced);
  if (!abs.startsWith(ROOT)) return null;
  if (abs === ROOT) return abs;
  return abs;
}

function assertServerArgs(label, args) {
  if (!Array.isArray(args) || args.length === 0) {
    fail(`${label}: args must be a non-empty array`);
    return;
  }
  for (const arg of args) {
    if (typeof arg !== "string") {
      fail(`${label}: arg is not a string`);
      continue;
    }
    const abs = repoPathFromToken(arg);
    if (abs && (abs.endsWith(".ts") || abs.endsWith(".mjs") || abs.endsWith(".js"))) {
      mustExist(`${label} arg`, abs);
    }
  }
}

function checkJsonFence(body, where) {
  let json;
  try {
    json = JSON.parse(body);
  } catch (e) {
    fail(`${where}: JSON does not parse: ${e instanceof Error ? e.message : e}`);
    return;
  }
  const bet = json?.mcpServers?.bet;
  if (bet) {
    if (bet.type !== undefined && bet.type !== "stdio") {
      fail(`${where}: mcpServers.bet.type must be "stdio" when set (got ${bet.type})`);
    }
    if (bet.command !== "node") fail(`${where}: mcpServers.bet.command must be "node"`);
    assertServerArgs(where, bet.args);
    const joined = (bet.args ?? []).join(" ");
    if (!joined.includes("packages/bet-mcp/")) {
      fail(`${where}: MCP args must point at packages/bet-mcp`);
    }
  }
  if (Array.isArray(json.plugin)) {
    if (json.plugins) fail(`${where}: OpenCode v1 config uses "plugin", not "plugins"`);
    for (const spec of json.plugin) {
      if (typeof spec !== "string" || !spec.startsWith("file://")) {
        fail(`${where}: plugin spec must be a file:// URL (got ${spec})`);
        continue;
      }
      const abs = repoPathFromToken(spec);
      if (!abs) {
        fail(`${where}: plugin URL is not inside the repo: ${spec}`);
        continue;
      }
      mustExist(`${where} plugin`, abs);
    }
  }
}

function checkTomlFence(body, where) {
  let doc;
  try {
    doc = parseToml(body);
  } catch (e) {
    fail(`${where}: TOML does not parse: ${e instanceof Error ? e.message : e}`);
    return;
  }
  if (doc.mcpServers) fail(`${where}: Codex key is mcp_servers, not mcpServers`);
  const bet = doc.mcp_servers?.bet;
  if (!bet) {
    fail(`${where}: missing [mcp_servers.bet]`);
    return;
  }
  if (bet.command !== "node") fail(`${where}: mcp_servers.bet.command must be "node"`);
  assertServerArgs(where, bet.args);
}

function makefileTargets(text) {
  const targets = new Map();
  let name = null;
  let recipe = [];
  const flush = () => {
    if (name) targets.set(name, recipe.join("\n"));
  };
  for (const line of text.split("\n")) {
    const header = line.match(/^([A-Za-z0-9_.-]+)\s*:/);
    if (header && !line.startsWith("\t") && !line.startsWith(" ")) {
      flush();
      name = header[1];
      recipe = [];
      continue;
    }
    if (name && (line.startsWith("\t") || line.startsWith(" "))) recipe.push(line);
    else if (line.trim() === "" || line.startsWith("#") || line.startsWith(".")) {
      /* keep collecting until the next target */
    } else if (!line.startsWith("\t")) {
      flush();
      name = null;
      recipe = [];
    }
  }
  flush();
  return targets;
}

const MAKEFILE = makefileTargets(read("Makefile"));

function checkMake(cmd) {
  const m = cmd.match(/^make ([A-Za-z0-9_-]+)$/);
  if (!m) return false;
  const target = m[1];
  const recipe = MAKEFILE.get(target);
  if (recipe === undefined) {
    fail(`Makefile has no target \`${target}\` (from \`${cmd}\`)`);
    return true;
  }
  if (target === "install" && !recipe.includes(".local/bin")) {
    fail("make install recipe must install into ~/.local/bin");
  }
  const script = recipe.match(/scripts\/[A-Za-z0-9_.-]+\.sh/);
  if (script) {
    const rel = script[0];
    mustExist(`make ${target}`, path.join(ROOT, rel));
    const syntax = spawnSync("bash", ["-n", rel], { cwd: ROOT, encoding: "utf8" });
    if (syntax.status !== 0) fail(`bash -n ${rel} failed: ${syntax.stderr}`);
  }
  if (target === "install-extension" && !recipe.includes("pi install .")) {
    fail("make install-extension must run `pi install .`");
  }
  return true;
}

const usage = { subcommands: new Set(), flags: new Set() };

function checkClaudeAdd(cmd) {
  if (/^claude mcp add --scope project\b/.test(cmd)) {
    fail(
      "`claude mcp add --scope project` writes a machine path into shared .mcp.json. Point at the ${CLAUDE_PROJECT_DIR:-.} JSON block instead.",
    );
    return;
  }
  const m = cmd.match(
    /^claude mcp add(?: --scope user)? bet -- node --experimental-strip-types (\S+)$/,
  );
  if (!m) {
    fail(
      `claude mcp add must match \`claude mcp add [--scope user] bet -- node --experimental-strip-types <entry>\`: ${cmd}`,
    );
    return;
  }
  const abs = repoPathFromToken(m[1]);
  if (!abs) fail(`claude mcp add path is not in the repo: ${m[1]}`);
  else mustExist("claude mcp add", abs);
}

function checkCodexAdd(cmd) {
  const m = cmd.match(/^codex mcp add bet -- node --experimental-strip-types (\S+)$/);
  if (!m) {
    fail(`codex mcp add must match \`codex mcp add bet -- node --experimental-strip-types <entry>\`: ${cmd}`);
    return;
  }
  const abs = repoPathFromToken(m[1]);
  if (!abs) fail(`codex mcp add path is not in the repo: ${m[1]}`);
  else mustExist("codex mcp add", abs);
}

export function classify(cmd) {
  if (cmd === "cargo build -p bet-cli --release" || cmd === "./target/release/bet --help") return "run";
  if (checkMake(cmd)) return "make";
  const bet = cmd.match(/^bet (host|join|balance)\b(.*)$/);
  if (bet) {
    usage.subcommands.add(bet[1]);
    for (const flag of bet[2].matchAll(/(--[a-z0-9-]+)/g)) usage.flags.add(flag[1]);
    return "usage";
  }
  if (cmd === "pi install .") {
    const pkg = JSON.parse(read("package.json"));
    if (!pkg.keywords?.includes("pi-package")) fail("root package.json must declare the pi-package keyword");
    const ext = pkg.pi?.extensions;
    if (!Array.isArray(ext) || ext.length === 0) fail("root package.json pi.extensions is empty");
    for (const rel of ext ?? []) {
      const abs = path.resolve(ROOT, rel);
      mustExist("pi extension", abs);
      if (!read(path.relative(ROOT, abs)).includes('registerCommand("b$t"')) {
        fail(`${rel} does not register /b$t`);
      }
    }
    return "pi";
  }
  if (cmd.startsWith("claude mcp add ")) {
    checkClaudeAdd(cmd);
    return "claude";
  }
  if (cmd === "claude mcp list" || cmd === "codex mcp list") return "list";
  if (cmd.startsWith("codex mcp add ")) {
    checkCodexAdd(cmd);
    return "codex";
  }
  if (cmd === "npm install") {
    mustExist("package.json", path.join(ROOT, "package.json"));
    return "npm";
  }
  if (cmd.startsWith("export PATH=") || cmd.startsWith("export BET_BIN=")) return "export";
  const nodeEntry = cmd.match(/^node (?:--experimental-strip-types )?(\S+)$/);
  if (nodeEntry) {
    const abs = repoPathFromToken(nodeEntry[1]);
    if (!abs) fail(`node entry is not in the repo: ${cmd}`);
    else mustExist(cmd, abs);
    return "node";
  }
  if (cmd.startsWith("npx @modelcontextprotocol/inspector ")) {
    const entry = cmd.split(/\s+/).find((p) => p.endsWith(".ts") || p.endsWith(".mjs"));
    if (!entry) fail(`inspector command does not name a server file: ${cmd}`);
    else mustExist("inspector", path.join(ROOT, entry));
    return "inspector";
  }
  fail(`unclassified shell command in a doc fence (refusing to run it): ${cmd}`);
  return "unknown";
}

function runSafe(commands) {
  for (const cmd of commands) {
    const bin = path.join(ROOT, "target/release/bet");
    if (cmd === "./target/release/bet --help" && !existsSync(bin)) {
      fail("refusing bet --help before cargo build has produced target/release/bet");
      continue;
    }
    const result = spawnSync("bash", ["-lc", cmd], {
      cwd: ROOT,
      encoding: "utf8",
      timeout: cmd.startsWith("cargo ") ? 600_000 : 30_000,
      env: process.env,
    });
    if (result.error) {
      fail(`\`${cmd}\` failed to spawn: ${result.error.message}`);
      continue;
    }
    if (result.status !== 0) {
      const tail = `${result.stdout ?? ""}\n${result.stderr ?? ""}`.trim().split("\n").slice(-30).join("\n");
      fail(`\`${cmd}\` exited ${result.status}\n${tail}`);
      continue;
    }
    if (cmd === "./target/release/bet --help") {
      const help = result.stdout ?? "";
      for (const sub of usage.subcommands) {
        if (!help.includes(sub)) fail(`bet --help does not mention documented subcommand \`${sub}\``);
      }
      for (const flag of usage.flags) {
        if (!help.includes(flag)) fail(`bet --help does not mention documented flag \`${flag}\``);
      }
      for (const env of ["BET_CONFIG_DIR", "BET_PLAYER", "BET_MOVES"]) {
        if (!help.includes(env)) fail(`bet --help does not mention ${env}`);
      }
      if (!/virtual/i.test(help)) fail("bet --help does not mention virtual points");
    }
  }
}

function checkSharedClaims(md, label) {
  if (/\bupcoming\b/i.test(md)) fail(`${label} contains stale "upcoming"`);
  if (!md.includes(PATH_CLI)) fail(`${label} is missing the PATH claim for bet_host/bet_join`);
  if (!md.includes(PATH_WASM)) fail(`${label} is missing the WASM claim for bet_status/bet_play`);
  if (!/virtual points/i.test(md)) fail(`${label} must say stakes are virtual points`);
}

function checkRoot(md) {
  checkSharedClaims(md, "README.md");
  const heads = headings(md);
  const need = [/CLI/i, /^pi$/i, /OpenCode/i, /Claude Code/i, /Codex/i, /MCP/i];
  for (const re of need) {
    if (!heads.some((h) => re.test(h))) fail(`README.md is missing a first-run heading matching ${re}`);
  }
  for (const rel of ["docs/STACK.md", "docs/QUALITY.md", "docs/EPICS.md", "packages/bet-mcp/README.md"]) {
    if (!md.includes(rel)) fail(`README.md must link ${rel}`);
    mustExist(rel, path.join(ROOT, rel));
  }
  for (const token of ["BET_CONFIG_DIR", "BET_PLAYER", "BET_MOVES", "/b$t", ".opencode/plugins/", "~/.config/opencode/plugins/"]) {
    if (!md.includes(token)) fail(`README.md must mention ${token}`);
  }
  const pluginFence = fences(md).find((f) => f.info === "json" && f.body.includes('"plugin"'));
  if (!pluginFence) fail("README.md needs a JSON fence with an OpenCode plugin array");
  const opencodeSrc = read("packages/bet-opencode/src/index.ts");
  if (!opencodeSrc.includes("@opencode-ai/plugin")) {
    fail("OpenCode plugin no longer imports @opencode-ai/plugin; README claim is stale");
  }
  if (/\bplugins\b/.test(md.match(/"plugin"[\s\S]{0,80}/)?.[0] ?? "")) {
    fail("README.md must not switch the OpenCode field to plugins");
  }
}

function checkMcp(md) {
  checkSharedClaims(md, "packages/bet-mcp/README.md");
  if (/\bif available\b/i.test(md)) {
    fail('packages/bet-mcp/README.md still hedges Codex with "if available"');
  }
  for (const token of [
    "--scope project",
    "--scope user",
    ".mcp.json",
    "~/.claude.json",
    "claude mcp list",
    "~/.codex/config.toml",
    ".codex/config.toml",
    "codex mcp list",
    "trusted",
    "/mcp",
    "Node ≥ 22",
    "${CLAUDE_PROJECT_DIR:-.}",
  ]) {
    if (!md.includes(token)) fail(`packages/bet-mcp/README.md must mention ${token}`);
  }
  const tools = ["bet_status", "bet_play", "bet_host", "bet_join"];
  for (const name of tools) {
    if (!sectionAfter(md, new RegExp(`^\`${name}\`$`))) {
      fail(`packages/bet-mcp/README.md is missing ### \`${name}\``);
    }
  }
}

export function documentedTools(md) {
  const out = new Map();
  for (const name of ["bet_status", "bet_play", "bet_host", "bet_join"]) {
    const section = sectionAfter(md, new RegExp(`^\`${name}\`$`));
    if (!section) continue;
    const table = parseTable(section);
    out.set(name, { section, table });
  }
  return out;
}

function compareTools(documented, listed) {
  const docNames = [...documented.keys()].sort();
  const liveNames = listed.map((t) => t.name).sort();
  if (docNames.join() !== liveNames.join()) {
    fail(`documented tools [${docNames}] != tools/list [${liveNames}]`);
  }
  for (const tool of listed) {
    const doc = documented.get(tool.name);
    if (!doc) continue;
    const props = Object.keys(tool.inputSchema?.properties ?? {}).sort();
    const required = new Set(tool.inputSchema?.required ?? []);
    if (!doc.table) {
      if (props.length !== 0) fail(`${tool.name} has args ${props.join(", ")} but the docs have no arg table`);
      if (!/no arguments/i.test(doc.section)) fail(`${tool.name} docs must say it takes no arguments`);
      continue;
    }
    const args = doc.table.map((r) => r.arg).sort();
    if (args.join() !== props.join()) {
      fail(`${tool.name} doc args [${args}] != schema [${props}]`);
    }
    for (const row of doc.table) {
      const marked = row.required.startsWith("yes") && !row.required.startsWith("yes*");
      const schemaRequired = required.has(row.arg);
      if (schemaRequired && !row.required.startsWith("yes")) {
        fail(`${tool.name}.${row.arg} is required by the server but the docs do not say yes`);
      }
      if (!schemaRequired && marked) {
        fail(`${tool.name}.${row.arg} is marked required in the docs but not in the schema`);
      }
      const prop = tool.inputSchema?.properties?.[row.arg] ?? {};
      const expected = expectedType(prop);
      const documentedType = normalizeType(row.type);
      if (documentedType !== expected) {
        fail(`${tool.name}.${row.arg} type \`${documentedType}\` != schema \`${expected}\``);
      }
    }
  }
}

function listTools(launch) {
  return new Promise((resolve, reject) => {
    const child = spawn(launch.command, launch.args, { cwd: ROOT, stdio: ["pipe", "pipe", "pipe"] });
    let stderr = "";
    let buf = "";
    const messages = [];
    child.stderr.on("data", (d) => {
      stderr += d.toString();
    });
    child.stdout.on("data", (d) => {
      buf += d.toString();
      let i;
      while ((i = buf.indexOf("\n")) >= 0) {
        const line = buf.slice(0, i).trim();
        buf = buf.slice(i + 1);
        if (!line) continue;
        try {
          messages.push(JSON.parse(line));
        } catch {
          reject(new Error(`non-JSON stdout from bet-mcp: ${line}\n${stderr}`));
        }
      }
    });
    const send = (obj) => child.stdin.write(`${JSON.stringify(obj)}\n`);
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`tools/list timed out\nstderr:\n${stderr}`));
    }, 20_000);
    child.on("error", (err) => {
      clearTimeout(timer);
      reject(err);
    });
    setTimeout(() => {
      send({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          protocolVersion: "2025-11-25",
          capabilities: {},
          clientInfo: { name: "docs-check", version: "0.0.0" },
        },
      });
    }, 50);
    const poll = setInterval(() => {
      const init = messages.find((m) => m.id === 1);
      if (!init) return;
      clearInterval(poll);
      if (init.error) {
        clearTimeout(timer);
        child.kill();
        reject(new Error(`initialize failed: ${JSON.stringify(init.error)}`));
        return;
      }
      send({ jsonrpc: "2.0", method: "notifications/initialized" });
      send({ jsonrpc: "2.0", id: 2, method: "tools/list", params: {} });
    }, 30);
    const pollList = setInterval(() => {
      const list = messages.find((m) => m.id === 2);
      if (!list) return;
      clearInterval(pollList);
      clearTimeout(timer);
      child.kill();
      if (list.error) {
        reject(new Error(`tools/list failed: ${JSON.stringify(list.error)}`));
        return;
      }
      resolve(list.result?.tools ?? []);
    }, 30);
  });
}

export function collectShell(md, rel) {
  const commands = [];
  for (const fence of fences(md)) {
    if (fence.info === "json") checkJsonFence(fence.body, rel);
    else if (fence.info === "toml") checkTomlFence(fence.body, rel);
    else if (fence.info === "bash" || fence.info === "sh" || fence.info === "shell") {
      for (const line of shellLines(fence.body)) commands.push(line);
    } else if (fence.info === "text") {
      /* directory sketch, not a command */
    } else if (fence.info) {
      fail(`${rel}: unclassified fence language \`${fence.info}\``);
    }
  }
  return commands;
}

async function main() {
const rootMd = read("README.md");
const mcpMd = read("packages/bet-mcp/README.md");
checkRoot(rootMd);
checkMcp(mcpMd);
checkPlaceholders(rootMd, "README.md");
checkPlaceholders(mcpMd, "packages/bet-mcp/README.md");
const commands = [...collectShell(rootMd, "README.md"), ...collectShell(mcpMd, "packages/bet-mcp/README.md")];
const kinds = commands.map((cmd) => classify(cmd));
const runnable = commands.filter((_, i) => kinds[i] === "run");

for (const required of [
  "cargo build -p bet-cli --release",
  "./target/release/bet --help",
]) {
  if (!runnable.includes(required)) fail(`README must show \`${required}\` in a bash fence`);
}
for (const required of ["make verify", "make install", "make install-extension", "pi install .", "claude mcp list", "codex mcp list"]) {
  if (!commands.includes(required)) fail(`docs must show \`${required}\``);
}
if (!read("scripts/verify-engine.sh").includes("node scripts/check-docs.mjs")) {
  fail("scripts/verify-engine.sh must run this check so CI picks it up");
}
if (!read(".github/workflows/ci.yml").includes("bash scripts/verify-engine.sh")) {
  fail("ci.yml must keep calling scripts/verify-engine.sh");
}

const documented = documentedTools(mcpMd);
const structureOk = failures.length === 0;

if (!structureOk) {
  console.error(`check-docs: ${failures.length} failure(s) before execution`);
  for (const msg of failures) console.error(`- ${msg}`);
  process.exit(1);
}

runSafe(runnable);
try {
  for (const launch of MCP_LIST_LAUNCHES) {
    const tools = await listTools(launch);
    compareTools(documented, tools);
  }
} catch (e) {
  fail(e instanceof Error ? e.message : String(e));
}

if (!existsSync(pathToFileURL(path.join(ROOT, "packages/bet-mcp/src/index.ts")))) {
  fail("internal: entry URL check broke");
}

if (failures.length) {
  console.error(`check-docs: ${failures.length} failure(s)`);
  for (const msg of failures) console.error(`- ${msg}`);
  process.exit(1);
}

console.log("check-docs: OK");
}

if (isMain) {
  const unit = spawnSync(process.execPath, [path.join(ROOT, "scripts/check-docs.test.mjs")], {
    cwd: ROOT,
    encoding: "utf8",
  });
  if (unit.stdout) process.stdout.write(unit.stdout);
  if (unit.stderr) process.stderr.write(unit.stderr);
  if (unit.status !== 0) process.exit(unit.status ?? 1);
  await main();
}
