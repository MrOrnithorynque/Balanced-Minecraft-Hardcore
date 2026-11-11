# The Navigator's Amulet and the client visuals (sky, cinematic, screens)

Conventions. W = the BMH worktree, S = the scratchpad. Java paths are under `W/common/src/main/java/github/mrornithorynque/bmh/` and written `J:<path>:<line>`. Horizon paths are under `S/src/horizon/` and written `H:<path>:<line>`; `H:native_hooks.rs` means `crates/horizon-client/src/game/native_hooks.rs`, `H:native-mods.md` means `docs/native-mods.md`. "Both catalogs" means I checked `S/src/horizon/native-hooks.json` (main) and `S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json` (Nightly). Every hook used below is in both catalogs except `world_hold` and `world_warp` (main only, and I do not need them).

## Verdict table

| # | Feature | Verdict | Mechanism in one line |
|---|---|---|---|
| 1a | Journey cinematic: drawing (fade, sky, stars, chain, dissolve, message) | HOOK | `gui_render_post` + `fill`/`quad`/`gui_paint` into the GUI batch, drawn over the death screen |
| 1b | Journey cinematic: signals server to client | HOOK | `net_send_to_player` / `net_from_server` (no title packets: `client_packet` is read-only) |
| 1c | Journey cinematic: hold the Respawn request until black | PARTIAL | Death-screen click and key cancelled with `screen_*_pre`, then `send_packet(ClientCommand{action:0})`. GAP for the immediate-respawn rule (no hook on that path) |
| 1d | Journey cinematic: input hold + overlay above the level-loading screen | PARTIAL | `mod_screen_open` as HoldScreen works only after the load. Whether `gui_render_post` draws over the loading screen is UNVERIFIED (doc says no, code looks unconditional) |
| 1e | Journey cinematic: music fade, chimes, bell | HOOK (parity risks) | `music` hook `volume`; `play_local_sound`; no sound category choice |
| 2 | Star reading in the sky | HOOK (raw wgpu), PINNED for exact parity | `world_drawn` + own wgpu pipeline, depth-tested against sky depth 0; Horizon's stars are vanilla's `buildStars` seed 10842 |
| 3 | Spyglass line name | PARTIAL | Draw in `gui_render_post`/`hud`; "is scoping" is not exposed to mods (approximate by held spyglass + use key), exact needs PINNED |
| 4 | Amulet needle (`bmh:north`) | HOOK (data only) | Vanilla `minecraft:compass` range property, `target: lodestone`, default component `minecraft:lodestone_tracker` far north |
| 5a | Amulet item: use / use on / sneak logic / reading rules | HOOK | `use_item_on` + `use_item` (claim), `inventory_tick`, `carried_stack`, `world`/`weather`/`day_time` helpers |
| 5b | Amulet points saved on the player, cap from config | HOOK | `AttachmentType` on `Holder::Player(uuid)`, `copy_on_death: true` |
| 5c | Naming screen, list screen | PARTIAL | `mod_screen_open` + `screen_render_post` + key/char/click hooks. No text field widget: the field is hand-written; no clipboard helper |
| 6 | Create New World "Balanced Hardcore" button | PARTIAL, true parity GAP | Can draw and click a button on screen id `world_create`; cannot read the Game tab, nor write the new world's rules from there |
| 7 | Config screen | PARTIAL | `settings` / `settings_click` give buttons only; numbers become stepper buttons. A text field screen cannot open from Options |
| 8 | Totem animation shows the Totem of Eternity | PINNED (else GAP) | Client picks the item in `Game::find_totem` (`game/totem.rs:166`); no hook |

Counts by row: HOOK 7 (1a, 1b, 1e, 2, 4, 5a, 5b), PARTIAL 6 (1c, 1d, 3, 5c, 6, 7), PINNED 1 (8), GAP 0 as a whole row. Two sub-parts are GAP inside PARTIAL rows: the immediate-respawn hold in 1c and the create-screen rules in 6.

Three findings that matter most:
1. The needle needs no code. Horizon's item model resolver has the vanilla `minecraft:compass` property (`H:crates/horizon-assets/src/item_model.rs:882`, `:1582`) and the lodestone target reads a stack component, so a default `minecraft:lodestone_tracker` pointing 30 000 000 blocks north gives the same needle for every holder (own hand, other players, frames), with spin in other dimensions. The conditional-only hook `item_model_property` could not do it (it gets no holder).
2. The whole cinematic can sit on `gui_render_post`, but two seams stay: the immediate-respawn rule calls `respawn_player()` with no hook (`H:crates/horizon-client/src/game/remote.rs:4634`), and the loading screen that a respawn opens (`remote.rs:3444`) may or may not be drawn over.
3. The Create World button can be drawn and clicked, but nothing exposes the screen's rules, tab or hardcore flag, so the Java behaviour (flip six rules in the creation state, button synced with the More tab, greyed in Hardcore) is not reproducible. The fallback is a mod setting applied when the new world first opens.

---

## 1. The journey cinematic

### Behaviour spec (what the Java mod does)

Signals (server to client), `J:handlers/JourneySignal.java`:
- DEPARTURE = key `bmh.journey.departure` (`:15`), sent at death by `onPlayerDeath` only when `RespawnPositionHandler.isDistantRespawn(player)` and the server is not hardcore (`:30-34`); then `RespawnSearch.start` runs on the server (`:33`). It is a vanilla `ClientboundSetTitleTextPacket` with a translatable component, taken out by `J:mixin/F_GuiTitleMixin.java:21-27` (`Gui.setTitle` HEAD, cancellable).
- ARRIVAL = key `bmh.journey.arrival` (`:17`), sent after the respawn. With the message: argument 0 is the component `bmh.journey.message` "A new journey begins for you, %s." with the player name (`:40-42`); without: empty (`:44`). Whether the message is sent follows the `displayTextOnRespawn` rule (`J:client/journey/JourneyOverlay.java:179` comment, `JourneySignal.sendArrival(.., withMessage)`).

Client state machine, `J:client/journey/JourneyCinematic.java`:
- `onTitle` DEPARTURE: remember the local player as "armed" (weak ref) (`:39`).
- `onRespawn(player)` (called from `J:mixin/F_LocalPlayerRespawnMixin.java:19-25`, HEAD of `LocalPlayer.respawn`, cancellable): if `sendingRespawn` pass; if already waiting return true (immediate respawn calls it every tick, `:68-70`); if not armed pass; if another overlay is active pass; else store as waiting, set `new JourneyOverlay(minecraft, player)` and return true (cancel) (`:62-87`).
- `sendRespawn` (`:102-119`): called by the overlay when the screen is black; calls the real `player.respawn()` with `sendingRespawn=true`. If the local player changed (disconnect) it sends nothing.
- `onOverlayChange` (from `J:mixin/F_MinecraftOverlayMixin.java:17-21`): if another overlay (resource reload) replaces the journey, send the held respawn now (`:94-99`).
- ARRIVAL: if the overlay is a JourneyOverlay call `arrive(message)`; if there is no overlay (player joined dead) start an overlay in black (`:40-52`).

