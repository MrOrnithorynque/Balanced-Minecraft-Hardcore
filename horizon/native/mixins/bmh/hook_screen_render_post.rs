// Hook `screen_render_post`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_screen_render_post(game, batch, screen);
    crate::game::bmh_amulet::on_screen_render_post(game, batch, screen);
}
