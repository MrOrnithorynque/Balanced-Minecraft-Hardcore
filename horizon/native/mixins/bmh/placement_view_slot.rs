fn bmh_view_place(view: &View<'_>, slot: usize, stack: &ItemStack) -> std::ops::ControlFlow<bool> {
    if crate::menu::bmh_placement::view_refuses(view, slot, stack) {
        return std::ops::ControlFlow::Break(false);
    }
    std::ops::ControlFlow::Continue(())
}
