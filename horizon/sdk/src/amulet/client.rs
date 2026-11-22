use std::sync::Mutex;

use crate::logic::amulet as logic;
use crate::logic::stars;
use crate::logic::wire::{AMULET_CHANNEL, AmuletRequest, AmuletSignal};
use mod_api::client::widgets::{Button, Outcome, TextField, WidgetId, Widgets};
use mod_api::{
    ClientHost, DrawContext, Flow, GuiDraw, GuiRenderPostEvent, ItemModelRangePropertyEvent,
    PayloadBytes, PayloadId, RenderLevelStage, RenderLevelStageEvent, ScreenCharTypedPreEvent,
    ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent,
    ScreenMouseDraggedPreEvent, ScreenRenderPostEvent, Text, WorldDepth, WorldQuad,
};

use super::{OVERWORLD, in_overworld};

pub(crate) const NORTH: &str = "bmh:north";

struct Naming {
    pos: [i32; 3],
    editing: bool,
    bed: bool,
    used: i32,
    max: i32,
    field: WidgetId,
    buttons: Vec<(logic::NamingButton, WidgetId)>,
    widgets: Widgets,
}

struct List {
    max: i32,
    points: Vec<(String, [i32; 3])>,
    page: usize,
}

enum Page {
    Naming(Naming),
    List(List),
}

enum Action {
    Close,
    Request(AmuletRequest, bool),
}

struct Screens {
    pending: Option<(u64, AmuletSignal)>,
    page: Option<Page>,
    action: Option<Action>,
}

static SCREENS: Mutex<Screens> = Mutex::new(Screens {
    pending: None,
    page: None,
    action: None,
});

pub(super) fn receive(host: &dyn ClientHost, signal: AmuletSignal) {
    if !matches!(signal, AmuletSignal::Cooldown { .. }) {
        let now = host.millis();
        SCREENS
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .pending = Some((now, signal));
    }
}

pub(crate) fn tick(host: &mut dyn ClientHost) {
    let own_screen = host.mod_screen() == Some(logic::SCREEN_ID);
    let (pending, action) = {
        let mut screens = SCREENS.lock().unwrap_or_else(|error| error.into_inner());
        if !own_screen {
            screens.page = None;
        }
        (screens.pending.take(), screens.action.take())
    };
    if own_screen && let Some(action) = action {
        match action {
            Action::Close => host.mod_screen_close(logic::SCREEN_ID),
            Action::Request(request, close) => {
                if let Err(error) = host.send_payload_to_server(
                    &PayloadId(AMULET_CHANNEL.into()),
                    PayloadBytes(request.encode()),
                ) {
                    eprintln!("[BMH] Could not send the amulet point request: {error:?}");
                } else if close {
                    host.mod_screen_close(logic::SCREEN_ID);
                }
            }
        }
    }
    let Some((received, signal)) = pending else {
        return;
    };
    if host.mod_screen() != Some(logic::SCREEN_ID) && !host.mod_screen_open(logic::SCREEN_ID) {
        if host.millis().saturating_sub(received) < 2_000 {
            let mut screens = SCREENS.lock().unwrap_or_else(|error| error.into_inner());
            if screens.pending.is_none() {
                screens.pending = Some((received, signal));
            }
        }
        return;
    }
    let page = match signal {
        AmuletSignal::Open {
            pos,
            name,
            used,
            max,
            bed,
        } => {
            let editing = name.is_some();
            let mut widgets = Widgets::new();
            let mut field = TextField::new([0.0; 4], logic::MAX_NAME_LENGTH);
            if let Some(name) = name {
                field.set_value(&name);
            }
            let field = widgets.add_text_field(field);
            let layout = logic::naming_layout(0, 0, editing);
            let buttons = layout
                .buttons
                .into_iter()
                .map(|(button, _)| {
                    let label = match button {
                        logic::NamingButton::Save => logic::KEY_SCREEN_SAVE,
                        logic::NamingButton::Remove => logic::KEY_SCREEN_REMOVE,
                        logic::NamingButton::Cancel => logic::KEY_CANCEL,
                    };
                    let id =
                        widgets.add_button(Button::new([0.0; 4], Text::translate(label, vec![])));
                    (button, id)
                })
                .collect();
            widgets.set_focus(host, Some(field));
            Page::Naming(Naming {
                pos,
                editing,
                bed,
                used,
                max,
                field,
                buttons,
                widgets,
            })
        }
        AmuletSignal::List { max, points } => {
            let screens = SCREENS.lock().unwrap_or_else(|error| error.into_inner());
            let page = match screens.page.as_ref() {
                Some(Page::List(list)) => list.page.min(logic::last_page(points.len())),
                _ => 0,
            };
            Page::List(List { max, points, page })
        }
        AmuletSignal::Cooldown { .. } => return,
    };
    SCREENS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .page = Some(page);
}

