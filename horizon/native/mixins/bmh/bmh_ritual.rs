// Area ritual: the edge between the game and `bmh_logic::moon`.
// It holds the moon ritual that wakes the Dormant Totem: the start rules, the tick that drains the levels
// or fails, the release, the wake, the rune particles and the first person look of the raised totem.
// Every decision is in `bmh_logic::moon`; this file reads the game and carries out the answers.
// Owner: the ritual agent. Log: docs/horizon/log/ritual.md.
//
// How it works:
// - The item is a "use over time" item: `use_duration` answers 300, so a right click starts a use on both
//   sides and consumes the click (T15).
// - The server decides. The start hook checks the start rules. The tick hook runs Java's `tick` and, on a fail,
//   sets `canceled`: Horizon then finishes the use, and the finish hook sees that the ritual is over and leaves
//   the Dormant Totem as it is. The release hook is the "let go too soon" fail.
// - The client only draws. It follows the use through the same hooks and moves the first person hand in
//   `render_hand_pose`. It never decides.
// - A hook of the use fires inside a player's turn, where `native_server_players` is empty and `run_command`
//   refuses. So the sounds, the particles and the advancement wait in `SERVER.effects` and go out at the end
//   of the server tick (one tick late, GAPS.md T9).

use crate::game::bmh_amulet::{sky, sky_at};
use bmh_logic::moon as logic;
use horizon_client::first_person::Hand;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::{NativeLifecycleContext, NativeServerPlayer, ServerTickPhase};
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_client::text::Text;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

const OVERWORLD: &str = bmh_logic::respawn::OVERWORLD;
/// The client keeps the pose while the last tick of the use is younger than this. A use that ends with no hook
/// (the hotbar changes, the server ends it) stops the pose after this time.
const POSE_FRESH: Duration = Duration::from_millis(400);
/// A ritual that no tick of the use touched for this many server ticks is over (a hotbar change ends a use
/// with no hook, as in the Java mod).
const STALE_TICKS: u64 = 3;
/// `GameType` number of creative mode in `NativeServerPlayer.game_mode`.
const CREATIVE: i32 = 1;

/// The lock of a static. A panic in another hook leaves the value usable.
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------------------------
// Server state
// ---------------------------------------------------------------------------------------------

/// A ritual that runs.
#[derive(Clone)]
struct Active {
    /// The totem is in the main hand, which the mod takes as the right one (Horizon has no helper for the
    /// main arm).
    main_hand: bool,
    /// The server tick of the last tick of the use.
    seen: u64,
}

/// Something to show or play at the end of the server tick.
enum Effect {
    /// A sound for the players near its place.
    Sound { dimension: String, at: [f64; 3], sound: &'static str, volume: f32, pitch: f32 },
    /// The puff of a fail: `own` for the player, `others` for the rest.
    Puff { dimension: String, player: u128, own: [f64; 3], others: [f64; 3] },
    /// The wake: the totem animation, which Horizon cannot show (GAPS.md T4), as a sound and particles.
    Woken { dimension: String, at: [f64; 3] },
    /// The advancement of the wake.
    Award { name: String },
}

/// What the server side keeps between hooks.
struct Server {
    /// The server tick now (`NativeLifecycleContext.tick`).
    tick: u64,
    /// The rituals that run, by player.
    active: BTreeMap<u128, Active>,
    /// The game time where the cooldown of a fail ends, by player (GAPS.md T7: the mod's own timer).
    cooling: BTreeMap<u128, i64>,
    /// The server tick of the last hit on a player, by entity id (Java: `hurtTime`).
    hits: BTreeMap<i32, u64>,
    /// What waits for the end of the tick.
    effects: Vec<Effect>,
    /// The source of the runes.
    rng: logic::Rng,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    tick: 0,
    active: BTreeMap::new(),
    cooling: BTreeMap::new(),
    hits: BTreeMap::new(),
    effects: Vec::new(),
    rng: logic::Rng::new(0x5EED_C0DE_0000_0001),
});

fn server() -> MutexGuard<'static, Server> {
    lock(&SERVER)
}

// ---------------------------------------------------------------------------------------------
// Client state
// ---------------------------------------------------------------------------------------------

/// The use of the Dormant Totem that this window follows.
struct ClientUse {
    hand: Hand,
    /// The ticks since the start (`logic::elapsed_after`).
    elapsed: i32,
    /// When the last tick of the use came.
    seen: Instant,
}

struct Client {
    using: Option<ClientUse>,
}

static CLIENT: Mutex<Client> = Mutex::new(Client { using: None });

fn client() -> MutexGuard<'static, Client> {
    lock(&CLIENT)
}

