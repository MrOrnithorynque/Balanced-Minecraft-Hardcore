// Area tools (owner: the tools agent): the wear rules and the worn out state of the Eternal tools, the
// 7 percent death wear, the tooltip lines and the bar, Echo Strike, Axe Smash, Echo Sense and the break
// speed of a worn out tool. Pure values: no game type. The module `bmh_tools` carries the answers out.
// Java reference: handlers/EternalWear, EchoStrikeHandler, AxeSmashHandler, EchoSenseHandler,
// items/EternalTooltips, items/Eternal*Item, mixin/E_*, mixin/L_*.

use crate::ids::{ETERNAL_AXE, ETERNAL_PICKAXE, ETERNAL_SWORD};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------------------------
// The tools and their wear (E2, E3, C12)
// ---------------------------------------------------------------------------------------------

/// The three Eternal tools (Java `EternalWear.isEternalTool`, EternalWear.java:32-37). The Eternal
/// Bundle and the totems are Eternal Items but no tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tool {
    Sword,
    Axe,
    Pickaxe,
}

/// The tool that an item id names.
pub fn tool_of(item: &str) -> Option<Tool> {
    match item {
        ETERNAL_SWORD => Some(Tool::Sword),
        ETERNAL_AXE => Some(Tool::Axe),
        ETERNAL_PICKAXE => Some(Tool::Pickaxe),
        _ => None,
    }
}

/// The durability of an Eternal tool: 33000 uses (BMHModTierInit.java:21, `ToolMaterial.durability`).
pub const MAX_DAMAGE: u32 = 33_000;

/// The share of the durability that a death removes (KeepItemAfterDeath.java:45, `reduceDurability(stack, 7)`).
pub const DEATH_WEAR_PERCENT: u32 = 7;

/// A worn out tool hits for this share of the damage (EternalWear.java:21).
pub const WORN_DAMAGE_FACTOR: f32 = 0.5;

/// A worn out tool mines at most at the speed of a stone tool (EternalWear.java:24).
pub const WORN_MINING_SPEED: f32 = 4.0;

/// The mining speed of the Eternal tools on the blocks they are made for (BMHModTierInit.java:21).
pub const TOOL_MINING_SPEED: f32 = 15.0;

/// `EternalWear.isWornOut`: the tool is at its last point of durability (EternalWear.java:39-42,
/// `getDamageValue() >= getMaxDamage() - 1`).
pub fn is_worn_out(damage: u32, max_damage: u32) -> bool {
    damage >= max_damage.saturating_sub(1)
}

/// `EternalWear.capDamage`: the wear that a use may add, never more than what leaves the last point
/// (EternalWear.java:48-55). A tool at or past its last point gets 0 or less, which leaves it whole.
pub fn cap_damage(damage: u32, amount: i32, max_damage: u32) -> i32 {
    let room = i64::from(max_damage.saturating_sub(1)) - i64::from(damage);
    let room = i32::try_from(room).unwrap_or(i32::MIN);
    amount.min(room)
}

/// `EternalWear.wear`: removes `percentage` of the durability, down to the last point
/// (EternalWear.java:58-67). None when the percentage is outside 0 to 100 (Java throws).
pub fn wear(damage: u32, percentage: u32, max_damage: u32) -> Option<u32> {
    if percentage > 100 {
        return None;
    }
    // Java: `(int) (getMaxDamage() * (percentage / 100.0))`, in double.
    let removed = (f64::from(max_damage) * (f64::from(percentage) / 100.0)) as u32;
    Some(damage.saturating_add(removed).min(max_damage.saturating_sub(1)))
}

/// C12: the damage of an Eternal tool after one death (KeepItemAfterDeath.java:41-46).
pub fn death_wear(damage: u32, max_damage: u32) -> u32 {
    // 7 is inside 0 to 100, so `wear` always answers.
    wear(damage, DEATH_WEAR_PERCENT, max_damage).unwrap_or(damage)
}

/// E2: the damage that a tool at or past its last point goes back to. Java reaches the same state at
/// the next wear: `capDamage` gives a negative amount, and `hurtAndBreak` adds it (EternalWear.java:45-47).
pub fn restore_last_point(damage: u32, max_damage: u32) -> u32 {
    damage.min(max_damage.saturating_sub(1))
}

// ---------------------------------------------------------------------------------------------
// The tooltip and the bar (E5)
// ---------------------------------------------------------------------------------------------

/// The colour of a ChatFormatting.DARK_AQUA line.
pub const DARK_AQUA: u32 = 0x00AAAA;
/// The colour of a ChatFormatting.RED line.
pub const RED: u32 = 0xFF5555;
/// The colour of a ChatFormatting.GOLD line.
pub const GOLD: u32 = 0xFFAA00;
/// The colour of a ChatFormatting.GRAY line.
pub const GRAY: u32 = 0xAAAAAA;

/// The lang key of the worn out line (EternalTooltips.java:33).
pub const KEY_WORN_OUT: &str = "tooltip.bmh.worn_out";
/// The lang key of the heavy line (EternalTooltips.java:23).
pub const KEY_HEAVY: &str = "tooltip.bmh.heavy_with_eternity";
/// The lang key of the death wear line (EternalTooltips.java:45).
pub const KEY_DEATH_WEAR: &str = "tooltip.bmh.death_wear";

/// The lang key of the ability line of a tool (EternalSwordItem.java:29, EternalAxeItem.java:28,
/// EternalPickaxeItem.java:27).
pub fn ability_key(tool: Tool) -> &'static str {
    match tool {
        Tool::Sword => "tooltip.bmh.ability.eternal_sword",
        Tool::Axe => "tooltip.bmh.ability.eternal_axe",
        Tool::Pickaxe => "tooltip.bmh.ability.eternal_pickaxe",
    }
}

/// One tooltip line: a lang key and its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub key: &'static str,
    pub color: u32,
}

/// The lines that an Eternal tool adds after its name, in the Java order: the ability line (red
/// "worn out" in its place when the tool is worn out), the heavy line, the death wear line
/// (EternalSwordItem.java:26-33). Empty for any other item.
pub fn tooltip_lines(item: &str, damage: u32, max_damage: u32) -> Vec<Line> {
    let Some(tool) = tool_of(item) else {
        return Vec::new();
    };
    let first = if is_worn_out(damage, max_damage) {
        Line { key: KEY_WORN_OUT, color: RED }
    } else {
        Line { key: ability_key(tool), color: DARK_AQUA }
    };
    vec![
        first,
        Line { key: KEY_HEAVY, color: GOLD },
        Line { key: KEY_DEATH_WEAR, color: GRAY },
    ]
}

/// The width of the durability bar: one pixel at least, so a worn out tool still shows it
/// (EternalSwordItem.java:43-45, `Math.max(1, super.getBarWidth(stack))`).
pub fn bar_width(vanilla: i32) -> i32 {
    vanilla.max(1)
}

// ---------------------------------------------------------------------------------------------
// The break speed of a worn out tool (E4)
// ---------------------------------------------------------------------------------------------

/// The block tag that a tool is made for (BlockTags.MINEABLE_WITH_AXE, MINEABLE_WITH_PICKAXE). The
/// sword has no `getDestroySpeed` override.
pub fn mineable_tag(tool: Tool) -> Option<&'static str> {
    match tool {
        Tool::Axe => Some("minecraft:mineable/axe"),
        Tool::Pickaxe => Some("minecraft:mineable/pickaxe"),
        Tool::Sword => None,
    }
}

/// The enchantment whose level adds to the tool speed.
pub const EFFICIENCY: &str = "minecraft:efficiency";

/// The `minecraft:mining_efficiency` that Efficiency gives at `level`: the level squared plus 1
/// (data/minecraft/enchantment/efficiency.json, `levels_squared` with `added` 1.0). 0 at level 0.
pub fn efficiency_bonus(level: u32) -> f32 {
    if level == 0 {
        return 0.0;
    }
    let level = level as f32;
    level * level + 1.0
}

