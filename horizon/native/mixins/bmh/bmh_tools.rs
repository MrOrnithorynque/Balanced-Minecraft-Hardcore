// Area tools: the edge between the game and `bmh_logic::tools`.
// It holds the wear rules and the worn out state of the Eternal tools, the 7 percent death wear, Echo
// Strike, Axe Smash, Echo Sense and the tooltip lines and bar. Log: docs/horizon/log/tools.md.
// Owner: the tools agent. The hook helpers (hook_*.rs) call the functions below.
//
// Where the state lives:
// - `SERVER`: what the server keeps between hooks, by the uuid of the player. Cleaned in
//   `player_logged_out`; the hit, the echoes, the fall and the held item also at `player_died`. Nothing
//   survives a restart: an echo waits 5 ticks and a glow 5 seconds.
// - `CLIENT`: the glows that the server told this window, and the server's mineable answers (E4).
// - `bmh_tools_lib` (horizon-client-lib): the blocks of fall that the smashes took, for `living_fall`.

use bmh_logic::contract::Rule;
use bmh_logic::eternal::PLAYER_SOUND_SOURCE;
use bmh_logic::tools as logic;
use bmh_logic::wire::{TOOLS_CHANNEL, ToolsSignal};
use horizon_client::items::ItemStack;
use horizon_client::mod_api::{ParticleOptions, ParticleType};
use horizon_client::text::Text;
use horizon_world::player_inventory::EquipmentSlot;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

const OVERWORLD: &str = bmh_logic::respawn::OVERWORLD;
/// The off hand, by `native_inventory_tick`'s numbers.
const OFF_HAND_SLOT: i32 = -106;
/// Java tests `GameType.SURVIVAL` only (KeepItemAfterDeath.java:38).
const SURVIVAL: i32 = bmh_logic::eternal::SURVIVAL;
/// The entity type of a player.
const PLAYER: &str = "minecraft:player";
/// The most hostile mobs the server list keeps. A mob that left is dropped at the next sweep of its
/// dimension.
const MAX_MOBS: usize = 8192;
/// Ticks between two sweeps of the glows, and between two sweeps of the mobs of one dimension.
const SWEEP_TICKS: i64 = 100;
/// The ender dragon is no mob of `native_mob`, but an echo may hit it.
const ENDER_DRAGON: &str = "minecraft:ender_dragon";
/// The most mineable answers the server remembers as sent. Past it, no answer is sent: the client then
/// scales the speed, which is the safe side.
const MAX_TOLD: usize = 65_536;
/// A tick of the game lasts 50 ms (the client counts the glow in real time).
const TICK: Duration = Duration::from_millis(50);
/// The most glows a client keeps.
const MAX_CLIENT_GLOWS: usize = 2048;

/// The lock of a static. A panic in another hook leaves the value usable.
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------------------------

/// A hit that the damage hook saw and that `hurt_enemy` confirms (the hit landed).
#[derive(Clone, Copy)]
enum Hit {
    /// A full or weak swing of the sword: the damage it deals and the strength of the swing.
    Sword { victim: i32, damage: f32, strength: f32 },
    /// A smash of the axe: the multiplier it applied.
    Smash { victim: i32, multiplier: f32 },
}

/// The fall of a player, with the entity id the lib needs.
struct Falling {
    entity: u32,
    fall: logic::Fall,
}

/// A reading of Echo Sense that waits for the end of the tick, when the players can be listed.
struct Reading {
    dimension: String,
    center: [i32; 3],
    positions: Vec<[i32; 3]>,
}

struct Server {
    /// The last two resets of the attack strength of each player.
    resets: BTreeMap<u128, logic::Resets>,
    /// The fall of each player.
    falls: BTreeMap<u128, Falling>,
    /// The hit that the damage hook saw and `hurt_enemy` did not confirm yet.
    hits: BTreeMap<u128, Hit>,
    /// The game time of the last main hit of each player: a second damage call in one tick is a sweep.
    main_hit: BTreeMap<u128, i64>,
    /// The echoes that wait for each player.
    echoes: BTreeMap<u128, Vec<logic::PendingEcho>>,
    /// The hostile mobs that joined a level: entity id and dimension.
    mobs: BTreeMap<u32, String>,
    /// The glows of Echo Sense.
    glows: logic::GlowBook,
    /// The game time when each player may read an ore again.
    next_sense: BTreeMap<u128, i64>,
    /// The readings to send at the end of the tick.
    readings: Vec<Reading>,
    /// Advancements to award where no player is swapped in: player, name, path.
    awards: Vec<(u128, String, &'static str)>,
    /// The game time of the last sweep of the glows.
    swept: i64,
    /// The game time of the last sweep of the mobs, by dimension.
    mob_sweeps: BTreeMap<String, i64>,
    /// The main-hand item of each player at its last tick (None: an empty hand).
    held: BTreeMap<u128, Option<String>>,
    /// The mineable answers that each player was sent (E4): each one goes once.
    told: BTreeSet<(u128, logic::Tool, String)>,
    /// The mineable answers to send at the end of the tick.
    answers: Vec<(u128, ToolsSignal)>,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    resets: BTreeMap::new(),
    falls: BTreeMap::new(),
    hits: BTreeMap::new(),
    main_hit: BTreeMap::new(),
    echoes: BTreeMap::new(),
    mobs: BTreeMap::new(),
    glows: logic::GlowBook::new(),
    next_sense: BTreeMap::new(),
    readings: Vec::new(),
    awards: Vec::new(),
    swept: 0,
    mob_sweeps: BTreeMap::new(),
    held: BTreeMap::new(),
    told: BTreeSet::new(),
    answers: Vec::new(),
});

