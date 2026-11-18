# Port log: amulet

Area: the Navigator's Amulet (GAPS.md rows A1 to A6). Briefs: `docs/horizon/briefs/port.md` and
`docs/horizon/briefs/area-amulet.md`. No game rights: the proof is the two gates only.

## Steps

1. Worktree set up. Branch `worktree-agent-afbf1a660f30c5b1a`. The base check failed at first, so the worktree was
   reset once to `72f0fc8aded3e4ca92139e16f8024820c010995d`. The guard file `horizon-mixin.toml` exists. Next: read
   ARCHITECTURE.md, HOOKS.md, GAPS.md, the audits, then the Java sources.
2. Read the briefs, ARCHITECTURE.md, HOOKS.md, GAPS.md (A1 to A6), audit 4 (sections 4, 5a, 5b, 5c), verify-4,
   verify-1 row 6, audit 6, and the Java sources of the area. Read the existing modules `bmh_settings.rs`,
   `bmh_respawn.rs`, `bmh_journey.rs` for helper patterns that already compile.
3. Settled by reading HN (Nightly fdc87ae, path HN in port.md):
   - Needle (A1). `horizon-world/src/item_defaults.rs:77` `native_modify_default_components` changes the defaults
     table that every reader uses; `horizon-client/src/item_render.rs:753` builds the item model prototypes from
     `item_defaults::shared`. `horizon-assets/src/item_model.rs:1541-1620` `compass()` reads
     `minecraft:lodestone_tracker.target` (dimension, pos array) for any item whose model uses the
     `minecraft:compass` property: no test of the item id. The refusal of the content loader
     (`mod_content.rs:830`) is for content files only, not for this hook. `maps.rs:1238`
     `use_lodestone_compass` acts only on `minecraft:compass`, so a click on a lodestone never retargets the amulet.
     `maps.rs:1284` `tick_lodestone_compasses` clears only a tracker with `tracked` absent or not 0, and only one
     in the stack's own patch: the default with `tracked: false` stays.
   - Clicks (A3). Server: `game.rs:6963-7210` `use_hand`: sneak with a full hand skips the block's own use, then
     `native_use_item_on`, then `native_use_item` when not claimed and the packet is not UseItemOn. An item on a
     use cooldown is never asked. Client: `remote.rs:9250` `remote_use_hands` asks `native_interaction(1, Off)`
     (the hook `interaction_key`) before the off hand: a cancel there skips the off hand.
   - Sneak in a use hook: `native_sneaking()` (`native_host_bridge.rs:118`, fragment host_bridge, tier F) reads the
     acting guest's sneak key, the same key `use_hand` reads. `NativeServerPlayer.sneaking`, `eye_height`,
     `selected_slot` (fragment server_player_state, tier F) are filled by `native_acting_player`.
   - Sky rules (A2). Rain: `native_weather()` and Horizon's own `Level.isRaining` threshold, rain above 0.2
     (`native_level_world.rs:10`). Night: Horizon has `Game::sky_darken` (private, `game.rs:11678`) from the
     26.3 timeline data; no helper exposes it and the data is not on this machine. The mod computes vanilla's
     `updateSkyBrightness` from `native_day_time()` and the weather (the formula of 1.21.1); the bed rule of
     Horizon uses the same test, `sky_darken() >= 4` (`game.rs:15026`). Open sky: Horizon's own `can_see_sky`
     is sky light 15 (`mobs/spawn_rules.rs:108`): `native_light(eye).1 >= 15`. Effects: `native_effects(uuid)`
     (fragment player_effects, tier F).
   - Charges. They stay on the stack, in `minecraft:custom_data` (`bmh_starlight`, `bmh_night`), the Java
     layout: the stack is saved with the world and synced to the client, so the bar and the tooltip read it.
     `native_carried_stack(slot)` changes a carried stack in place inside a player hook.
   - Points (A5). `AttachmentType` on `Holder::Player(uuid)` with a codec and `copy_on_death`
     (`horizon-world/src/attachments.rs`, docs native-mods.md "Server attachments"). In a player hook the
     acting player's set is swapped in; in `net_from_client` the online guest's set is read.
   - Sounds and awards. `native_run_command` refuses inside a player hook (`native_server.rs:732`), and
     `native_play_sound_to` reaches only the acting player there. So the chime for the others and the
     advancements go to a queue, flushed in `level_tick_post` (a change outside my files: the amulet is added
     to `hook_level_tick_post.rs`).
   - Screens (A6). `native_mod_screen_open(id)` takes any id (`native_hooks.rs:12375`); the amulet uses its
     own id `bmh:amulet`, so it never mixes with the journey's hold screen (id `bmh`). The screen hooks get
     `&Game`, so sending a message and closing the screen wait for the `frame` hook (a change outside my files:
     the amulet is added to `hook_frame.rs`). A mod screen's `rect` is the whole GUI window
     (`native_hooks.rs:4049`). Key names are vanilla's (`key.keyboard.enter`, `horizon-client-input/src/keybinds.rs`).
   - The command root. `native_server.rs:131` `command_tree` gives the whole `/bmh` root the LOWEST level of its
     usages, and `perform_native_command` (`native_server.rs:1475`) checks only that root level. HOOKS.md says
     "each usage keeps its own operator level": the code says no. A `bmh point` usage at level 0 would let every
     player run `/bmh rule <name> <value>` (the settings area does not test `player.permission`). Careful choice:
     `/bmh point` is registered at level 2, the level of `/bmh rule`. The screens do not need the command (they
     use `bmh:amulet` messages), so a player loses only the typed command. Open question for the integrator.
