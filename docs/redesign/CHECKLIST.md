# Checklist for Louis: the item redesign, one client session per loader

Nobody could run a game client during this run, so everything below needs your eyes. The server side (anvil, smithing,
recipes, tooltips as text, echo abilities with exact numbers, readings, charges, registration on a dedicated server) was run
headlessly on every loader of both sides: see the `log-*.md` files. This list is the six per-part checklists
(`checklist-a-fixes.md` to `checklist-f-sprites.md`) merged into one order, so that you stay in one world per session.

Do it top to bottom, once per loader (Forge, Fabric, NeoForge) on 1.20.4 (the `main` side) and once per loader on 1.21.1:
six sessions. Steps marked **[1.21.1]** only exist there. Commands that differ between the two sides are given for both.
The per-part files stay as the detailed source (they explain every number); where they disagree with this list, this list
is right (it knows about the five integration calls at the end).

## 0. Setup (once per session)

1. New world: Superflat, Creative, cheats on, difficulty Normal, the Game tab's "Balanced Hardcore" button ON. Keep a terminal
   on `<loader>/run/logs/latest.log` (the folder of the jar you launched).
2. Run: `/gamerule removeCoordinatesInSurvival false` (BMH hides F3 coordinates in survival), `/gamerule doDaylightCycle false`,
   `/gamerule doWeatherCycle false`, `/time set day`, `/weather clear`.
3. In the log, search `bmh`. Expected: `Echo Sense: tag bmh:echo_sense_ores lists 19 blocks.` ("empty or missing" means the ore
   tag did not load: stop and tell me), the `self-check` lines of the amulet and of the sky code say what they expected
   (azimuth 0 for north, 90 for east), and no mixin error or `ERROR` line mentions `bmh`.
4. Items with data, for later. Test tool: **1.20.4**
   `/give @s minecraft:netherite_sword{Enchantments:[{id:"minecraft:sharpness",lvl:5},{id:"minecraft:mending",lvl:1}],Damage:100,display:{Name:'{"text":"Test blade"}'}}`,
   **1.21.1**
   `/give @s minecraft:netherite_sword[enchantments={levels:{"minecraft:sharpness":5,"minecraft:mending":1}},damage=100,custom_name='"Test blade"']`.
   Same for `netherite_axe` and `netherite_pickaxe`.

## 1. Names, tab, sprites (parts B, F and call 2e)

5. Creative inventory, the tab with the totem icon. Expected: it is called "BMH Tab" (before: a raw `itemGroup.BMH_tab`).
   It holds, in this order: The Eternal Axe, Eternal Bundle, The Eternal Pickaxe, The Sword of Eternity, Navigator's Amulet,
   The Totem of Eternal Life, Dormant Totem of Eternal Life, Echo Heart, and a vanilla bundle.
6. Look at every item at GUI scale 2, 3 and 4 (Options, Video Settings). Items that show a magenta and black checkerboard have
   a model pointing at a missing texture: tell me which one.
7. `/give @s bmh:dormant_totem` and `/give @s bmh:totem_of_eternity`, side by side. Expected: the dormant one never sparkles and
   never moves: eyes closed (two dark lines), forehead gem dull grey-violet, the rest identical to the awakened totem's first
   frame. The awakened one keeps its animation, its green and cyan eyes and its violet gem. Both names are aqua (rare) and the
   awakened one glows like an enchanted item.
8. `/give @s bmh:echo_heart`. Expected: a violet heart jewel, its left lobe lit with a small white glint, and a cyan
   four-point star glowing in a dark diamond-shaped window at its centre. Check it in the hotbar at night or underground (dark
   background): the whole heart, point included, stays readable. Put one in an item frame too.
9. Tooltips of the new items. Dormant Totem: one gray line, "Awaken it at an anvil: Dormant Totem left, Echo Heart right".
   Echo Heart: one gray line, "Awakens a Dormant Totem at an anvil: totem left, heart right". Awakened totem: `Eternity`
   (gray, like an enchantment) then `Heavy with eternity: you wake farther while you carry it` (gold).
