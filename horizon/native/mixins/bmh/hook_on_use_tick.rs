// Hook `on_use_tick`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_on_use_tick(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    remaining: i32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_ritual::on_on_use_tick(game, stack, entity, remaining);
}
