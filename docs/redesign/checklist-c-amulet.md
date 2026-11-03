# Checklist, part C: Navigator's Amulet (needs eyes in the game)

Do it once per loader (Forge, Fabric, NeoForge), on 1.20.4 and on 1.21.1. Start in survival, in the Overworld, on open ground:
`/gamemode survival`, `/weather clear`, `/give @p bmh:navigator_amulet`. Servers are checked by script (see the log); these
steps are the ones that need the client: icon, needle, tooltip, bar, sounds, the arm swing.
BMH hides the coordinates from F3 in survival: for the checks that compare against coordinates or a facing, run
`/gamerule removeCoordinatesInSurvival false` first, or read `/data get entity @s Pos` and `/data get entity @s Rotation`
(yaw 180 is north, -90 is east, 0 is south, 90 is west).

1. **Look.** In the inventory the amulet shows a violet bar, full. Tooltip: `Navigator's Amulet`, then a violet
   `Starlight 3/3`, then two gray lines (use at night under open sky, sneak and use on a bed or a lodestone). No durability line.
2. **Needle, Overworld.** Hold it and turn on the spot (with F3 open, "Towards negative Z" is north). Facing north: needle
   straight up, red tip up. Facing east: tip to the left. South: down. West: to the right. The needle wobbles a little like a
   compass, and it must not point at spawn (walk away from spawn and check). Drop it: the item on the ground and in an item
   frame shows a needle too.
3. **Needle, other dimensions.** In the Nether and the End the needle spins freely, as a compass does there.
4. **Recipe.** Crafting table: ` S ` / `CKC` / ` Y ` (string, 2 copper ingots, compass, spyglass) gives the amulet. The old
   recipe (string, ender eyes, compass) gives nothing. Check the recipe book too.
5. **Stars hidden, nothing spent.** Right click in the air:
   - `/time set day`: action bar "The stars are hidden: wait for night."
   - `/time set midnight` then `/weather rain`: "The stars are hidden behind the clouds."
   - midnight, clear, under a roof: "The stars are hidden: you need open sky above you."
   - in the Nether: "The stars are hidden: they only answer in the Overworld."
   The bar stays full each time.
6. **A reading.** `/time set midnight`, `/weather clear`, open sky, right click. You hear an amethyst chime, your arm swings,
   the action bar says `X <x> · Y <y> · Z <z>` with your block coordinates (check against `/data get entity @s Pos`), the item bar drops to two
   thirds and the tooltip says `Starlight 2/3`. A second right click within 2 seconds does nothing (the item shows the
   cooldown sweep). Nothing shows Home's coordinates or distance, anywhere. With a shield, a torch or a bucket in the off hand
   and the amulet in the main hand, the click reads the stars and does not raise the shield, place the torch or empty the
   bucket (during the 2 second cooldown the off-hand item does work, like for any vanilla item on cooldown). Also try
   holding the amulet in the off hand with a sword in the main hand: the click still reads the stars.
7. **Charges.** Read two more times (2 seconds apart): `Starlight 0/3`, empty bar. The next right click says
   "No starlight left tonight." and plays nothing. Then `/time add 24000` (a new nightfall went by): within a second the bar
   is full and the tooltip says `Starlight 3/3` again. Sleeping through a night does not refill: only the next dusk does.
8. **Home on a bed.** Place a bed, sneak and right click it holding the amulet: action bar "Home set to this bed.", a lower
   chime, and you do not go to sleep. Same on a lodestone (`/give @p lodestone`): "Home set to this lodestone." Held right
   click does not machine-gun the chime (1 second cooldown). In the Nether, sneak and right click a bed with the amulet:
   "Home can only be set in the Overworld." and the bed does not explode.
9. **The reading contract.** After a reading, `/data get entity @s SelectedItem` shows `bmh_reading:{azimuth:...f,time:...L}`,
   `bmh_starlight:2` and `bmh_night:...L` (on 1.21.1 inside `components."minecraft:custom_data"`). With Home on a bed: stand
   due south of it, azimuth about 0; due north, about 180; Home to your east (you stand west of it), about 90; Home to
   your west, about 270. The sky lines that follow Home come from part D, not from here.
10. **Home survives death and relog.** With Home set, `/kill @s`, respawn. `/data get entity @s bmh_home` still shows the
    same `x`, `y`, `z`. Pick the amulet up where you died (it is not eternal), read the stars: the azimuth still points at
    that Home. Leave and rejoin the world: `bmh_home` is still there.
11. **No Home yet.** On a fresh player (or `/data remove` is not possible on players: use a new world), the reading points
    to your bed if it is in the Overworld, else to the world spawn.
12. **Creative.** In creative a reading spends no charge.
13. **Dedicated server.** Join one: the action bar texts (steps 5 to 8) now show for you. Before this change the coordinates
    never showed on a dedicated server.
14. **README.** The crafting grid picture of the amulet in `README.md` (`resources/navigator_amulet_crafting_grid.png`) still
    shows the old recipe: it needs a new in-game screenshot.
