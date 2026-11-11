// Area respawn: the edge between the game and `bmh_logic::respawn`.
// It holds the distance at death, the landing search, the position inside the respawn, the exile
// record, the 200 ticks of immunity, the arrival sounds, the journey signals (server side) and the
// journey advancements. Every decision is in `bmh_logic::respawn`; this file reads the game and
// carries out the answers. Owner: the respawn agent. Log: docs/horizon/log/respawn.md.
//
// The flow of one death, on the server:
// 1. `player_died` marks a real death (it comes after `living_death`, so no area cancelled it).
// 2. `loot_drops` counts the Eternal Items in the drops. With `keep_inventory` it does not fire, and
//    the count of the last `player_tick` stays.
// 3. `server_tick` (end) measures the distance, sends `Departure` and starts the landing search.
// 4. `level_tick_post` of the search's level advances the search while that level is in place.
//    The search sends `Ready` when it ends, and answers a held Respawn request (`bmh:respawn`).
// 5. `player_respawn_position` moves the respawn to the landing and keeps the exile.
// 6. `player_respawned` awards, starts the immunity, queues the sounds, sends `Arrival` and gives the
//    exile to the homecoming area.

use bmh_logic::contract::{Exile, Rule};
use bmh_logic::respawn as logic;
use bmh_logic::wire::{JOURNEY_CHANNEL, JourneySignal, RESPAWN_CHANNEL, RespawnRequest};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// A respawn point of a player: a bed, an anchor or a `/spawnpoint`.
#[derive(Debug, Clone, PartialEq)]
struct Point {
    dimension: String,
    pos: [i32; 3],
    forced: bool,
}

/// What the area keeps of a player between hooks.
#[derive(Debug, Clone, Default)]
struct Player {
    entity: i32,
    game_mode: i32,
    eternal_slots: i32,
    /// None: not known yet (read from the player's file at the next death). Some(None): the world
    /// spawn.
    point: Option<Option<Point>>,
}

/// A landing search and what the respawn compares it with.
struct Running {
    search: logic::Search,
    owner: String,
    started: std::time::Instant,
    /// The level and the block that `player_respawn_position` hands over when the respawn uses this
    /// search: the respawn point as the game keeps it, also when its bed is gone.
    expected: (String, [i32; 3]),
    world_distance: i32,
    ready_sent: bool,
}

/// A landing chunk that stays held for a while after the search found it.
struct Kept {
    owner: String,
    dimension: String,
    chunk: [i32; 2],
    ticks_left: u32,
}

/// The server state of the area. One value for the process (ARCHITECTURE.md, State): the server
/// thread only. Per-player entries go when the player logs out.
struct Server {
    players: BTreeMap<u128, Player>,
    /// Real deaths not handled yet, with the Eternal count of the drops when `loot_drops` gave one.
    deaths: BTreeMap<u128, Option<i32>>,
    searches: BTreeMap<u128, Running>,
    /// Exiles between `player_respawn_position` and `player_respawned` (Java `exiles`).
    exiles: BTreeMap<u128, Exile>,
    /// Ticks of damage immunity by network entity id.
    shields: BTreeMap<i32, i32>,
    /// Awards to run from `server_tick`: player, name, advancement path.
    awards: Vec<(u128, String, &'static str)>,
    /// Sound commands to run from `level_tick_post` of their level: player, level, command line.
    sounds: Vec<(u128, String, String)>,
    kept: Vec<Kept>,
    /// The world spawn that `spawn_change` told, with the world folder it belongs to.
    world_spawn: Option<(PathBuf, [i32; 3])>,
    rng: logic::Rng,
    seeded: bool,
    serial: u64,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    players: BTreeMap::new(),
    deaths: BTreeMap::new(),
    searches: BTreeMap::new(),
    exiles: BTreeMap::new(),
    shields: BTreeMap::new(),
    awards: Vec::new(),
    sounds: Vec::new(),
    kept: Vec::new(),
    world_spawn: None,
    rng: logic::Rng::new(0),
    seeded: false,
    serial: 0,
});

/// The server state. A lock that a panic left behind still holds valid data.
fn server() -> MutexGuard<'static, Server> {
    SERVER.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The next random value in [0, 1). The source mixes in the clock once.
