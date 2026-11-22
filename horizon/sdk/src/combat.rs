pub(super) mod bundle;
pub(super) mod eternal_support;
pub(super) mod ore;
pub(super) mod rest;

use crate::logic::{eternal, ids, tools, totem};
use mod_api::{
    AnvilUpdateEvent, AttackEntityEvent, ClientHost, Compound, DamageItemEvent,
    EntityJoinLevelEvent, EntityRef, Flow, Hand, Host, HostSide, ItemBarEvent, ItemExpireEvent,
    ItemPart, ItemSpawnEvent, ItemStack, LevelRef, LivingDamagePostEvent,
    LivingIncomingDamageEvent, ParticleKey, ParticleOptions, PlayerBreakSpeedEvent, PlayerRef,
    Registrar, ServerHost, SoundSource, Tag, Text, TooltipEvent,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

#[derive(Clone, Copy)]
enum Swing {
    Worn,
    Sword { strength: f32 },
    Smash { multiplier: f32 },
}

struct Attack {
    attacker: PlayerRef,
    target: i32,
    feet: [f64; 3],
    height: f64,
    level: LevelRef,
    dimension: String,
    swing: Swing,
}

struct Applied {
    attack: Attack,
    damage: f32,
}

#[derive(Default)]
struct State {
    falls: BTreeMap<u128, tools::Fall>,
    attacks: BTreeMap<i32, Attack>,
    applied: BTreeMap<i32, Applied>,
    landed: Vec<Applied>,
    echoes: BTreeMap<u128, Vec<tools::PendingEcho>>,
}

struct Particles {
    dust: ParticleKey,
    explosion: ParticleKey,
    cloud: ParticleKey,
    crit: ParticleKey,
    electric_spark: ParticleKey,
}

static PARTICLES: OnceLock<Particles> = OnceLock::new();
static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    let mut state: MutexGuard<'_, Option<State>> = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(state.get_or_insert_with(State::default))
}

pub(super) fn register(registry: &mut dyn Registrar) {
    ore::register(registry);
    bundle::register(registry);
    let _ = PARTICLES.set(Particles {
        dust: registry.particle_key("minecraft:dust"),
        explosion: registry.particle_key("minecraft:explosion"),
        cloud: registry.particle_key("minecraft:cloud"),
        crit: registry.particle_key("minecraft:crit"),
        electric_spark: registry.particle_key("minecraft:electric_spark"),
    });
    for id in [ids::ETERNAL_SWORD, ids::ETERNAL_AXE, ids::ETERNAL_PICKAXE] {
        let item = registry.item_key(id);
        registry.item_part(item, ToolPart);
    }
}

pub(super) fn clear() {
    with_state(|state| *state = State::default());
    eternal_support::clear();
    ore::clear();
    rest::clear();
}

pub(super) fn forget(uuid: u128) {
    ore::forget(uuid);
    with_state(|state| {
        state.falls.remove(&uuid);
        state.echoes.remove(&uuid);
        state
            .attacks
            .retain(|_, attack| attack.attacker.uuid() != uuid);
        state
            .applied
            .retain(|_, hit| hit.attack.attacker.uuid() != uuid);
    });
}