Overlay timeline, `J:client/journey/JourneyOverlay.java` (constants `:46-65`): FADE_OUT 600 ms from start; after black: BLACK 250, SKY_IN 1200, HOME_AT 700, TRACE_AT 1500, SEGMENT 210 (11 segments), MESSAGE_AT 1700, MESSAGE_IN 400, MESSAGE_HOLD 1600, MESSAGE_OUT 400, LAST_STAR_BEAT 400, STAR_LIGHT_UP 150, ARRIVAL_WAIT 1000, TERRAIN_GRACE 500, DISSOLVE 1000, GIVE_UP 30000 (counted from the respawn), SERVER_WAIT 150000 (counted from the start).
- Fade (`:222-224`): fill the screen black with alpha = smoothstep(t/600). The death screen is drawn under it (`:212-215`).
- At `now >= darkStart` it sends the respawn (`:268-271`). It fades game music with `1 - ease(t)` during the fade (`:264-266`, `J:client/journey/JourneyMusic.java:22-48`: sets the playing track's volume, stops it at 0).
- After the respawn (player object changed, `:273`): holds input with an empty non-pausing Screen (`HoldScreen`, `:921-949`) while no screen is open (`:374-379`); plays `SoundInit.RESPAWN_SOUND` at volume 0.3 at darkStart+250 (`:282-285`).
- Art (`:226-241`): grid of "art pixels" = round(guiScale/2) screen pixels. Sky: black fill then a vertical gradient SKY_TOP 0x0B1433 to SKY_BOTTOM 0x03040B scaled by `sky` (`:455-460`); a Milky Way band of 420 faint dust squares (`:475-497`); 1100 stars on 2 parallax layers (speeds 0.3, 0.6) plus 360 band stars; each star is one pixel plus "plus" arms, with twinkle (`:499-539`). A random zigzag chain of 12 stars (Home plus 11), segments 0.14 to 0.30 screen heights long, turning 18 to 62 degrees (`:152-176`); Home pulses with two halo rings; the line is drawn pixel by pixel, 7 pixels wide with a violet edge (LINE_CORE 0xF0E4FF, LINE_EDGE 0xB58CFF), stopped 8 px from each star (`:541-655`); the view scrolls to follow the head (`:429-433`).
- Chimes: amethyst chime per star with pitch 0.7 + 0.8*i/11, resonance for the last (`:388-412`), played as `level.playLocalSound(.., SoundSource.PLAYERS, vol, pitch, false)` at the player (`:414-419`).
- Message (`:657-691`): text at (width/2, 0.8*height), scale 2 if it fits else 1, colour 0xFFF1D0, alpha fade in/hold/out, skipped below alpha byte 4.
- Reveal condition (`:320-347`): after the trace ends (`TRACE_AT + 11*SEGMENT + LAST_STAR_BEAT`), arrival signal received (or 1 s after the respawn), message finished, terrain ready (`:352-371`: no LevelLoadingScreen open, the player's chunk present and seen for 500 ms), or the give-up timers.
- Dissolve (`:293-309, :693-706`): from the last star, blocks of `height/72` (min 4) art pixels vanish where `reveal > 0.6*radial + 0.4*bayer4x4/16`; during it the glow fades 5x faster; the amethyst break sound plays (`SoundEvents.AMETHYST_CLUSTER_BREAK`, 0.6, 0.8); at the end the overlay is removed and `SoundInit.BELL_SOUND` plays at 1.0.
- Perf note in Java: the thousands of fills are batched in one GUI element (`ArtLayer`, `:765-866`).

### Horizon mapping

Drawing over everything.
- `gui_render_post` (client, not cancellable, both catalogs). Doc, `H:native_hooks.rs:2517-2525`: "After the complete normal GUI batch, including screens, prompts, toasts, debug overlays, the pad cursor and IME. Fires even with F1 or a menu open; file/world loading and resource-reload overlays do not fire it. The batch is in GUI pixels." It runs in the window's Game. Called at `H:crates/horizon-client/src/game.rs:9573`: `self.native_gui_render_post(&mut hud, self.ui.hide_gui, self.focused);`. Typed event: `GuiRenderPostEvent { batch, gui_hidden, focused }` (`H:crates/horizon-client/examples/native_gui_overlay_probe.rs:58-60`, draws `host.fill(event.batch, [x,y,w,h], 0xffff00ff)`). It is drawn after the death screen (screens are in the batch it follows), so the cinematic can sit on it. Horizon's own death screen is `Screen::Dead` (`H:game.rs:695`), drawn at `game.rs:6235`.
- Primitives, all client, all with the batch in GUI pixels (f32):
  - `native_fill(&self, batch: &mut HudBatch, rect: [f32; 4], argb: u32)` (`H:native_hooks.rs:11108`): rectangles with alpha; fractional coordinates allowed.
  - `native_quad(&self, batch, points: [[f32; 2]; 4], argb: u32)` (`:11286`): a quad, one colour. There is no gradient helper (searched `fill_gradient`, `gradient` in the GUI crates: only `debug_chart.rs:454`). Vertical gradient = N horizontal `fill` bands.
  - `native_text(&self, batch, text, at: [f32;2], rgb: u32, shadow)` (`:11460-11472`): the code does `rgba(r, g, b, 0xFF)`: the alpha is forced opaque. Text with alpha must go through `native_gui_paint` (`H:crates/horizon-client/src/game/native_client_services.rs:427`) with `GuiDraw::Text { text, x, y, color, shadow }` (`H:crates/horizon-client-render/src/hud_layout.rs:109-115`, painted with `abgr(*color)` at `:942-951`, so alpha is kept) wrapped in `GuiDraw::Push(vec![PoseOp::Translate(..), PoseOp::Scale(..)])` / `GuiDraw::Pop` (`:13-16`, `:127-130`) for the 2x scale and centring. `x`/`y` are `i32`.
  - `native_text_width(&self, text) -> f32` (`:11115`-ish, "The width of `text` in GUI pixels") for centring.
  - `native_mod_image(&mut self, width, height, rgba: &[u8]) -> bool` (`:11120`): own RGBA image into one atlas slot (bounded by `MOD_IMAGE_SIZE`; I did not find the constant's value: UNVERIFIED), drawn with `native_sprite`. Not needed: the Java art is rectangles only.
  - Pixel-level drawing for the dissolve: one `native_fill` per art pixel or per block. `HudBatch::fill` appends vertices (`:11108-11111`), no per-element layer search as in Java 26.1, so thousands of fills should be cheap, but I could not measure it: UNVERIFIED.
  - Screen size: `native_client_state().gui_size` (`H:native_client_services.rs:29-39`) and `native_physical_window_size()` (`H:native_hooks.rs:11445`) give the grid; `batch.width`/`batch.height` are GUI pixels.
- Frame clock: `frame` hook (`H:native_hooks.rs:2181`, `native_frame(&mut self, elapsed: Duration)`, client) or `Instant::now()` in the draw hook. The Java uses `Util.getMillis()`.

Receiving the signals.
- Do not use the title packet. `client_packet` (`H:native_hooks.rs:8583`) is "Read-only: the game handles the packet as it would without the mod", so the title would still show.
- Use mod messages (`H:native-mods.md:4582-4595`, API 40): server `native_net_send_to_player(&mut self, player: u128, channel: &str, data: &[u8]) -> Result<(), String>` (`H:crates/horizon-client/src/game/native_net.rs:174`, "at the tick's end, from any server hook", 32 a tick, 30 000 bytes), client `native_net_from_server(&mut self, channel: &str, data: &[u8])` (`H:native_hooks.rs:8569`, "The window's game only"). Channel e.g. `bmh:journey`. A Horizon client without the mod gets nothing (no fallback text, unlike Java's translatableWithFallback; acceptable).
- Departure trigger on the server: `living_death` (s!, `hooks-compact.txt:398`) for "last, once no other mod can cancel it" is the respawn group's job. Arrival: `player_respawned` (s, `hooks-compact.txt:134`). They run in the server's Game; the message hook runs in the window's Game.

Holding the Respawn request (the part that decides 1c).
- The button path: `Screen::Dead if pressed && button == MouseButton::Left => self.death_click()?` (`H:game.rs:10877-10879`) to `death_action` (`H:crates/horizon-client/src/game/screens.rs:911-913`) to `respawn_player()` (`screens.rs:770-776`) to `remote_respawn()` which is `self.send(S::ClientCommand(play::ClientCommand { action: 0 }))` (`H:crates/horizon-client/src/game/remote.rs:10497-10499`, `S` = serverbound; protocol comment `H:crates/horizon-protocol/src/packets/play.rs:1980`: "ServerboundClientCommandPacket.Action: 0 respawn").
- Block it: `screen_mouse_button_pressed_pre` (client, cancellable, both catalogs; doc `H:native_hooks.rs:3750-3766`): "Set `canceled` and the press goes no further"; it fires "while a screen of the game's or the title's is open" and is dispatched at `H:game.rs:10312` before the match that reaches `Screen::Dead` at `:10877`; `screen.id` for the death screen is `"dead"` (`H:game.rs:695`). Keyboard path: `death_key` (`screens.rs:824`) is reached by `screen_key_pressed_pre` (`H:native_hooks.rs:3925-3947`, cancellable) so cancel Enter/Space there. The control-socket path `control.rs:1279` is Horizon's own tooling; ignore. Gamepad: UNVERIFIED (doc says a gamepad click "is not the mouse and fires nothing", `H:native_hooks.rs:3774`-ish).
- Send it later: `native_send_packet(&mut self, packet: ServerboundPlayPacket) -> Result<(), String>` (`H:crates/horizon-client/src/game/native_net.rs:309`; `ServerboundPlayPacket` re-exported at `H:mod_api.rs:92`; API 409, `H:native-mods.md:4644-4652`). The mod sends `ServerboundPlayPacket::ClientCommand(ClientCommand { action: 0 })` (variant name from `H:crates/horizon-protocol/src/packets/play.rs:2363`). It sends the packet as is, and the game does not reset its own client state (`toggles.respawn()` at `screens.rs:774` would be skipped: negligible, key toggles).
- The 1 s guard after death (`ui.died_at`, `screens.rs:779-785`) is Horizon's own and matches vanilla.
- GAP: the immediate-respawn rule. When the rule hides the death screen, the client calls `self.respawn_player()?` itself on the death packet (`H:remote.rs:4626-4635`, `else { self.respawn_player()?; }`). No hook is on that path (the screen hooks do not fire: no screen). So the request cannot be held on the client; the cinematic then starts with the "joined dead" behaviour (black at arrival, Java `JourneyCinematic.java:46-51`). A server-side hold would have to live in the respawn group's `RespawnSearch` (it already waits for a safe landing). Parity risk: no fade-out before the teleport under that rule.

Input hold and the loading screen.
- A respawn sends a `Respawn` packet and the client opens its loading screen: `reason_for_respawn(died, ..)` and `enter_remote_dimension(&p.spawn, true, reason)` (`H:remote.rs:3444-3455`), drawn by `remote_overlay` (`remote.rs:10511`), called at `game.rs:5928` inside `hud()` which then returns early. `screen_hooks_open()` is false during it (`H:native_hooks.rs:4138-4143`: `&& !self.remote_loading()`), so screen hooks and a mod screen are off. Whether `gui_render_post` fires: the doc says no (`:2519`) but `game.rs:9573` calls it after `hud(size)` with no visible guard. UNVERIFIED: run `crates/horizon-client/examples/native_gui_overlay_probe.rs` during a respawn.
  - If it fires: the cinematic covers the loading screen like the Java overlay does.
  - If it does not: black cannot be held across the load; the player would see Horizon's loading screen (a panorama with blur, `remote.rs:10586-10600`) between departure and arrival. That is a PARTIAL.
- HoldScreen replacement: `native_mod_screen_open(&mut self, mod_id: &str) -> bool` (`H:native_hooks.rs:11302-11330`; API 111; "opens only ... over the world with no overlay up", false "while another screen is (or another mod's), ... while the world loads"), closed by `native_mod_screen_close`. It frees the mouse and swallows keys like the Java screen, does not pause. Draw nothing in `screen_render_post`; `gui_render_post` draws the cinematic. Open it once `native_client_state().screen == Playing` after the load, close it at the dissolve start (Java `releaseInput`).
- Terrain readiness (Java `terrainReady`, `:352-371`): Horizon equivalents are `native_client_state().screen` (`NativeScreenKind::Other` while the loading screen is up, `H:native_client_services.rs:53-67`) and `native_client_block_obstruction(pos)` (`:~405`, returns `None` unless `self.world.loaded(x, z)`), which can test the player's column. The 500 ms grace is the mod's own timer.

Sounds and music.
- `native_play_local_sound(&mut self, sound: &str, at: [f64; 3], volume: f32, pitch: f32, delay: i32) -> bool` (`H:native_hooks.rs:11043-11056`; API 83). Vanilla events by id (`minecraft:block.amethyst_block.chime`, `...resonate`, `minecraft:block.amethyst_cluster.break`) and the mod's own `assets/<id>/sounds.json` events (`H:native-mods.md:4276-4284`; this is the content/asset group's job for `bmh:respawn`, `bell`). No sound source/category parameter: Java plays them as `SoundSource.PLAYERS` (`JourneyOverlay.java:417`), so a player's "Players" slider no longer scales them. Parity risk, small.
- Music fade: `music` (client, both catalogs; `H:native_hooks.rs:3070`): "Choose this client's music, once a client tick: `music` and `volume` hold what vanilla chose ... or change `volume` (0 to 1) to fade it." Set `volume *= 1 - ease(t)` during the fade; at 0 set `music = None`. It runs once per client tick (20 Hz), the Java fades per frame via the sound manager: steps of 1/12 over 600 ms, audible only as slight zippering. `native_muffle_sounds(amount, lowpass, ticks, exempt)` (`:11066`) is the alternative but it also mutes the chimes unless exempted.

### Verdict
PARTIAL. HOOK for everything drawn, the signals, the sounds and the music. Parts that are not HOOK:
- GAP: holding the respawn under the immediate-respawn rule (`remote.rs:4634` has no hook). Could become PINNED with a `HEAD` injection on `respawn_player` (`screens.rs:770`) that checks a mod flag, but a mixin on it also needs the mod to pin one commit.
- UNVERIFIED (could be a GAP): drawing over the level-loading screen.

### Parity risks
- No fade-to-black before the teleport under the immediate-respawn rule.
- Possible flash of Horizon's loading screen if `gui_render_post` does not draw over it.
- Vertical gradient is banded fills; the Java gradient is smooth.
- Chime volume follows the master and not the Players slider.
- Music fade steps at 20 Hz.
- The Java overlay blocks the death screen buttons visually; here the death screen under it still receives clicks unless the mod cancels `screen_mouse_button_pressed_pre` for `dead` while the cinematic runs (it must, in all cases after the first click).

---

## 2. The star reading in the sky

### Behaviour spec

- Source of data: the amulet stack's `minecraft:custom_data.bmh_reading` = `{azimuths: [float], names: [string], spawn: int, time: long}` (`J:handlers/AmuletReading.java:34-41, 232-239`; read at `J:client/sky/StarReadingRenderer.java:58-64, 172-209`). Azimuth is degrees clockwise from north (0 = -Z, 90 = +X) (`AmuletReading.java:63-74`).
- Start rule (`StarReadingRenderer.java:172-209`): each frame, for each hand, a held `NavigatorAmuletItem` with `bmh_reading` whose `time` is newer than the last shown and within `FRESH_TICKS = 40` of `level.getGameTime()` starts a reading: one star chain per azimuth, all with the same start tick. Plays to the end even if the amulet leaves the hand. Another level resets it (`:124-130`). Time base: the GUI tick counter plus partial tick (`:133`), not game time.
- Chain choice: `StarChain.pick(StarField.stars(), azimuth, timeOfDay)` (`:268-281`), then `StarTrace.triangles(stars, chain, seconds, skyAlpha, isSpawnLine)` builds per-vertex coloured triangles; `StarTrace.duration(chain.length)` ends it (`:147-152`). (I did not read `StarChain.java`/`StarTrace.java` line by line; their output is a float array of x,y,z,r,g,b,a per vertex, `StarTrace.STRIDE` = 7 (`:291`). A port needs their geometry rules: UNVERIFIED here, read them when porting.)
- `StarField` rebuilds vanilla's star field: 1500 tries, `Random(10842)`, `x,y,z = nextFloat()*2-1`, `size = 0.15 + nextFloat()*0.1`, keep when `0.010000001 < lengthSq < 1`, centre `normalize(100)`, `zRot = nextDouble()*pi*2`, four corners from `rotateTowards(-center, up).rotateZ(-zRot)` (`J:client/sky/StarField.java:70-91`).
- Draw (`:285-319`): vanilla's STARS pipeline with a colour per vertex: `ColorTargetState(BlendFunction.OVERLAY)` (alpha blending as vanilla's "SRC_ALPHA, ONE: light adds up" per the comment at `:77-79`), no depth test, back faces culled, drawn on the main target with the stars' pose. The mixin runs at the TAIL of `SkyRenderer.renderStars(float, PoseStack)` (`J:mixin/D_LevelRendererSkyMixin.java:24-28`), only when vanilla draws stars (brightness > 0). Alpha scale: `skyAlpha = min(1, starBrightness/0.5)` (`:142`).
- Sky turn: `STAR_ANGLE` attribute / 360 = timeOfDay (`:262-266`).

### Horizon mapping

- Hook: `world_drawn` (client, both catalogs; `H:crates/horizon-client-render/src/gfx/native_render.rs:46-95`, doc `H:native-mods.md:4696-4712`). It runs after "the world is drawn (sky, terrain, entities, particles, weather and the world post pass)", before the hand and the HUD. There is no sky stage and no per-stage event: one call after the whole level (the doc line `hooks-compact.txt:81` says "once the whole level is drawn"). `WorldDrawn` gives `device`, `queue`, `encoder: &mut wgpu::CommandEncoder`, `color_view`, `color_format`, `size`, `depth: &wgpu::TextureView` (reversed, 0 is the sky), `depth_size`, `view_proj` and `jittered` (camera-relative), `camera: DVec3`, `quad_indices`. Raw wgpu is re-exported (`wgpu` in `H:mod_api.rs:60-ish`: `pub use crate::gfx::native_render::{..., WorldDrawn, WorldDrawnEvent, WorldWarp, WorldWarpEvent, wgpu}`).
- Depth: the field comment says depth is kept "only when a mod asked for it (`RenderNeeds::world_depth`) or something else kept it" (`native_render.rs:61-63`). So implement `render_needs` (collect, `H:native-mods.md:4714-4718`, `native_render_needs(needs)`; `needs.world_depth = true`).
- Own pipeline: a mod builds a render pipeline on `device` (WGSL, position + colour, blend SrcAlpha/One or the mod's choice, depth compare so that only pixels with depth == 0 (sky) pass, no depth write), uploads the triangles each frame with `queue`, begins a render pass on `encoder` over `color_view` with `depth` as read-only attachment, and draws. Per-vertex colour and alpha and additive blending are fully in the mod's hands this way. The matrix is `view_proj` with the translation removed (the stars are at infinity: use the rotation part only, times the star-angle rotation about the east axis), so the pose is rebuilt by the mod. Reference for the same maths in Horizon's own renderer: stars are drawn in the sky pass with `clip_from_sky = Projection * view * sky rotation` (`H:crates/horizon-client-render/src/gfx/stars.rs:39-47`, `stars.wgsl:2-11`; vertex data `star_triangles()` `stars.rs:12-45`, same seed 10842, same stream, same corner maths as `StarField.java`). So `StarField` ports to Rust one to one and star i is Horizon's star i. UNVERIFIED: that the corner winding after `rotateTowards` matches exactly; Horizon's code re-derives `right/up` by hand (`stars.rs:30-37`) instead of copying JOML, so the two could differ by a roll on the same star centre. The centres (what the chain uses) match; only the quad's spin could differ, and the mod's lines use centres.
- Alternative simple primitives: `world_primitives` (`H:native-mods.md:4784-4811`): `line(from, to, color, width)` (width in pixels), `box_outline`, `filled_box`, `beam`, `text`, with `.through_walls()` or depth-tested, "They blend over the world and write no depth", ARGB colour per primitive. A chain could be made of `line`s at 100+ blocks. No additive blending, no per-vertex alpha, width in screen pixels (not angular), and the primitives are in world coordinates so the mod must keep them 100 blocks from the camera each frame. Usable as a cheap version, not as a copy.
- Sky angle and brightness inputs: Horizon computes `sky.star_angle` and `sky.star_brightness` from the environment attributes (`H:crates/horizon-client-render/src/environment.rs:38-39, 1011-1012`, used at `gfx.rs:8409` and `H:crates/horizon-client/src/game/frame_geometry.rs:146`) but no helper exposes them (searched `star_angle`, `star_brightness` in `native_hooks.rs` and `mod_api.rs`: none). Available: `native_clock()` (`:11474`-ish, "its game time and time of day as the server last sent them ... advanced each tick between packets"), `native_day_time()` (`:10734`), `native_weather() -> (rain, thunder)` (`:10728`), `native_camera()` (`:11405`, `NativeCamera { position, partial_tick, yaw, pitch, view, projection, ... }` at `H:mod_api.rs:175-189`). The mod recomputes the star angle and the night brightness. UNVERIFIED: the exact function (overworld timeline `visual/star_angle`, `horizon-world/src/environment_attributes.rs:126`). Port it from `environment_timeline.rs` or read the angle from the vanilla 26.x timeline; test against a screenshot.
- The hook runs in the window's Game; the reading data comes from the item (section 5a) or from a `bmh:reading` mod message. A message avoids the 40-tick "fresh" trick: the server says "reading started now with these azimuths", the client starts the animation, no stack scanning.

PINNED variant (exact parity with vanilla's draw): the sky is drawn in `H:crates/horizon-client-render/src/gfx.rs:10660-10667`:
```
if !frame.title && frame.stars.is_some() && !pack_draws_sky && !own_stars {
    match self.core_sky.as_ref().filter(|c| c.draws_stars()) {
        Some(core) => core.draw_stars(&mut pass),
        None => self.stars.draw(&mut pass),
    }
```
An `AFTER_CALL` injection (horizon-mixins kinds: HEAD, BEFORE_CALL, AFTER_CALL, RETURN... `S/src/horizon-mixins/README.md:5-6`) after `self.stars.draw(&mut pass)` could add a draw in the same pass, with the same `clip_from_sky` (the `Stars` uniform, `stars.rs:39-47`), before terrain. It then pins one commit and gets terrain/cloud ordering exactly like vanilla. I did not write the injection.

### Verdict
HOOK (raw wgpu inside `world_drawn`, with `render_needs.world_depth`). PINNED only if exact layering under clouds/weather matters.

### Parity risks
- Layering: vanilla draws the stars before terrain, clouds and weather. After the world, with depth == 0 as the test, stars show over translucent things that did not write depth (clouds? weather? particles?). I did not find out which of Horizon's world passes write depth. UNVERIFIED. At full night with no rain this is invisible.
- Resolution Scale: `depth_size` can be smaller than `size` (`native_render.rs:61`): the mod must sample/compare at the right size.
- Shader packs that draw their own sky (`pack_draws_sky`, `own_stars`, `gfx.rs:10658-10666`): vanilla stars absent, the mod's stars still draw. Java mixin runs only when vanilla draws stars. A mod would check `render_needs`/shader state: not exposed, UNVERIFIED.
- HDR output: Horizon's stars have a special HDR path (`stars.wgsl:17-24`); the mod's pass has none.
- Star angle/brightness are recomputed: a small drift would detach the line from vanilla's stars. This is testable.
- The mixin in Java ran inside the sky pass; `world_drawn` runs after the post pass if it applies ("the world post pass" is before the call, `native-mods.md:4701`): bloom, fog or tonemap no longer touch the mod's lines. Slightly different glow.

---

## 3. The spyglass line name

### Behaviour spec
`J:mixin/K_GuiSpyglassMixin.java` (61 lines; I read its purpose from `J:client/sky/StarReadingRenderer.java:211-242` and the class comment, not the mixin body): through a spyglass, draw the name of the line whose last (lowest) star is within `AIM_DEGREES = 0.8` of the camera's forward vector (dot product against the star's sky direction rotated by the sky turn), nearest first, only for named lines still in the sky (`:217-241`). Hooked into `Gui.extractSpyglassOverlay` (see `J:.claude/CLAUDE.md` "K_GuiSpyglassMixin"). Text style is whatever the mixin draws: UNVERIFIED (not read).

### Horizon mapping
- Drawing: `gui_render_post` or `hud` (`H:native_hooks.rs:2516`, "The HUD (not drawn under F1)", draws with `native_fill`/`native_text`). The vanilla spyglass overlay is not a GUI layer here: `native_gui_layer_pre` lists the layers (`crosshair, hotbar, effects, boss_overlay, vehicle_health, player_health, contextual_info_bar, selected_item_name, sleep_overlay, overlay_message, title, scoreboard_sidebar, subtitle_overlay, chat, tab_list`) and says "The camera overlays and the debug screen are not layers here" (`H:native_hooks.rs:2532-2552`). So there is no `gui_layer_post` for the spyglass; draw in `hud`.
- Is the player scoping? Internal: `Game::scoping()` is `self.using ... item == "minecraft:spyglass"` (`H:crates/horizon-client/src/game/fov.rs:38-42`). No hook or helper exposes `using`. I searched `spyglass`/`scoping` in `native_hooks.rs`, `mod_api.rs` and `native_client_services.rs`: only zoom comments (`native_hooks.rs:2849`, `:12633-12636`). `NativeClientState` has no use-item field (`native_client_services.rs:29-51`). Approximation: a spyglass in either hand (`native_living_hand_item`, `H:crates/horizon-client/src/game/native_living_refs.rs:107`) and the use key held (`native_mapping("key.use")`, `H:docs` mod-code table: "A vanilla control (key.forward, key.jump, key.attack…): whether it is held") with `screen == Playing`. The server-side release hook `native_release_using` etc. are server only.
- Camera forward: from `native_camera()` (`yaw`, `pitch`, `H:mod_api.rs:175-181`) or `view` matrix. During scoping Horizon also zooms (spyglass wins over a mod's `native_zoom`, `native_hooks.rs:2849`); `fov_degrees` is in `NativeCamera`.
- PINNED exact version: inject `RETURN` into `Game::scoping` (`fov.rs:38`) to record the answer, or inject into the spyglass overlay draw. Either pins a commit.

### Verdict
PARTIAL: HOOK for the drawing and the aim maths; PINNED for the exact "is scoping" flag; otherwise approximate.

### Parity risks
- The approximation says "scoping" while the use key is held with a spyglass but the game may not be scoping (item on cooldown, swapped away mid-click).
- Text style and position of the name: depends on the Java mixin body I did not read.
- No spyglass overlay layer: the name cannot be inserted "inside" the overlay's draw order, only on top.

---

## 4. The amulet needle

### Behaviour spec
`J:client/AmuletCompass.java`: numeric item property `bmh:north` (`:27`) = vanilla's compass state (`CompassAngleState`, wobble included) aimed at the block `NORTH_DISTANCE = 1.0E6` north of the owner: `targetX = floor(x)+0.5`, `targetZ = floor(z - 1e6)+0.5` (`:64-66`). In non-Overworld dimensions there is no target and the needle spins randomly (`:59-61, :85-94`). For the local player the wobbler is used (`:71-77`), for any other owner the direct angle (`:79`). Registered on all loaders by `J:mixin/M_RangeSelectPropertiesMixin.java:31-35`. Model `W/common/src/main/resources/assets/bmh/items/navigator_amulet.json`: `minecraft:range_dispatch`, `property: bmh:north`, `scale: 32.0`, 32 entries with thresholds 0.0, 0.5, 1.5, ..., each model `bmh:item/navigator_amulet_NN` (I read the first 6 entries and the line count, 244).

### Horizon mapping
- The hook `item_model_property` is not suitable: it is conditional only (boolean `value`), "Vanilla properties do not enter the hook", and its input is `StackView { item, count, patch, bundle_selected }` with no holder (`H:native-mods.md:39-59`; `H:crates/horizon-assets/src/item_model.rs:962-968`). The needle depends on the holder's yaw and position, so a boolean tree driven by the local player's yaw would be wrong for any other holder and for dropped items/frames. Also the doc says "Bundles need hooks: 385 once this pending hook is stamped" (`:58`): it is pending in the docs although both catalogs list it.
- Better: no code. Horizon resolves `minecraft:compass` as a range property (`H:crates/horizon-assets/src/item_model.rs:882-892`: `"minecraft:compass" => Self::Compass { wobble, target: none|lodestone|spawn|recovery }`) with the exact vanilla function `compass()` (`:1582-1650`): for `target: lodestone` it reads the stack's `minecraft:lodestone_tracker.target` (`{dimension, pos}`) (`:1588-1604`), returns the random-spin branch when the dimension differs from the level's (`:1613-1632`), otherwise the angle with the local-player wobble branch and the other-owner direct branch (`:1637-1650`), i.e. the same state machine the Java class copies. The tracker's `tracked` flag is not read.
- So the item gets `minecraft:lodestone_tracker = {target: {dimension: "minecraft:overworld", pos: [0, 64, -30000000]}, tracked: false}` as a default component, and its model JSON uses `"property": "minecraft:compass", "target": "lodestone"` (also `wobble`) in the same `range_dispatch` with `scale: 32`. A default component is set with `native_modify_default_components(item, components)` (`H:native-mods.md:5081-5099`, free function bundle target `horizon_world::item_defaults::native_modify_default_components`; "Client and server each apply their own mods, so both need the mod") or in the content mod's item definition if it supports components (the content/items group owns that: I did not check `docs/api/content.md`).
- Difference from the Java: a fixed target at x = 0, z = -30 000 000 instead of "1e6 north of the holder". At holder x = 20 000 the needle is off by atan(20000/3e7) = 0.04 degrees, at x = 1 000 000 by 1.9 degrees; a frame is 11.25 degrees wide, so invisible except near the world edge. `item_model.rs:1613` adds `(target - owner)` so it is relative to every owner.

### Verdict
HOOK (data only, no code), given that the item can carry a default `minecraft:lodestone_tracker`.

### Parity risks
- Needle error grows with |x|: under 2 degrees to x = 1 000 000.
- The lodestone tracker would also let a vanilla anvil/crafting rule treat the amulet as "a compass with a tracker" (e.g. `H:item_model.rs:1224`: `stack.item == "minecraft:compass" && component(...)`, a foil rule; since the item id is not `minecraft:compass` it does not apply).
- Vanilla's own "lodestone tracker" tooltip lines are only for the compass item.

---

## 5. The amulet item on the server

### 5a. Behaviour spec: use, use on, readings, charges (`J:items/NavigatorAmuletItem.java`, `J:handlers/AmuletReading.java`)

`useOn` (`NavigatorAmuletItem.java:60-88`), in order:
1. If no player, or not `isSecondaryUseActive()` (sneak), or the clicked block cannot hold a point (`AmuletPoints.canHold`: tag `BEDS` or `Blocks.LODESTONE`, `AmuletPoints.java` `canHold`): return PASS (falls through to `use`).
2. Server: if the level is the Overworld: find the existing point (`AmuletPoints.at`), send `AmuletPointSignal.open(player, pos, name|null, used, max, bed)` and add a cooldown of 20 ticks on the stack (`POINT_COOLDOWN_TICKS`, `:46`). Else send the overlay message `bmh.point.overworld`.
3. Return SUCCESS (both sides decide the same).

`use` (`:92-137`):
1. Client: return CONSUME with no decision (`:98-100`) so that the off-hand item is not also used.
2. Sneaking: send `AmuletPointSignal.list(player, points, max)`, cooldown 20, SUCCESS_SERVER (`:103-109`).
3. `hiddenReason = AmuletReading.hiddenReason(player)`; if not null send overlay message with that lang key, FAIL (`:111-117`).
4. `AmuletReading.refill(stack, level)`; if charges <= 0 and not creative: overlay `bmh.amulet.empty`, FAIL (`:119-125`).
5. `AmuletReading.record(player, stack)`; play `amethyst_block.chime` (1.0, 1.0, PLAYERS) at the player; overlay `bmh.amulet.reading` with block coordinates `(x,y,z)`; cooldown 40 ticks; SUCCESS_SERVER (`:127-136`).

Sky rule `skyHiddenReason(player, prefix)` (`AmuletReading.java:111-132`), in order, each returns `prefix + ".dimension|.rain|.day|.roof"` (prefix `bmh.amulet.hidden`): not Overworld; `level.isRaining()`; `!level.isDarkOutside()`; `!canSeeSky(blockPos(eyePosition))` or `sightBlocked` (Blindness, Darkness, eye in water or lava, eye in powder snow, `:138-145`).

`record` (`:210-251`): for each point: `azimuth(player.x, player.z, point.x+0.5, point.z+0.5)`; then the world spawn last with name "" and `spawnIndex = azimuths.size()` before adding; writes into `custom_data`: `bmh_reading{azimuths: List<Float>, names: List<String>, spawn: int, time: long gameTime}`; unless creative also `bmh_starlight = charges-1` and `bmh_night = nightIndex(overworldClockTime)`. Awards advancement `navigation/stargazer`. `azimuth = atan2(toX - fromX, fromZ - toZ)` in degrees, normalised to [0, 360), 360 collapsed to 0 (`:67-74`).

Charges: `MAX_CHARGES = 3` (`:44`); `charges(stack)` = `bmh_starlight` clamped 0..3, default 3 (`:148-157`). `nightIndex(dayTime) = floorDiv(dayTime + 24000 - 12000, 24000)` (`:90-93`). `refill`: if `bmh_night` exists and differs from the current night index, set `bmh_starlight = 3` and `bmh_night = night` (`:185-204`). `inventoryTick` once per second (`gameTime % 20 == 0`): `dropLegacyData`, `refill` (`NavigatorAmuletItem.java:140-147`). Item bar always visible: width `round(13 * charges / 3)`, colour 0xB58CFF (`:160-177`). Tooltip lines `bmh.amulet.starlight`, `.tooltip.read`, `.tooltip.home`, `.tooltip.list` (`:149-158`). No durability, no enchantments (`dropLegacyData` removes Unbreaking and Mending, `AmuletReading.java:163-182`).

### 5a. Horizon mapping
- Click interception, server side, for every player (the helpers read the acting one), asked once per hand:
  - `right_click_block` (s!, cancellable; `H:native_hooks.rs:8452-8463`: "where `ServerPlayerGameMode.useItemOn` starts, before the block's own use (a chest, a door, a lever) and before `native_use_item_on`. Set `canceled` to stop the click"). Not needed for the amulet: sneaking with an item in hand skips the bed in vanilla (UNVERIFIED that Horizon implements `shouldSneakBypassUse`; `use_item_on` doc says it fires "once the block itself let the click through (a chest still opens)", so if Horizon lets a sneaking click reach the bed, the player would also try to sleep. Test it).
  - `use_item_on` (`native_use_item_on(&mut self, stack: &ItemStack, pos: [i32;3], face: [i32;3], claimed: &mut bool)`, `H:native_hooks.rs:8156-8165`; both catalogs; contract `cancel` with out `claimed`): "A player uses `stack` (right click) with `self.hand` on the block at `pos` ... Set `claimed` to take the use: the hand swings and neither the item's `useOn` nor its `use` runs. Server-side ... an item on cooldown is never used." Use it for step 2 (sneak + bed/lodestone): claim when `stack.item == "bmh:navigator_amulet"` (item id is the mod's), sneaking, and the block is a bed or lodestone.
  - `use_item` (`native_use_item(&mut self, stack: &ItemStack, claimed: &mut bool)`, `:8197`, s!; "in the air, or on a block once `native_use_item_on` let it through ... Each click reaches it once. Set `claimed` to take the use: the hand swings and the item's own use doesn't run"): for list (sneak) and reading. Because use_item_on does not claim for a non-bed block, the non-bed click falls through to `use_item` exactly as Java falls from `useOn` PASS to `use`.
  - "an item on cooldown is never used" matches Java cooldowns: add a cooldown (20 or 40 ticks): I did not find a Horizon cooldown helper in the 177 helpers (searched names `cooldown`: none in the helper list). UNVERIFIED: that a mod can set an item cooldown. If absent, the mod keeps its own per-player timer (game ticks) and the hand "swing"/use animation difference is invisible.
  - Which hand: neither hook passes the hand (the doc says "with `self.hand`"). Use `native_held()` (`H:crates/horizon-client/src/game/native_server.rs:1253`, `(item, count, plain)`; "On a client, the player's own") or `equipment(slot)`. UNVERIFIED how the typed `UseItemEvent` carries the hand.
- Sneak: `sneaking()` on `Host`: "`sneaking()` is `isSecondaryUseActive` in a use hook" (`H:native-mods.md:5517-5518`); source `native_sneaking` `H:crates/horizon-client/src/game/native_host_bridge.rs:118-120`; not in the 177-helper catalog list (searched `sneak|shift` in both JSONs: only the hook `player_toggle_sneak`); it is documented as part of `Host`. UNVERIFIED which Nightly has it.
- Sky rule helpers: dimension (`native_acting_player().dimension`, `H:native_server.rs:873-890`, or `dimension` helper), rain (`native_weather() -> (rain, thunder)` `H:native_hooks.rs:10728`; Java `level.isRaining()` is `rainLevel > 0.2`: threshold UNVERIFIED), `isDarkOutside` (sky darkness from the day time and weather: the helper `light(pos) -> (block, sky)` `:10717` reads block light and sky light, not the sky darkening; compute from `native_day_time()` `:10734`; vanilla `isDarkOutside` is `skyDarken > 4`-ish via `getSkyDarken(1)`; I did not check Horizon's version: UNVERIFIED), `canSeeSky(eyePos)` (there is `surface_heights(...)`/`read_columns`/`level_height` helpers in the list; the heightmap test `eye.y >= surface height at x,z` approximates `canSeeSky`: UNVERIFIED which of them is exact), blindness/darkness effects (effect helpers exist in the catalog family `NativeEffect`, not read), eye-in-fluid/powder snow (`level_block(...)` at the eye position, `level_block_state`). Each has a catalog helper in the family but I did not read each doc. Overall buildable; exact `isDarkOutside` threshold is the one to check.
- Reading storage on the stack. `use_item` and `use_item_on` hand `&ItemStack` (immutable), so the stack cannot be changed there. A mod changes a carried stack with `native_carried_stack(&mut self, slot: i32) -> Option<&mut ItemStack>` (`H:native_hooks.rs:10648`; "The stack the acting player carries in `slot`, by `native_inventory_tick`'s numbers ... Change it as you like ... the player's client gets the change. API 39"): slots 0..35 inventory, 100..103 armour, -106 off hand (`:11958-11964`). The hook does not tell which slot was used. Plan: in `use_item`, record "pending reading for player P" (azimuths, names, time, new charges); in the next `inventory_tick(slot, selected)` (`H:native_hooks.rs:9346-9356`, every stack every tick for every living player, `selected` marks the held slot) apply it to the held amulet (main hand first, then off hand -106, as vanilla tries the main hand first). One tick of delay.
- Data layout in the stack: `ItemStack.data()`/`StackData::set_other(id, Option<Tag>)` (`H:crates/horizon-items/src/item_data.rs:197`) with `"minecraft:custom_data"` (a compound; reader `ItemStack::custom_data()` `item_data.rs:426`, used by Horizon itself for `horizon_fluid`/`horizon_energy`, `H:native-mods.md:1276, 1611`: "The amount lives in the stack's `minecraft:custom_data`, so it is saved and sent with the stack"). `native_set_component` (`H:crates/horizon-client/src/game/native_components.rs:48`) only accepts registered custom component types, not custom_data; so write custom_data through `StackData` directly. `ItemStack`, `StackData` are exported (`H:mod_api.rs:36-41, 52`).
- Client reads the stack through the same data (it is synced). The sky reading can use it or a message (section 2).
- Item bar and tooltip: client hooks `item_bar(&self, stack, bar: &mut Option<(i32, u32)>)` (`H:native_hooks.rs:8132`: "width 0..=13, colour rgb ... or `None` to hide it", client, "Asked each frame a stack is drawn") and tooltip lines are other hooks (`item_tooltip` family: not read; the tooltip is the items group's).
- Per-player data: `bmh_starlight`/`bmh_night` on the stack, nothing else.
- Advancement `navigation/stargazer`: the advancements group.
- Level sound: `play_sound` on `Host` (`H:native-mods.md:5497`, "`play_sound`, `send_particles` ... Work in the level in place, from any server hook") for `minecraft:block.amethyst_block.chime` at volume 1.0 pitch 1.0.
- Overlay message to the player: `action_bar(...)` helper (in the list: `action_bar`), translatable keys need the lang file of the content mod (items group).

### 5a. Verdict
HOOK. Open: item cooldown helper (UNVERIFIED), exact `isDarkOutside`/`canSeeSky`/`isRaining` equivalents, `sneaking()` availability in the pinned Nightly, and the one-tick deferral to write the stack.

### 5a. Parity risks
- Reading appears on the stack one tick after the click.
- No vanilla cooldown overlay on the item unless a cooldown helper exists.
- Charges tooltip and bar live on the client hooks; the stack needs to be synced to read them (it is).

### 5b. Points saved on the player (`J:handlers/AmuletPoints.java`)

Spec:
- Per player list of `Point(name, BlockPos)`; saved key `bmh_points` as a list of compounds `{name: string, x: int, y: int, z: int}`, always writing every field (`:63-72, :195-203`); legacy `bmh_home` (compound x,y,z) loads as a point named "Home" (`:205-215`); copied across death by `C_ServerPlayerMixin`.
- Cap: `maxAmuletPoints` from config, 1..16, default via `BMHConfigLimits` (`J:utilities/BMHConfigLimits.java:15-16`; `AmuletPoints.max()` `:98-103`). A player above the cap keeps their points but cannot add one.
- `MAX_NAME_LENGTH = 24`; `cleanName`: drop `§` and the next character, drop ISO control characters, trim, cut to 24, trim again (`:178-191`).
- A bed point is anchored at the HEAD half: `anchor` moves from the FOOT to `pos.relative(facing)` (`:123-128`); `at()` matches either half (`:131-142`).
- `put`: rename or add; awards `navigation/named_place`, and `navigation/own_sky` at 3 or more points (`OWN_SKY_POINTS = 3`, `:151-158`). `remove`, `removeAt` as described.
- `worldSpawn(player)` = `level().getRespawnData().pos()` (not a point, not counted).

Mapping: `AttachmentType<Vec<Point>>` on `Holder::Player(uuid)` with a codec and `copy_on_death: true`, `sync: false` (the screen gets its list by message) (`H:native-mods.md:5402-5462`: "`Holder::Player(uuid)` ... `{value: <compound>, copy_on_death: 1b, sync: 1b}`"; "`InvalidCopyPolicy` (death-copy without a codec)"). `get` clones the value and inserts the default on first use. The Java NBT layout need not be kept: no old Horizon worlds exist. Server-side only; works in a server hook with `host: &mut dyn Host`.
- Bed detection: block names `minecraft:<colour>_bed` (16 beds; the Java `BlockTags.BEDS`), `block_property(pos, "part")`/`"facing"` (`H:native_hooks.rs:11425-11437`, `native_block_property`) for the head; `level_block`/`level_block_property` for any level. Whether a tag helper helps: `block_name_in_tag(block, tag)` is in `Host` (`H:native-mods.md:5503`, from `server_block_in_tag`), so `minecraft:beds` is checkable.
- World spawn: I did not find a spawn-position helper in the 177 names (searched list for `spawn`: `spawn` is the mob helper family). The `level` host family (`level_*`) has `native_level_*` helpers (`H:crates/horizon-client/src/game/native_level_world.rs:20-100`: `native_level_seed`, `native_level_difficulty`, `native_level_game_rule`, `native_level_set_game_rule`); `LevelRef` respawn data exists internally (`item_model.rs` uses `level.respawn`). UNVERIFIED: a mod-facing helper for the world spawn point.

Verdict 5b: HOOK; open item: world spawn helper.

### 5c. Naming screen, list screen, signals, command

Spec (`J:client/amulet/PointScreen.java`, `PointListScreen.java`, `PointSignal.java`, `J:handlers/AmuletPointSignal.java`, `AmuletPointCommand.java`):
- Server opens screens with a title packet, key `bmh.point.open` args (x, y, z, name or "", used, max, "bed"|"lodestone") (`AmuletPointSignal.java:50-62`) or `bmh.point.list` args (max, count, then name, x, y, z per point) (`:30-48`).
- `PointScreen` (166 lines): title key `bmh.point.screen.new` or `.edit`, subtitle `bmh.point.screen.bed|lodestone`, an EditBox 200 wide max 24 chars prefilled with the name (`:init`), buttons Save / Remove (only when editing) / Cancel (64 wide, 4 gap); Save is disabled when the point is new and `used >= max` or `cleanName` is empty; count text `bmh.point.screen.count` (used + 1 if new, max) or `bmh.point.full`; Enter confirms; dim layer 0x80000000; does not pause; Save sends `/bmh point save x y z <clean name>`, Remove `/bmh point remove x y z`.
- `PointListScreen` (153 lines): a paged list (`<` `>` buttons, Done) with a Remove button per point; Remove sends `/bmh point delete x y z`; the server sends the list again after each removal (`AmuletPointCommand.delete`).
- Server checks in `AmuletPointCommand.allowed` (`:160-178`), in order: an amulet in either hand (`bmh.point.no_amulet`); Overworld (`bmh.point.overworld`); alive, not spectator, within 8 blocks of the block (`MAX_DISTANCE = 8.0`, `:?`), and the block can hold a point (`bmh.point.too_far`); at most one change per 20 ticks (`MIN_TICKS_BETWEEN_CHANGES`). `save`: loaded block pos, `cleanName` empty gives `bmh.point.no_name`; not a rename and `used >= max` gives `bmh.point.full`; then `put`, chime at pitch 0.7, overlay `bmh.point.renamed|saved`. `remove`: overlay `bmh.point.removed`. `delete`: needs only an amulet in hand, no block.

Mapping:
- Server to client: `native_net_send_to_player` (channel `bmh:point`), carrying the same fields as bytes (`H:native_net.rs:174`); client to server: `native_net_send_to_server(&mut self, channel, data)` (`:230`, "An error when the server doesn't carry mod messages (a vanilla server), and until the player has joined"), heard by `net_from_client(player, channel, data)` (`H:native_hooks.rs:8562-8567`: "treat `data` as the player's own bytes, checked by nothing. Runs in the tick after it arrives, with no player's state swapped in") so the server re-checks everything, exactly the Java stance. The `/bmh point ...` command and `J_CommandsMixin` are not needed. With no player state swapped in, the acting-player helpers (`held`, `native_acting_player`) do not apply there: the check "amulet in either hand" needs `equipment`/`player_stacks(uuid)` (`H:native-mods.md:4684-4686`, "`native_player_stacks(uuid)`: the player's 36 inventory slots", it refuses a player whose state is swapped in, the message hook is not that case) and the position via `entity_position`/`entity_*` handle helpers. UNVERIFIED: exact helper to read a named player's position and hand items inside `net_from_client`; the `entity` helper family (`entity_position`, `entity_level`, in the helper list) takes an `EntityRef` that I did not trace to a player UUID.
- Screen: `native_mod_screen_open(mod_id)` (`H:native_hooks.rs:11302`), then draw in `screen_render_post` (`native_screen_render_post(&self, batch, screen)` `:3739-3746`: "draw over it with `native_fill`, `native_sprite` and `native_text`, `screen.rect` being its container panel (or the whole window)"; every mod screen has `screen.id == "mod"`, check `native_mod_screen()` is yours), clicks in `screen_mouse_button_pressed_pre` (`:3767`, `screen.mouse` for position), keys in `screen_key_pressed_pre` (key name `key.keyboard.enter`, `backspace`; "a key with no name fires nothing") and characters in `screen_char_typed_pre` (`H:native-mods.md:1133`: "control characters (Enter, Backspace, Tab, Delete, Escape, a Control letter) are keys and fire nothing"). Buttons: `native_gui_nine_slice(batch, sprite, rect, color)` with vanilla button sprites (`H:native_client_services.rs:431`) and `native_text`/`native_cursor()` for hover. The mod screen "doesn't pause a single-player game" and Escape closes it, matching Java (`isPauseScreen` false).
- No text input widget for mod screens: `NativeTextField` (`H:crates/horizon-client/src/game/native_text_field.rs:13-90`) is for existing text boxes ("Focused existing text boxes, without exposing a mutable UI", `:1`). The mod writes its own line editor: insert at cursor, Backspace/Delete, Left/Right/Home/End, selection, caret blink, max length 24, `cleanName` filtering. No clipboard helper (searched `clipboard` in `native_hooks.rs`, `native_client_services.rs`, `mod_api.rs`: none), so Ctrl+V / Ctrl+C / Ctrl+A do not work. IME text arrives through the same char hook ("The input method's committed text fires the character hook too", `native-mods.md:1133`).
- The docs list "`screen_key_pressed_pre`, `screen_char_typed_pre`, `screen_mouse_scrolled_pre`" among hooks "not in the trait yet" for the typed API (`native-mods.md:5522-5524`), while the catalog gives them `mod_api` contracts (`native-hooks.json`: `{'contract': 'cancel', 'out': 'canceled'}`). The doc looks stale; UNVERIFIED which is right. If the typed trait lacks them, the mixin-helper way (HEAD of `native_screen_key_pressed_pre`) works on every release (`S/horizon-docs/mod-code.txt`: "Mixin helpers ... work on every Horizon release").
- A mod screen opens only "over the world with no overlay up" (`native_mod_screen_open` `:11302-11330`): fine, the amulet use happens in the world.

Verdict 5c: PARTIAL. HOOK for message flow and drawing; the hand-written text field and the missing clipboard are Horizon limits; no GAP for function.

Parity risks 5c: caret/selection feel differs from vanilla EditBox; no clipboard; screen look is the mod's own drawing (button sprites from `gui/sprites/widget/button*` through `gui_nine_slice`); no narrator support for the mod screen.

---

## 6. The Create New World button

### Behaviour spec
`J:mixin/CreateWorldGameTabMixin.java`: at the end of the constructor of `CreateWorldScreen$GameTab`, add a `CycleButton<Boolean>` "Balanced Hardcore: ON/OFF" (`bmh.createWorld.balancedHardcore`, tooltip `.info`), 210x20, in the next free row of the tab's one-column grid (`:34-55`). Initial state = any of the six BMH rules is true in the creation state (`isBalancedHardcoreEnabled` `:57-60`, `BMHGameRules.ALL_RULES`). Click: copy the creation state's rules, set all six to the value, `uiState.setGameRules(copy)` (`:62-71`). A listener keeps it in sync with the More tab and greys it out when the world is Hardcore or Debug (`:46-49`). The six rules: `random_death_spawn_point`, `display_text_on_respawn`, `food_and_health_regen_on_wake_up`, `only_carry_one_eternal_item_type`, `damage_eternal_item_when_dead`, `remove_coordinates_in_survival` (`J:utilities/BMHGameRules.java:40-46`; defaults per rule are in that file; `remove_coordinates_in_survival` default true at `:37`).

### Horizon mapping
- Horizon exposes the title's create screens to the screen hooks (API 326, `H:native-mods.md:1653`): ids `world_create`, `world_create_flat`, `world_create_buffet`, ...; `native_screen_render_post` "fires after the title draws its screen" with the whole window as `rect`; the `_pre` input hooks (mouse button, key, char) "fire on it and can cancel it as on any screen, so a mod draws its own button or browser over Create New World and takes the clicks". So a button can be drawn and clicked (`screen_render_post`, `screen_mouse_button_pressed_pre`). `screen_init_post` fires "observed opening or changing rectangle" (`H:native_hooks.rs:3713`) with the same `NativeScreenView` (no widgets). `menu_button`/`menu_button_v2` are for mod container menus on the server (`H:native-mods.md:849`), not for this.
- What is missing: `NativeScreenView` for title screens has "no menu, no title and no slots" (`H:mod_api.rs:322-352`), so the mod does not know which tab is open, where the Game tab's last row is, whether Hardcore/Debug is selected, or the current rules. I searched for a create-screen state helper (`world_create`, `CreateWorld`, `new world`) in `native_hooks.rs` and `mod_api.rs`: only doc mentions. There is no way found to write the new world's rule set from the create screen.
- Fallback that works: the button is a mod setting (`native_setting`/`native_set_setting`, `H:native_hooks.rs:11562`) shown over the screen at a fixed place (e.g. the lower right), and when the new world first opens the server applies the six rules with `native_level_set_game_rule(level, rule, GameRuleValue)` (`H:crates/horizon-client/src/game/native_level_world.rs:80-95`, API `level_world_setters`; takes names like `bmh:...`: depends on the rules group registering them). The first-open moment: `world_lifecycle` `WorldOpenReady` (`H:native-mods.md:394`, `hooks-compact.txt:5`) plus a per-world marker (an attachment on `Holder::Level`, `native-mods.md:5430-5434`) or `level_game_time == 0`. The setting is in the client's mod data (`native_data_dir`, `H:native_hooks.rs:11550`), but a single-player server runs in the same process; a dedicated server has no create screen (no issue).
- Effect: the rules are the default for every world the player creates after switching the setting, not per-creation-screen; it does not appear in the More tab until after creation; the button cannot grey out in Hardcore (the mod can still ignore it when the world's `hardcore` flag is set).

### Verdict
PARTIAL. HOOK for a button and its click; GAP for placement in the Game tab, state sync with the More tab, grey-out, and baking the rules into the creation state. The fallback gives the same outcome in the saved world.

### Parity risks
- Button is not in the Game tab; it floats over the screen (or on a fixed rect) and is not part of tab navigation or focus order; no narrator text.
- Not synced with edits in the More tab; not greyed out in Hardcore.
- Rules apply at first open, so the world list/create preview does not show them.

---

## 7. `client/config/ConfigScreen.java`

### Behaviour spec
Three integer fields: `minDistanceFromBed` (5000 to 200000), `maxDistanceFromBed` (5000 to 200000), `maxAmuletPoints` (1 to 16) (`J:utilities/BMHConfigLimits.java:9-16`; defaults 5000, 60000). Digits only, max length 6 (`ConfigScreen.java field`), validation: all in range, min <= max, errors `bmh.config.outOfRange`, `bmh.config.minAboveMax`; Done writes via `Services.PLATFORM.saveConfig(min, max, points)` and Cancel closes (`:27-48`). It edits the local file only: a dedicated server has its own (class comment). Reached from the mods list (Forge, NeoForge via `IConfigScreenFactory`, `neoforge/.../NeoForgeClientEvents.java`) and Mod Menu (Fabric).

### Horizon mapping
- `settings` hook (collect, `buttons`): `native_settings(&mut self, mod_id: &str, buttons: &mut Vec<String>)` "The buttons of mod `mod_id`'s settings page, under its keys: push their labels when it is yours. Asked again each time the page opens and after each click." (`H:native_hooks.rs:3168-3173`); `settings_click(mod_id, button: usize)` (`:3174`). The page is a list of labelled buttons only (also `H:horizon-docs/mod-code.txt` worked example: "Shown: ON/OFF").
- No text field on a settings page. Build each number as stepper buttons ("Min distance: 5000", click cycles through presets or steps by 5000/10 000; for points 1..16 a cycle button), validation min <= max by clamping in the click handler. Persist with `native_set_setting(mod_id, key, value)` (a file in the mod folder `horizon-mods/.horizon-mod-data/<id>/<key>`, `H:docs` mod-code table). A mod screen with real fields cannot be opened from the Options page: `native_mod_screen_open` is false "while another screen is open" (`native_hooks.rs:11302-11310`).
- Also: the real config belongs to the server (the Java comment says so). The server reads its own values; in Horizon the config files API (`NativeConfig`, `native_config_dir`, `H:mod_api.rs:40-48`: `NativeConfig`, `NativeConfigFile`, `NativeConfigKind`) is the config group's; this row is only the screen.
- A `/command` or a client command is another way: `native_command` (`H:docs mod-code`) lets `/bmh config min 5000`.

### Verdict
PARTIAL: HOOK for buttons that cycle values; the numeric text field is not available there.

### Parity risks
No free typing of arbitrary numbers; fewer values reachable with steppers; no inline error line (the label can show it).

---

## 8. The totem animation item

### Behaviour spec
`J:mixin/I_ClientPacketListenerTotemMixin.java`: `ClientPacketListener.findTotem(Player)` RETURN, cancellable. If a hand holds an item with `DEATH_PROTECTION` keep vanilla's answer. Else the first inventory stack that is a `TotemOfEternityItem` is returned as a copy; else a Dormant Totem in a hand becomes a new Totem of Eternity (moon ritual).

### Horizon mapping
- The client picks the item in `Game::totem_event` / `find_totem` (`H:crates/horizon-client/src/game/totem.rs:142-161, 166-170`): `let totem = body.protector.unwrap_or_else(|| self.find_totem());` then `self.item_activation.activate(totem, ..)`; `find_totem` reads "the first hand's item with `death_protection`, else a new totem". There is no hook for it (searched `totem` in `hooks-compact.txt`: only `player_totem` (s, listen only) at line 293 "NeoForge's LivingUseTotemEvent can cancel the save; this hook only listens", and `native_player_totem` `H:native_hooks.rs:9147-9158`, server).
- Data-only way: not possible because the totem sits in the inventory, not in a hand, and the stack lookup is hands-only.
- PINNED: horizon-mixins `RETURN`/`MODIFY_EXPRESSION_VALUE` into `Game::find_totem` (`totem.rs:166`) to return the inventory's Totem of Eternity. Pins one commit.

### Verdict
PINNED (otherwise the animation shows the vanilla totem of undying: cosmetic only).

### Parity risks
Without the pin: the animation shows a Totem of Undying. The server-side half of the save belongs to the totems group.

---

## Open questions
1. Does `gui_render_post` fire while the level-loading screen is up? The doc says not (`H:native_hooks.rs:2519`); `game.rs:9573` looks unconditional. Needs one run of `crates/horizon-client/examples/native_gui_overlay_probe.rs` across a respawn.
2. Which Horizon world passes write depth (clouds, weather, particles)? Decides whether `depth == 0` hides the star reading under clouds like vanilla.
3. How exactly does Horizon compute `visual/star_angle` and `visual/star_brightness` for the Overworld, and can a mod read them (no helper found)? A mismatch detaches the chain from the stars.
4. Is there a mod-facing way to set an item cooldown, to read the world spawn point, to know the acting hand in `use_item`, and to read another player's position/hand items inside `net_from_client`?
5. Does Horizon let a sneaking click on a bed with an item reach `use_item_on` without opening the bed (`shouldSneakBypassUse`)?
6. Is `sneaking()` in the Nightly the mod is pinned to (helper `native_sneaking`, not in the 177-name helper list of either catalog)?
7. Does the typed `Hooks` trait carry `screen_key_pressed_pre` and `screen_char_typed_pre` (docs `native-mods.md:5522` say "will join a later slice", catalog says contract `cancel`)?
8. Can a content mod set a default `minecraft:lodestone_tracker` on its own item in the item JSON (data only), or does it need `native_modify_default_components`?
9. Cost of several thousand `native_fill` calls per frame for the dissolve.
10. I did not read `J:client/sky/StarChain.java`, `StarTrace.java`, `J:mixin/K_GuiSpyglassMixin.java`, `PointListScreen.java` in full or `J:mixin/C_ServerPlayerMixin.java`; porting needs them.
11. `MOD_IMAGE_SIZE` value (not needed for this group).

## Step log
1. Read the common brief; ran the tree guard: `git -C W rev-parse --show-toplevel` printed W, `branch --show-current` printed `horizon-26.3`.
2. Read the small Java files of the cinematic (JourneyCinematic, JourneyMusic, F_GuiTitleMixin, F_LocalPlayerRespawnMixin, F_MinecraftOverlayMixin, JourneySignal) and all of JourneyOverlay.
3. Listed hooks in both catalogs (`hooks-compact.txt`, `native-hooks.json`, Nightly JSON) and diffed names: all relied-on hooks are in both, `world_hold`/`world_warp` main only.
4. Read the docs for `gui_render_post`, `gui_layer_*`, `screen_*_pre`, `client_packet`, `net_from_server`, and the helper bodies for `fill`, `quad`, `text`, `gui_paint`, `mod_image`, `play_local_sound`, `mod_screen_open`.
5. Traced Horizon's death screen: `Screen::Dead` id, `death_click`, `death_action`, `respawn_player`, `remote_respawn` (ClientCommand action 0), the immediate-respawn call at `remote.rs:4634`, the respawn loading screen path.
6. Read `native_render.rs` (`WorldDrawn`), the "Render hook points", "World warps", "World primitives", "Holding the world's picture" doc sections, and Horizon's own sky pass (`gfx.rs:10600-10670`, `stars.rs`, `stars.wgsl`, `core_sky.rs`).
7. Read the Java sky renderer (StarReadingRenderer, StarField, D_LevelRendererSkyMixin).
8. Read NavigatorAmuletItem, AmuletReading, AmuletPoints, AmuletPointSignal, AmuletPointCommand, PointScreen, PointSignal.
9. Checked Horizon's use hooks (`use_item_on`, `use_item`, `right_click_block`), `carried_stack`, `inventory_tick`, StackData/custom_data, attachments, `native_sneaking`.
10. Checked the item model resolver (`item_model.rs`) for `minecraft:compass` and the lodestone target; read AmuletCompass and the item JSON head.
11. Checked create-world hooks (API 326), `settings`/`settings_click`, level rule setters, `world_lifecycle`.
12. Checked totem client code (`totem.rs`) and the spyglass `scoping()`.
13. Wrote this report.
