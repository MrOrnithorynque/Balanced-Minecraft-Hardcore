---
name: verify-loaders
description: Verify a Balanced Minecraft Hardcore branch on Forge, Fabric and NeoForge with nobody at the keyboard (build, jar checks, a dedicated server with bots, the server self-tests of every feature, a quick-play client per loader, the 1.4.0 upgrade test, the Nether exile bot, a javac check on every Minecraft version), then a Sonnet reviewer reads the logs and screenshots. Use before a release, after porting a change to the dev branches, or when asked to test the mod, check every feature, or make sure it works on Forge, Fabric and NeoForge on every version.
---

# Verifying BMH on every loader

## The one command

1. Run the matrix in the background: `bash .claude/skills/verify-loaders/scripts/matrix.sh` (every branch of
   `branches.txt`; add `BMH_LOCAL=1` in front to take the local branches, so unpushed commits are tested). For one
   branch: `matrix.sh <branch> <mc version> <label>`. For uncommitted work in a tree, the self-tests alone:
   `selftest.sh --local=<abs tree> <mc version> <label>`. Set `BMH_VERIFY_OUT` to a fresh folder per run. A full matrix
   takes a long time: wait for the task notification, never poll.
2. When it ends, spawn one agent with `model: sonnet` whose prompt is the text of `review-brief.md` (next to this file)
   followed by `OUT=<the BMH_VERIFY_OUT folder>`. Run it in the foreground (or inline in the desktop app, whose Agent
   tool has none): its result is the report.
3. Before you tell the user anything, grep two or three of the report's quotes in the files it names. A finding whose
   quote is not there is dropped. Then report: the verdict line, each red cell with its cause, and the screenshots.

Nothing to install by hand: the bot scripts install mineflayer into `~/.cache/bmh-bots` on first use (see Setup).
Looks and feel stay the user's call: the reviewer describes the screenshots, it does not judge the art.

Scripts in `.claude/skills/verify-loaders/scripts/`, run with `bash`. They work in worktrees of their own under `.claude/worktrees/` of the main checkout (`verify-<label>`, made by `branch.sh` and reused by `server_only.sh`; `upgrade-old-<label>` and `upgrade-new-<label>`, made by `upgrade.sh`), write logs and screenshots to `$BMH_VERIFY_OUT` (default `/private/tmp/bmh-verify`) and never commit. The client hook's copy in `common/src` is never committed either: `branch.sh` copies it from `scripts/hook/`, which ships with the skill, into the verify worktree and removes it at the end, interrupted or not.

`selftest.sh --local=<tree>` changes the tree you name: it deletes `<loader>/run/world` and writes `server.properties` and `eula.txt` there, then removes its hook. Do not point it at a tree whose dev world you want to keep.

`death_client.sh` is the other exception: it runs in the tree you name (uncommitted work included), copies its hook into `common/src` there and removes it at the end, but it leaves `<loader>/run/options.txt` (master volume 0) and a fresh `run/world` behind: delete `options.txt` before you play in that tree yourself.

The worktrees stay after a run, ignored build files included. To clean up: `git worktree remove --force <path>` for each, then `git worktree prune`.

## Setup

- Homebrew `openjdk@17` and `openjdk@21` (`BMH_JDK17`, `BMH_JDK21` override), `node` and `npm`, `python3`, `coreutils` (`timeout`). Pillow only for `sheet.py`, which nothing else needs. The scripts stop with one clear line when one is missing.
- The bots drive a vanilla client with mineflayer. `need_bots` (in `lib.sh`, called by every script that runs a bot) installs `mineflayer@4.39.0` into `~/.cache/bmh-bots` the first time (`npm`, no install scripts) and uses it from then on. `BMH_BOT_MODULES=<node_modules folder>` points it elsewhere. `selftest.sh` and `compile_check.sh` need no bot.

## What runs

