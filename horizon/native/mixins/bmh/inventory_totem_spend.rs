fn bmh_spend(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<Option<Body>> {
    bmh_player_protection::spend(game)
}
