// Area journey: the edge between the game and `bmh_logic::journey`.
// It will hold: the cinematic on the client: timeline, drawing, sounds, the Respawn hold.
// Owner: the journey agent. The hook helpers (hook_*.rs) call the functions below.

// C4, C8: read the departure, arrival and ready signals on `bmh:journey`.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    let _ = (game, channel, data);
}

// C4: hold the Respawn click of the death screen until the landing is ready.
pub(super) fn on_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (game, screen, button, modifiers, canceled);
}

// C4: hold the keys of the death screen until the landing is ready.
pub(super) fn on_screen_key_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (game, screen, key, repeat, modifiers, canceled);
}

// C15, C16: draw the journey cinematic and hold the black screen.
pub(super) fn on_gui_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    gui_hidden: bool,
    focused: bool,
) {
    let _ = (game, batch, gui_hidden, focused);
}

// C17: fade the music during the journey.
pub(super) fn on_music(
    game: &crate::game::Game<'_>,
    music: &mut Option<horizon_client::music::Music>,
    volume: &mut f32,
) {
    let _ = (game, music, volume);
}
