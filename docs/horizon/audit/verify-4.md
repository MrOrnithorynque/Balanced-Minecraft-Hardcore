# Verification of audit 4 (the Navigator's Amulet and the client visuals)

Paths. H = S/src/horizon (main b880384). `H:game/x.rs` means `H/crates/horizon-client/src/game/x.rs`. N = Nightly 2026-11-08 catalog, M = main catalog, F = pending fragment in `H/tools/native-hook-fragments/`.
Tree guard passed: W printed itself, branch `horizon-26.3`, H at b880384691b88d317553067109d28e785990bda3.

## Corrected verdict table

| Row | Feature | Audit said | Corrected | Tier | Mechanism | Changed? |
|---|---|---|---|---|---|---|
| 1a | Cinematic drawing | HOOK | HOOK | N | `gui_render_post` + `fill`/`quad`/`gui_paint`/`text_width`; draws over the death screen and the respawn loading screen | no |
| 1b | Signals server to client | HOOK | HOOK | N | `living_death` / `player_respawned` + `net_send_to_player` + `net_from_server` (the owner of a single-player world is a hub guest, so it is heard) | no |
| 1c | Hold the Respawn request | PARTIAL | PARTIAL | F | Click and key: `screen_*_pre` (N) + `send_packet` (N). Gamepad click skips the hook. Immediate respawn: only through `server_packet` + `respawn_player` (both F) | no (mechanism changed) |
| 1d | Input hold and loading screen | PARTIAL | HOOK | N | `gui_render_post` DOES fire over the respawn loading screen (code). `mod_screen_open` retried each frame says "load over" | yes |
| 1e | Music fade, chimes, bell | HOOK | HOOK | N | `music` volume + `play_local_sound`; no sound category | no |
| 2 | Star reading in the sky | HOOK (raw wgpu), PINNED for exact | HOOK | N | `world_drawn` + `render_needs` + own wgpu pipeline, depth test against sky; star angle recomputed. Alternative `world_primitives` is F, not N | no |
| 3 | Spyglass line name | PARTIAL | HOOK | N | "Is scoping" is exposed: `living_entity_use_item_tick`/`start`/`stop` fire on the client for the local spyglass use. Draw in `gui_render_post` | yes |
| 4 | Amulet needle | HOOK (data only) | HOOK | N | vanilla `minecraft:compass` range property, target lodestone, default `minecraft:lodestone_tracker` via `modify_default_components` | no |
| 5a | Amulet use / readings / charges | HOOK | PARTIAL | X | `use_item_on` + `use_item` + `carried_stack` fine (N). World spawn has no reader in any catalog: X. `sneaking()`, server `play_sound`, `block_name_in_tag` are F | yes |
| 5b | Points saved on the player | HOOK | HOOK | N | `attachments` on `Holder::Player(uuid)`, `copy_on_death` | no |
| 5c | Naming and list screens | PARTIAL | PARTIAL | N | `mod_screen_open` + `screen_*` hooks + `net_*`. Hand-written text field, no clipboard, gamepad cannot click a mod screen | no |
| 6 | Create World button | PARTIAL, true parity GAP | PARTIAL | F | Button drawn on `world_create` (N). Rules written at first open with `level_set_game_rule` (F, `level_world_setters`) | no |
| 7 | Config screen | PARTIAL | PARTIAL | N | `settings` + `settings_click` buttons only | no |
| 8 | Totem animation item | PINNED (else GAP) | PINNED | X | Inject into `Game::find_totem` (`H:game/totem.rs:166`). For the owner the remote path passes no protector (`totem.rs:134`) | no |

## Row notes

