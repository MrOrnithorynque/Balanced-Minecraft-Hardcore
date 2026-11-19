// Area amulet: the edge between the game and `bmh_logic::amulet`.
// It holds the Navigator's Amulet: the use and the use on a block, the star reading and its charges,
// the named points on the player, `/bmh point`, and the naming and list screens. Every decision is in
// `bmh_logic::amulet`; this file reads the game and carries out the answers.
// Owner: the amulet agent. Log: docs/horizon/log/amulet.md.
//
// Where the data lives:
// - The charges and the last reading are on the stack, in `minecraft:custom_data`, with the keys of the
//   Java mod (AmuletReading.java:34-52). The stack is saved with the world and sent to the client, so the
//   bar and the tooltip read it there. The sky area gets each new reading in a mod message on `bmh:sky`.
// - The points are an attachment on the player (`Holder::Player`), copied across death (GAPS.md A5).
// - The cooldowns, the last change of each player, the world spawn and the queues of awards and sounds are
//   in `SERVER`. The screen that the window shows is in `CLIENT`.
//
// Server and client talk on `bmh:amulet` (`bmh_logic::wire::AmuletSignal` and `AmuletRequest`). The server
// checks every request as the Java command did (decision D7).

use bmh_logic::amulet as logic;
use bmh_logic::contract::{AmuletPoint, AmuletReading};
use bmh_logic::wire::{AMULET_CHANNEL, AmuletRequest, AmuletSignal, SKY_CHANNEL, SkySignal};
use horizon_client::items::ItemStack;
use horizon_client::mod_api::{AttachmentCodec, AttachmentError, AttachmentType, Compound, Holder, Tag};
use horizon_client::text::{self, Text};
use horizon_world::nbt::JavaString;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

const OVERWORLD: &str = bmh_logic::respawn::OVERWORLD;
/// The off hand, by `native_inventory_tick`'s numbers.
const OFF_HAND_SLOT: i32 = -106;
/// The component that holds the stack data of the mod.
const CUSTOM_DATA: &str = "minecraft:custom_data";
/// The flag in the saved points: the points of a Java world were brought in.
const IMPORTED_TAG: &str = "imported";

/// The lock of a static. A panic in another hook leaves the value usable.
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------------------------
// Server state
// ---------------------------------------------------------------------------------------------

/// What the server side keeps between hooks.
struct Server {
    /// The cooldown of the amulet of each player (GAPS.md A4: the mod's own timer).
    cooldowns: BTreeMap<u128, logic::Cooldown>,
    /// The game time of the last use on a block that the amulet took, for the use of the same click.
    took_click: BTreeMap<u128, i64>,
    /// The game time of the last point change (AmuletPointCommand.java:41).
    last_change: BTreeMap<u128, i64>,
    /// The world spawn that `spawn_change` told, with its world folder.
    world_spawn: Option<(PathBuf, [i32; 3])>,
    /// Advancements to award where no player is swapped in: player, name, path.
    awards: Vec<(u128, String, &'static str)>,
    /// Chimes for the other players: player, dimension, position, console line.
    sounds: Vec<(u128, String, [f64; 3], String)>,
}

static SERVER: Mutex<Server> = Mutex::new(Server {
    cooldowns: BTreeMap::new(),
    took_click: BTreeMap::new(),
    last_change: BTreeMap::new(),
    world_spawn: None,
    awards: Vec::new(),
    sounds: Vec::new(),
});

fn server() -> MutexGuard<'static, Server> {
    lock(&SERVER)
}

/// The game time of the server (`Level.getGameTime`, shared by every level).
fn game_time(game: &crate::game::Game<'_>) -> Option<i64> {
    let level = game.native_level(OVERWORLD)?;
    game.native_level_game_time(level)
}

/// Starts the amulet's cooldown for a player (`ItemCooldowns.addCooldown`). The server keeps the timer; the
/// player's window gets the length and draws the sweep on the icon (GAPS.md A4).
fn start_cooldown(game: &mut crate::game::Game<'_>, player: u128, now: i64, ticks: i64) {
    server().cooldowns.insert(player, logic::Cooldown { start: now, ticks });
    let ticks = i32::try_from(ticks).unwrap_or(i32::MAX);
    signal(game, player, &AmuletSignal::Cooldown { ticks });
}

/// A line over the player's hotbar (`sendOverlayMessage`).
fn tell(game: &mut crate::game::Game<'_>, player: u128, key: &str, args: &[String]) {
    let args: Vec<Text> = args.iter().map(|a| Text::literal(a.clone())).collect();
    game.native_action_bar(player, Text::translate(key, &args));
}

/// Sends a screen signal to the player's client. A client without the mod gets nothing.
fn signal(game: &mut crate::game::Game<'_>, player: u128, message: &AmuletSignal) {
    if let Err(error) = game.native_net_send_to_player(player, AMULET_CHANNEL, &message.encode()) {
        log::warn!("[BalancedMcHardcoreMain] Amulet screen not sent: {error}");
    }
}

/// The chime at a player: they hear it at once, the others near them at the next level tick
/// (`level.playSound(null, ...)` in Java reaches both).
fn chime(game: &mut crate::game::Game<'_>, player: u128, name: &str, dimension: &str, at: [f64; 3], pitch: f32) {
    game.native_play_sound_to(
        player,
        logic::CHIME_SOUND,
        logic::CHIME_CATEGORY,
        Some(at),
        logic::CHIME_VOLUME,
        pitch,
    );
    if let Some(line) = logic::chime_command(name, at, pitch) {
        server().sounds.push((player, dimension.to_owned(), at, line));
    }
}

/// Queues an advancement: `run_command` works only where no player is swapped in.
fn queue_award(player: u128, name: &str, path: &'static str) {
    server().awards.push((player, name.to_owned(), path));
}

/// The cap of points (mod config, AmuletPoints.java:72-75).
fn max_points(game: &mut crate::game::Game<'_>) -> i32 {
    crate::game::bmh_settings::config(game).max_amulet_points
}

// ---------------------------------------------------------------------------------------------
// The points on the player (A5)
// ---------------------------------------------------------------------------------------------

/// What the player keeps: the named points, and whether the points of a Java world were brought in. The
/// flag is saved with the points, so a restart between the first change and the import loses nothing.
#[derive(Clone)]
struct Saved {
    points: Vec<AmuletPoint>,
    imported: bool,
}

fn no_points() -> Saved {
    Saved { points: Vec::new(), imported: false }
}

/// The points of a player, saved with the player and copied across death (GAPS.md A5). The saved form of
/// the list is the Java one: a list of compounds with `name`, `x`, `y` and `z` (AmuletPoints.java:45-56).
static POINTS: AttachmentType<Saved> = AttachmentType {
    namespace: "bmh",
    name: "amulet_points",
    default: no_points,
    codec: Some(AttachmentCodec { encode: encode_points, decode: decode_points }),
    copy_on_death: true,
    sync: false,
};

fn key(name: &str) -> JavaString {
    JavaString::from(name)
}