fn random() -> f64 {
    let mut s = server();
    if !s.seeded {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        s.rng.mix(nanos);
        s.seeded = true;
    }
    s.rng.next_f64()
}

/// The world reads of a landing search in one level. The block reads work only while that level is in
/// place (`native_level_open`): `loaded` answers false otherwise, so the search waits.
struct LevelReads<'a, 'g> {
    game: &'a mut crate::game::Game<'g>,
    dimension: String,
    owner: String,
}

impl logic::Blocks for LevelReads<'_, '_> {
    fn block(&mut self, x: i32, y: i32, z: i32) -> logic::BlockInfo {
        let pos = [x, y, z];
        logic::BlockInfo {
            name: self.game.native_block(pos),
            waterlogged: self.game.native_block_property(pos, "waterlogged").as_deref() == Some("true"),
            boxes: self.game.native_block_collision(pos),
        }
    }

    fn surface(&mut self, x: i32, z: i32) -> Option<i32> {
        let heightmap = crate::game::native_world_edit::NativeHeightmap::MotionBlockingNoLeaves;
        self.game
            .native_surface_heights(&self.dimension, &[(x, z)], heightmap)
            .ok()
            .and_then(|heights| heights.first().copied().flatten())
    }
}

impl logic::SearchWorld for LevelReads<'_, '_> {
    fn random(&mut self) -> f64 {
        random()
    }

    fn inside_border(&mut self, x: i32, z: i32) -> bool {
        self.game
            .native_world_border(&self.dimension)
            .is_none_or(|border| logic::inside_border(border, x, z))
    }

    fn hold(&mut self, chunk: [i32; 2]) {
        self.game.native_force_chunk_in(&self.owner, &self.dimension, chunk, true);
    }

    fn release(&mut self, chunk: [i32; 2]) {
        self.game.native_force_chunk_in(&self.owner, &self.dimension, chunk, false);
    }

    fn loaded(&mut self, x: i32, z: i32) -> bool {
        self.game.native_level_open(&self.dimension) && logic::Blocks::surface(self, x, z).is_some()
    }

    fn biome(&mut self, x: i32, y: i32, z: i32) -> Option<String> {
        self.game.native_biome([x, y, z])
    }

    fn shape(&mut self) -> logic::LevelShape {
        let height = self
            .game
            .native_level(&self.dimension)
            .and_then(|level| self.game.native_level_height(level));
        // The vanilla heights when the level does not answer.
        let (min_y, height) = height.unwrap_or(if self.dimension == logic::OVERWORLD {
            (-64, 384)
        } else {
            (0, 256)
        });
        logic::LevelShape::of(&self.dimension, min_y, height)
    }
}

/// Sends a journey signal to a player's client. A player without the mod gets nothing.
fn send(game: &mut crate::game::Game<'_>, player: u128, signal: &JourneySignal) {
    if let Err(error) = game.native_net_send_to_player(player, JOURNEY_CHANNEL, &signal.encode()) {
        log::warn!("[BalancedMcHardcoreMain] Journey signal not sent: {error}");
    }
}

/// What `level.dat` says: hardcore (GAPS.md C5, substitute) and the world spawn.
fn level_dat(game: &crate::game::Game<'_>) -> (bool, Option<[i32; 3]>) {
    let level = game
        .native_world_dir()
        .and_then(|dir| horizon_client::save::Level::read(&dir));
    level.map_or((false, None), |level| (level.hardcore, level.world_spawn))
}

/// The world spawn: the one `spawn_change` told for this world, else the one of `level.dat`.
fn world_spawn(game: &crate::game::Game<'_>, from_file: Option<[i32; 3]>) -> Option<[i32; 3]> {
    let dir = game.native_world_dir();
    let told = server()
        .world_spawn
        .clone()
        .filter(|(world, _)| Some(world) == dir.as_ref())
        .map(|(_, pos)| pos);
    told.or(from_file)
}

