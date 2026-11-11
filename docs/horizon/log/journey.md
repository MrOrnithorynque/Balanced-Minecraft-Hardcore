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
   starts in black at the arrival, with no fade and no music fade, as the Java "joined dead" path.
3. The hold finds the Respawn press through a model of Horizon's death screen (button places, focus, quit
   confirmation), because no hook fires on the respawn itself (GAPS.md C4). A gamepad that moves the focus
   (pad_menu.rs) is not seen, so a key press after it may be read wrong.
4. The death screen buttons count as inactive for 1 s from the frame that first sees the death screen, and
   a press on them in that second is cancelled (vanilla's 20 ticks; Horizon's own delay does not run on a
   remote death because `died_at` is not set there, remote.rs:4810-4840). Only while a journey is armed.
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
    starts in black at the arrival and ends. Check: the `immediate_respawn` game rule on, die: black, sky,
    chain, dissolve; and `horizon-ctl` respawn while the death screen is up.
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
