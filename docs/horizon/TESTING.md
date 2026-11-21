# Testing the Horizon mod

See the latest integration results in [the port log](LOG.md). The dated runs below record earlier builds.

## What the owner does once

1. Install the official Minecraft launcher. Start Minecraft Java Edition 26.3 once and quit. This puts the version
   file, the client jar, the libraries and the assets in `~/Library/Application Support/minecraft`.
2. Get the 26.3 server jar. The URL is `downloads.server.url` in
   `~/Library/Application Support/minecraft/versions/26.3/26.3.json`. The page minecraft.net/download/server gives
   the same file while 26.3 is the current release. Save the jar anywhere. Its sha1 must be the one that
   `research-lock.json` of Horizon pins (the script checks it).
3. Install the JDK 25: `brew install openjdk@25`.
4. Run `sh horizon/tools/check.sh` once. It clones Horizon into the cache and compiles the mod. `prepare-game.sh`
   needs the source tree this makes.
5. Run `sh horizon/tools/prepare-game.sh --server-jar /path/to/server.jar`. Add `--minecraft-dir DIR` when the
   launcher keeps its files in another folder. The script never downloads Minecraft. When something is missing, it
   prints one `MISSING:` line for each thing, says what to do, and exits 78 without changing anything. When all is
   there, it builds and runs `horizon-prepare`, runs its `--check`, and prints the folder
   `~/Library/Caches/bmh-horizon/resources/26.3`. `--dry-run` prints the commands and runs none.

The prepared files are made from the owner's copy of the game. Never commit them and never share them.

## How to run the tests

| Command | What it does | Needs |
|---|---|---|
| `bash horizon/tools/test-logic.sh` | Unit tests of the logic crate (`cargo test`). | Rust |
| `sh horizon/tools/check.sh` | Compiles the native bundle into Horizon (about 30 s after the first run). | Rust, network once |
| `sh horizon/tools/content-check.sh` | Builds and checks the content mod (`.hmod`). | `cargo-horizon-mod` |
| `sh horizon/tools/test-slice.sh` | The in-game tests for the host and a LAN guest. | Prepared game files |

`test-slice.sh` options: `--only a,b` runs some scenarios, `--keep` keeps the scratch folder, `--window` shows a
window instead of rendering offscreen, `--no-build` skips `check.sh build`, `--resources DIR`, `--bin-dir DIR`,
`--out DIR` (where the screenshots go). Exit codes: 0 all checks passed, 1 a check failed, 77 no prepared files,
3 the game did not start, 64 a bad option.

How it works: it builds the game with `check.sh build` (the last line of that output is the folder of the binaries),
makes a scratch folder in the cache with its own `HORIZON_HOME` and game folder, launches the game with
`horizon-ctl launch --offscreen`, a new world with seed 20261111, then drives it. With `--package <zip>` both archives come from the distribution.
Without it, the runner loads the content folder with `--dev-mod`.
Sound, Discord and telemetry are off. A trap quits the game, stops it by force if it does not quit, and removes the
scratch world on every exit, also on a failure and on Ctrl+C.

Facts the test uses, and where they come from:

- The distance is `min + (max - min) * easy / 9` in integer arithmetic, with
  `easy = min(1 + eternal_slots + difficulty, 9)` (`bmh_logic::respawn::distance`). The test sets the difficulty to
  Normal (id 2) and carries no Eternal Item, so `easy = 3`. With the default config (5000, 60000) the distance is
  23333 blocks. The script reads `config/bmh-common.toml` in the scratch game folder when it exists.
- The landing is on the circle of that radius round the world spawn. The test takes the player's start position as
  the world spawn. It accepts a difference of 64 blocks (the spawn radius, the integer truncation of the point and
  the safe-ground search).
- `horizon-ctl respawn` does not go through the death screen, so the mod's hold of the Respawn click does not run
  (`docs/horizon/log/journey.md`, differences 2, and GAPS C4). If the command ran at once, the landing search might
  not be finished, and the respawn would be vanilla. So scenarios b, c, e and f wait for the mod's log line
  `Landing spot ready` (rule on) before they call `horizon-ctl respawn`. That is the path that works. Scenario d
  uses `horizon-ctl click-button respawn`, which clicks the death screen button. Whether that click reaches the
  mod's hold is one of the things the first run shows.

## Run the distribution tests

Build the native game from this checkout. Package the same sources and pass the ZIP to the runner.

```sh
sh horizon/tools/check.sh build
python3 horizon/tools/package.py
sh horizon/tools/test-slice.sh --no-build --package horizon/build/bmh-1.9.0-horizon-26.3.zip
```

To test a dedicated server, use the same package, the folder containing the built `horizon-client`, and the prepared game resources.

```sh
python3 horizon/tools/test-dedicated.py --package horizon/build/bmh-1.9.0-horizon-26.3.zip \
  --bin-dir "$HOME/Library/Caches/bmh-horizon/horizon-mods/native/target/cargo/fast" \
  --resources "$HOME/Library/Caches/bmh-horizon/resources/26.3"
```

The test boots a temporary world three times. It checks the installed item, config correction, console commands, rule persistence and import from a Java world. Use `--out <folder>` to keep the logs.

Use `--only abilities` to check all three Eternal tools. The tests measure enemy health, read earned advancements and compare the Echo Sense outline before and after expiry.
Use `--only z` for containers and transfers, `--only t` for death protection, and `--only rules,o,cooldown` for the world rules, world creation button and amulet HUD.

