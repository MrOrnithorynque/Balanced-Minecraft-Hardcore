// Area amulet (owner: the amulet agent). Log: docs/horizon/log/amulet.md.
// The Navigator's Amulet with no game type: the sky rules of a star reading, the charges and their
// nightly refill, the reading that goes on the stack, the named points and their checks, the
// `/bmh point` command line, the cooldowns, and the layout and text field of the two screens.
// Java reference: items/NavigatorAmuletItem, handlers/AmuletReading, handlers/AmuletPoints,
// handlers/AmuletPointCommand, handlers/AmuletPointSignal, client/amulet/PointScreen,
// client/amulet/PointListScreen, client/amulet/PointSignal.

use super::contract::{AmuletPoint, AmuletReading};

// ---------------------------------------------------------------------------------------------
// Ids, texts and sounds
// ---------------------------------------------------------------------------------------------

/// The item.
pub const AMULET: &str = super::ids::NAVIGATOR_AMULET;

/// The colour of the starlight line and of the charge bar (NavigatorAmuletItem.java:41).
pub const STARLIGHT_COLOR: u32 = 0xB5_8CFF;

/// The chime of a reading and of a saved point (NavigatorAmuletItem.java:131-132,
/// AmuletPointCommand.java:124-125, `SoundEvents.AMETHYST_BLOCK_CHIME`, `SoundSource.PLAYERS`).
pub const CHIME_SOUND: &str = "minecraft:block.amethyst_block.chime";
/// The sound category of the chime (`SoundSource.PLAYERS`).
pub const CHIME_CATEGORY: &str = "player";
/// The chime volume (NavigatorAmuletItem.java:132, AmuletPointCommand.java:125).
pub const CHIME_VOLUME: f32 = 1.0;
/// The pitch of a reading's chime (NavigatorAmuletItem.java:132).
pub const READING_PITCH: f32 = 1.0;
/// The pitch of a saved point's chime (AmuletPointCommand.java:125).
pub const SAVE_PITCH: f32 = 0.7;

/// The advancement of a star reading (AmuletReading.java:247).
pub const STARGAZER: &str = "navigation/stargazer";
/// The advancement of a named point (AmuletPoints.java:144).
pub const NAMED_PLACE: &str = "navigation/named_place";
/// The advancement of three points (AmuletPoints.java:147).
pub const OWN_SKY: &str = "navigation/own_sky";

/// Tooltip lines (NavigatorAmuletItem.java:153-157).
pub const KEY_STARLIGHT: &str = "bmh.amulet.starlight";
pub const KEY_TOOLTIP_READ: &str = "bmh.amulet.tooltip.read";
pub const KEY_TOOLTIP_HOME: &str = "bmh.amulet.tooltip.home";
pub const KEY_TOOLTIP_LIST: &str = "bmh.amulet.tooltip.list";
/// The action bar line of a reading, with x, y and z (NavigatorAmuletItem.java:133).
pub const KEY_READING: &str = "bmh.amulet.reading";
/// No charge left (NavigatorAmuletItem.java:123).
pub const KEY_EMPTY: &str = "bmh.amulet.empty";
/// The prefix of the sky refusals (AmuletReading.java:103).
pub const HIDDEN_PREFIX: &str = "bmh.amulet.hidden";

/// The texts of the points (AmuletPointCommand.java, PointScreen.java, PointListScreen.java).
pub const KEY_NO_AMULET: &str = "bmh.point.no_amulet";
pub const KEY_OVERWORLD: &str = "bmh.point.overworld";
pub const KEY_TOO_FAR: &str = "bmh.point.too_far";
pub const KEY_NO_NAME: &str = "bmh.point.no_name";
pub const KEY_FULL: &str = "bmh.point.full";
pub const KEY_SAVED: &str = "bmh.point.saved";
pub const KEY_RENAMED: &str = "bmh.point.renamed";
pub const KEY_REMOVED: &str = "bmh.point.removed";
pub const KEY_SCREEN_NEW: &str = "bmh.point.screen.new";
pub const KEY_SCREEN_EDIT: &str = "bmh.point.screen.edit";
pub const KEY_SCREEN_SAVE: &str = "bmh.point.screen.save";
pub const KEY_SCREEN_REMOVE: &str = "bmh.point.screen.remove";
pub const KEY_SCREEN_BED: &str = "bmh.point.screen.bed";
pub const KEY_SCREEN_LODESTONE: &str = "bmh.point.screen.lodestone";
pub const KEY_SCREEN_COUNT: &str = "bmh.point.screen.count";
pub const KEY_LIST_TITLE: &str = "bmh.point.list.title";
pub const KEY_LIST_COUNT: &str = "bmh.point.list.count";
pub const KEY_LIST_EMPTY: &str = "bmh.point.list.empty";
pub const KEY_LIST_SPAWN: &str = "bmh.point.list.spawn";
/// Vanilla's button texts (PointScreen.java:85, PointListScreen.java:92).
pub const KEY_CANCEL: &str = "gui.cancel";
pub const KEY_DONE: &str = "gui.done";

/// The id of the amulet's own screen (`native_mod_screen_open`). It is not the journey's `bmh`, so the
/// two screens never take each other's input.
pub const SCREEN_ID: &str = "bmh:amulet";

// ---------------------------------------------------------------------------------------------
// Charges and the stack data (AmuletReading.java)
// ---------------------------------------------------------------------------------------------

/// The charges of a full amulet (AmuletReading.java:44).
pub const MAX_CHARGES: i32 = 3;

/// The keys in the stack's `minecraft:custom_data` (AmuletReading.java:46-52). The Java layout is kept,
/// so an amulet from a Java world keeps its charges and its last reading.
pub const READING_TAG: &str = "bmh_reading";
pub const AZIMUTHS_TAG: &str = "azimuths";
pub const NAMES_TAG: &str = "names";
pub const SPAWN_TAG: &str = "spawn";
pub const TIME_TAG: &str = "time";
pub const CHARGES_TAG: &str = "bmh_starlight";
pub const NIGHT_TAG: &str = "bmh_night";
/// The use stamp of the amulet of 1.4.0 (AmuletReading.java:55).
pub const LEGACY_LAST_USE_TAG: &str = "LastUseTime";
/// The enchantments that an amulet of 1.4.0 can carry and that do nothing now (AmuletReading.java:175-180).
pub const USELESS_ENCHANTMENTS: [&str; 2] = ["minecraft:unbreaking", "minecraft:mending"];

/// One day in ticks (AmuletReading.java:57).
pub const DAY_LENGTH: i64 = 24_000;
/// The night index goes up at this time of the day (AmuletReading.java:59).
pub const NIGHTFALL: i64 = 12_000;
/// The refill runs once a second, at a game time that is a multiple of this (NavigatorAmuletItem.java:143).
pub const REFILL_PERIOD: i64 = 20;

/// The cooldown after a reading (NavigatorAmuletItem.java:44).
pub const READING_COOLDOWN_TICKS: i64 = 40;
/// The cooldown after the naming screen or the list opens (NavigatorAmuletItem.java:46).
pub const POINT_COOLDOWN_TICKS: i64 = 20;

/// What the stack's custom data says about the charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StackCharges {
    /// `bmh_starlight`, when the stack has it.
    pub charges: Option<i32>,
    /// `bmh_night`, when the stack has it.
    pub night: Option<i64>,
}

/// Charges left on the stack, 0 to 3; a stack that never read is full (AmuletReading.java:148-157).
pub fn charges(stack: StackCharges) -> i32 {
    stack.charges.unwrap_or(MAX_CHARGES).clamp(0, MAX_CHARGES)
}

/// The night that a day time belongs to: it goes up at each nightfall (AmuletReading.java:90-93).
pub fn night_index(day_time: i64) -> i64 {
    (day_time + DAY_LENGTH - NIGHTFALL).div_euclid(DAY_LENGTH)
}

/// The new charges and night when a new night has begun since the stack counted its charges, else None
/// (AmuletReading.java:185-204). A stack that never read has no night and is left alone.
pub fn refill(stack: StackCharges, day_time: i64) -> Option<StackCharges> {
    let counted = stack.night?;
    let night = night_index(day_time);
    (counted != night).then_some(StackCharges { charges: Some(MAX_CHARGES), night: Some(night) })
}

/// True on the ticks where the amulet refills (NavigatorAmuletItem.java:143, `getGameTime() % 20 == 0`).
pub fn refill_tick(game_time: i64) -> bool {
    game_time % REFILL_PERIOD == 0
}

/// The width of the charge bar, 0 to 13 (NavigatorAmuletItem.java:170, `Math.round(13F * charges / 3)`).
pub fn bar_width(charges: i32) -> i32 {
    java_round(13.0 * charges as f32 / MAX_CHARGES as f32)
}

