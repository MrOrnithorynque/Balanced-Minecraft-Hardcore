// Hook `living_fall`: calls the areas in the order of docs/horizon/HOOKS.md.
// This hook has no `Game`, so it cannot check `native_on`. Each area must do
// nothing when the mod is off (docs/horizon/HOOKS.md, "Hooks with no Game").
fn bmh_hook_living_fall(
    entity: u32,
    kind: &str,
    distance: &mut f64,
    multiplier: &mut f32,
    cancel: &mut bool,
) {
    crate::bmh_tools_lib::on_living_fall(entity, kind, distance, multiplier, cancel);
}
