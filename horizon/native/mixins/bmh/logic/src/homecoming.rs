// Area homecoming (owner: the homecoming agent): the pure logic of the end of an exile.
// Java reference: handlers/HomecomingHandler.java. The record, the home test (24 blocks, same dimension),
// the nights, the title and subtitle keys, the two advancements and the bell command.
// The game side is `bmh_homecoming.rs`; nothing here knows a game type.

use crate::contract::{AmuletPoint, Exile};
use crate::respawn::{BELL_SOUND, BELL_VOLUME, FAR_AWAY_BLOCKS, OVERWORLD, SOUND_RANGE, is_command_name};

/// A player is home within this horizontal distance of a home, in blocks (HomecomingHandler.java:49).
pub const RADIUS: i32 = 24;
/// The check runs this often, in ticks (HomecomingHandler.java:51).
pub const CHECK_INTERVAL: u64 = 20;
/// Ticks in one night, for the count of nights away (HomecomingHandler.java:54).
pub const TICKS_PER_NIGHT: i64 = 24_000;
/// The pitch of the bell (HomecomingHandler.java:55).
pub const BELL_PITCH: f32 = 0.8;
/// The sound category of the bell, `SoundSource.PLAYERS` (HomecomingHandler.java:177).
pub const BELL_CATEGORY: &str = "player";

/// The title times in ticks: fade in, stay, fade out (HomecomingHandler.java:202).
pub const TITLE_TIMES: [i32; 3] = [10, 70, 20];

/// The title of a home that is the center of the exile (HomecomingHandler.java:38).
pub const TITLE_HOME: &str = "bmh.homecoming.home";
/// The subtitle for two nights or more. Arguments: the blocks and the nights (HomecomingHandler.java:40).
pub const SUBTITLE: &str = "bmh.homecoming.subtitle";
/// The subtitle for exactly one night. Argument: the blocks (HomecomingHandler.java:42).
pub const SUBTITLE_ONE_NIGHT: &str = "bmh.homecoming.subtitle.one_night";
/// The subtitle for less than one night. Argument: the blocks (HomecomingHandler.java:44).
pub const SUBTITLE_NO_NIGHTS: &str = "bmh.homecoming.subtitle.no_nights";

/// The advancement of every homecoming (HomecomingHandler.java:167).
pub const ADV_HOME_AT_LAST: &str = "journey/home_at_last";
/// The advancement of a homecoming after 30 000 blocks or more (HomecomingHandler.java:169-171).
pub const ADV_LONG_WAY_HOME: &str = "journey/long_way_home";

/// The key of the record in the player data of a Java world (HomecomingHandler.java:48, `TAG`).
pub const JAVA_TAG: &str = "bmh_homecoming";
/// The fields of the saved record, with the Java names (HomecomingHandler.java:62-76).
pub const FIELD_DIMENSION: &str = "dimension";
pub const FIELD_ARRIVAL: &str = "arrival";
pub const FIELD_DISTANCE: &str = "distance";
pub const FIELD_HOMES: &str = "homes";
pub const FIELD_NAME: &str = "name";
pub const FIELD_X: &str = "x";
pub const FIELD_Y: &str = "y";
pub const FIELD_Z: &str = "z";

/// A place that ends the exile. The name is empty for the center of the exile (HomecomingHandler.java:57).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    pub name: String,
    pub pos: [i32; 3],
}

/// The exile a player is on: the dimension, the game time of the arrival, the distance and the homes
/// (HomecomingHandler.java:60-61). The saved form keeps the Java field names `dimension`, `arrival`,
/// `distance` and `homes` (HomecomingHandler.java:71-76).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub dimension: String,
    pub arrival: i64,
    pub distance: i32,
    pub homes: Vec<Home>,
}

/// The record of a new exile (HomecomingHandler.java:98-112). The center comes first, with an empty name, so
/// it wins a tie. The amulet points count only for an Overworld exile, in the order of the list.
pub fn new_record(exile: &Exile, game_time: i64, points: &[AmuletPoint]) -> Record {
    let mut homes = vec![Home { name: String::new(), pos: exile.center }];
    if exile.dimension == OVERWORLD {
        homes.extend(points.iter().map(|point| Home { name: point.name.clone(), pos: point.pos }));
    }
    Record { dimension: exile.dimension.clone(), arrival: game_time, distance: exile.distance, homes }
}

/// True on the ticks that run the check: one in `CHECK_INTERVAL` (E_ServerPlayerMixin.java:44).
pub fn is_check_tick(tick: u64) -> bool {
    tick.is_multiple_of(CHECK_INTERVAL)
}