/// E4: the dig speed of a worn out axe or pickaxe. Java caps the tool's own speed at 4 before the
/// efficiency bonus and the other factors (EternalAxeItem.java:48-53, `Math.min(speed, 4.0)`; then
/// `Player.getDestroySpeed` adds the bonus when the speed is above 1 and multiplies by Haste, Mining
/// Fatigue, the block break speed, water and air). The hook sees `(15 + bonus) * factors` on a block of
/// the tool. The answer multiplies by `(4 + bonus) / (15 + bonus)`, which gives `(4 + bonus) * factors`,
/// the Java value. On another block the tool speed is 1, under the cap, and nothing changes.
/// `mineable` says whether the block is in the tool's tag. None when this side cannot tell (a client
/// that has no answer from its server yet): the speed is scaled then too, so a client never digs faster
/// than its server, and the server never sends the block back.
pub fn worn_break_speed(speed: f32, mineable: Option<bool>, efficiency_level: u32) -> f32 {
    if mineable == Some(false) {
        return speed;
    }
    let bonus = efficiency_bonus(efficiency_level);
    speed * (WORN_MINING_SPEED + bonus) / (TOOL_MINING_SPEED + bonus)
}

/// The most mineable answers that a client keeps. A block past it has no answer and is scaled.
pub const MAX_MINEABLE_ANSWERS: usize = 4096;

/// What a client knows of the mineable tags: for a tool and a block name, whether the block is in the
/// tool's tag, as its server read it (E4). A client cannot read a tag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MineableAnswers {
    answers: BTreeMap<(Tool, String), bool>,
}

impl MineableAnswers {
    /// No answer. A `const` so that a static can hold it.
    pub const fn new() -> Self {
        Self { answers: BTreeMap::new() }
    }

    /// Keeps an answer of the server. A new block past `MAX_MINEABLE_ANSWERS` is not kept.
    pub fn set(&mut self, tool: Tool, block: &str, mineable: bool) {
        let key = (tool, block.to_owned());
        if self.answers.len() >= MAX_MINEABLE_ANSWERS && !self.answers.contains_key(&key) {
            return;
        }
        self.answers.insert(key, mineable);
    }

    /// The answer for a block, or None when the server did not send one.
    pub fn get(&self, tool: Tool, block: &str) -> Option<bool> {
        self.answers.get(&(tool, block.to_owned())).copied()
    }
}

// ---------------------------------------------------------------------------------------------
// The fall and Axe Smash (E8)
// ---------------------------------------------------------------------------------------------

/// A fall shorter than this one does not smash (AxeSmashHandler.java:30, `fallDistance > 1.5`).
pub const MIN_FALL_DISTANCE: f32 = 1.5;
/// The most that a smash multiplies the damage (AxeSmashHandler.java:31).
pub const MAX_MULTIPLIER: f32 = 3.0;
/// The vanilla gravity in blocks per tick squared (AxeSmashHandler.java:33).
pub const GRAVITY: f64 = 0.08;

/// The effects that reset the fall each tick (`LivingEntity.aiStep`: Slow Falling and Levitation).
pub const FLOATING_EFFECTS: [&str; 2] = ["minecraft:slow_falling", "minecraft:levitation"];

/// What the server knows of a player in one tick, to follow the fall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallInput {
    /// The height of the feet.
    pub y: f64,
    pub on_ground: bool,
    pub in_water: bool,
    pub climbing: bool,
    pub gliding: bool,
    /// Flying in creative or spectator (`Abilities.flying`): `Player.aiStep` resets the fall each tick.
    pub flying: bool,
    /// Under Slow Falling or Levitation: `LivingEntity.aiStep` resets the fall each tick.
    pub floating: bool,
}

/// The fall distance of a player, as `Entity.fallDistance` follows it: it grows with each block of
/// descent and goes back to 0 on the ground, in water, on a ladder, while gliding, while flying and
/// under Slow Falling or Levitation.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Fall {
    last_y: Option<f64>,
    distance: f64,
}

impl Fall {
    /// One tick. Returns the distance after it.
    pub fn step(&mut self, input: FallInput) -> f64 {
        if input.on_ground
            || input.in_water
            || input.climbing
            || input.gliding
            || input.flying
            || input.floating
        {
            self.distance = 0.0;
        } else if let Some(last) = self.last_y {
            if input.y < last {
                self.distance += last - input.y;
            }
        }
        self.last_y = Some(input.y);
        self.distance
    }

    /// The distance now.
    pub fn distance(&self) -> f64 {
        self.distance
    }

    /// `attacker.fallDistance *= 0.5` after a smash (AxeSmashHandler.java:67). Returns the distance
    /// that was removed.
    pub fn halve(&mut self) -> f64 {
        let removed = self.distance - self.distance * 0.5;
        self.distance -= removed;
        removed
    }
}

/// `AxeSmashHandler.multiplier` for a player that is smashing: `min(3, 1 + sqrt(2 * 0.08 * fall))`
/// (AxeSmashHandler.java:35-44). 1 when the fall is too short.
pub fn smash_multiplier(distance: f64) -> f32 {
    let distance = distance as f32;
    if distance <= MIN_FALL_DISTANCE {
        return 1.0;
    }
    let speed = (2.0 * GRAVITY * f64::from(distance)).sqrt();
    ((1.0 + speed) as f32).min(MAX_MULTIPLIER)
}

/// `Math.round(10 * multiplier)`: the particle count of a smash (AxeSmashHandler.java:70).
pub fn smash_dust(multiplier: f32) -> i32 {
    (10.0 * multiplier + 0.5).floor() as i32
}

/// A sound of an effect: id, volume and pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    pub id: &'static str,
    pub volume: f32,
    pub pitch: f32,
}

/// The two sounds of a smash, at the target (AxeSmashHandler.java:72-75).
pub const SMASH_SOUNDS: [Sound; 2] = [
    Sound { id: "minecraft:block.anvil.land", volume: 0.6, pitch: 0.7 },
    Sound { id: "minecraft:entity.player.attack.knockback", volume: 1.0, pitch: 0.6 },
];

/// The sound of Echo Strike, at the victim (EchoStrikeHandler.java:132).
pub const ECHO_SOUND: Sound = Sound { id: "minecraft:block.amethyst_block.resonate", volume: 0.6, pitch: 1.6 };

/// The sound of Echo Sense, at the broken block (EchoSenseHandler.java:139).
pub const SENSE_SOUND: Sound = Sound { id: "minecraft:block.amethyst_block.chime", volume: 0.8, pitch: 0.7 };

/// The particles of a smash at a target at (x, y, z) with a box `height` high: the explosion, the
/// cloud and the crit (AxeSmashHandler.java:77-82). `at` is where they appear, `spread` and `speed` as
/// `sendParticles` takes them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burst {
    pub kind: BurstKind,
    pub at: [f64; 3],
    pub count: i32,
    pub spread: [f64; 3],
    pub speed: f64,
}

/// The vanilla particle that a burst uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurstKind {
    Explosion,
    Cloud,
    Crit,
    ElectricSpark,
}

/// The three bursts of a smash.
pub fn smash_bursts(target: [f64; 3], height: f64, multiplier: f32) -> [Burst; 3] {
    let dust = smash_dust(multiplier);
    [
        Burst {
            kind: BurstKind::Explosion,
            at: [target[0], target[1] + 0.1, target[2]],
            count: 1,
            spread: [0.0, 0.0, 0.0],
            speed: 0.0,
        },
        Burst {
            kind: BurstKind::Cloud,
            at: [target[0], target[1] + 0.1, target[2]],
            count: dust,
            spread: [0.5, 0.05, 0.5],
            speed: 0.15,
        },
        Burst {
            kind: BurstKind::Crit,
            // `Entity.getY(0.5)`: half the height of the box.
            at: [target[0], target[1] + height * 0.5, target[2]],
            count: dust,
            spread: [0.4, 0.4, 0.4],
            speed: 0.5,
        },
    ]
}

