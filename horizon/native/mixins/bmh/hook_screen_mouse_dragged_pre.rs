fn bmh_hook_screen_mouse_dragged_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if game.native_on(crate::game::bmh::ID) {
        crate::game::bmh_amulet::on_screen_mouse_dragged_pre(game, screen, button, canceled);
    }
}