fn server() -> MutexGuard<'static, Server> {
    lock(&SERVER)
}

/// True while an echo hurts its victim: the damage hooks hear that hurt too, and an echo must not echo.
static ECHOING: AtomicBool = AtomicBool::new(false);

/// A glow that the server told this window.
struct ClientGlow {
    pos: [i32; 3],
    /// The block at the position when the glow came: the glow ends when it changes (the ore was mined).
    /// The window cannot read the ore tag, so a change of the block stands for "no ore any more".
    block: String,
    until: Instant,
}

struct Client {
    /// The blocks that glow.
    glows: Vec<ClientGlow>,
    /// Whether a block is in the mineable tag of a tool, as the server read it (E4).
    mineable: logic::MineableAnswers,
}

static CLIENT: Mutex<Client> =
    Mutex::new(Client { glows: Vec::new(), mineable: logic::MineableAnswers::new() });

fn client() -> MutexGuard<'static, Client> {
    lock(&CLIENT)
}

/// The game time of the server (`Level.getGameTime`, shared by every level).
fn game_time(game: &crate::game::Game<'_>) -> Option<i64> {
    let level = game.native_level(OVERWORLD)?;
    game.native_level_game_time(level)
}

/// Queues an advancement: `run_command` works only where no player is swapped in.
fn queue_award(player: u128, name: &str, path: &'static str) {
    server().awards.push((player, name.to_owned(), path));
}

/// The centre of the box of an entity and its height (Java: `position + (0, bbHeight / 2, 0)`).
fn center_of(game: &crate::game::Game<'_>, id: u32) -> Option<([f64; 3], f64)> {
    let entity = game.native_entity(id)?;
    let at = game.native_entity_position(entity)?;
    let height = game
        .native_entity_bounding_box(entity)
        .map_or(0.0, |[low, high]| (high[1] - low[1]).max(0.0));
    Some(([at[0], at[1] + height * 0.5, at[2]], height))
}

// ---------------------------------------------------------------------------------------------
// C12: the 7 percent death wear
// ---------------------------------------------------------------------------------------------

