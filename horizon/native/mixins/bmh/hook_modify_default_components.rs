// Hook `modify_default_components`: calls the areas in the order of docs/horizon/HOOKS.md.
// This hook has no `Game`, so it cannot check `native_on`. Each area must do
// nothing when the mod is off (docs/horizon/HOOKS.md, "Hooks with no Game").
fn bmh_hook_modify_default_components(
    item: &str,
    components: &mut serde_json::Map<String, serde_json::Value>,
) {
    crate::bmh_totem_world::on_modify_default_components(item, components);
    crate::bmh_amulet_world::on_modify_default_components(item, components);
}