4. Wrote `bmh_amulet_world.rs` (A1: the default `lodestone_tracker` of the amulet).
5. Wrote `logic/src/amulet.rs` (constants with their Java lines, sky rules, charges, reading, cooldowns, use
   order, points, name cleaning, change checks, the command line, the chime command, the two screen layouts,
   the text field) with 30 unit tests. The vanilla night formula reproduces the bed window of vanilla in clear
   weather (12542 to 23459), which the test pins. Added `contract::AmuletReading` (the reading contract for the
   sky area) and the amulet section of `wire.rs` (`AmuletSignal`, `AmuletRequest`, channel `bmh:amulet`) with
   round-trip and cut-input tests. `test-logic.sh`: 151 passed. Next: the module `bmh_amulet.rs`.
6. Wrote the module `bmh_amulet.rs`: every `on_<hook>` of the skeleton has its body, plus `on_frame` and
   `on_level_tick_post`, the contract `points(game, player)` and the contract `reading(stack)`. Wired the amulet
   into `hook_frame.rs` and `hook_level_tick_post.rs` (one line each) and updated their HOOKS.md rows; added the
   "Star reading of an amulet" row to HOOKS.md "Contracts".
7. First `check.sh` (native.py runs `cargo check -p horizon-client --bin horizon-client` with `-D warnings`, so
   the game binary is type-checked): `"check": "ok"`, no warning. The staged `game.rs` holds `mod bmh_amulet`.
8. Review pass against the Java sources (order of the checks, keys, numbers, sounds), then this log.
9. Final gates: `test-logic.sh` 151 passed, 0 failed; `check.sh` `"check": "ok"`. No em-dash or en-dash in the
   files I wrote or changed. Status: every row built (A2 and A4 as their substitutes). Nothing is proven in a
   game: the list "To check in the game" is open. Nothing committed (the brief forbids git add and commit).
