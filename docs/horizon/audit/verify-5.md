# Verification of audit 5

Paths: `S` = the scratchpad, `H` = `S/src/horizon` (main b880384), `M` = `S/src/horizon-mods`, `X` = `S/src/horizon-mixins`, `D` = `S/horizon-docs`, `W` = the BMH worktree. Guards passed: W root and branch `horizon-26.3`, H at b880384691b88d317553067109d28e785990bda3.

Audit 6 V0 (the single-player owner is a guest of the integrated hub) was checked first. `H/crates/horizon-client/src/game/integrated.rs:823` says "// No player of its own: the hub runs without a host." and `:880` says "host.hub.lan.connect_local()?". Both quotes are there. V0 holds. Audit 5 makes no claim that V0 overturns. It helps one open question (E3).

Method note for tiers on packaging rows: N = nothing beyond the Nightly 2026-11-08 catalog is needed. G = blocked without an owner action or a Horizon change.

## Corrected verdict table

| Row | Feature | Audit said | Corrected | Tier | Mechanism | Changed? |
|---|---|---|---|---|---|---|
| A1 | Layout: one mixin bundle, `[[crate]]` for bulk, content in a separate part | HOOK | HOOK | N | Holds. `smp` (26 injections, 36 modules, 25,928 lines) and `voicechat-server` (`[[crate]]` plus tests) prove the shape. | no |
| A2 | Typed `[api]` route is out because `native.py` does not handle `[api]` | HOOK | HOOK | N | Holds. `native.py` reads no `[api]` key. Registry CI compiles none (`native_compat.py:66-68`). | no |
| A3 | Typed `Host` and `AttachmentType` reachable from mixin helpers | HOOK | HOOK | N | Holds. Doc shows the `dyn Host` coercion. `AttachmentType` is in the Nightly (`horizon-world/src/attachments.rs:65`). | no |
| A4 | `Host` lacks `chat` and `frame_rate`; page text stale for `player`, `playing`, `window`, `text_width` | HOOK | HOOK | N | Holds. Script check of both catalogs: those four are helpers in both, `chat` and `frame_rate` are in neither. | no |
| A5 | Pure-logic `[[crate]]` unit-tested with plain `cargo test` | HOOK | HOOK | N | Holds. `voicechat-server` does it (own `[workspace]`, 15 tests). Do not name the package `core`. | no |
| A6 | Closest models (`smp`, `voicechat-server`, `recipes`, `minimap`); none uses `data/`, `AttachmentType` or `[api]` | HOOK | HOOK | N | Holds. Line counts and the empty search both reproduced. | no |
| B1 | Pin: Nightly fdc87ae (API 411), main is 415 | PINNED | PINNED | F | Holds for Nightly-only work. Any pending hook (tier F) or game function (tier X) forces a main commit, not a Nightly. | no |
| B2 | A mixin can target a PENDING hook function by path on a main commit | (open) | PINNED | F | Works with `target = "<fragment target>"`. `hook = "<name>"` fails (E_HOOK_UNKNOWN) because the catalog lacks it. | yes |
| B3 | horizon-mixins pin `e8cd0aa38cff...` | HOOK | HOOK | N | Holds: 8 of 8 native mods use it. The local clone has only e59da90, so the commit itself is UNVERIFIED. | no |
| B4 | "hooks" number = highest API number used, at most 411 on the Nightly pin | PARTIAL | PARTIAL | F | Holds. Pending hooks have no number, so a raw-target mod cannot declare them. `native.py` stage needs `tree API >= hooks`. | no |
| B5 | Content mod and native mod with the same id ship as one combined `.hmod` | PARTIAL | PARTIAL | N | Same id in ONE file is the design (`mod-files.md:28`). No local tool builds it. Needs native `.hmod` launcher support. | yes |
| B6 | Data files: compiled-in `data/` vs content `.hmod` | HOOK | HOOK | N | Holds. `native_data!` for bytes, content part for items, recipes, models, lang. Advancements in a content mod are not documented. | no |
| C1 | Build loop without the launcher: `native.py check/build/package` | HOOK | HOOK | N | Holds. All steps and line numbers reproduced. `TARGET` is fixed and line 186 overwrites `CARGO_TARGET_DIR`. | no |
| C2 | Disk is a blocker: 9.7 GB free, 98 % used | GAP | HOOK | N | Now false. `df` shows 93Gi available, 79 % used. Not a blocker today. Cache sizes in the audit are plausible. | yes |
| D1 | Owner must run Minecraft 26.3 once and fetch the server jar; this Mac has neither | GAP | GAP | G | Holds. Versions folder has 26.1.2 and 26.2-snapshot-3 only. Manifest file is dated Apr 19 2026. | no |
| D2 | Prepare command and what it reads | HOOK | HOOK | N | Holds. `artifacts.rs:20-22`, `:119-124`, `main.rs:60-64` quotes match. | no |
| E1 | The `horizon-ctl` commands listed for tests exist | HOOK | HOOK | N | Every listed command is in `horizon-ctl-commands.md`. `give bmh:x` is plausible, UNVERIFIED. | no |
| E2 | `--offscreen` and `--headless` run with the display asleep | HOOK | HOOK | N | Holds. `--headless` has no screenshots. An unlocked sleeping display is UNVERIFIED. | no |
| E3 | `respawn` command in single player | PARTIAL | PARTIAL | N | Needs `remote` set and the dead screen (`control.rs:1275-1278`). V0 makes this plausible. Not traced to the assignment. | yes |
| E4 | Dedicated server with the mod needs the launcher (`--trust-native-code`) | GAP | HOOK | N | Refuted in part. `horizon-client --dedicated` is the same game (`server.md:17-19`). A `native.py` build can run it. Needs `eula=true` from the owner. | yes |
| E5 | Two-game LAN tests and `horizon-bots` | HOOK | HOOK | N | Holds. `lan_e2e.py` runs two real games. Bots do not read mod messages. | no |
| F1 | Multiplayer model: four cases | HOOK | HOOK | N | Holds. Single player is a window game plus a hub with the owner as guest (V0). | no |
| F2 | Vanilla client on a server with BMH items | PARTIAL | PARTIAL | N | Holds. Refusal only for native data components or `horizon_clients_only` (`conn.rs:1331-1347`). BMH uses `custom_data`, so no auto refusal. | no |
| F3 | Mod messages `horizon:mod_net`, trust of client data | HOOK | HOOK | N | Holds (`native-mods.md:4582-4596`, API 40). | no |
| G1 | A 26.1 world opens and is upgraded | HOOK | HOOK | N | Holds (`world-data.md:69`). | no |
| G2 | `bmh:*` item ids and item components survive | PARTIAL | HOOK | N | Holds, and better than the audit said. Unknown component types go into `other` and are kept. | yes |
| G3 | Unknown player NBT keys survive; `entity_nbt` can read a player | PARTIAL | HOOK | N | Keys survive (`save.rs:4352-4370`). `player_nbt` returns `level.player_tag()`, which includes `player_extra`. Read only. | yes |
| G4 | BMH game rules are dropped at the first save (`save.rs:4818-4851`) | GAP | GAP | G | Holds, cite exact. No hook registers a mod rule. `level_game_rule` and `level_set_game_rule` (pending) take vanilla rules only. | no |
| H1 | Publishing path, review, licence, content rules | HOOK | PARTIAL | N | Holds, with a conflict in H's own docs about re-review of updates. Every cited line exists. | yes |
| H2 | An agent must not pass `--trust-native-code` | HOOK | HOOK | N | Holds (`native-mods.md:2162-2163`, `launcher-commands.md:121`). `native.py` has no such flag. | no |

