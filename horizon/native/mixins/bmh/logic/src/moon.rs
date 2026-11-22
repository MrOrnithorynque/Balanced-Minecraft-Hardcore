// Area ritual (owner: the ritual agent). Log: docs/horizon/log/ritual.md.
// The moon ritual of the Dormant Totem with no game type: the start rules, the moon direction, the
// aim test, the one-tick decision (fail reasons, level drain, chime), the fail cooldown, the places
// of the totem and of the effects, the rune particles, and the first person pose of the raised totem.
// Java reference: handlers/MoonRitualHandler, items/DormantTotemOfEternityItem, client/RitualHandRenderer,
// client/RitualParticles, handlers/AmuletReading.skyHiddenReason (shared with the amulet).

use super::amulet::{Sky, hidden_reason};
use super::ids;

// ---------------------------------------------------------------------------------------------
// Ids, numbers, texts and sounds
// ---------------------------------------------------------------------------------------------

/// The item that the ritual uses: the Dormant Totem (id kept from 1.8.0, ItemInit.java:74-79).
pub const RITUAL_ITEM: &str = ids::SLEEPING_TOTEM;
/// The item that the ritual makes (MoonRitualHandler.java:203).
pub const WOKEN_ITEM: &str = ids::TOTEM_OF_ETERNITY;

/// The levels that the ritual costs (MoonRitualHandler.java:36).
pub const RITUAL_LEVELS: i32 = 30;
/// The length of the ritual, in ticks (MoonRitualHandler.java:38).
pub const RITUAL_TICKS: i32 = 300;
/// The ritual drains one level each time this number of ticks goes by (MoonRitualHandler.java:40).
pub const TICKS_PER_LEVEL: i32 = RITUAL_TICKS / RITUAL_LEVELS;
/// The largest angle between the look direction and the moon, in degrees (MoonRitualHandler.java:42).
pub const AIM_DEGREES: f64 = 10.0;
/// The pitch of the chime at the first level and at the last one (MoonRitualHandler.java:45-46).
pub const CHIME_LOW_PITCH: f32 = 0.5;
pub const CHIME_HIGH_PITCH: f32 = 1.9;
/// A player who fails must wait this long before the next try (MoonRitualHandler.java:50).
pub const FAIL_COOLDOWN_TICKS: i64 = 20;
/// A hit sets the `hurtTime` of a player to 10 and it counts down once a tick (`LivingEntity.hurt`).
/// `MoonRitualHandler.tick` fails while it is above 0 (MoonRitualHandler.java:126).
pub const HURT_TICKS: u64 = 10;

/// The prefix of the sky messages: `.dimension`, `.rain`, `.day` and `.roof` follow
/// (MoonRitualHandler.java:88).
pub const HIDDEN_PREFIX: &str = "bmh.ritual.hidden";
/// The player does not look at the moon at the start (MoonRitualHandler.java:95).
pub const KEY_AIM: &str = "bmh.ritual.aim";
/// The same reason during the ritual (MoonRitualHandler.java:121).
pub const KEY_LOOKED_AWAY: &str = "bmh.ritual.looked_away";
/// A hit breaks the ritual (MoonRitualHandler.java:127).
pub const KEY_HURT: &str = "bmh.ritual.hurt";
/// No level left to give (MoonRitualHandler.java:136).
pub const KEY_LEVELS: &str = "bmh.ritual.levels";
/// The player lets go too soon (MoonRitualHandler.java:158).
pub const KEY_RELEASED: &str = "bmh.ritual.released";
/// The totem wakes (MoonRitualHandler.java:197).
pub const KEY_WOKEN: &str = "bmh.ritual.woken";

/// The advancement of a woken totem (MoonRitualHandler.java:193).
pub const ADVANCEMENT: &str = "eternity/moon_ritual";

/// The sound category of every sound of the ritual (`SoundSource.PLAYERS`).
pub const SOUND_CATEGORY: &str = "player";
/// A sound with its volume and pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    pub id: &'static str,
    pub volume: f32,
    pub pitch: f32,
}
/// The ritual starts (MoonRitualHandler.java:102-103, `RESPAWN_ANCHOR_CHARGE`).
pub const START_SOUND: Sound = Sound { id: "minecraft:block.respawn_anchor.charge", volume: 0.8, pitch: 1.3 };
/// The ritual fails (MoonRitualHandler.java:177-178, `FIRE_EXTINGUISH`).
pub const FAIL_SOUND: Sound = Sound { id: "minecraft:block.fire.extinguish", volume: 0.7, pitch: 1.4 };
/// The totem wakes (MoonRitualHandler.java:194-195, `AMETHYST_BLOCK_RESONATE`).
pub const WOKEN_SOUND: Sound = Sound { id: "minecraft:block.amethyst_block.resonate", volume: 1.0, pitch: 1.0 };
/// The chime of one level (MoonRitualHandler.java:143-144, `AMETHYST_BLOCK_CHIME`). The pitch rises.
pub const CHIME_ID: &str = "minecraft:block.amethyst_block.chime";
/// The sound of the totem animation that the Java mod starts with entity event 35 (`LivingEntity.handleEntityEvent`).
/// Horizon has no helper for the event (GAPS.md T4): the mod plays the sound and the particles.
pub const ANIMATION_SOUND: Sound = Sound { id: "minecraft:item.totem.use", volume: 1.0, pitch: 1.0 };

/// A sound heard within this many blocks of its place (`Level.playSound` with a volume of 1 or less).
pub const SOUND_RANGE: f64 = 16.0;
/// Particles reach a player within this many blocks (`ServerLevel.sendParticles`, without `force`).
pub const PARTICLE_RANGE: f64 = 32.0;

