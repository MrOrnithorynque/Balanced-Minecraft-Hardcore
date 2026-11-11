// Hook `loot_drops`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_respawn::on_loot_drops(game, context, drops);
    crate::game::bmh_tools::on_loot_drops(game, context, drops);
    crate::game::bmh_eternal::on_loot_drops(game, context, drops);
}
