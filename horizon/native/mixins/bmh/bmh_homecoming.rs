// Area homecoming: the edge between the game and `bmh_logic::homecoming`.
// It holds the record of an exile on the player, the check once a second, the title, the bell and the two
// advancements. Every decision is in `bmh_logic::homecoming`; this file reads the game and carries them out.
// Owner: the homecoming agent. Log: docs/horizon/log/homecoming.md.
//
// Where the data lives:
// - The record is an attachment on the player (`Holder::Player`), copied across death (GAPS.md C10). The saved
//   form keeps the Java field names. The flag `imported` ends the import of the record of a Java world.
// - The bells for the other players wait in `SERVER` until `level_tick_post` of their level.
//
// Where each effect runs: `server_tick` has no player swapped in, so `run_command` (the advancements) works
// there, and `native_title` and `native_play_sound_to` reach any player on. `/playsound` reaches only the
// players of the level in place, so the bell for the other players goes out at `level_tick_post`.

use bmh_logic::contract::{Exile, Rule};
use bmh_logic::homecoming as logic;
use bmh_logic::respawn::{BELL_SOUND, BELL_VOLUME, OVERWORLD, block_of};
use horizon_client::mod_api::{AttachmentCodec, AttachmentError, AttachmentType, Compound, Holder, Tag};
use horizon_client::text::Text;
use horizon_world::nbt::JavaString;
use std::sync::{Mutex, MutexGuard};

/// The lock of a static. A panic in another hook leaves the value usable.
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the server side keeps between hooks.
struct Server {
    /// Bells for the other players: player, dimension, position, console line.
    sounds: Vec<(u128, String, [f64; 3], String)>,
}

static SERVER: Mutex<Server> = Mutex::new(Server { sounds: Vec::new() });

fn server() -> MutexGuard<'static, Server> {
    lock(&SERVER)
}

// ---------------------------------------------------------------------------------------------
// The record on the player
// ---------------------------------------------------------------------------------------------

/// What the player keeps: the record of the exile (none when no exile is on), and whether a record of a Java
/// world was looked for. The flag is saved with the record, so a restart loses nothing.
#[derive(Clone)]
struct Saved {
    record: Option<logic::Record>,
    imported: bool,
}

fn no_saved() -> Saved {
    Saved { record: None, imported: false }
}

/// The key of the flag in the saved compound.
const IMPORTED_TAG: &str = "imported";

/// The record of the exile, saved with the player and copied across death (GAPS.md C10). The saved form is the
/// Java one (HomecomingHandler.java:62-76); a player with no exile writes only the flag.
static RECORD: AttachmentType<Saved> = AttachmentType {
    namespace: "bmh",
    name: "homecoming",
    default: no_saved,
    codec: Some(AttachmentCodec { encode: encode_saved, decode: decode_saved }),
    copy_on_death: true,
    sync: false,
};

fn key(name: &str) -> JavaString {
    JavaString::from(name)
}

fn encode_saved(saved: &Saved) -> Result<Compound, AttachmentError> {
    let mut data: Compound = Compound::new();
    data.insert(key(IMPORTED_TAG), Tag::Byte(i8::from(saved.imported)));
    if let Some(record) = &saved.record {
        data.insert(key(logic::FIELD_DIMENSION), Tag::String(key(&record.dimension)));
        data.insert(key(logic::FIELD_ARRIVAL), Tag::Long(record.arrival));
        data.insert(key(logic::FIELD_DISTANCE), Tag::Int(record.distance));
        data.insert(key(logic::FIELD_HOMES), Tag::List(record.homes.iter().map(home_tag).collect()));
    }
    Ok(data)
}

/// One home, every field written (HomecomingHandler.java:62-68: a value equal to the default must stay).
fn home_tag(home: &logic::Home) -> Tag {
    let [x, y, z] = home.pos;
    Tag::compound([
        (logic::FIELD_NAME, Tag::String(key(&home.name))),
        (logic::FIELD_X, Tag::Int(x)),
        (logic::FIELD_Y, Tag::Int(y)),
        (logic::FIELD_Z, Tag::Int(z)),
    ])
}

