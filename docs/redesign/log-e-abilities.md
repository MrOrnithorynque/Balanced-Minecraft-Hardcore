# Log: session E, echo abilities

Part E of `docs/redesign/BRIEF.md`: Echo Strike (sword), smash (axe), Echo Sense (pickaxe), on both sides.

- Main side: branch `redesign/e-abilities`, worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/e-abilities-f3d8e5` (Minecraft 1.20.4, must also compile on 1.20 to 1.20.3).
- 1.21.1 side: branch `redesign/e-abilities-1.21`, worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/e-abilities-f3d8e5-121` (cut from `origin/redesign/integration-1.21`, upstream unset on purpose: push with an explicit refspec only).
- Ports: A 25611, B 25612, C 25613, D 25614, E 25615 (RCON 25715, password `bmh`), G 25617.

## Resume here

State: both sides coded, review fixes applied, committed and pushed (main `redesign/e-abilities` `9d3d050`, 1.21.1 `redesign/e-abilities-1.21` `3079dc9`; the later commits of this log may move the main sha). Headless self-test green on all six loader/side combinations (see Verification item 6; Forge main re-run with the husk fix may still be pending).
The scratch working trees still carry the throwaway self-test (`E_SelfTestMixin` line in `bmh.mixins.json`, folder `selftest/`) as uncommitted changes: strip it with `docs/redesign/e-selftest/remove.sh <abs worktree>` before building the final jars, and never commit it.
Left to do: (1) strip the harness; (2) `git fetch`, merge `origin/redesign/integration` into the main branch and `origin/redesign/integration-1.21` into the 1.21 branch (both were still at the brief commit when last looked); (3) final `sh gradlew build` per side and a plain dedicated-server boot per loader (`runservers.sh ... plain forge fabric neoforge`), check the jars hold `bmh.refmap.json` (Forge and Fabric) and no `SelfTest`; (4) `git merge-tree --write-tree HEAD origin/redesign/<sibling>` for every sibling already pushed (only CHANGELOG.md and bmh.mixins.json conflict, both expected); (5) push, `get_usage`, report with the status line of rule 11.

## Design (calls made, with reasons)