10. Fix round 1 (reviews `docs/horizon/review/amulet-semantics.json` and `amulet-parity.json`):
    - Import of the Java points (semantics 1, parity 2): fixed. The old code read the points in a player's own turn,
      where `native_player_compound` gives None (native_player_state.rs:127-133), fell back to a file on disk and
      saved the result, even an empty one. Now `load_points` never imports. The import is `import_java_points`,
      called at the start of `on_level_tick_post`, where no player is swapped in and `native_player_compound` gives
      the record that Horizon loaded (`lan` is put back before the level passes: lan.rs:2676; levels.rs:560 calls
      the hook). There is no disk fallback: it could read a file that is not the one Horizon loaded (for the
      single-player owner level.dat names the file: playerdata.rs:85, save.rs:4332). The attachment now holds the
      points and a flag `imported` (saved with them, copied across death). Paths:
      - Owner, first use in an old Java world: the join is followed by a level pass in the same or the next tick,
        long before a click. `level_tick_post` reads the record (`bmh_points`, else `bmh_home`), saves
        `{points, imported: true}`. If the click came first (not expected), the player's change is saved with
        `imported: false`, and the import later puts the Java points first and the made points after them, once
        (`logic::merge_imported`).
      - Guest: the same path. The record is the guest's `players/data/<uuid>.dat` as loaded.
      - New world, no Java data: the first pass saves `{[], imported: true}`. The next passes read the flag and
        stop. Cost before the flag is saved: one attachment read for each player and each level pass.
      - Player not placed yet, or out of reach: `native_player_compound` is None or the attachment read fails.
        Nothing is saved. The next pass tries again.
      - Restart between the read and the save: nothing was saved, so the next join tries again. Restart after a
        first change with `imported: false`: the flag is on disk, so the import still runs and merges.
      - A save from before this change has no flag and reads as `imported: false`; the merge never doubles a point.
    - Reading contract (semantics 2): fixed in the text. The route named before does not exist. The verified routes
      are in the section "Reading contract for the sky area" and in the HOOKS.md contract row. No client code was
      written.
    - Night test (parity 1): listed difference, formula not changed. Two tests pin the ticks where our answer
      changes: clear weather 12542 and 23460, rain 0.2 (the highest level that is not "raining") 12465 and 23536,
      and the thunderstorm that is dark all day. No test can compare them with the host value: the 26.3 timeline
      data is not on this machine. The check stays in "To check in the game" (item 4).
    - `/bmh point` level (parity 3): not fixed, for the integrator. Level 2 stays (decision of the coordinator:
      native_server.rs:179-180 and :1488-1491). The text for GAPS.md A5 is in difference 7.

## How it is built

Server (the integrated server for the single-player owner, audit 6 V5; every hook below fires for the owner):

- `use_item_on` (player swapped in): `logic::use_on` in Java's order (cooldown, sneak, bed or lodestone,
  Overworld). Open: the `AmuletSignal::Open` message, a 20 tick cooldown, `took_click` noted, `claimed`.
  Another dimension: the action bar `bmh.point.overworld`, `took_click` noted, `claimed`.
- `use_item` (player swapped in): `logic::use_in_air` in Java's order (cooldown or same click, sneak: list,
  sky rules, charges). The stack is found in the main hand, then the off hand (`native_carried_stack`), and
  changed in place: the refill, the reading compound and the charges, with the Java keys. The chime goes to the
  player at once (`native_play_sound_to`) and to the others through a queued `/playsound`. The award
  `navigation/stargazer` is queued. The action bar shows `bmh.amulet.reading` with the block position.
- `inventory_tick`: once a second (game time multiple of 20) the refill and the drop of the 1.4.0 data.
- `perform_command` / `register_commands`: `/bmh point save|remove|delete` at level 2 (see step 3, command root).
- `net_from_client` (no player swapped in): `AmuletRequest` from the screens; the player is read from
  `native_server_players` and the hands through `native_entity`, `native_living`, `native_living_hand_item`.
  Both paths call one `change()` with the checks of `AmuletPointCommand` in Java's order.
- `level_tick_post`: brings the Java points to the players who are on (step 10), runs the queued
  `advancement grant` lines (any level) and the chime lines of that level.
- `spawn_change`: keeps the world spawn of this world folder; else the spawn is read from `level.dat`.
- `player_logged_out`: clears the per-player maps.

