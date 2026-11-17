// Area respawn (owner: the respawn agent): the pure logic of the distant respawn.
// It holds the distance formula at death, the level scaling of the distance, the candidate points,
// the safe-ground test, the landing search as a state machine over a small trait, the decision inside
// the respawn, the journey advancements, the damage immunity and the command lines of the sounds and
// the awards. The game module `bmh_respawn` reads the world and carries out what this file answers.
// Java reference: handlers/RespawnPositionHandler, RespawnSearch, RespawnEffectHandler, JourneySignal,
// BmhAdvancements, mixin/F_ServerPlayerJourneyMixin.

use crate::contract::Config;

/// The cap of the "easy life" factor (RespawnPositionHandler.java:40, `MAX_EASY_LIFE`).
pub const MAX_EASY_LIFE: i32 = 9;

/// The most candidate points of one search (RespawnPositionHandler.java:41, `MAX_POSITION_ATTEMPTS`).
pub const MAX_POSITION_ATTEMPTS: u32 = 32;

/// A search that runs longer than this fails, in milliseconds (RespawnSearch.java:35, `TIMEOUT_MS`).
pub const TIMEOUT_MS: u32 = 120_000;

/// The timeout in server ticks that ran (50 ms each). Java counts wall time, but its fallback is a
/// search at the respawn. Here the fallback is the vanilla point, so a paused server must not use up
/// the time (review semantics-7).
pub const TIMEOUT_TICKS: u32 = TIMEOUT_MS / 50;

/// The landing chunk stays held this many server ticks after the search found it
/// (RespawnSearch.java:37, `new TicketType(6000L, ...)`).
pub const LANDING_TICKET_TICKS: u32 = 6000;

/// The ticks of damage immunity after a distant respawn
/// (RespawnEffectHandler.java:21, `JOURNEY_INVULNERABLE_TICKS`).
pub const JOURNEY_INVULNERABLE_TICKS: i32 = 200;

/// The deaths that earn the hidden advancement (RespawnEffectHandler.java:24, `AGAIN_DEATHS`).
pub const AGAIN_DEATHS: i32 = 10;

/// The exile distance that earns "Far, far away" (BmhAdvancements.java:21, `FAR_AWAY_BLOCKS`).
pub const FAR_AWAY_BLOCKS: i32 = 30_000;

/// The farthest a world border centre can be (vanilla `WorldBorder.MAX_CENTER_COORDINATE`), the cap of
/// the level distance (RespawnPositionHandler.java:168).
pub const MAX_CENTER_COORDINATE: f64 = 2.999_998_4E7;

/// The criterion that the server code awards (BmhAdvancements.java:24, `CRITERION`).
pub const CRITERION: &str = "done";

/// Advancement `bmh:journey/exile` (RespawnEffectHandler.java:43).
pub const ADV_EXILE: &str = "journey/exile";
/// Advancement `bmh:journey/here_we_go_again` (RespawnEffectHandler.java:46).
pub const ADV_AGAIN: &str = "journey/here_we_go_again";
/// Advancement `bmh:journey/far_away` (RespawnEffectHandler.java:50).
pub const ADV_FAR_AWAY: &str = "journey/far_away";

/// The bell for the other players (SoundInit.java:15, `bell_sfx`; RespawnEffectHandler.java:72-81).
pub const BELL_SOUND: &str = "bmh:bell_sfx";
/// The volume of the bell (RespawnEffectHandler.java:79).
pub const BELL_VOLUME: f32 = 1.0;
/// The respawn sound for the other players (SoundInit.java:14, `end`; RespawnEffectHandler.java:83-92).
pub const RESPAWN_SOUND: &str = "bmh:end";
/// The volume of the respawn sound (RespawnEffectHandler.java:90).
pub const RESPAWN_VOLUME: f32 = 0.3;
/// The pitch of both sounds (RespawnEffectHandler.java:80, :91).
pub const SOUND_PITCH: f32 = 1.0;

/// The vanilla damage types in the tag `minecraft:bypasses_invulnerability`: the immunity does not stop
/// them (F_ServerPlayerJourneyMixin.java:45, `DamageTypeTags.BYPASSES_INVULNERABILITY`). Horizon has no
/// tag read for a damage type, so the list is the vanilla tag (GAPS.md C6).
pub const BYPASSES_INVULNERABILITY: [&str; 2] = ["minecraft:out_of_world", "minecraft:generic_kill"];

/// The Overworld.
pub const OVERWORLD: &str = "minecraft:overworld";
/// The Nether.
pub const NETHER: &str = "minecraft:the_nether";

/// A `/playsound` of volume 1 or less reaches the players within 16 blocks of its position
/// (vanilla `PlaySoundCommand`: `Math.max(volume, 1) * 16`, squared, compared with `distanceToSqr`).
pub const SOUND_RANGE: f64 = 16.0;

/// The sound of an anchor that loses a charge at a respawn (vanilla `PlayerList.respawn`:
/// `SoundEvents.RESPAWN_ANCHOR_DEPLETE`, `SoundSource.BLOCKS`, volume 1, pitch 1, to that player only).
pub const ANCHOR_DEPLETE_SOUND: &str = "minecraft:block.respawn_anchor.deplete";

/// The chat line of a respawn point whose block is gone or does not work (vanilla
/// `ClientboundGameEventPacket.NO_RESPAWN_BLOCK_AVAILABLE`, shown by the client as this key).
pub const NOT_VALID_KEY: &str = "block.minecraft.spawn.not_valid";

/// The id of a difficulty (vanilla `Difficulty.getId`): peaceful 0, easy 1, normal 2, hard 3.
/// None for another name.
pub fn difficulty_id(name: &str) -> Option<i32> {
    match name.strip_prefix("minecraft:").unwrap_or(name) {
        "peaceful" => Some(0),
        "easy" => Some(1),
        "normal" => Some(2),
        "hard" => Some(3),
        _ => None,
    }
}

/// The distance of the next respawn in Overworld blocks (RespawnPositionHandler.java:215-244).
/// `eternal_slots` is the number of inventory slots that hold an Eternal Item, `difficulty` the id of
/// `difficulty_id`. The Java code uses integer arithmetic: the division truncates.
pub fn distance(config: Config, eternal_slots: i32, difficulty: i32) -> i32 {
    let min = config.min_distance;
    // RespawnPositionHandler.java:223-226: a min above the max uses the min for both.
    let max = config.max_distance.max(min);
    // RespawnPositionHandler.java:220, :235-237.
    let easy_life = 1i64
        .saturating_add(i64::from(eternal_slots.max(0)))
        .saturating_add(i64::from(difficulty.max(0)))
        .min(i64::from(MAX_EASY_LIFE));
    // RespawnPositionHandler.java:239. The config limits keep the product far inside an i64.
    let span = i64::from(max) - i64::from(min);
    let value = i64::from(min) + span * easy_life / i64::from(MAX_EASY_LIFE);
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// The coordinate scale of a dimension type: 8 for the Nether, 1 for the others (vanilla dimension
/// types). Horizon has only the three vanilla levels.
pub fn coordinate_scale(dimension: &str) -> f64 {
    if dimension == NETHER { 8.0 } else { 1.0 }
}

/// The exile distance in blocks of the level (RespawnPositionHandler.java:164-174, `toLevelBlocks`):
/// `floor(clamp(world / scale + 0.5, 0, MAX_CENTER_COORDINATE))`.
pub fn level_distance(world_distance: i32, scale: f64) -> i32 {
    let scaled = (f64::from(world_distance) / scale + 0.5).clamp(0.0, MAX_CENTER_COORDINATE);
    // The clamp keeps the value inside an i32; a NaN scale gives 0, as `as` does.
    scaled.floor() as i32
}

/// A point on the circle of `radius` round `center`, from a random value in [0, 1)
/// (RespawnPositionHandler.java:303-311, `findRandomEdgePoint`). Java casts with `(int)`, which cuts
/// toward zero, as Rust's `as` does.
pub fn edge_point(center: [i32; 3], radius: i32, random: f64) -> (i32, i32) {
    let theta = 2.0 * std::f64::consts::PI * random;
    let dx = (f64::from(radius) * theta.cos()) as i32;
    let dz = (f64::from(radius) * theta.sin()) as i32;
    (center[0].saturating_add(dx), center[2].saturating_add(dz))
}

/// True when the point is inside the world border (vanilla `WorldBorder.isWithinBounds(x, z)`:
/// min inclusive, max exclusive). `border` is `[min_x, min_z, max_x, max_z]`.
pub fn inside_border(border: [f64; 4], x: i32, z: i32) -> bool {
    let (x, z) = (f64::from(x), f64::from(z));
    x >= border[0] && x < border[2] && z >= border[1] && z < border[3]
}

/// The vanilla biomes of the tags `minecraft:is_ocean`, `minecraft:is_deep_ocean` and
/// `minecraft:is_river` (RespawnPositionHandler.java:208). Horizon has no tag read for a biome, so the
/// list is the vanilla tags (GAPS.md C3): a biome of a data pack is not in it.
pub const REJECTED_BIOMES: [&str; 11] = [
    "minecraft:ocean",
    "minecraft:deep_ocean",
    "minecraft:frozen_ocean",
    "minecraft:deep_frozen_ocean",
    "minecraft:cold_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:lukewarm_ocean",
    "minecraft:deep_lukewarm_ocean",
    "minecraft:warm_ocean",
    "minecraft:river",
    "minecraft:frozen_river",
];

/// True when no landing goes in this biome (RespawnPositionHandler.java:206-210).
pub fn is_rejected_biome(biome: &str) -> bool {
    REJECTED_BIOMES.contains(&biome)
}

/// The vertical shape of a level, as the safe-ground search needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelShape {
    /// The lowest block y (`getMinY`).
    pub min_y: i32,
    /// True for a level with a bedrock roof (`dimensionType().hasCeiling()`): the Nether.
    pub has_ceiling: bool,
    /// `dimensionType().logicalHeight()`.
    pub logical_height: i32,
    /// `getSeaLevel()`: the y of the biome test (RespawnPositionHandler.java:206).
    pub sea_level: i32,
}