// ---------------------------------------------------------------------------------------------
// Echo Strike (E7)
// ---------------------------------------------------------------------------------------------

/// A full-strength hit only (EchoStrikeHandler.java:41).
pub const MIN_ATTACK_STRENGTH: f32 = 0.9;
/// The echo deals this share of the hit (EchoStrikeHandler.java:42).
pub const DAMAGE_RATIO: f32 = 0.5;
/// Ticks between the hit and its echo (EchoStrikeHandler.java:43).
pub const DELAY_TICKS: i64 = 5;
/// How far from the target an echo looks for a victim (EchoStrikeHandler.java:44).
pub const RANGE: f64 = 5.0;
/// The most echoes that wait for one player (EchoStrikeHandler.java:46).
pub const MAX_PENDING_ECHOES: usize = 8;
/// The colour of the line of an echo, in red, green and blue (EchoStrikeHandler.java:48).
pub const ECHO_COLOR: [f32; 3] = [0.1, 0.9, 1.0];
/// The damage type of an echo (`damageSources().playerAttack`).
pub const ECHO_DAMAGE_TYPE: &str = "minecraft:player_attack";
/// The sparks at the end of the line (EchoStrikeHandler.java:206).
pub const ECHO_SPARKS: Burst = Burst {
    kind: BurstKind::ElectricSpark,
    at: [0.0; 3],
    count: 6,
    spread: [0.2, 0.2, 0.2],
    speed: 0.1,
};

/// An echo on its way (EchoStrikeHandler.PendingEcho, EchoStrikeHandler.java:53-55).
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEcho {
    pub dimension: String,
    /// The entity id of the target of the hit.
    pub target: u32,
    /// The centre of the target when the hit landed.
    pub origin: [f64; 3],
    pub damage: f32,
    pub due: i64,
}

/// `EchoStrikeHandler.onSwordHit`: queues the echo of a hit. Nothing for a weak swing or a hit with no
/// damage. The queue keeps `MAX_PENDING_ECHOES`: the oldest echo goes first (EchoStrikeHandler.java:68-84).
pub fn queue_echo(
    queue: &mut Vec<PendingEcho>,
    dimension: &str,
    target: u32,
    origin: [f64; 3],
    damage: f32,
    attack_strength: f32,
    now: i64,
) -> bool {
    if attack_strength < MIN_ATTACK_STRENGTH || damage <= 0.0 {
        return false;
    }
    if queue.len() >= MAX_PENDING_ECHOES {
        queue.remove(0);
    }
    queue.push(PendingEcho {
        dimension: dimension.to_owned(),
        target,
        origin,
        damage: damage * DAMAGE_RATIO,
        due: now + DELAY_TICKS,
    });
    true
}

/// `EchoStrikeHandler.deliverDueEchoes`: takes the echoes that are due out of the queue first
/// (EchoStrikeHandler.java:91-113). The caller drops the ones of another dimension.
pub fn take_due(queue: &mut Vec<PendingEcho>, now: i64) -> Vec<PendingEcho> {
    let mut due = Vec::new();
    let mut waiting = Vec::new();
    for echo in queue.drain(..) {
        if echo.due <= now {
            due.push(echo);
        } else {
            waiting.push(echo);
        }
    }
    *queue = waiting;
    due
}

/// A mob that an echo may hit.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: u32,
    /// The centre of its box.
    pub center: [f64; 3],
}

/// `EchoStrikeHandler.nearestVictim`: the nearest candidate within `RANGE` of the origin, other than the
/// target (EchoStrikeHandler.java:144-163). On a tie the later one wins, as Java's `<=` makes it.
pub fn nearest_victim(origin: [f64; 3], target: u32, candidates: &[Candidate]) -> Option<u32> {
    let mut nearest = None;
    let mut nearest_distance = RANGE * RANGE;
    for candidate in candidates {
        if candidate.id == target {
            continue;
        }
        let distance = distance_squared(candidate.center, origin);
        if distance <= nearest_distance {
            nearest = Some(candidate.id);
            nearest_distance = distance;
        }
    }
    nearest
}

fn distance_squared(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    dx * dx + dy * dy + dz * dz
}

/// The distance between two points.
pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    distance_squared(a, b).sqrt()
}

/// The points of the line of an echo, one every 0.25 block (EchoStrikeHandler.java:189-205).
pub fn line_points(from: [f64; 3], to: [f64; 3]) -> Vec<[f64; 3]> {
    let length = distance(from, to);
    let steps = ((length / 0.25).ceil() as i64).max(2);
    (0..=steps)
        .map(|i| {
            let t = i as f64 / steps as f64;
            [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
                from[2] + (to[2] - from[2]) * t,
            ]
        })
        .collect()
}

/// The mobs that Java's `Enemy` interface names (EchoStrikeHandler.java:165-186). A player, an armor
/// stand and a villager are no `Enemy`, so the Java filter that spares them has no case here. The kinds:
/// the Monster classes, the slimes, the ghast, the phantom, the shulker, the hoglin and the ender dragon.
const ENEMIES: &[&str] = &[
    "minecraft:blaze",
    "minecraft:bogged",
    "minecraft:breeze",
    "minecraft:cave_spider",
    "minecraft:creaking",
    "minecraft:creeper",
    "minecraft:drowned",
    "minecraft:elder_guardian",
    "minecraft:ender_dragon",
    "minecraft:enderman",
    "minecraft:endermite",
    "minecraft:evoker",
    "minecraft:ghast",
    "minecraft:giant",
    "minecraft:guardian",
    "minecraft:hoglin",
    "minecraft:husk",
    "minecraft:illusioner",
    "minecraft:magma_cube",
    "minecraft:parched",
    "minecraft:phantom",
    "minecraft:piglin",
    "minecraft:piglin_brute",
    "minecraft:pillager",
    "minecraft:ravager",
    "minecraft:shulker",
    "minecraft:silverfish",
    "minecraft:skeleton",
    "minecraft:slime",
    "minecraft:spider",
    "minecraft:stray",
    "minecraft:vex",
    "minecraft:vindicator",
    "minecraft:warden",
    "minecraft:witch",
    "minecraft:wither",
    "minecraft:wither_skeleton",
    "minecraft:zoglin",
    "minecraft:zombie",
    "minecraft:zombie_villager",
    "minecraft:zombified_piglin",
];

/// True when `kind` is an `Enemy` that an echo may hit.
pub fn is_enemy(kind: &str) -> bool {
    ENEMIES.contains(&kind)
}

/// The advancement paths that the abilities award (EchoStrikeHandler.java:131, AxeSmashHandler.java:62,
/// EchoSenseHandler.java:137).
pub const AWARD_ECHO_STRIKE: &str = "eternity/echo_strike";
pub const AWARD_SMASH: &str = "eternity/smash";
pub const AWARD_ECHO_SENSE: &str = "eternity/echo_sense";

// ---------------------------------------------------------------------------------------------
// Echo Sense (E9)
// ---------------------------------------------------------------------------------------------

/// How far from the broken ore the same ore glows (EchoSenseHandler.java:67).
pub const RADIUS: i32 = 8;
/// The most glows around one spot (EchoSenseHandler.java:68).
pub const MAX_DISPLAYS: usize = 64;
/// How long a glow lasts, in ticks (EchoSenseHandler.java:69).
pub const LIFETIME_TICKS: i64 = 100;
/// Ticks between two readings of one player (EchoSenseHandler.java:71).
pub const COOLDOWN_TICKS: i64 = 4;
/// The colour of the glow (EchoSenseHandler.java:72).
pub const GLOW_COLOR: u32 = 0x00F0FF;
/// The glow box is a block drawn a hair inside: it starts 0.02 in and is 0.96 wide
/// (EchoSenseHandler.java:233-235).
pub const GLOW_INSET: f64 = 0.02;
pub const GLOW_SIZE: f64 = 0.96;

