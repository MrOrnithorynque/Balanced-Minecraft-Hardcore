---
name: horizon-build-test
description: Compile, build, prepare and test the Horizon port of Balanced Minecraft Hardcore (branch horizon-26.3). Use for any build error of the native bundle, a unit test failure, preparing the game files from Minecraft 26.3, the in-game test (test-slice.sh), a new machine, or an erased cache. Also use when check.sh waits on a lock, when prepare-game.sh refuses the version file, or when asked what a compile proves.
---

# Build and test BMH on Horizon

Read `docs/horizon/ARCHITECTURE.md` ("Module and helper rules", "Build and test") and the section "Resume here" of `docs/horizon/LOG.md` first. Run every command from the repo root of this branch. The scripts are written for `/bin/bash` 3.2 on macOS. Run them with `sh` or `bash`. The shell of the Bash tool is zsh: quote globs, use absolute paths, never `echo ====`.

## The cache

Default: `~/Library/Caches/bmh-horizon`. The variable `BMH_HORIZON_CACHE` changes it for `check.sh`, `prepare-game.sh` and `test-slice.sh`. A restart of the Mac keeps it. A scratchpad under `/private/tmp` does not survive a restart: that is why the cache is the default.

| Piece | Path under the cache | How to make it again |
|---|---|---|
| Horizon's registry clone, with `native/tools/native.py` | `horizon-mods/` | `sh horizon/tools/check.sh` clones it (`git clone --depth 1 https://github.com/horizon-rust/horizon-mods`) when `native/tools/native.py` is missing. |
| The Nightly's source tree (commit in `horizon/native/compatibility.json`, field `horizon_revision`) | `horizon-mods/native/target/horizon/<commit>` | `check.sh` makes it on its first run. `prepare-game.sh` needs it and `research-lock.json` inside it. |
| Cargo's cache and the built game | `horizon-mods/native/target/cargo/fast` (`horizon-client`, `horizon-ctl`) | `sh horizon/tools/check.sh build`. |
| The content checker | `target-tools/debug/cargo-horizon-mod` | In the Nightly's tree: `CARGO_TARGET_DIR=~/Library/Caches/bmh-horizon/target-tools cargo build --locked -p cargo-horizon-mod` (LOG.md, state of 2026-11-17). Then `BMH_HORIZON_MOD_BIN=<that file>`. |
| The 26.3 server jar | `minecraft-server-26.3/server.jar` | A download. Ask the owner first. The size is 62,294,556 bytes and the sha1 is `33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c` (LOG.md). `prepare-game.sh` checks the sha1 against `research-lock.json`. |
| The prepared game files | `resources/26.3` | `prepare-game.sh` (below). Made from the owner's Minecraft. Never commit them, never share them. |
| Logs | `logs/` | `check.sh` writes `<action>-<date>-<pid>.log` there. The progress lines are cut from the screen, so read the full log when an error is unclear. |
| The lock | `check.lock/` (a folder with a `pid` file) | `check.sh` makes and removes it. |