fn with_page<T>(host: &dyn ClientHost, f: impl FnOnce(&mut Page) -> T) -> Option<T> {
    if host.mod_screen() != Some(logic::SCREEN_ID) {
        return None;
    }
    let mut page = SCREENS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .page
        .take()?;
    let result = f(&mut page);
    SCREENS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .page = Some(page);
    Some(result)
}

fn active(naming: &Naming) -> bool {
    naming
        .widgets
        .text_field(naming.field)
        .is_some_and(|field| {
            logic::save_active(naming.editing, naming.used, naming.max, field.value())
        })
}

fn naming_result(naming: &Naming, outcome: Outcome) -> Option<Action> {
    let button = match outcome {
        Outcome::Pressed(id) => naming.buttons.iter().find(|(_, button)| *button == id)?.0,
        Outcome::Submitted(id) if id == naming.field => logic::NamingButton::Save,
        _ => return None,
    };
    match button {
        logic::NamingButton::Save if active(naming) => {
            let name = logic::clean_name(naming.widgets.text_field(naming.field)?.value());
            Some(Action::Request(
                AmuletRequest::Save {
                    pos: naming.pos,
                    name,
                },
                true,
            ))
        }
        logic::NamingButton::Remove => Some(Action::Request(
            AmuletRequest::Remove { pos: naming.pos },
            true,
        )),
        logic::NamingButton::Cancel => Some(Action::Close),
        _ => None,
    }
}

