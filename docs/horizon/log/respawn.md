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

## Fix round 1 (2026-11-17)

Inputs: `docs/horizon/review/respawn-parity.json` (parity-1 to parity-5), `docs/horizon/review/respawn-semantics.json`
(semantics-1 to semantics-7), `docs/horizon/log/first-run.md` (game-b0, game-b4). This round has game rights.

### Step log

1. Worktree `worktree-wf_8bd388a6-43e-1` was not on 8a963a8: `git reset --hard 8a963a8f51c6`. Guard files present.
   Read the brief, ARCHITECTURE.md, HOOKS.md, this log, both review files, first-run.md, content.md, TESTING.md,
   test-slice.sh, `bmh_respawn.rs` and `logic/src/respawn.rs`.
2. Read HN for each finding (guest_respawn lan_actions.rs:3892-4080, find_respawn_and_use respawn_anchor.rs:477-509,
   resolve_server_respawn server_travel.rs:639-672, spawn_player game.rs:7753-7790, Level::read save.rs:3816-3850 and the
   level.dat writer save.rs:4402, guest_die lan_actions.rs:4292-4420, flush_native_net native_net.rs:120-150, the
   helpers player_compound, player_advancements, set_block_property, play_sound_to, tell). Facts found:
   - The level.dat of a new world is written with `world_spawn.unwrap_or([0, 64, 0])` (save.rs:4402) before the spawn
     is found. `spawn_player` then sets the world spawn in memory only (game.rs:7787) and fires no `spawn_change`.
     So level.dat holds the placeholder [0, 64, 0] until the first save. No helper reads the world spawn.
   - `guest_respawn` hands the point: the guest's own `spawn`, else `level.world_spawn` + 0.5 (the real one). In a
     single-player world an Overworld point is handed as `pos + (0.5, 0.6, 0.5)`; a point in another level is not
     handed at all (`elsewhere` is only for a dedicated server): the world spawn is handed instead.
   - `native_player_compound(uuid)` gives the live `respawn` compound (dimension, pos, forced) of any player outside a
     player's turn, the dead included. It replaces the file read and the copy of `player_spawn_point_set`.
   - Mod messages leave in the next `tick_lan` flush; `PlayerCombatKill` leaves at once inside `guest_die`.
3. Reproduced the two game defects on the base code: `test-slice.sh --only b --keep`. b0 FAIL: log `Landing spot ready
   after 1847 ms: [18981, 102, -13568]` (a circle round x 0 z 0) then `No distant respawn for Player54828`. b0s PASS after
   `save`: `new respawn pos and bed : [..], [0, 126, 0]`. b4 FAIL: two `Sound command failed: playsound ...` per
   respawn and `Advancement bmh:journey/exile not awarded` on the second respawn. So game-b0's cause holds: the
   search used level.dat's placeholder spawn (y 64), the game handed the real one (y 126). Scratch folder removed.
4. Plan (one flow): at death read the live point (`player_compound`), hold its chunk and resolve it as vanilla does
   (anchor charge and Nether, bed and Overworld, forced point free), then search round the point or the world spawn;
   at the respawn accept the handed point when it is the point, or when it is the world spawn of a world-spawn
   search (learn it then); a point changed after the death gives the vanilla point; after the respawn use the anchor
   charge or tell `block.minecraft.spawn.not_valid`. Timeout in server ticks. A dead player seen with no search
   (relog, reopened world) gets one. Shields cleared at death. Sounds only when someone hears them; awards only
   when not done.
5. Wrote the logic (`TIMEOUT_TICKS`, `DEAD_WAIT_TICKS`, `RespawnPoint`, `PointUse`, `point_use`, `state_name`,
   `state_property`, `respawn_space`, `anchor_after_use`, `Fit`, `respawn_fit`, `someone_hears`, `count_eternal`,
   `Shields`, `Search::landing_chunk`) with 13 new tests, and rewrote `bmh_respawn.rs` on it. test-logic: 117 passed.
   check.sh: ok on the first compile.
