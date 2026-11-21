fn bmh_before_inventory_drops(game: &mut crate::game::Game<'_>, who: (u128, i32)) {
    if game.native_on(crate::game::bmh::ID) {
        crate::game::bmh_eternal::before_inventory_drops(game, who.0);
    }
}
