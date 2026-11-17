# First run in the real game, 2026-11-17

The mod ran in a real Horizon game for the first time (Nightly fdc87ae, Minecraft 26.3 files prepared, offscreen,
no audio, Apple M5, Metal). Paths: screenshots in `/Users/louisobenicherousse/Library/Caches/bmh-horizon/scratch/shots-final`
(last run) and `.../scratch/shots-run4` (the run before it). The scratch worlds were deleted.

## Step log

1. `sh horizon/tools/test-slice.sh --only a --keep`: the game built, started, and a1 to a3 passed on the first try. The launch
   reply holds the whole game state: the pid is in `launch.pid`, not at the top (script fix 1).
2. `--only b`: b1 and b2 failed: "No distant respawn" in the log, a vanilla respawn 3 to 11 blocks from the start.
3. Own probes (three throwaway scripts in the scratchpad, same launch flags): the search always ends ("Landing spot
   ready", 150 to 3700 ms). With `/spawnpoint` set first, the respawn is distant and right. With no bed it is vanilla,
   unless the world was saved before the death. See defect 1.
4. Script changes: `state` has `dimension` at the top, not in `player`; `ensure_point` (`/spawnpoint` on the player's
   block) for every scenario after b0; b0 and b0s; a4; b4; g1 reads "Unknown advancement" as a FAIL; c hits at tick
   140 instead of 50.
5. Full run 4: 14 PASS, 0 FAIL, 1 SKIP. A look at the log showed the Sound and Advancement warnings, so b4, a4 and g1 were
   added, and b0 was moved before scenario a (a1 saves the world and hid defect 1).
6. Full run 5 (final): 14 PASS, 4 FAIL, 0 SKIP. `pgrep -fl horizon-client` printed nothing after every run.
7. `prepare-game.sh`: the sounds step added after `--prepare` (`horizon-prepare sounds --minecraft-dir DIR --output OUT`).
   `bash -n` is OK, `--dry-run` prints the three commands. The preparation was not run again.

No crash, no panic, no crash report in any run.

## What was run

- `sh horizon/tools/test-slice.sh --only a --keep --out ...` (run 1), `--no-build --only b`, `--only b,c`, then the full
  `--no-build --keep --out .../shots-run4` and `.../shots-final`.
- `horizon-ctl` calls in own probes: `state`, `status`, `command "spawnpoint @s 3 127 3"`, `command "kill @s"`,
  `respawn`, `save`, `wait 60`.

## Scenarios

| Scenario | Result | Evidence |
|---|---|---|
| a1 Options > Mods | PASS | `options mods` opened, `ui` names the mod. Weak: the script greps "bmh" in the widget list. |
| a2 `/bmh rule` | PASS | log: `[CHAT] Game rule bmh:random_death_spawn_point is currently set to true` and five more. |
| a3 rules.txt | PASS | `saves/<world>/bmh/rules.txt`: header and six `name = true` lines. |
| a4 content loaded | FAIL | log: `Horizon: mod bmh content not loaded: bmh:navigator_amulet: minecraft:lodestone_tracker is not a known component type` |
| b0 fresh world, no bed | FAIL | 3.6 blocks from the start, expected 23333; log `Landing spot ready after 1649 ms: [-6829, 64, -22311]` then `No distant respawn for Player..., using default respawn position`. |
| b0s no bed, after a save | PASS | 23334.0 blocks from the world spawn. Log: `new respawn pos and bed : [..], [0, 126, 0]`. |
| b1 with `/spawnpoint` | PASS | 23332.0 blocks (formula: 23333). Run 3: 23333.3. |
| b2 | PASS | `new respawn pos and bed : [941, 65, 23320], [5, 127, 6]` and no new "No distant respawn". |
| b3 safe landing | PASS | health 20.0, on ground, on `minecraft:grass_block`, overworld, y=67 (run 5). Four landings seen this session were on grass or similar, none in water. |
| b4 sounds and award | FAIL | log: `Sound command failed: playsound bmh:bell_sfx player @a[name=!Player08996] 22247 65 -7034 1 1`, the same for `bmh:end`, and `Advancement bmh:journey/exile not awarded to Player08996` after every distant respawn. |
| c1 immunity at tick 140 | PASS | health 20.0 before and after `damage @s 6` (vanilla gives 60 ticks, so this proves more than vanilla). |
| c2 after the immunity | PASS | health 20.0 then 14.0 after 150 more ticks. |
| d3 screenshots | PASS | 15 files, none empty. See below. |
| e1 to e3 rule off | PASS | rules.txt shows `false`, respawn 0.7 blocks from the spawn point, no landing line, rule back to true. |
| f1 creative | PASS | 0.7 blocks from the spawn point, no landing line. |
| g1 advancement | FAIL | `advancement revoke @s only bmh:journey/exile` answers `Unknown advancement: bmh:journey/exile`. |

## Screenshots (looked at with the Read tool)

Folder `.../scratch/shots-run4` (the same steps are in `shots-final`):

- `01-death-screen.png`: vanilla "You Died!" screen, score 0, buttons Respawn and Title Screen, red tint over the world. The
  chat shows "Applied 6.0 damage" and "Killed". The mod changes nothing visible here.
- `02-arrival-after-40-ticks.png`: full-screen dark blue night sky, many stars, a purple constellation of three points with
  rings, a pale message at the bottom: "A new journey begins for you, Player35192." This is the cinematic.
- `06-journey-01.png`: almost black sky with small stars, one ringed star, no message (the fade or the dissolve).
- `08-journey-03.png`: night sky with a long constellation of nine ringed points and the message again.
- `10-journey-05.png`: the Overworld in daylight, grass, a savanna and jungle trees, hearts and food full, the vanilla toast
  "Move with W, A, S and D". This is the arrival. No chunk holes seen in this frame.

The sequence from the click: death screen, stars with the message, a dim frame, stars with the message, the world. The black
frame and the pixel dissolve were not isolated: the frames are one second apart. Judging the look stays with the owner.

## Works in the game

- The mod loads and its hooks run: `World rules created: java world = false, new world = true` (log).
- `/bmh rule` lists the six rules; `/bmh rule random_death_spawn_point false` writes rules.txt (e1).
- The distant respawn: with a respawn point, 23332 to 23333 blocks away, as the Java formula says, on safe ground (b1, b3).
- The search ends in 150 to 3700 ms and the log line `Landing spot ready` comes before the respawn (all runs).
- The 200 ticks of damage immunity hold at tick 140 and end later (c1, c2).
- The rule off and creative give the vanilla respawn (e2, f1).
- The journey cinematic draws: night sky, constellation, message with the player name, then the world (screenshots).
- `horizon-ctl click-button respawn` reaches the Respawn button and the respawn happens at the far landing (scenario d,
  the 15 frames, then a distant position).
- `run_command` from the hooks does run, but its commands fail for missing data (defect 2), so the hook path itself is not proven.

## Mod defects seen in the game

1. Respawn, major. A player with no bed and no `/spawnpoint` in a new world gets the vanilla respawn, until the world is saved
   once. Expected: the distant respawn round the world spawn. Evidence: log `Landing spot ready after 2447 ms: [-12113, 76, 19941]`
   then `No distant respawn`; after `horizon-ctl save` the same death gives `new respawn pos and bed : [4558, 80, 22883], [0, 126, 0]`.
   The player's vanilla respawn was a random spot of the spawn area (for example `[-9.5, 121.0, 0.5]`). Probable cause: the
   mod compares the point the game hands over with the world spawn it reads from `level.dat`, and a new world's `level.dat`
   has `spawn.pos` `[0, 64, 0]` until the first save (`[0, 126, 0]` after it). Not confirmed in the code. Reproduce:
   `sh horizon/tools/test-slice.sh --only b` (b0 fails, b0s passes). A 25 s wait before the death changed nothing.
2. Content, major. The content mod does not load: `minecraft:lodestone_tracker is not a known component type` (the
   Navigator's Amulet). So the items, the language file and the 20 advancements are absent, even for items unrelated to the
   amulet. Reproduce: `sh horizon/tools/test-slice.sh --only a` (a4).
3. Respawn, major, follows from 2. The arrival sounds `playsound bmh:bell_sfx` and `bmh:end` fail and the advancement
   `bmh:journey/exile` is not awarded (`Unknown advancement`). Reproduce: `--only b,g` (b4, g1). Whether the sound ids exist
   in the content was not checked. The reply text of the failed `playsound` was not captured.
## Open questions settled (GAPS.md section 7 and the area logs)

- respawn 1, `level_tick_post` fires while the owner is dead: yes. `Landing spot ready` is logged during the death screen.
- respawn 2, a far chunk held with `force_chunk_in` loads in seconds and gives a safe spot: yes, 150 to 3700 ms, on grass.
- respawn 3, collision, block and waterlogged reads from `level_tick_post`: not shown wrong; four landings were on grass.
  Ocean cases not tested.
- respawn 5, the point copy matches what the game hands over: yes with a `/spawnpoint`; no for the world spawn (defect 1).
- respawn 7, `level.dat` has spawn and hardcore in a new world before the first death: it exists, but its spawn is stale (defect 1).
- respawn 10, immunity: yes for damage from `damage @s`. Lava, fall and `/kill` were not tested.
- respawn 11, journey 2, the Respawn button waits for `Ready`: not proven. `click-button respawn` works, and the landing
  was ready first in every run, so a wait could not show.
- respawn 8, 9 (advancement screen, toast): not settled, the data is not loaded (defect 2).
- settings 1 to 4 partly: rules file written at the first open, `/bmh rule` answers, rule change takes effect with no restart
  (e2). Operator level, restart and Java import not tested.
- The default config gives 23333 blocks at Normal with no Eternal Item: yes (b1).
- `horizon-ctl respawn` while the death screen is up works (journey 14, only the move, not the look).
- The `damage` command exists in Horizon. `advancement revoke` exists.

## Still open

Oceans and leaves, bed and anchor centres, the Nether, hardcore, the toast, the sounds, the hold of the click, keys during
the black, music, GUI scales, a dedicated server, restart persistence, the Create World button, the Mods page buttons.

## Final PASS, FAIL, SKIP lines (run 5)

```
PASS a1, a2, a3
FAIL a4: the game did not load the content mod: Horizon: mod bmh content not loaded: bmh:navigator_amulet: minecraft:lodestone_tracker is not a known component type
FAIL b0: fresh world, no bed: 3.6 blocks from the world spawn, expected 23333
PASS b0s, b1, b2, b3
FAIL b4: 6 warning line(s) in the log: Sound command failed: playsound bmh:bell_sfx ...; Advancement bmh:journey/exile not awarded to Player08996
PASS c1, c2, d3, e1, e2, e3, f1
FAIL g1: the game does not know bmh:journey/exile (the advancement data is not loaded)
14 passed, 4 failed, 0 skipped
```
