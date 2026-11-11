// Hook `player_spawn_point_set`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_spawn_point_set(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    pos: [i32; 3],
    cause: &str,
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_spawn_point_set(
        game, player, name, dimension, pos, cause, cancel,
    );
}