/// Java `Math.round(float)`: the nearest integer, a half rounds up.
fn java_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

// ---------------------------------------------------------------------------------------------
// The sky rules (AmuletReading.java:111-145)
// ---------------------------------------------------------------------------------------------

/// What the sky rules read of the world and the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sky {
    /// The player is in the Overworld.
    pub overworld: bool,
    /// The rain and thunder levels, 0 to 1 (`Level.getRainLevel`, `getThunderLevel`).
    pub rain: f32,
    pub thunder: f32,
    /// The world's time of day in ticks (`Level.getOverworldClockTime`).
    pub day_time: i64,
    /// The sky light at the block of the eyes, 0 to 15 (`LightLayer.SKY`).
    pub sky_light: u8,
    /// Blindness or Darkness.
    pub blinded: bool,
    /// The eyes are in water or lava, or in powder snow.
    pub eyes_covered: bool,
}

/// Vanilla `Level.isRaining`: a rain level above 0.2 (Horizon's native_level_world.rs:10 too).
pub fn is_raining(rain: f32) -> bool {
    rain > 0.2
}

/// Vanilla `Mth.cos`: the lookup table of 65536 sines.
fn mth_cos(value: f32) -> f32 {
    let index = ((value * 10_430.378 + 16_384.0) as i32) & 65_535;
    (f64::from(index) * std::f64::consts::PI * 2.0 / 65_536.0).sin() as f32
}

/// Vanilla `DimensionType.timeOfDay`: the part of a full turn of the sun, 0 to 1.
fn time_of_day(day_time: i64) -> f32 {
    let x = day_time as f64 / DAY_LENGTH as f64 - 0.25;
    let d = x - x.floor();
    let e = 0.5 - (d * std::f64::consts::PI).cos() / 2.0;
    (d * 2.0 + e) as f32 / 3.0
}

/// Vanilla `Level.updateSkyBrightness` (1.21.1): how much the sky darkens, 0 at noon, 11 at night.
/// Horizon computes it from the 26.3 timeline data (`Game::sky_darken`), which no helper exposes; this
/// is the formula that the timeline replaced. Horizon's own tests give the same answers for noon, rainy
/// noon, a thunderstorm at noon, midnight and the morning (horizon-client-render environment.rs).
pub fn sky_darken(day_time: i64, rain: f32, thunder: f32) -> i32 {
    let d = 1.0 - f64::from(rain * 5.0) / 16.0;
    let e = 1.0 - f64::from(thunder * 5.0) / 16.0;
    let turn = time_of_day(day_time) * (std::f64::consts::PI * 2.0) as f32;
    let f = 0.5 + 2.0 * f64::from(mth_cos(turn)).clamp(-0.25, 0.25);
    ((1.0 - f * d * e) * 11.0) as i32
}

/// Vanilla `Level.isDarkOutside`: the sky darkens by 4 or more. The bed of Horizon uses the same test
/// (game.rs, `sky_darken() >= 4`).
pub fn is_dark_outside(day_time: i64, rain: f32, thunder: f32) -> bool {
    sky_darken(day_time, rain, thunder) >= 4
}

/// The lang key that says why the stars cannot be read, or None when they can (AmuletReading.java:111-132).
/// The order of the tests is Java's: dimension, rain, day, roof. `prefix` is `bmh.amulet.hidden`.
pub fn hidden_reason(sky: &Sky, prefix: &str) -> Option<String> {
    let reason = if !sky.overworld {
        "dimension"
    } else if is_raining(sky.rain) {
        "rain"
    } else if !is_dark_outside(sky.day_time, sky.rain, sky.thunder) {
        "day"
    } else if sky.sky_light < 15 || sky.blinded || sky.eyes_covered {
        // Vanilla `canSeeSky`: full sky light at the block (Horizon's mobs/spawn_rules.rs:108).
        "roof"
    } else {
        return None;
    };
    Some(format!("{prefix}.{reason}"))
}

/// The effects that hide the sky (AmuletReading.java:140-141).
pub const BLINDING_EFFECTS: [&str; 2] = ["minecraft:blindness", "minecraft:darkness"];

/// The eyes height that vanilla's `updateFluidOnEyes` tests: a little under the eyes.
pub const FLUID_EYE_OFFSET: f64 = 0.111_111_11;

/// True when a block holds water that covers the eyes: water itself, a bubble column, the plants that
/// are always in water, and a waterlogged block (vanilla `isEyeInFluid(FluidTags.WATER)`).
pub fn holds_water(block: &str, waterlogged: bool) -> bool {
    waterlogged
        || matches!(
            block,
            "minecraft:water"
                | "minecraft:bubble_column"
                | "minecraft:kelp"
                | "minecraft:kelp_plant"
                | "minecraft:seagrass"
                | "minecraft:tall_seagrass"
        )
}

/// True when a block is lava (vanilla `isEyeInFluid(FluidTags.LAVA)`).
pub fn holds_lava(block: &str) -> bool {
    block == "minecraft:lava"
}

/// True when a block is powder snow (AmuletReading.java:144).
pub fn is_powder_snow(block: &str) -> bool {
    block == "minecraft:powder_snow"
}

// ---------------------------------------------------------------------------------------------
// A reading (AmuletReading.java:63-74, 210-251)
// ---------------------------------------------------------------------------------------------

/// Degrees clockwise from north, in [0, 360): 0 is north (-Z), 90 east (+X) (AmuletReading.java:67-74).
pub fn azimuth(from_x: f64, from_z: f64, to_x: f64, to_z: f64) -> f32 {
    let degrees = (to_x - from_x).atan2(from_z - to_z).to_degrees();
    let azimuth = ((degrees % 360.0 + 360.0) % 360.0) as f32;
    // A tiny negative angle rounds up to 360 once it is a float.
    if azimuth >= 360.0 { 0.0 } else { azimuth }
}

/// The reading that the stack keeps (AmuletReading.java:210-245): one azimuth and one name per point,
/// toward the middle of the block, then the world spawn last with no name. `spawn` is None when the
/// server does not know the world spawn: then the reading has no spawn line.
pub fn reading(
    player: [f64; 3],
    points: &[AmuletPoint],
    spawn: Option<[i32; 3]>,
    game_time: i64,
) -> AmuletReading {
    let toward = |pos: [i32; 3]| azimuth(player[0], player[2], f64::from(pos[0]) + 0.5, f64::from(pos[2]) + 0.5);
    let mut azimuths: Vec<f32> = points.iter().map(|p| toward(p.pos)).collect();
    let mut names: Vec<String> = points.iter().map(|p| p.name.clone()).collect();
    let spawn = spawn.map(|pos| {
        let index = azimuths.len();
        azimuths.push(toward(pos));
        names.push(String::new());
        index
    });
    AmuletReading { azimuths, names, spawn, time: game_time }
}

/// The charges after a reading: one less, or None when the reading is free (creative,
/// AmuletReading.java:229-244: a free reading writes no charge and no night).
pub fn charges_after_reading(stack: StackCharges, creative: bool, day_time: i64) -> Option<StackCharges> {
    (!creative).then(|| StackCharges { charges: Some(charges(stack) - 1), night: Some(night_index(day_time)) })
}

/// The block of a position (`Entity.blockPosition`).
pub fn block_of(pos: [f64; 3]) -> [i32; 3] {
    pos.map(|v| v.floor() as i32)
}

// ---------------------------------------------------------------------------------------------
// Cooldowns and the order of a use (NavigatorAmuletItem.java:60-137)
// ---------------------------------------------------------------------------------------------

/// A cooldown of the amulet for one player: the game time it started and its length. Java keeps it per
/// item (`ItemCooldowns`); Horizon has no setter, so the mod keeps its own timer and draws the sweep
/// itself (GAPS.md A4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cooldown {
    pub start: i64,
    pub ticks: i64,
}

impl Cooldown {
    /// True while the cooldown runs. A game time before its start (another world) ends it.
    pub fn active(self, now: i64) -> bool {
        now >= self.start && now < self.start.saturating_add(self.ticks)
    }

    /// What is left of the cooldown, from 1 down to 0 (`ItemCooldowns.getCooldownPercent`). `partial` is
    /// the part of the current tick that has passed, from 0 to 1.
    pub fn remaining(self, now: i64, partial: f32) -> f32 {
        if !self.active(now) || self.ticks <= 0 {
            return 0.0;
        }
        let end = self.start.saturating_add(self.ticks);
        let left = (end - now) as f32 - partial.clamp(0.0, 1.0);
        (left / self.ticks as f32).clamp(0.0, 1.0)
    }
}

