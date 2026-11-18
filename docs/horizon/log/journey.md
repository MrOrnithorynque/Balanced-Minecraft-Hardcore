# Journey area: port log

Area: journey (GAPS.md rows C15, C16, C17, C8 on the client, C4 on the client). Files:
`horizon/native/mixins/bmh/bmh_journey.rs`, `horizon/native/mixins/bmh/logic/src/journey.rs`, this log, and the
content files `horizon/content/assets/bmh/sounds.json` and `horizon/content/assets/bmh/sounds/`.

## Steps

1. The worktree was not on BASE 63d0592 (`git merge-base --is-ancestor` failed). Branch
   `worktree-wf_618c86ed-4ef-2`, so the brief allows the reset: `git reset --hard
   63d059248b1538f975d952422d7a632d3fd14b4a`. Guard file `horizon/native/mixins/bmh/horizon-mixin.toml` present.
2. Read ARCHITECTURE.md, HOOKS.md, GAPS.md (C4, C8, C15, C16, C17, section 3, section 7 questions 2 and 16),
   audit 4 feature 1, verify-4 rows 1a to 1e, the respawn log and `bmh_respawn.rs` (when each signal leaves).
   Java: JourneyOverlay (the 26.1 version: it draws its own sky and uses no class of `client/sky/`, so
   `stars.rs` needs nothing from this area), JourneyCinematic, JourneyMusic, F_GuiTitleMixin,
   F_LocalPlayerRespawnMixin, F_MinecraftOverlayMixin, F_MusicManagerAccessor, F_SoundInstanceVolumeAccessor,
   JourneySignal, SoundInit.