/// The respawn point in the player's own file (`respawn` compound, as `Level::read_player` reads it,
/// save.rs:4029-4042). None when the player has none or the file cannot be read.
fn point_from_file(game: &crate::game::Game<'_>, player: u128) -> Option<Point> {
    use horizon_world::nbt::Tag;
    let dir = game.native_world_dir()?;
    let tag = horizon_client::save::read_dat(&horizon_client::lan::playerdata::path(&dir, player))?;
    let respawn = tag.get("respawn")?;
    let pos = match respawn.get("pos") {
        Some(Tag::IntArray(pos)) => match pos.as_slice() {
            [x, y, z] => [*x, *y, *z],
            _ => return None,
        },
        _ => return None,
    };
    let dimension = match respawn.get("dimension") {
        Some(Tag::String(text)) => text.to_string_lossy(),
        _ => logic::OVERWORLD.to_owned(),
    };
    let forced = matches!(respawn.get("forced"), Some(Tag::Byte(b)) if *b != 0);
    Some(Point { dimension, pos, forced })
}

/// True when the block of the point is still a bed or a respawn anchor (RespawnSearch.java:175-178).
/// A block the server cannot read now (its chunk is not loaded) counts as there: the game resolves it
/// at the respawn.
fn still_there(game: &crate::game::Game<'_>, point: &Point) -> bool {
    let block = game
        .native_level(&point.dimension)
        .and_then(|level| game.native_level_block(level, point.pos));
    block.is_none_or(|name| name.ends_with("_bed") || name == "minecraft:respawn_anchor")
}

/// C1, C8, C3: a real death of the last tick. Measures the distance, sends `Departure` and starts the
/// search (JourneySignal.java:28-35, RespawnSearch.java:66-92).
fn handle_death(game: &mut crate::game::Game<'_>, player: u128, drop_count: Option<i32>) {
    let Some(info) = server().players.get(&player).cloned() else {
        return;
    };
    // RespawnPositionHandler.java:215-244: the drops of this death, else the last count of a tick.
    let eternal_slots = drop_count.unwrap_or(info.eternal_slots);
    let config = crate::game::bmh_settings::config(game);
    let difficulty = game
        .native_level(logic::OVERWORLD)
        .and_then(|level| game.native_level_difficulty(level))
        .and_then(logic::difficulty_id)
        .unwrap_or(2);
    let world_distance = logic::distance(config, eternal_slots, difficulty);
    let rule = crate::game::bmh_settings::rule(game, Rule::RandomDeathSpawnPoint);
    let (hardcore, spawn_in_file) = level_dat(game);
    if !logic::is_distant_respawn(rule, info.game_mode, hardcore) {
        return;
    }
    send(game, player, &JourneySignal::Departure);
    // RespawnSearch.java:66-92: guess the level and the centre as the respawn will choose them.
    let Some(spawn) = world_spawn(game, spawn_in_file) else {
        // No search can start: the client must not hold the Respawn click for a search that never ends.
        log::warn!("[BalancedMcHardcoreMain] No world spawn known, using default respawn position");
        send(game, player, &JourneySignal::Ready);
        return;
    };
    let point = match info.point {
        Some(point) => point,
        None => {
            let point = point_from_file(game, player);
            if let Some(known) = server().players.get_mut(&player) {
                known.point = Some(point.clone());
            }
            point
        }
    };
    let expected = point
        .as_ref()
        .map_or((logic::OVERWORLD.to_owned(), spawn), |p| (p.dimension.clone(), p.pos));
    let (dimension, center) = match point {
        Some(p) if p.forced || still_there(game, &p) => (p.dimension, p.pos),
        _ => (logic::OVERWORLD.to_owned(), spawn),
    };
    let level_distance = logic::level_distance(world_distance, logic::coordinate_scale(&dimension));
    let owner = {
        let mut s = server();
        s.serial = s.serial.wrapping_add(1);
        format!("bmh:landing/{player:032x}/{}", s.serial)
    };
    cancel_search(game, player);
    let running = Running {
        search: logic::Search::new(&dimension, center, level_distance),
        owner,
        started: std::time::Instant::now(),
        expected,
        world_distance,
        ready_sent: false,
    };
    server().searches.insert(player, running);
    advance(game, player);
}

/// Drops a player's search and gives its chunk back (RespawnSearch.java:138-145, `cancel`).
fn cancel_search(game: &mut crate::game::Game<'_>, player: u128) {
    let Some(mut running) = server().searches.remove(&player) else {
        return;
    };
    let mut reads = LevelReads {
        game: &mut *game,
        dimension: running.search.dimension.clone(),
        owner: running.owner.clone(),
    };
    running.search.release(&mut reads);
}

