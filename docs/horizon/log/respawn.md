# Respawn area: port log

Area: respawn (GAPS.md rows C1 to C9). Files: `horizon/native/mixins/bmh/bmh_respawn.rs`,
`horizon/native/mixins/bmh/logic/src/respawn.rs`, this log.

## Steps

1. The worktree was not on BASE 6e5f17b (no `horizon/` folder). Branch `worktree-wf_1f6ae7b1-123-1`, so the
   brief allows the reset: `git reset --hard 6e5f17bf1161e8e0d8c1baae93374b379f60d472`. Guard file present.
2. Read ARCHITECTURE.md, HOOKS.md, GAPS.md, audit 1 features 1 and 2, verify-1 rows 1a to 2d, the Java sources
   (RespawnPositionHandler, RespawnSearch, RespawnEffectHandler, JourneySignal, F_PlayerListRespawnMixin,
   F_ServerGamePacketListenerMixin, F_ServerPlayerJourneyMixin, BmhAdvancements) and NeoForgeEvents.
3. Read HN (the Nightly). Findings that settle the open points of the brief:
   - Death order in `guest_die` (lan_actions.rs:4292): `living_death` (cancellable, no player swapped in, entity
     id only), then `player_died(uuid)` after `g.dead = true`, then `drop_inventory` (inside `as_guest`, so the
     player is swapped in) calls `loot_drops` unless `keep_inventory` is true or the player is a spectator, then
     the `deaths` statistic. So `player_died` marks a real death, and `loot_drops` gives the exact Eternal count.
   - `player_tick` does not fire for a dead player (doc native_hooks.rs:2786-2789). `server_tick` fires each
     tick, but "which level `self.world` holds here is not defined" (doc :2566). The column and block reads
     (`surface_heights`, `native_block`, `native_block_collision`, `native_biome`) read only the level in place
     (`native_level_open`, native_world_edit.rs:63). `tick_server` (levels.rs:491-560) calls
     `enter_level(None, dim)` and then `native_level_tick_post(dim)` after that level's tick. So the search
     advances in `level_tick_post` (stamped, API 23), which I wire (a change outside my files).
   - Chunk tickets: `native_force_chunk_in(owner, dimension, chunk, hold)` (native_chunk_tickets.rs, pending
     fragment `chunk_tickets_in_level`). Not time-limited: the mod releases. A held chunk loads with its ring.
   - Border: `native_world_border(dimension) -> Option<[f64; 4]>` (pending fragment `world_border`).
   - Biome: `native_biome(pos)` (pending fragment `session_state`), None where the chunk is not loaded. So the
     ocean and river test runs after the load (GAPS C3).
   - Difficulty: `native_level_difficulty(LevelRef)` (pending fragment `level_world`).
   - Deaths statistic: `native_player_stat(uuid, "minecraft:custom", "minecraft:deaths")` (pending fragment
     `player_records`). Tier F is allowed by the brief, so C9 uses the real statistic, not a count from install.
   - `living_incoming_damage_v2` fires only for a player's weapon hit on a mob (doc native_hooks.rs:9267-9273).
     So C6 uses `living_incoming_damage` (v1, no `Game`), as already wired to this area.
   - `run_command` runs "from the overworld's spawn" and only where no player is swapped in (native_server.rs:
     725-737). `cmd_playsound` sends only to players in `self.dim` and within the range (16 for volume 1). So
     the sounds run from `level_tick_post` of the player's level, and the awards from `server_tick`.
   - The respawn point: no helper reads it. `guest_respawn` (lan_actions.rs:3892-3960) hands
     `player_respawn_position` the point of `respawn_elsewhere` (pos + 0.5 in x and z) or the world spawn
     (`level.world_spawn` + 0.5). `floor(pos)` is the Java centre block. The search starts at death, so the
     area must know the point at death: it tracks `player_spawn_point_set` (stamped, API 277; wired by me) and
     seeds a player it does not know from the player's own `.dat` (`respawn` compound, the same one
     `Level::read_player` reads, save.rs:4029-4042). The world spawn comes from `level.dat` (`spawn.pos`) and
     `spawn_change` (wired, I add my call).
   - Hardcore (C5, substitute): `level.dat` `difficulty_settings.hardcore`, read with
     `horizon_client::save::Level::read(world_dir)` (save.rs:3816, :3857).
   - Every change of a respawn point fires `player_spawn_point_set`: bed (game.rs:14996, cause `bed`, the
     head block), anchor (respawn_anchor.rs:193), `/spawnpoint` (commands.rs:3479), and the reset of a point
     whose block is gone (server_travel.rs:660, `death_reset`).