Hooks, all common mixins (nothing in the loaders' shared event classes):

| Hook | Why this one |
|---|---|
| `E_PlayerAttackMixin` on `Player.attack`: HEAD reads the attack strength; `@ModifyArg` on the one `Entity.hurt` call (index 1) scales the hit for the axe and records its damage; `@Inject` before `Player.causeFoodExhaustion` (only reached when the hit landed) fires the echo and the smash effects. | `javap -c` on the vanilla jars of 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4 and 1.21.1: exactly one `Entity.hurt` call (the main target; the sweep uses `LivingEntity.hurt`), exactly one `causeFoodExhaustion`, in the "landed" branch. Vanilla resets the attack strength right after reading it (offset 93), Forge/NeoForge at the very end, so HEAD is the one place that sees the same value on every loader. No `@Redirect`: two mods redirecting the same call crash each other, `@ModifyArg` stacks. |
| `E_ServerPlayerMixin` on `ServerPlayer.tick` | Holds the attacker's pending echoes (a few ticks of delay) and runs them. State lives on the player, so nothing leaks between worlds and no loader tick event is needed. `ServerPlayer.tick` is the per-tick method the level calls (`doTick` is the connection's). |
| `E_ServerPlayerGameModeMixin` on `ServerPlayerGameMode.destroyBlock` | HEAD pushes the block state (air when the main hand is not the Eternal Pickaxe) on a small stack, RETURN pops it; `true` means the block really broke (Forge's break event and protection mods can still refuse). A stack, not a field, because a vein-miner mod calls `destroyBlock` again from inside a break event. The reading cooldown is stored as the next allowed game time (a first version stored the last reading as `Long.MIN_VALUE` and `now - Long.MIN_VALUE` overflowed: Echo Sense never fired). |
| `E_DisplayMixin` on `Display.tick` | Discards Echo Sense displays whose time is up or whose ore is gone, also ones reloaded from disk. |
| `E_DisplayInvoker`, `E_BlockDisplayInvoker` | `@Invoker` for the private `Display.setGlowColorOverride(int)`, `Display.setTransformation(Transformation)` and `BlockDisplay.setBlockState(BlockState)` (private, same names, 1.20 to 1.21.1). |
| `E_MinecraftServerMixin` on `MinecraftServer.loadLevel` (TAIL) | Logs the size of `bmh:echo_sense_ores` once at server start, so a headless boot proves the tag loads. |

Behaviour calls:

- Sword: the echo is queued only when the hit landed (`hurt` returned true), the weapon at HEAD was the Eternal Sword, the attack strength was 0.9 or more, and the attacker is a `ServerPlayer`. Echo damage = half of the float handed to `Entity.hurt` (after crit and enchant bonus, before armor). The echo is applied with `playerAttack(attacker)`, directly through `hurt`, never through `Player.attack`, so it cannot echo.
- Echo victim = an `Enemy` (hostile mob: `net.minecraft.world.entity.monster.Enemy`, on 1.20 to 1.21.1) within 5 blocks of the target, nearest first, minus: the attacker, the target, players, armor stands, villagers and wandering traders (`AbstractVillager`), the attacker's own tamed animals (`OwnableEntity` owned by the attacker), the attacker's vehicle, team allies, dead or invulnerable ones. The brief says "enemy" and lists what is spared; my first reading was "any living entity minus the list", the review pointed out that this would also hit cows, bees, endermen and a village's iron golem during a fight next to them, and a hurt neutral mob turns on the player. The call: `Enemy` only (the listed exclusions stay as written, they cost nothing). To widen it to every mob, drop the first test of `EchoStrikeHandler.isValidVictim`. The echo is shown (line, chime) only if `hurt` returned true.
- Axe: smash when the Eternal Axe is in the main hand, `fallDistance > 1.5`, not on ground, not gliding, not in water, not climbing, not a passenger (the passenger test copies vanilla's crit rule). No attack-strength requirement (the brief names none). Multiplier = `min(3, 1 + sqrt(0.16 * fallDistance))`, applied to the damage handed to `hurt`, so it multiplies vanilla's 1.5x crit too. Fall distance is halved and sound and particles play only if the hit landed.
- Pickaxe: "same ore within 8 blocks" = a sphere of radius 8 around the broken block. Family = the first vanilla ore tag (coal, copper, iron, gold, redstone, lapis, diamond, emerald) containing the broken state; nether quartz ore, nether gold ore and ancient debris match their own block. At most 64 displays alive per area (the displays within reach of a new reading are counted): the nearest matches first, existing Echo Sense displays on the same block only get their expiry extended, so a vein mined for a while keeps glowing a little longer than 5 seconds. A reading is skipped if the same player had one less than 4 ticks ago (a vein-miner mod breaks many ores in one tick). A display is also discarded as soon as the block under it is no longer an Echo Sense ore (a mined, lit ore must not stay as a ghost). The display is drawn a hair inside its block (inset 0.02, scale 0.96): an entity inside an opaque block gets that block's light, none, so a full-size display would turn an exposed ore dark and shimmer against the real faces; inset, the real ore keeps its look and the glow outline still shows through walls. This is untested visually (no client): item 9 of the checklist asks Louis to look.
- Echo displays carry two scoreboard tags: `bmh_echo` and `bmh_echo_until_<expiry game time>`.
- No lang keys: part E has no player-facing string (abilities are not in tooltips: B owns tooltips and its brief gives none for abilities).

## Expected axe damage (base 18 = 17 axe + 1 hand, full strength)

Multiplier = min(3, 1 + sqrt(0.16 * fall)); the cap is reached at a 25 block fall. Vanilla's own jump crit (x1.5) comes first when the player is not sprinting, the smash multiplies the result.

| Fall before the hit | Multiplier | Damage without the vanilla crit (sprinting) | Damage with the vanilla crit |
|---|---|---|---|
| 3 blocks | x1.693 | 30.5 | 45.7 |
| 10 blocks | x2.265 | 40.8 | 61.2 |
| 30 blocks | x3 (cap) | 54.0 | 81.0 |

Fall distance after the hit: half of the fall so far (3 -> 1.5, 10 -> 5, 30 -> 15).

## Review round (clean-context adversarial reviewer, foreground-style, read-only)

Findings, each re-checked by me before acting (file and line quoted by the reviewer, verified in the files):

1. BLOCKER, fixed: cooldown overflow described in the table above. A self-test run would have found it too.
2. Fixed: a mined lit ore left a ghost display for up to 5 s (the block just broken is no longer an ore, so it was neither extended nor discarded). Displays now end with their ore.
3. Decided: victims are `Enemy` mobs (see behaviour calls). My checklist had misquoted the brief ("any living entity"); corrected.
4. Fixed: the echo line and chime played even when invulnerability frames absorbed the echo; echoes are taken out of the queue before delivery (a mod reacting to the hurt could start a swing and modify the list); the break state is a stack.
5. Comment fixed (nether gold ore is in `#minecraft:gold_ores`), the 64 cap is per area (logged), the axe table above added (it was only in the checklist).
6. Not verified by anyone, no client: how the glowing cube looks (see the inset call). Rendering code read statically: `Display.getTeamColor()` returns the glow override, so the outline is cyan.
7. `E_SelfTestMixin` is in the working tree on purpose, never to be committed.

## Verification

Commands and results so far (numbered, no clock times: the harness cannot read the clock).

1. Vanilla bytecode of `Player.attack` on the Loom jars (`javap -p -c`, versions 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4, 1.21.1) and on the NeoForge-patched jars (1.20.4, 1.21.1): exactly one `Entity.hurt` call (offset 420 on 1.20.x, 458 on 1.21.1), one `LivingEntity.hurt` (the sweep), one `causeFoodExhaustion` (offset 1163 on 1.20.x, 1308 on 1.21.1; it sits in the `if (hit landed)` branch), one `resetAttackStrengthTicker` (offset 93 early in vanilla, last in the patched jars).
2. `Display.tick` public, `Display.setGlowColorOverride(int)` and `BlockDisplay.setBlockState(BlockState)` private, same names, on all six versions. `ServerPlayerGameMode.destroyBlock(BlockPos)`, `ServerPlayer.tick()`, `MinecraftServer.loadLevel()` same on all six.
3. `sh gradlew build` on the main side, JDK 17, `--max-workers=2`: BUILD SUCCESSFUL (Forge, Fabric, NeoForge jars). The Forge and Fabric jars contain `bmh.refmap.json` with every injection target of the seven `E_` mixins resolved (SRG `m_...` for Forge, `method_...` for Fabric, including the `@Invoker`s); the Forge manifest has `MixinConfigs: bmh.mixins.json`; the `@Shadow` field `player` is renamed to `f_9245_` (Forge) and `field_14008` (Fabric) inside the shipped mixin class. The NeoForge jar has no refmap, as it should.
4. No new class references `net/minecraft/client`, `com/mojang/blaze3d` or `net/minecraftforge/client` (`javap -v` on all 13 new classes of the Forge jar).
5. Compile matrix: `javac` 17 on the whole `common` source tree (38 files) against the Loom jars of 1.20, 1.20.1, 1.20.2, 1.20.3 and 1.20.4 (libs from the Gradle cache, `-proc:none`): 0 errors on each. `javac` 21 on the 1.21.1 worktree's `common` tree (39 files) against the 1.21.1 jar: 0 errors. This is how "main-side code compiles on 1.20 to 1.20.3 unchanged" is shown without building those branches.
6. Headless self-test (`docs/redesign/e-selftest/`, throwaway, not in any commit of code): a fake `ServerPlayer` (netty `EmbeddedChannel` as its connection) in the overworld, driven by the server ticks, 44 checks per boot. It found nothing wrong with the fixes below and would have caught the cooldown overflow. Results with the fixed code:
   - main side (1.20.4): Fabric 44/44, NeoForge 44/44, Forge 43/44 on its first run. The one miss was the test, not the mod: a zombie 6.4 blocks away "lost 1.0", exactly one fire tick, because zombies burn in daylight; the test now uses husks.
   - 1.21.1 side: Forge 44/44, Fabric 44/44, NeoForge 44/44.
   - Every boot logged `Echo Sense: tag bmh:echo_sense_ores lists 19 blocks.` (11 entries in the file: the 8 vanilla ore tags expand to 17 blocks, gold has three because `#minecraft:gold_ores` includes nether gold ore, plus quartz ore and ancient debris; nether gold ore is listed twice and counted once, so 19) and reached `Done` with the mixins applied (the mixins on `Player`, `ServerPlayer`, `ServerPlayerGameMode`, `Display`, `MinecraftServer` are applied lazily; the test touches every one of them).
   - Numbers the test checks and got: sword hit 11, echo 5.5 on the nearest enemy 5 ticks later, villager/tamed wolf/armor stand/cow/iron golem untouched, echo never echoes, weak swing no echo; axe 45.706 / 61.153 / 81.0 for falls of 3 / 10 / 30 blocks (crit included), fall distance halved, no smash at 1 and 1.5 blocks or on the ground; Echo Sense 3 of 5 irons lit, no ghost after mining a lit ore, gone after 100 ticks, 64 on a coal wall, stale displays discarded.
   - The first runs of the test failed on its own setup (a `Connection` without a channel, then without protocol attributes); both fixed in the test.
