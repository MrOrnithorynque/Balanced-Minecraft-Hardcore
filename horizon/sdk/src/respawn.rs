use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::logic::contract::{Exile, Rule};
use crate::logic::respawn::{self as logic, Blocks, SearchWorld};
use mod_api::{
    GameMode, Heightmap, LevelRef, LivingDeathEvent, PlayerRef, PlayerRespawnPositionEvent,
    RandomSource, Registrar, ServerHost, SoundSource, StatType, TicketController, TicketOwner,
    ValidateTicketsEvent,
};

use crate::logic::wire::JourneySignal;
use crate::{journey, settings};

static CONTROLLER: OnceLock<TicketController> = OnceLock::new();
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

#[derive(Default)]
struct State {
    deaths: HashMap<u128, i32>,
    searches: HashMap<u128, Pending>,
    arrivals: HashMap<u128, Exile>,
    releases: Vec<Release>,
    shields: logic::Shields,
}

struct Pending {
    point: Option<logic::RespawnPoint>,
    distance: i32,
    search: logic::Search,
    ticks: u32,
    early: bool,
    departure_sent: bool,
    ready_sent: bool,
}

struct Release {
    player: u128,
    dimension: String,
    chunk: [i32; 2],
    ticks: u32,
}

fn state() -> MutexGuard<'static, State> {
    STATE
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub(super) fn register(registrar: &mut dyn Registrar) {
    let _ = CONTROLLER.set(registrar.register_ticket_controller("landing", true));
}

pub(super) fn reset() {
    *state() = State::default();
}

pub(super) fn validate_tickets(event: &mut ValidateTicketsEvent<'_>) {
    if Some(&event.controller) == CONTROLLER.get() {
        for (player, _) in event.validation.entity_tickets() {
            event.validation.remove_entity_tickets(player);
        }
    }
}

pub(super) fn player_by_entity(host: &dyn ServerHost, entity: u32) -> Option<PlayerRef> {
    host.players()
        .server_players()
        .into_iter()
        .find(|player| player.entity().id() == entity)
}

pub(super) fn record_inventory(host: &dyn ServerHost, event: &LivingDeathEvent<'_>) {
    if !settings::rule(host, Rule::RandomDeathSpawnPoint) || event.kind != "minecraft:player" {
        return;
    }
    let Some(player) = player_by_entity(host, event.entity) else {
        return;
    };
    let items = player.inventory_slots(host).unwrap_or_default();
    let count = items
        .iter()
        .filter(|(_, stack)| crate::logic::ids::is_eternal(&stack.item().id(host)))
        .count();
    state().deaths.insert(player.uuid(), count as i32);
}

pub(super) fn on_player_died(host: &mut dyn ServerHost, uuid: u128) {
    let count = state().deaths.remove(&uuid).unwrap_or_default();
    logout(host, uuid);
    let Some(player) = host.player(uuid) else {
        return;
    };
    if player.game_mode(host) != Some(GameMode::Survival)
        || !settings::rule(host, Rule::RandomDeathSpawnPoint)
    {
        return;
    }
    let Some(overworld) = host.level(logic::OVERWORLD) else {
        return;
    };
    let Some(spawn) = overworld.respawn_data(host) else {
        return;
    };
    let difficulty = overworld
        .difficulty(host)
        .and_then(logic::difficulty_id)
        .unwrap_or(2);
    let distance = logic::distance(settings::config(), count, difficulty);
    let point = respawn_point(host, player);
    let (dimension, center) = point
        .as_ref()
        .map_or((logic::OVERWORLD, spawn.pos), |point| {
            (point.dimension.as_str(), point.pos)
        });
    let search = new_search(dimension, center, distance);
    state().searches.insert(
        uuid,
        Pending {
            point,
            distance,
            search,
            ticks: 0,
            early: false,
            departure_sent: false,
            ready_sent: false,
        },
    );
}

fn new_search(dimension: &str, center: [i32; 3], distance: i32) -> logic::Search {
    logic::Search::new(
        dimension,
        center,
        logic::level_distance(distance, logic::coordinate_scale(dimension)),
    )
}

fn respawn_point(host: &dyn ServerHost, player: PlayerRef) -> Option<logic::RespawnPoint> {
    let point = player.respawn_config(host).flatten()?;
    let pos = point.position.pos();
    Some(logic::RespawnPoint {
        dimension: point.position.dimension().to_string(),
        pos: [pos.x, pos.y, pos.z],
        yaw: point.yaw,
        forced: point.forced,
    })
}

fn ticket(host: &mut dyn ServerHost, level: LevelRef, player: u128, chunk: [i32; 2], hold: bool) {
    if let Some((controller, chunk)) = CONTROLLER.get().zip(level.chunk_ref(host, chunk)) {
        chunk.set_mod_forced(host, *controller, TicketOwner::Entity(player), hold, false);
    }
}

struct LevelReads<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
    player: u128,
    dimension: &'a str,
}

