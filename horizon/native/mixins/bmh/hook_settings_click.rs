// Hook `settings_click`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_settings_click(game, mod_id, button);
}
