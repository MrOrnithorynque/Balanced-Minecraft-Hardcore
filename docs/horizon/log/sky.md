# Log: sky area (wave 2, run of 2026-11-18)

Area agent of the BMH Horizon port. Brief: `docs/horizon/briefs/port.md` and `docs/horizon/briefs/area-sky.md`.
No game rights: the proof is the two gates (test-logic.sh, check.sh).

## Steps

1. Tree check: the root is `.claude/worktrees/agent-a66be0e25bedd2ee7`, branch `worktree-agent-a66be0e25bedd2ee7`.
   BASE 9773ae7 was not an ancestor, so the branch was reset to BASE (once, before any write). The guard file
   `horizon/native/mixins/bmh/horizon-mixin.toml` exists. Read port.md and area-sky.md.
2. Read ARCHITECTURE.md, HOOKS.md, GAPS.md rows V1 and V2, audit 4 sections 2 and 3, verify-4 rows 2 and 3, and
   the Java sources (StarReadingRenderer, StarField, StarChain, StarTrace, D_LevelRendererSkyMixin,
   K_GuiSpyglassMixin, AmuletReading lines 63-74).
3. Settled by reading HN (Horizon Nightly fdc87ae). Findings:
   - Star field: `horizon-client-render/src/gfx/stars.rs:12-42` (`star_triangles`) runs seed 10842, 1500 tries,
     the same filter, `normalize() * 100`, and the same corner maths as JOML `rotateTowards(-c, up).rotateZ(-zRot)`
     (columns left, up, forward; then the spin). Six vertices per kept star, triangles (q0 q1 q2) (q0 q2 q3), the
     same as StarTrace's own star square. So star i of the port is Horizon's star i. Horizon draws the stars with
     cull mode None.
   - Sky turn: `game.rs:12223-12291` (`celestial_geometry`): the stars' frame is `Ry(-90) * Rx(star_angle)`, and the
     sky pass draws them with `jittered * frame` (`gfx.rs:9141-9147`). This gives
     `world = (-(y sin a + z cos a), y cos a - z sin a, x)`: the Java `StarChain.toWorld`. Stars draw only in the
     Overworld, with no darkness or blindness effect, and only when the star brightness is above 0
     (`game.rs:16024-16032`, `12289-12290`).
   - Star angle and brightness: `environment.rs:239-290` samples the tracks of `data/minecraft/timeline/day.json`
     with `view_day_time()` (whole ticks of `world.day_time`, `pvp.rs:191-194`; a PvP and HUD fixed time can
     replace it), then `apply_weather` (`environment.rs:316-384`) moves the star brightness to 0 by the rain level
     (rain minus thunder) and then by the thunder level. The 26.3 data (cache resources/26.3, day.json):
     star_angle keyframes (6000, 360) and (6000, 0), ease cubic_bezier [0.362, 0.241, 0.638, 0.759]; star
     brightness 12 keyframes, linear, modifier "maximum" (Horizon ignores the modifier). Track sampling and the
     bezier solver are `environment.rs:14-118`. The port copies them, with the keyframes as constants.
   - Draw: `WorldDrawn` (`native_render.rs:49-88`): `depth` is Depth32Float, sampleable, reversed (0 is the sky),
     and `depth_size` can be smaller than `size`. So the pass does not attach the depth: the fragment shader loads
     the depth texel under the pixel (scaled to `depth_size`) and discards a pixel whose depth is not 0. The call
     site is `gfx.rs:11042-11071`, after the world post pass; `view_proj` has no jitter. The frame is the
     swapchain's 8-bit format without its sRGB suffix, or Rgba16Unorm (HDR, smooth gradients), or Rgba16Float
     (`gfx.rs:2021-2075`). Horizon's stars write raw values with the blend (One, One) and a premultiplied colour,
     and set the alpha to 0.175 (the HDR mark). The port writes raw values the same way and keeps the alpha that
     is there.
   - Time base: no helper gives the GUI tick count. `frame` gives the frame time (`extras.rs:52-64`). The
     `client_tick_pre` hook ticks while paused too. The port uses a monotonic clock (`Instant`), 20 ticks a second.
   - Level reset: `native_client_state().world` (`NativeWorldId`) and `native_player().dimension`. A change of
     either drops the shown reading and the last shown time.
   - Spyglass: `living_entity_use_item_tick` fires on the window for the local player (entity None) every client
     tick of a use (`remote.rs:9280-9310`, `item_use.rs:153-158`). Horizon draws the spyglass overlay in
     `fov.rs:252-264` only in first person while scoping, inside `if show_hud` (`game.rs:8508-8517`: F1 hides the
     camera overlays). So the name draws only with `gui_hidden` false. No helper gives the camera type: the port
     takes first person as "the camera is within 1 block of the player's eyes".
   - Text: `native_text` draws opaque only. `native_gui_paint` with `GuiDraw::Text` takes an ARGB colour
     (`hud_layout.rs:63-120`), as the journey area does. `batch.width` is `ceil(window / scale)` (`hud.rs:1636`),
     Java's `guiWidth()`.