/// The puff of a failed ritual: 6 particles, spread 0.1, speed 0.02 (MoonRitualHandler.java:171).
pub const PUFF_COUNT: i32 = 6;
pub const PUFF_SPREAD: f64 = 0.1;
pub const PUFF_SPEED: f64 = 0.02;
/// The puff of the player is this far in front of the eyes (MoonRitualHandler.java:166).
pub const PUFF_DISTANCE: f64 = 2.5;

/// The totem animation particles (entity event 35): how many, and how far from the player's centre.
/// `LivingEntity.makeParticle`-style burst of `TOTEM_OF_UNDYING`: 30 particles.
pub const ANIMATION_PARTICLES: i32 = 30;
pub const ANIMATION_SPREAD: f64 = 0.5;
pub const ANIMATION_SPEED: f64 = 0.4;

// ---------------------------------------------------------------------------------------------
// The moon
// ---------------------------------------------------------------------------------------------

/// The length of the day timeline (`timeline/day.json`, `period_ticks`).
pub const DAY_PERIOD: i64 = 24_000;
/// The tick where the moon angle is 180 degrees (`timeline/day.json`, `visual/moon_angle`: both
/// keyframes sit at tick 6000; the first holds 540, the second 180).
const MOON_KEY_TICKS: i64 = 6_000;
const MOON_FROM_DEGREES: f32 = 180.0;
const MOON_TO_DEGREES: f32 = 540.0;
/// The easing of the track: a cubic bezier with these control points (`timeline/day.json`, `ease`).
const MOON_EASE: [f32; 4] = [0.362, 0.241, 0.638, 0.759];

/// Horizon's cubic bezier easing (`horizon-client-render/src/environment.rs`, `Easing::apply`): the curve
/// of `y` over `x` for the control points `(x1, y1)` and `(x2, y2)`. The same steps, so the same values.
fn ease(x: f32) -> f32 {
    let [x1, y1, x2, y2] = MOON_EASE;
    let curve = |c1: f32, c2: f32| (3.0 * c1 - 3.0 * c2 + 1.0, -6.0 * c1 + 3.0 * c2, 3.0 * c1);
    let (ax, bx, cx) = curve(x1, x2);
    let (ay, by, cy) = curve(y1, y2);
    let sample = |(a, b, c): (f32, f32, f32), t: f32| ((a * t + b) * t + c) * t;
    let gradient = |t: f32| (3.0 * ax * t + 2.0 * bx) * t + cx;
    let mut t = x;
    let mut solved = false;
    for _ in 0..4 {
        let error = sample((ax, bx, cx), t) - x;
        if error.abs() < 1e-5 {
            solved = true;
            break;
        }
        let g = gradient(t);
        if g < 1e-5 {
            break;
        }
        t -= (error / g).clamp(-0.25, 0.25);
    }
    if !solved {
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        while t0 < t1 {
            let error = sample((ax, bx, cx), t) - x;
            if error.abs() < 1e-5 {
                break;
            }
            if error < 0.0 {
                t0 = t;
            } else {
                t1 = t;
            }
            let next = (t0 + t1) / 2.0;
            if next == t {
                break;
            }
            t = next;
        }
    }
    sample((ay, by, cy), t)
}

/// The moon angle in degrees at a time of day (`EnvironmentAttributes.MOON_ANGLE`, MoonRitualHandler.java:64).
/// The track runs from 180 degrees at tick 6000 to 540 degrees at tick 6000 of the next day, eased.
pub fn moon_angle(day_time: i64) -> f32 {
    let t = day_time.rem_euclid(DAY_PERIOD);
    // The segment that holds `t`: before the key tick it began one period earlier.
    let from_t = if t < MOON_KEY_TICKS { MOON_KEY_TICKS - DAY_PERIOD } else { MOON_KEY_TICKS };
    if t <= from_t {
        return MOON_FROM_DEGREES;
    }
    let alpha = ease((t - from_t) as f32 / DAY_PERIOD as f32);
    MOON_FROM_DEGREES + (MOON_TO_DEGREES - MOON_FROM_DEGREES) * alpha
}

/// The unit vector from the player toward the moon (MoonRitualHandler.java:62-68): `(-sin a, cos a, 0)`.
pub fn moon_direction(day_time: i64) -> [f64; 3] {
    let angle = f64::from(moon_angle(day_time)).to_radians();
    [-angle.sin(), angle.cos(), 0.0]
}

/// The look direction of an entity (`Entity.calculateViewVector`, from the yaw and pitch in degrees).
pub fn view_vector(yaw: f32, pitch: f32) -> [f64; 3] {
    let f = pitch * (std::f32::consts::PI / 180.0);
    let g = -yaw * (std::f32::consts::PI / 180.0);
    let (h, i) = (g.cos(), g.sin());
    let (j, k) = (f.cos(), f.sin());
    [f64::from(i * j), f64::from(-k), f64::from(h * j)]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// `Vec3.normalize`: a vector that is almost zero stays zero.
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let length = dot(a, a).sqrt();
    if length < 1.0e-4 { [0.0; 3] } else { scale(a, 1.0 / length) }
}

/// True when the look direction is within `AIM_DEGREES` of the moon (MoonRitualHandler.java:72-74).
pub fn aims_at_moon(yaw: f32, pitch: f32, day_time: i64) -> bool {
    dot(view_vector(yaw, pitch), moon_direction(day_time)) >= AIM_DEGREES.to_radians().cos()
}

/// The angle between the look direction and the moon, in degrees, for the log (MoonRitualHandler.java:78-82).
pub fn aim_error(yaw: f32, pitch: f32, day_time: i64) -> f64 {
    dot(view_vector(yaw, pitch), moon_direction(day_time)).clamp(-1.0, 1.0).acos().to_degrees()
}