/// The vanilla ore families (BlockTags.COAL_ORES and so on). "The same ore" is the same family, so the
/// stone and deepslate variants light up together (EchoSenseHandler.java:79-88).
const FAMILIES: [&[&str]; 8] = [
    &["minecraft:coal_ore", "minecraft:deepslate_coal_ore"],
    &["minecraft:copper_ore", "minecraft:deepslate_copper_ore"],
    &["minecraft:iron_ore", "minecraft:deepslate_iron_ore"],
    &["minecraft:gold_ore", "minecraft:deepslate_gold_ore", "minecraft:nether_gold_ore"],
    &["minecraft:redstone_ore", "minecraft:deepslate_redstone_ore"],
    &["minecraft:lapis_ore", "minecraft:deepslate_lapis_ore"],
    &["minecraft:diamond_ore", "minecraft:deepslate_diamond_ore"],
    &["minecraft:emerald_ore", "minecraft:deepslate_emerald_ore"],
];

/// The tags of the ore families, in the Java order (EchoSenseHandler.java:79-88, `ORE_FAMILIES`).
/// `FAMILIES` holds their vanilla blocks at the same index.
pub const FAMILY_TAGS: [&str; 8] = [
    "minecraft:coal_ores",
    "minecraft:copper_ores",
    "minecraft:iron_ores",
    "minecraft:gold_ores",
    "minecraft:redstone_ores",
    "minecraft:lapis_ores",
    "minecraft:diamond_ores",
    "minecraft:emerald_ores",
];

/// The tag of the ores that Echo Sense reads (EchoSenseHandler.java:64, `ECHO_SENSE_ORES`).
pub const ORE_TAG: &str = "bmh:echo_sense_ores";

/// The ores that stand alone: no vanilla tag groups them (EchoSenseHandler.java:81-84).
const SINGLE_ORES: [&str; 2] = ["minecraft:nether_quartz_ore", "minecraft:ancient_debris"];

/// The written copy of the tags `ORE_TAG` and `FAMILY_TAGS`, as the vanilla data and
/// data/bmh/tags/block/echo_sense_ores.json fill them. The server uses this copy only when its tags
/// cannot be read. False for any other tag.
pub fn listed_in_tag(block: &str, tag: &str) -> bool {
    if tag == ORE_TAG {
        return FAMILIES.iter().any(|family| family.contains(&block)) || SINGLE_ORES.contains(&block);
    }
    FAMILY_TAGS
        .iter()
        .zip(FAMILIES.iter())
        .any(|(family_tag, family)| *family_tag == tag && family.contains(&block))
}

/// `EchoSenseHandler.sameOre`: the first family tag that holds the broken block, else the block itself.
/// `in_tag(block, tag)` answers whether a block is in a tag.
pub fn same_ore<T>(broken: &str, other: &str, in_tag: T) -> bool
where
    T: Fn(&str, &str) -> bool,
{
    match FAMILY_TAGS.iter().find(|tag| in_tag(broken, tag)) {
        Some(tag) => in_tag(other, tag),
        None => broken == other,
    }
}

/// The offsets within `RADIUS` of the centre, in the Java loop order (x, then y, then z).
fn sphere() -> Vec<[i32; 3]> {
    let mut offsets = Vec::new();
    for dx in -RADIUS..=RADIUS {
        for dy in -RADIUS..=RADIUS {
            for dz in -RADIUS..=RADIUS {
                if dx * dx + dy * dy + dz * dz <= RADIUS * RADIUS {
                    offsets.push([dx, dy, dz]);
                }
            }
        }
    }
    offsets
}

/// `EchoSenseHandler.findSameOre`: the positions within `RADIUS` of the centre that hold the same ore,
/// nearest first (EchoSenseHandler.java:185-217). `block_at` answers the block name at a position, None
/// for a block that is not loaded. `same` answers whether a block name is the same ore as the broken one
/// (`same_ore`). The centre itself is left out: it is the block that breaks.
pub fn find_same_ore<F, S>(center: [i32; 3], mut block_at: F, mut same: S) -> Vec<[i32; 3]>
where
    F: FnMut([i32; 3]) -> Option<String>,
    S: FnMut(&str) -> bool,
{
    let mut found: Vec<([i32; 3], i32)> = Vec::new();
    for [dx, dy, dz] in sphere() {
        if dx == 0 && dy == 0 && dz == 0 {
            continue;
        }
        let pos = [center[0] + dx, center[1] + dy, center[2] + dz];
        let Some(name) = block_at(pos) else {
            continue;
        };
        if same(&name) {
            found.push((pos, dx * dx + dy * dy + dz * dz));
        }
    }
    // A stable sort keeps the loop order among equal distances, as `List.sort` does.
    found.sort_by_key(|(_, distance)| *distance);
    found.into_iter().map(|(pos, _)| pos).collect()
}

/// A glow on its way out: where, in which dimension, and the game time it ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glow {
    pub dimension: String,
    pub pos: [i32; 3],
    pub expiry: i64,
}

/// The glows that the server knows. It stands for the block displays of Java.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlowBook {
    glows: Vec<Glow>,
}

impl GlowBook {
    /// An empty book. A `const` so that a static can hold one.
    pub const fn new() -> Self {
        Self { glows: Vec::new() }
    }

    /// Drops the glows whose time is up (EchoSenseHandler.java:154-158).
    pub fn expire(&mut self, now: i64) {
        self.glows.retain(|glow| now < glow.expiry);
    }

    /// The positions of the glows within reach of a reading at `center`: the box of `RADIUS + 1`
    /// around the block (EchoSenseHandler.java:112).
    pub fn near(&self, dimension: &str, center: [i32; 3]) -> Vec<[i32; 3]> {
        let reach = RADIUS + 1;
        self.glows
            .iter()
            .filter(|glow| {
                glow.dimension == dimension
                    && (0..3).all(|axis| (glow.pos[axis] - center[axis]).abs() <= reach)
            })
            .map(|glow| glow.pos)
            .collect()
    }

    /// Takes out the glows of this dimension near `center` whose position `keep` refuses (their ore is
    /// gone: Java discards such a display at its next tick, EchoSenseHandler.java:164-168).
    pub fn forget_where<F>(&mut self, dimension: &str, center: [i32; 3], mut keep: F)
    where
        F: FnMut([i32; 3]) -> bool,
    {
        let reach = RADIUS + 1;
        self.glows.retain(|glow| {
            let near = glow.dimension == dimension
                && (0..3).all(|axis| (glow.pos[axis] - center[axis]).abs() <= reach);
            !near || keep(glow.pos)
        });
    }

    /// Sets the end of the glow at a position, or adds the glow.
    pub fn set(&mut self, dimension: &str, pos: [i32; 3], expiry: i64) {
        match self.glows.iter_mut().find(|g| g.dimension == dimension && g.pos == pos) {
            Some(glow) => glow.expiry = expiry,
            None => self.glows.push(Glow { dimension: dimension.to_owned(), pos, expiry }),
        }
    }

    /// The number of glows.
    pub fn len(&self) -> usize {
        self.glows.len()
    }

    /// True when no glow is known.
    pub fn is_empty(&self) -> bool {
        self.glows.is_empty()
    }
}

/// The result of one reading: the positions that start to glow and the ones that glow longer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reading {
    pub spawned: Vec<[i32; 3]>,
    pub extended: Vec<[i32; 3]>,
}

