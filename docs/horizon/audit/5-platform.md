# Packaging, build loop, multiplayer model, testing without a human, publishing

Paths: `S` is the scratchpad `/private/tmp/claude-501/-Users-louisobenicherousse-Projects-Balanced-Minecraft-Hardcore--claude-worktrees-busy-visvesvaraya-4dfefc/98064f78-7ec0-4895-a546-21e1425d1f47/scratchpad`.
Citations are `S/src/...` paths, shortened: `H/` = `S/src/horizon/`, `M/` = `S/src/horizon-mods/`, `X/` = `S/src/horizon-mixins/`, `D/` = `S/horizon-docs/`.
Horizon main in `H/` is commit b880384 (2026-11-11). The Nightly is tag `nightly-source/2026-11-08-fdc87ae` (commit `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`); the tag exists in the local clone, so I read its files with `git show <tag>:<path>` (read only).

## Summary table

| Question | Answer | Where |
|---|---|---|
| Pin | commit `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` (Nightly 2026-11-08), NATIVE_HOOKS_API 411 on it, 415 on main | section B |
| horizon-mixins pin | `e8cd0aa38cffef49cb2e088ef716baa107148c02` (every native mod in M/native uses it) | section B |
| "hooks" to declare | the highest API number among the hooks and helpers BMH really uses, never above 411 for this pin; unknown until the feature audits list them | section B |
| Layout | one bundle: thin helpers on named hooks, `[[module]]` for state and glue, `[[crate]]` for the bulk and the tests, content in a separate `.hmod` part; `[api]` crate only if the owner accepts launcher-only builds | section A |
| Closest model | `smp` (server, 26k lines, 36 modules, 26 injections) plus `voicechat-server` (the only one with a `[[crate]]` and tests) plus `recipes`/`minimap` (client) | section A |
| Display asleep | yes with `--offscreen` or `--headless` | section E |
| Vanilla client on a server with BMH items | not verified to work; the server only refuses vanilla clients when it has custom data components or `horizon_clients_only`; mod item ids sit after vanilla's in the network registry | section F |
| Java 26.1 world | opens and is upgraded by ported datafixers; `bmh:*` item ids and unknown player NBT keys survive; BMH game rules are DROPPED on the first save | section G |
| Agent may pass `--trust-native-code` | no | section H |

## A. Which way to write the code

### A.1 The two ways, side by side

| | Mixin helpers on hook points | Typed API (`Hooks` trait, `[api]`) |
|---|---|---|
| Manifest | `[[injection]] id=.. hook="tick" hook_version=1 kind="HEAD" helper="x.rs" bindings=[..]` | `[api] path = "engine"` and nothing else needed |
| Code shape | one small `fn` per hook, pasted into the hook point; state in `[[module]]` files | one crate, `impl Hooks`, one method per hook, `hooks()` exported |
| Works on every Horizon release | yes ("This works on every Horizon release") `D/mod-code.txt:11` | only Nightlies since 2026-11-03, launcher 0.2.37 or later `D/mod-code.txt:327` |
| Can hook the game's own functions | yes, many kinds (HEAD, BEFORE_CALL, RETURN, MODIFY_ARG, OVERWRITE...) `X/README.md:4-8` | only through an `[[injection]]` with `trait_method = "points"`, which pins the commit `D/mod-code.txt:452` |
| Built by `native.py check/build/package` | yes | no: "native.py doesn't know [api] yet: check and build don't handle it, and package leaves the crate's folder out of the .hnmod" `D/mod-code.txt:457` |
| Unit tests | pure logic in the module (`[lib] path` in Cargo.toml) `D/mod-code.txt:75-86` | `dispatch_<hook>` plus a test `Host` double `H/docs/native-mods.md:5338-5358` |
| State | `thread_local!` is fine for the window's game; the single-player server's game "runs on another [thread] and sees its own" `D/mod-code.txt:57` | `&self` hooks, `Send + Sync`, "Keep state in atomics or a Mutex, not in a thread_local!" `D/mod-code.txt:429` |
| Typed `Host` from it | yes: bind `&mut *self`, coerce to `&mut dyn Host` `H/docs/native-mods.md:5271-5290` | native |

Both are generated from the same catalog, so no hook exists in one and not the other: "Every hook point in the catalog is a method of that trait" `D/mod-code.txt:325`. The mixin side has one more thing the trait side lacks: raw game functions. The typed side has two more: `Hooks::render` (RenderHooks with wgpu, `H/crates/horizon-client/src/mod_api.rs:51`) and `[dependencies]` between typed mods `D/mod-code.txt:451`. RenderHooks have a mixin counterpart in "Render hook points" `H/docs/native-mods.md:4696` (the render audit owns the detail).

Important: the typed API is reachable from mixin code. A helper can coerce `Game` to `dyn Host` and use `AttachmentType` (player data with a codec, `copy_on_death`) `H/docs/native-mods.md:5271-5300`, `5402-5440`. So a mixin-only bundle loses nothing the BMH port needs from `Host`.

### A.2 Host methods missing today (verified)

`D/mod-code.txt:443` says "Some helpers aren't on Host yet, among them player, chat, playing, window, text_width and frame_rate". Verification:

- `mod_api.rs` has no static `Host`; it is generated: `pub use generated_host::Host;` `H/crates/horizon-client/src/mod_api.rs:1137`, and "Cargo generates the trait and dispatch from the catalog and source signatures into OUT_DIR" `mod_api.rs:5`.
- The generator makes one `Host` method for every helper in the catalog: `for helper in &helpers {` ... `host_traits.push_str(&declaration);` `H/crates/horizon-mod-api-gen/src/lib.rs:1065-1089`.
- The catalog (main, `H/native-hooks.json`, and the Nightly `S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json`) lists helpers named `player`, `playing`, `window`, `text_width`, `on`, `setting`, `set_setting`, `mod_key`, `fill`, `text`, `give`, `tell`, `feedback`, `set_block`, `net_send_to_player`, `net_send_to_server`, `run_command`, `held`, `entity_nbt`, `entity_merge_nbt`, `kill_mob`, `config_dir`, `world_dir`, `instance_dir`, `built_data`. I checked both files with a script: all present in both.
- Helpers NOT in either catalog, so NOT on `Host`: `chat` (`native_chat`, `native_hooks.rs:11528`), `frame_rate`, `clicks`, `mapping`, `corner_shown`, `data_dir` (use `config_dir` / `built_data` / `world_dir`).

So the page text is stale for `player`, `playing`, `window`, `text_width`: they are on `Host` in both catalogs. `chat` and `frame_rate` are really missing. For `chat`, `send_chat` and `client_action_bar` exist in the catalog. A mod that needs a missing one keeps a mixin helper for it ("one bundle can have both" `D/mod-code.txt:443`).

Hook counts: main 434 hooks and 177 helpers; Nightly 432 and 174. Main-only hooks: `world_hold`, `world_warp` (both client). Main-only helpers: `force_chunk`, `force_chunk_in`, `forced_chunks`. This matches "main has two more hooks and three more helpers". Side split on main: 270 server, 62 client, 10 both, 92 unmarked (tick, hud, key, net_* run in both games).

### A.3 What the registry accepts today

