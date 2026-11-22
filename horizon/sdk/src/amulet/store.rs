use crate::logic::amulet as logic;
use crate::logic::contract::AmuletPoint;
use mod_api::{
    AttachmentCodec, AttachmentError, AttachmentType, Compound, Holder, PlayerRef, ServerHost, Tag,
};

const IMPORTED_TAG: &str = "imported";

#[derive(Clone)]
struct Saved {
    points: Vec<AmuletPoint>,
    imported: bool,
}

fn no_points() -> Saved {
    Saved {
        points: Vec::new(),
        imported: false,
    }
}

/// The points of a player, saved with the player and copied across death (GAPS.md A5). The saved form of
/// the list is the Java one: a list of compounds with `name`, `x`, `y` and `z` (AmuletPoints.java:45-56).
static POINTS: AttachmentType<Saved> = AttachmentType {
    namespace: "bmh",
    name: "amulet_points",
    default: no_points,
    codec: Some(AttachmentCodec {
        encode: encode_points,
        decode: decode_points,
    }),
    copy_on_death: true,
    sync: false,
};

fn encode_points(saved: &Saved) -> Result<Compound, AttachmentError> {
    let list = saved.points.iter().map(point_tag).collect();
    Ok([
        (logic::POINTS_TAG.into(), Tag::List(list)),
        (IMPORTED_TAG.into(), Tag::Byte(i8::from(saved.imported))),
    ]
    .into())
}

fn point_tag(point: &AmuletPoint) -> Tag {
    let [x, y, z] = point.pos;
    Tag::compound([
        ("name", Tag::String(point.name.as_str().into())),
        ("x", Tag::Int(x)),
        ("y", Tag::Int(y)),
        ("z", Tag::Int(z)),
    ])
}

fn decode_points(data: &Compound) -> Result<Saved, AttachmentError> {
    // A missing flag reads as "not imported": the merge of the import never doubles a point.
    let imported = matches!(data.get(&IMPORTED_TAG.into()), Some(Tag::Byte(flag)) if *flag != 0);
    match data.get(&logic::POINTS_TAG.into()) {
        Some(Tag::List(list)) => Ok(Saved {
            points: points_of_list(list),
            imported,
        }),
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
pub(super) fn number(tag: &Tag) -> Option<i64> {
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
    match (
        data.get(logic::POINTS_TAG),
        data.get(logic::LEGACY_HOME_TAG),
    ) {
        (Some(Tag::List(list)), _) => points_of_list(list),
        (_, Some(home @ Tag::Compound(_))) => {
            vec![AmuletPoint {
                name: logic::LEGACY_HOME_NAME.to_owned(),
                pos: position_of(home),
            }]
        }
        _ => Vec::new(),
    }
}

pub(super) fn load(host: &mut dyn ServerHost, player: PlayerRef) -> Option<Vec<AmuletPoint>> {
    let holder = Holder::Player(player.uuid());
    let saved = match POINTS.get_existing(host.attachments(), holder) {
        Ok(saved) => saved,
        Err(error) => {
            eprintln!("[BMH] Could not read the amulet points: {error:?}");
            return None;
        }
    };
    let mut saved = saved.unwrap_or_else(no_points);
    if !saved.imported
        && let Some(data) = player.entity().nbt(host)
    {
        let imported = java_points(&Tag::Compound(data));
        if !imported.is_empty() {
            eprintln!(
                "[BMH] {} amulet points read from the Java player data",
                imported.len()
            );
        }
        saved.points = logic::merge_imported(&imported, &saved.points);
        saved.imported = true;
        let _ = POINTS.set(host.attachments(), holder, saved.clone());
    }
    Some(saved.points)
}

pub(super) fn save(host: &mut dyn ServerHost, player: PlayerRef, points: Vec<AmuletPoint>) -> bool {
    let holder = Holder::Player(player.uuid());
    let imported = POINTS
        .get_existing(host.attachments(), holder)
        .ok()
        .flatten()
        .is_some_and(|saved| saved.imported);
    match POINTS.set(host.attachments(), holder, Saved { points, imported }) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("[BMH] Could not save the amulet points: {error:?}");
            false
        }
    }
}
