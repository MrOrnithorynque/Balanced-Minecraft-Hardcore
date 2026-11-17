# Game checks 1: settings and journey in the real game

Date: 2026-11-17. Branch `horizon-26.3`. The three settings defects of the last table are fixed: see `settings.md`, "Fix round 1". Game: the built `horizon-client` of
`~/Library/Caches/bmh-horizon/horizon-mods/native/target/cargo/fast` (offscreen, no audio, window 2560 x 1440, GUI
scale auto = 6, GUI size 427 x 240). Scratch home and worlds only. I changed no code of the mod.

## Steps of the first run

1. `git rev-parse --show-toplevel` and `branch --show-current` gave the worktree and `horizon-26.3`. `pgrep -fl horizon-client` was empty.
2. Read `.claude/CLAUDE.md`, `TESTING.md`, `test-slice.sh`, the two area logs, GAPS section 7, `bmh_settings.rs`, `bmh_journey.rs`.
3. Launched the game with an empty home (`horizon-ctl launch`, `--dev-mod` the content folder). First screen: accessibility
   onboarding (click `continue`), then the title. Singleplayer opened the Create New World screen at once (no world saved).
4. Check 1: screenshots of the Create World screen at GUI scale auto (6) and 3, clicks, two worlds, `rules.txt` of both.
5. Check 3: ten `/bmh rule` lines. Check 2: Options > Mods, the settings page, clicks on all six buttons, limits, file.
6. Set min = max = 15000 through the page, died: 14999.7 blocks from the point, no restart.
7. Set a rule to false, quit, hand edited the config out of range, launched the same world: rule still false, defaults apply.
8. Check 5: died, waited for the landing line, `click-button respawn`, then a loop of screenshot plus `state` (about 30 frames a second,
   221 frames). Looked at the frames as contact sheets.
9. Checks 6, 7, 8, 9 and the `horizon-ctl respawn` path (below).
10. Wrote the scenarios k, l, n, m into `test-slice.sh`, ran each alone with `--only`, fixed three script bugs of my own
    (`launch_game --wait` in m, an extra amulet click in l, python warnings in n), then one full run.
11. After each run `pgrep -fl horizon-client` printed nothing.

## Check 1: the Create New World button

- Drawn: yes. "Balanced Hardcore: ON" at the top right of the screen, 210 x 20 GUI pixels, vanilla button sprite, white text.
  Evidence: `game-checks-1-shots/create-world-default-scale-button-over-tabs.png`.
- Position: it is NOT in the Game tab. It lies over the tab bar. At the default GUI scale of this window (GUI width 427) the tabs are
  Game 28 to 152, World 152 to 276, More 276 to 400 and the button spans about 213 to 423. So it covers the right half of the World
  tab and the whole More tab (the picture shows "Wo" cut off and no More tab). At GUI scale 3 (width 854) there is no overlap
  (`create-world-gui-scale-3-no-overlap.png`).
- A click at (380, 12), on the More tab, flipped the button and did not open the tab. The script's k2 does the same with the tab's middle
  and fails. The More tab cannot be reached with the mouse at this GUI scale.
- Flips: yes. Four clicks gave OFF, ON, OFF, ON; the choice file `horizon-mods/.horizon-mod-data/bmh/balanced_hardcore` held `false` then `true`.
- Two worlds: the world made with OFF (`worldoff`) has `<world>/bmh/rules.txt` with six lines `= false`. The world made with ON
  (`worldon`) has six lines `= true`. The log has "World rules created: java world = false, new world = true, choice = Some(false)" and
  `Some(true)`.
- Persistence: after the OFF world, a new Create World screen opened with OFF (screenshot of the second screen read "Balanced Hardcore: OFF").
  The Java button starts ON on each new screen (`CreateWorldGameTabMixin.java:39`: its state is the rules of the screen). A player
  who turned it off once and forgets makes a world with no BMH rule and gets the vanilla respawn. Script check k5 fails on this.
- The tooltip (the "turning it off disables every BMH game rule" text) shows on hover, drawn at the mouse. It covers the label while the
  mouse is on the button.

## Check 2: Options > Mods and the settings page

