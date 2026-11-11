// Hook `use_item`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_use_item(game, stack, claimed);
    crate::game::bmh_amulet::on_use_item(game, stack, claimed);
}
