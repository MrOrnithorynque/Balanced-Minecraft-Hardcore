// Area respawn: the edge between the game and `bmh_logic::respawn`.
// It will hold: distance at death, landing search, position inside the respawn, exile record, 200
// ticks immunity, arrival sounds, journey signals (server side), journey advancements.
// Owner: the respawn agent. The hook helpers (hook_*.rs) call the functions below.

// C2: move the respawn to the distant point that the landing search chose (the vanilla point when
// no landing is ready).
pub(super) fn on_player_respawn_position(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &mut String,
    pos: &mut [f64; 3],
    yaw: &mut f32,
    pitch: &mut f32,
) {
    let _ = (game, player, name, dimension, pos, yaw, pitch);
}

// C8: send the departure signal to the dying player. The helper calls this only when no area
// cancelled the death.
pub(super) fn on_living_death(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    kind: &str,
    damage_type: &str,
    attacker: u32,
    cancel: &mut bool,
) {
    let _ = (game, entity, kind, damage_type, attacker, cancel);
}

// C1: count the Eternal Items in the dying player's drops and store the respawn distance.
pub(super) fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    let _ = (game, context, drops);
}

// C9: count the death for the tenth death advancement (until `player_stat` is stamped).
pub(super) fn on_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    let _ = (game, player, cause);
}

// C1: cache the Eternal Item count for `keep_inventory`. C6: count down the damage immunity.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let _ = game;
}

// C2, C6, C7, C8: read the exile record, start the immunity, queue the arrival sounds, send the
// arrival signal. C9, C10: give back the exile of this respawn, once (Java `takeExile`). Give None
// when the respawn was not a distant one or the world is hardcore. The hook helper hands the exile
// to `bmh_homecoming::on_exile`.
pub(super) fn on_player_respawned(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) -> Option<bmh_logic::contract::Exile> {
    let _ = (game, player, name);
    None
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// C3: drive the landing search. C7: the bell and the sound for the players nearby. C9: the journey
// advancements.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}

// C6: cancel the damage during the 200 ticks of immunity after a distant respawn.
pub(super) fn on_living_incoming_damage(
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    direct: i32,
    amount: &mut f32,
    canceled: &mut bool,
) {
    let _ = (entity, kind, damage_type, causing, direct, amount, canceled);
}

// C1: note the difficulty for the distance formula.
pub(super) fn on_difficulty_changed(game: &mut crate::game::Game<'_>, from: &str, to: &str) {
    let _ = (game, from, to);
}

// C4, D7: check a respawn request that the client held, on `bmh:respawn`.
pub(super) fn on_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    let _ = (game, player, channel, data);
}
