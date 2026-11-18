// Hook `level_load`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_level_load(game: &mut crate::game::Game<'_>, dimension: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_level_load(game, dimension);
}