6. Game, `--only b` on the new code: b0, b0s, b1, b2, b3, b4 all PASS. Log: `new respawn pos and bed : [-14350, 80,
   -18397], [0, 126, 0]` for the fresh world (the exile centre is the game's world spawn), no WARN line of the mod.
7. test-slice.sh: launch in a function (`launch_game`), b4 now fails on any WARN line of the mod, new c3 (a death
   inside the immunity ends it), h (nine items, one screenshot), i (a /spawnpoint blocked by stone), j (dead when the
   game quits, relaunch on the same world). `bash -n` OK.
8. Game, `--only h,i,j`: h1, i1, i2 PASS; j1 FAIL. The log of the second launch: `Player28328 joined the game`, then
   `Player28328 died`, then `Respawn point ... reads minecraft:air: Forced` and `Landing spot ready`. So Horizon kills a
   player who logs in dead once more, and `player_died` fires again. The old code returned at once there (its player
   cache comes from `player_tick`, which a dead player never runs): that was the real cause of semantics-1. My scan
   for "dead players with no search" was never needed: removed it (and `DEAD_WAIT_TICKS`). The death path now takes
   the game mode from `server_players`, keeps a death whose player is not listed yet for the next tick, and uses the
   distance stored at the first death (Java's `distancesFromBed` survives a relog). j1 now checks the death screen and
   a new `Landing spot ready`. A launch without `--wait` gives no pid: the script finds the game by `--game-dir`.
9. Gates: test-logic 117 passed; check.sh `"check": "ok"`.
10. Full run with the build: 24 passed, 0 failed, 0 skipped, 3 min 23 s, no game left (`pgrep -fl horizon-client`
    empty). Mod log lines of that run, in order: b0 `new respawn pos and bed : [11002, 106, -20575], [0, 126, 0]`; a
    death 23 333 blocks from the point `Respawn point ... [-1070, 63, 23308] ... reads minecraft:short_grass: Forced`;
    i `reads minecraft:stone: Missing, centre [0, 126, 0]` then `new respawn pos and bed : [14224, 68, 18495], [0, 126,
    0]`; j after the relaunch `joined the game`, `died`, `Landing spot ready after 1436 ms: [3422, 137, -2186]`, `new
    respawn pos and bed : [3422, 137, -2186], [14224, 68, 18495]` (23 332 blocks). No WARN line of the mod in the whole
    run. The items screenshot (`~/Library/Caches/bmh-horizon/scratch/respawn-fix1-shots/h-items-inventory.png`, looked
    at): the inventory open, the nine items in the hotbar with their textures (sword, axe, pickaxe, bundle, amulet,
    Totem of Eternity, two dark totems, Echo Heart), the give lines in the chat and advancement toasts.
11. Docs: the two review files (status and note on each finding, top status `closed`), GAPS.md 3.3 (two lines),
    TESTING.md (checks b4, c3, h1, i1, i2, j1, j2, hand checks respawn 5, 5b, 5c, 5d, the run record).

### The fourteen items

| Item | Status | Evidence or reason |
|---|---|---|
| parity-1 | fixed (three parts listed below) | The point is resolved at death as vanilla does (`logic::point_use`); the anchor charge is used after the arrival; `block.minecraft.spawn.not_valid` is told. Unit test `the_point_resolves_as_vanilla_does`; game i1 (`reads minecraft:stone: Missing, centre [0, 126, 0]`), i2 (the chat line). |
| parity-2 | fixed | The point's chunk is held at death and read in `level_tick_post` of its level. Game: a death 23 333 blocks from the point gave `reads minecraft:short_grass: Forced` before the search. |
| parity-3 | fixed | `player_died` clears the shield. Unit test `the_shield_lasts_200_ticks_and_a_death_ends_it`; game c3 `health 20.0 then 14.0` at tick 80 after a vanilla respawn. |
| parity-4 | fixed | The world spawn is read only for no point or a point that does not work. Shown in the game for the Missing arm (i1). |
| parity-5 | fixed | New tests: `the_biome_test_reads_at_sea_level`, `chunks_of_negative_columns_round_down`, `a_fractional_border_keeps_min_inclusive_max_exclusive`, `the_handed_point_must_fit_the_search` (the matching is now `logic::respawn_fit`). |
| semantics-1 | fixed | Real cause: Horizon kills a player who logs in dead once more, and the old death path needed a cache that only `player_tick` fills. Game j1, j2 (exile 23 331.4 blocks from the point after quit and relaunch). |
| semantics-2 | fixed (hand check) | As parity-1: an anchor works only charged or forced, in the Nether; one charge goes after a non-forced anchor exile. Unit tests `the_point_resolves_as_vanilla_does`, `an_anchor_gives_one_charge_a_respawn`. Single player never hands a Nether point, so only a dedicated server can show it: TESTING.md respawn 5c. |
| semantics-3 | deferred | The evidence holds: `PlayerCombatKill` leaves at once (lan_actions.rs:4408), mod messages at the next flush (native_net.rs:120). No tier N or F hook sends before it. GAPS.md 3.3 (C8). |
| semantics-4 | fixed | The live point comes from `native_player_compound` at each death; nothing is cached. Game: every death with a point logs `Respawn point RespawnPoint { ... }` read that way. |
| semantics-5 | fixed | `logic::someone_hears` (16 blocks) before a sound, `native_player_advancements` before an award; a real failure still warns. Unit test `a_sound_needs_a_player_within_16_blocks`; game b4. |
| semantics-6 | fixed (not run in the game) | The landing ticket stays until the respawn takes the search, then 6000 ticks; logout and a new death release it. Unit test `the_search_draws_holds_and_finds` (`landing_chunk`). The 5-minute wait was not run. |
| semantics-7 | fixed (not run in the game) | The timeout counts server ticks that ran (`TIMEOUT_TICKS` 2400). Unit test `the_timeout_is_120_seconds_of_server_ticks`. A long pause was not run. |
| game-b0 | fixed | Cause confirmed in HN (save.rs:4402 placeholder, game.rs:7787) and in the game (step 3). A world-spawn search now fits the world spawn that the game hands (`Fit::WorldSpawn`), which is then learned. Game b0 PASS: `new respawn pos and bed : [11002, 106, -20575], [0, 126, 0]`. |
| game-b4 | fixed | Game b4 PASS: no WARN line of the mod after 3 distant respawns alone in the world. |

### Differences that remain (fix round 1)

1. No stand-up search round a bed or an anchor: an obstructed bed or anchor counts as one that works and centres the
   exile (Java: missing, world spawn). GAPS.md 3.3 C2.
2. A point that does not work is not forgotten: no helper clears a respawn point. Each later death resolves it again,
   so the exile centre is still the world spawn; Horizon keeps the point in the player's data. GAPS.md 3.3 C2.
3. In a world never saved, a search with no point is centred on level.dat's placeholder world spawn ([0, 64, 0]), as
   no helper reads the world spawn. The respawn takes the game's own world spawn as the exile centre and learns it for
   the next deaths. When the real spawn is not at x 0, z 0, that first circle is off by the distance between the two.
   GAPS.md 3.3 C2.
4. `Departure` reaches the client at least one flush after the death screen packet. GAPS.md 3.3 C8.
5. A point or world spawn that changes while the player is dead gives the vanilla respawn (Java searched again).
6. A forced point's room test treats any block with a collision box as solid (Java: `isSolid`), so a carpet or a
   similar thin block counts as no room.
7. The timeout counts server ticks, not wall time: on a lagging server it lasts longer than 120 s.
8. The landing ticket counts its 6000 ticks from the respawn, not from the end of the search.
9. In single player, a working Nether anchor gives the vanilla respawn: Horizon hands the world spawn, not the anchor,
   and places a single-player guest in the Overworld whatever the hook says.

### Changes outside my files (fix round 1)

1. `docs/horizon/GAPS.md`, section 3.3: two lines (C2 and C8), for the deferred differences above.
2. Allowed by the prompt: `horizon/tools/test-slice.sh`, `docs/horizon/TESTING.md`, the two review files.
No hook, manifest, `bmh.rs`, `wire.rs`, `contract.rs` or HOOKS.md change: the round uses hooks already wired.
