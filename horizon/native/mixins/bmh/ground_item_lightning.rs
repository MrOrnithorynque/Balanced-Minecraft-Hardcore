fn bmh_ground_item_lightning(game: &Game<'_>, h: EntityHandle) -> core::ops::ControlFlow<bool> {
    if let EntityHandle::Item(index) = h
        && game.native_on(crate::game::bmh::ID)
        && bmh_logic::ids::is_eternal(&game.dropped[index].stack.item)
    {
        return core::ops::ControlFlow::Break(false);
    }
    core::ops::ControlFlow::Continue(())
}
