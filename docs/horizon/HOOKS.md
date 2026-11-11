# BMH on Horizon: the hook points

This file lists every hook point that the native bundle (`horizon/native/mixins/bmh`) wires, and the order in
which each helper calls the areas. It belongs to the integrator. Read `ARCHITECTURE.md` first.

All signatures, paths and line numbers come from the Horizon Nightly of 2026-11-08, commit
`fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` (decision D1). Every injection names its hook by `target`
(decision D3), for stamped and pending hooks alike.

## Rules

- `hook_<hook>.rs` holds one function, `bmh_hook_<hook>`. Its parameters are the manifest's bindings: same names,
  same order. It checks `game.native_on(crate::game::bmh::ID)` first, then calls `on_<hook>` in each area, in the
  order of the table.
- An area function gets every value that the hook hands over, and the `Game` where the hook has one
  (`&mut Game` for a `&mut self` hook, `&Game` for a `&self` hook). An out parameter (`cancel`, `canceled`,
  `claimed`, `ran`, `amount` and so on) passes through. Each area reads it and decides for itself: an area that
  must not act after a cancel tests the flag.
- Where two areas share a hook and the Java mod used event priorities, the last column gives the reason for the
  order and the Java priority it replaces. "No order" means that the areas touch different items, screens or
  channels.