fn encode_points(saved: &Saved) -> Result<Compound, AttachmentError> {
    let list = saved.points.iter().map(point_tag).collect();
    Ok([
        (key(logic::POINTS_TAG), Tag::List(list)),
        (key(IMPORTED_TAG), Tag::Byte(i8::from(saved.imported))),
    ]
    .into())
}

fn point_tag(point: &AmuletPoint) -> Tag {
    let [x, y, z] = point.pos;
    Tag::compound([
        ("name", Tag::String(key(&point.name))),
        ("x", Tag::Int(x)),
        ("y", Tag::Int(y)),
        ("z", Tag::Int(z)),
    ])
}

fn decode_points(data: &Compound) -> Result<Saved, AttachmentError> {
    // A missing flag reads as "not imported": the merge of the import never doubles a point.
    let imported = matches!(data.get(&key(IMPORTED_TAG)), Some(Tag::Byte(flag)) if *flag != 0);
    match data.get(&key(logic::POINTS_TAG)) {
        Some(Tag::List(list)) => Ok(Saved { points: points_of_list(list), imported }),
        _ => Err(AttachmentError::Decode),
    }
}

/// The points of a saved list. A missing field reads as "" or 0, as the Java codec does
/// (AmuletPoints.java:45-56); an entry that is not a compound is skipped.
fn points_of_list(list: &[Tag]) -> Vec<AmuletPoint> {
    list.iter()
        .filter(|entry| matches!(entry, Tag::Compound(_)))
        .map(|entry| AmuletPoint {
            name: match entry.get("name") {
                Some(Tag::String(name)) => name.to_string_lossy(),
                _ => String::new(),
            },
            pos: position_of(entry),
        })
        .collect()
}

/// The `x`, `y`, `z` fields of a compound, 0 when one is missing.
fn position_of(tag: &Tag) -> [i32; 3] {
    ["x", "y", "z"].map(|axis| tag.get(axis).and_then(number).map_or(0, |v| v as i32))
}

/// A numeric tag as an integer (Java `NumericTag`).
fn number(tag: &Tag) -> Option<i64> {
    match tag {
        Tag::Byte(v) => Some(i64::from(*v)),
        Tag::Short(v) => Some(i64::from(*v)),
        Tag::Int(v) => Some(i64::from(*v)),
        Tag::Long(v) => Some(*v),
        Tag::Float(bits) => Some(f32::from_bits(*bits) as i64),
        Tag::Double(bits) => Some(f64::from_bits(*bits) as i64),
        _ => None,
    }
}

/// The points of a player in the data of a Java world: `bmh_points`, else the Home of 1.5.0 to 1.6.x
/// (AmuletPoints.java:203-215). `data` is the compound that Horizon loaded for the player, with the keys
/// that it does not model (`player_extra`).
fn java_points(data: &Tag) -> Vec<AmuletPoint> {
    match (data.get(logic::POINTS_TAG), data.get(logic::LEGACY_HOME_TAG)) {
        (Some(Tag::List(list)), _) => points_of_list(list),
        (_, Some(home @ Tag::Compound(_))) => {
            vec![AmuletPoint { name: logic::LEGACY_HOME_NAME.to_owned(), pos: position_of(home) }]
        }
        _ => Vec::new(),
    }
}

/// The saved points of a player: None when the player has none yet, Err when the player is out of reach.
fn stored(game: &mut crate::game::Game<'_>, player: u128) -> Result<Option<Saved>, AttachmentError> {
    let host: &mut dyn horizon_client::mod_api::Host = game;
    POINTS.get_existing(host, Holder::Player(player))
}

/// The points of a player (Java `AmuletPoints.get`). A player with no saved list has none: the points of
/// a Java world come in at `import_java_points`, never from a player's own turn, where Horizon gives no
/// record of the player. None when the player cannot be reached.
fn load_points(game: &mut crate::game::Game<'_>, player: u128) -> Option<Vec<AmuletPoint>> {
    match stored(game, player) {
        Ok(saved) => Some(saved.map_or_else(Vec::new, |saved| saved.points)),
        Err(error) => {
            log::debug!("[BalancedMcHardcoreMain] The amulet points of {player:032x} are out of reach: {error:?}");
            None
        }
    }
}

/// Keeps the points of a player. The "imported" flag stays as it is; a player with no saved list starts
/// with the flag off, so the import still brings the points of a Java world and merges them.
fn save_points(game: &mut crate::game::Game<'_>, player: u128, points: Vec<AmuletPoint>) -> bool {
    let imported = matches!(stored(game, player), Ok(Some(saved)) if saved.imported);
    set_saved(game, player, Saved { points, imported })
}

fn set_saved(game: &mut crate::game::Game<'_>, player: u128, saved: Saved) -> bool {
    let host: &mut dyn horizon_client::mod_api::Host = game;
    match POINTS.set(host, Holder::Player(player), saved) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("[BalancedMcHardcoreMain] The amulet points are not saved: {error:?}");
            false
        }
    }
}

/// Brings the points of a Java world to the players who are on, once for each player (the saved flag ends it, so a restart between the first change and the
/// import loses nothing). It runs at
/// `level_tick_post`, where no player is swapped in and `native_player_compound` gives the record that
/// Horizon loaded (the file that level.dat names for the single-player owner, `players/data/<uuid>.dat`
/// for a guest). A player with no record yet, or out of reach, is tried again at the next call. The
/// import ends for a player when the flag is saved, even when the Java data holds no point, so a new
/// world costs one pass for each player. Points that the player made before the import stay after the
/// Java points (`logic::merge_imported`).
fn import_java_points(game: &mut crate::game::Game<'_>) {
    let online: Vec<u128> = game.native_server_players().iter().map(|p| p.uuid).collect();
    for player in online {
        let saved = match stored(game, player) {
            Ok(saved) => saved,
            Err(_) => continue,
        };
        if saved.as_ref().is_some_and(|saved| saved.imported) {
            continue;
        }
        let Some(record) = game.native_player_compound(player) else {
            continue;
        };
        let java = java_points(&record);
        if !java.is_empty() {
            log::info!("[BalancedMcHardcoreMain] {} amulet points read from the Java player data", java.len());
        }
        let made = saved.map_or_else(Vec::new, |saved| saved.points);
        let points = logic::merge_imported(&java, &made);
        set_saved(game, player, Saved { points, imported: true });
    }
}

// Contract (docs/horizon/HOOKS.md, "Contracts"): the named points that this player has now, in the order
// of the list (Java `AmuletPoints.get`). The homecoming area reads them. Empty for a player out of reach.
pub(super) fn points(game: &mut crate::game::Game<'_>, player: u128) -> Vec<AmuletPoint> {
    load_points(game, player).unwrap_or_default()
}

/// The world spawn: the one `spawn_change` told for this world, else the one of `level.dat` (GAPS.md A2,
/// the substitute of the X helper). None when neither is known.
fn world_spawn(game: &crate::game::Game<'_>) -> Option<[i32; 3]> {
    let dir = game.native_world_dir()?;
    let known = server().world_spawn.clone().filter(|(world, _)| *world == dir).map(|(_, pos)| pos);
    known.or_else(|| horizon_client::save::Level::read(&dir).and_then(|level| level.world_spawn))
}