// ---------------------------------------------------------------------------------------------
// Reading the game
// ---------------------------------------------------------------------------------------------

/// True for the stack of the ritual item.
fn is_ritual(stack: &ItemStack) -> bool {
    &*stack.item == logic::RITUAL_ITEM
}

/// The game time of the server (`Level.getGameTime`, shared by every level).
fn game_time(game: &crate::game::Game<'_>) -> Option<i64> {
    let level = game.native_level(OVERWORLD)?;
    game.native_level_game_time(level)
}

/// The start test as the window can compute it, from the camera of the last frame (GAPS.md T7). None when
/// no frame exists yet: the window then lets the start through and the server decides.
fn window_start(game: &crate::game::Game<'_>, cooling: bool) -> Option<logic::Start> {
    let camera = game.native_camera()?;
    let sky = sky_at(game, &camera.dimension, camera.position, false);
    let look = logic::Look { yaw: camera.yaw, pitch: camera.pitch, day_time: game.native_day_time() };
    Some(logic::start_test(cooling, &sky, &look))
}

/// The start rules and the aim now (MoonRitualHandler.java:86-99).
fn blocked(game: &crate::game::Game<'_>, me: &NativeServerPlayer) -> Option<String> {
    let look = logic::Look { yaw: me.yaw, pitch: me.pitch, day_time: game.native_day_time() };
    logic::blocked_reason(&sky(game, me), &look)
}

/// A line over the player's hotbar (`sendOverlayMessage`). It also reaches the acting player inside a hook.
fn tell(game: &mut crate::game::Game<'_>, player: u128, key: &str) {
    game.native_action_bar(player, Text::translate(key, &[]));
}

// ---------------------------------------------------------------------------------------------
// The ritual on the server
// ---------------------------------------------------------------------------------------------

/// The ritual starts (MoonRitualHandler.java:102-111). The sound waits for the end of the tick.
fn begin(game: &crate::game::Game<'_>, me: &NativeServerPlayer, main_hand: bool) {
    {
        let mut s = server();
        let tick = s.tick;
        s.active.insert(me.uuid, Active { main_hand, seen: tick });
        s.effects.push(Effect::Sound {
            dimension: me.dimension.clone(),
            at: me.position,
            sound: logic::START_SOUND.id,
            volume: logic::START_SOUND.volume,
            pitch: logic::START_SOUND.pitch,
        });
    }
    let day = game.native_day_time();
    log::info!(
        "[BalancedMcHardcoreMain] Moon ritual started by {}: aimed {:.1} degrees from the moon, {} levels",
        me.name,
        logic::aim_error(me.yaw, me.pitch, day),
        me.xp_level
    );
}

/// The ritual fails (MoonRitualHandler.java:162-182): the totem stays dormant and the levels already taken stay
/// gone. A puff and a fizzle for the players near, the message for the player, and the cooldown.
fn fail(game: &mut crate::game::Game<'_>, me: &NativeServerPlayer, key: &str, main_hand: bool) {
    let eye = logic::eye(me.position, me.eye_height);
    let now = game_time(game).unwrap_or(0);
    {
        let mut s = server();
        s.active.remove(&me.uuid);
        s.cooling.insert(me.uuid, logic::cooldown_end(now));
        s.effects.push(Effect::Puff {
            dimension: me.dimension.clone(),
            player: me.uuid,
            own: logic::puff_position(eye, me.yaw, me.pitch),
            others: logic::raised_totem(eye, me.yaw, me.pitch, main_hand),
        });
        s.effects.push(Effect::Sound {
            dimension: me.dimension.clone(),
            at: me.position,
            sound: logic::FAIL_SOUND.id,
            volume: logic::FAIL_SOUND.volume,
            pitch: logic::FAIL_SOUND.pitch,
        });
    }
    tell(game, me.uuid, key);
    log::info!(
        "[BalancedMcHardcoreMain] Moon ritual failed for {}: {}, aimed {:.1} degrees from the moon, {} levels left",
        me.name,
        key,
        logic::aim_error(me.yaw, me.pitch, game.native_day_time()),
        me.xp_level
    );
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, name);
    let mut s = server();
    s.active.remove(&player);
    s.cooling.remove(&player);
}

// T7: the damage that breaks a ritual (MoonRitualHandler.java:126). The hook has no `Game`, so this notes
// the hit by entity id in a map that only the gated hooks read. It runs after the respawn area, which may
// cancel the hit during the immunity; Java sets `hurtTime` only for a hit that lands.
pub(super) fn on_living_incoming_damage(entity: i32, kind: &str, amount: f32, canceled: bool) {
    if kind != "minecraft:player" || canceled || amount.is_nan() || amount <= 0.0 {
        return;
    }
    let mut s = server();
    let tick = s.tick;
    s.hits.insert(entity, tick);
}