- `native.py` knows no `[api]`: its only manifest reads are `manifest["mod"]["id"]`, `["version"]` (`M/native/tools/native.py:52-53`), `compatibility.horizon_revision` (`:199`) and `crate` paths (`:204`). `check`/`build` run `horizon-mixins apply` then `cargo check/build` of `horizon-client` only (`:163`, `:173-188`), so an `[api]` crate is never compiled by them.
- `package` walks the whole bundle with `rglob` and keeps every `.rs` and `.toml` (`:205-208`). So an `engine/` crate's `src/lib.rs` and `Cargo.toml` would in fact be zipped, contrary to the doc's "leaves the crate's folder out". UNVERIFIED which of the two is true for a launcher that reads the `.hnmod`; I did not run it.
- The registry side does know `[api]`: `self.api = "api" in manifest or any(... "trait_method" ...)` `M/scripts/native_compat.py:68`, and its comment says "The launcher rewrites these before horizon-mixins and stages API packages for them (native_mods/inject.rs, api.rs), which this script doesn't: they are applied here, not compiled" `:66-67`, `:361-362`, `:406`. So an `[api]` mod is accepted but never compile-checked by the registry CI.
- The publish page itself says: "Build typed mods with horizon-launcher native-build, and use mixin helpers for a mod you want in the registry today." `D/mod-code.txt:457`.

### A.4 Recommended layout for about 9,000 lines of Java (server logic, client rendering, items, data, settings)

Decision: one mixin bundle, not an `[api]` crate, for now. Reasons, each with evidence:

1. Only this path is checked by `native.py`, packaged by it and compile-checked by the registry (A.3).
2. The build loop works without the launcher (section C). Typed mods need `horizon-launcher native-build`, which is deliberately not available to an agent: "MCP: none: the player's switch" `H/docs/launcher-commands.md:121`.
3. It loses nothing: `Host`, `AttachmentType` and `Flow`-style cancel hooks are reachable from helpers (A.1).
4. The size is not the limit: `smp` is 25,928 lines of Rust in one bundle (counted with `wc`). What costs time is recompiling `horizon-client`: "A helper's code changed: 48 s ... `horizon-client` only: the mod's crate is kept"; "The mod's crate changed: 11 s" `H/docs/native-mods.md:2063-2066` (the table under "Cache and disk"). So put the bulk in a crate and keep helpers thin.

Layout:

```
bmh-horizon/
  compatibility.json           game 26.3, hooks N, horizon_revision fdc87ae.., horizon_mixins_revision e8cd0aa..
  LICENSE                      Apache-2.0 (the Java repo's LICENSE is Apache 2.0)
  Cargo.toml                   [workspace] members = ["mixins/bmh/core"]   so `cargo test` runs the logic offline
  README.md
  proof.py                     optional: native.py `proof` loads it (see section E)
  mixins/bmh/
    horizon-mixin.toml         [mod] id="bmh" version priority=100; [compatibility]; [[crate]]; [[module]]; [[injection]]
    core/                      [[crate]] id="core" path="core": pure rules (respawn distance, landing search math,
      Cargo.toml                 charges, ritual constants, point codecs, wear). Registry crates in Horizon's Cargo.lock only.
      src/lib.rs
      data/                    bytes compiled in (a crate's own data/), only if the code needs them at run time
    state.rs                   [[module]] file=crates/horizon-client/src/game.rs name="bmh_state": statics (Mutex/atomics),
                                AttachmentType<..> for player data
    glue_server.rs, glue_client.rs   [[module]]: the code that touches Game / Host
    tick.rs, living_death.rs, ...    helpers: one per [[injection]], each a few lines calling the glue
  content/                     the .hmod part (horizon-mod.toml, assets/bmh/..., data/bmh/...), see B.4
```

Rules to respect (all from the docs):
- `[[crate]]` "can't depend on `horizon-client` ... or on any path or git crate" and uses only crates in Horizon's `Cargo.lock` `H/docs/native-mods.md:565-575`. So the crate holds maths, codecs and state machines; the `[[module]]` files hold everything that names a game type.
- Module and helper files "can't use inner attributes or //! comments, other files through mod or include!" `D/mod-manifest.txt` (the paragraph after "Helpers.").
- Name hooks with `hook = "..."`, `hook_version = 1` (horizon-mixins 0.6.0 or later) so the mod survives Horizon commits and declares `hooks` instead of pinning only the commit `D/mod-code.txt:69`, `D/mod-manifest.txt` ("Hook points or a pinned commit"). Any raw `target` (a game function) pins the commit and drops the `hooks` field `H/docs/native-mods.md:5328-5335`.
- The window's game and the single-player server's game both run `native_tick`: guard with `native_window()` / `host.window()` `D/mod-code.txt:22`, `:43`. Use Mutex-wrapped statics in modules even for mixin code, because the server game is a second thread.
- Check `native_on(id)` first in each helper `D/mod-code.txt:40`. Content mod data is separate (B.4).

If the owner is willing to depend on the launcher, the typed form of the same layout is: `[api] path="engine"`, the `core` crate as a `[[crate]]` dependency by sibling path (`shared = { path = "../shared" }`) `H/docs/native-mods.md:5125-5131`, tests through `dispatch_<hook>`. I would not start there.

### A.5 Closest model in `M/native/`

| Mod | Rust lines (`wc` of `*.rs`) | Layout | What to copy |
|---|---|---|---|
| `smp` | 25,928 | 36 `[[module]]` files, 26 `[[injection]]`, root `Cargo.toml` `[lib] path="src/lib.rs"` which `#[path]`-includes every module so `cargo test` runs them (`M/native/smp/src/lib.rs:4-63`), `hooks: 413`, no `data/`, no `[[crate]]` | server-heavy structure, tests of module files, pin to a commit (`horizon_revision 5de5b6f`, newer than the Nightly 411) |
| `voicechat-server` | 2,354 | `[[crate]] relay` with `src/tests.rs`, one `glue.rs` module, 5 injections (`net_from_client`, `server_tick`, `player_logged_out`, `world_lifecycle`), `hooks: 57` | the crate plus glue split, and the `mod_net` messages `M/native/voicechat-server/mixins/voicechat_server/horizon-mixin.toml:10-35` |
| `recipes` | 911 | client panel and server recipes, two modules, `hooks: 2` | client UI plus server data |
| `minimap` | 449 | client drawing, `hooks: 359` | client rendering |

None of the shipped native mods uses `data/`, an `AttachmentType` or an `[api]` crate (I searched `native_data!`, `AttachmentType` and `data` directories under `M/native`, excluding `target`). The only bundle using `data/`, `[[crate]]` and `[[post_build]]` together is the test fixture `launcher/tests/fixtures/native-crate-demo` `H/docs/native-mods.md:438-470`. So BMH would be the first big mod with items, data files and attachments in a bundle: expect to hit rough edges (see Risks).

## B. The exact files of a bundle

### B.1 `horizon-mixin.toml` keys (`D/mod-manifest.txt`, table under "The manifest")