fn decode_saved(data: &Compound) -> Result<Saved, AttachmentError> {
    // A missing flag reads as "not imported": the import never replaces a record that exists.
    let imported = matches!(data.get(&key(IMPORTED_TAG)), Some(Tag::Byte(flag)) if *flag != 0);
    Ok(Saved { record: record_of(&Tag::Compound(data.clone())), imported })
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

fn int_field(tag: &Tag, name: &str) -> Option<i32> {
    tag.get(name).and_then(number).and_then(|v| i32::try_from(v).ok())
}

fn string_field(tag: &Tag, name: &str) -> Option<String> {
    match tag.get(name) {
        Some(Tag::String(text)) => Some(text.to_string_lossy()),
        _ => None,
    }
}

/// One home of a saved list. Every field is required, as in the Java codec (HomecomingHandler.java:62-68).
fn home_of(tag: &Tag) -> Option<logic::Home> {
    Some(logic::Home {
        name: string_field(tag, logic::FIELD_NAME)?,
        pos: [int_field(tag, logic::FIELD_X)?, int_field(tag, logic::FIELD_Y)?, int_field(tag, logic::FIELD_Z)?],
    })
}

/// The record in a compound, or None. Every field is required, and one bad home voids the record, as the Java
/// codec does (`input.read(TAG, RECORD_CODEC).orElse(null)`, HomecomingHandler.java:70-76 and 223-226).
fn record_of(tag: &Tag) -> Option<logic::Record> {
    let Some(Tag::List(list)) = tag.get(logic::FIELD_HOMES) else {
        return None;
    };
    Some(logic::Record {
        dimension: string_field(tag, logic::FIELD_DIMENSION)?,
        arrival: tag.get(logic::FIELD_ARRIVAL).and_then(number)?,
        distance: int_field(tag, logic::FIELD_DISTANCE)?,
        homes: list.iter().map(home_of).collect::<Option<Vec<_>>>()?,
    })
}

/// The saved state of a player: Err when the player is out of reach, else the default for a player who has
/// none yet.
fn stored(game: &mut crate::game::Game<'_>, player: u128) -> Result<Saved, AttachmentError> {
    let host: &mut dyn horizon_client::mod_api::Host = game;
    RECORD.get_existing(host, Holder::Player(player)).map(|saved| saved.unwrap_or_else(no_saved))
}

fn keep(game: &mut crate::game::Game<'_>, player: u128, saved: Saved) -> bool {
    let host: &mut dyn horizon_client::mod_api::Host = game;
    match RECORD.set(host, Holder::Player(player), saved) {
        Ok(()) => true,
        Err(error) => {
            log::warn!("[BalancedMcHardcoreMain] The homecoming record is not saved: {error:?}");
            false
        }
    }
}

/// The game time of the server (`Level.getGameTime`, shared by every level).
fn game_time(game: &crate::game::Game<'_>) -> Option<i64> {
    let level = game.native_level(OVERWORLD)?;
    game.native_level_game_time(level)
}

// ---------------------------------------------------------------------------------------------
// The hooks
// ---------------------------------------------------------------------------------------------

// C10: save a new homecoming record on the player after a distant respawn. The hook helper of
// `player_respawned` calls this with the exile that `bmh_respawn::on_player_respawned` gave. The
// homes are the exile center and, for an Overworld exile, the amulet points of this moment
// (`bmh_amulet::points`). A new exile replaces the old record, and ends the look for a Java record: the
// flag is set. Java: `HomecomingHandler.onExile`.
pub(super) fn on_exile(game: &mut crate::game::Game<'_>, player: u128, exile: &Exile) {
    let points = if exile.dimension == OVERWORLD {
        crate::game::bmh_amulet::points(game, player)
    } else {
        Vec::new()
    };
    // No game time: keep the record with arrival 0, so the exile can still end (Java always saves it).
    // `show_title` then counts no nights, because it reads no game time either.
    let now = game_time(game).unwrap_or_else(|| {
        log::warn!("[BalancedMcHardcoreMain] No game time: the exile of {player:032x} gets the arrival 0");
        0
    });
    let record = logic::new_record(exile, now, &points);
    log::info!(
        "[BalancedMcHardcoreMain] Homecoming record: {} homes ({} amulet points), {} blocks, in {}",
        record.homes.len(),
        record.homes.len().saturating_sub(1),
        record.distance,
        record.dimension
    );
    keep(game, player, Saved { record: Some(record), imported: true });
}

// C10: check each exile once a second. `server_tick` has no player swapped in: the players come from
// `native_server_players`, the record from the attachment. The phase is `tick % 20 == 0` of the non-paused
// server ticks since the world opened (Java counts `tickCount` per player; only the rhythm matters).
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    if !logic::is_check_tick(context.tick.get()) {
        return;
    }
    for me in game.native_server_players() {
        // A player out of reach or with no exile costs one read.
        let Ok(Saved { record: Some(record), .. }) = stored(game, me.uuid) else {
            continue;
        };
        if record.dimension != me.dimension {
            continue;
        }
        if let Some(home) = logic::reached(&record, me.position[0], me.position[2]) {
            arrive(game, &me, &record, home);
        }
    }
}

/// The end of an exile (Java `arrive`, HomecomingHandler.java:158-178). The record goes first; if it cannot
/// be cleared nothing else happens, so a player never gets the title once a second.
fn arrive(
    game: &mut crate::game::Game<'_>,
    me: &horizon_client::mod_api::NativeServerPlayer,
    record: &logic::Record,
    home: &logic::Home,
) {
    if !keep(game, me.uuid, Saved { record: None, imported: true }) {
        return;
    }
    log::info!(
        "[BalancedMcHardcoreMain] {} is home after {} blocks ({})",
        me.name,
        record.distance,
        if home.name.is_empty() { "the center of the exile" } else { home.name.as_str() }
    );
    // The advancements do not depend on the text rule: the player is home either way.
    for path in logic::awards(record.distance) {
        let id = format!("bmh:{path}");
        let done = game
            .native_player_advancements(me.uuid)
            .and_then(|all| all.get(&id).and_then(|a| a.get("done")).and_then(|d| d.as_bool()))
            .unwrap_or(false);
        if done {
            log::debug!("[BalancedMcHardcoreMain] {} already has advancement {id}", me.name);
        } else if !crate::game::bmh::award(game, &me.name, path) {
            log::warn!("[BalancedMcHardcoreMain] Advancement {id} not awarded to {}", me.name);
        }
    }
    // The text follows the rule of the journey text. The sound does not.
    if crate::game::bmh_settings::rule(game, Rule::DisplayTextOnRespawn) {
        show_title(game, me.uuid, record, home);
    }
    ring_bell(game, me);
}

