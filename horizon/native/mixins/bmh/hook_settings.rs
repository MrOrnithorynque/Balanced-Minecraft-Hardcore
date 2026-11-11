// Hook `settings`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_settings(game, mod_id, buttons);
}
