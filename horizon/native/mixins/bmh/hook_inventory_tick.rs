// Hook `inventory_tick`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_inventory_tick(game, slot, selected);
    crate::game::bmh_tools::on_inventory_tick(game, slot, selected);
    crate::game::bmh_amulet::on_inventory_tick(game, slot, selected);
}
