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
// 3. `server_tick` (end) measures the distance, sends `Departure` and reads the player's respawn point
//    as the game keeps it now. A player who logs in dead dies once more in Horizon, so a relog or a
//    reopened world starts a new search here, with the distance of the first death.
// 4. With a point, `level_tick_post` of the point's level reads its block once the chunk is loaded and
//    resolves it as vanilla does (anchor charge, bed, forced point). The centre is the point, or the
//    world spawn for a point that does not work.
// 5. `level_tick_post` of the search's level advances the search while that level is in place.
//    The search sends `Ready` when it ends, and answers a held Respawn request (`bmh:respawn`).
// 6. `player_respawn_position` moves the respawn to the landing and keeps the exile.
// 7. `player_respawned` awards, starts the immunity, queues the sounds, uses the anchor charge or tells
//    that the point is gone, sends `Arrival` and gives the exile to the homecoming area.
// An early respawn (the search has not ended: `immediate_respawn`, or a press the client did not hold)
// keeps the vanilla point in step 6 and the search goes on. When it ends, `server_tick` moves the player
// to the landing with the host's teleport and runs step 7 there (`finish_early`).

use bmh_logic::contract::{Exile, Rule};
use bmh_logic::respawn as logic;
use bmh_logic::wire::{JOURNEY_CHANNEL, JourneySignal, RESPAWN_CHANNEL, RespawnRequest};
use horizon_world::nbt::Tag;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// What the area keeps of a living player between hooks.
#[derive(Debug, Clone, Default)]
struct Player {
    entity: i32,
    eternal_slots: i32,
}

/// Where a search is: the block of the point is read first, then the landing is looked for.
enum Phase {
    /// The chunk of the player's point is held until its block can be read.
    Resolving(logic::RespawnPoint),
    /// The landing search, round the point or the world spawn.
    Searching(logic::Search),
}

/// One dead player's search and what the respawn compares it with.
struct Running {
    phase: Phase,
    owner: String,
    /// For the log line only. The timeout counts `ticks`.
    started: std::time::Instant,
    /// Server ticks since the death.
    ticks: u32,
    world_distance: i32,
    game_mode: i32,
    /// The player's point at death as the game keeps it. None: the world spawn.
    point: Option<logic::RespawnPoint>,
    /// What vanilla's resolution gives for the point, once read.
    point_use: Option<logic::PointUse>,
    /// The point or the world spawn changed after the death: the search does not fit any more.
    stale: bool,
    ready_sent: bool,
    /// The player respawned before the search ended (review journey-semantics major 1, script n5).
    early: Option<Early>,
}

/// A respawn that came before its search ended: the `immediate_respawn` rule, or a Respawn press that
/// the client did not hold. Horizon has no hook that can hold `ClientCommand` (GAPS.md C4), so the player
/// respawns at the vanilla point, and `server_tick` moves them to the landing when the search ends.
struct Early {
    name: String,
    /// The respawn point that the game handed to `player_respawn_position`: dimension and block.
    handed: (String, [i32; 3]),
}

/// A distant respawn that the search allows: the landing, its level, the exile record and the point
/// effect for the arrival.
struct Placed {
    feet: [i32; 3],
    dimension: String,
    exile: Exile,
    effect: PointEffect,
}

/// What the arrival needs of the player who arrives: entity id, feet position and level.
struct Arrived {
    entity: i32,
    position: [f64; 3],
    dimension: String,
}

impl Running {
    fn search(&self) -> Option<&logic::Search> {
        match &self.phase {
            Phase::Searching(search) => Some(search),
            Phase::Resolving(_) => None,
        }
    }

    fn is_finished(&self) -> bool {
        self.search().is_some_and(logic::Search::is_finished)
    }

    /// True when the centre is the world spawn: no point, or a point that does not work.
    fn centre_is_world_spawn(&self) -> bool {
        self.point.is_none() || self.point_use == Some(logic::PointUse::Missing)
    }
}