4. Wrote `logic/src/respawn.rs`: constants with their Java lines, the distance formula, `level_distance`,
   `edge_point`, the border test, the biome list, the block tests (`is_free`, `is_safe_ground` on collision
   boxes), `find_safe_position` over trait `Blocks`, the search state machine `Search` over trait
   `SearchWorld`, `landing` (the decision inside the respawn), `journey_awards`, `award_command`,
   `sound_commands`, `blocks_damage`, `count_down`, and a SplitMix64 `Rng`. 31 unit tests with a fake
   world, Java values computed by hand (23333, 11111, 35555, 2917 for the Nether, the 45 degree point 707).
5. Wrote `bmh_respawn.rs` (flow at the top of the file) and the changes outside my files (below).
   First compile of the bundle: ok. The staged game.rs and native_hooks.rs hold the new code (checked).
6. Fixed one gap: when no world spawn is known, `Departure` went out with no search, so a client would hold
   the Respawn click for ever. Now `Ready` follows at once. Both gates pass (52 unit tests; check ok).

## State and the world lifecycle

All per-player state is in one `SERVER` static (a `Mutex`, BTreeMaps keyed by uuid or entity id). The
`player_logged_out` hook fires for each player when a server stops (doc native_hooks.rs:8445-8450), so a closed
single-player world leaves no player entry. The world spawn that `spawn_change` told is stored with the world
folder and used only for that folder. So the area needs no `world_lifecycle` wiring. Two leftovers are
harmless: a held landing chunk of a closed world (its release in the next world returns false), and the RNG.

## Differences from the Java mod

1. The ocean and river test runs after the candidate chunk loads, not before (GAPS C3: `native_biome` reads
   only loaded chunks). Each rejected candidate costs a load of a 5x5 ring. The biome list and the leaves,
   fluid and sturdy-face tests are hard-coded vanilla lists (GAPS C3). The sturdy test reads the collision
   boxes on a 16 by 16 grid, plus soul sand and mud (full support shape, lower collision box).
2. The candidate angles come from a SplitMix64 source seeded from the clock, not from the level's random
   (`serverLevel.getRandom()`, RespawnPositionHandler.java:199). Same distribution (uniform in [0, 1)),
   different draws. Allowed by the brief.
3. A search that did not end, or that does not fit the respawn, gives the vanilla point. Java searched again
   at once with a blocking chunk load (RespawnPositionHandler.java:139-145). Horizon has no blocking load that
   a hook may call (`/forceload` blocks up to 120 s, verify-1 row 1c). GAPS C4: a gamepad click,
   `immediate_respawn` and a client without the mod skip the hold, then C2 falls back to the vanilla point.
   Also no distant respawn when the server restarted between the death and the respawn (Java recomputed).
4. The search starts at the end of the server tick of the death (ServerTickEnd), not inside the death event,
   because the Eternal count comes from `loot_drops`, which fires after `living_death` and `player_died`.
   Same tick. The `Departure` message leaves at the tick's end in both cases.
5. The landing chunk is held for 6000 server ticks from the moment the search found it. Java's ticket counts
   from the moment it was added.
6. Hardcore (GAPS C5, substitute): read from `level.dat`. A world that has no `level.dat` yet counts as not
   hardcore. Also, in a hardcore world no search starts, so the respawn keeps the vanilla point. Java started
   no search either, but its respawn then searched with a blocking load and moved the hardcore spectator.
   GAPS C5 says "no distant respawn and no journey", which is what this build does.
7. The respawn point at death comes from the area's own copy (the `player_spawn_point_set` hook, else the
   player's `.dat` file), not from a read of the game. A mod that cancels the hook after BMH heard it makes
   the copy wrong; the respawn then does not match the search and keeps the vanilla point.
8. A bed that is destroyed between the death and the respawn: Java searched again around the world spawn.
   Here the found landing around the bed applies (the game hands over the old bed point, GAPS C2).
9. The 200 ticks count down at the end of each player tick (`player_tick`), Java at its start. Damage types
   that bypass the immunity are the vanilla tag as a list (GAPS C6).
10. The bell and the respawn sound play one level tick later, through `/playsound` at the player's block
    with `@a[name=!<player>]` (GAPS C7). `/playsound` plays to the players of the level in place within 16
    blocks, as `playSeededSound` does.
11. The advancements go through `advancement grant <name> only bmh:journey/... done` at the end of the tick
    (GAPS C9, verify-1 row 2d). The tenth death reads the real `minecraft:deaths` statistic with the pending
    helper `player_stat` (tier F), so the count does not start at install.
