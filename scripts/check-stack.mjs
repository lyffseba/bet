/**
 * Offline STACK.md pin gate.
 *
 * Compares requirement and locked columns in docs/STACK.md to Cargo.toml,
 * Cargo.lock, package.json, package-lock.json, .github/workflows/ci.yml,
 * and Formula/bet.rb. "Latest stable" and channel snapshots (rustc patch,
 * Node Current/LTS numbers, action tag-tip dates) are manual refreshes and
 * are not checked here — never call crates.io, npm, or other networks.
 *
 * Sources last used to refresh the manual column (2026-11-21) are listed
 * in docs/STACK.md, not consulted at gate time.
 */
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const STACK_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const PATH_CRATES = new Set(["bet-core", "bet-protocol", "bet-cli", "bet-wasm"]);

export function splitRow(line) {
  const body = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  const out = [];
  let cur = "";
  for (let i = 0; i < body.length; i++) {
    if (body[i] === "\\" && body[i + 1] === "|") {
      cur += "|";
      i++;
      continue;
    }
    if (body[i] === "|") {
      out.push(cur.trim().replace(/`/g, ""));
      cur = "";
      continue;
    }
    cur += body[i];
  }
  out.push(cur.trim().replace(/`/g, ""));
  return out;
}

export function markdownTables(md) {
  const lines = md.split("\n");
  const tables = [];
  for (let i = 0; i < lines.length; i++) {
    if (!lines[i].trim().startsWith("|")) continue;
    const block = [];
    let j = i;
    while (j < lines.length && lines[j].trim().startsWith("|")) {
      block.push(lines[j]);
      j++;
    }
    if (block.length >= 2) {
      const header = splitRow(block[0]);
      const rows = [];
      for (const line of block.slice(1)) {
        const cells = splitRow(line);
        if (cells.every((c) => /^[-: ]+$/.test(c))) continue;
        const row = {};
        header.forEach((h, idx) => {
          row[h.toLowerCase()] = cells[idx] ?? "";
        });
        rows.push(row);
      }
      tables.push({ header, rows, startLine: i + 1 });
    }
    i = j - 1;
  }
  return tables;
}

export function firstSemver(text) {
  const m = String(text ?? "").match(/\d+(?:\.\d+){0,3}/);
  return m ? m[0] : "";
}

function escapeRe(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function mentionsName(text, name) {
  const re = new RegExp(`(^|[^A-Za-z0-9_@-])${escapeRe(name)}(?![A-Za-z0-9_-])`);
  return re.test(String(text ?? ""));
}

function rowLabel(row) {
  const keys = Object.keys(row);
  return row[keys[0]] ?? "";
}

function rowsMentioning(tables, name) {
  const hits = [];
  for (const table of tables) {
    for (const row of table.rows) {
      if (mentionsName(rowLabel(row), name)) hits.push({ row, table });
    }
  }
  return hits;
}

function section(text, header) {
  const re = new RegExp(`^\\[${escapeRe(header)}\\]\\s*$`, "m");
  const m = re.exec(text);
  if (!m) return "";
  const start = m.index + m[0].length;
  const rest = text.slice(start);
  const next = rest.search(/^\[/m);
  return (next < 0 ? rest : rest.slice(0, next)).trim();
}

function tomlStringValue(raw) {
  const t = raw.trim();
  const q = t.match(/^"([^"]*)"/);
  return q ? q[1] : t.replace(/,#.*$/, "").trim();
}

function parseTomlAssignment(line) {
  const m = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.+)$/);
  if (!m) return null;
  const key = m[1];
  const raw = m[2].trim();
  if (raw.startsWith("{")) {
    const version = raw.match(/version\s*=\s*"([^"]+)"/)?.[1];
    const pathValue = raw.match(/path\s*=\s*"([^"]+)"/)?.[1];
    return { key, version, path: pathValue };
  }
  if (raw.startsWith('"')) return { key, version: tomlStringValue(raw) };
  return { key, version: tomlStringValue(raw) };
}

