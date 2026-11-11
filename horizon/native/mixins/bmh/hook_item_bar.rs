// Hook `item_bar`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_bar(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_item_bar(game, stack, bar);
    crate::game::bmh_amulet::on_item_bar(game, stack, bar);
}
