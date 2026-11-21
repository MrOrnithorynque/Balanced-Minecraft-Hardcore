fn bmh_config_text(game: &crate::game::Game<'_>, text: &str) -> std::ops::ControlFlow<bool> {
    if bmh_config_editor::is_open(game) && !text.chars().all(|c| c.is_ascii_digit()) {
        return std::ops::ControlFlow::Break(true);
    }
    std::ops::ControlFlow::Continue(())
}