10. Drop a Dormant Totem and an Echo Heart in lava: they do not burn (fire resistant).

## 2. The Eternal Bundle (parts A2, B3, F1 to F3)

11. `/give @s bmh:eternal_bundle`. Expected, empty: your dark bag and tan cord exactly as before, plus a small gold ring hanging
    under the knot of the cord, bottom centre (from GUI scale 2 up the ring shows a dark hole).
12. Put an item in it (pick the bundle up with the cursor and right-click a stack). Expected: the icon switches to the full,
    tied-neck bag with the same gold ring, now hanging under the end of the cord, as soon as one item is inside: in the
    inventory, the hotbar and in your hand. Take everything out: the empty bag again. Drop it full: the dropped item shows the
    filled texture too. If it keeps looking empty while it holds something, the `filled` property failed on this loader (search
    `filled` and `ClientItemProperties` in the log).
13. Hold it in first person, in third person (F5) and drop it (Q): the ring is a gold bump on the item, no stray or see-through
    pixels.
14. Tooltip: the contents preview right under the name (as in vanilla), then the fullness line (gray), then the gold
    `Heavy with eternity` line.

## 3. The Totem of Undying loses Fire Resistance (part A4)

15. `/give @s minecraft:totem_of_undying`, hold it in either hand, then `/effect give @s minecraft:fire_resistance 1000`.
16. `/damage @s 1000 minecraft:generic`. Expected: the usual pop (particles, sound), you stay alive on half a heart.
17. Look at the effect icons. Expected: Regeneration II (about 0:45) and Absorption II (about 0:05). No Fire Resistance, not
    even the one from step 15. Then `/setblock ~ ~ ~ minecraft:fire` and stand in it: you take fire damage.

## 4. Making the Totem of Eternal Life (part B, survival)

Switch to `/gamemode survival` and give yourself the ingredients with `/give` (netherite ingots, amethyst block, diamond
block, totem of undying, emerald block, crying obsidian, echo shards, nether star, XP via `/xp set @s 38 levels`).

18. Crafting table, Dormant Totem: netherite ingot, amethyst block, netherite ingot / diamond block, totem of undying, emerald
    block / (empty), netherite ingot, (empty). Expected: the Dormant Totem of Eternal Life (the old totem recipe).
19. Echo Heart: crying obsidian in the four corners, echo shards in the four edges, a nether star in the middle.
20. The old crafting recipes are gone: a netherite sword + netherite ingot + Totem of Eternal Life (old sword pattern) gives
    nothing, and the dragon egg pickaxe recipe gives nothing.
21. Anvil, `/xp set @s 38 levels`: Dormant Totem in the left slot, Echo Heart in the right slot. Expected: the result is the
    Totem of Eternal Life and the cost reads 39 and is red / not takeable at 38 levels.
22. `/xp set @s 39 levels`, take it. Expected: both inputs are gone, you are back to 0 levels. Type a name in the name field
    before taking it: the result is not renamed. Echo Heart on the left, Dormant Totem on the right: no result.
23. The awakened totem glows and its tooltip reads `Eternity` (gray) then the gold `Heavy with eternity` line. A grindstone
    refuses it in both slots (it only takes damageable or enchanted items), so the glint cannot be stripped.

## 5. Eternal tools (part B, calls 2c, A1)

24. Smithing table: The Totem of Eternal Life in the template slot, a test tool from step 4 in the base slot, an echo shard in
    the addition slot. Expected: the Eternal Sword / Axe / Pickaxe with Sharpness V, Mending, the name "Test blade" and the same
    wear (damage 100 out of 10000). Taking it consumes the totem, the tool and the shard. A diamond tool, or a diamond instead of
    the echo shard, gives nothing; the Dormant Totem is not accepted as template.
