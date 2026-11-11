# Respawn, journey (server side), game rules, config, commands, advancements

Paths: W = the BMH worktree, S = the scratchpad (see the common brief). "main" = S/src/horizon at commit b880384 (2026-11-11). "Nightly" = S/horizon-repo-docs/native-hooks.nightly-2026-11-08.json.

Vocabulary added by this audit (the common brief has no word for it): **PENDING** = the function exists in main's source and docs but its fragment in S/src/horizon/tools/native-hook-fragments/ has no API number yet. S/src/horizon/tools/native-hook-fragments/README.md says: "A hook in a pending fragment is not API yet; do not depend on it until it has a number." A PENDING item is in neither catalog (native-hooks.json has 434 hooks and 177 helpers, the Nightly 432 and 174, and 229 fragments are unstamped). A mod that calls it pins a main commit. I count such an item as PINNED in the table, and say "pending" next to it.

## Verdict table

| Feature | Verdict | Mechanism in one line |
|---|---|---|
| 1a. Capture at death (distance inputs, game mode, start of search) | PARTIAL | `loot_drops` (HOOK, both catalogs) is the first hook with the dying player swapped in and no revive possible; difficulty read is PENDING (`level_difficulty`) |
| 1b. Decide the distant position inside the respawn, before any position reaches the client | PINNED (pending hook) | `player_respawn_position` (fragment, main-only), called at lan_actions.rs:4083, before `hub.respawn` at :4156 |
| 1c. Landing search in the background, far unloaded chunks | PARTIAL | `force_chunk_in` (main-only, stamped API 413) + `surface_heights` (both catalogs) work; `native_biome` cannot read an unloaded chunk (GAP for the cheap ocean/river pre-filter) |
| 1d. Hold the Respawn request until the search ends | PINNED | cancellable HEAD mixin on `Game::guest_respawn` (lan_actions.rs:4019), re-driven by pending `respawn_player`; or hold it on the client |
| 1e. Exile bookkeeping, End exit excluded | HOOK | `player_respawned` does not fire for the End exit (native_hooks.rs:8657-8661) |
| 2a. 200 ticks damage immunity | HOOK | `living_incoming_damage` (cancel) + `player_tick` countdown, `net`-free |
| 2b. Bell and respawn sound for the others nearby | PARTIAL | `/playsound` through `run_command` (HOOK, tick context only), or `GameLevel::native_play_sound` (API 61, level hooks only), or pending `play_sound_to` |
| 2c. Journey signal server to client (no vanilla title trick needed) | HOOK | `net_send_to_player` / `net_from_server`, API 40 |
| 2d. Journey advancements awarded from code | HOOK | `native_trigger_criterion` (API 69) with a custom `bmh:` trigger |
| 3. Homecoming (record on the player, 1 Hz check, title, bell, 2 advancements) | HOOK | player attachment with `copy_on_death`, `server_tick` + `server_players()`, `run_command title` (or pending `title`) |
| 4. Mod game rules (`bmh:*`, `/gamerule`, saved with the world) | GAP | `RULES` is a fixed `const` table (gamerules.rs:50); no registration path |
| 5a. Config file (min/max distance, amulet points) | HOOK | `register_configs` Common kind gives `config/bmh-common.toml`, same file name as NeoForge |
| 5b. Config screen with text fields | PARTIAL | `settings` page gives buttons only; a text-field screen needs the mod-screen API (API 111) |
| 6. `/bmh point save\|remove\|delete` | HOOK | `register_commands` + `perform_command`, arguments `integer`, `block_pos`, `text` |
| 7. Advancement tab (20 entries) | HOOK | JSON in a content mod's `data/bmh/advancement/`; code awards via API 69; `minecraft:impossible` replaced by a `bmh:` trigger |
| 8a. SleepHandler (heal, food, haste, resistance on wake) | PINNED | `player_bed_leave` is HOOK; health, food and effect setters are pending (`set_player_status`, `add_effect`) |
| 8b. KeepItemAfterDeath | HOOK | take the Eternal stacks out of `loot_drops`, give them back in `player_respawned` with `give_stack` |
| 8c. EternalItemHandler (one of each Eternal Item) | PARTIAL | `player_tick` + `inventory_tick` + `item_picked_up` are HOOK; dropping the extra copy has no verified helper |
| 8d. DisableDebugInfo | PARTIAL | `run_command gamerule reduced_debug_info` is HOOK; reading the old value is pending (`level_game_rule`) |

Counts over the 19 rows: HOOK 9 (1e, 2a, 2c, 2d, 3, 5a, 6, 7, 8b), PARTIAL 6 (1a, 1c, 2b, 5b, 8c, 8d), PINNED 3 (1b, 1d, 8a), GAP 1 (4). Feature 1 decides whether BMH exists on Horizon: 1b is buildable only on a main commit, 1d needs a mixin.

---

## 1. The distant respawn

### Behaviour spec (what the Java mod does)

Trigger chain on NeoForge (the Forge and Fabric copies call the same handlers in the same order, ForgeEvents.java:44-61, ServerPlayerMixin.java:28-30):

1. `LivingDeathEvent` HIGHEST: `RespawnPositionHandler.onPlayerDeath` stores `distancesFromBed[uuid] = calculateDistanceFromBed(player)` (NeoForgeEvents.java:30-36, RespawnPositionHandler.java:59-62). The inventory still holds the Eternal Items at that moment.
2. `LivingDeathEvent` normal: `KeepItemAfterDeath.onPlayerDeath` removes Eternal stacks (feature 8b).
3. `LivingDeathEvent` LOWEST, only if no listener cancelled: `JourneySignal.onPlayerDeath` (NeoForgeEvents.java:47-53). If `isDistantRespawn` (rule `bmh:random_death_spawn_point` true and game mode SURVIVAL, RespawnPositionHandler.java:71-75) and the server is not hardcore: send the departure signal and `RespawnSearch.start` (JourneySignal.java:28-35).
4. The client sends PERFORM_RESPAWN. `F_ServerGamePacketListenerMixin.bmh$holdRespawn` (HEAD of `handleClientCommand`, F_ServerGamePacketListenerMixin.java:27-35) cancels it on the server thread while `RespawnSearch.holdRespawn` is true (search exists, not finished, health <= 0, not `wonGame`; RespawnSearch.java:125-135) and records `respawnWaiting`.
5. Each tick of the player's connection (`ServerGamePacketListenerImpl.tick` RETURN, F_ServerGamePacketListenerMixin.java:37-41, runs while dead) `RespawnSearch.tick` advances the search, fails it after `TIMEOUT_MS = 120_000` (RespawnSearch.java:35, 105-111) and, once finished, replays the held request with a new `ServerboundClientCommandPacket(PERFORM_RESPAWN)` (RespawnSearch.java:115-121). `onDisconnect` cancels the search (F_ServerGamePacketListenerMixin.java:43-47).
6. Vanilla `PlayerList.respawn` runs. `F_PlayerListRespawnMixin` stores the dead player and `endConquered = keepInventory` at HEAD (F_PlayerListRespawnMixin.java:32-39), and a `@ModifyVariable` right after `findRespawnPositionAndUseSpawnBlock` (so an anchor still loses its charge and a missing bed still prints its message) replaces the `TeleportTransition` position with `Vec3.atBottomCenterOf(respawnPosition)` (F_PlayerListRespawnMixin.java:44-62). This is "before vanilla sends any position to the client": the old approach (teleport from the respawn event) showed the spawn point first (class comment, F_PlayerListRespawnMixin.java:19-22).
7. After the respawn the loader fires `PlayerRespawnEvent`: `KeepItemAfterDeath.onPlayerRespawn` gives the items back, then at LOWEST `RespawnEffectHandler.onPlayerRespawn` (feature 2).

`RespawnPositionHandler.findRespawnPosition(player, respawnLevel, missingRespawnBlock, endConquered)` (RespawnPositionHandler.java:94-160), conditions in order:

- `endConquered` true: return null (vanilla position).
- take the stored distance, else recompute from the current inventory (restart case).
- rule `random_death_spawn_point` false in `respawnLevel`: null.
- game mode not SURVIVAL: null.
- centre: the player's respawn config (bed or anchor) position, but only if the block still exists (`missingRespawnBlock ? null : getRespawnConfig()`) and its dimension equals `respawnLevel`; otherwise `respawnLevel.getRespawnData().pos()` (the world spawn), lines 121-128.
- take the finished `RespawnSearch` if `matches(respawnLevel, bedLocation)` (same dimension, same centre; RespawnSearch.java:170-173), else warn and search synchronously with `calculateRandomPosition` (lines 131-145).
- null result: vanilla position. Otherwise `exiledPlayers.add(uuid)`, `exiles.put(uuid, Exile(center, distance))` and return the position. The bed or anchor spawn point is not touched (comment, line 158).

