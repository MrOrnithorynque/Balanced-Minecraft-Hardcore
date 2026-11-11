// Hook `register_configs`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_register_configs(
    game: &mut crate::game::Game<'_>,
    configs: &mut Vec<crate::game::native_configs::NativeConfig>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_register_configs(game, configs);
}
