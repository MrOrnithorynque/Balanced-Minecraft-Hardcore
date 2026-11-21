fn bmh_config_action(
    game: &mut crate::game::Game<'_>,
    click: &Option<Click>,
) -> std::ops::ControlFlow<Result<()>> {
    if bmh_config_editor::action(game, click.as_ref()) {
        return std::ops::ControlFlow::Break(Ok(()));
    }
    std::ops::ControlFlow::Continue(())
}
