// Area tools, the part that lives in horizon-client-lib: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_client_lib::bmh_tools_lib`.
// Owner: the tools agent. The hook helpers (hook_*.rs) call the functions below.

// E8: halve the landing after an Axe Smash.
pub(crate) fn on_living_fall(
    entity: u32,
    kind: &str,
    distance: &mut f64,
    multiplier: &mut f32,
    cancel: &mut bool,
) {
    let _ = (entity, kind, distance, multiplier, cancel);
}
