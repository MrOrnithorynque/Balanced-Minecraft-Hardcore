fn bmh_config_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    shortcut: Option<EditShortcut>,
    text: Option<&str>,
) -> std::ops::ControlFlow<Result<()>> {
    if bmh_config_editor::key(game, key, shortcut, text) {
        return std::ops::ControlFlow::Break(Ok(()));
    }
    std::ops::ControlFlow::Continue(())
}
