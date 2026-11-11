// Hook `spawn_change`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_spawn_change(game: &mut crate::game::Game<'_>, pos: [i32; 3], angle: [f32; 2]) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_spawn_change(game, pos, angle);
    crate::game::bmh_amulet::on_spawn_change(game, pos, angle);
}