### V0 (audit 6, checked first)
Citations hold. `H:game/integrated.rs:823` "// No player of its own: the hub runs without a host." `integrated.rs:880` "host.hub.lan.connect_local()?". `integrated.rs:977` "if let Some(g) = host.hub.guests.iter().find(|g| g.uuid == owner && g.placed) {". `H:game/lan_actions.rs:472` "pub(super) fn enter_guest(", `:483` "let i = host.hub.guests.iter().position(|g| g.conn == conn)?;", `:711` "self.lan_acting_uuid.replace(g.uuid),". `H/crates/horizon-client/src/lan/mod.rs:790` "pub fn connect_local(&self) -> io::Result<horizon_protocol::connection::Pipe> {" (audit 6 wrote `lan/mod.rs:788-790`, the path is under `src/lan/`, not `src/game/lan/`). V0 stands.
Effect on this group: audit 4 never said "not heard for the owner". All its server hooks fire for the owner. Two doc comments in H are stale and could mislead: `native_entity_data_to` ("Not heard: a player who isn't a guest (single player's own)", `H:game/native_hooks.rs` near 7698) and `native-mods.md` "will join a later slice" (see 5c). Proof the packet hook reaches the owner: `serve_local` uses `conn.rs`, whose `PlayerSender::send` calls `outgoing(self.player, body)` (`H/crates/horizon-client/src/lan/conn.rs:435`, defined `:283`).

### 1a
Checked `gui_render_post` doc `H:game/native_hooks.rs:2519` "After the complete normal GUI batch, including screens, prompts, toasts, debug overlays, the pad cursor and IME." and the call `H:game.rs:9573` "self.native_gui_render_post(&mut hud, self.ui.hide_gui, self.focused);" (exact). `native_fill` `native_hooks.rs:11108` takes `rect: [f32; 4], argb` and keeps alpha. `native_quad` `:11286`. `native_text` forces alpha: `:11468` "let color = rgba(r, g, b, 0xFF);" (so alpha text goes through `native_gui_paint`, `native_client_services.rs:~424`, `GuiDraw::Text`, `hud_layout.rs` shows `Text { text, x: i32, y: i32, color: u32, shadow }`). All N helpers (`fill`, `quad`, `gui_paint`, `text_width`, `gui_nine_slice`). Verdict stands.

### 1b
`client_packet` is read-only: `native_hooks.rs` "Read-only: the game handles the packet as it would without the mod." (about line 8580). `native_net_send_to_player` (`H:game/native_net.rs:172-187`) errors "a client has no players to send to" when `self.remote.is_some()`: it must run in the server Game, which it does for a server hook. `living_death`, `player_respawned`, `net_from_server`, `net_send_to_player` are all in N. Stands.

### 1c
Citations: death click chain `H:game.rs:10877` `Screen::Dead if pressed && button == MouseButton::Left => {` then `self.death_click()?` (10878); `screens.rs:911-913` `fn death_action` / `Some(0) => self.respawn_player()?,`; `screens.rs:770-775`; `remote.rs:10497-10498` `self.send(S::ClientCommand(play::ClientCommand { action: 0 }));`. The mouse hook is dispatched first: `game.rs:10312` `self.native_screen_mouse_button_pressed_pre(`. The keyboard hook: `game.rs:10219` `PhysicalKey::Code(key) => self.screen_key_pressed(key, key_event.repeat),` runs before `death_key` (`game.rs:11076`, `play_keys.rs:307`).
New finding 1, gamepad bypass is real in code: the whole hook block is guarded by `&& !self.pad_synthetic_input()` (`game.rs:~10306`), with the comment "A click the pad or the keyboard's focus made up is not the mouse: it fires nothing." So a pad click on Respawn reaches `death_click` with no hook. The audit left this UNVERIFIED.
New finding 2, the immediate-respawn gap can be closed with pending items. `remote.rs:4634` `self.respawn_player()?;` has no hook, as the audit says. But the death arrives as a server packet. `server_packet` (F, `server_packet.json`, side server, cancellable, "set `cancel` and it gets nothing") can drop `PlayerCombatKill` (and the `SetHealth` with health <= 0, which also opens the screen, `remote.rs:~4603`) for that player. `respawn_player(uuid)` (F, `respawn_player.json`, `H:game/native_travel.rs:369`, "respawns a dead server player as their client's Respawn click does") then respawns on a mod message "screen is black". This also removes the gamepad bypass. UNVERIFIED: how the client behaves while alive-looking at 0 health, and whether the server keeps a dead guest idle (it keeps `g.dead`, `native_travel.rs:381-390`). Tier F, confidence medium-low.
Tier for the main path (click and key): N. Verdict label stays PARTIAL, but the GAP is now "F-closable".

