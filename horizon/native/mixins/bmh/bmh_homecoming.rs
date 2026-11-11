// Area homecoming: the edge between the game and `bmh_logic::homecoming`.
// It will hold: the record on the player, the check once a second, title, bell, two advancements.
// Owner: the homecoming agent. The hook helpers (hook_*.rs) call the functions below.

// C10: save a new homecoming record on the player after a distant respawn. The hook helper of
// `player_respawned` calls this with the exile that `bmh_respawn::on_player_respawned` gave. The
// homes are the exile center and, for an Overworld exile, the amulet points of this moment
// (`bmh_amulet::points`). Java: `HomecomingHandler.onExile`.
pub(super) fn on_exile(
    game: &mut crate::game::Game<'_>,
    player: u128,
    exile: &bmh_logic::contract::Exile,
) {
    let points = crate::game::bmh_amulet::points(game, player);
    let _ = (game, exile, points);
}

// C10: check each exile once a second.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}
