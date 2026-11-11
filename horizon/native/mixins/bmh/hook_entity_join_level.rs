// Hook `entity_join_level`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_entity_join_level(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    entity: u32,
    kind: &str,
    pos: [f64; 3],
    loaded_from_disk: bool,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_entity_join_level(
        game,
        dimension,
        entity,
        kind,
        pos,
        loaded_from_disk,
        canceled,
    );
}