/// What the arrival does after a distant respawn, besides the journey (vanilla's resolution of the
/// point, which Horizon skips once the position is moved).
#[derive(Debug, Clone)]
enum PointEffect {
    None,
    /// A non-forced anchor gives one charge (vanilla `findRespawnAndUseSpawnBlock`).
    UseCharge { dimension: String, pos: [i32; 3] },
    /// The point does not work: vanilla tells the player (`NO_RESPAWN_BLOCK_AVAILABLE`).
    NotValid,
}

/// A landing chunk that stays held for a while after the respawn took the search.
struct Kept {
    owner: String,
    dimension: String,
    chunk: [i32; 2],
    ticks_left: u32,
}

/// The server state of the area. One value for the process (ARCHITECTURE.md, State): the server
/// thread only. Per-player entries go when the player logs out, except `distances` (below).
struct Server {
    players: BTreeMap<u128, Player>,
    /// Real deaths not handled yet, with the Eternal count of the drops when `loot_drops` gave one.
    deaths: BTreeMap<u128, Option<i32>>,
    /// The distance of each death until its respawn. It stays when the player logs out, as Java's
    /// `distancesFromBed` does, so a relog keeps the distance of the death.
    distances: BTreeMap<u128, i32>,
    searches: BTreeMap<u128, Running>,
    /// Exiles between `player_respawn_position` and `player_respawned` (Java `exiles`).
    exiles: BTreeMap<u128, (Exile, PointEffect)>,
    shields: logic::Shields,
    /// Awards to run from `server_tick`: player, name, advancement path.
    awards: Vec<(u128, String, &'static str)>,
    /// Sound commands to run from `level_tick_post` of their level: player, level, block, line.
    sounds: Vec<(u128, String, [i32; 3], String)>,
    /// Anchor charges to use from `level_tick_post` of their level: player, level, anchor.
    charges: Vec<(u128, String, [i32; 3])>,
    kept: Vec<Kept>,
    /// The world spawn that `spawn_change` told or a respawn showed, with the world folder it
    /// belongs to.
    world_spawn: Option<(PathBuf, [i32; 3])>,
    rng: logic::Rng,
    seeded: bool,
    serial: u64,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    players: BTreeMap::new(),
    deaths: BTreeMap::new(),
    distances: BTreeMap::new(),
    searches: BTreeMap::new(),
    exiles: BTreeMap::new(),
    shields: logic::Shields::new(),
    awards: Vec::new(),
    sounds: Vec::new(),
    charges: Vec::new(),
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

/// One block of the level in place, as the search and the point test need it.
fn block_info(game: &crate::game::Game<'_>, pos: [i32; 3]) -> logic::BlockInfo {
    logic::BlockInfo {
        name: game.native_block(pos),
        waterlogged: game.native_block_property(pos, "waterlogged").as_deref() == Some("true"),
        boxes: game.native_block_collision(pos),
    }
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
        block_info(self.game, [x, y, z])
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

/// The world spawn: the one `spawn_change` told or a respawn showed for this world, else the one of
/// `level.dat`. A world never saved has a placeholder there (save.rs:4402), which the first respawn
/// of a player with no point replaces (`logic::Fit::WorldSpawn`).
fn world_spawn(game: &crate::game::Game<'_>) -> Option<[i32; 3]> {
    let dir = game.native_world_dir();
    let known = server()
        .world_spawn
        .clone()
        .filter(|(world, _)| Some(world) == dir.as_ref())
        .map(|(_, pos)| pos);
    known.or_else(|| level_dat(game).1)
}

/// The `respawn` compound of a player's data (save.rs:4029-4042 reads the same keys).
fn point_of(tag: &Tag) -> Option<logic::RespawnPoint> {
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
    Some(logic::RespawnPoint { dimension, pos, forced })
}

/// The player's data as the server would save it now (`native_player_compound`), else the player's
/// file. Nothing is kept between deaths: each death reads the point again (review semantics-4).
fn player_data(game: &crate::game::Game<'_>, player: u128) -> Option<Tag> {
    game.native_player_compound(player).or_else(|| {
        let dir = game.native_world_dir()?;
        horizon_client::save::read_dat(&horizon_client::lan::playerdata::path(&dir, player))
    })
}

/// C1, C8, C3: a real death. Measures the distance, sends `Departure`, reads the respawn point and
/// starts its resolution or the search (JourneySignal.java:28-35, RespawnSearch.java:66-92). `stored`
/// is the distance of a death that a relog interrupted.
fn handle_death(
    game: &mut crate::game::Game<'_>,
    player: u128,
    eternal_slots: i32,
    game_mode: i32,
    stored: Option<i32>,
) {
    // RespawnPositionHandler.java:104-106 and :215-244.
    let world_distance = stored.unwrap_or_else(|| {
        let config = crate::game::bmh_settings::config(game);
        let difficulty = game
            .native_level(logic::OVERWORLD)
            .and_then(|level| game.native_level_difficulty(level))
            .and_then(logic::difficulty_id)
            .unwrap_or(2);
        logic::distance(config, eternal_slots, difficulty)
    });
    let rule = crate::game::bmh_settings::rule(game, Rule::RandomDeathSpawnPoint);
    let hardcore = level_dat(game).0;
    if !logic::is_distant_respawn(rule, game_mode, hardcore) {
        return;
    }
    server().distances.insert(player, world_distance);
    send(game, player, &JourneySignal::Departure);
    cancel_search(game, player);
    let point = player_data(game, player).as_ref().and_then(point_of);
    let owner = {
        let mut s = server();
        s.serial = s.serial.wrapping_add(1);
        format!("bmh:landing/{player:032x}/{}", s.serial)
    };
    let phase = match &point {
        // RespawnSearch.java:79: the block decides, so its chunk must be there (review parity-2).
        Some(p) => {
            let [x, _, z] = p.pos;
            game.native_force_chunk_in(&owner, &p.dimension, logic::chunk_of(x, z), true);
            Phase::Resolving(p.clone())
        }
        // RespawnSearch.java:81-82: no point, the world spawn of the Overworld.
        None => {
            let Some(spawn) = world_spawn(game) else {
                no_world_spawn(game, player);
                return;
            };
            let distance = logic::level_distance(world_distance, logic::coordinate_scale(logic::OVERWORLD));
            Phase::Searching(logic::Search::new(logic::OVERWORLD, spawn, distance))
        }
    };
    let running = Running {
        phase,
        owner,
        started: std::time::Instant::now(),
        ticks: 0,
        world_distance,
        game_mode,
        point,
        point_use: None,
        stale: false,
        ready_sent: false,
        early: None,
    };
    server().searches.insert(player, running);
    advance(game, player);
}

/// No search can start: the client must not hold the Respawn click for a search that never ends.
fn no_world_spawn(game: &mut crate::game::Game<'_>, player: u128) {
    log::warn!("[BalancedMcHardcoreMain] No world spawn known, using default respawn position");
    send(game, player, &JourneySignal::Ready);
}

/// Lets go of every chunk a search holds: the point, the candidate or the landing.
fn release_all(game: &mut crate::game::Game<'_>, running: &mut Running) {
    match &mut running.phase {
        Phase::Resolving(point) => {
            let [x, _, z] = point.pos;
            game.native_force_chunk_in(&running.owner, &point.dimension, logic::chunk_of(x, z), false);
        }
        Phase::Searching(search) => {
            let landing = search.landing_chunk();
            let dimension = search.dimension.clone();
            let mut reads = LevelReads { game: &mut *game, dimension, owner: running.owner.clone() };
            search.release(&mut reads);
            if let Some(chunk) = landing {
                game.native_force_chunk_in(&running.owner, &search.dimension, chunk, false);
            }
        }
    }
}

/// Drops a player's search and gives its chunks back (RespawnSearch.java:138-145, `cancel`).
fn cancel_search(game: &mut crate::game::Game<'_>, player: u128) {
    let Some(mut running) = server().searches.remove(&player) else {
        return;
    };
    release_all(game, &mut running);
}

/// The block of the point is readable in its level, now in place: resolve it as vanilla does and
/// start the search round the point or the world spawn (review parity-1, parity-2).
fn resolve(game: &mut crate::game::Game<'_>, player: u128) {
    let Some(mut running) = server().searches.remove(&player) else {
        return;
    };
    let Phase::Resolving(point) = &running.phase else {
        server().searches.insert(player, running);
        return;
    };
    let point = point.clone();
    let level = game.native_level(&point.dimension);
    let readable = game.native_level_open(&point.dimension)
        && level.is_some_and(|level| game.native_level_is_loaded(level, point.pos));
    let state = level.and_then(|level| game.native_level_block_state(level, point.pos));
    let Some(state) = state.filter(|_| readable) else {
        server().searches.insert(player, running);
        return;
    };
    let [x, y, z] = point.pos;
    let free_at = logic::respawn_space(&block_info(game, point.pos));
    let free_above = logic::respawn_space(&block_info(game, [x, y.saturating_add(1), z]));
    let point_use = logic::point_use(&point, &state, free_at, free_above);
    game.native_force_chunk_in(&running.owner, &point.dimension, logic::chunk_of(x, z), false);
    running.point_use = Some(point_use);
    let (dimension, centre) = if point_use == logic::PointUse::Missing {
        // RespawnSearch.java:81-82, RespawnPositionHandler.java:120-127.
        let Some(spawn) = world_spawn(game) else {
            no_world_spawn(game, player);
            return;
        };
        (logic::OVERWORLD.to_owned(), spawn)
    } else {
        (point.dimension.clone(), point.pos)
    };
    log::info!("[BalancedMcHardcoreMain] Respawn point {point:?} reads {state}: {point_use:?}, centre {centre:?}");
    let distance = logic::level_distance(running.world_distance, logic::coordinate_scale(&dimension));
    running.phase = Phase::Searching(logic::Search::new(&dimension, centre, distance));
    server().searches.insert(player, running);
    advance(game, player);
}

/// One step of a player's search. Sends `Ready` once when the search ends (RespawnSearch.java:95-122).
fn advance(game: &mut crate::game::Game<'_>, player: u128) {
    let Some(mut running) = server().searches.remove(&player) else {
        return;
    };
    let owner = running.owner.clone();
    let ticks = running.ticks;
    let elapsed = running.started.elapsed().as_millis();
    let mut ready = false;
    if let Phase::Searching(search) = &mut running.phase {
        if !search.is_finished() {
            let mut reads = LevelReads { game: &mut *game, dimension: search.dimension.clone(), owner };
            if logic::timed_out(ticks) {
                log::warn!(
                    "[BalancedMcHardcoreMain] The landing chunk took more than {} s, using the default respawn position",
                    logic::TIMEOUT_MS / 1000
                );
                search.fail(&mut reads);
            } else {
                logic::Search::advance(search, &mut reads);
            }
        }
        if search.is_finished() && !running.ready_sent {
            running.ready_sent = true;
            ready = true;
            if let Some(feet) = search.result() {
                log::info!("[BalancedMcHardcoreMain] Landing spot ready after {elapsed} ms: {feet:?}");
            }
        }
    } else if logic::timed_out(ticks) && !running.ready_sent {
        log::warn!(
            "[BalancedMcHardcoreMain] The respawn point's chunk took more than {} s, using the default respawn position",
            logic::TIMEOUT_MS / 1000
        );
        release_all(game, &mut running);
        let mut failed = logic::Search::new(logic::OVERWORLD, [0, 0, 0], 0);
        let dimension = logic::OVERWORLD.to_owned();
        failed.fail(&mut LevelReads { game: &mut *game, dimension, owner: running.owner.clone() });
        running.phase = Phase::Searching(failed);
        running.ready_sent = true;
        ready = true;
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
// no landing fits). RespawnPositionHandler.java:94-160 and F_PlayerListRespawnMixin.java:44-62.
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
    let running = {
        let mut s = server();
        s.distances.remove(&player);
        s.searches.remove(&player)
    };
    let Some(mut running) = running else {
        return;
    };
    let handed = (dimension.clone(), logic::block_of(*pos));
    if !running.is_finished() {
        if running.stale {
            // RespawnSearch.java:148-157, `take`: a search that did not end gives its chunks back.
            release_all(game, &mut running);
            log::info!("[BalancedMcHardcoreMain] The respawn point of {name} changed after the death, using default respawn position");
            // A client that waits for this exile in black must not wait for a search that is gone.
            send(game, player, &JourneySignal::Ready);
            return;
        }
        // Java holds the respawn request until the search ends (F_ServerGamePacketListenerMixin). Horizon
        // cannot hold it: the vanilla point now, the landing when the search ends (`finish_early`).
        log::info!("[BalancedMcHardcoreMain] The landing search of {name} did not end before the respawn: the player moves to the landing when it ends");
        running.early = Some(Early { name: name.to_owned(), handed });
        server().searches.insert(player, running);
        return;
    }
    let Some(placed) = place(game, name, running, &handed) else {
        return;
    };
    *pos = logic::bottom_center(placed.feet);
    dimension.clone_from(&placed.dimension);
    server().exiles.insert(player, (placed.exile, placed.effect));
}

/// The distant respawn that a finished search allows for the point that the game handed, or None for
/// the vanilla point (RespawnPositionHandler.java:94-160). The landing chunk stays held for a while from
/// now on (review semantics-6: not before the respawn).
fn place(
    game: &mut crate::game::Game<'_>,
    name: &str,
    running: Running,
    handed: &(String, [i32; 3]),
) -> Option<Placed> {
    let centre_is_world_spawn = running.centre_is_world_spawn();
    let Phase::Searching(search) = running.phase else {
        return None;
    };
    if let Some(chunk) = search.landing_chunk() {
        server().kept.push(Kept {
            owner: running.owner.clone(),
            dimension: search.dimension.clone(),
            chunk,
            ticks_left: logic::LANDING_TICKET_TICKS,
        });
    }
    if running.stale {
        log::info!("[BalancedMcHardcoreMain] The respawn point of {name} changed after the death, using default respawn position");
        return None;
    }
    let rule = crate::game::bmh_settings::rule(game, Rule::RandomDeathSpawnPoint);
    // The game hands over the respawn point as it keeps it (lan_actions.rs:3929-3955).
    let key = running.point.as_ref().map(|p| (p.dimension.as_str(), p.pos));
    let fit = logic::respawn_fit(key, centre_is_world_spawn, &handed.0, handed.1);
    let (level, centre) = match fit {
        Some(_) => (search.dimension.clone(), search.center),
        None => handed.clone(),
    };
    let Some(feet) = logic::landing(rule, running.game_mode, Some(&search), &level, centre) else {
        log::info!(
            "[BalancedMcHardcoreMain] No distant respawn for {name}, using default respawn position (handed {handed:?}, search {:?} {:?})",
            search.dimension,
            search.center
        );
        return None;
    };
    // The exile centre: the point, or the world spawn as the game keeps it.
    let exile_centre = match fit {
        Some(logic::Fit::WorldSpawn(spawn)) => {
            if let Some(world) = game.native_world_dir() {
                server().world_spawn = Some((world, spawn));
            }
            spawn
        }
        _ => search.center,
    };
    log::info!("[BalancedMcHardcoreMain] new respawn pos and bed : {feet:?}, {exile_centre:?}");
    let effect = match (&running.point, running.point_use) {
        (Some(p), Some(logic::PointUse::Anchor)) if !p.forced => PointEffect::UseCharge {
            dimension: p.dimension.clone(),
            pos: p.pos,
        },
        (Some(_), Some(logic::PointUse::Missing)) => PointEffect::NotValid,
        _ => PointEffect::None,
    };
    let exile = Exile {
        dimension: search.dimension.clone(),
        center: exile_centre,
        distance: running.world_distance,
    };
    Some(Placed { feet, dimension: search.dimension.clone(), exile, effect })
}

//// True while a player has respawned early and the move to the landing has not happened. The player then
/// stands at the vanilla point. The homecoming tick skips such a player (w1d).
pub(super) fn early_pending(player: u128) -> bool {
    server().searches.get(&player).is_some_and(|r| r.early.is_some())
}

// C2, n5: the search of a player who respawned early has ended. Move the player to the landing with the
/// host's teleport (from `server_tick`: nobody is swapped in, so it is done at once in the player's level,
/// or with the tick's level teleports for another level), then run the arrival once. Gives the exile for
/// the homecoming area, or None when the player keeps the vanilla point.
fn finish_early(game: &mut crate::game::Game<'_>, player: u128) -> Option<Exile> {
    let mut running = server().searches.remove(&player)?;
    let Some(early) = running.early.take() else {
        server().searches.insert(player, running);
        return None;
    };
    let name = early.name;
    let placed = place(game, &name, running, &early.handed)?;
    let online = game.native_server_players();
    let Some(me) = online.iter().find(|p| p.uuid == player && p.health > 0.0) else {
        log::info!("[BalancedMcHardcoreMain] {name} is not on the server or not alive: no move to the landing");
        return None;
    };
    let at = logic::bottom_center(placed.feet);
    if !game.native_teleport_player(player, &placed.dimension, at, me.yaw, me.pitch) {
        log::warn!("[BalancedMcHardcoreMain] {name} could not be moved to the landing {:?}", placed.feet);
        return None;
    }
    log::info!("[BalancedMcHardcoreMain] {name} moved to the landing after an early respawn: {:?}", placed.feet);
    let arrived = Arrived { entity: me.entity_id, position: at, dimension: placed.dimension.clone() };
    arrive(game, player, &name, placed.exile, placed.effect, Some(arrived))
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
    let count = logic::count_eternal(drops.iter().filter(|stack| stack.count > 0).map(|stack| &*stack.item));
    if let Some(pending) = server().deaths.get_mut(&player) {
        *pending = Some(count);
    }
}

// C1, C8: mark a real death. `living_death` came first and no area cancelled it. C6: the death ends
// the immunity (Java's counter lives on the `ServerPlayer` that dies).
pub(super) fn on_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    let _ = (game, cause);
    let mut s = server();
    s.deaths.insert(player, None);
    if let Some(entity) = s.players.get(&player).map(|p| p.entity) {
        s.shields.clear(entity);
    }
}

// C1: cache the Eternal Item count and the entity id. C6: count down the immunity.
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
    known.eternal_slots = count;
    s.shields.tick(me.entity_id);
}

// C2, C6, C7, C8: read the exile record, start the immunity, queue the arrival sounds, use the
// anchor charge or tell that the point is gone, send the arrival signal. C9, C10: give back the exile
// of this respawn, once (Java `takeExile`). Give None when the respawn was not a distant one or the
// world is hardcore. The hook helper hands the exile to `bmh_homecoming::on_exile`.
// RespawnEffectHandler.java:27-65. An early respawn (no exile yet) gets the immunity now: the Java
// player stays dead, so out of reach, until the search ends.
pub(super) fn on_player_respawned(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) -> Option<Exile> {
    let me = game.native_acting_player();
    let taken = server().exiles.remove(&player);
    let Some((exile, effect)) = taken else {
        let early = server().searches.get(&player).is_some_and(|r| r.early.is_some());
        if early && let Some(me) = &me {
            server().shields.start(me.entity_id);
        }
        return None;
    };
    let arrived = me.map(|me| Arrived { entity: me.entity_id, position: me.position, dimension: me.dimension });
    arrive(game, player, name, exile, effect, arrived)
}

/// The arrival steps of a distant respawn (RespawnEffectHandler.java:27-65), at the respawn or, after
/// an early respawn, at the move to the landing. Gives the exile back, or None in a hardcore world.
fn arrive(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    exile: Exile,
    effect: PointEffect,
    me: Option<Arrived>,
) -> Option<Exile> {
    // RespawnEffectHandler.java:38: no journey in a hardcore world.
    if level_dat(game).0 {
        return None;
    }
    let deaths = game
        .native_player_stat(player, "minecraft:custom", "minecraft:deaths")
        .unwrap_or(0);
    let text = crate::game::bmh_settings::rule(game, Rule::DisplayTextOnRespawn);
    match effect {
        // Vanilla PlayerList.respawn: NO_RESPAWN_BLOCK_AVAILABLE, shown as this chat line.
        PointEffect::NotValid => {
            let line = horizon_client::text::Text::translate(logic::NOT_VALID_KEY, &[]);
            game.native_tell(player, line);
        }
        PointEffect::UseCharge { dimension, pos } => server().charges.push((player, dimension, pos)),
        PointEffect::None => {}
    }
    {
        let mut s = server();
        for path in logic::journey_awards(deaths, exile.distance) {
            s.awards.push((player, name.to_owned(), path));
        }
        if let Some(me) = &me {
            // RespawnEffectHandler.java:59.
            s.shields.start(me.entity);
            // RespawnEffectHandler.java:68-93: at the player's block, for the others nearby.
            let block = logic::block_of(me.position);
            for line in logic::sound_commands(name, block).into_iter().flatten() {
                s.sounds.push((player, me.dimension.clone(), block, line));
            }
        }
    }
    // RespawnEffectHandler.java:63-64, JourneySignal.java:37-46.
    let arrival = JourneySignal::Arrival { name: text.then(|| name.to_owned()) };
    send(game, player, &arrival);
    Some(exile)
}

// State: remove this player from the area's per-player statics, and drop the search (Java
// `F_ServerGamePacketListenerMixin.bmh$dropSearch`). The distance of a death stays for a relog.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = name;
    cancel_search(game, player);
    let mut s = server();
    if let Some(known) = s.players.remove(&player) {
        s.shields.clear(known.entity);
    }
    s.deaths.remove(&player);
    s.exiles.remove(&player);
    s.awards.retain(|(who, _, _)| *who != player);
    s.sounds.retain(|(who, _, _, _)| *who != player);
    s.charges.retain(|(who, _, _)| *who != player);
}

// C1, C3, C8: the deaths of the tick, the search timeouts, the landing tickets. C9: the journey advancements (`run_command` works only where no player is swapped
// in). C2 (n5): the move to the landing after an early respawn, whose search ended. Gives the exiles of
// those moves; the hook helper hands each one to `bmh_homecoming::on_exile`.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) -> Vec<(u128, Exile)> {
    let _ = context;
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return Vec::new();
    };
    let online = game.native_server_players();
    let deaths = std::mem::take(&mut server().deaths);
    for (player, drop_count) in deaths {
        // A player who just logged in may not be listed yet: try again at the next tick.
        let Some(me) = online.iter().find(|p| p.uuid == player) else {
            server().deaths.insert(player, drop_count);
            continue;
        };
        // Review semantics-1: Horizon kills a player who logs in dead once more (log "joined the game"
        // then "died"). That death keeps the distance of the first one, as Java's `distancesFromBed`
        // keeps it over a relog (RespawnPositionHandler.java:104-106). After a restart the map is
        // empty and the inventory counts, as in Java.
        let (cached, stored) = {
            let s = server();
            (s.players.get(&player).map_or(0, |p| p.eternal_slots), s.distances.get(&player).copied())
        };
        handle_death(game, player, drop_count.unwrap_or(cached), me.game_mode, stored);
    }
    // RespawnSearch.java:103-112: the timeout, in server ticks that ran (review semantics-7).
    let late: Vec<u128> = {
        let mut s = server();
        for running in s.searches.values_mut() {
            running.ticks = running.ticks.saturating_add(1);
        }
        s.searches
            .iter()
            .filter(|(_, r)| !r.ready_sent && logic::timed_out(r.ticks))
            .map(|(player, _)| *player)
            .collect()
    };
    for player in late {
        advance(game, player);
    }
    // After the deaths of this tick: a new death of the same player has already dropped the old search.
    let early: Vec<u128> = server()
        .searches
        .iter()
        .filter(|(_, r)| r.early.is_some() && r.is_finished())
        .map(|(player, _)| *player)
        .collect();
    let mut exiles = Vec::new();
    for player in early {
        if let Some(exile) = finish_early(game, player) {
            exiles.push((player, exile));
        }
    }
    tick_kept(game);
    let awards = std::mem::take(&mut server().awards);
    for (player, name, path) in awards {
        // Review semantics-5: an advancement the player already has is no failure.
        let id = format!("bmh:{path}");
        let done = game
            .native_player_advancements(player)
            .and_then(|all| all.get(&id).and_then(|a| a.get("done")).and_then(|d| d.as_bool()))
            .unwrap_or(false);
        if done {
            log::debug!("[BalancedMcHardcoreMain] {name} already has advancement {id}");
        } else if !crate::game::bmh::award(game, &name, path) {
            log::warn!("[BalancedMcHardcoreMain] Advancement {id} not awarded to {name}");
        }
    }
    exiles
}