impl Blocks for LevelReads<'_> {
    fn block(&mut self, x: i32, y: i32, z: i32) -> logic::BlockInfo {
        let pos = [x, y, z];
        let Some(block) = self.level.block(self.host, pos) else {
            return logic::BlockInfo {
                waterlogged: true,
                ..logic::BlockInfo::default()
            };
        };
        let base = pos.map(f64::from);
        let boxes = self
            .level
            .collision_boxes(
                self.host,
                [
                    base[0],
                    base[1],
                    base[2],
                    base[0] + 1.0,
                    base[1] + 1.0,
                    base[2] + 1.0,
                ],
            )
            .unwrap_or_default();
        let boxes = boxes
            .into_iter()
            .map(|bounds| {
                [
                    bounds[0] - base[0],
                    bounds[1] - base[1],
                    bounds[2] - base[2],
                    bounds[3] - base[0],
                    bounds[4] - base[1],
                    bounds[5] - base[2],
                ]
            })
            .collect();
        logic::BlockInfo {
            name: block.id(self.host),
            waterlogged: self
                .level
                .block_property(self.host, pos, "waterlogged")
                .as_deref()
                == Some("true"),
            boxes,
        }
    }

    fn surface(&mut self, x: i32, z: i32) -> Option<i32> {
        self.host
            .surface_heights(self.dimension, &[(x, z)], Heightmap::MotionBlockingNoLeaves)
            .ok()?
            .first()
            .copied()
            .flatten()
    }
}

impl SearchWorld for LevelReads<'_> {
    fn random(&mut self) -> f64 {
        self.level
            .with_random(self.host, |random| random.next_double())
            .unwrap_or(0.0)
    }

    fn inside_border(&mut self, x: i32, z: i32) -> bool {
        self.level
            .world_border(self.host)
            .is_none_or(|border| logic::inside_border(border, x, z))
    }

    fn hold(&mut self, chunk: [i32; 2]) {
        ticket(self.host, self.level, self.player, chunk, true);
    }

    fn release(&mut self, chunk: [i32; 2]) {
        ticket(self.host, self.level, self.player, chunk, false);
    }

    fn loaded(&mut self, x: i32, z: i32) -> bool {
        self.surface(x, z).is_some()
    }

    fn biome(&mut self, x: i32, y: i32, z: i32) -> Option<String> {
        self.level.biome_at(self.host, [x, y, z])
    }

    fn shape(&mut self) -> logic::LevelShape {
        let (min_y, height) = self.level.height(self.host).unwrap_or((-64, 384));
        logic::LevelShape::of(self.dimension, min_y, height)
    }
}

pub(super) fn on_level_tick(host: &mut dyn ServerHost, level: LevelRef) {
    let Some(dimension) = level.dimension().id(host) else {
        return;
    };
    let players: Vec<_> = state()
        .searches
        .iter()
        .filter(|(_, pending)| pending.search.dimension == dimension)
        .map(|(&player, _)| player)
        .collect();
    for uuid in players {
        let Some(mut pending) = state().searches.remove(&uuid) else {
            continue;
        };
        if !pending.departure_sent
            && let Some(player) = host.player(uuid)
        {
            journey::send(host, player, JourneySignal::Departure);
            pending.departure_sent = true;
        }
        let mut world = LevelReads {
            host,
            level,
            player: uuid,
            dimension: &dimension,
        };
        pending.ticks = pending.ticks.saturating_add(1);
        if logic::timed_out(pending.ticks) && !pending.search.is_finished() {
            pending.search.fail(&mut world);
        } else {
            pending.search.advance(&mut world);
        }
        if pending.search.is_finished()
            && !pending.ready_sent
            && let Some(player) = host.player(uuid)
        {
            journey::send(host, player, JourneySignal::Ready);
            pending.ready_sent = true;
        }
        if pending.search.is_finished() && pending.early {
            if let (Some(player), Some(feet)) = (host.player(uuid), pending.search.result()) {
                let moved = player.entity().teleport_to_level(
                    host,
                    &dimension,
                    logic::bottom_center(feet),
                    None,
                );
                if moved {
                    record_arrival(player, &pending);
                } else {
                    release_search(host, uuid, &pending);
                    journey::send(host, player, JourneySignal::Arrival { name: None });
                }
            } else if let Some(player) = host.player(uuid) {
                journey::send(host, player, JourneySignal::Arrival { name: None });
            }
        } else {
            state().searches.insert(uuid, pending);
        }
    }
}