### 1d
Audit said UNVERIFIED and listed it as a possible GAP. Resolved by code. The respawn loading screen is drawn by `remote_overlay` (`H:game/remote.rs:10511`; branch `if r.joined || loading {` draws `LevelLoadingScreen`), called from `hud()` (`game.rs:~5927` "if self.remote_overlay(&mut batch) {" then `return batch;`). `hud()` returns to `render_frame`, which then calls `native_gui_render_post` at `game.rs:9573` with no guard. The frames that skip it are the ones that `return renderer.render(&loading_frame(&hud))` before `hud()` runs: the file fixer (`game.rs:~9097-9104`), the starting integrated server (`~9140-9148`), `pending_entry` (`~9166`) and the pack reload (`~9184`). The doc line "loading ... overlays do not fire it" (`native_hooks.rs:2521`, `native-mods.md:1823`) is about those four, not the level-loading screen of a respawn or portal. So the black cover holds across the load. Input hold: `native_mod_screen_open` is false "while the world loads" (`native_hooks.rs:11296-11299`, `blocked ... || self.remote_loading()`), so the mod calls it every frame until it returns true. That needs no `native_client_state().screen` (that field is F, `session_state`, `native_client_services.rs:41-42`). UNVERIFIED by a run: whether the load screen's own frame draws the world behind. Changed to HOOK, N, confidence medium.

### 1e
`music` doc (`native_hooks.rs:3061-3068`): "Choose this client's music, once a client tick ... change `volume` (0 to 1) to fade it." Contract listen, side client, N. `native_play_local_sound` (`:11043`), no source argument. Stands.