Distance formula (`calculateDistanceFromBed`, RespawnPositionHandler.java:215-244): `min = Services.PLATFORM.minDistanceFromBed()`, `max = maxDistanceFromBed()` (config; if `min > max`, `max = min`); `easyLife = 1 + (number of inventory slots whose item is an IEternalItem) + level.getDifficulty().getId()` (peaceful 0, easy 1, normal 2, hard 3), capped at `MAX_EASY_LIFE = 9`; `distance = min + (max - min) * easyLife / 9` in integer arithmetic. It iterates `getContainerSize()` slots, one count per slot (a stack of 2 counts once). Config defaults: min 5000, max 60000, ranges 5000 to 200000 (BMHConfigLimits.java:9-13).

Level scaling (`toLevelBlocks`, lines 164-174): `levelDistance = floor(clamp(worldDistance / coordinateScale + 0.5, 0, WorldBorder.MAX_CENTER_COORDINATE))`. A Nether anchor exile is divided by 8.

Candidate points (`candidatePoint`, lines 197-213): angle `theta = 2*pi*random`, point `(cx + (int)(r*cos), cz + (int)(r*sin))` (lines 303-311); reject if outside the world border; reject if the biome at `(x, seaLevel, z)` is in tags `IS_OCEAN`, `IS_DEEP_OCEAN` or `IS_RIVER`. The biome call "does not generate the chunk" (comment line 205). Up to `MAX_POSITION_ATTEMPTS = 32` candidates (line 41).

Safe position (`findSafePosition`, lines 248-286): if the dimension has a ceiling, start at `minY + logicalHeight - 2` and go down until a free block; else load the chunk and read `MOTION_BLOCKING_NO_LEAVES` height + 1 directly from the chunk (not `Level.getHeight`, which answers min height on an unloaded chunk: comment lines 261-262 and CLAUDE.md "Code map"). Then walk down through free space (no collision shape, no fluid) to the first solid block. Accept if `y > minY`, the ground block has no fluid, `isFaceSturdy(UP)`, is not in `BlockTags.LEAVES` and is not `MAGMA_BLOCK`, and the two blocks above are free (lines 276-281, 295-301). Result is the feet position.

`RespawnSearch` (RespawnSearch.java): per-player state `{dimension, center, distance, startedAt, attempts, candidate, result, failed, respawnWaiting}`. `start` (lines 66-92) guesses the level and centre as vanilla will: the player's respawn config level and position if the block is still a bed (`BlockTags.BEDS`) or a respawn anchor, or the config is `forced()`, else the Overworld world spawn. `advance` (lines 180-227): if a candidate exists, check `getChunkNow` (non-blocking) and, when complete, run `findSafePosition`; on success store `result` and keep the chunk ticket; else release the ticket. Then take the next `candidatePoint` and `addTicketWithRadius(TICKET, chunk, 0)` where `TICKET = new TicketType(6000L, FLAG_LOADING)` (kept 5 minutes so the landing chunk is still loaded on arrival, line 37). `failed` after 32 attempts.

Damage immunity after the journey: `F_ServerPlayerJourneyMixin` (F_ServerPlayerJourneyMixin.java:34-48): a counter decremented at `ServerPlayer.tick` HEAD, and `hurtServer` HEAD returns false while the counter is above 0 unless the source is in `DamageTypeTags.BYPASSES_INVULNERABILITY`. Set to 200 in `RespawnEffectHandler` (RespawnEffectHandler.java:21, 59).

### Horizon mapping

**Where the respawn is decided.** One function, in the server's Game: `pub(super) fn guest_respawn(&mut self, host: &mut LanHost, conn: u32)` at S/src/horizon/crates/horizon-client/src/game/lan_actions.rs:4019. It is reached by the client's PERFORM_RESPAWN: `S::ClientCommand(c) if c.action == 0 => self.guest_respawn(host, conn),` (lan_actions.rs:3089), and by the mod helper `native_respawn_player` through `run_native_moves` (native_travel.rs:438-451). The window sends that packet from `remote_respawn`: `self.send(S::ClientCommand(play::ClientCommand { action: 0 }));` (remote.rs:10497-10499), reached from `respawn_player` (screens.rs:770-777). The single-player owner is a guest of the integrated server (docs/native-mods.md:2134-2135 "the mods run in the host's game and its integrated server, so a server-side hook applies to every player of the world"; native_hooks.rs:7432-7434 and docs line 1647 say the owner is told like anyone else). I did not find a second respawn path for the owner (UNVERIFIED in the sense that I read no test that kills the owner; I searched `fn .*respawn` in all crates).

**The order inside `guest_respawn`** (lan_actions.rs, in order):

1. :4026-4031 `attachments.copy_on_death()` replaces the player's attachments (so `copy_on_death` data survives, others are removed).
2. :4033-4050 drops carried state, closes the container, resets equipment bookkeeping.
3. :4065-4081 computes the default respawn level and position (`spawn`, or the bed `+0.6` in y for a window world, or `elsewhere` for a dedicated server).
4. **:4082-4083**: `// PlayerRespawnPositionEvent: a point the mods choose is taken as it is.` then `self.native_player_respawn_position(uuid, &name, &mut dimension, &mut at, yaw, pitch);`. This is the mod's moment. Nothing has been sent to the client yet for this respawn.
5. :4084-4097 `chosen = (known && dimension != level.id()) || moved`; when chosen the bed/anchor/trip are dropped (`(elsewhere, in_frame_bed) = (None, None)`) and `spawn = at`.
6. :4098-4115 level switch for a dedicated server, then `g.dead = false`, `g.loading = true`, food 20, `g.respawn_health(...)`.
7. :4129-4135 `xp.respawn(...)`, then `self.native_player_clone(cloned, &name, true);` (hook `player_clone`, "the data is copied, the new player not yet placed").
8. **:4156** `host.hub.respawn(i, spawn, rotation, &ctx);`. In `Hub::respawn` (lan/hub.rs:5025-5100) the first packets leave: `C::RemoveEntities`, `g.position = at`, `C::Respawn`, `resend_teleport`, `default_spawn`, `SetExperience`, `SetChunkCacheCenter`, inventory. The quoted doc: "PlayerList.respawn: back to `at` with a fresh start. The vanilla client waits on its level loading screen from the Respawn packet until ... the chunk under the player" (hub.rs:5022-5024).
9. :4158-4172 hardcore: set spectator; trips for an anchor elsewhere or the world spawn's free column when nothing was chosen.
10. **:4205-4208** `self.as_guest(host, conn, |g| g.native_player_respawned(respawned, &name));` with the player's state swapped in.

So a mod that answers `player_respawn_position` runs at step 4, before step 8, which is the first place any position is sent. This is the equivalent of the Java `@ModifyVariable`: same constraint satisfied. A mod that waits for `player_respawned` (step 10) is too late, exactly the bug the Java mod's class comment describes.

**The hook.** `player_respawn_position`, a PENDING fragment:
- S/src/horizon/tools/native-hook-fragments/player_respawn_position.json: `"name": "player_respawn_position"`, `"neoforge": "PlayerRespawnPositionEvent"`, `"side": "server"`, `"contract": "listen"`.
- S/src/horizon/crates/horizon-client/src/game/native_hooks.rs:7441: `pub(super) fn native_player_respawn_position(&mut self, player: u128, name: &str, dimension: &mut String, pos: &mut [f64; 3], yaw: &mut f32, pitch: &mut f32)`.
- Doc, native_hooks.rs:7429-7440: "A dead server player is about to respawn (NeoForge's `PlayerRespawnPositionEvent` ...), once the Respawn click came and before they are placed: `dimension` ... and `pos` are their respawn point (their bed or anchor block's top, a forced point, else the world spawn) and `yaw`, `pitch` the look they get (0, 0). ... Change the dimension or the position and the player is placed exactly there, with no bed or anchor used and no spot looked for; a dimension the server does not have, or a position that is not finite, leaves that part as it was. On a LAN host's world (not a dedicated server) the level is the Overworld whatever is set. No player's state is swapped in, and the hub is out of `self.lan`, so `native_tell` reaches nobody here. Server-side. @API:player_respawn_position@."
- Catalog status: absent from native-hooks.json (main) and from the Nightly. grep for `respawn_position` in both JSON files and hooks-compact.txt returns 0; it exists in native_hooks.rs, lan_actions.rs, docs/native-mods.md:2855 and the fragment only. Main-only and unstamped. The mod pins a main commit until the stamp PR runs.
- Typed API: not checked in the generated trait (generated into OUT_DIR, not in S). The docs list `player_respawned` as "will join a later slice" for the typed `Hooks` trait (docs/native-mods.md, "The following catalog hooks are not in the trait yet"), so the helper-style (horizon-mixins HEAD injection with `hook = ...` or `target = "...native_player_respawn_position"`) is the safe way.