25. Tooltip of each tool. Expected, in this order: one dark aqua ability line (sword "Echo Strike: a full hit echoes onto a
    nearby hostile mob", axe "Smash: hits while falling land harder and soften the fall", pickaxe "Echo Sense: a mined ore makes
    the same ore nearby glow"), then `Heavy with eternity: ...` (gold), then `Each death: -7% durability` (gray), then the
    enchantments.
26. Repair: **1.20.4** `/give @s bmh:eternal_sword{Damage:5000}`, **1.21.1** `/give @s bmh:eternal_sword[damage=5000]`. Anvil
    with a gold block: the sword is repaired by 2500 (25%). With a Totem of Eternal Life as the material: no result.
27. Eternal Bundle recipe, crafting table: string, gold ingot, string / leather, Totem of Eternal Life, leather / leather,
    netherite scrap, leather. The vanilla bundle recipe (rabbit hide) still works.
28. **[1.21.1]** Enchanting: `/give @s bmh:eternal_sword` (and axe, pickaxe) and a Mending book,
    `/give @s minecraft:enchanted_book[minecraft:stored_enchantments={levels:{"minecraft:mending":1}}]`. Anvil, tool left, book
    right: a result with Mending (before the fix: no result), for all three tools. Enchanting table with the sword and lapis
    (`/xp add @s 30 levels`): offers appear. `/enchant @s minecraft:sharpness 5` with the sword in hand: success.
29. **[1.21.1]** Side effect of the tool tags: mine a placed `minecraft:decorated_pot` with each Eternal tool. Expected: it
    breaks into sherds like with a vanilla tool; with Silk Touch on the axe or the pickaxe it drops whole.

## 6. Echo abilities (part E)

`/gamemode creative` on the flat world, then give yourself the three tools. Echo victims are hostile mobs. Husks do not burn
in the sun, so they make good dummies.

30. Sword. `/summon minecraft:husk ~3 ~ ~ {NoAI:1b,Tags:["bmh_a"]}` and `/summon minecraft:husk ~3 ~ ~3 {NoAI:1b,Tags:["bmh_b"]}`.
    Hit A once with the Eternal Sword, indicator full. Expected: about a quarter second later a cyan dotted line from A to B,
    small sparks on B, a soft chime, B loses hearts: `/data get entity @e[tag=bmh_b,limit=1] Health` goes from 20 to about
    14.6 (the hit does 11, the echo is half of it, minus the husk's natural armor).
31. A rushed swing (indicator not full): no line, B untouched. Kill A with a full hit, then hit B alone: no line, no error in
    the log. The echo never goes to you, another player, a villager (`/summon minecraft:villager ~ ~ ~3 {NoAI:1b}`), an armor
    stand or your tamed wolf, and not to a cow, sheep, iron golem or bee next to A (the line still goes to B). Three husks in a
    row: only B is hurt, not C (the echo never echoes). If the echo kills B it drops its loot and XP as if you had.
32. Axe. `/summon minecraft:iron_golem ~ ~ ~ {NoAI:1b}` (100 hp). Teleport above it, fall onto it and hit it with the Eternal
    Axe just before the ground, indicator full. The fall counts up to the hit, about 3 blocks (your reach) before the ground:
    teleport `~ ~6 ~` for a 3 block fall, `~ ~13 ~` for 10, `~ ~33 ~` for 30. Read
    `/data get entity @e[type=iron_golem,limit=1] Health` (reset: `/data merge entity @e[type=iron_golem,limit=1] {Health:100f}`).

    | Fall before the hit | Multiplier | Health lost, no crit (sprinting) | Health lost, vanilla crit (not sprinting) |
    |---|---|---|---|
    | 3 blocks | x1.69 | 30.5 | 45.7 |
    | 10 blocks | x2.26 | 40.8 | 61.2 |
    | 30 blocks | x3 (cap, reached at 25 blocks) | 54 | 81 |

33. A smash sounds like an anvil clang and a thud, with an explosion puff, a ring of cloud and crit sparks at the target's
    feet (denser for bigger falls). It halves your fall distance: after a 30 block fall you take about half the fall damage.
    No smash, plain 18 damage (27 on a crit), when standing on the ground, in water, climbing, gliding with an elytra, or
    falling less than 1.5 blocks. No trees fall when you chop logs.
34. Pickaxe. Build the test wall: `/fill ~2 ~ ~2 ~9 ~4 ~9 minecraft:stone`, then `/setblock ~3 ~1 ~3 minecraft:iron_ore`,
    `/setblock ~5 ~1 ~3 minecraft:deepslate_iron_ore`, `/setblock ~3 ~3 ~5 minecraft:iron_ore`,
    `/setblock ~12 ~1 ~3 minecraft:iron_ore`, `/setblock ~4 ~2 ~4 minecraft:gold_ore`, `/setblock ~2 ~1 ~3 minecraft:iron_ore`.
    Mine the iron at `~2 ~1 ~3` with the Eternal Pickaxe. Expected: an amethyst chime and a cyan outline through the stone,
    for 5 seconds, around the stone iron at `~3 ~1 ~3`, the deepslate iron at `~5 ~1 ~3` and the iron at `~3 ~3 ~5`. The gold ore
    and the iron 10 blocks away at `~12 ~1 ~3` stay dark. After 5 seconds the outlines vanish by themselves.
35. Mine a stone block, or the gold ore with a normal iron pickaxe: nothing glows. Mine one of the glowing ores while it glows:
    its outline disappears at once (no ghost ore floating). Creative breaks count; spectator never breaks anything.
36. Quit to the title screen 2 seconds after an Echo Sense and come back: no ore keeps glowing (if one does,
    `/kill @e[type=block_display,tag=bmh_echo]` removes it, and tell me).
37. At most 64 glowing at once: `/fill ~2 ~ ~2 ~9 ~4 ~9 minecraft:coal_ore`, break one, then
    `/scoreboard objectives add bmh dummy`, `/execute store result score #n bmh if entity @e[type=block_display,tag=bmh_echo]`,
    `/scoreboard players get #n bmh`: 64 or less. Break a second ore right away: the count stays 64 or less, no doubled outlines.
38. Look at ores in the open, in a cave: the real ore must keep its normal look (not darker, no flicker) under its cyan outline
    (the glow cube is drawn a hair inside the block for this; nobody could look at it). In the Nether: quartz ore and ancient
    debris each light only their own kind; nether gold ore lights the gold ores.

## 7. The Navigator's Amulet (parts C and D, call 2b)

`/gamemode survival`, `/weather clear`, `/give @s bmh:navigator_amulet`. Start in the Overworld, on open ground, before you
set a Home or sleep in a bed.

39. Look. In the inventory: a violet bar, full. Tooltip: `Navigator's Amulet`, a violet `Starlight 3/3`, two gray lines (use at
    night under open sky; sneak and use on a bed or a lodestone). No durability line.
40. Recipe, crafting table: ` S ` / `CKC` / ` Y ` (string, 2 copper ingots, compass, spyglass) gives the amulet. The old recipe
    (string, ender eyes, compass) gives nothing. Check the recipe book too.
41. Needle, Overworld. Hold it and turn on the spot (F3 shows "Towards negative Z" for north). Facing north: needle straight up,
    red tip up. East: tip to the left. South: down. West: to the right. It wobbles a little like a compass and must not point at
    spawn (walk away from it and check). Drop it: the item on the ground and in an item frame shows a needle too. This is the
    registration that call 2a moved, so check it on every loader. In the Nether and the End the needle spins freely.
42. No Home yet. `/time set midnight`, clear sky, right click in the air. Expected: an amethyst chime, your arm swings, the action
    bar says `X <x> · Y <y> · Z <z>` with your block coordinates (compare with `/data get entity @s Pos`), the item bar drops to
    two thirds, the tooltip says `Starlight 2/3`. A second right click within 2 seconds does nothing (cooldown sweep). Nothing
    shows Home's coordinates or distance, anywhere.
43. Stars hidden, nothing spent (the bar stays at 2/3 each time): `/time set day`: "The stars are hidden: wait for night."
    Midnight plus `/weather rain`: "The stars are hidden behind the clouds." Midnight, clear, under a roof: "The stars are
    hidden: you need open sky above you." In the Nether: "The stars are hidden: they only answer in the Overworld."
44. Cannot see the sky (call 2b), midnight, clear, open sky, the bar must not drop in any of these. Each time the action bar says
    "The stars are hidden: you need open sky above you.": `/effect give @s minecraft:blindness 30`, then `darkness`; dig
    down and stand with your eyes in water, in lava (creative, then back to survival), and with your eyes inside powder snow
    (`/setblock ~ ~1 ~ minecraft:powder_snow` at head height). With only your feet in powder snow the reading works.
45. Charges. Read two more times (2 seconds apart): `Starlight 0/3`, empty bar. The next right click says "No starlight left
    tonight." and plays nothing. `/time add 24000` (a new nightfall went by): within a second the bar is full and the tooltip
    says `Starlight 3/3` again. Sleeping through a night does not refill: only the next dusk does. In creative a reading spends
    no charge.
46. The sky drawing (part D). `/gamemode creative` (free readings), `/time set 18000`, open sky, right click. Expected: within a
    moment a white line with a violet edge starts high in the sky and traces down toward the horizon, one segment about every
    0.3 s, through 5 to 7 real stars, each lighting up (white, soft violet glow) as the line reaches it; the lowest star has a
    slightly bigger glow. Turn toward it: the needle points about the same way. The log has
    `Star reading toward <azimuth>: <n> stars, the last at azimuth <a>, ...` with `<a>` within a few degrees of `<azimuth>` (up
    to 15 when the moon is low there).
47. It holds about 8 s after the trace ends, then fades over 2 s: no text, no number, nothing on the HUD. It sits on stars that
    were already there. Amulet in the offhand: same drawing. Switch to another item right after using it: the drawing still
    plays to the end. A second reading (after 2 s) replaces the old chain; with an amulet in each hand the newer reading shows,
    no flicker. `/weather rain` while it holds: it dims with the stars. `/time set 22000` facing north, east, south, west in
    turn: a chain every time (toward the west it may run across the low moon). `/time set 13000` and `23000`: fainter, like the
    stars.
48. Home. Back in survival. Place a bed, sneak and right click it with the amulet: action bar "Home set to this bed.", a lower
    chime, and you do not go to sleep. Same on a lodestone (`/give @s lodestone`): "Home set to this lodestone." Holding the
    click does not machine-gun the chime. In the Nether, sneak and right click a bed: "Home can only be set in the Overworld."
    and the bed does not explode.
49. The reading contract. After a reading, `/data get entity @s SelectedItem` shows `bmh_reading:{azimuth:...f,time:...L}`,
    `bmh_starlight` and `bmh_night` (**1.21.1**: inside `components."minecraft:custom_data"`). Stand due south of Home: azimuth
    about 0; due north: about 180; Home to your east: about 90; Home to your west: about 270.
50. Off hand. With a shield, a torch or a bucket in the off hand and the amulet in the main hand, the click reads the stars and
    does not raise the shield, place the torch or empty the bucket. The amulet in the off hand with a sword in the main hand:
    the click still reads the stars.

## 8. Death and exile (parts A3, B, C; do these last, each death sends you thousands of blocks away)

Empty your inventory first (`/clear @s`), difficulty Normal, the default config (`minDistanceFromBed` 5000, `maxDistanceFromBed`
60000). Both lines are in the log, search `Exile distance`.

51. In the Overworld: `/spawnpoint @s 0 100 0`, `/kill @s`, Respawn. Expected log line
    `Exile distance: 23333 world blocks, 23333 blocks in minecraft:overworld (coordinate scale 1.0)`, and you wake about 23333
    blocks from the origin (`/data get entity @s Pos`).
52. Nether anchor (creative, so the terrain cannot hurt you): `/execute in minecraft:the_nether run tp @s 0 100 0`,
    `/give @s minecraft:respawn_anchor`, `/give @s minecraft:glowstone`; place the anchor on a block you placed, charge it with
    the glowstone, right click it again with an empty hand ("Respawn point set"), `/gamemode survival`, `/kill @s`, Respawn.
    Expected log line `Exile distance: 23333 world blocks, 2917 blocks in minecraft:the_nether (coordinate scale 8.0)`, and
    you wake in the Nether about 2917 blocks from the anchor (before: 23333 Nether blocks, 186664 in the Overworld).
53. Die loaded: give yourself every item, set Home on a bed with the amulet (step 48), `/kill @s`, Respawn. Expected: the Totem
    of Eternal Life, the three tools and the bundle come back with you, the tools lose 7% durability; the Dormant Totem, the
    Echo Heart and the amulet drop where you died like any item. Your Home survives: `/data get entity @s bmh_home` shows
    the same `x`, `y`, `z`. Pick the amulet up where you died and read the stars: the azimuth still points at that Home. Leave
    and rejoin the world: `bmh_home` is still there.

## 9. Dedicated server and a second player (once per version, any loader)

54. Start `:<loader>:runServer`, join it with a client. The amulet's action bar texts (steps 42 to 48) show for you (before, the
    coordinates never showed on a dedicated server), and the sky drawing (step 46) still plays on the client of the player who
    used the amulet.
55. With a second client (LAN or the same server): only the player who used the amulet sees the chain.
56. Shader packs (Iris or Oculus), only if you use them: best effort. A pack that draws its own sky may hide or tint the chain;
    not a blocker.

## Coordinator's calls (made during the merge, each with its reason)

- **2b, a star reading is refused without spending a charge when the sky is out of sight** (Blindness, Darkness, eyes in water
  or lava, eyes inside powder snow). Reason: the sky drawing shows nothing then, so the charge was wasted (the gap in part D's
  checklist). It uses the no-open-sky message. Implementation note: "inside powder snow" means the eyes, like the sky drawing's
  own check; with only the feet in snow the sky is drawn and the reading works (step 44).
- **2c, each Eternal tool names its ability** in a dark aqua tooltip line above the gold one. Reason: without those lines the
  abilities are invisible to players. The changelog says so too.
- **2d, the awakened totem keeps "The Totem of Eternal Life"** (your name) and the new item is the "Dormant Totem of Eternal
  Life". Reason: you never asked for a rename, and the two names should match. The changelog and these checklists use the same
  names. The registry ids (`bmh:totem_of_eternity`, `bmh:dormant_totem`) and the `Eternity` tooltip line did not change.
- **2e, the creative tab's title** showed a raw key (an old bug found by part B: the code asks for `itemGroup.BMH_tab`, the lang
  file only had `itemGroup.bmh_creative_tab`). Reason: it is a bug anyone would see. The key the tab asks for now exists;
  `### Fixed` line in the changelog.

## Open before release

- The README crafting-grid pictures still show the old recipes (the amulet, the bundle, the three tools and the totem), and its
  Totem section still says "Totem of Eternity" and "only used to craft other eternal items". They need new screenshots and a
  rewrite.
- NeoForge 1.20.1 runs the Forge jar and has no boot of its own; the other 1.20.x jars (1.20, 1.20.1, 1.20.2, 1.20.3) are built
  from the same sources as 1.20.4 and were only built here, not run in a client.
- Nothing in this run touched `main`, a `dev-*` branch, a tag or a release.