impl Reading {
    /// Every position that the clients draw: the new glows and the longer ones.
    pub fn positions(&self) -> Vec<[i32; 3]> {
        self.spawned.iter().chain(self.extended.iter()).copied().collect()
    }
}

/// `EchoSenseHandler.onOreBroken` for the glows: a glow that exists gets a longer time, a new one
/// needs room in the budget of `MAX_DISPLAYS` around the spot (EchoSenseHandler.java:106-134). The
/// book keeps the result.
pub fn read_ores(
    book: &mut GlowBook,
    dimension: &str,
    center: [i32; 3],
    found: &[[i32; 3]],
    now: i64,
) -> Reading {
    let expiry = now + LIFETIME_TICKS;
    let shining = book.near(dimension, center);
    let mut budget = MAX_DISPLAYS.saturating_sub(shining.len());
    let mut reading = Reading::default();
    for pos in found {
        if shining.contains(pos) {
            book.set(dimension, *pos, expiry);
            reading.extended.push(*pos);
        } else if budget > 0 {
            book.set(dimension, *pos, expiry);
            budget -= 1;
            reading.spawned.push(*pos);
        }
    }
    reading
}

/// The box of a glow, drawn a hair inside its block (EchoSenseHandler.java:233-235).
pub fn glow_box(pos: [i32; 3]) -> ([f64; 3], [f64; 3]) {
    let min = [
        f64::from(pos[0]) + GLOW_INSET,
        f64::from(pos[1]) + GLOW_INSET,
        f64::from(pos[2]) + GLOW_INSET,
    ];
    let max = [min[0] + GLOW_SIZE, min[1] + GLOW_SIZE, min[2] + GLOW_SIZE];
    (min, max)
}

/// Clients draw the glow of a block within this distance of the camera. Java's displays have the view
/// range of the entity (64 blocks, `Display.viewRange` 1.0 times the 64 block base).
pub const GLOW_VIEW_DISTANCE: f64 = 64.0;

/// The ARGB colour of the box that a client draws (opaque, `GLOW_COLOR`).
pub const GLOW_ARGB: u32 = 0xFF00_0000 | GLOW_COLOR;

/// The width of the lines of the box, in pixels.
pub const GLOW_LINE_WIDTH: f32 = 2.0;

