// Hook `world_drawn`: calls the areas in the order of docs/horizon/HOOKS.md.
// This hook has no `Game`, so it cannot check `native_on`. Each area must do
// nothing when the mod is off (docs/horizon/HOOKS.md, "Hooks with no Game").
fn bmh_hook_world_drawn(world: &mut crate::gfx::native_render::WorldDrawn<'_>) {
    crate::bmh_sky_render::on_world_drawn(world);
}