/// What the area reads of a block that may hold a point.
struct Block {
    can_hold: bool,
    bed: bool,
    half: Option<logic::BedHalf>,
}

/// The block at `pos` in `dimension` (AmuletPoints.java:100-113). None where the chunk is not loaded.
fn block_at(game: &crate::game::Game<'_>, dimension: &str, pos: [i32; 3]) -> Option<Block> {
    let level = game.native_level(dimension)?;
    let name = game.native_level_block(level, pos)?;
    let bed = game
        .native_block_name_in_tag(&name, logic::BEDS_TAG)
        .unwrap_or_else(|_| logic::is_vanilla_bed(&name));
    let half = if bed {
        let part = game.native_level_block_property(level, pos, "part");
        let facing = game.native_level_block_property(level, pos, "facing");
        logic::bed_half(part.as_deref(), facing.as_deref())
    } else {
        None
    };
    Some(Block { can_hold: bed || name == logic::LODESTONE, bed, half })
}

/// The player who asks for a change, as the checks read them.
struct Who {
    uuid: u128,
    name: String,
    dimension: String,
    asker: logic::Asker,
}

fn is_amulet(stack: Option<&ItemStack>) -> bool {
    stack.is_some_and(|s| s.count > 0 && &*s.item == logic::AMULET)
}

/// The player whose state is swapped in (a use or a command).
fn acting_who(game: &mut crate::game::Game<'_>) -> Option<Who> {
    let me = game.native_acting_player()?;
    let main = is_amulet(game.native_carried_stack(i32::from(me.selected_slot)).map(|s| &*s));
    let off = is_amulet(game.native_carried_stack(OFF_HAND_SLOT).map(|s| &*s));
    Some(who_of(&me, main || off))
}

/// A player of the server with no state swapped in (a mod message): the hands through the living handle.
fn remote_who(game: &crate::game::Game<'_>, player: u128) -> Option<Who> {
    use horizon_client::first_person::Hand;
    let me = game.native_server_players().into_iter().find(|p| p.uuid == player)?;
    let living = u32::try_from(me.entity_id)
        .ok()
        .and_then(|id| game.native_entity(id))
        .and_then(|entity| game.native_living(entity));
    let holds = living.is_some_and(|living| {
        [Hand::Main, Hand::Off]
            .into_iter()
            .any(|hand| is_amulet(game.native_living_hand_item(living, hand).as_ref()))
    });
    Some(who_of(&me, holds))
}

fn who_of(me: &horizon_client::mod_api::NativeServerPlayer, holds_amulet: bool) -> Who {
    Who {
        uuid: me.uuid,
        name: me.name.clone(),
        dimension: me.dimension.clone(),
        asker: logic::Asker {
            holds_amulet,
            overworld: me.dimension == OVERWORLD,
            alive: me.health > 0.0,
            spectator: me.game_mode == 3,
            position: me.position,
        },
    }
}

/// Sends the list of the points to the player (AmuletPointSignal.java:30-48).
fn send_list(game: &mut crate::game::Game<'_>, player: u128) {
    let points = load_points(game, player).unwrap_or_default();
    let max = max_points(game);
    let list = points.into_iter().map(|p| (p.name, p.pos)).collect();
    signal(game, player, &AmuletSignal::List { max, points: list });
}

