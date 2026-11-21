fn bmh_drag_capacity(
    view: &View<'_>,
    slot: usize,
    stack: &ItemStack,
) -> std::ops::ControlFlow<u32> {
    if crate::menu::bmh_placement::drag_refuses(view, slot, stack) {
        return std::ops::ControlFlow::Break(0);
    }
    std::ops::ControlFlow::Continue(())
}