What the mod gets as input: `pos` is the feet position (bed: block pos + 0.5, +0.6 in y, lan_actions.rs:4066-4071; world spawn: `world_spawn_position()`), `dimension` is the Overworld on a single-player or LAN world, any of the three dimensions on a dedicated server where the player's anchor or forced point is in another level (lan_actions.rs:4051-4064 and :4072-4079). This gives the centre the Java uses (bed or anchor else world spawn), with one difference: for a bed that was destroyed the hook still hands over the old bed position (the bed check happens after, in `resolve_server_respawn`, :4185). The Java mod uses the world spawn then (`missingRespawnBlock`, RespawnPositionHandler.java:121). The mod can only detect this by reading the block at `pos`, which works only if that chunk is loaded.

What the mod cannot read at that moment: the hook says the hub is out of reach (`self.lan` empty), so `server_players()` (native_server.rs, `let Some(host) = &self.lan else { return Vec::new(); }`) returns nothing, and `player_stacks` also needs `self.lan` (native_menus.rs:232-243). The mod must therefore capture everything at death or from a snapshot of `server_players()` refreshed in `server_tick`: game mode (needed for the SURVIVAL test), the distance (needs the inventory and the difficulty), and whether the world is hardcore.

**Reading the formula inputs on Horizon.**

- Eternal Item count: slots of the dying player. First hook with the player's state swapped in during a death: `loot_drops` (catalog both, `[s v1] LivingDropsEvent / BlockDropsEvent`). The player-death call is `self.native_loot_drops(&context, &mut stacks)` at item_entities.rs:618, with `context.player = Some(who.0)`, `entity: Some((who.1 as u32, "minecraft:player"))`, `table: ""`, called from `drop_inventory` inside `self.as_guest(host, conn, |game| { ... game.drop_inventory(eyes, (uuid, id)); ...})` (lan_actions.rs:4611-4616). `stacks` is every dropped stack (inventory, armour, off hand: `death_drops(&mut self.inventory, &mut self.offhand, &mut self.armor)`, item_entities.rs:605). Counting Eternal Items there counts stacks, not slots, but `death_drops` yields one entry per occupied slot, so the count matches the Java loop. Limit: `drop_inventory` returns before this when the rule `keep_inventory` is true (item_entities.rs:602-604). With `keep_inventory` the mod gets no count (the Java mod does).
- Earlier hooks fire with the hub out of reach and the player marked dead: `player_died` is called after `g.dead = true` (lan_actions.rs:4510 and :4534) and `native_player_stacks` returns None for a dead player (native_menus.rs:240 `(!g.dead && ...)`). `living_death` (cancellable, catalog both) fires before that (lan_actions.rs:4488-4497) but with `entity: u32` only (a network id, not a uuid) and `self.lan` empty. So `loot_drops` is the right capture point. It also runs only when no mod revived the player, which is the "no other mod can cancel it" point the Java reaches with `EventPriority.LOWEST`.
- Difficulty: the stamped surface only has the change hook `difficulty_changed` (native_hooks.rs:8615, API 159). The read helper is PENDING: `level.difficulty(host)` returns `"peaceful"|"easy"|"normal"|"hard"` (native_level_world.rs:28-31 `native_level_difficulty`, fragment level_world.json, `@API:level_world@`). A mod can also seed its own copy from `difficulty_changed` plus a read at first use; no catalog read exists. Verdict for this input: PINNED on main.
- Config: `register_configs` (feature 5).
- Game mode: `server_players()[i].game_mode` (mod_api.rs:677 "0 survival, 1 creative, 2 adventure, 3 spectator"), read from `server_tick` into a map, or tracked by `player_game_mode_changed` (catalog both, cancellable) plus a read at `player_logged_in`.
- Hardcore: I found no helper that reads `level.hardcore` (grep `hardcore` in native_*.rs and docs/native-mods.md: only comments about the spectator switch, native_hooks.rs:4488-4493). GAP for a direct read. Workaround: the hardcore respawn forces SPECTATOR and `player_game_mode_changed` is heard with the reason documented at docs/native-mods.md:2274-2276; but that comes after the respawn, so the departure signal cannot be suppressed in a hardcore world on Horizon. UNVERIFIED whether any command output exposes it.

**Searching a far, unloaded landing chunk.**

- Ticket helper (non-blocking): `native_force_chunk_in(owner, dimension, chunk, hold)` (native_chunk_tickets.rs:37-47, "API 413"). Catalog: in main's native-hooks.json as helpers `force_chunk`, `forced_chunks`, `force_chunk_in`; NOT in the Nightly catalog (174 helpers vs main's 177; the three missing are exactly these). Main-only. Doc (docs/native-mods.md "Chunk tickets"): "holds a chunk of the level in place ... it is loaded with the ring around it, it ticks, and its level stays open"; "The Nether and the End open only when something goes there"; "Each ticket keeps a 5x5 ring loaded ... keep to what a machine needs (dozens, not thousands)"; "A dedicated server that pauses when empty ... stops ticking held chunks too"; tickets are not saved. `owner` must be namespaced (`bmh:search/<uuid>`). Hold and release semantics are not time-limited (the Java ticket is 6000 ticks): the mod must release.
- Async variant, PENDING: `native_chunk_load_request(owner, dimension, chunk)`, `native_chunk_load_state(...) -> NativeChunkLoad::{Pending, Loaded, Failed}`, `native_chunk_is_loaded`, `native_chunk_load_release` (native_chunk_tickets.rs:49-101, fragment chunk_load_async.json, `@API:chunk_load_async@`). `request` is just `native_force_chunk_in(.., true)`, so the stamped `force_chunk_in` + a "loaded?" test gives the same thing today. The "loaded?" test: `surface_heights` answers `None` for a column whose chunk is not loaded (native_world_columns.rs:15-27 and docs "Column reads": "`None` for a column whose chunk isn't loaded").
- Heights: `native_surface_heights(dimension, columns, heightmap)` (native_world_columns.rs:21), catalog helper `surface_heights` in main AND Nightly (API 127). `NativeHeightmap` has `MotionBlockingNoLeaves` (native_world_edit.rs:12-17), the Java heightmap. Limits: `Err` when `dimension` is not "the level being ticked" (`native_level_open`, native_world_columns.rs:26-28), so only the level in place; at most 65,536 columns; "They run on the server tick".
- Blocks around the surface: `native_read_columns(dimension, columns, heightmap, (below, above))` (both catalogs) returns block names; the safety test then needs name tables for "sturdy top face" and "leaves". Collision: `native_block_collision(pos)` exists (native_hooks.rs:11411, catalog helper `block_collision`) but takes a position in the loaded world only. Tags: no block-tag helper for a name in the stamped catalog; the Host table lists `block_name_in_tag(block, tag)` as a form of `server_block_in_tag` (docs line ~5500), which I did not trace. So `BlockTags.LEAVES` and the fluid and collision tests are hard-coded lists unless that helper works: PARTIAL, parity risk (modded blocks).
- Biome pre-filter: `native_biome(pos)` is "None where that chunk isn't loaded" (native_hooks.rs:11398-11402, doc "Level.getBiome ... None where that chunk isn't loaded"). The Java check "biome lookup does not generate the chunk" (RespawnPositionHandler.java:205) cannot be reproduced: GAP for the cheap rejection. Consequence: each candidate in an ocean or a river costs a ticket with a 5x5 ring (25 chunks) before it can be rejected. Up to 32 candidates, serial, so worst case a long search, but the 120 s timeout already exists. Also the biome tag test (`IS_OCEAN`, `IS_DEEP_OCEAN`, `IS_RIVER`) has no tag helper for biomes (grep native_*.rs: none): hard-coded name list, data pack biomes missed.
- Border: `world_border` is PENDING (fragment world_border.json: "bounds of a level's world border"); not in either catalog.
- Dimension info: `dimensionType().hasCeiling()`, `logicalHeight()`, `coordinateScale()`: no helper. `level_height` (catalog both, `LevelRef.height(host)` -> `Option<(i32, i32)>`, native_level_refs.rs:84) gives min y and height. Horizon has only three dimensions (`native_dim` in native_chunk_tickets.rs: overworld, the_nether, the_end), so hard-code scale 8 for the Nether, 1 otherwise, ceiling for the Nether only.
- The level in place: all of the above work only for the Overworld in a single-player or LAN world (the server holds one level in place, `self.dim`; other dimensions are parked, `levels.parked_world`). For a dedicated server with anchors in the Nether, searching a Nether chunk from the death hook needs the Nether in place: `force_chunk_in` can open it ("a level held this way opens at the next server tick", docs) but `surface_heights` then reads it only when it is the level being ticked: UNVERIFIED how a mod reads a parked level's heights. The respawn level is the Overworld in the window world, so the normal case is fine.