3. Read HN (the Nightly). Findings:
   - Drawing: `gui_render_post` is `&self` (native_hooks.rs:2633, called at game.rs:15960 after `hud()`).
     `native_fill` keeps alpha; `native_text` forces it opaque, so the message goes through `native_gui_paint`
     with `GuiDraw::Push/Text/Pop` (hud_layout.rs:63-130). `HudBatch` has `scale`, `width`, `height` (hud.rs:1567).
     No gradient helper among the N/F helpers (`HudBatch::fill_gradient` needs the private atlas).
   - The screen hooks and `music` are `&self` too. Sending (`native_send_packet`, `native_net_send_to_server`),
     `native_play_local_sound` and `native_mod_screen_open/close` need `&mut Game`. So the timeline runs in the
     stamped `frame` hook (`native_frame(&mut self, elapsed)`, native_hooks.rs:2298, catalog `frame`, called in
     `extras_frame`, extras.rs:64, once a frame after the frame's ticks and before its drawing). I wire it.
   - Client state: `native_client_state()` (native_client_services.rs:239) gives `screen` (Death for
     `Screen::Dead`, Other for a loading screen, a mod screen and any other screen), `world` and `gui_size`
     (`gui_size` is `HudBatch::new(renderer size)`, the same size the death screen click uses).
   - The respawn on the client: `C::Respawn` (remote.rs:3652) calls `enter_remote_dimension`, which sets
     `self.screen = Screen::Playing` and starts the level load. So "the death screen closed after the request"
     is Java's "the player object changed". A death respawn loads with `Reason::Other`, so `remote_loading()`
     is true and `native_mod_screen_open` refuses until the load ends (native_hooks.rs:10973-10995).
   - The death screen (screens.rs:691-842): left click only; Respawn (focus 0) respawns, Title Screen opens a
     confirmation whose "no" (and Escape) respawns. Enter, keypad Enter and Space act on the focus, Tab and
     the arrows move it. Horizon's remote death sets no `died_at` (remote.rs:4810-4840), so its 1 s delay may
     not apply. The confirmation layout is `Choice::new` (confirm_screen.rs:120-150).
   - `screen_mouse_button_pressed_pre` fires before the death click, not for a gamepad click
     (game.rs, `pad_synthetic_input`). `screen_key_pressed_pre` fires after the window shortcuts (F2, F11) and
     gets vanilla key names (`key.keyboard.enter`).
   - Music: `native_music` runs once a client tick (music.rs:20). Horizon's `MusicManager::fade` eases the
     gain toward the volume by 3 % a tick downward (music.rs:683-709), so a volume of 0 takes about 15 s. A
     `Music` with `replace_current_music` stops the current track at once (music.rs:527-539). None keeps quiet.
   - `native_play_local_sound(sound, at, volume, pitch, delay)` has no sound category (native_hooks.rs:10738).
4. Wrote `logic/src/journey.rs`: every constant with its Java line, `JavaRandom` (a port of
   `java.util.Random`), the sky, the chain, the drawing as fills, the timeline (`Journey::update`), the music
   actions, the message layout, a model of Horizon's death screen input (`DeathScreen`) and the client state
   machine (`Client`). To pin the Java values I copied the drawing code of JourneyOverlay into a plain Java
   program (scratchpad `journey-ref/JourneyRef.java`, Minecraft types taken out) and ran it on OpenJDK 21: the
   random values, the chain of seed 12345, three whole frames (count and hash of every fill). The Rust port
   gives the same fills, bit for bit, for all three frames. 30 journey tests, 82 in the crate, all pass.
   The program and its output are kept in `docs/horizon/log/journey-ref/` (first written to the
   scratchpad, then moved into the repo, so the evidence stays with the tests).
5. Wrote `bmh_journey.rs` (flow at the top of the file), `hook_frame.rs` and its `[[injection]]`, and the
   HOOKS.md row. First compile of the bundle: ok. The staged `native_hooks.rs` holds `bmh_hook_frame`
   (checked in the stage folder).
6. No change to `sounds.json` or `sounds/`: the Java file is the same, and the area plays `bmh:end` and
   `bmh:bell_sfx`, which it defines. The other sounds are vanilla events. No lang key added:
   `bmh.journey.message` is in `en_us.json`, and `deathScreen.quit.confirm` is vanilla's.
7. Both gates pass (see the report).

## How the rows are built

- C8 (client): `net_from_server` decodes `JourneySignal` on `bmh:journey`. As in Java
  (JourneyCinematic.java:30-59), `Departure` arms the next Respawn click, it does not start the fade by
  itself. `Ready` lets the held respawn go. `Arrival { name }` gives the message to the running journey, or
  starts one in black when none runs.
- C4 (client, substitute): `DeathScreen` is a model of Horizon's death screen input. On the `dead` screen,
  with a journey armed, a press that would respawn (the Respawn button, the "no" of the quit confirmation,
  Escape in the confirmation, Enter or Space on the Respawn focus) is cancelled and starts the journey. The
  next frame sends `RespawnRequest::Held`. The respawn request (`ClientCommand { action: 0 }` through
  `native_send_packet`) leaves when the screen is black (600 ms) and the server said `Ready`. While a
  journey runs, every press of the death screen and of the hold screen is cancelled (the Java overlay took
  all input).
- C15: `gui_render_post` draws the picture of the last frame: the fade fill in GUI pixels, then the sky,
  the glow, the stars and the chain as fills in art pixels scaled by `pixel / batch.scale`, then the message
  through `native_gui_paint` (Push, Text with alpha, Pop).
- C16: the timeline runs from the frame clock whatever screen is up. After the respawn, `frame` asks
  `native_mod_screen_open("bmh")` each frame until the dissolve; it opens once the load is over. Every key
  and click on that screen is cancelled while the journey runs. The screen closes at the dissolve, at the
  end, and when the world is left.
- C17: `music` scales the volume by `1 - ease(t / 600)` during the fade, asks once for a track that replaces
  the current one at the black (the Java `stopPlaying`), then keeps quiet until the journey ends. The
  chimes, the resonance, the break, `bmh:end` and `bmh:bell_sfx` play through `native_play_local_sound` at
  the player's last known place, with the Java volumes and pitches.
- The journey always ends: a new or no world ends it at once (Java: level or player null). The Java
  timeouts stay: 30 s after the respawn (`GIVE_UP`), 150 s after the start (`SERVER_WAIT`).

## Differences from the Java mod

1. The respawn request waits for the server's `Ready`, not only for the black (GAPS.md C4, substitute).
   Java sent it at the black and the server held it.
2. The hold covers the mouse and the keyboard of the death screen only. A gamepad click, the
   `immediate_respawn` rule and `horizon-ctl`'s respawn go without it (GAPS.md C4): the cinematic then
   starts in black, with no fade and no music fade, as the Java "joined dead" path. After a death screen,
   it starts in the first frame that shows the world (the respawn's loading screen, which the game reports
   as `Other` like a prompt, can show before it). With `immediate_respawn` it starts when `Departure` comes,
   which can be after the respawn: the vanilla point can show for a few frames. Java faded from the dead
   player's view and the server held the respawn. Fix round 1: the exile itself now happens on these paths
   (`bmh_respawn.rs`, `finish_early`; respawn.md difference 3).
3. The hold finds the Respawn press through a model of Horizon's death screen (button places, focus, quit
   confirmation), because no hook fires on the respawn itself (GAPS.md C4). A gamepad that moves the focus
   (pad_menu.rs) is not seen, so a key press after it may be read wrong.
