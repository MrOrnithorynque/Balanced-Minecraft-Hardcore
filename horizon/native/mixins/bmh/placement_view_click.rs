fn bmh_view_click(
    view: &View<'_>,
    drag: &mut Option<(u8, Vec<usize>)>,
    slot: i32,
    button: i32,
    mode: i32,
) -> std::ops::ControlFlow<(Vec<ItemStack>, Vec<usize>)> {
    if crate::menu::bmh_placement::view_start(view, drag, slot, button, mode) {
        return std::ops::ControlFlow::Break((Vec::new(), Vec::new()));
    }
    std::ops::ControlFlow::Continue(())
}
