// Area amulet, the part that lives in horizon-world: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_world::bmh_amulet_world`.
// Owner: the amulet agent. The hook helpers (hook_*.rs) call the functions below.

// A1: a default `lodestone_tracker` target for the amulet (fallback of the content item).
pub(crate) fn on_modify_default_components(
    item: &str,
    components: &mut serde_json::Map<String, serde_json::Value>,
) {
    let _ = (item, components);
}
