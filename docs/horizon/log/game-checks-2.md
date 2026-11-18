# Game checks 2: eternal and amulet in the real game

Branch `horizon-26.3`, tip `c260f94961e44348977f8f746ea6edfc8430bcb9`, tree
`/Users/louisobenicherousse/Projects/Balanced-Minecraft-Hardcore/.claude/worktrees/busy-visvesvaraya-4dfefc`.
Game: offscreen, no audio, scratch folders of `test-slice.sh`. Date of the start: 2026-11-18 (06:58 by `date`).

## Steps

1. The guard command printed the tree, `horizon-26.3`, the tip above and no changed file. No `horizon-client` was running.
2. Read the brief, the build skill, the two area logs and `test-slice.sh`.
3. Started `sh horizon/tools/check.sh build` in the background (the binary on disk dated from 03:02, before the merges).

4. Build done (`check.sh build`, exit 0, 1 min 37 s). Explored by hand in a scratch game (offscreen, own socket, own home under
   `/private/tmp/claude-501/gc2dev`), before writing the scenarios. Results of the hand run (the scenarios `p` and `q` repeat them):
   - C14 pass. Survival, rule on: `gamerule reduced_debug_info` says true; F3 shows no XYZ, no Block, no Facing, no Biome
     (`eternal-shots/c14-f3-survival.png`). `/gamemode creative`: the rule is false and F3 shows XYZ, Block, Chunk, Facing
     (`c14-f3-creative.png`). Survival again: true. `/bmh rule remove_coordinates_in_survival false`: the rule is false and F3
     shows the coordinates (`c14-f3-ruleoff.png`). Set back to true: true. No WARN line "did not run".
   - C11 pass. Six Eternal Items given (bundle with a stone, sword, axe, pickaxe, Totem of Eternity, Sleeping Totem) plus a Dormant
     Totem and a Navigator's Amulet. After `/kill` only the amulet and the Dormant Totem lay on the ground. After the respawn the six
     were in the inventory, the bundle kept its stone. `players/data/<uuid>.dat` held `bmh_eternal_kept` while dead and not after
     the respawn. A second death gave no second copy (counts stay 1).
   - C11 restart while dead pass. Died, `quit`, launched again: the death screen, then the respawn gave the six once; the key was
     gone from the file after.
   - C11 change of world while dead (check 15) pass. In world A died with the six, left through the Title Screen button, opened
     world B (new), died and respawned there: empty inventory. Opened A: dead, respawned: the six, once, key gone.
   - E11 pass. A second `bmh:eternal_sword` lay on the ground and was never picked up over 160 ticks.
   - E10 pass. `data get entity` of the dropped sword: `Health: 32767s`. After `tick sprint 6000` the item was still there and
     `Age` was -26095 (reset at age 6000).
   - E12 partial. An Ender Chest put in the bundle while the bundle sat in a slot: out within a tick, back in the inventory. Dirt
     stayed. A bundle with dirt, right click: the dirt dropped on the ground; an empty bundle: nothing. Not testable by clicks: a sword
     in a bundle (the game's bundle never took the sword). The Ender Chest does sit in the bundle while the bundle is on the cursor
     (listed difference 10 of the area, not a new defect).
   - C13 pass. Bed placed, `/time set 13000`, health 17: after the sleep health 20 and `effects` lists haste and resistance, both
     amplifier 1, duration 3593 ticks.
   - A2 pass (star reading). Night 18000, clear, open sky: action bar `X 46122 - Y 64 - Z 27874`, log line `Star reading for ...
     azimuths [301.14682], 2 charges left`, stack custom data `bmh_starlight:2`, `bmh_reading`, and the advancement `Reading the
     sky` (proved by `advancement revoke`, which said "Revoked").
   - A1 pass (needle). Four crops of the hotbar slot at yaw 180, -90, 0, 90 differ and the red tip turns as north moves
     (`amulet-shots/needle-icon-yaw-180-m90-0-90.png`).
   - Day refusal pass: `time set 6000`, use: action bar "The stars are hidden: wait for night." (`amulet-shots/refusal-day.png`).
   - Naming screen pass. Sneak and use on a bed opens "Name this point / On this bed", Save grey, "Point 1 of 5"
     (`naming-screen-bed.png`). On a lodestone: "On this lodestone", "Point 2 of 5" (`naming-screen-lodestone.png`). Typed `Camp`,
     Enter: action bar "Point saved: Camp".
   - List screen pass (`list-after-death.png`, `list-after-restart.png`). The point Camp is in the list after a death (the amulet
     itself dropped) and after a restart.

5. Wrote scenarios `p` (eternal, 20 checks) and `q` (amulet, 16 checks) in `horizon/tools/test-slice.sh`, before the scenario `m`;
   added them to the header of the script and to the table of `docs/horizon/TESTING.md`. Test bugs of my own fixed on the way
   (none was a defect of the mod): the world change waited for `loaded` too early; the first click on the Title Screen button of the
   death screen does nothing, so the script clicks until the title shows; `time query daytime` does not exist on 26.3 (it wants a
   timeline), the time of day is `state.time.time_of_day`; the wake-up check read the effects before the night was skipped;
   `time set` day and back to night refills the amulet (the Java rule: the night index changed), so the charge check reads
   "three readings, then the message" after the time changes; a point at the same bed is renamed, not added (q12 renamed Camp to One).
6. Runs: `--only p,q` gave 35 passed, 1 failed (p20, the time helper above). `--only p` after the fix gave 20 passed, 0 failed.
   The q checks passed 16 of 16 in the second run of `--only p,q`. No change of the mod was needed. I changed no file of the mod.
7. One more hand run (not a scenario): an Eternal Bundle on the cursor at death. It fell on the ground at the death spot with
   the amulet and the dirt, and it was not given back at the respawn (inventory empty). See "Observations".
8. The whole script, `sh horizon/tools/test-slice.sh --no-build`: see "Last full run".

## Defects that are left

No new defect of the mod was found in the eternal and amulet areas. Every check of `p` and `q` passes in the full run. I changed
no file of the mod. The one red check of the whole script is the old one:

| Area | Severity | What happens | Expected | Evidence | Command |
|---|---|---|---|---|---|
| respawn (not this wave) | major | `immediate_respawn` true: the player respawns at the point with a WARN "landing search ... did not end" | the exile still happens | `n5` of `test-slice.sh`, `game-checks-1.md` | `sh horizon/tools/test-slice.sh --no-build --only n` |

## Observations (not defects, or not decided)

1. The kept `reduced_debug_info` value. The world saves the rule when the mod forces it on, and the mod forgets its value before when
   a world opens again. After that, creative keeps the rule true until the player sets it false by hand. Java says the same in
   `DisableDebugInfo.java:21-24`. The first full run showed it (p2 and p3 red in a world that earlier scenarios had saved). The
   scenario now sets the rule by hand first. A player who plays a world twice with the mod on will see no coordinates in creative
   until he runs `/gamerule reduced_debug_info false`. Worth a line in the README or a fix in a later round (a design choice).
2. An Eternal Item on the cursor at death falls on the ground at the death spot and is not given back (hand run, step 7). Java drops
   the cursor stack the same way when the menu closes. The item lies there with Health 32767 (not checked for this item). Parity
   probable, not proven.
3. A second click is needed on the "Title Screen" button of the death screen in the control tool (the first click returned
   "clicked" and the screen stayed). Not a BMH hook. Not investigated.
4. Light lag: a single block roof read as open sky when the amulet was used 50 ticks after the block was set. A later try with
   the light read (`sky` 14 at the eye) refused. The check `q6` uses a 7 by 7 roof and waits 60 ticks.
5. `time set` day and then night refills the amulet: the night index changes, as in Java. Not a defect.
6. A point saved on the same bed again is a rename (q12 made Camp into One). Java does the same (a point at one block).
7. The amulet is not Eternal: it falls at death and the point stays on the player (checks q15 and q16).
8. A sword cannot be put into a bundle by clicks in this game (the bundle never took it), so the refusal of an Eternal Sword was
   not exercised, only the Ender Chest. E12 for a bundle on the cursor stays a listed difference (difference 10 of the area log).

## Last full run

First full run of the final script: 73 passed, 14 failed (a world made by `open` follows the Create New World choice file that
scenario k leaves OFF, so no landing line there; and the saved rule above). Second: 86 passed, 2 failed (`n5` and `m2`: the
sleep in p19 moved the respawn point, so m2 measured from the old point). Third (after `p` put a respawn point at the player again): `test-slice.sh: 87 passed, 1 failed, 0 skipped`. The one red check
is `n5`. All 20 checks of `p` and all 16 of `q` passed in that run.

## What a person must still look at by hand

- The look of the F3 screen (`eternal-shots/*`), the needle in the hand and the hotbar at several angles, the star reading drawing in
  the sky (the sky area is not ported: the reading is only written on the stack), the naming and list screens at other GUI scales,
  the particles and the sounds (the chime at pitch 1.0 and 0.7, the bundle drop sound).
- The Nether refusal of the amulet, eyes in water, a Vanishing curse Eternal Item, `keep_inventory`, a full inventory at the respawn,
  two players (LAN), a Java world with BMH points, fire, lava, cactus, TNT and lightning on a dropped Eternal Item,
  a bundle in a chest or on the cursor.

## Never tested

- The sound (every run is muted).
- Anything with two players, a dedicated server or a LAN guest.
- The Nether and the End.