## Row notes

### A1: layout
Checked `M/native/smp`: `horizon-mixin.toml` has 26 `[[injection]]` and 36 `[[module]]` (grep -c), 25,928 Rust lines (`find ... | wc -l`), `[lib] path = "src/lib.rs"` in `smp/Cargo.toml:11-12`, and `smp/src/lib.rs:4` `#[path = "../mixins/smp/smp_admin.rs"]` includes the modules so `cargo test` runs them. `voicechat-server` has one `[[crate]] id = "relay"` (`horizon-mixin.toml:12-14`). The layout rules (crate cannot depend on `horizon-client`, no `Cargo.lock` or `[workspace]` in the crate) match `native-mods.md:565-575` and `X/docs/crates-data-and-named-hooks.md:90-112`.

One fix to the audit's tree: the `[[crate]] id="core" path="core"` package must not be called `core`. A package named `core` shadows the standard library name in the crate and `X/docs/crates-data-and-named-hooks.md:110` refuses names that clash. Use `bmh_core`. This is a nit, not a verdict change.

### A2: why the typed `[api]` route is out
Read `M/native/tools/native.py` in full.
- The only manifest reads: `manifest["mod"]["id"]` and `["version"]` (`native.py:52-53`), `compatibility.horizon_revision` (`:199`), `crate` paths (`:204`). No `api`, `dependencies`, `trait_method` string appears (grep `api\]|\[api|trait_method|"api"` over `native.py` found nothing).
- `check` and `build` run `cargo check/build --locked -p horizon-client --bin horizon-client` in the stage (`:175`) after `horizon-mixins apply` (`:163`). An `[api]` crate is never compiled by them.
- The registry side knows `[api]` but only strips it: `M/scripts/native_compat.py:68` `self.api = "api" in manifest or any(isinstance(i, dict) and "trait_method" in i`, and `:87` `LAUNCHER_TABLES = ("api", "dependencies")`. `native.py` does not strip those tables before `horizon-mixins apply`. Whether `apply` refuses an unknown `[api]` table is UNVERIFIED (no run allowed).
- The page quote is exact: `D/mod-code.txt:457` "Not yet. horizon-mods' native.py doesn't know [api] yet: check and build don't handle it, and package leaves the crate's folder out of the .hnmod."
- The audit's side finding is also right: `package` keeps every `.rs` and `.toml` under the bundle (`native.py:205-208`), so an `engine/` crate WOULD be zipped. The page says the opposite. Code wins (docs drift). Not load-bearing for the recommended layout.
- Trap in `package`: `rglob` also takes `.rs` and `.toml` from any `target/` inside the bundle. The audit's layout puts the workspace root at the mod root, so `target/` lands outside the bundle. Keep it that way.

