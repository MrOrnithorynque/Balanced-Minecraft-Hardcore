fn bmh_hand_effects(
    mut effects: Vec<horizon_client::items::ConsumeEffect>,
    game: &crate::game::Game<'_>,
) -> Vec<horizon_client::items::ConsumeEffect> {
    if game.native_on(crate::game::bmh::ID) {
        effects.push(horizon_client::items::ConsumeEffect::RemoveEffects(vec![
            bmh_logic::totem::UNDYING_LOSES.to_owned(),
        ]));
    }
    effects
}
