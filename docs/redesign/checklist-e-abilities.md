# Checklist: part E, echo abilities (for Louis, in game)

Same checks on every loader and on both sides (1.20.4 and 1.21.1). Use a creative flat world with cheats on,
then switch to survival where a step says so. Give yourself the tools first:

```
/give @s bmh:eternal_sword
/give @s bmh:eternal_axe
/give @s bmh:eternal_pickaxe
```

The server log shows one line at start: `[BalancedMcHardcoreMain] Echo Sense: tag bmh:echo_sense_ores lists 19 blocks.`
(`latest.log`). If it says "empty or missing", stop: the ore tag did not load.
Per-use lines (`Echo Strike: ...`, `Smash: ...`, `Echo Sense: ...`) are DEBUG, in `debug.log` on Forge and NeoForge.

## 1. Sword: Echo Strike

Setup (zombies that stand still, two of them 3 blocks apart):

```
/summon minecraft:zombie ~3 ~ ~ {NoAI:1b,Tags:["bmh_a"]}
/summon minecraft:zombie ~3 ~ ~3 {NoAI:1b,Tags:["bmh_b"]}
```

1. Hold the Eternal Sword and hit zombie A once, waiting until the attack indicator under the crosshair is full.
   You should see: about a quarter of a second after the hit, a cyan dotted line from A to B, small sparks on B, a soft
   chime, and B losing hearts. Check with `/data get entity @e[tag=bmh_b,limit=1] Health`: B had 20, it should now be
   14.5 (the hit does 11, the echo 5.5, a bit less if the zombie wears armor).
2. Hit zombie A with a rushed swing (click again right away, indicator not full). No cyan line, B untouched.
3. Kill A with a full hit, then hit B alone: no line (no other creature around), no error in the log.
4. The echo must never go to: you; another player; a villager (`/summon minecraft:villager ~ ~ ~3 {NoAI:1b}` next to A);
   an armor stand; a wolf you tamed (feed it bones). Each time the line goes elsewhere or nowhere.
5. A cow near the target is a valid victim (the brief reads "any living entity" and lists what is spared). Say so if you
   do not want that: it is one line in `EchoStrikeHandler.isValidVictim`.
6. If the echo kills B, B drops its loot and experience as if you had killed it.
7. The echo never echoes: with three zombies in a row, only the one nearest the target is hurt, not the third.

## 2. Axe: smash

Use an iron golem so the damage shows in its health (100 hp, it does not move with `NoAI`):

```
/summon minecraft:iron_golem ~ ~ ~ {NoAI:1b}
```

Stand 3, 10 or 30 blocks above it (`/tp @s ~ ~10 ~`, you fall from there; a teleport resets the fall), fall onto it and
hit it with the Eternal Axe just before you reach it, with the attack indicator full. Read
`/data get entity @e[type=iron_golem,limit=1] Health` (use `/data merge entity @e[type=iron_golem,limit=1] {Health:100f}` to reset).

| Fall before the hit | Multiplier | Expected loss, hit without crit (sprinting) | Expected loss, vanilla crit (not sprinting) |
|---|---|---|---|
| 3 blocks | x1.69 | 30.5 | 45.7 |
| 10 blocks | x2.26 | 40.8 | 61.2 |
| 30 blocks | x3 (cap, reached at 25 blocks) | 54 | 81 |

1. On a smash you hear an anvil clang and a thud, and see an explosion puff, a ring of cloud and crit sparks at the
   target's feet. The bigger the fall, the denser the dust.
2. Your fall distance is halved by the blow: after hitting a target on the ground from a 30 block fall you take about
   half the fall damage you would have taken. (Check once: note your health before and after landing next to the golem.)
3. No smash, plain 18 damage (27 on a crit that is not a smash): standing on the ground; in water; climbing a ladder or
   vines; gliding with an elytra; falling less than 1.5 blocks.
4. A rushed swing while falling still smashes, but its base damage is already scaled down by vanilla.
5. No trees fall when you chop logs.

## 3. Pickaxe: Echo Sense

Build a small test wall (so the ores are behind stone, not in sight), standing on the flat world:

```
/fill ~2 ~ ~2 ~9 ~4 ~9 minecraft:stone
/setblock ~3 ~1 ~3 minecraft:iron_ore
/setblock ~5 ~1 ~3 minecraft:deepslate_iron_ore
/setblock ~3 ~3 ~5 minecraft:iron_ore
/setblock ~8 ~2 ~8 minecraft:iron_ore
/setblock ~4 ~2 ~4 minecraft:gold_ore
/setblock ~2 ~1 ~3 minecraft:iron_ore
```

1. Mine the iron ore at `~2 ~1 ~3` with the Eternal Pickaxe. You should hear an amethyst chime and see a cyan outline
   around the two other iron ores near it (`~3 ~1 ~3` stone variant and `~5 ~1 ~3` deepslate variant) and `~3 ~3 ~5`,
   through the stone, for 5 seconds. The gold ore does not glow, and neither does the iron ore at `~8 ~2 ~8` if it is
   more than 8 blocks away from the one you mined.
2. After 5 seconds the outlines disappear by themselves.
3. Mine a stone block, then the gold ore with a normal iron pickaxe: nothing glows.
4. Quit to the title screen 2 seconds after an Echo Sense (or leave the server), come back: no ore keeps glowing.
   If one does, `/kill @e[type=block_display,tag=bmh_echo]` removes it, and tell me.
5. At most 64 at once: `/fill ~2 ~ ~2 ~9 ~4 ~9 minecraft:coal_ore` (about 160 ores), break one, then count the glowing
   ones:
   ```
   /scoreboard objectives add bmh dummy
   /execute store result score #n bmh if entity @e[type=block_display,tag=bmh_echo]
   /scoreboard players get #n bmh
   ```
   It must be 64 or less, the ones nearest the block you broke. Break a second ore right away: the glow of the first
   cluster is extended and the count stays 64 or less, no doubled outlines.
6. In the Nether: quartz ore, nether gold ore and ancient debris each light only blocks of their own kind.
7. Creative mode breaks count too. Spectator mode never breaks anything.