/// One step of a player's search, or its failure after the timeout. Sends `Ready` once when the
/// search ends, and keeps the landing chunk held (RespawnSearch.java:95-122).
fn advance(game: &mut crate::game::Game<'_>, player: u128) {
    let Some(mut running) = server().searches.remove(&player) else {
        return;
    };
    let mut reads = LevelReads {
        game: &mut *game,
        dimension: running.search.dimension.clone(),
        owner: running.owner.clone(),
    };
    if !running.search.is_finished() {
        if logic::timed_out(running.started.elapsed().as_millis()) {
            log::warn!(
                "[BalancedMcHardcoreMain] The landing chunk took more than {} s, using the default respawn position",
                logic::TIMEOUT_MS / 1000
            );
            running.search.fail(&mut reads);
        } else {
            logic::Search::advance(&mut running.search, &mut reads);
        }
    }
    let ready = running.search.is_finished() && !running.ready_sent;
    if ready {
        running.ready_sent = true;
        if let Some(feet) = running.search.result() {
            log::info!(
                "[BalancedMcHardcoreMain] Landing spot ready after {} ms: {feet:?}",
                running.started.elapsed().as_millis()
            );
            server().kept.push(Kept {
                owner: running.owner.clone(),
                dimension: running.search.dimension.clone(),
                chunk: logic::chunk_of(feet[0], feet[2]),
                ticks_left: logic::LANDING_TICKET_TICKS,
            });
        }
    }
    server().searches.insert(player, running);
    if ready {
        send(game, player, &JourneySignal::Ready);
    }
}

/// Counts the landing tickets down and lets each one go at the end (RespawnSearch.java:37).
fn tick_kept(game: &mut crate::game::Game<'_>) {
    let ended: Vec<Kept> = {
        let mut s = server();
        for kept in &mut s.kept {
            kept.ticks_left = kept.ticks_left.saturating_sub(1);
        }
        let (ended, left) = std::mem::take(&mut s.kept).into_iter().partition(|k| k.ticks_left == 0);
        s.kept = left;
        ended
    };
    for kept in ended {
        game.native_force_chunk_in(&kept.owner, &kept.dimension, kept.chunk, false);
    }
}

// C2: move the respawn to the distant point that the landing search chose (the vanilla point when
// no landing is ready). RespawnPositionHandler.java:94-160 and F_PlayerListRespawnMixin.java:44-62.
pub(super) fn on_player_respawn_position(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &mut String,
    pos: &mut [f64; 3],
    yaw: &mut f32,
    pitch: &mut f32,
) {
    // The look stays the vanilla one (Java changes the position only).
    let _ = (yaw, pitch);
    let (running, game_mode) = {
        let mut s = server();
        let running = s.searches.remove(&player);
        (running, s.players.get(&player).map_or(-1, |p| p.game_mode))
    };
    let Some(mut running) = running else {
        return;
    };
    // RespawnSearch.java:148-157, `take`: a search that did not end gives its chunk back.
    if !running.search.is_finished() {
        let mut reads = LevelReads {
            game: &mut *game,
            dimension: running.search.dimension.clone(),
            owner: running.owner.clone(),
        };
        running.search.release(&mut reads);
        log::warn!("[BalancedMcHardcoreMain] The landing search of {name} did not end, using default respawn position");
    }
    let rule = crate::game::bmh_settings::rule(game, Rule::RandomDeathSpawnPoint);
    // The game hands over the respawn point as it keeps it. When that is the point the search was
    // made for, the search's own level and centre apply (the world spawn for a bed that is gone).
    let handed = (dimension.clone(), logic::block_of(*pos));
    let (level, center) = if handed == running.expected {
        (running.search.dimension.clone(), running.search.center)
    } else {
        handed
    };
    let Some(feet) = logic::landing(rule, game_mode, Some(&running.search), &level, center) else {
        log::info!("[BalancedMcHardcoreMain] No distant respawn for {name}, using default respawn position");
        return;
    };
    log::info!("[BalancedMcHardcoreMain] new respawn pos and bed : {feet:?}, {center:?}");
    *pos = logic::bottom_center(feet);
    dimension.clone_from(&running.search.dimension);
    let exile = Exile {
        dimension: running.search.dimension.clone(),
        center: running.search.center,
        distance: running.world_distance,
    };
    server().exiles.insert(player, exile);
}