/// What the aim test reads of the player: where he looks, in degrees, and the time of day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
    pub day_time: i64,
}

/// None when a ritual can start or go on, else the lang key that says why not (MoonRitualHandler.java:86-99).
/// The sky rules come first, in the amulet's order (dimension, rain, day, roof), then the aim.
pub fn blocked_reason(sky: &Sky, look: &Look) -> Option<String> {
    if let Some(hidden) = hidden_reason(sky, HIDDEN_PREFIX) {
        return Some(hidden);
    }
    if !aims_at_moon(look.yaw, look.pitch, look.day_time) {
        return Some(KEY_AIM.to_owned());
    }
    None
}

// ---------------------------------------------------------------------------------------------
// The start, the tick and the cooldown
// ---------------------------------------------------------------------------------------------

/// What a click on the Dormant Totem does (DormantTotemOfEternityItem.java:48-65).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    /// The item is cooling down after a fail: Java never reaches `use`, so there is no message.
    Cooling,
    /// The ritual cannot start: the message says why.
    Refused(String),
    /// The ritual starts.
    Go,
}

/// Decides a start. `blocked` is `blocked_reason`.
pub fn start(cooling: bool, blocked: Option<String>) -> Start {
    if cooling {
        return Start::Cooling;
    }
    match blocked {
        Some(key) => Start::Refused(key),
        None => Start::Go,
    }
}

/// The whole start test, the same for the server and the window (GAPS.md T7): the cooldown, then the sky
/// rules and the aim. Both sides call this one function with what they can read, so they answer the same.
pub fn start_test(cooling: bool, sky: &Sky, look: &Look) -> Start {
    start(cooling, blocked_reason(sky, look))
}

/// True while the item cools down: the fail cooldown ends at the game time `until`.
pub fn cooling(until: Option<i64>, now: i64) -> bool {
    until.is_some_and(|until| now < until)
}

/// The game time where the cooldown of a fail at `now` ends (MoonRitualHandler.java:180).
pub fn cooldown_end(now: i64) -> i64 {
    now.saturating_add(FAIL_COOLDOWN_TICKS)
}

/// True while a hit is recent: the player was hurt less than `HURT_TICKS` ticks ago (`hurtTime > 0`).
pub fn is_hurt(hit_tick: Option<u64>, now: u64) -> bool {
    hit_tick.is_some_and(|hit| now.saturating_sub(hit) < HURT_TICKS)
}

/// What one tick of the ritual reads (MoonRitualHandler.java:115-150).
#[derive(Debug, Clone, PartialEq)]
pub struct TickInput {
    /// `blocked_reason` now.
    pub blocked: Option<String>,
    /// A recent hit (`hurtTime > 0`).
    pub hurt: bool,
    /// The ticks left, 300 down to 1 (`getUseItemRemainingTicks` in `onUseTick`).
    pub remaining: i32,
    /// Creative mode (`getAbilities().instabuild`): no level is drained.
    pub creative: bool,
    /// The experience level of the player.
    pub levels: i32,
}

/// What one tick of the ritual decides.
#[derive(Debug, Clone, PartialEq)]
pub enum Tick {
    /// The ritual fails with this lang key.
    Fail(String),
    /// The ritual goes on. On a level tick the server takes a level (`drain`, not in creative) and plays a
    /// chime of this pitch.
    Go { drain: bool, chime: Option<f32> },
}

/// One tick of the ritual, in Java's order: the blocked reasons (the aim is renamed), the hit, then every
/// `TICKS_PER_LEVEL` ticks the level and the chime.
pub fn tick(input: &TickInput) -> Tick {
    if let Some(reason) = &input.blocked {
        let key = if reason == KEY_AIM { KEY_LOOKED_AWAY } else { reason };
        return Tick::Fail(key.to_owned());
    }
    if input.hurt {
        return Tick::Fail(KEY_HURT.to_owned());
    }
    let elapsed = RITUAL_TICKS.saturating_sub(input.remaining).saturating_add(1);
    if elapsed % TICKS_PER_LEVEL != 0 {
        return Tick::Go { drain: false, chime: None };
    }
    if !input.creative && input.levels < 1 {
        return Tick::Fail(KEY_LEVELS.to_owned());
    }
    // A chime at each level, higher and higher (MoonRitualHandler.java:142).
    let pitch = CHIME_LOW_PITCH + (CHIME_HIGH_PITCH - CHIME_LOW_PITCH) * elapsed as f32 / RITUAL_TICKS as f32;
    Tick::Go { drain: !input.creative, chime: Some(pitch) }
}

// ---------------------------------------------------------------------------------------------
// Places
// ---------------------------------------------------------------------------------------------

/// The eyes of a player: the feet and the eye height.
pub fn eye(feet: [f64; 3], eye_height: f32) -> [f64; 3] {
    [feet[0], feet[1] + f64::from(eye_height), feet[2]]
}

/// About where the totem is in the ritual pose seen from outside (MoonRitualHandler.java:210-219): the
/// shoulder is 0.24 blocks under the eyes and 0.31 blocks to the side, the totem 0.85 blocks along the look.
/// `right_arm` is true when the totem is in the right hand. Horizon has no helper for the main arm, so the
/// caller takes the main hand as the right one.
pub fn raised_totem(eye: [f64; 3], yaw: f32, pitch: f32, right_arm: bool) -> [f64; 3] {
    let look = view_vector(yaw, pitch);
    let radians = f64::from(yaw * (std::f32::consts::PI / 180.0));
    let right = [-radians.cos(), 0.0, -radians.sin()];
    let side = if right_arm { 0.31 } else { -0.31 };
    let shoulder = add(add(eye, [0.0, -0.24, 0.0]), scale(right, side));
    add(shoulder, scale(look, 0.85))
}

