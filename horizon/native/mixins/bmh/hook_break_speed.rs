// Hook `break_speed`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_break_speed(game: &crate::game::Game<'_>, pos: [i32; 3], speed: &mut f32) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_break_speed(game, pos, speed);
}