/// Carries out a change of a point after the checks of the Java command (AmuletPointCommand.java:62-151).
fn change(game: &mut crate::game::Game<'_>, who: &Who, change: logic::Change) {
    let Some(now) = game_time(game) else {
        return;
    };
    let last = server().last_change.get(&who.uuid).copied();
    match change {
        logic::Change::Save { pos, name } => {
            let block = block_at(game, &who.dimension, pos);
            let can_hold = block.as_ref().is_some_and(|b| b.can_hold);
            if let Err(refusal) = logic::check_change(&who.asker, pos, can_hold, last, now) {
                if let Some(key) = refusal.key() {
                    tell(game, who.uuid, key, &[]);
                }
                return;
            }
            let clean = logic::clean_name(&name);
            let half = block.and_then(|b| b.half);
            let Some(mut points) = load_points(game, who.uuid) else {
                return;
            };
            let renamed = logic::point_at(&points, pos, half).is_some();
            let max = max_points(game);
            if let Err(stop) = logic::check_save(&clean, renamed, points.len(), max) {
                let args = if stop == logic::SaveStop::Full { vec![max.to_string()] } else { Vec::new() };
                tell(game, who.uuid, stop.key(), &args);
                return;
            }
            let renamed = logic::put(&mut points, pos, half, &clean);
            let count = points.len();
            if !save_points(game, who.uuid, points) {
                return;
            }
            for path in logic::awards_after_put(count) {
                queue_award(who.uuid, &who.name, path);
            }
            server().last_change.insert(who.uuid, now);
            chime(game, who.uuid, &who.name, &who.dimension, who.asker.position, logic::SAVE_PITCH);
            let key = if renamed { logic::KEY_RENAMED } else { logic::KEY_SAVED };
            tell(game, who.uuid, key, &[clean]);
        }
        logic::Change::Remove { pos } => {
            let block = block_at(game, &who.dimension, pos);
            let can_hold = block.as_ref().is_some_and(|b| b.can_hold);
            if let Err(refusal) = logic::check_change(&who.asker, pos, can_hold, last, now) {
                if let Some(key) = refusal.key() {
                    tell(game, who.uuid, key, &[]);
                }
                return;
            }
            let Some(mut points) = load_points(game, who.uuid) else {
                return;
            };
            let Some(removed) = logic::remove_on_block(&mut points, pos, block.and_then(|b| b.half)) else {
                return;
            };
            if !save_points(game, who.uuid, points) {
                return;
            }
            server().last_change.insert(who.uuid, now);
            tell(game, who.uuid, logic::KEY_REMOVED, &[removed.name]);
        }
        logic::Change::Delete { pos } => {
            if !who.asker.holds_amulet {
                tell(game, who.uuid, logic::KEY_NO_AMULET, &[]);
                return;
            }
            let Some(mut points) = load_points(game, who.uuid) else {
                return;
            };
            let Some(removed) = logic::remove_exact(&mut points, pos) else {
                return;
            };
            if !save_points(game, who.uuid, points) {
                return;
            }
            tell(game, who.uuid, logic::KEY_REMOVED, &[removed.name]);
            send_list(game, who.uuid);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The stack data (A1, A2)
// ---------------------------------------------------------------------------------------------

/// A field of the stack's custom data.
fn custom_field<'a>(stack: &'a ItemStack, name: &str) -> Option<&'a Tag> {
    stack.custom_data()?.get(&key(name))
}

/// The charges that the stack's custom data holds (AmuletReading.java:148-157).
fn stack_charges(stack: &ItemStack) -> logic::StackCharges {
    logic::StackCharges {
        charges: custom_field(stack, logic::CHARGES_TAG).and_then(number).map(|v| v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32),
        night: custom_field(stack, logic::NIGHT_TAG).and_then(number),
    }
}

/// Changes the stack's custom data with `edit`, and the component goes when it is empty (Java
/// `CustomData.update`).
fn edit_custom_data(stack: &mut ItemStack, edit: impl FnOnce(&mut Compound)) {
    let mut data = stack.custom_data().cloned().unwrap_or_default();
    edit(&mut data);
    let value = (!data.is_empty()).then_some(Tag::Compound(data));
    stack.data_mut().set_other(CUSTOM_DATA, value);
    stack.tidy();
}

/// Writes the charges on the stack.
fn write_charges(stack: &mut ItemStack, charges: logic::StackCharges) {
    edit_custom_data(stack, |data| {
        if let Some(count) = charges.charges {
            data.insert(key(logic::CHARGES_TAG), Tag::Int(count));
        }
        if let Some(night) = charges.night {
            data.insert(key(logic::NIGHT_TAG), Tag::Long(night));
        }
    });
}

/// Writes a reading on the stack (AmuletReading.java:232-245).
fn write_reading(stack: &mut ItemStack, reading: &AmuletReading) {
    edit_custom_data(stack, |data| {
        let azimuths = reading.azimuths.iter().map(|a| Tag::Float(a.to_bits())).collect();
        let names = reading.names.iter().map(|n| Tag::String(key(n))).collect();
        let mut compound = Compound::new();
        compound.insert(key(logic::AZIMUTHS_TAG), Tag::List(azimuths));
        compound.insert(key(logic::NAMES_TAG), Tag::List(names));
        if let Some(spawn) = reading.spawn.and_then(|s| i32::try_from(s).ok()) {
            compound.insert(key(logic::SPAWN_TAG), Tag::Int(spawn));
        }
        compound.insert(key(logic::TIME_TAG), Tag::Long(reading.time));
        data.insert(key(logic::READING_TAG), Tag::Compound(compound));
    });
}

/// An amulet carried over from 1.4.0 drops its use stamp, its wear, Unbreaking and Mending
/// (AmuletReading.java:163-182). It changes the stack only when there is something to drop.
fn drop_legacy_data(stack: &mut ItemStack) {
    if custom_field(stack, logic::LEGACY_LAST_USE_TAG).is_some() {
        edit_custom_data(stack, |data| {
            data.remove(&key(logic::LEGACY_LAST_USE_TAG));
        });
    }
    if stack.damage != 0 {
        stack.damage = 0;
    }
    let useless = |id: &str| logic::USELESS_ENCHANTMENTS.contains(&id);
    if stack.enchantments().iter().any(|(id, _)| useless(&**id)) {
        stack.data_mut().enchantments.retain(|(id, _)| !useless(&**id));
        stack.tidy();
    }
}

// Contract (docs/horizon/HOOKS.md, "Contracts"): sends a new star reading to the player's client, for
// the sky area (StarReadingRenderer.java:172-209). The client cannot read the main hand stack, so the
// reading goes in a mod message (`SkySignal::Reading` on `bmh:sky`). A client without the mod gets nothing.
fn send_reading(game: &mut crate::game::Game<'_>, player: u128, reading: &AmuletReading) {
    let message = SkySignal::Reading(reading.clone());
    if let Err(error) = game.native_net_send_to_player(player, SKY_CHANNEL, &message.encode()) {
        log::warn!("[BalancedMcHardcoreMain] Star reading not sent to the sky: {error}");
    }
}

/// The slot of the amulet that a use takes: the main hand first, then the off hand (the client never
/// tries the off hand when the main hand holds the amulet, see `on_interaction_key`).
fn used_slot(game: &mut crate::game::Game<'_>, selected: u8) -> Option<i32> {
    let main = i32::from(selected);
    [main, OFF_HAND_SLOT]
        .into_iter()
        .find(|slot| is_amulet(game.native_carried_stack(*slot).map(|s| &*s)))
}

/// Reads the sky at eye level for an amulet reading or a moon ritual.
pub(super) fn sky_at(
    game: &crate::game::Game<'_>,
    dimension: &str,
    eye_position: [f64; 3],
    blinded: bool,
) -> logic::Sky {
    let eye = logic::block_of(eye_position);
    let wet = logic::block_of([eye_position[0], eye_position[1] - logic::FLUID_EYE_OFFSET, eye_position[2]]);
    let wet_block = game.native_block(wet);
    let waterlogged = game.native_block_property(wet, "waterlogged").as_deref() == Some("true");
    let (rain, thunder) = game.native_weather();
    logic::Sky {
        overworld: dimension == OVERWORLD,
        rain,
        thunder,
        day_time: game.native_day_time(),
        sky_light: game.native_light(eye).1,
        blinded,
        eyes_covered: logic::holds_water(&wet_block, waterlogged)
            || logic::holds_lava(&wet_block)
            || logic::is_powder_snow(&game.native_block(eye)),
    }
}

/// The sky for the player whose state is swapped in (server).
pub(super) fn sky(game: &crate::game::Game<'_>, me: &horizon_client::mod_api::NativeServerPlayer) -> logic::Sky {
    let [x, y, z] = me.position;
    let blinded = game
        .native_effects(me.uuid)
        .is_some_and(|effects| effects.iter().any(|e| logic::BLINDING_EFFECTS.contains(&e.id.as_str())));
    sky_at(game, &me.dimension, [x, y + f64::from(me.eye_height), z], blinded)
}

// ---------------------------------------------------------------------------------------------
// Server hooks
// ---------------------------------------------------------------------------------------------

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, name);
    let mut s = server();
    s.cooldowns.remove(&player);
    s.took_click.remove(&player);
    s.last_change.remove(&player);
    s.awards.retain(|(who, _, _)| *who != player);
    s.sounds.retain(|(who, _, _, _)| *who != player);
}

// A5, A6: check and apply a point save, rename or remove that a screen asks for, on `bmh:amulet`. No
// player is swapped in here: the player is read from the server's list.
pub(super) fn on_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if channel != AMULET_CHANNEL {
        return;
    }
    let Some(request) = AmuletRequest::decode(data) else {
        log::debug!("[BalancedMcHardcoreMain] A bad amulet request was dropped");
        return;
    };
    let Some(who) = remote_who(game, player) else {
        return;
    };
    let wanted = match request {
        AmuletRequest::Save { pos, name } => logic::Change::Save { pos, name },
        AmuletRequest::Remove { pos } => logic::Change::Remove { pos },
        AmuletRequest::Delete { pos } => logic::Change::Delete { pos },
    };
    change(game, &who, wanted);
}

// A5: register `/bmh point` (AmuletPointCommand.java:43-55), at level 0 as in Java. The settings area
// refuses `/bmh rule` below level 2 by itself: see `bmh_logic::amulet::COMMAND_LEVEL`.
pub(super) fn on_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    let _ = game;
    for usage in logic::COMMAND_USAGES {
        commands.push(horizon_client::mod_api::NativeCommand::op(usage, logic::COMMAND_LEVEL));
    }
}