// C12: take the 7 percent death wear from each Eternal tool in the drops (KeepItemAfterDeath.java:36-52).
// This runs before the eternal area takes the stacks out of `drops` (HOOKS.md, `loot_drops`).
pub(super) fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    // A player's own death: the entity is the player and no loot table rolled.
    let Some((_, PLAYER)) = context.entity else {
        return;
    };
    let Some(player) = context.player.filter(|_| context.table.is_empty()) else {
        return;
    };
    if !drops.iter().any(|stack| stack.count > 0 && logic::tool_of(&stack.item).is_some()) {
        return;
    }
    // The dying player's state is swapped in. Where it is not, the game mode is taken as survival.
    let survival = game
        .native_acting_player()
        .filter(|me| me.uuid == player)
        .is_none_or(|me| me.game_mode == SURVIVAL);
    if !survival || !crate::game::bmh_settings::rule(game, Rule::DamageEternalItemWhenDead) {
        return;
    }
    for stack in drops.iter_mut() {
        if stack.count > 0 && logic::tool_of(&stack.item).is_some() {
            stack.damage = logic::death_wear(stack.damage);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// E2: the tools never break
// ---------------------------------------------------------------------------------------------

// E2: cap the damage so an Eternal tool keeps its last point (EternalWear.java:48-55, L_ItemStackWearMixin).
pub(super) fn on_item_damaged(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    item: &str,
    amount: &mut i32,
    cancel: &mut bool,
) {
    let _ = (player, name, cancel);
    if logic::tool_of(item).is_none() {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    // The tool that wears is the one in the main hand, else the one in the off hand.
    let slots = [i32::from(me.selected_slot), OFF_HAND_SLOT];
    let damage = slots.into_iter().find_map(|slot| {
        game.native_carried_stack(slot)
            .filter(|stack| stack.count > 0 && &*stack.item == item)
            .map(|stack| stack.damage)
    });
    if let Some(damage) = damage {
        *amount = logic::cap_damage(damage, *amount);
    }
}

// E2: a tool past its last point (an older version of the mod) goes back to it.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = selected;
    let Some(stack) = game.native_carried_stack(slot) else {
        return;
    };
    if stack.count == 0 || logic::tool_of(&stack.item).is_none() {
        return;
    }
    stack.damage = logic::restore_last_point(stack.damage);
}

// ---------------------------------------------------------------------------------------------
// E3, E7, E8: the hit
// ---------------------------------------------------------------------------------------------

// E3: halve the hit of a worn out tool. E8: multiply the hit of an Axe Smash. E7: note a sword hit for the
// echo that `hurt_enemy` confirms (E_PlayerAttackMixin.java:62-95).
pub(super) fn on_living_incoming_damage_v2(
    game: &mut crate::game::Game<'_>,
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    source_position: Option<[f64; 3]>,
    victim_position: [f64; 3],
    victim_view: [f64; 3],
    weapon: Option<&ItemStack>,
    amount: &mut f32,
    canceled: &mut bool,
) {
    let _ = (kind, damage_type, source_position, victim_position, victim_view);
    if *canceled || ECHOING.load(Ordering::Relaxed) {
        return;
    }
    let Some(weapon) = weapon else {
        return;
    };
    let Some(tool) = logic::tool_of(&weapon.item) else {
        return;
    };
    let Some(me) = game.native_acting_player().filter(|me| me.entity_id == causing) else {
        return;
    };
    let now = game_time(game).unwrap_or_default();
    let mut s = server();
    // Java scales the one hit on the main target. A sweep hits the others in the same tick, after it.
    if s.main_hit.get(&me.uuid) == Some(&now) {
        return;
    }
    s.main_hit.insert(me.uuid, now);
    s.hits.remove(&me.uuid);
    if logic::is_worn_out(weapon.damage) {
        *amount *= logic::WORN_DAMAGE_FACTOR;
        return;
    }
    match tool {
        logic::Tool::Axe => {
            let distance = s.falls.get(&me.uuid).map_or(0.0, |falling| falling.fall.distance());
            let multiplier = logic::smash_multiplier(distance);
            *amount *= multiplier;
            if multiplier > 1.0 {
                s.hits.insert(me.uuid, Hit::Smash { victim: entity, multiplier });
            }
        }
        logic::Tool::Sword => {
            let strength = s
                .resets
                .get(&me.uuid)
                .copied()
                .unwrap_or_default()
                .strength_at(now, logic::SWORD_ATTACK_SPEED);
            s.hits.insert(me.uuid, Hit::Sword { victim: entity, damage: *amount, strength });
        }
        logic::Tool::Pickaxe => {}
    }
}

// E7, E8: the hit landed. The sword queues its echo, the axe makes the impact felt (E_PlayerAttackMixin.java:85-95).
pub(super) fn on_hurt_enemy(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: u32,
    kind: &str,
) {
    let _ = kind;
    if logic::tool_of(&stack.item).is_none() {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let Ok(victim) = i32::try_from(entity) else {
        return;
    };
    let hit = {
        let mut s = server();
        match s.hits.get(&me.uuid).copied() {
            Some(Hit::Sword { victim: v, .. } | Hit::Smash { victim: v, .. }) if v == victim => {
                s.hits.remove(&me.uuid)
            }
            _ => None,
        }
    };
    let Some(hit) = hit else {
        return;
    };
    let Some((center, height)) = center_of(game, entity) else {
        return;
    };
    match hit {
        Hit::Sword { damage, strength, .. } => {
            let now = game_time(game).unwrap_or_default();
            let mut s = server();
            let queue = s.echoes.entry(me.uuid).or_default();
            logic::queue_echo(queue, &me.dimension, entity, center, damage, strength, now);
        }
        Hit::Smash { multiplier, .. } => smash(game, &me, center, height, multiplier),
    }
}

// E8: the effects of a smash that landed (AxeSmashHandler.java:57-82).
fn smash(
    game: &mut crate::game::Game<'_>,
    me: &horizon_client::mod_api::NativeServerPlayer,
    center: [f64; 3],
    height: f64,
    multiplier: f32,
) {
    log::debug!(
        "[BalancedMcHardcoreMain] Smash: {} hits x{multiplier}",
        me.name
    );
    queue_award(me.uuid, &me.name, logic::AWARD_SMASH);
    // Half the fall distance, about half the fall damage: the landing loses what the smash took.
    let removed = server().falls.get_mut(&me.uuid).map_or(0.0, |falling| falling.fall.halve());
    if let Ok(player) = u32::try_from(me.entity_id) {
        horizon_client_lib::bmh_tools_lib::add_softening(player, removed);
    }
    // The sounds and particles are at the target: its feet (the centre less half its height).
    let feet = [center[0], center[1] - height * 0.5, center[2]];
    for sound in logic::SMASH_SOUNDS {
        game.native_play_sound_in_place(sound.id, feet, PLAYER_SOUND_SOURCE, sound.volume, sound.pitch);
    }
    for burst in logic::smash_bursts(feet, height, multiplier) {
        send_burst(game, &burst);
    }
}

/// Sends one burst of vanilla particles to the players near it.
fn send_burst(game: &mut crate::game::Game<'_>, burst: &logic::Burst) {
    let kind = match burst.kind {
        logic::BurstKind::Explosion => ParticleType::Explosion,
        logic::BurstKind::Cloud => ParticleType::Cloud,
        logic::BurstKind::Crit => ParticleType::Crit,
        logic::BurstKind::ElectricSpark => ParticleType::ElectricSpark,
    };
    game.native_send_particles_in_place(
        &ParticleOptions::simple(kind),
        None,
        burst.at,
        burst.spread,
        burst.speed,
        burst.count,
    );
}

// E7: estimate the attack strength of the next hit. The punch packet resets it after the attack's own reset,
// so the estimate keeps the last two resets of the player.
pub(super) fn on_player_attack_cooldown_reset(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) {
    let _ = name;
    let now = game_time(game).unwrap_or_default();
    server().resets.entry(player).or_default().reset(now);
}

// E7: keep the list of mobs that an echo can hit.
pub(super) fn on_entity_join_level(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    entity: u32,
    kind: &str,
    pos: [f64; 3],
    loaded_from_disk: bool,
    canceled: &mut bool,
) {
    let _ = (game, pos, loaded_from_disk, canceled);
    if !logic::is_enemy(kind) {
        return;
    }
    let mut s = server();
    if s.mobs.len() >= MAX_MOBS {
        // The oldest ids go first.
        if let Some(oldest) = s.mobs.keys().next().copied() {
            s.mobs.remove(&oldest);
        }
    }
    s.mobs.insert(entity, dimension.to_owned());
}

// ---------------------------------------------------------------------------------------------
// E7, E8: the player's tick
// ---------------------------------------------------------------------------------------------

// E7: land the echoes that are due, at the head of the tick of their player (EchoStrikeHandler.java:91-113).
// E8: follow the fall of the player (the fall distance is the mod's: no helper reads it).
// E7: a new item in the main hand resets the attack strength (Player.tick, `resetAttackStrengthTicker`).
// The mobs of the player's dimension that left are dropped here, where that level is in place.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    deliver_due_echoes(game, &me);
    track_fall(game, &me);
    note_held_item(game, &me);
    sweep_mobs(game, &me.dimension);
}

// E7: Java compares the main-hand stack with the one of the last tick and resets the attack strength
// when the item differs (`ItemStack.isSameItem`). A player with no entry had an empty hand, as Java's
// `lastItemInMainHand` starts empty.
fn note_held_item(game: &crate::game::Game<'_>, me: &horizon_client::mod_api::NativeServerPlayer) {
    let held = game.native_held().map(|(item, _, _)| item);
    let Some(now) = game_time(game) else {
        return;
    };
    let mut s = server();
    let before = s.held.insert(me.uuid, held.clone()).flatten();
    if before != held {
        s.resets.entry(me.uuid).or_default().reset(now);
    }
}

// E7: every 100 ticks, drop the mobs of this dimension that are gone. `native_entity` reads the level in
// place only, and in a player's tick that level is the player's dimension.
fn sweep_mobs(game: &crate::game::Game<'_>, dimension: &str) {
    let Some(now) = game_time(game) else {
        return;
    };
    let ids: Vec<u32> = {
        let mut s = server();
        let last = s.mob_sweeps.get(dimension).copied().unwrap_or(i64::MIN);
        if now >= last && now.saturating_sub(last) < SWEEP_TICKS {
            return;
        }
        s.mob_sweeps.insert(dimension.to_owned(), now);
        s.mobs.iter().filter(|(_, d)| d.as_str() == dimension).map(|(id, _)| *id).collect()
    };
    let gone: Vec<u32> = ids.into_iter().filter(|id| game.native_entity(*id).is_none()).collect();
    let mut s = server();
    for id in gone {
        s.mobs.remove(&id);
    }
}

fn deliver_due_echoes(game: &mut crate::game::Game<'_>, me: &horizon_client::mod_api::NativeServerPlayer) {
    if server().echoes.get(&me.uuid).is_none_or(|queue| queue.is_empty()) {
        return;
    }
    let Some(now) = game_time(game) else {
        return;
    };
    let due = match server().echoes.get_mut(&me.uuid) {
        Some(queue) => logic::take_due(queue, now),
        None => return,
    };
    for echo in due {
        // The attacker may have changed dimension since the hit: the echo is lost with it.
        if echo.dimension == me.dimension {
            deliver(game, me, &echo);
        }
    }
}

// E7: one echo (EchoStrikeHandler.java:115-140).
fn deliver(
    game: &mut crate::game::Game<'_>,
    me: &horizon_client::mod_api::NativeServerPlayer,
    echo: &logic::PendingEcho,
) {
    // The target may have died or been knocked away: follow it while it lives.
    let origin = center_of(game, echo.target).map_or(echo.origin, |(center, _)| center);
    let ids: Vec<u32> = server()
        .mobs
        .iter()
        .filter(|(_, dimension)| **dimension == me.dimension)
        .map(|(id, _)| *id)
        .collect();
    // Java skips a mob that is not alive (EchoStrikeHandler.java:170). A mob in its death animation is
    // still in the level, and `native_hurt_entity` refuses it: leave it out.
    let candidates: Vec<logic::Candidate> = ids
        .into_iter()
        .filter(|id| is_alive(game, &me.dimension, *id))
        .filter_map(|id| center_of(game, id).map(|(center, _)| logic::Candidate { id, center }))
        .collect();
    let Some(victim) = logic::nearest_victim(origin, echo.target, &candidates) else {
        return;
    };
    let Some(end) = candidates.iter().find(|c| c.id == victim).map(|c| c.center) else {
        return;
    };
    let Ok(attacker) = u32::try_from(me.entity_id) else {
        return;
    };
    // The hurt of an echo is heard by the damage hooks: this flag keeps it from echoing again.
    ECHOING.store(true, Ordering::Relaxed);
    let hurt = game.native_hurt_entity(&me.dimension, victim, echo.damage, logic::ECHO_DAMAGE_TYPE, Some(attacker));
    ECHOING.store(false, Ordering::Relaxed);
    // Invulnerability frames can absorb the echo: then there is nothing to show either.
    if !hurt {
        return;
    }
    let dust = ParticleOptions::dust(logic::ECHO_COLOR, 1.0);
    for point in logic::line_points(origin, end) {
        game.native_send_particles_in_place(&dust, None, point, [0.0; 3], 0.0, 1);
    }
    send_burst(game, &logic::Burst { at: end, ..logic::ECHO_SPARKS });
    queue_award(me.uuid, &me.name, logic::AWARD_ECHO_STRIKE);
    let sound = logic::ECHO_SOUND;
    game.native_play_sound_in_place(sound.id, end, PLAYER_SOUND_SOURCE, sound.volume, sound.pitch);
    log::debug!(
        "[BalancedMcHardcoreMain] Echo Strike: {} echoes {} onto {victim} ({} blocks away)",
        me.name,
        echo.damage,
        logic::distance(origin, end)
    );
}

/// True when the hostile mob `id` of the level in place lives. The ender dragon is no mob of
/// `native_mob`: it counts while the level holds it.
fn is_alive(game: &crate::game::Game<'_>, dimension: &str, id: u32) -> bool {
    if game.native_mob(dimension, id).is_some() {
        return true;
    }
    game.native_entity(id).and_then(|e| game.native_entity_kind(e)).as_deref() == Some(ENDER_DRAGON)
}

// E8: one tick of the fall of the player.
fn track_fall(game: &mut crate::game::Game<'_>, me: &horizon_client::mod_api::NativeServerPlayer) {
    let Ok(id) = u32::try_from(me.entity_id) else {
        return;
    };
    let entity = game.native_entity(id);
    let in_water = entity.and_then(|e| game.native_entity_is_in_water(e)).unwrap_or(false);
    let gliding = entity.and_then(|e| game.native_entity_pose(e)) == Some("fall_flying");
    let feet = me.position.map(|v| v.floor() as i32);
    let climbing = !me.on_ground && !in_water && {
        let block = game.native_block(feet);
        game.native_block_name_in_tag(&block, "minecraft:climbable").unwrap_or(false)
    };
    let floating = game
        .native_effects(me.uuid)
        .is_some_and(|effects| effects.iter().any(|e| logic::FLOATING_EFFECTS.contains(&e.id.as_str())));
    let input = logic::FallInput {
        y: me.position[1],
        on_ground: me.on_ground,
        in_water,
        climbing,
        gliding,
        flying: me.flying,
        floating,
    };
    let distance = {
        let mut s = server();
        let falling = s
            .falls
            .entry(me.uuid)
            .or_insert_with(|| Falling { entity: id, fall: logic::Fall::default() });
        falling.entity = id;
        falling.fall.step(input)
    };
    // The landing hook has run: a fall that ended takes the smashes' share with it.
    if distance == 0.0 {
        horizon_client_lib::bmh_tools_lib::clear_softening(id);
    }
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, name);
    forget_body(player);
    let mut s = server();
    s.next_sense.remove(&player);
    s.awards.retain(|(who, _, _)| *who != player);
    s.told.retain(|(who, _, _)| *who != player);
    s.answers.retain(|(who, _)| *who != player);
}

// State: Java keeps the echo queue, the attack strength, the last main-hand item and the fall on the
// ServerPlayer object, which a respawn replaces (EchoStrikeQueue.java:7). A death ends them here.
pub(super) fn on_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    let _ = (game, cause);
    forget_body(player);
}