Also needed: Rust (`cargo`; the Nightly's `Cargo.toml` sets `rust-version = "1.95"`; the message in `prepare-game.sh` says 1.90, which is wrong), `python3` (3.11.4 or newer: `native.py` uses `tomllib` and `extractall(filter=...)`), `git`, and the JDK that `research-lock.json` pins (`prepare-game.sh` looks for `/opt/homebrew/opt/openjdk@<major>`; it was `openjdk@25` when last run). A first compile of the bundle takes minutes. A later one takes about 30 seconds. The game build (`check.sh build`) took 3 minutes 13 seconds once (LOG.md).

## The scripts

### `horizon/tools/test-logic.sh [cargo test arguments]`

Runs `cargo test` in `horizon/native` with `RUSTFLAGS="-D warnings"`. It needs no game and no cache. Extra arguments go to `cargo test` (a test name filter, for example). A pass proves the pure logic matches the Java values the tests pin. It does not prove that any hook calls the logic. 106 tests passed at the merge of the first three areas (LOG.md step 3).

### `horizon/tools/check.sh [check|build]`

- `check` (default): runs `python3 native/tools/native.py check <bundle>` inside `horizon-mods`. It compiles the bundle into the Nightly's source. It prints the log without cargo's progress lines. A pass ends with a JSON line that has `"check": "ok"`.
- `build`: runs `native.py build`. It builds the game and `horizon-ctl` with the mod inside, in Horizon's fast profile. The last line it prints is the folder of both binaries.
- Lock: it takes `check.lock` in the cache. If another run holds it, it prints who and waits one second at a time. A lock whose process is dead is taken over. It gives up after 20 minutes.
- A lock folder with no `pid` file is never taken over, so the run waits the full 20 minutes. A failed `git clone` of `horizon-mods` ends the run with git's status. Exit codes: 0 pass; 64 unknown action; 75 gave up on the lock; 130 and 143 interrupted; any other value is the status of `native.py`.
- A pass proves that the code compiles with `-D warnings` against the pinned Horizon source and that the manifest is accepted. It does not prove that a hook fires, when, or for which player.
- Two runs never overlap: the lock serializes them, also across worktrees. Never delete a lock whose process is alive.

### `horizon/tools/content-check.sh`

Runs `cargo-horizon-mod horizon-mod build` in `horizon/content`, then `horizon-mod check target/horizon-mod/bmh.hmod`. The binary comes from `BMH_HORIZON_MOD_BIN` (default `cargo-horizon-mod` on the PATH). Exit 127 if the binary is missing. A pass proves the `.hmod` builds and Horizon's checker accepts its manifest and files. It does not prove that an advancement, a tag or a recipe is in the right folder for the game to read it. Check the folder names by hand: a data file in the wrong folder may be ignored with no error (the content mod uses `data/bmh/advancement/`, `recipe/` and `tags/`).

### `horizon/tools/prepare-game.sh [--minecraft-dir DIR] [--server-jar PATH] [--dry-run]`

Prepares the game files once, from the owner's own Minecraft 26.3. It never downloads Minecraft. It checks four things: the launcher folder (default `~/Library/Application Support/minecraft`: `versions/26.3/26.3.json`, `versions/26.3/26.3.jar`, `libraries/`, `assets/indexes/`), the server jar (`--server-jar` or `HORIZON_SERVER_JAR`, sha1 checked against `research-lock.json`), the JDK, and the Horizon source tree in the cache. For each missing thing it prints a `MISSING:` line and exits 78 without changing anything. When all is there it runs `horizon-prepare --prepare`, then `horizon-prepare sounds`, then `--check`, in the Nightly's tree, and prints `resources/26.3`. `--dry-run` prints the three commands and runs none. Exit codes: 0 done; 78 something is missing; 64 bad option; when `horizon-prepare` fails, the script exits with the status of `cargo run` (the log of the last run ends with `exit=1`). A pass proves the preparer accepted the files and its own check passed. The whole script has not been seen to pass: the last log in `<cache>/logs/prepare-game.log` shows `--prepare` passing and `--check` failing with "the sounds missing", before the script ran `sounds`.

### `horizon/tools/test-slice.sh [options]`

The in-game test of settings, respawn and journey. It drives a real game through `horizon-ctl`, offscreen. Options: `--only a,b` (scenarios a to g), `--keep` (keep the scratch folder `<cache>/slice.XXXXXX`), `--window` (a window, not offscreen), `--no-build` (skip `check.sh build`, use `<cache>/horizon-mods/native/target/cargo/fast`), `--resources DIR`, `--bin-dir DIR`, `--out DIR` (screenshots). Exit codes: 0 all checks passed (SKIPs allowed); 1 a check failed; 77 no prepared files; 3 the game did not start or the build failed; 64 bad option. A trap quits the game and removes the scratch world on every exit.

What each scenario proves and does not prove is in `docs/horizon/TESTING.md` ("The checks of test-slice.sh"). Scenarios: a mods row, `/bmh rule` and `rules.txt`; b distant respawn and its distance; c 200 ticks of immunity; d screenshots of the cinematic (a person must look at them); e rule off; f creative; g the advancement `bmh:journey/exile`. The look, the sound and the feel are never tested by the script: they are the hand table of TESTING.md.

Open questions for the first real run (TESTING.md, last paragraph). Expect to fix the script:

- Does `launch --wait` accept the game arguments after `--wait` as the script writes them?
- Does `click-button respawn` reach the mod's hold of the Respawn click?
- Do the `damage` command and `advancement revoke` exist? (Scenario c skips without `damage`.)
- Does `save` write the advancement file of the world?
- Does the reply of the command `/bmh rule` carry its text? (a2 also reads `messages`.)
- Do the JSON field names the script reads match the real replies? (`state.player.position`, `block_pos`, `health`, `on_ground`, `dimension`, `events.last`, `block.block`, `launch` reply `pid`.)

Also open from GAPS.md section 7: for example whether `run_command` works from `server_tick` on the integrated server, and whether `gui_render_post` covers every frame between death and arrival. Each area log has its own "To check in the game".

## The preparer's refusal of the version file

State on 2026-11-12 (LOG.md, "Resume here"). On 2026-11-17 the file `<Horizon tree>/.local/minecraft/26.3/manifest.json` exists in this cache and `--prepare` passed (the cache log shows it). The file lives inside the Nightly's tree, so an erased cache or a new pinned commit loses it. Before that, `prepare-game.sh` built `horizon-prepare`, then failed with: `versions/26.3/26.3.json is not a 26.3 version file Horizon knows (sha1 9f67e10ed013deeaad20ac3c800652765d493408)`.

- Cause: the launcher's file (or Mojang's republished file) has a checksum that is in the accepted list of neither the Nightly nor Horizon `main` (read at d9b9996, 2026-11-12). A newer Horizon does not help.
- Fix: save Mojang's own file `https://piston-meta.mojang.com/v1/packages/11f1aefe473eefba0af62e3286c33b692807b2eb/26.3.json` (about 45 KB) as `<Horizon tree>/.local/minecraft/26.3/manifest.json`, then run `prepare-game.sh` again. The Horizon tree is `$BMH_HORIZON_CACHE/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`.
- A download needs the owner's word. The owner gave it for the server jar. No file records his word for this version file. Ask first, and state the file name, the source and the size.
- Do not edit the launcher's file. Later work: make the script print this fix when the preparer refuses, or fetch the file behind a flag.