/// The white shade of a cooldown on an item icon, in GUI pixels (`GuiGraphicsExtractor.itemCooldown`):
/// `(top, height)` from the icon's top. None when nothing is left to show.
pub fn sweep_rows(remaining: f32) -> Option<(i32, i32)> {
    if remaining.is_nan() || remaining <= 0.0 {
        return None;
    }
    let remaining = remaining.min(1.0);
    let top = (16.0 * (1.0 - remaining)).floor() as i32;
    let height = (16.0 * remaining).ceil() as i32;
    Some((top, height))
}

/// The colour of the sweep: white at half opacity (`Integer.MAX_VALUE` = 0x7FFFFFFF in vanilla).
pub const SWEEP_COLOR: u32 = 0x7FFF_FFFF;

/// What a sneaking use on a block does (NavigatorAmuletItem.java:60-88).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseOn {
    /// Not the amulet's click: the use goes on (Java PASS).
    Pass,
    /// The naming screen of this block opens, with a cooldown (Java SUCCESS, server side).
    Open,
    /// Points are only in the Overworld: the action bar says so (Java SUCCESS).
    NotOverworld,
}

/// The order of `useOn`: an item on cooldown is never used, then sneak and a block that holds a point,
/// then the dimension.
pub fn use_on(on_cooldown: bool, sneaking: bool, can_hold: bool, overworld: bool) -> UseOn {
    if on_cooldown || !sneaking || !can_hold {
        UseOn::Pass
    } else if overworld {
        UseOn::Open
    } else {
        UseOn::NotOverworld
    }
}

/// What a use in the air does (NavigatorAmuletItem.java:92-137).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Use {
    /// Nothing: an item on cooldown, or the second packet of a click that a use on a block took.
    Ignore,
    /// The list of the points opens, with a cooldown.
    List,
    /// The sky is hidden: the action bar gives this key (Java FAIL).
    Hidden(String),
    /// No charge left (Java FAIL).
    Empty,
    /// A reading.
    Read,
}

/// The order of `use`. `took_click`: a use on a block of this click was taken (Java returns SUCCESS
/// from `useOn` and never calls `use`). `charges` is the count after the refill.
pub fn use_in_air(
    on_cooldown: bool,
    took_click: bool,
    sneaking: bool,
    hidden: Option<String>,
    charges: i32,
    creative: bool,
) -> Use {
    if on_cooldown || took_click {
        Use::Ignore
    } else if sneaking {
        Use::List
    } else if let Some(key) = hidden {
        Use::Hidden(key)
    } else if charges <= 0 && !creative {
        Use::Empty
    } else {
        Use::Read
    }
}

/// True when a `use_item` belongs to the same click as a `use_item_on` that the amulet took at game
/// time `took_at`. The client sends UseItem right after UseItemOn (remote.rs, `remote_use_hands`); one
/// tick of margin covers a packet that the server reads in the next tick. A held click repeats after 4
/// ticks at the earliest, so a new click is never dropped.
pub fn same_click(took_at: Option<i64>, now: i64) -> bool {
    took_at.is_some_and(|at| now >= at && now - at <= 1)
}

// ---------------------------------------------------------------------------------------------
// The points (AmuletPoints.java)
// ---------------------------------------------------------------------------------------------

/// The longest name (AmuletPoints.java:32), in Java characters (UTF-16 units).
pub const MAX_NAME_LENGTH: usize = 24;
/// The points that earn "A sky of your own" (AmuletPoints.java:35).
pub const OWN_SKY_POINTS: usize = 3;
/// The key of the points in the player data of a Java world (AmuletPoints.java:37).
pub const POINTS_TAG: &str = "bmh_points";
/// The Home of 1.5.0 to 1.6.x in the player data (AmuletPoints.java:39-40).
pub const LEGACY_HOME_TAG: &str = "bmh_home";
pub const LEGACY_HOME_NAME: &str = "Home";

/// A player may stand this far from the block of a change (AmuletPointCommand.java:38).
pub const MAX_DISTANCE: f64 = 8.0;
/// At most one change in this many ticks (AmuletPointCommand.java:39).
pub const MIN_TICKS_BETWEEN_CHANGES: i64 = 20;

/// The lodestone (`Blocks.LODESTONE`, AmuletPoints.java:102).
pub const LODESTONE: &str = "minecraft:lodestone";
/// The bed tag (`BlockTags.BEDS`).
pub const BEDS_TAG: &str = "minecraft:beds";

/// The sixteen vanilla beds: the bed tag when the server cannot read its tags.
pub fn is_vanilla_bed(block: &str) -> bool {
    const COLOURS: [&str; 16] = [
        "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan",
        "purple", "blue", "brown", "green", "red", "black",
    ];
    block
        .strip_prefix("minecraft:")
        .and_then(|path| path.strip_suffix("_bed"))
        .is_some_and(|colour| COLOURS.contains(&colour))
}

/// One half of a bed: its `part` and its `facing` properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedHalf {
    /// True for the foot (`BedPart.FOOT`).
    pub foot: bool,
    /// The block step of `facing`.
    pub facing: [i32; 3],
}

/// The step of a horizontal `facing` value.
pub fn facing_step(facing: &str) -> Option<[i32; 3]> {
    match facing {
        "north" => Some([0, 0, -1]),
        "south" => Some([0, 0, 1]),
        "west" => Some([-1, 0, 0]),
        "east" => Some([1, 0, 0]),
        _ => None,
    }
}

/// A bed half from its properties, or None when they are missing.
pub fn bed_half(part: Option<&str>, facing: Option<&str>) -> Option<BedHalf> {
    let foot = match part? {
        "foot" => true,
        "head" => false,
        _ => return None,
    };
    Some(BedHalf { foot, facing: facing_step(facing?)? })
}

fn step(pos: [i32; 3], by: [i32; 3], sign: i32) -> [i32; 3] {
    [pos[0] + sign * by[0], pos[1] + sign * by[1], pos[2] + sign * by[2]]
}

/// The block a point keeps: the head of a bed, else the block itself (AmuletPoints.java:106-113).
pub fn anchor(pos: [i32; 3], bed: Option<BedHalf>) -> [i32; 3] {
    match bed {
        Some(half) if half.foot => step(pos, half.facing, 1),
        _ => pos,
    }
}

/// The other half of a bed, or the block itself (AmuletPoints.java:119, `BedBlock.getConnectedDirection`).
pub fn other_half(pos: [i32; 3], bed: Option<BedHalf>) -> [i32; 3] {
    match bed {
        Some(half) if half.foot => step(pos, half.facing, 1),
        Some(half) => step(pos, half.facing, -1),
        None => pos,
    }
}

/// The index of the point on this block: a bed matches on either half (AmuletPoints.java:117-128).
pub fn point_at(points: &[AmuletPoint], pos: [i32; 3], bed: Option<BedHalf>) -> Option<usize> {
    let other = other_half(pos, bed);
    points.iter().position(|p| p.pos == pos || p.pos == other)
}

/// The points after a save on this block: the point there is renamed, or a new point is added at the
/// anchor (AmuletPoints.java:131-149). The caller checks the cap first. True when it was a rename.
pub fn put(points: &mut Vec<AmuletPoint>, pos: [i32; 3], bed: Option<BedHalf>, name: &str) -> bool {
    match point_at(points, pos, bed) {
        Some(index) => {
            if let Some(point) = points.get_mut(index) {
                point.name = name.to_owned();
            }
            true
        }
        None => {
            points.push(AmuletPoint { name: name.to_owned(), pos: anchor(pos, bed) });
            false
        }
    }
}

/// The advancements that a save earns, with the count of points after it (AmuletPoints.java:144-148).
pub fn awards_after_put(count: usize) -> Vec<&'static str> {
    let mut awards = vec![NAMED_PLACE];
    if count >= OWN_SKY_POINTS {
        awards.push(OWN_SKY);
    }
    awards
}

/// Removes the point on this block and gives it back (AmuletPoints.java:152-165).
pub fn remove_on_block(points: &mut Vec<AmuletPoint>, pos: [i32; 3], bed: Option<BedHalf>) -> Option<AmuletPoint> {
    let index = point_at(points, pos, bed)?;
    (index < points.len()).then(|| points.remove(index))
}

/// Removes every point at exactly this position and gives back the last of them, whose name the action
/// bar shows (AmuletPointCommand.java:73-85 picks the last match, AmuletPoints.java:87-97 removes all).
pub fn remove_exact(points: &mut Vec<AmuletPoint>, pos: [i32; 3]) -> Option<AmuletPoint> {
    let last = points.iter().rev().find(|p| p.pos == pos).cloned()?;
    points.retain(|p| p.pos != pos);
    Some(last)
}