pub(super) fn on_attack(host: &dyn ServerHost, event: &AttackEntityEvent<'_>) {
    let Some(player) = host.players().acting_player() else {
        return;
    };
    let entity = player.entity();
    let Ok(id) = i32::try_from(entity.id()) else {
        return;
    };

    with_state(|state| {
        state.attacks.remove(&id);
        state.applied.remove(&id);
    });
    let Some(living) = host.living(entity) else {
        return;
    };
    let Some(weapon) = living.hand_item(host, Hand::Main) else {
        return;
    };
    let Some(tool) = tools::tool_of(&weapon.item().id(host)) else {
        return;
    };
    let Some(level) = player.level(host) else {
        return;
    };
    let swing = if worn(&weapon) {
        Swing::Worn
    } else {
        match tool {
            tools::Tool::Sword => Swing::Sword {
                strength: player.attack_strength_scale(host, 0.5).unwrap_or_default(),
            },
            tools::Tool::Axe => {
                let multiplier = tools::smash_multiplier(track_fall(host, player));
                if multiplier <= 1.0 {
                    return;
                }
                Swing::Smash { multiplier }
            }
            tools::Tool::Pickaxe => return,
        }
    };
    let height = u32::try_from(event.entity)
        .ok()
        .and_then(|id| host.entity(id))
        .and_then(|target| target.dimensions(host))
        .map_or(0.0, |[_width, height]| height);

    with_state(|state| {
        state.attacks.insert(
            id,
            Attack {
                attacker: player,
                target: event.entity,
                feet: event.pos,
                height,
                level,
                dimension: event.dimension.to_owned(),
                swing,
            },
        );
    });
}

pub(super) fn incoming_damage(event: &mut LivingIncomingDamageEvent<'_>) {
    if event.cancelled || !is_melee(event.source) {
        return;
    }
    let source = event.source.causing;
    let attack = with_state(|state| {
        if state
            .attacks
            .get(&source)
            .is_some_and(|attack| attack.target == event.entity)
        {
            state.attacks.remove(&source)
        } else {
            None
        }
    });
    let Some(attack) = attack else {
        return;
    };

    let damage = event.container.new_damage();
    let factor = match attack.swing {
        Swing::Worn => tools::WORN_DAMAGE_FACTOR,
        Swing::Smash { multiplier } => multiplier,
        Swing::Sword { .. } => 1.0,
    };
    if factor != 1.0 && event.container.scale_original(factor).is_err() {
        return;
    }
    if !matches!(attack.swing, Swing::Worn) {
        with_state(|state| state.applied.insert(source, Applied { attack, damage }));
    }
}

pub(super) fn damage_post(event: &LivingDamagePostEvent<'_>) {
    if !is_melee(event.source) {
        return;
    }
    let source = event.source.causing;
    with_state(|state| {
        if state
            .applied
            .get(&source)
            .is_some_and(|hit| hit.attack.target == event.entity)
            && let Some(hit) = state.applied.remove(&source)
            && event.container.health_damage() > 0.0
        {
            state.landed.push(hit);
        }
    });
}

fn is_melee(source: &mod_api::HurtSource<'_>) -> bool {
    source.damage_type == tools::ECHO_DAMAGE_TYPE
        && source.causing != 0
        && source.direct == source.causing
}

pub(super) fn track_fall(host: &dyn ServerHost, player: PlayerRef) -> f64 {
    let entity = player.entity();
    let (Some([_, y, _]), Some(living)) = (entity.position(host), host.living(entity)) else {
        return 0.0;
    };
    let on_ground = entity.on_ground(host).unwrap_or(true);
    let input = tools::FallInput {
        y,
        on_ground,
        in_water: !on_ground && entity.is_in_water(host).unwrap_or_default(),
        climbing: !on_ground && living.is_climbing(host).unwrap_or_default(),
        gliding: !on_ground && living.is_gliding(host).unwrap_or_default(),
        flying: !on_ground
            && player
                .abilities(host)
                .is_some_and(|abilities| abilities.flying),
        floating: !on_ground
            && tools::FLOATING_EFFECTS.into_iter().any(|id| {
                host.effect_key(id)
                    .is_some_and(|key| living.potion_affected(host, key))
            }),
    };
    with_state(|state| state.falls.entry(player.uuid()).or_default().step(input))
}

