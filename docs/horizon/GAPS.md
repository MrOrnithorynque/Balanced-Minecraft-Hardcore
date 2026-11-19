# BMH on Horizon 26.3: the gap list

Inputs: the five feasibility audits (`audit/1-respawn-server.md` to `audit/5-platform.md`), their verifications (`audit/verify-1.md` to `audit/verify-5.md`), the single-player owner audit (`audit/6-single-player-owner.md`) and `horizon/content/NOTES.md`. Where two documents disagree, this file read the Horizon source and decided. The decisions are in "Settled disagreements" at the end.

Short names. `v1` to `v5` = `audit/verify-1.md` to `verify-5.md`, followed by the row id. `a1` to `a6` = the audits. `NOTES` = `horizon/content/NOTES.md`. `H` = the Horizon clone (main b880384691b88d317553067109d28e785990bda3). `G/` = `H/crates/horizon-client/src/game/`. `tag` = the Nightly source `nightly-source/2026-11-08-fdc87ae` (commit fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca). Line numbers are main's unless the text says `tag`.

Tiers (from the verification brief): N = every hook and helper is in the Nightly 2026-11-08 catalog. M = stamped in main's catalog only. F = a pending fragment, compiled in, with no API number. X = an injection into one of the game's own functions. G = not buildable without a change in Horizon.

Tier rule of this file: a row's tier is the worst tier that the Java behaviour needs, as the verify files rate it. When the recommended build ships at a lower tier with a difference, the "How it is built" cell starts with "Ships at N:".

## 1. Initial feasibility verdict

1. BMH can be ported. 54 of the 59 player-visible features can be built, 36 of them with Nightly hooks only.
2. Build on the Nightly commit fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca with a pinned `horizon_revision`. The core (the distant respawn point) needs the pending hook `player_respawn_position`, which that Nightly compiles but does not stamp.
3. Five features cannot be built as in Java (G): the six `bmh:*` game rules, the Create World button inside the Game tab, one Totem of Eternity per container in vanilla menus and in chest minecarts or boats, and the amulet's cooldown sweep (this one is drawn by the mod since 2026-11-19, see A4). Each one ships with a substitute.
4. Seven features need an injection (X) for parity: the respawn hold on every path, the hardcore check, the totem animation, the ritual hand, the ritual glint, dropper insertion and the amulet's world spawn line. Each one also has a substitute that ships without the injection.
5. Nothing can run until the owner plays Minecraft 26.3 once and supplies the 26.3 server jar (v5 D1).

## 2. Feature matrix