// The journey signal needs the Eternal count of `loot_drops`, which fires after this hook. So
// `player_died` marks the death (it also comes only when no area cancelled it) and `server_tick`
// sends the signal. This hook has nothing to do.
pub(super) fn on_living_death(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    kind: &str,
    damage_type: &str,
    attacker: u32,
    cancel: &mut bool,
) {
    let _ = (game, entity, kind, damage_type, attacker, cancel);
}

// C1: count the Eternal Items in the dying player's drops (one per slot, as the Java loop counts).
pub(super) fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    let _ = game;
    // A player's own death: the entity is the player and no loot table rolled.
    let Some((_, "minecraft:player")) = context.entity else {
        return;
    };
    let Some(player) = context.player.filter(|_| context.table.is_empty()) else {
        return;
    };
    let count = drops
        .iter()
        .filter(|stack| stack.count > 0 && bmh_logic::ids::is_eternal(&stack.item))
        .count();
    if let Some(pending) = server().deaths.get_mut(&player) {
        *pending = Some(i32::try_from(count).unwrap_or(i32::MAX));
    }
}

// C1, C8: mark a real death. `living_death` came first and no area cancelled it.
pub(super) fn on_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    let _ = (game, cause);
    server().deaths.insert(player, None);
}

// C1: cache the Eternal Item count, the game mode and the entity id. C6: count down the immunity.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    // `Inventory.getContainerSize` slots: 36, the armour and the off hand (RespawnPositionHandler.java:228).
    let slots = (0..36).chain(100..=103).chain([-106]);
    let mut count = 0;
    for slot in slots {
        if game
            .native_carried_stack(slot)
            .is_some_and(|stack| stack.count > 0 && bmh_logic::ids::is_eternal(&stack.item))
        {
            count += 1;
        }
    }
    let mut s = server();
    let known = s.players.entry(me.uuid).or_default();
    known.entity = me.entity_id;
    known.game_mode = me.game_mode;
    known.eternal_slots = count;
    if let Some(ticks) = s.shields.get_mut(&me.entity_id) {
        *ticks = logic::count_down(*ticks);
    }
    s.shields.retain(|_, ticks| *ticks > 0);
}

// C2, C6, C7, C8: read the exile record, start the immunity, queue the arrival sounds, send the
// arrival signal. C9, C10: give back the exile of this respawn, once (Java `takeExile`). Give None
// when the respawn was not a distant one or the world is hardcore. The hook helper hands the exile
// to `bmh_homecoming::on_exile`. RespawnEffectHandler.java:27-65.
pub(super) fn on_player_respawned(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) -> Option<Exile> {
    let exile = server().exiles.remove(&player)?;
    // RespawnEffectHandler.java:38: no journey in a hardcore world.
    if level_dat(game).0 {
        return None;
    }
    let deaths = game
        .native_player_stat(player, "minecraft:custom", "minecraft:deaths")
        .unwrap_or(0);
    let me = game.native_acting_player();
    let text = crate::game::bmh_settings::rule(game, Rule::DisplayTextOnRespawn);
    {
        let mut s = server();
        for path in logic::journey_awards(deaths, exile.distance) {
            s.awards.push((player, name.to_owned(), path));
        }
        if let Some(me) = &me {
            // RespawnEffectHandler.java:59.
            s.shields.insert(me.entity_id, logic::JOURNEY_INVULNERABLE_TICKS);
            // RespawnEffectHandler.java:68-93: at the player's block, for the others nearby.
            let block = logic::block_of(me.position);
            for line in logic::sound_commands(name, block).into_iter().flatten() {
                s.sounds.push((player, me.dimension.clone(), line));
            }
        }
    }
    // RespawnEffectHandler.java:63-64, JourneySignal.java:37-46.
    let arrival = JourneySignal::Arrival { name: text.then(|| name.to_owned()) };
    send(game, player, &arrival);
    Some(exile)
}

// State: remove this player from the area's per-player statics, and drop the search (Java
// `F_ServerGamePacketListenerMixin.bmh$dropSearch`).
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = name;
    cancel_search(game, player);
    let mut s = server();
    if let Some(known) = s.players.remove(&player) {
        s.shields.remove(&known.entity);
    }
    s.deaths.remove(&player);
    s.exiles.remove(&player);
    s.awards.retain(|(who, _, _)| *who != player);
    s.sounds.retain(|(who, _, _)| *who != player);
}

