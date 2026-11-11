// Hook `screen_char_typed_pre`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_screen_char_typed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    c: char,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_amulet::on_screen_char_typed_pre(game, screen, c, modifiers, canceled);
}
