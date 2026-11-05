---
name: verify-loaders
description: Verify a Balanced Minecraft Hardcore branch on Forge, Fabric and NeoForge with nobody at the keyboard (build, jar checks, a dedicated server with bots and a quick-play client per loader, the 1.4.0 upgrade test, the Nether exile bot, a javac check on every Minecraft version). Use before a release, after porting a change to the dev branches, or when asked to make sure the mod works on Forge, Fabric and NeoForge on every version.
---

# Verifying BMH on every loader

Scripts in `.claude/skills/verify-loaders/scripts/`, run with `bash`. They work in worktrees of their own under `.claude/worktrees/` of the main checkout (`verify-<label>`, made by `branch.sh` and reused by `server_only.sh`; `upgrade-old-<label>` and `upgrade-new-<label>`, made by `upgrade.sh`), write logs and screenshots to `$BMH_VERIFY_OUT` (default `/private/tmp/bmh-verify`) and never commit. The client hook's copy in `common/src` is never committed either: `branch.sh` copies it from `scripts/hook/`, which ships with the skill, into the verify worktree and removes it at the end, interrupted or not.

`death_client.sh` is the exception: it runs in the tree you name (uncommitted work included), copies its hook into `common/src` there and removes it at the end, but it leaves `<loader>/run/options.txt` (master volume 0) and a fresh `run/world` behind: delete `options.txt` before you play in that tree yourself.

The worktrees stay after a run, ignored build files included. To clean up: `git worktree remove --force <path>` for each, then `git worktree prune`.

## Setup, once per machine

- Homebrew `openjdk@17` and `openjdk@21` (`BMH_JDK17`, `BMH_JDK21` override), `node`, `python3`, `coreutils` (`timeout`), Pillow for `sheet.py`.
- The bots drive a vanilla client with mineflayer: `npm install --ignore-scripts --prefix <scratch dir> mineflayer@4.39.0`, then `export BMH_BOT_MODULES=<scratch dir>/node_modules`. Every script that boots a server refuses to start without it.

## What runs