pub(super) fn tick(host: &mut dyn ServerHost) {
    ore::tick(host);
    rest::tick(host);
    let landed = with_state(|state| {
        state.attacks.clear();
        state.applied.clear();
        std::mem::take(&mut state.landed)
    });
    for hit in landed {
        finish_hit(host, hit);
    }

    for player in host.players().server_players() {
        let Some(level) = player.level(host) else {
            continue;
        };
        let Some(now) = level.game_time(host) else {
            continue;
        };
        let due = with_state(|state| {
            let Some(queue) = state.echoes.get_mut(&player.uuid()) else {
                return Vec::new();
            };
            tools::take_due(queue, now)
        });
        for echo in due {
            if level.dimension().id(host).as_deref() == Some(echo.dimension.as_str()) {
                deliver_echo(host, player, level, echo);
            }
        }
    }
}

fn finish_hit(host: &mut dyn ServerHost, hit: Applied) {
    let attack = hit.attack;
    let player = attack.attacker;
    let level = attack.level;
    let feet = attack.feet;
    let center = [feet[0], feet[1] + 0.5 * attack.height, feet[2]];
    match attack.swing {
        Swing::Worn => {}
        Swing::Sword { strength } => {
            let (Ok(target), Some(now)) = (u32::try_from(attack.target), level.game_time(host))
            else {
                return;
            };
            with_state(|state| {
                tools::queue_echo(
                    state.echoes.entry(player.uuid()).or_default(),
                    &attack.dimension,
                    target,
                    center,
                    hit.damage,
                    strength,
                    now,
                );
            });
        }
        Swing::Smash { multiplier } => {
            if player.level(host) == Some(level) && player.entity().on_ground(host) == Some(false) {
                let distance = with_state(|state| {
                    state.falls.get_mut(&player.uuid()).map(|fall| {
                        fall.halve();
                        fall.distance()
                    })
                });
                if let Some(distance) = distance {
                    player.entity().set_fall_distance(host, distance);
                }
            }
            for sound in tools::SMASH_SOUNDS {
                level.play_sound(
                    host,
                    sound.id,
                    feet,
                    SoundSource::Players,
                    sound.volume,
                    sound.pitch,
                );
            }
            for burst in tools::smash_bursts(feet, attack.height, multiplier) {
                send_burst(host, level, &burst);
            }
            award(host, player, tools::AWARD_SMASH);
        }
    }
}

fn deliver_echo(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    level: LevelRef,
    echo: tools::PendingEcho,
) {
    if host
        .living(player.entity())
        .is_none_or(|living| living.alive(host) != Some(true))
    {
        return;
    }

    let origin = host
        .entity(echo.target)
        .filter(|target| target.level(host) == Some(level))
        .and_then(|target| center_of(host, target))
        .unwrap_or(echo.origin);
    let min = origin.map(|value| value - tools::RANGE);
    let max = origin.map(|value| value + tools::RANGE);
    let candidates: Vec<_> = level
        .entities_in(host, min, max)
        .unwrap_or_default()
        .into_iter()
        .filter(|entity| {
            entity
                .kind(host)
                .is_some_and(|kind| tools::is_enemy(&kind.id(host)))
        })
        .filter_map(|entity| {
            center_of(host, entity).map(|center| tools::Candidate {
                id: entity.id(),
                center,
            })
        })
        .collect();
    let Some(id) = tools::nearest_victim(origin, echo.target, &candidates) else {
        return;
    };
    let (Some(victim), Some(end)) = (
        host.entity(id),
        candidates
            .iter()
            .find(|candidate| candidate.id == id)
            .map(|candidate| candidate.center),
    ) else {
        return;
    };

    let attacker = player.entity();
    if victim.hurt(
        host,
        tools::ECHO_DAMAGE_TYPE,
        echo.damage,
        Some(attacker),
        Some(attacker),
    ) != Some(true)
    {
        return;
    }
    if let Some(particles) = PARTICLES.get() {
        let dust = ParticleOptions::dust(particles.dust, tools::ECHO_COLOR, 1.0);
        for point in tools::line_points(origin, end) {
            level.send_particles(host, &dust, None, point, [0.0; 3], 0.0, 1);
        }
    }
    send_burst(
        host,
        level,
        &tools::Burst {
            at: end,
            ..tools::ECHO_SPARKS
        },
    );
    let sound = tools::ECHO_SOUND;
    level.play_sound(
        host,
        sound.id,
        end,
        SoundSource::Players,
        sound.volume,
        sound.pitch,
    );
    award(host, player, tools::AWARD_ECHO_STRIKE);
}