/// The players that see a reading: the ones in the dimension within the view distance of the ore.
pub fn sees_glow(player_dimension: &str, player: [f64; 3], dimension: &str, center: [i32; 3]) -> bool {
    let at = [f64::from(center[0]) + 0.5, f64::from(center[1]) + 0.5, f64::from(center[2]) + 0.5];
    player_dimension == dimension && distance(player, at) <= GLOW_VIEW_DISTANCE + f64::from(RADIUS)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- wear ----

    #[test]
    fn the_tools_are_the_three_ids() {
        assert_eq!(tool_of("bmh:eternal_sword"), Some(Tool::Sword));
        assert_eq!(tool_of("bmh:eternal_axe"), Some(Tool::Axe));
        assert_eq!(tool_of("bmh:eternal_pickaxe"), Some(Tool::Pickaxe));
        // The bundle and the totems are Eternal Items but no tools (EternalWear.java:32-37).
        assert_eq!(tool_of("bmh:eternal_bundle"), None);
        assert_eq!(tool_of("bmh:totem_of_eternity"), None);
        assert_eq!(tool_of("minecraft:netherite_sword"), None);
    }

    #[test]
    fn a_tool_is_worn_out_at_its_last_point() {
        assert!(!is_worn_out(0, MAX_DAMAGE));
        assert!(!is_worn_out(32_998, MAX_DAMAGE));
        assert!(is_worn_out(32_999, MAX_DAMAGE));
        assert!(is_worn_out(33_000, MAX_DAMAGE));
    }

    #[test]
    fn a_use_never_takes_the_last_point() {
        assert_eq!(cap_damage(0, 1, MAX_DAMAGE), 1);
        assert_eq!(cap_damage(32_990, 20, MAX_DAMAGE), 9);
        // At the last point the room is 0: the tool stays whole.
        assert_eq!(cap_damage(32_999, 1, MAX_DAMAGE), 0);
        // Past it (an older version of the mod): negative, the tool gets its last point back.
        assert_eq!(cap_damage(33_000, 1, MAX_DAMAGE), -1);
        assert_eq!(cap_damage(40_000, 1, MAX_DAMAGE), -7001);
        // A smaller wear passes.
        assert_eq!(cap_damage(100, -3, MAX_DAMAGE), -3);
    }

    #[test]
    fn seven_percent_of_33000_is_2310() {
        assert_eq!(wear(0, 7, MAX_DAMAGE), Some(2310));
        assert_eq!(death_wear(0, MAX_DAMAGE), 2310);
        assert_eq!(death_wear(10_000, MAX_DAMAGE), 12_310);
        // Down to the last point, never past it.
        assert_eq!(death_wear(31_000, MAX_DAMAGE), 32_999);
        assert_eq!(death_wear(32_999, MAX_DAMAGE), 32_999);
        assert_eq!(death_wear(33_000, MAX_DAMAGE), 32_999);
    }

    #[test]
    fn a_percentage_outside_0_to_100_is_refused() {
        assert_eq!(death_wear(0, 100), 7);
        assert_eq!(death_wear(98, 100), 99);
        assert_eq!(wear(0, 101, MAX_DAMAGE), None);
        assert_eq!(wear(5, 0, MAX_DAMAGE), Some(5));
        assert_eq!(wear(0, 100, MAX_DAMAGE), Some(32_999));
        assert_eq!(wear(0, 50, MAX_DAMAGE), Some(16_500));
    }

    #[test]
    fn the_last_point_comes_back() {
        assert_eq!(restore_last_point(33_000, MAX_DAMAGE), 32_999);
        assert_eq!(restore_last_point(32_999, MAX_DAMAGE), 32_999);
        assert_eq!(restore_last_point(5, MAX_DAMAGE), 5);
    }

    // ---- tooltip and bar ----

    #[test]
    fn a_fresh_tool_names_its_ability_then_heavy_then_death_wear() {
        let lines = tooltip_lines("bmh:eternal_sword", 0, MAX_DAMAGE);
        assert_eq!(
            lines,
            vec![
                Line { key: "tooltip.bmh.ability.eternal_sword", color: 0x00AAAA },
                Line { key: "tooltip.bmh.heavy_with_eternity", color: 0xFFAA00 },
                Line { key: "tooltip.bmh.death_wear", color: 0xAAAAAA },
            ]
        );
        assert_eq!(tooltip_lines("bmh:eternal_axe", 1, MAX_DAMAGE)[0].key, "tooltip.bmh.ability.eternal_axe");
        assert_eq!(tooltip_lines("bmh:eternal_pickaxe", 1, MAX_DAMAGE)[0].key, "tooltip.bmh.ability.eternal_pickaxe");
    }

    #[test]
    fn a_worn_out_tool_shows_a_red_line_in_place_of_its_ability() {
        let lines = tooltip_lines("bmh:eternal_axe", 32_999, MAX_DAMAGE);
        assert_eq!(lines[0], Line { key: "tooltip.bmh.worn_out", color: 0xFF5555 });
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1].key, KEY_HEAVY);
        assert_eq!(lines[2].key, KEY_DEATH_WEAR);
    }

    #[test]
    fn other_items_get_no_line() {
        assert!(tooltip_lines("bmh:eternal_bundle", 0, MAX_DAMAGE).is_empty());
        assert!(tooltip_lines("minecraft:stick", 0, MAX_DAMAGE).is_empty());
    }

    #[test]
    fn the_bar_keeps_one_pixel() {
        assert_eq!(bar_width(0), 1);
        assert_eq!(bar_width(-3), 1);
        assert_eq!(bar_width(1), 1);
        assert_eq!(bar_width(13), 13);
    }

    // ---- break speed ----

    #[test]
    fn a_worn_out_tool_mines_at_stone_speed_on_its_blocks() {
        assert!((worn_break_speed(15.0, Some(true), 0) - 4.0).abs() < 1e-6);
        // Under the cap already: the other blocks keep their speed.
        assert!((worn_break_speed(1.0, Some(false), 0) - 1.0).abs() < 1e-6);
        assert!((worn_break_speed(0.2, Some(false), 5) - 0.2).abs() < 1e-6);
        // The factors after the tool speed stay: off the ground (divided by 5), 15 / 5 = 3 becomes 4 / 5.
        assert!((worn_break_speed(3.0, Some(true), 0) - 0.8).abs() < 1e-6);
        assert_eq!(mineable_tag(Tool::Axe), Some("minecraft:mineable/axe"));
        assert_eq!(mineable_tag(Tool::Pickaxe), Some("minecraft:mineable/pickaxe"));
        assert_eq!(mineable_tag(Tool::Sword), None);
    }

    #[test]
    fn efficiency_adds_to_the_capped_speed_as_in_java() {
        // Efficiency gives level * level + 1 (efficiency.json, levels_squared plus 1).
        assert_eq!(efficiency_bonus(0), 0.0);
        assert_eq!(efficiency_bonus(1), 2.0);
        assert_eq!(efficiency_bonus(5), 26.0);
        // Efficiency V: Java digs at min(15, 4) + 26 = 30. The hook sees 15 + 26 = 41.
        assert!((worn_break_speed(41.0, Some(true), 5) - 30.0).abs() < 1e-4);
        // With Haste II (factor 1.4) and under water (factor 0.2): Java (4 + 26) * 1.4 * 0.2 = 8.4.
        assert!((worn_break_speed(41.0 * 1.4 * 0.2, Some(true), 5) - 8.4).abs() < 1e-4);
        // Efficiency III: (4 + 10) / (15 + 10) of 25 = 14.
        assert!((worn_break_speed(25.0, Some(true), 3) - 14.0).abs() < 1e-4);
        // A huge level from a command does not panic: the ratio goes to 1.
        let huge = worn_break_speed(1.0, Some(true), u32::MAX);
        assert!(huge.is_finite() && huge <= 1.0);
    }

    #[test]
    fn a_client_with_no_answer_never_digs_faster_than_its_server() {
        for level in [0, 1, 5] {
            let bonus = efficiency_bonus(level);
            for factor in [1.0_f32, 0.2, 0.04, 0.3, 1.4] {
                // A block of the tool: the server scales; with no answer the client scales the same.
                let tool_block = (15.0 + bonus) * factor;
                let server = worn_break_speed(tool_block, Some(true), level);
                assert!((worn_break_speed(tool_block, None, level) - server).abs() < 1e-5);
                // Another block: the server keeps the speed; the client with no answer is slower.
                let other = factor;
                assert!(worn_break_speed(other, None, level) <= worn_break_speed(other, Some(false), level));
            }
        }
    }

    #[test]
    fn a_client_keeps_the_answers_of_its_server() {
        let mut answers = MineableAnswers::new();
        assert_eq!(answers.get(Tool::Pickaxe, "minecraft:stone"), None);
        answers.set(Tool::Pickaxe, "minecraft:stone", true);
        answers.set(Tool::Pickaxe, "minecraft:dirt", false);
        assert_eq!(answers.get(Tool::Pickaxe, "minecraft:stone"), Some(true));
        assert_eq!(answers.get(Tool::Pickaxe, "minecraft:dirt"), Some(false));
        // The answer is for one tool.
        assert_eq!(answers.get(Tool::Axe, "minecraft:stone"), None);
        // A new answer replaces the old one.
        answers.set(Tool::Pickaxe, "minecraft:stone", false);
        assert_eq!(answers.get(Tool::Pickaxe, "minecraft:stone"), Some(false));
        // Past the cap a new block is not kept, a known one still changes.
        for i in 0..MAX_MINEABLE_ANSWERS {
            answers.set(Tool::Axe, &format!("test:block_{i}"), true);
        }
        answers.set(Tool::Axe, "test:one_more", true);
        assert_eq!(answers.get(Tool::Axe, "test:one_more"), None);
        answers.set(Tool::Pickaxe, "minecraft:stone", true);
        assert_eq!(answers.get(Tool::Pickaxe, "minecraft:stone"), Some(true));
    }

    // ---- the fall and the smash ----

    fn air(y: f64) -> FallInput {
        FallInput {
            y,
            on_ground: false,
            in_water: false,
            climbing: false,
            gliding: false,
            flying: false,
            floating: false,
        }
    }

    #[test]
    fn the_fall_grows_with_the_descent_and_ends_on_the_ground() {
        let mut fall = Fall::default();
        assert_eq!(fall.step(air(70.0)), 0.0);
        assert!((fall.step(air(69.0)) - 1.0).abs() < 1e-9);
        assert!((fall.step(air(66.5)) - 3.5).abs() < 1e-9);
        // Going up does not shorten it, as in vanilla.
        assert!((fall.step(air(67.0)) - 3.5).abs() < 1e-9);
        let ground = FallInput { y: 60.0, on_ground: true, ..air(60.0) };
        assert_eq!(fall.step(ground), 0.0);
    }

    #[test]
    fn water_a_ladder_a_glide_a_flight_and_slow_falling_reset_the_fall() {
        for reset in [
            FallInput { in_water: true, ..air(50.0) },
            FallInput { climbing: true, ..air(50.0) },
            FallInput { gliding: true, ..air(50.0) },
            FallInput { flying: true, ..air(50.0) },
            FallInput { floating: true, ..air(50.0) },
        ] {
            let mut fall = Fall::default();
            fall.step(air(60.0));
            assert!(fall.step(air(55.0)) > 4.0);
            assert_eq!(fall.step(reset), 0.0);
        }
        // A drift down under Slow Falling never builds a fall: no smash at the end of it.
        let mut fall = Fall::default();
        for y in (0..=20).rev() {
            fall.step(FallInput { floating: true, ..air(f64::from(y)) });
        }
        assert_eq!(smash_multiplier(fall.distance()), 1.0);
        assert_eq!(FLOATING_EFFECTS, ["minecraft:slow_falling", "minecraft:levitation"]);
    }

    #[test]
    fn a_smash_halves_the_fall() {
        let mut fall = Fall::default();
        fall.step(air(80.0));
        fall.step(air(70.0));
        let removed = fall.halve();
        assert!((removed - 5.0).abs() < 1e-9);
        assert!((fall.distance() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn the_smash_multiplier_follows_the_free_fall_speed() {
        assert_eq!(smash_multiplier(0.0), 1.0);
        assert_eq!(smash_multiplier(1.5), 1.0);
        // sqrt(2 * 0.08 * 10) = 1.2649
        assert!((smash_multiplier(10.0) - 2.2649).abs() < 1e-3);
        // sqrt(0.16 * 4) = 0.8
        assert!((smash_multiplier(4.0) - 1.8).abs() < 1e-6);
        // The cap is 3 (a fall of 25 blocks gives 1 + 2).
        assert!((smash_multiplier(25.0) - 3.0).abs() < 1e-6);
        assert_eq!(smash_multiplier(500.0), 3.0);
    }

    #[test]
    fn the_smash_dust_rounds_as_java_does() {
        assert_eq!(smash_dust(1.8), 18);
        assert_eq!(smash_dust(3.0), 30);
        assert_eq!(smash_dust(2.2649), 23);
        assert_eq!(smash_dust(1.05), 11);
    }

    #[test]
    fn the_smash_effects_are_the_java_numbers() {
        assert_eq!(SMASH_SOUNDS[0], Sound { id: "minecraft:block.anvil.land", volume: 0.6, pitch: 0.7 });
        assert_eq!(
            SMASH_SOUNDS[1],
            Sound { id: "minecraft:entity.player.attack.knockback", volume: 1.0, pitch: 0.6 }
        );
        let bursts = smash_bursts([1.0, 64.0, 2.0], 1.9, 3.0);
        assert_eq!(bursts[0].kind, BurstKind::Explosion);
        assert_eq!(bursts[0].count, 1);
        assert!((bursts[0].at[1] - 64.1).abs() < 1e-9);
        assert_eq!(bursts[1].kind, BurstKind::Cloud);
        assert_eq!(bursts[1].count, 30);
        assert_eq!(bursts[1].spread, [0.5, 0.05, 0.5]);
        assert_eq!(bursts[1].speed, 0.15);
        assert_eq!(bursts[2].kind, BurstKind::Crit);
        assert!((bursts[2].at[1] - 64.95).abs() < 1e-9);
        assert_eq!(bursts[2].spread, [0.4, 0.4, 0.4]);
        assert_eq!(bursts[2].speed, 0.5);
    }

    // ---- Echo Strike ----

    #[test]
    fn an_echo_needs_a_full_swing_and_some_damage() {
        let mut queue = Vec::new();
        assert!(!queue_echo(&mut queue, "minecraft:overworld", 7, [0.0; 3], 10.0, 0.89, 100));
        assert!(!queue_echo(&mut queue, "minecraft:overworld", 7, [0.0; 3], 0.0, 1.0, 100));
        assert!(queue.is_empty());
        assert!(queue_echo(&mut queue, "minecraft:overworld", 7, [1.0, 2.0, 3.0], 12.0, 0.9, 100));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].damage, 6.0);
        assert_eq!(queue[0].due, 105);
        assert_eq!(queue[0].origin, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn eight_echoes_wait_at_most() {
        let mut queue = Vec::new();
        for i in 0..10u32 {
            queue_echo(&mut queue, "minecraft:overworld", i, [0.0; 3], 4.0, 1.0, i64::from(i));
        }
        assert_eq!(queue.len(), 8);
        // The two oldest went.
        assert_eq!(queue[0].target, 2);
        assert_eq!(queue[7].target, 9);
    }

    #[test]
    fn the_due_echoes_leave_the_queue_first() {
        let mut queue = Vec::new();
        queue_echo(&mut queue, "minecraft:overworld", 1, [0.0; 3], 4.0, 1.0, 0);
        queue_echo(&mut queue, "minecraft:overworld", 2, [0.0; 3], 4.0, 1.0, 3);
        assert!(take_due(&mut queue, 4).is_empty());
        let due = take_due(&mut queue, 5);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].target, 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(take_due(&mut queue, 100).len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn the_victim_is_the_nearest_other_mob_within_five_blocks() {
        let candidates = [
            Candidate { id: 1, center: [0.0, 0.0, 0.0] },
            Candidate { id: 2, center: [3.0, 0.0, 0.0] },
            Candidate { id: 3, center: [2.0, 0.0, 0.0] },
            Candidate { id: 4, center: [5.1, 0.0, 0.0] },
        ];
        // The target (1) is never its own victim.
        assert_eq!(nearest_victim([0.0; 3], 1, &candidates), Some(3));
        // Exactly five blocks still counts.
        let edge = [Candidate { id: 9, center: [5.0, 0.0, 0.0] }];
        assert_eq!(nearest_victim([0.0; 3], 1, &edge), Some(9));
        // Nothing within reach.
        let far = [Candidate { id: 4, center: [5.1, 0.0, 0.0] }];
        assert_eq!(nearest_victim([0.0; 3], 1, &far), None);
        assert_eq!(nearest_victim([0.0; 3], 1, &[]), None);
    }

    #[test]
    fn on_a_tie_the_later_candidate_wins() {
        let candidates = [
            Candidate { id: 5, center: [2.0, 0.0, 0.0] },
            Candidate { id: 6, center: [0.0, 2.0, 0.0] },
        ];
        assert_eq!(nearest_victim([0.0; 3], 1, &candidates), Some(6));
    }

    #[test]
    fn the_line_has_a_point_every_quarter_block() {
        let points = line_points([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], [0.0, 0.0, 0.0]);
        assert!((points[4][0] - 1.0).abs() < 1e-9);
        // 2.1 blocks: ceil(8.4) = 9 steps, 10 points.
        assert_eq!(line_points([0.0; 3], [2.1, 0.0, 0.0]).len(), 10);
        // Two points on the spot: at least 2 steps, 3 points.
        assert_eq!(line_points([1.0; 3], [1.0; 3]).len(), 3);
    }

    #[test]
    fn the_echo_numbers_are_the_java_numbers() {
        assert_eq!(ECHO_COLOR, [0.1, 0.9, 1.0]);
        assert_eq!(ECHO_SOUND, Sound { id: "minecraft:block.amethyst_block.resonate", volume: 0.6, pitch: 1.6 });
        assert_eq!(ECHO_SPARKS.count, 6);
        assert_eq!(ECHO_SPARKS.spread, [0.2, 0.2, 0.2]);
        assert_eq!(ECHO_SPARKS.speed, 0.1);
        assert_eq!(DELAY_TICKS, 5);
        assert_eq!(RANGE, 5.0);
        assert_eq!(MAX_PENDING_ECHOES, 8);
    }

    #[test]
    fn the_enemies_are_hostile_mobs_only() {
        for kind in ["minecraft:zombie", "minecraft:skeleton", "minecraft:creeper", "minecraft:ender_dragon", "minecraft:slime"] {
            assert!(is_enemy(kind), "{kind}");
        }
        // Players, villagers, armor stands and animals are no Enemy.
        for kind in ["minecraft:player", "minecraft:villager", "minecraft:armor_stand", "minecraft:cow", "minecraft:iron_golem", "minecraft:wolf", ""] {
            assert!(!is_enemy(kind), "{kind}");
        }
    }

    // ---- Echo Sense ----

    #[test]
    fn the_ore_tag_is_the_json_file() {
        // data/bmh/tags/block/echo_sense_ores.json: eight families, quartz, nether gold, ancient debris.
        for block in [
            "minecraft:coal_ore", "minecraft:deepslate_coal_ore", "minecraft:copper_ore", "minecraft:iron_ore",
            "minecraft:gold_ore", "minecraft:nether_gold_ore", "minecraft:redstone_ore", "minecraft:lapis_ore",
            "minecraft:diamond_ore", "minecraft:deepslate_emerald_ore", "minecraft:nether_quartz_ore",
            "minecraft:ancient_debris",
        ] {
            assert!(listed_in_tag(block, ORE_TAG), "{block}");
        }
        for block in ["minecraft:stone", "minecraft:raw_iron_block", "minecraft:iron_block", "minecraft:air", ""] {
            assert!(!listed_in_tag(block, ORE_TAG), "{block}");
        }
        assert_eq!(ORE_TAG, "bmh:echo_sense_ores");
        // The written families answer for their own tag only.
        assert!(listed_in_tag("minecraft:deepslate_iron_ore", "minecraft:iron_ores"));
        assert!(!listed_in_tag("minecraft:deepslate_iron_ore", "minecraft:gold_ores"));
        assert!(listed_in_tag("minecraft:nether_gold_ore", "minecraft:gold_ores"));
        assert!(!listed_in_tag("minecraft:iron_ore", "minecraft:mineable/pickaxe"));
    }

    #[test]
    fn the_same_ore_is_the_same_family() {
        let same = |broken: &str, other: &str| same_ore(broken, other, listed_in_tag);
        assert!(same("minecraft:iron_ore", "minecraft:deepslate_iron_ore"));
        assert!(same("minecraft:deepslate_iron_ore", "minecraft:iron_ore"));
        assert!(!same("minecraft:iron_ore", "minecraft:gold_ore"));
        // Nether gold ore is in the gold family.
        assert!(same("minecraft:nether_gold_ore", "minecraft:gold_ore"));
        assert!(same("minecraft:gold_ore", "minecraft:nether_gold_ore"));
        // Quartz and ancient debris match their own block only.
        assert!(same("minecraft:nether_quartz_ore", "minecraft:nether_quartz_ore"));
        assert!(!same("minecraft:nether_quartz_ore", "minecraft:nether_gold_ore"));
        assert!(same("minecraft:ancient_debris", "minecraft:ancient_debris"));
    }

    #[test]
    fn a_data_pack_ore_follows_the_tags_of_the_server() {
        // A data pack puts test:tin_ore in the iron family. The server reads the tags, so it is iron.
        let in_tag = |block: &str, tag: &str| {
            listed_in_tag(block, tag) || (block == "test:tin_ore" && (tag == "minecraft:iron_ores" || tag == ORE_TAG))
        };
        assert!(same_ore("test:tin_ore", "minecraft:iron_ore", in_tag));
        assert!(same_ore("minecraft:deepslate_iron_ore", "test:tin_ore", in_tag));
        // The first family that holds the broken block wins, as the Java loop does.
        let both = |block: &str, tag: &str| {
            listed_in_tag(block, tag) || (block == "minecraft:coal_ore" && tag == "minecraft:iron_ores")
        };
        assert!(same_ore("minecraft:coal_ore", "minecraft:deepslate_coal_ore", both));
        assert!(!same_ore("minecraft:coal_ore", "minecraft:iron_ore", both));
    }

    #[test]
    fn the_sphere_holds_the_blocks_within_eight() {
        let offsets = sphere();
        // The Gauss count of lattice points with x2 + y2 + z2 <= 64 is 2109.
        assert_eq!(offsets.len(), 2109);
        assert!(offsets.contains(&[8, 0, 0]));
        assert!(offsets.contains(&[0, -8, 0]));
        assert!(!offsets.contains(&[8, 1, 0]));
        assert!(!offsets.contains(&[5, 5, 5]));
        assert!(offsets.contains(&[4, 4, 4]));
    }

    #[test]
    fn the_ores_come_nearest_first_and_the_centre_is_left_out() {
        let ores = [[3, 0, 0], [1, 0, 0], [0, 2, 0], [20, 0, 0]];
        let same = |name: &str| same_ore("minecraft:iron_ore", name, listed_in_tag);
        let found = find_same_ore([0, 0, 0], |pos| {
            if ores.contains(&pos) || pos == [0, 0, 0] {
                Some("minecraft:deepslate_iron_ore".to_owned())
            } else if pos == [0, 0, 1] {
                Some("minecraft:gold_ore".to_owned())
            } else if pos == [0, 0, 2] {
                None
            } else {
                Some("minecraft:stone".to_owned())
            }
        }, same);
        assert_eq!(found, vec![[1, 0, 0], [0, 2, 0], [3, 0, 0]]);
    }

    #[test]
    fn a_reading_adds_the_new_glows_and_extends_the_old_ones() {
        let mut book = GlowBook::default();
        let found = [[1, 0, 0], [2, 0, 0]];
        let first = read_ores(&mut book, "minecraft:overworld", [0, 0, 0], &found, 1000);
        assert_eq!(first.spawned, vec![[1, 0, 0], [2, 0, 0]]);
        assert!(first.extended.is_empty());
        assert_eq!(book.len(), 2);
        // A second reading meets the same glows: they only last longer.
        let second = read_ores(&mut book, "minecraft:overworld", [0, 0, 0], &[[1, 0, 0], [3, 0, 0]], 1050);
        assert_eq!(second.extended, vec![[1, 0, 0]]);
        assert_eq!(second.spawned, vec![[3, 0, 0]]);
        assert_eq!(second.positions(), vec![[3, 0, 0], [1, 0, 0]]);
        // The glow at [2,0,0] ends at 1100, the two others at 1150.
        book.expire(1099);
        assert_eq!(book.len(), 3);
        book.expire(1100);
        assert_eq!(book.len(), 2);
        book.expire(1150);
        assert!(book.is_empty());
    }

    #[test]
    fn at_most_64_glows_live_around_a_spot() {
        let mut book = GlowBook::default();
        let found: Vec<[i32; 3]> = (0..100).map(|i| [i % 5, i / 25, (i / 5) % 5]).collect();
        let unique: std::collections::BTreeSet<[i32; 3]> = found.iter().copied().collect();
        let found: Vec<[i32; 3]> = unique.into_iter().collect();
        assert_eq!(found.len(), 100);
        let reading = read_ores(&mut book, "minecraft:overworld", [0, 0, 0], &found, 0);
        assert_eq!(reading.spawned.len(), 64);
        assert_eq!(book.len(), 64);
        // No room is left for a new glow, the old ones are extended.
        let again = read_ores(&mut book, "minecraft:overworld", [0, 0, 0], &found, 10);
        assert!(again.spawned.is_empty());
        assert_eq!(again.extended.len(), 64);
    }

    #[test]
    fn a_glow_in_another_dimension_does_not_count() {
        let mut book = GlowBook::default();
        book.set("minecraft:the_nether", [1, 0, 0], 500);
        assert!(book.near("minecraft:overworld", [0, 0, 0]).is_empty());
        assert_eq!(book.near("minecraft:the_nether", [0, 0, 0]), vec![[1, 0, 0]]);
        // Farther than RADIUS + 1 is out of reach.
        assert!(!book.near("minecraft:the_nether", [10, 0, 0]).is_empty());
        assert!(book.near("minecraft:the_nether", [11, 0, 0]).is_empty());
    }

    #[test]
    fn a_glow_whose_ore_is_gone_is_forgotten() {
        let mut book = GlowBook::default();
        book.set("minecraft:overworld", [1, 0, 0], 500);
        book.set("minecraft:overworld", [2, 0, 0], 500);
        book.set("minecraft:overworld", [40, 0, 0], 500);
        book.forget_where("minecraft:overworld", [0, 0, 0], |pos| pos == [2, 0, 0]);
        // [1,0,0] is gone, [2,0,0] is kept, [40,0,0] is out of reach and kept.
        assert_eq!(book.len(), 2);
    }

    #[test]
    fn the_glow_box_sits_a_hair_inside_the_block() {
        let (min, max) = glow_box([10, 20, 30]);
        assert!((min[0] - 10.02).abs() < 1e-9);
        assert!((max[0] - 10.98).abs() < 1e-9);
        assert!((max[1] - 20.98).abs() < 1e-9);
        assert_eq!(GLOW_ARGB, 0xFF00F0FF);
    }

    #[test]
    fn a_player_sees_a_glow_in_the_dimension_within_range() {
        assert!(sees_glow("minecraft:overworld", [0.0, 64.0, 0.0], "minecraft:overworld", [10, 64, 0]));
        assert!(!sees_glow("minecraft:the_nether", [0.0, 64.0, 0.0], "minecraft:overworld", [10, 64, 0]));
        assert!(!sees_glow("minecraft:overworld", [0.0, 64.0, 0.0], "minecraft:overworld", [200, 64, 0]));
    }

    #[test]
    fn the_sense_numbers_are_the_java_numbers() {
        assert_eq!(RADIUS, 8);
        assert_eq!(MAX_DISPLAYS, 64);
        assert_eq!(LIFETIME_TICKS, 100);
        assert_eq!(COOLDOWN_TICKS, 4);
        assert_eq!(GLOW_COLOR, 0x00F0FF);
        assert_eq!(SENSE_SOUND, Sound { id: "minecraft:block.amethyst_block.chime", volume: 0.8, pitch: 0.7 });
    }
}