// A5: run `/bmh point`. A line of another area (`bmh rule ...`) is left alone. The player's state is
// swapped in.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if *ran {
        return;
    }
    let parsed = logic::parse_point_command(command, player.position);
    let wanted = match parsed {
        logic::PointCommand::NotMine => return,
        logic::PointCommand::Incomplete => {
            *ran = true;
            game.native_feedback(Text::translate("command.unknown.command", &[]).with_color(text::RED));
            return;
        }
        logic::PointCommand::Bad => {
            *ran = true;
            game.native_feedback(Text::translate("command.unknown.argument", &[]).with_color(text::RED));
            return;
        }
        logic::PointCommand::Change(wanted) => wanted,
    };
    *ran = true;
    // `BlockPosArgument.getLoadedBlockPos`: a save or a remove needs a loaded block of the player's level.
    if let logic::Change::Save { pos, .. } | logic::Change::Remove { pos } = &wanted {
        let level = game.native_level(&player.dimension);
        let inside = level
            .and_then(|l| game.native_level_height(l))
            .is_some_and(|(min_y, height)| pos[1] >= min_y && pos[1] < min_y + height);
        let loaded = level.is_some_and(|l| game.native_level_is_loaded(l, *pos));
        if !inside || !loaded {
            let key = if inside { "argument.pos.unloaded" } else { "argument.pos.outofworld" };
            game.native_feedback(Text::translate(key, &[]).with_color(text::RED));
            return;
        }
    }
    let Some(who) = acting_who(game) else {
        return;
    };
    change(game, &who, wanted);
}

// A2: refill the charges once a second, and drop the data of a 1.4.0 amulet (NavigatorAmuletItem.java:
// 140-147). The eternal area may set a count of 0 first: such a stack is skipped.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = selected;
    if !is_amulet(game.native_carried_stack(slot).map(|s| &*s)) {
        return;
    }
    if !game_time(game).is_some_and(logic::refill_tick) {
        return;
    }
    let day = game.native_day_time();
    let Some(stack) = game.native_carried_stack(slot) else {
        return;
    };
    drop_legacy_data(stack);
    if let Some(full) = logic::refill(stack_charges(stack), day) {
        write_charges(stack, full);
    }
}

// A2: read the stars, or open the list when sneaking (NavigatorAmuletItem.java:92-137). A3: drop the use
// that follows a use on a block of the same click. The player's state is swapped in.
pub(super) fn on_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    if *claimed || !is_amulet(Some(stack)) {
        return;
    }
    let (Some(me), Some(now)) = (game.native_acting_player(), game_time(game)) else {
        return;
    };
    let (on_cooldown, took_click) = {
        let s = server();
        let cooling = s.cooldowns.get(&me.uuid).is_some_and(|c| c.active(now));
        (cooling, logic::same_click(s.took_click.get(&me.uuid).copied(), now))
    };
    let sneaking = game.native_sneaking();
    let Some(slot) = used_slot(game, me.selected_slot) else {
        return;
    };
    let hidden = if on_cooldown || took_click || sneaking {
        None
    } else {
        logic::hidden_reason(&sky(game, &me), logic::HIDDEN_PREFIX)
    };
    let day = game.native_day_time();
    let stored = game.native_carried_stack(slot).map(|s| stack_charges(s)).unwrap_or_default();
    let refilled = logic::refill(stored, day);
    let current = refilled.unwrap_or(stored);
    let creative = me.game_mode == 1;
    let decision =
        logic::use_in_air(on_cooldown, took_click, sneaking, hidden, logic::charges(current), creative);
    match decision {
        logic::Use::Ignore => {}
        logic::Use::List => {
            send_list(game, me.uuid);
            start_cooldown(game, me.uuid, now, logic::POINT_COOLDOWN_TICKS);
            *claimed = true;
        }
        logic::Use::Hidden(key) => tell(game, me.uuid, &key, &[]),
        logic::Use::Empty => {
            if let (Some(full), Some(stack)) = (refilled, game.native_carried_stack(slot)) {
                write_charges(stack, full);
            }
            tell(game, me.uuid, logic::KEY_EMPTY, &[]);
        }
        logic::Use::Read => {
            let points = load_points(game, me.uuid).unwrap_or_default();
            let read = logic::reading(me.position, &points, world_spawn(game), now);
            let after = logic::charges_after_reading(current, creative, day);
            let written = if let Some(stack) = game.native_carried_stack(slot) {
                if let Some(full) = refilled {
                    write_charges(stack, full);
                }
                write_reading(stack, &read);
                if let Some(after) = after {
                    write_charges(stack, after);
                }
                true
            } else {
                false
            };
            if written {
                send_reading(game, me.uuid, &read);
            }
            queue_award(me.uuid, &me.name, logic::STARGAZER);
            chime(game, me.uuid, &me.name, &me.dimension, me.position, logic::READING_PITCH);
            let [x, y, z] = logic::block_of(me.position).map(|v| v.to_string());
            tell(game, me.uuid, logic::KEY_READING, &[x, y, z]);
            start_cooldown(game, me.uuid, now, logic::READING_COOLDOWN_TICKS);
            *claimed = true;
            log::info!(
                "[BalancedMcHardcoreMain] Star reading for {}: azimuths {:?}, {} charges left",
                me.name,
                read.azimuths,
                after.map_or(logic::charges(current), logic::charges)
            );
        }
    }
}

// A3, A5: sneak and use on a bed or a lodestone opens the naming screen of its point
// (NavigatorAmuletItem.java:60-88). The player's state is swapped in.
pub(super) fn on_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let _ = face;
    if *claimed || !is_amulet(Some(stack)) {
        return;
    }
    let (Some(me), Some(now)) = (game.native_acting_player(), game_time(game)) else {
        return;
    };
    let on_cooldown = server().cooldowns.get(&me.uuid).is_some_and(|c| c.active(now));
    let sneaking = game.native_sneaking();
    let block = block_at(game, &me.dimension, pos);
    let can_hold = block.as_ref().is_some_and(|b| b.can_hold);
    match logic::use_on(on_cooldown, sneaking, can_hold, me.dimension == OVERWORLD) {
        logic::UseOn::Pass => {}
        logic::UseOn::Open => {
            let points = load_points(game, me.uuid).unwrap_or_default();
            let half = block.as_ref().and_then(|b| b.half);
            let name = logic::point_at(&points, pos, half).and_then(|i| points.get(i)).map(|p| p.name.clone());
            let used = i32::try_from(points.len()).unwrap_or(i32::MAX);
            let max = max_points(game);
            let bed = block.is_some_and(|b| b.bed);
            signal(game, me.uuid, &AmuletSignal::Open { pos, name, used, max, bed });
            start_cooldown(game, me.uuid, now, logic::POINT_COOLDOWN_TICKS);
            server().took_click.insert(me.uuid, now);
            *claimed = true;
        }
        logic::UseOn::NotOverworld => {
            tell(game, me.uuid, logic::KEY_OVERWORLD, &[]);
            server().took_click.insert(me.uuid, now);
            *claimed = true;
        }
    }
}

