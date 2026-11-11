// Hook `interaction_key`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_interaction_key(
    game: &mut crate::game::Game<'_>,
    button: i32,
    key_mapping: &str,
    hand: horizon_client::first_person::Hand,
    swing_hand: &mut bool,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_amulet::on_interaction_key(
        game,
        button,
        key_mapping,
        hand,
        swing_hand,
        canceled,
    );
}