Client (the window's game):

- `modify_default_components` (both sides): the default `minecraft:lodestone_tracker` of the amulet.
- `tooltip`, `item_bar`: read the charges from the stack's custom data.
- `interaction_key`: cancels the off hand try (and its swing) when the main hand holds the amulet.
- `net_from_server`: opens the screen `bmh:amulet` with the naming state or the list state.
- `screen_render_post`, `screen_mouse_button_pressed_pre`, `screen_key_pressed_pre`, `screen_char_typed_pre`:
  draw and edit the two screens; every key but Escape stops at the screen.
- `frame`: sends the requests of the screens on `bmh:amulet`, closes the screen when asked, and forgets a
  screen that the game closed.

## Reading contract for the sky area

`bmh_logic::contract::AmuletReading { azimuths: Vec<f32>, names: Vec<String>, spawn: Option<usize>, time: i64 }`,
read with `bmh_amulet::reading(stack)` from an amulet in either hand of the window's player. What the sky needs,
from `client/sky/`: `StarReadingRenderer.startFreshReading` starts a reading when `time` is newer than the last
one shown and within 40 ticks of the level's game time (`native_clock().game_time` on the client), one chain per
azimuth with `StarChain.pick(stars, azimuth, timeOfDay)`, the name of each chain for the spyglass (`aimedName`,
names that are not empty), and the spawn index for the lighter line. With an amulet in each hand, the newest
reading wins.

Routes to the two hand stacks on the window's game (verified in HN; the first draft of this log named a route that
does not exist):

- Off hand: `native_carried_stack(-106)` (native_hooks.rs:10399; the slot numbers are at :11568-11572, -106 is
  `self.offhand`). It takes `&mut self`, so it works in any hook that gets `&mut Game` (`frame`, `net_from_server`).
  Its doc comment speaks of the acting player; on a client `self.offhand` is the window player's own off hand (the
  first-person renderer shows it from the same field, hand.rs:558).
- Main hand: `native_render_hand` (native_hooks.rs:3031, called from game.rs:14405 with `Hand::Main` and the
  displayed stack, `&self`). It fires only when the first-person hands are drawn, once for each visible hand. A mod
  that needs the stack elsewhere keeps what this hook saw. No helper gives a client the held slot number, so
  `native_carried_stack(selected)` cannot be called (`native_held`, native_server.rs:1035, gives item id, count and a
  plain flag, no components).
- `native_living_hand_item` (native_living_refs.rs:107) needs a `LivingRef` of the window's own entity. The entity id
  is `own_entity_id` (perspective.rs:1180), which is private. No route there.
- Without a first-person hand (third person, F1) the main hand has no route. Proposal if the sky area needs it: the
  server sends the reading on `bmh:amulet` when it writes it (a new `AmuletSignal`), and the sky area keeps the
  newest one. Not built.

## Differences from the Java mod

1. A1, needle: a fixed target far north of the origin (0, 64, -30 000 000) instead of 1.0E6 north of the holder.
   Up to 1.9 degrees off at x = 1 000 000 (GAPS.md A1, listed).
2. A4, cooldown: the mod keeps its own timer per player (20 or 40 game ticks). No white sweep on the icon (GAPS.md
   A4, listed). Java's client also skips the use of an item that cools down and then tries the off hand; here the
   off hand is never tried when the main hand holds the amulet (A3 filter), so an off hand item does not act then.
3. A2, world spawn line: from `spawn_change` or `level.dat` (GAPS.md A2, the listed substitute). With neither, the
   reading has no spawn line (Java always has one). Reason: no helper reads the world spawn.