// T9: send the ritual runes to the viewers. Also the end of the tick for everything that waits: the sounds,
// the puffs, the wake effects, the advancement. The tick id is noted at both phases.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &NativeLifecycleContext,
    phase: ServerTickPhase,
) {
    let tick = context.tick.get();
    match phase {
        ServerTickPhase::ServerTickStart { .. } => {
            server().tick = tick;
            return;
        }
        ServerTickPhase::ServerTickEnd { .. } => {}
    }
    let (effects, running) = {
        let mut s = server();
        s.tick = tick;
        s.hits.retain(|_, hit| logic::is_hurt(Some(*hit), tick));
        s.active.retain(|_, a| tick.saturating_sub(a.seen) <= STALE_TICKS);
        let running: Vec<(u128, Active)> = s.active.iter().map(|(who, a)| (*who, a.clone())).collect();
        (std::mem::take(&mut s.effects), running)
    };
    if effects.is_empty() && running.is_empty() {
        return;
    }
    let online = game.native_server_players();
    for effect in effects {
        match effect {
            Effect::Sound { dimension, at, sound, volume, pitch } => {
                hear(game, &online, &dimension, at, sound, volume, pitch);
            }
            Effect::Puff { dimension, player, own, others } => {
                let puff = ParticleOptions::simple(ParticleType::Poof);
                for viewer in online.iter().filter(|p| p.dimension == dimension) {
                    let at = if viewer.uuid == player { own } else { others };
                    game.native_send_particles_to(
                        viewer.uuid,
                        &puff,
                        None,
                        at,
                        [logic::PUFF_SPREAD; 3],
                        logic::PUFF_SPEED,
                        logic::PUFF_COUNT,
                        false,
                    );
                }
            }
            Effect::Woken { dimension, at } => {
                let sound = logic::ANIMATION_SOUND;
                hear(game, &online, &dimension, at, sound.id, sound.volume, sound.pitch);
                let burst = ParticleOptions::simple(ParticleType::TotemOfUndying);
                let centre = [at[0], at[1] + 1.0, at[2]];
                for viewer in online.iter().filter(|p| p.dimension == dimension) {
                    game.native_send_particles_to(
                        viewer.uuid,
                        &burst,
                        None,
                        centre,
                        [logic::ANIMATION_SPREAD; 3],
                        logic::ANIMATION_SPEED,
                        logic::ANIMATION_PARTICLES,
                        false,
                    );
                }
            }
            Effect::Award { name } => {
                if !crate::game::bmh::award(game, &name, logic::ADVANCEMENT) {
                    log::warn!("[BalancedMcHardcoreMain] Advancement bmh:{} not awarded to {name}", logic::ADVANCEMENT);
                }
            }
        }
    }
    runes(game, &online, &running);
}

/// A sound for every player within `SOUND_RANGE` of its place, in its level (`Level.playSound(null, ...)`).
fn hear(
    game: &mut crate::game::Game<'_>,
    online: &[NativeServerPlayer],
    dimension: &str,
    at: [f64; 3],
    sound: &str,
    volume: f32,
    pitch: f32,
) {
    let range = logic::SOUND_RANGE * logic::SOUND_RANGE;
    for listener in online.iter().filter(|p| p.dimension == dimension) {
        if logic::distance_squared(listener.position, at) <= range {
            game.native_play_sound_to(listener.uuid, sound, logic::SOUND_CATEGORY, Some(at), volume, pitch);
        }
    }
}

/// One rune a tick for every ritual and every viewer (RitualParticles.java). The player sees the totem where
/// the first person view draws it, the others where the raised hand holds it. The server cannot know a third
/// person camera, so the player always gets the first person place.
fn runes(game: &mut crate::game::Game<'_>, online: &[NativeServerPlayer], running: &[(u128, Active)]) {
    let enchant = ParticleOptions::simple(ParticleType::Enchant);
    let range = logic::PARTICLE_RANGE * logic::PARTICLE_RANGE;
    for (who, active) in running {
        let Some(owner) = online.iter().find(|p| p.uuid == *who && p.health > 0.0) else {
            continue;
        };
        let eye = logic::eye(owner.position, owner.eye_height);
        for viewer in online.iter().filter(|p| p.dimension == owner.dimension) {
            let totem = if viewer.uuid == owner.uuid {
                logic::first_person_totem(eye, owner.yaw, owner.pitch)
            } else {
                logic::raised_totem(eye, owner.yaw, owner.pitch, active.main_hand)
            };
            if logic::distance_squared(totem, viewer.position) > range {
                continue;
            }
            let camera = logic::eye(viewer.position, viewer.eye_height);
            let rune = {
                let mut s = server();
                logic::rune(totem, camera, &mut s.rng)
            };
            // A count of 0 sends one directional particle: it is given the offset as its motion, so the
            // enchanting table particle starts at `at + offset` and flies to `at`.
            game.native_send_particles_to(viewer.uuid, &enchant, None, rune.at, rune.offset, 1.0, 0, false);
        }
    }
}