fn center_of(host: &dyn ServerHost, entity: EntityRef) -> Option<[f64; 3]> {
    let [x, y, z] = entity.position(host)?;
    let height = entity
        .dimensions(host)
        .map_or(0.0, |[_width, height]| height);
    Some([x, y + height * 0.5, z])
}

fn send_burst(host: &mut dyn ServerHost, level: LevelRef, burst: &tools::Burst) {
    let Some(particles) = PARTICLES.get() else {
        return;
    };
    let kind = match burst.kind {
        tools::BurstKind::Explosion => particles.explosion,
        tools::BurstKind::Cloud => particles.cloud,
        tools::BurstKind::Crit => particles.crit,
        tools::BurstKind::ElectricSpark => particles.electric_spark,
    };
    level.send_particles(
        host,
        &ParticleOptions::simple(kind),
        None,
        burst.at,
        burst.spread,
        burst.speed,
        burst.count,
    );
}

fn award(host: &mut dyn ServerHost, player: PlayerRef, path: &str) {
    let Some(name) = player.name(host) else {
        return;
    };
    if !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        host.commands()
            .run_command(&format!("advancement grant {name} only bmh:{path}"));
    }
}

pub(super) fn item_expire(host: &dyn ServerHost, event: &mut ItemExpireEvent<'_>) {
    event.extra_life = eternal::extra_life(&event.stack.item().id(host), event.extra_life);
}

pub(super) fn item_spawn(host: &mut dyn ServerHost, event: &ItemSpawnEvent<'_>) {
    if ids::is_eternal(event.item)
        && let Some(entity) = host.entity(event.entity)
    {
        entity.set_invulnerable(host, true);
    }
}

pub(super) fn entity_join_level(host: &mut dyn ServerHost, event: &EntityJoinLevelEvent<'_>) {
    if event.kind == "minecraft:item"
        && event.loaded_from_disk
        && let Some(entity) = host.entity(event.entity)
        && let Some(stack) = host.item_entity(entity).and_then(|item| item.stack(host))
        && ids::is_eternal(&stack.item().id(host))
    {
        entity.set_invulnerable(host, true);
    }
}

pub(super) fn anvil_update(host: HostSide, event: &mut AnvilUpdateEvent<'_>) {
    let host: &dyn Host = match host {
        HostSide::Client(host) => host,
        HostSide::Server(host) => host,
    };
    let update = &mut *event.update;
    let Some(right) = update.right.as_ref() else {
        return;
    };
    let left = &update.left;
    if totem::anvil_case(&left.item().id(host), Some(&right.item().id(host)))
        != Some(totem::Anvil::GoldRepair)
    {
        return;
    }
    if update.output.is_none()
        && update.vanilla.output.is_none()
        && update.vanilla.xp_cost >= totem::TOO_EXPENSIVE
        && let Some(repair) = totem::gold_repair(left.damage(), right.count())
    {
        let custom = match component(left, "minecraft:custom_name") {
            Some(Tag::String(name)) => Some(name.to_string_lossy()),
            _ => None,
        };
        let rename = totem::anvil_rename(
            update.name.as_deref(),
            custom.as_deref(),
            component(left, "minecraft:custom_name").is_some(),
        );
        let blocks_penalty = component(right, "minecraft:repair_cost").map_or(0, component_int);
        if !totem::gold_repair_refused(blocks_penalty, repair.blocks, rename != totem::Rename::Keep)
        {
            let mut repaired = left.clone();
            repaired.set_damage(repair.damage);
            match rename {
                totem::Rename::Keep => {}
                totem::Rename::Clear => {
                    repaired
                        .copy_component_from(&ItemStack::of(left.item()), "minecraft:custom_name");
                }
                totem::Rename::Set => {
                    if let Some(name) = update.name.as_deref() {
                        let mut patch = Compound::new();
                        patch.insert("minecraft:custom_name".into(), Tag::String(name.into()));
                        if repaired.apply_components(&patch).is_err() {
                            return;
                        }
                    }
                }
            }
            update.output = Some(repaired);
            update.material_cost = i32::try_from(repair.blocks).unwrap_or(i32::MAX);
        }
    }
    if let Some(output) = update.output.as_mut() {
        output.copy_component_from(left, "minecraft:repair_cost");
        update.xp_cost = totem::GOLD_REPAIR_COST;
    }
}