4. A2, night: vanilla's `updateSkyBrightness` formula of 1.21.1 from `native_day_time` and `native_weather`.
   Horizon computes the darkening from the 26.3 timeline data, which no helper exposes. The formula gives the
   vanilla bed window (12542 to 23459 in clear weather) and Horizon's own test cases; the two can differ by a few
   ticks at dusk and dawn, more in rain. GAPS.md A2 and verify-4 5a name this route ("the mod computes from
   day_time, weather").
5. A2, eyes in a fluid: the block a little under the eyes is water, lava, a bubble column, kelp, seagrass or a
   waterlogged block; vanilla also compares the fluid height. Powder snow is the block at the eyes, as Java.
   Reason: no fluid height helper. GAPS.md A2 lets the sky rules be the mod's.
6. A3, double use: the use of the same click is dropped when it comes in the same tick as the taken use on a
   block, or the tick after (GAPS.md A3: "drop a use_item in the same tick").
7. (Replaced in settings fix round 2: `/bmh point` is level 0 again, as in Java, and `/bmh rule` refuses a player
   below level 2 inside the settings area. The text below is the old state.) A5, `/bmh point` is for operators of level 2; Java's is for every player. Reason: Horizon checks one level for
   the whole `/bmh` root, the lowest of its usages, so a level 0 usage would open `/bmh rule` to everyone (step 3).
   The screens use mod messages and need no command. Text for GAPS.md row A5: "`/bmh point` needs operator level
   2, not level 0. Horizon gives the whole `/bmh` root the lowest level of its usages and checks only that level
   (native_server.rs:179-180, :1488-1491), so a level 0 usage would open `/bmh rule` to every player. The screens
   do not use the command." Typed coordinates: `~` and `~n` work, `^` does not (GAPS.md
   A5 allows it). A bad or short line answers vanilla's `command.unknown.argument` or `command.unknown.command`.
8. A5, the last change time of a player is forgotten at logout (ARCHITECTURE.md, State); Java keeps it for the
   server's run. Effect: a relog within one second allows one more change.
9. A6, screens: drawn by the mod (GAPS.md A6): no selection, no clipboard, no Tab focus, no narrator, no blur
   behind the dim layer. A gamepad cannot click them. The screen opens only over the world with no other screen
   (`native_mod_screen_open`); Java's `setScreen` replaced any screen.
10. A6, names: counted in UTF-16 units as Java does, but a character outside the BMP is never cut in two (Java can
    leave half of it).
11. Points of a Java world: brought in at `level_tick_post` from the record that Horizon loaded (`bmh_points`, else
    `bmh_home`), once for each player (step 10). A player who acts in the very first turn, before the first level
    pass, sees no Java points until the pass; they are merged then, not lost.
12. The attachment holds one more key, `imported`, next to the Java list `bmh_points`. Only Horizon reads it.

## To check in the game

1. The needle points north in the Overworld and spins in the Nether (A1). Hold the amulet, turn round: the needle
   frame follows north. `horizon-ctl` screenshot facing north and facing east.
2. A star reading works for the single-player owner (A2): at night (`/time set 18000`, `/weather clear`), outside,
   use the amulet. Expect the action bar `X .. · Y .. · Z ..`, the chime, the log line `Star reading for <name>`,
   the bar at 9 of 13 pixels, the tooltip `Starlight 2/3`, and the advancement "Reading the sky".
3. The game time and the sky reads work inside `use_item` (`native_level` and `native_level_game_time` in a
   player's turn): without them the amulet does nothing at all. The log line of check 2 proves it.
4. The sky refusals in order: in the Nether (`.dimension`), `/weather rain` (`.rain`), `/time set 6000` (`.day`),
   under a block (`.roof`), with `/effect give @s blindness` (`.roof`), eyes in water (`.roof`). Compare the dusk
   tick where `.day` stops with the tick where a bed lets the player sleep (both are `isDarkOutside`).
5. Sneak and use on a bed (head and foot) and on a lodestone: the naming screen opens once per click, the bed does
   not put the player to sleep, the list screen does not open in the same click (A3 double use), and an off hand
   shield, bucket or ender pearl does not act.
6. The naming screen: type a name (letters do not open the inventory), Backspace, arrows, Enter saves; Save is grey
   with an empty name or a full cap; Remove and Cancel close it; Escape closes it. Then the action bar says "Point
   saved", the chime plays at pitch 0.7, the advancement "A name for a place" shows, and the third point gives "A
   sky of your own".
7. The list screen: sneak and use in the air; pages of 5, `<` and `>`, Remove updates the list in place, Done
   closes it.
8. Points survive death and a restart (attachment with `copy_on_death`), for the owner and for a LAN guest; the
   homecoming area reads them (`bmh_amulet::points`).
9. The charges come back at nightfall: spend three readings, `/time add 24000`, wait one second: the bar is full.
10. The server checks a request: a player who walks more than 8 blocks away before Save gets `bmh.point.too_far`;
    with no amulet in hand, `bmh.point.no_amulet`; two saves within one second, the second does nothing.
11. Another player within 16 blocks hears the chime of a reading and of a save (`/playsound` from
    `level_tick_post`); the reader hears it once, not twice.
12. `advancement grant` from `level_tick_post` shows the toast (GAPS.md question 8).
13. `native_net_send_to_player` from `use_item_on` reaches the owner's window (the screen opens), and
    `native_net_send_to_server` from `frame` reaches `net_from_client` (the save lands).
14. `/bmh point save ~ ~ ~ Home` as any player, standing on a bed: it saves the bed's point (level 0 since settings
    fix round 2). A player under level 2 cannot run `/bmh rule`: he gets "Unknown or incomplete command".
