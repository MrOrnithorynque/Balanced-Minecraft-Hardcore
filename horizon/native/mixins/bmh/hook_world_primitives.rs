// Hook `world_primitives`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_world_primitives(
    game: &crate::game::Game<'_>,
    partial: f32,
    camera: [f64; 3],
    primitives: &mut horizon_client::world_primitives::WorldPrimitives,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_world_primitives(game, partial, camera, primitives);
}