impl LevelShape {
    /// The shape of a vanilla level: `min_y` comes from the game, the rest from the vanilla dimension
    /// types and noise settings (Nether: ceiling, logical height 128, sea level 32; Overworld: sea
    /// level 63; End: sea level 0).
    pub fn of(dimension: &str, min_y: i32, height: i32) -> LevelShape {
        match dimension {
            NETHER => LevelShape { min_y, has_ceiling: true, logical_height: 128, sea_level: 32 },
            OVERWORLD => LevelShape { min_y, has_ceiling: false, logical_height: height, sea_level: 63 },
            _ => LevelShape { min_y, has_ceiling: false, logical_height: height, sea_level: 0 },
        }
    }
}

/// What the search knows of one block.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockInfo {
    /// The block id, `minecraft:grass_block`.
    pub name: String,
    /// The block state has `waterlogged=true`.
    pub waterlogged: bool,
    /// The collision boxes of the state, block-local: `[min_x, min_y, min_z, max_x, max_y, max_z]`.
    pub boxes: Vec<[f64; 6]>,
}

/// Blocks whose fluid state is water or lava without a `waterlogged` property (vanilla
/// `BlockState.getFluidState`).
const FLUID_BLOCKS: [&str; 7] = [
    "minecraft:water",
    "minecraft:lava",
    "minecraft:bubble_column",
    "minecraft:seagrass",
    "minecraft:tall_seagrass",
    "minecraft:kelp",
    "minecraft:kelp_plant",
];

/// Blocks whose support shape is a full block while their collision box is lower (vanilla
/// `getBlockSupportShape` overrides of `SoulSandBlock` and `MudBlock`).
const FULL_SUPPORT: [&str; 2] = ["minecraft:soul_sand", "minecraft:mud"];

/// True when the block holds a fluid (`getFluidState().isEmpty()` is false).
pub fn has_fluid(block: &BlockInfo) -> bool {
    block.waterlogged || FLUID_BLOCKS.contains(&block.name.as_str())
}

/// Free space: no collision box and no fluid (RespawnPositionHandler.java:288-293, `isFreeSpace`).
pub fn is_free(block: &BlockInfo) -> bool {
    block.boxes.is_empty() && !has_fluid(block)
}

/// True when the collision boxes cover the whole top face, on a 16 by 16 grid (vanilla
/// `isFaceSturdy(UP)` with `SupportType.FULL`, which reads the shape at y = 1).
fn top_face_full(boxes: &[[f64; 6]]) -> bool {
    const EPS: f64 = 1.0E-7;
    (0..16).all(|i| {
        (0..16).all(|k| {
            let (cx, cz) = ((f64::from(i) + 0.5) / 16.0, (f64::from(k) + 0.5) / 16.0);
            boxes.iter().any(|b| {
                b[4] >= 1.0 - EPS && b[0] <= cx && cx <= b[3] && b[2] <= cz && cz <= b[5]
            })
        })
    })
}

/// True for leaves (vanilla tag `minecraft:leaves`: every vanilla leaves block id ends in `_leaves`).
pub fn is_leaves(name: &str) -> bool {
    name.ends_with("_leaves")
}

/// Ground a player can stand on (RespawnPositionHandler.java:295-301, `isSafeGround`): no fluid, a
/// sturdy top face, not leaves, not a magma block. The order of the Java checks is kept.
pub fn is_safe_ground(block: &BlockInfo) -> bool {
    !has_fluid(block)
        && (top_face_full(&block.boxes) || FULL_SUPPORT.contains(&block.name.as_str()))
        && !is_leaves(&block.name)
        && block.name != "minecraft:magma_block"
}

/// The block reads of the safe-ground search, in the level of the search.
pub trait Blocks {
    /// The block at x y z.
    fn block(&mut self, x: i32, y: i32, z: i32) -> BlockInfo;
    /// The height of the column on `MOTION_BLOCKING_NO_LEAVES`: the y over the highest block that
    /// counts, as `Level.getHeight` answers it. None when the chunk is not loaded.
    fn surface(&mut self, x: i32, z: i32) -> Option<i32>;
}

/// The feet position on solid, dry ground with two free blocks above, or None
/// (RespawnPositionHandler.java:248-286, `findSafePosition`). The chunk must be loaded.
pub fn find_safe_position(world: &mut impl Blocks, shape: LevelShape, x: i32, z: i32) -> Option<[i32; 3]> {
    let min_y = shape.min_y;
    let mut y;
    if shape.has_ceiling {
        // RespawnPositionHandler.java:253-259: start under the roof and get out of the solid layer.
        y = min_y.saturating_add(shape.logical_height).saturating_sub(2);
        while y > min_y && !is_free(&world.block(x, y, z)) {
            y -= 1;
        }
    } else {
        // RespawnPositionHandler.java:263-264: the heightmap height plus one, read from the chunk.
        // `surface` already answers the y over the highest block, which is the Java value.
        y = world.surface(x, z)?;
    }
    // RespawnPositionHandler.java:269-271: go down through free space to the first real block.
    while y > min_y && is_free(&world.block(x, y, z)) {
        y -= 1;
    }
    // RespawnPositionHandler.java:276-281, in the Java order.
    let feet = y.checked_add(1)?;
    let head = feet.checked_add(1)?;
    let safe = y > min_y
        && is_safe_ground(&world.block(x, y, z))
        && is_free(&world.block(x, feet, z))
        && is_free(&world.block(x, head, z));
    safe.then_some([x, feet, z])
}

/// What the landing search asks of the world.
pub trait SearchWorld: Blocks {
    /// A random value in [0, 1) (Java `serverLevel.getRandom().nextDouble()`).
    fn random(&mut self) -> f64;
    /// True when the point is inside the level's world border.
    fn inside_border(&mut self, x: i32, z: i32) -> bool;
    /// Holds the chunk `[x >> 4, z >> 4]` loaded (Java `addTicketWithRadius`).
    fn hold(&mut self, chunk: [i32; 2]);
    /// Lets the chunk go (Java `removeTicketWithRadius`).
    fn release(&mut self, chunk: [i32; 2]);
    /// True when the chunk of the column is loaded in the level (Java `getChunkNow` is not null).
    fn loaded(&mut self, x: i32, z: i32) -> bool;
    /// The biome at the block, None when it is not known.
    fn biome(&mut self, x: i32, y: i32, z: i32) -> Option<String>;
    /// The vertical shape of the level.
    fn shape(&mut self) -> LevelShape;
}