15. The tooltip shows the four lines after the name, the first in amethyst, the others grey; the item bar is
    amethyst and always shown.
16. A Java world with BMH points: the points show in the list after the first open in Horizon. To make such a
    world without the Java mod: stop the game, take a world folder, and add to the player's compound (the file that
    level.dat names in `singleplayer_uuid` for the owner, `players/data/<uuid>.dat` for a guest) a list
    `bmh_points` of compounds `{name: "Camp", x: 10, y: 64, z: 20}` (or a compound `bmh_home` `{x, y, z}`), with an
    NBT editor. Open the world, wait a second, sneak and use in the air: the list shows the points. The log has
    `amulet points read from the Java player data`. Then save a new point, leave and join again: the Java points
    and the new one are there, none doubled. Also open a new world (no `bmh_points`): the log line is absent and
    the list is empty.
    Also check that the owner's record, in `native_player_compound` from `level_tick_post`, holds the unknown keys
    (`player_extra`): with a world where it does not, the import finds nothing and the list stays empty.
17. Mod screen look: whether Horizon draws its own background behind a mod screen, and whether the button and
    text field sprites resolve (`gui/sprites/widget/button`, `.../text_field_highlighted`).

## Changes outside my files

1. `horizon/native/mixins/bmh/hook_frame.rs`: one line, `crate::game::bmh_amulet::on_frame(game);` after the
   journey. Reason: the screen hooks give `&Game`; sending a request and closing the screen need `&mut Game`.
2. `horizon/native/mixins/bmh/hook_level_tick_post.rs`: one line, `crate::game::bmh_amulet::on_level_tick_post(game,
   dimension);` after respawn. Reason: `run_command` (awards, chimes for others) refuses inside a player's hook.
3. `horizon/native/mixins/bmh/logic/src/contract.rs`: the type `AmuletReading` (the brief asks for it).
4. `horizon/native/mixins/bmh/logic/src/wire.rs`: the amulet section (`AMULET_CHANNEL`, `MAX_LIST_POINTS`,
   `AmuletSignal`, `AmuletRequest`, two private position helpers) and two tests (the brief asks for it).
5. `docs/horizon/HOOKS.md`: the `frame` and `level_tick_post` rows name the amulet; a "Contracts" row for the reading.
6. No lang key was added: every key the area shows is in `en_us.json`. No content file was changed.

## Open questions

1. The `/bmh` root level (step 3). Fix proposal for the integrator: the settings area tests
   `player.permission >= COMMAND_LEVEL` in `on_perform_command`, then `bmh_logic::amulet::COMMAND_LEVEL` can be 0
   (Java parity). HOOKS.md "The /bmh command root" says each usage keeps its level: the code (`command_tree`,
   `perform_native_command`) says the root's lowest level decides.
2. The night formula (difference 4): a helper that exposes `Game::sky_darken` would give exact parity.