### A3: Host and attachments from helper code
`H/docs/native-mods.md:5271-5300` (heading "The typed API from mixin code"): "On a `Game` method, bind the game and coerce it to `dyn Host`". The example uses a raw `target = "...::Game<'_>::tick_mobs"`. For a catalog hook the same binding works with `hook =`, since `hook` replaces `target` before planning (`X/docs/crates-data-and-named-hooks.md:44`). Attachments: `Holder::Player(u128)` at `horizon-world/src/attachments.rs:47`, `copy_on_death` and `sync` fields at `:70-72`. All three exist at the Nightly tag (`git grep` on the tag found `attachments.rs:47`, `:65`, `:72`).

### A4: Host methods
Script over `H/native-hooks.json` and `S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json`: helpers `player`, `playing`, `window`, `text_width`, `on`, `setting`, `set_setting`, `mod_key`, `fill`, `text`, `give`, `tell`, `feedback`, `set_block`, `net_send_to_player`, `net_send_to_server`, `run_command`, `held`, `entity_nbt`, `entity_merge_nbt`, `kill_mob`, `config_dir`, `world_dir`, `instance_dir`, `built_data`, `send_chat`, `client_action_bar` are in both. `chat`, `frame_rate`, `clicks`, `mapping`, `corner_shown`, `data_dir` are in neither. Counts: main 434 hooks and 177 helpers, Nightly 432 and 174. Main-only hooks `world_hold`, `world_warp`. Main-only helpers `force_chunk`, `force_chunk_in`, `forced_chunks`. Side split on main: server 270, client 62, both 10, unmarked 92. All match the audit.
Generator evidence: `horizon-mod-api-gen/src/lib.rs:1065-1089` loops over every helper and pushes a `Host` declaration. `build.rs:57` also reads the pending fragments, so pending helpers are on `Host` on main without any injection.

### A5: unit tests of a `[[crate]]`
`M/native/voicechat-server/Cargo.toml` is `[workspace] members = ["mixins/voicechat_server/relay"]`, `resolver = "3"`. The relay crate depends on `ring = "0.17"`, `serde`, dev `serde_json`. Tests: `proto.rs` 6, `tests.rs` 9 (grep -c `#[test]`). README line 50: "Tests: `cargo test` here." The crate files are copied by the stage, so the same crate compiles in both places. Registry dependencies must exist in Horizon's `Cargo.lock` (`X/docs/crates-data-and-named-hooks.md:90-95`, `E_CRATE_DEPENDENCY`). So `cargo test` works offline-by-lock only after the first fetch.

### A6: models
Line counts reproduced: smp 25,928, voicechat-server 2,354, recipes 911, minimap 449. `find $M -maxdepth 4 -type d -name data` found nothing. A search for `native_data!` and `AttachmentType` in `*.rs` under `M/native` (excluding target) found nothing.

