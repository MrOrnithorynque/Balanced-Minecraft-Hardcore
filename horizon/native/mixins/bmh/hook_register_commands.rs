// Hook `register_commands`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_register_commands(game, commands);
    crate::game::bmh_amulet::on_register_commands(game, commands);
}
