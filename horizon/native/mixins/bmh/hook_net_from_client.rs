// Hook `net_from_client`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_net_from_client(game, player, channel, data);
    crate::game::bmh_amulet::on_net_from_client(game, player, channel, data);
}