/// The chunk of a block column.
pub fn chunk_of(x: i32, z: i32) -> [i32; 2] {
    [x >> 4, z >> 4]
}

/// The landing search of one player (RespawnSearch.java). The Java search tested the biome before it
/// loaded the chunk. Horizon reads a biome only in a loaded chunk, so this search tests the biome once
/// the chunk is loaded (GAPS.md C3).
#[derive(Debug, Clone, PartialEq)]
pub struct Search {
    /// The level of the respawn.
    pub dimension: String,
    /// The centre of the exile: the bed or anchor block, or the world spawn.
    pub center: [i32; 3],
    /// The distance in blocks of the level.
    pub distance: i32,
    attempts: u32,
    candidate: Option<(i32, i32)>,
    result: Option<[i32; 3]>,
    failed: bool,
}

impl Search {
    /// A search that has drawn no candidate yet (RespawnSearch.java:55-60).
    pub fn new(dimension: &str, center: [i32; 3], distance: i32) -> Search {
        Search {
            dimension: dimension.to_owned(),
            center,
            distance,
            attempts: 0,
            candidate: None,
            result: None,
            failed: false,
        }
    }

    /// True once the search found a spot or failed (RespawnSearch.java:159-162).
    pub fn is_finished(&self) -> bool {
        self.result.is_some() || self.failed
    }

    /// The feet position, or None (RespawnSearch.java:165-168).
    pub fn result(&self) -> Option<[i32; 3]> {
        self.result
    }

    /// The number of candidates drawn.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// The chunk of the current candidate, if any.
    pub fn held_chunk(&self) -> Option<[i32; 2]> {
        self.candidate.map(|(x, z)| chunk_of(x, z))
    }

    /// The chunk of the landing. Its ticket stays after the search ends (RespawnSearch.java:199-206),
    /// so the owner of the search must let it go.
    pub fn landing_chunk(&self) -> Option<[i32; 2]> {
        self.result.map(|feet| chunk_of(feet[0], feet[2]))
    }

    /// True when the respawn uses the level and the centre of this search (RespawnSearch.java:170-173).
    pub fn matches(&self, dimension: &str, center: [i32; 3]) -> bool {
        self.dimension == dimension && self.center == center
    }

    /// One step of the search (RespawnSearch.java:180-227, `advance`). It never waits: a candidate whose
    /// chunk is not loaded yet stays for the next call.
    pub fn advance(&mut self, world: &mut impl SearchWorld) {
        if self.is_finished() {
            return;
        }
        if let Some((x, z)) = self.candidate {
            if !world.loaded(x, z) {
                return;
            }
            let shape = world.shape();
            // The biome test of RespawnPositionHandler.java:206-210, now that the chunk is there.
            let wet = world
                .biome(x, shape.sea_level, z)
                .is_some_and(|biome| is_rejected_biome(&biome));
            let found = if wet { None } else { find_safe_position(world, shape, x, z) };
            if let Some(feet) = found {
                // RespawnSearch.java:199-206: the ticket stays, so the chunk is loaded on arrival.
                self.result = Some(feet);
                self.candidate = None;
                return;
            }
            self.release(world);
        }
        // RespawnSearch.java:212-226.
        while self.attempts < MAX_POSITION_ATTEMPTS {
            self.attempts += 1;
            let (x, z) = edge_point(self.center, self.distance, world.random());
            // RespawnPositionHandler.java:201-203: the border test loads no chunk.
            if world.inside_border(x, z) {
                self.candidate = Some((x, z));
                world.hold(chunk_of(x, z));
                return;
            }
        }
        self.failed = true;
    }

    /// The search ends with no spot and gives its chunk back (RespawnSearch.java:229-233, `fail`).
    pub fn fail(&mut self, world: &mut impl SearchWorld) {
        self.release(world);
        self.failed = true;
    }

    /// Gives the chunk of the current candidate back (RespawnSearch.java:235-250).
    pub fn release(&mut self, world: &mut impl SearchWorld) {
        if let Some((x, z)) = self.candidate.take() {
            world.release(chunk_of(x, z));
        }
    }
}

/// True when a search that started `ticks` server ticks ago must fail (RespawnSearch.java:105).
pub fn timed_out(ticks: u32) -> bool {
    ticks > TIMEOUT_TICKS
}

/// A respawn point as the player's data keeps it (vanilla `ServerPlayer.RespawnConfig`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnPoint {
    /// The level of the point.
    pub dimension: String,
    /// The bed head, the anchor or the block of a `/spawnpoint`.
    pub pos: [i32; 3],
    /// Set by `/spawnpoint`.
    pub forced: bool,
}

/// What vanilla's resolution of the point gives at the respawn
/// (`ServerPlayer.findRespawnAndUseSpawnBlock`, Horizon `find_respawn_and_use`, respawn_anchor.rs:477-509).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointUse {
    /// A respawn anchor that works: in the Nether, charged or forced. A charge goes when not forced.
    Anchor,
    /// A bed where beds set the spawn: the Overworld, not a straw bed.
    Bed,
    /// A forced point (`/spawnpoint`) on no bed and no anchor, with room for a player.
    Forced,
    /// Anything else: vanilla forgets the point and the exile centre is the world spawn
    /// (RespawnPositionHandler.java:120-127, `missingRespawnBlock`).
    Missing,
}

/// The block id of a block state string (`minecraft:respawn_anchor[charges=2]` gives
/// `minecraft:respawn_anchor`).
pub fn state_name(state: &str) -> &str {
    state.split('[').next().unwrap_or(state)
}

/// One property of a block state string, None when the state does not have it.
pub fn state_property<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let inside = state.split_once('[')?.1.strip_suffix(']')?;
    inside.split(',').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

/// Room to respawn in this block (vanilla `Block.isPossibleToRespawnInThis`: neither solid nor
/// liquid). Horizon gives no `isSolid` for a block, so a block with a collision box counts as solid.
pub fn respawn_space(block: &BlockInfo) -> bool {
    const LIQUIDS: [&str; 3] = ["minecraft:water", "minecraft:lava", "minecraft:bubble_column"];
    block.boxes.is_empty() && !LIQUIDS.contains(&block.name.as_str())
}

/// Vanilla's resolution of the point at the respawn, in its order (respawn_anchor.rs:483-500):
/// the anchor first, then the bed, then a forced point. `state` is the block state at the point,
/// `free_at` and `free_above` are `respawn_space` of that block and of the one over it. The stand-up
/// search round a bed or an anchor is not done: an obstructed bed counts as one that works.
pub fn point_use(point: &RespawnPoint, state: &str, free_at: bool, free_above: bool) -> PointUse {
    let name = state_name(state);
    let anchor = name == "minecraft:respawn_anchor";
    let charges = state_property(state, "charges").and_then(|c| c.parse::<i32>().ok()).unwrap_or(0);
    // `RespawnAnchorBlock.canSetSpawn`: only the Nether's dimension type lets an anchor work.
    if anchor && (point.forced || charges > 0) && point.dimension == NETHER {
        return PointUse::Anchor;
    }
    // `BedRule.canSetSpawn`: only the Overworld's beds set the spawn, straw beds never.
    if name.ends_with("_bed") && name != "minecraft:straw_bed" && point.dimension == OVERWORLD {
        return PointUse::Bed;
    }
    if point.forced && !anchor && free_at && free_above {
        return PointUse::Forced;
    }
    PointUse::Missing
}

/// The charges an anchor keeps after a respawn uses it, None when it has none to give.
pub fn anchor_after_use(charges: i32) -> Option<i32> {
    (charges > 0).then(|| charges - 1)
}

/// How the point that the game hands to the respawn fits the search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// The handed point is the player's point: the search's level and centre apply.
    Point,
    /// The search was round the world spawn and the game hands its own world spawn, this block.
    /// The search's circle stays; the exile centre is this block, the game's world spawn.
    WorldSpawn([i32; 3]),
}

