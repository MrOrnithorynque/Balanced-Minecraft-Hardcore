// Hook `hopper_pickup_item`: calls the areas in the order of docs/horizon/HOOKS.md.
// This hook has no `Game`, so it cannot check `native_on`. Each area must do
// nothing when the mod is off (docs/horizon/HOOKS.md, "Hooks with no Game").
// It binds the level, which holds the block entities, so the totem area can read the hopper.
fn bmh_hook_hopper_pickup_item(
    level: &crate::game::GameLevel<'_>,
    dimension: crate::game::LevelDimension,
    pos: horizon_core::position::BlockPos,
    item: &str,
    count: u32,
    cancel: &mut bool,
) {
    crate::game::bmh_totem::on_hopper_pickup_item(level, dimension, pos, item, count, cancel);
}
