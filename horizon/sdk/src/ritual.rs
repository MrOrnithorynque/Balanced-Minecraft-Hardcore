use crate::logic::{ids, moon as logic, totem};
use mod_api::{
    AnvilUpdateEvent, ClientHost, Compound, Flow, GameMode, Hand, Host, HostSide, Identifier,
    ItemDisplayContext, ItemPart, ItemStack, LivingDamagePostEvent, LivingEntityUseItemFinishEvent,
    LivingEntityUseItemStartEvent, LivingEntityUseItemStopEvent, LivingEntityUseItemTickEvent,
    PlayerRef, Registrar, RenderHandEvent, ServerHost, ServerTickEvent, ServerTickPhase,
    SoundSource, StatType, Tag, Text, UseAnimation, UseAnimationEvent, UseDurationEvent,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

mod effects;

const STALE_TICKS: u64 = 3;
const FULL_BRIGHT: u32 = 0xf0f0;

#[derive(Clone, Copy)]
struct Active {
    hand: Hand,
    seen: u64,
    completed: bool,
}

struct Server {
    tick: u64,
    active: BTreeMap<u128, Active>,
    hits: BTreeMap<i32, u64>,
    awards: Vec<u128>,
    puffs: Vec<(u128, Hand)>,
    sounds: Vec<(u128, logic::Sound)>,
    rng: logic::Rng,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    tick: 0,
    active: BTreeMap::new(),
    hits: BTreeMap::new(),
    awards: Vec::new(),
    puffs: Vec::new(),
    sounds: Vec::new(),
    rng: logic::Rng::new(0x5EED_C0DE_0000_0001),
});

fn server() -> MutexGuard<'static, Server> {
    SERVER.lock().unwrap_or_else(|error| error.into_inner())
}

struct DormantTotem;

#[mod_api::horizon_part]
impl ItemPart for DormantTotem {
    fn use_duration(&self, _host: &dyn Host, event: &mut UseDurationEvent<'_>) {
        event.duration = logic::RITUAL_TICKS;
    }

    fn use_animation(&self, _host: &dyn ClientHost, event: &mut UseAnimationEvent<'_>) {
        *event.animation = UseAnimation::Bow;
    }
}

pub(super) fn register(registrar: &mut dyn Registrar) {
    let dormant = registrar.item_key(logic::RITUAL_ITEM);
    registrar.item_part(dormant, DormantTotem);
    effects::register(registrar);
}

pub(super) fn anvil_update(host: HostSide, event: &mut AnvilUpdateEvent<'_>) {
    let host: &dyn Host = match host {
        HostSide::Server(host) => host,
        HostSide::Client(host) => host,
    };
    let update = &mut event.update;
    let left = update.left.item().id(host);
    let right = update.right.as_ref().map(|stack| stack.item().id(host));
    if totem::anvil_case(&left, right.as_deref()) != Some(totem::Anvil::Awaken) {
        return;
    }
    if let Some(awake) = item(host, ids::TOTEM_OF_ETERNITY) {
        update.output = Some(ItemStack::of(awake));
        update.xp_cost = totem::AWAKEN_COST;
        update.material_cost = totem::AWAKEN_MATERIAL_COST;
    }
}

fn item(host: &dyn Host, name: &str) -> Option<mod_api::ItemKey> {
    host.item_key(&Identifier::parse_namespaced(name)?)
}

fn is_ritual(host: &dyn Host, stack: &ItemStack) -> bool {
    item(host, logic::RITUAL_ITEM).is_some_and(|dormant| stack.item() == dormant)
}

fn look(host: &dyn Host, player: PlayerRef) -> Option<logic::Look> {
    let [yaw, pitch] = player.entity().rotation(host)?;
    let day_time = player.level(host)?.day_time(host)?;
    Some(logic::Look {
        yaw,
        pitch,
        day_time,
    })
}

fn tell(host: &mut dyn ServerHost, player: PlayerRef, key: &str) {
    let _ = player.action_bar(host, Text::translate(key, vec![]));
}

fn sound(host: &mut dyn ServerHost, player: PlayerRef, cue: logic::Sound) {
    if let (Some(level), Some(at)) = (player.level(host), player.entity().position(host)) {
        level.play_sound(
            host,
            cue.id,
            at,
            SoundSource::Players,
            cue.volume,
            cue.pitch,
        );
    }
}

fn fail(host: &mut dyn ServerHost, player: PlayerRef, key: &str, stack: &ItemStack) {
    {
        let mut state = server();
        if let Some(active) = state.active.remove(&player.uuid()) {
            state.puffs.push((player.uuid(), active.hand));
        }
    }
    let group = host.item_cooldown_group(stack);
    player.set_item_cooldown(host, &group, logic::FAIL_COOLDOWN_TICKS as i32);
    server().sounds.push((player.uuid(), logic::FAIL_SOUND));
    tell(host, player, key);
}

