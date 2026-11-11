// Hook `screen_mouse_button_pressed_pre`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_screen_mouse_button_pressed_pre(
        game, screen, button, modifiers, canceled,
    );
    crate::game::bmh_journey::on_screen_mouse_button_pressed_pre(
        game, screen, button, modifiers, canceled,
    );
    crate::game::bmh_amulet::on_screen_mouse_button_pressed_pre(
        game, screen, button, modifiers, canceled,
    );
}
