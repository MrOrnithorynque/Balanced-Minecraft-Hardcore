// Area ritual: the edge between the game and `bmh_logic::moon`.
// It will hold: the moon ritual on the server and its look on the client.
// Owner: the ritual agent. The hook helpers (hook_*.rs) call the functions below.

// State: remove this player from the area's per-player statics.
pub(super) fn on_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    let _ = (game, player, name);
}

// T9: send the ritual runes to the viewers.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = (game, context, phase);
}

// T7, T15: the use time of the Dormant Totem, so that a click starts a use and consumes the click.
pub(super) fn on_use_duration(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    duration: &mut i32,
) {
    let _ = (game, stack, duration);
}

// T7: check the start rules of the ritual.
pub(super) fn on_living_entity_use_item_start(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: &mut i32,
    canceled: &mut bool,
) {
    let _ = (game, stack, entity, hand, duration, canceled);
}

// T7: keep the ritual going, or stop it on a fail.
pub(super) fn on_living_entity_use_item_tick(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: &mut i32,
    canceled: &mut bool,
) {
    let _ = (game, stack, entity, hand, duration, canceled);
}

// T7: a ritual that the player lets go fails.
pub(super) fn on_living_entity_use_item_stop(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: i32,
    canceled: &mut bool,
) {
    let _ = (game, stack, entity, hand, duration, canceled);
}

// T7: a finished ritual wakes the totem.
pub(super) fn on_living_entity_use_item_finish(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: i32,
    result: &mut Option<horizon_client::items::ItemStack>,
) {
    let _ = (game, stack, entity, hand, duration, result);
}

// T7: drain the levels each tick of the ritual.
pub(super) fn on_on_use_tick(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    remaining: i32,
) {
    let _ = (game, stack, entity, remaining);
}

// T7: the release of the ritual item.
pub(super) fn on_release_using(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    time_left: i32,
    used: &mut bool,
) {
    let _ = (game, stack, entity, time_left, used);
}

// T8: raise the totem toward the moon and shake it in first person.
pub(super) fn on_render_hand_pose(
    game: &crate::game::Game<'_>,
    hand: horizon_client::first_person::Hand,
    stack: Option<&horizon_client::items::ItemStack>,
    partial: f32,
    swing: f32,
    equip: f32,
    x: &mut f32,
    y: &mut f32,
    z: &mut f32,
    pitch: &mut f32,
    yaw: &mut f32,
    roll: &mut f32,
    scale: &mut f32,
) {
    let _ = (
        game, hand, stack, partial, swing, equip, x, y, z, pitch, yaw, roll, scale,
    );
}