### 2
`WorldDrawn` fields verified at `H/crates/horizon-client-render/src/gfx/native_render.rs:47-95` ("`depth` the world's depth, reversed (0 is the sky)"; `depth ... holds the frame's depth only when a mod asked for it ([`RenderNeeds::world_depth`])"). `render_needs` (N, collect) sets `world_depth` (`native_render.rs:154`, `:222`). Star port: `H/crates/horizon-client-render/src/gfx/stars.rs:12-45` seeds `LegacyRandom::new(10842)`, 1500 tries, same filter. Vanilla draw site `gfx.rs:10660-10667` quoted correctly. `own_stars` there comes from Horizon's shader packs (`native_pack.rs:21`, "`vanilla_stars`"), not from mods: so no mod hook draws inside the sky pass. No helper exposes `star_angle` (searched `star_angle`, `star_brightness` in `native_hooks.rs`, `mod_api.rs`: only `frame_geometry.rs` and render crates). Stands.
Citation problem: the audit's header says "Every hook used below is in both catalogs except world_hold and world_warp". The alternative it names, `world_primitives`, is in NEITHER catalog: it is the fragment `H/tools/native-hook-fragments/world_primitives.json` (tier F). Docs describe it (`native-mods.md:4784`). Not load-bearing (the main path is `world_drawn`).

### 3 (changed to HOOK)
Audit claimed "is scoping is not exposed to mods" (searched `spyglass`/`scoping`). Missed: `living_entity_use_item_start` / `_tick` / `_stop` (N, side `None` = both). Doc `native_hooks.rs:8232-8244`: "`entity` ... None for a player ... spyglasses and mods' items count ... Both sides, as vanilla's: on the server for every player of the world, and on a server's client for the uses it predicts." Fire sites: `H:game/item_use.rs:45` (start), `:162` (tick, "Runs every tick of every use: check the item first"), `:191` (stop). `fov.rs:38-42` `Game::scoping` reads `self.using`, the same state these hooks follow. Plan: in `_tick` with `stack.item == "minecraft:spyglass"` on the window's Game (`window()` helper, N) stamp "scoping until now+2 ticks". Hand change stops a use without a release (doc `:8348-8351`), so a tick timestamp is safer than start/stop flags. The audit's `native_mapping` approximation is also not available to typed mods: `native_mapping` is only in the mixin-helper table (`horizon-docs/mod-code.txt:46`) and in no catalog or fragment.
The Java mixin body `K_GuiSpyglassMixin` was not read by the audit, nor by me (UNVERIFIED text style).

### 4
`item_model.rs:882-892` `"minecraft:compass" => Self::Compass { wobble, target: ... "lodestone" => CompassTarget::Lodestone ...}` (exact). `compass()` `:1581-1650` reads `minecraft:lodestone_tracker` target dimension and pos, returns the random spin when `t.dimension == level.dimension` fails (matches Java's Nether spin), local-player wobble `if owner.is_local_player && level.runs_normally`. Components read patch first, then item defaults (`item_model.rs:916-921`, `:1122`), so a default component works. `modify_default_components` is N (side both, collect). Doc `native-mods.md:5081-5099`: "Any item can be changed". UNVERIFIED: that the BMH item is a row of the defaults table `items.json` (it is a content-mod item: the items group owns this). The audit's note that `item_model_property` cannot do it is right (`native-mods.md:45` "Vanilla properties do not enter the hook").

### 5a (changed to PARTIAL, tier X)
What holds. `use_item_on` and `use_item` docs at `native_hooks.rs:8147-8165` and `:8190-8197`: "Server-side, for every player of the world ... an item on cooldown is never used." Fire site `H:game/use_hand.rs:231-236`: `self.native_use_item_on(&stack, hit.pos, hit.face, &mut claimed);` then `if !claimed && !on_block { self.native_use_item(&stack, &mut claimed); }`. Sneak bypass is implemented, so audit Q5 is answered YES: `use_hand.rs:~12` `let sneaking = self.keys.held(Mapping::Sneak);`, `:~13` `let hands_full = ...`, and `:~44` `if !(sneaking && hands_full)` skips the block's own use. Sneak plus bed with the amulet in hand reaches `use_item_on` without opening the bed.
Corrections and new findings.
- The hand IS available: `right_click_block_v2` (F) gives it; and `acting_player().selected_slot` and `.sneaking` exist (`mod_api.rs` `NativeServerPlayer`, "pub selected_slot: u8", "pub sneaking: bool"), but those fields come from the pending `server_player_state.json` (tier F). In N: track sneak with `player_toggle_sneak` (N, server, `native_hooks.rs:4562` "A guest's client changed sneaking"), and the hotbar slot with `inventory_tick(slot, selected)`.
- Writing the stack needs no one-tick delay: `native_carried_slot` (`native_hooks.rs:11958`) indexes the acting player's inventory with no hook check, and the state is swapped in during `use_item` (audit's own `native_acting_player` doc). Find the slot by `selected_slot` (F) or by scanning slots 0..8 for the stack passed in (N).
- `sneaking()` is NOT in either catalog: it is in fragment `host_bridge.json` (helper `sneaking`, `native_host_bridge.rs:118`), tier F. The same fragment holds `play_sound` (server sound in place), `block_name_in_tag`, `equipment`, `component`. The audit marked them "UNVERIFIED which Nightly has it"; none is in N or M.
- Item cooldown (audit Q4): no setter exists. `player_item_cooldown` (N, server, `native_hooks.rs:5151`) only cancels a cooldown the game starts. Searched both catalogs, all fragments, `fn native_.*cooldown`. The mod keeps its own per-player timer.
- Double fire (audit missed): the client treats an unknown mod item as "pass". `H:game/remote.rs:8920-8922` `let Some(info) = self.items().get(&held.item).cloned() else { return false; };` and `remote_use_hands` (`remote.rs:9225-9249`) sends `UseItem` for the main hand, then offers the OFF hand (`native_interaction(1, Hand::Off)`, `remote_use_block(Hand::Off)`, `remote_use_item(Hand::Off)`). So a sneak click on a bed sends `UseItemOn` (claimed: screen opens) and then `UseItem` (the sneak "list screen" fires too), and an off-hand shield, bucket or pearl also acts. Java avoids both by returning CONSUME on the client. Fixes in N: ignore `use_item` for a player within the same tick as a claimed `use_item_on`; cancel `interaction_key` (N, client, `native_hooks.rs:2781-2791`, "once for each hand it tries") when `hand == Off` and the main hand holds the amulet.
- World spawn (audit Q4, left open) has no reader. `world_spawn_position` (`H:game/lan.rs:1254`) is `pub(super)`, not a helper. `spawn_change` (N) fires only on a change (`commands.rs:3881`, from the command). `player_respawn_position` (F) fires at respawn. So the Java "spawn line" in `AmuletReading.record` needs a mixin helper calling `world_spawn_position`: tier X.
- Position and hands of a player inside `net_from_client` (audit Q4): possible in N. `server_players()` gives `NativeServerPlayer` (position, dimension, health, game_mode, `entity_id`); `entity(id)` ("among the server's players", `native_entity_refs.rs:62-67`) then `living(entity)` then `living_equipment(living, MainHand/OffHand)` (`native_living_refs.rs:~73-98`, the `Target::Guest(conn)` arm reads the guest's hands). UNVERIFIED by a run.
- `isDarkOutside` and `canSeeSky` still have no helper (no `dark_outside` in the crates). The mod computes from `day_time`, `weather`, `surface_heights`.
Tier X comes only from the world spawn. Without the spawn line the row is HOOK at N (with the F items replaced by the N ways above).

### 5b
`attachments` is an N helper. `native-mods.md:5443` "`Holder::Entity(id)`, or `Holder::Player(uuid)`", `:5454` "`{value: <compound>, copy_on_death: 1b, sync: 1b}`", `:5478` "`copy_on_death` requires a codec". Stands, N. Not touched by the world-spawn gap (that is a read, not stored).

### 5c
`mod_screen_open` quoted correctly (`native_hooks.rs:11290-11323`). New: "A gamepad can close it (B, Start) but not click on it: its clicks don't reach the mouse hooks." (`:11300-11301`), so a gamepad player cannot use Save/Remove buttons. Audit Q7 (do `screen_key_pressed_pre` and `screen_char_typed_pre` exist in the typed trait): the doc at `native-mods.md:5520-5524` ("will join a later slice") is older than the catalog. The Nightly catalog gives both a `mod_api` contract (`screen_char_typed_pre`: contract cancel, out canceled, side client), and the generator builds the trait from the catalog. Treat the doc as stale; UNVERIFIED by a build. The pending `screen_text_field` fragment does not help: its doc excludes "Options, preparing screens, game-rule/numeric editors, books, signs and dialogs" and mod screens. Stands PARTIAL, N.

### 6
Hook ids verified at `native-mods.md:1653`. No create-screen state helper found (searched `world_create`, `CreateWorld`, `create_world` in `native_hooks.rs`, `mod_api.rs`). `level_set_game_rule` is a helper of fragment `level_world_setters.json` (F), `level_game_rule` of `level_world.json` (F); the audit cited them as if stamped. Stands PARTIAL, tier F.

### 7
`settings`, `settings_click` (N, collect and listen). Stands.

### 8
`H:game/totem.rs:158` `let totem = body.protector.unwrap_or_else(|| self.find_totem());`, `:166-170` `find_totem` reads `[self.held(), self.offhand.as_ref()]` only. Extra: the server half `apply_totem` (`totem.rs:66-115`) also looks only at the hands (`cell = if main { &mut self.inventory[self.selected] } else { &mut self.offhand }`), and the owner receives the event over the guest path `remote_totem` with `protector: None` (`totem.rs:134`). So even a Totem of Undying held by the owner shows its animation by lookup, not by the server's copy. A Totem of Eternity in the inventory shows the fallback `TOTEM`. Alternative, partial and unproven: `server_packet` (F) cancels entity event 35 for the player and a `gui_render_post` overlay plus `play_local_sound` redraws it; particles, rumble and vanilla geometry would be redone by hand. Stands PINNED, tier X.

## Claims refuted
- "Is scoping is not exposed to mods" (audit row 3): `living_entity_use_item_tick` fires on the client for a spyglass use (`H:game/item_use.rs:162`, doc `native_hooks.rs:8232-8244`).
- "Whether `gui_render_post` draws over the loading screen is UNVERIFIED (doc says no)" (row 1d): the respawn level-loading screen goes through `hud()` and `game.rs:9573`; only four early-return frames skip the hook.
- "The immediate-respawn hold is a GAP with no hook" (row 1c): closable with `server_packet` + `respawn_player` (both F). Not a hard GAP.
- "`sneaking()`... UNVERIFIED which Nightly has it" (5a): it is in neither catalog; it is pending in `host_bridge.json`.
- "Every hook used below is in both catalogs except world_hold and world_warp": `world_primitives` is in neither (fragment).
- "Because use_item_on does not claim for a non-bed block, the non-bed click falls through to use_item exactly as Java falls from useOn PASS to use" (5a): the client also sends `UseItem` after a claimed `UseItemOn`, and then offers the off hand (`remote.rs:9225-9249`), which Java prevents with CONSUME.
- "One tick of delay to write the stack" (5a): not needed, `carried_stack` works inside `use_item`.

## Citations that did not match
- `native-mods.md:58` for "Bundles need hooks: 385 once this pending hook is stamped": the line is `native-mods.md:65`.
- `H:native_hooks.rs:2517-2525` for the `gui_render_post` doc: the text is at 2519-2525 (off by two lines, content right).
- Audit 6 V0 cites `lan/mod.rs:788-790`: the file is `H/crates/horizon-client/src/lan/mod.rs` (line 790 holds `connect_local`), not under `game/`.
- `native_hooks.rs:4138-4143` (`screen_hooks_open`) matches at 4138-4144.
- `native_mapping` is cited as a typed-API helper ("H:docs mod-code table"): it exists only in the mixin-helper table and `native_hooks.rs:11506`, in no catalog or fragment.
- Other line cites checked and right within a few lines: `game.rs:9573`, `:695`, `:6235`, `:10312`, `:10877`; `screens.rs:770-776`, `:911-913`; `remote.rs:10497-10499`, `:4634`, `:3444`; `native_hooks.rs:11108`, `:11286`, `:11302`; `fov.rs:38-42`; `totem.rs:166`; `item_model.rs:882`, `:1582-1650`; `stars.rs:12-45`; `gfx.rs:10660-10667`; `native_render.rs:46-95`; `native_net.rs:174`, `:309`.

## Hooks and helpers the rows need (name, kind, tier, catalog)

| Name | Kind | Tier | Catalog |
|---|---|---|---|
| gui_render_post | hook (client) | N | N, M |
| fill, quad, gui_paint, text_width, gui_nine_slice, sprite | helpers (client) | N | N, M |
| net_send_to_player, net_send_to_server | helpers | N | N, M |
| net_from_server, net_from_client | hooks | N | N, M |
| living_death, player_respawned | hooks (server) | N | N, M |
| screen_mouse_button_pressed_pre, screen_key_pressed_pre, screen_char_typed_pre, screen_render_post, screen_init_post | hooks (client) | N | N, M |
| send_packet | helper (client) | N | N, M |
| mod_screen_open, mod_screen_close, mod_screen | helpers | N | N, M |
| music | hook (client) | N | N, M |
| play_local_sound | helper (client) | N | N, M |
| world_drawn, render_needs | hooks (client) | N | N, M |
| camera, clock, day_time, weather, window, surface_heights, light | helpers | N | N, M |
| living_entity_use_item_tick, _start, _stop | hooks (both) | N | N, M |
| modify_default_components | hook (both) | N | N, M |
| use_item_on, use_item, inventory_tick | hooks (server) | N | N, M |
| carried_stack, held, acting_player, server_players, entity, living, living_equipment, player_stacks, action_bar, block_property, level_block | helpers | N | N, M |
| attachments | helper | N | N, M |
| interaction_key | hook (client) | N | N, M |
| player_toggle_sneak | hook (server) | N | N, M |
| settings, settings_click, setting, set_setting | hooks and helpers | N | N, M |
| item_bar | hook (client) | N | N, M |
| server_packet, watch_server_packets | hook and helper (server) | F | fragment `server_packet.json` |
| respawn_player | helper (server) | F | fragment `respawn_player.json` |
| sneaking, play_sound (in place), block_name_in_tag, equipment | helpers | F | fragment `host_bridge.json` |
| NativeServerPlayer.sneaking, .selected_slot | fields | F | fragment `server_player_state.json` |
| right_click_block_v2 (hand) | hook (server) | F | fragment `right_click_block_v2.json` |
| NativeClientState.screen | field | F | fragment `session_state` (not needed if `mod_screen_open` is retried) |
| level_set_game_rule, level_game_rule | helpers | F | fragments `level_world_setters.json`, `level_world.json` |
| world_primitives | hook (client) | F | fragment (alternative only) |
| world spawn read (`Game::world_spawn_position`, `lan.rs:1254`) | mixin helper | X | none |
| native_mapping (key.use held) | mixin helper | X | `mod-code.txt:46`, not typed |
| Game::find_totem (RETURN or MODIFY_EXPRESSION_VALUE), `totem.rs:166` | injection | X | none |
| item cooldown setter | missing | G (mod timer instead) | none |
| clipboard, text-field widget for mod screens | missing | G (hand-written) | none |
| Create-World screen state (tab, hardcore flag, rules) | missing | G | none |

## Step log
1. Ran the tree guard (W, branch, H commit): all three matched.
2. Read the brief and the whole audit (two pages).
3. Checked audit 6's V0 citations in `integrated.rs`, `lan_actions.rs`, `lan/mod.rs`: they hold.
4. Compared hook and helper names of both catalogs with the names the audit relies on (python over the JSON): found `sneaking`, `mapping`, `play_sound`, `block_name_in_tag`, `equipment` in neither.
5. Listed the 228 pending fragments and read the relevant ones: `server_packet`, `respawn_player`, `host_bridge`, `server_player_state`, `right_click_block_v2`, `world_primitives`, `world_canvas`, `level_world*`, `compass_target`, `screen_text_field`, `draw_text`.
6. Traced the death and respawn path (`game.rs`, `screens.rs`, `remote.rs`); found the pad bypass and the `server_packet` + `respawn_player` route.
7. Resolved `gui_render_post` over the loading screen from `render_frame` early returns and `hud()`.
8. Traced `use_hand.rs` and `remote.rs` use paths: sneak bypass present, double fire and off-hand use found, `interaction_key` as the fix.
9. Found `living_entity_use_item_*` fire sites in `item_use.rs` for the spyglass.
10. Read `compass()` and the component order in `item_model.rs`.
11. Read `native_render.rs` (`WorldDrawn`), `stars.rs`, `gfx.rs` sky site, `native_pack.rs` (`own_stars` is a shader pack feature).
12. Read `totem.rs` (`apply_totem`, `remote_totem`, `find_totem`).
13. Searched for a world spawn reader, an item cooldown setter, a clipboard helper: none in catalogs or fragments.
14. Wrote this file.
