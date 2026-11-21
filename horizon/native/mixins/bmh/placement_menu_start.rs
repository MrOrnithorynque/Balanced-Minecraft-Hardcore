fn bmh_menu_start(
    menu: &Menu,
    slot: i32,
    button: i32,
    input: Input,
) -> std::ops::ControlFlow<Result<(), &'static str>> {
    if bmh_placement::menu_start(menu, slot, button, input) {
        return std::ops::ControlFlow::Break(Ok(()));
    }
    std::ops::ControlFlow::Continue(())
}