4. The death screen buttons count as inactive for 1 s from the frame that first sees the death screen, and
   a press on them in that second is cancelled (vanilla's 20 ticks; Horizon's own delay does not run on a
   remote death because `died_at` is not set there, remote.rs:4810-4840). Fix round 1: also with no journey
   armed, so a fast press waits for `Departure`. So with the mod on, every remote death screen has vanilla's
   delay, also a death with no distant respawn.
5. The mod sends the respawn packet itself, so Horizon's `respawn_player` does not run: its focus and
   death text resets and `toggles.respawn()` (the toggle keys) are skipped (audit 4, "negligible").
6. A journey that gives up before `Ready` (150 s) sends the respawn request at the dissolve, so the player is
   not left dead under nothing. Java had sent it at the black. Needed by the always-end rule.
7. The night gradient is one fill per run of art rows with the same colour, each row with the colour at its
   middle (GAPS.md C15). At most 64 bands; the colour steps are the 8-bit steps a smooth gradient also has.
8. The music: Horizon's music manager eases its gain toward the volume by 3 % a tick downward, so the
   600 ms fade only lowers the track a little before the cut at the black. Java set the track's volume each
   frame. The cut uses a track id that no resource defines (`bmh:journey_music_cut`, with 24000-tick
   delays); after the cut the music stays quiet until the journey ends, then the game's choice returns
   (GAPS.md C17 allows a 20 Hz fade; this is a further, audible difference).
9. The sounds have no category: the Players slider does not scale them (GAPS.md C17). They play at the last
   place where the client had a player, which during the respawn load is the death place.
10. A resource reload does not end the journey: Horizon has no overlay slot that a reload takes, so the
    journey goes on under the reload screen and ends by itself. Java ended it and sent the respawn at once
    (F_MinecraftOverlayMixin), because nothing could send it later there.
11. The sky's random source is a port of `java.util.Random`; the seed comes from the wall clock and a
    counter, as Java's `new Random()` does. `nextGaussian` uses Rust's `ln` and `sqrt`, which can differ
    from `StrictMath` in the last bit.
12. The twinkle and the pulse of Home read the area's own clock, not the absolute `Util.getMillis()`: the
    same speeds, another phase.
13. A client without the mod sees nothing (GAPS.md C8).
14. Java ties the hold to the dead `LocalPlayer`. Horizon gives a mod no player object and no client health,
    so the arm ends at the first frame that shows the world (screen kind Playing or Chat). A frame of kind
    `Other` (a prompt, book, sign or pack reload over the death screen, and also the respawn's loading
    screen) changes nothing: the game reports them all as `Other` (native_client_services.rs:75-84).
15. The terrain test counts every `Other` screen after the respawn as the loading screen, except the mod's
    hold screen (parity minor 2, not fixed): Horizon has no screen kind or helper for the level loading
    screen alone (`NativeScreenKind`, native_client_services.rs:60-71; `remote_loading` has no mod helper). A
    pause menu or a server dialog in the frames before the hold screen opens keeps the reveal back, up to
    GIVE_UP. The hold screen opens in the first frame after the load, so the gap is small.

