// Area sky, the part that lives in horizon-client-render: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_client_render::bmh_sky_render`.
// Owner: the sky agent. The hook helpers (hook_*.rs) call the functions below.

// V1: draw the star reading in the sky with the world depth.
pub(crate) fn on_world_drawn(world: &mut crate::gfx::native_render::WorldDrawn<'_>) {
    let _ = world;
}

// V1: ask for the world depth while a reading shows.
pub(crate) fn on_render_needs(needs: &mut crate::gfx::native_render::RenderNeeds) {
    let _ = needs;
}