**Holding the click.** Horizon has no hook before `guest_respawn` that can refuse the request. I searched hooks-compact.txt for a serverbound packet hook: `net_from_client` is the mod channel only; `server_packet` (fragment, PENDING) hears outgoing packets only ("each play packet the server is about to send", docs "Server hooks for plugin ports: packets"). Three ways:
- PINNED: horizon-mixins cancellable HEAD (README: "HEAD-only hooks, including cancellable HEAD", docs/advanced-injections.md:109-133, `ControlFlow<R>`), target `horizon_client::game::Game<'_>::guest_respawn` (lan_actions.rs:4019, a private `pub(super)` method). Return `Break(())` while the search runs, then call the pending `native_respawn_player(uuid)` (native_travel.rs:369-392, "Respawns the dead server player `uuid` as their client's Respawn click does ... API @API:respawn_player@") from `server_tick` when the search ends. `native_respawn_player` re-enters `guest_respawn` (native_travel.rs:446), so the mixin needs a one-shot flag, as the Java replay does (F_ServerGamePacketListenerMixin.java:30-33). The mod then pins one commit.
- Client side: the window's respawn path is `respawn_player` (screens.rs:770), also a private method, so a client-side delay is also a mixin (the Java mod does the same on the client with F_LocalPlayerRespawnMixin, another group's file).
- No hold at all: if `player_respawn_position` finds the search unfinished it can only fall back to vanilla's position, or choose a spot whose chunk is not loaded yet. I found no evidence of how Horizon treats a `pos` in an unloaded chunk (the client "waits on its level loading screen ... and the chunk under the player", hub.rs:5022-5024, and the server streams chunks around `g.position`; the generation cost lands on the server thread: UNVERIFIED). The Java comment says the same freeze happened for the old code (RespawnSearch.java:25-30).

**Mapping of the Java parts to Horizon, one line each.**

