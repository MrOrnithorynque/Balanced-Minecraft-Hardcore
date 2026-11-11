// Hook `player_died`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_died(game, player, cause);
}