// A3: sneaking is read where it matters, in the use hooks (`native_sneaking`, the key that vanilla's
// `isSecondaryUseActive` reads). This hook has nothing to keep.
pub(super) fn on_player_toggle_sneak(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    sneaking: bool,
) {
    let _ = (game, player, name, sneaking);
}

// A2: `/setworldspawn` moved the world spawn, the last line of a reading.
pub(super) fn on_spawn_change(game: &mut crate::game::Game<'_>, pos: [i32; 3], angle: [f32; 2]) {
    let _ = angle;
    if let Some(world) = game.native_world_dir() {
        server().world_spawn = Some((world, pos));
    }
}

// A2, A5: the import of the Java points, the advancements and the chimes for the other players.
// `run_command` works here: no player is swapped in, and `/playsound` reaches the players of the level in
// place.
pub(super) fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    import_java_points(game);
    let (awards, sounds) = {
        let mut s = server();
        let awards = std::mem::take(&mut s.awards);
        let (here, other): (Vec<_>, Vec<_>) =
            std::mem::take(&mut s.sounds).into_iter().partition(|(_, level, _, _)| level == dimension);
        s.sounds = other;
        (awards, here)
    };
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
    if sounds.is_empty() {
        return;
    }
    let online = game.native_server_players();
    for (player, _, at, line) in sounds {
        let others = online
            .iter()
            .filter(|p| p.uuid != player && p.dimension == dimension)
            .map(|p| p.position);
        // With nobody near, `/playsound` fails by design: skip it.
        if !bmh_logic::respawn::someone_hears(logic::block_of(at), others) {
            continue;
        }
        if !game.native_run_command(&line) {
            log::debug!("[BalancedMcHardcoreMain] Chime command failed: {line}");
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Client state and hooks (A1, A3, A6)
// ---------------------------------------------------------------------------------------------

/// The naming screen (PointScreen.java).
#[derive(Debug, Clone)]
struct Naming {
    pos: [i32; 3],
    existing: Option<String>,
    used: i32,
    max: i32,
    bed: bool,
    field: logic::TextField,
    opened: std::time::Instant,
}

/// The list screen (PointListScreen.java).
#[derive(Debug, Clone)]
struct List {
    points: Vec<(String, [i32; 3])>,
    max: i32,
    page: usize,
}

#[derive(Debug, Clone)]
enum Shown {
    Naming(Naming),
    List(List),
}

/// What the window keeps: the screen it shows, the requests to send and whether to close the screen.
/// The screen hooks get `&Game`, so sending and closing wait for `on_frame`.
struct Client {
    shown: Option<Shown>,
    outbox: Vec<AmuletRequest>,
    close: bool,
    /// Each window draws its own cooldown in the world where the server sent it.
    cooldowns: BTreeMap<u64, (crate::game::native_client_services::NativeWorldId, logic::Cooldown)>,
}

static CLIENT: Mutex<Client> = Mutex::new(Client {
    shown: None,
    outbox: Vec::new(),
    close: false,
    cooldowns: BTreeMap::new(),
});

fn client() -> MutexGuard<'static, Client> {
    lock(&CLIENT)
}

impl Client {
    fn cooldown_fraction(&mut self, game: &crate::game::Game<'_>) -> f32 {
        if self.cooldowns.is_empty() {
            return 0.0;
        }
        let Some(state) = game.native_client_state() else { return 0.0 };
        let Some((world, cooldown)) = self.cooldowns.get(&state.window_id) else { return 0.0 };
        if state.world.as_ref() == Some(world)
            && let Some(clock) = game.native_clock()
        {
            let left = cooldown.remaining(clock.game_time, 0.0);
            if left > 0.0 {
                return left;
            }
        }
        self.cooldowns.remove(&state.window_id);
        0.0
    }
}

/// True when the amulet's own screen is open in this window.
fn screen_open(game: &crate::game::Game<'_>) -> bool {
    game.native_mod_screen() == Some(logic::SCREEN_ID)
}

/// Opens the amulet's screen, or keeps it when it is open: false when another screen is up.
fn open_screen(game: &mut crate::game::Game<'_>) -> bool {
    screen_open(game) || game.native_mod_screen_open(logic::SCREEN_ID)
}

// A6: open the naming screen or the point list that the server sends on `bmh:amulet`
// (PointSignal.java:23-88). A naming screen in use is kept; an open list takes the new list in place.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != AMULET_CHANNEL || !game.native_window() {
        return;
    }
    let Some(message) = AmuletSignal::decode(data) else {
        log::debug!("[BalancedMcHardcoreMain] A bad amulet signal was dropped");
        return;
    };
    if let AmuletSignal::Cooldown { ticks } = message {
        if let (Some(state), Some(clock)) = (game.native_client_state(), game.native_clock())
            && let Some(world) = state.world
        {
            let cooldown = logic::Cooldown { start: clock.game_time, ticks: i64::from(ticks) };
            client().cooldowns.insert(state.window_id, (world, cooldown));
        }
        return;
    }
    let open = screen_open(game);
    let shown = client().shown.clone();
    let naming_open = open && matches!(shown, Some(Shown::Naming(_)));
    match message {
        AmuletSignal::Open { pos, name, used, max, bed } => {
            if naming_open || !open_screen(game) {
                return;
            }
            let field = logic::TextField::with_text(name.as_deref().unwrap_or(""));
            let naming = Naming { pos, existing: name, used, max, bed, field, opened: std::time::Instant::now() };
            client().shown = Some(Shown::Naming(naming));
        }
        AmuletSignal::Cooldown { .. } => {}
        AmuletSignal::List { max, points } => {
            if naming_open {
                return;
            }
            let page = match (open, shown) {
                (true, Some(Shown::List(list))) => list.page.min(logic::last_page(points.len())),
                _ => 0,
            };
            if !open_screen(game) {
                return;
            }
            client().shown = Some(Shown::List(List { points, max, page }));
        }
    }
}

// A6: send what the screens asked for and close the screen they closed. A screen that the game closed
// (Escape) leaves no state behind.
pub(super) fn on_frame(game: &mut crate::game::Game<'_>) {
    if !game.native_window() {
        return;
    }
    let open = screen_open(game);
    let (requests, close) = {
        let mut c = client();
        // Remove the timer when this window leaves the world, even with no amulet on screen.
        c.cooldown_fraction(game);
        let close = std::mem::take(&mut c.close);
        if !open || close {
            c.shown = None;
        }
        (std::mem::take(&mut c.outbox), close)
    };
    for request in requests {
        if let Err(error) = game.native_net_send_to_server(AMULET_CHANNEL, &request.encode()) {
            log::warn!("[BalancedMcHardcoreMain] Amulet request not sent: {error}");
        }
    }
    if close && open {
        game.native_mod_screen_close(logic::SCREEN_ID);
    }
}

