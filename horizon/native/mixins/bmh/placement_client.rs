fn bmh_placement_gate(game: &crate::game::Game<'_>) {
    let enabled = game.native_on(crate::game::bmh::ID);
    horizon_client::menu::bmh_placement::set_enabled(enabled);
    horizon_client::bmh_bundle_click::set_rule(
        enabled.then_some(bmh_logic::eternal::bundle_refuses),
    );
}