pub(super) fn start(host: HostSide, event: &LivingEntityUseItemStartEvent<'_>) -> Flow {
    let host = match host {
        HostSide::Client(host) => {
            if event.entity.is_none()
                && is_ritual(host, event.stack)
                && let Some(player) = host.local_player()
                && let (Some(sky), Some(look)) =
                    (crate::amulet::sky(host, player), look(host, player))
                && logic::start_test(false, &sky, &look) != logic::Start::Go
            {
                // A refused start still consumes the click on this side.
                return Flow::Cancel;
            }
            return Flow::Continue;
        }
        HostSide::Server(host) => host,
    };
    if event.entity.is_some() || !is_ritual(host, event.stack) {
        return Flow::Continue;
    }
    let Some(player) = host.players().acting_player() else {
        return Flow::Cancel;
    };
    let (Some(sky), Some(look)) = (crate::amulet::sky(host, player), look(host, player)) else {
        return Flow::Cancel;
    };
    let group = host.item_cooldown_group(event.stack);
    let cooling = player.has_item_cooldown(host, &group).unwrap_or(false);
    match logic::start_test(cooling, &sky, &look) {
        logic::Start::Cooling => Flow::Cancel,
        logic::Start::Refused(key) => {
            tell(host, player, &key);
            Flow::Cancel
        }
        logic::Start::Go => {
            let mut state = server();
            let seen = state.tick;
            state.active.insert(
                player.uuid(),
                Active {
                    hand: event.hand,
                    seen,
                    completed: false,
                },
            );
            state.sounds.push((player.uuid(), logic::START_SOUND));
            Flow::Continue
        }
    }
}

pub(super) fn tick(host: HostSide, event: &LivingEntityUseItemTickEvent<'_>) -> Flow {
    let HostSide::Server(host) = host else {
        return Flow::Continue;
    };
    if event.entity.is_some() || !is_ritual(host, event.stack) {
        return Flow::Continue;
    }
    let Some(player) = host.players().acting_player() else {
        return Flow::Cancel;
    };
    let hurt = {
        let mut state = server();
        let now = state.tick;
        let hurt = logic::is_hurt(state.hits.get(&(player.entity().id() as i32)).copied(), now);
        let Some(active) = state.active.get_mut(&player.uuid()) else {
            return Flow::Cancel;
        };
        if active.hand != event.hand {
            return Flow::Cancel;
        }
        active.seen = now;
        hurt
    };
    let (Some(sky), Some(look)) = (crate::amulet::sky(host, player), look(host, player)) else {
        server().active.remove(&player.uuid());
        return Flow::Cancel;
    };
    let input = logic::TickInput {
        blocked: logic::blocked_reason(&sky, &look),
        hurt,
        remaining: event.duration,
        creative: player.game_mode(host) == Some(GameMode::Creative),
        levels: player
            .experience(host)
            .map_or(0, |experience| experience.level),
    };
    match logic::tick(&input) {
        logic::Tick::Fail(key) => {
            fail(host, player, &key, event.stack);
            Flow::Cancel
        }
        logic::Tick::Go { drain, chime } => {
            if drain && !player.give_levels(host, -1) {
                fail(host, player, logic::KEY_LEVELS, event.stack);
                return Flow::Cancel;
            }
            if let Some(pitch) = chime {
                server().sounds.push((
                    player.uuid(),
                    logic::Sound {
                        id: logic::CHIME_ID,
                        volume: 1.0,
                        pitch,
                    },
                ));
            }
            if event.duration == 1
                && let Some(active) = server().active.get_mut(&player.uuid())
            {
                active.completed = true;
            }
            Flow::Continue
        }
    }
}

pub(super) fn stop(host: HostSide, event: &LivingEntityUseItemStopEvent<'_>) {
    let HostSide::Server(host) = host else {
        return;
    };
    if event.entity.is_some() || !is_ritual(host, event.stack) {
        return;
    }
    if let Some(player) = host.players().acting_player()
        && server().active.contains_key(&player.uuid())
    {
        fail(host, player, logic::KEY_RELEASED, event.stack);
    }
}