// T7, T15: the use time of the Dormant Totem, so that a click starts a use and consumes the click
// (DormantTotemOfEternityItem.java:74-76). Both sides give the same answer, from the stack alone.
pub(super) fn on_use_duration(
    game: &crate::game::Game<'_>,
    stack: &ItemStack,
    duration: &mut i32,
) {
    let _ = game;
    if is_ritual(stack) {
        *duration = logic::RITUAL_TICKS;
    }
}

// T7: check the start rules of the ritual (DormantTotemOfEternityItem.java:48-65). The server decides: it
// sends the reason to the player and cancels the start. The window lets the start through (it predicts, as
// Horizon's client does, GAPS.md T7) and notes the use for the first person pose.
pub(super) fn on_living_entity_use_item_start(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    hand: Hand,
    duration: &mut i32,
    canceled: &mut bool,
) {
    let _ = duration;
    if !is_ritual(stack) || entity.is_some() {
        return;
    }
    if game.native_window() {
        // The same start test as the server's. A refused start leaves no client use, so no raised totem
        // stays while the use key is held. The cooldown comes from the item's `use_cooldown` (the host
        // skips its prediction then); the mod's own timer is only on the server.
        if window_start(game, false).is_some_and(|start| start != logic::Start::Go) {
            *canceled = true;
            return;
        }
        client().using = Some(ClientUse { hand, elapsed: 0, seen: Instant::now() });
        return;
    }
    let Some(me) = game.native_acting_player() else {
        // Without the player nothing can be checked: the ritual does not start.
        *canceled = true;
        log::warn!("[BalancedMcHardcoreMain] A Dormant Totem use started with no acting player: refused");
        return;
    };
    let now = game_time(game).unwrap_or(0);
    let until = server().cooling.get(&me.uuid).copied();
    let look = logic::Look { yaw: me.yaw, pitch: me.pitch, day_time: game.native_day_time() };
    match logic::start_test(logic::cooling(until, now), &sky(game, &me), &look) {
        logic::Start::Cooling => *canceled = true,
        logic::Start::Refused(key) => {
            *canceled = true;
            tell(game, me.uuid, &key);
        }
        logic::Start::Go => begin(game, &me, hand == Hand::Main),
    }
}

// T7: one tick of the ritual on the server (MoonRitualHandler.java:115-150): the fail reasons, the hit, the
// level and the chime. A fail sets `canceled`: Horizon then finishes the use, and the finish hook leaves the
// totem dormant. The window has nothing to do here.
pub(super) fn on_living_entity_use_item_tick(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    hand: Hand,
    duration: &mut i32,
    canceled: &mut bool,
) {
    if !is_ritual(stack) || entity.is_some() || game.native_window() {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        *canceled = true;
        return;
    };
    let (known, hurt, main_hand) = {
        let mut s = server();
        let tick = s.tick;
        let hurt = logic::is_hurt(s.hits.get(&me.entity_id).copied(), tick);
        match s.active.get_mut(&me.uuid) {
            Some(active) => {
                active.seen = tick;
                (true, hurt, active.main_hand)
            }
            None => (false, hurt, hand == Hand::Main),
        }
    };
    if !known {
        // A use that the start hook did not note: it is not a ritual of this mod. It ends with no wake.
        *canceled = true;
        return;
    }
    let input = logic::TickInput {
        blocked: blocked(game, &me),
        hurt,
        remaining: *duration,
        creative: me.game_mode == CREATIVE,
        levels: me.xp_level,
    };
    match logic::tick(&input) {
        logic::Tick::Fail(key) => {
            fail(game, &me, &key, main_hand);
            *canceled = true;
        }
        logic::Tick::Go { drain, chime } => {
            if drain && !game.native_give_levels(me.uuid, -1) {
                // The level could not be taken: the ritual cannot go on for free.
                fail(game, &me, logic::KEY_LEVELS, main_hand);
                *canceled = true;
                return;
            }
            if let Some(pitch) = chime {
                server().effects.push(Effect::Sound {
                    dimension: me.dimension.clone(),
                    at: me.position,
                    sound: logic::CHIME_ID,
                    volume: 1.0,
                    pitch,
                });
            }
        }
    }
}

