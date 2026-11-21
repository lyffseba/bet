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
const RUNNER_RE = "ubuntu-latest|macos-latest|windows-latest";

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

function lockedCell(hit) {
  return String(hit.row.locked ?? hit.row["locked / resolved"] ?? "").trim();
}

function hasLockedCell(hit) {
  return lockedCell(hit).length > 0;
}

function todayISO() {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function caretSatisfies(req, version) {
  const raw = String(req ?? "").trim().replace(/^[\^~]/, "");
  const r = raw.split(".").filter(Boolean).map((n) => parseInt(n, 10));
  const v = String(version ?? "").split(".").filter(Boolean).map((n) => parseInt(n, 10));
  if (!r.length || !v.length || r.some((n) => Number.isNaN(n)) || v.some((n) => Number.isNaN(n))) {
    return false;
  }
  const [r0 = 0, r1 = 0, r2 = 0] = r;
  const [v0 = 0, v1 = 0, v2 = 0] = v;
  if (r0 !== 0) {
    if (v0 !== r0) return false;
    if (v1 > r1) return true;
    if (v1 < r1) return false;
    return v2 >= r2;
  }
  if (r1 !== 0) {
    if (v0 !== 0 || v1 !== r1) return false;
    return v2 >= r2;
  }
  return v0 === 0 && v1 === 0 && v2 === r2;
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

function isDepHeader(header) {
  return /^(?:target\..+\.)?dependencies(?:\.[A-Za-z0-9_-]+)?$/.test(header);
}

export function parseCrateDeps(toml) {
  const out = {};
  const re = /^\[(.+)\]\s*$/gm;
  const marks = [...toml.matchAll(re)];
  for (let i = 0; i < marks.length; i++) {
    const header = marks[i][1].trim();
    if (!isDepHeader(header)) continue;
    const body = toml.slice(marks[i].index + marks[i][0].length, marks[i + 1]?.index);
    const dotted = header.match(/\.?dependencies\.([A-Za-z0-9_-]+)$/);
    if (dotted) {
      const version = body.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1];
      if (version) out[dotted[1]] = version;
      continue;
    }
    for (const line of body.split("\n")) {
      const item = parseTomlAssignment(line.trim());
      if (!item) continue;
      if (item.path && !item.version) continue;
      if (item.version) out[item.key] = item.version;
    }
  }
  return { dependencies: out, wasmOpt: wasmOptFromToml(toml) };
}

export function wasmOptFromToml(toml) {
  for (const raw of String(toml).split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    if (/^wasm-opt\s*=\s*false\b/.test(line)) return false;
    if (/^wasm-opt\s*=\s*true\b/.test(line)) return true;
  }
  return null;
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
  const actionRefs = {};
  const actions = {};
  for (const m of yml.matchAll(/uses:\s*(\S+)@(\S+)/g)) {
    const name = m[1];
    const ref = m[2];
    actionRefs[name] ??= [];
    if (!actionRefs[name].includes(ref)) actionRefs[name].push(ref);
    actions[name] = ref;
  }
  const nodeVersion = yml.match(/node-version:\s*"?([^"\s]+)"?/)?.[1] ?? "";
  const jobsBlock = yml.split(/^jobs:\s*$/m)[1] ?? "";
  const jobs = [...jobsBlock.matchAll(/^  ([A-Za-z0-9_-]+):\s*$/gm)].map((m) => m[1]);
  const os = (yml.match(/os:\s*\[([^\]]+)\]/)?.[1] ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  return { actions, actionRefs, nodeVersion, jobs, os };
}

export function allActionRefs(ci) {
  const map = {};
  const add = (name, ref) => {
    if (!name || ref == null || ref === "") return;
    const refs = Array.isArray(ref) ? ref : [ref];
    map[name] ??= [];
    for (const r of refs) {
      if (r != null && r !== "" && !map[name].includes(r)) map[name].push(r);
    }
  };
  for (const [name, ref] of Object.entries(ci?.actionRefs ?? {})) add(name, ref);
  for (const [name, ref] of Object.entries(ci?.actions ?? {})) add(name, ref);
  return map;
}

