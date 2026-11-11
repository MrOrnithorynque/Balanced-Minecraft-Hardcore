// Hook `player_container_open`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_container_open(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_player_container_open(game, player, name, dimension, menu, pos);
}
