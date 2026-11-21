// The settings page opens a local Horizon dialog. Its EditBoxes own the text,
// selection, clipboard shortcuts, focus, scrolling and mouse input.
use super::{AfterAction, Body, Click, Control, Dialog, Field, Game, Kind, Open, Text};
use bmh_logic::contract::Config;
use horizon_client::clipboard::{Clipboard, EditShortcut};
use horizon_client::dialog::{Action, Button, Input};
use std::sync::OnceLock;
use winit::keyboard::KeyCode;

const SAVE: &str = "bmh:save_local_config";
static LOCAL_FORM: OnceLock<Text> = OnceLock::new();

fn tr(key: &str) -> Text {
    Text::translate(key, &[])
}

fn button(key: &str, action: Option<Action>) -> Button {
    Button {
        label: tr(key),
        tooltip: None,
        width: 98,
        action,
    }
}

fn field(key: &str, value: i32, min: i32, max: i32) -> Input {
    Input {
        key: key.to_owned(),
        control: Control::Text {
            width: 160,
            label: Text::tr(&format!("bmh.config.{key}"), &[&min, &max]),
            label_visible: true,
            initial: value.to_string(),
            max_length: 6,
            multiline: None,
        },
    }
}

pub(in crate::game) fn open(game: &mut Game<'_>, config: Config) {
    let save = Action::Static(Click::Custom {
        id: SAVE.into(),
        payload: None,
    });
    game.open_dialog(Dialog {
        title: tr("bmh.config.title"),
        external_title: Some(LOCAL_FORM.get_or_init(|| tr("bmh.config.title")).clone()),
        kind: Kind::Confirmation {
            yes: button("gui.done", Some(save)),
            no: button("gui.cancel", None),
        },
        can_close_with_escape: true,
        pause: true,
        after_action: AfterAction::Close,
        body: vec![Body::Message {
            contents: tr("bmh.config.note"),
            width: 300,
        }],
        inputs: vec![
            field(
                "minDistanceFromBed",
                config.min_distance,
                Config::MIN_DISTANCE,
                Config::MAX_DISTANCE,
            ),
            field(
                "maxDistanceFromBed",
                config.max_distance,
                Config::MIN_DISTANCE,
                Config::MAX_DISTANCE,
            ),
            field(
                "maxAmuletPoints",
                config.max_amulet_points,
                Config::MIN_AMULET_POINTS,
                Config::MAX_AMULET_POINTS,
            ),
        ],
    });
    if let Some(open) = &mut game.dialogs.open {
        open.focus = Some(0);
    }
}

// Text clones share their source component. A decoded server dialog gets a
// separate allocation even if it copies the complete local form and action.
pub(in crate::game) fn is_open(game: &Game<'_>) -> bool {
    let Some(key) = LOCAL_FORM.get().and_then(Text::translation_key) else {
        return false;
    };
    !game.dialogs.warning
        && game
            .dialogs
            .open
            .as_ref()
            .and_then(|open| open.dialog.external_title.as_ref())
            .and_then(Text::translation_key)
            .is_some_and(|candidate| std::ptr::eq(candidate, key))
}

fn read(open: &Open) -> Result<Config, &'static str> {
    let mut fields = open.fields.iter().map(|field| match field {
        Some(Field::Line(edit)) => edit.value.parse::<i32>().ok(),
        _ => None,
    });
    let mut number = |min, max| {
        fields
            .next()
            .flatten()
            .filter(|n| (min..=max).contains(n))
            .ok_or("bmh.config.outOfRange")
    };
    let min_distance = number(Config::MIN_DISTANCE, Config::MAX_DISTANCE)?;
    let max_distance = number(Config::MIN_DISTANCE, Config::MAX_DISTANCE)?;
    let max_amulet_points = number(Config::MIN_AMULET_POINTS, Config::MAX_AMULET_POINTS)?;
    if min_distance > max_distance {
        return Err("bmh.config.minAboveMax");
    }
    Ok(Config {
        min_distance,
        max_distance,
        max_amulet_points,
    })
}

fn message(open: &mut Open, error: Option<&'static str>) {
    let key = error.unwrap_or("bmh.config.note");
    let Some(Body::Message { contents, .. }) = open.dialog.body.first_mut() else {
        return;
    };
    if contents.translation_key() != Some(key) {
        *contents = if error.is_some() {
            tr(key).with_color(0xFF8080)
        } else {
            tr(key)
        };
    }
}

pub(in crate::game) fn update(game: &mut Game<'_>) {
    if is_open(game)
        && let Some(open) = &mut game.dialogs.open
    {
        let error = read(open).err();
        if error.is_some() || open.dialog.body.first().is_some_and(|body| {
            matches!(body, Body::Message { contents, .. } if contents.translation_key() != Some("bmh.config.writeError"))
        }) {
            message(open, error);
        }
    }
}

fn submit(game: &mut Game<'_>) {
    let Some(open) = &mut game.dialogs.open else {
        return;
    };
    match read(open) {
        Err(key) => message(open, Some(key)),
        Ok(config) => match crate::game::bmh_settings::save_config(config) {
            Err(error) => {
                log::warn!("BMH config save failed: {error}");
                message(open, Some("bmh.config.writeError"));
            }
            Ok(()) => {
                game.clear_dialog();
                if let Some(i) = game
                    .settings
                    .mods
                    .installed
                    .iter()
                    .position(|m| m.id == crate::game::bmh::ID)
                {
                    game.native_settings_refresh(i);
                }
            }
        },
    }
}

pub(super) fn action(game: &mut Game<'_>, click: Option<&Click>) -> bool {
    if !is_open(game) || !matches!(click, Some(Click::Custom { id, .. }) if id == SAVE) {
        return false;
    }
    submit(game);
    true
}

pub(super) fn key(
    game: &mut Game<'_>,
    key: KeyCode,
    shortcut: Option<EditShortcut>,
    text: Option<&str>,
) -> bool {
    if !is_open(game) {
        return false;
    }
    if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter) {
        submit(game);
        return true;
    }
    if shortcut == Some(EditShortcut::Paste) {
        let text = horizon_client::clipboard::filter_text(&game.clipboard.get(), false);
        if text.chars().all(|c| c.is_ascii_digit()) {
            game.dialog_type(&text);
        }
        return true;
    }
    shortcut.is_none()
        && text.is_some_and(|s| s.chars().any(|c| !c.is_control() && !c.is_ascii_digit()))
}