export function parseFormula(rb) {
  const urlTag = rb.match(/archive\/refs\/tags\/(v[\d.]+)\.tar\.gz/)?.[1] ?? "";
  const headBranch = rb.match(/head\s+"[^"]+"\s*,\s*branch:\s*"([^"]+)"/)?.[1] ?? "";
  const rustBuild = /depends_on\s+"rust"\s*=>\s*:build/.test(rb);
  return { urlTag, headBranch, rustBuild };
}

function collectNpm(root) {
  const reqsByPkg = {};
  const rootPkg = path.join(root, "package.json");
  try {
    reqsByPkg["package.json"] = JSON.parse(readFileSync(rootPkg, "utf8"));
  } catch {
    // optional in fixtures
  }
  const packagesDir = path.join(root, "packages");
  let dirs = [];
  try {
    dirs = readdirSync(packagesDir, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name);
  } catch {
    dirs = [];
  }
  for (const name of dirs) {
    const rel = `packages/${name}/package.json`;
    try {
      reqsByPkg[rel] = JSON.parse(readFileSync(path.join(root, rel), "utf8"));
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
    if (rel === "package.json") continue;
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
    for (const field of ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"]) {
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
  const re = new RegExp(`${escapeRe(name)}\\s+([^\\s,;/]+)`);
  const m = label.match(re);
  return m ? m[1] : "";
}

function sameReq(doc, manifest) {
  const a = String(doc ?? "").trim().replace(/^"|"$/g, "");
  const b = String(manifest ?? "").trim().replace(/^"|"$/g, "");
  return a === b;
}

export function npmRequirementMatches(docReq, uniqueReqs) {
  const reqs = uniqueReqs.map((v) => String(v));
  if (reqs.every((v) => sameReq(docReq, v))) return true;
  const set = new Set(reqs);
  if (set.size <= 2 && set.has("latest") && [...set].every((v) => v === "latest" || v === "*")) {
    return /\blatest\b/.test(docReq) && /(^|[^A-Za-z0-9_])\*(?![A-Za-z0-9_])/.test(docReq) && /peer/i.test(docReq);
  }
  return false;
}

function normalizeRef(ref) {
  return String(ref ?? "").trim().replace(/^@/, "");
}

function versionTags(text) {
  return [...String(text).matchAll(/v\d+(?:\.\d+)*/g)].map((m) => m[0]);
}

function formulaTableValue(tables, field) {
  for (const table of tables) {
    const h0 = (table.header[0] || "").toLowerCase();
    if (h0 !== "field") continue;
    for (const row of table.rows) {
      const label = rowLabel(row).trim().toLowerCase();
      if (label === field.toLowerCase()) return (row.value ?? "").trim();
    }
  }
  return "";
}

function toolchainRequirement(tables, tool) {
  const hits = rowsMentioning(tables, tool);
  const preferred = hits.filter((h) => {
    const h0 = (h.table.header[0] || "").toLowerCase();
    return h0 === "tool" || h0 === "component";
  });
  return (preferred.length ? preferred : hits)
    .map((h) => (h.row.requirement ?? "").trim())
    .filter(Boolean);
}

export function documentedCi(md) {
  const section = md.split(/^## CI\s*$/m)[1]?.split(/^## /m)[0] ?? md;
  const jobs = [];
  const addJob = (name) => {
    if (!name || jobs.includes(name)) return;
    if (new RegExp(`^(${RUNNER_RE})$`).test(name)) return;
    if (name.includes(".") || name.includes("/")) return;
    jobs.push(name);
  };
  for (const m of section.matchAll(/^\s*-\s*`([A-Za-z0-9_-]+)`/gm)) addJob(m[1]);
  const intro = section.split(/\n\s*-\s*`/)[0] ?? section;
  for (const line of intro.split("\n")) {
    const idx = line.search(/jobs?:/i);
    if (idx < 0) continue;
    for (const m of line.slice(idx).matchAll(/`([A-Za-z0-9_.-]+)`/g)) addJob(m[1]);
  }
  const runners = [];
  const addRunner = (name) => {
    if (!name || runners.includes(name)) return;
    runners.push(name);
  };
  for (const line of section.split("\n")) {
    if (/\bomitted\b|\bnot used\b|\bnot included\b/i.test(line)) continue;
    for (const m of line.matchAll(new RegExp(`\`(${RUNNER_RE})\``, "g"))) addRunner(m[1]);
  }
  return { jobs, runners };
}

function setsEqual(a, b) {
  if (a.size !== b.size) return false;
  for (const x of a) if (!b.has(x)) return false;
  return true;
}

function list(xs) {
  return [...xs].join(", ") || "(none)";
}

export function checkStackPins(md, manifests) {
  const failures = [];
  const fail = (msg) => failures.push(msg);
  const tables = markdownTables(md);

  const checked = md.match(/Last checked:\s*(20\d{2}-\d{2}-\d{2})/i);
  if (!checked) {
    fail("docs/STACK.md must have a Last checked: YYYY-MM-DD date");
  } else if (checked[1] > todayISO()) {
    fail(`docs/STACK.md Last checked date ${checked[1]} is in the future`);
  }

  if (/only checks that `?docs\/STACK\.md`? is non-empty/i.test(md)) {
    fail(
      "docs/STACK.md maintenance still says verify-engine only checks STACK.md is non-empty; the pin gate also runs now",
    );
  }
  if (!/check-docs/.test(md) && !/manifest/i.test(md)) {
    fail(
      "docs/STACK.md maintenance must say check-docs compares requirement/locked pins to repo manifests",
    );
  } else if (!/check-docs/.test(md) || !/manifest/i.test(md)) {
    fail(
      "docs/STACK.md maintenance must say check-docs compares requirement/locked pins to repo manifests",
    );
  }

  const { workspace } = manifests;
  if (workspace.version && !md.includes(workspace.version)) {
    fail(`docs/STACK.md must mention workspace version ${workspace.version}`);
  }
  if (workspace.edition) {
    if (!new RegExp(`edition[^\\n]{0,80}${escapeRe(workspace.edition)}`, "i").test(md)) {
      fail(`docs/STACK.md must record edition ${workspace.edition}`);
    }
  }
  const resolverReqs = toolchainRequirement(tables, "Resolver");
  if (workspace.resolver) {
    if (!resolverReqs.length) {
      fail(`docs/STACK.md Toolchain Resolver row is missing (workspace resolver is "${workspace.resolver}")`);
    }
    for (const raw of resolverReqs) {
      const quoted = raw.match(/"(\d+)"/)?.[1] ?? raw.match(/\b(\d+)\b/)?.[1];
      if (quoted !== workspace.resolver) {
        fail(
          `docs/STACK.md Toolchain Resolver requirement is ${raw}, workspace resolver is "${workspace.resolver}"`,
        );
      }
    }
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
      fail(`docs/STACK.md is missing a row for crate ${name} (manifest ${req})`);
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
    const lockedHits = hits.filter(hasLockedCell);
    if (!lockedHits.length) {
      fail(
        `docs/STACK.md has no Locked cell for crate ${name} (Cargo.lock ${list(lockVers) || "unlisted"})`,
      );
    }
    for (const hit of lockedHits) {
      const cell = lockedCell(hit);
      if (lockVers.length === 1) {
        const doc = firstSemver(cell);
        if (doc !== lockVers[0]) {
          fail(`docs/STACK.md locked ${name} is ${doc || cell}, Cargo.lock has ${lockVers[0]}`);
        }
      } else if (lockVers.length > 1) {
        for (const v of lockVers) {
          if (!cell.includes(v)) {
            fail(
              `docs/STACK.md locked ${name} must mention every Cargo.lock copy (${lockVers.join(", ")}); missing ${v}`,
            );
          }
        }
        const direct = cell.match(/(\d+(?:\.\d+){0,3})\s*\(direct\)/i)?.[1];
        if (!direct) {
          fail(
            `docs/STACK.md locked ${name} must mark the direct Cargo.lock version with (direct); lock has ${lockVers.join(", ")}`,
          );
        } else if (!lockVers.includes(direct)) {
          fail(`docs/STACK.md marks ${name} ${direct} as (direct), but Cargo.lock has ${lockVers.join(", ")}`);
        } else if (!caretSatisfies(req, direct)) {
          const matching = lockVers.filter((v) => caretSatisfies(req, v));
          fail(
            `docs/STACK.md marks ${name} ${direct} as (direct), but requirement ${req} is locked at ${matching.join(", ") || lockVers.join(", ")}`,
          );
        }
      }
    }
  }

  const { reqs: npmReqs, origins } = directNpmReqs(manifests);
  for (const [name, req] of Object.entries(npmReqs)) {
    const hits = rowsMentioning(tables, name);
    if (!hits.length) {
      fail(`docs/STACK.md is missing a row for npm package ${name} (manifest ${req})`);
      continue;
    }
    const withReq = hits.filter((h) => h.row.requirement);
    if (!withReq.length) {
      fail(`docs/STACK.md has no requirement for npm package ${name} (manifest ${req})`);
    }
    const uniqueReqs = [...new Set(origins[name].map((o) => o.version))];
    const matching = withReq.filter((h) => npmRequirementMatches(h.row.requirement.trim(), uniqueReqs));
    if (withReq.length && !matching.length) {
      const docReq = withReq.map((h) => h.row.requirement.trim()).join(" | ");
      fail(
        `docs/STACK.md requirement for ${name} is ${docReq}, package.json has ${uniqueReqs.join(", ")}`,
      );
    }
    const lockedHits = hits.filter(hasLockedCell);
    if (!lockedHits.length) {
      fail(
        `docs/STACK.md has no Locked cell for npm package ${name} (package-lock.json ${manifests.npmLock[name] ?? "unlisted"})`,
      );
    }
    const lockVer = manifests.npmLock[name];
    for (const hit of lockedHits) {
      if (!lockVer) continue;
      const doc = firstSemver(lockedCell(hit));
      if (doc !== lockVer) {
        fail(`docs/STACK.md locked ${name} is ${doc || lockedCell(hit)}, package-lock.json has ${lockVer}`);
      }
    }
  }

  for (const [rel, pkg] of Object.entries(manifests.npmByPackage)) {
    if (rel === "package.json") continue;
    const ts = pkg.devDependencies?.typescript;
    if (!ts) fail(`${rel} is missing devDependencies.typescript (STACK.md says every package has it)`);
  }

  const tsNeed = { strict: true, module: "NodeNext", moduleResolution: "NodeNext", target: "ES2022" };
  if (/Shared options/.test(md)) {
    for (const [rel, cfg] of Object.entries(manifests.tsconfigs)) {
      const opt = cfg.compilerOptions ?? {};
      for (const [key, value] of Object.entries(tsNeed)) {
        if (opt[key] !== value) {
          fail(`${rel} compilerOptions.${key} is ${JSON.stringify(opt[key])}, STACK.md shared options say ${value}`);
        }
      }
    }
  }

  const ci = manifests.ci ?? {};
  const actionMap = allActionRefs(ci);
  for (const [name, refs] of Object.entries(actionMap)) {
    const hits = rowsMentioning(tables, name);
    if (!hits.length) {
      fail(`docs/STACK.md is missing a row for ${name} (ci.yml uses @${refs.join(", @")})`);
      continue;
    }
    const docRefs = [];
    for (const hit of hits) {
      if (!Object.prototype.hasOwnProperty.call(hit.row, "ref we use") && !hit.row.requirement) {
        continue;
      }
      const raw = (hit.row["ref we use"] ?? "").trim();
      if (Object.prototype.hasOwnProperty.call(hit.row, "ref we use") && raw === "") {
        fail(
          `docs/STACK.md has an empty Ref we use for ${name}; ci.yml has @${refs.join(", @")}`,
        );
        continue;
      }
      const docRef = normalizeRef(raw || hit.row.requirement || "");
      if (!docRef) continue;
      if (hit.row["ref we use"] != null && raw !== "") docRefs.push(docRef);
      if (!refs.includes(docRef)) {
        fail(`docs/STACK.md ref for ${name} is ${docRef}, ci.yml has @${refs.join(", @")}`);
      }
    }
    for (const ref of refs) {
      if (docRefs.length && !docRefs.includes(ref)) {
        fail(`docs/STACK.md Ref we use for ${name} is ${docRefs.join(", ")}, ci.yml also has @${ref}`);
      }
    }
  }

  const nodeReqs = toolchainRequirement(tables, "Node.js");
  if (ci.nodeVersion) {
    if (!nodeReqs.length) {
      fail(`docs/STACK.md Toolchain Node.js requirement is missing; ci.yml has node-version ${ci.nodeVersion}`);
    }
    for (const raw of nodeReqs) {
      const docNode = raw.replace(/["'`]/g, "").match(/\d+/)?.[0];
      if (docNode !== ci.nodeVersion) {
        fail(
          `docs/STACK.md Toolchain Node.js requirement is ${raw}, ci.yml has node-version ${ci.nodeVersion}`,
        );
      }
    }
  }
  const engines = Object.values(manifests.npmByPackage ?? {})
    .map((pkg) => pkg.engines?.node)
    .filter(Boolean);
  for (const eng of engines) {
    const major = firstSemver(String(eng).replace(/^>=/, ""));
    if (major && ci.nodeVersion && major !== ci.nodeVersion) {
      fail(`package engines.node ${eng} does not match CI node-version ${ci.nodeVersion}`);
    }
  }

  const docCi = documentedCi(md);
  const ciJobs = new Set((ci.jobs ?? []).filter((j) => j !== "concurrency"));
  const docJobs = new Set(docCi.jobs);
  if (ciJobs.size || docJobs.size) {
    if (!setsEqual(ciJobs, docJobs)) {
      fail(`docs/STACK.md CI job list is ${list(docJobs)}; ci.yml has ${list(ciJobs)}`);
    }
  }
  const ciOs = new Set(ci.os ?? []);
  const docOs = new Set(docCi.runners);
  if (ciOs.size || docOs.size) {
    if (!setsEqual(ciOs, docOs)) {
      fail(`docs/STACK.md claims runner(s) ${list(docOs)}; ci.yml matrix is ${list(ciOs)}`);
    }
  }

  const urlValue = formulaTableValue(tables, "url");
  const headValue = formulaTableValue(tables, "head");
  if (manifests.formula?.urlTag) {
    if (!urlValue) {
      fail(`docs/STACK.md Formula table is missing a url row (Formula/bet.rb is ${manifests.formula.urlTag})`);
    } else {
      const tags = versionTags(urlValue);
      if (!tags.includes(manifests.formula.urlTag)) {
        fail(
          `docs/STACK.md Formula url tag is ${tags.join(", ") || urlValue}, Formula/bet.rb url is ${manifests.formula.urlTag}`,
        );
      }
    }
  }
  if (manifests.formula?.headBranch) {
    if (!headValue) {
      fail(
        `docs/STACK.md Formula table is missing a head row (Formula/bet.rb head branch is ${manifests.formula.headBranch})`,
      );
    } else {
      const docHead = headValue.replace(/["'`]/g, "").match(/^[A-Za-z0-9._/-]+/)?.[0];
      if (docHead !== manifests.formula.headBranch) {
        fail(
          `docs/STACK.md Formula head is ${docHead || headValue}, Formula/bet.rb head branch is ${manifests.formula.headBranch}`,
        );
      }
    }
  }
  if (manifests.formula && manifests.formula.rustBuild === false) {
    fail(
      'Formula/bet.rb must keep depends_on "rust" => :build (STACK.md records Homebrew rust as the build dependency)',
    );
  }

  const docSaysWasmOff = /wasm-opt`?\s*off/i.test(md) || /wasm-opt\s*=\s*false/.test(md);
  if (manifests.wasmOpt === false && !docSaysWasmOff) {
    fail("docs/STACK.md must record wasm-opt off for bet-wasm");
  }
  if (docSaysWasmOff && manifests.wasmOpt !== false) {
    fail(
      "docs/STACK.md says wasm-opt is off, but crates/bet-wasm/Cargo.toml does not set wasm-opt = false on an active line",
    );
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