- Side. A `client` hook fires only in the window's `Game`, a `server` hook only in a `Game` that runs the world. A
  `both` hook fires in each of them (in single player, once in the window's `Game` and once in the integrated
  server's `Game`, on two threads). An area tells them apart with `game.native_window()`
  (`crates/horizon-client/src/game/native_hooks.rs:10726`, "This `Game` is the window's, not a server's"): true in
  the window's `Game`, false in a server's `Game`. Rules for a `both` hook:
  - Change the world (items, XP, effects, sounds for others, advancements, mod messages to players) only when
    `native_window()` is false. The use hooks say so in their doc ("change the world only on the server
    (`native_window` is false there)", native_hooks.rs:8050-8051).
  - An answer that the hook hands back (`use_duration`'s `duration`, the `canceled` flag and `duration` of the use
    hooks, `anvil_update`'s `output`, `xp_cost` and `material_cost`, `modify_default_components`'s `components`)
    must be the same on both sides. Compute it from the values the hook gives and from state that both sides have
    (the stack, the config), never from a `SERVER_*` static alone.
  - Keep the client's own state in `CLIENT_*` statics and the server's in `SERVER_*` statics (ARCHITECTURE.md,
    State). Pick the static with `native_window()`.
  - The `both` hooks are: `register_configs`, `config_loading`, `config_reloading`, `use_duration`,
    `living_entity_use_item_start`, `living_entity_use_item_tick`, `living_entity_use_item_stop`, `on_use_tick`,
    `release_using`, `anvil_update`, `living_fall`, `modify_default_components`.
  - `living_fall` and `modify_default_components` have no `Game`, so they cannot call `native_window()` and cannot
    tell the sides apart. Their area gives the same answer on both sides: `modify_default_components` answers
    from the item id alone, and `living_fall` answers only from values that both sides have for that entity.
- The API column gives the number in the hook's doc comment in the Nightly (`API n`, or `n.` at its end). The
  highest stamped number among the wired hooks is 410 (`gui_render_post`), so `compatibility.json` has
  `"hooks": 410`. The Nightly's `NATIVE_HOOKS_API` is 411.

### Hooks with no Game

Six wired hooks have no `Game`, so their helper cannot check `native_on`:

| Hook | Why there is no `Game` | Where the area code lives |
|---|---|---|
| `living_incoming_damage` | an associated function with no `self` ("the game is in the middle of the hit") | `bmh_respawn.rs` (game module) |
| `hopper_pickup_item` | a method of `GameLevel`, which holds no `Game` | `bmh_totem.rs` (game module) |
| `living_fall` | a free function of the client library (`horizon_client_lib::landing`) | `bmh_tools_lib.rs`, a `pub` module in `crates/horizon-client/src/client_lib.rs` |
| `world_drawn`, `render_needs` | free functions of `horizon-client-render` | `bmh_sky_render.rs`, a `pub` module in `crates/horizon-client-render/src/lib.rs` |
| `modify_default_components` | a free function of `horizon-world` | `bmh_totem_world.rs` and `bmh_amulet_world.rs`, `pub` modules in `crates/horizon-world/src/lib.rs` |

Rule for these hooks: the area acts only on state that a gated hook (one that checked `native_on`) wrote. With the
mod off, that state stays empty, so the hook does nothing. `modify_default_components` is the exception: its doc
asks for the same answer each time, and the launcher leaves a mod that is off out of its next build.

A module in another crate cannot call the game module (the dependency goes the other way) and cannot use
`bmh_logic` (the `[[crate]]` is a dependency of `horizon-client` only). The game side reaches it by its crate path:
`horizon_client_lib::bmh_tools_lib`, `horizon_client_render::bmh_sky_render`, `horizon_world::bmh_totem_world`,
`horizon_world::bmh_amulet_world`. Its owner makes a function `pub` when the game side needs it.

## Wired hooks

| Hook | Function in HN | Parameters as HN declares them | Side | API | Areas, in order | Why this order |
|---|---|---|---|---|---|---|
| `mods` | `horizon_client::game::native_hooks::Game<'_>::native_mods` (crates/horizon-client/src/game/native_hooks.rs:3246) | `&mut self, mods: &mut Vec<NativeMod>` | client | stamped, no API number in its doc comment | bmh (`bmh::row`) | The row in Options > Mods. No `native_on` check: the row shows when the mod is off, so the player can turn it on again. |
| `player_respawn_position` | `horizon_client::game::native_hooks::Game<'_>::native_player_respawn_position` (crates/horizon-client/src/game/native_hooks.rs:7317) | `&mut self, player: u128, name: &str, dimension: &mut String, pos: &mut [f64; 3], yaw: &mut f32, pitch: &mut f32` | server | pending (fragment player_respawn_position.json) | respawn | One area. |
| `living_death` | `horizon_client::game::native_hooks::Game<'_>::native_living_death` (crates/horizon-client/src/game/native_hooks.rs:6567) | `&mut self, entity: u32, kind: &str, damage_type: &str, attacker: u32, cancel: &mut bool` | server | 392 | totem, respawn | The totem decides first whether the player dies. The departure signal goes last and only for a real death (Java: `announceJourney` at `Priority.LOWEST`, a revived player must not see the journey). |
| `loot_drops` | `horizon_client::game::native_hooks::Game<'_>::native_loot_drops` (crates/horizon-client/src/game/native_hooks.rs:8951) | `&mut self, context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>` | server | 75 (player inventory: 405) | respawn, tools, eternal | Respawn counts the Eternal Items while they are still in the drops (Java: `measureRespawnDistance` at `Priority.HIGHEST`). Tools wears them while they are still there. Eternal takes them out last (Java: `keepEternalItems` at the normal priority). |
| `player_died` | `horizon_client::game::native_hooks::Game<'_>::native_player_died` (crates/horizon-client/src/game/native_hooks.rs:8704) | `&mut self, player: u128, cause: Option<&str>` | server | 40 | respawn | One area. |
| `player_tick` | `horizon_client::game::native_hooks::Game<'_>::native_player_tick` (crates/horizon-client/src/game/native_hooks.rs:2791) | `&mut self` | server | 31 | respawn, totem, tools | No shared state: the order of ARCHITECTURE.md. |
| `player_respawned` | `horizon_client::game::native_hooks::Game<'_>::native_player_respawned` (crates/horizon-client/src/game/native_hooks.rs:8459) | `&mut self, player: u128, name: &str` | server | 115 | eternal, respawn, homecoming (`on_exile`) | Java: `giveBackEternalItems` at the normal priority, then `playRespawnEffects` at `Priority.LOWEST`, which ends with `HomecomingHandler.onExile`. The record needs the exile center that respawn reads first: `bmh_respawn::on_player_respawned` gives back `Option<bmh_logic::contract::Exile>`, and the helper calls `bmh_homecoming::on_exile(game, player, &exile)` only when it is `Some` (section "Contracts" below). |
| `player_logged_out` | `horizon_client::game::native_hooks::Game<'_>::native_player_logged_out` (crates/horizon-client/src/game/native_hooks.rs:8451) | `&mut self, player: u128, name: &str` | server | 57 | respawn, totem, ritual, tools, amulet | Cleanup only (ARCHITECTURE.md, State). No order. |
| `server_tick` | `horizon_client::game::native_hooks::Game<'_>::native_server_tick` (crates/horizon-client/src/game/native_hooks.rs:2571) | `&mut self, context: &NativeLifecycleContext, phase: ServerTickPhase` | server | 4 | respawn, homecoming, eternal, totem, ritual, tools | No shared state: the order of ARCHITECTURE.md. Each area acts on one phase. |
| `living_incoming_damage` | `horizon_client::game::native_hooks::Game<'_>::native_living_incoming_damage` (crates/horizon-client/src/game/native_hooks.rs:9202) | `entity: i32, kind: &str, damage_type: &str, causing: i32, direct: i32, amount: &mut f32, canceled: &mut bool` | server | 50 | respawn | One area. The hook has no `self`, so no `native_on` check: the area acts only on immunity entries that gated hooks wrote. |
| `living_incoming_damage_v2` | `horizon_client::game::native_hooks::Game<'_>::native_living_incoming_damage_v2` (crates/horizon-client/src/game/native_hooks.rs:9286) | `&mut self, entity: i32, kind: &str, damage_type: &str, causing: i32, source_position: Option<[f64; 3]>, victim_position: [f64; 3], victim_view: [f64; 3], weapon: Option<&ItemStack>, amount: &mut f32, canceled: &mut bool` | server | 384 | tools | One area. |
| `difficulty_changed` | `horizon_client::game::native_hooks::Game<'_>::native_difficulty_changed` (crates/horizon-client/src/game/native_hooks.rs:8412) | `&mut self, from: &str, to: &str` | server | 159 | respawn | One area. |
| `net_from_server` | `horizon_client::game::native_hooks::Game<'_>::native_net_from_server` (crates/horizon-client/src/game/native_hooks.rs:8366) | `&mut self, channel: &str, data: &[u8]` | client | 40 | journey, tools, amulet | Each area reads only its own channel. No order. Journey reads `bmh:journey` (`bmh_logic::wire::JOURNEY_CHANNEL`, `JourneySignal`), which the respawn area sends. |
| `net_from_client` | `horizon_client::game::native_hooks::Game<'_>::native_net_from_client` (crates/horizon-client/src/game/native_hooks.rs:8361) | `&mut self, player: u128, channel: &str, data: &[u8]` | server | 40 | respawn, amulet | Each area reads only its own channel. No order. Respawn reads `bmh:respawn` (`bmh_logic::wire::RESPAWN_CHANNEL`, `RespawnRequest`), which the journey area sends. |
| `screen_mouse_button_pressed_pre` | `horizon_client::game::native_hooks::Game<'_>::native_screen_mouse_button_pressed_pre` (crates/horizon-client/src/game/native_hooks.rs:3789) | `&self, screen: &NativeScreenView<'_>, button: i32, modifiers: u32, canceled: &mut bool` | client | 80 (screen view: 326) | settings, journey, amulet | Each area acts on its own screen. No order. |
| `screen_key_pressed_pre` | `horizon_client::game::native_hooks::Game<'_>::native_screen_key_pressed_pre` (crates/horizon-client/src/game/native_hooks.rs:3920) | `&self, screen: &NativeScreenView<'_>, key: &str, repeat: bool, modifiers: u32, canceled: &mut bool` | client | 104 (screen view: 326) | journey, amulet | Each area acts on its own screen. No order. |
| `screen_char_typed_pre` | `horizon_client::game::native_hooks::Game<'_>::native_screen_char_typed_pre` (crates/horizon-client/src/game/native_hooks.rs:3979) | `&self, screen: &NativeScreenView<'_>, c: char, modifiers: u32, canceled: &mut bool` | client | 104 | amulet | One area. |
| `screen_render_post` | `horizon_client::game::native_hooks::Game<'_>::native_screen_render_post` (crates/horizon-client/src/game/native_hooks.rs:3761) | `&self, batch: &mut HudBatch, screen: &NativeScreenView<'_>` | client | 78 (screen view: 326) | settings, amulet | Each area draws on its own screen. No order. |
| `gui_render_post` | `horizon_client::game::native_hooks::Game<'_>::native_gui_render_post` (crates/horizon-client/src/game/native_hooks.rs:2633) | `&self, batch: &mut HudBatch, gui_hidden: bool, focused: bool` | client | 410 | sky, journey | Journey draws last, so the black screen of the journey covers the spyglass name and the rest of the HUD. |
| `music` | `horizon_client::game::native_hooks::Game<'_>::native_music` (crates/horizon-client/src/game/native_hooks.rs:3160) | `&self, music: &mut Option<Music>, volume: &mut f32` | client | 119 | journey | One area. |
| `world_drawn` | `horizon_client_render::gfx::native_render::native_world_drawn` (crates/horizon-client-render/src/gfx/native_render.rs:152) | `world: &mut WorldDrawn<'_>` | client | 58 | sky | One area. The hook is in horizon-client-render, which has no `Game`: module `bmh_sky_render` there. No `native_on` check: it draws only what the client hooks of the sky area stored. |
| `render_needs` | `horizon_client_render::gfx::native_render::native_render_needs` (crates/horizon-client-render/src/gfx/native_render.rs:165) | `needs: &mut RenderNeeds` | client | 58 | sky | One area. Same crate and rule as `world_drawn`. |
| `register_configs` | `horizon_client::game::native_hooks::Game<'_>::native_register_configs` (crates/horizon-client/src/game/native_hooks.rs:2539) | `&mut self, configs: &mut Vec<NativeConfig>` | both | 397 | settings | One area. |
| `config_loading` | `horizon_client::game::native_hooks::Game<'_>::native_config_loading` (crates/horizon-client/src/game/native_hooks.rs:2547) | `&mut self, config: &NativeConfigFile` | both | 397 | settings | One area. |
| `config_reloading` | `horizon_client::game::native_hooks::Game<'_>::native_config_reloading` (crates/horizon-client/src/game/native_hooks.rs:2555) | `&mut self, config: &NativeConfigFile` | both | 397 | settings | One area. |
| `register_commands` | `horizon_client::game::native_hooks::Game<'_>::native_register_commands` (crates/horizon-client/src/game/native_hooks.rs:4313) | `&mut self, commands: &mut Vec<NativeCommand>` | server | 8 | settings, amulet | Both are under `/bmh`. No order. |
| `perform_command` | `horizon_client::game::native_hooks::Game<'_>::native_perform_command` (crates/horizon-client/src/game/native_hooks.rs:4323) | `&mut self, player: &NativeServerPlayer, command: &str, ran: &mut bool` | server | 8 | settings, amulet | Each area runs only its own subcommand and sets `ran`. No order. |
| `settings` | `horizon_client::game::native_hooks::Game<'_>::native_settings` (crates/horizon-client/src/game/native_hooks.rs:3252) | `&mut self, mod_id: &str, buttons: &mut Vec<String>` | client | stamped, no API number in its doc comment | settings | One area. |
| `settings_click` | `horizon_client::game::native_hooks::Game<'_>::native_settings_click` (crates/horizon-client/src/game/native_hooks.rs:3256) | `&mut self, mod_id: &str, button: usize` | client | stamped, no API number in its doc comment | settings | One area. |
| `world_lifecycle` | `horizon_client::game::native_hooks::Game<'_>::native_world_lifecycle` (crates/horizon-client/src/game/native_hooks.rs:2403) | `&mut self, context: &NativeLifecycleContext, phase: WorldPhase, error_text: &str` | server | 3 | settings | One area. |
| `player_bed_leave` | `horizon_client::game::native_hooks::Game<'_>::native_player_bed_leave` (crates/horizon-client/src/game/native_hooks.rs:4937) | `&mut self, player: u128, name: &str, pos: [i32; 3]` | server | 252 | eternal | One area. |
| `inventory_tick` | `horizon_client::game::native_hooks::Game<'_>::native_inventory_tick` (crates/horizon-client/src/game/native_hooks.rs:9107) | `&mut self, slot: i32, selected: bool` | server | 39 | eternal, tools, amulet | Eternal can set the count to 0 first; the slot empties only at the end of the hook, so later areas must skip a stack with count 0. |
| `item_picked_up` | `horizon_client::game::native_hooks::Game<'_>::native_item_picked_up` (crates/horizon-client/src/game/native_hooks.rs:8988) | `&mut self, player: u128, item: &str, count: u32, canceled: &mut bool` | server | 40 | totem, eternal | Each area tests its own items. No order (Java: one `onItemPickup` handler). |
| `item_damaged` | `horizon_client::game::native_hooks::Game<'_>::native_item_damaged` (crates/horizon-client/src/game/native_hooks.rs:8969) | `&mut self, player: u128, name: &str, item: &str, amount: &mut i32, cancel: &mut bool` | server | 162 | tools | One area. |
| `hurt_enemy` | `horizon_client::game::native_hooks::Game<'_>::native_hurt_enemy` (crates/horizon-client/src/game/native_hooks.rs:9094) | `&mut self, stack: &ItemStack, entity: u32, kind: &str` | server | 39 | tools | One area. |
| `player_attack_cooldown_reset` | `horizon_client::game::native_hooks::Game<'_>::native_player_attack_cooldown_reset` (crates/horizon-client/src/game/native_hooks.rs:5396) | `&mut self, player: u128, name: &str` | server | 250 | tools | One area. |
| `entity_join_level` | `horizon_client::game::native_hooks::Game<'_>::native_entity_join_level` (crates/horizon-client/src/game/native_hooks.rs:8755) | `&mut self, dimension: &str, entity: u32, kind: &str, pos: [f64; 3], loaded_from_disk: bool, canceled: &mut bool` | server | 47 | tools | One area. |
| `destroy_block` | `horizon_client::game::native_hooks::Game<'_>::native_destroy_block` (crates/horizon-client/src/game/native_hooks.rs:8244) | `&mut self, pos: [i32; 3], state: BlockStateId, canceled: &mut bool` | server | 14 | tools | One area. |
| `living_fall` | `horizon_client_lib::landing::native_living_fall` (crates/horizon-client/src/landing.rs:32) | `entity: u32, kind: &str, distance: &mut f64, multiplier: &mut f32, cancel: &mut bool` | both | 357 | tools | One area. The hook is in the client library, which has no `Game`: module `bmh_tools_lib` there. No `native_on` check: it acts only on falls that `player_tick` (gated) recorded. |
| `item_expire` | `horizon_client::game::native_hooks::Game<'_>::native_item_expire` (crates/horizon-client/src/game/native_hooks.rs:9002) | `&mut self, entity: u32, stack: &ItemStack, extra_life: &mut i32` | server | 325 | eternal | One area. |
| `item_spawn` | `horizon_client::game::native_hooks::Game<'_>::native_item_spawn` (crates/horizon-client/src/game/native_hooks.rs:5967) | `&mut self, entity: u32, item: &str, count: i32, pos: [f64; 3]` | server | 234 | eternal | One area. |
| `use_item` | `horizon_client::game::native_hooks::Game<'_>::native_use_item` (crates/horizon-client/src/game/native_hooks.rs:8004) | `&mut self, stack: &ItemStack, claimed: &mut bool` | server | 13 | eternal, amulet | Each area tests its own item. No order. |
| `use_item_on` | `horizon_client::game::native_hooks::Game<'_>::native_use_item_on` (crates/horizon-client/src/game/native_hooks.rs:7963) | `&mut self, stack: &ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool` | server | 13 | amulet | One area. |
| `interaction_key` | `horizon_client::game::native_hooks::Game<'_>::native_interaction_key` (crates/horizon-client/src/game/native_hooks.rs:2882) | `&mut self, button: i32, key_mapping: &str, hand: Hand, swing_hand: &mut bool, canceled: &mut bool` | client | 138 | amulet | One area. |
| `player_toggle_sneak` | `horizon_client::game::native_hooks::Game<'_>::native_player_toggle_sneak` (crates/horizon-client/src/game/native_hooks.rs:4521) | `&mut self, player: u128, name: &str, sneaking: bool` | server | 351 | amulet | One area. |
| `spawn_change` | `horizon_client::game::native_hooks::Game<'_>::native_spawn_change` (crates/horizon-client/src/game/native_hooks.rs:6323) | `&mut self, pos: [i32; 3], angle: [f32; 2]` | server | 244 | respawn, amulet | No shared state: each area keeps its own copy of the world spawn (respawn: the centre of an exile with no bed). |
| `level_tick_post` | `horizon_client::game::native_hooks::Game<'_>::native_level_tick_post` (crates/horizon-client/src/game/native_hooks.rs:2583) | `&mut self, dimension: &str` | server | 23 | respawn | One area. Called after each level's tick pass with that level in place (`enter_level`, levels.rs:528-560), so the landing search reads that level's blocks and `/playsound` reaches its players. |
| `player_spawn_point_set` | `horizon_client::game::native_hooks::Game<'_>::native_player_spawn_point_set` (crates/horizon-client/src/game/native_hooks.rs:5120) | `&mut self, player: u128, name: &str, dimension: &str, pos: [i32; 3], cause: &str, cancel: &mut bool` | server | 277 | respawn | One area. Respawn only listens (it never sets `cancel`): it keeps each player's respawn point for the search at death. |
| `use_duration` | `horizon_client::game::native_hooks::Game<'_>::native_use_duration` (crates/horizon-client/src/game/native_hooks.rs:8018) | `&self, stack: &ItemStack, duration: &mut i32` | both | 90 | ritual | One area. |
| `living_entity_use_item_start` | `horizon_client::game::native_hooks::Game<'_>::native_living_entity_use_item_start` (crates/horizon-client/src/game/native_hooks.rs:8052) | `&mut self, stack: &ItemStack, entity: Option<i32>, hand: Hand, duration: &mut i32, canceled: &mut bool` | both | 90 | ritual | One area. |
| `living_entity_use_item_tick` | `horizon_client::game::native_hooks::Game<'_>::native_living_entity_use_item_tick` (crates/horizon-client/src/game/native_hooks.rs:8116) | `&mut self, stack: &ItemStack, entity: Option<i32>, hand: Hand, duration: &mut i32, canceled: &mut bool` | both | 131 | ritual, sky | Each area tests its own item. No order. |
| `living_entity_use_item_stop` | `horizon_client::game::native_hooks::Game<'_>::native_living_entity_use_item_stop` (crates/horizon-client/src/game/native_hooks.rs:8160) | `&mut self, stack: &ItemStack, entity: Option<i32>, hand: Hand, duration: i32, canceled: &mut bool` | both | 131 | ritual | One area. |
| `living_entity_use_item_finish` | `horizon_client::game::native_hooks::Game<'_>::native_living_entity_use_item_finish` (crates/horizon-client/src/game/native_hooks.rs:8078) | `&mut self, stack: &ItemStack, entity: Option<i32>, hand: Hand, duration: i32, result: &mut Option<ItemStack>` | server | 90 | ritual | One area. |
| `on_use_tick` | `horizon_client::game::native_hooks::Game<'_>::native_on_use_tick` (crates/horizon-client/src/game/native_hooks.rs:8144) | `&mut self, stack: &ItemStack, entity: Option<i32>, remaining: i32` | both | 131 | ritual | One area. |
| `release_using` | `horizon_client::game::native_hooks::Game<'_>::native_release_using` (crates/horizon-client/src/game/native_hooks.rs:8229) | `&mut self, stack: &ItemStack, entity: Option<i32>, time_left: i32, used: &mut bool` | both | 131 | ritual | One area. |
| `render_hand_pose` | `horizon_client::game::native_hooks::Game<'_>::native_render_hand_pose` (crates/horizon-client/src/game/native_hooks.rs:3055) | `&self, hand: Hand, stack: Option<&ItemStack>, partial: f32, swing: f32, equip: f32, x: &mut f32, y: &mut f32, z: &mut f32, pitch: &mut f32, yaw: &mut f32, roll: &mut f32, scale: &mut f32` | client | 369 | ritual | One area. |
| `modify_default_components` | `horizon_world::item_defaults::native_modify_default_components` (crates/horizon-world/src/item_defaults.rs:77) | `item: &str, components: &mut Map<String, Value>` | both | 79 | totem, amulet | Each area changes its own item. No order. The hook is in horizon-world, which has no `Game`: modules `bmh_totem_world` and `bmh_amulet_world` there. No `native_on` check: the doc asks for the same answer each time, and the launcher leaves a mod that is off out of its next build. |
| `tooltip` | `horizon_client::game::native_hooks::Game<'_>::native_tooltip` (crates/horizon-client/src/game/native_hooks.rs:7882) | `&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>` | client | 12 | totem, tools, amulet | Each area adds lines for its own items. No order. |
| `item_bar` | `horizon_client::game::native_hooks::Game<'_>::native_item_bar` (crates/horizon-client/src/game/native_hooks.rs:7939) | `&self, stack: &ItemStack, bar: &mut Option<(i32, u32)>` | client | 60 | tools, amulet | Each area sets the bar of its own items. No order. |
| `inventory_move_item` | `horizon_client::game::native_hooks::Game<'_>::native_inventory_move_item` (crates/horizon-client/src/game/native_hooks.rs:6811) | `&self, source: [i32; 3], destination: [i32; 3], item: &str, cancel: &mut bool` | server | 334 | totem | One area. |
| `hopper_pickup_item` | `horizon_client::game::native_hooks::GameLevel<'_>::native_hopper_pickup_item` (crates/horizon-client/src/game/native_hooks.rs:1175) | `&self (GameLevel), dimension: LevelDimension, pos: BlockPos, item: &str, count: u32, cancel: &mut bool` | server | 232 | totem | One area. The hook is a method of `GameLevel`, which has no `Game`: no `native_on` check. The area must act only on state that gated hooks wrote. |
| `player_container_open` | `horizon_client::game::native_hooks::Game<'_>::native_player_container_open` (crates/horizon-client/src/game/native_hooks.rs:8559) | `&mut self, player: u128, name: &str, dimension: &str, menu: &str, pos: Option<[i32; 3]>` | server | 152 | totem | One area. |
| `player_container_close` | `horizon_client::game::native_hooks::Game<'_>::native_player_container_close` (crates/horizon-client/src/game/native_hooks.rs:8587) | `&mut self, player: u128, name: &str, dimension: &str, menu: &str, pos: Option<[i32; 3]>` | server | 152 | totem | One area. |
| `anvil_update` | `horizon_client::game::native_hooks::Game<'_>::native_anvil_update` (crates/horizon-client/src/game/native_hooks.rs:6067) | `&mut self, player: u128, name: &str, left: &ItemStack, right: Option<&ItemStack>, item_name: Option<&str>, vanilla_output: Option<&ItemStack>, vanilla_xp_cost: i32, vanilla_material_cost: i32, output: &mut Option<ItemStack>, xp_cost: &mut i32, material_cost: &mut i32, canceled: &mut bool` | both | pending (fragment anvil_update.json) | totem | One area (ARCHITECTURE.md gives the anvil to the totem area). |
| `world_primitives` | `horizon_client::game::native_hooks::Game<'_>::native_world_primitives` (crates/horizon-client/src/game/native_hooks.rs:2982) | `&self, partial: f32, camera: [f64; 3], primitives: &mut WorldPrimitives` | client | pending (fragment world_primitives.json) | tools | One area. |

Counts: 65 hooks. 62 stamped, 3 pending (`player_respawn_position`, `anvil_update`, `world_primitives`).

## Contracts

Where one area gives a value and another area reads it, the integrator fixed the shape before the areas start, so
the two owners can work at the same time. An owner fills the body of its side and keeps the signature. A change to
a contract is an integrator change (ARCHITECTURE.md, "Areas, and who owns which file"). The shared types are in
integrator files of `bmh_logic`: `ids.rs`, `wire.rs` (every mod message, as ARCHITECTURE.md says) and
`contract.rs`.

| Contract | Giver | Reader | Shape and path |
|---|---|---|---|
| Eternal Items | integrator | respawn (C1 count), eternal, tools, totem, amulet | `bmh_logic::ids::is_eternal(&str) -> bool` and `ids::ETERNAL_ITEMS`: the Java `IEternalItem` set (sword, axe, pickaxe, bundle, `totem_of_eternity`, `sleeping_totem`). The type of the "one of each type" rule is the item id. |
| Journey signals | respawn (server) | journey (client) | `bmh_logic::wire::JourneySignal` on `wire::JOURNEY_CHANNEL` (`bmh:journey`): `Departure` tag 1, `Ready` tag 2, `Arrival { name: Option<String> }` tag 3. |
| Held Respawn request | journey (client) | respawn (server, checks it, D7) | `bmh_logic::wire::RespawnRequest` on `wire::RESPAWN_CHANNEL` (`bmh:respawn`): `Held` tag 1. The server answers `JourneySignal::Ready` when the landing is already found. |
| Exile of a distant respawn | respawn | homecoming | `bmh_respawn::on_player_respawned` gives back `Option<bmh_logic::contract::Exile>` (`dimension`, `center`, `distance`). `hook_player_respawned.rs` hands it to `bmh_homecoming::on_exile(game, player, &exile)`. |
| Amulet points of a player | amulet | homecoming | `bmh_amulet::points(game, player) -> Vec<bmh_logic::contract::AmuletPoint>` (`name`, `pos`). |
| World rules and config values | settings | respawn, homecoming, eternal, tools, amulet | `bmh_settings::rule(game, bmh_logic::contract::Rule) -> bool` and `bmh_settings::config(game) -> bmh_logic::contract::Config` (`min_distance`, `max_distance`, `max_amulet_points`). Server side only. Both carry `#[allow(dead_code)]` until every reader is merged. |

### The /bmh command root

Two areas own subcommands of `/bmh`: settings owns `rule`, amulet owns `point`. The helpers
`hook_register_commands.rs` and `hook_perform_command.rs` already call both areas, settings first.
Neither area touches the file of the other. An area adds a subcommand like this:

- In `on_register_commands`, push one `NativeCommand` for each usage, and start each usage with `bmh <sub>`
  (`bmh point save <pos:block_pos> <name:text>`). The server merges all usages that share the word `bmh` into
  one tree, and each usage keeps its own operator level (`NativeCommand::new` for everyone, `op(usage, 2)` for
  operators). Do not push a bare `bmh` usage.
- In `on_perform_command`, split the line into words. If the first two words are not `bmh <sub>`, return at once
  and leave `ran` alone. If they are, set `*ran = true` and answer with `native_feedback`.
- Settings pushes `bmh rule`, `bmh rule <name:word>` and `bmh rule <name:word> <value:boolean>` at level 2 and
  runs them through `bmh_logic::settings::parse_rule_command`, which answers `NotMine` for any other line.
  Amulet needs nothing from settings.

A message between the two sides of one area (the Echo Sense glow of tools, the amulet screens) also goes in
`wire.rs`, in a section with the area's name, as an integrator change that the area names in its report.

## Content files (`horizon/content`)

The content mod is shared by every area. Each file has one owner:

| Files | Owner | Rule |
|---|---|---|
| `assets/bmh/lang/*.json` (ten languages) | integrator | An area does not edit them. It lists each new key and its English text in its LOG.md entry. The integrator adds the keys to `en_us.json` and to the nine other files. |
| `data/bmh/advancement/journey/exile.json`, `far_away.json`, `here_we_go_again.json` | respawn | The area that awards an advancement (`minecraft:impossible`, criterion `done`) owns its file. |
| `data/bmh/advancement/journey/home_at_last.json`, `long_way_home.json` | homecoming | |
| `data/bmh/advancement/eternity/saved.json` | totem | |
| `data/bmh/advancement/eternity/moon_ritual.json` | ritual | |
| `data/bmh/advancement/eternity/echo_strike.json`, `smash.json`, `echo_sense.json` | tools | |
| `data/bmh/advancement/navigation/stargazer.json`, `named_place.json`, `own_sky.json` | amulet | |
| `assets/bmh/sounds.json`, `assets/bmh/sounds/` | journey | Respawn and homecoming play the events that exist (`bmh:bell_sfx`, `bmh:end`) and do not edit these files. |
| All other files: `advancement/root.json`, the `inventory_changed` advancements, items, models, textures, recipes, tags, `data/bmh/horizon/`, `horizon-mod.toml`, `NOTES.md` | integrator | |

## Hooks GAPS.md names that are not wired, and why

| Name | GAPS rows | Why it is not wired |
|---|---|---|
| `server_packet` (pending, `tools/native-hook-fragments/server_packet.json`, native_hooks.rs:7554) | C4 (option) | GAPS marks it as an option of the C4 route. C4 ships at N without it: the client holds the death screen, and the server says "ready" by a mod message. |
| `Game::guest_respawn` (lan_actions.rs:3892) | C4 | Tier X: a function of the game, not a hook point (decision D8). |
| `Game.level.hardcore` read (save.rs:3733, field `hardcore`) | C5 | Tier X (decision D8). The substitute reads `hardcore` from `level.dat` with the `world_dir` helper. |
| `Game::update_totem_ready`, `apply_totem`, `find_totem` (totem.rs:52, :66, :178) | T4 | Tier X (decision D8). The substitute plays the totem sound and particles with `run_command`. |
| hand light (frame_geometry.rs) | T8 | Tier X (decision D8). The substitute is `render_hand_pose`, which is wired. |
| `has_foil` (horizon-assets item_model.rs:1175) | T10 | Tier X (decision D8). The substitute shows no glint while the ritual charges. |
| `dropper_insert` | T13 | Tier X (decision D8). No function of that name in the Nightly (searched `fn dropper_insert` in every crate). |
| `Game::world_spawn_position` | A2 | Tier X (decision D8). No function of that name in the Nightly (searched `fn world_spawn_position`); GAPS names the field `level.world_spawn` instead. The substitute reads `level.dat` and listens to `spawn_change`, which is wired. |
| `Game::attack_mob` (game.rs:11976) | E7 (option) | Tier X (decision D8). The substitute estimates the strength from `player_attack_cooldown_reset`, which is wired. |
| `start_mod_use` | T15 | Not a hook point: an internal method of `Game` (item_use.rs:94) that consumes the click when `use_duration` answers above 0. T15 uses `use_duration`, which is wired. |

The helpers of GAPS.md section 4 (`run_command`, `carried_stack`, `net_send_to_player`, `level_difficulty` and so
on) are functions that a module calls. They need no injection, so they are not in the tables above.