function parseWorkspaceCargo(toml) {
  const ws = section(toml, "workspace");
  const pkg = section(toml, "workspace.package");
  const depsBlock = section(toml, "workspace.dependencies");
  const members = [...ws.matchAll(/"([^"]+)"/g)]
    .map((m) => m[1])
    .filter((s) => s.startsWith("crates/"));
  const dependencies = {};
  for (const line of depsBlock.split("\n")) {
    const item = parseTomlAssignment(line.trim());
    if (!item) continue;
    if (item.path && !item.version) continue;
    if (item.version) dependencies[item.key] = item.version;
  }
  const edition = parseTomlAssignment(pkg.split("\n").find((l) => l.startsWith("edition")) ?? "")?.version;
  const version = parseTomlAssignment(pkg.split("\n").find((l) => l.startsWith("version")) ?? "")?.version;
  const resolver = parseTomlAssignment(ws.split("\n").find((l) => l.trim().startsWith("resolver")) ?? "")?.version;
  return { members, dependencies, edition, version, resolver };
}

function parseCrateDeps(toml) {
  const deps = section(toml, "dependencies");
  const out = {};
  for (const line of deps.split("\n")) {
    const item = parseTomlAssignment(line.trim());
    if (!item) continue;
    if (item.path && !item.version) continue;
    if (item.version) out[item.key] = item.version;
  }
  const wasmOpt = /wasm-opt\s*=\s*false/.test(toml) ? false : /wasm-opt\s*=\s*true/.test(toml) ? true : null;
  return { dependencies: out, wasmOpt };
}

export function parseCargoLock(text) {
  const versions = {};
  for (const block of text.split("\n[[package]]\n")) {
    const name = block.match(/^name = "([^"]+)"/m)?.[1];
    const version = block.match(/^version = "([^"]+)"/m)?.[1];
    if (!name || !version) continue;
    versions[name] ??= [];
    versions[name].push(version);
  }
  return versions;
}

export function parseCi(yml) {
  const actions = {};
  for (const m of yml.matchAll(/uses:\s*(\S+)@(\S+)/g)) {
    actions[m[1]] = m[2];
  }
  const nodeVersion = yml.match(/node-version:\s*"?([^"\s]+)"?/)?.[1] ?? "";
  const jobsBlock = yml.split(/^jobs:\s*$/m)[1] ?? "";
  const jobs = [...jobsBlock.matchAll(/^  ([A-Za-z0-9_-]+):\s*$/gm)].map((m) => m[1]);
  const os = (yml.match(/os:\s*\[([^\]]+)\]/)?.[1] ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  return { actions, nodeVersion, jobs, os };
}

