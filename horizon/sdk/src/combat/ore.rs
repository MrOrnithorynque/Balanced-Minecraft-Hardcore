use crate::logic::{
    ids, tools,
    wire::{TOOLS_CHANNEL, ToolsSignal},
};
use mod_api::{
    BlockDropsEvent, ClientHost, IPayloadContext, IPayloadHandler, ItemKey, LevelHost, LevelRef,
    PayloadBytes, PayloadHandleError, PayloadId, Registrar, ServerHost, SoundSource,
    SubmitCustomGeometryEvent, WorldDepth, WorldLine,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

struct BrokenOre {
    player: u128,
    dimension: &'static str,
    block: String,
    pos: [i32; 3],
}

struct Server {
    pending: Vec<BrokenOre>,
    cooldowns: BTreeMap<u128, i64>,
    glows: tools::GlowBook,
}

impl Server {
    const fn new() -> Self {
        Self {
            pending: Vec::new(),
            cooldowns: BTreeMap::new(),
            glows: tools::GlowBook::new(),
        }
    }
}

static SERVER: Mutex<Server> = Mutex::new(Server::new());
static PICKAXE: OnceLock<ItemKey> = OnceLock::new();
const MAX_CLIENT_GLOWS: usize = 2048;

struct Glow {
    level: LevelRef,
    block: String,
    until: Instant,
}

static CLIENT: Mutex<BTreeMap<[i32; 3], Glow>> = Mutex::new(BTreeMap::new());

pub(in crate::combat) fn register(registry: &mut dyn Registrar) {
    let _ = PICKAXE.set(registry.item_key(ids::ETERNAL_PICKAXE));
    registry
        .payload_handlers()
        .registrar("1")
        .optional()
        .play_to_client(PayloadId(TOOLS_CHANNEL.into()), Some(Arc::new(GlowHandler)));
}

pub(in crate::combat) fn clear() {
    *SERVER.lock().unwrap_or_else(|e| e.into_inner()) = Server::new();
}

pub(in crate::combat) fn forget(uuid: u128) {
    let mut server = SERVER.lock().unwrap_or_else(|e| e.into_inner());
    server.cooldowns.remove(&uuid);
    server.pending.retain(|ore| ore.player != uuid);
}

pub(crate) fn block_drops(level: &dyn LevelHost, event: &BlockDropsEvent<'_>) {
    let (Some(player), Some(stack)) = (event.breaker, event.tool) else {
        return;
    };
    if PICKAXE.get().copied() != Some(stack.item()) || super::worn(stack) {
        return;
    }
    let Some(block) = level.state_block(event.state) else {
        return;
    };
    SERVER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .pending
        .push(BrokenOre {
            player,
            dimension: event.dimension.key().id(),
            block: block.to_owned(),
            pos: [event.pos.x, event.pos.y, event.pos.z],
        });
}

pub(in crate::combat) fn tick(host: &mut dyn ServerHost) {
    let pending = std::mem::take(&mut SERVER.lock().unwrap_or_else(|e| e.into_inner()).pending);
    for ore in pending {
        read(host, ore);
    }
}

fn in_tag(host: &dyn ServerHost, block: &str, tag: &str) -> bool {
    host.block_key(block)
        .and_then(|key| host.block_in_tag(key, tag))
        .unwrap_or_else(|| tools::listed_in_tag(block, tag))
}

fn read(host: &mut dyn ServerHost, ore: BrokenOre) {
    let (Some(player), Some(level)) = (host.player(ore.player), host.level(ore.dimension)) else {
        return;
    };
    let Some(now) = level.game_time(host) else {
        return;
    };
    if player.level(host) != Some(level) || !in_tag(host, &ore.block, tools::ORE_TAG) {
        return;
    }
    let near = {
        let mut server = SERVER.lock().unwrap_or_else(|e| e.into_inner());
        let next = server.cooldowns.entry(ore.player).or_default();
        if now < *next {
            return;
        }
        *next = now + tools::COOLDOWN_TICKS;
        server.glows.expire(now);
        server.glows.near(ore.dimension, ore.pos)
    };
    let standing: Vec<_> = near
        .into_iter()
        .filter(|&pos| {
            level
                .block(host, pos)
                .is_some_and(|key| in_tag(host, &key.id(host), tools::ORE_TAG))
        })
        .collect();
    let found = tools::find_same_ore(
        ore.pos,
        |pos| level.block(host, pos).map(|key| key.id(host)),
        |block| tools::same_ore(&ore.block, block, |id, tag| in_tag(host, id, tag)),
    );
    let reading = {
        let mut server = SERVER.lock().unwrap_or_else(|e| e.into_inner());
        server
            .glows
            .forget_where(ore.dimension, ore.pos, |pos| standing.contains(&pos));
        tools::read_ores(&mut server.glows, ore.dimension, ore.pos, &found, now)
    };
    if !reading.spawned.is_empty() {
        let sound = tools::SENSE_SOUND;
        let center = ore.pos.map(|coordinate| f64::from(coordinate) + 0.5);
        level.play_sound(
            host,
            sound.id,
            center,
            SoundSource::Players,
            sound.volume,
            sound.pitch,
        );
        super::award(host, player, tools::AWARD_ECHO_SENSE);
    }
    let positions = reading.positions();
    if positions.is_empty() {
        return;
    }
    let bytes = ToolsSignal::Glow {
        ticks: tools::LIFETIME_TICKS as u32,
        positions,
    }
    .encode();
    let id = PayloadId(TOOLS_CHANNEL.into());
    for other in host.players().server_players() {
        if other.level(host) == Some(level)
            && let Some(at) = other.entity().position(host)
            && tools::sees_glow(ore.dimension, at, ore.dimension, ore.pos)
        {
            let _ = other.send_payload(host, &id, PayloadBytes(bytes.clone()));
        }
    }
}

struct GlowHandler;

impl IPayloadHandler for GlowHandler {
    fn handle(
        &self,
        payload: PayloadBytes,
        context: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        let Some(ToolsSignal::Glow { ticks, positions }) = ToolsSignal::decode(&payload.0) else {
            return Err(PayloadHandleError::Failed(
                "Invalid Echo Sense packet".into(),
            ));
        };
        let Ok(host) = context.client_host() else {
            return Ok(());
        };
        let Some(level) = host.local_player().and_then(|player| player.level(host)) else {
            return Ok(());
        };
        let now = Instant::now();
        let until =
            now + Duration::from_millis(50 * u64::from(ticks.min(tools::LIFETIME_TICKS as u32)));
        let additions: Vec<_> = positions
            .into_iter()
            .filter_map(|pos| {
                let block = level.block(host, pos)?.id(host);
                Some((
                    pos,
                    Glow {
                        level,
                        block,
                        until,
                    },
                ))
            })
            .collect();
        let mut glows = CLIENT.lock().unwrap_or_else(|e| e.into_inner());
        glows.retain(|_, glow| glow.level == level && glow.until > now);
        for (pos, glow) in additions {
            if glows.len() < MAX_CLIENT_GLOWS || glows.contains_key(&pos) {
                glows.insert(pos, glow);
            }
        }
        Ok(())
    }
}

pub(crate) fn draw(host: &dyn ClientHost, event: &mut SubmitCustomGeometryEvent<'_>) {
    let Some(level) = host.local_player().and_then(|player| player.level(host)) else {
        return;
    };
    let now = Instant::now();
    let glows = {
        let mut glows = CLIENT.lock().unwrap_or_else(|e| e.into_inner());
        glows.retain(|_, glow| glow.level == level && glow.until > now);
        glows
            .iter()
            .map(|(pos, glow)| (*pos, glow.block.clone()))
            .collect::<Vec<_>>()
    };
    let Some(dimension) = level.dimension().id(host) else {
        return;
    };
    let capacity = event.draw.usage();
    let available = capacity.line_capacity.saturating_sub(capacity.lines) as usize;
    let mut lines = Vec::new();
    for (pos, original) in glows {
        if lines.len() + 12 > available {
            break;
        }
        if !tools::sees_glow(&dimension, event.view.camera, &dimension, pos)
            || level
                .block(host, pos)
                .is_none_or(|block| block.id(host) != original)
        {
            continue;
        }
        append_box(&mut lines, pos);
    }
    if !lines.is_empty() {
        let _ = event.draw.lines(&lines);
    }
}

fn append_box(lines: &mut Vec<WorldLine>, pos: [i32; 3]) {
    let (min, max) = tools::glow_box(pos);
    for axis in 0..3 {
        let first = (axis + 1) % 3;
        let second = (axis + 2) % 3;
        for first_coordinate in [min[first], max[first]] {
            for second_coordinate in [min[second], max[second]] {
                let mut from = min;
                from[first] = first_coordinate;
                from[second] = second_coordinate;
                let mut to = from;
                to[axis] = max[axis];
                lines.push(WorldLine::new(
                    from,
                    to,
                    tools::GLOW_ARGB,
                    tools::GLOW_LINE_WIDTH,
                    WorldDepth::ThroughWalls,
                ));
            }
        }
    }
}