/// True when the check may look at a player. A dead player is out of reach, as in Java (a dead `ServerPlayer`
/// does not tick). A player with an early respawn pending stands at the vanilla point until the move to the
/// landing, which may be the home of the old exile: that is no homecoming.
pub fn may_check(health: f32, early_pending: bool) -> bool {
    health > 0.0 && !early_pending
}

/// The nearest home within the radius, or None. Only x and z count. The first home wins a tie
/// (HomecomingHandler.java:130-150).
pub fn reached(record: &Record, x: f64, z: f64) -> Option<&Home> {
    let mut nearest = None;
    let mut nearest_squared = f64::from(RADIUS) * f64::from(RADIUS);
    for home in &record.homes {
        let dx = x - (f64::from(home.pos[0]) + 0.5);
        let dz = z - (f64::from(home.pos[2]) + 0.5);
        let squared = dx * dx + dz * dz;
        if squared <= nearest_squared {
            nearest = Some(home);
            nearest_squared = squared - 1.0E-9;
        }
    }
    nearest
}

/// Whole nights between the arrival of the exile and now (HomecomingHandler.java:154-156).
pub fn nights(record: &Record, game_time: i64) -> i64 {
    (game_time.saturating_sub(record.arrival) / TICKS_PER_NIGHT).max(0)
}

/// The subtitle: a translation key and its arguments as plain text (HomecomingHandler.java:187-195).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subtitle {
    pub key: &'static str,
    pub args: Vec<String>,
}

/// The subtitle for this distance and this count of nights: two nights or more, exactly one, or none.
pub fn subtitle(distance: i32, nights: i64) -> Subtitle {
    let blocks = distance.to_string();
    if nights >= 2 {
        Subtitle { key: SUBTITLE, args: vec![blocks, nights.to_string()] }
    } else if nights == 1 {
        Subtitle { key: SUBTITLE_ONE_NIGHT, args: vec![blocks] }
    } else {
        Subtitle { key: SUBTITLE_NO_NIGHTS, args: vec![blocks] }
    }
}

/// The title of a home (HomecomingHandler.java:197-199): the key "Home" for the center of the exile, else the
/// name of the point as literal text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Title<'a> {
    Key(&'static str),
    Literal(&'a str),
}

pub fn title(home: &Home) -> Title<'_> {
    if home.name.is_empty() { Title::Key(TITLE_HOME) } else { Title::Literal(&home.name) }
}

/// The advancements of one homecoming, in the Java order (HomecomingHandler.java:167-171).
pub fn awards(distance: i32) -> Vec<&'static str> {
    let mut awards = vec![ADV_HOME_AT_LAST];
    if distance >= FAR_AWAY_BLOCKS {
        awards.push(ADV_LONG_WAY_HOME);
    }
    awards
}

/// Where the bell sounds. Java plays it with `level.playSound(null, BlockPos, ...)`, which sounds at the center
/// of the block (HomecomingHandler.java:177).
pub fn bell_position(block: [i32; 3]) -> [f64; 3] {
    block.map(|axis| f64::from(axis) + 0.5)
}

/// The console line of the bell for the other players near `at`. The player hears it from `play_sound_to`.
/// None for a name that is not a player name.
pub fn bell_command(name: &str, at: [f64; 3]) -> Option<String> {
    is_command_name(name).then(|| {
        let [x, y, z] = at;
        format!("playsound {BELL_SOUND} {BELL_CATEGORY} @a[name=!{name}] {x} {y} {z} {BELL_VOLUME} {BELL_PITCH}")
    })
}

