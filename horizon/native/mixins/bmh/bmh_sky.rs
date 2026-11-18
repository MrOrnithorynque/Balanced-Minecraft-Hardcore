// Area sky: the edge between the game and `bmh_logic::stars`.
// It will hold: the star reading drawn in the sky, the spyglass name.
// Owner: the sky agent. The hook helpers (hook_*.rs) call the functions below.

// V1: keep the newest star reading that the amulet area sent on `bmh:sky`
// (`bmh_logic::wire::SkySignal::Reading`, docs/horizon/HOOKS.md "Contracts").
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    let _ = (game, channel, data);
}

// V2: draw the line name through the spyglass.
pub(super) fn on_gui_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    gui_hidden: bool,
    focused: bool,
) {
    let _ = (game, batch, gui_hidden, focused);
}

// V2: note on the client that the player looks through a spyglass.
pub(super) fn on_living_entity_use_item_tick(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: &mut i32,
    canceled: &mut bool,
) {
    let _ = (game, stack, entity, hand, duration, canceled);
}
