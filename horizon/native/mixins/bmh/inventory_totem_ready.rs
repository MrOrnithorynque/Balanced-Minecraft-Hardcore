fn bmh_ready(value: (), game: &mut crate::game::Game<'_>) {
    let _ = value;
    bmh_player_protection::update_ready(game);
}
