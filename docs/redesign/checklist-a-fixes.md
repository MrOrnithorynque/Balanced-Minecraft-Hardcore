# Checklist, part A (fixes and rule tweaks)

What needs eyes in the game. The servers were booted and checked over RCON (see `log-a-fixes.md`); a client was never run.
Do each step once per loader (Forge, Fabric, NeoForge), on the main side (1.20.4) and on 1.21.1, unless a step says
otherwise. Use a singleplayer world with cheats on, Survival, difficulty Normal.

## A2. The Eternal Bundle shows its filled texture (all loaders, both sides)

1. `/give @p bmh:eternal_bundle`
2. Look at the hotbar icon. Expected: the plain bundle texture.
3. Put something in it: pick the bundle up with the cursor in the inventory, right-click a stack of any item (or hold the
   stack and right-click the bundle). Expected: the icon switches to the filled texture (`eternal_bundle_filled.png`) as soon
   as one item is inside, in the inventory, the hotbar and in your hand.
4. Take everything out (right-click the bundle with an empty cursor, or use it in hand to drop the contents). Expected: the
   plain texture again.
5. Drop the full bundle on the ground. Expected: the dropped item shows the filled texture too.

## A3. An anchor exile is the same distance in world terms (all loaders, both sides)

Default config (`minDistanceFromBed` 5000, `maxDistanceFromBed` 60000), difficulty Normal, no Eternal Item in the
inventory: the exile distance is 23333 world blocks.

1. In the Overworld: `/spawnpoint @s 0 100 0`, then `/kill @s`, then click Respawn.
   Expected log line (`<loader>/run/logs/latest.log`, search `Exile distance`):
   `Exile distance: 23333 world blocks, 23333 blocks in minecraft:overworld (coordinate scale 1.0)`.
   You wake about 23333 blocks from the origin (`/data get entity @s Pos`).
2. Go to the Nether: `/execute in minecraft:the_nether run tp @s 0 100 0`. Get a respawn anchor and a glowstone block:
   `/give @p minecraft:respawn_anchor`, `/give @p minecraft:glowstone`. Place the anchor, right-click it with the
   glowstone to charge it, right-click it again with an empty hand ("Respawn point set").
3. `/kill @s`, then click Respawn. Expected log line:
   `Exile distance: 23333 world blocks, 2917 blocks in minecraft:the_nether (coordinate scale 8.0)`.
   You wake in the Nether about 2917 blocks from the anchor (before this fix: 23333 Nether blocks, which is 186664
   Overworld blocks).

## A4. The Totem of Undying gives no Fire Resistance (all loaders, both sides)

1. `/give @p minecraft:totem_of_undying`, hold it in either hand.
2. Optional, to see that old effects go as in vanilla: `/effect give @s minecraft:fire_resistance 1000`.
3. `/damage @s 1000 minecraft:generic`. Expected: the usual totem pop (particles, sound, you stay alive on half a heart).
4. Look at the effect icons. Expected: Regeneration II (about 0:45) and Absorption II (about 0:05). No Fire Resistance, not
   even the one from step 2.
5. Stand in a fire or step in lava (`/setblock ~ ~ ~ minecraft:fire`). Expected: you take fire damage, nothing protects you.

A mob holding a totem is unchanged (the servers checked it: a cow keeps Fire Resistance).

## A1. The Eternal tools take enchantments again (1.21.1 only, all three loaders)

Over RCON each loader already accepted `/enchant ... mending` on a zombie holding the sword, axe and pickaxe. Check the
players' side:

1. `/give @p bmh:eternal_sword`, `/give @p bmh:eternal_axe`, `/give @p bmh:eternal_pickaxe`, and a Mending book:
   `/give @p minecraft:enchanted_book[minecraft:stored_enchantments={levels:{"minecraft:mending":1}}]`
2. Anvil: Eternal Sword on the left, the book on the right. Expected: a result with Mending (before the fix: no result).
   Repeat for the axe and the pickaxe.
3. Enchanting table: put the Eternal Sword and lapis lazuli in it (give yourself levels with `/xp add @s 30 levels`).
   Expected: enchantment offers appear.
4. With the sword in hand, `/enchant @s minecraft:sharpness 5`. Expected: success.