- The row exists: "Balanced Minecraft Hardcore 1.9.0: ON" with a Settings button. The page "Mod Settings" has six buttons, all
  labels fit (`mod-settings-page-six-buttons.png`):
  `[+] Minimum respawn distance: 5000`, `[-] Minimum ...`, `[+] Maximum respawn distance: 60000`, `[-] Maximum ...`,
  `[+] Amulet points: 5`, `[-] Amulet points: 5`.
- Clicks (value shown | file `config/bmh-common.toml`):
  - min `[-]` at 5000: stays 5000. min `[+]` twice: 15000, file `minDistanceFromBed = 15000`.
  - max `[-]` twelve times (from 60000): stops at 15000 (the min). Another min `[+]`: stays 15000. So min cannot pass max, and max cannot go below min.
  - max `[+]` 40 times: 200000 (the top). Points `[-]` five times: 1. Points `[+]` 20 times: 16.
  - Each click changes the label and writes the file in the NeoForge layout with the comments and `#Range:` lines.
- The next death: min = max = 15000 set through the page, then `kill`, `respawn`: 14999.7 blocks from the point (the default formula
  would give 23333). No restart.
- No error line and no free number: as the area log says.

## Check 3: `/bmh rule`

| Command | Reply |
|---|---|
| `bmh rule` | six lines "Game rule bmh:<name> is currently set to true" |
| `bmh rule display_text_on_respawn` | "Game rule bmh:display_text_on_respawn is currently set to true" |
| `bmh rule display_text_on_respawn false` | "... is now set to false" |
| `bmh rule nonsense` and `bmh rule nonsense true` | "Unknown BMH rule: nonsense" (red) |
| `bmh rule display_text_on_respawn maybe` | "Expected true or false, found: maybe" |
| `bmh rule display_text_on_respawn false extra` | "Usage: /bmh rule [<rule> [true|false]]" |
| `bmh` | "Unknown or incomplete command. See below for error" |
| `bmh rule DISPLAY_TEXT_ON_RESPAWN true` | "Unknown BMH rule: DISPLAY_TEXT_ON_RESPAWN" (case matters) |
| `bmh rule bmh:display_text_on_respawn true` | accepted (the `bmh:` prefix is allowed) |

- A rule set to false survives a quit and a second launch of the same world (script m1): the reply after the launch was "currently set to false".
- Operator rights: not checked. `horizon-ctl command` runs with the owner's rights, a world made with Allow Commands OFF accepts it, and the
  game has no `op` or `deop` command (both answer "Unknown or incomplete command"). The control tool cannot lower the rights.

## Check 4: the config file by hand

Edited while the game was closed: `minDistanceFromBed = 3000`, `maxDistanceFromBed = 999999`, `maxAmuletPoints = 99`. After the start:
the page showed 5000, 60000 and 5 (each key took its own default), the next death landed 23332 blocks from the point (the formula with
the defaults gives 23333), and the file still held 3000, 999999, 99: the mod did not rewrite it. NeoForge's own config writes the
corrected value back. Not a gameplay defect: a later click on the page overwrites the whole file.

## Check 5: the real death screen path (`click-button respawn`)

221 frames, about 30 a second, from the click. Contact sheets: `journey-click-fade-black-sky.png`,
`journey-sky-chain-message-dissolve-start.png`, `journey-dissolve-into-world.png`.

| Time after the click | What the frames show |
|---|---|
| 0.0 to 0.3 s | the death screen, then it darkens (buttons grey) |
| 0.3 to 0.6 s | the fade; one frame at 0.56 s is nearly black with a ghost of the death screen |
| 0.6 s | the screen name changes to `mod`, the player is at the new place |
| 0.65 to 1.0 s | black |
| 1.0 s on | the night sky comes in; stars, the dust band |
| about 1.5 s | the home star ring appears |
| 2.0 to 3.0 s | the star chain grows from the home star, the view pans |
| 3.0 to 4.5 s | the message "A new journey begins for you, Player98132." fades in under the chain, then out |
| 4.7 s | the screen name is `playing`; the pixel dissolve (green and brown dots) opens on the world |
| 4.7 to 5.7 s | the dissolve; the HUD, the toast and the chat come with the world |