4. Wrote `horizon/native/mixins/bmh/logic/src/stars.rs`: the star field (JOML float maths, `cosFromSin`), the
   chain search, the trace, the 26.3 timeline (star angle with Horizon's bezier solver, star brightness, the
   weather), the client state `Sky` (hear, level, frame with the start rule, aimed_name), the spyglass name fade
   and place, the log lines (Java `%.1f` rounds the shortest decimal half up: `java_fixed1`). 27 unit tests pass
   (`sh horizon/tools/test-logic.sh stars`). Next: pin the Java values with a run of the real Java classes
   (StarField, StarChain, StarTrace; JOML 1.10.5 from the Gradle cache), in `<R>/build/sky-ref` (ignored by git,
   deleted at the end).
5. Java reference run: javac (openjdk 21) on StarField, StarChain, StarTrace of common/src and a small driver
   `SkyRef` (in `<R>/build/sky-ref`, not kept), with JOML 1.10.5 from the Gradle cache. It printed: 780 stars;
   centres of stars 0, 1, 2, 500, 779; the chains of 8 (azimuth, time of day) cases; the trace of the 90 degree
   chain at 0.15, 1, 20 and 33 seconds (float count, a weighted sum, the first vertex); Java `%.1f` of 0.15,
   -0.04, 12.25F (0.2, -0.0, 12.3). Three tests pin them (`the_field_is_the_java_field`,
   `the_chains_are_the_java_chains`, `the_trace_is_the_java_trace`): all pass, the centres to 1e-9.
6. Wrote the module files. `bmh_sky.rs`: `on_net_from_server` (window only, `SkySignal::decode`, `Sky::hear`),
   `on_frame` (new: level key, clock, weather, `Sky::frame`, log lines, `bmh_sky_render::show`),
   `on_gui_render_post` (not under F1, first person, spyglass stamp, aimed name, fade, `GuiDraw::Text` with ARGB and
   a shadow), `on_living_entity_use_item_tick` (window, entity None, `minecraft:spyglass`: stamp the client game
   time). `bmh_sky_render.rs`: `show`, `on_render_needs` (world_depth while a frame waits), `on_world_drawn` (takes
   the frame, a pipeline built once per device and format, the depth as a texture, the sky test in the shader).
   Integrator change: `hook_frame.rs` calls `bmh_sky::on_frame` after amulet, and the HOOKS.md `frame` row names
   sky and the order. Next: both gates.
7. Gates. `sh horizon/tools/test-logic.sh`: 224 passed (33 of them in `stars`). `check.sh`: `"check": "ok"`
   (log `~/Library/Caches/bmh-horizon/logs/check-20261118-163754-52612.log`). The WGSL of `bmh_sky_render.rs` is
   compiled only at run time, so the compile does not prove it: `naga` 29.0.3 (`~/.cargo/bin/naga`) validated it
   ("Validation successful"); Horizon uses naga 30.
8. Notes for GAPS.md question 15. The angle and the brightness: the port samples the same 26.3 tracks with a copy
   of Horizon's sampler, so it gives Horizon's `sky.star_angle` and `sky.star_brightness` for the same day time
   and weather (Horizon uses the frame's partial weather, the port `native_weather`, which is the weather at the
   end of the tick). The star field: the port's star i is Horizon's star i (same stream and filter). Horizon's
   `next_double` multiplies by 2^-53 and Java by the float 1.110223E-16F, so each quad's spin can differ by about
   1e-6 radians: not visible. Which passes write depth (clouds, weather, particles) is not settled by reading: a
   game check (entry 6).