pub(super) fn on_position(host: &mut dyn ServerHost, event: &mut PlayerRespawnPositionEvent<'_>) {
    let uuid = event.player.uuid();
    let Some(mut pending) = state().searches.remove(&uuid) else {
        return;
    };
    if event.player.game_mode(host) != Some(GameMode::Survival)
        || !settings::rule(host, Rule::RandomDeathSpawnPoint)
    {
        release_search(host, uuid, &pending);
        if pending.departure_sent {
            journey::send(host, event.player, JourneySignal::Ready);
            journey::send(host, event.player, JourneySignal::Arrival { name: None });
        }
        return;
    }
    let dimension = event.original_level.key().id();
    let center = if event.missing_respawn_block || pending.point.is_none() {
        host.level(dimension)
            .and_then(|level| level.respawn_data(host))
            .map_or_else(
                || logic::block_of(event.original_position),
                |point| point.pos,
            )
    } else if let Some(point) = respawn_point(host, event.player) {
        point.pos
    } else {
        release_search(host, uuid, &pending);
        return;
    };
    if !pending.search.matches(dimension, center) {
        release_search(host, uuid, &pending);
        pending.search = new_search(dimension, center, pending.distance);
        pending.ticks = 0;
    }
    if let Some(feet) = pending.search.result() {
        *event.position = logic::bottom_center(feet);
        record_arrival(event.player, &pending);
    } else if !pending.search.is_finished() {
        pending.early = true;
        state().searches.insert(uuid, pending);
    } else if pending.departure_sent {
        journey::send(host, event.player, JourneySignal::Arrival { name: None });
    }
}

fn record_arrival(player: PlayerRef, pending: &Pending) {
    let mut state = state();
    state.arrivals.insert(
        player.uuid(),
        Exile {
            dimension: pending.search.dimension.clone(),
            center: pending.search.center,
            distance: pending.search.distance,
        },
    );
    state.shields.start(player.entity().id() as i32);
    if let Some(chunk) = pending.search.landing_chunk() {
        state.releases.push(Release {
            player: player.uuid(),
            dimension: pending.search.dimension.clone(),
            chunk,
            ticks: logic::LANDING_TICKET_TICKS,
        });
    }
}

pub(super) fn end_tick(host: &mut dyn ServerHost) {
    let (arrivals, releases) = {
        let mut state = state();
        for player in host.players().server_players() {
            state.shields.tick(player.entity().id() as i32);
        }
        let mut ready = Vec::new();
        let mut keep = Vec::new();
        for mut release in std::mem::take(&mut state.releases) {
            release.ticks = release.ticks.saturating_sub(1);
            if release.ticks == 0 {
                ready.push(release);
            } else {
                keep.push(release);
            }
        }
        state.releases = keep;
        (std::mem::take(&mut state.arrivals), ready)
    };
    for release in releases {
        if let Some(level) = host.level(&release.dimension) {
            ticket(host, level, release.player, release.chunk, false);
        }
    }
    for (uuid, exile) in arrivals {
        if let Some(player) = host.player(uuid) {
            arrive(host, player, &exile);
        }
    }
    for player in host.players().server_players() {
        if !state().searches.contains_key(&player.uuid()) {
            crate::homecoming::tick(host, player);
        }
    }
}

fn arrive(host: &mut dyn ServerHost, player: PlayerRef, exile: &Exile) {
    let Some(name) = player.name(host) else {
        return;
    };
    crate::homecoming::arrived(host, player, exile);
    let journey_name = settings::rule(host, Rule::DisplayTextOnRespawn).then(|| name.clone());
    journey::send(host, player, JourneySignal::Arrival { name: journey_name });
    eprintln!("[BMH] {name} respawned {} blocks from home", exile.distance);
    player.play_sound(
        host,
        logic::RESPAWN_SOUND,
        SoundSource::Players,
        1.0,
        logic::SOUND_PITCH,
    );
    if let Some((level, pos)) = player.level(host).zip(player.entity().position(host)) {
        level.play_sound(
            host,
            logic::BELL_SOUND,
            pos,
            SoundSource::Players,
            logic::BELL_VOLUME,
            logic::SOUND_PITCH,
        );
    }
    let deaths = player
        .stat(host, StatType::Custom, "minecraft:deaths")
        .unwrap_or(0);
    for award in logic::journey_awards(deaths, exile.distance) {
        if let Some(command) = logic::award_command(&name, award) {
            host.commands().run_command(&command);
        }
    }
}

fn release_search(host: &mut dyn ServerHost, player: u128, pending: &Pending) {
    if let Some(level) = host.level(&pending.search.dimension) {
        if let Some(chunk) = pending
            .search
            .held_chunk()
            .or_else(|| pending.search.landing_chunk())
        {
            ticket(host, level, player, chunk, false);
        }
    }
}

pub(super) fn logout(host: &mut dyn ServerHost, player: u128) {
    let pending = {
        let mut state = state();
        state.arrivals.remove(&player);
        state.deaths.remove(&player);
        if let Some(player) = host.player(player) {
            state.shields.clear(player.entity().id() as i32);
        }
        state.searches.remove(&player)
    };
    if let Some(pending) = pending {
        release_search(host, player, &pending);
    }
}

pub(super) fn blocks_damage(entity: i32, damage_type: &str) -> bool {
    state().shields.blocks(entity, damage_type)
}

pub(super) fn awaiting_respawn(player: u128) -> bool {
    state().searches.contains_key(&player)
}
