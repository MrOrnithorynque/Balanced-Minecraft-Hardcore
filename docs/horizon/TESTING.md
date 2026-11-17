# Testing the Horizon mod

Status on 2026-11-17: `test-slice.sh` ran against the real game (see `docs/horizon/log/first-run.md`). The script bugs
of the first run are fixed. The four FAILs left are mod defects. `prepare-game.sh` now also runs the sounds step.

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
| `sh horizon/tools/test-slice.sh` | The in-game test of settings, respawn and journey. | Prepared game files |

`test-slice.sh` options: `--only a,b` runs some scenarios, `--keep` keeps the scratch folder, `--window` shows a
window instead of rendering offscreen, `--no-build` skips `check.sh build`, `--resources DIR`, `--bin-dir DIR`,
`--out DIR` (where the screenshots go). Exit codes: 0 all checks passed, 1 a check failed, 77 no prepared files,
3 the game did not start, 64 a bad option.

How it works: it builds the game with `check.sh build` (the last line of that output is the folder of the binaries),
makes a scratch folder in the cache with its own `HORIZON_HOME` and game folder, launches the game with
`horizon-ctl launch --offscreen`, the content mod as `--dev-mod`, and a new world with seed 20261111, then drives it.
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

## The checks of test-slice.sh

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

A person does these in a window (`test-slice.sh --window --keep` leaves a world, or start the game by hand).

| Area, number | What to do by hand |
|---|---|
| respawn 3, 4 | Die many times near an ocean. No landing in water, on leaves or on a beach of an ocean. The log shows released candidates. |
| respawn 5 | Sleep in a bed, die: the exile is round the bed. Break the bed, die: round the world spawn. `/spawnpoint` somewhere, die. Watch for `No distant respawn` in the log. |
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
| settings 6, 7 | The Create World button: top right, not under the tab bar at a large GUI scale, flips ON/OFF, a new world made while OFF has six `false` lines. A click does not reach a widget under it. |
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
