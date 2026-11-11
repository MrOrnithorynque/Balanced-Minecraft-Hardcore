// Hook `net_from_server`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_journey::on_net_from_server(game, channel, data);
    crate::game::bmh_tools::on_net_from_server(game, channel, data);
    crate::game::bmh_amulet::on_net_from_server(game, channel, data);
}
