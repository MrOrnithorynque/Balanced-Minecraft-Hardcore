# Log: session E, echo abilities

Part E of `docs/redesign/BRIEF.md`: Echo Strike (sword), smash (axe), Echo Sense (pickaxe), on both sides.

- Main side: branch `redesign/e-abilities`, worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/e-abilities-f3d8e5` (Minecraft 1.20.4, must also compile on 1.20 to 1.20.3).
- 1.21.1 side: branch `redesign/e-abilities-1.21`, worktree `/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/e-abilities-f3d8e5-121` (cut from `origin/redesign/integration-1.21`, upstream unset on purpose: push with an explicit refspec only).
- Ports: A 25611, B 25612, C 25613, D 25614, E 25615 (RCON 25715, password `bmh`), G 25617.

## Resume here

Setup done (branch renamed, 1.21 worktree created). Vanilla study done for 1.20 to 1.21.1. No code written yet. Next: main-side code (mixins + handlers + tag + changelog), then build, then the 1.21.1 port.

## Design (calls made, with reasons)

Hooks, all common mixins (nothing in the loaders' shared event classes):

| Hook | Why this one |
|---|---|
| `E_PlayerAttackMixin` on `Player.attack`: HEAD reads the attack strength; `@ModifyArg` on the one `Entity.hurt` call (index 1) scales the hit for the axe and records its damage; `@Inject` before `Player.causeFoodExhaustion` (only reached when the hit landed) fires the echo and the smash effects. | `javap -c` on the vanilla jars of 1.20, 1.20.1, 1.20.2, 1.20.3, 1.20.4 and 1.21.1: exactly one `Entity.hurt` call (the main target; the sweep uses `LivingEntity.hurt`), exactly one `causeFoodExhaustion`, in the "landed" branch. Vanilla resets the attack strength right after reading it (offset 93), Forge/NeoForge at the very end, so HEAD is the one place that sees the same value on every loader. No `@Redirect`: two mods redirecting the same call crash each other, `@ModifyArg` stacks. |
| `E_ServerPlayerMixin` on `ServerPlayer.tick` | Holds the attacker's pending echoes (a few ticks of delay) and runs them. State lives on the player, so nothing leaks between worlds and no loader tick event is needed. `ServerPlayer.tick` is the per-tick method the level calls (`doTick` is the connection's). |
| `E_ServerPlayerGameModeMixin` on `ServerPlayerGameMode.destroyBlock` | HEAD remembers the block state and whether the main hand is the Eternal Pickaxe, RETURN `true` means the block really broke (Forge's break event and protection mods can still refuse). |
| `E_DisplayMixin` on `Display.tick` | Discards expired Echo Sense displays, also ones reloaded from disk. |
| `E_DisplayInvoker`, `E_BlockDisplayInvoker` | `@Invoker` for the private `Display.setGlowColorOverride(int)` and `BlockDisplay.setBlockState(BlockState)` (private, same names, 1.20 to 1.21.1). |
| `E_MinecraftServerMixin` on `MinecraftServer.loadLevel` (TAIL) | Logs the size of `bmh:echo_sense_ores` once at server start, so a headless boot proves the tag loads. |

Behaviour calls:

- Sword: the echo is queued only when the hit landed (`hurt` returned true), the weapon at HEAD was the Eternal Sword, the attack strength was 0.9 or more, and the attacker is a `ServerPlayer`. Echo damage = half of the float handed to `Entity.hurt` (after crit and enchant bonus, before armor). The echo is applied with `playerAttack(attacker)`, directly through `hurt`, never through `Player.attack`, so it cannot echo.
- Echo victim = any living entity within 5 blocks of the target, nearest first, minus: the attacker, the target, players, armor stands, villagers and wandering traders (`AbstractVillager`), the attacker's own tamed animals (`OwnableEntity` owned by the attacker), the attacker's vehicle, team allies, dead or invulnerable ones. The brief's list is read literally: cows and pigs are valid victims.
- Axe: smash when the Eternal Axe is in the main hand, `fallDistance > 1.5`, not on ground, not gliding, not in water, not climbing, not a passenger (the passenger test copies vanilla's crit rule). No attack-strength requirement (the brief names none). Multiplier = `min(3, 1 + sqrt(0.16 * fallDistance))`, applied to the damage handed to `hurt`, so it multiplies vanilla's 1.5x crit too. Fall distance is halved and sound and particles play only if the hit landed.
- Pickaxe: "same ore within 8 blocks" = a sphere of radius 8 around the broken block. Family = the first vanilla ore tag (coal, copper, iron, gold, redstone, lapis, diamond, emerald) containing the broken state; nether quartz ore, nether gold ore and ancient debris match their own block. At most 64 displays alive per area: the nearest matches first, existing Echo Sense displays on the same block only get their expiry extended.
- Echo displays carry two scoreboard tags: `bmh_echo` and `bmh_echo_until_<expiry game time>`.
- No lang keys: part E has no player-facing string (abilities are not in tooltips: B owns tooltips and its brief gives none for abilities).

## Verification

(nothing yet)