## Current gameplay checks

| Scenario | Coverage |
|---|---|
| `t` and `z` | Inventory death protection with both hands occupied, protector priority, death bypasses, both anvil result takes, and immediate storage refusal. |
| `respawnpoints,anchor,respawnrace` | A valid or blocked bed, an active Nether anchor, and a point that changes between death and respawn. The immediate path checks that the charged anchor loses one charge. |
| `r,abilities,x,ground` | Death wear with Keep Inventory or Vanishing, damage and dug blocks, both PvP abilities on a LAN guest, and damage immunity for drops with an ordinary item as a control. |
| `s,p,bundlebar` | Ritual, item-use counters, retained items and bundle contents, click refusal, and the empty, partial and full bar. |
| `q,l,needle` | Point editing, exact settings and the six original buttons, and the needle at the world center and 20 million blocks east or west. |
| `hardcoredeath,homeawards` | A real Hardcore death, the tenth-death and 30,000-block thresholds, and return to a named home. |
| `v,w,n,cooldown` | Spyglass labels, black frames, blocked input, immediate respawn, and the cooldown sweep. Some checks compare screenshots. |

Run `homeawards` and `respawnrace` explicitly. The ordinary point editing test uses keyboard selection. The optional `BMH_TEST_CLIPBOARD=1` check also uses the system clipboard.

## Baseline checks