## Rules that break a build

From `ARCHITECTURE.md` ("Module and helper rules", "State", "Logic crate rules") and LOG.md:

- A helper file (`hook_*.rs`) holds exactly one function. Its parameters are the manifest's bindings: same names, same order. Bind only what the function uses. An unused parameter is an error.
- A module function that takes `Game` is `pub(super)`, not `pub`.
- No inner attribute (`#![...]`), no `//!` comment, no `mod other;`, no `include!` in a module or helper file. Use ordinary comments and inline modules.
- Warnings are errors (`RUSTFLAGS="-D warnings"`): no unused import, variable, function or constant. A function nobody calls yet gets a real caller or does not exist.
- Module ids and injection ids share one namespace in horizon-mixins. Injection ids keep the `hook_` prefix (`E_DUPLICATE_ID` otherwise).
- A helper is pasted as a nested function inside the target function, so `crate::` is the target's crate. A module for a hook in `horizon-world`, `horizon-client-render` or the client library can call only a module of that same crate. The `[[crate]]` `bmh_logic` is a dependency of `horizon-client` only: the four modules outside the game module (`bmh_tools_lib.rs`, `bmh_sky_render.rs`, `bmh_totem_world.rs`, `bmh_amulet_world.rs`) cannot call it.
- The logic crate reaches the world only through a trait or closure. No dependency outside std unless the crate and version are in Horizon's `Cargo.lock`.
- Pending hooks (no API number) work only by `target = "<path>::native_<hook>"`. Re-read each target in the Nightly's tree: line numbers and some names differ from Horizon `main`.
- Never call a `game.native_*` helper while a lock is held. No `thread_local!`. Server state is `SERVER_*`, client state is `CLIENT_*`.
- Never an em-dash or an en-dash in a source file or comment.

## When a check fails

1. Read the full log named on the first lines of `check.sh` output (`<cache>/logs/`).
2. Find the hook or helper in the Nightly's tree (`<Nightly tree>/crates/horizon-client/src/game/native_hooks.rs`, large: search it; `<Nightly tree>/docs/native-mods.md` documents every hook; the tree path is in the table above). Read its doc comment and its call site. The name alone is not enough.
3. If you changed one area file and the error is in another file, you may have hit an integrator file. Make the smallest change, and name it in your report.
4. After two or three rounds without progress, stop and get an independent read of the error and the evidence.
