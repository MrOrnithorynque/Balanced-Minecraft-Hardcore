// Hook `music`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_music(
    game: &crate::game::Game<'_>,
    music: &mut Option<horizon_client::music::Music>,
    volume: &mut f32,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_journey::on_music(game, music, volume);
}
