// Hook `inventory_click`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_inventory_click(
    game: &crate::game::Game<'_>,
    player: u128,
    name: &str,
    window: i32,
    slot: i32,
    button: i32,
    mode: i32,
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_inventory_click(game, player, name, window, slot, button, mode, cancel);
}