/// True when another player is close enough to hear `/playsound` at `at` (the range of 16 blocks). Without
/// a listener the command fails by design, so the caller skips it.
pub fn someone_hears(at: [f64; 3], others: impl IntoIterator<Item = [f64; 3]>) -> bool {
    others.into_iter().any(|p| {
        let d = [p[0] - at[0], p[1] - at[1], p[2] - at[2]];
        d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= SOUND_RANGE * SOUND_RANGE
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exile(dimension: &str, center: [i32; 3], distance: i32) -> Exile {
        Exile { dimension: dimension.to_owned(), center, distance }
    }

    fn point(name: &str, pos: [i32; 3]) -> AmuletPoint {
        AmuletPoint { name: name.to_owned(), pos }
    }

    fn home(name: &str, x: i32, z: i32) -> Home {
        Home { name: name.to_owned(), pos: [x, 64, z] }
    }

    fn record(homes: Vec<Home>) -> Record {
        Record { dimension: OVERWORLD.to_owned(), arrival: 1000, distance: 12_345, homes }
    }

    #[test]
    fn the_constants_are_the_java_values() {
        assert_eq!(RADIUS, 24);
        assert_eq!(CHECK_INTERVAL, 20);
        assert_eq!(TICKS_PER_NIGHT, 24_000);
        assert_eq!(BELL_PITCH, 0.8);
        assert_eq!(TITLE_TIMES, [10, 70, 20]);
        assert_eq!(BELL_SOUND, "bmh:bell_sfx");
        assert_eq!(BELL_VOLUME, 1.0);
    }

    #[test]
    fn an_overworld_exile_keeps_the_center_first_then_the_points_in_order() {
        let points = [point("Camp", [10, 70, 20]), point("Base", [-5, 12, 0])];
        let r = new_record(&exile(OVERWORLD, [100, 64, -200], 23_333), 4242, &points);
        assert_eq!(r.dimension, OVERWORLD);
        assert_eq!(r.arrival, 4242);
        assert_eq!(r.distance, 23_333);
        assert_eq!(
            r.homes,
            vec![
                Home { name: String::new(), pos: [100, 64, -200] },
                Home { name: "Camp".to_owned(), pos: [10, 70, 20] },
                Home { name: "Base".to_owned(), pos: [-5, 12, 0] },
            ]
        );
    }

    #[test]
    fn a_nether_exile_ignores_the_amulet_points() {
        let points = [point("Camp", [10, 70, 20])];
        let r = new_record(&exile("minecraft:the_nether", [8, 64, 8], 2917), 7, &points);
        assert_eq!(r.dimension, "minecraft:the_nether");
        assert_eq!(r.homes, vec![Home { name: String::new(), pos: [8, 64, 8] }]);
    }

    #[test]
    fn an_exile_with_no_points_has_only_the_center() {
        let r = new_record(&exile(OVERWORLD, [0, 64, 0], 5000), 0, &[]);
        assert_eq!(r.homes.len(), 1);
    }

    #[test]
    fn the_check_runs_one_tick_in_twenty() {
        assert!(is_check_tick(0));
        assert!(!is_check_tick(1));
        assert!(!is_check_tick(19));
        assert!(is_check_tick(20));
        assert!(is_check_tick(40));
        assert!((0..200).filter(|t| is_check_tick(*t)).count() == 10);
    }

    #[test]
    fn a_home_is_reached_within_24_blocks_of_its_block_center() {
        // Home block (100, 100): its center is (100.5, 100.5).
        let r = record(vec![home("", 100, 100)]);
        assert!(reached(&r, 100.5, 100.5).is_some());
        // Exactly 24 blocks east of the center: 576 <= 576, inside.
        assert!(reached(&r, 124.5, 100.5).is_some());
        // Just past 24 blocks.
        assert!(reached(&r, 124.51, 100.5).is_none());
        // 24 blocks west of the center is x = 76.5.
        assert!(reached(&r, 76.5, 100.5).is_some());
        assert!(reached(&r, 76.49, 100.5).is_none());
        // 17 east and 17 south: 578 > 576, outside. 16 and 17: 545, inside.
        assert!(reached(&r, 117.5, 117.5).is_none());
        assert!(reached(&r, 116.5, 117.5).is_some());
    }

    #[test]
    fn the_height_is_ignored() {
        let r = Record { homes: vec![Home { name: String::new(), pos: [0, -60, 0] }], ..record(vec![]) };
        assert!(reached(&r, 0.5, 0.5).is_some());
    }

    #[test]
    fn the_nearest_home_wins() {
        let r = record(vec![home("", 0, 0), home("Camp", 10, 0)]);
        // At x = 9.5 the camp (center 10.5) is 1 block away and the center (0.5) is 9 blocks away.
        assert_eq!(reached(&r, 9.5, 0.5).map(|h| h.name.as_str()), Some("Camp"));
        assert_eq!(reached(&r, 1.5, 0.5).map(|h| h.name.as_str()), Some(""));
    }

    #[test]
    fn the_first_home_wins_a_tie() {
        // The center and a point on the same block: the center comes first and wins.
        let r = record(vec![home("", 50, 50), home("Same", 50, 50)]);
        assert_eq!(reached(&r, 50.5, 50.5).map(|h| h.name.as_str()), Some(""));
        // Two homes at equal distance: the first wins. x = 5.5 is 5 from 0.5 and 5 from 10.5.
        let r = record(vec![home("A", 0, 0), home("B", 10, 0)]);
        assert_eq!(reached(&r, 5.5, 0.5).map(|h| h.name.as_str()), Some("A"));
        let r = record(vec![home("B", 10, 0), home("A", 0, 0)]);
        assert_eq!(reached(&r, 5.5, 0.5).map(|h| h.name.as_str()), Some("B"));
    }

    #[test]
    fn no_home_is_reached_with_no_homes() {
        assert!(reached(&record(vec![]), 0.0, 0.0).is_none());
    }

    #[test]
    fn nights_count_whole_days_since_the_arrival() {
        let r = record(vec![]);
        assert_eq!(nights(&r, 1000), 0);
        assert_eq!(nights(&r, 1000 + 23_999), 0);
        assert_eq!(nights(&r, 1000 + 24_000), 1);
        assert_eq!(nights(&r, 1000 + 47_999), 1);
        assert_eq!(nights(&r, 1000 + 48_000), 2);
        assert_eq!(nights(&r, 1000 + 240_000), 10);
    }

    #[test]
    fn nights_never_go_below_zero() {
        let r = record(vec![]);
        assert_eq!(nights(&r, 0), 0);
        assert_eq!(nights(&r, -50_000), 0);
        assert_eq!(nights(&r, i64::MIN), 0);
    }

    #[test]
    fn the_subtitle_follows_the_nights() {
        assert_eq!(
            subtitle(23_333, 3),
            Subtitle { key: "bmh.homecoming.subtitle", args: vec!["23333".to_owned(), "3".to_owned()] }
        );
        assert_eq!(subtitle(23_333, 2).key, SUBTITLE);
        assert_eq!(
            subtitle(23_333, 1),
            Subtitle { key: "bmh.homecoming.subtitle.one_night", args: vec!["23333".to_owned()] }
        );
        assert_eq!(
            subtitle(5000, 0),
            Subtitle { key: "bmh.homecoming.subtitle.no_nights", args: vec!["5000".to_owned()] }
        );
    }

    #[test]
    fn the_title_is_home_for_the_center_and_the_name_for_a_point() {
        assert_eq!(title(&home("", 0, 0)), Title::Key("bmh.homecoming.home"));
        assert_eq!(title(&home("Camp", 0, 0)), Title::Literal("Camp"));
        // A name stays literal text, with quotes and backslashes as typed.
        let odd = r#"A "b" \c"#;
        assert_eq!(title(&home(odd, 0, 0)), Title::Literal(odd));
    }

    #[test]
    fn home_at_last_always_and_long_way_home_from_30000_blocks() {
        assert_eq!(awards(5000), vec!["journey/home_at_last"]);
        assert_eq!(awards(29_999), vec!["journey/home_at_last"]);
        assert_eq!(awards(30_000), vec!["journey/home_at_last", "journey/long_way_home"]);
        assert_eq!(awards(60_000), vec!["journey/home_at_last", "journey/long_way_home"]);
        assert_eq!(awards(0), vec!["journey/home_at_last"]);
    }

    #[test]
    fn the_bell_sounds_at_the_center_of_the_block() {
        assert_eq!(bell_position([100, 64, -20]), [100.5, 64.5, -19.5]);
    }

    #[test]
    fn the_bell_line_is_exact() {
        let at = bell_position([100, 64, -20]);
        assert_eq!(
            bell_command("Alex_1", at).as_deref(),
            Some("playsound bmh:bell_sfx player @a[name=!Alex_1] 100.5 64.5 -19.5 1 0.8")
        );
    }

    #[test]
    fn the_bell_line_refuses_a_name_that_is_not_a_player_name() {
        let at = [0.5, 64.5, 0.5];
        assert_eq!(bell_command("", at), None);
        assert_eq!(bell_command("a b", at), None);
        assert_eq!(bell_command("x]@a[", at), None);
        assert_eq!(bell_command("seventeen_chars_x", at), None);
    }

    #[test]
    fn someone_hears_within_16_blocks_only() {
        let at = [0.5, 64.5, 0.5];
        assert!(!someone_hears(at, []));
        assert!(someone_hears(at, [[16.5, 64.5, 0.5]]), "exactly 16 blocks");
        assert!(!someone_hears(at, [[16.6, 64.5, 0.5]]));
        assert!(someone_hears(at, [[500.0, 0.0, 0.0], [10.0, 70.0, 10.0]]));
    }

    #[test]
    fn a_dead_player_or_an_early_respawn_pending_is_not_checked() {
        assert!(may_check(20.0, false));
        assert!(may_check(0.5, false));
        assert!(!may_check(0.0, false), "dead");
        assert!(!may_check(20.0, true), "early respawn, still at the vanilla point");
        assert!(!may_check(0.0, true));
    }
}