### B1: pin
`git show nightly-source/2026-11-08-fdc87ae:crates/horizon-client/src/game/native_hooks.rs` has line 1170 `pub(crate) const NATIVE_HOOKS_API: u32 = 411;`. Main: `H/crates/horizon-client/src/game/native_hooks.rs:1086` `= 415`. The tag resolves to `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`.
Rule from `D/mod-manifest.txt:84-90`: a mod on the game's own functions "builds only on the commit it pins ... a Nightly's ... all the pinned mods of one build must pin the same commit". A tier F or X feature needs a raw `target`, so BMH then pins a commit. The doc says "Take a Nightly's". `native.py` downloads any commit (`native.py:105` codeload tarball), but the launcher accepting a non-Nightly main commit is UNVERIFIED. `smp` pins `5de5b6f` and `voicechat-server` pins `f483362`; the clone is shallow so I could not tell if those are Nightlies. Cost: if any BMH feature is tier F or X, the whole mod pins a main commit and is built only by `native.py` for now.

### B2: pending hook by path on a main commit
Checked the horizon-mixins README (`X/README.md:49-63`, `:167-172`) and `X/docs/crates-data-and-named-hooks.md:25-57`.
- `hook = "name"` plus `hook_version` is resolved from the workspace's `native-hooks.json`. The doc says: "`E_HOOK_UNKNOWN` | the name is not in the catalog". A pending hook is in no catalog (`H/tools/native-hook-fragments/README.md`: "A hook in a pending fragment is not API yet; do not depend on it until it has a number"). So `hook =` fails.
- `target = "<path>"` takes a function path (`X/README.md:59`). A pending fragment lists its path. Example `H/tools/native-hook-fragments/living_death_message.json`: `"target": "horizon_client_lib::living_death::native_living_death_message"`. The function exists: `H/crates/horizon-client/src/living_death.rs:9` `pub fn native_living_death_message(`, and its body is `crate::native_client_hooks::living_death_message(entity, kind, message);`. Another: `native_hooks.rs:8476` `pub(super) fn native_wind_burst(` with doc "Pending API @API:wind_burst@."
- Injections go at the head of the body before dispatch (`native-mods.md:5400-5402`: "Mixin injections go at the head of the body, before the dispatch").
- So the claim holds: a mixin can hit a PENDING hook by path on a main commit. Cost: the `hooks` field cannot name it, the mod pins the commit (`native-mods.md:5333-5335`: "A raw `target` ... is not a hook point and may change in any commit: such a mod pins `horizon_revision` and leaves `hooks` out"), and the path of a free function (not `Game` method) starts with `horizon_client_lib::`, so copy the fragment's string exactly. Whether horizon-mixins resolves that crate path is UNVERIFIED (no run).
- Tier of a row that needs such a hook: F.

### B3: horizon-mixins pin
`grep horizon_mixins_revision M/native/*/compatibility.json` gives 8 lines, all `e8cd0aa38cffef49cb2e088ef716baa107148c02`. The doc example shows `0d6b103c...` (`D/mod-manifest.txt`, "compatibility.json" block). Use the revision Horizon's own mods use (the doc says so). `git -C X cat-file -t e8cd0aa...` fails: the clone has one commit (e59da90). UNVERIFIED that e8cd0aa is reachable, but 8 shipped mods depend on it.

### B4: the "hooks" number
`native.py` `stage()` (`:149-151`): `if api is None or api < m.compat["hooks"]: sys.exit(...)`, with `api` read from the tree's `NATIVE_HOOKS_API`. `package` writes `hooks` from `compatibility.json` into `horizon-native.json` (`:196-198`), and the schema calls it optional (`mod-files.md`, field table). `compat["hooks"]` is a required key in `native.py` (KeyError if absent). Shipped mods with raw targets use small numbers: `coords-hud` has `"hooks": 1` and `target = ...native_hud` (`mixins/coords_hud/horizon-mixin.toml:20`). So the practical rule: set `hooks` to the highest API number among CATALOG hooks and helpers used, at most 411 on the Nightly pin, 415 on main. Pending fragments add nothing. The audit's statement is right.

### B5: combined `.hmod`
Quotes found:
- `H/docs/api/mod-files.md:28` "**Both, in one `.hmod`:** the two manifests name the same mod id."
- `mod-files.md:30-31` "A combined file needs a launcher and game with native `.hmod` support; keep publishing code-only `.hnmod`s for older launchers."
- `H/docs/native-mods.md:743-755` "**One `.hmod` for content and code.**"
So the audit's open question 2 ("may both share the id `bmh`") is answered: in one file they MUST share it. Two separate files with the same id is not documented.
Tools: `H/crates/cargo-horizon-mod/src/main.rs:208-215` `build_in` refuses a folder with a `Cargo.toml` ("a Cargo.toml: code mods are native now") and writes a content-only `.hmod` through `Package::write_files(&manifest_text, None, &files)`. `native.py package` writes only `.hnmod`. No tool in the three repos builds the combined file. A hand-built zip is possible but UNVERIFIED. For local tests no combined file is needed: `--dev-mod <content folder>` (`launch_options.rs:325`) plus the `native.py` build.
Also: `cargo horizon-mod build` needs `capabilities` to contain `registries` (`main.rs:213-217`). The audit says so too.

