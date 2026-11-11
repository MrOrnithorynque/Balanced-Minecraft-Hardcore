// Area eternal: the edge between the game and `bmh_logic::eternal`.
// It will hold: Eternal Items kept after death, one of each type, items on the ground, the Eternal
// Bundle, sleep bonus, hidden coordinates.
// Owner: the eternal agent. The hook helpers (hook_*.rs) call the functions below.

// C11: take the Eternal Items out of the drops and keep them for the respawn.
pub(super) fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    let _ = (game, context, drops);
}

// C11: give back the Eternal Items kept at death.
pub(super) fn on_player_respawned(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// C14: force `reduced_debug_info` in survival.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}

// C13: the wake-up rewards.
pub(super) fn on_player_bed_leave(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    pos: [i32; 3],
) {
    let _ = (game, player, name, pos);
}

// E11: drop a second Eternal Item of a type. E12: move a refused stack out of the Eternal Bundle.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = (game, slot, selected);
}

// E11: refuse a second Eternal Item of a type.
pub(super) fn on_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    let _ = (game, player, item, count, canceled);
}

// E10: an Eternal Item on the ground never despawns.
pub(super) fn on_item_expire(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    stack: &horizon_client::items::ItemStack,
    extra_life: &mut i32,
) {
    let _ = (game, entity, stack, extra_life);
}

// E10: give an Eternal Item on the ground its health and fire resistance.
pub(super) fn on_item_spawn(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    item: &str,
    count: i32,
    pos: [f64; 3],
) {
    let _ = (game, entity, item, count, pos);
}

// E12: the Eternal Bundle drops all its items at once.
pub(super) fn on_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    let _ = (game, stack, claimed);
}
