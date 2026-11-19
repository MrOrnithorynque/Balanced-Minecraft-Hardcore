// The player tick checks the container after Horizon applies this player's click.
fn bmh_inventory_click(game: &crate::game::Game<'_>, player: u128, window: i32) {
    if game.native_on(crate::game::bmh::ID) {
        crate::game::bmh_totem::note_menu_click(player, window);
    }
}