// C1, C3, C8: the deaths of the tick, the search timeouts, the landing tickets. C9: the journey
// advancements (`run_command` works only where no player is swapped in).
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = context;
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    let deaths = std::mem::take(&mut server().deaths);
    for (player, drop_count) in deaths {
        handle_death(game, player, drop_count);
    }
    // RespawnSearch.java:103-112: the timeout runs whether the level ticks or not.
    let late: Vec<u128> = server()
        .searches
        .iter()
        .filter(|(_, r)| !r.search.is_finished() && logic::timed_out(r.started.elapsed().as_millis()))
        .map(|(player, _)| *player)
        .collect();
    for player in late {
        advance(game, player);
    }
    tick_kept(game);
    let awards = std::mem::take(&mut server().awards);
    for (_, name, path) in awards {
        if !crate::game::bmh::award(game, &name, path) {
            log::warn!("[BalancedMcHardcoreMain] Advancement bmh:{path} not awarded to {name}");
        }
    }
}

// C3: advance the searches of this level, now in place. C7: the arrival sounds of this level
// (`/playsound` reaches only the players of the level in place).
pub(super) fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    let lines: Vec<String> = {
        let mut s = server();
        let (here, other): (Vec<_>, Vec<_>) =
            std::mem::take(&mut s.sounds).into_iter().partition(|(_, level, _)| level == dimension);
        s.sounds = other;
        here.into_iter().map(|(_, _, line)| line).collect()
    };
    for line in lines {
        if !game.native_run_command(&line) {
            log::warn!("[BalancedMcHardcoreMain] Sound command failed: {line}");
        }
    }
    let players: Vec<u128> = server()
        .searches
        .iter()
        .filter(|(_, r)| r.search.dimension == dimension && !r.search.is_finished())
        .map(|(player, _)| *player)
        .collect();
    for player in players {
        advance(game, player);
    }
}

// C6: cancel the damage during the 200 ticks of immunity after a distant respawn. No `Game` here:
// the area reads only the immunity entries that gated hooks wrote, so a mod that is off does nothing.
pub(super) fn on_living_incoming_damage(
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    direct: i32,
    amount: &mut f32,
    canceled: &mut bool,
) {
    let _ = (causing, direct, amount);
    if kind != "minecraft:player" {
        return;
    }
    let ticks = server().shields.get(&entity).copied().unwrap_or(0);
    if logic::blocks_damage(ticks, damage_type) {
        *canceled = true;
    }
}

// C1: the distance reads the difficulty at death with `level_difficulty`, so a change needs no copy.
pub(super) fn on_difficulty_changed(game: &mut crate::game::Game<'_>, from: &str, to: &str) {
    let _ = (game, from, to);
}

// C4, D7: check a respawn request that the client held, on `bmh:respawn`. The server answers only a
// dead player that has a search of its own, and only once that search ended (RespawnSearch.java:124-135).
pub(super) fn on_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if channel != RESPAWN_CHANNEL || RespawnRequest::decode(data) != Some(RespawnRequest::Held) {
        return;
    }
    let Some(finished) = server().searches.get(&player).map(|r| r.search.is_finished()) else {
        return;
    };
    let alive = game
        .native_server_players()
        .iter()
        .any(|p| p.uuid == player && p.health > 0.0);
    if finished && !alive {
        send(game, player, &JourneySignal::Ready);
    }
}

// C2: the respawn point of a player changes (a bed, an anchor, `/spawnpoint`, or a reset at death).
pub(super) fn on_player_spawn_point_set(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    pos: [i32; 3],
    cause: &str,
    cancel: &mut bool,
) {
    let _ = (game, name);
    if *cancel {
        return;
    }
    // Vanilla `/spawnpoint` sets a forced point; a bed and an anchor do not.
    let point = (cause != "death_reset").then(|| Point {
        dimension: dimension.to_owned(),
        pos,
        forced: cause == "command",
    });
    server().players.entry(player).or_default().point = Some(point);
}

// C2: `/setworldspawn` moved the world spawn, the centre of an exile with no bed.
pub(super) fn on_spawn_change(game: &mut crate::game::Game<'_>, pos: [i32; 3], angle: [f32; 2]) {
    let _ = angle;
    if let Some(world) = game.native_world_dir() {
        server().world_spawn = Some((world, pos));
    }
}