// C3: resolve the points and advance the searches of this level, now in place. C7: the arrival
// sounds of this level (`/playsound` reaches only the players of the level in place). The anchor
// charges of this level.
pub(super) fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    let (sounds, charges) = {
        let mut s = server();
        let (here, other): (Vec<_>, Vec<_>) =
            std::mem::take(&mut s.sounds).into_iter().partition(|(_, level, _, _)| level == dimension);
        s.sounds = other;
        let (charges, other): (Vec<_>, Vec<_>) =
            std::mem::take(&mut s.charges).into_iter().partition(|(_, level, _)| level == dimension);
        s.charges = other;
        (here, charges)
    };
    if !sounds.is_empty() {
        let online = game.native_server_players();
        for (player, _, block, line) in sounds {
            // Review semantics-5: with nobody near, `/playsound` fails by design; skip it.
            let others = online
                .iter()
                .filter(|p| p.uuid != player && p.dimension == dimension)
                .map(|p| p.position);
            if !logic::someone_hears(block, others) {
                continue;
            }
            if !game.native_run_command(&line) {
                log::warn!("[BalancedMcHardcoreMain] Sound command failed: {line}");
            }
        }
    }
    for (player, _, pos) in charges {
        // Vanilla `findRespawnAndUseSpawnBlock` with `useCharge`, then the deplete sound to the player.
        let charges = game
            .native_block_property(pos, "charges")
            .and_then(|c| c.parse::<i32>().ok())
            .and_then(logic::anchor_after_use);
        if let Some(left) = charges.filter(|_| game.native_block(pos) == "minecraft:respawn_anchor") {
            game.native_set_block_property(pos, "charges", &left.to_string());
            let at = Some(pos.map(f64::from));
            game.native_play_sound_to(player, logic::ANCHOR_DEPLETE_SOUND, "block", at, 1.0, 1.0);
        }
    }
    let (resolving, searching): (Vec<u128>, Vec<u128>) = {
        let s = server();
        let resolving = s
            .searches
            .iter()
            .filter(|(_, r)| matches!(&r.phase, Phase::Resolving(p) if p.dimension == dimension))
            .map(|(player, _)| *player)
            .collect();
        let searching = s
            .searches
            .iter()
            .filter(|(_, r)| r.search().is_some_and(|x| x.dimension == dimension && !x.is_finished()))
            .map(|(player, _)| *player)
            .collect();
        (resolving, searching)
    };
    for player in resolving {
        resolve(game, player);
    }
    for player in searching {
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
    if server().shields.blocks(entity, damage_type) {
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
    let Some(finished) = server().searches.get(&player).map(Running::is_finished) else {
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
// The point is read at death, so only a change while dead matters: the search then does not fit the
// respawn any more (Java searched again; here the respawn keeps the vanilla point).
pub(super) fn on_player_spawn_point_set(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    pos: [i32; 3],
    cause: &str,
    cancel: &mut bool,
) {
    let _ = (game, name, dimension, pos, cause);
    if *cancel {
        return;
    }
    // After an early respawn the player is alive: the respawn itself can reset a point that does not
    // work, and the fit with the handed point was already decided.
    if let Some(running) = server().searches.get_mut(&player).filter(|r| r.early.is_none()) {
        running.stale = true;
    }
}

// C2: `/setworldspawn` moved the world spawn, the centre of an exile with no working point. A search
// already round the old one does not fit any more.
pub(super) fn on_spawn_change(game: &mut crate::game::Game<'_>, pos: [i32; 3], angle: [f32; 2]) {
    let _ = angle;
    let Some(world) = game.native_world_dir() else {
        return;
    };
    let mut s = server();
    s.world_spawn = Some((world, pos));
    for running in s.searches.values_mut() {
        if running.search().is_some() && running.centre_is_world_spawn() {
            running.stale = true;
        }
    }
}
