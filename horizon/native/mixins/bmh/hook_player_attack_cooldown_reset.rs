// Hook `player_attack_cooldown_reset`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_attack_cooldown_reset(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_player_attack_cooldown_reset(game, player, name);
}