/// The points of a player after the import of the Java points ran late. The Java points come first, in
/// their order. Then come the points that the player made before the import, except a point that is in
/// both lists (same name and same position), so no point is lost and none is doubled. Java has no
/// import: this is the port's own rule (docs/horizon/log/amulet.md, step 10).
pub fn merge_imported(java: &[AmuletPoint], made: &[AmuletPoint]) -> Vec<AmuletPoint> {
    let mut all = java.to_vec();
    for point in made {
        if !java.contains(point) {
            all.push(point.clone());
        }
    }
    all
}

/// The name as the server keeps it (AmuletPoints.java:171-190): no formatting code (the section sign and
/// the character after it), no control character, no space at the ends, at most 24 characters. Java
/// counts UTF-16 units: so does this, and it never cuts a character in two.
pub fn clean_name(raw: &str) -> String {
    let mut name = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\u{a7}' {
            chars.next();
        } else if !c.is_control() {
            name.push(c);
        }
    }
    let trimmed = java_trim(&name);
    if utf16_len(trimmed) <= MAX_NAME_LENGTH {
        return trimmed.to_owned();
    }
    java_trim(&cut_utf16(trimmed, MAX_NAME_LENGTH)).to_owned()
}

/// Java `String.trim`: the characters up to U+0020 at both ends.
fn java_trim(text: &str) -> &str {
    text.trim_matches(|c: char| c <= ' ')
}

/// The length of a text in UTF-16 units (Java `String.length`).
pub fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

/// The start of a text that holds at most `units` UTF-16 units, whole characters only.
fn cut_utf16(text: &str, units: usize) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|c| {
            used += c.len_utf16();
            used <= units
        })
        .collect()
}

/// Why the server refuses a change of a point (AmuletPointCommand.java:154-177).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No amulet in either hand.
    NoAmulet,
    /// Not in the Overworld.
    Overworld,
    /// Dead, a spectator, too far, or the block cannot hold a point.
    TooFar,
    /// A change less than a second ago: refused with no message.
    TooSoon,
}

impl Refusal {
    /// The action bar key, or None for a refusal with no message.
    pub fn key(self) -> Option<&'static str> {
        match self {
            Refusal::NoAmulet => Some(KEY_NO_AMULET),
            Refusal::Overworld => Some(KEY_OVERWORLD),
            Refusal::TooFar => Some(KEY_TOO_FAR),
            Refusal::TooSoon => None,
        }
    }
}

/// What the server knows of the player who asks for a change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asker {
    pub holds_amulet: bool,
    pub overworld: bool,
    pub alive: bool,
    pub spectator: bool,
    /// The feet (`Entity.position`).
    pub position: [f64; 3],
}

/// True when the player stands within 8 blocks of the middle of the block (`distanceToSqr`).
pub fn within_reach(position: [f64; 3], pos: [i32; 3]) -> bool {
    let d = [0, 1, 2].map(|i| position[i] - (f64::from(pos[i]) + 0.5));
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= MAX_DISTANCE * MAX_DISTANCE
}

/// True when the last change is at least 20 ticks old, or later than now (another world)
/// (AmuletPointCommand.java:173-176).
pub fn change_allowed(last: Option<i64>, now: i64) -> bool {
    last.is_none_or(|last| now - last >= MIN_TICKS_BETWEEN_CHANGES || now < last)
}

/// The checks that a save and a remove share, in Java's order (AmuletPointCommand.java:154-177).
pub fn check_change(asker: &Asker, pos: [i32; 3], can_hold: bool, last: Option<i64>, now: i64) -> Result<(), Refusal> {
    if !asker.holds_amulet {
        return Err(Refusal::NoAmulet);
    }
    if !asker.overworld {
        return Err(Refusal::Overworld);
    }
    if !asker.alive || asker.spectator || !within_reach(asker.position, pos) || !can_hold {
        return Err(Refusal::TooFar);
    }
    if !change_allowed(last, now) {
        return Err(Refusal::TooSoon);
    }
    Ok(())
}

/// Why a save stops after the shared checks (AmuletPointCommand.java:106-120).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStop {
    /// The clean name is empty.
    NoName,
    /// A new point and every place is used. The action bar gives the cap.
    Full,
}

impl SaveStop {
    pub fn key(self) -> &'static str {
        match self {
            SaveStop::NoName => KEY_NO_NAME,
            SaveStop::Full => KEY_FULL,
        }
    }
}

/// The checks of a save after the shared ones: a name, then a free place unless it is a rename.
pub fn check_save(clean: &str, renamed: bool, used: usize, max: i32) -> Result<(), SaveStop> {
    if clean.is_empty() {
        return Err(SaveStop::NoName);
    }
    if !renamed && i64::try_from(used).unwrap_or(i64::MAX) >= i64::from(max) {
        return Err(SaveStop::Full);
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// The command `/bmh point` (AmuletPointCommand.java:43-55)
// ---------------------------------------------------------------------------------------------

/// The operator level of `/bmh point`: 0, as in Java (every player). Horizon gives the whole `/bmh`
/// root the lowest level of its usages (native_server.rs, `command_tree`) and checks only that level.
/// So `/bmh rule` refuses a player below its own level inside the settings area
/// (`bmh_logic::settings::may_run_rule`).
pub const COMMAND_LEVEL: u8 = 0;

/// The usages of `/bmh point`, as the registration reads them.
pub const COMMAND_USAGES: [&str; 3] = [
    "bmh point save <pos:block_pos> <name:text>",
    "bmh point remove <pos:block_pos>",
    "bmh point delete <x:integer> <y:integer> <z:integer>",
];

/// A change of a point, from the command or from a screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Save or rename the point on the block (the name as it was typed).
    Save { pos: [i32; 3], name: String },
    /// Remove the point on the block.
    Remove { pos: [i32; 3] },
    /// Remove the point at these coordinates, with no block needed.
    Delete { pos: [i32; 3] },
}

/// A command line, as the amulet reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointCommand {
    /// Not `bmh point ...`: another area's line.
    NotMine,
    /// A change to make.
    Change(Change),
    /// `bmh point ...` with a word or an argument missing.
    Incomplete,
    /// `bmh point ...` with an argument that is not valid.
    Bad,
}

/// One coordinate of a block position: an integer, or `~` with an optional offset from the player
/// (vanilla `WorldCoordinate.parseInt`). Local `^` coordinates are not read.
fn coordinate(word: &str, base: f64) -> Option<i32> {
    match word.strip_prefix('~') {
        Some("") => Some(base.floor() as i32),
        Some(offset) => {
            let value = (base + offset.parse::<f64>().ok().filter(|v| v.is_finite())?).floor();
            (value >= f64::from(i32::MIN) && value <= f64::from(i32::MAX)).then_some(value as i32)
        }
        None => word.parse::<i32>().ok(),
    }
}

/// Reads `bmh point save|remove|delete ...` (without its slash). `player` is the feet of the player
/// who runs it, for `~`.
pub fn parse_point_command(line: &str, player: [f64; 3]) -> PointCommand {
    let words: Vec<&str> = line.split(' ').filter(|w| !w.is_empty()).collect();
    if words.first() != Some(&"bmh") || words.get(1) != Some(&"point") {
        return PointCommand::NotMine;
    }
    let Some(action) = words.get(2) else {
        return PointCommand::Incomplete;
    };
    let pos = |relative: bool| -> Result<[i32; 3], PointCommand> {
        let mut out = [0; 3];
        for (i, slot) in out.iter_mut().enumerate() {
            let word = words.get(3 + i).ok_or(PointCommand::Incomplete)?;
            let value = if relative {
                coordinate(word, player[i])
            } else {
                word.parse::<i32>().ok()
            };
            *slot = value.ok_or(PointCommand::Bad)?;
        }
        Ok(out)
    };
    let result = match *action {
        "save" => pos(true).and_then(|pos| {
            // The name is the rest of the line after the three coordinates (a greedy string).
            let rest = rest_after_words(line, 6);
            if rest.is_empty() {
                Err(PointCommand::Incomplete)
            } else {
                Ok(Change::Save { pos, name: rest.to_owned() })
            }
        }),
        "remove" => pos(true).and_then(|pos| exact_words(&words, 6, Change::Remove { pos })),
        "delete" => pos(false).and_then(|pos| exact_words(&words, 6, Change::Delete { pos })),
        _ => Err(PointCommand::Bad),
    };
    match result {
        Ok(change) => PointCommand::Change(change),
        Err(command) => command,
    }
}

/// The change when the line has exactly `count` words, else `Bad`.
fn exact_words(words: &[&str], count: usize, change: Change) -> Result<Change, PointCommand> {
    if words.len() == count { Ok(change) } else { Err(PointCommand::Bad) }
}

