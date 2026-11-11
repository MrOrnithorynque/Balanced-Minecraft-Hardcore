// Area totem: the edge between the game and `bmh_logic::totem`.
// It will hold: the save from death, its effects, the Dormant Totem, one Totem of Eternity per
// inventory and per container, the Totem of Undying without Fire Resistance, the anvil.
// Owner: the totem agent. The hook helpers (hook_*.rs) call the functions below.

// T2: a Totem of Eternity in the inventory cancels the death and records the entity for the swap.
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

// T2: swap the saving totem for a Dormant Totem. T11, T12: drop the extra Totems of Eternity.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let _ = game;
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// T3: give the effects of a save.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}

// T11: refuse a second Totem of Eternity.
pub(super) fn on_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    let _ = (game, player, item, count, canceled);
}

// T1: the tooltip lines of the totems.
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    let _ = (game, stack, lines);
}

// T13: refuse a second Totem of Eternity into a container (hoppers).
pub(super) fn on_inventory_move_item(
    game: &crate::game::Game<'_>,
    source: [i32; 3],
    destination: [i32; 3],
    item: &str,
    cancel: &mut bool,
) {
    let _ = (game, source, destination, item, cancel);
}

// T13: refuse a second Totem of Eternity that a hopper picks up from the ground.
pub(super) fn on_hopper_pickup_item(
    dimension: crate::game::LevelDimension,
    pos: horizon_core::position::BlockPos,
    item: &str,
    count: u32,
    cancel: &mut bool,
) {
    let _ = (dimension, pos, item, count, cancel);
}

// T12: note the container to scan for a second Totem of Eternity.
pub(super) fn on_player_container_open(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    let _ = (game, player, name, dimension, menu, pos);
}

// T12: scan the closed container and take out a second Totem of Eternity.
pub(super) fn on_player_container_close(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    dimension: &str,
    menu: &str,
    pos: Option<[i32; 3]>,
) {
    let _ = (game, player, name, dimension, menu, pos);
}

// T6: awaken a Hollow Totem with an Echo Heart for 39 levels. E6: the gold block repair for 15
// levels.
pub(super) fn on_anvil_update(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    left: &horizon_client::items::ItemStack,
    right: Option<&horizon_client::items::ItemStack>,
    item_name: Option<&str>,
    vanilla_output: Option<&horizon_client::items::ItemStack>,
    vanilla_xp_cost: i32,
    vanilla_material_cost: i32,
    output: &mut Option<horizon_client::items::ItemStack>,
    xp_cost: &mut i32,
    material_cost: &mut i32,
    canceled: &mut bool,
) {
    let _ = (
        game,
        player,
        name,
        left,
        right,
        item_name,
        vanilla_output,
        vanilla_xp_cost,
        vanilla_material_cost,
        output,
        xp_cost,
        material_cost,
        canceled,
    );
}