pub(super) fn break_speed(host: HostSide, event: &mut PlayerBreakSpeedEvent) {
    let (host, player): (&dyn Host, _) = match host {
        HostSide::Server(host) => (host, host.players().acting_player()),
        HostSide::Client(host) => (host, host.local_player()),
    };
    let Some(player) = player.filter(|player| player.entity().id() == event.player) else {
        return;
    };
    let Some(stack) = host
        .living(player.entity())
        .and_then(|living| living.hand_item(host, Hand::Main))
        .filter(worn)
    else {
        return;
    };
    let Some(tag) = tools::tool_of(&stack.item().id(host)).and_then(tools::mineable_tag) else {
        return;
    };
    let mineable = player
        .level(host)
        .and_then(|level| level.block(host, event.pos))
        .and_then(|block| host.block_in_tag(block, tag));
    let efficiency = match component(&stack, "minecraft:enchantments") {
        Some(Tag::Compound(enchantments)) => {
            let levels = match enchantments.get(&"levels".into()) {
                Some(Tag::Compound(levels)) => levels,
                _ => enchantments,
            };
            levels
                .get(&tools::EFFICIENCY.into())
                .map_or(0, component_int)
        }
        _ => 0,
    };
    event.new_speed = tools::worn_break_speed(event.new_speed, mineable, efficiency);
}

fn component<'a>(stack: &'a ItemStack, id: &str) -> Option<&'a Tag> {
    stack.components()?.as_nbt().get(&id.into())
}

fn component_int(tag: &Tag) -> u32 {
    match tag {
        Tag::Int(value) => u32::try_from(*value).unwrap_or_default(),
        _ => 0,
    }
}

fn max_damage(stack: &ItemStack) -> u32 {
    component(stack, "minecraft:max_damage").map_or(tools::MAX_DAMAGE, component_int)
}

fn worn(stack: &ItemStack) -> bool {
    tools::is_worn_out(stack.damage(), max_damage(stack))
}

struct ToolPart;

#[mod_api::horizon_part]
impl ItemPart for ToolPart {
    fn damage_item(&self, _host: &mut dyn ServerHost, event: &mut DamageItemEvent<'_>) -> Flow {
        event.amount =
            tools::cap_damage(event.stack.damage(), event.amount, max_damage(event.stack));
        event.break_now = false;
        Flow::Continue
    }

    fn tooltip(&self, host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let item = event.stack.item().id(host);
        for (offset, line) in
            tools::tooltip_lines(&item, event.stack.damage(), max_damage(event.stack))
                .into_iter()
                .enumerate()
        {
            let index = (1 + offset).min(event.lines.len());
            event
                .lines
                .insert(index, (Text::translate(line.key, vec![]), line.color));
        }
    }

    fn item_bar(&self, _host: &dyn ClientHost, event: &mut ItemBarEvent<'_>) {
        if let Some((width, _)) = event.bar {
            *width = tools::bar_width(*width);
        }
    }
}
