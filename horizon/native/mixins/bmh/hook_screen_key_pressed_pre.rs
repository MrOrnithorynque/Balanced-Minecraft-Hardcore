// Hook `screen_key_pressed_pre`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_screen_key_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_journey::on_screen_key_pressed_pre(
        game, screen, key, repeat, modifiers, canceled,
    );
    crate::game::bmh_amulet::on_screen_key_pressed_pre(
        game, screen, key, repeat, modifiers, canceled,
    );
}