- Wrong frames: none found. The brightness of each frame (mean grey of a 64 x 36 copy) goes 80, 67, 41, 15, 1, 0, 0 ...: the
  fade is smooth, there is no bright frame between the first dark frame and the dissolve (the script found none in three runs), no
  loading screen, no HUD, no flash of the old place or of the world spawn.
- The message is one line, centred across the screen and not cut. Its size against the Java text (twice the GUI text size) was not measured.
- The timeline matches the constants of the Java overlay (600 ms fade, 250 black, 1200 sky, message at 1700 ms).
- A dead world reopened after a quit (the player dead at the load) runs the same journey (frames not kept): fade, black, sky, message.
- The first screen of the dissolve shows the new place at once, no loading screen of Horizon.

## Check 6: `display_text_on_respawn false`

Script n2: the journey ran (53 frames of the hold screen, 8 black frames) and no frame had more than 10 cream pixels (the message colour
0xFFF1D0). With the rule on (n1) the most was 356 pixels. The message is off with the rule off, the rest of the journey stays.

## Check 7: a journey must always end

- Rule `random_death_spawn_point` false: `click-button respawn` goes straight to `playing`, every frame has the same brightness (129), no
  journey. Creative: the same, vanilla respawn at the point.
- Input: during the `mod` screen, W held, then Escape, E, Space tapped, and a left click: the position stayed the same for the whole
  journey (39 to 50 frames) and the screen stayed `mod`. After the journey `move forward 1` moved the player 1.2 blocks and
  `mouse-move` turned the view. The pointer is grabbed again (`pointer_grabbed` true). Script n3 and n4.
- Close the game during the journey (`quit` 1.5 s after the click, screen `mod`): the next launch opened the world with the player alive
  at the exile place, health 20, no overlay (`game-checks-1` run, not kept).
- Title Screen on the death screen, then the second "Title Screen" (leave): back at the title with no journey, no error.
- Quit to the title during the black: the control tool cannot do it (`open` answers "a world is already open", the hold screen takes
  every key). Not checked.

## Check 8: the log

The only line of the mod with `WARN` in all runs: `[BalancedMcHardcoreMain] The landing search of Player... did not end, using default
respawn position`, and it came with `immediate_respawn` true (check 9). No `ERROR`, no panic. The game's own WARN is the audio-off line.
No line of the game about the mod's screens or messages.

## Check 9: `immediate_respawn` true

`gamerule immediate_respawn true` works in the control tool. Then `kill @s`: the player is back alive after 0.46 s at the spawn point
(4, 128, 4), 0.7 blocks from it. No black, no sky, no journey. The log says `The landing search ... did not end, using default respawn
position`. So the exile does not happen: the area log says the cinematic starts in black at the arrival, but here even the exile is
gone. `immediate-respawn-no-exile-no-journey.png` (frames 0, 1, 2, 4, 8, 31: the world, no black). Script n5 fails on this.

## `horizon-ctl respawn` while the death screen is up (GAPS 7, question 16)

The command skips the hold. Frame at 0.4 s after the command shows the arrival world for one frame (sky, hand, hotbar), then black,
the sky, the chain and the message as usual (`journey-ctl-respawn-flash-of-arrival-world.png`, third picture of the first row). So the
cinematic does start, in black after one frame of the new place, with no fade. The same would hold for a gamepad click.

## Mod defects seen

| Area | Severity | What | Expected | Evidence | Reproduce |
|---|---|---|---|---|---|
| settings | major | FIXED 2026-11-17 (settings fix round 1). The Create New World button lies over the World and More tabs at GUI width 427 (default scale of 2560 x 1440). A click on the More tab flips the button; the More tab cannot be opened with the mouse. | The button must not hide a tab (the Java button sits in the Game tab). | `create-world-default-scale-button-over-tabs.png`, script k2. Now: the button sits in the Game tab under Allow Commands, and only there (`settings-fix-1-shots/`, k1, k2, k2b pass). | `bash horizon/tools/test-slice.sh --only k` |
| settings | minor | FIXED 2026-11-17 (settings fix round 1). The OFF choice stays for the next Create New World screen. | The Java button starts ON on each new screen. | script k5, second Create screen read "OFF". Now: k5, k5c and k5d pass; `settings-fix-1-shots/game-tab-gui854-new-screen-starts-on.png`. | `bash horizon/tools/test-slice.sh --only k` |
| respawn | major | With the game rule `immediate_respawn` true, a death gives the vanilla respawn at the point and a WARN: no exile, no journey. | The exile happens as with the death screen. | `immediate-respawn-no-exile-no-journey.png`, log WARN, script n5 | `bash horizon/tools/test-slice.sh --only n` |
| journey | minor | After `horizon-ctl respawn` (and so the gamepad path) one frame of the arrival world shows before the black. | The cinematic starts in black. | `journey-ctl-respawn-flash-of-arrival-world.png` | `horizon-ctl command "kill @s"`, wait for the landing line, `horizon-ctl respawn`, then `screenshot` every 0.1 s |
| settings | minor | FIXED 2026-11-17 (settings fix round 1). A hand edited config value out of range is not written back (the page shows the defaults, the file keeps the bad value). | NeoForge corrects the file. | check 4. Now: m3 is a real check and passes (the file holds 5000, 60000, 5 after the start, one rewrite line in the log). | `bash horizon/tools/test-slice.sh --only m` |

