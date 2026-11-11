// Area amulet: the edge between the game and `bmh_logic::amulet`.
// It will hold: use and use-on, readings and charges, named points on the player, /bmh point, the
// naming and list screens.
// Owner: the amulet agent. The hook helpers (hook_*.rs) call the functions below.

// A5: the named points that this player has now, in the order of the list (Java
// `AmuletPoints.get`). Contract: `bmh_homecoming::on_exile` reads them. Keep this signature.
pub(super) fn points(
    game: &mut crate::game::Game<'_>,
    player: u128,
) -> Vec<bmh_logic::contract::AmuletPoint> {
    let _ = (game, player);
    Vec::new()
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// A6: open the naming screen or the point list on `bmh:amulet`.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    let _ = (game, channel, data);
}

// A5, A6: check and apply a point save, rename or remove, on `bmh:amulet`.
pub(super) fn on_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    let _ = (game, player, channel, data);
}

// A6: clicks on the naming screen and the point list.
pub(super) fn on_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (game, screen, button, modifiers, canceled);
}

// A6: the keys of the naming field.
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

// A6: the characters typed in the naming field.
pub(super) fn on_screen_char_typed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    c: char,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (game, screen, c, modifiers, canceled);
}

// A6: draw the naming screen and the point list.
pub(super) fn on_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    let _ = (game, batch, screen);
}

// A5: register `/bmh point`.
pub(super) fn on_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    let _ = (game, commands);
}

// A5: run `/bmh point`.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let _ = (game, player, command, ran);
}

// A2: refill the charges of the amulet each night.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = (game, slot, selected);
}

// A2: read the stars. A3: drop the use that follows a claimed `use_item_on`.
pub(super) fn on_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    let _ = (game, stack, claimed);
}

// A3, A5: sneak and use on a bed or a lodestone opens the naming of a point.
pub(super) fn on_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let _ = (game, stack, pos, face, claimed);
}

// A3: cancel the off hand use of the amulet.
pub(super) fn on_interaction_key(
    game: &mut crate::game::Game<'_>,
    button: i32,
    key_mapping: &str,
    hand: horizon_client::first_person::Hand,
    swing_hand: &mut bool,
    canceled: &mut bool,
) {
    let _ = (game, button, key_mapping, hand, swing_hand, canceled);
}

// A3: track sneaking for the use on a bed or a lodestone.
pub(super) fn on_player_toggle_sneak(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    sneaking: bool,
) {
    let _ = (game, player, name, sneaking);
}

// A2: the world spawn line of a reading.
pub(super) fn on_spawn_change(game: &mut crate::game::Game<'_>, pos: [i32; 3], angle: [f32; 2]) {
    let _ = (game, pos, angle);
}

// A1: the tooltip lines of the amulet.
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    let _ = (game, stack, lines);
}

// A1: the charge bar of the amulet.
pub(super) fn on_item_bar(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    let _ = (game, stack, bar);
}