/// Removes what lives and dies with the player's body: the hits, the echoes, the attack strength, the
/// main-hand item and the fall.
fn forget_body(player: u128) {
    let mut s = server();
    s.resets.remove(&player);
    if let Some(falling) = s.falls.remove(&player) {
        horizon_client_lib::bmh_tools_lib::clear_softening(falling.entity);
    }
    s.hits.remove(&player);
    s.main_hit.remove(&player);
    s.echoes.remove(&player);
    s.held.remove(&player);
}

// ---------------------------------------------------------------------------------------------
// The end of the server tick: readings, awards, sweeps
// ---------------------------------------------------------------------------------------------

// E9: send the readings. Awards: the advancements of the abilities. A sweep drops what is gone.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = context;
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    send_readings(game);
    send_answers(game);
    flush_awards(game);
    sweep_glows(game);
}

// E4: send each player the mineable answers that its dig asked for this tick.
fn send_answers(game: &mut crate::game::Game<'_>) {
    let answers = std::mem::take(&mut server().answers);
    for (player, signal) in answers {
        if let Err(error) = game.native_net_send_to_player(player, TOOLS_CHANNEL, &signal.encode()) {
            log::debug!("[BalancedMcHardcoreMain] Mineable answer not sent: {error}");
        }
    }
}

