// Hook `anvil_update`: calls the areas in the order of docs/horizon/HOOKS.md.
fn bmh_hook_anvil_update(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    left: &horizon_client::items::ItemStack,
    right: Option<&horizon_client::items::ItemStack>,
    item_name: Option<&str>,
    vanilla_output: Option<&horizon_client::items::ItemStack>,
    vanilla_xp_cost: i32,
    vanilla_material_cost: i32,
    output: &mut Option<horizon_client::items::ItemStack>,
    xp_cost: &mut i32,
    material_cost: &mut i32,
    canceled: &mut bool,
) {
    if !game.native_on(crate::game::bmh::ID) {
        return;
    }
    crate::game::bmh_totem::on_anvil_update(
        game,
        player,
        name,
        left,
        right,
        item_name,
        vanilla_output,
        vanilla_xp_cost,
        vanilla_material_cost,
        output,
        xp_cost,
        material_cost,
        canceled,
    );
}
