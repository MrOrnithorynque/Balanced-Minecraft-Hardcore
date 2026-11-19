// Hook `player_totem`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_totem(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_player_totem(game, player, name);
}