// E9: tell the players near an ore which blocks glow.
fn send_readings(game: &mut crate::game::Game<'_>) {
    let readings = std::mem::take(&mut server().readings);
    if readings.is_empty() {
        return;
    }
    let online = game.native_server_players();
    for reading in readings {
        let message = ToolsSignal::Glow {
            ticks: u32::try_from(logic::LIFETIME_TICKS).unwrap_or_default(),
            positions: reading.positions,
        }
        .encode();
        for player in &online {
            if !logic::sees_glow(&player.dimension, player.position, &reading.dimension, reading.center) {
                continue;
            }
            if let Err(error) = game.native_net_send_to_player(player.uuid, TOOLS_CHANNEL, &message) {
                log::debug!("[BalancedMcHardcoreMain] Echo Sense glow not sent: {error}");
            }
        }
    }
}

// The advancements of the abilities. `run_command` works here: no player is swapped in.
fn flush_awards(game: &mut crate::game::Game<'_>) {
    let awards = std::mem::take(&mut server().awards);
    for (player, name, path) in awards {
        let id = format!("bmh:{path}");
        let done = game
            .native_player_advancements(player)
            .and_then(|all| all.get(&id).and_then(|a| a.get("done")).and_then(|d| d.as_bool()))
            .unwrap_or(false);
        if !done && !crate::game::bmh::award(game, &name, path) {
            log::warn!("[BalancedMcHardcoreMain] Advancement {id} not awarded to {name}");
        }
    }
}

