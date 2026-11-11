// Area tools: the edge between the game and `bmh_logic::tools`.
// It will hold: wear rules, worn out state, 7 percent death wear, Echo Strike, Axe Smash, Echo
// Sense, tooltips.
// Owner: the tools agent. The hook helpers (hook_*.rs) call the functions below.

// C12: take the 7 percent death wear from each Eternal tool in the drops.
pub(super) fn on_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    let _ = (game, context, drops);
}

// E8: track the fall of a player who holds an Eternal Axe.
pub(super) fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let _ = game;
}

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// E7: run the Echo Strike queue.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}

// E3: halve the hit of a worn out tool. E7: queue an Echo Strike. E8: multiply the hit of an Axe
// Smash.
pub(super) fn on_living_incoming_damage_v2(
    game: &mut crate::game::Game<'_>,
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    source_position: Option<[f64; 3]>,
    victim_position: [f64; 3],
    victim_view: [f64; 3],
    weapon: Option<&horizon_client::items::ItemStack>,
    amount: &mut f32,
    canceled: &mut bool,
) {
    let _ = (
        game,
        entity,
        kind,
        damage_type,
        causing,
        source_position,
        victim_position,
        victim_view,
        weapon,
        amount,
        canceled,
    );
}

// E9: read the ore positions to glow on `bmh:tools`.
pub(super) fn on_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    let _ = (game, channel, data);
}

// E2: keep the last point of durability. E4: the stone speed patch of a worn out tool.
pub(super) fn on_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32, selected: bool) {
    let _ = (game, slot, selected);
}

// E2: cap the damage so an Eternal tool keeps its last point.
pub(super) fn on_item_damaged(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    item: &str,
    amount: &mut i32,
    cancel: &mut bool,
) {
    let _ = (game, player, name, item, amount, cancel);
}

// E7: start an Echo Strike from a hit of the Eternal Sword.
pub(super) fn on_hurt_enemy(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: u32,
    kind: &str,
) {
    let _ = (game, stack, entity, kind);
}

// E7: estimate the attack strength of the next hit.
pub(super) fn on_player_attack_cooldown_reset(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
) {
    let _ = (game, player, name);
}

// E7: keep the list of mobs that an echo can hit.
pub(super) fn on_entity_join_level(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    entity: u32,
    kind: &str,
    pos: [f64; 3],
    loaded_from_disk: bool,
    canceled: &mut bool,
) {
    let _ = (
        game,
        dimension,
        entity,
        kind,
        pos,
        loaded_from_disk,
        canceled,
    );
}

// E9: Echo Sense finds the ores near a block that an Eternal Pickaxe breaks.
pub(super) fn on_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    state: horizon_world::registry::BlockStateId,
    canceled: &mut bool,
) {
    let _ = (game, pos, state, canceled);
}

// E5: the ability, worn out, heavy and death wear lines.
pub(super) fn on_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    let _ = (game, stack, lines);
}

// E5: the 1 pixel durability bar of an Eternal tool.
pub(super) fn on_item_bar(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    bar: &mut Option<(i32, u32)>,
) {
    let _ = (game, stack, bar);
}

// E9: draw the boxes of the glowing ores through walls.
pub(super) fn on_world_primitives(
    game: &crate::game::Game<'_>,
    partial: f32,
    camera: [f64; 3],
    primitives: &mut horizon_client::world_primitives::WorldPrimitives,
) {
    let _ = (game, partial, camera, primitives);
}
