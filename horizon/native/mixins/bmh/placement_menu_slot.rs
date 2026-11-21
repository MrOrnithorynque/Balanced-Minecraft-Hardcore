fn bmh_menu_place(menu: &Menu, slot: usize, stack: &ItemStack) -> std::ops::ControlFlow<bool> {
    if crate::menu::bmh_placement::menu_refuses(menu, slot, stack) {
        return std::ops::ControlFlow::Break(false);
    }
    std::ops::ControlFlow::Continue(())
}
