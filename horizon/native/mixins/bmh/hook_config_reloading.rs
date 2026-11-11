// Hook `config_reloading`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_config_reloading(
    game: &mut crate::game::Game<'_>,
    config: &crate::game::native_configs::NativeConfigFile,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_settings::on_config_reloading(game, config);
}