/// Where the player sees his own failed puff: in front of the eyes (MoonRitualHandler.java:166).
pub fn puff_position(eye: [f64; 3], yaw: f32, pitch: f32) -> [f64; 3] {
    add(eye, scale(view_vector(yaw, pitch), PUFF_DISTANCE))
}

/// Where the totem is in first person (RitualHandRenderer.java:49-57): just under the crosshair,
/// 0.75 blocks in front of the eyes and 0.1 blocks down in the camera's up direction.
pub fn first_person_totem(eye: [f64; 3], yaw: f32, pitch: f32) -> [f64; 3] {
    let look = view_vector(yaw, pitch);
    let right = cross(look, [0.0, 1.0, 0.0]);
    // Straight up or down, the cross product is zero: any right vector does.
    let right = if dot(right, right) < 1.0e-6 { [1.0, 0.0, 0.0] } else { normalize(right) };
    let up = cross(right, look);
    add(add(eye, scale(look, f64::from(-HOLD_Z))), scale(up, f64::from(HOLD_Y)))
}

/// The squared distance between two places.
pub fn distance_squared(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

// ---------------------------------------------------------------------------------------------
// The runes (RitualParticles.java)
// ---------------------------------------------------------------------------------------------

/// The runes start between these distances from the totem (RitualParticles.java:27-28).
pub const RUNE_MIN_DISTANCE: f64 = 1.2;
pub const RUNE_MAX_DISTANCE: f64 = 2.4;
/// Vanilla's enchanting table particle ends this far below the point it is given (RitualParticles.java:30).
pub const RUNE_DROP: f64 = 1.2;

/// A small random source, so that a test gets exact values. xorshift64 with a multiplier.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// A source from a seed. The state is never 0.
    pub const fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in 0 (included) to 1 (excluded).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A number of the normal distribution (`RandomSource.nextGaussian`), by the Box-Muller method.
    pub fn gaussian(&mut self) -> f64 {
        let u1 = 1.0 - self.uniform();
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// One rune: the particle is given `at` and `offset`. The enchanting table particle starts at `at + offset`
/// and flies to `at`, where it goes out (vanilla `EnchantmentTableParticle`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rune {
    pub at: [f64; 3],
    pub offset: [f64; 3],
}

/// One rune for a totem seen from `camera` (RitualParticles.java:44-69). The start is on the far side of the
/// totem from the camera, so that no rune passes in front of the eyes. The point given is 1.2 blocks above the
/// totem, because the particle ends 1.2 blocks below it.
pub fn rune(totem: [f64; 3], camera: [f64; 3], rng: &mut Rng) -> Rune {
    let away = normalize(sub(totem, camera));
    let mut direction = normalize([rng.gaussian(), rng.gaussian(), rng.gaussian()]);
    // Keep the start on the far side of the totem, as seen from the camera.
    let side = dot(direction, away);
    if side < 0.0 {
        direction = sub(direction, scale(away, 2.0 * side));
    }
    let distance = RUNE_MIN_DISTANCE + rng.uniform() * (RUNE_MAX_DISTANCE - RUNE_MIN_DISTANCE);
    let start = add(totem, scale(direction, distance));
    Rune {
        at: [totem[0], totem[1] + RUNE_DROP, totem[2]],
        offset: [start[0] - totem[0], start[1] - totem[1] - RUNE_DROP, start[2] - totem[2]],
    }
}

// ---------------------------------------------------------------------------------------------
// The first person pose (RitualHandRenderer.java)
// ---------------------------------------------------------------------------------------------

/// Where the totem stops, in the camera's space: x to the right, y up, z toward the camera
/// (RitualHandRenderer.java:25-26).
pub const HOLD_Y: f32 = -0.1;
pub const HOLD_Z: f32 = -0.75;
/// Where it starts: the normal place of an item in the hand (RitualHandRenderer.java:28-30).
pub const START_X: f32 = 0.56;
pub const START_Y: f32 = -0.52;
pub const START_Z: f32 = -0.72;
/// The totem takes this many ticks to rise (RitualHandRenderer.java:31).
pub const RAISE_TICKS: f32 = 10.0;
/// The size of the totem in the Java drawing (RitualHandRenderer.java:32).
pub const SCALE: f32 = 0.16;
/// The tremble at full charge (RitualHandRenderer.java:36-37).
pub const MAX_SHAKE: f32 = 0.012;
pub const MAX_SHAKE_DEGREES: f32 = 7.0;
/// The hover (RitualHandRenderer.java:67).
const HOVER_SPEED: f32 = 0.12;
const HOVER_AMOUNT: f32 = 0.012;

/// The size of a flat item model in Horizon's hand against the Java drawing. Java draws the model with the
/// `fixed` display (scale 0.5 for `item/generated`) at `SCALE`. Horizon draws it with the first person hand
/// display (scale 0.68 for `item/generated`), so the pose scales by this factor. Only the game can confirm it.
pub const HAND_SCALE: f32 = SCALE * 0.5 / 0.68;
/// The turn that brings the flat model face to the camera: the hand display turns it by -90 degrees around
/// the vertical axis. Only the game can confirm the sign.
pub const FACE_YAW: f32 = 90.0;

/// The place of the totem in the camera's space for a time `elapsed` (RitualHandRenderer.java:60-77):
/// the offset, and the roll of the tremble in degrees. `side` is 1 for the right hand and -1 for the left.
/// `elapsed` is the ticks since the start with the part of the tick (`300 - useItemRemaining + partialTick`).
pub fn totem_target(elapsed: f32, side: f32) -> ([f32; 3], f32) {
    let raise = (elapsed / RAISE_TICKS).clamp(0.0, 1.0);
    // Smooth start and smooth stop.
    let raise = raise * raise * (3.0 - 2.0 * raise);
    let time = elapsed;
    let hover = (time * HOVER_SPEED).sin() * HOVER_AMOUNT * raise;
    let charge = (elapsed / RITUAL_TICKS as f32).clamp(0.0, 1.0);
    let shake = charge * charge;
    // Two fast waves of different speeds on each axis: an uneven tremble, not a regular swing.
    let shake_x = ((time * 2.9).sin() + (time * 4.3 + 1.0).sin()) * 0.5 * shake;
    let shake_y = ((time * 3.7 + 2.0).sin() + (time * 5.1).sin()) * 0.5 * shake;
    let lerp = |from: f32, to: f32| from + (to - from) * raise;
    (
        [
            lerp(side * START_X, 0.0) + shake_x * MAX_SHAKE,
            lerp(START_Y, HOLD_Y) + hover + shake_y * MAX_SHAKE,
            lerp(START_Z, HOLD_Z),
        ],
        shake_x * MAX_SHAKE_DEGREES,
    )
}

/// The pose that Horizon's `render_hand_pose` adds to the hand: an offset in blocks, the turns in degrees and
/// a scale factor. Horizon applies `translate * rotate(pitch, yaw, roll) * scale` to the hand, arm and item
/// together, in the camera's space, before the hand's own pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandPose {
    pub offset: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub scale: f32,
}