## To check in the game

1. `gui_render_post` draws over the death screen and over the respawn loading screen on every frame between
   the click and the reveal (GAPS 7, question 2). Check: with the distant respawn rule on (`/bmh rule`), die and
   click Respawn: the screen fades to black and never shows the loading screen or the world before the
   dissolve.
2. `native_client_state().screen` is `Death` while the death screen is up, and leaves `Death` when the
   server's respawn packet comes. Check: the log line "Landing spot ready" comes before the world shows; a
   held click never respawns before it.
3. `screen_mouse_button_pressed_pre` and `screen_key_pressed_pre` fire on the death screen with
   `screen.id == "dead"` and the cursor in GUI pixels, and `gui_size` is the size the death click uses.
   Check: click Respawn at its edges, use Tab then Enter, Title Screen then No, Title Screen then Escape:
   each one starts the fade. Title Screen then Yes leaves the world.
4. The owner of a single-player world hears `Departure`, `Ready` and `Arrival` in the window's
   `net_from_server` (verify-4 1b). Check: the fade starts on a click in single player and on a server.
5. `native_send_packet(ClientCommand { action: 0 })` respawns the player on Horizon's own server and on the
   integrated server. Check: the death screen closes about 600 ms after the click.
6. `native_net_send_to_server` works while the player is dead (`joined` stays true). Check: the server log
   has no "Held respawn not told" line; with a slow search, the respawn comes when the search ends.
7. `native_mod_screen_open` opens the hold screen right after the respawn load, and the cancelled keys and
   clicks keep the player still (no walking, no inventory, Escape does nothing) until the dissolve. Check:
   hold W and press Escape and E during the black after the load.
8. `native_mod_screen_close` gives the mouse back to the game at the dissolve (`capture(true)`). Check: the
   view turns with the mouse right after the dissolve.
9. The terrain test (no loading screen, the player's chunk on the client for 500 ms) keeps the reveal until
   the landing is drawn. Check: the dissolve never shows missing chunks.
10. The music stops at the black and comes back some time after the journey. Check: die with music on.
11. The chimes, the resonance, the break and the bell are heard at the right moments and volume, also
    during the respawn load (sounds at the death place). Check: listen through a whole journey.
12. The message shows in the player's language at twice the GUI text size, fades in and out, and never
    flickers at a low alpha. Check: the display text rule on, then off (`/bmh rule`).
13. `native_physical_window_size()` and `gui_size` give the GUI scale that the batch uses, at scales 1 to 6
    and after a resize. Check: the stars are one or two screen pixels, the art fills the window.
14. With the gamepad, `immediate_respawn` or `horizon-ctl`'s `respawn` (GAPS 7, question 16), the cinematic
    starts in black and ends. Fix round 1: with `immediate_respawn` the player ends at the landing (script n5)
    and the black starts when `Departure` comes; after `horizon-ctl respawn` no frame shows the arrival world
    before the black. Check: the `immediate_respawn` game rule on, die: black, sky, chain, message, dissolve
    at the landing; and `horizon-ctl` respawn while the death screen is up, `screenshot` every 0.1 s.
15. The frame cost of up to about 15 000 fills a frame during the dissolve. Check: the frame rate in F3
    during the dissolve at a large window.

## Changes outside my files

1. `horizon/native/mixins/bmh/hook_frame.rs` (new) and its `[[injection]]` in `horizon-mixin.toml`: wires the
   stamped hook `frame` (catalog name `frame`, no API number) to `bmh_journey::on_frame`. Reason: sending the
   held respawn, the sounds and the hold screen need `&mut Game`; `gui_render_post`, `music` and the screen
   hooks are `&self`.
2. `docs/horizon/HOOKS.md`: the row of `frame`, and the counts line (66 hooks, 63 stamped).
3. `docs/horizon/log/journey-ref/JourneyRef.java` and `out.txt` (new): the Java reference program of the
   unit tests and its output. Not built by Gradle (under `docs/`).

`logic/src/stars.rs` is unchanged: the 26.1 overlay draws its own sky and uses no class of `client/sky/`.
The logic reads `bmh_logic::respawn::{BELL_SOUND, RESPAWN_SOUND}` (the same `SoundInit` ids) and changes
nothing there.

## Fix round 1

Brief: the coordinator's fix round for the journey area and the respawn defect n5. Base `9e847b8` (tip of
`horizon-26.3`). Reviews: `review/journey-parity.json`, `review/journey-semantics.json`.

