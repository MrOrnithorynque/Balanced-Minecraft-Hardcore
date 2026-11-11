// Hook `player_respawned`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_player_respawned(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_eternal::on_player_respawned(game, player, name);
    let exile = crate::game::bmh_respawn::on_player_respawned(game, player, name);
    if let Some(exile) = exile {
        crate::game::bmh_homecoming::on_exile(game, player, &exile);
    }
}
