// Hook `level_tick_post`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_level_tick_post(game, dimension);
    crate::game::bmh_amulet::on_level_tick_post(game, dimension);
}