/// The title and the subtitle (HomecomingHandler.java:180-206). `native_title` sends the times, the subtitle
/// and the title in the order of `/title`. A point name is literal text, so no name can change the layout.
fn show_title(game: &mut crate::game::Game<'_>, player: u128, record: &logic::Record, home: &logic::Home) {
    // No game time leaves the nights at 0, the text for less than one night.
    let nights = game_time(game).map_or(0, |now| logic::nights(record, now));
    let subtitle = logic::subtitle(record.distance, nights);
    let args: Vec<Text> = subtitle.args.into_iter().map(Text::literal).collect();
    let subtitle = Text::translate(subtitle.key, &args);
    let title = match logic::title(home) {
        logic::Title::Key(key) => Text::translate(key, &[]),
        logic::Title::Literal(name) => Text::literal(name),
    };
    let [fade_in, stay, fade_out] = logic::TITLE_TIMES;
    if !game.native_title(player, &title, &subtitle, fade_in, stay, fade_out) {
        log::warn!("[BalancedMcHardcoreMain] The homecoming title of {player:032x} was not sent");
    }
}

/// The bell (HomecomingHandler.java:176-177): everybody near hears it, the player too. The player hears it at
/// once from `play_sound_to`; the others near hear the console line at `level_tick_post` of their level.
fn ring_bell(game: &mut crate::game::Game<'_>, me: &horizon_client::mod_api::NativeServerPlayer) {
    let at = logic::bell_position(block_of(me.position));
    if !game.native_play_sound_to(me.uuid, BELL_SOUND, logic::BELL_CATEGORY, Some(at), BELL_VOLUME, logic::BELL_PITCH) {
        log::warn!("[BalancedMcHardcoreMain] The homecoming bell of {} was not sent", me.name);
    }
    if let Some(line) = logic::bell_command(&me.name, at) {
        server().sounds.push((me.uuid, me.dimension.clone(), at, line));
    }
}

/// Looks for the record of a Java world in the players who are on, once for each player (the saved flag ends
/// it). It runs at `level_tick_post`, where no player is swapped in and `native_player_compound` gives the
/// record that Horizon loaded, with the keys it does not model. A player with no data yet, or out of reach, is
/// tried again at the next call. A new exile and a homecoming set the flag too, so the Java record of a player
/// who has had an exile in Horizon never comes back.
fn import_java_records(game: &mut crate::game::Game<'_>) {
    let online: Vec<u128> = game.native_server_players().iter().map(|p| p.uuid).collect();
    for player in online {
        let Ok(saved) = stored(game, player) else {
            continue;
        };
        if saved.imported {
            continue;
        }
        let Some(data) = game.native_player_compound(player) else {
            continue;
        };
        let java = data.get(logic::JAVA_TAG).and_then(record_of);
        if let Some(record) = &java {
            log::info!(
                "[BalancedMcHardcoreMain] Homecoming record read from the Java player data: {} homes",
                record.homes.len()
            );
        }
        keep(game, player, Saved { record: saved.record.or(java), imported: true });
    }
}

// C10: cleanup. Remove the bell lines of a player who logs out: their level may never tick again.
pub(super) fn on_player_logged_out(_game: &mut crate::game::Game<'_>, player: u128, _name: &str) {
    server().sounds.retain(|(who, _, _, _)| *who != player);
}

// C10: the Java record, and the bells for the other players. `/playsound` reaches only the players of the
// level in place, and `run_command` works here: no player is swapped in.
pub(super) fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    import_java_records(game);
    let sounds = {
        let mut s = server();
        let (here, other): (Vec<_>, Vec<_>) =
            std::mem::take(&mut s.sounds).into_iter().partition(|(_, level, _, _)| level == dimension);
        s.sounds = other;
        here
    };
    if sounds.is_empty() {
        return;
    }
    let online = game.native_server_players();
    for (player, _, at, line) in sounds {
        let others = online.iter().filter(|p| p.uuid != player && p.dimension == dimension).map(|p| p.position);
        // With nobody near, `/playsound` fails by design: skip it.
        if !logic::someone_hears(at, others) {
            continue;
        }
        if !game.native_run_command(&line) {
            log::debug!("[BalancedMcHardcoreMain] Bell command failed: {line}");
        }
    }
}
