// Hook `use_duration`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_use_duration(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    duration: &mut i32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_ritual::on_use_duration(game, stack, duration);
}
