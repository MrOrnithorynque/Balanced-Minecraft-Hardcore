// Hook `item_spawn`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_item_spawn(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    item: &str,
    count: i32,
    pos: [f64; 3],
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_item_spawn(game, entity, item, count, pos);
}