fn flow(handled: bool, action: Option<Action>) -> Flow {
    if let Some(action) = action {
        SCREENS
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .action = Some(action);
    }
    if handled {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

pub(crate) fn click(host: &dyn ClientHost, event: &ScreenMouseButtonPressedPreEvent<'_>) -> Flow {
    let (handled, action) = with_page(host, |page| match page {
        Page::Naming(naming) => {
            let outcome = naming.widgets.mouse_pressed(
                host,
                event.screen.mouse,
                event.button,
                event.modifiers,
            );
            (outcome.handled(), naming_result(naming, outcome))
        }
        Page::List(list) if event.button == 0 => {
            let (width, height) = logic::window_size(event.screen.rect);
            let layout = logic::list_layout(width, height, list.points.len(), list.page);
            let clicked = layout
                .buttons
                .into_iter()
                .find(|(_, rect, active)| *active && logic::inside(*rect, event.screen.mouse));
            let Some((button, _, _)) = clicked else {
                return (false, None);
            };
            let action = match button {
                logic::ListButton::Remove(index) => list
                    .points
                    .get(index)
                    .map(|(_, pos)| Action::Request(AmuletRequest::Delete { pos: *pos }, false)),
                logic::ListButton::Previous => {
                    list.page = logic::turn_page(list.page, -1, list.points.len());
                    None
                }
                logic::ListButton::Next => {
                    list.page = logic::turn_page(list.page, 1, list.points.len());
                    None
                }
                logic::ListButton::Done => Some(Action::Close),
            };
            (true, action)
        }
        Page::List(_) => (false, None),
    })
    .unwrap_or_default();
    flow(handled, action)
}

pub(crate) fn release(
    host: &dyn ClientHost,
    event: &ScreenMouseButtonReleasedPreEvent<'_>,
) -> Flow {
    let handled = with_page(host, |page| match page {
        Page::Naming(naming) => naming.widgets.mouse_released(event.button).handled(),
        Page::List(_) => false,
    })
    .unwrap_or(false);
    flow(handled, None)
}

pub(crate) fn drag(host: &dyn ClientHost, event: &ScreenMouseDraggedPreEvent<'_>) -> Flow {
    let handled = with_page(host, |page| match page {
        Page::Naming(naming) => naming
            .widgets
            .mouse_dragged(host, event.screen.mouse, event.button)
            .handled(),
        Page::List(_) => false,
    })
    .unwrap_or(false);
    flow(handled, None)
}

pub(crate) fn key(host: &dyn ClientHost, event: &ScreenKeyPressedPreEvent<'_>) -> Flow {
    let (handled, action) = with_page(host, |page| match page {
        Page::Naming(naming) => {
            let outcome = naming.widgets.key_pressed(host, event.key, event.modifiers);
            (outcome.handled(), naming_result(naming, outcome))
        }
        Page::List(_) if logic::is_confirmation(event.key) => (true, Some(Action::Close)),
        Page::List(_) => (false, None),
    })
    .unwrap_or_default();
    flow(handled, action)
}

pub(crate) fn character(host: &dyn ClientHost, event: &ScreenCharTypedPreEvent<'_>) -> Flow {
    let handled = with_page(host, |page| match page {
        Page::Naming(naming) => naming.widgets.char_typed(host, event.c).handled(),
        Page::List(_) => false,
    })
    .unwrap_or(false);
    flow(handled, None)
}

fn translated(key: &str, args: impl IntoIterator<Item = String>) -> Text {
    Text::translate(key, args.into_iter().map(Text::literal).collect())
}

fn centered(
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    text: Text,
    center: i32,
    y: i32,
    color: u32,
) {
    let Ok(width) = host.text_width(&text) else {
        return;
    };
    let x = logic::centered_x(center, width as f32);
    let _ = draw.gui_paint(&[GuiDraw::Text {
        text,
        x,
        y,
        color,
        shadow: false,
    }]);
}

pub(crate) fn screen(host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
    let _ = with_page(host, |page| {
        let (width, height) = logic::window_size(event.screen.rect);
        let draw = &mut *event.draw;
        draw.fill(event.screen.rect, logic::DIM_COLOR);
        match page {
            Page::Naming(naming) => {
                let layout = logic::naming_layout(width, height, naming.editing);
                let title = if naming.editing {
                    logic::KEY_SCREEN_EDIT
                } else {
                    logic::KEY_SCREEN_NEW
                };
                let subtitle = if naming.bed {
                    logic::KEY_SCREEN_BED
                } else {
                    logic::KEY_SCREEN_LODESTONE
                };
                centered(
                    host,
                    draw,
                    translated(title, []),
                    layout.center_x,
                    layout.title_y,
                    logic::TEXT_COLOR,
                );
                centered(
                    host,
                    draw,
                    translated(subtitle, []),
                    layout.center_x,
                    layout.subtitle_y,
                    logic::NOTE_COLOR,
                );
                if let Some(field) = naming.widgets.text_field_mut(naming.field) {
                    field.rect = layout.field.map(|value| value as f32);
                }
                let save_active = active(naming);
                for ((button, id), (_, rect)) in naming.buttons.iter().zip(layout.buttons) {
                    if let Some(widget) = naming.widgets.button_mut(*id) {
                        widget.rect = rect.map(|value| value as f32);
                        widget.active = *button != logic::NamingButton::Save || save_active;
                    }
                }
                let _ = naming.widgets.draw(draw, host, event.screen.mouse);
                let full = logic::screen_full(naming.editing, naming.used, naming.max);
                let (key, values) = logic::count_line(naming.editing, naming.used, naming.max);
                let color = if full {
                    logic::FULL_COLOR
                } else {
                    logic::NOTE_COLOR
                };
                let text = translated(key, values.iter().map(i32::to_string));
                centered(host, draw, text, layout.center_x, layout.count_y, color);
            }
            Page::List(list) => {
                draw_list(host, draw, event.screen.mouse, width, height, list);
            }
        }
    });
}

fn draw_list(
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    mouse: [f32; 2],
    width: i32,
    height: i32,
    list: &List,
) {
    let layout = logic::list_layout(width, height, list.points.len(), list.page);
    centered(
        host,
        draw,
        translated(logic::KEY_LIST_TITLE, []),
        layout.center_x,
        layout.title_y,
        logic::TEXT_COLOR,
    );
    let count = [list.points.len().to_string(), list.max.to_string()];
    centered(
        host,
        draw,
        translated(logic::KEY_LIST_COUNT, count),
        layout.center_x,
        layout.count_y,
        logic::NOTE_COLOR,
    );
    if list.points.is_empty() {
        centered(
            host,
            draw,
            translated(logic::KEY_LIST_EMPTY, []),
            layout.center_x,
            layout.empty_y,
            logic::NOTE_COLOR,
        );
    }
    for (index, y) in layout.rows {
        let Some((name, [x, point_y, z])) = list.points.get(index) else {
            continue;
        };
        let fitted = logic::fit_width(name, logic::NAME_WIDTH, |text| draw.text_width(text));
        draw.text(
            &fitted,
            [layout.left as f32, y as f32],
            logic::TEXT_COLOR,
            false,
        );
        draw.text(
            &format!("{x} {point_y} {z}"),
            [(layout.left + logic::COORDS_X) as f32, y as f32],
            logic::NOTE_COLOR,
            false,
        );
    }
    for (button, rect, active) in layout.buttons {
        let label = match button {
            logic::ListButton::Remove(_) => translated(logic::KEY_SCREEN_REMOVE, []),
            logic::ListButton::Previous => Text::literal("<"),
            logic::ListButton::Next => Text::literal(">"),
            logic::ListButton::Done => translated(logic::KEY_DONE, []),
        };
        let highlighted = active && logic::inside(rect, mouse);
        let _ = draw.widget_button(rect.map(|value| value as f32), &label, active, highlighted);
    }
    centered(
        host,
        draw,
        translated(logic::KEY_LIST_SPAWN, []),
        layout.center_x,
        layout.spawn_y,
        logic::NOTE_COLOR,
    );
}

struct SkyState {
    sky: stars::Sky,
    fade: stars::NameFade,
}

static SKY: Mutex<SkyState> = Mutex::new(SkyState {
    sky: stars::Sky::new(),
    fade: stars::NameFade::new(),
});

pub(super) fn reading(reading: crate::logic::contract::AmuletReading) {
    SKY.lock()
        .unwrap_or_else(|error| error.into_inner())
        .sky
        .hear(reading);
}

pub(crate) fn unload() {
    let mut sky = SKY.lock().unwrap_or_else(|error| error.into_inner());
    sky.sky = stars::Sky::new();
    sky.fade = stars::NameFade::new();
    drop(sky);
    *SCREENS.lock().unwrap_or_else(|error| error.into_inner()) = Screens {
        pending: None,
        page: None,
        action: None,
    };
    *NEEDLE.lock().unwrap_or_else(|error| error.into_inner()) = Needle::EMPTY;
}

pub(crate) fn draw_sky(host: &dyn ClientHost, event: &mut RenderLevelStageEvent<'_>) {
    if event.stage != RenderLevelStage::AfterSky {
        return;
    }
    let Some(level) = host.level(OVERWORLD) else {
        return;
    };
    let Some(player) = host.local_player() else {
        return;
    };
    if player.level(host) != Some(level) {
        return;
    }
    let (Some(day_time), Some((rain, thunder))) = (level.day_time(host), level.weather(host))
    else {
        return;
    };
    let star_brightness = stars::star_brightness(day_time, rain, thunder);
    let view = stars::SkyView {
        now_ms: milliseconds(host),
        game_time: event.view.game_time,
        day_time,
        star_brightness,
        stars_drawn: star_brightness > 0.0,
    };
    let (frame, logs) = {
        let mut state = SKY.lock().unwrap_or_else(|error| error.into_inner());
        state.sky.level(Some(OVERWORLD.to_owned()));
        state.sky.frame(&view)
    };
    for line in logs {
        eprintln!("[BMH] {line}");
    }
    let Some(frame) = frame else { return };
    let camera = event.view.camera;
    let (sin, cos) = f64::from(frame.star_angle).to_radians().sin_cos();
    let quad_limit = event
        .draw
        .usage()
        .quad_capacity
        .saturating_sub(event.draw.usage().quads);
    let stride = stars::STRIDE;
    let quads: Vec<_> = frame
        .vertices
        .chunks_exact(stride * 3)
        .take(quad_limit as usize)
        .map(|triangle| {
            let vertex = |index: usize| {
                let [x, y, z] = [0, 1, 2].map(|axis| f64::from(triangle[index * stride + axis]));
                [
                    camera[0] - y * sin - z * cos,
                    camera[1] + y * cos - z * sin,
                    camera[2] + x,
                ]
            };
            let channel = |offset: usize| {
                let total =
                    triangle[offset] + triangle[stride + offset] + triangle[2 * stride + offset];
                (total.clamp(0.0, 3.0) * (255.0 / 3.0)).round() as u32
            };
            // The public draw context gives a quad one colour. A degenerate corner keeps the triangle's outline.
            let argb = (channel(6) << 24) | (channel(3) << 16) | (channel(4) << 8) | channel(5);
            let [a, b, c] = [0, 1, 2].map(vertex);
            WorldQuad::new([a, b, c, c], argb, WorldDepth::Tested)
        })
        .collect();
    let _ = event.draw.quads(&quads);
}

pub(crate) fn spyglass(host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
    if event.gui_hidden
        || !host.camera_type().is_first_person()
        || host
            .observed_screen_id()
            .is_some_and(|screen| screen.id.as_str() != "chat")
    {
        return;
    }
    let Some(player) = host.local_player() else {
        return;
    };
    let Some(living) = host.living(player.entity()) else {
        return;
    };
    let Some(hand) = living.used_item_hand(host) else {
        return;
    };
    let Some(stack) = living.hand_item(host, hand) else {
        return;
    };
    if stack.item().id(host) != "minecraft:spyglass" {
        return;
    }
    let Some(day) = player.level(host).and_then(|level| level.day_time(host)) else {
        return;
    };
    let Some(camera) = host.camera() else { return };
    let look = stars::look(camera.yaw, camera.pitch);
    let now = milliseconds(host);
    let name = {
        let mut state = SKY.lock().unwrap_or_else(|error| error.into_inner());
        let name = state.sky.aimed_name(now, day, look).map(str::to_owned);
        let alpha = state.fade.update(name.as_deref(), now);
        name.zip(alpha)
    };
    let Some((name, alpha)) = name else { return };
    let size = event.batch.size().map(|value| value as i32);
    let width = event.batch.text_width(&name).ceil() as i32;
    let line = stars::name_draw(&name, width, size, alpha);
    let _ = event.batch.gui_paint(&[GuiDraw::Text {
        text: Text::literal(line.text),
        x: line.x,
        y: line.y,
        color: line.argb,
        shadow: true,
    }]);
}

fn milliseconds(host: &dyn ClientHost) -> i64 {
    host.millis().try_into().unwrap_or(i64::MAX)
}

struct Needle {
    tick: Option<i64>,
    phase: f32,
    velocity: f32,
    north: bool,
    random: u64,
}

impl Needle {
    const EMPTY: Self = Self {
        tick: None,
        phase: 0.0,
        velocity: 0.0,
        north: true,
        random: 1,
    };

    fn update(&mut self, tick: i64, north: bool, target: f32) -> f32 {
        if self.north != north {
            *self = Self {
                north,
                ..Self::EMPTY
            };
        }
        if self.tick != Some(tick) {
            self.tick = Some(tick);
            let target = if north {
                target
            } else {
                self.random = self
                    .random
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                (self.random >> 40) as f32 / 16_777_216.0
            };
            let delta = (target - self.phase + 0.5).rem_euclid(1.0) - 0.5;
            self.velocity = (self.velocity + delta * 0.1) * 0.8;
            self.phase = (self.phase + self.velocity).rem_euclid(1.0);
        }
        self.phase
    }
}

static NEEDLE: Mutex<Needle> = Mutex::new(Needle::EMPTY);

pub(crate) fn compass(host: &dyn ClientHost, event: &mut ItemModelRangePropertyEvent<'_>) {
    if event.property != NORTH {
        return;
    }
    let Some(player) = host.local_player() else {
        return;
    };
    let Some(level) = player.level(host) else {
        return;
    };
    let Some(tick) = level.game_time(host) else {
        return;
    };
    let entity = player.entity();
    let Some([x, _, z]) = entity.position(host) else {
        return;
    };
    let Some([yaw, _]) = entity.rotation(host) else {
        return;
    };
    let dx = x.floor() + 0.5 - x;
    let dz = (z - 1_000_000.0).floor() + 0.5 - z;
    let bearing = (dz.atan2(dx) / f64::from(std::f32::consts::TAU)) as f32;
    let north = in_overworld(host, player);
    let target = 0.5 - ((yaw / 360.0).rem_euclid(1.0) - 0.25);
    let phase = NEEDLE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .update(tick, north, target);
    event.value = (phase + if north { bearing } else { 0.0 }).rem_euclid(1.0);
}
