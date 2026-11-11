// Hook `tooltip`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_tooltip(game, stack, lines);
    crate::game::bmh_tools::on_tooltip(game, stack, lines);
    crate::game::bmh_amulet::on_tooltip(game, stack, lines);
}