/// The state of the screen when the hook's screen is the amulet's.
fn shown_on(game: &crate::game::Game<'_>, screen: &horizon_client::mod_api::NativeScreenView<'_>) -> Option<Shown> {
    if screen.id != "mod" || !screen_open(game) {
        return None;
    }
    client().shown.clone()
}

// A6: clicks on the naming screen and the point list.
pub(super) fn on_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = modifiers;
    if *canceled {
        return;
    }
    let Some(shown) = shown_on(game, screen) else {
        return;
    };
    *canceled = true;
    if button != 0 {
        return;
    }
    let (width, height) = logic::window_size(screen.rect);
    match shown {
        Shown::Naming(naming) => {
            let layout = logic::naming_layout(width, height, naming.existing.is_some());
            let text = naming.field.text();
            let active = logic::save_active(naming.existing.is_some(), naming.used, naming.max, &text);
            let hit = layout.buttons.iter().find(|(_, rect)| logic::inside(*rect, screen.mouse)).map(|b| b.0);
            let mut c = client();
            match hit {
                Some(logic::NamingButton::Save) if active => {
                    // PointScreen.java:96-105: the clean name goes to the server, then the screen closes.
                    c.outbox.push(AmuletRequest::Save { pos: naming.pos, name: logic::clean_name(&text) });
                    c.close = true;
                }
                Some(logic::NamingButton::Remove) => {
                    c.outbox.push(AmuletRequest::Remove { pos: naming.pos });
                    c.close = true;
                }
                Some(logic::NamingButton::Cancel) => c.close = true,
                Some(logic::NamingButton::Save) => {}
                None if logic::inside(layout.field, screen.mouse) => {
                    drop(c);
                    let widths: Vec<f32> =
                        (0..=naming.field.len()).map(|i| game.native_text_width(&naming.field.prefix(i))).collect();
                    let x = screen.mouse[0] - (layout.field[0] + 4) as f32;
                    if let Some(Shown::Naming(open)) = client().shown.as_mut() {
                        open.field.click(&widths, x);
                    }
                }
                None => {}
            }
        }
        Shown::List(list) => {
            let layout = logic::list_layout(width, height, list.points.len(), list.page);
            let hit = layout
                .buttons
                .iter()
                .find(|(_, rect, active)| *active && logic::inside(*rect, screen.mouse))
                .map(|b| b.0);
            let mut c = client();
            match hit {
                Some(logic::ListButton::Remove(index)) => {
                    if let Some((_, pos)) = list.points.get(index) {
                        c.outbox.push(AmuletRequest::Delete { pos: *pos });
                    }
                }
                Some(logic::ListButton::Previous) | Some(logic::ListButton::Next) => {
                    let by = if hit == Some(logic::ListButton::Previous) { -1 } else { 1 };
                    if let Some(Shown::List(open)) = c.shown.as_mut() {
                        open.page = logic::turn_page(open.page, by, open.points.len());
                    }
                }
                Some(logic::ListButton::Done) => c.close = true,
                None => {}
            }
        }
    }
}

// A6: the keys of the naming field. Escape stays the game's: it closes the screen. Every other key stops
// here, as a vanilla screen with a focused text box takes it (the inventory key types a letter).
pub(super) fn on_screen_key_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (repeat, modifiers);
    if *canceled || logic::is_escape(key) {
        return;
    }
    let Some(shown) = shown_on(game, screen) else {
        return;
    };
    *canceled = true;
    let Shown::Naming(naming) = shown else {
        return;
    };
    let mut c = client();
    if logic::is_confirmation(key) {
        let text = naming.field.text();
        if logic::save_active(naming.existing.is_some(), naming.used, naming.max, &text) {
            c.outbox.push(AmuletRequest::Save { pos: naming.pos, name: logic::clean_name(&text) });
            c.close = true;
        }
        return;
    }
    if let (Some(field_key), Some(Shown::Naming(open))) = (logic::FieldKey::of(key), c.shown.as_mut()) {
        open.field.key(field_key);
    }
}

// A6: the characters typed in the naming field.
pub(super) fn on_screen_char_typed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    c: char,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = modifiers;
    if *canceled || shown_on(game, screen).is_none() {
        return;
    }
    *canceled = true;
    if let Some(Shown::Naming(open)) = client().shown.as_mut() {
        open.field.type_char(c);
    }
}

/// A line of text, its colour opaque.
fn draw_text(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, line: &str, at: [i32; 2], argb: u32) {
    game.native_text(batch, line, [at[0] as f32, at[1] as f32], argb & 0xFF_FFFF, true);
}

/// A line of text centred on `center_x` (vanilla `centeredText`).
fn draw_centered(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, line: &str, center_x: i32, y: i32, argb: u32) {
    let x = logic::centered_x(center_x, game.native_text_width(line));
    draw_text(game, batch, line, [x, y], argb);
}

fn translated(key: &str, args: &[String]) -> String {
    let args: Vec<Text> = args.iter().map(|a| Text::literal(a.clone())).collect();
    Text::translate(key, &args).plain()
}

/// A vanilla button: its sprite and its centred label (`AbstractButton`).
fn draw_button(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    rect: logic::Rect,
    label: &str,
    active: bool,
    mouse: [f32; 2],
) {
    let sprite = if !active {
        "gui/sprites/widget/button_disabled"
    } else if logic::inside(rect, mouse) {
        "gui/sprites/widget/button_highlighted"
    } else {
        "gui/sprites/widget/button"
    };
    let area = rect.map(|v| v as f32);
    // The fill shows if the sprite is missing; the sprite covers it otherwise.
    game.native_fill(batch, area, 0xFF30_3030);
    game.native_gui_nine_slice(batch, sprite, area, 0xFFFF_FFFF);
    let color = if active { logic::BUTTON_TEXT } else { logic::BUTTON_TEXT_INACTIVE };
    draw_centered(game, batch, label, rect[0] + rect[2] / 2, rect[1] + (rect[3] - 8) / 2, color);
}

// A6: draw the naming screen and the point list (PointScreen.java:136-159, PointListScreen.java:107-146).
pub(super) fn on_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    let Some(shown) = shown_on(game, screen) else {
        return;
    };
    let (width, height) = logic::window_size(screen.rect);
    game.native_fill(batch, screen.rect, logic::DIM_COLOR);
    match shown {
        Shown::Naming(naming) => draw_naming(game, batch, screen.mouse, width, height, &naming),
        Shown::List(list) => draw_list(game, batch, screen.mouse, width, height, &list),
    }
}