/// Whether the respawn uses the search (RespawnPositionHandler.java:120-136). `point` is the player's
/// point at death as the game keeps it (`dimension`, block), `centre_is_world_spawn` is true when the
/// search went round the world spawn (no point, or a point that does not work). The game hands the
/// player's point as it keeps it, also a point that does not work (lan_actions.rs:3929-3955). With no
/// point, or in single player with a point outside the Overworld, it hands its world spawn
/// (`level.world_spawn`), which a never-saved `level.dat` does not hold yet (save.rs:4402).
pub fn respawn_fit(
    point: Option<(&str, [i32; 3])>,
    centre_is_world_spawn: bool,
    handed_dimension: &str,
    handed_block: [i32; 3],
) -> Option<Fit> {
    if point == Some((handed_dimension, handed_block)) {
        return Some(Fit::Point);
    }
    let not_an_overworld_point = point.is_none_or(|(dimension, _)| dimension != OVERWORLD);
    (centre_is_world_spawn && handed_dimension == OVERWORLD && not_an_overworld_point)
        .then_some(Fit::WorldSpawn(handed_block))
}

/// True when another player is close enough to hear a `/playsound` at `block` (`SOUND_RANGE`).
/// `others` are the feet positions of the other players in that level.
pub fn someone_hears(block: [i32; 3], others: impl IntoIterator<Item = [f64; 3]>) -> bool {
    let at = block.map(f64::from);
    others.into_iter().any(|p| {
        let d = [p[0] - at[0], p[1] - at[1], p[2] - at[2]];
        d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= SOUND_RANGE * SOUND_RANGE
    })
}

/// The Eternal Items among item ids, one per slot (RespawnPositionHandler.java:228-233).
pub fn count_eternal<'a>(ids: impl IntoIterator<Item = &'a str>) -> i32 {
    let count = ids.into_iter().filter(|id| crate::ids::is_eternal(id)).count();
    i32::try_from(count).unwrap_or(i32::MAX)
}

/// The 200 ticks of damage immunity after a distant respawn, by network entity id
/// (F_ServerPlayerJourneyMixin.java). Java keeps the count on the `ServerPlayer`, and a respawn makes a
/// new one at 0: so a death clears the entry here (review parity-3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shields {
    ticks: std::collections::BTreeMap<i32, i32>,
}

impl Shields {
    /// No immunity for anyone.
    pub const fn new() -> Shields {
        Shields { ticks: std::collections::BTreeMap::new() }
    }

    /// Starts the immunity of a distant respawn (RespawnEffectHandler.java:59).
    pub fn start(&mut self, entity: i32) {
        self.ticks.insert(entity, JOURNEY_INVULNERABLE_TICKS);
    }

    /// One tick of the player (F_ServerPlayerJourneyMixin.java:34-40).
    pub fn tick(&mut self, entity: i32) {
        if let Some(ticks) = self.ticks.get_mut(&entity) {
            *ticks = count_down(*ticks);
        }
        self.ticks.retain(|_, ticks| *ticks > 0);
    }

    /// The player died or left: the immunity ends.
    pub fn clear(&mut self, entity: i32) {
        self.ticks.remove(&entity);
    }

    /// True when the immunity stops this damage.
    pub fn blocks(&self, entity: i32, damage_type: &str) -> bool {
        blocks_damage(self.ticks.get(&entity).copied().unwrap_or(0), damage_type)
    }
}

/// The decision inside the respawn (RespawnPositionHandler.java:94-160, `findRespawnPosition`), for
/// the parts that need no world read. The conditions keep the Java order: the rule, the game mode, then
/// a finished search for this level and this centre. Horizon cannot load a far chunk inside the respawn,
/// so a search that did not end, or that does not fit, keeps the vanilla point (GAPS.md C4).
/// Returns the feet block of the landing, or None for the vanilla point.
pub fn landing(
    rule_on: bool,
    game_mode: i32,
    search: Option<&Search>,
    dimension: &str,
    center: [i32; 3],
) -> Option<[i32; 3]> {
    // RespawnPositionHandler.java:108-118: the rule, then survival (game mode 0).
    if !rule_on || game_mode != 0 {
        return None;
    }
    let search = search?;
    // RespawnPositionHandler.java:134-136.
    if search.is_finished() && search.matches(dimension, center) {
        search.result()
    } else {
        None
    }
}

/// The position of the player for a feet block (vanilla `Vec3.atBottomCenterOf`,
/// F_PlayerListRespawnMixin.java:61).
pub fn bottom_center(feet: [i32; 3]) -> [f64; 3] {
    [f64::from(feet[0]) + 0.5, f64::from(feet[1]), f64::from(feet[2]) + 0.5]
}

/// The block of a position (vanilla `BlockPos.containing`).
pub fn block_of(pos: [f64; 3]) -> [i32; 3] {
    // `as` saturates and gives 0 for NaN: a position from the game is finite.
    [pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32]
}

/// True when the departure signal goes and the search starts (JourneySignal.java:30,
/// RespawnPositionHandler.java:71-75): the rule is on, the player is in survival and the world is not
/// hardcore.
pub fn is_distant_respawn(rule_on: bool, game_mode: i32, hardcore: bool) -> bool {
    rule_on && game_mode == 0 && !hardcore
}

/// The journey advancements of one distant respawn, in the Java order (RespawnEffectHandler.java:43-51).
/// `deaths` is the vanilla statistic `minecraft:deaths`, `distance` the exile distance in Overworld
/// blocks.
pub fn journey_awards(deaths: i32, distance: i32) -> Vec<&'static str> {
    let mut awards = vec![ADV_EXILE];
    if deaths >= AGAIN_DEATHS {
        awards.push(ADV_AGAIN);
    }
    if distance >= FAR_AWAY_BLOCKS {
        awards.push(ADV_FAR_AWAY);
    }
    awards
}

/// True when a player name is safe in a command line and a selector: the vanilla name characters.
pub fn is_command_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 16
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The console line that awards an advancement of BMH (BmhAdvancements.java:31-36, `award`): Horizon
/// awards it through the vanilla command (verify-1 row 2d). None for a name that is not a player name.
pub fn award_command(name: &str, path: &str) -> Option<String> {
    is_command_name(name).then(|| format!("advancement grant {name} only bmh:{path} {CRITERION}"))
}

/// The two console lines of the arrival sounds for the other players near `block`
/// (RespawnEffectHandler.java:68-93). Java plays them at the block position, not at its centre, to
/// every player in range except the respawned one. `/playsound` keeps the range of 16 blocks for a
/// volume of 1 or less, as `playSeededSound` does. None for a name that is not a player name.
pub fn sound_commands(name: &str, block: [i32; 3]) -> Option<[String; 2]> {
    if !is_command_name(name) {
        return None;
    }
    let [x, y, z] = block;
    let line = |sound: &str, volume: f32| {
        format!("playsound {sound} player @a[name=!{name}] {x} {y} {z} {volume} {SOUND_PITCH}")
    };
    Some([line(BELL_SOUND, BELL_VOLUME), line(RESPAWN_SOUND, RESPAWN_VOLUME)])
}

/// True when the immunity stops this damage (F_ServerPlayerJourneyMixin.java:42-48): ticks are left and
/// the damage type does not bypass invulnerability.
pub fn blocks_damage(ticks_left: i32, damage_type: &str) -> bool {
    ticks_left > 0 && !BYPASSES_INVULNERABILITY.contains(&damage_type)
}

/// The immunity after one tick (F_ServerPlayerJourneyMixin.java:34-40).
pub fn count_down(ticks_left: i32) -> i32 {
    if ticks_left > 0 { ticks_left - 1 } else { ticks_left }
}