### B6: where data files go
`H/docs/native-mods.md:726-730`: "A bundle's `data/` folder (any files) is staged as `native-data/<mod>/`... `horizon_client::native_data!("<mod id>", "path/in/data")`". Content part: `D/content-mods.txt:104-117` lists recipes, loot tables, tags, models, names (and structures). The audit's list also names advancements. Content mods do not list advancements. Two other routes exist: `native_data_pack_files` (API 106, `native-mods.md:1137`) accepts `advancement/ns:path` files from a native mod, and `native-mods.md:934-935` says a trigger can fire "an advancement of a content mod or a data pack". So advancements are reachable at tier N through the API 106 hook. Not this audit's row; noted for the advancement owner.

### C1: build loop
Re-read `native.py:99-188`. `source()` uses `https://codeload.github.com/{REPO}/tar.gz/{revision}` (a download), `tool()` uses `cargo install --locked --git` (a download). Both are allowed for the owner, not for this verification. `check` runs `RUSTFLAGS="-D warnings"` (`:186`), so any warning in helper code fails the check. Add that to the audit's list of loop facts. `build` prints the path from `binaries / "horizon-client"` (`:257`). `TARGET = NATIVE / "target"` (`:34`) and `env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET / "cargo"), ...)` (`:186`) confirm the audit. The local toolchain: `rustc 1.96.0`, plus a `1.95.0-aarch64-apple-darwin` toolchain (`rustup toolchain list`), `H/Cargo.toml:60` `rust-version = "1.95"`, JDKs 17, 21, 25 under `/opt/homebrew/opt`.

### C2: disk
Audit: "9.7 GB free ... 98 %". Now: `df -h /System/Volumes/Data` prints `460Gi 335Gi 93Gi 79%`. The risk text no longer holds. Another session may have freed space. Re-run `df` before a build. The full-loop size estimate (about 4 GB) stays plausible: `native-mods.md:2060-2062` "`native/target/` about 2 GB" and `:2086-2088` "1.5 GB".

### D1: what the owner must do, and what the Mac has
`~/Library/Application Support/minecraft/versions` has `26.1.2/26.1.2.json`, `26.2-snapshot-3/26.2-snapshot-3.json` (no jars), `jre_manifest.json`, `version_manifest_v2.json`. The manifest file is dated Apr 19 2026. Its `latest` is `{'release': '26.1.2', 'snapshot': '26.2-snapshot-3'}` and no `26.3` id. `H/research-lock.json` pins `"version": "26.3"`. `D/install.txt:15` "Minecraft: Java Edition 26.3, played once." So nothing can start until the owner plays 26.3 once and supplies the server jar. This is the first real blocker (agent cannot do it: it needs a Mojang login and a download).
`artifacts.rs:20-22` quote matches: "Horizon does not download the game: download the {version} server jar yourself". `artifacts.rs:119-124` default macOS path is `~/Library/Application Support/minecraft`.

### D2: prepare
`main.rs:60-64` "Needs Minecraft played once from the official launcher, and Java of the version research-lock.json pins." matches. Time and disk figures are from docs, not re-measured.

### E1: the `horizon-ctl` commands
Every command the audit lists is in `H/docs/horizon-ctl-commands.md` at the cited line: `launch` 10, `wait-for` 11, `wait-ready` 12, `logs` 13, `save` 22, `quit` 23, `screenshot` 24, `events` 30, `state` 32, `entities` 34, `block` 35, `inventory` 39, `screen` 40, `messages` 41, `tp` 58, `click-button` 94, `command` 100, `connect` 114, `lan` 117, `give` 124, `time` 125, `summon` 127, `xp` 130, `wait` 136. Also present: `respawn`, `ui`, `click`, `chat`, `status`, `looking-at`, `goto`, `look`, `look-at`, `move`, `jump`, `select`, `interact`, `use-item`, `attack`, `place`, `break`, `script`, `setblock`, `fill`, `container`, `effect`. None is missing. `command` and `chat` are marked "(network)" in the MCP column.
`give` code: `control.rs:936-947` looks the id up in `self.res.items()` (adds `minecraft:` only when there is no colon) and in single player runs `give @s {item} {count}` through `server.call(... owner_console_command ...)`. So `give bmh:x` works if the content mod's item is in `res.items()`. UNVERIFIED by run. The audit's fallback (`command "give @s bmh:..."`) is the safe path.

