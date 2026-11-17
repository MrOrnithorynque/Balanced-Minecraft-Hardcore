# Balanced Minecraft Hardcore on Horizon (branch `horizon-26.3`)

Author: MrOrnithorynque. This branch is the port of the Java mod BMH (mod id `bmh`) to Horizon. It is not the Java project. The Java code map of the old CLAUDE.md is `docs/horizon/JAVA-CODE-MAP.md`. To work on the Java mod, use `main` and the `mc-*` branches.

## Where you are, in five lines

- Horizon is Minecraft: Java Edition 26.3 written again in Rust. Java mods do not run in it.
- A content mod (`.hmod`) is data and assets. A native mod is Rust source that Horizon's tools add to the game's source before the game compiles. The game calls the mod at hook points (`native_<hook>` functions of `Game`).
- Gradle, Forge, Fabric, NeoForge, mixins, `mc-*` branches and the Java release flow do not apply here.
- The Java sources stay on this branch as the behaviour reference. They go away when the port is complete.
- The game runs only with files prepared from the owner's own Minecraft 26.3. Without them a compile and the unit tests are all the proof there is.

## Live state: read this first

`docs/horizon/LOG.md`, section "Resume here" (the last entry is the newest). It says what is done, what is open and which usage rules stop the run. Read it before anything else.

## Layout

- `horizon/content/`: the content mod (`horizon-mod.toml`, `data/`, `assets/`, `NOTES.md`).
- `horizon/native/`: the native bundle. `Cargo.toml` (workspace of the logic crate), `compatibility.json` (game, hooks number, Horizon commit, horizon-mixins commit).
- `horizon/native/mixins/bmh/`:
  - `horizon-mixin.toml`: the manifest, one `[[module]]` per module file and one `[[injection]]` per hook.
  - `logic/`: crate `bmh_logic`. Pure Rust, no game type, unit tests. `src/<area>.rs` per area, plus `ids.rs`, `wire.rs` (every mod message), `contract.rs`.
  - `bmh.rs`: ids and small helpers every module uses. `bmh_<area>.rs`: the module of an area, the edge between the game and the logic. `bmh_*_lib.rs`, `bmh_*_render.rs`, `bmh_*_world.rs`: modules for hooks that have no `Game`.
  - `hook_<hook>.rs`: one helper per hook point. It calls the areas in a fixed order.
- `horizon/tools/`: `check.sh`, `test-logic.sh`, `content-check.sh`, `prepare-game.sh`, `test-slice.sh`.
- `docs/horizon/`: `ARCHITECTURE.md` (the contract: decisions, areas, rules), `HOOKS.md` (every wired hook, its order, the contracts between areas, who owns each content file), `GAPS.md` (feature rows C1..S4, tiers, what differs from Java), `LOG.md` (build log and "Resume here"), `TESTING.md` (the in-game test and the hand checks), `briefs/` (the prompts of agents: `port.md` for an area agent), `log/<area>.md` (one log per ported area), `review/*.json` (review findings), `audit/` (the feasibility audits and their verifications), `JAVA-CODE-MAP.md`.

## Commands

Run from the repo root. Use `sh` or `bash`, with absolute paths in scripts you write.

| Command | What a pass proves | Time |
|---|---|---|
| `sh horizon/tools/test-logic.sh` | The unit tests of `bmh_logic` pass, with `-D warnings`. No game. | seconds |
| `sh horizon/tools/check.sh` | The bundle compiles into Horizon's source at the pinned commit (`native.py check`). Not that a hook fires. | about 30 s warm, minutes the first time |
| `sh horizon/tools/content-check.sh` | The content mod builds and `cargo horizon-mod check` accepts it. Needs `BMH_HORIZON_MOD_BIN`. | short |
| `sh horizon/tools/check.sh build` | The game and `horizon-ctl` build with the bundle. It prints the folder of the binaries last. | 3 min 13 s once (LOG.md) |
| `sh horizon/tools/prepare-game.sh --server-jar <jar>` | The game files are prepared in `~/Library/Caches/bmh-horizon/resources/26.3`. | not measured |
| `sh horizon/tools/test-slice.sh` | The in-game checks of settings, respawn and journey. A first run was started on 2026-11-17; its result is not recorded here. | not measured |

The two gates of every change are `test-logic.sh` and `check.sh`. Load the skill `horizon-build-test` for options, exit codes and errors.

## Rules that bind every change

Decisions (full text and reasons: `docs/horizon/ARCHITECTURE.md`, "Decisions"):

