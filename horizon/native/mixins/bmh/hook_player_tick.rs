// Hook `player_tick`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_tick(game: &mut crate::game::Game<'_>) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_tick(game);
    crate::game::bmh_totem::on_player_tick(game);
    crate::game::bmh_tools::on_player_tick(game);
}
