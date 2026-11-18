# Area brief: sky (wave 2 of the run of 2026-11-18)

Read `docs/horizon/briefs/port.md` first and follow it exactly. This file is the part of your prompt that is about
your area. Your prompt gives BASE (the sha your worktree must be on).

No game in this step. Do not run `check.sh build`, `test-slice.sh`, `play.sh` or the game: another agent has the
game rights. Your proof is the two gates of port.md. What only a game can show goes in "To check in the game".

## Your area

The star reading of the Navigator's Amulet, drawn in the night sky on the client: one chain of real stars for
each place (the named points, then the world spawn as a lighter line), traced star by star, held, then faded.
Through a spyglass, the name of the line whose last star the view aims at shows under the centre of the view and
fades in over 2 seconds. Today the amulet writes the reading and the sky draws nothing (game checks 2: "the sky
area is not ported: the reading is only written on the stack").

## Your files

- `<R>/horizon/native/mixins/bmh/bmh_sky.rs` (the module in the game crate: the hooks that have a `Game`).
- `<R>/horizon/native/mixins/bmh/bmh_sky_render.rs` (a `pub` module in `horizon-client-render`, for the hooks
  `world_drawn` and `render_needs`, which have no `Game`). This crate cannot use `bmh_logic` and cannot call the
  game module (HOOKS.md, "Hooks with no Game"). So `bmh_sky.rs` computes the geometry with `bmh_logic::stars` and
  hands plain data (vertices, a pose, a strength) to a `pub` function of `bmh_sky_render`, which keeps it in its
  own static and draws it. With the mod off, that static stays empty and the render hooks draw nothing.
- `<R>/horizon/native/mixins/bmh/logic/src/stars.rs` (the logic, with its unit tests): the star field, the chain
  choice, the trace geometry, the timeline, the aim test, the constants.
- `<R>/docs/horizon/log/sky.md` (your log, from your first step).
- No content file.

## Your rows of GAPS.md

V1 (the star reading drawn in the sky; tier N): `world_drawn` with `render_needs.world_depth`, an own wgpu
pipeline, a depth test against the sky; the star angle recomputed from `clock`. Listed difference (GAPS.md 3.3):
the lines draw after the world, not in the sky pass, so they can show over clouds and rain. V2 (the line name
through the spyglass; tier N): `living_entity_use_item_tick` on the client for `minecraft:spyglass`, the name
drawn in `gui_render_post`. Read also GAPS.md section 7, question 15 (do the lines match Horizon's stars, angle
and brightness; which world passes write depth).

## Audit sections

`<R>/docs/horizon/audit/4-amulet-client.md`, section 2 "The star reading in the sky" and section 3 "The spyglass
line name", and `<R>/docs/horizon/audit/verify-4.md`, rows 2 and 3 and the table "Hooks and helpers the rows
need" (the verification wins: row 3 changed to HOOK through `living_entity_use_item_tick`). Audit 4 did not read
`StarChain.java`, `StarTrace.java` or `K_GuiSpyglassMixin.java`: read them yourself. It cites Horizon main's line
numbers: find each function again in HN.

## Java sources

Under `<R>/common/src/main/java/github/mrornithorynque/bmh/`:

- `client/sky/StarReadingRenderer.java` (the start rule, the time base, the level reset, `aimedName`, `lastStar`,
  `timeOfDay`, `pick`, the log lines).
- `client/sky/StarField.java` (vanilla's star field, seed 10842), `client/sky/StarChain.java` (the chain choice,
  `toWorld`, `toward`, `behindSunOrMoon`), `client/sky/StarTrace.java` (the trace in time, the colours, the
  widths, `STRIDE` 7).
- `mixin/D_LevelRendererSkyMixin.java` (the draw runs only when vanilla draws stars, with the stars' pose).
- `mixin/K_GuiSpyglassMixin.java` (the name: colour `0xB58CFF`, fade in over 2000 ms, not drawn below alpha 4,
  centred at half the GUI width and half the GUI height plus 16; a new aimed name starts the fade again).
- `<R>/common/src/main/java/github/mrornithorynque/bmh/handlers/AmuletReading.java` (lines 63-74: the azimuth
  convention, 0 is north (-Z), 90 is east (+X)).

Rule of the Java sky pass (old CLAUDE.md, `docs/horizon/JAVA-CODE-MAP.md`): the pass culls back faces; triangles
wind counter-clockwise as seen from the camera, as vanilla's star quads. Keep that winding in your geometry and
pin it with a unit test.

## Contracts (HOOKS.md, "Contracts": keep every signature)

- You READ the star reading (row "Star reading of an amulet", fixed for this wave). The amulet area (server)
  sends `bmh_logic::wire::SkySignal::Reading(bmh_logic::contract::AmuletReading)` on `wire::SKY_CHANNEL`
  (`bmh:sky`) in the same use that writes the reading on the stack, only to the player who read. You read it in
  `bmh_sky::on_net_from_server(game, channel, data)` (already wired in `hook_net_from_server.rs`, with an empty
  body): test `channel == SKY_CHANNEL` and `game.native_window()` first, then `SkySignal::decode`. The decoded
  reading has as many names as azimuths; test `spawn` against the line count before you use it.
- Do not read the amulet stack. `docs/horizon/log/amulet.md`, section "Reading contract for the sky area", names
  hand routes; this contract replaces them (no route reads the main hand on the window outside the first-person
  hand pass).
- Keep the Java start rule (StarReadingRenderer.java:172-209): a reading starts when its `time` is newer than the
  last one shown and within 40 ticks of the client's game time (`native_clock()`); it plays to the end; another
  level resets it (:124-130). With two readings in one tick, the newest wins.
- You GIVE nothing to another area.
- You SHARE `gui_render_post` with journey: sky draws first, then journey, so the black screen of the journey
  covers the name. Keep that order.

## Points to settle yourself, by reading HN, and to record in your log

- Where the geometry is built each frame. `world_drawn` has no `Game`. The `frame` hook runs once a frame, after
  the ticks and before the drawing (HOOKS.md row `frame`, `extras_frame`, game/extras.rs:64), with `&mut Game`.
  Wiring sky into `hook_frame.rs` (one line, and its HOOKS.md row) is an allowed change outside your files.
- The sky turn and the star brightness. Horizon computes `star_angle` and `star_brightness` in
  `horizon-client-render/src/environment.rs` (lines 133 and 288 for the angle) and no helper exposes them (audit
  4 section 2, verify-4 row 2). Recompute them from `native_clock()` (game time and time of day) and
  `native_weather()`, port the 26.x timeline that Horizon uses, and pin the values with unit tests. Java scales
  the alpha by `min(1, starBrightness / 0.5)` (StarReadingRenderer.java:142) and draws only when vanilla draws
  stars (brightness above 0).
- Horizon's own stars. `horizon-client-render/src/gfx/stars.rs` rebuilds the same field (seed 10842, 1500 tries).
  Check that star i of your port is star i there, and how Horizon turns the sky (`clip_from_sky`).
- The draw. `WorldDrawn` (gfx/native_render.rs:47-95) gives `device`, `queue`, `encoder`, `color_view`,
  `color_format`, `depth` (reversed: 0 is the sky), `depth_size` (can be smaller than `size`), `view_proj`
  (camera relative, no jitter). Set `world_depth` in `render_needs` only while a reading shows. Java's blend is
  vanilla's star blend (SRC_ALPHA, ONE: light adds up), no depth write, back faces culled. Build the pipeline
  once and keep it; a frame without a reading draws nothing.
- The time base. Java counts GUI ticks plus the partial tick, not game time (StarReadingRenderer.java:131-133).
  Find the client tick count or a frame clock in HN and say which you use.
- The level reset. Find what tells the window that the world or the dimension changed (`native_player()`,
  `native_clock()`, the world lifecycle) and drop the shown reading then.
- The spyglass. `living_entity_use_item_tick` fires on both sides (HOOKS.md, `both` rules): act only when
  `native_window()` is true and the stack is `minecraft:spyglass`, and keep a "scoping until tick n" stamp, since
  a hand change stops a use without a release (verify-4 row 3). The view direction comes from `native_camera()`
  (yaw, pitch or `view`). Draw the name with `native_text` in `gui_render_post`. Java draws it at the end of
  the spyglass overlay: read whether vanilla 26.1 draws that overlay under F1 (`Gui.extractCameraOverlays`) and
  use `gui_hidden` the same way. The fade uses wall time in milliseconds, as Java does.
- The log lines. Keep the Java lines `Sky azimuth self-check: ...` (once) and `Star reading toward ...` (one per
  line) with the `[BalancedMcHardcoreMain]` prefix: the game agent reads them.
- State. `CLIENT_*` statics only, behind a `Mutex`. No lock held across a call into the game or the renderer.
  Per-frame work must stay cheap: compute the chains once per reading, not once per frame.

## Differences from the Java mod that you may list

- V1 (GAPS.md 3.3): drawn after the world, so the lines can show over clouds and rain.
- The reading comes in a mod message, not from the stack in a hand. Effect: an amulet with a fresh reading that
  another player hands over within 2 seconds starts nothing. Java started it (StarReadingRenderer.java:172-185).
- The star angle and brightness are recomputed (GAPS.md V1); any drift is a listed risk until a screenshot
  compares them.

## Worked examples

- `<R>/docs/horizon/briefs/area-amulet.md` and `<R>/docs/horizon/log/amulet.md`: a client area with `CLIENT`
  state, a mod message read in `net_from_server`, drawing in a client hook, and the `frame` hook used for work
  that needs `&mut Game`.
- `<R>/docs/horizon/log/journey.md`: drawing in `gui_render_post` over the whole HUD, and a timeline driven from
  `frame`.
- `<R>/docs/horizon/briefs/area-eternal.md`: the shape of a brief.
- No mod in MODS draws in `world_drawn` (searched). Read HN/docs/native-mods.md (the `world_drawn` section) and
  HN/crates/horizon-client-render/src/gfx/native_render.rs (its unit tests draw with a pass of their own).

## Open review findings that touch your area

None open. `review/amulet-semantics.json` finding 1 (the hand route of the reading) is fixed in the text, and
the contract above replaces the route it named. The journey area, which shares `gui_render_post` with you, is
not reviewed yet.

## To check in the game (start your log's list with these)

1. At night (`/time set 18000`, `/weather clear`), outside, use the amulet with one named point: within a second
   a chain of 5 to 7 stars traces toward the point, holds about 30 seconds and fades. The log has
   `Star reading toward <azimuth>` for each line. A screenshot facing the point shows the chain over Horizon's
   own stars, star on star.
2. The spawn line is lighter than a point line. A reading with no spawn (the server did not know it) draws only
   the point lines.
3. Third person (F5) and F1: the chain still draws (the reason the reading comes in a message).
4. Through a spyglass, aim at the last (lowest) star of a named line: the name fades in under the centre, in
   amethyst. Move away: it goes at once. The spawn line shows no name. F1 does what the log says vanilla does.
5. Rain or day: the chain is fainter or absent, as the stars are. Go to the Nether during a reading: it stops.
6. Clouds and weather over the chain (the listed difference): note how it looks, with a screenshot.
7. A relog during a reading, or an amulet picked up 3 seconds after another player read: nothing starts.
8. The cost: the frame time with a reading on screen and with none (F3), so the render hooks cost nothing when
   idle.
9. The sky azimuth self-check line is in the log once, with the expected values.
