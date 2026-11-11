// Hook `release_using`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_release_using(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: Option<i32>,
    time_left: i32,
    used: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_ritual::on_release_using(game, stack, entity, time_left, used);
}