| Java | Horizon |
|---|---|
| `LivingDeathEvent` HIGHEST, distance stored | `loot_drops` (swapped-in player, after all cancels). Store in a `thread_local!` on the server thread (docs/mod-code: "the single-player server's game runs on another thread and sees its own") or in an attachment |
| `JourneySignal.onPlayerDeath` + `RespawnSearch.start` | same `loot_drops` call; `net_send_to_player` for the signal; `force_chunk_in` for the first ticket |
| search `tick` driven by the connection tick | `server_tick` (catalog both) or `player_tick` (does not fire for a dead player: native_hooks.rs:2679-2683 "a dead player doesn't"), so use `server_tick` |
| hold the request | PINNED mixin on `guest_respawn` |
| `@ModifyVariable` on the transition | `player_respawn_position` (pending) |
| `PlayerRespawnEvent` | `player_respawned` (catalog both, swapped in, not for End exit) |
| `exiledPlayers` / `exiles` | a map filled in `player_respawn_position`, consumed in `player_respawned` (both run in the server's Game) |

### Verdict

PARTIAL, with the load-bearing hook PINNED: 1b PINNED (pending `player_respawn_position`, main-only; becomes HOOK when stamped), 1c PARTIAL (tickets main-only, biome pre-filter GAP), 1d PINNED, 1e HOOK. Without 1b nothing else matters: today the only stamped respawn hooks (`player_respawned`, `player_clone`) fire after the first position is sent (lan_actions.rs:4156 sends, :4208 fires). Using `teleport` from `player_respawned` would reproduce the very bug F_PlayerListRespawnMixin was written to fix.

### Parity risks

- The cheap biome pre-filter is gone: more chunk generation per respawn, possible long waits.
- Bed destroyed: Java centres on the world spawn, Horizon hands the old bed position to the hook.
- `keep_inventory` true: no Eternal count at death, no wear, no keep/restore step (the Java mod still runs).
- Hardcore worlds: the Java departure signal is suppressed on the server; Horizon has no hardcore read.
- Vanilla `PlayerRespawnPositionEvent` equivalent exists, but the landing y comes from heights read before the player arrives; the Java mod also accepted the chunk may change. Same.
- Per-candidate cost is 25 chunks (5x5 ring) instead of 1 (radius-0 ticket).
- The search runs on the level in place only.

---

## 2. Respawn effects and the journey signal

### Behaviour spec

`RespawnEffectHandler.onPlayerRespawn(player, endConquered)` (RespawnEffectHandler.java:27-65), at LOWEST on the respawn event, after the kept items are back:
1. Return if `endConquered` or not a `ServerPlayer`.
2. `exiled = takeExiled(player)`, `exile = takeExile(player)`; return if not exiled or the server is hardcore.
3. Award `bmh:journey/exile`. If the stat `Stats.DEATHS` is >= 10 award `journey/here_we_go_again`. If `exile.distance >= 30000` (`BmhAdvancements.FAR_AWAY_BLOCKS`, BmhAdvancements.java:21) award `journey/far_away`.
4. `HomecomingHandler.onExile(player, exile)` (feature 3).
5. `setJourneyInvulnerableTicks(200)`.
6. `playRespawnSound`: `level.playSeededSound(serverPlayer, x, y, z, BELL_SOUND, PLAYERS, 1.0, 1.0, 0)` then `RESPAWN_SOUND` at volume 0.3 pitch 1.0, both excluding the respawned player (lines 68-93: "The respawned player hears the sounds from the cinematic on its own client").
7. `JourneySignal.sendArrival(player, rule display_text_on_respawn)`.

`JourneySignal` (JourneySignal.java): three translation keys carried in `ClientboundSetTitleTextPacket`: `bmh.journey.departure` (sent at death, empty fallback), `bmh.journey.arrival` (after the respawn; argument = the message component when the rule is on, else empty fallback) and `bmh.journey.message` ("A new journey begins for you, %s." with the player name). A client mixin removes them before vanilla draws (F_GuiTitleMixin, client group). A fake player with no connection is skipped (lines 48-54).

### Horizon mapping

- Immunity: `living_incoming_damage` (catalog both, cancellable, `[s! v1]`): signature `native_living_incoming_damage(entity: i32, kind: &str, damage_type: &str, causing: i32, direct: i32, amount: &mut f32, canceled: &mut bool)` (native_hooks.rs:9451-9460, doc 9425-9443: "Change `amount`, or set `canceled`, and nothing of `hurtServer` itself happens ... It runs on the thread that ticks the world ... No `self`"). No `self`, so the immunity map is a `static`/`thread_local!` keyed by network entity id. The entity id of a player is `NativeServerPlayer.entity_id` (mod_api.rs:684-687, "the one hooks that only know entity ids (`living_incoming_damage`, ...) give for the player", API 398). `BYPASSES_INVULNERABILITY` has no tag helper: compare `damage_type` to `minecraft:out_of_world` and `minecraft:generic_kill` (names UNVERIFIED against Horizon's 26.3 data; the hook gives names like `minecraft:fall` per the doc). Countdown: `player_tick` (catalog both, `[s v1] PlayerTickEvent.Post`, once a tick per living player, swapped in), or a count in `server_tick`. Horizon also gives its own respawn protection: "g.loading" and the load timeout (`CLIENT_LOADED_TIMEOUT`, lan_actions.rs:4101-4103, hub.rs:4990) whose effect on damage I did not trace.
- Sound for the others: three options.
  1. `run_command("playsound bmh:bell player @a[...] x y z 1 1")`: `native_run_command(line)` (native_server.rs:950, API 21, catalog both) "Runs `line` (no slash) as the server's console does: permission level 4 ... Only where no player's state is swapped in (`native_tick`, `native_menu_clicked`): false inside a player's hook". `player_respawned` is a player hook (swapped in), so call it later from `server_tick`. `cmd_playsound` (commands/world_cmds.rs:614-700) supports a category literal, targets, position, volume, pitch and a range of 16 blocks (`range = if volume > 1.0 { 16.0 * volume } else { 16.0 }`). Excluding the respawned player: selector `@a[name=!Name]` (selector support UNVERIFIED in this commands module).
  2. `GameLevel::native_play_sound(sound, at, source, volume, pitch)` (native_hooks.rs:10119-10138, API 61): "Play a sound event for nearby players in this server level ... Another namespace's event (a mod's, `mymod:machine_hum`) is sent by name ... players hear it if their resources define it (`assets/mymod/sounds.json`, a content mod's)". It is on `GameLevel`, so only inside level hooks (a block entity tick, `level_tick_*`), no player exclusion.
  3. PENDING `native_play_sound_to(uuid, sound, category, at, volume, pitch)` (native_hooks.rs:10303, fragment play_sound_to.json, "plays a sound to one player alone ... it leaves at the tick's end, so it works from any server hook"): call it for each player within range except the respawned one (`server_players()` has positions).
  Custom sounds `bmh:bell` and `bmh:respawn` need `assets/bmh/sounds.json` + `.ogg` in the content part of the mod (docs/native-mods.md "One .hmod for content and code"). Sounds are another group's.
- Journey signal: `native_net_send_to_player(player, channel, data)` (API 40, docs "Mod messages"): "bytes between a mod's server half and its client half, on the one custom payload channel `horizon:mod_net` ... channel is `mod_id:name` ... data is at most 30 000 bytes ... a message to a player leaves at the end of the tick, from any server hook." Client hears `native_net_from_server(channel, data)`. Both hooks are in both catalogs (`net_from_client`, `net_from_server` in hooks-compact.txt:52-53). A vanilla client never gets these, which also means the whole fallback-text design of JourneySignal disappears: send `bmh:journey` with a 1-byte kind and the player name. F_GuiTitleMixin is not needed on Horizon.
- Advancement awards: feature 7.
- Death count for `here_we_go_again`: pending `native_player_stat(uuid, stat_type, key) -> Option<i32>` (native_player_reads.rs:49, fragment player_records.json). Stamped alternative: count `player_stat_award` events (API 321, `native_player_stat_award(player, name, stat_type, stat, value, cancel)`) or `player_died` yourself in an attachment; this loses deaths before the mod was installed.

### Verdict

HOOK for immunity, signal, awards. PARTIAL for the sound (all three routes work but each has a limit). The pending reads (`player_stat`) are PINNED.

### Parity risks

- Immunity: Horizon's own respawn grace (client-loaded timeout) may stack with the 200 ticks; a damage type name mismatch could let void or `/kill` damage fail.
- Sound exclusion of the respawned player needs selector support or the pending `play_sound_to`.
- A death count that restarts at install.

---

## 3. Homecoming

### Behaviour spec

HomecomingHandler.java and the two mixins.

- Record (lines 57-59): `Record(String dimension, long arrivalTime, int distance, List<Home> homes)`, `Home(String name, BlockPos pos)`. Saved on the player by `C_ServerPlayerMixin` under the key `bmh_homecoming` (TAG, line 49), codec fields `dimension` (string), `arrival` (long), `distance` (int), `homes` (list of `{name, x, y, z}`), all required, all written (lines 61-74; the layout is the pre-26.1 NBT layout). Written in `addAdditionalSaveData` TAIL, read in `readAdditionalSaveData` TAIL, copied in `restoreFrom` TAIL so it survives death and the End exit (C_ServerPlayerMixin.java:59-78).
- `onExile(player, exile)` (lines 98-112): homes = `[Home("", exile.center)]` plus, only when the exile level is the Overworld, one `Home(point.name, point.pos)` per amulet point at that moment. Record = `{level.dimension().identifier, level.getGameTime(), exile.distance, homes}`. A new exile replaces an old record.
- Check: every `CHECK_INTERVAL = 20` ticks, `tick % 20 == 0` in `ServerPlayer.tick` HEAD (E_ServerPlayerMixin.java:44-52). `tick(player)`: skip if no record or the record's dimension differs from the player's level; `reached(record, x, z)` picks the nearest home with `(x - (hx + 0.5))^2 + (z - (hz + 0.5))^2 <= 24^2`, the first home wins a tie (`nearestSquared = squared - 1.0E-9`, lines 130-150). Y is ignored.
- `arrive` (lines 158-178): clear the record; award `journey/home_at_last`; if `distance >= 30000` also `journey/long_way_home`; if rule `display_text_on_respawn` is true show the title; always play `BELL_SOUND` at `blockPosition`, `PLAYERS`, volume 1.0, pitch 0.8, to everybody including the player (excluded player `null`).
- Title (lines 180-206): `nights = max(0, (gameTime - arrival) / 24000)`. Subtitle key: `bmh.homecoming.subtitle` (nights >= 2: "%s blocks and %s nights since you left"), `.one_night` (nights == 1), `.no_nights` (else, "%s blocks since you left"). Title: `bmh.homecoming.home` ("Home") when the home name is empty, else the literal point name. Packets in order: titles animation `(10, 70, 20)`, subtitle, title ("Same order as the /title command").

### Horizon mapping

- Saved per-player data surviving death and relog: server attachments (docs/native-mods.md "Server attachments", lines 5402-5532). `AttachmentType<T> { namespace: "bmh", name: "homecoming", default: || None, codec: Some(AttachmentCodec{encode, decode}), copy_on_death: true, sync: false }`; `Holder::Player(uuid)`; "Player UUIDs cover online guests and the acting player"; "A saved value is stored under `horizon:attachments`, then the mod namespace and attachment name, as `{value: <compound>, copy_on_death: 1b, sync: 1b}`"; "`copy_on_death` requires a codec. Player respawn copies opted-in entries". Evidence of survival: the test `player_attachments_survive_playerdata_and_respawn` (lan/playerdata.rs:917-944) saves a player, loads it, calls `copy_on_death()` and reads 31 back; and `guest_respawn` calls `copy_on_death()` at its start (lan_actions.rs:4026-4031). The catalog helper is `attachments` (both); the typed form needs `Host`. The layout can keep the Java NBT keys `dimension`, `arrival`, `distance`, `homes[{name,x,y,z}]` in the compound; the key `bmh_homecoming` is not reproducible (attachment key is `(namespace, name)` under `horizon:attachments`), so a Java save cannot be read by the Horizon mod unless a migration reads the Java player `.dat`: a Horizon world is not a Java world anyway (UNVERIFIED how worlds are imported).
- Write at `player_respawned` (swapped in, API 115): set the record with `Holder::Player(uuid)`. The exile centre comes from the value stashed in `player_respawn_position`.
- The Java `restoreFrom` copy is not needed: `copy_on_death: true` does it, and the End exit is also a `copy_on_death` respawn path? UNVERIFIED for the End exit; `player_clone` with `was_death = false` (native_hooks.rs:8704-8709) fires for it, so a mod can copy by hand there.
- 1 Hz check: `server_tick` (catalog both), count ticks yourself; iterate `server_players()` (catalog helper `server_players`, gives `uuid`, `dimension`, `position`, `game_mode`); read the record with the attachment host. `player_tick` (swapped in, per player) is the alternative and also gives the acting-player helpers; use `game.tick_count % 20`-style own counter.
- Game time: `level_game_time` (catalog both, `LevelRef.game_time(host)`, native_level_refs.rs).
- Title: PENDING `native_title(uuid, title, subtitle, fade_in, stay, fade_out)` (native_hooks.rs:10419, fragment title.json, "as `/title` and Adventure's `showTitle`"). Stamped alternative: `native_run_command("title <Name> times 10 70 20")`, `title <Name> subtitle {...}`, `title <Name> title {...}` from `server_tick` (`cmd_title` exists, commands.rs:1235). The text uses translatable components `{"translate":"bmh.homecoming.subtitle","with":[...]}`; the keys come from the content part's `assets/bmh/lang/en_us.json` (the Java file already has them, lines 77-80).
- Bell: as feature 2.
- Advancements: feature 7.

### Verdict

HOOK (title through `run_command` today, `title` helper when stamped).

### Parity risks

- Y ignored in both; same.
- Title text via `run_command` uses the player's name as target; names with spaces cannot occur in Minecraft.
- If the check runs in `server_tick` instead of `ServerPlayer.tick`, a dying or just-respawned player is handled the same way.

---

## 4. Game rules

### Behaviour spec

`BMHGameRules` (BMHGameRules.java:27-46): six boolean rules registered in the game rule registry with ids `bmh:random_death_spawn_point`, `bmh:display_text_on_respawn`, `bmh:food_and_health_regen_on_wake_up`, `bmh:only_carry_one_eternal_item_type`, `bmh:damage_eternal_item_when_dead`, `bmh:remove_coordinates_in_survival`, all default true, category MISC, type BOOL (`registerBoolean`, lines 79-85, through `Services.PLATFORM.register(Registries.GAME_RULE, name, ...)`). They show in `/gamerule` and are saved in the game rules saved-data file. `ALL_RULES` is flipped together by the world-creation button (`CreateWorldGameTabMixin`, client group). `N_SavedDataGameRulesMixin` (N_SavedDataGameRulesMixin.java:24-28) rewrites old camelCase keys (`randomDeathSpawnPoint` and so on, BMHGameRules.java:49-55) to the new ids in `readTagFromDisk` RETURN, converting text "true"/"false" to booleans (lines 61-75). Rules are read per level: `level.getGameRules().get(RULE_X)`.

### Horizon mapping

- Rules are a closed table: `pub const RULES: &[Rule] = &[ b("advance_time", "updates", true), ... ]` (S/src/horizon/crates/horizon-client-core/src/gamerules.rs:50). `pub fn rule(name: &str) -> Option<&'static Rule>` (:138) looks up that table. `Rule` has `&'static str` names. Persistence: "persistence in the world's `data/minecraft/game_rules.dat` saved-data file" (:1-3). I searched native_hooks.rs, native-mods.md, native-hooks.json and the fragments for `game_rule` / `gamerule` / `GameRule`: there is no registration hook. Hooks and helpers that exist only read, set or hear vanilla rules:
  - `world_game_rule_change(player, name, rule, value)` hook (catalog both, API 247, native_hooks.rs:6403-6411): "A server game rule changed to its final `value`", "after an operator changes a world game rule through the server's game rule screen".
  - `LevelRef.game_rule(host, "keep_inventory")` -> `Option<GameRuleValue>` (native_level_world.rs:34-41, `gamerules::rule(rule)?` so an unknown rule gives None) and `LevelRef.set_game_rule(host, rule, value)` (native_level_world.rs:80-95): both PENDING (fragments level_world.json and level_world_setters, placeholders `@API:level_world@` and `@API:level_world_setters@` in docs/native-mods.md:3867-3877). The doc claims only `advance_time` and `advance_weather`; the code accepts any rule in the table (native_level_world.rs:80-84).
- Closest replacements for the six BMH rules:
  1. A `Server`-kind config (`NativeConfigKind::Server`, native_configs.rs:34-41): "the world's `serverconfig/<file>` when there is one, else `config/<file>`", i.e. a per-world TOML, which is the closest thing to a per-world rule store. It is read at world start and reloaded within a second when the file changes (native_configs.rs:1-12, `CHECK_EVERY = 1 s`). Changes through a command need the mod to write the file (it can: `config_dir()` and ordinary file I/O).
  2. A level attachment (`Holder::Level`): saved in the dimension's `data/minecraft/horizon_attachments.dat` (docs line ~5465), changed by a `/bmh rule <name> <true|false>` command (feature 6). Not visible in `/gamerule`, tab completion or the game rules screen.
  3. A vanilla rule used as a switch is not possible: no unused rules exist.
- Reading vanilla rules the mod needs (`reduced_debug_info`, `keep_inventory`, `show_death_messages`) is the pending `level.game_rule`; `reduced_debug_info`, `keep_inventory`, `immediate_respawn`, `respawn_radius` are in the table (gamerules.rs:74-75, 109-110).
- Migration of old keys (N_SavedDataGameRulesMixin) is irrelevant: a Horizon world has no BMH rules to migrate.

### Verdict

GAP for registered, namespaced, `/gamerule`-visible rules saved with the world. Missing in Horizon: a registration path for `Rule` (the table is a `const` of `&'static` data). PARTIAL replacement through a Server config or a Level attachment, with the visible-in-/gamerule behaviour lost.

### Parity risks

Players lose `/gamerule bmh:...`, tab completion and the create-world button's integration (the button is client UI; its target store changes). Per-world values live in a file or an attachment, not in the game rules screen.

---

## 5. Config

### Behaviour spec

NeoForgeConfig.java:13-34: `config/bmh-common.toml` via `ModConfigSpec`, three int values, with comments: `minDistanceFromBed` (default 5000, range 5000..200000), `maxDistanceFromBed` (default 60000, same range), `maxAmuletPoints` (default 5, range 1..16); limits in BMHConfigLimits.java:9-17. `save(min, max, points)` sets values and writes the file (lines 37-43). Read by the platform helper (`NeoForgePlatformHelper.minDistanceFromBed()`, lines 53-55). `ConfigScreen` (206 lines, client only): three EditBoxes with the ranges, a Done button that calls `saveConfig`; "It changes the local file only: a dedicated server keeps its own file" (ConfigScreen.java:20-24).

### Horizon mapping

- `register_configs` (catalog both, side `both`) and `config_loading`, `config_reloading` (both, `[b v1]`), `config_unloading` (`[s v1]`): `native_register_configs(&mut Vec<NativeConfig>)`, `native_config_loading(&NativeConfigFile)` (native_configs.rs:250-310, mod_api types at :22-95). `NativeConfig::new("bmh", NativeConfigKind::Common, DEFAULT_TEXT)` writes `config/bmh-common.toml` when missing (native_configs.rs:63-73: "`config/<mod_id>-<kind>.toml`, `default` when missing"), the same file name as the NeoForge build. `Common` is "Once per process: the window when the game starts, or a dedicated server when it starts. A single player's server thread shares the window's." The file text is handed over ("the mod parses it"), so the TOML parse, the clamping to the ranges and the `min > max` rule are the mod's code. Reload is by modification time, once a second (`CHECK_EVERY`), fires `native_config_reloading` with the new text; the same text saved again fires nothing.
- Per-world override with `Server` kind (`serverconfig/` of the world), a thing the Java mod does not have.
- Settings page: `native_mods(&mut Vec<NativeMod>)` (Options > Mods row), `native_settings(mod_id, &mut Vec<String>)` (push button labels, asked again after each click), `native_settings_click(mod_id, button)` (native_hooks.rs:3165-3177). Hooks `settings`, `settings_click` are in the mod-code table (S/horizon-docs/mod-code.txt, "The first eleven"). Buttons only: no slider, no text box. `native_set_setting(mod_id, key, value)` / `native_setting` store one file per key in `horizon-mods/.horizon-mod-data/<id>/` (native_hooks.rs:11551-11570), per instance and unrelated to `config/`. The three config values can be edited by cycling presets with buttons, or by a mod-owned screen: `native_mod_screen_open`, `native_mod_screen_close`, `native_mod_screen` (API 111, native_hooks.rs:641-642) plus `NativeTextField` (public in mod_api.rs, `crate::game::native_text_field::NativeTextField`). I read neither in detail (UNVERIFIED feasibility of a text field with range validation).
- Server side values: on a single-player or LAN world the window's Common file is the server's. A client joining a remote server never needs the values.
- `config_dir()` (catalog both, API 26, native_hooks.rs:11578): `config/` beside `options.txt`; "a dedicated server's is in its own folder, beside `server.properties`".

### Verdict

HOOK for the file and its values. PARTIAL for the screen (buttons only without the mod-screen API).

### Parity risks

Hand editing the file works the same; the in-game screen differs in look and validation; a dedicated server and a client still keep separate files, as in Java.

---

## 6. Commands

### Behaviour spec

AmuletPointCommand.java (registered into the Brigadier dispatcher by `J_CommandsMixin` at `Commands.<init>` TAIL, J_CommandsMixin.java:21-25). Three subcommands under `/bmh point` (lines 45-54), permission: none (any player):
- `save <pos:block_pos> <name:greedy string>`: `BlockPosArgument.getLoadedBlockPos`; `allowed(player, pos)`; `AmuletPoints.cleanName(name)` (empty gives `bmh.point.no_name`); if the point is new and `used >= AmuletPoints.max()` give `bmh.point.full` (arg max); `AmuletPoints.put`; remember the game time; `playSound(amethyst_block.chime, PLAYERS, 1.0, 0.7)` at the player; action-bar message `bmh.point.renamed` or `bmh.point.saved` with the name; return 1 (lines 97-129).
- `remove <pos:block_pos>`: `allowed`, find the point at `pos` and its block, remove it, remember time, action-bar `bmh.point.removed` (lines 131-151).
- `delete <x:int> <y:int> <z:int>`: only needs an amulet in either hand; removes the point at those coordinates (no block, no distance check), action-bar `bmh.point.removed`, then `AmuletPointSignal.list` resends the list (lines 62-89). The class comment says "BMH has no network channel, so the screen sends a vanilla command packet" and lists only save and remove plus delete (lines 28-34).
- `allowed` (lines 154-177): holds the amulet (main or off hand) else `bmh.point.no_amulet`; dimension is the Overworld else `bmh.point.overworld`; alive, not spectator, within 8 blocks (`distanceToSqr` to block centre <= 64) and `AmuletPoints.canHold(blockstate)` (a bed or lodestone) else `bmh.point.too_far`; at least 20 ticks since the player's last change (also allowed when game time went backwards).
- Messages are action bar (overlay) messages, translatable (`en_us.json` lines 59-70).

### Horizon mapping

- `register_commands` (catalog both, `[s v1] RegisterCommandsEvent`): `native_register_commands(&mut Vec<NativeCommand>)` (native_hooks.rs:4354). Usage strings (mod_api.rs:742-777): "words, then arguments `<name:kind>`, optional ones last in brackets ... Kinds: `word`, `text` (the rest of the line, last), `integer`, `number`, `boolean`, `player`, `item`, `block_pos`". So `NativeCommand::new("bmh point save <pos:block_pos> <name:text>")`, `"bmh point remove <pos:block_pos>"`, `"bmh point delete <x:integer> <y:integer> <z:integer>"`. "A name vanilla has, or a usage that doesn't parse, is left out with a warning in the log" (docs line 2243-2244): `bmh` is free. `NativeCommand::new` is level 0 (everyone).
- Execution: `perform_command` (catalog both, no side tag): `native_perform_command(&mut self, player: &NativeServerPlayer, command: &str, ran: &mut bool)` (native_hooks.rs:4364-4372): "A player's command on the server, without its slash, for a command a mod registered and the player's level allows: set `ran` when it is yours. The player's state is swapped in for the call, so `native_player` and the other helpers read that player; `native_feedback` answers them." The mod gets the raw command line and parses it itself (the kinds drive client completion and the server's usage match; whether `block_pos` arguments with `~` are resolved for the mod is UNVERIFIED: the mod receives `command: &str`). `player_command` (catalog both, cancellable, native_hooks.rs:4615) is the earlier hook that sees every command including vanilla ones; not needed.
- Because the swapped-in state is the player's, the checks map: item in hand (`native_held()`, native_server.rs:1253, "The main hand's stack"), dimension (`native_player()` gives `dimension`, mod-code.txt), alive (swapped-in player is alive: dead players do not act), spectator (`NativeServerPlayer.game_mode` via `native_acting_player`), distance (`position`), block state at `pos` (`native_block(pos)`, native_hooks.rs:11388-11392, "air where nothing is loaded", which also stands in for `getLoadedBlockPos`), game time for the 20-tick limit (`level_game_time`).
- Messages: `action_bar` (catalog helper, native_server.rs) for the overlay line; sound via feature 2's routes (the player alone: pending `play_sound_to`, or `/playsound` from a non-player tick).
- The client screen (PointScreen) sends the command with `native_send_chat("/bmh point save ...")` (API 404, docs "Chat and commands a client mod sends", `ClientPacketListener.sendCommand`: "A leading `/` sends the rest as a command, signed where its arguments are messages"). A mod message (`net_send_to_server` on `mod_id:name`) is cleaner: `native_net_from_client(player, channel, data)` hears it; "What a client sent is checked by nothing, so treat it as a player's own input", the same server-side checks apply. Both keep the "server checks everything" design.

### Verdict

HOOK.

### Parity risks

Brigadier features lost: no `~` relative coordinates guaranteed, no colour-coded syntax for `greedy string` beyond the usage kinds, error messages for bad input are the mod's. The three subcommands behave the same.

---

## 7. Advancements

### Behaviour spec

W/common/src/main/resources/data/bmh/advancement/ holds `root.json` and 19 entries in three folders (20 files total): `eternity/` (awakening, echo_heart, echo_sense, echo_strike, eternal_tool, everything_stays, hollow_totem, moon_ritual, saved, smash), `journey/` (exile, far_away, here_we_go_again, home_at_last, long_way_home) and `navigation/` (amulet, named_place, own_sky, stargazer). `root.json`: criterion `start` with trigger `minecraft:tick`, display icon `bmh:navigator_amulet`, `announce_to_chat: false`, `show_toast: false`, background `minecraft:gui/advancements/backgrounds/stone`. A code-awarded entry such as `journey/exile.json`: parent `bmh:root`, criterion `done` with trigger `minecraft:impossible`, requirements `[["done"]]`, icon `minecraft:bell`, `announce_to_chat: false`; `journey/home_at_last` has parent `bmh:journey/exile` and icon `minecraft:red_bed`. The others in the code-awarded set use the same `done` criterion; entries with `minecraft:inventory_changed` need no code. `BmhAdvancements.award(player, path)` finds `bmh:<path>` in the server's advancement manager and calls `player.getAdvancements().award(holder, "done")`, returning whether this call completed it; `exists` and `isDone` exist too (BmhAdvancements.java:31-50). `here_we_go_again` is hidden until earned (CLAUDE.md Code map). Awarded from: RespawnEffectHandler (`journey/exile`, `here_we_go_again` when deaths >= 10, `far_away` when distance >= 30000), HomecomingHandler (`home_at_last`, `long_way_home` when distance >= 30000), and handlers of other groups (eternity and navigation).

### Horizon mapping

- Advancement JSON: yes, as data. docs/mods.md:31 lists "loot tables, advancements, functions" among content mod data; docs/native-mods.md "Advancement triggers": "The advancement is an ordinary file of a content mod or a world's data pack (`data/mymod/advancement/connect_a_pipe.json`)". The folder is `advancement` (singular), the same as the Java 26.1 branch. A native `.hmod` can carry both: "A mod's content and its native code ship as one `.hmod`" with `horizon-mod.toml`, `assets/` and `data/` for content (docs/native-mods.md, "One .hmod for content and code"). The hook `data_pack_files` (catalog both, `[? v1]`, "the hook `data_pack_files`" in native-mods.md "Data files" and the API 108 text) exists for a native mod to add data files at run time; I did not need it.
- Awarding from code: `native_trigger_criterion(uuid, trigger, test)` (native_criteria.rs:50-87, API 69, header comment native_hooks.rs:765-766). "Fires the mod's criteria trigger `trigger` (`mymod:pipe_connected`) for the player with that UUID ... Each criterion of theirs with that trigger, not yet granted, of an advancement not yet done, is tested by `test` and then by its `player` predicate; those that pass are granted; an advancement they complete is awarded with its toast, chat line and rewards". The trigger must be namespaced and not `minecraft:` (`mod_trigger`, native_criteria.rs:36-40: `ns != "minecraft"`), so `minecraft:impossible` cannot be fired and every code-awarded JSON changes to `"trigger": "bmh:award"` (one trigger, a `conditions` field `{"id": "journey/exile"}` tested in the closure with `criterion.conditions["id"]`; `NativeCriterion.conditions` is a `serde_json::Value`, native_criteria.rs:12-20). Reach: "the acting player is reached inside their hook, and any player on the server ... only where no player's state is swapped in (`native_tick`, a menu hook)". The respawn and the 1 Hz check run in `player_respawned` (acting player) and `server_tick` (any player): both reach it. In `server_tick`, "Fires ... for one player, on a server only" (`self.remote.is_some()` returns false).
- Catalog status of `trigger_criterion`: not in native-hooks.json (main), the Nightly or hooks-compact.txt (grep: 0). It is in the source with an API number, API 69 ("API 69 adds a mod's own advancement criteria triggers (`native_trigger_criterion`, in `native_criteria.rs`)", native_hooks.rs:765-766). Typed form: `Host::trigger_criterion(uuid, trigger, &test)` and `trigger_mod_criterion` (fragment host_bridge.json; docs table "| `trigger_criterion(uuid, trigger, &test)` | `trigger_criterion` | The same call, with the test as `&dyn Fn`"). Reading the catalogs only, this would look unavailable; reading the source it is a stamped API-69 `Game` method a mixin helper can call. I rate it HOOK on the strength of the API number and flag the catalog gap as an open question.
- Reading progress (`isDone`, `exists`): hooks `player_advancement_done` and `player_advancement_progress` (catalog both) hear completions; the pending `native_player_advancements(uuid)` reads progress (fragment player_records.json); display read `advancement_display` is pending (native-mods.md:5554-5556). `BmhAdvancements.isDone/exists` are not needed for the awards themselves; other handlers' uses are in other groups.
- Hidden advancement and `announce_to_chat: false`: plain JSON fields, handled by the loader.

### Verdict

HOOK. Cost: rewrite the code-awarded JSON triggers to a `bmh:` trigger, keep the rest of the JSON. `minecraft:impossible` as a Horizon trigger is UNVERIFIED (not searched: irrelevant once replaced).

### Parity risks

None visible in the player experience if `bmh:award` is handled. Players who do not run the mod see unknown trigger errors in the advancement files only if the content part is installed without the code part (a single `.hmod` avoids it).

---

## 8. Smaller server handlers

### 8a. SleepHandler

Java: `PlayerWakeUpEvent` (Forge/NeoForge) or a Fabric mixin on `Player.stopSleepInBed` calls `onPlayerWakeUp` (SleepHandler.java:21-33). If the rule `food_and_health_regen_on_wake_up` is true, `healthAndFoodRegen` (lines 35-77) with `food = foodData.getFoodLevel()`: `food >= 15`: heal 6, add exhaustion 4, and if health was already `>= 20` give Haste II (`MobEffects.HASTE`, 3600 ticks, amplifier 1, no ambient, no particles, icon shown) and Resistance II (3600 ticks) (the check uses `playerHealth` read before healing), then `food - 4`; `food >= 10`: heal 4, exhaustion 4, `food - 2`; `food >= 5`: heal 3, exhaustion 4, `food - 1`; `food > 1`: heal 1, exhaustion 1 (`4/4`), `food - 1`.

Horizon: `player_bed_leave(player, name, pos)` (catalog both, `[s v1] PlayerWakeUpEvent`, API 252): "A server player woke or stood up from a bed, after their sleeping state changed ... helpers looking up the guest through `self.lan` may find nothing during guest packet handling" (native-mods.md:2451). Health, food, exhaustion and effects: PENDING helpers `native_player_food(uuid) -> (food, saturation, exhaustion)`, `native_set_player_status(uuid, health, food, xp)` (native_player_state.rs:216, :248, fragment player_status.json: "sets each that is given ... Health is capped at their maximum") and `native_add_effect(uuid, &NativeEffect)` (native_player_state.rs:337, fragment player_effects.json; `NativeEffectV2` has `show_icon`). Exhaustion can be set only through `native_set_player_status` (`setExhaustion`). `heal` and `food_level_changed` exist as hooks (`living_heal`, `food_level_changed` in hooks-compact.txt) that hear only. Stamped fallback: `run_command("effect give <Name> minecraft:instant_health ...")` and `effect give ... haste 180 1`; there is no vanilla command for food or exhaustion. Verdict: PINNED (pending status and effect helpers).

### 8b. KeepItemAfterDeath

Java (KeepItemAfterDeath.java:32-74): at death (before drops), for every inventory slot holding an `IEternalItem`: if survival and rule `damage_eternal_item_when_dead` call `reduceDurability(stack, 7)` (the 7% wear), save `stack.copy()` in `itemsToKeep[uuid]`, remove the stack from the inventory. At respawn, `player.getInventory().add(stack)` for each, then forget.

Horizon: `loot_drops` receives the same stacks as the death drops (item_entities.rs:605-618, "Add, change or take out stacks; empty ones don't drop", native_hooks.rs:9189-9190). Take the Eternal stacks out of `drops`, apply the wear through the stack's components (`component`/`set_component` helpers, `StackData`; the damage component details are another group's), remember them in a map by uuid; in `player_respawned` call `native_give_stack(uuid, stack) -> Option<ItemStack>` (native_server.rs:1328, API 98: "Gives `stack` whole (its damage, name, enchantments... kept) to the player `uuid`, as `Inventory.add` places it ... What did not fit comes back", catalog both). Note the Java adds to the new inventory with `Inventory.add`, same placement. Limits: `keep_inventory` true skips `drop_inventory` entirely (nothing taken, nothing worn); creative and spectator deaths: spectators do not drop (`drops` is `None` for spectators, lan_actions.rs:4521-4523), creative drops happen (the Java mod keeps the item but skips the wear outside survival: the mod needs the game mode map). HOOK.

### 8c. EternalItemHandler

Java (EternalItemHandler.java): `onPlayerTick` at the end of every server player tick: `TotemOfEternityHandler.dropExtraTotems`; if the rule `only_carry_one_eternal_item_type` is true and the player is in survival, scan the inventory; for the second and later stack of the same Eternal item, `player.drop(copy, false)` and `shrink(1)` (the stack count shrinks by one only; empty slot cleared) (lines 24-55). `shouldBlockPickup(player, stack)` cancels the pickup of an Eternal Item when the player already has that item type; for the Totem of Eternity always, for others only when the rule applies (lines 60-77).

Horizon: scan in `player_tick` (catalog both, swapped in) or `inventory_tick(slot, selected)` (catalog both, API 39, `native_carried_stack(slot)` read and change); pickup block in `item_picked_up` (`[s! v1]`, catalog both, cancellable, API 40: "`player` is about to take `count` of `item` from the ground ... Set `canceled` to leave it there: the stack isn't offered again for 10 ticks", native_hooks.rs:9234-9236) or the pending `item_picked_up_v2` with the whole stack. Dropping the extra copy: I found no stamped helper that spawns an item entity from a mod. `native_summon(dimension, kind, pos, nbt)` (native_entities.rs:215) can summon from NBT, "a kind Horizon can't load" returns None: UNVERIFIED for `minecraft:item`. `native_set_item_entity` / `native_item_entities` (native_hooks.rs:10787, 10815) exist for existing entities. PARTIAL.

### 8d. DisableDebugInfo

Java (DisableDebugInfo.java:29-60): on every player tick, if the rule `remove_coordinates_in_survival` is true and any online player is in SURVIVAL, force the vanilla rule `reduced_debug_info` to true (remembering the previous value once, `valueBeforeForcing`), calling `gameRules.set` only on a change; otherwise restore the saved value. State is forgotten when the server instance changes.

Horizon: `reduced_debug_info` is a vanilla rule (gamerules.rs:109) and the window applies it: `set_reduced_debug_info(self.ui.rules.bool("reduced_debug_info"))` for the hub (lan.rs:2797) and `debug_options.rs:216 let reduced = self.reduced_debug_info()`. Set it with `native_run_command("gamerule reduced_debug_info true")` from `server_tick` (no player swapped in), reading `server_players()` for game modes. The previous value needs the pending `level.game_rule(host, "reduced_debug_info")` (native_level_world.rs:34); without it, assume false. HOOK for forcing, PARTIAL for the restore. The "rule on/off" is the mod's own switch (feature 4).

---

## Open questions

1. Is `player_respawn_position` going to be stamped soon? It sits in a fragment, unstamped, with 228 other fragments. The whole port is blocked on it unless the mod pins a main commit.
2. Does `surface_heights` or any other helper read a parked level (the Nether during an anchor respawn on a dedicated server)? I only found "the level being ticked" (native_world_columns.rs:26-28).
3. What happens to a player placed at a `pos` whose chunk is not loaded (`Hub::respawn`, then streaming)? Cost on the server thread and the player's fall are unverified.
4. Does the End-exit respawn (`player_clone` with `was_death = false`) apply `copy_on_death`? `guest_respawn` is the death path; the End exit is another route (`respawn_trip`, `arrive_server_trips`).
5. Is the hub really out of reach (`self.lan` empty) inside `living_death`, `player_died` and `loot_drops` of a guest death? I read `guest_die(&mut self, host: &mut LanHost, ...)` taking the hub as a parameter, which implies it, but `as_guest` swaps the player in for `loot_drops`; I did not trace `self.lan` in `tick_guests`.
6. Damage type names for `BYPASSES_INVULNERABILITY` on 26.3 (`minecraft:out_of_world`, `minecraft:generic_kill`).
7. Does `block_name_in_tag(block, tag)` (docs Host table) work on the server for sturdy-face or leaves tests, and are there biome tags? Not traced.
8. Selector support (`@a[name=!X]`, `distance=`) in Horizon's `/playsound` through `run_command`.
9. `native_trigger_criterion` is absent from both catalog JSONs but stamped API 69 in the source; confirm that a `hooks: 69` mixin-style bundle can call it.
10. Is there any way to read `hardcore`? None found.
11. Can a native mod text-field config screen be built with the mod-screen API (API 111) and `NativeTextField`? Not read.
12. Whether a mod crate may parse TOML with a dependency (the Java config is TOML); docs "The mod's crate" (native-mods.md:565) was not read.

## Step log

1. Verified the worktree: `git rev-parse --show-toplevel` printed W and the branch printed `horizon-26.3`.
2. Read the common brief; listed the scratchpad.
3. Scanned hooks-compact.txt for the hook names of the group and listed the catalog size (434/177 main, 432/174 Nightly) with a python parse of both JSON files.
4. Found `native_player_respawn_position` in native_hooks.rs:7441 with no catalog entry; found its fragment and the fragments README ("not API yet").
5. Read make-a-mod.txt and mod-code.txt (two ways to write code, "Pinned to a Nightly").
6. Read `guest_respawn` (lan_actions.rs:4019-4224) and `Hub::respawn` (hub.rs:5025-5100), the client's respawn path (screens.rs:770, remote.rs:10497) and the packet dispatch (lan_actions.rs:3082-3090).
7. Read `guest_die` (lan_actions.rs:4471-4640) and `drop_inventory` (item_entities.rs:601-625): found `native_loot_drops` as the swapped-in capture point.
8. Read all of the Java: RespawnPositionHandler, RespawnSearch, the three F_ mixins, RespawnEffectHandler, JourneySignal, HomecomingHandler, E_/C_ServerPlayerMixin, BMHGameRules, N_SavedDataGameRulesMixin, BMHConfigLimits, NeoForgeConfig, BmhAdvancements, AmuletPointCommand, J_CommandsMixin, SleepHandler, KeepItemAfterDeath, EternalItemHandler, DisableDebugInfo, the loaders' event classes, and the advancement JSON.
9. Read the chunk ticket, column read, biome, collision, sound, title, attachment, config, command, criteria and level-world sources and their docs sections in native-mods.md.
10. Checked catalog membership of every hook and helper relied on in both catalogs (python parse), and the fragment list for pending ones.
11. Checked horizon-mixins README and advanced-injections.md for cancellable HEAD.
12. Wrote this report.
