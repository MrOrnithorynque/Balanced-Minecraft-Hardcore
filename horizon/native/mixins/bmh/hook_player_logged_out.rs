// Hook `player_logged_out`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_logged_out(game, player, name);
    crate::game::bmh_totem::on_player_logged_out(game, player, name);
    crate::game::bmh_ritual::on_player_logged_out(game, player, name);
    crate::game::bmh_tools::on_player_logged_out(game, player, name);
    crate::game::bmh_amulet::on_player_logged_out(game, player, name);
    crate::game::bmh_homecoming::on_player_logged_out(game, player, name);
}
