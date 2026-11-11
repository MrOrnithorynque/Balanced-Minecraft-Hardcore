// Hook `hurt_enemy`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_hurt_enemy(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    entity: u32,
    kind: &str,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_tools::on_hurt_enemy(game, stack, entity, kind);
}
