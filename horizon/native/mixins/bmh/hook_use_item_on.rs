// Hook `use_item_on`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_amulet::on_use_item_on(game, stack, pos, face, claimed);
}
