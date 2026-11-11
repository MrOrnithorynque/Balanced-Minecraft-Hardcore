// Hook `player_toggle_sneak`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_toggle_sneak(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    sneaking: bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_amulet::on_player_toggle_sneak(game, player, name, sneaking);
}
