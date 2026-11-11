// Area journey: the edge between the game and `bmh_logic::journey`.
// It holds the journey cinematic of a distant respawn on the client: the timeline, the drawing, the
// sounds, the music fade and the hold of the Respawn click. Every decision is in `bmh_logic::journey`;
// this file reads the clock and the screen, asks the logic, and draws or plays what it answers. Owner:
// the journey agent. Log: docs/horizon/log/journey.md.
//
// The flow of one journey, on the client (the window's `Game` only):
// 1. `net_from_server` hears `Departure` on `bmh:journey`: the next Respawn click is held.
// 2. `screen_mouse_button_pressed_pre` and `screen_key_pressed_pre` on the death screen: a press that
//    would respawn is cancelled and starts the journey (the fade to black).
// 3. `frame` (once a frame, with `&mut Game`) moves the timeline on: it sends `RespawnRequest::Held`, then
//    the real respawn request once the screen is black and the server said `Ready`, plays the sounds and
//    opens the mod's empty screen to hold the input after the respawn.
// 4. `gui_render_post` draws the picture of the last frame over the whole GUI, the death screen and the
//    respawn loading screen included.
// 5. `music` fades the game music during the fade and stops it at the black.
// 6. `Arrival` gives the name for the message. With no journey running (a gamepad click, the
//    immediate respawn rule), it starts the cinematic in black.

use bmh_logic::journey as logic;
use bmh_logic::wire::{JOURNEY_CHANNEL, JourneySignal, RESPAWN_CHANNEL, RespawnRequest};
use horizon_client::mod_api::{NativeScreenKind, NativeWorldId};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

/// The client state of the area. One value for the process (ARCHITECTURE.md, State): the window's
/// thread only.
struct Client {
    journey: logic::Client,
    /// The start of the area's clock.
    epoch: Option<Instant>,
    /// The world of the last frame: a new world ends a journey.
    world: Option<NativeWorldId>,
    /// The last place of the local player: the sounds play there while the respawn loads.
    position: Option<[f64; 3]>,
    /// Mixed into the seed of each sky.
    serial: i64,
}

static CLIENT: Mutex<Client> = Mutex::new(Client {
    journey: logic::Client::new(),
    epoch: None,
    world: None,
    position: None,
    serial: 0,
});

/// The client state. A lock that a panic left behind still holds valid data.
fn client() -> MutexGuard<'static, Client> {
    CLIENT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The area's clock in milliseconds, as Java's `Util.getMillis()` (a monotonic clock).
fn now(client: &mut Client) -> i64 {
    let epoch = *client.epoch.get_or_insert_with(Instant::now);
    i64::try_from(epoch.elapsed().as_millis()).unwrap_or(i64::MAX)
}

/// A seed for a new sky: the wall clock and a counter (Java `new Random()` seeds from the clock too).
fn seed(client: &mut Client) -> i64 {
    client.serial = client.serial.wrapping_add(1);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i64);
    nanos ^ client.serial.wrapping_mul(0x5DEE_CE66D)
}

/// The art grid of the window, from its size in screen pixels and its GUI size.
fn grid(game: &crate::game::Game<'_>, gui_width: f32) -> logic::Grid {
    match game.native_physical_window_size() {
        Some(window) => logic::grid(logic::gui_scale(window[0], gui_width), window),
        None => logic::Grid::default(),
    }
}

/// The block of a position.
fn block_of(pos: [f64; 3]) -> [i32; 3] {
    pos.map(|v| v.floor() as i32)
}

// C4, C8: read the departure, arrival and ready signals on `bmh:journey`.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != JOURNEY_CHANNEL || !game.native_window() {
        return;
    }
    let Some(signal) = JourneySignal::decode(data) else {
        log::warn!("[BalancedMcHardcoreMain] A journey signal that is not valid was ignored");
        return;
    };
    let mut c = client();
    let (now, seed) = (now(&mut c), seed(&mut c));
    c.journey.signal(now, signal, seed);
}

// C4: hold the Respawn click of the death screen until the screen is black and the landing is ready.
// While a journey runs, every click of the death screen and of the hold screen goes to it, as the Java
// overlay took them.
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
    let cancel = match screen.id {
        "dead" => {
            let size = game
                .native_client_state()
                .map_or([screen.rect[2], screen.rect[3]], |state| state.gui_size);
            let title = horizon_client::text::Text::translate(logic::CONFIRM_TITLE_KEY, &[]).plain();
            let title_width = game.native_text_width(&title);
            let mut c = client();
            let (now, seed) = (now(&mut c), seed(&mut c));
            c.journey.death_click(now, button, screen.mouse, size, title_width, seed)
        }
        "mod" => game.native_mod_screen() == Some(crate::game::bmh::ID) && client().journey.holds_input(),
        _ => false,
    };
    if cancel {
        *canceled = true;
    }
}

// C4, C16: hold the keys of the death screen that would respawn, and every key of the hold screen while
// a journey runs (Java `HoldScreen` swallows them, Escape included).
pub(super) fn on_screen_key_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = repeat;
    if *canceled {
        return;
    }
    let cancel = match screen.id {
        "dead" => {
            let mut c = client();
            let (now, seed) = (now(&mut c), seed(&mut c));
            c.journey.death_key(now, key, modifiers, seed)
        }
        "mod" => game.native_mod_screen() == Some(crate::game::bmh::ID) && client().journey.holds_input(),
        _ => false,
    };
    if cancel {
        *canceled = true;
    }
}