pub(super) fn finish(host: HostSide, event: &mut LivingEntityUseItemFinishEvent<'_>) {
    let HostSide::Server(host) = host else {
        return;
    };
    if event.entity.is_some() || !is_ritual(host, event.stack) {
        return;
    }
    let Some(player) = host.players().acting_player() else {
        return;
    };
    let Some(active) = server().active.remove(&player.uuid()) else {
        return;
    };
    // A tick cancellation also fires Finish. Only the final good tick wakes the item.
    if !active.completed || active.hand != event.hand {
        return;
    }
    if let Some(awake) = item(host, logic::WOKEN_ITEM) {
        *event.result = Some(ItemStack::of(awake));
        player.award_stat(host, StatType::Used, logic::RITUAL_ITEM, 1);
        server().awards.push(player.uuid());
        tell(host, player, logic::KEY_WOKEN);
    }
}

pub(super) fn damage_post(event: &LivingDamagePostEvent<'_>) {
    if event.kind == "minecraft:player" && event.container.inflicted() > 0.0 {
        let mut state = server();
        let tick = state.tick;
        state.hits.insert(event.entity, tick);
    }
}

pub(super) fn logout(player: u128) {
    let mut state = server();
    state.active.remove(&player);
    state.awards.retain(|id| *id != player);
}

pub(super) fn server_tick(host: &mut dyn ServerHost, event: &ServerTickEvent<'_>) {
    let tick = event.context.tick.get();
    let (awards, puffs, sounds, running) = {
        let mut state = server();
        if tick < state.tick {
            state.active.clear();
            state.hits.clear();
            state.awards.clear();
            state.puffs.clear();
            state.sounds.clear();
        }
        state.tick = tick;
        if !matches!(event.phase, ServerTickPhase::ServerTickEnd { .. }) {
            return;
        }
        state
            .active
            .retain(|_, active| tick.saturating_sub(active.seen) <= STALE_TICKS);
        state.hits.retain(|_, hit| logic::is_hurt(Some(*hit), tick));
        let running = state
            .active
            .iter()
            .filter(|(_, active)| active.seen == tick)
            .map(|(&player, active)| (player, active.hand))
            .collect::<Vec<_>>();
        (
            std::mem::take(&mut state.awards),
            std::mem::take(&mut state.puffs),
            std::mem::take(&mut state.sounds),
            running,
        )
    };
    for (player, hand) in running {
        if let Some(owner) = host.player(player) {
            effects::particles(host, owner, hand, false);
        }
    }
    for (player, hand) in puffs {
        if let Some(owner) = host.player(player) {
            effects::particles(host, owner, hand, true);
        }
    }
    for (player, cue) in sounds {
        if let Some(player) = host.player(player) {
            sound(host, player, cue);
        }
    }
    for player in awards {
        let Some(player) = host.player(player) else {
            continue;
        };
        player.entity().broadcast_event(host, 35);
        sound(host, player, logic::WOKEN_SOUND);
        if let Some(name) = player.name(host) {
            host.commands().run_command(&format!(
                "advancement grant {name} only bmh:{}",
                logic::ADVANCEMENT
            ));
        }
    }
}

pub(super) fn render_hand(host: &dyn ClientHost, event: &mut RenderHandEvent<'_, '_>) -> Flow {
    let render = &event.render;
    let Some(stack) = render.stack.filter(|stack| is_ritual(host, stack)) else {
        return Flow::Continue;
    };
    let Some(living) = host
        .local_player()
        .and_then(|player| host.living(player.entity()))
    else {
        return Flow::Continue;
    };
    if living.used_item_hand(host) != Some(render.hand) {
        return Flow::Continue;
    }
    let Some(remaining) = living
        .item_use_remaining(host)
        .filter(|remaining| *remaining > 0)
    else {
        return Flow::Continue;
    };
    let elapsed = (logic::RITUAL_TICKS - remaining) as f32 + render.partial;
    let side = if render.hand == Hand::Main { 1.0 } else { -1.0 };
    let (offset, roll) = logic::totem_target(elapsed, side);
    let light = event.draw.light();
    event.draw.push();
    event.draw.translate(offset);
    event.draw.rotate_about([0.0, 0.0, 1.0], roll, [0.0; 3]);
    event.draw.scale([logic::SCALE; 3]);
    event.draw.rotate_about([0.0, 1.0, 0.0], 180.0, [0.0; 3]);
    event.draw.set_light(FULL_BRIGHT);
    let mut charging = stack.clone();
    let mut glint = Compound::new();
    glint.insert("minecraft:enchantment_glint_override".into(), Tag::Byte(1));
    let _ = charging.apply_components(&glint);
    let drawn = event.draw.item(
        &charging,
        ItemDisplayContext::Fixed,
        None,
        render.player_entity_id,
    );
    event.draw.set_light(light);
    event.draw.pop();
    if drawn { Flow::Cancel } else { Flow::Continue }
}
