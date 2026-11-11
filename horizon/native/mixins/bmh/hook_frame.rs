// Hook `frame`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_frame(game: &mut crate::game::Game<'_>) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_journey::on_frame(game);
}