// C4, C15, C16, C17: move the timeline on once a frame, then send, play and hold what it answers.
pub(super) fn on_frame(game: &mut crate::game::Game<'_>) {
    if !game.native_window() {
        return;
    }
    let state = game.native_client_state();
    let world = state.as_ref().and_then(|s| s.world.clone());
    let kind = state.as_ref().map_or(NativeScreenKind::Other, |s| s.screen);
    let hold_open = game.native_mod_screen() == Some(crate::game::bmh::ID);
    let position = game.native_player().map(|p| [p.x, p.y, p.z]);
    // Java `terrainReady`: no loading screen, and the player's chunk on the client. Right after a
    // respawn the screen is Playing; the only other screen until the hold opens is the loading screen.
    let loading = position.is_none() || (kind == NativeScreenKind::Other && !hold_open);
    let chunk = position.is_some_and(|p| game.native_client_block_obstruction(block_of(p)).is_some());
    let grid = state.as_ref().map_or(logic::Grid::default(), |s| grid(game, s.gui_size[0]));
    let (commands, at) = {
        let mut c = client();
        let now = now(&mut c);
        let same = c.world == world;
        if !same {
            c.world.clone_from(&world);
        }
        if position.is_some() {
            c.position = position;
        }
        let view = logic::View {
            now,
            in_world: world.is_some() && same,
            death_screen: kind == NativeScreenKind::Death,
            loading,
            chunk,
            grid,
        };
        (c.journey.frame(&view), c.position)
    };
    if commands.send_held
        && let Err(error) = game.native_net_send_to_server(RESPAWN_CHANNEL, &RespawnRequest::Held.encode())
    {
        log::warn!("[BalancedMcHardcoreMain] Held respawn not told to the server: {error}");
    }
    if commands.send_respawn {
        // `LocalPlayer.respawn`: `ServerboundClientCommandPacket` PERFORM_RESPAWN (remote.rs:10494-10496).
        let packet = horizon_client::mod_api::ServerboundPlayPacket::ClientCommand(
            horizon_protocol::packets::play::ClientCommand { action: 0 },
        );
        if let Err(error) = game.native_send_packet(packet) {
            log::warn!("[BalancedMcHardcoreMain] Respawn request not sent: {error}");
        }
    }
    if let Some(at) = at {
        for sound in &commands.sounds {
            game.native_play_local_sound(sound.id, at, sound.volume, sound.pitch, 0);
        }
    }
    // The mod's empty screen takes the keys and the mouse while the screen is dark. It opens only over
    // the world with no other screen, so it waits for the end of the respawn load.
    if commands.hold && !game.native_mod_screen().is_some_and(|id| id == crate::game::bmh::ID) {
        game.native_mod_screen_open(crate::game::bmh::ID);
    }
    if commands.release {
        game.native_mod_screen_close(crate::game::bmh::ID);
    }
}

// C15, C16: draw the journey cinematic over the whole GUI, the death screen and the loading screen.
pub(super) fn on_gui_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    gui_hidden: bool,
    focused: bool,
) {
    // The Java overlay draws under F1 and without focus too.
    let _ = (gui_hidden, focused);
    if !game.native_window() {
        return;
    }
    let Some(picture) = client().journey.picture() else {
        return;
    };
    if let Some(color) = picture.fade {
        game.native_fill(batch, [0.0, 0.0, batch.width, batch.height], color);
    }
    // One art pixel is `pixel` screen pixels; the batch is in GUI pixels (JourneyOverlay.java:204).
    let k = picture.grid.pixel as f32 / batch.scale.max(1.0);
    for fill in picture.sky.iter().chain(&picture.art) {
        let rect = [
            fill.x0 as f32 * k,
            fill.y0 as f32 * k,
            (fill.x1 - fill.x0) as f32 * k,
            (fill.y1 - fill.y0) as f32 * k,
        ];
        game.native_fill(batch, rect, fill.argb);
    }
    if let Some(message) = &picture.message {
        use horizon_client::hud_layout::PoseOp;
        use horizon_client::mod_api::GuiDraw;
        use horizon_client::text::Text;
        let text = Text::translate_with_fallback(
            logic::MESSAGE_KEY,
            logic::MESSAGE_FALLBACK,
            &[Text::literal(message.name.clone())],
        );
        // Java `Font.width` is a whole number of GUI pixels.
        let width = game.native_text_width(&text.plain()).ceil() as i32;
        let layout = logic::message_layout(width, batch.width as i32, batch.height as i32);
        let draws = [
            GuiDraw::Push(vec![
                PoseOp::Translate(layout.translate[0], layout.translate[1]),
                PoseOp::Scale(layout.scale, layout.scale),
            ]),
            GuiDraw::Text {
                text,
                x: layout.at[0],
                y: layout.at[1],
                color: message.color(),
                shadow: false,
            },
            GuiDraw::Pop,
        ];
        game.native_gui_paint(batch, &draws);
    }
}

// C17: fade the game music during the fade to black, and stop it at the black (JourneyMusic.java).
pub(super) fn on_music(
    game: &crate::game::Game<'_>,
    music: &mut Option<horizon_client::music::Music>,
    volume: &mut f32,
) {
    if !game.native_window() {
        return;
    }
    let action = {
        let mut c = client();
        let now = now(&mut c);
        c.journey.music(now)
    };
    match action {
        logic::MusicAction::Keep => {}
        logic::MusicAction::Scale(part) => *volume *= part,
        logic::MusicAction::Cut => {
            // A track that replaces the current one stops it at once; nothing plays for this id.
            *music = Some(horizon_client::music::Music::new(
                logic::MUSIC_CUT,
                logic::MUSIC_CUT_DELAY,
                logic::MUSIC_CUT_DELAY,
                true,
            ));
        }
        logic::MusicAction::Quiet => *music = None,
    }
}