## Settled

- GAPS 7 question 2: `gui_render_post` covers every frame between the click and the arrival on the click path: no frame of the death
  screen, the loading screen, the HUD or the old place shows through the black (frame brightness table above).
- journey.md "To check" 1, 2, 3 (click on Respawn starts the fade; the click is held until the landing), 4, 5, 6 (the respawn leaves after
  about 600 ms, the screen name changes), 7 and 8 (the hold takes the keys, the mouse comes back), 12 (message and its rule), 14
  (`horizon-ctl respawn` runs the cinematic, in black after one frame), partly 9 (the dissolve shows terrain at once).
- settings.md "To check" 2 (`world_lifecycle` writes `rules.txt`), 4 (the rule is read at once and survives a restart), 6 and 7
  (the button is drawn and flips, but it lies over the tabs), 8 (the six buttons fit), 9 (a changed distance applies with no restart; the
  config reload works), 10 (`config_loading` fires in the integrated server: the value read is the file's).
- GAPS 7 question 16: see the `horizon-ctl respawn` section.
- GAPS 7 question 11: not needed here: the world spawn and `rules.txt` read fine in a new world (the choice reached the file at game time 0).

## Still open and why

- Operator level of `/bmh rule`: the control tool runs commands with full rights and has no `op` or `deop`.
- Quit to the title during the black: no control tool path (the hold screen takes the keys, `open` is refused).
- Journey 10, 11 (music and sounds): audio is off in these runs.
- Journey 13, 15: only GUI scale auto (6) was run for the journey. No frame rate was read.
- Journey 9 (missing chunks at the dissolve): the frames show terrain at once, but `render-distance` and slow disks were not tried.
- Settings 1 and 5: no Java world and no amulet area in these runs.
- The look of the Create World button against the tabs at other window sizes: only GUI width 427 and 854 were seen.

## Screenshots kept

`docs/horizon/log/game-checks-1-shots/`: `create-world-default-scale-button-over-tabs.png`, `create-world-gui-scale-3-no-overlap.png`,
`mod-settings-page-six-buttons.png`, `journey-click-fade-black-sky.png`, `journey-sky-chain-message-dissolve-start.png`,
`journey-dissolve-into-world.png`, `journey-ctl-respawn-flash-of-arrival-world.png`,
`immediate-respawn-no-exile-no-journey.png`.

## Scenarios added to `test-slice.sh`

k0 to k6 (the button and two worlds), l0 to l7 (page, limits, file, next death), n1 to n5 (message rule, held input, immediate
respawn), m1 and m2 (rule and config across a restart). Expected failing now: n5 (a mod defect). k2 and k5 pass since
settings fix round 1, and m3 is a real check there (k2b, k5c, k5d are new).

## Full run of `test-slice.sh`

`bash horizon/tools/test-slice.sh --no-build`, about 6 minutes: `43 passed, 6 failed` on the first full run, then, after three fixes of my own
script (wait for the end of the running journey before leaving the world, no vacuous k2, count message pixels only on hold screen
frames), `test-slice.sh: 43 passed, 3 failed, 0 skipped`. The three FAIL lines are mod defects and stay failing: k2, k5 and n5.
`pgrep -fl horizon-client` was empty after every run.