12. The journey signals are mod messages (decision D6): `Departure`, `Ready`, `Arrival { name }` on
    `bmh:journey`. A client without the mod sees nothing (GAPS C8).
13. Difficulty: read at death with the pending `level_difficulty`; when it answers nothing, normal (2) is
    used. `difficulty_changed` is not needed and its function does nothing.

## To check in the game

1. `level_tick_post` fires for the Overworld of a single-player world while the owner is dead on the death
   screen (the integrated server is not paused there). Check: die in survival, watch the log for
   `[BalancedMcHardcoreMain] Landing spot ready after`.
2. A chunk held with `force_chunk_in` far away loads, and `surface_heights` answers for it, within seconds,
   with the chunk fully generated (trees, features). Check: the same log line, then look at the landing.
3. `native_block_collision`, `native_block` and `native_block_property(.., "waterlogged")` read the level in
   place from `level_tick_post`. Check: a landing never in water, never on leaves (die near an ocean).
4. `native_biome` answers in a loaded far chunk. Check: many deaths near oceans give no beach or ocean
   landings in water; the log shows released candidates.
5. The respawn point copy matches what the game hands to `player_respawn_position`: sleep in a bed, die,
   respawn far around the bed. Then break the bed, die: the exile is around the world spawn. Then
   `/spawnpoint` somewhere, die. Then die in a new world with no bed (centre: the world spawn of `level.dat`).
   Watch for "No distant respawn for <name>" in the log, which means a mismatch.
6. The owner's `.dat` (players/data/<uuid>.dat) holds the `respawn` compound in single player.
7. `level.dat` exists with `spawn.pos` and `difficulty_settings.hardcore` in a brand new world before the
   first death (GAPS 7, question 11). Check: create a hardcore world, die at once: no journey.
8. `run_command` works from `server_tick` (ServerTickEnd) and from `level_tick_post` in single player (GAPS 7,
   question 1). Check: `/advancement` screen shows "Not the end" after the first distant respawn, and a second
   player near the landing hears the bell.
9. `advancement grant` through `run_command` shows the toast (GAPS 7, question 8).
10. `living_incoming_damage` fires for a player and its `entity` equals `acting_player().entity_id`. Check:
    after a distant respawn, jump from a height or stand in lava within 10 s: no damage; `/kill` still kills.
11. The client of the journey area receives `Departure`, then `Ready`, then `Arrival` and releases its held
    Respawn click on `Ready` (C4). Check: the death screen button waits until the log line "Landing spot
    ready", then the respawn is at the far landing.
12. `native_server_players()` lists a dead player with `health` 0 inside `net_from_client`.
13. `native_player_stat(uuid, "minecraft:custom", "minecraft:deaths")` counts this death in
    `player_respawned`. Check: the tenth death gives "Shit, here we go again".
14. `native_level_difficulty` answers on the integrated server at the end of a server tick.

## Changes outside my files

1. `horizon/native/mixins/bmh/hook_level_tick_post.rs` (new) and its `[[injection]]` in `horizon-mixin.toml`:
   wires the stamped hook `level_tick_post` (API 23) to `bmh_respawn::on_level_tick_post`. Reason: the
   search must read the blocks of its level while that level is in place, and `/playsound` reaches only the
   players of the level in place.
2. `horizon/native/mixins/bmh/hook_player_spawn_point_set.rs` (new) and its `[[injection]]`: wires the
   stamped hook `player_spawn_point_set` (API 277) to `bmh_respawn::on_player_spawn_point_set`. Reason: no
   helper reads a player's respawn point, and the search starts at death.
3. `horizon/native/mixins/bmh/hook_spawn_change.rs`: one line, `bmh_respawn::on_spawn_change` before the
   amulet's call. Reason: the world spawn is the centre of an exile with no bed. The amulet's call is
   unchanged.
4. `horizon/native/mixins/bmh/bmh.rs`: `pub(super) fn award(game, name, path) -> bool`, built on
   `advancement grant <name> only bmh:<path> done` (the line comes from
   `bmh_logic::respawn::award_command`). Other areas reuse it from `server_tick`.
5. `docs/horizon/HOOKS.md`: rows for `level_tick_post` and `player_spawn_point_set`, `spawn_change` now
   lists respawn, the counts line (65 hooks, 62 stamped).

No lang key added. Content: the area plays `bmh:bell_sfx` and `bmh:end` and awards the three journey
advancements, all of which exist in `horizon/content`.
