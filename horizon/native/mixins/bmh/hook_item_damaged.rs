// Hook `item_damaged`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_damaged(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    item: &str,
    amount: &mut i32,
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_item_damaged(game, player, name, item, amount, cancel);
}
