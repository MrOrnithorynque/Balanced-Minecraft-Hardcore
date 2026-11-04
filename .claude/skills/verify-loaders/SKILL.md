---
name: verify-loaders
description: Verify a Balanced Minecraft Hardcore branch on Forge, Fabric and NeoForge with nobody at the keyboard (build, jar checks, a dedicated server with bots and a quick-play client per loader, the 1.4.0 upgrade test, the Nether exile bot, a javac check on every Minecraft version). Use before a release, after porting a change to the dev branches, or when asked to make sure the mod works on Forge, Fabric and NeoForge on every version.
---

# Verifying BMH on every loader

Scripts in `.claude/skills/verify-loaders/scripts/`, run with `bash`. They work in their own worktrees (`.claude/worktrees/verify-<label>` of the main checkout), write logs and screenshots to `$BMH_VERIFY_OUT` (default `/private/tmp/bmh-verify`) and never commit. Do not commit the client hook either (`scripts/hook/`): `branch.sh` copies it into its verify worktree and removes it at the end, interrupted or not.

## Setup, once per machine

- Homebrew `openjdk@17` and `openjdk@21` (`BMH_JDK17`, `BMH_JDK21` override), `node`, `python3`, `coreutils` (`timeout`), Pillow for `sheet.py`.
- The bots drive a vanilla client with mineflayer: `npm install --ignore-scripts --prefix <scratch dir> mineflayer@4.39.0`, then `export BMH_BOT_MODULES=<scratch dir>/node_modules`. Every script that boots a server refuses to start without it.

## What runs

| Script | Checks |
|---|---|
| `compile_check.sh [version...]` | `javac -proc:none` of `common/src/main/java` against each Loom jar (1.20 to 1.20.4; `1.21.1` on `dev-52.1.20`), no Gradle slot. A failure is a vanilla signature missing on that version. Run it first. |
| `branch.sh <origin branch or tag> <mc version> <label> [loaders]` | A pristine `gradlew build`, then `jarcheck.py` on every jar (mixin configs and classes, refmap names, renamed `@Shadow` fields, models, textures, recipes and tags in the right version folder, no test class shipped). Then per loader (forge, fabric, and neoforge when it built): a dedicated server on a flat world with `amulet_scenario.js` (28 checks) and `smoke_bot.js` (32: items and recipes, totem nerf, Echo Sense, Echo Strike, distant respawn, anvil step), then a quick-play client on a copy of that world with the hook: a star reading, two screenshots, `[BMH-SMOKE] stopping`. |
| `matrix.sh [-f table \| <branch> <mc version> <label> ...]` | `branch.sh` for each cell. No argument: `branches.txt`, one cell per Minecraft version. A run on other branches (a feature ported everywhere) passes its own triples. |
| `upgrade.sh <old ref> <new ref> <mc version> <loader> <label>` | The 1.4.0 upgrade test: `upgrade_bot.js` gives a long-time player's items on the old release (a tag such as `mc-1.20.4-v1.4.0`), then checks on the new code that they carried over and still work (9 checks). |
| `server_only.sh <label> <mc version> <loader> <bot> <name>` | One server of an existing verify worktree and one bot, e.g. `exile_bot.js` (a Nether anchor sends you 8 times fewer blocks away). |
| `sheet.py`, `rcon.py` | A contact sheet of screenshots; a tiny RCON client (`rcon.py <port> <command>`). |

One branch: `bash .claude/skills/verify-loaders/scripts/branch.sh dev-49.0.31 1.20.4 1.20.4`. Every branch: `matrix.sh`. A run nobody watches, the matrix above all, goes as `caffeinate -i bash .../matrix.sh`: the app's keep-awake lets go minutes after the session idles, and a sleeping Mac suspends every timeout (a night once gave a 6 h 56 min cell with invalid numbers). Rerun any cell whose wall time is far above its siblings'.

## Reading `summary.txt`

One line per cell: `<label> <sha> build=ok jars=148/148 | forge up=1 amulet=28/28 smoke=32/32 srvErr=0 tag=1 client=1 shots=2 cliErr=0 | fabric ... | neoforge ...`. Green is `build=ok`, `jars` all passed, and per loader `up=1`, both bots all passed, `srvErr=0`, `tag=1` (the Echo Sense ore tag loaded with its 19 blocks), `client=1` (the client reached its stop line), `shots=2` and `cliErr=0`. `amulet=-` means the branch has no redesigned amulet. Look at the screenshots (`<label>/<loader>-bmh-smoke-*.png`, `sheet.py` for one sheet): a counter cannot see a broken sky. `build FAILED` stops the cell; read `<label>/build.log`.

## Rules the bots learned

- A server's first boot in a fresh worktree runs the plugin's setup: the scripts wait up to 15 minutes for `Done (`. Read the log for it before calling a boot dead.
- Every boot and Gradle run takes the server lock `/private/tmp/bmh-runserver.lock` first, then a Gradle slot `/private/tmp/bmh-gradle.1` or `.2` (an owner file in each, polled every second; a lock older than 20 minutes whose port is silent is dropped). Other sessions on the machine use the same two, in that order. Ports: 25618 (25619 for the upgrade test), RCON at port + 100, password `bmh`; `BMH_PORT` and `BMH_UPGRADE_PORT` override them.
- RCON answers before the world has applied the command: wait 10 s after `forceload add` and 2 s after `summon`, or the next `data get entity` says `No entity was found`.
- Effects: `data get entity <sel> active_effects` from 1.20.2, `ActiveEffects` (numeric `Id`) before: read both, as `effects()` in `smoke_bot.js` does. Item NBT in `give` and `summon`: `Count:1b` and `{Enchantments:[...]}` up to 1.20.4, `count:1` and `[minecraft:enchantments=...]` components on 1.21.1. `Found no elements matching ActiveEffects` is the wrong spelling for the version.
- A joining player ignores damage for 60 ticks: the bots wait 4 s before any `damage`. Totem checks: the player with a totem in the off hand and `damage <bot> 1000`, the control on a cow holding one (mobs keep vanilla's totem).
- A `ServerPlayer` made by a temporary self-test has no network connection (`connection` is null): anything that sends a packet (menus, tooltips) needs the bot. The bot is a vanilla client and sees nothing of modded items: `give` and `data get entity` do the reading.
- NeoForge client trap: ModDevGradle takes `--args` as the whole command line, so `--quickPlaySingleplayer` there dies with `Could not find main class or main method. Given main class: --quickPlaySingleplayer`. `branch.sh` adds the arguments to the `client` run in `neoforge/build.gradle` for that run and restores the file at the end.
- Harmless reds: `Couldn't parse element ... bmh:blocks/test_block`, `No key layers in MapLike[{}]` (a flat world without generator settings), the Forge-only netty `Reflective setAccessible(true) disabled`, and the add-mixin skill's refmap and `JAVA_17` lines.
- Content moves: the bots check this release's items and numbers. A new mixin with by-name targets or a `@Shadow` field needs its runtime names in `jarcheck.py` (`REFMAP_IDS`, `SHADOW_FIELDS`, see the add-mixin skill, step 4).