1. Guards: worktree `.claude/worktrees/agent-a144e37037d0a85ef`, branch `worktree-agent-a144e37037d0a85ef`, reset to
   `9e847b8`. Read ARCHITECTURE.md, HOOKS.md, this log, respawn.md, game-checks-1.md (checks 9, 10, the defects
   table), both review files, `bmh_journey.rs`, `logic/src/journey.rs`, `bmh_respawn.rs`, script n5 of
   `test-slice.sh` (it waits 6 s after `kill @s` with `immediate_respawn` true, and wants the landing distance and
   no new WARN line of the mod).
2. Host facts checked in the Nightly (HN = `crates/horizon-client/src/game/`):
   - The death packets go out in the death tick (`remote.rs:4820-4840` on the client). A mod message sent in
     `ServerTickEnd` leaves at the next tick's `flush_native_net` (`lan.rs:2826`, after `lan_poll`), so `Departure`
     always comes after the death packets, and with `immediate_respawn` it can come before or after the client
     handles `C::Respawn` (the server handles `ClientCommand` in that next tick's poll, `lan.rs:2332`).
   - `native_teleport_player(uuid, dimension, pos, yaw, pitch)` (`native_travel.rs:55`, pending fragment
     `teleport_player.json`, no API number, like `play_sound_to` that the bundle already uses): from a tick hook
     with nobody swapped in, `native_teleport_now` (`:333`) moves a placed, living guest at once in its level
     (`teleport_to_in_level`, `commands.rs:1531`, which drops the guest's pending server trips, so a world spawn
     trip of the vanilla respawn cannot move the player back), or queues a level teleport for another level.
   - From `ServerTickEnd`, `native_player_stat`, `native_tell`, `native_server_players` and the player attachments
     (`mod_api_host.rs:135-142`, `guest_attachments`) reach a guest, so the arrival steps and homecoming's
     `on_exile` can run there.
   - Client: `NativeScreenKind::of` (`native_client_services.rs:75-84`) gives `Other` both for the respawn's level
     load (`remote_screen_up`) and for a prompt, book, sign or pack reload over the death screen. No helper tells
     them apart, and none gives the level-loading state alone or the local player's health
     (`NativePlayer`, `native_hooks.rs:2030`). `native_mod_screen_open` refuses over any other screen
     (`native_hooks.rs:10973-10995`), so the hold screen never replaces the death screen.
3. Server fix for n5 (`bmh_respawn.rs`, owner respawn, changed here for the coordinator's n5). Design, checked
   against the points above: `player_respawn_position` with a search that has not ended keeps it in `searches`
   with `early = Some(Early { name, handed })` and lets the vanilla point stand (an info line, no WARN). A stale
   search is dropped as before, and the client gets `Ready` so that it does not wait. `player_respawned` with no
   exile starts the 200 ticks of immunity at once for an early respawn. At `ServerTickEnd`, after the deaths of
   the tick, each early search that ended goes to `finish_early`: the same placement as the normal path
   (`place`, the old body of `player_respawn_position`), a living player on the server, `native_teleport_player`
   to the landing with the player's own look, then the same arrival steps (`arrive`, the old body of
   `player_respawned`: awards, immunity, sounds, anchor charge, `Arrival`). The exile goes back through
   `hook_server_tick.rs` to `bmh_homecoming::on_exile`. No lock is held across a call into the game. A quit in
   the gap drops the search (`player_logged_out`), and a restart loses the static: in both cases the player
   keeps the vanilla point, alive, with no exile. A new death drops the old search first (`handle_death` runs
   before `finish_early` in the same tick). `player_spawn_point_set` no longer marks an early search stale (the
   respawn itself can reset a point that does not work). The normal path (search ended before the respawn) runs
   the same code as before, moved into `place` and `arrive`.
4. Client fixes (`logic/src/journey.rs`, `bmh_journey.rs`):
   - `Client.armed` is a plain bool that belongs to one death: `Departure` arms it only when the world does not
     show (`playing` false: the death screen, or `Other` that may cover it). The press uses it, and the first
     frame that shows the world again (`View` neither `death_screen` nor `covered`) turns it into a journey in
     black (`Journey::waiting`) and clears it. So a stale arm cannot hold a later death (parity major 1,
     semantics major 2).
   - `Departure` while the world shows (immediate respawn, or a respawn before it came) starts
     `Journey::waiting` at once: black, no music fade, no respawn request. A waiting journey reveals only after
     `Arrival`, or ARRIVAL_WAIT after `Ready` when no arrival comes (the server kept the vanilla point); it
     gives up SERVER_WAIT after its start, or GIVE_UP after the arrival; the arrival starts the terrain test
     again (the player moves). A `Departure` while a journey runs and the world shows makes that journey wait
     for the new arrival.
   - `View.covered` (`Other`, not the mod's own hold screen): such a frame changes nothing in the death screen
     state, so a prompt, book or pack reload over the death screen keeps the arm and the quit confirmation
     (parity minor 3, semantics minor). The respawn's loading screen is also `Other`, so after a respawn
     without the hold the black starts at the first frame of the world, not at the loading screen (arrival
     frame minor).
   - `press` with no arm cancels a `Swallow` (a button press in DEATH_DELAY), as vanilla's inactive buttons do,
     so a fast press waits for `Departure` (semantics minor, press before Departure).
   - `on_net_from_server` reads the world first and resets the client state for a new world there
     (`Client::leave_world`), so the frame of that world no longer wipes the signal (semantics minor).
   - Tests: 8 new client and journey tests, 2 old ones rewritten for the new behaviour
     (`the_client_starts_in_black_when_the_respawn_went_without_it` became
     `a_death_screen_closed_without_the_hold_starts_black_in_that_frame`;
     `a_departure_before_the_death_screen_stays_armed` became `a_departure_under_a_covering_screen_stays_armed`:
     on Horizon `Departure` always comes after the death packets, step 2). `test-logic.sh`: 351 passed.
5. Docs: HOOKS.md rows `server_tick` (respawn gives the early exiles, the helper hands them to
   `bmh_homecoming::on_exile`), contracts "Journey signals" and "Exile of a distant respawn". This log:
   Differences 2 and 4 changed, 14 and 15 added, check 14 changed. respawn.md: difference 3. Reviews: `status`
   and a `round1` field on each finding (parity 2 fixed, 1 not fixed; semantics 6 fixed).
6. Gates: `test-logic.sh` 351 passed, 0 failed. `check.sh`: `{"mods": ["bmh"], ..., "check": "ok"}`. No game
   was run (brief). Nothing committed (the coordinator commits).

### What a game check must prove (fix round 1)

1. n5: script n5 (`test-slice.sh --only n`): with `immediate_respawn` true the player ends at the landing
   distance and no new WARN line of the mod. The log has "did not end before the respawn" then "moved to the
   landing after an early respawn". Also look: black from `Departure`, then the sky, the message, the dissolve
   at the landing; the advancements and the homecoming record ("Homecoming record:" line) come once.
2. Arrival frame: `kill @s`, wait for "Landing spot ready", `horizon-ctl respawn`, `screenshot` every 0.1 s: no
   frame of the arrival world before the black (a loading screen frame may show).
3. Stale arm: `immediate_respawn` true, die once (journey in black), set it false, `bmh rule
   random_death_spawn_point false`, die: the Respawn click respawns at once, no black.
4. Screen over the death screen: die, open a prompt over the death screen if the game allows one (a server
   dialog or pack prompt), close it, click Respawn: the fade starts (held).
5. Press before Departure: die and click Respawn as fast as possible: the click is held (fade), or ignored in
   the first second.
6. First frame signal: needs a slow link; not covered by a script.
