// Area totem, the part that lives in horizon-world: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_world::bmh_totem_world`.
// Owner: the totem agent. The hook helpers (hook_*.rs) call the functions below.

// T5: remove Fire Resistance from the Totem of Undying.
pub(crate) fn on_modify_default_components(
    item: &str,
    components: &mut serde_json::Map<String, serde_json::Value>,
) {
    let _ = (item, components);
}