// Every 100 ticks: drop the glows whose time is up. The mobs are swept in each player's tick, where the
// level of that player is in place: at the end of the server tick the level in place is not defined.
fn sweep_glows(game: &mut crate::game::Game<'_>) {
    let Some(now) = game_time(game) else {
        return;
    };
    let mut s = server();
    if now.saturating_sub(s.swept) < SWEEP_TICKS && now >= s.swept {
        return;
    }
    s.swept = now;
    s.glows.expire(now);
}

// ---------------------------------------------------------------------------------------------
// E9: Echo Sense
// ---------------------------------------------------------------------------------------------

// E9: Echo Sense finds the ores near a block that an Eternal Pickaxe breaks (E_ServerPlayerGameModeMixin.java:44-74).
// The hook runs before the break, so the broken block itself is left out of the search.
pub(super) fn on_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    canceled: &mut bool,
) {
    let _ = state;
    if *canceled {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let tool = game
        .native_carried_stack(i32::from(me.selected_slot))
        .filter(|stack| stack.count > 0)
        .map(|stack| (logic::tool_of(&stack.item), stack.damage));
    // The Eternal Pickaxe, not worn out.
    let Some((Some(logic::Tool::Pickaxe), damage)) = tool else {
        return;
    };
    if logic::is_worn_out(damage) {
        return;
    }
    let broken = game.native_block(pos);
    if !in_tag(game, &broken, logic::ORE_TAG) {
        return;
    }
    let Some(now) = game_time(game) else {
        return;
    };
    {
        let mut s = server();
        if now < s.next_sense.get(&me.uuid).copied().unwrap_or_default() {
            return;
        }
        s.next_sense.insert(me.uuid, now + logic::COOLDOWN_TICKS);
    }
    // A glow whose ore is gone ends at once, so it does not count in the budget.
    let near = server().glows.near(&me.dimension, pos);
    let standing: Vec<[i32; 3]> =
        near.into_iter().filter(|glow| in_tag(game, &game.native_block(*glow), logic::ORE_TAG)).collect();
    let found = logic::find_same_ore(
        pos,
        |at| {
            // Never load a chunk for a glow: a block that is not loaded reads as air.
            let name = game.native_block(at);
            (name != "minecraft:air").then_some(name)
        },
        |name| logic::same_ore(&broken, name, |block, tag| in_tag(game, block, tag)),
    );
    let reading = {
        let mut s = server();
        s.glows.forget_where(&me.dimension, pos, |glow| standing.contains(&glow));
        logic::read_ores(&mut s.glows, &me.dimension, pos, &found, now)
    };
    log::debug!(
        "[BalancedMcHardcoreMain] Echo Sense: {} lit {} ores of the same kind, {} already shining.",
        me.name,
        reading.spawned.len(),
        reading.extended.len()
    );
    if !reading.spawned.is_empty() {
        queue_award(me.uuid, &me.name, logic::AWARD_ECHO_SENSE);
        let sound = logic::SENSE_SOUND;
        let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5];
        game.native_play_sound_in_place(sound.id, at, PLAYER_SOUND_SOURCE, sound.volume, sound.pitch);
    }
    let positions = reading.positions();
    if !positions.is_empty() {
        server().readings.push(Reading { dimension: me.dimension, center: pos, positions });
    }
}