| Script | Checks |
|---|---|
| `compile_check.sh [--selftest] [version...]` | `javac -proc:none` of `common/src/main/java` (with `--selftest`, plus the self-test hook and its `MockPlayers` copy for each version) against each Loom jar, no Gradle slot. With no argument it reads `minecraft_version` in the tree's `gradle.properties`: a value starting with `1.21` gives `1.21.1`, anything else gives `1.20 1.20.1 1.20.2 1.20.3 1.20.4`. A failure is a vanilla signature missing on that version. Run it first. |
| `branch.sh <origin branch or tag> <mc version> <label> [loaders]` | A clean-checkout `gradlew build`, then `jarcheck.py` on every jar (mixin configs and classes, refmap names, renamed `@Shadow` fields and added methods, models, textures, recipes and tags in the right version folder, no test class shipped). Then per loader (forge, fabric, and neoforge when it built): a dedicated server on a flat world with `amulet_scenario.js` (28 checks) and `smoke_bot.js` (46: items and recipes, totem nerf, Echo Sense, Echo Strike, distant respawn, anvil step, Eternal Items on the ground, one Totem of Eternity per container, the Totem of Eternity's save), then a quick-play client on a copy of that world with the hook: a star reading, the Eternal Sword in the hand, a save by the Totem of Eternity, four screenshots, `[BMH-SMOKE] stopping`. The hook logs `sword in-hand model differs from the inventory one: true, inventory model kept in the GUI: true` and `after the totem: health=<below 20> slot5=bmh:dormant_totem`: read them, no counter does. |
| `selftest.sh <origin branch or tag \| --local=<abs tree>> <mc version> <label> [loaders]` | The server self-tests (see below): per loader a dedicated server on a fresh flat world with `BMH_SELFTEST=1`, which runs every test of `hook/selftest/Tests.java` with mock players and halts. `selftest-<loader>.log`, `selftest.txt` and one line in `summary.txt`. A branch or tag uses the worktree `verify-<label>`; `--local` uses the tree as it is on disk (no push, no commit needed). Exit 0 only when every check passed. |
| `matrix.sh [-f table \| <branch> <mc version> <label> ...]` | `branch.sh`, then `selftest.sh`, for each cell. No argument: `branches.txt`, one cell per Minecraft version. A run on other branches (a feature ported everywhere) passes its own triples. |
| `upgrade.sh <old ref> <new ref> <mc version> <loader> <label>` | The 1.4.0 upgrade test: `upgrade_bot.js` gives a long-time player's items on the old release (a tag such as `mc-1.20.4-v1.4.0`), then checks on the new code that they carried over and still work (9 checks). |
| `death_client.sh <worktree> <label> <loader> [time] [level type] [immediate] [reload]` | A real death on a quick-play client of a tree as it is on disk (uncommitted work included), with the hook in `scripts/hook/death/`. It prints no PASS line: read `journey.log` for these. The first position packet after the respawn is the far place (a respawn that shows the spawn point first fails here, never in `smoke_bot.js`), the journey overlay ends, the sounds come in order, the player is alive with no screen left. `immediate=1` tests `doImmediateRespawn`, `reload=1` a resource reload during the fade. Frames and `journey.log` in `$BMH_VERIFY_OUT/death-<label>`. |
| `server_only.sh <label> <mc version> <loader> <bot> <name>` | One server of an existing verify worktree and one bot, e.g. `exile_bot.js` (a Nether anchor sends you 8 times fewer blocks away). |
| `sheet.py`, `rcon.py` | A contact sheet of screenshots; a tiny RCON client (`rcon.py <port> <command>`). |

One branch: `bash .claude/skills/verify-loaders/scripts/branch.sh dev-49.0.31 1.20.4 1.20.4`. Every branch: `matrix.sh`. A run nobody watches needs the charger in and the lid open (2026-11-04: a closed lid on 3 % battery slept the Mac while the app held its keep-awake; `caffeinate -i` covers idle sleep only). Before trusting a night's cells, run `pmset -g log | grep 'Entering Sleep'` over the run's window, and rerun any cell whose wall time is far above its siblings'.

## Reading `summary.txt`

One line per cell: `<label> <sha> build=ok jars=250/250 | forge up=1 amulet=28/28 smoke=46/46 srvErr=0 tag=1 client=1 shots=4 cliErr=0 | fabric ... | neoforge ...`. Green is `build=ok`, `jars` all passed, and per loader `up=1`, both bots all passed, `srvErr=0`, `tag=1` (the Echo Sense ore tag loaded with its 19 blocks), `client=1` (the client reached its stop line), `shots=4` and `cliErr=0`. `amulet=-` means the branch has no redesigned amulet. Look at the screenshots (`<label>/<loader>-bmh-smoke-*.png`, `sheet.py` for one sheet): a counter cannot see a broken sky. `build FAILED` stops the cell; read `<label>/build.log`.

Then one line per cell from `selftest.sh`: `<label> <sha> selftest | forge up=1 selftest=<n>/<n> end=1 fail=0 | fabric ... | neoforge ...`. Green is, per loader, `up=1`, `end=1` (the last test ran), `fail=0` and both numbers of `selftest` equal. The failed checks are in `<label>/steps.txt` and `selftest-<loader>.log` (`[BMH-SELFTEST] FAIL <test> / <check> | <detail>`).

`branch.sh` and `selftest.sh` check out `origin/<branch>` unless `BMH_LOCAL=1` is set: then the local branch of that name. Compare the sha in `summary.txt` with `git rev-parse --short <branch>`. A sha that ends in `+local` comes from `selftest.sh --local` on a tree with uncommitted changes (tracked or new files, the `run` folders aside).

## The server self-tests

`hook/selftest/` holds Java sources that `selftest.sh` copies into the tree's `common/src` (package `selftest`, plus the mixin `Z_SelfTestMixin` on `MinecraftServer.tickServer`) and removes at the end. They never reach a commit or a jar: `jarcheck.py` fails a jar with any `selftest` class or `Z_` mixin. With `BMH_SELFTEST=1` the server runs the tests of `Tests.ALL` one after the other from tick 60, prints one `[BMH-SELFTEST] PASS|FAIL` line per check and `END <passed>/<total>`, then halts. Each test gets its own place in the world (64 blocks apart) and its own mock players, and the game rules it sets with `ctx.rule(...)` come back when it ends.

| Test | Checks |
|---|---|
| `GateTest` | The mock player itself: joins, its player tick runs (attack charge), dies, respawns, leaves. If it fails, the other tests say nothing. |
| `RecipeSetTest` | The `bmh` recipes are exactly `EXPECTED` (a stray recipe, like the 2024 blast furnace totem, fails). A new recipe goes into `EXPECTED`. |
| `OneEternalPerTypeTest` | `onlyCarryOneEternalItemType`: a second sword, bundle or totem falls out and stays on the ground (the issue 5 loop), also with the first copy in the off hand; dirt beside it is picked up (control); rule off or creative keeps both. |
| `DeathKeepTest` | Death keeps the Eternal Sword and the Eternal Bundle with its contents, drops the dirt; `damageEternalItemWhenDead` takes exactly 7 % of the durability, none when off. |
| `RespawnTest` | The distant respawn distance follows the formula (config min and max, difficulty), 200 ticks of immunity, the departure title at death and the arrival title after (with the message only when `displayTextOnRespawn` is on); with `randomDeathSpawnPoint` off, a respawn near the spawn, vanilla immunity, no journey title. |
| `SleepTest` | `foodAndHealthRegenOnWakeUp`: the food and health tiers, Haste II and Resistance II with no particles at full health, nothing when off. |
| `CoordinatesTest` | `removeCoordinatesInSurvival` forces `reducedDebugInfo` on while a survival player is online and gives the old value back after. |
| `AxeSmashTest` | The Eternal Axe hits at least 2.5 times harder while falling 10 blocks, and halves the fall. |
| `MenuTest` | Smithing: netherite sword + Totem of Eternity + echo shard gives the Eternal Sword, Sharpness V kept, inputs used up. Anvil: a gold block repairs the Eternal Pickaxe and the left slot empties (no copy). |

The bots keep what needs a real client connection (`smoke_bot.js`: Echo Sense, Echo Strike, the totem nerf, the anvil awakening; `amulet_scenario.js`), and the quick-play clients what needs a screen. Not covered anywhere: the bell other players hear after a respawn, a hardcore world, and other mods (Easy Anvils, JEI).

How the mock players work: `MockPlayers` (one copy per Minecraft version in `hook/selftest/mock/`, picked by `selftest.sh`: 1.20 and 1.20.1, 1.20.2 to 1.20.4, 1.21.1) builds a `ServerPlayer` with a `Connection` on a netty `EmbeddedChannel` and places it with `PlayerList.placeNewPlayer`, like vanilla's `GameTestHelper.makeMockServerPlayerInLevel`. So the loaders' login, tick, death, respawn and wake-up events fire for it, menus work, and every packet it is sent stays readable (`MockPlayers.sent`, `ctx.titles`). The server never ticks a mock player's connection, so `SelfTest` calls its `doTick` every tick. A mock player sends no move packet either, so it holds no chunk tickets where it stands: `SelfTest` forces the 5 x 5 chunks around each test's place and starts the test 20 ticks later, once their entities have loaded. Without that, an item dropped in a chunk next to the origin chunk was alive but invisible to `getEntitiesOfClass` (found on 2026-11-05: on Fabric 1.21.1 a second totem seemed to vanish; a probe proved a sword vanished the same way in that chunk). Calls that changed between 1.20.x and 1.21.1 (title text, enchantments, bundle contents, item data equality) are helpers in each `MockPlayers` copy.

To add a test: a class in `hook/selftest/` that implements `SelfTest.Test`, added to `Tests.ALL`. Vanilla and BMH common code only. Then `compile_check.sh --selftest` on every version (for 1.21.1, `BMH_TREE=<a 1.21.1 tree>`), then `selftest.sh --local=<tree> <mc version> <label> fabric` for a quick run. Rules: the server runs in peaceful, so use passive mobs (iron golems take hits), and peaceful refills food and health on every player tick, so check those in the same step as the action. A check must be able to fail: give it a control (the dirt next to the sword) or check the opposite case with the rule off, and run the new test once on a release from before the fix. A loop can look green in a snapshot: the issue 5 pickup loop showed "1 in the inventory, 1 on the ground" at almost every tick, so `OneEternalPerTypeTest` follows the dropped entity itself. `player.drop` throws an item about 2 blocks forward, out of reach of a player who stands still: move the player onto it. The respawn creates a new `ServerPlayer`: read it back with `ctx.live(player)`.

Proof that the suite catches real bugs (2026-11-05): on the release tag `mc-1.20.4-v1.6.2`, from before the issue 5 fixes, `selftest.sh` fails `recipes / no unexpected recipe` (the blast furnace totem) and the pickup loop checks for the sword, the bundle and the off hand.

## CI

`.github/workflows/build.yml` builds every jar and runs `jarcheck.py` on each push to `main` or a `dev-*` branch: a red run is a build that does not compile or a jar that fails a check (`jarcheck.py` exits 1 on any FAIL). It starts no Minecraft: the servers, bots and clients run here. `publish.yml` runs the same `jarcheck.py` on a tag before it uploads anything (a tag from before this skill is not checked).

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