### E2: `--offscreen`, `--headless`
`H/docs/horizon-ctl-commands.md:10` "`--offscreen` ... render without showing a window ... locked or headless sessions" and "`--headless` ... no display, window or GPU ... (no screenshots)". `agent-control.md:217-221` quote matches. `tools/agent_control_e2e.py:27-28` "on macOS the game renders offscreen, so it also runs in a locked session or over SSH". `docs/benchmark.md:30` "On a Mac whose session is locked, Horizon draws only with `--offscreen`". Flags are parsed in `launch_options.rs:250-253` (`--headless` also sets `offscreen`). Holds.

### E3: `respawn` in single player
`control.rs:1275-1278`: `if self.remote.is_none() || self.screen != Screen::Dead { return Err("respawn needs the death screen on a server".into()); }`. V0 says the window's game is a client of the integrated hub, so `remote` should be `Some`. I did not trace where `remote` is assigned after `connect_local`, so this stays UNVERIFIED. `agent_control_e2e.py:436-441` uses `kill @s`, `wait ticks=25`, `respawn`, `wait_for event respawn`. One more gap: the BMH journey overlay is a client-side screen the audit has not designed in Rust; whether `respawn` sees `Screen::Dead` under it is open.

### E4: dedicated server with the mod (audit said: needs launcher export)
Refuted in part. `H/docs/server.md:17-19`: "The dedicated server is the game itself, run headless: `horizon-client --resources <prepared> --dedicated [--nogui] [--port N] [--world NAME] [--universe DIR] [--initSettings]`". `launch_options.rs:242` `"--dedicated" => { o.execution_role = ...DedicatedServer; o.dedicated = Some(args.by_ref().collect()); break; }`. The `native.py build` output is that `horizon-client`, with the mods compiled in. `native.py` also has `NATIVE_NO_DEFAULT_FEATURES` for the engine-style build. `native-mods.md:2137-2139` says native mods on a dedicated server run "in a standalone engine export built with `--server-output`, not in an instance build". That sentence describes the launcher's instance builds. It does not say `horizon-client --dedicated` ignores hooks, and the hooks are compiled into `Game`. I did not run it, so the cell is medium confidence.
Rule: `server.md:19-23` "refuses to start until `eula.txt` says `eula=true`". An agent must not accept that for the owner. The owner writes `eula=true` once.

### E5: two games, bots
`tools/lan_e2e.py:1-30` docstring: host world "runs on its integrated server thread", two Horizon clients under `horizon-ctl`, damage, death, respawn, hazards, pvp. `horizon-bots/src/main.rs:1-30` quote: "load-test bots for any Minecraft Java Edition 26.3 server", `--behaviour fight`. They speak vanilla protocol only, so they cannot read `horizon:mod_net`.

### F1 to F3: multiplayer
`native-mods.md:2132-2139` quotes match ("the mods run in the host's game and its integrated server, so a server-side hook applies to every player"). Mod messages: `native-mods.md:4582-4596` (API 40; 30,000 bytes; 32 per tick; "What a client sent is checked by nothing"). Vanilla clients: `lan/conn.rs:1329-1334` is `if brand != "horizon" && (shared.config.horizon_clients_only || horizon_protocol::item::custom::entries().next().is_some())`, then a disconnect. `item/custom.rs:1` says those entries are "Internal wire IDs for native data components". BMH stores its data in `minecraft:custom_data` (W: `handlers/AmuletReading.java:150` uses `CustomData`), so it registers none, and a vanilla client is not refused. `conn.rs:1421-1422` `if brand == "horizon" && let Some(hash) = sync.content_identity` confirms the content check is for Horizon clients only. A vanilla client would then meet unknown item ids. UNVERIFIED by test, as the audit said.

### G1 to G3: world data
- `world-data.md:69` contains "Every server data fix is ported, from the saves of 1.8 on".
- `save.rs:4300-4315` and `:4352-4370` quotes match.
- `item_nbt.rs:77-96`: `item_from_nbt` reads any valid identifier, drops only `minecraft:air`, then `decode_components(tag.get("components"))`.
- The audit left unknown item components UNVERIFIED. Resolved: `horizon-items/src/item_data.rs:1370` `other: Vec::new()`, and in the loop `:1392-1396` `else if !MODELLED.contains(&key.as_str()) && !ominous_part { ... data.other.push((key, value)); }`. Any component key not in `MODELLED` is kept in `other` and written back. Tests at `:1548-1568` keep `minecraft:custom_data` (`{keep:7}`). So BMH's `custom_data` and any `bmh:` component survive.
- Player read: `game/commands/data.rs:67-82` `entity_nbt` and `player_nbt` returns `self.level.player_tag()`, which starts from `player_extra` (`save.rs:4353`). `native_entity_nbt` (`native_entity_data.rs:17`) goes through `entity_nbt`. So the one-time migration can READ the Java mod's raw player keys. `native_entity_merge_nbt` refuses players (`native_entity_data.rs:31-33` doc: "False for a player"), so the migration must write through attachments, as the audit said. I did not trace `Target::Player` inside `entity_nbt_in_level` to its end, so medium confidence.

