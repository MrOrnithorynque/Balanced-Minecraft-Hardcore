// Hook `player_command`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_command(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    command: &str,
    canceled: &mut bool,
    replies: &mut Vec<horizon_client::text::Text>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_player_command(game, player, name, command, canceled, replies);
}
