// Hook `difficulty_changed`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_difficulty_changed(game: &mut crate::game::Game<'_>, from: &str, to: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_difficulty_changed(game, from, to);
}