| # | Feature | Tier | How it is built (hooks and helpers by name) | What differs from the Java mod | Source |
|---|---|---|---|---|---|
| C1 | Respawn distance at death: Eternal Item count, difficulty, config min and max | F | Count Eternal slots in `loot_drops` (player inventory, API 405), else a cache from `player_tick` with `carried_stack`. Difficulty: pending `level_difficulty` (`level_world`). Ships at N: `difficulty_changed` hears changes only. | With `keep_inventory` the count comes from the tick cache. | v1 1a |
| C2 | Distant respawn point, chosen before the client sees any position; bed, anchor or world spawn centre; Nether anchor divided by 8 | F | Pending hook `player_respawn_position` (raw `target`), called before `hub.respawn`. Exile record in a map, read in `player_respawned`. | A destroyed bed hands over the old bed position; the mod must test the block. The Nether anchor works in single player too (see Settled 1). | v1 1b, 1e; a6 V1 |
| C3 | Landing search in the background on far unloaded chunks: 32 candidates, no ocean or river, safe ground, inside the border | F | `force_chunk_in` (stamped API 413 on main; pending fragment `chunk_tickets_in_level` on the tag), `surface_heights`, `read_columns`, `level_height`, driven from `server_tick`. Border: pending `world_border`. | No cheap biome pre-filter: each rejected ocean candidate loads a 5x5 chunk ring. Leaves, sturdy-face and biome tags are hard-coded lists. Water depth comes from `OceanFloor` heights after the load. | v1 1c |
| C4 | Hold the Respawn request until the landing is ready | X | Ships at N: the client cancels the death screen click and keys (`screen_mouse_button_pressed_pre`, `screen_key_pressed_pre`) and sends `ClientCommand{action:0}` with `send_packet` when the server says "ready" by `net_send_to_player`. Exact: cancellable HEAD on `Game::guest_respawn` (G/lan_actions.rs:4019) plus pending `respawn_player`. | A gamepad click, the `immediate_respawn` rule (G/remote.rs:4634) and a client without the mod skip the client hold. Then C2 falls back to the vanilla point or to an unfinished search. | v1 1d; v4 1c |
| C5 | Hardcore world: no distant respawn and no journey | X | Mixin helper reads `self.level.hardcore` (field at `H/crates/horizon-client/src/save.rs:3989`). Ships at N: read `hardcore` from `level.dat` in `world_dir` (written at save.rs:4707). | Without a hardcore read, `player_respawn_position` runs before the hardcore spectator switch, so the mod would exile a hardcore player. | v1 1a; src save.rs:3989, :4117, :4707 |
| C6 | 200 ticks of damage immunity after a distant respawn | N | `living_incoming_damage` cancel, keyed by `acting_player().entity_id`; countdown in `player_tick`. | Bypass damage types (void, `/kill`) are a hard-coded list. | v1 2a |
| C7 | Bell and respawn sound for the other players nearby | N | `run_command("playsound ... @a[name=!X,distance=..16]")` from `server_tick`. | One tick late. Range 16 blocks as `/playsound`. | v1 2b |
| C8 | Journey signals: departure at death, arrival with the optional message | N | `living_death` or `loot_drops` and `player_respawned` on the server, `net_send_to_player`; `net_from_server` on the client. | No title packet trick. A client without the mod sees nothing. | v1 2c; v4 1b |
| C9 | Journey advancements: exile, far away, here we go again (tenth death) | F | `run_command("advancement grant <name> only bmh:journey/... done")` from `server_tick`. Death count: pending `player_stat` (`player_records`). Ships at N: count `player_died` in an attachment. | The N count starts at install. | v1 2d; a1 2 |
| C10 | Homecoming: arrival within 24 blocks of a home, title, subtitle, bell, two advancements | N | Attachment with `copy_on_death`, `server_tick` at 1 Hz, `server_players`, `level_game_time`, `run_command` for `title`, `playsound`, `advancement grant`. | None. The record survives the End exit (v1 1e note). | v1 3 |
| C11 | Eternal Items kept through death | N | Take the stacks out of `loot_drops` (API 405), give them back with `give_stack` in `player_respawned`. | Vanishing curse items are gone before the hook. With `keep_inventory` the hook is not heard (items stay, no wear). | v1 8b; v3 6 |
| C12 | 7 percent death wear of Eternal tools | N | Edit `stack.damage` inside `loot_drops`. | No wear with `keep_inventory`. | v3 3d |
| C13 | Wake-up rewards: heal, food cost, Haste II and Resistance II | F | `player_bed_leave`; pending `player_food`, `set_player_status` (`player_status`) and `add_effect` (`player_effects`). | The N fallback (`effect give instant_health`) heals 4 or 8, not 6 or 3, and cannot lower food. Not shipped. | v1 8a |
| C14 | Hide coordinates in survival (`reduced_debug_info`) | F | Force with `run_command("gamerule reduced_debug_info true")` from `server_tick`. Restore: pending `level_game_rule`. Ships at N: restore to false. | An operator's own value is lost at restore. | v1 8d |
| C15 | Journey cinematic: fade, night sky, star chain, message, pixel dissolve | N | `gui_render_post` with `fill`, `quad`, `gui_paint`, `text_width`. | The gradient is banded fills. Text with alpha goes through `gui_paint`. | v4 1a |
| C16 | Black screen held across the respawn load; input held | N | `gui_render_post` also fires over the level-loading screen of a respawn. `mod_screen_open`, retried each frame, holds input after the load. | None expected. Not proven by a run. | v4 1d |
| C17 | Music fade, star chimes, bell at the end | N | `music` hook `volume`; `play_local_sound`. | No sound category: the Players slider does not scale them. The fade steps at 20 Hz. | v4 1e |
| T1 | Totem items: Totem of Eternity, Dormant, Hollow, Echo Heart; ids, names, rarity, fire resistance, glint, tooltip lines | N | Content mod item JSON (already in `horizon/content`), `tooltip` hook. | None. | v2 2; v3 1a; NOTES |
| T2 | Save from death: a Totem of Eternity anywhere in the inventory saves at 1 health and becomes a Dormant Totem in the same slot | N | Route A (2026-11-19, `docs/horizon/solutions/totem-save.md`): the content item carries `minecraft:death_protection` with the three Java effects, so Horizon's own hand check saves the player. The `player_totem` hook swaps the hands when a Totem of Undying is in the off hand (it saves first, as in Java) or records the slot; the end of the server tick puts the Dormant Totem there. The old `living_death` path stays. | Only a Totem of Eternity in a hand saves: one in another slot does not (Java: any slot). The reason: Horizon calls `living_death` for a player only when trait mods exist (`guest_die`, `lan_actions.rs:4293-4296`), and BMH has none (D2). Any slot needs an upstream change or an injection (proposal in the solution file, D8). A mob that holds a Totem of Eternity is saved by it once (Java: never). Bypass types are a hard-coded list. Checks t1 to t5, t11 (off hand) and t12 (Undying first) pass in the game on 2026-11-19; t13 shows the gap. | v2 1a; v4 5a; LOG.md step 11; totem-save.md |
| T3 | Save effects | N | The game applies the death protection component of the held totem. | The effects apply on the lethal hit. | See `solutions/totem-save.md`. |
| T4 | Save animation shows the Totem of Eternity | X | The game sends event 35 and the consumed hand stack. | A held Totem of Eternity shows its own item. | See `solutions/totem-save.md`. |
| T5 | Totem of Undying gives no Fire Resistance | N | `modify_default_components` on `minecraft:totem_of_undying`. | Mobs holding a totem lose it too (they read the same component). | v2 1d |
| T6 | Anvil awakening: Hollow Totem plus Echo Heart for 39 levels | F | Pending hook `anvil_update` (called at G/lan_stations.rs:200). | None once wired. | v2 3 |
| T7 | Moon ritual: start rules, aim at the moon, drain 30 levels, fail reasons, 20-tick fail cooldown, wake | F | `use_duration`, `living_entity_use_item_start`, `_tick`, `_stop`, `_finish`, `on_use_tick`, `release_using`, `give_levels`, `action_bar`, `minecraft:use_cooldown` component. XP level read: pending `server_player_state`. Blindness and Darkness test: pending `player_effects`. | The client predicts the start. The host uses its damage event for interruptions. The item-use statistic and the third-person bow pose differ. | v2 4, 7 |
| T8 | Ritual first-person look: the totem alone, full bright, trembling | X | Ships at N: `render_hand_pose` offsets. Exact: the hand light at G/frame_geometry.rs:469 and an arm-only hide, no hook. | The arm holds the totem; the totem is lit by the night light. | v2 5a |
| T9 | Ritual runes flying into the totem | N | `run_command("particle enchant ...")` per viewer from `server_tick`; `/particle` takes a `viewers` selector (G/commands/world_cmds.rs:888-892). | One tick late, server positions. | v2 5b; src world_cmds.rs:751, :888 |
| T10 | Dormant Totem glint while it charges | X | Exact: a per-stack foil read at `H/crates/horizon-assets/src/item_model.rs:1216`. Ships at N: no glint while charging. | No glint during the ritual. | v2 5b |
| T11 | One Totem of Eternity per player: extras dropped, pickup refused | N | `player_tick` scan with `carried_stack`, `summon` of `minecraft:item`; `item_picked_up` cancel. | A dropped extra falls in place, not thrown forward. | v2 6a; v1 8c |
| T12 | One active Totem per vanilla container | G | The mod reads the open menu after a click or drag and returns the extra stack. | A rapid close or simultaneous clicks can escape the check or return a stack to the wrong player. | See `solutions/totem-containers.md`. |
| T13 | One active Totem per hopper or dropper destination | X | A guard in the pinned `Game::add_item` checks the destination before the transfer. | A rejected stack stays at its source. | See `solutions/totem-containers.md`. |
| T14 | One active Totem per chest vehicle | G | The menu check reads the vehicle inventory. The transfer guard checks the same inventory for automation. | Manual transfers have the T12 timing limits. | See `solutions/totem-containers.md`. |
| T15 | A use of the ritual item consumes the click (no off-hand double use) | N | `start_mod_use` consumes the click on both sides. | None. | v2 7 |
| E1 | Eternal sword, axe, pickaxe: 33000 durability, damage, speed, enchantability, repair material | N | Content mod item JSON on netherite templates. | The 26.3 attribute modifier ids are UNVERIFIED. | v3 1a, 1b; NOTES |
| E2 | Eternal tools never break (keep the last point) | N | `item_damaged` caps `amount`; damage read with `carried_stack` on the slot that `inventory_tick` marks `selected`. | None. | v3 3a |
| E3 | Worn tools deal half damage | N | The attack hook records the weapon and primary target. The incoming damage hook halves that hit. | The damage hook runs after the armadillo-specific reduction. | v3 3b |
| E4 | Worn-out tool: stone mining speed | N | The `break_speed` hook applies stone speed. The server sends its mineable tag answer on `bmh:tools`. | The first crack on a guest can precede the tag answer from the server. | v3 3c |
| E5 | Tooltip lines (ability, worn out, heavy, death wear) and the 1-pixel bar | N | `tooltip` hook, `item_bar` hook. | None. | v3 3e |
| E6 | Gold block repair at the anvil: flat 15 levels, prior work penalty kept | F | Pending hook `anvil_update`. | None once wired. | v2 3 |
| E7 | Echo Strike | F | The attack hook reads the actual strength before the swing resets. A confirmed hit queues an echo. | Echo Strike triggers on mobs. The mod identifies enemies by type. | v3 4a |
| E8 | Axe Smash | N | The incoming damage hook scales a primary axe hit from the fall distance recorded on player ticks. | Axe Smash triggers on mobs. The mod estimates fall distance. | v3 4b |
| E9 | Echo Sense (pickaxe): ores glow through walls for 5 s | F | `destroy_block`, `level_block_state`, own ore list; positions sent to nearby players with `net_send_to_player`; client draws with pending `world_primitives` boxes `.through_walls()`. | A box outline, not the entity silhouette. Only players with the mod see it. | v3 4c |
| E10 | Eternal Items on the ground never despawn and resist damage | N | The expiry hook extends the lifetime. The spawn hook raises the health of dropped Eternal Items. Content declares fire resistance. | Cactus and explosions still damage dropped items. A drop that bypasses the spawn hook can keep its original health. | v3 5 |
| E11 | One Eternal Item per type: extras dropped, pickup refused | N | `inventory_tick` with `carried_stack`, `summon` of `minecraft:item`, `item_picked_up` cancel. | None. | v1 8c; v3 6b |
| E12 | Eternal Bundle: refuses Eternal Items and ender chests, drops all at once | N | Template `minecraft:bundle`; `inventory_tick` moves a refused stack out of `bundle_contents`; `use_item` claims the click and drops all with `summon`. | A refused item sits in the bundle for one tick. | v3 7 |
| A1 | Navigator's Amulet item, needle to the north, charge bar, tooltip | N | Content item on the `minecraft:compass` template with a `lodestone_tracker` target; `item_bar`, `tooltip`. Fallback for the tracker: `modify_default_components`. | The needle aims at a fixed far block: up to 1.9 degrees off at x = 1 000 000. | v4 4; NOTES |
| A2 | Star reading at night: sky rules, charges refilled each night, chime, reading on the stack, stargazer advancement, world spawn line | X | `use_item`, `carried_stack`, `day_time`, `weather`, `surface_heights`, `action_bar`, `run_command`. Spawn line: mixin helper calls `Game::world_spawn_position` (G/lan.rs:1254). Blindness and Darkness test: pending `player_effects`. Ships at N: spawn read from `level.dat` `spawn.pos` (save.rs:4099-4114) plus `spawn_change`. | Without X the spawn line may lag a `/setworldspawn` until the next save. | v4 5a; src lan.rs:1254, save.rs:4099 |
| A3 | Amulet clicks: sneak on a bed or lodestone, no double use, no off-hand use | N | `use_item_on` claim; drop a `use_item` in the same tick as a claimed `use_item_on`; client cancels the off hand in `interaction_key`; sneak from `player_toggle_sneak`. | None if the filters are right. | v4 5a |
| A4 | Cooldown sweep on the hotbar after a reading or a point change | N | The mod keeps its own per-player timer and draws the white sweep itself: `item_decorations` plus `AmuletSignal::Cooldown`. No item cooldown setter exists (a `use_cooldown` component does not start on a claimed click). Proof: `docs/horizon/solutions/amulet-cooldown.md`, scenario cooldown. | The sweep steps once per tick (no partial tick); the game's own cooldown table stays empty. | v4 5a |
| A5 | Named points on beds and lodestones: save, rename, remove, cap from config, survive death, two advancements | N | `attachments` on `Holder::Player`, `copy_on_death`; checks from `net_from_client` with `server_players`, `entity`, `living_equipment`; or `/bmh point` with `register_commands`, `perform_command`. | Players can use `/bmh point` at level 0. The rule subcommand checks operator permission separately. Relative coordinates can differ. | v4 5b, 5c; v1 6 |
| A6 | Naming screen and point list screen | N | `mod_screen_open`, `screen_render_post`, `screen_mouse_button_pressed_pre`, `screen_key_pressed_pre`, `screen_char_typed_pre`, `gui_nine_slice`, `net_send_to_server`. | Hand-written text field: no clipboard, own caret feel. A gamepad cannot click it. | v4 5c |
| V1 | Star reading drawn in the sky | N | `world_drawn` with `render_needs.world_depth`, own wgpu pipeline, depth test against the sky; star angle recomputed from `clock`. | Weather and clouds can appear behind a line. Generated frames can omit the reading. | v4 2 |
| V2 | Line name through the spyglass | N | Read the same first-person and scope state as the game. Draw the name under the center in `gui_render_post`. | The label appears over the world and chat. Menus hide it. | v4 3 |
| V3 | Models, textures, the sword's two sprites, sounds, lang, recipes, tags | N | Content mod data and assets (already in `horizon/content`). | None. | v3 2, 8; NOTES |
| V4 | Creative tab | N | Content `creative_tab/*.json`. | The tab sits after the vanilla tabs, in pages of ten. | v3 8 |
| V5 | Advancement tab (20 entries, one hidden) | N | Content mod `data/bmh/advancement/`; awards by `run_command("advancement grant ...")`. | None. The toast on the command path is UNVERIFIED. | v1 7; src docs/mods.md:31 |
| S1 | Six `bmh:*` game rules, in `/gamerule`, saved with the world | G | The world keeps six values in `bmh/rules.txt`. Players can use the `/gamerule bmh:<name>` alias. Operators and the console can use `/bmh rule`. | The vanilla rule list, functions and command blocks do not include the alias. A Java world imports its BMH values once. | v1 4; v5 G4 |
| S2 | "Balanced Hardcore" button on Create New World | G | The Game tab shows a button. Re-Create copies all six saved values. A click changes them together. | The More tab omits the six mod rules. The button is drawn over the Game tab. | v4 6; v5 G4 |
| S3 | Config file `config/bmh-common.toml` (min and max distance, max points) | N | `register_configs` (Common), `config_loading`, `config_reloading`. | None. | v1 5a |
| S4 | Config screen | N | Options > Mods page: `settings`, `settings_click` stepper buttons. A text-field screen can open only in the world (`mod_screen_open`). | No free typing on the Mods page. | v1 5b; v4 7 |

