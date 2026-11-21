fn bmh_ground_item_command(game: &Game<'_>, id: u32) -> core::ops::ControlFlow<bool> {
    if game.native_on(crate::game::bmh::ID)
        && game.dropped.iter().any(|drop| drop.id == id && bmh_logic::ids::is_eternal(&drop.stack.item))
    {
        return core::ops::ControlFlow::Break(false);
    }
    core::ops::ControlFlow::Continue(())
}
