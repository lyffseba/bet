# Stack

Pins for the v2 monorepo (`2.0.0-alpha.0`): Rust workspace, WASM host, npm workspaces, CI, and the Homebrew formula. Official links below are each project's current stable docs (language `/stable` or `/latest`, docs.rs `/latest`, or the project's own docs site).

**Last checked: 2026-09-25.**

| Column | Meaning |
|--------|---------|
| **Requirement** | Text in `Cargo.toml`, `package.json`, the workflow, or the formula |
| **Locked** | Version `Cargo.lock` or `package-lock.json` actually builds |
| **Latest stable** | Newest non-prerelease that day: crates.io `max_stable_version`, the npm `latest` dist-tag, `[pkg.rustc]` on the Rust stable channel, `nodejs.org/dist/index.json`, or the action's newest stable GitHub release |

Cargo treats a bare version as a caret requirement (`5.1.2` means `^5.1.2`). npm `^` ranges work the same way. The string `latest` is an npm dist-tag; the lockfile is what `npm install` keeps until the manifest changes.

`legacy_rust/` is outside the workspace and is omitted here.

## Toolchain

| Tool | Requirement | Where | Official docs | Role |
|------|-------------|-------|---------------|------|
| Rust / Cargo | `stable` channel | CI `dtolnay/rust-toolchain@stable` | [stable std](https://doc.rust-lang.org/stable/), [Cargo book](https://doc.rust-lang.org/cargo/), [rustup](https://rust-lang.github.io/rustup/) | Build, test, clippy |
| Edition | `2024` | `[workspace.package]` | [Edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) | All workspace crates |
| Resolver | `"2"` (explicit) | `[workspace]` | [Dependency resolution](https://doc.rust-lang.org/cargo/reference/resolver.html) | Feature unification. Edition 2024's default is resolver `"3"` (MSRV-aware `fallback`) |
| Clippy | component on `stable` | CI `components: clippy` | [Clippy](https://doc.rust-lang.org/clippy/) | `make clippy` / `verify-engine.sh`, `-D warnings` |
| `wasm32-unknown-unknown` | CI target | workflow `targets:` | [Platform support](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html) | `bet-wasm` clippy and the wasm-bindgen fallback |
| wasm-pack | installer script, unpinned | `scripts/build-wasm.sh`, CI | [wasm-pack book](https://wasm-bindgen.github.io/wasm-pack/) | `wasm-pack build --target nodejs` into `packages/bet-ts/pkg` |
| wasm-bindgen-cli | `cargo install --locked` when wasm-pack is absent | `scripts/build-wasm.sh` | [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/) | Fallback bindgen for the same Node package |
| Node.js | `"22"` | `actions/setup-node`, `bet-mcp` `engines` | [v22 API](https://nodejs.org/docs/latest-v22.x/api/) (this line), [Active LTS v24](https://nodejs.org/docs/latest-v24.x/api/) | Typecheck, `node --experimental-strip-types` tests ([type stripping](https://nodejs.org/docs/latest-v22.x/api/typescript.html)) |
| npm workspaces | lockfileVersion `3` | root `package.json` | [Workspaces](https://docs.npmjs.com/cli/using-npm/workspaces/) | `packages/*` |
| TypeScript | `^7.0.2` | every package `devDependencies` | [Handbook](https://www.typescriptlang.org/docs/) | `tsc --noEmit`. Shared options: `strict`, `module`/`moduleResolution` `NodeNext`, `target` `ES2022` |
| GNU Make | unpinned | `Makefile` | [GNU Make](https://www.gnu.org/software/make/manual/make.html) | `make verify` runs `scripts/verify-engine.sh` |
| Python 3 | unpinned `python3` | `scripts/verify-engine.sh` | [json](https://docs.python.org/3/library/json.html) | Reads `protocols/fixtures/wasm_goldens.json` during the gate |

CI policy, from `.github/workflows/ci.yml`: Rust stays on `stable` (no MSRV pin, no beta cell). Node stays on 22.

## Rust crates

Workspace members: `bet-core`, `bet-protocol`, `bet-cli`, `bet-wasm`. Direct requirements live in the root `[workspace.dependencies]` unless noted.

### `bet-core`

Rules and the virtual-points ledger. No filesystem, sockets, or `thread_rng`.

| Crate | Use | Docs |
|-------|-----|------|
| `serde` `1` (optional, default feature) | Derive on public types when the `serde` feature is on | [serde.rs](https://serde.rs/), [docs.rs](https://docs.rs/serde/latest/serde/) |

### `bet-protocol`

Room authority. Join, stake, and settle live in `Table`. Sockets are `std::net` (TCP NDJSON); the wire format is JSON lines.

| Crate | Use | Docs |
|-------|-----|------|
| `serde` `1` | `ClientMsg` / `ServerMsg` | [serde.rs](https://serde.rs/) |
| `serde_json` `1` | `encode_line` / `decode_*_line` | [docs.rs](https://docs.rs/serde_json/latest/serde_json/) |

### `bet-cli`

The `bet` binary: ratatui hub, local games, and host/join. Multiplayer rules stay in `bet-protocol`.

| Crate | Use | Docs |
|-------|-----|------|
| `ratatui` `0.30.0` | Terminal widgets and `CrosstermBackend` | [ratatui.rs](https://ratatui.rs/), [docs.rs](https://docs.rs/ratatui/latest/ratatui/) |
| `crossterm` `0.28` | Raw mode, alternate screen, key events | [docs.rs](https://docs.rs/crossterm/latest/crossterm/) |
| `qrcode` `0.14.1` | QR for invite URLs in the TUI | [docs.rs](https://docs.rs/qrcode/latest/qrcode/) |
| `open` `5.1.2` | `open::that_detached` for those URLs | [docs.rs](https://docs.rs/open/latest/open/) |
| `rand` `0.8` | Matrix and chess shuffle in the CLI. Core RNG is `XorShift64` | [The Rust Rand Book](https://rust-random.github.io/book/), [docs.rs](https://docs.rs/rand/latest/rand/) |
| `shakmaty` `0.30.0` | Chess position and moves | [docs.rs](https://docs.rs/shakmaty/latest/shakmaty/) |
| `directories` `6.0.0` | Default ledger directory (`BET_CONFIG_DIR` overrides it) and matrix scores | [docs.rs](https://docs.rs/directories/latest/directories/) |
| `serde` / `serde_json` | Ledger file on disk | [serde.rs](https://serde.rs/), [serde_json](https://docs.rs/serde_json/latest/serde_json/) |

### `bet-wasm`

`cdylib` over `bet-core` for the TypeScript hosts. `wasm-pack` writes `packages/bet-ts/pkg` (`bet_wasm`, Node target, release, `wasm-opt` off). The package is forced to `"type": "commonjs"` so the ESM workspaces can `createRequire` it.

| Crate | Use | Docs |
|-------|-----|------|
| `wasm-bindgen` `0.2` | `#[wasm_bindgen]` exports; errors are `JsValue` | [guide](https://wasm-bindgen.github.io/wasm-bindgen/), [docs.rs](https://docs.rs/wasm-bindgen/latest/wasm_bindgen/) |
| `serde_json` | Hangman word lists and ledger balances across the boundary | [docs.rs](https://docs.rs/serde_json/latest/serde_json/) |
| `js-sys` `0.3` | Declared direct dependency | [docs.rs](https://docs.rs/js-sys/latest/js_sys/) |
| `serde-wasm-bindgen` `0.6` | Declared direct dependency | [docs.rs](https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/) |

`js-sys` and `serde-wasm-bindgen` are in `crates/bet-wasm/Cargo.toml`. Crate sources call `wasm_bindgen` and `serde_json` by name.

## npm packages

Root package `bet-monorepo` is private. Workspaces:

| Package | Role | Depends on |
|---------|------|------------|
| `@lyffseba/bet-ts` | Loads the WASM pkg. `@lyffseba/bet-ts/play` is the hangman/ttt facade | TypeScript, `@types/node` |
| `bet-pi-hub` (`packages/bet-pi`) | pi extension `/b$t` | `@lyffseba/bet-ts`; peers `@earendil-works/pi-coding-agent`, `@earendil-works/pi-tui` |
| `bet-opencode` | In-process `bet_status` / `bet_play` | `@lyffseba/bet-ts`; peer `@opencode-ai/plugin` |
| `@lyffseba/bet-mcp` | MCP stdio. WASM play, plus `bet_host` / `bet_join` spawning the native `bet` binary | `@lyffseba/bet-ts`, `@modelcontextprotocol/server`, `zod` |

| Package | Requirement | Locked | Docs | Use |
|---------|-------------|--------|------|-----|
| `typescript` | `^7.0.2` | `7.0.2` | [Handbook](https://www.typescriptlang.org/docs/) | `tsc -p tsconfig.json --noEmit` |
| `@types/node` | `^22.0.0` | `22.20.1` | [DefinitelyTyped `node`](https://github.com/DefinitelyTyped/DefinitelyTyped/tree/master/types/node) | `types: ["node"]` on Node 22 |
| `@earendil-works/pi-coding-agent` | `latest` (dev + peer `*`) | `0.82.1` | [pi docs](https://pi.dev/docs/latest), [extensions](https://pi.dev/docs/latest/extensions) | `ExtensionAPI` for the `/b$t` extension |
| `@earendil-works/pi-tui` | `latest` (dev + peer `*`) | `0.82.1` | [TUI](https://pi.dev/docs/latest/tui) | `matchesKey`, `visibleWidth` |
| `@opencode-ai/plugin` | `latest` (dev + peer `*`) | `1.18.8` | [Plugins](https://opencode.ai/docs/plugins/) | `Plugin` and `tool` |
| `@modelcontextprotocol/server` | `^2.0.0` | `2.0.0` | [SDK v2](https://ts.sdk.modelcontextprotocol.io/v2/) (stable line for the [2026-07-28 spec](https://modelcontextprotocol.io/docs/2026-07-28/getting-started/intro)) | `McpServer`, `serveStdio` |
| `zod` | `^4.0.0` | `4.5.4` | [zod.dev](https://zod.dev/) | Tool input schemas in `packages/bet-mcp` |

Pi and OpenCode devDependencies use the `latest` tag so a fresh resolve moves. CI runs `npm install` against the committed lockfile, so the locked column is what the gate typechecks.

## CI

Workflow: `.github/workflows/ci.yml`. Triggers: pull requests, and pushes to `main`. Two jobs:

- `mcp-typecheck` on `ubuntu-latest`: checkout, Node 22, `npm install`, `npm run typecheck -w @lyffseba/bet-mcp`.
- `verify-engine` on `ubuntu-latest` and `macos-latest`: stable Rust + clippy + `wasm32-unknown-unknown`, `Swatinem/rust-cache`, Node 22, wasm-pack installer, then `bash scripts/verify-engine.sh`.

Windows is omitted: the gate is Unix bash (`verify-engine.sh`, `build-wasm.sh`, `e2e-mp.sh`).

| Action | Ref we use | That ref today | Newest stable release | Docs |
|--------|------------|----------------|----------------------|------|
| `actions/checkout` | `v4` | `v4` updated 2026-07-16 | `v7.0.1` | [checkout](https://github.com/actions/checkout), [Actions](https://docs.github.com/en/actions) |
| `actions/setup-node` | `v4` | `v4` updated 2025-04-02 | `v7.0.0` | [setup-node](https://github.com/actions/setup-node) |
| `dtolnay/rust-toolchain` | `stable` | channel commit 2026-09-03, rustc **1.98.1** | action tag `v1` (2026-09-12) is the action, separate from the channel | [rust-toolchain](https://github.com/dtolnay/rust-toolchain) |
| `Swatinem/rust-cache` | `v2` | tag points at **2.9.2** (2026-08-06) | `v2.9.2` | [rust-cache](https://github.com/Swatinem/rust-cache) |

`@stable` on `dtolnay/rust-toolchain` is the Rust channel ref. It is how this repo tracks stable rustc. The action's `v1` tag is a different ref and has moved since the `stable` ref's last channel bump.

## Formula

`Formula/bet.rb` is a Homebrew formula. It builds `crates/bet-cli` with `cargo install` and `depends_on "rust" => :build` (Homebrew's `rust`, the stable toolchain).

| Field | Value |
|-------|--------|
| Docs | [Formula cookbook](https://docs.brew.sh/Formula-Cookbook) |
| `url` | `v1.0.0` tarball |
| `head` | `main` |
| Workspace version | `2.0.0-alpha.0` |

The tagged source URL is the v1.0.0 release. `head` follows `main`.

## Ours vs latest

Checked 2026-09-25. "In sync" means the locked build matches the latest stable release, or a floating channel is defined to track it.

### Toolchain

| Component | Requirement | Locked / resolved | Latest stable | Drift |
|-----------|-------------|-------------------|---------------|-------|
| rustc | `stable` | 1.98.1 via `@stable` (2026-09-03) | 1.98.1 | in sync (channel) |
| edition | 2024 | 2024 | 2024 | in sync |
| Cargo resolver | `"2"` | `"2"` | `"3"` (edition 2024 default) | explicit pin behind the edition default |
| Node.js | `22` | newest 22.x at CI time | 22.23.3 on this line; Active LTS 24.21.0 (Krypton); Current 26.10.0 | line is Maintenance LTS (Jod); patch floats |
| TypeScript | `^7.0.2` | 7.0.2 | 7.0.2 | in sync |
| wasm-pack | installer | whatever the installer ships that run | 0.15.0 | tracks latest |
| wasm-bindgen-cli | `--locked` install, fallback only | installed on demand | 0.2.129 | unpinned fallback |
| Python 3 | `python3` on PATH | runner default | 3.14.7 | unpinned |
| `actions/checkout` | `@v4` | v4 line | v7.0.1 | major behind |
| `actions/setup-node` | `@v4` | v4 line (last move 2025-04-02) | v7.0.0 | major behind |
| `Swatinem/rust-cache` | `@v2` | 2.9.2 | 2.9.2 | in sync |
| Homebrew formula source | `v1.0.0` tarball | tag v1.0.0 | workspace `2.0.0-alpha.0` on `main` | release tarball behind `head` |

### Rust (direct)

| Crate | Requirement | Locked | Latest stable | Drift |
|-------|-------------|--------|---------------|-------|
| `crossterm` | 0.28 | 0.28.1 (direct). `ratatui-crossterm` 0.1.0 also locks 0.29.0 | 0.29.0 | direct requirement one minor behind; two copies in the lockfile |
| `qrcode` | 0.14.1 | 0.14.1 | 0.14.1 | in sync |
| `rand` | 0.8 | 0.8.5 | 0.10.3 | minor line behind |
| `ratatui` | 0.30.0 | 0.30.0 | 0.30.2 | patch behind |
| `open` | 5.1.2 | 5.3.3 | 5.4.4 | patch behind the caret range's ceiling |
| `shakmaty` | 0.30.0 | 0.30.0 | 0.30.1 | patch behind |
| `directories` | 6.0.0 | 6.0.0 | 6.0.0 | in sync |
| `serde` | 1 | 1.0.228 | 1.0.229 | patch behind |
| `serde_json` | 1 | 1.0.149 | 1.0.151 | patch behind |
| `wasm-bindgen` | 0.2 | 0.2.114 | 0.2.129 | patch line behind |
| `js-sys` | 0.3 | 0.3.91 | 0.3.106 | patch line behind |
| `serde-wasm-bindgen` | 0.6 | 0.6.5 | 0.6.5 | in sync |

### npm (direct)

| Package | Requirement | Locked | Latest stable | Drift |
|---------|-------------|--------|---------------|-------|
| `typescript` | `^7.0.2` | 7.0.2 | 7.0.2 | in sync |
| `@types/node` | `^22.0.0` | 22.20.1 | 22.20.4 on the 22 line; `latest` tag is 26.6.2 | patch behind on the Node 22 line |
| `zod` | `^4.0.0` | 4.5.4 | 4.6.5 | minor behind |
| `@modelcontextprotocol/server` | `^2.0.0` | 2.0.0 | 2.1.0 | minor behind |
| `@earendil-works/pi-coding-agent` | `latest` | 0.82.1 | 0.87.1 | lock behind the dist-tag |
| `@earendil-works/pi-tui` | `latest` | 0.82.1 | 0.87.1 | lock behind the dist-tag |
| `@opencode-ai/plugin` | `latest` | 1.18.8 | 1.18.32 | lock behind the dist-tag |

## Maintenance

Update this file in the same change as a direct pin, a lockfile bump that moves a row above, or a CI/formula toolchain change. Re-read the official docs URL if the project moved its stable book.

Refresh the latest-stable column from the registries. Keep that refresh out of CI. `scripts/verify-engine.sh` only checks that `docs/STACK.md` is non-empty and that `README.md` links to it.

```bash
# crates.io max_stable_version
curl -fsSL -A bet-stack-docs https://crates.io/api/v1/crates/ratatui \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['crate']['max_stable_version'])"

# npm latest
npm view typescript version
npm view @types/node version

# Rust stable
curl -fsSL https://static.rust-lang.org/dist/channel-rust-stable.toml | awk '/\[pkg.rustc\]/{p=1} p&&/^version =/{print; exit}'

# Node index (first row is Current)
curl -fsSL https://nodejs.org/dist/index.json | python3 -c "import json,sys; rows=json.load(sys.stdin); print(rows[0]['version'], 'lts', rows[0]['lts'])"
```

After a Rust or WASM engine change, the golden-fixture rules in `docs/QUALITY.md` still apply. This page does not replace that checklist.