The original feasibility audit classified the 59 rows as N 36, M 0, F 11, X 7 and G 5.

## 3. Not identical

### 3.1 Cannot be built (G)

- S1. Horizon keeps a static rule table. The mod stores six values per world and handles its own game rule commands.
- S2. The button appears on the Game tab and preserves mixed values on Re-Create until the player clicks it.
- T12. The mod recovers extra totems after a menu click or drag.
- T14. The open menu exposes the chest vehicle inventory. Machine transfers check the destination before insertion.
- A4, cooldown sweep. No item cooldown setter exists in a catalog or fragment. Substitute: a mod timer, and the mod draws the white sweep with `item_decorations` (solved at N, `docs/horizon/solutions/amulet-cooldown.md`). The upstream ask in section 6 now only removes the small differences.

### 3.2 Built another way (substitute design)

- C4, the respawn hold. The client cancels the death screen click and waits for a server "ready" message. A gamepad click and `immediate_respawn` skip it. Best fix: the X injection on `guest_respawn`.
- C5, hardcore. Without the X field read, the mod reads `level.dat`. A brand new world may have no `level.dat` yet (open question).
- C9, the tenth death. Until `player_stat` is stamped, deaths count from install.
- C14, coordinates. The restore sets `false` until `level_game_rule` is stamped.
- T2 and T3, the totem save. Only a Totem of Eternity in a hand saves (route A). The effects come at once; the Dormant Totem lands at the end of the same server tick.
- T4. The game shows the held protector and plays the save effects.
- T9, runes. Sent by the server through `/particle`, one tick late.
- T13. The pinned `Game::add_item` guard covers dropper insertion.
- E7. The mod reads attack strength from the host at the start of the attack. The echo damage uses the pending `hurt_entity` helper.
- E9, Echo Sense. Box outlines through walls on clients with the mod, in place of glowing block displays.
- A2, the spawn line. From `level.dat` until the X helper.
- S4, config screen. Stepper buttons on the Mods page.