### G4: game rules
`save.rs:4818-4851` quoted exactly: `if !saved_data_path(dir, "game_rules").exists() || !self.game_rules.is_empty() {`, the comment `// Only rules vanilla knows: an unknown key makes it reject the file.`, then a loop `for (name, default) in GAME_RULES` that writes `minecraft:{name}` keys only. On load (`:4143-4152`) any key goes into the map (`name.strip_prefix("minecraft:").unwrap_or(&name)`), so `bmh:*` keys are read into memory and then not written back. The rules are lost at the first save.
Search for a way around it:
- Catalog hooks and helpers: only `world_game_rule_change` (server, API 247) matches. It hears changes to vanilla rules.
- Pending: `level_world.json` (`level_game_rule`) and `level_world_setters.json` (`level_set_game_rule`). Implementation `game/native_level_world.rs:40` `let r = horizon_client::gamerules::rule(rule)?;` returns None for a name not in the vanilla table. `:86-88` the setter returns false for it.
- `Host` / mod_api: only `GameRuleValue` (`mod_api.rs:47`).
So no route registers or stores a mod game rule. The verdict GAP holds. Workaround stays a level attachment plus own commands, as the audit proposed. Changes in the Horizon source would be needed for a true mod rule. Parity risk: the six rules would not show in `/gamerule` or the create-world screen.

### H1, H2: publishing and trust
All cited lines exist: `D/publish-a-mod.txt:14, 17, 64, 68, 74`; `M/MOD-POLICY.md:32`; `W/LICENSE` is Apache 2.0. One conflict inside the sources: `H/docs/native-mods.md:2117-2120` says each release "is a version once the bundle passes the registry's checks, with no further review". The website page `D/publish-a-mod.txt:74` says "every version of a native mod is read again". The audit used the second. The page is newer in tone ("a maintainer reads every line") but I cannot date either, so plan for re-review and treat the first as the optimistic case.
`--trust-native-code`: `native-mods.md:2163` "`--trust-native-code` records the operator's decision; an agent must not supply that consent." `launcher-commands.md:119-121` ends "MCP: none: the player's switch." Both quotes are in the files. `native.py` has no such flag. The audit's opinion that running `native.py build` on code the agent wrote is a policy question has no rule against it in these repos, so it stays an owner call.

## Claims refuted

1. "No source-only way to export a server engine was found" (E.4, Risk 6, open question 9): `H/docs/server.md:17-19` and `launch_options.rs:242` show the game itself runs as the dedicated server (`horizon-client --dedicated`), so a `native.py` build can host BMH for tests without the launcher or `--trust-native-code`.
2. "Disk: 9.7 GB free, 98 % used" (C.3, Risk 2): `df -h /System/Volumes/Data` now reports 93Gi available and 79 % used.
3. "Unknown item components of `bmh` type in old worlds: UNVERIFIED" (G, Risk 8): `item_data.rs:1392-1396` keeps every component outside `MODELLED` in `other`, so they survive.
4. "`Host::entity_nbt` ... whether it covers a player is UNVERIFIED" (G, open question 10): `commands/data.rs:67-82` returns the player compound including `player_extra`, so reads work (writes by merge do not).
5. "May the content `.hmod` and the native `.hnmod` share the id `bmh`?" (B.4, open question 2): in a combined `.hmod` the id must be the same (`mod-files.md:28`).
6. The page claim "package leaves the crate's folder out of the .hnmod" is false against the code (`native.py:205-208`), as the audit said; this verification agrees.

## Citations that did not match