| Script | Checks |
|---|---|
| `compile_check.sh [version...]` | `javac -proc:none` of `common/src/main/java` against each Loom jar, no Gradle slot. With no argument it reads `minecraft_version` in the tree's `gradle.properties`: a value starting with `1.21` gives `1.21.1`, anything else gives `1.20 1.20.1 1.20.2 1.20.3 1.20.4`. A failure is a vanilla signature missing on that version. Run it first. |
| `branch.sh <origin branch or tag> <mc version> <label> [loaders]` | A clean-checkout `gradlew build`, then `jarcheck.py` on every jar (mixin configs and classes, refmap names, renamed `@Shadow` fields and added methods, models, textures, recipes and tags in the right version folder, no test class shipped). Then per loader (forge, fabric, and neoforge when it built): a dedicated server on a flat world with `amulet_scenario.js` (28 checks) and `smoke_bot.js` (46: items and recipes, totem nerf, Echo Sense, Echo Strike, distant respawn, anvil step, Eternal Items on the ground, one Totem of Eternity per container, the Totem of Eternity's save), then a quick-play client on a copy of that world with the hook: a star reading, the Eternal Sword in the hand, a save by the Totem of Eternity, four screenshots, `[BMH-SMOKE] stopping`. The hook logs `sword in-hand model differs from the inventory one: true, inventory model kept in the GUI: true` and `after the totem: health=<below 20> slot5=bmh:dormant_totem`: read them, no counter does. |
| `matrix.sh [-f table \| <branch> <mc version> <label> ...]` | `branch.sh` for each cell. No argument: `branches.txt`, one cell per Minecraft version. A run on other branches (a feature ported everywhere) passes its own triples. |
| `upgrade.sh <old ref> <new ref> <mc version> <loader> <label>` | The 1.4.0 upgrade test: `upgrade_bot.js` gives a long-time player's items on the old release (a tag such as `mc-1.20.4-v1.4.0`), then checks on the new code that they carried over and still work (9 checks). |
| `death_client.sh <worktree> <label> <loader> [time] [level type] [immediate] [reload]` | A real death on a quick-play client of a tree as it is on disk (uncommitted work included), with the hook in `scripts/hook/death/`. It prints no PASS line: read `journey.log` for these. The first position packet after the respawn is the far place (a respawn that shows the spawn point first fails here, never in `smoke_bot.js`), the journey overlay ends, the sounds come in order, the player is alive with no screen left. `immediate=1` tests `doImmediateRespawn`, `reload=1` a resource reload during the fade. Frames and `journey.log` in `$BMH_VERIFY_OUT/death-<label>`. |
| `server_only.sh <label> <mc version> <loader> <bot> <name>` | One server of an existing verify worktree and one bot, e.g. `exile_bot.js` (a Nether anchor sends you 8 times fewer blocks away). |
| `sheet.py`, `rcon.py` | A contact sheet of screenshots; a tiny RCON client (`rcon.py <port> <command>`). |

One branch: `bash .claude/skills/verify-loaders/scripts/branch.sh dev-49.0.31 1.20.4 1.20.4`. Every branch: `matrix.sh`. A run nobody watches needs the charger in and the lid open (2026-11-04: a closed lid on 3 % battery slept the Mac while the app held its keep-awake; `caffeinate -i` covers idle sleep only). Before trusting a night's cells, run `pmset -g log | grep 'Entering Sleep'` over the run's window, and rerun any cell whose wall time is far above its siblings'.

## Reading `summary.txt`

One line per cell: `<label> <sha> build=ok jars=250/250 | forge up=1 amulet=28/28 smoke=46/46 srvErr=0 tag=1 client=1 shots=4 cliErr=0 | fabric ... | neoforge ...`. Green is `build=ok`, `jars` all passed, and per loader `up=1`, both bots all passed, `srvErr=0`, `tag=1` (the Echo Sense ore tag loaded with its 19 blocks), `client=1` (the client reached its stop line), `shots=4` and `cliErr=0`. `amulet=-` means the branch has no redesigned amulet. Look at the screenshots (`<label>/<loader>-bmh-smoke-*.png`, `sheet.py` for one sheet): a counter cannot see a broken sky. `build FAILED` stops the cell; read `<label>/build.log`.

`branch.sh` checks out `origin/<branch>`, never your local branch: push first, and compare the sha in `summary.txt` with `git rev-parse --short <branch>`.

## Rules the bots learned

- Quick-play clients run on flat worlds. A copied world with normal terrain can crash Forge 52.1.20 on its first tick: a landing falling block breaks a neighbour whose loot drop asks for Forge's `LootModifierManager` before it is loaded (`Can not retrieve LootModifierManager until resources have loaded once`, no BMH frame in the stack).
- A server's first boot in a fresh worktree runs the plugin's setup: the scripts wait up to 15 minutes for `Done (`. Read the log for it before calling a boot dead.
- Every server boot and quick-play client takes the server lock `/private/tmp/bmh-runserver.lock` first, then a Gradle slot `/private/tmp/bmh-gradle.1` or `.2` (an owner file in each, polled every second; a lock older than 20 minutes whose port is silent is dropped); a build takes a slot only. Other sessions on the machine use the same two, in that order. Ports: 25618 (25619 for the upgrade test, 25620 for `death_client.sh`), RCON at port + 100, password `bmh`; `BMH_PORT` and `BMH_UPGRADE_PORT` override them. The test servers listen on 127.0.0.1 only (`server-ip` in the `server.properties` that `lib.sh` writes), since the RCON password is published here.
- RCON answers before the world has applied the command: wait 10 s after `forceload add` and 2 s after `summon`, or the next `data get entity` says `No entity was found`.
- Effects: `data get entity <sel> active_effects` from 1.20.2, `ActiveEffects` (numeric `Id`) before: read both, as `effects()` in `smoke_bot.js` does. Item NBT in `give` and `summon`: `Count:1b` and `{Enchantments:[...]}` up to 1.20.4, `count:1` and `[minecraft:enchantments=...]` components on 1.21.1. `Found no elements matching ActiveEffects` is the wrong spelling for the version.
- A joining player ignores damage for 60 ticks: the bots wait 4 s before any `damage`. After a distant respawn it is 200 ticks, so a bot that uses `damage` after a death waits 10 s (`kill` goes through). Totem checks: the player with a totem in the off hand and `damage <bot> 1000`, the control on a cow holding one (mobs keep vanilla's totem).
- A `ServerPlayer` made by a temporary self-test has no network connection (`connection` is null): anything that sends a packet (menus, tooltips) needs the bot. The bot is a vanilla client and sees nothing of modded items: `give` and `data get entity` do the reading.
- NeoForge client trap: ModDevGradle takes `--args` as the whole command line, so `--quickPlaySingleplayer` there dies with `Could not find main class or main method. Given main class: --quickPlaySingleplayer`. `branch.sh` adds the arguments to the `client` run in `neoforge/build.gradle` for that run and restores the file at the end.
- Harmless reds: `Couldn't parse element ... bmh:blocks/test_block`, `No key layers in MapLike[{}]` (a flat world without generator settings), the Forge-only netty `Reflective setAccessible(true) disabled`, and the add-mixin skill's refmap and `JAVA_17` lines.
- Content moves: the bots check this release's items and numbers. A new mixin with by-name targets or a `@Shadow` field needs its runtime names in `jarcheck.py` (`REFMAP_IDS`, `SHADOW_FIELDS`, see the add-mixin skill, step 4). A mixin that only adds a method overriding a vanilla one goes in `ADDED_METHODS`. `SHADOW_FIELDS` covers the shadow fields of parts B and E only (`B_AnvilMenuMixin`, `E_ServerPlayerGameModeMixin`; `D_LevelRendererSkyMixin`'s `level` is not checked): a new `@Shadow` needs its runtime names added there.
