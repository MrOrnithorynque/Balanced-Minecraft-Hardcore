// Hook `living_entity_use_item_stop`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_living_entity_use_item_stop(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    hand: horizon_client::first_person::Hand,
    duration: i32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_ritual::on_living_entity_use_item_stop(
        game, stack, entity, hand, duration, canceled,
    );
}