### 3.3 Built, but looks or feels different

- C2 (respawn fix round 1): once the mod moves the respawn, Horizon skips the bed and anchor resolution. The mod
  resolves the point at death itself (anchor charge and Nether, bed and Overworld, room at a forced point), uses the
  anchor charge and tells `block.minecraft.spawn.not_valid` after the arrival. Three parts stay different: no
  stand-up search (an obstructed bed or anchor still counts as one that works), a point that does not work is not
  forgotten (no helper clears a respawn point), and in a world never saved the first circle with no point is centred
  on level.dat's placeholder world spawn (no helper reads the world spawn; the game's one is learned at that respawn).
- C8 (respawn fix round 1): `Departure` reaches the client after the death screen packet. Horizon sends
  `PlayerCombatKill` at once and mod messages at the next flush, so no hook can put a message before it.
- C3: a search can take longer (no cheap ocean filter, 25 chunks per candidate).
- C6, T2: void and `/kill` types come from a hard-coded list.
- C7, C17: sounds one tick late; journey chimes ignore the Players slider.
- C15: banded sky gradient.
- T5: mobs' Totems of Undying lose Fire Resistance too.
- T7: a client that predicts a start the server refuses shows the ritual for a moment.
- T8: first person shows the arm holding the totem, lit by the night.
- T10: no glint while the ritual charges.
- E3: PvP hits with a worn-out tool are not halved.
- E8: fall distance is approximate.
- E10: explosions slowly drain a dropped Eternal Item.
- E12: a refused item flickers in the bundle for one tick.
- A1: the needle can be off by up to 1.9 degrees very far from the origin.
- A6: no clipboard in the naming field; a gamepad cannot click the screen.
- V1: the star lines can draw over clouds and rain.
- V4: the creative tab sits after the vanilla tabs.

