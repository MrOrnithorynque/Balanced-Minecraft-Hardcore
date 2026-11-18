# Settings area: port log

Area: settings (GAPS.md rows S1 to S4). Files: `horizon/native/mixins/bmh/bmh_settings.rs`,
`horizon/native/mixins/bmh/logic/src/settings.rs`, this log.

## Steps

1. The worktree was not on BASE 63d0592 (HEAD was 3fadae8, no `horizon/` folder). Branch
   `worktree-wf_618c86ed-4ef-1`, so the brief allows the reset: `git reset --hard 63d059248b1538f975d952422d7a632d3fd14b4a`.
   Guard file present.
2. Read ARCHITECTURE.md, HOOKS.md, GAPS.md (S1 to S4, section 3, Settled 3 and 8), audit 1 features 4 to 6,
   verify-1 rows 4 to 6, audit 4 features 6 and 7, verify-4 rows 6 and 7, audit 5 section G, verify-5 G1 to G4,
   the Java sources (`BMHGameRules`, `BMHConfigLimits`, `NeoForgeConfig`, `CreateWorldGameTabMixin`,
   `ConfigScreen`) and the respawn area (the worked example, and the two callers of `rule` and `config`).
3. Read HN (the Nightly). Findings that settle the design:
   - A level attachment (`Holder::Level`) is the attachments map of the `World` that the `Game` holds at that
     moment (mod_api_host.rs:13). A Nether level has its own map, and a hook cannot tell which level is in place
     (`server_tick` doc, native_hooks.rs:2566). So one rule would have up to three values. Reaching it also
     needs the `dyn Host` coercion and a `GameLevel` in some hooks. Decision: a file of the mod in the world
     folder, `<world>/bmh/rules.txt` (`native_world_dir`, native_server.rs:899). The doc of that helper asks
     for exactly this ("a server mod keeps its data in a folder of its own there, so the world's backups carry
     it"). Every dimension and every hook sees the same file.
   - `WorldOpenReady` (`world_lifecycle`) fires after the level the world opens in is loaded (native-mods.md
     API 134) and before the first save. The Java game rules saved data is still on disk then, so the area reads
     `data/minecraft/game_rules.dat` with `horizon_client::save::read_dat` (save.rs:3017, `pub`). It reads the
     `data` compound and looks for `bmh:<name>` and the old camelCase name, as `N_SavedDataGameRulesMixin` does.
   - The config hooks (`config_loading`, `config_reloading`) give the file text in `NativeConfigFile.text`
     (native_configs.rs). A `Common` config is one file for the window, the integrated server and a dedicated
     server, so one static holds the values. The game rewrites a missing file from the default text given to
     `NativeConfig::new`.
   - The page hooks: `native_settings` is asked again after each click (native_hooks.rs:3248-3251), so the
     labels follow the values. A label is a `String`, so the area resolves its lang keys with
     `Text::translate(key, args).plain()` (horizon-client-core/src/text.rs:239).
   - Commands: the usage strings of several areas that start with `bmh` merge into one tree, and each usage keeps
     its own level (smp registers `eco stats` at level 0 and `eco give` at level 3, glue.rs). `perform_command`
     gives the line without the slash; `native_feedback` answers the player.
   - The title's create screen is id `world_create` with the whole window as `rect` (native-mods.md API 326).
     `screen.mouse` is the pointer, so the click hook needs no position of its own.
4. Wrote `logic/src/settings.rs`: `Rules` (six values), the rules file text and parser, `Saved` and
   `import_rules` (the Java world), `initial_rules` (the order file, Java world, choice, default), the command
   parser `parse_rule_command`, the config text in the NeoForge layout and `parse_config`, the stepper
   (`step`, six buttons, the min not above max rule), and the button rectangle of the Create World screen.
   24 unit tests. Java values pinned: the six names, the defaults 5000, 60000 and 5, the ranges 5000 to 200000
   and 1 to 16, the text and byte forms of a saved rule (`BMHGameRules.java:67-69`).
5. Wrote `bmh_settings.rs`: `rule` and `config` (the contract, signatures kept), `/bmh rule`, the first open of a
   world, the Create World button, the config hooks and the page.
6. Added seven keys to `horizon/content/assets/bmh/lang/en_us.json` (below) and the section "The /bmh command
   root" to HOOKS.md.
7. Gates: `sh horizon/tools/test-logic.sh` (76 tests pass, 24 of them mine) and `check.sh` (`"check": "ok"`).

## How the area works

World rules (S1):
- The store is `<world>/bmh/rules.txt`, six lines `name = true|false`. `rule(game, Rule)` reads
  `SERVER_RULES` (a cache that follows the world folder) and falls back to the file. A world with no file
  answers `true` for every rule and does not cache that, so the file is read again at the next call.
- `/bmh rule` (level 2, the level of `/gamerule`) lists the six rules, shows one or sets one. A set writes the
  cache first, then the file (temporary file and rename), so it takes effect at once for every reader.
  The answer uses vanilla's `commands.gamerule.query` and `commands.gamerule.set` texts with the id
  `bmh:<name>`.
- The first open (`world_lifecycle`, `WorldOpenReady`) of a world that has no file creates it, with this order:
  a Java world's saved values; for a new world (game time 0) the Create World choice; every rule on. The close
  phases empty the cache.

Config (S3): `config/bmh-common.toml`, kind `Common`, in the layout NeoForge writes, with the Java comments and
ranges. `parse_config` gives a value outside its range, or one that is not an integer, the default of its key
(`ModConfigSpec` corrects such a value to the default of the key). Since fix round 1 the area also writes
the corrected file back once (see "Fix round 1"). Min above max stays in `config()`, as in Java: the respawn area uses `max(min, max)`
(`RespawnPositionHandler.java:223-226`).

Page (S4): six buttons, a "[+]" and a "[-]" for each of the min distance, the max distance and the amulet
points, with the value in the label. A click steps 5000 blocks (distances) or 1 (points) inside the Java
range. The minimum never goes above the maximum: a click that would do it changes nothing. A click writes the
whole file at once and updates the value. The reload that follows within a second gives the same values.

Create World (S2): a button 210 by 20 in the Game tab of screen `world_create`, in the row under Allow Commands
(first version: floating in the top right corner, fixed in "Fix round 1"). It is drawn in `screen_render_post`
with the vanilla button sprite, a label "Balanced Hardcore: ON/OFF" and the Java tooltip. A click flips the
choice and stores it with `set_setting` (`balanced_hardcore`). Each new Create New World screen starts ON.

## What I could and could not do for a Java world (verify-5 G4)

Could:
- Read the six values from `data/minecraft/game_rules.dat` at `WorldOpenReady`, by the new id and by the old
  name, with text values ("true", "false", any case) and byte values, once per world, and write them into
  `<world>/bmh/rules.txt`. After that the file is the truth, so Horizon dropping the keys at its first save
  changes nothing.
- A rule that the Java file does not name stays on, as in Java.

Could not:
- Read the values before Horizon loads the world. If Horizon rewrote `game_rules.dat` before `WorldOpenReady`
  (the world upgrade at the open), the BMH keys are already gone and the world starts with every rule on. See
  "To check in the game" 1.
- Keep the rules in `/gamerule`, its completion and the game rules screen (GAPS S1, decision D5).
- Read a pre-26.1 world whose rules Horizon's data fixers kept only in `level.dat`. Java's mixin handles the saved
  data file only, and so does this area.
- Tell a new world from an old world with game time 0 (a world created and never played). Both get the choice.

## Contracts

- `rule` and `config` keep their signatures and their `#[allow(dead_code)]`. Server side only.
- The client side of other areas needs one config value: `maxAmuletPoints`. The Java amulet screens show
  "Point 2 of 5" and "3 of 5 points used" (`bmh.point.screen.count`, `bmh.point.list.count`), and
  `AmuletPoints.max()` reads it on both sides (AmuletPoints.java:74). The amulet area must send
  `config.max_amulet_points` to the client in its own message (it holds the server's `config()`). No rule is
  read on a client: `remove_coordinates_in_survival` is read where the debug screen is decided
  (DisableDebugInfo.java:43), which the eternal area handles. I built no sync.
- `/bmh point` belongs to the amulet area. HOOKS.md, "The /bmh command root", says how it adds its usages and
  its branch of `on_perform_command` without touching this file.

## Differences from the Java mod

1. The six rules are not game rules: no `/gamerule bmh:...`, no completion, no game rules screen. They live in
   `<world>/bmh/rules.txt` and `/bmh rule`. GAPS S1, decision D5.
2. The Create World button is drawn by the mod in the Game tab, but it is not a widget of the tab. It is not synced
   with the More tab and not greyed out in Hardcore or Debug. It sets the value for the world made from its
   screen, as the Java button does. GAPS S2. There is also no narrator text and no click sound. More in "Fix
   round 1", "What still differs from the Java mod".
3. The config page has buttons only: a step of 5000 blocks and 1 point, no free number, no error line
   (`bmh.config.outOfRange` and `bmh.config.minAboveMax` have no place to show). The note text
   (`bmh.config.note`) has no place either. A click that would put the minimum above the maximum does nothing.
   GAPS S4, Settled 8.
4. A hand edited config value outside its range gets the key's default. The integrator's
   `Config::clamped` (contract.rs) clamps instead, and this area does not use it, because Forge's
   `defineInRange` corrects to the default. The file is written back once with the corrected values
   (fix round 1), as NeoForge does.
5. The page labels use three new short names, because the Java labels carry the range ("Minimum respawn
   distance (5000 to 200000 blocks)") and would not fit in a button.
6. The first open of a world checks "new world" with the game time. Java registers the rules at world
   creation. Allowed by GAPS S2 ("applied at the world's first open").

## To check in the game

1. A Java world with BMH rules set to false opens with the same values. Make the world in the Java mod, run
   `/gamerule bmh:random_death_spawn_point false`, open it in Horizon with the mod, type `/bmh rule`. If every
   rule reads true, Horizon rewrote `game_rules.dat` before `WorldOpenReady`: then the import needs an earlier
   hook. Look at `<world>/bmh/rules.txt` and the log line "World rules created: java world = true".
2. `world_lifecycle` fires `WorldOpenReady` on the integrated server's `Game` with `native_world_dir()` set and
   `native_level("minecraft:overworld")` answering. Look for the log line "World rules created" at the first
   open of a new world, and for `<world>/bmh/rules.txt`.
3. `/bmh rule` appears in the chat completion for an operator (level 2) and the answer text reads "Game rule
   bmh:x has been updated to: false". Check that the toast comes through `native_feedback`, and that a player
   under level 2 does not see the command.
4. `/bmh rule random_death_spawn_point false`, then die: the respawn is the vanilla one at once, with no restart
   (the rule is read from the cache). Then restart the game and the world: the value is still false.
5. The merged tree: with the amulet area ported, `/bmh point` and `/bmh rule` both complete and both run.
6. The Create World button: the screen id is `world_create`. Settled in fix round 1: the button shows in the
   Game tab under Allow Commands and not over the tab bar. The sprite names `gui/sprites/widget/button` and
   `button_highlighted` draw a button with the nine-slice call (the fill under it shows if they do not). The
   click flips ON/OFF, and a new world made while it reads OFF has `bmh/rules.txt` with six `false` lines.
7. The click on the button does not also reach a vanilla widget under it (the hook sets `canceled`).
8. Options > Mods > Balanced Minecraft Hardcore opens a page with six buttons, labelled "[+] Minimum respawn
   distance: 5000" and so on. A click changes the label and `config/bmh-common.toml`. Check the labels fit
   the button width.
9. Editing `config/bmh-common.toml` by hand changes the respawn distance at the next death within a second
   (`config_reloading`). The game writes the default file at the first start.
10. `config_loading` fires in the window's `Game` and in a dedicated server's `Game` before the first death. If
    it did not fire in the integrated server's `Game`, the shared static still holds the window's value.

## Changes outside my files

- `docs/horizon/HOOKS.md`: a new section "The /bmh command root" before the paragraph on messages. It tells the
  amulet area how to add `/bmh point` usages and run them. No row of a table changed.
- `horizon/content/assets/bmh/lang/en_us.json`: seven keys added after `bmh.config.note`:
  - `bmh.config.step`: `[%s] %s: %s`
  - `bmh.config.short.minDistanceFromBed`: `Minimum respawn distance`
  - `bmh.config.short.maxDistanceFromBed`: `Maximum respawn distance`
  - `bmh.config.short.maxAmuletPoints`: `Amulet points`
  - `bmh.rule.unknown`: `Unknown BMH rule: %s`
  - `bmh.rule.bad_value`: `Expected true or false, found: %s`
  - `bmh.rule.usage`: `Usage: /bmh rule [<rule> [true|false]]`

  The other nine lang files do not hold the `bmh.config.*` keys of the Java mod either, so I did not add the
  seven keys there. The integrator adds them with the others (HOOKS.md, "Content files").

## Fix round 1 (2026-11-17)

Three defects of `game-checks-1.md`, all in the settings area. Files: `bmh_settings.rs`, `logic/src/settings.rs`,
`tools/test-slice.sh`, `TESTING.md`, this log. The worktree was cut from `horizon-26.3` at 3ef393f.

### Steps

1. Read the game's source for what a hook tells about the open Create New World screen.
   `NativeScreenView` has `id`, `menu`, `title`, `rect`, `slots`, `mouse` and `partial_tick`. It has no tab and no
   widget list (mod_api.rs:297-335). The control tool's `ui` lists the widgets, but a mod cannot call it.
2. Found the tab anyway. The title screen keeps it in `Game.title`: `Title.mode` is `Mode::Create { options, .. }` and
   `options.tab` is 0 for Game, 1 for World, 2 for More (title.rs:272, title/create_world.rs `WorldOptions.tab`). All
   three are `pub`, and so is `Title::preparing()`. `Game.title` itself is private to `game.rs`, and the settings
   module is declared inside `game.rs`, so the module can read it. The manifest, the hooks and HOOKS.md did not change.
3. Defect 1 (the button over the tabs). `logic::button_rect(window, tab)` now gives the place of the button, or
   None when the tab is not the Game tab. It uses the arithmetic of the game's own `grid()`: the Game tab's grid
   is 210 wide and 117 high, at half of the free width and one sixth of the free height (`ceil` of the size, then
   `trunc`). Allow Commands is the last row of that grid. The new row is 8 pixels under it, 210 by 20. At GUI
   427 x 240 that is (108, 160) and at 854 x 480 it is (322, 200). Both match the bounds the game reports for
   Allow Commands (108, 132) and (322, 172), plus 28. The module draws the button and takes the click only when
   `create_tab` says the Game tab is open. It also leaves the button out while the title prepares the world
   (`Title::preparing()`), because the title takes no click then.
4. Defect 2 (the choice stays). `track_screen` runs in the render hook and in the click hook. A Create New World
   screen (`world_create`, `world_create_flat`, `world_create_buffet`) that follows any other screen is a new one: the
   choice in the window becomes ON and `true` goes to the setting file. The setting file is only the channel to the
   server, which reads it when it opens the new world. So the world made from a screen reads that screen's choice.
   The Superflat editor belongs to the same screen and keeps the choice. `create_choice` no longer reads the file.
5. Defect 3 (the config file). `logic::parse_config_checked` gives the values and whether the file held a value to
   correct: a known key whose value is no integer or is out of range. `apply_config` writes `config_text(values)`
   once when it is true. It does not write when the file already holds that text, so a second hook call with the
   old text does nothing. The corrected file asks for no second write. The log says "The config file held a value
   outside its range. It is written again with the default of that key." once.
6. Unit tests: 122 pass. Six new tests in `logic/src/settings.rs` replace the one test of the corner place: the
   button place at two GUI sizes, only in the Game tab, never over the footer, the three Create screens, the
   correction flag and the clean corrected text.
7. Test script: k1, k2 and k5 now use the real place of the button (read from the Allow Commands widget). New
   checks: k2b (no button on More and World), k5c (Cancel, then a new screen starts ON), k5d (the Superflat editor keeps
   the choice). m3 is a real check now: the file holds the defaults, keeps its three `#Range` lines, and the log has
   exactly one rewrite line. TESTING.md has the rows.
8. Looked at the game by hand (offscreen, a scratch home under the scratchpad, never the owner's `play` folder) at
   GUI width 427 and 854 for each tab, and for the OFF state, the Superflat editor and a new screen.

### Evidence

Gates: `sh horizon/tools/test-logic.sh` 122 passed, `sh horizon/tools/check.sh` `"check": "ok"`.
`sh horizon/tools/test-slice.sh --no-build --only k,l,m` (the game built from this tree with `check.sh build`):
`21 passed, 0 failed, 0 skipped`, among them k1, k2, k2b, k5, k5c, k5d and m3. The k2, k5 and m3 lines read
"leaves the button alone", "starts ON ... file 'true', first click 'false', second 'true'" and "the out of range file
is written back once, with the defaults: minDistanceFromBed=5000 maxDistanceFromBed=60000 maxAmuletPoints=5".

Screenshots, 1280 pixels wide, in `docs/horizon/log/settings-fix-1-shots/`. I looked at each one.

| File | What it shows |
|---|---|
| `game-tab-gui427-on.png` | GUI 427 x 240: "Balanced Hardcore: ON" under Allow Commands, same width, tabs free. |
| `game-tab-gui427-off.png` | The same after one click: OFF. |
| `world-tab-gui427.png` | World tab at 427: no button, every World widget in place. |
| `more-tab-gui427.png` | More tab at 427: no button, the three More buttons. |
| `game-tab-gui854-on.png` | GUI 854 x 480 (GUI scale 3): the button under Allow Commands. |
| `game-tab-gui854-off.png` | The same after a click: OFF. |
| `world-tab-gui854.png`, `more-tab-gui854.png` | World and More at 854: no button. |
| `game-tab-gui854-off-kept-after-flat-editor.png` | OFF, Superflat editor opened and closed: still OFF. |
| `game-tab-gui854-new-screen-starts-on.png` | After the OFF screen, Cancel and a new Create New World screen: ON. |

By hand at GUI 427, with the choice file read after each click: a click on the button in the Game tab flipped it
(true to false). A click at the same place on the More tab and on the World tab changed nothing.

### What still differs from the Java mod

- The Java place was reachable, but the mechanism is not the Java one. The button is drawn and clicked by the mod
  over the title's screen, not a child of the tab's grid. The title's own widgets do not move: the Horizon grid is
  117 high and the Java grid, with the extra row, would be 145 high and one row higher. Here the three vanilla
  buttons stay where Horizon puts them and the new row is under them.
- Not greyed out in Hardcore or Debug. `options.hardcore` is readable now (`Mode::Create`, `WorldOptions`), so a
  later round can grey it out. Not done here.
- Not synced with the Game Rules screen of the More tab. The screen's rules (`options.rules`) are the game's own
  rules and know no BMH rule. The button holds its own choice.
- No narrator text and no click sound.
- A window less than 240 GUI pixels high has no room for the row above the footer: the button is left out so that
  it never lies over Create New World and Cancel. The game keeps the GUI at 320 x 240 or more (setting GUI scale 8
  here left the GUI at 427 x 240, scale 6), so a normal window never reaches this.
- The module reads `Game.title`, `Mode::Create` and `options.tab`, which are not a hook API. They are the pinned
  Nightly's own types (fdc87aec). A Horizon update that renames one of them stops the build, it does not break the
  button in silence. D8 is about injections into game functions, and this is not one.
- The setting file `balanced_hardcore` keeps the last screen's choice between screens. Nothing reads it except the
  server at the first open of a new world, and each new screen writes `true` first.
- Config: only a known key with a bad value counts as a correction. A missing key, a key of another name and a
  minimum above the maximum are left alone (NeoForge adds a missing key, this area does not). The write replaces the
  whole file with the NeoForge layout, so a hand written comment is lost, as NeoForge's own correction does.

### To check in the game

1. The tooltip of the button (hover) at its new place: only the old place was seen in the first run.
2. The Buffet editor (`world_create_buffet`) keeps the choice as the Superflat editor does. Only Superflat was driven.
3. A real mouse at GUI scale 2 and 4 (only 427 and 854 were seen).
4. A dedicated server with a bad config: the first `config_loading` there writes the corrected file.


### The coordinator's check of fix round 1 (2026-11-18)

The coordinator read the fix against the game's source before the merge and found one hole.

- The hole. `track_screen` took the id of the screen that fired the hook. Data Packs (More tab) opens a screen of
  the game's own over the title: `Action::DataPacks` sets `Screen::Options` (`game.rs`, pinned Nightly), and
  `fire_title_screen_render_post` says "Options draws and fires its own". So that screen fired the hooks with its
  own id, `CREATE_OPEN` went to false, and the way back to Create New World counted as a new screen: the choice
  went back to ON with no sign to the player.
- Shown in the game first. New check `k5e` of `test-slice.sh`: OFF, More, Data Packs, Done, Game. Before the
  change it failed: "the choice changed behind the Data Packs screen: 'false' before, 'true' after".
- The change. `track_screen` reads the title's own state (`game.title`, `Mode::id()`), which still holds the
  Create New World screen while the Data Packs screen is over it. One function of `bmh_settings.rs`, no change in
  the logic crate.
- Game Rules (More tab) is a list inside the title's own screen and never had the fault: new check `k5f`.
- Experiments (More tab) was not driven. It is a list inside the title too (`self.experiments`, same shape as the
  Game Rules list in `title.rs`), by reading only.
- Not checked against Java: Re-Create of a world. Vanilla copies the old world's game rules into the new screen,
  so the Java button most likely shows the old world's value there. Here the screen starts ON. The parity review
  of this area must look at it.