/// True when the server's tags put `block` in `tag` (`BlockState.is(TagKey)`). Where the tags cannot be
/// read, the written copy of the tags of Echo Sense answers.
fn in_tag(game: &crate::game::Game<'_>, block: &str, tag: &str) -> bool {
    game.native_block_name_in_tag(block, tag).unwrap_or_else(|_| logic::listed_in_tag(block, tag))
}

// E9: read the ore positions to glow on `bmh:tools`. E4: keep the mineable answers of the server.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != TOOLS_CHANNEL {
        return;
    }
    match ToolsSignal::decode(data) {
        Some(ToolsSignal::Glow { ticks, positions }) => glow(game, ticks, positions),
        Some(ToolsSignal::Mineable { tool, block, mineable }) => {
            client().mineable.set(tool, &block, mineable);
        }
        None => {}
    }
}

// E9: the glows of a reading. The block of each position is read now: the glow ends when it changes.
fn glow(game: &crate::game::Game<'_>, ticks: u32, positions: Vec<[i32; 3]>) {
    let until = Instant::now() + TICK * ticks;
    // A block that reads as air is mined already, or not loaded in this window: it does not glow.
    let blocks: Vec<([i32; 3], String)> = positions
        .into_iter()
        .map(|pos| (pos, game.native_block(pos)))
        .filter(|(_, block)| block != "minecraft:air")
        .collect();
    let mut c = client();
    for (pos, block) in blocks {
        match c.glows.iter_mut().find(|glow| glow.pos == pos) {
            Some(glow) => {
                glow.until = until;
                glow.block = block;
            }
            None => c.glows.push(ClientGlow { pos, block, until }),
        }
    }
    if c.glows.len() > MAX_CLIENT_GLOWS {
        let extra = c.glows.len() - MAX_CLIENT_GLOWS;
        c.glows.drain(..extra);
    }
}