| Check | What it does | A PASS proves | It does not prove |
|---|---|---|---|
| a1 | Opens Options > Mods (`options mods`), looks for the mod name in `ui`. SKIP if the page does not open. | The mod is loaded and listed. | That the page with the six buttons works or fits. |
| a2 | `/bmh rule` and `messages`; looks for the rule names. | The command is registered and answers. | The text of the answer, the toast, the operator level. |
| a3 | `<world>/bmh/rules.txt` has six `name = true/false` lines. | `world_lifecycle` ran at the first open and wrote the rules. | A Java world import. |
| b1 | Survival, kill, wait for the death event, wait for `Landing spot ready`, `horizon-ctl respawn`, wait for the respawn event; the horizontal distance from the spawn is within 64 blocks of the Java formula. | The respawn mixin chain gives the position to the game, at the right distance. | The distance with an Eternal Item, other difficulties, a bed, the Nether. |
| b2 | The log has a new `new respawn pos and bed` line and no new `No distant respawn`. | The landing came from the search, not the vanilla fallback. | Why the search chose that spot. |
| b3 | After the respawn: health above 0, on the ground, feet not in water or lava, the block below is not air, water, lava or leaves, the dimension. | A usable landing in this one case. | Many landings near oceans (hand check 3 and 4). |
| c1 | Right after the arrival, `damage @s 6`; ten ticks later the health has not dropped. SKIP if the command does not exist. | `living_incoming_damage` cancels damage after a distant respawn. | That `/kill`, the void and other bypass damage still work. |
| c2 | After 220 more ticks, `damage @s 6` lowers the health. | The immunity ends. The count is about 200 ticks. | The exact tick (the test is accurate to a few ticks). |
| d1, d2 | Takes the screenshots of the cinematic: death screen, arrival, and ten frames one second apart after a click on Respawn. | Nothing but that the steps ran. | Everything about the look. |
| d3 | Each screenshot file exists and is not empty. | The game can screenshot at these moments. | That a frame shows the black, the sky or the dissolve. A person or a reviewer agent must look at the files. |
| e1 | `/bmh rule random_death_spawn_point false` and `rules.txt` shows it. | The command writes the rule. | The restart. |
| e2 | Kill and respawn with the rule off: within 200 blocks of the spawn and no new landing line. | The rule is read at each death, with no restart. | That the position is exactly vanilla. |
| e3 | The rule is set back to true. | Cleanup. | Nothing. |
| f1 | Creative mode, kill, respawn: within 200 blocks of the spawn and no new landing line. | Creative players get the vanilla respawn. | Adventure and spectator. |
| g1 | After a distant respawn: `save`, then the advancement file of the world has `bmh:journey/exile` with `done` true. If there is no file, `advancement revoke @s only bmh:journey/exile` says "Revoked". | The award command from `server_tick` works. | The toast, the other 19 advancements. |
| b4 | After b0, b0s and b1 (alone in the world): the log has no `WARN` line of the mod at all. | Sounds with nobody near and an award already held are not logged as failures (review semantics-5, first-run defect 3). | That a sound plays for a second player (hand check respawn 8). |
| c3 | Distant respawn, `/kill` at about tick 40, rule off, vanilla respawn, `damage @s 6` at tick 80: the health drops. | A death inside the 200 ticks ends the immunity (review parity-3). With the old shield, 80 ticks would be left. | The same after a distant respawn (the shield starts again then). |
| h1 | Gives the nine items of `ids.rs`, opens the inventory, keeps `h-items-inventory.png` under `--out`, then `clear @s`. The file exists and is not empty. | The screenshot is on disk. | The look: a person or a reviewer opens the file. |
| i1 | `/spawnpoint` 3 blocks from the player, stone in it and above it, death: the landing is about 23333 blocks from the world spawn and the log has `reads minecraft:stone: Missing`. | A forced point with no room counts as a missing respawn block, as vanilla's resolution says, so the centre is the world spawn (review parity-1). | A bed slept in, a Nether anchor (hand checks below). |
| i2 | After i1: the chat has "You have no home bed or charged respawn anchor, or it was obstructed". | The player is told, as vanilla's `NO_RESPAWN_BLOCK_AVAILABLE` does. | That the point is forgotten (it is not: GAPS.md 3.3 C2). |
| j1 | `/spawnpoint`, `/kill`, `horizon-ctl quit` while dead, a second launch on the same world: the death screen, then a new `Landing spot ready` line. | A player dead when the game closes gets a search when the world opens (review semantics-1). Horizon kills such a player once more at login. | A relog on a dedicated server (no server in these tests). |
| j2 | After j1, `horizon-ctl respawn`: about 23333 blocks from the point. | The exile happens after the restart. | The distance of a death with Eternal Items across a restart. |
| k0, k1 | Pause, Save and Quit to Title, Singleplayer, Create New World. Clicks the mod's button twice and reads the mod setting file `horizon-mods/.horizon-mod-data/bmh/balanced_hardcore`. The button is drawn by the mod and `ui` does not list it, so the script finds its place from the Allow Commands widget: the middle of that widget, plus 28 pixels down (the 20 high button and the 8 pixel row gap). | The button flips ON and OFF in the Game tab, in the row under Allow Commands, and stores the choice. | The look (see `settings-fix-1-shots/`). |
| k2 | A click on the middle of the More tab must open the tab and leave the choice alone. | The button does not lie over a tab at the default GUI scale of a 2560 x 1440 window (GUI width 427). | Other GUI scales (width 854 was seen by hand, `settings-fix-1-shots/`). |
| k2b | On the More tab and on the World tab, a click at the place the button has in the Game tab leaves the choice alone. | The button is drawn and clickable in the Game tab only. | The look of the other tabs (see `settings-fix-1-shots/`). |
| k3, k4 | Makes a world with the button ON, then one with it OFF: `rules.txt` holds six `true` and six `false` lines. | The choice reaches the new world. | A Java world import. |
| k5 | After a world made with OFF, a new Create New World screen starts ON, as the Java button does: the choice file reads `true`, the first click turns it OFF and the second ON (so the button's own state started ON, not only the file). | Each new screen starts ON. | The picture of the button (see `settings-fix-1-shots/game-tab-gui854-new-screen-starts-on.png`). |
| k5c | Button OFF, Cancel, Create New World again: ON. | A screen that was left starts ON again. | |
| k5d | Button OFF, World tab, World Type to Superflat, Customize, Done, Game tab: the choice is still OFF. | The Superflat editor is part of the same screen and keeps the choice. | The Buffet editor (same code path, ids `world_create_flat` and `world_create_buffet`). |
| k5e | Button OFF, More tab, Data Packs, Done (clicked by place: `ui` lists no button of that screen), Game tab: the choice is still OFF. | A screen of the game's own over Create New World does not reset the choice. It failed before the change of 2026-11-18 (`settings.md`, "The coordinator's check"). | The Experiments list (not driven). |
| k5f | Button OFF, More tab, Game Rules, Done, Game tab: the choice is still OFF. | The Game Rules list keeps the choice. | That the list shows no BMH rule (a listed difference). |
| k6 | `horizon-ctl open` opens the test world again. | Cleanup. | |
| l0, l1 | Options > Mods has the BMH row. Its settings page has six buttons with the ids `minimum-respawn-distance`, `maximum-respawn-distance` and `amulet-points`, each with a `-2` twin that lowers. | The page and the labels. | That the labels fit at every GUI scale. |
| l2 to l5 | Clicks on every button: 5000 steps, one step for the points, min not above max, ranges 5000 to 200000 and 1 to 16, read from `config/bmh-common.toml`. | The stepper and its limits. | The error lines of the Java screen (there are none here). |
| l6 | Min = max = 15000 through the page, then a death: 15000 blocks from the point, with no restart. | A changed distance applies to the next death. | |
| l7 | Back to 5000, 60000 and 5 through the page. | Cleanup. | |
| n1, n2 | Click-button Respawn, then seven seconds of screenshots with the screen name. Counts cream pixels (the message colour) and black frames with PIL. n1 rule `display_text_on_respawn` on: message. n2 off: the journey runs, no message. SKIP when PIL is missing. | The message rule. | The look and the text position. |
| n3 | The same with W held and Escape, E, Space tapped when the screen is `mod`: one position only, the screen stays `mod`. | The hold screen takes the input. | A real keyboard (the control tool sends the keys). |
| n4 | `move forward 1` after the journey: the position changes. | Input comes back. | |
| n5 | `gamerule immediate_respawn true`, `/kill`: the exile must still happen (23333 blocks) and the log has no new `WARN` line of the mod. | Passes since the journey fix round 1 (2026-11-18): the exile happens after the early respawn, with no `WARN`. | The cinematic after an immediate respawn (see w1c). |
| w1 | `immediate_respawn` true after a first plain exile: `kill @s`, 12 s of frames. The player ends 23333 blocks (within 64) from the point; no new `WARN`; the log has the line "did not end before the respawn" once, then "moved to the landing after an early respawn" once; exactly one new "Homecoming record:" line. | The early respawn moves the player to the landing, once, with no warning. | That the player never stands at the point on screen (w1c). |
| w1b | After w1 and `save`, the advancement file has `bmh:journey/exile` done. | The award reached the player on the early path. | That it was awarded exactly once (the file holds a flag, not a count). |
| w1c | The frames of w1: at least 200 ms of continuous black after the kill, at least 5 frames of the `mod` screen, no frame of the landing world before the first black frame. Contact sheet `w1-sheet.png`. | The journey starts in black after the early respawn, then the sky, the message and the dissolve at the landing. | One frame of the world at the point shows between the kill and the black (about 0.3 s, `Departure` comes after the death packets). The look. |
| w1d | The log has no new "is home after" line during w1. | The early respawn at the old exile's home is no homecoming. | Fails now: the homecoming fires at the vanilla point, then the "Home" title shows at the landing (see `game-checks-5.md`, defect 1). |
| w2 | `kill @s`, wait for the death screen and "Landing spot ready", then `respawn` and a frame every 0.1 s for 7 s. At least 3 black frames and no frame of the arrival world (playing, near the landing) before the first black. Contact sheet `w2-sheet.png`. | The arrival world is never drawn before the black. | The loading screen frames (allowed) and the look. |
| w3 | `immediate_respawn` true: die once. Then `immediate_respawn` false and `random_death_spawn_point` false, die, click Respawn: no black frame, no `mod` screen, the player is within 64 blocks of the point. | A hold armed by an old death does not hold a later one. | |
| w4-escape, w4-button | Die, click "Title Screen" (a confirmation opens over the death screen), then close it with Escape (w4-escape) or its Respawn button (w4-button). At least 1 frame of the `mod` screen, at least 1 black frame, no plain respawn at the point. | The request that closes a screen over the death screen is held, as the click on the death screen is. | A pack prompt, a book or a sign over the death screen (no tool opens one). Closing the confirmation IS a respawn request, so "close, then click" cannot be split. |
| w5 | Die, wait for the death screen, click Respawn at once, 3 s of frames: no frame of a living player at the point. | A click before the `Departure` is held or ignored, never a plain respawn. | The exact first second: the click was ignored here (the buttons are inactive for 20 ticks). |
| x1 | A second game joins over the loopback address (`lan open`, `connect`): a LAN guest (level 0) runs `/bmh rule` and `/bmh rule random_death_spawn_point false`. Two red unknown-command lines, `bmh/rules.txt` unchanged. | A player below level 2 cannot list or set the rules. | A level 2 or more other than the owner: a LAN guest has level 0 even with "commands" on. The owner (level 4) is tested by a2, e1, l6 and m1. |
| x2 | The owner gives the guest an amulet and puts a bed; the guest runs `/bmh point save x y z GuestCamp`: the action bar says "Point saved: GuestCamp". | `/bmh point save` works at level 0. | The naming screen with a real mouse. |
| x3 | The guest at level 0 types `/gamerule bmh:random_death_spawn_point false`: the log says "taken from a player" and the chat shows the unknown-command text. | `player_command` hears a guest, and `/bmh rule` refuses a level below 2 when it runs the line as the guest. | A guest at level 2 or higher. |
| rules0 | The owner of a world with Allow Commands off types `/gamerule bmh:...`: taken, refused, `rules.txt` unchanged. | The owner reaches `player_command`, and the level check holds for the owner. | |
| rules1 to rules3 | A world made with Allow Commands on: `/gamerule bmh:display_text_on_respawn false`, the query, then `true`. | The Java command sets, reads and resets a BMH rule, with vanilla's answer texts. | The look of the chat line. |
| rules4, rules5 | `/gamerule keep_inventory true` runs in vanilla. `/gamerule bmh:nonsense true` says "Unknown BMH rule: nonsense". | A vanilla rule is not taken, and a wrong BMH name is answered. | |
| rules6 | No "ran nothing for the line" warning in the log. | Every queued line ran. | |
| rules8 | A rule set to false, Save and Quit to Title, open the world again: still false. | The value is kept in the world. | A dedicated server. |
| rules9 | `/bmh rule rand` in the chat box shows a suggestion row. Screenshots are kept. | The server answers `tab_complete_v2`. | That Tab applies it (seen in `z9-completion-applied.png`, not asserted). |
| o1, o2 | Game Mode: Hardcore: a click on the button changes nothing. Back out: it takes clicks again. | The button is greyed in Hardcore. | The debug world case and the grey look (read the screenshot). |
| o3 | Re-Create of a world made with the button OFF shows OFF, of a world with rules on shows ON. | The button follows the old world's rules. | See o4 and o5 for mixed values. |
| o4, o5 | Re-create a world with mixed rules. Repeat with two clicks on the Game tab. | The new world preserves all six values until a click sets them together. | |
| y1 to y3 | Quit, add `bmh:random_death_spawn_point` (a byte, 0) and `bmh:display_text_on_respawn` (a string, "false") to `data/minecraft/game_rules.dat`, delete `bmh/rules.txt`, launch again: `rules.txt` has those two false and four true (y1), `/bmh rule` lists them so (y2), the log says "java world = true" (y3). | The import of a Java world's rules works in single player. | A dedicated server and its console `/bmh rule` (no runner in `horizon/tools`), and a world that Java really saved. |
| m1 | A rule at false, `quit`, a second launch: the rule is still false. | Rules survive a restart. | |
| m2 | A hand edited config (3000, 999999, 99) before the second launch: a death uses the defaults (23333 blocks). | Out of range values give the key's default. | |
| m3 | After m2: the file holds 5000, 60000 and 5, still has its three `#Range` lines, and the log has exactly one line "The config file held a value outside its range". | The mod writes the corrected file back once, as NeoForge's config does. | A key that is missing from the file (the mod leaves it alone, as before). |
| p1 to p4 | `gamerule reduced_debug_info` in survival (true), creative (false), survival again (true), `/bmh rule remove_coordinates_in_survival false` (false), the rule on again (true); no "did not run" line. | The C14 forcing runs and restores. F3 reads this rule (area log). | What F3 draws: kept screenshots `docs/horizon/log/eternal-shots/c14-f3-*.png` are hand evidence. |
| p5 to p10 | Six Eternal Items, a Dormant Totem and an amulet, `/kill`: only the last two fall; `bmh_eternal_kept` is in a player file while dead and not after the respawn; the six are back once; the bundle keeps its stone; a second death gives no second copy. | C11 in the game, also `give_stack` from `player_respawned`. | A full inventory at the respawn, a Vanishing curse, `keep_inventory`. |
| p11, p12 | A restart while dead; a change of world while dead (world B gets nothing, world A gives the six once). | Check 4 and check 15 of the eternal log. | Two players. |
| p13 to p15 | A second Eternal Sword stays on the ground; its `Health` is 32767s; after `tick sprint 6000` it is still there and `Age` is negative. | E11 and E10. | Fire, lava, cactus, TNT, lightning (a hand check). |
| p16 to p18 | An Ender Chest put in the bundle comes out; a right click with a loaded bundle drops its stacks; an empty bundle drops nothing. | E12 for a bundle in a player slot. | A sword in a bundle (the game's bundle does not take it by clicks), a bundle in a chest or on the cursor, the off hand. |
| p19, p20 | A bed, night, health 16, sleep: Haste II and Resistance II are on; with `food_and_health_regen_on_wake_up` false none. | C13 (effects). | The food values, leaving the bed by hand. |
| q1, q2 | Night, clear, open sky: the action bar, the log line, two charges left on the stack, the advancement `navigation/stargazer`. | A2 in the game: the sky rules and `native_level` work in `use_item`. | The sound, the sky drawing, the Nether. |
| q3 to q6 | Rain, day, blindness, a roof: the four refusal messages. | A2 refusals in the Java order. | The dimension refusal, eyes in water. |
| q7, q8 | Three readings, then "No starlight left tonight."; the next night refills to 3. | A4 charges. | The hotbar sweep is checked by `cooldown1`. |
| cooldown1 | A reading at night: three screenshots (4, 24 and 80 ticks after the use) in `docs/horizon/solutions/amulet-cooldown-shots/`; PIL measures the white shade in the hotbar: about 0.9 of the icon, about 0.4, none. SKIP when PIL is missing. | The cooldown sweep of A4 is drawn and shrinks. | A LAN guest, the 20 tick sweep, the look in a live window. |
| q9 to q11 | Sneak and use on a bed and on a lodestone opens the mod screen; typing Camp and Enter saves ("Point saved: Camp"). | A3, A5, A6. | The look (kept screenshots `docs/horizon/log/amulet-shots/`). |
| q12 to q14 | `/bmh point save` twice inside one second (the second is ignored); 20 blocks away ("Stand next to"); no amulet in hand. | The server checks of `AmuletPointCommand`. | The same through the screen's own message. |
| q15, q16 | After a death and after a restart the next reading names the point Camp. | A5 persistence (attachment with `copy_on_death`). | A LAN guest, a Java world import. |
| t6, t7 | A Totem of Undying in the hand saves first (Regeneration and Absorption, no Fire Resistance) and the Totem of Eternity stays; two Totems of Eternity given: one stays, one drops. | T5 (order of the totems), T11 (the extra totem). | The container checks (chest, hopper), a LAN guest. |
| t8, t9 | Anvil: Hollow Totem and Echo Heart give a Totem of Eternity at 39 levels; an Eternal Sword (damage 20000, penalty 40) and two gold blocks give damage 3500 at 15 levels. | The anvil awakening and E6 (numbers read from `horizon-ctl menu`). | Typing a name in the anvil, the result's repair cost after a take. |
| t1 to t5 | A lethal `/damage @s 100` with a Totem of Eternity: the player lives, the totem is a Dormant Totem, the three save effects, the log line `saved ... and is now dormant`, a later `/damage @s 1` lowers the health. | T2, T3: the save from death. Red on 2026-11-18: see `game-checks-3.md`. | The void (not saved by design), a mob death. |
| t10 | `/kill` kills a player who carries a Totem of Eternity. | The totem does not save from `/kill`. | The void. |
| u1 | After a distant respawn the log has `Homecoming record: 1 homes ...`. | C10 `on_exile`. | A Nether exile, amulet points as homes. |
| u2, u3 | A teleport back to the exile center: the title `Home`, the subtitle `N blocks since you left`, the advancement `Home at last`. | C10 `arrive`, `show_title`, the award. | The bell, the second player, `The long way home` (30 000 blocks), the nights count. |
| u4 | `display_text_on_respawn false`: the walk back gives no title and the advancement still comes. | The rule gates the text only. | The bell with the rule off. |
| r0 | A vanilla iron and wooden pickaxe dig stone with `horizon-ctl break`; the 5 ticks of vanilla delay after a break are taken off. | The dig clock matches Java (8 and 23 ticks). | Digs in the air or under water. |
| r1 to r3 | A fresh, a worn (damage 32999) and a worn Efficiency V Eternal pickaxe dig stone: the worn one is slower than the fresh one, takes 12 ticks (Java 12), and 2 ticks with Efficiency V (Java 2); the block is still air 20 ticks later. | E4, the `break_speed` hook for the single-player owner. | A LAN guest or a dedicated server client (the case that E4 was written for), Mining Fatigue, jumping, water. |
| r4 | The pickaxe at 32998 digs five blocks: it is still there, damage 32998 or 32999. | E2, `item_damaged` caps the wear. | The sword and the axe, a repair. |
| r5 to r7 | Eternal sword, a death in survival: damage +2310; with `damage_eternal_item_when_dead false`: no change; a `/kill` in creative: no change. | C12, the death wear and its two gates. | The pickaxe and the axe, an in-game tooltip, the echo, the smash (they need hostile mobs). |
| s1 | By day (time 6000), a Dormant Totem and the use button held: no `Moon ritual started` line, the action bar says the moon is hidden. | The start is refused by day. | The cooldown of 20 ticks. |
| s2, s3 | At night (17850, moon straight up, pitch -90), 40 levels: the ritual starts; looking away fails it with `looked_away`; the totem stays; no `no acting player` line. | `native_acting_player()` answers for the single-player owner; the fail path. | A hit during the ritual, a release, a LAN guest, the chimes (muted). |
| s4 to s6 | A full ritual (340 ticks held): a Totem of Eternity, `Moon ritual done`, 40 levels become 10, the advancement `Moon-touched`. | T7, the tick, the level drain and the finish. | The hand pose, the particles, the sound, creative. `horizon-ctl xp N` does not set the level the mod reads: the scenario uses `xp add @s 40 levels`. |
| v1 to v3 | An amulet reading at night under open sky from a stone slab 60 blocks up, with a point Camp: `Star reading toward` lines, no `does not draw` line, the azimuth self-check line once with the expected values. | The sky area started the reading and its render pipeline built (check 16). | That the lines look right. |
| v4, v5 | Screenshots under `--out DIR` in `DIR/sky/` (1280 px): the chain over the stars, terrain hides the low end, a spyglass on the last star. The script only checks that the files exist. | Nothing about the look: a person must open them. | The name under the spyglass: it did not show in two runs (see game-checks-4). |

## SKIPs the test has

| Check | Why it can skip |
|---|---|
| a1 | `horizon-ctl options mods` may fail on a build without that page. |
| c | The `damage` command may not exist in Horizon. The test does not invent another way. |
| g1 | The advancement file may not be written by `save`. Then the test falls back to `advancement revoke`. If the reply is not readable, g1 skips. |
| d (look) | Not a skip, but the script cannot judge the screenshots. This part is always a hand check. |

Not testable with the commands that exist in `horizon-ctl` (no skip line, the check is absent): the sound, the music
fade, the mouse capture, the held keys during the black, the toast, the keyboard and gamepad paths of the death
screen, the frame cost. See the next section.

## Checks from the area logs that the script does not cover

`docs/horizon/log/game-checks-1.md` has the first in-game check of the settings and journey areas (what was settled, what is open). A person does the rest in a window (`test-slice.sh --window --keep` leaves a world, or start the game by hand).

| Area, number | What to do by hand |
|---|---|
| respawn 3, 4 | Die many times near an ocean. No landing in water, on leaves or on a beach of an ocean. The log shows released candidates. |
| respawn 5 | Sleep in a bed, die: the log has `Respawn point ... reads minecraft:<color>_bed[...]: Bed, centre <the bed>` and the exile is round the bed. Break the bed, die: `reads minecraft:air: Missing, centre <world spawn>`, the exile is round the world spawn and the chat says "You have no home bed or charged respawn anchor, or it was obstructed". Put stone round the bed so no spot is free, die: the mod still says `Bed` (no stand-up search, GAPS.md 3.3 C2). Watch for `No distant respawn` in the log. |
| respawn 5b | Sleep in a bed, walk 300 blocks away (the bed's chunk unloads), die: the log line `Respawn point ... reads` comes a tick or more later, then the search; break the bed first and the line says `Missing`. |
| respawn 5c | Dedicated server only (single player never hands a Nether point to the hook, lan_actions.rs:3929-3940): charge a respawn anchor in the Nether twice, use it, die in survival. `reads minecraft:respawn_anchor[charges=2]: Anchor`, the exile is in the Nether round the anchor at 23333 / 8 blocks, the anchor then shows 1 charge and the player hears the deplete sound. Use it until 0 charges, die: `Missing`, exile in the Overworld round the world spawn, chat line "no home bed". In single player the same anchor gives `No distant respawn` (Horizon respawns at the world spawn). |
| respawn 5d | `/setworldspawn` or `/spawnpoint <you>` while you are dead: the log says `changed after the death, using default respawn position` and the respawn is vanilla. |
| respawn 6 | Open `players/data/<uuid>.dat` of the world: it has the `respawn` compound. |
| respawn 7 | Make a hardcore world and die at once: no journey. |
| respawn 8 | The advancement screen shows "Not the end". A second player near the landing hears the bell. |
| respawn 9 | The toast of an advancement shows. |
| respawn 10 | After the arrival, jump from a height or stand in lava for 10 s: no damage. `/kill` still kills. |
| respawn 11, journey 2 | Click Respawn on the death screen: the button waits until `Landing spot ready`, then the respawn is at the far landing. A held click never respawns before it. |
| respawn 13 | Die ten times: "Shit, here we go again". |
| settings 1 | Make a Java world with BMH, run `/gamerule bmh:random_death_spawn_point false`, open it in Horizon, run `/bmh rule`. Check `rules.txt` and the log line `World rules created: java world = true`. |
| settings 3 | `/bmh rule` completes for an operator and not under level 2. The answer reads "Game rule bmh:x has been updated to: false". |
| settings 4 | Set a rule, restart the game and the world: the value stays. |
| settings 6, 7 | The Create World button: in the Game tab under Allow Commands, flips ON/OFF, a new world made while OFF has six `false` lines (k3, k4). The look is in `log/settings-fix-1-shots/`. The button is not greyed in Hardcore and does not follow the Game Rules screen (settings.md, "Differences"). |
| settings 8 | Options > Mods > BMH: six buttons, the labels fit, a click changes `config/bmh-common.toml`. |
| settings 9, 10 | Edit `config/bmh-common.toml` by hand: the next death uses the new distance within a second. The config loads in a dedicated server too. |
| journey 1, 3 | Die in the world, click Respawn: fade to black, no loading screen or world before the dissolve. Click at the button edges, Tab then Enter, Title Screen then No or Escape. |
| journey 4, 5, 6 | The fade starts in single player and on a server. The death screen closes about 600 ms after the click. The server log has no `Held respawn not told`. |
| journey 7, 8 | Hold W and press Escape and E during the black: nothing moves. At the dissolve the mouse comes back at once. |
| journey 9 | The dissolve never shows missing chunks. |
| journey 10, 11 | The music stops at the black and returns later. The chimes, the break and the bell play at the right moments. |
| journey 12 | The message shows in the player's language, at twice the text size; test the display text rule on and off. |
| journey 13, 15 | GUI scales 1 to 6 and a window resize: the stars are one or two pixels. F3 frame rate during the dissolve at a large window. |
| journey 14 | With the `immediate_respawn` rule on, die: black, sky, chain, dissolve. Also `horizon-ctl respawn` while the death screen is up (the test does this, but does not look at the result). |

## How to add a scenario when a new area is ported

1. Read the area log `docs/horizon/log/<area>.md`, section "To check in the game". Pick the items that a command and a
   state read can decide. The rest goes to the hand table above.
2. Check every `horizon-ctl` command you use: `horizon-ctl help <command>`. Use only commands that exist. When a step
   has no command, write `skip <id> "<why>"` and add a row to "SKIPs the test has".
3. Add a block `if wanted h; then ... fi` to `test-slice.sh` before the summary. Use the helpers: `ctl` (one call),
   `jget` (read a JSON field), `death_and_respawn MODE WAIT_LANDING`, `count_log PATTERN`, `pass`, `fail`, `skip`.
   Print the numbers you saw in every PASS and FAIL line.
4. Take the log lines from the area code: search it for `BalancedMcHardcoreMain`.
5. Add the scenario letter to the header of the script, to the table "The checks of test-slice.sh" (what it does, what
   a PASS proves, what it does not prove), and move the items it covers out of the hand table.
6. Run `bash -n horizon/tools/test-slice.sh`, then the scenario alone: `test-slice.sh --only h`.

## What was run on 2026-11-11

No game was started. Paths with `<tmp>` are a temporary folder that was removed. `<cache>` is the cache folder of
this session, where the Horizon source tree of the mod's commit is.

`bash -n` on both scripts, run with `/bin/bash` 3.2.57:

```
$ bash -n horizon/tools/prepare-game.sh && bash -n horizon/tools/test-slice.sh && echo SYNTAX_OK
SYNTAX_OK
```

shellcheck:

```
$ command -v shellcheck
(no output: shellcheck is not installed on this machine, so it did not run)
```

`prepare-game.sh` with nothing installed (empty `HOME`, empty cache), exit 78:

```
MISSING: Minecraft 26.3 is not in '<tmp>/home/Library/Application Support/minecraft' (missing: versions/26.3/26.3.json versions/26.3/26.3.jar libraries/ assets/indexes/). Install the official Minecraft launcher, start the game version 26.3 once, quit it, then rerun. If your launcher keeps its files elsewhere, pass --minecraft-dir DIR.
MISSING: The 26.3 server jar is not given. Download it (the URL is downloads.server.url in versions/26.3/26.3.json of the Minecraft folder, or minecraft.net/download/server), then rerun with --server-jar PATH or set HORIZON_SERVER_JAR.
NOTE: the JDK version is not checked: research-lock.json is not in the cache yet (see the Horizon source line).
MISSING: The Horizon source tree is not in the cache (<tmp>/home/Library/Caches/bmh-horizon/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca). Run: sh horizon/tools/check.sh (it clones and compiles it once), then rerun.
prepare-game.sh: 3 thing(s) missing. Nothing was done.
exit=78
```

The same with `--dry-run` printed the same three `MISSING` lines, then the two commands it would run (`cargo run
--release -p horizon-prepare -- --prepare ...` and `-- --check ...`, in the missing tree, with `JAVA_HOME='<JDK>'`),
then "3 thing(s) missing. Nothing was done." and exit 78.

`prepare-game.sh` with the real cache (the tree and `research-lock.json` are there, JDK 25 is installed), a fake
Minecraft folder built in a temporary folder (empty version json and jar, empty `libraries/` and `assets/indexes/`),
a fake server jar, and `--dry-run`. The Minecraft check passes, the JDK is found, the sha1 check fails, exit 78:

```
MISSING: The server jar '<tmp>/fake-server.jar' has sha1 9e38b12405b3a73bf8cb49876cd36b93bf9f1a17, but research-lock.json pins 33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c. Download the official 26.3 server jar again.
prepare-game.sh: commands (tree <cache>/horizon-mods/native/target/horizon/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca):
  (cd '<tree>' && JAVA_HOME='/opt/homebrew/opt/openjdk@25' '/Users/louisobenicherousse/.cargo/bin/cargo' run --release -p horizon-prepare -- --prepare --server-jar '<tmp>/fake-server.jar' --minecraft-dir '<tmp>/mc' --output '<cache>/resources/26.3')
  (cd '<tree>' && JAVA_HOME='/opt/homebrew/opt/openjdk@25' '/Users/louisobenicherousse/.cargo/bin/cargo' run --release -p horizon-prepare -- --check --output '<cache>/resources/26.3')
prepare-game.sh: 1 thing(s) missing. Nothing was done.
exit=78
```

`prepare-game.sh` with the real cache and an empty `HOME`: two `MISSING` lines (Minecraft, server jar), JDK and tree
found, exit 78. `prepare-game.sh --bogus`: `unknown option '--bogus'.`, exit 64.

What these runs do **not** show: the success path of `prepare-game.sh` (it needs a real server jar with the pinned
sha1), and whether `horizon-prepare` accepts the Minecraft folder of a real launcher.

`test-slice.sh` with no prepared files (empty cache), exit 77:

```
SKIP: no prepared game files. Run horizon/tools/prepare-game.sh first.
exit=77
```

`test-slice.sh --help` printed the usage block at the top of the script and exited 0. `test-slice.sh --bogus` printed
`unknown option '--bogus'.` and exited 64.

Checks of the commands the scripts use, against the built `horizon-ctl help <command>`: `launch`, `wait-for`,
`events`, `state`, `command`, `respawn`, `screenshot`, `quit`, `key`, `wait`, `ui`, `options`, `click-button`, `logs`,
`save`, `block`. All exist with the syntax used. `resume` and `messages` exist in the command list. The game flags
`--resources`, `--game-dir`, `--home`, `--dev-mod`, `--offscreen` are in the game's argument parser
(`crates/horizon-client/src/game.rs`).

Not run and not checked: every scenario of `test-slice.sh`. Reply shapes the script assumes, from
`HN/tools/agent_control_e2e.py`: `state.player.{position,block_pos,health,on_ground,dimension}`, `events.last`,
`block.block`, `launch` reply with an optional `pid`. Open questions for the first run: whether `launch --wait`
accepts the game arguments after `--wait` as written, whether `click-button respawn` reaches the mod's hold, whether
`damage` and `advancement revoke` exist, whether `save` writes the advancement file, and whether the command reply
carries the text of `/bmh rule` (a2 also reads `messages`).

## What was run on 2026-11-17

The script ran against the real game (offscreen, no audio). Result of the last full run: 14 passed, 4 failed, 0 skipped.
The details, the screenshots and the defects are in `docs/horizon/log/first-run.md`. Script changes:

- The launch reply is the whole game state: the pid is `launch.pid`. The script prints a short form.
- `state` has `dimension` at the top level, not in `player`.
- New checks: b0 (fresh world, no bed, runs before a), b0s (the same after a save), a4 (the content mod loaded), b4 (no
  Sound or Advancement warning), g1 fails on "Unknown advancement" instead of skipping.
- All scenarios after b0 set a `/spawnpoint` on the player's block first (`ensure_point`) and measure from it. Without it the
  mod gives the vanilla respawn (defect 1), and the rest of the chain could not be tested.
- Scenario c hits at about tick 140 (vanilla immunity is 60 ticks), then once more after the end.
- `prepare-game.sh` runs `horizon-prepare sounds --minecraft-dir DIR --output OUT` after `--prepare`. Checked with `bash -n`
  and `--dry-run` only.

The open questions of the first run (reply of `click-button respawn`, `damage`, `advancement revoke`, `save`) are answered:
`damage` and `advancement revoke` exist, `click-button respawn` works, whether `save` writes an advancement file is open (the advancement data was not loaded).

## What was run on 2026-11-17, respawn fix round 1

`sh horizon/tools/test-slice.sh --keep --out ~/Library/Caches/bmh-horizon/scratch/respawn-fix1-shots` (with the build),
3 min 23 s, offscreen, no audio. `pgrep -fl horizon-client` printed nothing after it. Result: 24 passed, 0 failed,
0 skipped (a1, a2, a3, a4, b0, b0s, b1, b2, b3, b4, c1, c2, c3, d3, e1, e2, e3, f1, g1, h1, i1, i2, j1, j2). Details in
`docs/horizon/log/respawn.md`, "Fix round 1". New: b4 checks every WARN line of the mod, c3, h, i, j; the launch is the
function `launch_game` (j launches a second time; without `--wait` the reply has no pid, so the script finds the game
by its `--game-dir`).