/// The text after the first `count` words of a line, as typed.
fn rest_after_words(line: &str, count: usize) -> &str {
    let mut rest = line;
    for _ in 0..count {
        rest = rest.trim_start_matches(' ');
        rest = rest.find(' ').map_or("", |at| &rest[at..]);
    }
    rest.trim_start_matches(' ')
}

/// The console line of the chime for the other players near a block (`level.playSound(null, ...)` reaches
/// everyone in 16 blocks; the player who acts hears it from `play_sound_to`). None for a name that is not
/// a player name.
pub fn chime_command(name: &str, at: [f64; 3], pitch: f32) -> Option<String> {
    super::respawn::is_command_name(name).then(|| {
        let [x, y, z] = at;
        format!("playsound {CHIME_SOUND} {CHIME_CATEGORY} @a[name=!{name}] {x} {y} {z} {CHIME_VOLUME} {pitch}")
    })
}

// ---------------------------------------------------------------------------------------------
// The naming screen (PointScreen.java)
// ---------------------------------------------------------------------------------------------

/// The size of the name field and of the buttons (PointScreen.java:25-27).
pub const FIELD_WIDTH: i32 = 200;
pub const BUTTON_WIDTH: i32 = 64;
pub const BUTTON_HEIGHT: i32 = 20;
pub const GAP: i32 = 4;
/// Colours of the texts (PointScreen.java:29-31), opaque.
pub const TEXT_COLOR: u32 = 0xFFFF_FFFF;
pub const NOTE_COLOR: u32 = 0xFFA0_A0A0;
pub const FULL_COLOR: u32 = 0xFFFF_8080;
/// The dim layer under the screen (PointScreen.java:141).
pub const DIM_COLOR: u32 = 0x8000_0000;
/// Vanilla button text: white when active, grey when not (`AbstractButton`).
pub const BUTTON_TEXT: u32 = 0xFF_FFFF;
pub const BUTTON_TEXT_INACTIVE: u32 = 0xA0_A0A0;
/// The window size in whole GUI pixels (Java `Screen.width`, `height`).
pub fn window_size(rect: [f32; 4]) -> (i32, i32) {
    (rect[2].ceil() as i32, rect[3].ceil() as i32)
}

/// A rectangle `[x, y, width, height]` in GUI pixels.
pub type Rect = [i32; 4];

/// True when the pointer is on the rectangle.
pub fn inside(rect: Rect, pointer: [f32; 2]) -> bool {
    let [x, y, w, h] = rect.map(|v| v as f32);
    pointer[0] >= x && pointer[0] < x + w && pointer[1] >= y && pointer[1] < y + h
}

/// A button of the naming screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingButton {
    Save,
    Remove,
    Cancel,
}

/// Where the naming screen draws each part (PointScreen.java:60-88, 145-159).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamingLayout {
    pub center_x: i32,
    pub title_y: i32,
    pub subtitle_y: i32,
    pub count_y: i32,
    pub field: Rect,
    pub buttons: Vec<(NamingButton, Rect)>,
}

/// The layout for a window of `width` x `height`. `editing` adds Remove.
pub fn naming_layout(width: i32, height: i32, editing: bool) -> NamingLayout {
    let top = height / 2 - 30;
    let field = [(width - FIELD_WIDTH) / 2, top, FIELD_WIDTH, 20];
    let kinds: &[NamingButton] = if editing {
        &[NamingButton::Save, NamingButton::Remove, NamingButton::Cancel]
    } else {
        &[NamingButton::Save, NamingButton::Cancel]
    };
    let count = kinds.len() as i32;
    let mut x = (width - (count * BUTTON_WIDTH + (count - 1) * GAP)) / 2;
    let y = top + 30;
    let mut buttons = Vec::new();
    for kind in kinds {
        buttons.push((*kind, [x, y, BUTTON_WIDTH, BUTTON_HEIGHT]));
        x += BUTTON_WIDTH + GAP;
    }
    NamingLayout {
        center_x: width / 2,
        title_y: top - 30,
        subtitle_y: top - 16,
        count_y: top + 58,
        field,
        buttons,
    }
}

/// A new point needs a free place; a rename does not (PointScreen.java:54-57).
pub fn screen_full(editing: bool, used: i32, max: i32) -> bool {
    !editing && used >= max
}

/// Save is active when the place is free and the clean name is not empty (PointScreen.java:91-94).
pub fn save_active(editing: bool, used: i32, max: i32, text: &str) -> bool {
    !screen_full(editing, used, max) && !clean_name(text).is_empty()
}

/// The count line: its key and its two numbers (PointScreen.java:155-157).
pub fn count_line(editing: bool, used: i32, max: i32) -> (&'static str, Vec<i32>) {
    if screen_full(editing, used, max) {
        (KEY_FULL, vec![max])
    } else {
        (KEY_SCREEN_COUNT, vec![used + i32::from(!editing), max])
    }
}

/// True for Enter and the keypad Enter (`KeyEvent.isConfirmation`).
pub fn is_confirmation(key: &str) -> bool {
    matches!(key, "key.keyboard.enter" | "key.keyboard.keypad.enter")
}

/// True for Escape: the game closes the screen with it.
pub fn is_escape(key: &str) -> bool {
    key == "key.keyboard.escape"
}

/// Fit replacement text to the 24 UTF-16 units in a Java point name.
pub fn fit_name_insert(value: &str, selection: &str, insert: &str) -> String {
    let kept = utf16_len(value).saturating_sub(utf16_len(selection));
    cut_utf16(insert, MAX_NAME_LENGTH.saturating_sub(kept))
}

// ---------------------------------------------------------------------------------------------
// The list screen (PointListScreen.java)
// ---------------------------------------------------------------------------------------------

/// Rows on a page and the panel (PointListScreen.java:23-29).
pub const PER_PAGE: usize = 5;
pub const PANEL_WIDTH: i32 = 280;
pub const ROW_HEIGHT: i32 = 24;
pub const NAME_WIDTH: f32 = 110.0;
pub const COORDS_X: i32 = 114;
pub const LIST_BUTTON_X: i32 = 216;

/// The last page of a list (PointListScreen.java:53-56).
pub fn last_page(count: usize) -> usize {
    count.saturating_sub(1) / PER_PAGE
}

/// A page moved by `by` and kept in range (PointListScreen.java:96-100).
pub fn turn_page(page: usize, by: i32, count: usize) -> usize {
    let moved = i64::try_from(page).unwrap_or(0) + i64::from(by);
    usize::try_from(moved.max(0)).unwrap_or(0).min(last_page(count))
}

/// A button of the list screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListButton {
    /// Remove the point at this index of the whole list.
    Remove(usize),
    Previous,
    Next,
    Done,
}

/// Where the list screen draws each part (PointListScreen.java:68-94, 116-146).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListLayout {
    pub left: i32,
    pub top: i32,
    pub center_x: i32,
    pub title_y: i32,
    pub count_y: i32,
    pub empty_y: i32,
    pub spawn_y: i32,
    /// The text rows on this page: the point index and the y of its text.
    pub rows: Vec<(usize, i32)>,
    /// Each button, and whether it is active.
    pub buttons: Vec<(ListButton, Rect, bool)>,
}

/// The layout for a window of `width` x `height`, a list of `count` points, on `page`.
pub fn list_layout(width: i32, height: i32, count: usize, page: usize) -> ListLayout {
    let left = (width - PANEL_WIDTH) / 2;
    let top = height / 2 - 70;
    let first = page * PER_PAGE;
    let mut rows = Vec::new();
    let mut buttons = Vec::new();
    for row in 0..PER_PAGE {
        let index = first + row;
        if index >= count {
            break;
        }
        let row_y = top + row as i32 * ROW_HEIGHT;
        rows.push((index, row_y + 6));
        buttons.push((ListButton::Remove(index), [left + LIST_BUTTON_X, row_y, BUTTON_WIDTH, BUTTON_HEIGHT], true));
    }
    let y = top + PER_PAGE as i32 * ROW_HEIGHT + 24;
    buttons.push((ListButton::Previous, [left, y, 20, 20], page > 0));
    buttons.push((ListButton::Next, [left + PANEL_WIDTH - 20, y, 20, 20], page < last_page(count)));
    buttons.push((ListButton::Done, [width / 2 - 40, y, 80, 20], true));
    ListLayout {
        left,
        top,
        center_x: width / 2,
        title_y: top - 28,
        count_y: top - 14,
        empty_y: top + 30,
        spawn_y: top + PER_PAGE as i32 * ROW_HEIGHT + 6,
        rows,
        buttons,
    }
}

/// The start of a name that fits in `max` pixels (vanilla `Font.plainSubstrByWidth`): `width` measures a
/// text.
pub fn fit_width(name: &str, max: f32, width: impl Fn(&str) -> f32) -> String {
    let mut fitted = String::new();
    for c in name.chars() {
        fitted.push(c);
        if width(&fitted) > max {
            fitted.pop();
            break;
        }
    }
    fitted
}