// E9: draw the boxes of the glowing ores through walls.
pub(super) fn on_world_primitives(
    game: &crate::game::Game<'_>,
    partial: f32,
    camera: [f64; 3],
    primitives: &mut horizon_client::world_primitives::WorldPrimitives,
) {
    let _ = partial;
    let live: Vec<([i32; 3], String)> = {
        let mut c = client();
        let now = Instant::now();
        c.glows.retain(|glow| glow.until > now);
        c.glows.iter().map(|glow| (glow.pos, glow.block.clone())).collect()
    };
    if live.is_empty() {
        return;
    }
    // An ore that was mined stops glowing (EchoSenseHandler.java:164-168). The window cannot read the ore
    // tag: a block that is no longer the one the glow came with stands for it.
    let gone: Vec<[i32; 3]> =
        live.iter().filter(|(pos, block)| game.native_block(*pos) != *block).map(|(pos, _)| *pos).collect();
    if !gone.is_empty() {
        client().glows.retain(|glow| !gone.contains(&glow.pos));
    }
    let live: Vec<[i32; 3]> = live.into_iter().map(|(pos, _)| pos).collect();
    for pos in live.iter().filter(|pos| !gone.contains(pos)) {
        let (min, max) = logic::glow_box(*pos);
        let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5, (min[2] + max[2]) * 0.5];
        if logic::distance(centre, camera) > logic::GLOW_VIEW_DISTANCE {
            continue;
        }
        primitives.box_outline(min, max, logic::GLOW_ARGB, logic::GLOW_LINE_WIDTH).through_walls();
    }
}

// ---------------------------------------------------------------------------------------------
// E4: the dig speed of a worn out tool
// ---------------------------------------------------------------------------------------------

// E4: a worn out axe or pickaxe mines at the speed of a stone tool (EternalAxeItem.java:48-53). The hook runs
// on the server for each guest and in the client's own prediction. Both sides read the main-hand stack
// itself (`native_equipment`: the acting player's on a server, the window's own on a client). The server
// reads the tag; a client cannot, so the server sends it each answer it reads (`ToolsSignal::Mineable`).
// A client with no answer yet scales the speed: it digs slower than its server, never faster, so the
// server never sends the block back.
pub(super) fn on_break_speed(game: &crate::game::Game<'_>, pos: [i32; 3], speed: &mut f32) {
    let Some(stack) = game.native_equipment(EquipmentSlot::MainHand) else {
        return;
    };
    let Some(tool) = logic::tool_of(&stack.item) else {
        return;
    };
    let Some(tag) = logic::mineable_tag(tool) else {
        return;
    };
    let block = game.native_block(pos);
    let mineable = match game.native_block_name_in_tag(&block, tag) {
        Ok(mineable) => {
            // A fresh tool also tells the client, so that it knows the block before the tool wears out.
            tell_client(game, tool, &block, mineable);
            Some(mineable)
        }
        Err(_) => client().mineable.get(tool, &block),
    };
    if !logic::is_worn_out(stack.damage) {
        return;
    }
    *speed = logic::worn_break_speed(*speed, mineable, stack.enchantment_level(logic::EFFICIENCY));
}

/// Queues the mineable answer for the player that digs, once per player, tool and block.
fn tell_client(game: &crate::game::Game<'_>, tool: logic::Tool, block: &str, mineable: bool) {
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let mut s = server();
    let key = (me.uuid, tool, block.to_owned());
    if s.told.contains(&key) || s.told.len() >= MAX_TOLD {
        return;
    }
    s.told.insert(key);
    s.answers.push((me.uuid, ToolsSignal::Mineable { tool, block: block.to_owned(), mineable }));
}

// ---------------------------------------------------------------------------------------------
// E5: tooltip and bar
// ---------------------------------------------------------------------------------------------

// E5: the ability, worn out, heavy and death wear lines, after the name (EternalTooltips.java).
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    let _ = game;
    let added = logic::tooltip_lines(&stack.item, stack.damage);
    if added.is_empty() {
        return;
    }
    let at = lines.len().min(1);
    let added = added
        .into_iter()
        .map(|line| (Text::translate(line.key, &[]).with_color(line.color), u32::MAX));
    lines.splice(at..at, added);
}

// E5: the 1 pixel durability bar of an Eternal tool (EternalSwordItem.java:43-45).
pub(super) fn on_item_bar(
    game: &crate::game::Game<'_>,
    stack: &ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    let _ = game;
    if logic::tool_of(&stack.item).is_none() {
        return;
    }
    if let Some((width, _)) = bar {
        *width = logic::bar_width(*width);
    }
}