/// A point turned by the pose without the offset: scale, then roll around z, then yaw around y.
fn turned(point: [f32; 3], yaw: f32, roll: f32, scale: f32) -> [f32; 3] {
    let [x, y, z] = [point[0] * scale, point[1] * scale, point[2] * scale];
    let (sr, cr) = roll.to_radians().sin_cos();
    let (x, y) = (x * cr - y * sr, x * sr + y * cr);
    let (sy, cy) = yaw.to_radians().sin_cos();
    [x * cy + z * sy, y, -x * sy + z * cy]
}

/// The pose that puts the item of the hand where Java draws the totem. The item of a hand sits at
/// `(side * 0.56, -0.52, -0.72)` in the camera's space (`applyItemArmTransform`); the pose shrinks the hand
/// around the camera and turns it, then the offset moves that place onto `totem_target`. The arm shrinks too,
/// so almost nothing of it shows (Java draws the totem alone, GAPS.md T8).
pub fn hand_pose(elapsed: f32, side: f32) -> HandPose {
    let (target, roll) = totem_target(elapsed, side);
    let place = turned([side * START_X, START_Y, START_Z], FACE_YAW, roll, HAND_SCALE);
    HandPose {
        offset: [target[0] - place[0], target[1] - place[1], target[2] - place[2]],
        pitch: 0.0,
        yaw: FACE_YAW,
        roll,
        scale: HAND_SCALE,
    }
}

/// The place where the item of a hand ends up under a pose (for the tests).
pub fn pose_place(pose: &HandPose, side: f32) -> [f32; 3] {
    let moved = turned([side * START_X, START_Y, START_Z], pose.yaw, pose.roll, pose.scale);
    [moved[0] + pose.offset[0], moved[1] + pose.offset[1], moved[2] + pose.offset[2]]
}