/// The x where a centred text starts (vanilla `centeredText`: the centre less half the whole width).
pub fn centered_x(center: i32, width: f32) -> i32 {
    center - (width.ceil() as i32) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(name: &str, pos: [i32; 3]) -> AmuletPoint {
        AmuletPoint { name: name.to_owned(), pos }
    }

    #[test]
    fn the_azimuth_self_check_of_java_holds() {
        // AmuletReading.java:78-86: Home (0, -100) from (0, 0) is 0, Home (100, 0) is 90.
        assert_eq!(azimuth(0.0, 0.0, 0.0, -100.0), 0.0);
        assert_eq!(azimuth(0.0, 0.0, 100.0, 0.0), 90.0);
        assert_eq!(azimuth(0.0, 0.0, 0.0, 100.0), 180.0);
        assert_eq!(azimuth(0.0, 0.0, -100.0, 0.0), 270.0);
        assert_eq!(azimuth(0.0, 0.0, 100.0, -100.0), 45.0);
        // The point itself: atan2(0, 0) is 0.
        assert_eq!(azimuth(5.0, 5.0, 5.0, 5.0), 0.0);
        // A tiny angle west of north is close to 360, never 360.
        let a = azimuth(0.0, 0.0, -1e-12, -100.0);
        assert!((0.0..360.0).contains(&a), "{a}");
    }

    #[test]
    fn the_night_index_goes_up_at_nightfall() {
        // AmuletReading.java:90-93: floorDiv(dayTime + 24000 - 12000, 24000).
        assert_eq!(night_index(0), 0);
        assert_eq!(night_index(11_999), 0);
        assert_eq!(night_index(12_000), 1);
        assert_eq!(night_index(35_999), 1);
        assert_eq!(night_index(36_000), 2);
        assert_eq!(night_index(-12_001), -1);
        assert_eq!(night_index(-12_000), 0);
    }

    #[test]
    fn charges_default_full_and_stay_in_range() {
        assert_eq!(charges(StackCharges::default()), 3);
        assert_eq!(charges(StackCharges { charges: Some(2), night: Some(4) }), 2);
        assert_eq!(charges(StackCharges { charges: Some(-5), night: None }), 0);
        assert_eq!(charges(StackCharges { charges: Some(99), night: None }), 3);
    }

    #[test]
    fn a_new_night_refills_and_a_stack_that_never_read_is_left_alone() {
        assert_eq!(refill(StackCharges::default(), 50_000), None);
        let spent = StackCharges { charges: Some(0), night: Some(1) };
        assert_eq!(refill(spent, 20_000), None);
        assert_eq!(refill(spent, 36_000), Some(StackCharges { charges: Some(3), night: Some(2) }));
        // A night counted in the future (time set back) also refills, as Java compares with !=.
        assert_eq!(refill(StackCharges { charges: Some(1), night: Some(9) }, 0), Some(StackCharges { charges: Some(3), night: Some(0) }));
        assert!(refill_tick(0) && refill_tick(40) && !refill_tick(41));
    }

    #[test]
    fn the_bar_is_rounded_as_java_rounds() {
        // Math.round(13F * charges / 3).
        assert_eq!(bar_width(0), 0);
        assert_eq!(bar_width(1), 4);
        assert_eq!(bar_width(2), 9);
        assert_eq!(bar_width(3), 13);
    }

    #[test]
    fn the_late_import_keeps_every_point_once() {
        let p = |name: &str, x: i32| AmuletPoint { name: name.to_owned(), pos: [x, 64, 0] };
        let java = [p("Home", 1), p("Mine", 2)];
        // No point made before the import: the Java list as it is.
        assert_eq!(merge_imported(&java, &[]), java.to_vec());
        // A new point goes after the Java points.
        assert_eq!(merge_imported(&java, &[p("Camp", 3)]), vec![p("Home", 1), p("Mine", 2), p("Camp", 3)]);
        // A point that the disk view already showed is not doubled.
        assert_eq!(merge_imported(&java, &[p("Mine", 2), p("Camp", 3)]), vec![p("Home", 1), p("Mine", 2), p("Camp", 3)]);
        // Same place, other name: both stay (a rename is a different point to the merge).
        assert_eq!(merge_imported(&java, &[p("Mine 2", 2)]).len(), 3);
        // No Java points: the made points as they are.
        assert_eq!(merge_imported(&[], &[p("Camp", 3)]), vec![p("Camp", 3)]);
    }

    #[test]
    fn the_sky_darkens_as_vanilla_says() {
        // Horizon's own cases (horizon-client-render environment.rs, dark_outside_follows_time_and_weather_levels).
        assert!(!is_dark_outside(6000, 0.0, 0.0), "clear noon");
        assert!(!is_dark_outside(6000, 1.0, 0.0), "rainy noon");
        assert!(is_dark_outside(6000, 1.0, 1.0), "thunderstorm at noon");
        assert!(is_dark_outside(18_000, 0.0, 0.0), "clear midnight");
        assert!(!is_dark_outside(1000, 0.0, 0.0), "clear morning");
        assert_eq!(sky_darken(18_000, 0.0, 0.0), 11);
        assert_eq!(sky_darken(6000, 0.0, 0.0), 0);
        // Vanilla: a bed works in clear weather from 12542 to 23459 (the night of isDarkOutside).
        assert!(!is_dark_outside(12_541, 0.0, 0.0));
        assert!(is_dark_outside(12_542, 0.0, 0.0));
        assert!(is_dark_outside(23_459, 0.0, 0.0));
        assert!(!is_dark_outside(23_460, 0.0, 0.0));
        // The day time keeps counting past 24000.
        assert!(is_dark_outside(18_000 + 24_000 * 7, 0.0, 0.0));
    }

    /// The first dark tick and the first light tick after it, in one day of 24000 ticks.
    fn dark_window(rain: f32, thunder: f32) -> (i64, i64) {
        let dark = |t: i64| is_dark_outside(t, rain, thunder);
        let start = (0..24_000).find(|t| dark(*t) && !dark((*t + 23_999) % 24_000)).unwrap_or(-1);
        let end = (0..24_000).find(|t| !dark(*t) && dark((*t + 23_999) % 24_000)).unwrap_or(-1);
        (start, end)
    }

    #[test]
    fn the_dark_window_is_pinned_where_the_formula_changes_its_answer() {
        // Clear weather: the vanilla bed window, from 12542 to 23459 (first light tick 23460).
        assert_eq!(dark_window(0.0, 0.0), (12_542, 23_460));
        // The highest rain that is not "raining" (above 0.2 the reading is refused for rain first).
        assert_eq!(dark_window(0.2, 0.0), (12_465, 23_536));
        // A thunderstorm (rain 1, thunder 1) is dark all day: no tick changes the answer.
        assert_eq!(dark_window(1.0, 1.0), (-1, -1));
    }

    fn clear_night() -> Sky {
        Sky {
            overworld: true,
            rain: 0.0,
            thunder: 0.0,
            day_time: 18_000,
            sky_light: 15,
            blinded: false,
            eyes_covered: false,
        }
    }

    #[test]
    fn the_sky_rules_follow_the_java_order() {
        let p = HIDDEN_PREFIX;
        assert_eq!(hidden_reason(&clear_night(), p), None);
        let all_bad = Sky {
            overworld: false,
            rain: 1.0,
            thunder: 0.0,
            day_time: 6000,
            sky_light: 0,
            blinded: true,
            eyes_covered: true,
        };
        assert_eq!(hidden_reason(&all_bad, p).as_deref(), Some("bmh.amulet.hidden.dimension"));
        let rain = Sky { overworld: true, ..all_bad };
        assert_eq!(hidden_reason(&rain, p).as_deref(), Some("bmh.amulet.hidden.rain"));
        // A rain level of 0.2 is not rain yet.
        let drizzle = Sky { rain: 0.2, ..clear_night() };
        assert_eq!(hidden_reason(&drizzle, p), None);
        let day = Sky { rain: 0.0, ..rain };
        assert_eq!(hidden_reason(&day, p).as_deref(), Some("bmh.amulet.hidden.day"));
        let roof = Sky { day_time: 18_000, ..day };
        assert_eq!(hidden_reason(&roof, p).as_deref(), Some("bmh.amulet.hidden.roof"));
        for sky in [
            Sky { sky_light: 14, ..clear_night() },
            Sky { blinded: true, ..clear_night() },
            Sky { eyes_covered: true, ..clear_night() },
        ] {
            assert_eq!(hidden_reason(&sky, p).as_deref(), Some("bmh.amulet.hidden.roof"));
        }
        // The moon ritual can pass its own prefix (AmuletReading.java:111).
        assert_eq!(hidden_reason(&all_bad, "x").as_deref(), Some("x.dimension"));
    }

    #[test]
    fn the_fluid_and_snow_tests_know_their_blocks() {
        assert!(holds_water("minecraft:water", false));
        assert!(holds_water("minecraft:oak_stairs", true));
        assert!(holds_water("minecraft:kelp_plant", false));
        assert!(!holds_water("minecraft:air", false));
        assert!(holds_lava("minecraft:lava"));
        assert!(is_powder_snow("minecraft:powder_snow"));
        assert!(!is_powder_snow("minecraft:snow"));
    }

    #[test]
    fn a_reading_points_to_each_point_then_the_spawn() {
        let points = [point("Home", [0, 70, -101]), point("Mine", [99, 12, 0])];
        let r = reading([0.5, 64.0, 0.5], &points, Some([-101, 64, 0]), 1234);
        assert_eq!(r.azimuths, vec![0.0, 90.0, 270.0]);
        assert_eq!(r.names, vec!["Home".to_owned(), "Mine".to_owned(), String::new()]);
        assert_eq!(r.spawn, Some(2));
        assert_eq!(r.time, 1234);
        // No point: the spawn line alone, at index 0.
        let alone = reading([0.5, 64.0, 0.5], &[], Some([0, 64, 100]), 7);
        assert_eq!((alone.azimuths, alone.spawn), (vec![180.0], Some(0)));
        // No known spawn: no spawn line.
        let none = reading([0.5, 64.0, 0.5], &points, None, 7);
        assert_eq!((none.azimuths.len(), none.spawn), (2, None));
    }

    #[test]
    fn a_reading_costs_one_charge_unless_free() {
        let full = StackCharges::default();
        assert_eq!(charges_after_reading(full, false, 18_000), Some(StackCharges { charges: Some(2), night: Some(1) }));
        assert_eq!(charges_after_reading(full, true, 18_000), None);
        assert_eq!(block_of([-0.5, 64.9, 3.2]), [-1, 64, 3]);
    }

    #[test]
    fn the_cooldown_runs_for_its_ticks_and_ends_in_another_world() {
        let c = Cooldown { start: 100, ticks: 40 };
        assert!(!c.active(99));
        assert!(c.active(100));
        assert!(c.active(139));
        assert!(!c.active(140));
        assert!(!Cooldown { start: i64::MAX, ticks: 40 }.active(0));
    }

    #[test]
    fn the_sweep_shrinks_from_the_top_as_vanilla_draws_it() {
        let c = Cooldown { start: 100, ticks: 40 };
        assert_eq!(c.remaining(99, 0.0), 0.0);
        assert_eq!(c.remaining(100, 0.0), 1.0);
        assert_eq!(c.remaining(120, 0.0), 0.5);
        assert_eq!(c.remaining(120, 0.5), 0.4875);
        assert_eq!(c.remaining(140, 0.0), 0.0);
        assert_eq!(sweep_rows(1.0), Some((0, 16)));
        assert_eq!(sweep_rows(0.5), Some((8, 8)));
        assert_eq!(sweep_rows(0.01), Some((15, 1)));
        assert_eq!(sweep_rows(0.0), None);
        assert_eq!(sweep_rows(f32::NAN), None);
        assert_eq!(SWEEP_COLOR, i32::MAX as u32);
    }

    #[test]
    fn a_use_on_a_block_follows_the_java_order() {
        assert_eq!(use_on(false, true, true, true), UseOn::Open);
        assert_eq!(use_on(false, true, true, false), UseOn::NotOverworld);
        assert_eq!(use_on(false, false, true, true), UseOn::Pass);
        assert_eq!(use_on(false, true, false, true), UseOn::Pass);
        assert_eq!(use_on(true, true, true, true), UseOn::Pass);
    }

    #[test]
    fn a_use_in_the_air_follows_the_java_order() {
        let hidden = Some("bmh.amulet.hidden.day".to_owned());
        assert_eq!(use_in_air(true, false, true, None, 3, false), Use::Ignore);
        assert_eq!(use_in_air(false, true, true, None, 3, false), Use::Ignore);
        assert_eq!(use_in_air(false, false, true, hidden.clone(), 0, false), Use::List);
        assert_eq!(use_in_air(false, false, false, hidden.clone(), 0, false), Use::Hidden("bmh.amulet.hidden.day".to_owned()));
        assert_eq!(use_in_air(false, false, false, None, 0, false), Use::Empty);
        assert_eq!(use_in_air(false, false, false, None, 0, true), Use::Read);
        assert_eq!(use_in_air(false, false, false, None, 1, false), Use::Read);
        assert!(same_click(Some(10), 10) && same_click(Some(10), 11));
        assert!(!same_click(Some(10), 12) && !same_click(Some(10), 9) && !same_click(None, 10));
    }

    #[test]
    fn beds_are_anchored_at_the_head_and_match_on_either_half() {
        let foot = bed_half(Some("foot"), Some("north"));
        let head = bed_half(Some("head"), Some("north"));
        assert_eq!(foot, Some(BedHalf { foot: true, facing: [0, 0, -1] }));
        assert_eq!(bed_half(Some("foot"), Some("up")), None);
        assert_eq!(bed_half(None, Some("north")), None);
        assert_eq!(anchor([5, 64, 5], foot), [5, 64, 4]);
        assert_eq!(anchor([5, 64, 4], head), [5, 64, 4]);
        assert_eq!(anchor([1, 2, 3], None), [1, 2, 3]);
        assert_eq!(other_half([5, 64, 4], head), [5, 64, 5]);
        // A Home of 1.6.x kept the foot: a click on the head finds it.
        let points = vec![point("Home", [5, 64, 5])];
        assert_eq!(point_at(&points, [5, 64, 4], head), Some(0));
        assert_eq!(point_at(&points, [5, 64, 5], foot), Some(0));
        assert_eq!(point_at(&points, [9, 64, 9], None), None);
        assert!(is_vanilla_bed("minecraft:light_gray_bed"));
        assert!(!is_vanilla_bed("minecraft:bedrock"));
        assert!(!is_vanilla_bed("mymod:red_bed"));
    }

    #[test]
    fn put_renames_or_adds_and_earns_the_advancements() {
        let mut points = vec![point("Home", [5, 64, 4])];
        let foot = bed_half(Some("foot"), Some("north"));
        assert!(put(&mut points, [5, 64, 5], foot, "Bed"));
        assert_eq!(points, vec![point("Bed", [5, 64, 4])]);
        assert!(!put(&mut points, [0, 70, 0], None, "Stone"));
        assert_eq!(points.len(), 2);
        assert_eq!(awards_after_put(2), vec![NAMED_PLACE]);
        assert_eq!(awards_after_put(3), vec![NAMED_PLACE, OWN_SKY]);
    }

    #[test]
    fn remove_takes_the_block_point_or_every_exact_match() {
        let mut points = vec![point("A", [1, 1, 1]), point("B", [2, 2, 2]), point("C", [1, 1, 1])];
        assert_eq!(remove_exact(&mut points, [1, 1, 1]), Some(point("C", [1, 1, 1])));
        assert_eq!(points, vec![point("B", [2, 2, 2])]);
        assert_eq!(remove_exact(&mut points, [7, 7, 7]), None);
        assert_eq!(remove_on_block(&mut points, [2, 2, 2], None), Some(point("B", [2, 2, 2])));
        assert!(points.is_empty());
        assert_eq!(remove_on_block(&mut points, [2, 2, 2], None), None);
    }

    #[test]
    fn the_name_is_cleaned_as_java_cleans_it() {
        assert_eq!(clean_name("  Home  "), "Home");
        assert_eq!(clean_name("\u{a7}cRed\u{a7}r base"), "Red base");
        assert_eq!(clean_name("a\u{7}b\u{85}c\td"), "abcd");
        assert_eq!(clean_name(""), "");
        assert_eq!(clean_name("   "), "");
        // 30 characters cut to 24, then trimmed again.
        assert_eq!(clean_name("abcdefghijklmnopqrstuvw xyzabc"), "abcdefghijklmnopqrstuvw");
        assert_eq!(clean_name(&"x".repeat(24)), "x".repeat(24));
        // A character outside the BMP counts two units and is never cut in two.
        let stars = "\u{1F31F}".repeat(13);
        assert_eq!(utf16_len(&clean_name(&stars)), 24);
        // A section sign at the end has no character after it.
        assert_eq!(clean_name("end\u{a7}"), "end");
    }

    fn asker() -> Asker {
        Asker { holds_amulet: true, overworld: true, alive: true, spectator: false, position: [0.5, 64.0, 0.5] }
    }

    #[test]
    fn a_change_is_checked_in_the_java_order() {
        let at = [3, 64, 3];
        assert_eq!(check_change(&asker(), at, true, None, 100), Ok(()));
        let none = Asker { holds_amulet: false, overworld: false, alive: false, ..asker() };
        assert_eq!(check_change(&none, at, false, Some(100), 100), Err(Refusal::NoAmulet));
        let nether = Asker { holds_amulet: true, ..none };
        assert_eq!(check_change(&nether, at, false, Some(100), 100), Err(Refusal::Overworld));
        let dead = Asker { overworld: true, ..nether };
        assert_eq!(check_change(&dead, at, true, None, 100), Err(Refusal::TooFar));
        let spectator = Asker { spectator: true, ..asker() };
        assert_eq!(check_change(&spectator, at, true, None, 100), Err(Refusal::TooFar));
        assert_eq!(check_change(&asker(), at, false, None, 100), Err(Refusal::TooFar));
        assert_eq!(check_change(&asker(), [9, 64, 0], true, None, 100), Err(Refusal::TooFar));
        assert_eq!(check_change(&asker(), at, true, Some(90), 100), Err(Refusal::TooSoon));
        assert_eq!(check_change(&asker(), at, true, Some(80), 100), Ok(()));
        assert_eq!(check_change(&asker(), at, true, Some(500), 100), Ok(()));
        assert_eq!(Refusal::TooSoon.key(), None);
        assert_eq!(Refusal::TooFar.key(), Some("bmh.point.too_far"));
        // Exactly 8 blocks from the middle of the block is in reach.
        assert!(within_reach([8.5, 64.5, 0.5], [0, 64, 0]));
        assert!(!within_reach([8.6, 64.5, 0.5], [0, 64, 0]));
    }

    #[test]
    fn a_save_needs_a_name_then_a_free_place() {
        assert_eq!(check_save("", false, 0, 5), Err(SaveStop::NoName));
        assert_eq!(check_save("A", false, 5, 5), Err(SaveStop::Full));
        assert_eq!(check_save("A", true, 9, 5), Ok(()));
        assert_eq!(check_save("A", false, 4, 5), Ok(()));
        assert_eq!(SaveStop::Full.key(), "bmh.point.full");
    }

    #[test]
    fn the_command_line_is_read_with_relative_coordinates() {
        let me = [10.7, 64.0, -3.2];
        assert_eq!(parse_point_command("bmh rule x", me), PointCommand::NotMine);
        assert_eq!(parse_point_command("bmhx point", me), PointCommand::NotMine);
        assert_eq!(parse_point_command("bmh point", me), PointCommand::Incomplete);
        assert_eq!(
            parse_point_command("bmh point save 1 2 3 My  home ", me),
            PointCommand::Change(Change::Save { pos: [1, 2, 3], name: "My  home ".to_owned() })
        );
        assert_eq!(
            parse_point_command("bmh point save ~ ~1 ~-0.5 Here", me),
            PointCommand::Change(Change::Save { pos: [10, 65, -4], name: "Here".to_owned() })
        );
        assert_eq!(parse_point_command("bmh point save 1 2 3", me), PointCommand::Incomplete);
        assert_eq!(parse_point_command("bmh point save 1 2", me), PointCommand::Incomplete);
        assert_eq!(parse_point_command("bmh point save ^ ^ ^ x", me), PointCommand::Bad);
        assert_eq!(
            parse_point_command("bmh point remove -5 70 8", me),
            PointCommand::Change(Change::Remove { pos: [-5, 70, 8] })
        );
        assert_eq!(parse_point_command("bmh point remove 1 2 3 4", me), PointCommand::Bad);
        assert_eq!(
            parse_point_command("bmh point delete 1 -2 3", me),
            PointCommand::Change(Change::Delete { pos: [1, -2, 3] })
        );
        // `delete` takes plain integers only (IntegerArgumentType).
        assert_eq!(parse_point_command("bmh point delete ~ 2 3", me), PointCommand::Bad);
        assert_eq!(parse_point_command("bmh point move 1 2 3", me), PointCommand::Bad);
        assert_eq!(parse_point_command("bmh point save ~nan 2 3 x", me), PointCommand::Bad);
        assert_eq!(parse_point_command("bmh point save ~1e300 2 3 x", me), PointCommand::Bad);
    }

    #[test]
    fn the_chime_command_reaches_the_others() {
        assert_eq!(
            chime_command("Steve", [1.5, 64.0, -2.25], SAVE_PITCH).as_deref(),
            Some("playsound minecraft:block.amethyst_block.chime player @a[name=!Steve] 1.5 64 -2.25 1 0.7")
        );
        assert_eq!(chime_command("a b", [0.0; 3], 1.0), None);
    }

    #[test]
    fn the_naming_screen_is_laid_out_as_java() {
        // A 427 x 240 GUI window (854 x 480 at scale 2).
        let new = naming_layout(427, 240, false);
        assert_eq!(new.field, [113, 90, 200, 20]);
        assert_eq!(new.buttons, vec![(NamingButton::Save, [147, 120, 64, 20]), (NamingButton::Cancel, [215, 120, 64, 20])]);
        assert_eq!((new.center_x, new.title_y, new.subtitle_y, new.count_y), (213, 60, 74, 148));
        let edit = naming_layout(427, 240, true);
        assert_eq!(edit.buttons.first().map(|b| b.1), Some([113, 120, 64, 20]));
        assert_eq!(edit.buttons.get(1).map(|b| b.0), Some(NamingButton::Remove));
        assert_eq!(edit.buttons.get(2).map(|b| b.1), Some([249, 120, 64, 20]));
        assert!(inside([10, 10, 5, 5], [10.0, 14.9]) && !inside([10, 10, 5, 5], [15.0, 12.0]));
        assert_eq!(window_size([0.0, 0.0, 426.5, 240.0]), (427, 240));
    }

    #[test]
    fn save_is_active_with_a_name_and_a_free_place() {
        assert!(!save_active(false, 5, 5, "Home"));
        assert!(save_active(true, 5, 5, "Home"));
        assert!(!save_active(false, 0, 5, "  "));
        assert_eq!(count_line(false, 2, 5), (KEY_SCREEN_COUNT, vec![3, 5]));
        assert_eq!(count_line(true, 2, 5), (KEY_SCREEN_COUNT, vec![2, 5]));
        assert_eq!(count_line(false, 5, 5), (KEY_FULL, vec![5]));
    }

    #[test]
    fn the_name_field_counts_utf16_units_after_replacing_the_selection() {
        let value = "a".repeat(23);
        assert_eq!(fit_name_insert(&value, "", "🌟b"), "");
        assert_eq!(fit_name_insert(&value, "a", "🌟b"), "🌟");
        assert_eq!(fit_name_insert(&"🌟".repeat(12), "🌟", "abc"), "ab");
        assert!(is_confirmation("key.keyboard.keypad.enter") && !is_confirmation("key.keyboard.space"));
        assert!(is_escape("key.keyboard.escape"));
    }

    #[test]
    fn the_list_pages_and_buttons_follow_java() {
        assert_eq!(last_page(0), 0);
        assert_eq!(last_page(5), 0);
        assert_eq!(last_page(6), 1);
        assert_eq!(turn_page(0, -1, 12), 0);
        assert_eq!(turn_page(0, 1, 12), 1);
        assert_eq!(turn_page(2, 1, 12), 2);
        assert_eq!(turn_page(3, 0, 6), 1);
        let l = list_layout(427, 240, 7, 1);
        assert_eq!((l.left, l.top, l.title_y, l.count_y, l.spawn_y), (73, 50, 22, 36, 176));
        assert_eq!(l.rows, vec![(5, 56), (6, 80)]);
        assert_eq!(l.buttons.first(), Some(&(ListButton::Remove(5), [289, 50, 64, 20], true)));
        assert!(l.buttons.contains(&(ListButton::Previous, [73, 194, 20, 20], true)));
        assert!(l.buttons.contains(&(ListButton::Next, [333, 194, 20, 20], false)));
        assert!(l.buttons.contains(&(ListButton::Done, [173, 194, 80, 20], true)));
        assert_eq!(list_layout(427, 240, 0, 0).rows, vec![]);
    }

    #[test]
    fn names_fit_their_column_and_text_is_centred() {
        let width = |s: &str| s.chars().count() as f32 * 6.0;
        assert_eq!(fit_width("abcdefghijklmnopqrstuvwx", 110.0, width), "abcdefghijklmnopqr");
        assert_eq!(fit_width("abc", 110.0, width), "abc");
        assert_eq!(centered_x(213, 41.0), 193);
    }
}