// T7: the use stops before its end. The window forgets the use. The server does nothing here: the release
// hook that follows is the fail.
pub(super) fn on_living_entity_use_item_stop(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    hand: Hand,
    duration: i32,
    canceled: &mut bool,
) {
    let _ = (hand, duration, canceled);
    if is_ritual(stack) && entity.is_none() && game.native_window() {
        client().using = None;
    }
}

// T7: a finished ritual wakes the totem (MoonRitualHandler.java:188-204). A ritual that failed in its tick is
// over (its record is gone): the result stays the Dormant Totem. The advancement and the effects go out at
// the end of the tick.
pub(super) fn on_living_entity_use_item_finish(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    hand: Hand,
    duration: i32,
    result: &mut Option<ItemStack>,
) {
    let _ = (hand, duration);
    if !is_ritual(stack) || entity.is_some() {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    if server().active.remove(&me.uuid).is_none() {
        return;
    }
    *result = Some(ItemStack::new(logic::WOKEN_ITEM, 1));
    tell(game, me.uuid, logic::KEY_WOKEN);
    {
        let mut s = server();
        s.effects.push(Effect::Award { name: me.name.clone() });
        s.effects.push(Effect::Sound {
            dimension: me.dimension.clone(),
            at: me.position,
            sound: logic::WOKEN_SOUND.id,
            volume: logic::WOKEN_SOUND.volume,
            pitch: logic::WOKEN_SOUND.pitch,
        });
        s.effects.push(Effect::Woken { dimension: me.dimension.clone(), at: me.position });
    }
    log::info!(
        "[BalancedMcHardcoreMain] Moon ritual done: the totem of {} is awake, aimed {:.1} degrees from the moon, {} levels left",
        me.name,
        logic::aim_error(me.yaw, me.pitch, game.native_day_time()),
        me.xp_level
    );
}

// T7: the window follows the ticks of the use for the first person pose. The server does its work in the
// tick hook, because only that hook can end the use.
pub(super) fn on_on_use_tick(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    remaining: i32,
) {
    if !is_ritual(stack) || entity.is_some() || !game.native_window() {
        return;
    }
    if let Some(using) = client().using.as_mut() {
        using.elapsed = logic::elapsed_after(remaining);
        using.seen = Instant::now();
    }
}

// T7: the release of the ritual item (DormantTotemOfEternityItem.java:116-123): a ritual that the player
// lets go fails. `used` starts the item's use cooldown too, where the content mod gives the item one.
pub(super) fn on_release_using(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    entity: Option<i32>,
    time_left: i32,
    used: &mut bool,
) {
    let _ = time_left;
    if !is_ritual(stack) || entity.is_some() {
        return;
    }
    if game.native_window() {
        client().using = None;
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let Some(main_hand) = server().active.get(&me.uuid).map(|a| a.main_hand) else {
        return;
    };
    fail(game, &me, logic::KEY_RELEASED, main_hand);
    *used = true;
}

// T8: raise the totem toward the moon and shake it in first person (RitualHandRenderer.java:59-80). The
// pose moves the arm and the item together and shrinks them, so the arm almost vanishes: the item stands
// where Java draws the totem. The light stays the hand's (GAPS.md T8: no hook sets it).
pub(super) fn on_render_hand_pose(
    game: &crate::game::Game<'_>,
    hand: Hand,
    stack: Option<&ItemStack>,
    partial: f32,
    swing: f32,
    equip: f32,
    x: &mut f32,
    y: &mut f32,
    z: &mut f32,
    pitch: &mut f32,
    yaw: &mut f32,
    roll: &mut f32,
    scale: &mut f32,
) {
    let _ = (game, swing, equip);
    if !stack.is_some_and(is_ritual) {
        return;
    }
    let ticks = {
        let c = client();
        match &c.using {
            Some(using) if using.hand == hand && using.seen.elapsed() < POSE_FRESH => using.elapsed,
            _ => return,
        }
    };
    let elapsed = ticks as f32 + partial.clamp(0.0, 1.0);
    // The main hand is the right one (Horizon has no helper for the main arm).
    let side = if hand == Hand::Main { 1.0 } else { -1.0 };
    let pose = logic::hand_pose(elapsed, side);
    *x += pose.offset[0];
    *y += pose.offset[1];
    *z += pose.offset[2];
    *pitch += pose.pitch;
    *yaw += pose.yaw;
    *roll += pose.roll;
    *scale *= pose.scale;
}