/// A small random source (SplitMix64) for the candidate angles. Java uses the level's random; the
/// draws cannot be the same, the distribution is (uniform in [0, 1)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A source from a seed.
    pub const fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    /// The next value in [0, 1), with 53 random bits as `Random.nextDouble` has.
    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Mixes more bits into the state (the time of a death, a player id).
    pub fn mix(&mut self, bits: u64) {
        self.state ^= bits.rotate_left(17);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    const DEFAULT: Config = Config::DEFAULT;

    #[test]
    fn the_distance_follows_the_java_formula() {
        // 5000 + 55000 * 3 / 9 = 23333 (one easy life, normal difficulty, no Eternal Item).
        assert_eq!(distance(DEFAULT, 0, 2), 23_333);
        // 5000 + 55000 * 1 / 9 = 11111 (peaceful).
        assert_eq!(distance(DEFAULT, 0, 0), 11_111);
        // 5000 + 55000 * 5 / 9 = 35555 (two Eternal Items, normal).
        assert_eq!(distance(DEFAULT, 2, 2), 35_555);
        // 1 + 6 + 3 = 10, capped at 9: the max.
        assert_eq!(distance(DEFAULT, 6, 3), 60_000);
        assert_eq!(distance(DEFAULT, 100, 3), 60_000);
    }

    #[test]
    fn a_min_above_the_max_uses_the_min_for_both() {
        let config = Config { min_distance: 10_000, max_distance: 5000, max_amulet_points: 5 };
        assert_eq!(distance(config, 3, 3), 10_000);
        let equal = Config { min_distance: 7000, max_distance: 7000, max_amulet_points: 5 };
        assert_eq!(distance(equal, 0, 0), 7000);
    }

    #[test]
    fn the_config_extremes_do_not_overflow() {
        let wide = Config { min_distance: 5000, max_distance: 200_000, max_amulet_points: 5 };
        assert_eq!(distance(wide, 0, 0), 5000 + 195_000 / 9);
        assert_eq!(distance(wide, i32::MAX, i32::MAX), 200_000);
        assert_eq!(distance(wide, -5, -5), 5000 + 195_000 / 9);
    }

    #[test]
    fn the_difficulty_ids_are_vanilla() {
        assert_eq!(difficulty_id("peaceful"), Some(0));
        assert_eq!(difficulty_id("easy"), Some(1));
        assert_eq!(difficulty_id("normal"), Some(2));
        assert_eq!(difficulty_id("minecraft:hard"), Some(3));
        assert_eq!(difficulty_id("nightmare"), None);
    }

    #[test]
    fn the_nether_divides_the_distance_by_eight() {
        assert_eq!(level_distance(23_333, coordinate_scale(OVERWORLD)), 23_333);
        // 23333 / 8 + 0.5 = 2917.125.
        assert_eq!(level_distance(23_333, coordinate_scale(NETHER)), 2917);
        // 60000 / 8 + 0.5 = 7500.5.
        assert_eq!(level_distance(60_000, coordinate_scale(NETHER)), 7500);
        assert_eq!(level_distance(5000, coordinate_scale("minecraft:the_end")), 5000);
    }

    #[test]
    fn the_level_distance_is_clamped() {
        assert_eq!(level_distance(-5, 1.0), 0);
        assert_eq!(level_distance(i32::MAX, 1.0e-9), 29_999_984);
        assert_eq!(level_distance(100, f64::NAN), 0);
    }

    #[test]
    fn the_edge_point_truncates_toward_zero() {
        let center = [100, 64, -200];
        assert_eq!(edge_point(center, 1000, 0.0), (1100, -200));
        // A quarter turn: cos is 6e-17, which truncates to 0.
        assert_eq!(edge_point(center, 1000, 0.25), (100, 800));
        assert_eq!(edge_point(center, 1000, 0.5), (-900, -200));
        assert_eq!(edge_point(center, 1000, 0.75), (100, -1200));
        // 1000 * cos(45 degrees) = 707.1, cut to 707; the negative side cuts to -707.
        assert_eq!(edge_point([0, 0, 0], 1000, 0.125), (707, 707));
        assert_eq!(edge_point([0, 0, 0], 1000, 0.625), (-707, -707));
    }

    #[test]
    fn the_border_includes_its_min_and_excludes_its_max() {
        let border = [-100.0, -50.0, 100.0, 50.0];
        assert!(inside_border(border, -100, -50));
        assert!(inside_border(border, 99, 49));
        assert!(!inside_border(border, 100, 0));
        assert!(!inside_border(border, 0, 50));
        assert!(!inside_border(border, -101, 0));
    }

    #[test]
    fn oceans_and_rivers_are_rejected() {
        assert!(is_rejected_biome("minecraft:deep_lukewarm_ocean"));
        assert!(is_rejected_biome("minecraft:frozen_river"));
        assert!(!is_rejected_biome("minecraft:beach"));
        assert!(!is_rejected_biome("minecraft:plains"));
        assert!(!is_rejected_biome("minecraft:mushroom_fields"));
    }

    fn full(name: &str) -> BlockInfo {
        BlockInfo { name: name.into(), waterlogged: false, boxes: vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]] }
    }

    fn air() -> BlockInfo {
        BlockInfo { name: "minecraft:air".into(), ..BlockInfo::default() }
    }

    fn water() -> BlockInfo {
        BlockInfo { name: "minecraft:water".into(), ..BlockInfo::default() }
    }

    #[test]
    fn the_block_tests_follow_vanilla() {
        assert!(is_free(&air()));
        assert!(is_free(&BlockInfo { name: "minecraft:short_grass".into(), ..BlockInfo::default() }));
        assert!(!is_free(&water()));
        assert!(!is_free(&BlockInfo { name: "minecraft:kelp".into(), ..BlockInfo::default() }));
        assert!(!is_free(&full("minecraft:stone")));
        assert!(is_safe_ground(&full("minecraft:grass_block")));
        assert!(!is_safe_ground(&full("minecraft:oak_leaves")));
        assert!(!is_safe_ground(&full("minecraft:magma_block")));
        let mut wet_slab = full("minecraft:oak_slab");
        wet_slab.waterlogged = true;
        assert!(!is_safe_ground(&wet_slab));
        let bottom_slab = BlockInfo {
            name: "minecraft:oak_slab".into(),
            waterlogged: false,
            boxes: vec![[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]],
        };
        assert!(!is_safe_ground(&bottom_slab));
        // A top-half stair: a full top slab plus a lower box.
        let top_stair = BlockInfo {
            name: "minecraft:oak_stairs".into(),
            waterlogged: false,
            boxes: vec![[0.0, 0.5, 0.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.5, 0.5, 1.0]],
        };
        assert!(is_safe_ground(&top_stair));
        // Two halves that cover the face together.
        let halves = BlockInfo {
            name: "minecraft:test".into(),
            waterlogged: false,
            boxes: vec![[0.0, 0.0, 0.0, 0.5, 1.0, 1.0], [0.5, 0.0, 0.0, 1.0, 1.0, 1.0]],
        };
        assert!(is_safe_ground(&halves));
        let soul_sand = BlockInfo {
            name: "minecraft:soul_sand".into(),
            waterlogged: false,
            boxes: vec![[0.0, 0.0, 0.0, 1.0, 0.875, 1.0]],
        };
        assert!(is_safe_ground(&soul_sand));
        let path = BlockInfo {
            name: "minecraft:dirt_path".into(),
            waterlogged: false,
            boxes: vec![[0.0, 0.0, 0.0, 1.0, 0.9375, 1.0]],
        };
        assert!(!is_safe_ground(&path));
    }

    /// A fake level: columns of blocks by (x, z), air elsewhere, and a set of loaded chunks.
    #[derive(Default)]
    struct Fake {
        columns: BTreeMap<(i32, i32), Vec<(i32, BlockInfo)>>,
        biomes: BTreeMap<(i32, i32), String>,
        loaded: BTreeSet<[i32; 2]>,
        held: Vec<[i32; 2]>,
        released: Vec<[i32; 2]>,
        randoms: Vec<f64>,
        border: Option<[f64; 4]>,
        shape: Option<LevelShape>,
        reads: u32,
        biome_y: Vec<i32>,
    }

    impl Fake {
        fn ground(&mut self, x: i32, z: i32, top: i32, block: BlockInfo) {
            let column = self.columns.entry((x, z)).or_default();
            for y in -64..top {
                column.push((y, full("minecraft:stone")));
            }
            column.push((top, block));
        }
        fn put(&mut self, x: i32, y: i32, z: i32, block: BlockInfo) {
            self.columns.entry((x, z)).or_default().push((y, block));
        }
    }

    impl Blocks for Fake {
        fn block(&mut self, x: i32, y: i32, z: i32) -> BlockInfo {
            self.reads += 1;
            self.columns
                .get(&(x, z))
                .and_then(|c| c.iter().rev().find(|(at, _)| *at == y))
                .map_or_else(air, |(_, b)| b.clone())
        }
        fn surface(&mut self, x: i32, z: i32) -> Option<i32> {
            if !self.loaded.contains(&chunk_of(x, z)) {
                return None;
            }
            let top = self.columns.get(&(x, z)).and_then(|c| {
                c.iter().filter(|(_, b)| !b.boxes.is_empty() || has_fluid(b)).map(|(y, _)| *y).max()
            });
            Some(top.map_or(-64, |y| y + 1))
        }
    }

    impl SearchWorld for Fake {
        fn random(&mut self) -> f64 {
            if self.randoms.is_empty() { 0.0 } else { self.randoms.remove(0) }
        }
        fn inside_border(&mut self, x: i32, z: i32) -> bool {
            self.border.is_none_or(|b| inside_border(b, x, z))
        }
        fn hold(&mut self, chunk: [i32; 2]) {
            self.held.push(chunk);
        }
        fn release(&mut self, chunk: [i32; 2]) {
            self.released.push(chunk);
        }
        fn loaded(&mut self, x: i32, z: i32) -> bool {
            self.loaded.contains(&chunk_of(x, z))
        }
        fn biome(&mut self, x: i32, y: i32, z: i32) -> Option<String> {
            self.biome_y.push(y);
            self.biomes.get(&(x, z)).cloned()
        }
        fn shape(&mut self) -> LevelShape {
            self.shape.unwrap_or(LevelShape::of(OVERWORLD, -64, 384))
        }
    }

    const OW: LevelShape = LevelShape { min_y: -64, has_ceiling: false, logical_height: 384, sea_level: 63 };

    #[test]
    fn flat_grass_gives_the_block_above_it() {
        let mut world = Fake::default();
        world.ground(10, 20, 64, full("minecraft:grass_block"));
        world.loaded.insert(chunk_of(10, 20));
        assert_eq!(find_safe_position(&mut world, OW, 10, 20), Some([10, 65, 20]));
    }

    #[test]
    fn plants_and_snow_layers_are_walked_through() {
        let mut world = Fake::default();
        world.ground(0, 0, 70, full("minecraft:dirt"));
        world.put(0, 71, 0, BlockInfo { name: "minecraft:short_grass".into(), ..BlockInfo::default() });
        world.loaded.insert([0, 0]);
        assert_eq!(find_safe_position(&mut world, OW, 0, 0), Some([0, 71, 0]));
    }

    #[test]
    fn water_and_lava_are_refused() {
        let mut world = Fake::default();
        world.ground(0, 0, 60, full("minecraft:sand"));
        world.put(0, 61, 0, water());
        world.loaded.insert([0, 0]);
        assert_eq!(find_safe_position(&mut world, OW, 0, 0), None);
        let mut lava = Fake::default();
        lava.ground(0, 0, 60, BlockInfo { name: "minecraft:lava".into(), ..BlockInfo::default() });
        lava.loaded.insert([0, 0]);
        assert_eq!(find_safe_position(&mut lava, OW, 0, 0), None);
    }

    #[test]
    fn a_low_roof_is_refused() {
        let mut world = Fake::default();
        world.ground(0, 0, 64, full("minecraft:stone"));
        world.put(0, 66, 0, full("minecraft:stone"));
        world.loaded.insert([0, 0]);
        // The heightmap is over the roof (67): the walk stops on the roof, whose head room is free.
        assert_eq!(find_safe_position(&mut world, OW, 0, 0), Some([0, 67, 0]));
        // A roof with one block of head room under it, and a second roof: the walk stops on the top one.
        let mut cave = Fake::default();
        cave.ground(0, 0, 64, full("minecraft:stone"));
        cave.put(0, 66, 0, full("minecraft:stone"));
        cave.put(0, 67, 0, full("minecraft:stone"));
        cave.put(0, 69, 0, BlockInfo { name: "minecraft:magma_block".into(), ..full("minecraft:magma_block") });
        cave.loaded.insert([0, 0]);
        assert_eq!(find_safe_position(&mut cave, OW, 0, 0), None);
    }

    #[test]
    fn an_unloaded_chunk_gives_none() {
        let mut world = Fake::default();
        world.ground(0, 0, 64, full("minecraft:stone"));
        assert_eq!(find_safe_position(&mut world, OW, 0, 0), None);
    }

    #[test]
    fn the_void_is_refused() {
        let mut world = Fake::default();
        world.loaded.insert([0, 0]);
        assert_eq!(find_safe_position(&mut world, OW, 0, 0), None);
    }

    #[test]
    fn the_nether_starts_under_the_roof() {
        let shape = LevelShape::of(NETHER, 0, 256);
        assert_eq!(shape.logical_height, 128);
        let mut world = Fake::default();
        // Netherrack from 0 to 40, air from 41 to 99, netherrack from 100 to 127.
        for y in 0..=40 {
            world.put(5, y, 5, full("minecraft:netherrack"));
        }
        for y in 100..=127 {
            world.put(5, y, 5, full("minecraft:netherrack"));
        }
        world.loaded.insert([0, 0]);
        // Start at 126 (solid), go down to 99 (free), then down to the floor at 40.
        assert_eq!(find_safe_position(&mut world, shape, 5, 5), Some([5, 41, 5]));
    }

    #[test]
    fn the_search_draws_holds_and_finds() {
        let mut world = Fake::default();
        world.randoms = vec![0.0];
        world.ground(1100, 0, 70, full("minecraft:grass_block"));
        let mut search = Search::new(OVERWORLD, [100, 64, 0], 1000);
        search.advance(&mut world);
        assert_eq!(search.attempts(), 1);
        assert_eq!(world.held, vec![chunk_of(1100, 0)]);
        assert!(!search.is_finished());
        // The chunk is not loaded: the search waits and reads nothing.
        search.advance(&mut world);
        assert_eq!(world.reads, 0);
        assert!(!search.is_finished());
        world.loaded.insert(chunk_of(1100, 0));
        search.advance(&mut world);
        assert_eq!(search.result(), Some([1100, 71, 0]));
        // The ticket of the landing stays.
        assert!(world.released.is_empty());
        assert_eq!(search.held_chunk(), None);
        assert_eq!(search.landing_chunk(), Some(chunk_of(1100, 0)));
        assert!(search.matches(OVERWORLD, [100, 64, 0]));
        assert!(!search.matches(NETHER, [100, 64, 0]));
        assert!(!search.matches(OVERWORLD, [100, 65, 0]));
    }

    #[test]
    fn a_wet_or_unsafe_candidate_gives_its_chunk_back() {
        let mut world = Fake::default();
        // First draw east (ocean), second draw west (water surface), third draw south (grass).
        world.randoms = vec![0.0, 0.5, 0.25];
        world.biomes.insert((1000, 0), "minecraft:ocean".into());
        world.ground(-1000, 0, 60, full("minecraft:sand"));
        world.put(-1000, 61, 0, water());
        world.ground(0, 1000, 64, full("minecraft:grass_block"));
        for chunk in [chunk_of(1000, 0), chunk_of(-1000, 0), chunk_of(0, 1000)] {
            world.loaded.insert(chunk);
        }
        let mut search = Search::new(OVERWORLD, [0, 64, 0], 1000);
        search.advance(&mut world);
        search.advance(&mut world);
        assert_eq!(world.released, vec![chunk_of(1000, 0)]);
        search.advance(&mut world);
        assert_eq!(world.released, vec![chunk_of(1000, 0), chunk_of(-1000, 0)]);
        search.advance(&mut world);
        assert_eq!(search.result(), Some([0, 65, 1000]));
        assert_eq!(search.attempts(), 3);
    }

    #[test]
    fn the_border_rules_out_a_candidate_without_a_ticket() {
        let mut world = Fake::default();
        world.border = Some([-500.0, -500.0, 500.0, 500.0]);
        let mut search = Search::new(OVERWORLD, [0, 64, 0], 1000);
        search.advance(&mut world);
        // Every point of the circle is outside: 32 draws, no ticket, the search fails.
        assert!(search.is_finished());
        assert_eq!(search.result(), None);
        assert_eq!(search.attempts(), MAX_POSITION_ATTEMPTS);
        assert!(world.held.is_empty());
    }

    #[test]
    fn the_search_fails_after_32_candidates() {
        let mut world = Fake::default();
        let mut search = Search::new(OVERWORLD, [0, 64, 0], 100);
        // Every candidate is the same loaded void column.
        world.loaded.insert(chunk_of(100, 0));
        for _ in 0..MAX_POSITION_ATTEMPTS {
            search.advance(&mut world);
            assert!(!search.is_finished());
        }
        assert_eq!(search.attempts(), MAX_POSITION_ATTEMPTS);
        search.advance(&mut world);
        assert!(search.is_finished());
        assert_eq!(search.result(), None);
        assert_eq!(world.held.len(), 32);
        assert_eq!(world.released.len(), 32);
        // A finished search does nothing more.
        search.advance(&mut world);
        assert_eq!(world.held.len(), 32);
    }

    #[test]
    fn a_failed_search_gives_its_chunk_back() {
        let mut world = Fake::default();
        let mut search = Search::new(OVERWORLD, [0, 64, 0], 100);
        search.advance(&mut world);
        assert_eq!(search.held_chunk(), Some(chunk_of(100, 0)));
        search.fail(&mut world);
        assert!(search.is_finished());
        assert_eq!(world.released, vec![chunk_of(100, 0)]);
        assert_eq!(search.held_chunk(), None);
    }

    #[test]
    fn the_timeout_is_120_seconds_of_server_ticks() {
        // 120 000 ms / 50 ms a tick.
        assert_eq!(TIMEOUT_TICKS, 2400);
        assert!(!timed_out(0));
        assert!(!timed_out(2400));
        assert!(timed_out(2401));
    }

    #[test]
    fn the_biome_test_reads_at_sea_level() {
        // RespawnPositionHandler.java:206: `atY(serverLevel.getSeaLevel())`.
        for (dimension, sea) in [(OVERWORLD, 63), (NETHER, 32), ("minecraft:the_end", 0)] {
            let mut world = Fake::default();
            world.randoms = vec![0.0];
            world.shape = Some(LevelShape::of(dimension, if dimension == OVERWORLD { -64 } else { 0 }, 256));
            world.loaded.insert(chunk_of(100, 0));
            let mut search = Search::new(dimension, [0, 64, 0], 100);
            search.advance(&mut world);
            search.advance(&mut world);
            assert_eq!(world.biome_y, vec![sea], "{dimension}");
        }
    }

    #[test]
    fn chunks_of_negative_columns_round_down() {
        assert_eq!(chunk_of(-1, -17), [-1, -2]);
        assert_eq!(chunk_of(-16, 15), [-1, 0]);
        assert_eq!(chunk_of(-17, 16), [-2, 1]);
        assert_eq!(chunk_of(0, -16), [0, -1]);
    }

    #[test]
    fn a_fractional_border_keeps_min_inclusive_max_exclusive() {
        let border = [-100.5, -100.5, 100.5, 100.5];
        assert!(!inside_border(border, -101, 0));
        assert!(inside_border(border, -100, 0));
        assert!(inside_border(border, 100, 100));
        assert!(!inside_border(border, 101, 0));
        let tight = [10.25, 10.25, 10.75, 10.75];
        assert!(!inside_border(tight, 10, 10), "10 is under the min 10.25");
        assert!(!inside_border(tight, 11, 11), "11 is over the max 10.75");
    }

    #[test]
    fn block_states_split_into_name_and_properties() {
        let state = "minecraft:respawn_anchor[charges=2]";
        assert_eq!(state_name(state), "minecraft:respawn_anchor");
        assert_eq!(state_property(state, "charges"), Some("2"));
        assert_eq!(state_property(state, "facing"), None);
        assert_eq!(state_name("minecraft:air"), "minecraft:air");
        assert_eq!(state_property("minecraft:air", "charges"), None);
        let bed = "minecraft:red_bed[facing=east,occupied=false,part=head]";
        assert_eq!(state_name(bed), "minecraft:red_bed");
        assert_eq!(state_property(bed, "part"), Some("head"));
        // A broken string gives no property and no panic.
        assert_eq!(state_property("minecraft:x[charges=1", "charges"), None);
    }

    fn point(dimension: &str, forced: bool) -> RespawnPoint {
        RespawnPoint { dimension: dimension.into(), pos: [5, 70, -3], forced }
    }

    #[test]
    fn the_point_resolves_as_vanilla_does() {
        let anchor = |c: i32| format!("minecraft:respawn_anchor[charges={c}]");
        // A charged anchor in the Nether works; an empty one is gone unless forced.
        assert_eq!(point_use(&point(NETHER, false), &anchor(1), false, false), PointUse::Anchor);
        assert_eq!(point_use(&point(NETHER, false), &anchor(0), false, false), PointUse::Missing);
        assert_eq!(point_use(&point(NETHER, true), &anchor(0), false, false), PointUse::Anchor);
        // An anchor works only in the Nether, even forced and with room.
        assert_eq!(point_use(&point(OVERWORLD, false), &anchor(4), true, true), PointUse::Missing);
        assert_eq!(point_use(&point(OVERWORLD, true), &anchor(4), true, true), PointUse::Missing);
        // A bed works in the Overworld only, and a straw bed never.
        let bed = "minecraft:white_bed[facing=north,occupied=false,part=head]";
        assert_eq!(point_use(&point(OVERWORLD, false), bed, false, false), PointUse::Bed);
        assert_eq!(point_use(&point(NETHER, false), bed, false, false), PointUse::Missing);
        assert_eq!(point_use(&point(OVERWORLD, false), "minecraft:straw_bed[part=head]", false, false), PointUse::Missing);
        // A bed that is gone.
        assert_eq!(point_use(&point(OVERWORLD, false), "minecraft:air", true, true), PointUse::Missing);
        // A forced point holds while a player fits in it, in any level.
        assert_eq!(point_use(&point(OVERWORLD, true), "minecraft:air", true, true), PointUse::Forced);
        assert_eq!(point_use(&point(NETHER, true), "minecraft:air", true, true), PointUse::Forced);
        assert_eq!(point_use(&point(OVERWORLD, true), "minecraft:stone", false, true), PointUse::Missing);
        assert_eq!(point_use(&point(OVERWORLD, true), "minecraft:air", true, false), PointUse::Missing);
        assert_eq!(point_use(&point(OVERWORLD, false), "minecraft:air", true, true), PointUse::Missing);
    }

    #[test]
    fn respawn_space_is_neither_solid_nor_liquid() {
        assert!(respawn_space(&air()));
        assert!(respawn_space(&BlockInfo { name: "minecraft:oak_sign".into(), ..BlockInfo::default() }));
        assert!(!respawn_space(&full("minecraft:stone")));
        assert!(!respawn_space(&water()));
        assert!(!respawn_space(&BlockInfo { name: "minecraft:lava".into(), ..BlockInfo::default() }));
    }

    #[test]
    fn an_anchor_gives_one_charge_a_respawn() {
        assert_eq!(anchor_after_use(4), Some(3));
        assert_eq!(anchor_after_use(1), Some(0));
        assert_eq!(anchor_after_use(0), None);
    }

    #[test]
    fn the_handed_point_must_fit_the_search() {
        let bed = Some((OVERWORLD, [5, 70, -3]));
        // The game hands the bed the search was made for (working or not).
        assert_eq!(respawn_fit(bed, false, OVERWORLD, [5, 70, -3]), Some(Fit::Point));
        assert_eq!(respawn_fit(bed, true, OVERWORLD, [5, 70, -3]), Some(Fit::Point));
        // Another point: no fit (Java searched again; Horizon cannot).
        assert_eq!(respawn_fit(bed, false, OVERWORLD, [6, 70, -3]), None);
        assert_eq!(respawn_fit(bed, true, OVERWORLD, [0, 126, 0]), None);
        // No point: the game's own world spawn fits a world-spawn search, also when level.dat had
        // another one (a world never saved, game-b0).
        assert_eq!(respawn_fit(None, true, OVERWORLD, [0, 126, 0]), Some(Fit::WorldSpawn([0, 126, 0])));
        assert_eq!(respawn_fit(None, true, NETHER, [0, 126, 0]), None);
        // A Nether anchor that is gone: single player hands the Overworld world spawn.
        let anchor = Some((NETHER, [10, 40, 10]));
        assert_eq!(respawn_fit(anchor, true, OVERWORLD, [0, 126, 0]), Some(Fit::WorldSpawn([0, 126, 0])));
        // A working Nether anchor in single player: the game hands the world spawn, no fit.
        assert_eq!(respawn_fit(anchor, false, OVERWORLD, [0, 126, 0]), None);
        // A dedicated server hands the anchor itself.
        assert_eq!(respawn_fit(anchor, false, NETHER, [10, 40, 10]), Some(Fit::Point));
    }

    #[test]
    fn a_sound_needs_a_player_within_16_blocks() {
        let block = [100, 64, -20];
        assert!(!someone_hears(block, []));
        assert!(someone_hears(block, [[116.0, 64.0, -20.0]]), "16 blocks away hears");
        assert!(!someone_hears(block, [[116.1, 64.0, -20.0]]));
        assert!(someone_hears(block, [[500.0, 0.0, 0.0], [109.0, 73.0, -11.0]]), "15.6 blocks");
        assert!(!someone_hears(block, [[110.0, 74.0, -10.0]]), "17.3 blocks");
    }

    #[test]
    fn eternal_items_are_counted_by_id() {
        assert_eq!(count_eternal([]), 0);
        assert_eq!(
            count_eternal(["bmh:eternal_sword", "minecraft:stone", "bmh:sleeping_totem", "bmh:dormant_totem"]),
            2
        );
    }

    #[test]
    fn the_shield_lasts_200_ticks_and_a_death_ends_it() {
        let mut shields = Shields::new();
        assert!(!shields.blocks(7, "minecraft:fall"));
        shields.start(7);
        for _ in 0..199 {
            shields.tick(7);
        }
        assert!(shields.blocks(7, "minecraft:fall"));
        assert!(!shields.blocks(7, "minecraft:generic_kill"));
        assert!(!shields.blocks(8, "minecraft:fall"), "another entity");
        shields.tick(7);
        assert!(!shields.blocks(7, "minecraft:fall"));
        assert_eq!(shields, Shields::new(), "an ended entry goes");
        // A death in the middle of the immunity ends it (review parity-3).
        shields.start(7);
        shields.tick(7);
        shields.clear(7);
        assert!(!shields.blocks(7, "minecraft:lava"));
    }

    #[test]
    fn the_respawn_keeps_the_java_order_of_conditions() {
        let mut world = Fake::default();
        world.ground(1000, 0, 64, full("minecraft:grass_block"));
        world.loaded.insert(chunk_of(1000, 0));
        let mut search = Search::new(OVERWORLD, [0, 64, 0], 1000);
        let center = [0, 64, 0];
        assert_eq!(landing(true, 0, Some(&search), OVERWORLD, center), None, "not finished");
        search.advance(&mut world);
        search.advance(&mut world);
        assert_eq!(landing(true, 0, Some(&search), OVERWORLD, center), Some([1000, 65, 0]));
        assert_eq!(landing(false, 0, Some(&search), OVERWORLD, center), None, "rule off");
        assert_eq!(landing(true, 1, Some(&search), OVERWORLD, center), None, "creative");
        assert_eq!(landing(true, 0, Some(&search), OVERWORLD, [1, 64, 0]), None, "another centre");
        assert_eq!(landing(true, 0, Some(&search), NETHER, center), None, "another level");
        assert_eq!(landing(true, 0, None, OVERWORLD, center), None, "no search");
    }

    #[test]
    fn positions_and_blocks_round_trip() {
        assert_eq!(bottom_center([10, 65, -3]), [10.5, 65.0, -2.5]);
        assert_eq!(block_of([10.5, 65.0, -2.5]), [10, 65, -3]);
        assert_eq!(block_of([-0.5, 64.6, 0.5]), [-1, 64, 0]);
    }

    #[test]
    fn the_departure_needs_the_rule_survival_and_no_hardcore() {
        assert!(is_distant_respawn(true, 0, false));
        assert!(!is_distant_respawn(false, 0, false));
        assert!(!is_distant_respawn(true, 2, false));
        assert!(!is_distant_respawn(true, 0, true));
    }

    #[test]
    fn the_awards_follow_the_java_order() {
        assert_eq!(journey_awards(1, 5000), vec![ADV_EXILE]);
        assert_eq!(journey_awards(10, 5000), vec![ADV_EXILE, ADV_AGAIN]);
        assert_eq!(journey_awards(9, 30_000), vec![ADV_EXILE, ADV_FAR_AWAY]);
        assert_eq!(journey_awards(42, 60_000), vec![ADV_EXILE, ADV_AGAIN, ADV_FAR_AWAY]);
        assert_eq!(journey_awards(0, 29_999), vec![ADV_EXILE]);
    }

    #[test]
    fn the_command_lines_are_exact() {
        assert_eq!(
            award_command("Alex", ADV_EXILE).as_deref(),
            Some("advancement grant Alex only bmh:journey/exile done")
        );
        assert_eq!(award_command("bad name", ADV_EXILE), None);
        assert_eq!(award_command("", ADV_EXILE), None);
        assert_eq!(award_command("@a", ADV_EXILE), None);
        let lines = sound_commands("Steve_2", [10, 64, -3]);
        assert_eq!(
            lines,
            Some([
                "playsound bmh:bell_sfx player @a[name=!Steve_2] 10 64 -3 1 1".to_owned(),
                "playsound bmh:end player @a[name=!Steve_2] 10 64 -3 0.3 1".to_owned(),
            ])
        );
        assert_eq!(sound_commands("x]y", [0, 0, 0]), None);
        assert_eq!(sound_commands("abcdefghijklmnopq", [0, 0, 0]), None);
    }

    #[test]
    fn the_immunity_stops_damage_but_not_the_void() {
        assert!(blocks_damage(200, "minecraft:fall"));
        assert!(blocks_damage(1, "minecraft:player_attack"));
        assert!(!blocks_damage(0, "minecraft:fall"));
        assert!(!blocks_damage(200, "minecraft:out_of_world"));
        assert!(!blocks_damage(200, "minecraft:generic_kill"));
        let mut ticks = JOURNEY_INVULNERABLE_TICKS;
        for _ in 0..200 {
            assert!(blocks_damage(ticks, "minecraft:lava"));
            ticks = count_down(ticks);
        }
        assert_eq!(ticks, 0);
        assert_eq!(count_down(0), 0);
        assert!(!blocks_damage(ticks, "minecraft:lava"));
    }

    #[test]
    fn the_random_source_is_uniform_and_in_range() {
        let mut rng = Rng::new(42);
        let mut low = 0;
        for _ in 0..10_000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v));
            if v < 0.5 {
                low += 1;
            }
        }
        assert!((4800..5200).contains(&low), "{low}");
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        assert_eq!(a.next_f64(), b.next_f64());
        b.mix(99);
        assert_ne!(a.next_f64(), b.next_f64());
    }

    #[test]
    fn the_constants_are_the_java_values() {
        assert_eq!(MAX_EASY_LIFE, 9);
        assert_eq!(MAX_POSITION_ATTEMPTS, 32);
        assert_eq!(TIMEOUT_MS, 120_000);
        assert_eq!(LANDING_TICKET_TICKS, 6000);
        assert_eq!(JOURNEY_INVULNERABLE_TICKS, 200);
        assert_eq!(AGAIN_DEATHS, 10);
        assert_eq!(FAR_AWAY_BLOCKS, 30_000);
    }
}
