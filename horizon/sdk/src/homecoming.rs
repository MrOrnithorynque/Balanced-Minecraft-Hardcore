use crate::logic::contract::{Exile, Rule};
use crate::logic::{homecoming as logic, respawn};
use mod_api::{
    AttachmentCodec, AttachmentError, AttachmentType, Compound, Holder, PlayerRef, ServerHost,
    SoundSource, Tag,
};

use crate::{amulet, settings};

static HOME: AttachmentType<Option<logic::Record>> = AttachmentType {
    namespace: "bmh",
    name: "homecoming",
    default: || None,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: true,
    sync: false,
};

fn encode(record: &Option<logic::Record>) -> Result<Compound, AttachmentError> {
    let Some(record) = record else {
        return Ok(Compound::new());
    };
    let homes = record
        .homes
        .iter()
        .map(|home| {
            let [x, y, z] = home.pos;
            Tag::compound([
                (logic::FIELD_NAME, Tag::String(home.name.as_str().into())),
                (logic::FIELD_X, Tag::Int(x)),
                (logic::FIELD_Y, Tag::Int(y)),
                (logic::FIELD_Z, Tag::Int(z)),
            ])
        })
        .collect();
    Ok([
        (
            logic::FIELD_DIMENSION.into(),
            Tag::String(record.dimension.as_str().into()),
        ),
        (logic::FIELD_ARRIVAL.into(), Tag::Long(record.arrival)),
        (logic::FIELD_DISTANCE.into(), Tag::Int(record.distance)),
        (logic::FIELD_HOMES.into(), Tag::List(homes)),
    ]
    .into())
}

fn decode(tag: &Compound) -> Result<Option<logic::Record>, AttachmentError> {
    if tag.is_empty() {
        return Ok(None);
    }
    let (
        Some(Tag::String(dimension)),
        Some(Tag::Long(arrival)),
        Some(Tag::Int(distance)),
        Some(Tag::List(homes)),
    ) = (
        tag.get(&logic::FIELD_DIMENSION.into()),
        tag.get(&logic::FIELD_ARRIVAL.into()),
        tag.get(&logic::FIELD_DISTANCE.into()),
        tag.get(&logic::FIELD_HOMES.into()),
    )
    else {
        return Err(AttachmentError::Decode);
    };
    let homes = homes
        .iter()
        .filter_map(|home| {
            let Some(Tag::String(name)) = home.get(logic::FIELD_NAME) else {
                return None;
            };
            let [Some(Tag::Int(x)), Some(Tag::Int(y)), Some(Tag::Int(z))] =
                [logic::FIELD_X, logic::FIELD_Y, logic::FIELD_Z].map(|axis| home.get(axis))
            else {
                return None;
            };
            Some(logic::Home {
                name: name.to_string_lossy(),
                pos: [*x, *y, *z],
            })
        })
        .collect();
    Ok(Some(logic::Record {
        dimension: dimension.to_string_lossy(),
        arrival: *arrival,
        distance: *distance,
        homes,
    }))
}

pub(super) fn arrived(host: &mut dyn ServerHost, player: PlayerRef, exile: &Exile) {
    let Some(time) = host
        .level(&exile.dimension)
        .and_then(|level| level.game_time(host))
    else {
        return;
    };
    let points = amulet::points(host, player);
    let record = logic::new_record(exile, time, &points);
    if let Err(error) = HOME.set(
        host.attachments(),
        Holder::Player(player.uuid()),
        Some(record),
    ) {
        eprintln!("[BMH] Could not save the homecoming record: {error:?}");
    }
}

pub(super) fn tick(host: &mut dyn ServerHost, player: PlayerRef) {
    let Some(level) = player.level(host) else {
        return;
    };
    let Some(time) = level
        .game_time(host)
        .filter(|time| logic::is_check_tick(*time as u64))
    else {
        return;
    };
    if !host
        .living(player.entity())
        .and_then(|living| living.alive(host))
        .unwrap_or(false)
    {
        return;
    }

    let holder = Holder::Player(player.uuid());
    let record =
        match HOME.get_existing(host.attachments(), holder) {
            Ok(Some(record)) => record,
            Ok(None) => player.persistent_data(host).and_then(|data| {
                match data.get(&logic::JAVA_TAG.into()) {
                    Some(Tag::Compound(data)) => decode(data).ok().flatten(),
                    _ => None,
                }
            }),
            Err(_) => return,
        };
    let Some(record) = record else { return };
    if level.dimension().id(host).as_deref() != Some(&record.dimension) {
        return;
    }
    let Some([x, _, z]) = player.entity().position(host) else {
        return;
    };
    let Some(home) = logic::reached(&record, x, z) else {
        return;
    };
    let Some(name) = player
        .name(host)
        .filter(|name| respawn::is_command_name(name))
    else {
        return;
    };

    if HOME.set(host.attachments(), holder, None).is_err() {
        return;
    }

    player.play_sound(
        host,
        respawn::BELL_SOUND,
        SoundSource::Players,
        respawn::BELL_VOLUME,
        logic::BELL_PITCH,
    );
    for award in logic::awards(record.distance) {
        if let Some(command) = respawn::award_command(&name, award) {
            host.commands().run_command(&command);
        }
    }

    if settings::rule(host, Rule::DisplayTextOnRespawn) {
        let title = match logic::title(home) {
            logic::Title::Key(key) => format!(r#"{{"translate":"{key}"}}"#),
            logic::Title::Literal(name) => format!(r#"{{"text":{}}}"#, quoted(name)),
        };
        let sub = logic::subtitle(record.distance, logic::nights(&record, time));
        let args = sub
            .args
            .iter()
            .map(|arg| quoted(arg))
            .collect::<Vec<_>>()
            .join(",");
        let subtitle = format!(r#"{{"translate":"{}","with":[{args}]}}"#, sub.key);
        let [fade, stay, disappear] = logic::TITLE_TIMES;
        for command in [
            format!("title {name} times {fade} {stay} {disappear}"),
            format!("title {name} subtitle {subtitle}"),
            format!("title {name} title {title}"),
        ] {
            host.commands().run_command(&command);
        }
    }
}

fn quoted(text: &str) -> String {
    let mut result = String::from("\"");
    for ch in text.chars().filter(|ch| !ch.is_control()) {
        if ch == '"' || ch == '\\' {
            result.push('\\');
        }
        result.push(ch);
    }
    result.push('"');
    result
}
