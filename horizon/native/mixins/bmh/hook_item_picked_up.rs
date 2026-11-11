// Hook `item_picked_up`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_item_picked_up(game, player, item, count, canceled);
    crate::game::bmh_eternal::on_item_picked_up(game, player, item, count, canceled);
}
