// Hook `inventory_move_item`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_inventory_move_item(
    game: &crate::game::Game<'_>,
    source: [i32; 3],
    destination: [i32; 3],
    item: &str,
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_inventory_move_item(game, source, destination, item, cancel);
}