/// The ticks the ritual has run, from the ticks left that the last tick of the use told (Java:
/// `RITUAL_TICKS - getUseItemRemainingTicks()`, where the count is already one lower after that tick).
pub fn elapsed_after(remaining: i32) -> i32 {
    RITUAL_TICKS.saturating_sub(remaining).saturating_add(1).clamp(0, RITUAL_TICKS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NIGHT: i64 = 18_000;

    fn close(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    fn night_sky() -> Sky {
        Sky {
            overworld: true,
            rain: 0.0,
            thunder: 0.0,
            day_time: NIGHT,
            sky_light: 15,
            blinded: false,
            eyes_covered: false,
        }
    }

    fn up() -> Look {
        Look { yaw: 0.0, pitch: -90.0, day_time: NIGHT }
    }

    // ---- constants (MoonRitualHandler.java:36-50)

    #[test]
    fn the_numbers_are_the_java_ones() {
        assert_eq!(RITUAL_LEVELS, 30);
        assert_eq!(RITUAL_TICKS, 300);
        assert_eq!(TICKS_PER_LEVEL, 10);
        assert_eq!(FAIL_COOLDOWN_TICKS, 20);
        assert!(close(AIM_DEGREES.to_radians().cos(), 0.984_807_753, 1e-8));
        assert_eq!(RITUAL_ITEM, "bmh:sleeping_totem");
        assert_eq!(WOKEN_ITEM, "bmh:totem_of_eternity");
    }

    // ---- the moon

    #[test]
    fn the_easing_is_the_curve_of_the_timeline() {
        assert!(close(f64::from(ease(0.0)), 0.0, 1e-6));
        assert!(close(f64::from(ease(1.0)), 1.0, 1e-4));
        assert!(close(f64::from(ease(0.5)), 0.5, 1e-4));
        // The curve is odd around the middle.
        for x in [0.1f32, 0.25, 0.4] {
            assert!(close(f64::from(ease(x) + ease(1.0 - x)), 1.0, 1e-3), "{x}");
        }
    }

    #[test]
    fn the_moon_angle_follows_the_old_time_of_day_curve() {
        // Vanilla 1.21 `DimensionType.timeOfDay` (the curve that the 26 timeline replaced) gave the sun angle;
        // the moon is 180 degrees away.
        let old_sun = |t: i64| {
            let x = t as f64 / 24_000.0 - 0.25;
            let d = x - x.floor();
            let e = 0.5 - (d * std::f64::consts::PI).cos() / 2.0;
            (d * 2.0 + e) / 3.0 * 360.0
        };
        for t in (0..24_000).step_by(500) {
            let moon = f64::from(moon_angle(t)).rem_euclid(360.0);
            let expected = (old_sun(t) + 180.0).rem_euclid(360.0);
            let diff = (moon - expected).abs();
            assert!(diff.min(360.0 - diff) < 1.5, "tick {t}: {moon} against {expected}");
        }
    }

    #[test]
    fn the_moon_is_under_the_world_at_noon_and_over_it_at_midnight() {
        assert_eq!(moon_angle(6_000), 180.0);
        assert!(close(f64::from(moon_angle(NIGHT)), 360.0, 1e-3));
        let noon = moon_direction(6_000);
        assert!(close(noon[1], -1.0, 1e-6) && close(noon[0], 0.0, 1e-6));
        let midnight = moon_direction(NIGHT);
        assert!(close(midnight[1], 1.0, 1e-6) && close(midnight[0], 0.0, 1e-6));
    }

    #[test]
    fn the_moon_sets_in_the_west() {
        // At tick 0 (dawn) the moon is low; its x is negative (west), as the sun at dusk.
        let dawn = moon_direction(0);
        assert!(dawn[0] < -0.8 && dawn[1] < 0.0, "{dawn:?}");
        // At tick 12000 (dusk) it rises in the east.
        let dusk = moon_direction(12_000);
        assert!(dusk[0] > 0.8 && dusk[1] < 0.5, "{dusk:?}");
    }

    #[test]
    fn the_moon_direction_is_a_unit_vector_and_the_time_wraps() {
        for t in [0, 1, 5_999, 6_000, 6_001, 12_000, 23_999, 24_000, 100_000, -5] {
            let v = moon_direction(t);
            assert!(close(dot(v, v), 1.0, 1e-6), "{t}");
        }
        assert_eq!(moon_angle(30_000), moon_angle(6_000));
    }

    #[test]
    fn the_view_vector_is_vanillas() {
        let south = view_vector(0.0, 0.0);
        assert!(close(south[2], 1.0, 1e-6) && close(south[0], 0.0, 1e-6));
        let west = view_vector(90.0, 0.0);
        assert!(close(west[0], -1.0, 1e-6));
        let up = view_vector(0.0, -90.0);
        assert!(close(up[1], 1.0, 1e-6));
        let down = view_vector(33.0, 90.0);
        assert!(close(down[1], -1.0, 1e-6));
    }

    #[test]
    fn the_aim_is_ten_degrees_wide() {
        // Midnight: the moon is straight up.
        assert!(aims_at_moon(0.0, -90.0, NIGHT));
        assert!(aims_at_moon(123.0, -81.0, NIGHT));
        assert!(!aims_at_moon(0.0, -79.0, NIGHT));
        assert!(!aims_at_moon(0.0, 0.0, NIGHT));
        assert!(close(aim_error(0.0, -80.0, NIGHT), 10.0, 1e-3));
        assert!(close(aim_error(0.0, 0.0, NIGHT), 90.0, 1e-3));
    }

    // ---- the start

    #[test]
    fn the_sky_comes_before_the_aim() {
        let look = Look { yaw: 0.0, pitch: 0.0, day_time: NIGHT };
        assert_eq!(blocked_reason(&night_sky(), &look), Some("bmh.ritual.aim".into()));
        assert_eq!(blocked_reason(&night_sky(), &up()), None);
        let mut sky = night_sky();
        sky.overworld = false;
        assert_eq!(blocked_reason(&sky, &look), Some("bmh.ritual.hidden.dimension".into()));
        let mut sky = night_sky();
        sky.rain = 0.9;
        assert_eq!(blocked_reason(&sky, &look), Some("bmh.ritual.hidden.rain".into()));
        let mut sky = night_sky();
        sky.day_time = 6_000;
        assert_eq!(blocked_reason(&sky, &up()), Some("bmh.ritual.hidden.day".into()));
        let mut sky = night_sky();
        sky.sky_light = 14;
        assert_eq!(blocked_reason(&sky, &up()), Some("bmh.ritual.hidden.roof".into()));
        let mut sky = night_sky();
        sky.blinded = true;
        assert_eq!(blocked_reason(&sky, &up()), Some("bmh.ritual.hidden.roof".into()));
        let mut sky = night_sky();
        sky.eyes_covered = true;
        assert_eq!(blocked_reason(&sky, &up()), Some("bmh.ritual.hidden.roof".into()));
    }

    #[test]
    fn a_start_is_silent_while_cooling_then_refused_or_allowed() {
        assert_eq!(start(true, Some("x".into())), Start::Cooling);
        assert_eq!(start(true, None), Start::Cooling);
        assert_eq!(start(false, Some("bmh.ritual.aim".into())), Start::Refused("bmh.ritual.aim".into()));
        assert_eq!(start(false, None), Start::Go);
    }

    #[test]
    fn the_start_test_is_the_same_for_both_sides() {
        let sky = night_sky();
        assert_eq!(start_test(false, &sky, &up()), Start::Go);
        let down = Look { yaw: 0.0, pitch: 90.0, day_time: NIGHT };
        assert_eq!(start_test(false, &sky, &down), Start::Refused(KEY_AIM.to_owned()));
        assert_eq!(start_test(true, &sky, &down), Start::Cooling);
        let rain = Sky { rain: 1.0, ..sky };
        assert!(matches!(start_test(false, &rain, &up()), Start::Refused(_)));
        let nether = Sky { overworld: false, ..sky };
        assert!(matches!(start_test(false, &nether, &up()), Start::Refused(_)));
        let roof = Sky { sky_light: 0, ..sky };
        assert!(matches!(start_test(false, &roof, &up()), Start::Refused(_)));
    }

    #[test]
    fn the_fail_cooldown_lasts_twenty_ticks() {
        let end = cooldown_end(1_000);
        assert_eq!(end, 1_020);
        assert!(cooling(Some(end), 1_000));
        assert!(cooling(Some(end), 1_019));
        assert!(!cooling(Some(end), 1_020));
        assert!(!cooling(None, 0));
        assert_eq!(cooldown_end(i64::MAX), i64::MAX);
    }

    // ---- the tick

    fn input(remaining: i32) -> TickInput {
        TickInput { blocked: None, hurt: false, remaining, creative: false, levels: 5 }
    }

    #[test]
    fn a_level_is_taken_every_ten_ticks_with_a_rising_chime() {
        // The first tick of the use has 300 ticks left: elapsed 1.
        assert_eq!(tick(&input(300)), Tick::Go { drain: false, chime: None });
        assert_eq!(tick(&input(292)), Tick::Go { drain: false, chime: None });
        // Elapsed 10.
        let Tick::Go { drain, chime: Some(pitch) } = tick(&input(291)) else { panic!("no chime") };
        assert!(drain);
        assert!(close(f64::from(pitch), 0.5 + 1.4 * 10.0 / 300.0, 1e-6));
        // Elapsed 300, the last tick: the chime is at 1.9.
        let Tick::Go { chime: Some(last), .. } = tick(&input(1)) else { panic!("no chime") };
        assert!(close(f64::from(last), 1.9, 1e-6));
    }

    #[test]
    fn thirty_levels_are_taken_in_a_full_ritual() {
        let drained = (1..=RITUAL_TICKS)
            .rev()
            .filter(|remaining| matches!(tick(&input(*remaining)), Tick::Go { drain: true, .. }))
            .count();
        assert_eq!(drained, 30);
    }

    #[test]
    fn creative_keeps_the_levels_and_still_chimes() {
        let creative = TickInput { creative: true, levels: 0, ..input(291) };
        assert!(matches!(tick(&creative), Tick::Go { drain: false, chime: Some(_) }));
    }

    #[test]
    fn no_level_left_fails_on_a_level_tick_only() {
        let broke = TickInput { levels: 0, ..input(291) };
        assert_eq!(tick(&broke), Tick::Fail("bmh.ritual.levels".into()));
        let between = TickInput { levels: 0, ..input(290) };
        assert_eq!(tick(&between), Tick::Go { drain: false, chime: None });
        let one = TickInput { levels: 1, ..input(291) };
        assert!(matches!(tick(&one), Tick::Go { drain: true, .. }));
    }

    #[test]
    fn the_checks_run_in_javas_order() {
        let all = TickInput {
            blocked: Some("bmh.ritual.aim".into()),
            hurt: true,
            remaining: 291,
            creative: false,
            levels: 0,
        };
        assert_eq!(tick(&all), Tick::Fail("bmh.ritual.looked_away".into()));
        let roof = TickInput { blocked: Some("bmh.ritual.hidden.roof".into()), ..all.clone() };
        assert_eq!(tick(&roof), Tick::Fail("bmh.ritual.hidden.roof".into()));
        let hurt = TickInput { blocked: None, ..all.clone() };
        assert_eq!(tick(&hurt), Tick::Fail("bmh.ritual.hurt".into()));
        let levels = TickInput { blocked: None, hurt: false, ..all };
        assert_eq!(tick(&levels), Tick::Fail("bmh.ritual.levels".into()));
    }

    #[test]
    fn a_hit_counts_for_ten_ticks() {
        assert!(!is_hurt(None, 100));
        assert!(is_hurt(Some(100), 100));
        assert!(is_hurt(Some(100), 109));
        assert!(!is_hurt(Some(100), 110));
        // A tick id that went back (a new world) is no hit.
        assert!(is_hurt(Some(100), 50));
    }

    #[test]
    fn the_elapsed_count_follows_the_ticks_left() {
        assert_eq!(elapsed_after(300), 1);
        assert_eq!(elapsed_after(1), 300);
        assert_eq!(elapsed_after(0), 300);
        assert_eq!(elapsed_after(400), 0);
    }

    // ---- places

    #[test]
    fn the_raised_totem_is_at_the_shoulder_and_along_the_look() {
        // Facing south (yaw 0): the right of the player is west, -x.
        let right = raised_totem([0.0, 1.62, 0.0], 0.0, 0.0, true);
        assert!(close(right[0], -0.31, 1e-6) && close(right[1], 1.62 - 0.24, 1e-6) && close(right[2], 0.85, 1e-6));
        let left = raised_totem([0.0, 1.62, 0.0], 0.0, 0.0, false);
        assert!(close(left[0], 0.31, 1e-6));
        // Looking up: the totem is above the shoulder.
        let high = raised_totem([0.0, 1.62, 0.0], 0.0, -90.0, true);
        assert!(close(high[1], 1.62 - 0.24 + 0.85, 1e-6));
    }

    #[test]
    fn the_puff_of_the_player_is_two_and_a_half_blocks_ahead() {
        let at = puff_position([1.0, 2.0, 3.0], 0.0, 0.0);
        assert!(close(at[0], 1.0, 1e-6) && close(at[1], 2.0, 1e-6) && close(at[2], 5.5, 1e-6));
    }

    #[test]
    fn the_first_person_totem_is_under_the_crosshair() {
        let totem = first_person_totem([0.0, 1.62, 0.0], 0.0, 0.0);
        assert!(close(totem[0], 0.0, 1e-6) && close(totem[1], 1.62 - 0.1, 1e-6) && close(totem[2], 0.75, 1e-6));
        // Straight up or down, the vector stays finite.
        let high = first_person_totem([0.0, 1.62, 0.0], 0.0, -90.0);
        assert!(high.iter().all(|v| v.is_finite()));
        assert!(close(high[1], 1.62 + 0.75, 1e-6));
    }

    // ---- the runes

    #[test]
    fn the_random_source_is_repeatable_and_spread() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut r = Rng::new(0);
        let mut sum = 0.0;
        let mut square = 0.0;
        for _ in 0..4_000 {
            let u = r.uniform();
            assert!((0.0..1.0).contains(&u));
            let g = r.gaussian();
            assert!(g.is_finite());
            sum += g;
            square += g * g;
        }
        assert!((sum / 4_000.0).abs() < 0.1);
        assert!((square / 4_000.0 - 1.0).abs() < 0.15);
    }

    #[test]
    fn a_rune_starts_on_the_far_side_of_the_totem() {
        let totem = [10.0, 70.0, -4.0];
        let camera = [10.0, 71.6, -5.0];
        let away = normalize(sub(totem, camera));
        let mut rng = Rng::new(42);
        for _ in 0..200 {
            let r = rune(totem, camera, &mut rng);
            // The particle is given the point 1.2 above the totem.
            assert_eq!(r.at, [10.0, 71.2, -4.0]);
            // The start is the totem plus a vector of 1.2 to 2.4 blocks.
            let start = [r.offset[0], r.offset[1] + RUNE_DROP, r.offset[2]];
            let length = dot(start, start).sqrt();
            assert!((RUNE_MIN_DISTANCE - 1e-9..=RUNE_MAX_DISTANCE + 1e-9).contains(&length), "{length}");
            assert!(dot(start, away) >= -1e-9);
        }
    }

    #[test]
    fn a_rune_with_the_camera_on_the_totem_still_works() {
        let mut rng = Rng::new(1);
        let r = rune([0.0; 3], [0.0; 3], &mut rng);
        assert!(r.offset.iter().all(|v| v.is_finite()));
    }

    // ---- the pose

    #[test]
    fn the_totem_starts_in_the_hand_and_stops_under_the_crosshair() {
        let (start, roll) = totem_target(0.0, 1.0);
        assert_eq!(start, [0.56, -0.52, -0.72]);
        assert_eq!(roll, 0.0);
        let (left, _) = totem_target(0.0, -1.0);
        assert_eq!(left[0], -0.56);
        // After the raise the totem is centred and 0.75 blocks ahead, within the hover and the tremble.
        let (hold, _) = totem_target(10.0, 1.0);
        assert!(hold[0].abs() < 0.001 && (hold[1] + 0.1).abs() < 0.015 && (hold[2] + 0.75).abs() < 1e-6, "{hold:?}");
        // Half way up the smooth step is at the middle.
        let (mid, _) = totem_target(5.0, 1.0);
        assert!((mid[2] - (START_Z + HOLD_Z) / 2.0).abs() < 1e-6);
    }

    #[test]
    fn the_tremble_grows_with_the_square_of_the_charge() {
        // The widest roll over a stretch of time: a late charge trembles far more than an early one.
        let widest = |from: f32| (0..60).map(|t| totem_target(from + t as f32 * 0.1, 1.0).1.abs()).fold(0.0f32, f32::max);
        let (early, late) = (widest(20.0), widest(290.0));
        assert!(late > early * 10.0, "{early} {late}");
        assert!(late <= MAX_SHAKE_DEGREES);
    }

    #[test]
    fn the_pose_puts_the_item_of_the_hand_where_java_draws_the_totem() {
        for side in [1.0f32, -1.0] {
            for elapsed in [0.0f32, 3.5, 10.0, 77.7, 299.9] {
                let pose = hand_pose(elapsed, side);
                let (target, _) = totem_target(elapsed, side);
                let place = pose_place(&pose, side);
                for axis in 0..3 {
                    assert!((place[axis] - target[axis]).abs() < 1e-5, "{side} {elapsed} {axis}");
                }
                assert_eq!(pose.scale, HAND_SCALE);
                assert_eq!(pose.pitch, 0.0);
                assert!(pose.offset.iter().all(|v| v.is_finite() && v.abs() < 16.0));
            }
        }
    }

    #[test]
    fn the_hand_scale_is_the_ratio_of_the_two_displays() {
        assert!((HAND_SCALE - 0.16 * 0.5 / 0.68).abs() < 1e-7);
        assert!((0.01..=100.0).contains(&HAND_SCALE));
    }

    #[test]
    fn the_animation_and_puff_numbers_are_the_java_ones() {
        assert_eq!((PUFF_COUNT, PUFF_SPREAD, PUFF_SPEED), (6, 0.1, 0.02));
        assert_eq!(START_SOUND, Sound { id: "minecraft:block.respawn_anchor.charge", volume: 0.8, pitch: 1.3 });
        assert_eq!(FAIL_SOUND.volume, 0.7);
        assert_eq!(FAIL_SOUND.pitch, 1.4);
        assert_eq!(WOKEN_SOUND.id, "minecraft:block.amethyst_block.resonate");
        assert_eq!(CHIME_ID, "minecraft:block.amethyst_block.chime");
    }
}