export function parseFormula(rb) {
  const urlTag = rb.match(/archive\/refs\/tags\/(v[\d.]+)\.tar\.gz/)?.[1] ?? "";
  const headBranch = rb.match(/head\s+"[^"]+"\s*,\s*branch:\s*"([^"]+)"/)?.[1] ?? "";
  const rustBuild = /depends_on\s+"rust"\s*=>\s*:build/.test(rb);
  return { urlTag, headBranch, rustBuild };
}

function collectNpm(root) {
  const reqsByPkg = {};
  const packagesDir = path.join(root, "packages");
  const dirs = readdirSync(packagesDir, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name);
  for (const name of dirs) {
    const rel = `packages/${name}/package.json`;
    const abs = path.join(root, rel);
    try {
      reqsByPkg[rel] = JSON.parse(readFileSync(abs, "utf8"));
    } catch {
      // skip missing package.json
    }
  }
  return reqsByPkg;
}

export function readStackManifests(root = STACK_ROOT) {
  const cargoToml = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const workspace = parseWorkspaceCargo(cargoToml);
  const crateDeps = {};
  let wasmOpt = null;
  for (const member of workspace.members) {
    const toml = readFileSync(path.join(root, member, "Cargo.toml"), "utf8");
    const parsed = parseCrateDeps(toml);
    crateDeps[member] = parsed.dependencies;
    if (parsed.wasmOpt !== null) wasmOpt = parsed.wasmOpt;
  }
  const cargoLock = parseCargoLock(readFileSync(path.join(root, "Cargo.lock"), "utf8"));
  const npmByPackage = collectNpm(root);
  const lock = JSON.parse(readFileSync(path.join(root, "package-lock.json"), "utf8"));
  const npmLock = {};
  for (const [key, meta] of Object.entries(lock.packages ?? {})) {
    if (!key.startsWith("node_modules/") || !meta?.version) continue;
    const name = key.slice("node_modules/".length);
    if (name.includes("/node_modules/")) continue;
    npmLock[name] = meta.version;
  }
  const tsconfigs = {};
  for (const rel of Object.keys(npmByPackage)) {
    const dir = path.dirname(rel);
    try {
      tsconfigs[`${dir}/tsconfig.json`] = JSON.parse(
        readFileSync(path.join(root, dir, "tsconfig.json"), "utf8"),
      );
    } catch {
      // optional
    }
  }
  return {
    workspace,
    crateDeps,
    cargoLock,
    npmByPackage,
    npmLock,
    lockfileVersion: lock.lockfileVersion,
    ci: parseCi(readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8")),
    formula: parseFormula(readFileSync(path.join(root, "Formula/bet.rb"), "utf8")),
    tsconfigs,
    wasmOpt,
  };
}

function directRustReqs(manifests) {
  const reqs = { ...manifests.workspace.dependencies };
  for (const deps of Object.values(manifests.crateDeps)) {
    for (const [name, version] of Object.entries(deps)) {
      if (PATH_CRATES.has(name)) continue;
      reqs[name] ??= version;
    }
  }
  return reqs;
}

function directNpmReqs(manifests) {
  const reqs = {};
  const origins = {};
  for (const [rel, pkg] of Object.entries(manifests.npmByPackage)) {
    for (const field of ["dependencies", "devDependencies", "peerDependencies"]) {
      for (const [name, version] of Object.entries(pkg[field] ?? {})) {
        if (name.startsWith("@lyffseba/")) continue;
        origins[name] ??= [];
        origins[name].push({ rel, field, version });
        reqs[name] ??= version;
      }
    }
  }
  return { reqs, origins };
}

function requirementOf(hit, name) {
  if (hit.row.requirement) return hit.row.requirement.trim();
  const label = rowLabel(hit.row);
  // "ratatui 0.30.0" in a Crate column
  const re = new RegExp(`${escapeRe(name)}\\s+([^\\s,;/]+)`);
  const m = label.match(re);
  return m ? m[1] : "";
}

function sameReq(doc, manifest) {
  const a = String(doc ?? "").trim().replace(/^"|"$/g, "");
  const b = String(manifest ?? "").trim().replace(/^"|"$/g, "");
  return a === b;
}

export function checkStackPins(md, manifests) {
  const failures = [];
  const fail = (msg) => failures.push(msg);
  const tables = markdownTables(md);

  if (!/Last checked:\s*20\d{2}-\d{2}-\d{2}/i.test(md)) {
    fail("docs/STACK.md must have a Last checked: YYYY-MM-DD date");
  }

  if (/only checks that `?docs\/STACK\.md`? is non-empty/i.test(md)) {
    fail(
      "docs/STACK.md maintenance still says verify-engine only checks STACK.md is non-empty; the pin gate also runs now",
    );
  }
  if (!/check-docs/.test(md) || !/manifest/i.test(md)) {
    fail(
      "docs/STACK.md maintenance must say check-docs compares requirement/locked pins to repo manifests",
    );
  }

  const { workspace } = manifests;
  if (workspace.version && !md.includes(workspace.version)) {
    fail(`docs/STACK.md must mention workspace version ${workspace.version}`);
  }
  if (workspace.edition && !mentionsName(md, workspace.edition) && !md.includes(`edition | ${workspace.edition}`.replace("|", ""))) {
    // edition is a short token like 2024; require it next to "edition"
    if (!new RegExp(`edition[^\\n]{0,80}${escapeRe(workspace.edition)}`, "i").test(md)) {
      fail(`docs/STACK.md must record edition ${workspace.edition}`);
    }
  }
  if (workspace.resolver && !md.includes(`"${workspace.resolver}"`)) {
    fail(`docs/STACK.md must record Cargo resolver "${workspace.resolver}"`);
  }
  for (const member of workspace.members) {
    const crate = member.replace(/^crates\//, "");
    if (!md.includes(crate)) fail(`docs/STACK.md must list workspace member ${crate}`);
  }

  if (manifests.lockfileVersion != null) {
    const lv = String(manifests.lockfileVersion);
    const documented =
      md.includes("lockfileVersion `" + lv + "`") ||
      new RegExp("lockfileVersion\\s+[\\`\"]?" + lv + "[\\`\"]?").test(md);
    if (!documented) {
      fail(`docs/STACK.md must record npm lockfileVersion ${lv}`);
    }
  }

  const rustReqs = directRustReqs(manifests);
  for (const [name, req] of Object.entries(rustReqs)) {
    const hits = rowsMentioning(tables, name);
    if (!hits.length) {
      fail(`docs/STACK.md is missing a row for crate ${name}`);
      continue;
    }
    const withReq = hits.filter((h) => requirementOf(h, name));
    if (!withReq.length) {
      fail(`docs/STACK.md has no requirement for crate ${name} (manifest ${req})`);
    }
    for (const hit of withReq) {
      const docReq = requirementOf(hit, name);
      if (!sameReq(docReq, req)) {
        fail(`docs/STACK.md requirement for ${name} is ${docReq}, manifest has ${req}`);
      }
    }
    const lockVers = manifests.cargoLock[name] ?? [];
    const lockedHits = hits.filter((h) => h.row.locked);
    for (const hit of lockedHits) {
      if (lockVers.length === 1) {
        const doc = firstSemver(hit.row.locked);
        if (doc !== lockVers[0]) {
          fail(`docs/STACK.md locked ${name} is ${doc || hit.row.locked}, Cargo.lock has ${lockVers[0]}`);
        }
      } else if (lockVers.length > 1) {
        for (const v of lockVers) {
          if (!hit.row.locked.includes(v)) {
            fail(`docs/STACK.md locked ${name} must mention every Cargo.lock copy (${lockVers.join(", ")}); missing ${v}`);
          }
        }
      }
    }
  }

  const { reqs: npmReqs, origins } = directNpmReqs(manifests);
  for (const [name, req] of Object.entries(npmReqs)) {
    const hits = rowsMentioning(tables, name);
    if (!hits.length) {
      fail(`docs/STACK.md is missing a row for npm package ${name}`);
      continue;
    }
    const withReq = hits.filter((h) => h.row.requirement);
    if (!withReq.length) {
      fail(`docs/STACK.md has no requirement for npm package ${name} (manifest ${req})`);
    }
    const uniqueReqs = [...new Set(origins[name].map((o) => o.version))];
    for (const hit of withReq) {
      const docReq = hit.row.requirement.trim();
      // `latest` (dev + peer `*`) style: accept if every origin version is mentioned or equals docReq
      const ok = uniqueReqs.every(
        (v) => sameReq(docReq, v) || docReq.includes(v) || (v === "*" && /peer/.test(docReq)),
      );
      if (!ok && !uniqueReqs.some((v) => sameReq(docReq, v))) {
        // For mixed latest + *, the documented requirement should include `latest` and `*`.
        const wantsLatest = uniqueReqs.includes("latest");
        if (wantsLatest && /\blatest\b/.test(docReq) && uniqueReqs.every((v) => v === "latest" || v === "*" || docReq.includes(v))) {
          continue;
        }
        fail(`docs/STACK.md requirement for ${name} is ${docReq}, package.json has ${uniqueReqs.join(", ")}`);
      }
    }
    const lockVer = manifests.npmLock[name];
    for (const hit of hits.filter((h) => h.row.locked)) {
      if (!lockVer) continue;
      const doc = firstSemver(hit.row.locked);
      if (doc !== lockVer) {
        fail(`docs/STACK.md locked ${name} is ${doc || hit.row.locked}, package-lock.json has ${lockVer}`);
      }
    }
  }

  // typescript appears in every workspace package
  for (const [rel, pkg] of Object.entries(manifests.npmByPackage)) {
    const ts = pkg.devDependencies?.typescript;
    if (!ts) fail(`${rel} is missing devDependencies.typescript (STACK.md says every package has it)`);
  }

  const tsNeed = { strict: true, module: "NodeNext", moduleResolution: "NodeNext", target: "ES2022" };
  if (/Shared options:\s*`strict`/.test(md) || /`strict`.+`NodeNext`.+`ES2022`/s.test(md) || /module`\/`moduleResolution`\s*`NodeNext`/.test(md) || /Shared options/.test(md)) {
    for (const [rel, cfg] of Object.entries(manifests.tsconfigs)) {
      const opt = cfg.compilerOptions ?? {};
      for (const [key, value] of Object.entries(tsNeed)) {
        if (opt[key] !== value) {
          fail(`${rel} compilerOptions.${key} is ${JSON.stringify(opt[key])}, STACK.md shared options say ${value}`);
        }
      }
    }
  }

  const ci = manifests.ci;
  const expectedActions = {
    "actions/checkout": ci.actions["actions/checkout"],
    "actions/setup-node": ci.actions["actions/setup-node"],
    "dtolnay/rust-toolchain": ci.actions["dtolnay/rust-toolchain"],
    "Swatinem/rust-cache": ci.actions["Swatinem/rust-cache"],
  };
  for (const [name, ref] of Object.entries(expectedActions)) {
    if (!ref) {
      fail(`ci.yml is missing uses: ${name}`);
      continue;
    }
    const hits = rowsMentioning(tables, name);
    if (!hits.length) {
      fail(`docs/STACK.md is missing a row for ${name}`);
      continue;
    }
    for (const hit of hits) {
      const docRef = (hit.row["ref we use"] || hit.row.requirement || "").trim();
      if (!docRef) continue;
      const normalized = docRef.replace(/^@/, "");
      if (normalized !== ref && !docRef.endsWith(ref) && docRef !== `@${ref}` && docRef !== ref) {
        fail(`docs/STACK.md ref for ${name} is ${docRef}, ci.yml has @${ref}`);
      }
    }
  }
  if (ci.nodeVersion && !mentionsName(md, ci.nodeVersion)) {
    // Node 22 appears many times; require the CI / engines claim
    if (!new RegExp(`node[^\\n]{0,40}${escapeRe(ci.nodeVersion)}`, "i").test(md)) {
      fail(`docs/STACK.md must record Node.js ${ci.nodeVersion} from ci.yml`);
    }
  }
  for (const job of ci.jobs ?? []) {
    if (job === "concurrency") continue;
    if (!md.includes(job)) fail(`docs/STACK.md must mention CI job ${job}`);
  }
  for (const os of ci.os ?? []) {
    if (!md.includes(os)) fail(`docs/STACK.md must mention runner ${os}`);
  }

  const engines = Object.values(manifests.npmByPackage)
    .map((pkg) => pkg.engines?.node)
    .filter(Boolean);
  for (const eng of engines) {
    if (!md.includes("engines") && !/bet-mcp/.test(md)) {
      fail("docs/STACK.md must mention bet-mcp engines");
    }
    // engines is ">=22"; STACK may say "22"
    const major = firstSemver(eng.replace(/^>=/, ""));
    if (major && ci.nodeVersion && major !== ci.nodeVersion) {
      fail(`bet-mcp engines.node ${eng} does not match CI node-version ${ci.nodeVersion}`);
    }
  }

  if (manifests.formula.urlTag && !md.includes(manifests.formula.urlTag)) {
    fail(`docs/STACK.md must record formula url tag ${manifests.formula.urlTag}`);
  }
  if (manifests.formula.headBranch && !/\bhead\b[^\n]*\bmain\b/i.test(md) && !md.includes(manifests.formula.headBranch)) {
    fail(`docs/STACK.md must record formula head branch ${manifests.formula.headBranch}`);
  }
  if (manifests.wasmOpt === false && !/wasm-opt`?\s*off/i.test(md) && !/wasm-opt = false/.test(md)) {
    fail("docs/STACK.md must record wasm-opt off for bet-wasm");
  }

  return failures;
}

const isMain = path.resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url);
if (isMain) {
  const md = readFileSync(path.join(STACK_ROOT, "docs/STACK.md"), "utf8");
  const failures = checkStackPins(md, readStackManifests(STACK_ROOT));
  if (failures.length) {
    console.error(`check-stack: ${failures.length} failure(s)`);
    for (const msg of failures) console.error(`- ${msg}`);
    process.exit(1);
  }
  console.log("check-stack: OK");
}
