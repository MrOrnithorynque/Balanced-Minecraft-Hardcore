// Hook `inventory_drag`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_inventory_drag(
    game: &crate::game::Game<'_>,
    player: u128,
    name: &str,
    window: i32,
    kind: i32,
    slots: &[i32],
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_inventory_drag(game, player, name, window, kind, slots, cancel);
}
