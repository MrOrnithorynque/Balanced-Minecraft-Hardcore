// Hook `world_lifecycle`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_world_lifecycle(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::WorldPhase,
    error_text: &str,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_world_lifecycle(game, context, phase, error_text);
}
