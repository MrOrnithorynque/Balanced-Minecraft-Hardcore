# Agent hooks: verify-loaders test hooks on Minecraft 26.1.2

Running log. A fresh agent must be able to resume from this file alone.
Worktree: `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/respawn-distance-art-848c97` (branch `mc-26.1`).
Scope: `.claude/skills/verify-loaders/scripts/hook/` and the version switches in the scripts of that skill.
Scratch: `$SCRATCH/hk2/hc.sh <tag> <mock file> <selftest|client|both>` compiles common plus the hooks against the 26.1.2 client jar.

## Step 1: start (2026-11-06 04:0x by `date`, see below)
- Tree guard ok, branch `mc-26.1`, working tree clean, `c26.sh` on common: 0 errors.
- First run of `compile_check.sh --selftest 26.1.2` (1.21.1 mock): fails in the hook only (GameRules, Inventory fields, `kill()`, IronGolem package, MobEffects names, ...).

## Step 2: server self-test hook (done)
- New `hook/selftest/mock/MockPlayers_26.1.java`, picked for any `26.*` (`mock_variant` in `compile_check.sh`, `case "$MCV"` in `selftest.sh`).
- Every call that differs on 26.1 is now a helper with the same signature in all four MockPlayers copies: `kill`, `select`, `setOffhand`, `golem`, `worldSpawn`, `getRule`, `setRule`, `reducedDebugInfoRule`, `bmhRecipes`, `haste`, `resistance`, `quickMove`, `hasTag` (plus the existing `titleText`, `enchantSharpness`, `sharpness`, `fillBundle`, `sameData`).
- Test files now call the helpers. `SelfTest.Ctx.rule` takes an `Object` key (a `GameRules.Key` up to 1.21.1, a `GameRule<Boolean>` on 26.1). Inventory `.items.set` became `getInventory().setItem` (same effect for slots 0..35 on every version).
- Test logic and checks are unchanged. The old copies keep their old code bodies.
- Compile runs (tail lines):
  - `compile_check.sh --selftest 26.1.2` (ROOT): `26.1.2: OK (107 classes)`
  - `BMH_TREE=<1.21.1 tree> compile_check.sh --selftest 1.21.1`: `1.21.1: OK (108 classes)`
  - `BMH_TREE=<1.20.4 tree> compile_check.sh --selftest 1.20 1.20.1 1.20.2 1.20.3 1.20.4`: all five `OK (108 classes)`

## Step 3: 26.1 behaviour of the mock players (found by reading 26.1.2 sources, not by running)
- `ServerPlayer.isInvulnerableTo` is true while `connection.hasClientLoaded()` is false (60 ticks after a join, and after a death until the Respawn command), whatever the damage type. `/kill` would have done nothing to a mock player. `MockPlayers_26.1.place` now sends `ServerboundPlayerLoadedPacket` to the listener, and `respawn` sends the `PERFORM_RESPAWN` command packet through the listener (so `waitingForRespawn` is cleared and the listener holds the new player), then the loaded packet again.
- `ServerPlayer.spawnInvulnerableTime` is gone (commit 513fb5a). `MockPlayers.immunityTicks` exists in all four copies: the old ones read the vanilla field, the 26.1 one reads `bmh$journeyTicks` of `F_ServerPlayerJourneyMixin`. `RespawnTest` is unchanged. Its check "vanilla immunity <= 60" passes with 0 on 26.1.

## Step 4: client hooks (done)
- `SmokeClient`, `death/JourneySmoke` and the three `Z_*` mixins are shared. Mixin targets `Minecraft.tick`, `ClientPacketListener.handleMovePlayer` and `handleRespawn` exist unchanged in 26.1.2.
- New `hook/compat/ClientCompat_1.21.1.java` (1.20 to 1.21.1) and `ClientCompat_26.1.java` (26.x). Methods: `versionName`, `filledBundleCommand`, `gamerule`, `dayTime`, `worldSpawn`, `onCooldown`, `selectSlot`, `soundName`, `press`, `screenshot`, `swordModelLine`. Picked by `client_variant` in `lib.sh` (copied by `branch.sh` and `death_client.sh`, removed in their `unhook`) and in `compile_check.sh`.
- `ClientCompat_26.1.gamerule` maps the old rule names to the 26.1 ids and throws for a name it does not know. `screenshot` queues the grab with `schedule(wrapRunnable(...))` like the removed `Minecraft.tell`. `swordModelLine` resolves the stack for NONE, FIRST_PERSON_RIGHT_HAND and GUI with `ItemModelResolver.updateForTopItem` and compares the sprite name of the first layer.
- `compile_check.sh` got `--client` (compiles the client hooks with the `ClientCompat` copy). `jarcheck.py` THROWAWAY regex got `clientcompat`.
- `SKILL.md`: new section "Version variants of the hooks", JDK list, compile_check row.
- Not changed: `branches.txt` (no `mc-26.1` cell until the branch is on origin), `.claude/CLAUDE.md` (its text about `Minecraft.tell` and `scripts/hook/` is about the 1.20/1.21 hook).

## Final runs (after step 4)
- `compile_check.sh --selftest 26.1.2` (ROOT): `26.1.2: OK (107 classes)`; with `--client`: `26.1.2: OK (113 classes)`
- `BMH_TREE=<mc-1.21.1 tree> compile_check.sh --selftest 1.21.1`: `1.21.1: OK (108 classes)`; with `--client`: `1.21.1: OK (114 classes)`
- `BMH_TREE=<main tree> compile_check.sh --selftest 1.20 1.20.1 1.20.2 1.20.3 1.20.4`: all five `OK (108 classes)`; with `--client`: all five `OK (114 classes)`
- Not run (not allowed): any server, client or Gradle. Everything above proves compilation only.
