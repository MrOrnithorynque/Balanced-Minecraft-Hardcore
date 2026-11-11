// Hook `living_incoming_damage_v2`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_living_incoming_damage_v2(
    game: &mut crate::game::Game<'_>,
    entity: i32,
    kind: &str,
    damage_type: &str,
    causing: i32,
    source_position: Option<[f64; 3]>,
    victim_position: [f64; 3],
    victim_view: [f64; 3],
    weapon: Option<&horizon_client::items::ItemStack>,
    amount: &mut f32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_living_incoming_damage_v2(
        game,
        entity,
        kind,
        damage_type,
        causing,
        source_position,
        victim_position,
        victim_view,
        weapon,
        amount,
        canceled,
    );
}