- D1: build on the Nightly of 2026-11-08, commit `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`. Read signatures in that tree, not in Horizon `main`.
- D2: mixin helpers on hook points plus one logic crate. No typed `[api]` crate.
- D3: an injection names its hook by `target = "<path>::native_<hook>"`, never by `hook = "<name>"`.
- D4: mod id `bmh`. Item ids are the Java ids, so old worlds keep their items.
- D5: the six BMH game rules are world settings of the mod, `/bmh rule`. Not in `/gamerule`.
- D6: server and client talk through mod messages (all in `wire.rs`), not title packets.
- D7: the server decides. The client draws and asks. The server checks every client request.
- D8: no injection into the game's own functions, except where GAPS.md says so.

Ownership and form:

- One area, one owner. An agent edits only its area files (its `bmh_<area>*.rs` modules, its logic file as `ARCHITECTURE.md` lists it, `docs/horizon/log/<area>.md`, and the content files `HOOKS.md` gives it). The integrator owns the manifest, `hook_*.rs`, `bmh.rs`, `logic/src/lib.rs`, `wire.rs`, `ids.rs`, `contract.rs`, `logic/Cargo.toml`, HOOKS.md and the content files that HOOKS.md ("Content files") does not give to an area. Skill `horizon-port-area` has the details.
- Parity first: numbers, orders, texts and sounds come from the Java source, with the Java `file:line` in a comment and a unit test that pins the value.
- A helper file holds one function. A warning is an error. A panic stops the game: no `unwrap`, `expect` or panicking index on data from the world, a file or a message. Where the hook has a `Game`, check `game.native_on(bmh::ID)` first in the hook helper. `ARCHITECTURE.md` ("Module and helper rules", "State") has the rest.
- A compile proves little. A hook that compiles can fire at another moment, for another player, or not at all. Nothing is done until a run in the game shows it. Each area keeps "To check in the game" in `docs/horizon/log/<area>.md`. Never write that a feature "works" from a compile and unit tests.
- Review is not optional. The respawn area compiled and passed its tests, and two reviewers still found major defects (`docs/horizon/review/`).
- Writing style: Simplified Technical English (ASD-STE100) in comments, logs and messages: short sentences (at most 25 words), active voice, simple present, no idioms. Never an em-dash or an en-dash. Do not rewrite old text only to change its style.
- Changelog: `CHANGELOG.md` at the repo root is the Java mod's, with a section `## [Unreleased]`. Whether Horizon changes get lines there is not decided in any file under `docs/horizon`: ask the owner before you edit it.

## Traps

- Session branch. When the desktop app starts the session again, the coordinator's tree is back on the session's own `claude/...` branch (the Java `main`), and `horizon/` shows as untracked build folders. Nothing is lost: `git switch horizon-26.3`. Run `git branch --show-current` before you edit.
- Worktree base. The app cuts an agent's worktree from the commit the session started on, not from your HEAD. Commit first, give the tip sha as BASE in the prompt, and make the agent reset its throwaway `worktree-*` branch to BASE. `docs/horizon/briefs/port.md` does this.
- One crate compiles as a whole, so a half-written file of one agent breaks every other agent's build in a shared tree. One area, one worktree.
- The cache. The tools live in `~/Library/Caches/bmh-horizon` (`BMH_HORIZON_CACHE` changes it). A restart of the Mac keeps it. A `/private/tmp` scratchpad does not survive a restart. Never commit or share the prepared game files in it.
- `check.sh` holds a lock folder `check.lock` in the cache. A second run waits (up to 20 minutes, then exit 75). Do not run it while another run is in progress, and do not delete a lock whose process is alive.
- No download of Minecraft or Mojang files without the owner's word. `check.sh` fetches Horizon's own sources by itself on a cold cache (`horizon-mods`, the Nightly's tree). `prepare-game.sh` never downloads Minecraft. `/Applications/Horizon.app` is refused by Gatekeeper; only the launcher needs it, the build from source does not.
- Usage rules of the owner's long runs (stop at 60 % of the 5-hour window; weekly 40 % all models and 50 % Fable) are in LOG.md "Resume here" and in `horizon-port-area`.
- Shell is zsh: quote globs, never `echo ====`, `cd` only in a subshell, absolute paths.

## Skills

- `horizon-build-test`: any build error, game preparation, in-game test, a new machine or an erased cache.
- `horizon-port-area`: port, review or fix one area as the coordinator; merge an agent's work; the order of the areas left.
- The old Java skills (`add-mixin`, `port-minecraft-version`, `release`, `verify-loaders`) are for the Java branches. Do not use them here. They exist on `main` and the `mc-*` branches and are planned to leave this branch.