fn draw_naming(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    mouse: [f32; 2],
    width: i32,
    height: i32,
    naming: &Naming,
) {
    let editing = naming.existing.is_some();
    let layout = logic::naming_layout(width, height, editing);
    let title = if editing { logic::KEY_SCREEN_EDIT } else { logic::KEY_SCREEN_NEW };
    draw_centered(game, batch, &translated(title, &[]), layout.center_x, layout.title_y, logic::TEXT_COLOR);
    let subtitle = if naming.bed { logic::KEY_SCREEN_BED } else { logic::KEY_SCREEN_LODESTONE };
    draw_centered(game, batch, &translated(subtitle, &[]), layout.center_x, layout.subtitle_y, logic::NOTE_COLOR);
    // The name field (vanilla `EditBox`, focused).
    let field = layout.field;
    game.native_fill(batch, field.map(|v| v as f32), 0xFF00_0000);
    game.native_gui_nine_slice(batch, "gui/sprites/widget/text_field_highlighted", field.map(|v| v as f32), 0xFFFF_FFFF);
    let text = naming.field.text();
    let text_x = field[0] + 4;
    let text_y = field[1] + (field[3] - 8) / 2;
    draw_text(game, batch, &text, [text_x, text_y], logic::FIELD_TEXT);
    let blink = naming.opened.elapsed().as_millis() / logic::CARET_BLINK_MS % 2 == 0;
    if blink {
        let caret_x = text_x as f32 + game.native_text_width(&naming.field.before_caret());
        if naming.field.caret() >= naming.field.len() {
            game.native_text(batch, "_", [caret_x, text_y as f32], logic::FIELD_TEXT, true);
        } else {
            game.native_fill(batch, [caret_x, (text_y - 1) as f32, 1.0, 10.0], logic::CARET_COLOR);
        }
    }
    let active = logic::save_active(editing, naming.used, naming.max, &text);
    for (kind, rect) in &layout.buttons {
        let (label, on) = match kind {
            logic::NamingButton::Save => (translated(logic::KEY_SCREEN_SAVE, &[]), active),
            logic::NamingButton::Remove => (translated(logic::KEY_SCREEN_REMOVE, &[]), true),
            logic::NamingButton::Cancel => (translated(logic::KEY_CANCEL, &[]), true),
        };
        draw_button(game, batch, *rect, &label, on, mouse);
    }
    let full = logic::screen_full(editing, naming.used, naming.max);
    let (key, numbers) = logic::count_line(editing, naming.used, naming.max);
    let args: Vec<String> = numbers.iter().map(i32::to_string).collect();
    let color = if full { logic::FULL_COLOR } else { logic::NOTE_COLOR };
    draw_centered(game, batch, &translated(key, &args), layout.center_x, layout.count_y, color);
}

fn draw_list(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    mouse: [f32; 2],
    width: i32,
    height: i32,
    list: &List,
) {
    let layout = logic::list_layout(width, height, list.points.len(), list.page);
    draw_centered(game, batch, &translated(logic::KEY_LIST_TITLE, &[]), layout.center_x, layout.title_y, logic::TEXT_COLOR);
    let count = [list.points.len().to_string(), list.max.to_string()];
    draw_centered(game, batch, &translated(logic::KEY_LIST_COUNT, &count), layout.center_x, layout.count_y, logic::NOTE_COLOR);
    if list.points.is_empty() {
        draw_centered(game, batch, &translated(logic::KEY_LIST_EMPTY, &[]), layout.center_x, layout.empty_y, logic::NOTE_COLOR);
    }
    for (index, y) in &layout.rows {
        let Some((name, pos)) = list.points.get(*index) else {
            continue;
        };
        let fitted = logic::fit_width(name, logic::NAME_WIDTH, |s| game.native_text_width(s));
        draw_text(game, batch, &fitted, [layout.left, *y], logic::TEXT_COLOR);
        let coords = format!("{} {} {}", pos[0], pos[1], pos[2]);
        draw_text(game, batch, &coords, [layout.left + logic::COORDS_X, *y], logic::NOTE_COLOR);
    }
    for (button, rect, active) in &layout.buttons {
        let label = match button {
            logic::ListButton::Remove(_) => translated(logic::KEY_SCREEN_REMOVE, &[]),
            logic::ListButton::Previous => "<".to_owned(),
            logic::ListButton::Next => ">".to_owned(),
            logic::ListButton::Done => translated(logic::KEY_DONE, &[]),
        };
        draw_button(game, batch, *rect, &label, *active, mouse);
    }
    draw_centered(game, batch, &translated(logic::KEY_LIST_SPAWN, &[]), layout.center_x, layout.spawn_y, logic::NOTE_COLOR);
}

// A3: the off hand never acts in the click of an amulet in the main hand. Java's `use` answers CONSUME on
// the client (NavigatorAmuletItem.java:96-100), so vanilla does not try the other hand (a shield, a bucket,
// an ender pearl). The window's hook, asked once for each hand.
pub(super) fn on_interaction_key(
    game: &mut crate::game::Game<'_>,
    button: i32,
    key_mapping: &str,
    hand: horizon_client::first_person::Hand,
    swing_hand: &mut bool,
    canceled: &mut bool,
) {
    let _ = key_mapping;
    if *canceled || button != 1 || hand != horizon_client::first_person::Hand::Off || !game.native_window() {
        return;
    }
    if game.native_held().is_some_and(|(item, count, _)| count > 0 && item == logic::AMULET) {
        *canceled = true;
        *swing_hand = false;
    }
}

// A1: the tooltip lines of the amulet, after its name (NavigatorAmuletItem.java:149-158).
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    let _ = game;
    if &*stack.item != logic::AMULET {
        return;
    }
    let charges = logic::charges(stack_charges(stack));
    let starlight = Text::translate(
        logic::KEY_STARLIGHT,
        &[Text::literal(charges.to_string()), Text::literal(logic::MAX_CHARGES.to_string())],
    )
    .with_color(logic::STARLIGHT_COLOR);
    let hints = [logic::KEY_TOOLTIP_READ, logic::KEY_TOOLTIP_HOME, logic::KEY_TOOLTIP_LIST]
        .map(|key| Text::translate(key, &[]).with_color(text::GRAY));
    let at = lines.len().min(1);
    let added = std::iter::once(starlight).chain(hints).map(|line| (line, u32::MAX));
    lines.splice(at..at, added);
}

// A1: the charge bar, always shown (NavigatorAmuletItem.java:160-177).
pub(super) fn on_item_bar(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    let _ = game;
    if &*stack.item != logic::AMULET {
        return;
    }
    *bar = Some((logic::bar_width(logic::charges(stack_charges(stack))), logic::STARLIGHT_COLOR));
}

// A4: the white cooldown sweep on every icon of the amulet (the game draws it for items with a cooldown of its
// own; this item has none, so the mod draws the same shade after the game's decorations). The timer is the
// one the server told in `AmuletSignal::Cooldown`.
pub(super) fn on_item_decorations(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    x: f32,
    y: f32,
    batch: &mut horizon_client::hud::HudBatch,
) {
    if &*stack.item != logic::AMULET || !game.native_window() {
        return;
    }
    let Some((top, height)) = logic::sweep_rows(client().cooldown_fraction(game)) else {
        return;
    };
    game.native_fill(batch, [x, y + top as f32, 16.0, height as f32], logic::SWEEP_COLOR);
}