9. Removed `<R>/build/sky-ref` (the Java driver, its classes and output, the shader copy).

## Fix round 1

Fix agent of the sky area. Inputs: `docs/horizon/review/sky-parity.json` (2 minor) and
`docs/horizon/review/sky-semantics.json` (2 major, 4 minor). No game rights: the proof is the two gates.

1. Tree check: `git rev-parse --show-toplevel` prints this worktree. Read port.md, this log and both reviews, then
   `bmh_sky.rs`, `bmh_sky_render.rs`, the `Sky` state and `scoping` in `logic/src/stars.rs`, and
   StarReadingRenderer.java:110-210.
2. Evidence of each finding, read in HN and in the code:
   - Semantics major 1: confirmed. `bmh_sky_render::build` makes the module and the pipeline with no error scope.
     Horizon guards its own builds with a Validation and an Internal scope, popped with `pollster::block_on`
     (`gfx/animate.rs:399-428`, `gfx/core_lines.rs:48-49`, `gfx/world_post.rs:180-181`). Horizon binds the world
     depth as an unfilterable float texture: `gfx/world_post.rs:158-160` ("the GLSL backend can read it") and
     `gfx/pack_post.wgsl:14,29` (`texture_2d<f32>`, `.r`). `pollster` 0.4 is a dependency of horizon-client-render.
   - Semantics major 2: confirmed. `gui_render_post` fires with a menu open (native_hooks.rs:2626-2627), and the
     client tick, so the use tick and the game time, stops while paused (game.rs:18475). The helper is
     `native_client_state().screen` (`NativeScreenKind`: Playing, Chat, Death, Other; native_client_services.rs:57-84),
     which the journey area already uses.
   - Parity minor 1: confirmed (stars.rs:977, bmh_sky.rs:180). The fix of major 2 (a stamp on the area's clock)
     fixes it too.
   - Parity minor 2: confirmed. `Sky::level` drops `pending`; Java resets only `lastShownTime` and `shown`
     (StarReadingRenderer.java:124-130), and the reading stays on the stack.
   - Semantics minor, frame generation: confirmed. `frame_gen.generate` (gfx.rs:11011) runs before
     `native_world_drawn` (gfx.rs:11055). A mod cannot draw into the made frame: not fixable here.
   - Semantics minor, splitscreen: confirmed (split.rs:15, one `Game` per player in one process; players in one
     window share its `window_id`). A fix needs per-player state in every area (ARCHITECTURE.md State says one
     value for the process): not small.
   - Semantics minor, HDR alpha: confirmed (stars.wgsl:16-19 writes 0.175; `keep_world`, gfx.rs:11333, copies the
     world after `world_drawn`, so the reading's alpha reaches hdr.wgsl:202-212). Small: fixable.
   - Semantics minor, old device: confirmed (`GPU` keeps a device clone until the next drawn reading).
3. Major 1: `bmh_sky_render.rs`. `build` runs inside a Validation and an Internal error scope (`Checks`, the
   pattern of gfx/animate.rs:408-422) and gives None on an error; the per-frame bind group (Horizon's depth view)
   is checked the same way. On an error: one warn line, `BROKEN` set, nothing draws for the rest of the session.
   The depth binds as `texture_2d<f32>` (unfilterable float) and the shader reads `.r`. naga 29.0.3 validates it
   and writes GLSL ES 3.10 for both entry points (`sampler2D`, `texelFetch`).
4. Major 2 and parity minor 1: `bmh_sky.rs`. The name draws only when `native_client_state().screen` is Playing or
   Chat. The use tick stamps the area's clock (`now`), and `stars::scoping` holds the stamp `SCOPING_MILLIS`
   (100 ms, the old two ticks); `SCOPING_TICKS` is gone. A pause runs no use tick, so the stamp runs out.
5. Parity minor 2: `Sky::level` keeps `pending` on a change to another level, and drops it out of a world. New
   test `another_level_keeps_the_waiting_reading`. Semantics minor HDR: the shader writes the alpha 0.175 with the
   alpha blend REPLACE, and discards a pixel that adds less than half an 8-bit step (no change on an 8-bit frame;
   the sky keeps its mark there). Semantics minor device: `on_world_drawn` drops `GPU` when the device differs,
   before it looks at the frame. `test-logic.sh`: 225 passed.
6. Gates after the round: `sh horizon/tools/test-logic.sh` 225 passed, 0 failed; `check.sh` printed
   `"check": "ok"` with no warning (log `~/Library/Caches/bmh-horizon/logs/check-20261118-165717-62307.log`).
7. Set the status and a note on each finding in `review/sky-parity.json` and `review/sky-semantics.json`.
   Differences 7 to 10 and game checks 12, 16 to 18 are new or changed. No change outside the area's files in this
   round.
8. Removed a check that the next line repeats in `on_world_drawn`, then ran both gates again: 225 passed,
   `"check": "ok"`.

## Differences from the Java mod

1. V1, GAPS.md 3.3: the lines draw after the world (`world_drawn`), not in the sky pass. Where a pass writes no
   depth (clouds, rain, maybe particles), the lines show over it. Bloom, fog and the tonemap of the world post
   pass do not touch them.
2. The reading comes in a mod message, not from the stack in a hand (HOOKS.md contract "Star reading of an
   amulet"). An amulet with a fresh reading that another player hands over within 2 seconds starts nothing; Java
   started it (StarReadingRenderer.java:172-185).
3. V1: the star angle and the star brightness are recomputed from `native_clock` and `native_weather`
   (GAPS.md V1, "a small angle drift is possible"). Not followed: a PvP and HUD fixed time or weather of Horizon
   (a client option, `pvp.rs:191-200`) turns Horizon's sky but not the port's; the weather is the end of the tick,
   not the partial tick.
4. Not in GAPS.md, strong reason: no helper tells the window that the local player has darkness or blindness
   (`native_effects` needs the player's uuid, which the window has no helper for), and none tells whether a
   shader pack draws its own sky (`pack_draws_sky`, `own_stars`). Java drew nothing then, as vanilla drew no
   stars. The port draws the reading in those cases.
5. Not in GAPS.md, strong reason: the time base. Java counts GUI ticks, which stop while the game is paused in
   single player. No helper gives that count; `client_tick_pre` ticks while paused too. The port uses a monotonic
   clock, so a pause does not hold the reading (it runs on behind the pause screen).
6. V2, not in GAPS.md, strong reason: no helper gives the camera type. The port takes "first person" as "the
   camera is within 1 block of the player's eyes" (`stars::first_person`). A third person camera pushed within
   1 block of the eyes by a wall would show the name; a fall faster than 1 block a tick while scoping would hide it.
7. V2: "the player scopes" is a stamp of the area's clock in each use tick of a spyglass; it holds 100 ms
   (`SCOPING_MILLIS`, two ticks). The name can stay one tick after the use stops. (Fix round 1: it was the client
   game time, which the server's time packets move and a pause stops.) The name draws only with no screen or with
   the chat open: Java draws it under any screen, but `gui_render_post` draws over the screens.
8. V1, not in GAPS.md, no fix in a mod (fix round 1): with Horizon's frame generation on, the made frames come
   from the world before `world_drawn` (gfx.rs:11011, 11055), so the reading is only in the real frames and can
   blink at the display rate. Proposed GAPS.md V1 line: "Frame generation: the reading draws in the real frames
   only; it may flicker. Needs a Horizon hook on the made frame."
9. Not in GAPS.md (fix round 1): splitscreen is not supported. The area state is one value for the process
   (ARCHITECTURE.md State), and Horizon runs one window `Game` per player in one process (split.rs:15); players in
   one window share its `window_id`. A reading would show for every player, and two players in different levels
   reset it each frame. Proposed GAPS.md line (all areas): "Horizon splitscreen: not supported; the client state
   of every area is one value for the process."
10. Fix round 1, parity minor 2 left: Java can start a reading it already showed again in a new level while it
    is fresh (the stack still holds it, and `lastShownTime` is reset). The port took it out of `pending` when it
    started, so it does not show twice. A change of level within 2 seconds of a reading is rare.

## To check in the game

1. At night (`/time set 18000`, `/weather clear`), outside, use the amulet with one named point: within a second
   a chain of 5 to 7 stars traces toward the point, holds about 30 seconds and fades. The log has
   `[BalancedMcHardcoreMain] Star reading toward <azimuth>` for each line. A screenshot facing the point shows the
   chain over Horizon's own stars, star on star.
2. The spawn line is lighter than a point line. A reading with no spawn (the server did not know it) draws only
   the point lines.
3. Third person (F5) and F1: the chain still draws (the reason the reading comes in a message).
4. Through a spyglass, aim at the last (lowest) star of a named line: the name fades in under the centre, in
   amethyst, with a shadow. Move away: it goes at once. The spawn line shows no name. F1 hides it (Horizon hides
   the spyglass overlay under F1, game.rs:8508-8517). In F5 with a spyglass no name shows.
5. Rain or day: the chain is fainter or absent, as the stars are. Go to the Nether during a reading: it stops.
6. Clouds and weather over the chain (difference 1): note how it looks, with a screenshot. This also answers
   which passes write depth (GAPS.md question 15).
7. A relog during a reading, or an amulet picked up 3 seconds after another player read: nothing starts.
8. The cost: the frame time with a reading on screen and with none (F3), so the render hooks cost nothing when
   idle.
9. The sky azimuth self-check line is in the log once, with the expected values: `from (0, 0), azimuth 0 aims at
   (0, -100), azimuth 90 at (100, 0)`.
10. The pass draws at all: back face culling (front face counter-clockwise) keeps the triangles. The unit tests
    prove that every triangle winds counter-clockwise as seen from the centre; that wgpu's front face then
    matches Horizon's `view_proj` is the game's to show. If nothing draws but the log has the lines, set
    `cull_mode` to None in `bmh_sky_render::build` and check again.
11. The depth test: `world_depth` keeps the depth, and the sky's depth is exactly 0 (terrain hides the lines, the
    open sky shows them). Check at a Resolution Scale below 100 % too (`depth_size` smaller than `size`).
12. The colours on an HDR frame (Rgba16Unorm or Rgba16Float): the port writes raw values as Horizon's stars do,
    with the alpha 0.175 (fix round 1); the lines must look as bright as their stars, with no lifted box of sky
    around them (`HORIZON_HDR_DEBUG=1` tints lifted pixels magenta). Compare with the 8-bit frame.
13. `living_entity_use_item_tick` fires on the window with `entity` None for the local player's spyglass in
    single player and on a server, every tick of the use (remote.rs:9280-9310).
14. A respawn in the same dimension during a reading keeps it (the level key stays while there is no player);
    a change of dimension ends it.
15. Pause in single player during a reading: the port goes on (difference 5).
16. Fix round 1: no `[BalancedMcHardcoreMain] The star reading does not draw in this session` line in the log on
    Metal, and none with the OpenGL backend (`WGPU_BACKEND=gl` if Horizon reads it): the shader reads the depth
    as `texture_2d<f32>` there. If the line shows, the reading is off and the game goes on.
17. Fix round 1: scope at a named star, then press Esc: the name goes at once and does not show over the pause
    menu, the inventory or the death screen. With the chat open (T) while scoping, the name stays.
18. Fix round 1: frame generation on, a reading on screen: note any flicker (difference 8).

## Changes outside my files

- `horizon/native/mixins/bmh/hook_frame.rs`: one line, `crate::game::bmh_sky::on_frame(game);` after the amulet.
  Reason: the reading must start and its triangles must be built once a frame with `&mut Game`, before the
  drawing; `world_drawn` has no `Game` (HOOKS.md, "Hooks with no Game").
- `docs/horizon/HOOKS.md`: the `frame` row names sky after journey and amulet, and why it goes last.
- No lang key, no content file, no manifest change, no new binding.

## Status

Fix round 1: semantics majors 1 and 2, parity minors 1 and 2, semantics minors HDR and device are fixed; frame
generation and splitscreen are differences 8 and 9, with proposed GAPS.md lines. Both gates pass after the round
(`test-logic.sh` 225 passed; `check.sh` ok, log `check-20261118-165717-62307.log`). No game was run.

Rows V1 and V2 are built. Both gates pass (step 7). No game was run (no game rights): nothing here is shown to
work in the game; the list above says what to check. Nothing is committed.
