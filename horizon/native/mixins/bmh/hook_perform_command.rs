// Hook `perform_command`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_perform_command(game, player, command, ran);
    crate::game::bmh_amulet::on_perform_command(game, player, command, ran);
}
