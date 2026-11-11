// Hook `player_bed_leave`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_bed_leave(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    pos: [i32; 3],
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_player_bed_leave(game, player, name, pos);
}