- `native-mods.md:2063-2066` (audit A.4 point 4, the "48 s" and "11 s" rows): the rows are at `native-mods.md:2072-2075`. Lines 2063-2066 hold the first-build table. Content as quoted is right.
- `native-mods.md:2085` ("64 MB per commit"): line 2085 is the "Rust 1.99.1 453 MB" row. The "Source of one commit, and the stage | 64 MB each" row is a few lines later (`:2087`).
- `mod-files.md:30-34` (combined `.hmod`): the bullets are at `mod-files.md:26-31`.
- `voicechat-server` tests: the audit does not give a count, I found 15 (`proto.rs` 6 plus `tests.rs` 9). No conflict.
- Disk figures (see C2) are stale, not mis-cited.
- Everything else I opened matched: `native.py` lines 34, 52-53, 99-120, 144-188, 191-221, 199-201, 204-208; `native_compat.py:66-68, :87`; `integrated.rs:823, :880`; `save.rs:4143-4152, :4300-4315, :4352-4388, :4818-4851`; `item_nbt.rs:77-96`; `conn.rs:1331-1347, :1421-1422`; `control.rs:1275-1278`; `artifacts.rs:20-22, :119-124`; `horizon-ctl-commands.md` line numbers; `agent_control_e2e.py:27-28, :77-82, :436-441`; `native-mods.md:565-575, :726-756, :2110-2116, :2132-2139, :2162-2163, :4582-4596, :5271-5300, :5402-5440`; `launcher-commands.md:121`; `D/mod-code.txt:11, :22, :57, :69, :325, :327, :429, :443, :451, :452, :457`.

## Hooks and helpers the rows need (name, kind, tier, catalog version)

This group needs few hooks. It names the packaging and test surface.

| Name | Kind | Tier | Where |
|---|---|---|---|
| `tick`, `native_window()` guard | hook, Game helper | N | catalog (hook `tick`), `native_window` is a `Game` method in `D/mod-code.txt:43` |
| `net_from_client`, `net_send_to_player`, `net_send_to_server` | hook, helpers | N | API 40, Nightly |
| `world_game_rule_change` | hook (server) | N | API 247, Nightly |
| `native_data_pack_files` | hook | N | API 106, Nightly (route for advancement or recipe files) |
| `entity_nbt` | helper (reads player data too) | N | API 386, Nightly |
| `AttachmentType`, `Holder::Player`, `copy_on_death`, `sync` | typed API | N | `horizon-world/src/attachments.rs:47, :65-72` on the Nightly tag |
| `level_game_rule`, `level_set_game_rule` | helpers (vanilla rules only) | F | pending fragments `level_world.json`, `level_world_setters.json` |
| any other pending hook | hook | F | needs `target = "<fragment path>"` and a main-commit pin (B2) |
| `world_hold`, `world_warp`, `force_chunk`, `force_chunk_in`, `forced_chunks` | hooks, helpers | M | stamped in main (415), not in the Nightly (411) |
| `chat`, `frame_rate`, `clicks`, `mapping`, `corner_shown` | `Game` helpers (not on `Host`) | N (mixin only) | callable from a helper, not from a typed mod |

## Step log

1. Ran the three guards: W root, W branch, H commit. All matched.
2. Read the brief and the whole audit file.
3. Read `native.py` in full and grepped it and `native_compat.py` for `[api]`.
4. Read the horizon-mixins README and `docs/crates-data-and-named-hooks.md`; found the clone has one commit, so `e8cd0aa` is not in it.
5. Read the fragment folder README and three fragments; confirmed the pending hook functions exist (`living_death.rs:9`, `native_hooks.rs:8476`).
6. Checked `compatibility.json` of all 8 native mods and their manifests for raw targets and `hooks` values.
7. Verified `D/mod-code.txt` quotes (11, 57, 69, 325, 327, 429, 443, 451, 452, 457).
8. Compared both catalogs with a script (counts, helper names, main-only entries, side split).
9. Read `native-mods.md` sections on mixin-to-typed API, attachments, `[[crate]]`, data, combined `.hmod`, trust, multiplayer, server export.
10. Read `cargo-horizon-mod/src/main.rs` `build_in` and `check`.
11. Read `save.rs` (game rules, player, inventory), `item_nbt.rs`, `item_data.rs` `decode_components`, `commands/data.rs`, `native_entity_data.rs`, `native_level_world.rs`.
12. Checked all `horizon-ctl` commands against `horizon-ctl-commands.md`; read `launch_options.rs`, `launch.rs`, `control.rs` (`give`, `respawn`), `agent_control_e2e.py`, `lan_e2e.py`, `horizon-bots` header.
13. Read `lan/conn.rs`, `content_identity.rs`, `item/custom.rs` for the vanilla-client rules.
14. Checked the Mac read-only: Minecraft versions folder, manifest, `df`, toolchains, JDKs.
15. Checked `server.md` and `launch_options.rs:242` for the dedicated mode.
16. Checked the Nightly tag for `NATIVE_HOOKS_API` and for the attachment types.
17. Wrote this file. No build, no download, no other file written.
