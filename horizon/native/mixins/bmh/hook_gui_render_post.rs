// Hook `gui_render_post`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_gui_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    gui_hidden: bool,
    focused: bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_sky::on_gui_render_post(game, batch, gui_hidden, focused);
    crate::game::bmh_journey::on_gui_render_post(game, batch, gui_hidden, focused);
}
