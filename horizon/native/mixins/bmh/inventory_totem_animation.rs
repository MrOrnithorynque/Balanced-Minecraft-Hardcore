fn bmh_animation(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<ItemStack> {
    bmh_player_protection::animation(game)
}
