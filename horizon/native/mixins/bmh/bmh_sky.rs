// Area sky: the edge between the game and `bmh_logic::stars`.
// It holds the star reading of the Navigator's Amulet, drawn in the night sky, and the line name through the
// spyglass. Every decision is in `bmh_logic::stars`; this file reads the clock, the weather, the level and the
// camera, asks the logic, and hands the triangles to `horizon_client_render::bmh_sky_render`, which draws them in
// `world_drawn`. Owner: the sky agent. Log: docs/horizon/log/sky.md.
//
// The flow on the client (the window's `Game` only):
// 1. `net_from_server` hears `SkySignal::Reading` on `bmh:sky` (the amulet area sends it): the reading waits.
// 2. `frame` (once a frame, before the drawing) starts a waiting reading by the Java start rule, builds the
//    triangles of this frame and gives them to the render module.
// 3. `living_entity_use_item_tick` stamps the area's clock while the local player uses a spyglass.
// 4. `gui_render_post` draws the name of the line that the view aims at, while the spyglass shows and no screen
//    covers the world.

use bmh_logic::stars as logic;
use bmh_logic::wire::{SKY_CHANNEL, SkySignal};
use horizon_client::mod_api::NativeScreenKind;
use horizon_client_render::bmh_sky_render;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

/// The client state of the area. One value for the process (ARCHITECTURE.md, State): the window's thread only.
struct Client {
    sky: logic::Sky,
    fade: logic::NameFade,
    /// The area's clock (`now`) at the last use tick of a spyglass.
    scoping: Option<i64>,
    /// The start of the area's clock.
    epoch: Option<Instant>,
    /// The sky azimuth self-check is in the log.
    checked: bool,
}

static CLIENT: Mutex<Client> = Mutex::new(Client {
    sky: logic::Sky::new(),
    fade: logic::NameFade::new(),
    scoping: None,
    epoch: None,
    checked: false,
});

fn client() -> MutexGuard<'static, Client> {
    CLIENT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The area's clock in milliseconds: a monotonic clock, as Java's `Util.getMillis()`. The animation runs on it
/// (Java counts GUI ticks, 20 a second: StarReadingRenderer.java:131-133).
fn now(client: &mut Client) -> i64 {
    let epoch = *client.epoch.get_or_insert_with(Instant::now);
    i64::try_from(epoch.elapsed().as_millis()).unwrap_or(i64::MAX)
}

// V1: keep the newest star reading that the amulet area sent on `bmh:sky`
// (`bmh_logic::wire::SkySignal::Reading`, docs/horizon/HOOKS.md "Contracts").
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != SKY_CHANNEL || !game.native_window() {
        return;
    }
    let Some(SkySignal::Reading(reading)) = SkySignal::decode(data) else {
        log::warn!("[BalancedMcHardcoreMain] A star reading from the server could not be read");
        return;
    };
    client().sky.hear(reading);
}

// V1: once a frame, before the drawing: start a fresh reading and give this frame's triangles to the renderer.
pub(super) fn on_frame(game: &mut crate::game::Game<'_>) {
    if !game.native_window() {
        return;
    }
    let world = game.native_client_state().and_then(|state| state.world);
    let player = game.native_player();
    // Java resets on another `ClientLevel`: another world or another dimension (StarReadingRenderer.java:124-130).
    // In a world with no player yet (a loading screen), the level stays as it was.
    let level = match (&world, &player) {
        (None, _) => Some(None),
        (Some(world), Some(player)) => Some(Some(format!("{world:?}|{}", player.dimension))),
        (Some(_), None) => None,
    };
    let clock = game.native_clock();
    let (rain, thunder) = game.native_weather();
    let (frame, logs, check) = {
        let mut c = client();
        let check = !c.checked;
        c.checked = true;
        if let Some(level) = level {
            c.sky.level(level);
        }
        let now = now(&mut c);
        let (frame, logs) = match &clock {
            Some(clock) => {
                let star_brightness = logic::star_brightness(clock.day_time, rain, thunder);
                // Horizon draws its stars only in the Overworld, with a star brightness above 0
                // (game.rs:16024-16032, 12289-12290). Java draws the reading only when vanilla draws the stars.
                let overworld = player.as_ref().is_some_and(|p| p.dimension == "minecraft:overworld");
                let view = logic::SkyView {
                    now_ms: now,
                    game_time: clock.game_time,
                    day_time: clock.day_time,
                    star_brightness,
                    stars_drawn: overworld && star_brightness > 0.0,
                };
                c.sky.frame(&view)
            }
            None => (None, Vec::new()),
        };
        (frame, logs, check)
    };
    if check {
        log::info!("{}", logic::self_check_log());
    }
    for line in logs {
        log::info!("{line}");
    }
    bmh_sky_render::show(
        frame.map(|f| bmh_sky_render::SkyFrame { vertices: f.vertices, star_angle: f.star_angle }),
    );
}

// V2: draw the line name through the spyglass (K_GuiSpyglassMixin.java). Java draws it at the end of the spyglass
// overlay, which vanilla draws only in first person while the player scopes, and which F1 hides (Horizon
// game.rs:8508-8517, fov.rs:252-264). The overlay is part of the HUD, under any screen, but this hook draws after
// the screens (native_hooks.rs:2626-2627). So the name draws only with no screen, or with the chat, which leaves
// the centre of the view open (a minimap draws under it too: NativeScreenKind::Chat).
pub(super) fn on_gui_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    gui_hidden: bool,
    _focused: bool,
) {
    if gui_hidden || !game.native_window() {
        return;
    }
    let open = game.native_client_state().map(|state| state.screen);
    if !matches!(open, Some(NativeScreenKind::Playing | NativeScreenKind::Chat)) {
        return;
    }
    let (Some(camera), Some(clock), Some(player)) = (game.native_camera(), game.native_clock(), game.native_player())
    else {
        return;
    };
    let eyes = [player.x, player.y + f64::from(camera.eye_height), player.z];
    if !logic::first_person(camera.position, eyes) {
        return;
    }
    let look = logic::look(camera.yaw, camera.pitch);
    let shown = {
        let mut c = client();
        let now = now(&mut c);
        if !logic::scoping(c.scoping, now) {
            return;
        }
        let name = c.sky.aimed_name(now, clock.day_time, look).map(str::to_owned);
        let alpha = c.fade.update(name.as_deref(), now);
        name.zip(alpha)
    };
    let Some((name, alpha)) = shown else {
        return;
    };
    // Java `Font.width` is a whole number of GUI pixels.
    let width = game.native_text_width(&name).ceil() as i32;
    let draw = logic::name_draw(&name, width, [batch.width as i32, batch.height as i32], alpha);
    // Java `centeredText` draws with a shadow.
    let text = horizon_client::mod_api::GuiDraw::Text {
        text: horizon_client::text::Text::literal(draw.text),
        x: draw.x,
        y: draw.y,
        color: draw.argb,
        shadow: true,
    };
    game.native_gui_paint(batch, &[text]);
}

// V2: note on the client that the local player looks through a spyglass. The hook fires every tick of a use, on
// both sides; on the window, `entity` is None for the local player (native_hooks.rs:8096-8115).
pub(super) fn on_living_entity_use_item_tick(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    _hand: horizon_client::first_person::Hand,
    _duration: &mut i32,
    _canceled: &mut bool,
) {
    if entity.is_some() || &*stack.item != "minecraft:spyglass" || !game.native_window() {
        return;
    }
    let mut c = client();
    c.scoping = Some(now(&mut c));
}
