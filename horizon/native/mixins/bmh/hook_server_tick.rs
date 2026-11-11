// Hook `server_tick`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_server_tick(game, context, phase);
    crate::game::bmh_homecoming::on_server_tick(game, context, phase);
    crate::game::bmh_eternal::on_server_tick(game, context, phase);
    crate::game::bmh_totem::on_server_tick(game, context, phase);
    crate::game::bmh_ritual::on_server_tick(game, context, phase);
    crate::game::bmh_tools::on_server_tick(game, context, phase);
}
