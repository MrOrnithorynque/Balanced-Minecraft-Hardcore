// Hook `item_expire`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_expire(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    stack: &horizon_client::items::ItemStack,
    extra_life: &mut i32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_item_expire(game, entity, stack, extra_life);
}
