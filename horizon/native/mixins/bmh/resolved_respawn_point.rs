// The server has consumed a valid anchor or handled the player's missing point.
fn bmh_resolved_respawn_point(outcome: Option<(Vec3, f32)>, game: &mut Game<'_>, config: RespawnConfig) -> Option<(Vec3, f32)> {
    if game.native_on(crate::game::bmh::ID) {
        crate::game::bmh_respawn::on_vanilla_point(game, config, outcome.is_some());
    }
    outcome
}
