// Hook `living_incoming_damage`: calls the areas in the order of docs/horizon/HOOKS.md.
// This hook has no `Game`, so it cannot check `native_on`. Each area must do
// nothing when the mod is off (docs/horizon/HOOKS.md, "Hooks with no Game").
fn bmh_hook_living_incoming_damage(
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    direct: i32,
    amount: &mut f32,
    canceled: &mut bool,
) {
    crate::game::bmh_respawn::on_living_incoming_damage(
        entity,
        kind,
        damage_type,
        causing,
        direct,
        amount,
        canceled,
    );
    crate::game::bmh_tools::on_living_incoming_damage(entity, damage_type, amount, *canceled);
    // After the respawn area, which may cancel the hit: a canceled hit breaks no ritual.
    crate::game::bmh_ritual::on_living_incoming_damage(entity, kind, *amount, *canceled);
}