| Key | Use for BMH |
|---|---|
| `[mod] id` (`a-z0-9_`), `version`, `priority = 100` | `id = "bmh"` (same as the Java mod id). Keep the id once published |
| `[compatibility] horizon_revision` | `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` |
| `[[module]] id, file, name, source` | `file = "crates/horizon-client/src/game.rs"`, `name = "bmh_state"` makes `crate::game::bmh_state` |
| `[[injection]] id, hook, hook_version, kind="HEAD", helper, bindings=[{name,ty,value}]` | one per hook; `target = ".."` only for a game function (pins the commit) |
| `[[injection]] trait_method = "points"` | typed event with the `Host` at a game function; pins the commit |
| `[[crate]] id, path, dependent` | the `core` crate |
| `[[post_build]] crate, bin, args` | not needed (runs on the player's machine after the build) |
| `[api] path`, `[dependencies]` | only for the typed form |

### B.2 `compatibility.json` (checked into the bundle root, beside `mixins/`)

```
{
  "game": "26.3",
  "hooks": <highest API number BMH uses, at most 411>,
  "horizon_repository": "https://github.com/horizon-rust/horizon",
  "horizon_revision": "fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca",
  "horizon_mixins_repository": "https://github.com/horizon-rust/horizon-mixins",
  "horizon_mixins_revision": "e8cd0aa38cffef49cb2e088ef716baa107148c02"
}
```
Keys from `D/mod-manifest.txt` ("compatibility.json"). `package` refuses the mod when `horizon-mixin.toml` and `compatibility.json` name different commits `M/native/tools/native.py:199-201`. The doc example shows mixins revision `0d6b103c...`; every native mod actually in `M/native/*/compatibility.json` uses `e8cd0aa38cffef49cb2e088ef716baa107148c02`, and `[[crate]]` needs horizon-mixins 0.6.0 or later `X/README.md` (the 0.6.0 paragraph). Use `e8cd0aa`.

Pin evidence: `git show nightly-source/2026-11-08-fdc87ae:crates/horizon-client/src/game/native_hooks.rs` line 1170 is `pub(crate) const NATIVE_HOOKS_API: u32 = 411;`; main (`H/crates/horizon-client/src/game/native_hooks.rs:1086`) is `415`. The Nightly catalog has 432 hooks, main 434. The tag's `research-lock.json` pins Minecraft 26.3, client sha1 `e877b6a07acd633fb3bb475002175cec036e7b87`, server sha1 `33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c`, Java 25 (same as main).

"hooks" number: each catalog hook was added at some API number, stated in the "API N adds ..." paragraphs of `H/docs/native-mods.md`. The `version` field in the catalog is the hook's contract version (it is 1 for the entries I printed), not the API number. So the right value is `max(API N)` over the hooks BMH uses, found by searching the paragraph of each. I cannot compute it yet: the feature audits have not listed the hooks. If a hook BMH needs is main-only (`world_hold`, `world_warp`), the pin must move off the Nightly and `hooks` can exceed 411 (as `smp` does: 413).

### B.3 Cargo.toml

- Root `Cargo.toml` for tests: `[lib] path = ".."` (Playtime pattern, `D/mod-code.txt:75-86`) or a `[workspace]` with the `core` crate (as `voicechat-server`: `members = ["mixins/voicechat_server/relay"]`, `resolver = "3"`, `M/native/voicechat-server/Cargo.toml`). `edition = "2024"`, `publish = false`.
- `core/Cargo.toml`: `[dependencies]` only registry crates present in Horizon's `Cargo.lock` with a matching version: `serde = "1"`, `serde_json = "1"` are used by `smp`'s root manifest; others must be checked against the lock. A feature that turns on an absent dependency fails at `cargo fetch --locked` `H/docs/native-mods.md:580-600`.
- A `Cargo.lock` is not allowed in the staged crate ("has no `Cargo.lock`, `[workspace]` or build of its own") `H/docs/native-mods.md:565-572`; the root one is only for your own `cargo test`. Copy Horizon's lock for that, as the typed doc advises `D/mod-code.txt:415`.

### B.4 Where data files go

Two different things, do not mix them:

1. Compiled-in bytes: `mixins/<id>/data/...` (read with `horizon_client::native_data!("<mod id>", "path")`) and a crate's own `data/` (read with `include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/..."))`). "Both are compiled in: neither folder is beside the game when it runs." A file read at run time must be written by a `[[post_build]]` step into `HORIZON_NATIVE_DATA_DIR` `H/docs/native-mods.md:726-756`. Limit: 512 files, 64 MiB in the `.hnmod` `:744-746`.
2. Game content (items, blocks, recipes, advancements, tags, lang, models, textures, sounds) is a content mod: `horizon-mod.toml` (`id`, `name`, `version`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`) with `assets/` and `data/` `D/content-mods.txt` ("The manifest"). Items are declared in `data/<id>/horizon/item/*.json`, copying a vanilla template. The BMH Java resources are exactly this shape: `assets/bmh/{textures,models,items,sounds,lang}`, `data/bmh/{advancement,recipe,tags}`, `data/minecraft/tags` (counted under `W/common/src/main/resources`). Sound events go in `assets/<ns>/sounds.json` and files at the archive top level, not under `mixins/` `H/docs/native-mods.md:760-780`.

"One `.hmod` for content and code": `horizon-native.json` plus `mixins/<id>/` plus `horizon-mod.toml`, `assets/`, `data/` in one file; the two manifests must name the same id `H/docs/native-mods.md` (paragraph "One .hmod for content and code"), `H/docs/api/mod-files.md:30-34`. A combined file "needs a launcher and game with native .hmod support". `native.py package` writes only a code-only `.hnmod` (`M/native/tools/native.py:192-221`). I found no tool in the reviewed repos that builds the combined `.hmod` for a local bundle: UNVERIFIED (searched `cargo-horizon-mod` docs in `H/docs/mods.md` and `H/docs/api/mod-files.md`). Plan on two artifacts: content `.hmod` built by `cargo horizon-mod build`, code `.hnmod` from `native.py package`. Whether both may share the id `bmh` is an open question.

### B.5 The `.hnmod`

A zip: `horizon-native.json` (`schema 1, id, version, game, horizon, mixins, hooks`, `M/native/tools/native.py:196-198`), then `mixins/<id>/...` with `.rs`, `.toml`, the data folders above and `LICENSE` copied from the mod root (`:209-211`). Deterministic bytes (`:216-218`), so the SHA-256 can be re-derived from source. Deviation from the doc ("nothing else"): README, LICENSE, COPYING, NOTICE and `.gitignore` are also accepted `H/docs/api/mod-files.md:56`.

## C. The build loop on macOS without the launcher

### C.1 What `native.py` does (read from the code)

| Step | What | Where it lands |
|---|---|---|
| `Mod(name)` | reads `<mod>/compatibility.json` and the first `mixins/*/horizon-mixin.toml`; an absolute path argument works (`NATIVE / "/abs"` is the absolute path) | `M/native/tools/native.py:40-53` |
| `source(rev)` | downloads `https://codeload.github.com/horizon-rust/horizon/tar.gz/<rev>` (no git), unpacks to `native/target/horizon/<rev>/`, deletes symlinks | `:99-120` (about 64 MB per commit, `H/docs/native-mods.md:2085`) |
| `tool(mods)` | `cargo install --locked --quiet --git <horizon-mixins repo> --rev <rev> --root native/target/horizon-mixins/<rev> horizon-mixins-cli` | `:128-141` (7 MB) |
| `stage()` | rewrites `horizon_revision` in copies, runs `horizon-mixins apply --workspace <tree> --mods <bundles> --out <fresh>`, `rsync --checksum --delete` into `native/target/stage/<ids>/` | `:144-170` (133 MB observed) |
| `check` | `cargo check --locked -p horizon-client --bin horizon-client` in the stage, `RUSTFLAGS="-D warnings"`, `CARGO_TARGET_DIR=native/target/cargo`, so debug profile | `:173-187`, `:251-253` |
| `build` | `cargo build --locked -p horizon-client --bin horizon-client [-p horizon-client-lib] --bin horizon-ctl --profile fast` and prints `native/target/cargo/fast/horizon-client` | `:173-188`, `:255-258` |
| `package` | writes `native/target/<id>-<version>.hnmod`, prints size and sha256 | `:191-221` |
| `proof --resources PATH` | build, then runs `<mod>/proof.py`'s `run(ctl_binary, resources, out_dir)` | `:259-267` |
| flags | `--horizon REV` (`latest` = newest Nightly via the GitHub API, which needs network and optional `GITHUB_TOKEN`), `--resources`. Env: `FLEET_LOAD_JOBS` or `CARGO_BUILD_JOBS` (adds `-j`), `NATIVE_NO_DEFAULT_FEATURES` (adds `--no-default-features`: no audio, maybe not a playable client, UNVERIFIED) | `:228-229`, `:174-185` |

`TARGET` is hard-coded to `native/target` next to the script (`:34`) and line 186 sets `CARGO_TARGET_DIR` itself, overwriting any value you export. So the environment variable cannot move the game build. Only a symlink (`native/target` -> another volume) or moving the clone does. `CARGO_HOME` is honoured (it goes to `cargo install` and the dependency fetches).

### C.2 Command sequence

```
# once
git clone --depth 1 https://github.com/horizon-rust/horizon-mods.git ~/hz/horizon-mods     # small
# only if a second volume exists: ln -s /Volumes/EXT/hz-target ~/hz/horizon-mods/native/target   (before the first run)
export CARGO_BUILD_JOBS=6          # memory, not disk
# toolchain already here: rustc 1.96.0 (Horizon needs 1.95: H/Cargo.toml:60 rust-version = "1.95")

M=~/hz/horizon-mods/native/tools/native.py
python3 "$M" check   /abs/path/bmh-horizon          # compile errors, debug profile, minutes
python3 "$M" build   /abs/path/bmh-horizon          # fast profile; prints {"game": ".../cargo/fast/horizon-client"}
python3 "$M" package /abs/path/bmh-horizon          # native/target/bmh-<ver>.hnmod
cargo binstall --git https://github.com/horizon-rust/horizon cargo-horizon-mod    # prebuilt, not a source build
cargo horizon-mod check native/target/bmh-<ver>.hnmod
(cd /abs/path/bmh-horizon && cargo test)            # logic crate only; no game build
```
`cargo horizon-mod check` and `binstall`: `H/docs/mods.md` ("Making a content mod") and `D/test-a-mod.txt`. `horizon-ctl` is built by `build` (`native.py:179-181`), so no second build. For the building of `horizon-client` and `horizon-ctl` from a plain Horizon checkout, the docs say `cargo build --profile fast -p horizon-client-lib --bin horizon-ctl` `H/docs/api/agent-control.md` ("Guide") and `cargo build --locked -p horizon-client --profile <profile>` builds "the game and horizon-ctl" `H/docs/build-times.md:11`, but `native.py build` already does that on the patched tree.

System packages: README lists "Rust 1.90+, Python 3.10+, JDK 25, and on Linux libasound2-dev libudev-dev" `H/README.md` ("Build from source"). Nothing extra is named for macOS. Here: Xcode at `/Applications/Xcode.app` (xcode-select path present), rustc 1.96.0 and an installed 1.95.0 toolchain, `/opt/homebrew/opt/openjdk@25`, Python 3.14. The `nix.md` route is for Linux and is not needed.

### C.3 Cache folders and disk

| Folder | Size | Source |
|---|---|---|
| `native/target/horizon/<rev>/` | 64 MB (132 MB seen) | `H/docs/native-mods.md:2085`; `du` of `M/native/target/horizon` |
| `native/target/horizon-mixins/` | 7 MB | same |
| `native/target/stage/` | 64 to 133 MB | same |
| `native/target/cargo/debug` (`check`) | 1.3 GB seen about 1 to 2 minutes into someone else's run | `du -sh M/native/target/cargo` |
| `native/target/cargo/fast` (`build`) | about 2 GB for the whole game (384 crates), 1.5 GB measured on a Mac mini with release | `H/docs/native-mods.md:2063-2080` |
| `~/.cargo` registry and git | 289 to 657 MB (287 MB now) | same |
| `~/.rustup` | 1.8 GB now, already installed | `du -sh ~/.rustup` |

Note: a build is running right now inside `M/native/target` (files modified at 11:19 to 11:20, `fps_overlay` stage, `cargo/debug`). It is not mine; I changed nothing there. Whoever runs it is spending disk on this Mac's 9.7 GB free (`df`: `/System/Volumes/Data` 98 %).

Expected total for one full loop on this Mac: about 1.4 GB (debug check) + 2 GB (fast build) + 0.5 GB (sources, stage, cargo home) = about 4 GB, plus the prepared resources (D.4). Steps that can fill the 12 GB:
1. Running both `check` and `build`: two profiles of the whole game (about 3.3 GB). After the code compiles, delete `native/target/cargo/debug`.
2. `cargo test` inside an `[api]` crate: "compiles Horizon once, which takes a few minutes" `D/mod-code.txt:415`, a third copy with its own target dir. Avoid; point it at a shared `CARGO_TARGET_DIR` only if you really use the typed form.
3. `--horizon latest` or a new pin: another source tree plus a full recompile (the stage folder is shared but cargo rebuilds): up to +2 GB; the launcher itself prunes at 30 GiB `H/docs/native-mods.md:24-30`, native.py does not prune anything.
4. `release` profile: no incremental, "7 min 6 s for the first build" `:2090`; do not use it here.
5. Building `horizon-launcher` from source (GPUI): a separate workspace `H/launcher/Cargo.toml`, large, not needed.

To keep caches off the main disk: symlink `native/target`; set `CARGO_HOME` (and `CARGO_TARGET_DIR` for the one-off `cargo install` of horizon-mixins and `cargo test`); `horizon-prepare --root ... --output ...` for the resources. If no other volume exists, the only lever is `rm -rf native/target/cargo/debug native/target/stage` between steps.

## D. What the game needs to run at all

### D.1 What the preparer reads (`H/crates/horizon-prepare/src/main.rs`, `artifacts.rs`)

- The pin: `H/research-lock.json`: Minecraft `26.3`, client sha1 `e877b6a0...`, server sha1 `33680f5f...`, Java 25.
- Needs "Minecraft played once from the official launcher, and Java of the version research-lock.json pins" `main.rs:62-63`.
- Client side, found by default in `~/Library/Application Support/minecraft` on macOS (`artifacts.rs:119-124`; override `--minecraft-dir` or `HORIZON_MINECRAFT_DIR`): `versions/26.3/26.3.json`, `versions/26.3/26.3.jar`, `libraries/`, `assets/`. Other layouts: ATLauncher, Prism/MultiMC (`--prism-dir`), or single files (`--version-json`, `--client-jar`, `--libraries-dir`, `--assets-dir`) `main.rs:33-45`. The version JSON must have one of the accepted sha1 values or the preparer refuses it `artifacts.rs:722-745`.
- Server jar: it is NOT in the official launcher folder. `--server-jar PATH` or `HORIZON_SERVER_JAR`: "Horizon does not download the game: download the 26.3 server jar yourself (minecraft.net/download/server while 26.3 is the current release, otherwise the downloads.server.url in your launcher's versions/26.3/26.3.json)" `artifacts.rs:20-22`. Without it, the preparer falls back to the client jar only for Prism installs `artifacts.rs:777-799`.
- Nothing is downloaded by `horizon-prepare`; the exporters run Mojang's code through a JDK 25 (`/opt/homebrew/opt/openjdk@25` exists here; `--java-home` or `JAVA_HOME`, `main.rs:44-45`).
- Command: `cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar` and optionally `... -- sounds` (about 380 MB, copied from the launcher's asset store; without it the game is silent) `H/README.md` and `main.rs:75-77`. Run it from a folder that has `research-lock.json` (any Horizon tree: the `native.py` stage tree and `native/target/horizon/<rev>/` both qualify) or pass `--root`.
- Output: `<root>/.local/horizon-client/26.3` (or `--output`), with `horizon-resources.json`; imported jars land in `<root>/.local/minecraft/26.3/` `artifacts.rs:61` and `main.rs:48`. `horizon-prepare --check` tells whether it is complete.
- Time: a fresh preparation is about 1 to 2 minutes of Java on Linux (`H/docs/launcher-preparation.md:86-92`, 45 to 180 s per run), peak about 1.06 GB of memory.
- Disk: not documented. The copy helper is `clone_or_copy` (`lib.rs` export), which clones on APFS; I did not find a size figure, so UNVERIFIED (plan 1 to 2 GB; `assets` alone on this Mac is 570 MB).

The launcher route is the same data: "Choose Prepare. It finds your Minecraft folder by itself" `D/install.txt:32`, and `horizon-ctl launch` finds `<launcher>/versions/<tag>/.local/horizon-client/<mc version>` itself `H/crates/horizon-client/src/bin/horizon-ctl/launch.rs:44-62`. For a game built by `native.py`, pass `--resources` explicitly.

### D.2 What this Mac has (read only)

`~/Library/Application Support/minecraft/versions/` holds `26.1.2/26.1.2.json` and `26.2-snapshot-3/26.2-snapshot-3.json` and no jar (only the JSON files exist in both folders). Its `version_manifest_v2.json` lists latest release `26.1.2`, latest snapshot `26.2-snapshot-3`, and no `26.3` entry at all: the cached manifest is old or 26.3 is not yet offered there. `assets/` is 570 MB. So neither version can be used.

### D.3 What the owner must do once

1. Open the official Minecraft launcher (this refreshes its version manifest), create or select an installation for Minecraft 26.3 (enable "snapshots" in Installations if 26.3 is typed as a snapshot, UNVERIFIED), press Play once, wait for the title screen, quit. This creates `versions/26.3/26.3.json`, `26.3.jar`, the libraries and the 26.3 asset index in `~/Library/Application Support/minecraft`.
2. Download the 26.3 server jar (the URL is `downloads.server.url` inside `versions/26.3/26.3.json`; minecraft.net/download/server only while 26.3 is the current release). Check sha1 `33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c`.
3. Run `JAVA_HOME=/opt/homebrew/opt/openjdk@25 cargo run --release -p horizon-prepare -- --root <horizon tree> --output ~/hz/res/26.3 --prepare --server-jar <server.jar>`, then `... -- sounds --output ~/hz/res/26.3` (optional).
4. The prepared files then live at `~/hz/res/26.3` (or `<root>/.local/horizon-client/26.3`). That folder is what `horizon-client --resources` and `horizon-ctl launch ... --resources` take (`launch_options.rs:274`). Never commit it ("never commit or share them" `H/README.md`).

The Horizon launcher DMG gives the same result with a click on Prepare, and it is the only way to get `horizon-launcher native-build` (typed route, server export). Installing it is a download: not done here.

## E. Testing with nobody at the keyboard

### E.1 Launching

`horizon-ctl [--socket PATH] [--timeout S] [--api LINE] [--pretty] COMMAND` `H/crates/horizon-client/src/bin/horizon-ctl/commands.rs:635`. `launch [--world NAME] [--seed N] [--creative] [--version TAG] [--game PATH] [--wait] [GAME ARGS...]`, plus `--offscreen` and `--headless` `H/docs/horizon-ctl-commands.md:10`.

- `--game PATH` starts any `horizon-client` binary, such as the `native.py build` output. `--version` and the default pick a launcher-installed version `launch.rs:64-120`.
- `--wait` waits until the world is ready; `launch` itself waits up to 300 s for the socket `launch.rs` (deadline loop).
- Everything else after the known flags is passed to the game: `--world`, `--seed`, `--creative`, `--resources`, `--dev-mod`, `--offscreen`, `--headless`, `--world-frames`, `--control-socket` are parsed in `H/crates/horizon-client/src/game/launch_options.rs:247-338`. `launch` adds `--resources` itself only when a prepared install sits beside the binary; for a `native.py` build pass it: `--resources ~/hz/res/26.3` `launch.rs:199-202`.
- Content that is a folder mod: `--dev-mod <folder or .hmod>` (always on) `launch_options.rs:325`, `H/docs/mods.md:65`.
- Scratch homes: `HORIZON_HOME` sets the game home (saves, options, logs, `horizon-mods/.horizon-mod-data/<id>/`) `H/crates/horizon-client-render/src/logging.rs:113-122`. `agent_control_e2e.py` also sets `HORIZON_TELEMETRY_URL=off` and `HORIZON_NO_AUDIO=1` (`H/tools/agent_control_e2e.py:27-31`, `:77-82`). The developer workflow block also exports `HORIZON_LAUNCHER_HOME`, `XDG_*` `H/docs/native-mods.md:2165-2178`.
- A fresh world: `--world Name --seed 5` creates it when absent (`launch` doc: "world folder to open or create"). The mode is creative with `--creative`; for BMH death tests use `command "gamemode survival"` after the world loads (same `command` as below).

Example:
```
export HORIZON_HOME=$SCRATCH/home HORIZON_NO_AUDIO=1 HORIZON_TELEMETRY_URL=off
horizon-ctl --socket $SCRATCH/ctl.sock launch --game M/native/target/cargo/fast/horizon-client \
  --resources ~/hz/res/26.3 --dev-mod /abs/bmh-horizon/content --world BmhTest --seed 5 --creative --offscreen --wait
```

### E.2 Display asleep

Yes. "on macOS the game renders offscreen, so it also runs in a locked session or over SSH" `H/tools/agent_control_e2e.py:27-28`. `--offscreen` "render without showing a window ... locked or headless sessions" and screenshots still work `H/docs/horizon-ctl-commands.md:10`. `docs/benchmark.md:30`: "On a Mac whose session is locked, Horizon draws only with `--offscreen`". `--headless` has no window, display or GPU, plays through commands only, and `screenshot` and GUI-point clicks answer with an error `H/docs/agent-control.md:217-221`. Caveat: the benchmark doc says offscreen runs are "not a player number" `benchmark-report.md:70`; fine for tests. A sleeping display with a user session (not locked) is not documented; UNVERIFIED, but offscreen does not wait for the window.

### E.3 The commands a BMH test would use

All from `H/docs/horizon-ctl-commands.md`. Replies are one JSON line.

| Need | Command (line in the doc) | Notes |
|---|---|---|
| Start/stop | `launch ... --wait` (10), `quit` (23), `save` (22), `wait-ready [SECONDS]` (12) | |
| Run any command | `command TEXT` (100), `chat TEXT` | `command "kill @s"`, `command "gamerule ..."`, `command "time set night"`, `command "give @s bmh:..."`, `command "tp @s x y z"`, `command "gamemode survival"`. The e2e does exactly `kill @s` then `respawn` `H/tools/agent_control_e2e.py:436-441` |
| Quick cheats | `give ITEM [COUNT]` (124), `time TICKS` (125), `tp X Y Z` (58), `summon KIND [X Y Z] [baby]` (127), `effect`, `xp LEVELS` (130), `setblock`, `fill`, `container X Y Z [ITEM COUNT]` | `give` reads the item table: `control.rs:948` errors "unknown item"; prefer `command "give"` for `bmh:*` (UNVERIFIED that `give` accepts mod items) |
| State | `state` (32): player, world, screen, target; `status` (reply fields `screen`, `dimension`, `creative`, `player{position,velocity,on_ground,health,max_health,food}`, `yaw`, `pitch`, `held`, `offhand`, `day_time`, `ticks`) `horizon-ctl-commands.md:167-186` | health, position, dimension, day time all there |
| Inventory | `inventory` (39): hotbar, main, armor, offhand, carried stack, item counts; `entities [RADIUS] [TYPE]` (34); `block X Y Z` (35); `looking-at` | whether item components (custom data) are in the reply: UNVERIFIED |
| Events | `events [--since SEQ] [--limit N]` (30). Event kinds from `H/crates/horizon-client/src/game/events.rs:113-160`: `world` (opened/closed), `dimension` (from,to), `screen` (from,to), `death` (with cause), `respawn`, `chat`, `hurt` (amount, health, cause), `heal`, `food`, `item` (item, delta) | a native mod's own state is not an event; use chat or log lines |
| Wait for an event | `wait-for screen NAME\|widget ID\|loaded\|chat TEXT\|event KIND [--timeout S] [--since SEQ]` (11) | e.g. `wait-for event death --since N`, `wait-for chat "text" --since N`; the e2e uses all of these `agent_control_e2e.py:419-441` |
| Passing time | `wait TICKS` (136) | "the death screen's buttons wake after 20 ticks" `agent_control_e2e.py:438` |
| Death screen | `respawn` (101): "Respawn from the death screen"; fallback `ui` (89), `click ID`, `click-button LABEL` (94), `screen` (40) | `control.rs:1275-1278` gates `respawn` on `self.remote.is_some() && Screen::Dead` with the message "respawn needs the death screen on a server", yet the e2e calls it in a single-player world (`:436-441`), so single player apparently runs through a remote-style client. I did not trace why: UNVERIFIED. A BMH journey overlay that replaces the death screen may change what `respawn` sees |
| Read the log | `logs [--lines N] [--crash]` (13); `messages [N]` (41) also returns title and action bar | BMH can print its own lines for the test to read |
| Screenshots | `screenshot PATH` (24): PNG of the next frame | works with `--offscreen`, not with `--headless` |
| Move/aim | `goto X Y Z`, `look YAW PITCH`, `look-at`, `move`, `jump`, `select SLOT\|ITEM`, `interact X Y Z`, `use-item [--hold S]`, `attack`, `place`, `break` | enough for the moon ritual (hold use while looking at the moon) |
| Many commands | `script FILE`, `horizon-ctl mcp` | `script` runs a command file `H/docs/native-client.md:364` |
| Events as a stream | not there yet ("`events --since` polls today") `H/docs/api/agent-control.md` ("Not there yet") | |

Extra: `horizon-ctl help --game` lists "about 160" socket commands "test hooks included" `agent-control.md:15`; a tick script env `HORIZON_TICK_SCRIPT` exists `agent-control.md:325-330`. I did not read their list. `docs/agent-control.md:325` says these runs do not cover "native mods" for `screenshot-sequence`.

### E.4 A second player, a dedicated server

- LAN: `lan open [PORT] [commands] [force]`, `lan close`, `lan status` (117); a guest runs `connect ADDRESS [NAME]` (114) in its own game with its own `--socket` and scratch `HORIZON_HOME`. `H/tools/lan_e2e.py` does exactly this with two real games, each under `horizon-ctl`, and tests damage, death, respawn and dimension change for the guest (see its docstring: hazards, pvp, menu-transfer after death). Each game must have the mod built in; the guest needs the client half.
- Dedicated server with the mod: build with `horizon-launcher native-build FOLDER --server-output DIR --version TAG --trust-native-code`, then `horizon-server --dir SERVER config set engine.path=DIR/horizon-engine` and `horizon-server --dir SERVER run` `H/docs/native-mods.md:2141-2160`. It needs the launcher and an installed prepared base release, and the trust flag must be given by a person (H.3). No source-only way to export a server engine was found: UNVERIFIED.
- `horizon-bots`: vanilla-protocol load bots (offline mode, creative by default, `fight` behaviour in survival) `H/crates/horizon-bots/src/main.rs:1-30`. They can join and move; they do not read or send BMH's mod messages. Not a second scripted player for a death test, except `fight` (they attack mobs and other bots).
- A vanilla-protocol bot such as mineflayer (used for the Java mod) also needs 26.3 support; not checked.

### E.5 Per-test hygiene

Own `HORIZON_HOME`, own `--socket` path (short: `launch` refuses a path too long `launch.rs` test `launch_refuses_a_socket_path_too_long`), `HORIZON_NO_AUDIO=1`, `quit` at the end, delete the world. Plan for the "no second game on the socket": `launch` fails if something already listens (`launch.rs:181-186`).

## F. Multiplayer model

### F.1 The four cases

| Case | What runs | Evidence |
|---|---|---|
| Single player | Window game (client hooks) plus the integrated server's game on another thread (server hooks). `native_tick` runs in both: check `native_window()` | `D/mod-code.txt:22`, `:57`; `H/docs/native-mods.md:5249-5252` |
| LAN host | "the mods run in the host's game and its integrated server, so a server-side hook applies to every player of the world" | `H/docs/native-mods.md:2134` |
| Horizon client joins a dedicated Horizon server with the mod | server half in the exported engine; the joining client runs only its own client hooks ("only client-side hooks do anything; the protocol is unchanged") | `:2136`, `:2137-2139`; export `:2141-2163` |
| Vanilla 26.3 client joins it | allowed by the server for plain gameplay: "Vanilla 26.3 clients and Horizon clients can join it" `D/hosting.txt:2`, `D/multiplayer.txt:45` | see F.3 |

Mod messages between the halves: API 40, channel `horizon:mod_net`, `native_net_send_to_player(player, channel, data)` on the server and `native_net_send_to_server(channel, data)` on the client; channel `mod_id:name`, data at most 30,000 bytes, 32 messages a tick each way per player; "A vanilla client never gets them, and a vanilla server isn't sent any"; what a client sends "is checked by nothing" `H/docs/native-mods.md:4582-4596`. Other client-visible signals: `native_send_particles` to Horizon clients only `:1080`; recipes sync `native_datapack_sync` to Horizon clients, "nothing to a vanilla client" `:1174`; attachments with `sync: true` go to Horizon clients only `:5470-5480`. A client half learns server state only through these, so the BMH `JourneySignal` (vanilla title packets read by a client mixin) would either be reimplemented with `mod_net` or keep working through vanilla title packets (that works for any client brand).

The server also checks the content mods: in the configuration phase it sends id, version and SHA-256 of every `side = "both"` `.hmod` and refuses a Horizon client whose list differs `H/docs/mods.md:113-119`; and a registry identity hash on `horizon:block_registry` `H/crates/horizon-client/src/lan/conn.rs:1421-1459`, message "Content registry mismatch: install the same content mods as the server." `H/crates/horizon-client/src/content_identity.rs:17-18`. "Native mods are not in the list yet: their manifest has no side." `mods.md:119-120`. So both halves must carry the same content `.hmod` (with `side = "both"`) and each side builds its own native code; nothing checks the native halves match.

### F.2 Trust

"The first native install shows a trust notice once. The command needs `--trust-native-code`; the window shows a warning in red before every build" `H/docs/native-mods.md:2110-2116`; the registry lists a native mod only after a maintainer read it `:2117-2120`; native mods "are always on in the launcher" and "play in a game of their own, built for the instance they are installed in" `:5-12`; the server export "is not sandboxed" `:2162`.

### F.3 What a player without the mod sees when the server has mod items

Facts found:
- Mod items get network ids "after vanilla's" (menu, entity type, effect and fluid types say so explicitly, `H/docs/mods.md:346`, `:368`, `:399`, `:441`; for items the registry hash covers it `content_identity.rs:1-5`).
- The server refuses a vanilla-branded client only in two cases: `horizon_clients_only` is set, or custom data components exist (`horizon_protocol::item::custom::entries()`), message "Content registry mismatch ..." `H/crates/horizon-client/src/lan/conn.rs:1331-1347` (comment: "Custom component IDs need Horizon's registry check").
- The content registry check (`horizon:block_registry`) is only sent to clients whose brand is `horizon`: `if brand == "horizon" && let Some(hash) = sync.content_identity` `conn.rs:1421-1422`. A vanilla client is not asked and is not refused by it.
- The docs say a vanilla client "on a server with it may be disconnected when one spawns" for a mod entity type `H/docs/mods.md:405-407`, and "Clients join a server only with the same mods' effects" `:349-350`; the multiplayer sentence for content mods is "a client whose mods add different blocks, items, effects, entity types or menus is refused when it joins" `H/docs/modpacks.md:295-296`. That sentence is about Horizon clients, since the check needs the brand.
- Item defaults changed by a native mod: "Client and server each apply their own mods, so both need the mod. A client without it (a vanilla Java client, or Horizon without the mod) fills in vanilla's defaults for the same stacks" and a stack above 99 "cannot [be] show[n]" `H/docs/native-mods.md:5094-5100`, `:5092-5093`.

Conclusion (UNVERIFIED by test): a vanilla 26.3 client is not stopped at the door by the content check, but it has no `bmh:*` ids in its registry, so item stacks in its inventory or on the ground, and block states in chunks if BMH adds blocks (BMH adds none; the Java mod has items only), would not decode. Expect a disconnect or invisible items, not a graceful "unknown item". Set `horizon_clients_only` on the server (`conn.rs:1331`) to give a clear refusal. The Java BMH has the same limit: its CLAUDE.md says "needed on every client" and vanilla was never supported; so parity is kept by refusing vanilla clients on purpose. Needs a real test with a vanilla client; no such client exists on this Mac.

Particles, sounds and recipes the mod sends to Horizon clients are skipped for vanilla ones (list above), so a vanilla player who does get in sees vanilla gameplay minus BMH visuals.

## G. World compatibility

Question: can a Java BMH world (26.1, `bmh:*` items in inventories, BMH data in player NBT, level game rules) open in Horizon with the Horizon BMH mod and keep its items and data?

| Part | Result | Evidence |
|---|---|---|
| Version upgrade 26.1 to 26.3 data | yes: "A world saved by an older version is upgraded when it is opened, as 26.3 does: the file fixes (FileFixerUpper) and level.dat, then vanilla's DataFixers over every chunk, entity chunk, POI chunk, player, saved data, advancements and statistics file ... Every server data fix is ported, from the saves of 1.8 on" | `H/docs/world-data.md:69` |
| Item ids `bmh:*` in inventories | kept: `item_from_nbt` reads any valid identifier and returns a stack with that id (`"stone"` becomes `minecraft:stone`; only `minecraft:air` is dropped); `player` slots are stored as strings and written back | `H/crates/horizon-items/src/item_nbt.rs:77-96`; `H/crates/horizon-client/src/save.rs:4300-4315`, `:4372-4388` |
| Item components with unknown (`bmh:*`) component types | UNVERIFIED. `StackData` has an `other(..)` bucket used for `minecraft:custom_data` (`item_data.rs:427`) and I did not read how an unknown component type is handled by `decode_components` (`item_data.rs:1248`). A BMH item that carries a custom data component of its own (the Java mod's tooltips and wear are item data) must be tested |
| Unknown player NBT keys (BMH data on the player) | kept: the player compound is `compound_or_empty(self.player_extra.clone())` and only a fixed list of keys is rewritten (`Inventory`, `EnderItems`, `equipment`, `active_effects`, `post_effects`, `Pos`, `respawn`, `recipeBook`, `LastDeathLocation`, `previousPlayerGameType`) | `save.rs:4352-4370` |
| ...but the Horizon mod does not read them as-is | a Horizon mod keeps player data as attachments under `horizon:attachments.<namespace>.<name>` with the shape `{value:<compound>, copy_on_death, sync}`; "Unknown mods' saved entries survive load and save without being decoded" | `H/docs/native-mods.md:5430-5437` |
| BMH layout | The Java mod writes its own keys on the player compound (`C_ServerPlayerMixin` for amulet points and the exile record, see `W/.claude/CLAUDE.md` code map). A migration must read those raw keys once and move them into attachments. There is a `Host::entity_nbt` helper (catalog helper `entity_nbt`) but whether it covers a player is UNVERIFIED (searched the helper list only) |
| Game rules: BMH's `bmh:*` rules | LOST on the next save. On load, `game_rules` saved data keys go into the level's map `save.rs:4143-4152`, but on save "Only rules vanilla knows: an unknown key makes it reject the file", and the file is rewritten from the fixed `GAME_RULES` list `save.rs:4818-4851`. No hook to register a mod game rule was found (searched `game_rule` in the catalog: only `world_game_rule_change`, which hears changes). So the BMH rules need a new store (attachment on the level) and a one-time import of the old values | |
| Level attachments | `data/minecraft/horizon_attachments.dat` of the dimension, not `level.dat` | `native-mods.md:5437-5439` |
| Opening the Horizon world back in Java | not a goal; the `horizon:attachments` entries are plain NBT |
| World opened without the mod | BMH blocks: none. Mod effects are dropped from players and mobs "A world opened without the mod drops the effect" `H/docs/mods.md:349`; mod blocks become air in vanilla, and Horizon "will ask first" is not done (`modpacks.md:297-298`). BMH has effects? No; so only items matter, and they stay as ids in inventories (above) |

Bottom line: items and raw player NBT survive; game rules do not; the BMH-specific data needs a migration step that reads the Java layout and writes attachments. Data pack files: a world's own `datapacks/` folder is read; Horizon "applies most of a data pack's world generation" `H/docs/datapack-upgrade.md` (not relevant to BMH).

## H. Publishing

| Topic | Rule | Evidence |
|---|---|---|
| Path | `https://playhorizon.dev/mods/submit`, sign in with GitHub, upload the `.hnmod` (or `.hmod`), id, version, game 26.3, changelog, page fields (name, summary, Markdown description, authors, licence, source URL, optional 256 KiB square PNG icon) | `D/publish-a-mod.txt:56-64` |
| What happens | The form opens a pull request on `horizon-rust/horizon-mods` adding `mods/<id>.toml`; CI runs `cargo horizon-mod check` and reports | `:64`, `M/README.md` ("Submit") |
| Review | "a maintainer reads every line" of a native mod; "every version of a native mod is read again", even from an owner | `D/publish-a-mod.txt:68`, `:74`; `M/README.md` ("Updates") |
| Licence | public source repository, licence that "lets players use it (MIT, Apache-2.0, GPL and so on, as an SPDX name)"; the entry's licence must equal the repository's; reserved-rights needs a written grant. BMH is Apache-2.0 (`W/LICENSE`) | `D/publish-a-mod.txt:17`; `M/MOD-POLICY.md` ("Licences") |
| Content rules | No Minecraft, Mojang or Microsoft code, assets or data beyond the usage guidelines; "Game-data tables copied from any version count as Mojang data ... Read them at runtime from the player's own files instead". BMH ships its own textures/models/sounds (assets/bmh counted: 44 textures, 43 models, 11 sounds); any copy of vanilla tables in the port must be avoided | `M/MOD-POLICY.md` (promise 3) |
| Updates | same form with a new version, a new `version` each time ("a published file never changes"); with a project key, files must carry a minisign signature | `D/publish-a-mod.txt:14`, `:35-54`, `:72-76` |
| Compatibility CI | newest Nightly with the most downloaded native mods; a version that does not apply is marked `broken_on` in the index | `M/README.md` (`native-compat.yml` row) |
| Pins | The bundle pins an exact Horizon and horizon-mixins revision; "Other revisions have not been validated" | `M/README.md` (paragraph on native source mods) |
| Registry entry examples | `M/mods/minimap.toml`: `kind = "native-mod"`, `file = "minimap-2.0.0.hnmod"`, `sha256`, `game`, `license`, `source`, `owners`, `owner_ids` | |
| Content part | A content mod's entry may turn `native-mod` once and never back (`H/docs/native-mods.md`, "Trust", second bullet, about lines 2117-2125) | |

`--trust-native-code` and agents: "`--trust-native-code` records the operator's decision; an agent must not supply that consent." `H/docs/native-mods.md:2162-2163`. Also `horizon-launcher native-build` has "MCP: none: the player's switch" `H/docs/launcher-commands.md:121`, and the first install shows a trust notice. So no: an agent must not pass it, and in this project the owner has to type it for any `horizon-launcher native-build` run (typed mod builds, server export). `native.py` has no such flag: it applies and compiles code locally under the owner's own command; running it still compiles code the owner wrote, which is the owner's own source and not the launcher's consent step. Whether an agent may run `native.py build` on code that the agent itself wrote is a policy call for the owner; I did not find a rule against it in the repos.

A real blocker for a registry listing is time, not rules: the publish page says a person reads every line of every version, and a 9,000-line port will be a large pull request.

## Risks for the BMH port

Ranked by how likely they are to stop or reshape the port:

1. Prepared resources cannot exist yet. This Mac has no Minecraft 26.3 jar; the cached manifest does not list 26.3 (D.2). Nothing can start (no game, no `horizon-ctl`, no test) until the owner installs 26.3 once and downloads the 26.3 server jar by hand (D.3).
2. Disk. 9.7 GB free, 98 % used. A full loop is about 4 GB plus resources; two profiles, a typed-mod `cargo test` or a second pin can double it. A build is already running in `M/native/target` now (C.3). Plan symlinked caches and delete the debug profile after `check`.
3. Pin and "hooks" number are open. The Nightly is API 411 and main 415; main-only hooks `world_hold`, `world_warp` (client) and helpers `force_chunk*`, `forced_chunks`, plus anything else the feature audits need, would force a non-Nightly pin (as `smp` does at 413) and the launcher then needs the commit to stay kept.
4. Game rules do not persist (G): BMH's six `bmh:*` rules are rewritten away at the first save (`save.rs:4818-4851`) and no registration hook exists, so the six rules, `ALL_RULES` and the create-world toggle need a new design (level attachment plus commands).
5. No example of the combination BMH needs. No shipped native mod uses `data/`, `AttachmentType` or `[api]`; the combined content-plus-code `.hmod` needs a launcher with native `.hmod` support and I found no local packaging tool for it (B.4). Expect to do two artifacts and to hit rough edges.
6. Typed API not usable through `native.py` or agents: its build needs `horizon-launcher native-build --trust-native-code` (owner only). The recommended mixin-helper layout avoids that but cannot unit-test code that touches `Host` (only the pure `core` crate), so the 13 server tests the Java repo has through mineflayer must be re-created through `horizon-ctl`.
7. Vanilla clients. The server does not refuse a vanilla-brand client that lacks the `bmh:*` ids unless `horizon_clients_only` is set (F.3). Set it, and say so in the mod page.
8. Unknown item components of `bmh` type in old worlds: UNVERIFIED (G). A custom data component the Java mod writes could be dropped or kept; needs a test world.
9. `respawn` through `horizon-ctl` is unverified for the BMH journey overlay (E.3). The BMH Java overlay replaces the death screen; the Rust port may have no screen to click, so tests would use `wait-for event respawn`.
10. Per-frame state across threads: BMH has per-player server state and a client overlay. The doc asks for Mutex or atomics (A.1); a `thread_local!` keeps the window's value separate from the server game's value, which is a bug source in single player.
11. Review load. Every version of a native mod is read by a person (H); a 9k-line port per release is slow to publish. Plan for an unlisted local build (`.hnmod` handed to friends) in the meantime.

## Open questions

1. Which hooks and helpers BMH really needs, and their API numbers (this fixes the `hooks` value and whether the pin leaves the Nightly).
2. May the content `.hmod` and the native `.hnmod` share the id `bmh`, or must the native one have its own id? Is there a tool that builds the combined `.hmod` from a bundle folder?
3. Does `native.py check/build` fail on a manifest that has `[api]` (E_MANIFEST)? The launcher strips `[api]` and `[dependencies]` before horizon-mixins `M/scripts/native_compat.py:84-110`, `native.py` does not.
4. Does `native.py package` really include an `[api]` crate's `src/lib.rs` (the code says yes via `rglob`, the page says no)?
5. Does 26.3 show in the official launcher's version list for this account (release or snapshot)? The cached manifest says no.
6. Size on disk of the prepared resources (not documented), and whether `horizon-prepare` clones or copies on this APFS volume.
7. Does `horizon-ctl give` accept `bmh:*` ids? Does `inventory` show item component data? Where is the single-player `respawn` path?
8. Does a vanilla 26.3 client really crash on an unknown item id from a modded Horizon server, and how does a mod message differ from vanilla title packets in what a vanilla client shows?
9. Can the engine export for a dedicated server be built without the launcher (source-only)? I found no such command.
10. Does the player-NBT `Host` access (`entity_nbt`) work for players, so a migration can read the Java mod's raw keys?
11. Is an agent allowed to run `native.py build` on the owner's behalf? No rule found; a policy decision.

## Step log

1. Read the common brief; checked repo root and branch (`horizon-26.3`).
2. Read `D/mod-code.txt` end to end and `M/native/tools/native.py` end to end.
3. Listed the headings of `H/docs/native-mods.md`; read the bundle, crate, data, post-build, trust, multiplayer, standalone export, cache and disk, and trait mod sections plus "The typed API from mixin code" and "Testing a trait mod".
4. Checked `mod_api.rs` for `Host`; found it generated; read the generator loop in `horizon-mod-api-gen/src/lib.rs`.
5. Compared both catalogs with a script (counts, main-only entries, helpers on `Host`); read `NATIVE_HOOKS_API` on main and, with `git show`, on the Nightly tag.
6. Counted lines and listed the files of every native mod in `M/native`; read `smp`, `voicechat-server` and `recipes` manifests and `compatibility.json`.
7. Read the horizon-mixins README for the injection kinds and 0.6.0 features.
8. Read `H/docs/launcher-preparation.md`, `D/install.txt`, `H/README.md` build section, `horizon-prepare/src/main.rs` help and the import code in `artifacts.rs`; checked the Mac's Minecraft folder, rustc, JDK and disk (read only).
9. Read `H/docs/agent-control.md`, `docs/api/agent-control.md`, `horizon-ctl-commands.md`, `launch.rs`, `launch_options.rs`, `events.rs`, parts of `agent_control_e2e.py`, the docstring of `lan_e2e.py`, and the `horizon-bots` header.
10. Read `H/docs/mods.md`, `modpacks.md`, `content-mods.txt` and the join code in `lan/conn.rs` for multiplayer and vanilla clients.
11. Read `world-data.md`, `datapack-upgrade.md`, the player and game rule code in `save.rs` and `item_nbt.rs`.
12. Read `D/publish-a-mod.txt`, `M/README.md`, `M/MOD-POLICY.md`, a registry entry, and `M/scripts/native_compat.py`.
13. Wrote this report. No build, no download, no file written other than this report (I created the `docs/horizon/audit` folder for it).
