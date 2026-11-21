fn bmh_ground_item_tick(game: &Game<'_>) {
    let filter = game.native_on(crate::game::bmh::ID).then_some(bmh_logic::ids::is_eternal as fn(&str) -> bool);
    horizon_client_lib::bmh_ground_items::set_filter(filter);
}
