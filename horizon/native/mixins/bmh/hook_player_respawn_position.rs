// Hook `player_respawn_position`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_respawn_position(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &mut String,
    pos: &mut [f64; 3],
    yaw: &mut f32,
    pitch: &mut f32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_player_respawn_position(
        game, player, name, dimension, pos, yaw, pitch,
    );
}
