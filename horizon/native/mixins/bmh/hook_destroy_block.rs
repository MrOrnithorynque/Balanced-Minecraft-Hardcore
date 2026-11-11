// Hook `destroy_block`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_destroy_block(game, pos, state, canceled);
}
