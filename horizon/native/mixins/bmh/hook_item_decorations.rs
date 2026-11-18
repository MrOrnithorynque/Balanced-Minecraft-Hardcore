// Hook `item_decorations`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_decorations(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    x: f32,
    y: f32,
    batch: &mut horizon_client::hud::HudBatch,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_amulet::on_item_decorations(game, stack, x, y, batch);
}
