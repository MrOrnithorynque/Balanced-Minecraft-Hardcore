// Hook `living_death`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_living_death(
    game: &mut crate::game::Game<'_>,
    entity: u32,
    kind: &str,
    damage_type: &str,
    attacker: u32,
    cancel: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_living_death(game, entity, kind, damage_type, attacker, cancel);
    // The journey signal is only for a real death: skip it when an area cancelled.
    if *cancel {
        return;
    }
    crate::game::bmh_respawn::on_living_death(game, entity, kind, damage_type, attacker, cancel);
}