## 4. Hooks and helpers needed

Tier is for the Nightly pin. API numbers come from the doc comments in `G/native_hooks.rs` and `H/docs/native-mods.md`; "catalog" means the name is in the catalog and no number was cited.

| Name | Hook or helper | Tier | API | Used by |
|---|---|---|---|---|
| `living_death` | hook | N | 392 | C8, T2 |
| `loot_drops` (dying player's inventory) | hook | N | 75, extension 405 | C1, C8, C11, C12 |
| `player_died` | hook | N | 40 | C9 |
| `player_tick` | hook | N | 31 | C1, C6, T2, T11, E8 |
| `player_totem` | hook | N | 182 | T2 |
| `player_respawned` | hook | N | 115 | C2, C8, C11 |
| `server_tick` | hook | N | 4 | C3, C7, C9, C10, C14, T3, T9, E7 |
| `living_incoming_damage` | hook | N | 50 | C6 |
| `difficulty_changed` | hook | N | 159 | C1 |
| `net_from_server`, `net_from_client` | hooks | N | 40 | C4, C8, A5, A6, E9 |
| `screen_mouse_button_pressed_pre` | hook | N | 80, 326 | C4, A6, S2 |
| `screen_key_pressed_pre`, `screen_char_typed_pre` | hooks | N | 104 | C4, A6 |
| `screen_render_post` | hook | N | 78, 326 | A6, S2 |
| `gui_render_post` | hook | N | 410 | C15, C16, V2 |
| `music` | hook | N | 119 | C17 |
| `world_drawn`, `render_needs` | hooks | N | 58 (world_drawn) | V1 |
| `register_configs`, `config_loading`, `config_reloading` | hooks | N | 397 | S3 |
| `register_commands`, `perform_command` | hooks | N | 8 | A5, S1 |
| `settings`, `settings_click` | hooks | N | catalog | S4 |
| `world_lifecycle` | hook | N | 3 | S2 |
| `player_bed_leave` | hook | N | 252 | C13 |
| `inventory_tick` | hook | N | 39 | E2, E4, E11, E12, A2 |
| `item_picked_up` | hook | N | 40 | T11, E11 |
| `item_damaged` | hook | N | 162 | E2 |
| `hurt_enemy` | hook | N | 39 | E7, E8 |
| `attack_entity` | hook | N | 22 | E3, E7, E8 |
| `entity_join_level` | hook | N | 47 | E7 |
| `destroy_block` | hook | N | 14 | E9 |
| `living_fall` | hook | N | 357 | E8 |
| `item_expire`, `item_spawn` | hooks | N | 325, 234 | E10 |
| `use_item`, `use_item_on` | hooks | N | 13 | A2, A3, E12 |
| `interaction_key` | hook | N | 138 | A3 |
| `player_toggle_sneak` | hook | N | 351 | A3 |
| `spawn_change` | hook | N | 244 | A2 |
| `use_duration`, `living_entity_use_item_start`, `living_entity_use_item_finish` | hooks | N | 90 | T7, T15 |
| `living_entity_use_item_tick`, `living_entity_use_item_stop`, `on_use_tick`, `release_using` | hooks | N | 131 | T7, V2 |
| `render_hand_pose` | hook | N | 369 | T8 |
| `modify_default_components` | hook | N | 79 | T5, A1 |
| `tooltip` | hook | N | 12 | T1, E5, A1 |
| `item_bar` | hook | N | 60 | E5, A1 |
| `inventory_move_item`, `hopper_pickup_item` | hooks | N | 334, 232 | T13 |
| `player_container_open`, `player_container_close` | hooks | N | 152 | T12 |
| `run_command` | helper | N | 21 | C7, C9, C10, C14, T3, T4, T9, E7, E8, E9, V5 |
| `acting_player`, `action_bar` | helpers | N | 21 | C6, T2, T7, A2, A5 |
| `server_players` | helper | N | catalog | C10, C14, A5, E9 |
| `attachments` | helper | N | catalog | C9, C10, A5, S1, S2 |
| `give_stack` | helper | N | 98 | C11, T12 |
| `carried_stack` | helper | N | 39 | C1, T2, T11, E2, E4, E11, E12, A2 |
| `give_levels` | helper | N | 101 | T7 |
| `summon` | helper | N | 59 | T11, E11, E12 |
| `entity_merge_nbt` | helper | N | 386 | E10 |
| `surface_heights`, `read_columns` | helpers | N | 127 | C3, A2 |
| `level_height`, `level_game_time` | helpers | N | 381 | C3, C10 |
| `level_block_state` | helper | N | catalog | C3, E9, A5 |
| `entity_on_ground` | helper | N | 379 | E8 |
| `entity`, `entity_position`, `living`, `living_equipment` | helpers | N | 373, 382 | E7, A5 |
| `world_dir` | helper | N | catalog | C5, A2 |
| `net_send_to_player`, `net_send_to_server` | helpers | N | 40 | C4, C8, A6, E9 |
| `send_packet` | helper | N | 409 | C4 |
| `mod_screen_open`, `mod_screen_close`, `mod_screen` | helpers | N | 111 | C16, A6, S4 |
| `fill`, `quad`, `text`, `gui_paint`, `text_width`, `gui_nine_slice`, `sprite` | helpers | N | 68, 111, 19 | C15, A6, V2, S2 |
| `play_local_sound` | helper | N | 83 | C17 |
| `camera`, `clock`, `day_time`, `weather`, `window` | helpers | N | 109, 92, 97 | T7, A2, V1 |
| `setting`, `set_setting` | helpers | N | catalog | S2, S4 |
| `extract_item` | helper | N | 48 | T12 |
| `container_slots` | Game method, mixin code only (not in either catalog) | N | 48 | T12 |
| `force_chunk_in` | helper | M (F on the tag: `chunk_tickets_in_level`) | 413 on main | C3 |
| `player_respawn_position` | hook | F | none | C2, C4 |
| `level_difficulty`, `level_game_rule` (`level_world`) | helpers | F | none | C1, C14 |
| `world_border` | helper | F | none | C3 |
| `respawn_player` | helper | F | none | C4 |
| `server_packet` | hook | F | none | C4 (option) |
| `player_food`, `set_player_status` (`player_status`) | helpers | F | none | C13 |
| `add_effect`, `effects` (`player_effects`) | helpers | F | none | C13, T7, A2 |
| `player_stat` (`player_records`) | helper | F | none | C9 |
| `anvil_update` | hook | F | none | T6, E6 |
| `server_player_state` (`xp_level` field) | struct fields | F | none | T7 |
| `hurt_entity` | helper | F | none | E7 |
| `world_primitives` | hook | F | none | E9 |
| `Game::guest_respawn` (lan_actions.rs:4019; tag :3892) | injection | X | none | C4 |
| `Game.level.hardcore` read (save.rs:3989) | mixin helper | X | none | C5 |
| `Game::update_totem_ready`, `apply_totem`, `find_totem` (totem.rs:53, :67, :166; tag :52, :66, :178) | injections | X | none | T4 |
| hand light (frame_geometry.rs:469) | injection | X | none | T8 |
| `has_foil` (item_model.rs:1216; tag :1175) | injection | X | none | T10 |
| `Game::add_item` (machines.rs on the pin) | injection | X | none | T13 |
| `Game::world_spawn_position` (lan.rs:1254; absent on the tag, field `level.world_spawn` instead) | mixin helper | X | none | A2 |
| `Game::attack_mob` (melee.rs:200; tag game.rs:11976) | injection | X (option) | none | E7 exact strength |

Highest stamped API number needed: 410 (`gui_render_post`) on the Nightly pin. On main it would be 413 (`force_chunk_in`). Set `hooks` to 410.

Pending fragments needed (each one forces a pinned commit): `player_respawn_position`, `level_world`, `world_border`, `chunk_tickets_in_level` (tag only; stamped on main), `respawn_player`, `player_status`, `player_effects`, `player_records`, `anvil_update`, `server_player_state`, `hurt_entity`, `world_primitives`. Optional: `server_packet`, `session_state`, `break_speed`, `host_bridge`, `send_particles_to`, `client_particles`, `entities_in`. All of them are in the tag's source (checked with `git cat-file` on `tools/native-hook-fragments/<name>.json`).

## 5. Pin

Build on the Nightly commit fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca (API 411), not on main b880384691b88d317553067109d28e785990bda3.

Why:
1. The docs ask for a Nightly's commit. `horizon-docs/mod-manifest.txt:84-90` says a mod on the game's own functions pins "a Nightly's" commit, and "Every Nightly's commit is kept". Main b880384 is not a Nightly. Whether the launcher builds a mod pinned to it is UNVERIFIED (v5 B1).
2. The tag compiles every pending fragment that the F rows need. `git cat-file` finds all twelve files in the tag's `tools/native-hook-fragments/`. The respawn hook is called at tag `lan_actions.rs:3960`, `anvil_update` at tag `lan_stations.rs:200`. `force_chunk_in` is the pending fragment `chunk_tickets_in_level` there.
3. The tag's generator reads the fragments (tag `crates/horizon-mod-api-gen/src/build.rs:55`), so pending helpers are on `Host`.
4. The owner analysis holds on the tag (a6 V10; v3 V0 check).

Costs:
- Each pending hook is a raw `target` with the tag's function path; `hook =` fails (v5 B2). Pending signatures can change before they are stamped.
- The verify files cite main's lines. A porting agent must re-read each target on the tag. Two X targets have other names there: `world_spawn_position` (use the field `level.world_spawn`, tag save.rs:3729) and `dropper_insert`.
- `native.py` builds a pinned commit; the player then plays that Nightly with the mod.

Use main instead only if a fragment the port needs changes shape between the tag and main in a way that matters, or a main fix is needed.

To move to named hooks on a later Nightly:
1. Horizon stamps the twelve fragments of section 4 and a Nightly ships them.
2. Each X row either gets the upstream hook of section 6, or ships its N substitute without the injection.
3. Then BMH drops `horizon_revision`, uses `hook =` names, and sets `hooks` to the highest number it uses. One remaining raw `target` keeps the pin.

## 6. Upstream asks

G rows:
- S1: a hook to register mod game rules (namespaced, boolean and integer), kept in `game_rules.dat` and shown by `/gamerule`. Model: vanilla `GameRules` registration (26.1 registry `Registries.GAME_RULE`). The save must keep unknown keys (save.rs:4818-4851).
- S2: the create-world state for the screen hooks: read and write the creation game rules, the open tab, the hardcore and debug flags, and a row in the Game tab. Model: vanilla `WorldCreationUiState.getGameRules`/`setGameRules` and `CreateWorldScreen.GameTab`.
- T12: a slot rule for vanilla menus, or the container contents and the cursor stack in `inventory_click` and `inventory_drag`. Model: vanilla `Slot.mayPlace`.
- T14: container reads and the move hook for entity containers. Model: vanilla `AbstractMinecartContainer` and `AbstractChestBoat` `canPlaceItem`.
- A4: an item cooldown setter. Model: vanilla `ItemCooldowns.addCooldown`.

X rows:
- C4: a cancellable server hook on the respawn request, and a stamp for `respawn_player`. Model: vanilla `ServerGamePacketListenerImpl.handleClientCommand` (PERFORM_RESPAWN). Also a client hook on the immediate-respawn path (remote.rs:4634). Model: vanilla `LocalPlayer.respawn`.
- C5: a read helper for the hardcore flag. Model: vanilla `MinecraftServer.isHardcore`.
- T4: a cancellable death-protection hook that lets a mod choose the totem stack, and a helper that sends entity event 35. Model: NeoForge `LivingUseTotemEvent` and vanilla `Level.broadcastEntityEvent`; client side vanilla `ClientPacketListener.findTotem`.
- T8: a first-person hand hook that draws a replacement and sets its light. Model: NeoForge `RenderHandEvent` and vanilla `ItemInHandRenderer.renderArmWithItem` (packed light).
- T10: a per-stack foil hook on the client. Model: vanilla `Item.isFoil(ItemStack)`.
- T13: fire `inventory_move_item` (or a new hook) when a dropper inserts into a container. Model: vanilla `Container.canPlaceItem` as `HopperBlockEntity.addItem` asks it.
- A2: a read helper for the world spawn. Model: vanilla `ServerLevel.getRespawnData`.
- E7 (option): the attack strength in `living_incoming_damage_v2` or `hurt_enemy`. Model: vanilla `Player.getAttackStrengthScale`.

F rows: stamp the twelve fragments of section 4. `player_respawn_position` comes first: the core needs it.

## 7. Open questions that need a running game

1. Does `run_command` work from `server_tick` on the integrated server (is `self.lan` set there)? C7, C9, C10, C14, T3, T9 depend on it (v2 1b, medium confidence; a6 V6 says yes).
2. Does `gui_render_post` cover every frame between death and arrival (v4 1d)?
3. Where does a respawn land when its chunk is not loaded, and can the search read a parked Nether level for an anchor exile (a1 open questions 2 and 3)?
4. Which damage type names skip the immunity and the totem (`minecraft:out_of_world`, `minecraft:generic_kill`, others) (v1 2a)?
5. Does `summon` accept `minecraft:item` NBT with custom components (v1 8c)?
6. Does the client receive the `minecraft:tool` patch of a worn-out tool (v3 3c)?
7. Does a guest's landing call `living_fall` (v3 4b)?
8. Does `advancement grant` through `run_command` show the toast (v1 2d)?
9. Does a content item accept a default `lodestone_tracker`, and does a click on a lodestone retarget the amulet (NOTES questions 1 and 4)?
10. Does a content item accept a `damage_resistant` list for cactus (v3 5)?
11. Is `level.dat` on disk with the current `hardcore` and `spawn` values when the mod reads it in a new world (C5, A2 substitutes)?
12. Does a server-side glint component on the Dormant Totem keep the client's use running (a T10 substitute)? The server keeps the use while the slot is the same (G/using_item.rs:84-86).
13. Does `give_stack` to the owner from `server_tick` fail (a6 V6 trap)? The plan gives only inside the owner's own hooks.
14. Does Horizon draw the outline of an invisible glowing mob? If yes, a summoned glowing shulker could replace the E9 client boxes for players without the mod (an idea, not checked in the source).
15. Do the star lines match Horizon's stars (angle and brightness), and which world passes write depth (a4 open questions 2 and 3)?
16. What does the `respawn` command of `horizon-ctl` see while the journey overlay is up (v5 E3)?

## Settled disagreements

1. Nether anchor in single player and LAN. v1 (V0 note) said Horizon ignores it off a dedicated server. a6 V1 said the opposite. The code: G/integrated.rs:448 sets `options.dedicated = Some(Vec::new())` on the integrated server, so G/lan_actions.rs:4054 `let server = self.options.dedicated.is_some();` is true and `elsewhere` keeps the anchor. a6 was right.
2. Sounds and particles "from any server hook". v2 and v3 rated `Host::play_sound` and `Host::send_particles` as N. In H/crates/horizon-client/src/mod_api.rs:1216-1229 they belong to `LevelWriteHost` (level hooks only). The any-hook form is in the pending `host_bridge.json` (v4 was right). The N route is `/playsound` and `/particle` through `run_command`; `/particle` takes `viewers` (G/commands/world_cmds.rs:888-892).
3. Create World rules through `level_set_game_rule`. v4 row 6 used it. G/native_level_world.rs:86-88 return false for a rule outside the vanilla table, so BMH rules cannot go there (v5 G4 was right).
4. Advancements in a content mod. v5 B6 said content mods do not list them (from the older site page). H/docs/mods.md:31 lists "advancements" in a content mod's data. v1 was right.
5. Pending fragments force a main pin. v5 B1 said so. The tag carries all the needed fragments and the docs ask for a Nightly's commit (section 5). v5 B1 was wrong on this point.
6. One totem per container in menus. v2 rated it F, but its own evidence says `inventory_click` gives no contents and no cursor stack, so F cannot decide. This file rates it G.
7. `container_slots`. v2 listed it as a catalog helper. It is API 48 in G/machines/native_items.rs:110-115 and in docs/native-mods.md:3823, but it is in neither catalog: only mixin code can call it.
8. Config screen. v1 5b proposed a mod screen with text fields; v4 7 said buttons only. Both hold in part: `native_mod_screen_open` is false "while another screen is" (G/native_hooks.rs:11290-11301), so the Mods page gets buttons and a text-field screen opens only in the world.
9. Ritual runes per viewer. v2 5b needed pending `send_particles_to`. `/particle ... <viewers>` through `run_command` does it at N.
10. The respawn hold. v1 1d (X plus F on `guest_respawn`) and v4 1c (F with `server_packet` and `respawn_player`) give two partial routes. Neither alone covers the gamepad, `immediate_respawn` and the server side, so the row is X.
